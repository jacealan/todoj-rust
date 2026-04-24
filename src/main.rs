//! todoj - Terminal TODO application
//! 
//! A simple yet powerful terminal-based TODO manager written in Rust.
//! Supports local SQLite storage with future plans for PostgreSQL sync.
//! 
//! # Features
//! 
//! - Add, edit, remove, and complete todos
//! - Due date tracking with flexible input formats
//! - Priority levels (1-4)
//! - Sub-todos (parent-child relationships)
//! - Progress tracking (0%, 20%, 40%, 60%, 80%, 100%)
//! - Order by due date, priority, or creation time
//! - Show/hide completed todos
//! - Local SQLite storage
//! - (Planned) PostgreSQL sync for multi-device
//! 
//! # Usage
//! 
//! ```bash
//! todoj              # Interactive mode
//! todoj -l           # List mode (show and exit)
//! todoj --db /path    # Custom database path
//! ```
//! 
//! # Building
//! 
//! ```bash
//! cargo build --release
//! ```

mod cli;
mod commands;
mod db;
mod formatters;

use clap::Parser;
use cli::{parse_command, parse_list_range, Args};
use db::{SqliteRepo, Todo, TodoRepository};
use formatters::{clear_screen, print_help};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

/// Get default database path
/// 
/// Default location: ~/.todoj.db
fn get_default_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".todoj.db")
}

/// Format todo for display
/// 
/// Creates display string with:
/// - Line number (padded)
/// - Indent for sub-todos
/// - Checkbox [ ] or [x]
/// - Parent reference (5>)
/// - Content
/// - Due date @YY-MM-DD(Wday)
/// - Progress %XX (for incomplete with progress)
/// - Priority ^X
/// - Done date %YY-MM-DD (for completed)
/// 
/// # Arguments
/// * `num` - Display line number
/// * `width` - Width for padding line number
/// * `item` - Todo item
/// * `is_done` - Whether todo is completed (done=5)
/// * `display_ids` - All display IDs for parent lookup
/// * `use_order` - Whether ordered display (sub-todos under parent)
fn format_todo(
    num: usize,
    width: usize,
    item: &Todo,
    is_done: bool,
    display_ids: &[i64],
    use_order: bool,
) -> String {
    let num_str = format!("{}", num);
    let padded_num = format!("{:>width$}", num_str);

    // Indent sub-todos when using order mode
    let indent = if use_order && item.up_id.is_some() {
        "  "
    } else {
        ""
    };
    let check = if is_done { "[x]" } else { "[ ]" };

    // Find parent in display list
    let parent_ref = if let Some(parent_id) = item.up_id {
        display_ids.iter().position(|&x| x == parent_id).map(|p| p + 1)
    } else {
        None
    };
    let parent_str = parent_ref.map(|p| format!("{p}>")).unwrap_or_default();

    // Format due date
    let due = item.due_date.as_ref().and_then(|d| formatters::format_date(d));
    let due_str = due.map(|d| format!(" @{d}")).unwrap_or_default();

    // Show progress percentage for incomplete todos with progress
    let progress = if !is_done && item.done > 0 {
        format!(" {}%", (item.done as usize) * 20)
    } else {
        String::new()
    };

    // Format done date for completed todos
    let done_str = if is_done {
        item.done_at
            .as_ref()
            .and_then(|dt| formatters::format_date(dt))
            .map(|formatted| format!(" %{}", formatted))
            .unwrap_or_default()
    } else {
        String::new()
    };

    format!(
        "{}{} {} {} {}{} {}{} ^{}{}\n",
        padded_num,
        indent,
        check,
        parent_str,
        item.todo,
        due_str,
        progress,
        "",
        item.priority,
        done_str
    )
}

