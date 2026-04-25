//! Database abstraction layer for todoj
//! 
//! Provides trait-based abstraction for different database backends.
//! Currently supports SQLite; PostgreSQL planned for future.
//! 
//! # Architecture
//! 
//! - [TodoRepository] trait: defines CRUD operations
//! - [Todo], [NewTodo], [UpdateTodo]: data structures
//! - [SqliteRepo]: SQLite implementation

mod sqlite;

pub use sqlite::SqliteRepo;

use serde::{Deserialize, Serialize};

/// Todo item stored in database
/// 
/// # Fields
/// 
/// * `id` - Unique primary key
/// * `todo` - Todo content text
/// * `due_date` - Due date in "YYYYMMDD" format, nullable
/// * `priority` - Priority 1-4 (default 3, where 1 is highest)
/// * `up_id` - Parent todo ID for sub-todos, nullable
/// * `done` - Done level 0-5 (5 = complete)
/// * `done_at` - Completion date in "YYYYMMDD", nullable
/// * `deleted_at` - Deletion timestamp, nullable for soft delete
/// * `created_at` - Creation timestamp "YYYYMMDDTHHMMSS"
/// * `updated_at` - Last update timestamp "YYYYMMDDTHHMMSS"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub todo: String,
    pub due_date: Option<String>,
    pub priority: i32,
    pub up_id: Option<i64>,
    pub done: i32,
    pub done_at: Option<String>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// New todo data for creation
/// 
/// Used when creating a new todo item.
/// Fields mirror Todo but without auto-generated fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTodo {
    pub todo: String,
    pub due_date: Option<String>,
    pub priority: Option<i32>,
    pub up_id: Option<i64>,
}

/// Todo update data
/// 
/// All fields are optional - only non-None fields are updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub todo: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<i32>,
    pub up_id: Option<i64>,
}

/// Database repository trait
/// 
/// Defines all CRUD operations for todo persistence.
/// Implement this trait to add new database backends.
/// 
/// # Design
/// 
/// This trait is synchronous (blocking) for simplicity.
/// For async databases like PostgreSQL, use async variant.
/// 
/// # Implementations
/// 
/// - [crate::SqliteRepo]: SQLite (current)
/// - [crate::PostgresRepo]: PostgreSQL (future)
pub trait TodoRepository: Send + Sync {
    /// Initialize database schema
    /// 
    /// Called once at startup to create tables if not exist.
    fn init(&self) -> Result<(), String>;
    
    /// Create new todo
    /// 
    /// # Arguments
    /// * `new_todo` - New todo data
    /// 
    /// # Returns
    /// * `Ok(todo)` - Created todo with generated ID
    /// * `Err(message)` on error
    fn create(&self, new_todo: NewTodo) -> Result<Todo, String>;
    
    /// Find todo by ID
    /// 
    /// # Arguments
    /// * `id` - Todo ID
    /// 
    /// # Returns
    /// * `Ok(Some(todo))` if found
    /// * `Ok(None)` if not found
    /// * `Err(message)` on error
    fn find_by_id(&self, id: i64) -> Result<Option<Todo>, String>;
    
    /// Find all todos (optionally including completed)
    /// 
    /// # Arguments
    /// * `include_done` - If true, include completed (done=5) todos
    /// 
    /// # Returns
    /// * `Ok(todos)` - Vector of todos sorted appropriately
    /// * `Err(message)` on error
    fn find_all(&self, include_done: bool) -> Result<Vec<Todo>, String>;
    
    /// Update existing todo
    /// 
    /// # Arguments
    /// * `id` - Todo ID to update
    /// * `update` - Update data (only non-None fields changed)
    /// 
    /// # Returns
    /// * `Ok(todo)` - Updated todo
    /// * `Err(message)` on error
    fn update(&self, id: i64, update: UpdateTodo) -> Result<Todo, String>;
    
    /// Soft delete todo
    /// 
    /// Sets deleted_at timestamp instead of removing.
/// 
    /// # Arguments
    /// * `id` - Todo ID to delete
    /// 
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(message)` on error
    fn delete(&self, id: i64) -> Result<(), String>;
    
    /// Set done level
    /// 
    /// Updates done level and sets done_at when completing.
/// 
    /// # Arguments
    /// * `id` - Todo ID
    /// * `done_level` - Done level 0-5, or None to toggle
    /// 
    /// # Returns
    /// * `Ok(todo)` - Updated todo
    /// * `Err(message)` on error
    fn set_done(&self, id: i64, done_level: Option<i32>) -> Result<Todo, String>;
}