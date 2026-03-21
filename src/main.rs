// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};
use oasis_flow::{run_add, run_edit, run_finish};
use std::error::Error;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands{
    Add {
        content:String
    },
    Finish {
        content: String,
    },
    Edit {
        find: String,
        replace: String,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let file_path = "todos.toml";

    match cli.command {
        Commands::Add{content} => {
            run_add(file_path, content)?;
        }
        Commands::Finish{content} => {
            run_finish(file_path, content)?;
        }
        Commands::Edit { find, replace } => {
            run_edit(file_path, find, replace)?;
        }
    }

    Ok(())
}


#[cfg(test)]
mod args_test {
    use super::*;

    #[test]
    fn test_add() {
        let args = vec!["oasis_flow", "add", "Buy milk"];
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
        let args = vec!["oasis_flow", "finish", "Buy milk"];
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
        let args = vec!["oasis_flow", "edit", "Buy milk", "Buy eggs"];
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