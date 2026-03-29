// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main binary entry point for oflow CLI.
//!
//! This binary provides a cli and tui interface for managing todos.
//! For using oflow as a library in your Rust project, see [`oflow`](crate).

use clap::Parser;
use directories::ProjectDirs;
use oflow::{Cli, TodoList, execute_command};
use sqlx::SqlitePool;
use std::path::PathBuf;

fn get_data_dir(data_dir_option: Option<String>, is_release: bool) -> color_eyre::Result<PathBuf> {
    // If user specified a custom directory, use it
    if let Some(dir) = data_dir_option {
        let path = PathBuf::from(dir);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        return Ok(path);
    }

    // Only use OS-specific data directory in release mode
    if is_release && let Some(proj_dirs) = ProjectDirs::from("com", "oasistodo", "oasistodo") {
        let data_dir = proj_dirs.data_dir().to_path_buf();
        // Create directory if it doesn't exist
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }
        return Ok(data_dir);
    }

    // Fallback to current directory
    Ok(PathBuf::from("."))
}

#[cfg(not(debug_assertions))]
const IS_RELEASE: bool = true;

#[cfg(debug_assertions)]
const IS_RELEASE: bool = false;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let data_dir = get_data_dir(cli.data_dir.clone(), IS_RELEASE)?;

    let todos_toml_path = data_dir.join("todos.toml");
    let todos_db_path = data_dir.join("todos.db");

    let db_existed = todos_db_path.exists();
    let toml_existed = todos_toml_path.exists();

    // Read existing TOML data (returns empty list if file missing)
    let mut tdlist = TodoList::read_from_file(todos_toml_path.to_str().unwrap_or("todos.toml"))?;

    // Connect to SQLite; mode=rwc creates the file if it doesn't exist
    let db_url = format!("sqlite:{}?mode=rwc", todos_db_path.display());
    let pool = SqlitePool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    if db_existed {
        // Database exists: load from it (authoritative source)
        tdlist.load_from_db(&pool).await?;
    } else if toml_existed {
        // No DB but TOML exists: seed the new database from TOML data
        tdlist.sync_to_db(&pool).await?;
    }
    // Neither existed: empty list, empty DB — nothing to seed

    tdlist = execute_command(cli, tdlist, &pool, &data_dir).await?;

    tdlist.write_to_file(todos_toml_path.to_str().unwrap_or("todos.toml"))?;
    Ok(())
}
