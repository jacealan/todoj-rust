//! Formatting utilities for todoj
//! 
//! This module provides common formatting and utility functions:
//! - Date formatting (format_date, parse_date)
//! - Color support (colors for priority and due dates)
//! - Calendar display (show_calendar)
//! - Screen utilities (clear_screen)
//! - Help display (print_help)

use chrono::{Datelike, Local, NaiveDate};

/// ANSI color codes for terminal output
pub mod colors {
    /// Due date colors (softer)
    /// Today/overdue - Soft red
    pub const DUE_TODAY: &str = "\x1b[38;2;239;68;68m";   // #EF4444
    /// This week - Soft yellow
    pub const DUE_WEEK: &str = "\x1b[38;2;251;191;36m";   // #FBBF24
    /// Future - Soft green
    pub const DUE_FUTURE: &str = "\x1b[38;2;34;197;94m";    // #22C55E
}

/// Get color for due date
/// 
/// Compares due date with today to determine color:
/// - Today or past: red (overdue)
/// - This week (within 7 days): yellow
/// - Future: no color
/// 
/// # Arguments
/// * `due_date` - Due date in "YYYYMMDD" format
/// 
/// # Returns
/// * ANSI color code
pub fn color_for_due_date(due_date: &str) -> &'static str {
    if let (Ok(year), Ok(month), Ok(day)) = (
        due_date[..4].parse(),
        due_date[4..6].parse(),
        due_date[6..8].parse(),
    ) {
        if let Some(due) = NaiveDate::from_ymd_opt(year, month, day) {
            let today = Local::now().naive_local().date();
            let days_until = (due - today).num_days();
            
            if days_until <= 0 {
                colors::DUE_TODAY  // Today or overdue
            } else if days_until <= 7 {
                colors::DUE_WEEK   // Within 7 days
            } else {
                colors::DUE_FUTURE // Future
            }
        } else {
            colors::DUE_FUTURE
        }
    } else {
        colors::DUE_FUTURE
    }
}

/// Highlight keyword in text with blue color
pub fn highlight_keyword(text: &str, keyword: &str) -> String {
    if keyword.is_empty() {
        return text.to_string();
    }
    let lower_text = text.to_lowercase();
    let lower_keyword = keyword.to_lowercase();
    
    if let Some(pos) = lower_text.find(&lower_keyword) {
        let before = &text[..pos];
        let match_text = &text[pos..pos + keyword.len()];
        let after = &text[pos + keyword.len()..];
        format!("{}\x1b[38;2;59;130;246m{}\x1b[0m{}", before, match_text, after)
    } else {
        text.to_string()
    }
}

/// Get current date and time for prompt
/// 
/// Format: "[YY-MM-DD HH:MM]"
pub fn now_prompt() -> String {
    Local::now().format("[%y-%m-%d(%a) %H:%M]").to_string()
}

