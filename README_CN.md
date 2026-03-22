# Oasis Flow (oflow)

一个基于 Rust 开发的待办事项 CLI 工具。

## 功能特性

- **添加任务**：快速添加新的待办事项
- **完成任务**：将任务标记为已完成
- **编辑任务**：修改现有任务的内容
- **清理**：删除所有已完成的任务

## 安装

```bash
cargo install oflow
```

## 使用方法

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

## 数据存储

任务数据存储在：
- 当前目录下的 `todos.toml` 文件
- `todos.db` SQLite 数据库（用于持久化）

## 语言

- [English](README.md)
- [中文](README_CN.md)

## 许可证

本项目基于 Mozilla Public License 2.0 许可证开源。详见 [LICENSE](LICENSE) 文件。
