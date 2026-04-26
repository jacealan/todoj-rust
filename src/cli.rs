//! CLI argument parsing and command utilities for todoj
//! 
//! This module handles all command-line interface related functionality:
//! - [Args] struct for parsing CLI arguments
//! - Command parsing (parse_command)
//! - Number parsing for list ranges (parse_numbers, parse_list_range)

use clap::Parser;

/// CLI arguments passed to the todoj application
/// 
/// Parsed from command line using clap derive macro.
/// # Examples
/// 
/// ```bash
/// todoj --db /path/to/db.sqlite  # Use custom database path
/// todoj -l                     # List mode (show todos and exit)
/// ```
#[derive(Parser)]
pub struct Args {
    /// List mode: show todos and exit immediately
    /// 
    /// When set, the app displays todos and exits without entering interactive mode.
    /// Useful for scripting or one-shot displays.
    #[arg(short, long)]
    pub list_mode: bool,
    
    /// Custom database path
    /// 
    /// Path to SQLite database file. If not specified, uses default location ~/.todoj.db
    #[arg(long)]
    pub db: Option<String>,
}

/// Parse a command line input into command and arguments
/// 
/// Splits input string by whitespace, treating first token as command,
/// remaining tokens as arguments.
/// 
/// # Arguments
/// * `input` - Raw input string from user
/// 
/// # Returns
/// * `Some((cmd, args))` if input is non-empty
/// * `None` if input is empty or only whitespace
/// 
/// # Examples
/// 
/// ```rust
/// let input = "add Buy milk -d 3/15 -p 2";
/// let (cmd, args) = parse_command(input).unwrap();
/// assert_eq!(cmd, "add");
/// assert_eq!(args, vec!["Buy", "milk", "-d", "3/15", "-p", "2"]);
/// ```
pub fn parse_command(input: &str) -> Option<(String, Vec<&str>)> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let cmd = normalize_cmd(parts[0]);
    let rest: Vec<&str> = parts[1..].to_vec();
    Some((cmd, rest))
}

/// Normalize command: convert Korean shortcuts to English
/// a=ㅁ, e=ㄷ, r=ㄱ, d=ㅇ, l=ㅣ, c=ㅊ, o=ㅐ, s=search, p=ㅔ, h=ㅗ, q=ㅂ
fn normalize_cmd(input: &str) -> String {
    let lower = input.to_lowercase();
    match lower.as_str() {
        "ㅁ" => "add".to_string(),
        "ㄷ" => "edit".to_string(),
        "ㄱ" => "remove".to_string(),
        "ㅇ" => "done".to_string(),
        "ㅣ" => "list".to_string(),
        "ㅊ" => "calendar".to_string(),
        "ㅐ" => "order".to_string(),
        "ㄴ" => "search".to_string(),
        "ㅔ" => "show".to_string(),
        "p" => "show".to_string(),
        "ㅗ" => "help".to_string(),
        "ㅂ" => "quit".to_string(),
        "search" => "search".to_string(),
        _ => input.to_string(),
    }
}

/// Parse number string into list of indices
/// 
/// Supports three formats:
/// - Single number: "5" -> [5]
/// - Comma-separated: "1,3,5" -> [1, 3, 5]
/// - Range: "2-5" -> [2, 3, 4, 5]
/// 
/// # Arguments
/// * `input` - Number string (e.g., "1,3-5,7")
/// 
/// # Returns
/// * Vec of 0-based indices
/// 
/// # Examples
/// 
/// ```rust
/// assert_eq!(parse_numbers("1"), vec![1]);
/// assert_eq!(parse_numbers("1,3,5"), vec![1, 3, 5]);
/// assert_eq!(parse_numbers("2-4"), vec![2, 3, 4]);
/// ```
pub fn parse_numbers(input: &str) -> Vec<usize> {
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

/// Parse list display options
/// 
/// Supports multiple ways to specify which items to display:
/// - Limit only: "10" -> show first 10 items
/// - Page: "10/2" -> 10 items per page, page 2
/// - Range: "3-5" -> show items 3 through 5
/// 
/// # Arguments
/// * `args` - Vector of string arguments
/// 
/// # Returns
/// * Tuple of (limit, page, range_start, range_end)
/// 
/// # Examples
/// 
/// ```rust
/// let args = vec!["10"];
/// let (limit, page, start, end) = parse_list_range(&args);
/// assert_eq!(limit, Some(10));
/// ```
pub fn parse_list_range(args: &[&str]) -> (Option<usize>, Option<usize>, Option<usize>, Option<usize>) {
    let mut limit = None;
    let mut page = None;
    let mut range_start = None;
    let mut range_end = None;

    for arg in args {
        // Page format: "10/2" means 10 items per page, page 2
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
        } 
        // Range format: "3-5" means items 3 through 5
        else if arg.contains('-') {
            let parts: Vec<&str> = arg.split('-').collect();
            if parts.len() == 2 {
                if let Ok(s) = parts[0].parse() {
                    range_start = Some(s);
                }
                if let Ok(e) = parts[1].parse() {
                    range_end = Some(e);
                }
            }
        } 
        // Simple limit
        else if let Ok(n) = arg.parse::<usize>() {
            limit = Some(n);
        }
    }

    (limit, page, range_start, range_end)
}