/// Format database date string for display
/// 
/// Converts "YYYYMMDD" to "YY-MM-DD(Wday)" format.
pub fn format_date(date_str: &str) -> Option<String> {
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

/// Parse user date input into database format
/// 
/// Supports: day-only (15), month/day (3/15), full (26/3/15), keywords (@today, @tom, @월, @내일, etc.)
pub fn parse_date(input: &str) -> Option<String> {
    let today = Local::now().naive_local().date();
    let today_wd = today.weekday();
    
    // Helper to convert weekday to number (Mon=1, Tue=2, ..., Sun=7)
    let weekday_to_num = |wd: chrono::Weekday| -> u32 {
        match wd {
            chrono::Weekday::Mon => 1,
            chrono::Weekday::Tue => 2,
            chrono::Weekday::Wed => 3,
            chrono::Weekday::Thu => 4,
            chrono::Weekday::Fri => 5,
            chrono::Weekday::Sat => 6,
            chrono::Weekday::Sun => 7,
        }
    };
    
    // Handle special keywords (English and Korean)
    let lower = input.to_lowercase();
    
    // English keywords
    if lower == "today" {
        return Some(today.format("%Y%m%d").to_string());
    }
    if lower == "tomorrow" || lower == "tom" {
        return Some((today + chrono::Duration::days(1)).format("%Y%m%d").to_string());
    }
    
    // Korean keywords
    if lower == "오늘" || lower == "jntn" {
        return Some(today.format("%Y%m%d").to_string());
    }
    if lower == "내일" || lower == "tkfq" {
        return Some((today + chrono::Duration::days(1)).format("%Y%m%d").to_string());
    }
    
    // Weekday keywords mapping
    let weekday_data: &[(&str, chrono::Weekday)] = &[
        ("월", chrono::Weekday::Mon),
        ("화", chrono::Weekday::Tue),
        ("수", chrono::Weekday::Wed),
        ("목", chrono::Weekday::Thu),
        ("금", chrono::Weekday::Fri),
        ("토", chrono::Weekday::Sat),
        ("일", chrono::Weekday::Sun),
        ("mon", chrono::Weekday::Mon),
        ("tue", chrono::Weekday::Tue),
        ("wed", chrono::Weekday::Wed),
        ("thu", chrono::Weekday::Thu),
        ("fri", chrono::Weekday::Fri),
        ("sat", chrono::Weekday::Sat),
        ("sun", chrono::Weekday::Sun),
    ];
    
    for (keyword, target_wd) in weekday_data {
        if lower == *keyword {
            let today_num = weekday_to_num(today_wd);
            let target_num = weekday_to_num(*target_wd);
            let days_until = if target_num > today_num {
                (target_num - today_num) as i64
            } else {
                (7 - today_num + target_num) as i64
            };
            return Some((today + chrono::Duration::days(days_until)).format("%Y%m%d").to_string());
        }
    }
    
    // Weekday keywords: find next occurrence of that day
    let weekday_to_num = |wd: chrono::Weekday| -> u32 {
        match wd {
            chrono::Weekday::Sun => 0,
            chrono::Weekday::Mon => 1,
            chrono::Weekday::Tue => 2,
            chrono::Weekday::Wed => 3,
            chrono::Weekday::Thu => 4,
            chrono::Weekday::Fri => 5,
            chrono::Weekday::Sat => 6,
        }
    };
    
    let weekday_map: &[(&str, chrono::Weekday)] = &[
        ("sun", chrono::Weekday::Sun),
        ("mon", chrono::Weekday::Mon),
        ("tue", chrono::Weekday::Tue),
        ("wed", chrono::Weekday::Wed),
        ("thu", chrono::Weekday::Thu),
        ("fri", chrono::Weekday::Fri),
        ("sat", chrono::Weekday::Sat),
    ];
    
    for (keyword, target_day) in weekday_map {
        if lower == *keyword || lower == format!("{}s", keyword) {
            let today_wd = today.weekday();
            let today_num = weekday_to_num(today_wd);
            let target_num = weekday_to_num(*target_day);
            let days_until = if target_num == today_num {
                7 // Next week if today is the same day
            } else if target_num > today_num {
                (target_num - today_num) as i64
            } else {
                (7 - today_num + target_num) as i64
            };
            return Some((today + chrono::Duration::days(days_until)).format("%Y%m%d").to_string());
        }
    }
    
    let parts: Vec<&str> = input.split(|c| c == '/' || c == '-').collect();

    match parts.len() {
        // Day only: find nearest future date with that day
        1 => {
            let day: u32 = parts[0].parse().ok()?;
            if day < 1 || day > 31 {
                return None;
            }
            // Search for nearest future date with that day
            let mut check_date = today;
            for _ in 0..365 {
                check_date = check_date + chrono::Duration::days(1);
                if check_date.day() as u32 == day {
                    return Some(check_date.format("%Y%m%d").to_string());
                }
            }
            None
        }
        // Month/Day: "3/15" or "3-15" - use next year if past
        2 => {
            let month: u32 = parts[0].parse().ok()?;
            let day: u32 = parts[1].parse().ok()?;
            if month < 1 || month > 12 {
                return None;
            }
            if day < 1 || day > 31 {
                return None;
            }
            if day > days_in_month(month, today.year()) {
                return None;
            }
            let year = today.year();
            let date = NaiveDate::from_ymd_opt(year, month, day);
            if date.is_some() {
                return date.map(|d| d.format("%Y%m%d").to_string());
            }
            // Next year if month/day already passed
            if day <= days_in_month(month, year + 1) {
                return Some(NaiveDate::from_ymd_opt(year + 1, month, day).unwrap().format("%Y%m%d").to_string());
            }
            None
        }
        // Full: "2026/3/15" or "26/3/15"
        3 => {
            let year: i32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            let day: u32 = parts[2].parse().ok()?;
            if month < 1 || month > 12 {
                return None;
            }
            if day < 1 || day > 31 {
                return None;
            }
            let full_year = if year < 100 { 2000 + year } else { year };
            if day > days_in_month(month, full_year) {
                return None;
            }
            let date = NaiveDate::from_ymd_opt(full_year, month, day)?;
            Some(date.format("%Y%m%d").to_string())
        }
        _ => None
    }
}

/// Get number of days in a month
fn days_in_month(month: u32, year: i32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30
    }
}

