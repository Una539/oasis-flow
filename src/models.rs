// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::fmt::{self, Display};
use uuid::Uuid;

/// A single todo item.
///
/// Represents one task with a unique identifier, content, timestamps,
/// and completion status.
#[derive(Deserialize, Serialize, Debug, FromRow, Clone)]
pub struct Todo {
    pub id: uuid::Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub finished: bool,
}

impl Todo {
    /// Create a new unfinished todo with the given content.
    pub fn new(content: String) -> Todo {
        Todo {
            id: Uuid::new_v4(),
            content,
            created_at: Utc::now(),
            finished_at: None,
            finished: false,
        }
    }

    /// Mark this todo as finished and return its content.
    pub fn mark_finished(&mut self) -> &str {
        self.finished = true;
        self.finished_at = Some(Utc::now());
        &self.content
    }

    /// Replace the content and return the new value.
    pub fn edit(&mut self, new_content: String) -> &str {
        self.content = new_content;
        &self.content
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}

impl Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.finished { " ✓ " } else { "   " };
        write!(f, "[{}] {}", status, self.content)
    }
}

/// An ordered collection of [`Todo`] items keyed by their UUID.
///
/// Supports file-based I/O (TOML) and database-backed operations
/// via the [`database`](crate::database) module.
#[derive(Deserialize, Serialize, Debug)]
pub struct TodoList {
    #[serde(flatten)]
    pub todos: BTreeMap<String, Todo>,
}

impl TodoList {
    /// Create an empty todo list.
    pub fn new() -> TodoList {
        TodoList {
            todos: BTreeMap::new(),
        }
    }

    /// Read a todo list from a TOML file.
    ///
    /// Returns an empty list if the file does not exist or cannot be parsed.
    pub fn read_from_file(file_path: &str) -> color_eyre::Result<TodoList> {
        let tdlist = if std::path::Path::new(file_path).exists() {
            let content = std::fs::read_to_string(file_path)?;
            toml::from_str(&content).unwrap_or_else(|_| TodoList::new())
        } else {
            TodoList::new()
        };

        Ok(tdlist)
    }

    /// Write the todo list to a TOML file.
    pub fn write_to_file(&self, file_path: &str) -> color_eyre::Result<()> {
        let toml_str = toml::to_string_pretty(self)?;
        std::fs::write(file_path, toml_str)?;
        Ok(())
    }

    /// Add a new todo via the database, skipping if content already exists.
    pub async fn add_todo_db(
        &mut self,
        content: String,
        pool: &SqlitePool,
    ) -> color_eyre::Result<()> {
        if let Some(task) = TodoList::find_by_content(pool, &content).await? {
            println!("Task already exists: {}", task.get_content());
        } else {
            println!("Created task: {}", content);
            let new_todo = Todo::new(content.clone());
            self.todos.insert(new_todo.id.to_string(), new_todo);
        }
        Ok(())
    }

    /// Mark a todo as finished by searching its content in the database.
    pub async fn finish_todo_db(
        &mut self,
        content: String,
        pool: &SqlitePool,
    ) -> color_eyre::Result<()> {
        if let Some(task_from_db) = TodoList::find_by_content(pool, &content).await? {
            let id_str = task_from_db.id.to_string();
            if let Some(todo) = self.todos.get_mut(&id_str) {
                todo.mark_finished();
                println!("Finished task: {}", todo.content);
            }
        } else {
            println!("Task not found: {}", content);
        }
        Ok(())
    }

    /// Edit a todo's content by searching for the old value in the database.
    pub async fn edit_todo_db(
        &mut self,
        before: String,
        after: String,
        pool: &SqlitePool,
    ) -> color_eyre::Result<()> {
        if let Some(task_from_db) = TodoList::find_by_content(pool, &before).await? {
            let id_str = task_from_db.id.to_string();
            if let Some(todo) = self.todos.get_mut(&id_str) {
                todo.edit(after);
                println!("Updated task: {}", todo.content);
            }
        } else {
            println!("Task not found: {}", before);
        }
        Ok(())
    }

