// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::ListState};

use crate::TodoList;

#[derive(Debug, Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    Adding,
}

/// Core application state for the TUI.
///
/// Holds the todo list, the currently selected item state, and an exit flag
/// that controls the main event loop.
#[derive(Debug, Default)]
pub struct AppTodo {
    pub todos: TodoList,
    /// When set to `true`, the event loop terminates and the TUI exits.
    pub exit: bool,
    /// Tracks which item in the list is currently highlighted.
    pub state: ListState,
    pub input_mode: InputMode,
    pub input: String,
}

impl AppTodo {
    /// Create a new `AppTodo` from a [`TodoList`].
    ///
    /// If the list is non-empty, the first item is automatically selected.
    pub fn new(todos: TodoList) -> Self {
        let mut state = ListState::default();
        if !todos.todos.is_empty() {
            state.select(Some(0));
        }
        Self {
            todos,
            exit: false,
            state,
            input_mode: InputMode::Normal,
            input: String::new(),
        }
    }

    /// Run the main draw-then-handle loop until [`exit`](Self::exit) is set.
    ///
    /// Each iteration:
    /// 1. Takes ownership of `self.state` to pass into the draw closure.
    /// 2. Draws the current frame via [`draw_stateful`](Self::draw_stateful).
    /// 3. Restores `self.state` and waits for the next terminal event.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            let mut state = std::mem::take(&mut self.state);
            terminal.draw(|frame| self.draw_stateful(frame, &mut state))?;
            self.state = state;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Render the stateful widget (self) into the given frame.
    pub fn draw_stateful(&self, frame: &mut ratatui::Frame, state: &mut ListState) {
        frame.render_stateful_widget(self, frame.area(), state);
    }

