// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering logic for the TUI.
//!
//! Implements [`StatefulWidget`] for [`&AppTodo`](super::state::AppTodo)
//! so the todo list can be rendered as a ratatui [`List`] widget with
//! keyboard shortcut hints in the bottom border.

use std::env;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, StatefulWidget},
};

use super::state::AppTodo;
use crate::tui::state::InputMode;

impl StatefulWidget for &AppTodo {
    type State = ListState;

    /// Render the todo list into the given buffer area.
    ///
    /// The widget consists of:
    /// - A bordered [`Block`] titled "Todo List".
    /// - An instruction bar at the bottom showing vim-style keybindings.
    /// - A [`List`] of todos with the currently selected item highlighted.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Layout depends on input mode
        let chunks = if self.input_mode != InputMode::Normal {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(3), // input box
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1)])
                .split(area)
        };

        // Bottom help bar
        let help_text = match self.input_mode {
            InputMode::Normal => Line::from(vec![
                " Next ".into(),
                "<j/↓>".blue().bold(),
                " Prev ".into(),
                "<k/↑>".blue().bold(),
                " Toggle ".into(),
                "<Space>".blue().bold(),
                " Add ".into(),
                "<a>".blue().bold(),
                " Edit ".into(),
                "<e>".blue().bold(),
                " Del ".into(),
                "<d>".blue().bold(),
                " Quit ".into(),
                "<q>".blue().bold(),
            ]),
            InputMode::Adding | InputMode::Insert => Line::from(vec![
                " Confirm ".into(),
                "<Enter>".blue().bold(),
                " Cancel ".into(),
                "<Esc>".blue().bold(),
            ]),
        };

        let version = env!("CARGO_PKG_VERSION");

        let block = Block::bordered()
            .title(Line::from(" Todo List ".to_string() + version + " ").centered())
            .title_bottom(help_text.right_aligned())
            .border_type(BorderType::Rounded);

        let items: Vec<ListItem> = self
            .todos
            .todos
            .values()
            .map(|t| ListItem::new(format!("{}", t)))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(ratatui::style::Color::Rgb(104, 204, 255)));

        StatefulWidget::render(&list, chunks[0], buf, state);

        // Render input box
        if self.input_mode != InputMode::Normal {
            let title = if self.input_mode == InputMode::Adding {
                " New Todo "
            } else {
                " Edit Todo "
            };
            let input_widget = Paragraph::new(self.input.as_str())
                .block(Block::default().title(title).borders(Borders::all()));
            ratatui::widgets::Widget::render(input_widget, chunks[1], buf);
        }
    }
}
