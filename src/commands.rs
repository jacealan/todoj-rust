//! Command implementations for todoj
//! 
//! This module implements all CLI commands that modify or query todos.
//! Each function corresponds to a user command and interacts with the database through TodoRepository.

use crate::cli;
use crate::db::{NewTodo, TodoRepository, UpdateTodo};
use crate::formatters::parse_date;
use std::sync::Arc;

/// Add a new todo item
/// 
/// Creates a new todo in the database.
/// Supports content, due date, priority, and parent todo (sub-todo).
/// 
/// # Arguments
/// * `repo` - Database repository
/// * `args` - Command arguments containing:
///   - Content (required): everything not starting with -
///   - `-d` or `--due`: due date (optional)
///   - `-p` or `--priority`: priority 1-4 (optional, default 3)
///   - `-u` or `--up`: parent todo number (optional, creates sub-todo)
/// * `display_ids` - Current display IDs for resolving parent references
/// 
/// # Returns
/// * `Ok(true)` on success
/// * `Err(message)` on failure
/// 
/// # Examples
/// 
/// ```bash
/// add Buy milk
/// add Review PR -d 3/15 -p 2
/// add Sub-task -u 5  # Create sub-task under todo #5
/// ```
pub fn cmd_add(
    repo: &Arc<dyn TodoRepository>, 
    args: &[&str], 
    display_ids: &[i64]
) -> Result<bool, String> {
    let mut content = String::new();
    let mut due_date = None;
    let mut priority = None;
    let mut up_id = None;
    let mut i = 0;

    // Parse arguments
    while i < args.len() {
        match args[i] {
            // Due date: -d 3/15 or --due=3/15
            "-d" if i + 1 < args.len() => {
                due_date = parse_date(args[i + 1]);
                i += 2;
            }
            // Priority: -p 1-4 (default 3)
            "-p" if i + 1 < args.len() => {
                priority = args[i + 1].parse().ok();
                i += 2;
            }
            // Parent todo: -u 5 (creates sub-task under todo #5)
            "-u" if i + 1 < args.len() => {
                if let Ok(num) = args[i + 1].parse::<usize>() {
                    if num > 0 && num <= display_ids.len() {
                        up_id = Some(display_ids[num - 1]);
                    }
                }
                i += 2;
            }
            // Content (everything else)
            _ => {
                if !content.is_empty() {
                    content.push(' ');
                }
                content.push_str(args[i]);
                i += 1;
            }
        }
    }

    // Validate content
    if content.is_empty() {
        return Err("TODO 내용을 입력해주세요.".to_string());
    }

    // Create in database
    repo.create(NewTodo {
        todo: content,
        due_date,
        priority,
        up_id,
    })?;
    println!("추가되었습니다.");
    Ok(true)
}

