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
use commands::cmd_search;
use db::{SqliteRepo, Todo, TodoRepository};
use formatters::{clear_screen, print_help, show_calendar, show_calendar_weeks, parse_calendar_args, now_prompt};
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

/// Build parent chain (e.g., "1>2>" for multi-level parent)
fn build_parent_chain(parent_id: i64, display_ids: &[i64], all_todos: &[Todo]) -> String {
    let parent_color = "\x1b[38;2;156;163;175m";
    let reset = "\x1b[0m";
    let all_ids: std::collections::HashSet<i64> = all_todos.iter().map(|t| t.id).collect();
    
    // Collect parent IDs from root to immediate (1->2->3)
    let mut parent_ids: Vec<i64> = Vec::new();
    let mut current_id = Some(parent_id);
    
    while let Some(id) = current_id {
        if let Some(p) = all_todos.iter().find(|t| t.id == id) {
            parent_ids.push(p.id);
            current_id = p.up_id;
        } else if all_ids.contains(&id) {
            // Found but deleted - mark as x>
            parent_ids.push(id);
            break;
        } else {
            // Not found - mark as x> and stop
            parent_ids.push(id);
            break;
        }
    }
    
    // Reverse: from root to immediate parent
    parent_ids.reverse();
    
    // Build chain string (1>2> instead of 2>1>)
    let mut chain = String::new();
    let display_id_set: std::collections::HashSet<i64> = display_ids.iter().cloned().collect();
    
    for id in &parent_ids {
        if display_id_set.contains(id) {
            // Parent is visible in display - show list number
            if let Some(pos) = display_ids.iter().position(|&x| x == *id) {
                chain.push_str(&format!("{}{}>", parent_color, pos + 1));
            }
        } else {
            // Parent not visible - show x>
            chain.push_str(&format!("{}x>", parent_color));
        }
    }
    
    if chain.is_empty() {
        String::new()
    } else {
        chain.push_str(reset);
        chain
    }
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
    all_todos: &[Todo],
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

    // Build parent chain (e.g., "1>2>" for multi-level parent)
    let parent_str = if let Some(parent_id) = item.up_id {
        build_parent_chain(parent_id, display_ids, all_todos)
    } else {
        String::new()
    };

    // Format due date with color (@date - both @ and date colored)
    let due_str = if let Some(ref d) = item.due_date {
        let color = formatters::color_for_due_date(d);
        let date = formatters::format_date(d).unwrap_or_default();
        if color.is_empty() {
            format!(" @{}", date)
        } else {
            format!(" {}@{}{}", color, date, "\x1b[0m")
        }
    } else {
        String::new()
    };

    // Format priority with color (^number - both ^ and number colored)
    let priority_colors = ["\x1b[38;2;235;137;53m", "\x1b[38;2;59;130;246m", "\x1b[38;2;52;211;153m", "\x1b[38;2;156;163;175m"];
    let pri_color = *priority_colors.get((item.priority - 1) as usize).unwrap_or(&"");
    let priority_str = if pri_color.is_empty() {
        format!(" ^{}", item.priority)
    } else {
        format!(" {}^{}{}", pri_color, item.priority, "\x1b[0m")
    };

    // Show progress percentage for incomplete todos (after priority)
    let progress = if !is_done && item.done > 0 {
        format!(" {}%", (item.done as usize) * 20)
    } else {
        String::new()
    };

    // Format done date for completed todos (%)
    let done_str = if is_done {
        item.done_at.as_ref().and_then(|dt| formatters::format_date(dt))
            .map(|d| format!(" %{}{}", d, "\x1b[0m"))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let formatted_str = if parent_str != "" {
        format!(
            "{}{} {} {} {}{}{}{}{}\n",
            padded_num, indent, check, parent_str,
            item.todo, due_str, priority_str, progress, done_str
        )
    } else {
        format!(
            "{}{} {} {}{}{}{}{}{}\n",
            padded_num, indent, check, parent_str,
            item.todo, due_str, priority_str, progress, done_str
        )
    };

    formatted_str
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
    // Fetch all todos including deleted for parent lookup
    let all_todos = repo.find_all_including_deleted()?;

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
        // IDs in incomplete list
        let incomplete_ids: std::collections::HashSet<i64> = incomplete.iter().map(|t| t.id).collect();
        
        // Deleted IDs from all_todos
        let deleted_ids: std::collections::HashSet<i64> = all_todos.iter()
            .filter(|t| t.deleted_at.is_some())
            .map(|t| t.id)
            .collect();
        
        // Recursive function to add item and all its descendants
        fn add_with_descendants<'a>(
            item: &'a Todo,
            incomplete: &'a[Todo],
            ordered: &mut Vec<&'a Todo>
        ) {
            if !ordered.iter().any(|x| x.id == item.id) {
                ordered.push(item);
            }
            // Add all children of this item
            for child in incomplete {
                if child.up_id == Some(item.id) {
                    add_with_descendants(child, incomplete, ordered);
                }
            }
        }
        
        // Top-level: no parent OR parent is completed OR parent is deleted or not in incomplete
        let top_level: Vec<_> = incomplete.iter()
            .filter(|t| {
                if t.up_id.is_none() {
                    true // No parent
                } else if let Some(pid) = t.up_id {
                    // Check if parent chain is broken
                    let parent_in_incomplete = incomplete_ids.contains(&pid);
                    let parent_deleted = deleted_ids.contains(&pid);
                    parent_in_incomplete == false || parent_deleted
                } else {
                    false
                }
            })
            .collect();
        
        // Sort top-level by due date, priority, created
        let mut sorted_toplevel: Vec<_> = top_level.iter().collect();
        sorted_toplevel.sort_by(|a, b| {
            let a_has_due = a.due_date.is_some();
            let b_has_due = b.due_date.is_some();
            if a_has_due != b_has_due {
                b_has_due.cmp(&a_has_due)
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
        
        // Build ordered list recursively
        let mut ordered: Vec<&Todo> = Vec::new();
        for item in sorted_toplevel {
            add_with_descendants(item, &incomplete, &mut ordered);
        }
        
        // Add remaining items (shouldn't happen but just in case)
        let remaining: Vec<_> = incomplete.iter()
            .filter(|t| !ordered.iter().any(|x| x.id == t.id))
            .collect();
        for item in remaining {
            add_with_descendants(item, &incomplete, &mut ordered);
        }
        
        ordered.into_iter().cloned().collect()
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
                &all_todos,
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
                &all_todos,
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
    let repo: Arc<dyn TodoRepository> = if let Some(db_path) = &cli_args.db {
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
        println!();
        println!("{}", now_prompt());
        print!("|> ");
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
            "more" | "m" | "ㅡ" => {
                if rest.is_empty() {
                    println!("사용법: more <리스트번호>[,...] [-d 날짜] [-p 우선순위] [-u 리스트번호]");
                } else if let Ok(updated) = commands::cmd_more(&repo, &rest, &display_ids) {
                    needs_redraw = updated;
                } else if let Err(e) = commands::cmd_more(&repo, &rest, &display_ids) {
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
            "past" | "p" | "ㅔ" => {
                show_done = !show_done;
                let (todos, ids) = redraw(&repo, show_done, use_order);
                display_ids = ids;
                _all_todos = todos;
            }
            "search" | "s" | "ㄴ" => {
                if rest.is_empty() {
                    println!("사용법: search <검색어>");
                } else {
                    let keyword = rest.join(" ");
                    if let Err(e) = cmd_search(&repo, &keyword) {
                        println!("검색 오류: {}", e);
                    }
                }
            }
            "calendar" | "c" => {
                if rest.is_empty() {
                    // No args: show 4 weeks from today
                    show_calendar_weeks();
                } else if let Some((year, month)) = parse_calendar_args(&rest) {
                    show_calendar(year, month);
                } else {
                    println!("사용법: calendar [m] [y] 또는 calendar yy/mm");
                }
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
