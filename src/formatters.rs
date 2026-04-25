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
    /// Reset color to default
    pub const RESET: &str = "\x1b[0m";
    
    /// Bold reset
    pub const BOLD_RESET: &str = "\x1b[0;1m";
    
    // Priority colors (softer)
    /// Priority 1 - Soft orange
    pub const PRIORITY_1: &str = "\x1b[38;2;235;137;53m"; // #EB8935
    /// Priority 2 - Soft blue  
    pub const PRIORITY_2: &str = "\x1b[38;2;59;130;246m"; // #3B82F6
    /// Priority 3 - Soft green
    pub const PRIORITY_3: &str = "\x1b[38;2;52;211;153m"; // #34D399
    /// Priority 4 - Soft gray
    pub const PRIORITY_4: &str = "\x1b[38;2;156;163;175m"; // #9CA3AF
    
    // Due date colors (softer)
    /// Today/overdue - Soft red
    pub const DUE_TODAY: &str = "\x1b[38;2;239;68;68m";   // #EF4444
    /// This week - Soft yellow
    pub const DUE_WEEK: &str = "\x1b[38;2;251;191;36m";   // #FBBF24
    /// Future - Soft green
    pub const DUE_FUTURE: &str = "\x1b[38;2;34;197;94m";    // #22C55E
    
    // Done color
    /// Completed - Soft green
    pub const DONE: &str = "\x1b[38;2;52;211;153m";      // #34D399
}

/// Get color for priority level
/// 
/// # Arguments
/// * `priority` - Priority level (1-4)
/// 
/// # Returns
/// * ANSI color code
pub fn color_for_priority(priority: i32) -> &'static str {
    match priority {
        1 => colors::PRIORITY_1,
        2 => colors::PRIORITY_2,
        3 => colors::PRIORITY_3,
        4 => colors::PRIORITY_4,
        _ => colors::RESET,
    }
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

/// Get color for done status
/// 
/// # Arguments
/// * `done` - Done level (0-5)
/// 
/// # Returns
/// * ANSI color code (green if done=5)
pub fn color_for_done(done: i32) -> &'static str {
    if done == 5 {
        colors::DONE
    } else {
        colors::RESET
    }
}

/// Get current datetime as formatted string
/// 
/// Format: "YYYYMMDDTHHMMSS" (e.g., "20260315T143052")
/// Used for created_at and updated_at timestamps in database.
pub fn now_datetime() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%S").to_string()
}

/// Get current date as formatted string
/// 
/// Format: "YYYYMMDD" (e.g., "20260315")
/// Used for due_date and done_at in database.
pub fn now_date() -> String {
    Local::now().format("%Y%m%d").to_string()
}

/// Format database date string for display
/// 
/// Converts "YYYYMMDD" to "YY-MM-DD(Wday)" format.
/// 
/// # Arguments
/// * `date_str` - Date string in "YYYYMMDD" format
/// 
/// # Returns
/// * `Some("YY-MM-DD(Wday)")` if valid date
/// * `None` if invalid or too short
/// 
/// # Examples
/// 
/// ```rust
/// let formatted = format_date("20260315").unwrap();
/// assert_eq!(formatted, "26-03-15(일)");
/// ```
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
/// Supports multiple input formats:
/// - Day only: "15" -> current month, day 15
/// - Month/Day: "3/15" or "3-15" -> month/day
/// - Full: "2026/3/15" or "2026-3-15" or "26/3/15"
/// 
/// # Arguments
/// * `input` - Date string in various formats
/// 
/// # Returns
/// * `Some("YYYYMMDD")` if valid date input
/// * `None` if invalid or unparseable
/// 
/// # Examples
/// 
/// ```rust
/// assert_eq!(parse_date("15"), Some("20260315".to_string()));  // Today
/// assert_eq!(parse_date("3/15"), Some("20260315".to_string()));
/// assert_eq!(parse_date("26-3-15"), Some("20260315".to_string()));
/// ```
pub fn parse_date(input: &str) -> Option<String> {
    let today = Local::now().naive_local().date();
    let parts: Vec<&str> = input.split(|c| c == '/' || c == '-').collect();

    match parts.len() {
        // Day only: "15" means 15th of current month
        1 => {
            let day: u32 = parts[0].parse().ok()?;
            let date = NaiveDate::from_ymd_opt(today.year(), today.month(), day)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(today.year(), today.month() + 1, day).unwrap());
            Some(date.format("%Y%m%d").to_string())
        }
        // Month/Day: "3/15" or "3-15"
        2 => {
            let month: u32 = parts[0].parse().ok()?;
            let day: u32 = parts[1].parse().ok()?;
            let year = today.year();
            let date = NaiveDate::from_ymd_opt(year, month, day)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, month, day).unwrap());
            Some(date.format("%Y%m%d").to_string())
        }
        // Full: "2026/3/15" or "26/3/15"
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
  edit <list#> [-d date] [-p priority]              Edit TODO
  remove <list#>[,...]                            Remove TODO (1,2-5, etc)
  done <list#>[,...] [0-5]                        Set done level
  list (l) [n|/n/p/|-n]                         Show todos
      n    : show first n
      n/p  : n per page, page p
      n-m  : show n to m
  calendar (c) [m] [y]                         Show calendar
      no args: show 4 weeks from today
      m     : show month (e.g., 4)
      y/m   : show year/month (e.g., 25/3)
  order (o)                                     Toggle parent-child order
  show (s)                                      Toggle show completed
  help (h)                                      Show this help
  quit (q)                                      Quit
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

/// Calculate ISO week number for a given date
fn iso_week_number(year: i32, month: u32, day: u32) -> u32 {
    let date = match NaiveDate::from_ymd_opt(year, month, day) {
        Some(d) => d,
        None => return 1,
    };
    let iso = date.format("%V").to_string();
    iso.parse().unwrap_or(1)
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