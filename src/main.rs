// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod commands;
mod database;
mod models;

use crate::commands::Cli;
use crate::models::TodoList;
use clap::Parser;
use sqlx::SqlitePool;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut tdlist = TodoList::read_from_file("todos.toml")?;
    let pool = SqlitePool::connect("sqlite:todos.db").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tdlist.sync_to_db(&pool).await?;

    tdlist = commands::execute_command(cli, tdlist, &pool).await?;

    tdlist.write_to_file("todos.toml")?;
    Ok(())
}
