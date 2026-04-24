use chrono::{Datelike, Local, NaiveDate, Utc};
use clap::Parser;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;
use std::io::Write;

static DB_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

fn get_db_path() -> PathBuf {
    let mut guard = DB_PATH.lock().unwrap();
    if let Some(ref path) = *guard {
        return path.clone();
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let path = home.join(".todoj.db");
    *guard = Some(path.clone());
    path
}

fn init_db() -> Result<Connection, rusqlite::Error> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            todo TEXT NOT NULL,
            due_date TEXT,
            priority INTEGER DEFAULT 3,
            up_id INTEGER,
            done INTEGER DEFAULT 0,
            done_at TEXT,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

fn now_datetime() -> String {
    Utc::now().format("%Y%m%dT%H%M%S").to_string()
}

fn now_date() -> String {
    Local::now().format("%Y%m%d").to_string()
}

fn format_date(date_str: &str) -> Option<String> {
    if date_str.len() >= 8 {
        let year: i32 = date_str[..4].parse().ok()?;
        let month: u32 = date_str[4..6].parse().ok()?;
        let day: u32 = date_str[6..8].parse().ok()?;
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let weekday = date.format("%a").to_string();
        let formatted = date.format("%y-%m-%d").to_string();
        Some(format!("{}({})", formatted, weekday))
    } else {
        None
    }
}

fn parse_date(input: &str) -> Option<String> {
    let today = Local::now().naive_local().date();
    let parts: Vec<&str> = input.split(|c| c == '/' || c == '-').collect();

    match parts.len() {
        1 => {
            let day: u32 = parts[0].parse().ok()?;
            let date = NaiveDate::from_ymd_opt(today.year(), today.month(), day)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(today.year(), today.month() + 1, day).unwrap());
            Some(date.format("%Y%m%d").to_string())
        }
        2 => {
            let month: u32 = parts[0].parse().ok()?;
            let day: u32 = parts[1].parse().ok()?;
            let year = today.year();
            let date = NaiveDate::from_ymd_opt(year, month, day)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, month, day).unwrap());
            Some(date.format("%Y%m%d").to_string())
        }
        3 => {
            let year: i32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            let day: u32 = parts[2].parse().ok()?;
            let full_year = if year < 100 { 2000 + year } else { year };
            let date = NaiveDate::from_ymd_opt(full_year, month, day)?;
            Some(date.format("%Y%m%d").to_string())
        }
        _ => None,
    }
}

fn add_todo(conn: &Connection, args: &[&str], display_ids: &[i64]) -> Result<(), String> {
    let mut content = String::new();
    let mut due_date: Option<String> = None;
    let mut priority: i32 = 3;
    let mut up_id: Option<i64> = None;
    let mut i = 0;

    while i < args.len() {
        match args[i] {
            "-d" if i + 1 < args.len() => {
                due_date = parse_date(args[i + 1]);
                i += 2;
            }
            "-p" if i + 1 < args.len() => {
                priority = args[i + 1].parse().unwrap_or(3);
                i += 2;
            }
            "-u" if i + 1 < args.len() => {
                if let Ok(num) = args[i + 1].parse::<usize>() {
                    if num > 0 && num <= display_ids.len() {
                        up_id = Some(display_ids[num - 1]);
                    } else {
                        return Err("유효한 리스트 번호를 입력해주세요.".to_string());
                    }
                }
                i += 2;
            }
            _ => {
                if !content.is_empty() {
                    content.push(' ');
                }
                content.push_str(args[i]);
                i += 1;
            }
        }
    }

    if content.is_empty() {
        return Err("TODO 내용을 입력해주세요.".to_string());
    }

    let created = now_datetime();
    let updated = created.clone();

    conn.execute(
        "INSERT INTO todos (todo, due_date, priority, up_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![content, due_date, priority, up_id, created, updated],
    ).map_err(|e| e.to_string())?;

    println!("추가되었습니다.");
    Ok(())
}

