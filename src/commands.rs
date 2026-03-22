// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::models::TodoList;
use clap::{Parser, Subcommand};
use sqlx::SqlitePool;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add { content: String },
    Finish { content: String },
    Edit { find: String, replace: String },
    Clean,
}

pub async fn execute_command(
    cli: Cli,
    mut tdlist: TodoList,
    pool: &SqlitePool,
) -> Result<TodoList, Box<dyn Error>> {
    match cli.command {
        Commands::Add { content } => {
            tdlist.add_todo_db(content, pool).await?;
        }
        Commands::Finish { content } => {
            tdlist.finish_todo_db(content, pool).await?;
        }
        Commands::Edit { find, replace } => {
            tdlist.edit_todo_db(find, replace, pool).await?;
        }
        Commands::Clean => {
            tdlist.clean_todo_db(pool).await?;
        }
    }

    tdlist.sync_to_db(pool).await?;
    Ok(tdlist)
}

#[cfg(test)]
mod args_test {
    use super::*;

    #[test]
    fn test_add() {
        let args = vec!["oflow", "add", "Buy milk"];
        let cli = Cli::try_parse_from(args).expect("Failed to parse arguments");

        match cli.command {
            Commands::Add { content } => {
                assert_eq!(content, "Buy milk");
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_finish() {
        let args = vec!["oflow", "finish", "Buy milk"];
        let cli = Cli::try_parse_from(args).expect("Failed to parse arguments");

        match cli.command {
            Commands::Finish { content } => {
                assert_eq!(content, "Buy milk");
            }
            _ => panic!("Expect Finish command"),
        }
    }

    #[test]
    fn test_edit() {
        let args = vec!["oflow", "edit", "Buy milk", "Buy eggs"];
        let cli = Cli::try_parse_from(args).expect("Failed to parse arguments");

        match cli.command {
            Commands::Edit { find, replace } => {
                assert_eq!(find, "Buy milk");
                assert_eq!(replace, "Buy eggs");
            }
            _ => panic!("Expect Edit command"),
        }
    }
}
