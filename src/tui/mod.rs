// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Terminal user interface module.
//!
//! - [`state`] — Application state ([`AppTodo`]) and event handling.
//! - [`ui`] — Ratatui rendering via the [`StatefulWidget`](ratatui::widgets::StatefulWidget) trait.
//! - [`app`] — Public entry point [`run_tui`].

mod state;
mod ui;

pub mod app;
