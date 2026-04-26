//! SQLite repository implementation for todoj
//! 
//! Implements [TodoRepository] for local SQLite database.
//! Uses rusqlite with bundled SQLite for zero-configuration local storage.
//! 
//! # Database Location
//! 
//! Default: ~/.todoj.db
//! Custom: --db /path/to/db option
//! 
//! # Schema
//! 
//! ```sql
//! CREATE TABLE todos (
//!     id INTEGER PRIMARY KEY AUTOINCREMENT,
//!     todo TEXT NOT NULL,
//!     due_date TEXT,
//!     priority INTEGER DEFAULT 3,
//!     up_id INTEGER,
//!     done INTEGER DEFAULT 0,
//!     done_at TEXT,
//!     deleted_at TEXT,
//!     created_at TEXT NOT NULL,
//!     updated_at TEXT NOT NULL
//! );
//! ```

use chrono::{Local, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use super::{NewTodo, Todo, TodoRepository, UpdateTodo};

/// SQLite repository
/// 
/// Local file-based SQLite database.
/// Thread-safe through per-operation connection opening.
pub struct SqliteRepo {
    /// Path to SQLite database file
    path: PathBuf,
}

impl SqliteRepo {
    /// Create new SQLite repository
    /// 
    /// # Arguments
    /// * `path` - Path to SQLite database file
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

/// PostgreSQL placeholder for future implementation
/// 
/// Will be implemented with async PostgreSQL driver
/// when web API layer is added.
pub struct PostgresRepo;

impl PostgresRepo {
    /// Create placeholder PostgreSQL repository
    /// 
    /// Not yet implemented - requires async runtime.
    pub fn new(_connection_string: String) -> Self {
        Self
    }
}

impl TodoRepository for SqliteRepo {
    /// Initialize SQLite database
    /// 
    /// Creates todos table if not exists.
    fn init(&self) -> Result<(), String> {
        Self::with_conn(self, |conn| {
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
            ).map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    /// Create new todo in SQLite
    fn create(&self, new_todo: NewTodo) -> Result<Todo, String> {
        Self::with_conn(self, |conn| {
            let created = Utc::now().format("%Y%m%dT%H%M%S").to_string();
            let priority = new_todo.priority.unwrap_or(3);

            conn.execute(
                "INSERT INTO todos (todo, due_date, priority, up_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![new_todo.todo, new_todo.due_date, priority, new_todo.up_id, created, created],
            ).map_err(|e| e.to_string())?;

            let id = conn.last_insert_rowid();
            self.find_by_id(id)?
                .ok_or_else(|| "Failed to create todo".to_string())
        })
    }

    /// Find todo by ID
    fn find_by_id(&self, id: i64) -> Result<Option<Todo>, String> {
        Self::with_conn(self, |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, todo, due_date, priority, up_id, done, done_at, deleted_at, created_at, updated_at 
                 FROM todos WHERE id = ?1"
            ).map_err(|e| e.to_string())?;

            let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
            
            if let Some(row) = rows.next().map_err(|e| e.to_string())? {
                Ok(Some(Todo {
                    id: row.get(0).map_err(|e| e.to_string())?,
                    todo: row.get(1).map_err(|e| e.to_string())?,
                    due_date: row.get(2).map_err(|e| e.to_string())?,
                    priority: row.get(3).map_err(|e| e.to_string())?,
                    up_id: row.get(4).map_err(|e| e.to_string())?,
                    done: row.get(5).map_err(|e| e.to_string())?,
                    done_at: row.get(6).map_err(|e| e.to_string())?,
                    deleted_at: row.get(7).map_err(|e| e.to_string())?,
                    created_at: row.get(8).map_err(|e| e.to_string())?,
                    updated_at: row.get(9).map_err(|e| e.to_string())?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    /// Find all non-deleted todos
    /// 
    /// Sorts by:
    /// 1. Due date (nulls last)
    /// 2. Priority (ascending, 1 first)
    /// 3. Created at (descending)
    fn find_all(&self, include_done: bool) -> Result<Vec<Todo>, String> {
        Self::with_conn(self, |conn| {
            let sql = if include_done {
                "SELECT id, todo, due_date, priority, up_id, done, done_at, deleted_at, created_at, updated_at 
                 FROM todos WHERE deleted_at IS NULL
                 ORDER BY CASE WHEN due_date IS NULL THEN 1 ELSE 0 END, due_date ASC, priority ASC, created_at DESC"
            } else {
                "SELECT id, todo, due_date, priority, up_id, done, done_at, deleted_at, created_at, updated_at 
                 FROM todos WHERE deleted_at IS NULL AND done != 5
                 ORDER BY CASE WHEN due_date IS NULL THEN 1 ELSE 0 END, due_date ASC, priority ASC, created_at DESC"
            };

            let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    todo: row.get(1)?,
                    due_date: row.get(2)?,
                    priority: row.get(3)?,
                    up_id: row.get(4)?,
                    done: row.get(5)?,
                    done_at: row.get(6)?,
                    deleted_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            }).map_err(|e| e.to_string())?;

            let mut todos = Vec::new();
            for row in rows {
                todos.push(row.map_err(|e| e.to_string())?);
            }
            Ok(todos)
        })
    }

    /// Update todo
    fn update(&self, id: i64, update: UpdateTodo) -> Result<Todo, String> {
        Self::with_conn(self, |conn| {
            // Get current values
            let current = self.find_by_id(id)?
                .ok_or_else(|| "Todo not found".to_string())?;

            // Apply updates (None means keep current value)
            let todo = update.todo.unwrap_or(current.todo);
            let due_date = update.due_date.or(current.due_date);
            let priority = update.priority.unwrap_or(current.priority);
            let up_id = update.up_id.or(current.up_id);
            let updated = Utc::now().format("%Y%m%dT%H%M%S").to_string();

            conn.execute(
                "UPDATE todos SET todo = ?1, due_date = ?2, priority = ?3, up_id = ?4, updated_at = ?5 WHERE id = ?6",
                params![todo, due_date, priority, up_id, updated, id],
            ).map_err(|e| e.to_string())?;

            self.find_by_id(id)?
                .ok_or_else(|| "Todo not found".to_string())
        })
    }

    /// Soft delete todo (sets deleted_at timestamp)
    fn delete(&self, id: i64) -> Result<(), String> {
        Self::with_conn(self, |conn| {
            let deleted = Utc::now().format("%Y%m%dT%H%M%S").to_string();
            
            conn.execute(
                "UPDATE todos SET deleted_at = ?1 WHERE id = ?2",
                params![deleted, id],
            ).map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    /// Set done level
    /// 
    /// If done=5 (complete), sets done_at to current date.
    /// Otherwise, clears done_at.
    fn set_done(&self, id: i64, done_level: Option<i32>) -> Result<Todo, String> {
        Self::with_conn(self, |conn| {
            let current = self.find_by_id(id)?
                .ok_or_else(|| "Todo not found".to_string())?;

            // Toggle if no level specified
            let new_done = done_level.unwrap_or(if current.done == 5 { 0 } else { 5 });
            let done_at = if new_done == 5 { 
                Some(Local::now().format("%Y%m%d").to_string()) 
            } else { 
                None 
            };

            conn.execute(
                "UPDATE todos SET done = ?1, done_at = ?2 WHERE id = ?3",
                params![new_done, done_at, id],
            ).map_err(|e| e.to_string())?;

            self.find_by_id(id)?
                .ok_or_else(|| "Todo not found".to_string())
        })
    }

    /// Search todos by keyword (includes completed)
    fn search(&self, keyword: &str) -> Result<Vec<Todo>, String> {
        Self::with_conn(self, |conn| {
            let pattern = format!("%{}%", keyword);
            let sql = "SELECT id, todo, due_date, priority, up_id, done, done_at, deleted_at, created_at, updated_at 
                     FROM todos WHERE todo LIKE ?1 AND deleted_at IS NULL
                     ORDER BY done ASC, priority ASC, due_date ASC, created_at DESC";
            
            let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
            let rows = stmt.query_map(params![pattern], |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    todo: row.get(1)?,
                    due_date: row.get(2)?,
                    priority: row.get(3)?,
                    up_id: row.get(4)?,
                    done: row.get(5)?,
                    done_at: row.get(6)?,
                    deleted_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            }).map_err(|e| e.to_string())?;

            let mut todos = Vec::new();
            for row in rows {
                todos.push(row.map_err(|e| e.to_string())?);
            }
            Ok(todos)
        })
    }
}

/// Helper: execute closure with new SQLite connection
/// 
/// Opens connection, executes closure, returns result.
/// Opens fresh connection each call for thread safety.
impl SqliteRepo {
    fn with_conn<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, String>,
    {
        let conn = Connection::open(&self.path).map_err(|e| e.to_string())?;
        f(&conn)
    }
}