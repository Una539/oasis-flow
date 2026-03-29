// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! TUI entry point.
//!
//! Provides [`run_tui`] which initializes the terminal, runs the event loop,
//! and returns the mutated [`TodoList`] when the user quits.

use super::state::AppTodo;
use crate::TodoList;

/// Launch the interactive TUI with the given todo list.
///
/// Blocks until the user presses `q` to quit. Returns the (possibly modified)
/// [`TodoList`] so the caller can persist changes back to the database.
pub fn run_tui(tdlist: TodoList) -> color_eyre::Result<TodoList> {
    let mut app = AppTodo::new(tdlist);

    ratatui::run(|terminal| app.run(terminal))?;
    Ok(app.todos)
}