    /// Read a single terminal event and dispatch it.
    ///
    /// Only [`KeyEventKind::Press`] events are processed; all other events
    /// (release, repeat) are silently ignored.
    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_events(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    /// Map a key press to its corresponding action.
    ///
    /// | Key            | Action                          |
    /// |----------------|---------------------------------|
    /// | `q`            | Exit the TUI                    |
    /// | `j` / `Down`   | Select next item                |
    /// | `k` / `Up`     | Select previous item            |
    /// | `Space`        | Toggle finished status of item  |
    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Down => self.state.select_next(),
                KeyCode::Up => self.state.select_previous(),
                KeyCode::Char('j') => self.state.select_next(),
                KeyCode::Char('k') => self.state.select_previous(),
                KeyCode::Char(' ') => self.toggle_selected(),
                KeyCode::Char('a') => {
                    self.input_mode = InputMode::Adding;
                    self.input.clear();
                }
                KeyCode::Char('d') => self.delete_selected(),
                KeyCode::Char('e') => self.edit_selected(),
                _ => {}
            },
            InputMode::Adding | InputMode::Insert => match key_event.code {
                KeyCode::Enter => {
                    let content = self.input.clone();
                    if !content.is_empty() {
                        if self.input_mode == InputMode::Adding {
                            self.add_todo(content);
                        } else if let Some(selected) = self.state.selected()
                            && let Some(todo) = self.todos.todos.values_mut().nth(selected)
                        {
                            todo.edit(content);
                        }
                    }
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                _ => {}
            },
        }
    }

    /// Toggle the `finished` status of the currently selected todo.
    ///
    /// - If unfinished: marks it as finished (sets `finished = true` and
    ///   records `finished_at`).
    /// - If finished: resets it to unfinished (clears both fields).
    pub fn toggle_selected(&mut self) {
        if let Some(selected) = self.state.selected()
            && let Some(todo) = self.todos.todos.values_mut().nth(selected)
        {
            if !todo.finished {
                let _ = todo.mark_finished();
            } else {
                todo.finished = false;
                todo.finished_at = None;
            }
        }
    }

    pub fn add_todo(&mut self, content: String) {
        let todo = crate::Todo::new(content);
        self.todos.todos.insert(todo.id.to_string(), todo);
        if self.todos.todos.len() == 1 {
            self.state.select(Some(0));
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(selected) = self.state.selected()
            && let Some(todo) = self.todos.todos.values().nth(selected)
        {
            let id = todo.id.to_string();
            self.todos.todos.remove(&id);
        }
    }

    pub fn edit_selected(&mut self) {
        if let Some(selected) = self.state.selected() {
            let context = self
                .todos
                .todos
                .values()
                .nth(selected)
                .map(|t| t.content.clone())
                .unwrap_or_default();
            self.input = context;
            self.input_mode = InputMode::Insert;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Todo;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Helper: build an `AppTodo` with `n` items.
    fn app_with(n: usize) -> AppTodo {
        let mut tdlist = TodoList::new();
        for i in 0..n {
            let todo = Todo::new(format!("task-{}", i));
            tdlist.todos.insert(todo.id.to_string(), todo);
        }
        AppTodo::new(tdlist)
    }

    // ── new ───────────────────────────────────────────────────────────

    #[test]
    fn new_empty_list_selects_none() {
        let app = AppTodo::new(TodoList::new());
        assert!(app.state.selected().is_none());
        assert!(!app.exit);
        assert!(app.todos.todos.is_empty());
    }

    #[test]
    fn new_nonempty_list_selects_first() {
        let app = app_with(3);
        assert_eq!(app.state.selected(), Some(0));
    }

    // ── handle_key_events ─────────────────────────────────────────────

    #[test]
    fn key_q_sets_exit() {
        let mut app = app_with(3);
        app.handle_key_events(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(app.exit);
    }

    #[test]
    fn key_j_selects_next() {
        let mut app = app_with(3);
        app.handle_key_events(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(app.state.selected(), Some(1));
    }

    #[test]
    fn key_k_selects_previous() {
        let mut app = app_with(3);
        // Move to index 1 first, then back to 0.
        app.state.select(Some(1));
        app.handle_key_events(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        assert_eq!(app.state.selected(), Some(0));
    }

    #[test]
    fn key_down_selects_next() {
        let mut app = app_with(3);
        app.handle_key_events(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.state.selected(), Some(1));
    }

    #[test]
    fn key_up_selects_previous() {
        let mut app = app_with(3);
        app.state.select(Some(2));
        app.handle_key_events(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.state.selected(), Some(1));
    }

    #[test]
    fn key_space_toggles_selected() {
        let mut app = app_with(1);
        // First toggle: unfinished -> finished
        app.handle_key_events(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let first = app.todos.todos.values().next().unwrap();
        assert!(first.finished);
        assert!(first.finished_at.is_some());

        // Second toggle: finished -> unfinished
        app.handle_key_events(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let first = app.todos.todos.values().next().unwrap();
        assert!(!first.finished);
        assert!(first.finished_at.is_none());
    }

    #[test]
    fn unknown_key_is_ignored() {
        let mut app = app_with(3);
        app.handle_key_events(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        assert!(!app.exit);
        assert_eq!(app.state.selected(), Some(0));
    }

    // ── toggle_selected ───────────────────────────────────────────────

    #[test]
    fn toggle_marks_finished_and_sets_timestamp() {
        let mut app = app_with(2);
        app.toggle_selected();
        let first = app.todos.todos.values().next().unwrap();
        assert!(first.finished);
        assert!(first.finished_at.is_some());
    }

    #[test]
    fn toggle_unfinished_clears_timestamp() {
        let mut app = app_with(1);
        // Finish first
        app.toggle_selected();
        // Unfinish
        app.toggle_selected();
        let first = app.todos.todos.values().next().unwrap();
        assert!(!first.finished);
        assert!(first.finished_at.is_none());
    }

    #[test]
    fn toggle_with_no_selection_is_noop() {
        let mut app = AppTodo::new(TodoList::new());
        // No panic, no change
        app.toggle_selected();
        assert!(app.todos.todos.is_empty());
    }

    #[test]
    fn toggle_only_affects_selected_item() {
        let mut app = app_with(3);
        // Select second item and toggle it
        app.state.select(Some(1));
        app.toggle_selected();

        let items: Vec<&Todo> = app.todos.todos.values().collect();
        assert!(!items[0].finished, "first item should stay unfinished");
        assert!(items[1].finished, "second item should be finished");
        assert!(!items[2].finished, "third item should stay unfinished");
    }
}