fn edit_todo(conn: &Connection, id: i64, args: &[&str], display_ids: &[i64]) -> Result<(), String> {
    let mut stmt = conn.prepare("SELECT id, todo, due_date, priority, up_id FROM todos WHERE id = ?1 AND deleted_at IS NULL")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let mut content: String = row.get(1).unwrap_or_default();
        let mut due_date: Option<String> = row.get(2).unwrap_or(None);
        let mut priority: i32 = row.get(3).unwrap_or(3);
        let mut up_id: Option<i64> = row.get(4).unwrap_or(None);

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "-d" if i + 1 < args.len() => {
                    due_date = parse_date(args[i + 1]);
                    i += 2;
                }
                "-p" if i + 1 < args.len() => {
                    priority = args[i + 1].parse().unwrap_or(priority);
                    i += 2;
                }
                "-u" if i + 1 < args.len() => {
                    if let Ok(num) = args[i + 1].parse::<usize>() {
                        if num > 0 && num <= display_ids.len() {
                            up_id = Some(display_ids[num - 1]);
                        } else {
                            return Err("유효한 리스트 번호를 입력해주세요.".to_string());
                        }
                    }
                    i += 2;
                }
                _ => {
                    content = args[i..].join(" ");
                    break;
                }
            }
        }

        let updated = now_datetime();
        conn.execute(
            "UPDATE todos SET todo = ?1, due_date = ?2, priority = ?3, up_id = ?4, updated_at = ?5 WHERE id = ?6",
            params![content, due_date, priority, up_id, updated, id],
        ).map_err(|e| e.to_string())?;

        println!("수정되었습니다.");
    } else {
        return Err("해당 TODO를 찾을 수 없습니다.".to_string());
    }

    Ok(())
}

fn remove_todo(conn: &Connection, id: i64) -> Result<(), String> {
    let deleted = now_datetime();
    conn.execute(
        "UPDATE todos SET deleted_at = ?1 WHERE id = ?2",
        params![deleted, id],
    ).map_err(|e| e.to_string())?;
    println!("삭제되었습니다.");
    Ok(())
}

fn set_done(conn: &Connection, id: i64, done_level: Option<i32>) -> Result<(), String> {
    let mut stmt = conn.prepare("SELECT done FROM todos WHERE id = ?1 AND deleted_at IS NULL")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let current_done: i32 = row.get(0).unwrap_or(0);
        let new_done = done_level.unwrap_or(if current_done == 5 { 0 } else { 5 });
        let done_at = if new_done == 5 { Some(now_date()) } else { None };

        conn.execute(
            "UPDATE todos SET done = ?1, done_at = ?2 WHERE id = ?3",
            params![new_done, done_at, id],
        ).map_err(|e| e.to_string())?;

        println!("완료 상태가 변경되었습니다.");
    } else {
        return Err("해당 TODO를 찾을 수 없습니다.".to_string());
    }

    Ok(())
}

fn remove_todos(conn: &Connection, ids: &[i64]) -> Result<(), String> {
    let deleted = now_datetime();
    for id in ids {
        conn.execute(
            "UPDATE todos SET deleted_at = ?1 WHERE id = ?2",
            params![deleted, id],
        ).map_err(|e| e.to_string())?;
    }
    println!("삭제되었습니다.");
    Ok(())
}

fn set_done_batch(conn: &Connection, ids: &[i64], level: Option<i32>) -> Result<(), String> {
    for id in ids {
        let mut stmt = conn.prepare("SELECT done FROM todos WHERE id = ?1 AND deleted_at IS NULL")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let current_done: i32 = row.get(0).unwrap_or(0);
            let new_done = level.unwrap_or(if current_done == 5 { 0 } else { 5 });
            let done_at = if new_done == 5 { Some(now_date()) } else { None };

            conn.execute(
                "UPDATE todos SET done = ?1, done_at = ?2 WHERE id = ?3",
                params![new_done, done_at, id],
            ).map_err(|e| e.to_string())?;
        }
    }
    println!("완료 상태가 변경되었습니다.");
    Ok(())
}

fn parse_numbers(input: &str) -> Vec<usize> {
    let mut numbers = Vec::new();
    for part in input.split(',') {
        if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (range_parts[0].parse(), range_parts[1].parse()) {
                    for i in start..=end {
                        numbers.push(i);
                    }
                }
            }
        } else if let Ok(n) = part.parse() {
            numbers.push(n);
        }
    }
    numbers
}