/// List todos with various display options
/// 
/// Handles sorting, filtering, and pagination.
/// 
/// # Sorting (incomplete first, then completed)
/// 
/// Incomplete sorted by:
/// 1. Due date (nulls last)
/// 2. Priority (ascending)
/// 3. Created at (descending)
/// 
/// Completed sorted by:
/// - Done date (descending)
/// 
/// # Arguments
/// * `repo` - Database repository
/// * `show_done` - Include completed todos
/// * `use_order` - Order by parent-child (sub-todos under parent)
/// * `limit` - Max items to show
/// * `page` - Page number (for pagination)
/// * `range` - (start, end) for range display
fn list_todos(
    repo: &Arc<dyn TodoRepository>,
    show_done: bool,
    use_order: bool,
    limit: Option<usize>,
    page: Option<usize>,
    range: (Option<usize>, Option<usize>),
) -> Result<(Vec<Todo>, Vec<i64>), String> {
    // Fetch all todos
    let todos = repo.find_all(show_done)?;

    // Split into incomplete and completed
    let incomplete: Vec<_> = todos.iter().filter(|t| t.done != 5).cloned().collect();
    let completed: Vec<_> = todos.iter().filter(|t| t.done == 5).cloned().collect();

    // Build header
    let mut flags = String::new();
    if use_order {
        flags.push_str(" order");
    }
    if show_done {
        flags.push_str(" show");
    }

    let mut output = String::new();
    output.push_str(&format!(
        "=== TODO{} ===\n",
        if flags.is_empty() {
            String::new()
        } else {
            flags
        }
    ));

    // Empty check
    if incomplete.is_empty() && (!show_done || completed.is_empty()) {
        output.push_str("TODO가 없습니다.\n");
        print!("{}", output);
        return Ok((Vec::new(), Vec::new()));
    }

    // Sort incomplete (respecting order mode)
    let display_incomplete: Vec<_> = if use_order {
        // Separate parents and children
        let mut parents: Vec<_> = incomplete.iter().filter(|t| t.up_id.is_none()).cloned().collect();
        let children: Vec<_> = incomplete.iter().filter(|t| t.up_id.is_some()).cloned().collect();

        // Sort parents by due date, priority, created
        parents.sort_by(|a, b| {
            let a_has_due = a.due_date.is_some();
            let b_has_due = b.due_date.is_some();
            if a_has_due != b_has_due {
                b_has_due.cmp(&a_has_due)  // Due dates first, then nulls
            } else {
                a.due_date
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("")
                    .cmp(b.due_date.as_ref().map(|s| s.as_str()).unwrap_or(""))
                    .then(a.priority.cmp(&b.priority))
                    .then(b.id.cmp(&a.id))
            }
        });

        // Build ordered list: parents first, then their children
        let mut ordered: Vec<Todo> = Vec::new();
        for parent in &parents {
            ordered.push((*parent).clone());
            for child in &children {
                if child.up_id == Some(parent.id) {
                    ordered.push((*child).clone());
                }
            }
        }
        ordered
    } else {
        incomplete
    };

    // Sort completed by done date (descending)
    let mut sorted_completed = completed.clone();
    sorted_completed.sort_by(|a, b| b.done_at.cmp(&a.done_at));

    let incomplete_len = display_incomplete.len();
    let mut display_ids = Vec::new();

    // Calculate which items to show
    let items_to_show = {
        let (rs, re) = range;
        let total_incomplete = display_incomplete.len();
        let total_completed = if show_done {
            sorted_completed.len()
        } else {
            0
        };
        let total = total_incomplete + total_completed;

        if let Some(start) = rs {
            Some((start, re.unwrap_or(total)))
        } else if let (Some(lim), Some(p)) = (limit, page) {
            let start_idx = (p - 1) * lim;
            let end_idx = start_idx + lim;
            Some((start_idx + 1, end_idx.min(total)))
        } else if let Some(lim) = limit {
            Some((1, lim.min(total)))
        } else {
            None
        }
    };

    // Display incomplete
    if !display_incomplete.is_empty() {
        let max_num = display_incomplete.len();
        let width = max_num.to_string().len();

        let filtered: Vec<_> = match items_to_show {
            Some((start, end)) => display_incomplete
                .iter()
                .enumerate()
                .filter(|(idx, _)| {
                    let num = idx + 1;
                    num >= start && num <= end
                })
                .collect(),
            None => display_incomplete.iter().enumerate().collect(),
        };

        // Collect IDs first
        for (_, item) in &filtered {
            display_ids.push(item.id);
        }
        // Then format
        for (idx, item) in &filtered {
            output.push_str(&format_todo(
                idx + 1,
                width,
                item,
                false,
                &display_ids,
                use_order,
            ));
        }
    }

    // Display completed (if enabled)
    if show_done && !sorted_completed.is_empty() {
        if incomplete_len > 0 {
            output.push('\n');
        }
        let start_num = incomplete_len + 1;
        let width = (start_num + sorted_completed.len() - 1).to_string().len();
        for (_, item) in sorted_completed.iter().enumerate() {
            display_ids.push(item.id);
        }
        for (idx, item) in sorted_completed.iter().enumerate() {
            output.push_str(&format_todo(
                start_num + idx,
                width,
                item,
                true,
                &display_ids,
                use_order,
            ));
        }
    }

    print!("{}", output);
    std::io::stdout().flush().ok();

    Ok((todos, display_ids))
}

