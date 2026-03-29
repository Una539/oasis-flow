// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::models::TodoList;
use clap::{Parser, Subcommand};
use sqlx::SqlitePool;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Custom data directory path (defaults to OS-specific data directory)
    /// This directory should contain todos.db and todos.toml
    #[arg(short, long, default_value = None)]
    pub data_dir: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new todo item
    ///
    /// Example: oflow add "Buy milk"
    Add { content: String },

    /// Mark a todo as finished
    ///
    /// Example: oflow finish "Buy milk"
    Finish { content: String },

    /// Edit an existing todo's content
    ///
    /// Example: oflow edit "Buy milk" "Buy eggs"
    Edit {
        /// The content to search for
        find: String,
        /// The new content to replace with
        replace: String,
    },

    /// Clean all finished/completed todos
    ///
    /// Example: oflow clean
    Clean,

    /// List all the todos
    ///
    /// Example: oflow list
    List,

    /// Run the TUI interface
    ///
    /// Example: oflow tui
    Tui,
}

pub async fn execute_command(
    cli: Cli,
    mut tdlist: TodoList,
    pool: &SqlitePool,
) -> color_eyre::Result<TodoList> {
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
        Commands::List => {
            tdlist.list_todos();
        }
        Commands::Tui => {
            tdlist = crate::tui::app::run_tui(tdlist)?;
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
