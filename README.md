# todoj - Terminal TODO Manager

A simple yet powerful terminal-based TODO manager written in Rust.

## Features

- **CRUD Operations**: Add, edit, remove, and complete todos
- **Clone Todos**: Clone existing todos with `more` command
- **Recurring Todos**: Daily, every other day, weekly, monthly, yearly repetition with `-r` flag
- **Due Date Tracking**: Flexible input formats (3/15, 2026/3/15, @today, @tom, etc.)
- **Priority Levels**: 1 (highest) to 4 (lowest), default 3
- **Sub-todos**: Parent-child relationships with `-u` flag or inline format
- **Progress Tracking**: 0%, 20%, 40%, 60%, 80%, 100%
- **Calendar View**: 4 weeks or specific month display
- **Order Mode**: Parent-child hierarchy display
- **Completed Filtering**: Show/hide completed todos
- **Search**: Search todos by keyword, includes completed
- **Multiple Databases**: Use different DB files for work/personal
- **Local SQLite Storage**: Zero-configuration local storage
- **Color Display**: Priority and due dates shown with colors

## Installation

### From Source

```bash
# Clone and build
git clone https://github.com/yourusername/todoj.git
cd todoj
cargo build --release

# Install to PATH
cargo install --path .
```

## Usage

### Interactive Mode

```bash
$ todoj
=== TODO ===
1 [ ]  Buy groceries  @26-03-20 ^3
2 [ ]  Walk dog  @26-03-21 ^2

> add Finish report -d 3/25 -p 1
추가되었습니다.

> done 1 5
완료 상태가 변경되었습니다.

> quit
```

### List Mode (non-interactive)

```bash
# Show all incomplete todos and exit
todoj -l

# Show with completed todos
todoj -l --show
```

## Commands

### add (a) - Add new todo

```bash
# Basic
add Buy milk

# With due date and priority (using flags)
add Finish report -d 3/25 -p 1

# With due date and priority (inline format)
add Finish report @3/25 ^1

# Using keywords (today, tomorrow, weekday)
add Task @today       # Today
add Task @tom        # Tomorrow  
add Task @mon        # Next Monday
add Task @ tue      # Next Tuesday
add Task @ fri      # Next Friday

# As sub-task under todo #3
add Sub-task -u 3

# With repetition (recurring todos)
add Daily task -r d           # Every day
add Every other day -r e      # Every 2 days
add Weekly meeting -r w      # Every week (same weekday)
add Monthly bill -r 15        # Every month on day 15
add Birthday -r 2/3          # Every year on Feb 3

# Mixed format
add Task @today -p 2     # @today as date, -p for priority
add Task -d 3/25 ^1     # -d for date, ^1 inline
```

**Options:**
- `-d DATE`: Due date (format: d, m/d, y/m/d, or keywords: today, tom, mon, tue, wed, thu, fri, sat, sun)
- `-p PRIORITY`: Priority 1-4 (default 3)
- `-u N`: Parent todo number (creates sub-task)
- `-r PERIOD`: Repetition period:
  - `d` = daily (every day)
  - `e` = every other day (every 2 days)
  - `w` = weekly (same weekday)
  - `N` = monthly on day N (1-31, adjusts for shorter months)
  - `M/D` or `M-D` = yearly on month M, day D

**Inline format (at end of content):**
- `@DATE` - Due date (e.g., `@3/15`, `@today`, `@mon`)
- `^PRIORITY` - Priority 1-4 (e.g., `^1`, `^2`)

### edit (e) - Edit todo

```bash
# Batch edit: change due date and priority (using flags)
edit 1,2 -d 3/30 -p 2

# Batch edit: change using inline format
edit 1,2 @3/30 ^2

# Batch edit: change parent (sub-task)
edit 1 -u 3

# Batch edit: change content only
edit 1,2 New content here

# Interactive edit: prompts for new content
edit 1
수정: Updated content @3/15 ^1
```

**Options:**
- `-d DATE`: New due date
- `-p PRIORITY`: New priority (1-4)
- `-u N`: Change parent todo number
- `-r PERIOD`: Change repetition period (same as add)
- `-r 0`: Clear repetition (remove recurring)

**Inline format (for batch edit):**
- `@DATE` - Due date
- `^PRIORITY` - Priority 1-4

### remove (r) - Delete todo

```bash
# Single
remove 1

# Multiple (comma-separated)
remove 1,3,5

# Range
remove 2-5
```

### done (d) - Mark done

```bash
# Toggle complete/incomplete
done 1

# Set specific level
done 1 5     # Complete
done 1 3     # 60% done
done 1 0     # Not done

# Multiple
done 1,2,3 5
```

**Done Levels:**
| Level | Meaning |
|-------|----------|
| 0 | Not started |
| 1 | 20% done |
| 2 | 40% done |
| 3 | 60% done |
| 4 | 80% done |
| 5 | Complete |

**Recurring Todos:**
When a recurring todo (with `-r` flag) is marked as complete (done=5):
- A new todo is automatically created with the same content, priority, and parent
- The new todo's due date is calculated based on the repetition period:
  - Daily (`-r d`): Next day
  - Every other day (`-r e`): 2 days later
  - Weekly (`-r w`): Same weekday next week
  - Monthly (`-r N`): Same day next month (adjusts for shorter months)
  - Yearly (`-r M/D`): Same date next year
- If original todo has no due date, uses completion date as base

### list (l) - Display todos

