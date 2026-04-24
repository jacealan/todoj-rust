//! Formatting utilities for todoj
//! 
//! This module provides common formatting and utility functions:
//! - Date formatting (format_date, parse_date)
//! - Screen utilities (clear_screen)
//! - Help display (print_help)

use chrono::{Datelike, Local, NaiveDate};

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
/// Shown when user types "help", "h", or "?".
pub fn print_help() {
    println!(
        r#"
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
"#
    );
}