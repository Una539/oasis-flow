// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// src/lib.rs
mod models;

use crate::models::{Todo, TodoList};
use std::error::Error;

pub enum TodoAction {
    Add(String),
    Done(String),
    Remove(String),
    List,
}

pub fn run(file_path: &str, content: String) -> Result<(), Box<dyn Error>> {
    let mut tdlist = if std::path::Path::new(file_path).exists() {
        let content = std::fs::read_to_string(file_path)?;
        toml::from_str(&content).unwrap_or_else(|_| TodoList::new())
    } else {
        TodoList::new()
    };

    let existing_todo = tdlist
        .todos
        .values_mut()
        .find(|t| t.get_content() == content);
    if let Some(todo) = existing_todo {
        println!("任务已存在: {}", todo.get_content());
    } else {
        let new_todo = Todo::new(content.clone());
        tdlist.add(new_todo);
        println!("添加新任务: {}", content);
    }

    let toml_str = toml::to_string_pretty(&tdlist)?;
    std::fs::write(file_path, toml_str)?;
    Ok(())
}