/// Clear terminal screen
/// 
/// Uses ANSI escape sequences to clear screen and move cursor to top-left.
/// Works on most modern terminals.
pub fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

/// Print help message
/// 
/// Displays all available commands with usage examples.
/// Print help message
/// 
/// Displays all available commands with usage examples.
/// Shown when user types "help", "h", or "?".
pub fn print_help() {
    println!(
        r#"
Commands:
  add <content> [-d date] [-p priority] [-u list#]   Add TODO
      @DATE : inline date (e.g., @3/15, @today, @tom)
      ^N   : inline priority (e.g., ^1, ^4)

  edit <list#> [-d date] [-p priority]              Edit TODO
      @DATE : inline date, ^N : inline priority

  remove <list#>[,...]                            Remove TODO (1,2-5, etc)

  done <list#>[,...] [0-5]                        Set done level

  list (l) [n|/n/p/|-n]                         Show todos
      n    : show first n
      n/p  : n per page, page p
      n-m  : show n to m

  search <keyword>                                Search todos
      s ㄴ search word

  calendar (c) [m] [y]                         Show calendar
      no args: show 4 weeks from today
      m     : show month (e.g., 4)
      y/m   : show year/month (e.g., 25/3)

  order (o)                                     Toggle parent-child order
  past (p)                                     Toggle show completed
  help (h)                                      Show this help
  quit (q)                                      Quit

Date: d, m/d, y/m/d (e.g., 15, 3/15, 26/3/15)
Keywords: @today, @tom (tomorrow), @mon/@tue/@wed/@thu/@fri/@sat/@sun
 Korean: @오늘, @내일, @월/@화/@수/@목/@금/@토/@일
Priority: 1 (highest) to 4 (lowest), default 3
Done levels: 0 (not started) to 5 (complete)
Shortcuts: a=ㅁ e=ㄷ r=ㄱ d=ㅇ l=ㅣ c=ㅊ o=ㅐ s=ㄴ p=ㅔ h=ㅗ q=ㅂ
"#
    );
}

/// Show calendar for a specific month and year
/// 
/// Displays a month calendar with Sunday as first day of week.
/// Current day highlighted with ◉.
/// 
/// # Arguments
/// * `year` - Year (4 digits)
/// * `month` - Month (1-12)
pub fn show_calendar(year: i32, month: u32) {
    let first_day = match NaiveDate::from_ymd_opt(year, month, 1) {
        Some(d) => d,
        None => {
            println!("잘못된 날짜입니다.");
            return;
        }
    };
    
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap().pred_opt().unwrap().day()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap().pred_opt().unwrap().day()
    };
    
    let start_weekday = first_day.weekday().num_days_from_sunday() as usize;
    
    let today = Local::now().naive_local().date();
    let is_current_month = today.year() == year && today.month() == month;
    let today_day = if is_current_month { today.day() as u32 } else { 0 };
    
    let month_name = format!("{}/{}", year, month);
    
    println!("\n{}", month_name);
    println!("Sun Mon Tue Wed Thu Fri Sat");
    
    for _ in 0..start_weekday {
        print!("    ");
    }
    
    let mut day = 1;
    let mut pos = start_weekday;
    while day <= last_day {
        let cell = if is_current_month && day == today_day {
            format!("◉{:>2}", day)
        } else {
            format!("{:>3}", day)
        };
        print!("{} ", cell);
        
        day += 1;
        pos += 1;
        
        if pos > 6 {
            println!();
            pos = 0;
        }
    }
    
    if pos != 0 {
        println!();
    }
}