fn parse_list_range(args: &[&str]) -> (Option<usize>, Option<usize>, Option<usize>, Option<usize>) {
    let mut limit = None;
    let mut page = None;
    let mut range_start = None;
    let mut range_end = None;

    for arg in args {
        if arg.contains('/') {
            let parts: Vec<&str> = arg.split('/').collect();
            if parts.len() == 2 {
                if let Ok(l) = parts[0].parse() {
                    limit = Some(l);
                }
                if let Ok(p) = parts[1].parse() {
                    page = Some(p);
                }
            }
        } else if arg.contains('-') {
            let parts: Vec<&str> = arg.split('-').collect();
            if parts.len() == 2 {
                if let Ok(s) = parts[0].parse() {
                    range_start = Some(s);
                }
                if let Ok(e) = parts[1].parse() {
                    range_end = Some(e);
                }
            }
        } else if let Ok(n) = arg.parse::<usize>() {
            limit = Some(n);
        }
    }

    (limit, page, range_start, range_end)
}

fn list_todos(conn: &Connection, show_done: bool, use_order: bool, limit: Option<usize>, page: Option<usize>, range: (Option<usize>, Option<usize>)) -> Result<Vec<i64>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, todo, due_date, priority, up_id, done, done_at
         FROM todos WHERE deleted_at IS NULL
         ORDER BY CASE WHEN due_date IS NULL THEN 1 ELSE 0 END, due_date ASC, priority ASC, created_at DESC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, i32>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    }).map_err(|e| e.to_string())?;

    let mut incomplete: Vec<(i64, String, Option<String>, i32, Option<i64>, i32, Option<String>)> = Vec::new();
    let mut completed: Vec<(i64, String, Option<String>, i32, Option<i64>, i32, Option<String>)> = Vec::new();

    for row in rows {
        let item = row.map_err(|e| e.to_string())?;
        if item.5 == 5 {
            completed.push(item);
        } else {
            incomplete.push(item);
        }
    }

    let mut flags = String::new();
    if use_order {
        flags.push_str(" order");
    }
    if show_done {
        flags.push_str(" show");
    }

    let mut output = String::new();
    output.push_str(&format!("=== TODO{} ===\n", if flags.is_empty() { String::new() } else { flags }));

    if incomplete.is_empty() && (!show_done || completed.is_empty()) {
        output.push_str("TODO가 없습니다.\n");
        print!("{}", output);
        return Ok(Vec::new());
    }

    let display_incomplete: Vec<_> = if use_order {
        let mut parents: Vec<_> = incomplete.iter().filter(|(_, _, _, _, up, _, _)| up.is_none()).cloned().collect();
        let children: Vec<_> = incomplete.iter().filter(|(_, _, _, _, up, _, _)| up.is_some()).cloned().collect();

        parents.sort_by(|a, b| {
            let (_, _, duea, pria, _, _, _) = a;
            let (_, _, dueb, prib, _, _, _) = b;
            let a_has_due = duea.is_some();
            let b_has_due = dueb.is_some();
            if a_has_due != b_has_due {
                b_has_due.cmp(&a_has_due)
            } else {
                duea.as_ref().map(|s| s.as_str()).unwrap_or("").cmp(dueb.as_ref().map(|s| s.as_str()).unwrap_or(""))
                    .then(pria.cmp(prib))
                    .then(b.0.cmp(&a.0))
            }
        });

        let mut ordered: Vec<_> = Vec::new();
        for parent in &parents {
            ordered.push((*parent).clone());
            for child in &children {
                if child.4 == Some(parent.0) {
                    ordered.push(child.clone());
                }
            }
        }
        ordered.into_iter().enumerate().collect()
    } else {
        incomplete.into_iter().enumerate().collect()
    };

    let display_completed: Vec<_> = completed.into_iter().enumerate().collect();
    let incomplete_len = display_incomplete.len();

    let mut display_ids = Vec::new();

    let (items_to_show, (range_start, range_end)) = {
        let (rs, re) = range;
        let total_incomplete = display_incomplete.len();
        let total_completed = if show_done { display_completed.len() } else { 0 };
        let total = total_incomplete + total_completed;

        if let Some(start) = rs {
            let end = re.unwrap_or(total);
            (Some((start, end)), (Some(start), Some(end)))
        } else if let (Some(lim), Some(p)) = (limit, page) {
            let start_idx = (p - 1) * lim;
            let end_idx = start_idx + lim;
            (Some((start_idx + 1, end_idx.min(total))), (None, None))
        } else if let Some(lim) = limit {
            let end_idx = lim.min(total);
            (Some((1, end_idx)), (None, None))
        } else {
            (None, (None, None))
        }
    };

    if !display_incomplete.is_empty() {
        let max_num = display_incomplete.len();
        let width = max_num.to_string().len();

        let filtered: Vec<_> = match items_to_show {
            Some((start, end)) => display_incomplete.iter().enumerate()
                .filter(|(idx, _)| {
                    let num = idx + 1;
                    num >= start && num <= end
                })
                .collect(),
            None => display_incomplete.iter().enumerate().collect(),
        };

        for (idx, item) in &filtered {
            display_ids.push(item.1.0);
        }
        for (idx, item) in &filtered {
            output.push_str(&format_todo(*idx + 1, width, &item.1, false, &display_ids, use_order));
        }
    }

    if show_done && !display_completed.is_empty() {
        if incomplete_len > 0 {
            output.push('\n');
        }
        let mut sorted_completed = display_completed;
        sorted_completed.sort_by(|a, b| {
            let date_a = &a.1.6;
            let date_b = &b.1.6;
            date_b.cmp(date_a)
        });
        let start_num = incomplete_len + 1;
        let width = (start_num + sorted_completed.len() - 1).to_string().len();
        for (idx, item) in &sorted_completed {
            display_ids.push(item.0);
        }
        for (idx, item) in &sorted_completed {
            output.push_str(&format_todo(start_num + idx, width, item, true, &display_ids, use_order));
        }
    }

    print!("{}", output);
    std::io::stdout().flush().ok();

    Ok(display_ids)
}



