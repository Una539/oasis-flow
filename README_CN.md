# Oasis Flow (oflow)

一个基于 Rust 开发的待办事项 CLI 工具与 Rust 库。

## 功能特性

- **添加任务**：快速添加新的待办事项
- **完成任务**：将任务标记为已完成
- **编辑任务**：修改现有任务的内容
- **清理**：删除所有已完成的任务
- **列出任务**：显示所有任务及其完成状态
- **TUI 模式**：交互式终端界面，支持 vim 风格快捷键
- **库 API**：通过 [`TodoManager`] 将 `oflow` 作为 Rust 库使用

## 安装

```bash
cargo install oflow-todo
```

## CLI 使用方法

### 全局选项

| 选项 | 说明 |
|---|---|
| `--data-dir <路径>` | 自定义数据目录（覆盖系统默认路径） |

### 添加任务

```bash
oflow add "购买日用品"
```

### 完成任务

```bash
oflow finish "购买日用品"
```

### 编辑任务

```bash
oflow edit "购买日用品" "购买蔬菜"
```

### 清理已完成的任务

```bash
oflow clean
```

### 列出任务

```bash
oflow list
```

### TUI 模式

```bash
oflow tui
```

![Empty TUI](assets/empty-tui.png)

![Input Mode](assets/input_mode.png)

启动交互式终端界面。键盘快捷键：

| 按键 | 操作 |
|---|---|
| `j` / `Down` | 选择下一项 |
| `k` / `Up` | 选择上一项 |
| `Space` | 切换完成状态 |
| `a` | 添加新任务 |
| `e` | 编辑选中的任务 |
| `d` | 删除选中的任务 |
| `q` | 退出 TUI |

输入模式下（添加/编辑）：

| 按键 | 操作 |
|---|---|
| `Enter` | 确认输入 |
| `Esc` | 取消输入 |

## 库使用方式

在 `Cargo.toml` 中添加 `oflow`：

```toml
[dependencies]
oflow = "0.4.0"
```

然后使用 [`TodoManager`](https://docs.rs/oflow) API：

```rust,ignore
use oflow::TodoManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = TodoManager::new().await?;

    manager.add("买牛奶").await?;
    manager.add("读一本书").await?;
    manager.finish("买牛奶").await?;

    for todo in manager.list() {
        println!("{}", todo);
    }

    manager.clean().await?;
    Ok(())
}
```

使用自定义数据库路径：

```rust,ignore
let mut manager = TodoManager::with_db_path("path/to/todos.db").await?;
```

## 数据存储

任务数据存储在：
- `todos.db` SQLite 数据库（用于持久化）
- `todos.toml` 文件（用于 TOML 导出）
- 发布模式下：系统特定的数据目录（如 Linux 上的 `~/.local/share/oflow/`）
- 调试模式下：当前工作目录
- 可通过 `--data-dir <路径>` 或 `TodoManager::with_db_path()` 覆盖

## 项目结构

```
src/
├── main.rs          # CLI 入口
├── lib.rs           # 库根模块，TodoManager API
├── commands.rs      # CLI 参数解析（clap）
├── models.rs        # Todo 和 TodoList 数据类型
├── database.rs      # SQLite 持久化（同步、加载、查询）
└── tui/
    ├── mod.rs       # TUI 模块根
    ├── app.rs       # TUI 入口（run_tui）
    ├── state.rs     # 应用状态与事件处理
    └── ui.rs        # Ratatui 渲染
migrations/          # SQLx 数据库迁移
```

## 语言

- [English](README.md)
- [中文](README_CN.md)

## 许可证

本项目基于 Mozilla Public License 2.0 许可证开源。详见 [LICENSE](LICENSE) 文件。