/// Calculate ISO week number for a given date
/// 
/// Displays 4 weeks (28 days) starting from the Sunday of current week.
/// Header shows month range: "2026/4 - 2026/5"
/// Current day highlighted with ◉ before the date.
/// 
pub fn show_calendar_weeks() {
    let today = Local::now().naive_local().date();
    
    // Find Sunday of this week
    let today_weekday = today.weekday().num_days_from_sunday() as i32;
    let sunday = today - chrono::Duration::days(today_weekday as i64);
    
    // Collect dates for 4 weeks
    let mut dates: Vec<NaiveDate> = Vec::new();
    let mut current_date = sunday;
    for _ in 0..28 {
        dates.push(current_date);
        current_date = current_date + chrono::Duration::days(1);
    }
    
    // Find first and last month
    let first_month = dates[0].month();
    let last_month = dates[27].month();
    
// Print header with month range
    if first_month == last_month {
        println!("\n2026/{}", first_month);
    } else {
        println!("\n2026/{} - 2026/{}", first_month, last_month);
    }
    println!("Sun Mon Tue Wed Thu Fri Sat");
    
    // Print each day with ◉ before today
    for (i, d) in dates.iter().enumerate() {
        let day = d.day() as u32;
        let is_today = *d == today;
        
        let cell = if is_today {
            format!("◉{:>2}", day)  // ◉ + 2 digits = 3 chars
        } else {
            format!("{:>3}", day)   // space + 2 digits = 3 chars
        };
        print!("{} ", cell);  // adds space to make 4 chars total
        
        // New line after Saturday
        if (i + 1) % 7 == 0 {
            println!();
        }
    }
    
    if dates.len() % 7 != 0 {
        println!();
    }
}

/// Parse calendar arguments to year and month
/// 
/// Returns a tuple (year, month) for single month display.
/// If no args, returns (year, month) of today's week start (for 4-week view).
/// 
/// # Arguments
/// * `args` - Vector of string arguments
/// 
/// # Returns
/// * `Some((year, month))` if valid
/// * `None` if invalid
/// 
/// # Examples
/// 
/// ```rust
/// parse_calendar_args(&[]);           // (2026, 4) - this week's month
/// parse_calendar_args(&["4"]);        // (2026, 4)
/// parse_calendar_args(&["25/3"]);    // (2025, 3)
/// parse_calendar_args(&["3", "2026"]);  // (2026, 3)
/// ```
pub fn parse_calendar_args(args: &[&str]) -> Option<(i32, u32)> {
    let today = Local::now().naive_local().date();
    let current_year = today.year();
    let current_month = today.month();
    
    match args.len() {
        0 => Some((current_year, current_month)),
        1 => {
            let parts: Vec<&str> = args[0].split(|c| c == '/' || c == '-').collect();
            match parts.len() {
                1 => {
                    let month: u32 = parts[0].parse().ok()?;
                    if month >= 1 && month <= 12 {
                        Some((current_year, month))
                    } else {
                        None
                    }
                }
                2 => {
                    let part0: i32 = parts[0].parse().ok()?;
                    let part1: i32 = parts[1].parse().ok()?;
                    
                    if part0 <= 12 {
                        let year = if part1 < 100 { 2000 + part1 } else { part1 };
                        Some((year, part0 as u32))
                    } else {
                        let year = if part0 < 100 { 2000 + part0 } else { part0 };
                        Some((year, part1 as u32))
                    }
                }
                _ => None,
            }
        }
        2 => {
            let month: u32 = args[0].parse().ok()?;
            let year: i32 = args[1].parse().ok()?;
            let full_year = if year < 100 { 2000 + year } else { year };
            if month >= 1 && month <= 12 {
                Some((full_year, month))
            } else {
                None
            }
        }
        _ => None,
    }
}