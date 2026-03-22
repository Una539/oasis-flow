# Oasis Flow (oflow)

A focused todo CLI tool built with Rust.

## Features

- **Add Tasks**: Quickly add new todo items
- **Finish Tasks**: Mark tasks as completed
- **Edit Tasks**: Modify existing task content
- **Clean**: Remove all finished tasks

## Installation

```bash
cargo install oflow
```

## Usage

### Add a Task

```bash
oflow add "Buy groceries"
```

### Finish a Task

```bash
oflow finish "Buy groceries"
```

### Edit a Task

```bash
oflow edit "Buy groceries" "Buy vegetables"
```

### Clean Finished Tasks

```bash
oflow clean
```

## Data Storage

Tasks are stored in:
- A `todos.toml` file in the current directory
- A `todos.db` SQLite database for persistence

## Language

- [English](README.md)
- [中文](README_CN.md)

## License

This project is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.