    /// Remove all finished todos from both the database and in-memory map.
    pub async fn clean_todo_db(
        &mut self,
        pool: &sqlx::SqlitePool,
    ) -> color_eyre::Result<()> {
        use sqlx::Row;

        // Delete finished rows and return their IDs
        let rows = sqlx::query("DELETE FROM todos WHERE finished = 1 RETURNING id")
            .fetch_all(pool)
            .await?;

        if rows.is_empty() {
            println!("Nothing to delete!");
            return Ok(());
        }

        for row in rows {
            // Decode the UUID (sqlx handles BLOB automatically)
            let id: Uuid = row.try_get(0)?;

            // Convert to String to match BTreeMap key
            let id_str = id.to_string();
            if let Some(removed_todo) = self.todos.remove(&id_str) {
                println!(
                    "Successfully cleaned task: [{}] \"{}\" (Finished at: {:?})",
                    id_str, removed_todo.content, removed_todo.finished_at
                );
            } else {
                // DB row deleted but not found in memory (should not happen unless state is out of sync)
                println!(
                    "Warning: Task ID {} was deleted from DB but not found in memory map.",
                    id_str
                );
            }
        }

        Ok(())
    }

    /// Print all todos to stdout.
    pub fn list_todos(&self) {
        println!("{}", self);
    }
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TodoList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.todos.is_empty() {
            return write!(f, "No todos");
        }
        for (i, (_, todo)) in self.todos.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, todo)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod performance_test {
    use super::*;
    use sqlx::SqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::Instant;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                finished_at TEXT,
                finished INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    fn create_large_todo_list(count: usize) -> TodoList {
        let mut tdlist = TodoList::new();
        for i in 0..count {
            let todo = Todo::new(format!("Test task {}", i));
            tdlist.todos.insert(todo.id.to_string(), todo);
        }
        tdlist
    }

    #[tokio::test]
    async fn test_add_10000_todos_performance() {
        let pool = create_test_pool().await;
        let mut tdlist = TodoList::new();
        let start = Instant::now();

        for i in 0..10000 {
            let content = format!("Performance test task {}", i);
            let new_todo = Todo::new(content);
            tdlist.todos.insert(new_todo.id.to_string(), new_todo);
        }

        tdlist.sync_to_db(&pool).await.unwrap();

        let elapsed = start.elapsed();
        println!("Adding 10000 todos took: {:?}", elapsed);
        assert!(
            elapsed.as_secs() < 30,
            "Adding 10000 todos took too long: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_finish_10000_todos_performance() {
        let pool = create_test_pool().await;
        let mut tdlist = create_large_todo_list(10000);

        tdlist.sync_to_db(&pool).await.unwrap();

        let start = Instant::now();

        for (_, todo) in tdlist.todos.iter_mut() {
            todo.mark_finished();
        }

        tdlist.sync_to_db(&pool).await.unwrap();

        let elapsed = start.elapsed();
        println!("Finishing 10000 todos took: {:?}", elapsed);
        assert!(
            elapsed.as_secs() < 30,
            "Finishing 10000 todos took too long: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_clean_10000_completed_todos_performance() {
        let pool = create_test_pool().await;
        let mut tdlist = TodoList::new();

        for i in 0..10000 {
            let mut todo = Todo::new(format!("Task to clean {}", i));
            todo.mark_finished();
            tdlist.todos.insert(todo.id.to_string(), todo);
        }

        tdlist.sync_to_db(&pool).await.unwrap();

        let start = Instant::now();

        tdlist.clean_todo_db(&pool).await.unwrap();

        let elapsed = start.elapsed();
        println!("Cleaning 10000 completed todos took: {:?}", elapsed);
        assert!(
            elapsed.as_secs() < 30,
            "Cleaning 10000 todos took too long: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_mixed_operations_10000_todos() {
        let pool = create_test_pool().await;
        let mut tdlist = create_large_todo_list(10000);

        tdlist.sync_to_db(&pool).await.unwrap();

        let start_add = Instant::now();
        for i in 0..1000 {
            let content = format!("Additional task {}", i);
            let new_todo = Todo::new(content);
            tdlist.todos.insert(new_todo.id.to_string(), new_todo);
        }
        tdlist.sync_to_db(&pool).await.unwrap();
        println!("Adding 1000 more todos took: {:?}", start_add.elapsed());

        let start_finish = Instant::now();
        for (_, todo) in tdlist.todos.iter_mut().take(5000) {
            todo.mark_finished();
        }
        tdlist.sync_to_db(&pool).await.unwrap();
        println!("Finishing 5000 todos took: {:?}", start_finish.elapsed());

        let start_clean = Instant::now();
        tdlist.clean_todo_db(&pool).await.unwrap();
        println!(
            "Cleaning 5000 completed todos took: {:?}",
            start_clean.elapsed()
        );

        let remaining = tdlist.todos.len();
        println!("Remaining todos after operations: {}", remaining);
        assert_eq!(remaining, 6000, "Expected 6000 remaining todos");
    }
}
