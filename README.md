# todoj - Terminal TODO Manager

A simple yet powerful terminal-based TODO manager written in Rust.

## Features

- **CRUD Operations**: Add, edit, remove, and complete todos
- **Due Date Tracking**: Flexible input formats (3/15, 2026/3/15, etc.)
- **Priority Levels**: 1 (highest) to 4 (lowest), default 3
- **Sub-todos**: Parent-child relationships with `-u` flag
- **Progress Tracking**: 0%, 20%, 40%, 60%, 80%, 100%
- **Flexible Display**: Order by due date, priority, or creation time
- **Completed Filtering**: Show/hide completed todos
- **Local SQLite Storage**: Zero-configuration local storage
- **Cross-device Sync**: PostgreSQL support planned (future)

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

### Pre-built (future)

```bash
# Not yet available
```

## Usage

### Interactive Mode

```bash
$ todoj
=== TODO ===
1 [ ]  Buy groceries  @26-03-20(금) ^3
2 [ ]  Walk dog  @26-03-21(토) ^2

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

# With due date and priority
add Finish report -d 3/25 -p 1

# As sub-task under todo #3
add Sub-task -u 3
```

**Options:**
- `-d DATE`: Due date (format: d, m/d, y/m/d)
- `-p PRIORITY`: Priority 1-4 (default 3)
- `-u N`: Parent todo number (creates sub-task)

### edit (e) - Edit todo

```bash
# Batch edit: change due date and priority
edit 1,2 -d 3/30 -p 2

# Interactive edit: prompts for new content
edit 1
수정: Updated content here
```

**Options:**
- `-d DATE`: New due date
- `-p PRIORITY`: New priority

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
1 [ ] Parent task     @26-03-20(금) ^3
2 [ ] 1> Sub-task   @26-03-21(토) ^3
3 [x] Completed    %26-03-19(수) ^2   # Shows completion date
```

**Format:**
- `N` - Line number (padded)
- `[ ]` / `[x]` - Incomplete / Complete
- `N>` - Parent reference
- `@YY-MM-DD(W)` - Due date with weekday
- `%XX` - Progress percentage
- `%YY-MM-DD` - Completion date
- `^N` - Priority

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection (future) | - |

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