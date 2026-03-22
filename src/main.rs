// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main binary entry point for oasistodo CLI.
//!
//! This binary provides a command-line interface for managing todos.
//! For using oasistodo as a library in your Rust project, see [`oasistodo`](crate).

use oasistodo::{Cli, TodoList, execute_command};
use clap::Parser;
use directories::ProjectDirs;
use sqlx::SqlitePool;
use std::error::Error;
use std::path::PathBuf;

fn get_data_dir(data_dir_option: Option<String>, is_release: bool) -> PathBuf {
    // If user specified a custom directory, use it
    if let Some(dir) = data_dir_option {
        let path = PathBuf::from(dir);
        if !path.exists() {
            std::fs::create_dir_all(&path).ok();
        }
        return path;
    }

    // Only use OS-specific data directory in release mode
    if is_release && let Some(proj_dirs) = ProjectDirs::from("com", "oasistodo", "oasistodo") {
        let data_dir = proj_dirs.data_dir().to_path_buf();
        // Create directory if it doesn't exist
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).ok();
        }
        return data_dir;
    }

    // Fallback to current directory
    PathBuf::from(".")
}

#[cfg(not(debug_assertions))]
const IS_RELEASE: bool = true;

#[cfg(debug_assertions)]
const IS_RELEASE: bool = false;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let data_dir = get_data_dir(cli.data_dir.clone(), IS_RELEASE);

    let todos_toml_path = data_dir.join("todos.toml");
    let todos_db_path = data_dir.join("todos.db");

    let mut tdlist = TodoList::read_from_file(todos_toml_path.to_str().unwrap_or("todos.toml"))?;
    let pool = SqlitePool::connect(&format!("sqlite:{}", todos_db_path.display())).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tdlist.sync_to_db(&pool).await?;

    tdlist = execute_command(cli, tdlist, &pool).await?;

    tdlist.write_to_file(todos_toml_path.to_str().unwrap_or("todos.toml"))?;
    Ok(())
}
