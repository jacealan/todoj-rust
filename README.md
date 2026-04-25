# todoj - Terminal TODO Manager

A simple yet powerful terminal-based TODO manager written in Rust.

## Features

- **CRUD Operations**: Add, edit, remove, and complete todos
- **Due Date Tracking**: Flexible input formats (3/15, 2026/3/15, @today, @tom, etc.)
- **Priority Levels**: 1 (highest) to 4 (lowest), default 3
- **Sub-todos**: Parent-child relationships with `-u` flag or inline format
- **Progress Tracking**: 0%, 20%, 40%, 60%, 80%, 100%
- **Calendar View**: 4 weeks or specific month display
- **Flexible Display**: Order by due date, priority, or creation time
- **Completed Filtering**: Show/hide completed todos
- **Local SQLite Storage**: Zero-configuration local storage
- **Cross-device Sync**: PostgreSQL support planned (future)
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

# Mixed format
add Task @today -p 2     # @today as date, -p for priority
add Task -d 3/25 ^1     # -d for date, ^1 inline
```

**Options:**
- `-d DATE`: Due date (format: d, m/d, y/m/d, or keywords: today, tom, mon, tue, wed, thu, fri, sat, sun)
- `-p PRIORITY`: Priority 1-4 (default 3)
- `-u N`: Parent todo number (creates sub-task)

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
|-------|---------|
| 0 | Not started |
| 1 | 20% done |
| 2 | 40% done |
| 3 | 60% done |
| 4 | 80% done |
| 5 | Complete |

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

### show (s) - Toggle showing completed

Shows completed (done=5) todos at the bottom.

### help (h) - Show help

### quit (q) - Exit application

## Date Formats

Due date supports flexible input:

| Format | Example | Output |
|--------|--------|--------|
| Day | 15 | 2026-03-15 (this month) |
| Month/Day | 3/15 or 3-15 | 2026-03-15 (this year) |
| Year/Month/Day | 26/3/15 | 2026-03-15 |
| Full | 2026/3/15 | 2026-03-15 |
| Keywords | @today, @tom, @mon | Today, Tomorrow, Next Monday |

## Database

### Location

Default: `~/.todoj.db`

### Custom Database

```bash
todoj --db /path/to/custom.db
```

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

- [ ] PostgreSQL support for cross-device sync
- [ ] Web interface
- [ ] Mobile app
- [ ] Tags/categories
- [ ] Recurring todos

## License

MIT License

## Contributing

Pull requests welcome!