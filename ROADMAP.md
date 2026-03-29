# Oasis Flow (oflow) Roadmap

## Current State (v0.4.3)

Completed features:

- CLI commands: add / finish / edit / clean / list / path / tui
- TUI mode (vim-style keybindings, ratatui rendering)
- SQLite persistence + TOML export
- `TodoManager` library API
- Unit tests + 10,000-item performance tests

---

## Phase 1 — Data Model Enhancement (v0.5.x)

| Feature | Description |
|---|---|
| **Priority** | Add `priority` field to `Todo` (Low/Medium/High/Urgent). CLI support: `oflow add -p high "..."` |
| **Due Date** | `due_at: Option<DateTime<Utc>>`. CLI support: `oflow add -d 2026-04-01 "..."` |
| **Tags/Categories** | `tags: Vec<String>`. CLI support: `oflow add -t work "..."`, filter by tag: `oflow list --tag work` |
| **Database Migration** | New migration scripts to add columns to `todos` table |
| **Sorting** | TUI and list support sorting by priority / creation date / due date |

---

## Phase 2 — TUI Enhancement (v0.6.x)

| Feature | Description |
|---|---|
| **Search/Filter** | `/` key enters search mode with real-time list filtering |
| **Sidebar** | Left panel showing tag list; click to toggle filter |
| **Detail Panel** | Right panel showing full info for selected todo (created at, due date, tags, etc.) |
| **Theme Support** | Light/dark theme switching |
| **Help Page** | `?` key displays full keyboard shortcut reference |

---

## Phase 3 — Import/Export & Sync (v0.7.x)

| Feature | Description |
|---|---|
| **JSON Import/Export** | `oflow export --format json` / `oflow import file.json` |
| **Markdown Export** | `oflow export --format md` generates checkbox format |
| **CSV Import/Export** | Interoperability with spreadsheet tools |
| **TOML Import** | `oflow import file.toml` to restore from TOML file |

---

## Phase 4 — Advanced Features (v0.8.x)

| Feature | Description |
|---|---|
| **Subtasks** | Add `parent_id` to `Todo` for task nesting |
| **Recurring Tasks** | `repeat: Option<RepeatRule>` (daily/weekly/monthly) |
| **Reminders/Notifications** | Desktop notifications for due tasks (`notify-rust`) |
| **Batch Operations** | `oflow finish --all`, `oflow clean --older-than 30d` |
| **Undo/Redo** | Operation history stack with `oflow undo` |

---

## Phase 5 — Engineering Quality & Release (v0.9.x → v1.0)

| Task | Description |
|---|---|
| **CI/CD** | GitHub Actions: lint + test + build + release |
| **Cross-Platform Builds** | Linux/macOS/Windows binary releases |
| **crates.io Publish** | Improve `Cargo.toml` metadata, publish library crate |
| **Shell Completions** | `oflow completions bash/zsh/fish` |
| **Man Page** | Generate man page |
| **Integration Tests** | End-to-end CLI tests (`assert_cmd`) |
| **Error Messages** | Friendlier error hints with spelling suggestions |
| **Documentation** | Improve `cargo doc` + online docs |

---

## License Note

Current license is **MPL-2.0** (see `Cargo.toml`). New dependencies must be compatible with MPL-2.0. Existing dependencies (anyhow, chrono, clap, sqlx, ratatui, etc.) are MIT/Apache dual-licensed or MPL-compatible.
