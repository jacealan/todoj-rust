//! Command implementations for todoj
//! 
//! This module implements all CLI commands that modify or query todos.
//! Each function corresponds to a user command and interacts with the database through TodoRepository.

use crate::cli;
use crate::db::{NewTodo, TodoRepository, UpdateTodo};
use crate::formatters::parse_date;
use std::sync::Arc;

/// Parse inline date/priority from end of content
/// 
/// Extracts @date and ^priority from the end of content string.
/// Only parses if these appear at the END and don't interfere with content.
/// 
/// # Arguments
/// * `input` - Raw input string
/// 
/// # Returns
/// * `(content, due_date, priority)` tuple
/// 
/// # Examples
/// 
/// ```rust
/// parse_inline("Buy milk @3/15 ^2") -> ("Buy milk", Some("20260315"), Some(2))
/// parse_inline("check @email") -> ("check @email", None, None)  // @ not at end
/// ```
fn parse_inline(input: &str) -> (String, Option<String>, Option<i32>) {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return (input.to_string(), None, None);
    }
    
    let last = words.last().unwrap_or(&"");
    let second_last = words.get(words.len().saturating_sub(2));
    
    // Check for ^priority at end (1-4 only)
    let priority = if last.starts_with('^') && last.len() > 1 {
        let pri = last[1..].parse::<i32>().ok();
        if let Some(p) = pri {
            if p >= 1 && p <= 4 { Some(p) } else { None }
        } else { None }
    } else {
        None
    };
    
    // Check for @date at end (if no ^priority) or second to last (if ^priority exists)
    // Only parse if it looks like a date - contains / or - or all digits
    let mut due_date: Option<String> = None;
    let mut has_due_at_end = false;
    let mut has_due_second_last = false;
    
    let looks_like_date = |s: &str| -> bool {
        let inner = s.strip_prefix('@').unwrap_or(s).to_lowercase();
        inner == "today" || inner == "tom" || inner == "tomorrow" 
            || inner == "mon" || inner == "monday"
            || inner == "tue" || inner == "tuesday"
            || inner == "wed" || inner == "wednesday"
            || inner == "thu" || inner == "thursday"
            || inner == "fri" || inner == "friday"
            || inner == "sat" || inner == "saturday"
            || inner == "sun" || inner == "sunday"
            // Korean keywords
            || inner == "오늘" || inner == "jntn"
            || inner == "내일" || inner == "tkfq"
            || inner == "월" || inner == "화" || inner == "수" || inner == "목" || inner == "금" || inner == "토" || inner == "일"
            || inner.contains('/') || inner.contains('-') || inner.chars().all(|c| c.is_ascii_digit())
    };
    
    if last.starts_with('@') && last.len() > 1 {
        let inner = &last[1..];
        if looks_like_date(last) {
            if let Some(parsed) = parse_date(inner) {
                due_date = Some(parsed);
                has_due_at_end = true;
            } else {
                // Date-like pattern but invalid - return content as-is (don't parse)
                return (input.to_string(), None, None);
            }
        }
    } else if let Some(sl) = second_last {
        if sl.starts_with('@') && sl.len() > 1 && priority.is_some() {
            let inner = &sl[1..];
            if looks_like_date(sl) {
                if let Some(parsed) = parse_date(inner) {
                    due_date = Some(parsed);
                    has_due_second_last = true;
                } else {
                    return (input.to_string(), None, None);
                }
            }
        }
    }
    
    // Rebuild content without @date and ^priority
    let mut new_content = String::new();
    let mut skip_last = false;
    let mut skip_second_last = false;
    
    if priority.is_some() && last.starts_with('^') {
        skip_last = true;
    }
    if has_due_at_end {
        skip_last = true;
    } else if has_due_second_last {
        skip_second_last = true;
    }
    
    let len = words.len();
    for (i, word) in words.iter().enumerate() {
        let should_skip = if i == len - 1 && skip_last {
            true
        } else if len >= 2 && i == len - 2 && skip_second_last {
            true
        } else {
            false
        };
        
        if !should_skip {
            if !new_content.is_empty() {
                new_content.push(' ');
            }
            new_content.push_str(word);
        }
    }
    
    (new_content, due_date, priority)
}

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
    let mut raw_content = String::new();
    let mut due_date = None;
    let mut priority = None;
    let mut up_id = None;
    let mut i = 0;

    // First pass: collect raw content and check for -d, -p, -u flags
    while i < args.len() {
        match args[i] {
            // Due date: -d 3/15
            "-d" if i + 1 < args.len() => {
                if args[i + 1].starts_with('@') {
                    return Err("날짜는 @ 없이 입력: -d 3/15 또는 @3/15 (예: @today, @3/15, @tom)".to_string());
                }
                due_date = parse_date(args[i + 1]);
                if due_date.is_none() && args[i + 1].chars().any(|c| c.is_ascii_digit()) {
                    return Err(format!("잘못된 날짜입니다: {} - calendar 명령어로 확인하세요.", args[i + 1]));
                }
                i += 2;
            }
            // Priority: -p 1-4 (default 3)
            "-p" if i + 1 < args.len() => {
                let p = args[i + 1].parse::<i32>().ok();
                if p.is_none() || p.unwrap() < 1 || p.unwrap() > 4 {
                    return Err("잘못된 우선순위입니다. 1-4 숫자를 입력: -p 1~4 또는 ^1~4".to_string());
                }
                priority = p;
                i += 2;
            }
            // Parent todo: -u 5 (creates sub-task under todo #5)
            "-u" if i + 1 < args.len() => {
                if let Ok(num) = args[i + 1].parse::<usize>() {
                    if num > 0 && num <= display_ids.len() {
                        up_id = Some(display_ids[num - 1]);
                    }
                } else {
                    return Err("잘못된 리스트 번호입니다. 리스트 번호를 입력해주세요.".to_string());
                }
                i += 2;
            }
            // Content (everything else)
            _ => {
                if !raw_content.is_empty() {
                    raw_content.push(' ');
                }
                raw_content.push_str(args[i]);
                i += 1;
            }
        }
    }

    // Parse inline @date and ^priority from content end (only if -d/-p NOT provided)
    let use_inline = due_date.is_none() && priority.is_none();
    let (content, inline_due, inline_pri) = if use_inline {
        parse_inline(&raw_content)
    } else {
        // Keep @ and ^ as content if -d/-p provided
        (raw_content.clone(), None, None)
    };
    
    // Validate: only check invalid patterns when using inline
    if use_inline && !raw_content.is_empty() {
        let words: Vec<&str> = raw_content.split_whitespace().collect();
        if let Some(last) = words.last() {
            if last.starts_with('@') && last.len() > 1 {
                let inner = last.strip_prefix('@').unwrap_or("");
                if (inner.chars().all(|c| c.is_ascii_digit()) || inner.contains('/') || inner.contains('-')) && inline_due.is_none() {
                    let suggestion = if inner.contains('/') { 
                        "월/일 형식으로 입력: @월/일 (예: @3/15, @12/31)".to_string() 
                    } else if inner.contains('-') {
                        "년-월-일 형식으로 입력: @년-월-일 (예: @26-3-15)".to_string()
                    } else {
                        format!("{}월의 날짜가 없습니다. calendar 명령어로 확인하세요.", inner)
                    };
                    return Err(format!("잘못된 날짜입니다: @{} - {}", inner, suggestion));
                }
            }
            if last.starts_with('^') && last.len() > 1 {
                let p_str = &last[1..];
                if p_str.parse::<i32>().is_err() || p_str.parse::<i32>().map(|p| p < 1 || p > 4).unwrap_or(true) {
                    return Err("잘못된 우선순위입니다. ^1, ^2, ^3, ^4 중 하나를 입력해주세요.".to_string());
                }
            }
        }
    }
    
    // Use -d/-p if provided, otherwise inline values
    let final_due = due_date.or(inline_due);
    let final_pri = priority.or(inline_pri);

    // Validate content
    if content.is_empty() {
        return Err("TODO 내용을 입력해주세요.".to_string());
    }

    // Create in database
    repo.create(NewTodo {
        todo: content,
        due_date: final_due,
        priority: final_pri,
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

        // Parse update options (including inline @date and ^priority, and -u for parent)
        let mut raw_new_content: Option<String> = None;
        let mut due_date = None;
        let mut priority = None;
        let mut up_id = None;
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
                "-u" if i + 1 < args.len() => {
                    if let Ok(num) = args[i + 1].parse::<usize>() {
                        if num > 0 && num <= display_ids.len() {
                            up_id = Some(display_ids[num - 1]);
                        }
                    }
                    i += 2;
                }
                _ => {
                    // Collect other args as content
                    if raw_new_content.is_none() {
                        raw_new_content = Some(args[i].to_string());
                    } else {
                        raw_new_content = Some(format!("{} {}", raw_new_content.unwrap(), args[i]));
                    }
                    i += 1;
                }
            }
        }

        // Parse inline @date and ^priority (only if -d/-p NOT provided)
        let use_inline = due_date.is_none() && priority.is_none();
        let (content_changed, inline_due, inline_pri) = if let Some(ref raw) = raw_new_content {
            if use_inline {
                parse_inline(raw)
            } else {
                // Keep raw content when using -d/-p
                (raw.clone(), None, None)
            }
        } else {
            (String::new(), None, None)
        };
        
        // Validate inline date (only when using inline and -d not provided)
        if use_inline && due_date.is_none() && raw_new_content.is_some() {
            let raw = raw_new_content.as_ref().unwrap();
            let words: Vec<&str> = raw.split_whitespace().collect();
            if let Some(last) = words.last() {
                if last.starts_with('@') && last.len() > 1 {
                    let inner = last.strip_prefix('@').unwrap_or("");
                    if (inner.contains('/') || inner.contains('-') || inner.chars().all(|c| c.is_ascii_digit())) && inline_due.is_none() {
                        return Err(format!("잘못된 날짜입니다: @{} - calendar 명령어로 확인하세요.", inner));
                    }
                }
            }
        }
        
        // Validate inline priority (only when using inline and -p not provided)
        if use_inline && priority.is_none() && raw_new_content.is_some() {
            let raw = raw_new_content.as_ref().unwrap();
            let words: Vec<&str> = raw.split_whitespace().collect();
            if let Some(last) = words.last() {
                if last.starts_with('^') && last.len() > 1 {
                    let p_str = &last[1..];
                    if p_str.parse::<i32>().is_err() || p_str.parse::<i32>().map(|p| p < 1 || p > 4).unwrap_or(true) {
                        return Err("잘못된 우선순위입니다. ^1, ^2, ^3, ^4 중 하나를 입력해주세요.".to_string());
                    }
                }
            }
        }
        
        // Use -d/-p if provided, otherwise inline values
        let final_due = due_date.or(inline_due);
        let final_pri = priority.or(inline_pri);
        let final_up_id = up_id;
        