fn format_todo(num: usize, width: usize, item: &(i64, String, Option<String>, i32, Option<i64>, i32, Option<String>), is_done: bool, display_ids: &[i64], use_order: bool) -> String {
    let (id, todo, due_date, priority, up_id, done, done_at) = item;
    let num_str = format!("{}", num);
    let padded_num = format!("{:>width$}", num_str);

    let indent = if use_order && up_id.is_some() { "  " } else { "" };
    let check = if is_done { "[x]" } else { "[ ]" };

    let parent_ref = if let Some(parent_id) = up_id {
        if let Some(idx) = display_ids.iter().position(|&x| x == *parent_id) {
            Some(idx + 1)
        } else {
            None
        }
    } else {
        None
    };
    let parent_str = parent_ref.map(|p| format!("{p}>")).unwrap_or_default();

    let due = due_date.as_ref().map(|d| format_date(d)).flatten();
    let due_str = due.map(|d| format!(" @{d}")).unwrap_or_default();

    let progress = if !is_done && *done > 0 {
        format!(" {}%", (*done as usize) * 20)
    } else {
        String::new()
    };

    let done_str = if is_done {
        if let Some(ref dt) = done_at {
            if let Some(formatted) = format_date(dt) {
                format!(" %{}", formatted)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!("{}{} {} {} {}{} {}{} ^{}{}\n", padded_num, indent, check, parent_str, todo, due_str, progress, "", priority, done_str)
}

fn print_todo(num: usize, width: usize, item: &(i64, String, Option<String>, i32, Option<i64>, i32, Option<String>), is_done: bool, display_ids: &[i64], use_order: bool) {
    print!("{}", format_todo(num, width, item, is_done, display_ids, use_order));
}

fn main() {
    let args = Args::parse();

    let conn = match init_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("DB 오류: {}", e);
            return;
        }
    };

    if args.list_mode {
        let _ = list_todos(&conn, false, false, None, None, (None, None));
        return;
    }

    let mut show_done = false;
    let mut use_order = false;

    fn redraw(conn: &Connection, show_done: bool, use_order: bool) -> Vec<i64> {
        clear_screen();
        list_todos(conn, show_done, use_order, None, None, (None, None)).unwrap_or_default()
    }

    let mut display_ids = redraw(&conn, show_done, use_order);

    loop {
        print!("\n> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0];
        let rest: Vec<&str> = parts[1..].to_vec();
        let mut needs_redraw = false;

        match cmd {
            "add" | "a" => {
                if rest.is_empty() {
                    println!("사용법: add <내용> [-d 날짜] [-p 우선순위] [-u 리스트번호]");
                } else if let Err(e) = add_todo(&conn, &rest, &display_ids) {
                    println!("{}", e);
                } else {
                    needs_redraw = true;
                }
            }
            "edit" | "e" => {
                if rest.is_empty() {
                    println!("사용법: edit <리스트번호> [-d 날짜] [-p 우선순위] [-u 리스트번호]");
                } else if let Ok(num) = rest[0].parse::<usize>() {
                    if num == 0 || num > display_ids.len() {
                        println!("유효한 리스트 번호를 입력해주세요.");
                    } else if rest.len() == 1 {
                        println!("사용법: edit {} <새 내용> [-d 날짜] [-p 우선순위] [-u 리스트번호>", num);
                        print!("수정: ");
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_ok() {
                            let new_input = input.trim();
                            if new_input.is_empty() {
                                println!("수정 취소됨.");
                            } else {
                                let parts: Vec<&str> = new_input.split_whitespace().collect();
                                if let Err(e) = edit_todo(&conn, display_ids[num - 1], &parts, &display_ids) {
                                    println!("{}", e);
                                } else {
                                    needs_redraw = true;
                                }
                            }
                        }
                    } else {
                        if let Err(e) = edit_todo(&conn, display_ids[num - 1], &rest[1..], &display_ids) {
                            println!("{}", e);
                        } else {
                            needs_redraw = true;
                        }
                    }
                } else {
                    println!("유효한 리스트 번호를 입력해주세요.");
                }
            }
            "remove" | "r" => {
                if rest.is_empty() {
                    println!("사용법: remove <리스트번호>[,...]");
                } else {
                    let nums = parse_numbers(rest[0]);
                    let mut valid_ids = Vec::new();
                    let mut invalid = false;
                    for n in &nums {
                        if *n == 0 || *n > display_ids.len() {
                            invalid = true;
                            break;
                        }
                        valid_ids.push(display_ids[n - 1]);
                    }
                    if invalid || valid_ids.is_empty() {
                        println!("유효한 리스트 번호를 입력해주세요.");
                    } else if let Err(e) = remove_todos(&conn, &valid_ids) {
                        println!("{}", e);
                    } else {
                        needs_redraw = true;
                    }
                }
            }
            "done" | "d" => {
                if rest.is_empty() {
                    println!("사용법: done <리스트번호>[,...] [0-5]");
                } else {
                    let nums = parse_numbers(rest[0]);
                    let level = if rest.len() > 1 { rest[1].parse().ok() } else { None };
                    let mut valid_ids = Vec::new();
                    let mut invalid = false;
                    for n in &nums {
                        if *n == 0 || *n > display_ids.len() {
                            invalid = true;
                            break;
                        }
                        valid_ids.push(display_ids[n - 1]);
                    }
                    if invalid || valid_ids.is_empty() {
                        println!("유효한 리스트 번호를 입력해주세요.");
                    } else if let Err(e) = set_done_batch(&conn, &valid_ids, level) {
                        println!("{}", e);
                    } else {
                        needs_redraw = true;
                    }
                }
            }
            "list" | "l" => {
                let (limit, page, range_start, range_end) = parse_list_range(&rest);
                match list_todos(&conn, show_done, use_order, limit, page, (range_start, range_end)) {
                    Ok(ids) => { display_ids = ids; }
                    Err(e) => println!("{}", e),
                }
            }
            "order" | "o" => {
                use_order = !use_order;
                display_ids = redraw(&conn, show_done, use_order);
            }
            "show" | "s" => {
                show_done = !show_done;
                display_ids = redraw(&conn, show_done, use_order);
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

        if needs_redraw {
            display_ids = redraw(&conn, show_done, use_order);
        }
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

fn print_help() {
    println!(r#"
Commands:
  add <내용> [-d 날짜] [-p 우선순위] [-u 리스트번호]  - TODO 추가
  edit <리스트번호> [-d 날짜] [-p 우선순위] [-u 리스트번호]  - TODO 수정
  remove <리스트번호>[,...]              - TODO 삭제 (2,3,5 또는 7-9)
  done <리스트번호>[,...] [0-5]          - 완료 상태 변경
  list (l) [n|/n/p/|-n]                - 리스트 보기
      n    : n개만 보기
      n/p  : n개씩 페이징, p번째 페이지
      n-m  : n~m번 보기
  order (o)                             - 순서 적용 토글
  show (s)                              - 완료 포함 토글
  help (h)                              - 도움말
  quit (q)                              - 종료
"#);
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    list_mode: bool,
}