/// Main entry point
/// 
/// # Flow
/// 
/// 1. Parse CLI arguments
/// 2. Create database repository
/// 3. Initialize database (create tables)
/// 4. Either list and exit (list mode) or enter interactive loop
/// 5. In interactive mode:
///    - Display current todos
///    - Read command
///    - Execute command
///    - Redraw if needed
///    - Repeat until quit
fn main() {
    // Parse CLI arguments
    let cli_args = Args::parse();

    // Create repository (SQLite for now)
    let repo: Arc<dyn TodoRepository> = if let Some(db_path) = cli_args.db {
        Arc::new(SqliteRepo::new(PathBuf::from(db_path)))
    } else {
        // Check for PostgreSQL connection string (future use)
        let db_url = std::env::var("DATABASE_URL").ok();
        if db_url.is_some() {
            eprintln!("PostgreSQL not yet supported, switching to SQLite");
        }
        Arc::new(SqliteRepo::new(get_default_db_path()))
    };

    // Initialize database
    if let Err(e) = repo.init() {
        eprintln!("DB 오류: {}", e);
        return;
    }

    // List mode: show and exit
    if cli_args.list_mode {
        let _ = list_todos(&repo, false, false, None, None, (None, None));
        return;
    }

    // Interactive mode state
    let mut show_done = false;
    let mut use_order = false;

    // Helper: redraw screen and return todos/display_ids
    fn redraw(
        repo: &Arc<dyn TodoRepository>,
        show_done: bool,
        use_order: bool,
    ) -> (Vec<Todo>, Vec<i64>) {
        clear_screen();
        list_todos(repo, show_done, use_order, None, None, (None, None)).unwrap_or_default()
    }

    // Initial display
    let (todos, display_ids) = redraw(&repo, show_done, use_order);
    let mut display_ids = display_ids;
    let mut _all_todos = todos;

    // Interactive loop
    loop {
        print!("\n> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }

        // Parse command
        let (cmd, rest) = match parse_command(&input) {
            Some(c) => c,
            None => continue,
        };
        let mut needs_redraw = false;

        // Dispatch to command handlers
        match cmd.as_str() {
            "add" | "a" => {
                if rest.is_empty() {
                    println!("사용법: add <내용> [-d 날짜] [-p 우선순위] [-u 리스트번호]");
                } else if let Ok(updated) = commands::cmd_add(&repo, &rest, &display_ids) {
                    needs_redraw = updated;
                } else if let Err(e) = commands::cmd_add(&repo, &rest, &display_ids) {
                    println!("{}", e);
                }
            }
            "edit" | "e" => {
                if rest.is_empty() {
                    println!("사용법: edit <리스트번호>[,...] [-d 날짜] [-p 우선순위]");
                } else if let Ok(updated) = commands::cmd_edit(&repo, &rest, &display_ids) {
                    needs_redraw = updated;
                } else if let Err(e) = commands::cmd_edit(&repo, &rest, &display_ids) {
                    println!("{}", e);
                }
            }
            "remove" | "r" => {
                if rest.is_empty() {
                    println!("사용법: remove <리스트번호>[,...]");
                } else if let Ok(updated) = commands::cmd_remove(&repo, &rest, &display_ids) {
                    needs_redraw = updated;
                } else if let Err(e) = commands::cmd_remove(&repo, &rest, &display_ids) {
                    println!("{}", e);
                }
            }
            "done" | "d" => {
                if rest.is_empty() {
                    println!("사용법: done <리스트번호>[,...] [0-5]");
                } else if let Ok(updated) = commands::cmd_done(&repo, &rest, &display_ids) {
                    needs_redraw = updated;
                } else if let Err(e) = commands::cmd_done(&repo, &rest, &display_ids) {
                    println!("{}", e);
                }
            }
            "list" | "l" => {
                let (limit, page, range_start, range_end) = parse_list_range(&rest);
                match list_todos(
                    &repo,
                    show_done,
                    use_order,
                    limit,
                    page,
                    (range_start, range_end),
                ) {
                    Ok((todos, ids)) => {
                        display_ids = ids;
                        _all_todos = todos;
                    }
                    Err(e) => println!("{}", e),
                }
            }
            "order" | "o" => {
                use_order = !use_order;
                let (todos, ids) = redraw(&repo, show_done, use_order);
                display_ids = ids;
                _all_todos = todos;
            }
            "show" | "s" => {
                show_done = !show_done;
                let (todos, ids) = redraw(&repo, show_done, use_order);
                display_ids = ids;
                _all_todos = todos;
            }
            "help" | "h" | "?" => {
                print_help();
            }
            "quit" | "q" | "exit" => {
                break;
            }
            _ => {
                println!("알 수 없는 명령어: {}", cmd);
                print_help();
            }
        }

        // Redraw if needed
        if needs_redraw {
            let (todos, ids) = redraw(&repo, show_done, use_order);
            display_ids = ids;
            _all_todos = todos;
        }
    }
}