// Update todo content if:
// 1. raw_new_content was provided AND
// 2. Either of these is true:
//    - inline patterns (@ or ^) were found (content_changed different from raw)
//    - OR no @/^ patterns at all in raw (plain text content)
// 3. IMPORTANT: @/^ pattern alone (e.g., "@today") should NOT update content
let final_todo = if let Some(ref raw) = raw_new_content {
    let has_inline_patterns = (raw.contains('@') || raw.contains('^')) && (content_changed != *raw);
    let has_plain_text = !raw.contains('@') && !raw.contains('^');
    
    // Check if @/^ is alone (no other words in input)
    // "e 11 @today" → ["e", "11", "@today"] → @today is the only non-number
    // In this case, don't update content - only date/priority
    let words: Vec<&str> = raw.split_whitespace().collect();
    let other_words: Vec<&&str> = words.iter()
        .filter(|w| !w.starts_with('@') && !w.starts_with('^'))
        .collect();
    let has_only_inline = other_words.is_empty() && !words.is_empty();
    
    if has_inline_patterns || has_plain_text {
        if has_only_inline {
            // Only @/^ present (e.g., "@today" alone) - don't update content
            None
        } else {
            Some(content_changed)
        }
    } else {
        None
    }
} else {
    None
};

        // Apply updates
        let mut updated = false;
        for id in valid_ids {
            repo.update(
                id,
                UpdateTodo {
                    todo: final_todo.clone(),
                    due_date: final_due.clone(),
                    priority: final_pri,
                    up_id: final_up_id.clone(),
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
        
        // Prompt for new content (supports inline @date ^priority)
        println!("사용법: edit {} <새 내용> [-d 날짜] [-p 우선순위] [-u 리스트번호]", num);
        println!("     또는: edit {} <새 내용@날짜 ^우선순위>", num);
        print!("수정: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let new_input = input.trim();
            if !new_input.is_empty() {
                // Use inline parsing - it should extract content without @/^, leaving them as date/priority
                let (content, due_date, priority) = parse_inline(new_input);
                
                // Update todo content if changed (parsed doesn't have @ or ^ at end)
                let update_content = if !content.is_empty() {
                    Some(content)
                } else {
                    // No inline patterns found, use full input as content
                    Some(new_input.to_string())
                };
                
                repo.update(
                    display_ids[num - 1],
                    UpdateTodo {
                        todo: update_content,
                        due_date,
                        priority,
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