```bash
# Show first 10
list 10

# Pagination: 10 per page, page 2
list 10/2

# Show range: items 3-5
list 3-5
```

### calendar (c) - Show calendar

```bash
# Show 4 weeks from today (default)
calendar

# Show specific month (current year)
calendar 4
calendar 4

# Show specific month/year
calendar 4/2026
calendar 25/3     # year 2025, month 3
```

**Calendar Display:**
- Without arguments: Shows 4 weeks starting from today's week
- Header shows month range: "2026/4 - 2026/5"
- Today marked with ◉ before the date

Example:
```
> calendar
2026/4 - 2026/5
Sun Mon Tue Wed Thu Fri Sat
 19  20  21  22  23  24 ◉25 
 26  27  28  29  30   1   2 
  3   4   5   6   7   8   9 
 10  11  12  13  14  15 16 
```

### order (o) - Toggle order mode

Shows sub-todos under their parent todos:
```
1 [ ] Parent task     ^3
   [ ] Sub-task 1   ^3
   [ ] Sub-task 2   ^3
2 [ ] Another       ^2
```

Deleted parent shows as `x>` (gray):
```
1 [x] Completed    ^3
   [ ] Sub-task   ^3     # parent is deleted
```

### search (s) - Search todos

Search for todos containing keyword:
```bash
search test              # Search for "test"
s test                 # Same, using shortcut
ㄴ test                # Same, using Korean shortcut
```

Shows all matching todos including completed. Results displayed 10 at a time:
- Press Enter to show next 10
- Press q to end search

### more (m) - Clone todo

Clone existing todo(s) with same content, priority, parent:
```bash
# Clone single todo
more 1

# Clone multiple
more 1,3,5
more 1-5

# With date/priority override
more 1 -d 3/15 -p 2
more 1 @3/15 ^2      # Inline format

# Change parent when cloning
more 1 -u 3         # Make sub-task of todo #3
more 1 -u 0         # Make top-level (remove parent)
```

### past (p) - Toggle showing completed

Shows completed (done=5) todos at the bottom.

### help (h) - Show help

### quit (q) - Exit application

## Shortcuts

| Shortcut | Command | English |
|----------|---------|---------|
| ㅁ | add | a |
| ㄷ | edit | e |
| ㄱ | remove | r |
| ㅇ | done | d |
| ㅣ | list | l |
| ㅊ | calendar | c |
| ㅐ | order | o |
| ㄴ | search | s |
| p | past | p |
| ㅗ | help | h |
| ㅂ | quit | q |
| m,ㅡ | more | - |

## Date Formats

Due date supports flexible input:

| Format | Example | Output |
|--------|--------|--------|
| Day only | 15 | Nearest future date with day 15 |
| Month/Day | 3/15 or 3-15 | 2026-03-15 (this year) |
| Year/Month/Day | 26/3/15 | 2026-03-15 |
| Full | 2026/3/15 | 2026-03-15 |
| Keywords | @today, @tom, @mon | Today, Tomorrow, Next Monday |
| Korean | @오늘, @내일, @월 | Today, Tomorrow, Next Monday |

**Day only behavior:**
- `add Task @15` when today is 4/26 → sets to 5/1 (nearest future 15th)
- Finds the closest future date with that day number

**Invalid date handling:**
- `@0` → "Invalid date"
- `@33` → "33일은 없습니다. 1~31 사이 날짜"

## Database

### Default Location

`~/.todoj.db`

### Custom Database

```bash
# Use different database file
todoj ~/icloud/todo.db     # Cloud sync from iCloud Drive
todoj ./work.db          # Work todos
todoj ~/personal.db    # Personal todos
```

You can have multiple databases for different purposes (work, personal, etc.).

### Schema

```sql
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    todo TEXT NOT NULL,
    due_date TEXT,
    priority INTEGER DEFAULT 3,
    up_id INTEGER,
    done INTEGER DEFAULT 0,
    done_at TEXT,
    deleted_at TEXT,
    repetition_period TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## Display Format

```
1 [ ] Parent task     @26-03-20 ^3
2 [ ] 1> Sub-task   @26-03-21 ^3
3 [x] Completed    %26-03-19 ^2
```

**Format:**
- `N` - Line number (padded)
- `[ ]` / `[x]` - Incomplete / Complete
- `N>` - Parent reference (gray)
- `@YY-MM-DD` - Due date
- `%XX` - Progress percentage
- `%YY-MM-DD` - Completion date
- `^N` - Priority (1=high, 4=low)
- `*X` - Repetition period:
  - `*D` = daily
  - `*E` = every other day
  - `*Mon` = weekly (shows weekday)
  - `*15` = monthly on day 15
  - `*2/3` = yearly on Feb 3

## Color Legend

### Due Date Colors (@)
| Color | Meaning |
|-------|----------|
| Red | Today or overdue |
| Yellow | Within 7 days |
| Green | Future |

### Priority Colors (^)
| Color | Priority |
|-------|----------|
| Orange | ^1 (highest) |
| Blue | ^2 |
| Green | ^3 (default) |
| Gray | ^4 (lowest) |

## Roadmap

- [x] Clone todos (`more` command)
- [x] Multiple databases
- [x] Recurring todos (`-r` flag)
- [ ] PostgreSQL support for cross-device sync
- [ ] Web interface
- [ ] Tags/categories

## License

MIT License

## Contributing

Pull requests welcome!