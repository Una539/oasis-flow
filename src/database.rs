// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Database persistence layer for [`TodoList`].
//!
//! This module provides SQLite-backed storage methods that are attached to
//! [`TodoList`] via `impl` blocks. These methods handle syncing in-memory
//! state to the database, loading from the database, and querying by content.

use crate::models::{Todo, TodoList};
use sqlx::SqlitePool;

impl TodoList {
    /// Synchronize all in-memory todos to the database.
    ///
    /// Performs an upsert (INSERT ... ON CONFLICT DO UPDATE) for each todo
    /// in a single transaction. This ensures that the database reflects the
    /// current state of the in-memory [`TodoList`].
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite connection pool
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn sync_to_db(&self, pool: &SqlitePool) -> color_eyre::Result<()> {
        let mut tx = pool.begin().await?;

        for todo in self.todos.values() {
            sqlx::query(
                "INSERT INTO todos (id, content, created_at, finished_at, finished)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    content = excluded.content,
                    finished_at = excluded.finished_at,
                    finished = excluded.finished",
            )
            .bind(todo.id)
            .bind(&todo.content)
            .bind(todo.created_at)
            .bind(todo.finished_at)
            .bind(todo.finished)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Load all todos from the database into memory.
    ///
    /// Replaces the current in-memory todo map with all rows from the
    /// `todos` table.
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite connection pool
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn load_from_db(&mut self, pool: &SqlitePool) -> color_eyre::Result<()> {
        let rows = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
            .fetch_all(pool)
            .await?;

        self.todos = rows.into_iter().map(|t| (t.id.to_string(), t)).collect();

        Ok(())
    }

    /// Find a todo by its content.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `content` - Content to search for
    ///
    /// # Returns
    ///
    /// Returns the Todo if found, None otherwise.
    pub async fn find_by_content(
        pool: &SqlitePool,
        content: &str,
    ) -> color_eyre::Result<Option<Todo>> {
        let res = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE content = ? LIMIT 1")
            .bind(content)
            .fetch_optional(pool)
            .await?;

        Ok(res)
    }
}
