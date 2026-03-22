// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # oflow
//!
//! A focused todo + pomodoro CLI library.
//!
//! ## Quick Start
//!
//! ```rust
//! use oflow::TodoManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut manager = TodoManager::new().await?;
//!     manager.add("Buy milk").await?;
//!     manager.finish("Buy milk").await?;
//!     Ok(())
//! }
//! ```

pub mod commands;
pub mod database;
pub mod models;

// Re-export public types
pub use commands::{Cli, Commands, execute_command};
pub use models::{Todo, TodoList};
use sqlx::SqlitePool;
use std::error::Error;

/// A high-level manager for Todo operations.
///
/// Provides a simple API to manage todos without dealing with
/// database connections and migrations manually.
pub struct TodoManager {
    /// The underlying todo list
    pub todolist: TodoList,
    pool: SqlitePool,
}

impl TodoManager {
    /// Create a new TodoManager with automatic database initialization.
    ///
    /// This will:
    /// 1. Connect to SQLite database (creates `todos.db` if not exists)
    /// 2. Run migrations
    /// 3. Load existing todos from database
    ///
    /// # Errors
    ///
    /// Returns an error if database connection fails or migrations cannot run.
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let pool = SqlitePool::connect("sqlite:todos.db").await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        let mut todolist = TodoList::new();
        todolist.load_from_db(&pool).await?;

        Ok(Self { todolist, pool })
    }

    /// Create a new TodoManager with custom database path.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to SQLite database file
    ///
    /// # Errors
    ///
    /// Returns an error if database connection fails or migrations cannot run.
    pub async fn with_db_path(db_path: &str) -> Result<Self, Box<dyn Error>> {
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        let mut todolist = TodoList::new();
        todolist.load_from_db(&pool).await?;

        Ok(Self { todolist, pool })
    }

    /// Add a new todo item.
    ///
    /// # Arguments
    ///
    /// * `content` - The todo content
    ///
    /// # Errors
    ///
    /// Returns an error if database operation fails.
    pub async fn add(&mut self, content: &str) -> Result<(), Box<dyn Error>> {
        self.todolist
            .add_todo_db(content.to_string(), &self.pool)
            .await?;
        self.todolist.sync_to_db(&self.pool).await?;
        Ok(())
    }

    /// Mark a todo as finished.
    ///
    /// # Arguments
    ///
    /// * `content` - The todo content to mark as finished
    ///
    /// # Errors
    ///
    /// Returns an error if database operation fails.
    pub async fn finish(&mut self, content: &str) -> Result<(), Box<dyn Error>> {
        self.todolist
            .finish_todo_db(content.to_string(), &self.pool)
            .await?;
        self.todolist.sync_to_db(&self.pool).await?;
        Ok(())
    }

    /// Edit a todo's content.
    ///
    /// # Arguments
    ///
    /// * `find` - The content to search for
    /// * `replace` - The new content
    ///
    /// # Errors
    ///
    /// Returns an error if database operation fails.
    pub async fn edit(&mut self, find: &str, replace: &str) -> Result<(), Box<dyn Error>> {
        self.todolist
            .edit_todo_db(find.to_string(), replace.to_string(), &self.pool)
            .await?;
        self.todolist.sync_to_db(&self.pool).await?;
        Ok(())
    }

    /// Clean all completed/finished todos.
    ///
    /// # Errors
    ///
    /// Returns an error if database operation fails.
    pub async fn clean(&mut self) -> Result<(), Box<dyn Error>> {
        self.todolist.clean_todo_db(&self.pool).await?;
        Ok(())
    }

    /// Get an iterator over all todos.
    pub fn list(&self) -> impl Iterator<Item = &Todo> {
        self.todolist.todos.values()
    }

    /// Get an iterator over pending (not finished) todos.
    pub fn list_pending(&self) -> impl Iterator<Item = &Todo> {
        self.todolist.todos.values().filter(|t| !t.finished)
    }

    /// Get an iterator over finished todos.
    pub fn list_finished(&self) -> impl Iterator<Item = &Todo> {
        self.todolist.todos.values().filter(|t| t.finished)
    }

    /// Get todo by content.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to search for
    ///
    /// # Returns
    ///
    /// Returns the Todo if found, None otherwise.
    pub async fn get(&self, content: &str) -> Result<Option<Todo>, Box<dyn Error>> {
        // Query from database for the most up-to-date data
        let result = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE content = ? LIMIT 1")
            .bind(content)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires migrations directory to exist
    async fn test_todo_manager_new() {
        // This test requires the migrations directory to exist
        // Use #[ignore] since it depends on external resources
        let result = TodoManager::with_db_path(":memory:").await;
        assert!(result.is_ok(), "TodoManager should initialize with valid migrations");
    }
}