/// Edit existing todo(s)
/// 
/// Supports two modes:
/// 1. Batch edit: edit multiple todos with new due date or priority
/// 2. Interactive edit: edit single todo with new content
/// 
/// # Arguments
/// * `repo` - Database repository
/// * `args` - Command arguments:
///   - First arg: todo number(s) like "1" or "1,3,5" or "2-4"
///   - `-d` or `--due`: new due date (optional)
///   - `-p` or `--priority`: new priority (optional)
///   - If only number provided: enter interactive mode
/// * `display_ids` - Current display IDs
/// 
/// # Returns
/// * `Ok(true)` if modified
/// * `Err(message)` on error
pub fn cmd_edit(
    repo: &Arc<dyn TodoRepository>,
    args: &[&str],
    display_ids: &[i64],
) -> Result<bool, String> {
    // Print usage if no arguments
    if args.is_empty() {
        return Err("사용법: edit <리스트번호>[,...] [-d 날짜] [-p 우선순위]".to_string());
    }

    // Batch edit mode: has more arguments after todo number
    if args.len() > 1 {
        let nums = cli::parse_numbers(args[0]);
        let mut valid_ids = Vec::new();
        
        // Validate todo numbers
        for n in &nums {
            if *n == 0 || *n > display_ids.len() {
                return Err("유효한 리스트 번호를 입력해주세요.".to_string());
            }
            valid_ids.push(display_ids[n - 1]);
        }

        // Parse update options
        let mut due_date = None;
        let mut priority = None;
        let mut i = 1;
        while i < args.len() {
            match args[i] {
                "-d" if i + 1 < args.len() => {
                    due_date = parse_date(args[i + 1]);
                    i += 2;
                }
                "-p" if i + 1 < args.len() => {
                    priority = args[i + 1].parse().ok();
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        // Apply updates
        let mut updated = false;
        for id in valid_ids {
            repo.update(
                id,
                UpdateTodo {
                    todo: None,  // Content unchanged in batch mode
                    due_date: due_date.clone(),
                    priority,
                    up_id: None,  // Parent unchanged
                },
            )?;
            updated = true;
        }
        if updated {
            println!("수정되었습니다.");
        }
        return Ok(true);
    }

    // Interactive edit mode: single todo number
    if let Ok(num) = args[0].parse::<usize>() {
        if num == 0 || num > display_ids.len() {
            return Err("유효한 리스트 번호를 입력해주세요.".to_string());
        }
        
        // Prompt for new content
        println!("사용법: edit {} <새 내용> [-d 날짜] [-p 우선순위] [-u 리스트번호>", num);
        print!("수정: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let new_input = input.trim();
            if !new_input.is_empty() {
                repo.update(
                    display_ids[num - 1],
                    UpdateTodo {
                        todo: Some(new_input.to_string()),
                        due_date: None,
                        priority: None,
                        up_id: None,
                    },
                )?;
                println!("수정되었습니다.");
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        Err("유효한 리스트 번호를 입력해주세요.".to_string())
    }
}

/// Remove (soft delete) todo(s)
/// 
/// Performs soft delete by setting deleted_at timestamp.
/// Todo is marked as deleted but not permanently removed.
/// 
/// # Arguments
/// * `repo` - Database repository
/// * `args` - Todo numbers like "1" or "1,3" or "2-5"
/// * `display_ids` - Current display IDs
/// 
/// # Returns
/// * `Ok(true)` on success
/// * `Err(message)` on error
pub fn cmd_remove(
    repo: &Arc<dyn TodoRepository>,
    args: &[&str],
    display_ids: &[i64],
) -> Result<bool, String> {
    if args.is_empty() {
        return Err("사용법: remove <리스트번호>[,...]".to_string());
    }

    let nums = cli::parse_numbers(args[0]);
    let mut valid_ids = Vec::new();
    
    // Validate numbers
    for n in &nums {
        if *n == 0 || *n > display_ids.len() {
            return Err("유효한 리스트 번호를 입력해주세요.".to_string());
        }
        valid_ids.push(display_ids[n - 1]);
    }

    // Soft delete each todo
    for id in valid_ids {
        repo.delete(id)?;
    }
    println!("삭제되었습니다.");
    Ok(true)
}

/// Mark todo(s) as done or in progress
/// 
/// Done levels:
/// - 0: Not done
/// - 1: 20% done
/// - 2: 40% done
/// - 3: 60% done
/// - 4: 80% done
/// - 5: Complete
/// 
/// If no level specified, toggles between 0 and 5.
/// 
/// # Arguments
/// * `repo` - Database repository
/// * `args` - First arg: todo number(s), optional second: done level (0-5)
/// * `display_ids` - Current display IDs
/// 
/// # Returns
/// * `Ok(true)` on success
/// * `Err(message)` on error
pub fn cmd_done(
    repo: &Arc<dyn TodoRepository>,
    args: &[&str],
    display_ids: &[i64],
) -> Result<bool, String> {
    if args.is_empty() {
        return Err("사용법: done <리스트번호>[,...] [0-5]".to_string());
    }

    let nums = cli::parse_numbers(args[0]);
    
    // Parse done level (optional, defaults to toggle)
    let level = if args.len() > 1 {
        args[1].parse().ok()
    } else {
        None
    };
    
    let mut valid_ids = Vec::new();
    for n in &nums {
        if *n == 0 || *n > display_ids.len() {
            return Err("유효한 리스트 번호를 입력해주세요.".to_string());
        }
        valid_ids.push(display_ids[n - 1]);
    }

    // Set done level for each
    for id in valid_ids {
        repo.set_done(id, level)?;
    }
    println!("완료 상태가 변경되었습니다.");
    Ok(true)
}