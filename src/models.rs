use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::error::Error;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, FromRow, Clone)]
pub struct Todo {
    pub id: uuid::Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub finished: bool,
}

impl Todo {
    pub fn new(content: String) -> Todo {
        Todo {
            id: Uuid::new_v4(),
            content,
            created_at: Utc::now(),
            finished_at: None,
            finished: false,
        }
    }

    pub fn finished(&mut self) -> &str {
        self.finished = true;
        self.finished_at = Some(Utc::now());
        &self.content
    }

    pub fn edit(&mut self, new_content: String) -> &str {
        self.content = new_content;
        &self.content
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TodoList {
    #[serde(flatten)]
    pub todos: BTreeMap<String, Todo>,
}

impl TodoList {
    pub fn new() -> TodoList {
        TodoList {
            todos: BTreeMap::new(),
        }
    }

    pub fn read_from_file(file_path: &str) -> Result<TodoList, Box<dyn Error>> {
        let tdlist = if std::path::Path::new(file_path).exists() {
            let content = std::fs::read_to_string(file_path)?;
            toml::from_str(&content).unwrap_or_else(|_| TodoList::new())
        } else {
            TodoList::new()
        };

        Ok(tdlist)
    }

    pub fn write_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let tdlist = self;

        let toml_str = toml::to_string_pretty(tdlist)?;
        std::fs::write(file_path, toml_str)?;
        Ok(())
    }

    pub async fn add_todo_db(
        &mut self,
        content: String,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(task) = TodoList::find_by_content(pool, &content).await? {
            println!("任务已存在: {}", task.get_content());
        } else {
            println!("创建任务: {}", content);
            let new_todo = Todo::new(content.clone());
            self.todos.insert(new_todo.id.to_string(), new_todo);
        }
        Ok(())
    }

    pub async fn finish_todo_db(
        &mut self,
        content: String,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(task_from_db) = TodoList::find_by_content(pool, &content).await? {
            let id_str = task_from_db.id.to_string();
            if let Some(todo) = self.todos.get_mut(&id_str) {
                todo.finished();
                println!("完成任务:{}", todo.content);
            }
        } else {
            println!("没有该任务:{}", content);
        }
        Ok(())
    }

    pub async fn edit_todo_db(
        &mut self,
        before: String,
        after: String,
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(task_from_db) = TodoList::find_by_content(pool, &before).await? {
            let id_str = task_from_db.id.to_string();
            if let Some(todo) = self.todos.get_mut(&id_str) {
                todo.edit(after);
                println!("修改为:{}", todo.content);
            }
        } else {
            println!("没有该任务:{}", before);
        }
        Ok(())
    }

    pub async fn clean_todo_db(
        &mut self,
        pool: &sqlx::SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::Row;

        // 执行删除并返回被删掉的 ID 字符串
        let rows = sqlx::query("DELETE FROM todos WHERE finished = 1 RETURNING id")
            .fetch_all(pool)
            .await?;

        if rows.is_empty() {
            println!("Nothing to delete!");
            return Ok(());
        }

        for row in rows {
            // 2. 告诉 sqlx 按照 Uuid 类型解码（这会自动处理 BLOB）
            let id: Uuid = row.try_get(0)?;

            // 3. 转为 String 匹配你的 BTreeMap Key
            let id_str = id.to_string();
            if let Some(removed_todo) = self.todos.remove(&id_str) {
                // 3. 在这里打印日志
                println!(
                    "Successfully cleaned task: [{}] \"{}\" (Finished at: {:?})",
                    id_str, removed_todo.content, removed_todo.finished_at
                );
            } else {
                // 如果数据库里删了但内存里没找到（理论上不应该发生，除非状态不同步）
                println!(
                    "Warning: Task ID {} was deleted from DB but not found in memory map.",
                    id_str
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod performance_test {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use std::time::Instant;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                finished_at TEXT,
                finished INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    fn create_large_todo_list(count: usize) -> TodoList {
        let mut tdlist = TodoList::new();
        for i in 0..count {
            let todo = Todo::new(format!("Test task {}", i));
            tdlist.todos.insert(todo.id.to_string(), todo);
        }
        tdlist
    }

    #[tokio::test]
    async fn test_add_10000_todos_performance() {
        let pool = create_test_pool().await;
        let mut tdlist = TodoList::new();
        let start = Instant::now();

        for i in 0..10000 {
            let content = format!("Performance test task {}", i);
            let new_todo = Todo::new(content);
            tdlist.todos.insert(new_todo.id.to_string(), new_todo);
        }

        tdlist.sync_to_db(&pool).await.unwrap();

        let elapsed = start.elapsed();
        println!("Adding 10000 todos took: {:?}", elapsed);
        assert!(elapsed.as_secs() < 30, "Adding 10000 todos took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_finish_10000_todos_performance() {
        let pool = create_test_pool().await;
        let mut tdlist = create_large_todo_list(10000);

        tdlist.sync_to_db(&pool).await.unwrap();

        let start = Instant::now();

        for (_, todo) in tdlist.todos.iter_mut() {
            todo.finished();
        }

        tdlist.sync_to_db(&pool).await.unwrap();

        let elapsed = start.elapsed();
        println!("Finishing 10000 todos took: {:?}", elapsed);
        assert!(elapsed.as_secs() < 30, "Finishing 10000 todos took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_clean_10000_completed_todos_performance() {
        let pool = create_test_pool().await;
        let mut tdlist = TodoList::new();

        for i in 0..10000 {
            let mut todo = Todo::new(format!("Task to clean {}", i));
            todo.finished();
            tdlist.todos.insert(todo.id.to_string(), todo);
        }

        tdlist.sync_to_db(&pool).await.unwrap();

        let start = Instant::now();

        tdlist.clean_todo_db(&pool).await.unwrap();

        let elapsed = start.elapsed();
        println!("Cleaning 10000 completed todos took: {:?}", elapsed);
        assert!(elapsed.as_secs() < 30, "Cleaning 10000 todos took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_mixed_operations_10000_todos() {
        let pool = create_test_pool().await;
        let mut tdlist = create_large_todo_list(10000);

        tdlist.sync_to_db(&pool).await.unwrap();

        let start_add = Instant::now();
        for i in 0..1000 {
            let content = format!("Additional task {}", i);
            let new_todo = Todo::new(content);
            tdlist.todos.insert(new_todo.id.to_string(), new_todo);
        }
        tdlist.sync_to_db(&pool).await.unwrap();
        println!("Adding 1000 more todos took: {:?}", start_add.elapsed());

        let start_finish = Instant::now();
        for (_, todo) in tdlist.todos.iter_mut().take(5000) {
            todo.finished();
        }
        tdlist.sync_to_db(&pool).await.unwrap();
        println!("Finishing 5000 todos took: {:?}", start_finish.elapsed());

        let start_clean = Instant::now();
        tdlist.clean_todo_db(&pool).await.unwrap();
        println!("Cleaning 5000 completed todos took: {:?}", start_clean.elapsed());

        let remaining = tdlist.todos.len();
        println!("Remaining todos after operations: {}", remaining);
        assert_eq!(remaining, 6000, "Expected 6000 remaining todos");
    }
}
