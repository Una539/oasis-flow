use std::collections::BTreeMap;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct Todo {
    pub id: uuid::Uuid,
    content: String,
    created_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    finished: bool,
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

    pub fn finished(&mut self) -> &str{
        self.finished = true;
        self.finished_at = Some(Utc::now());
        &self.content
    }

    pub fn edit(&mut self, new_content: String) -> &str{
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

    pub fn add(&mut self, todo: Todo) {
        self.todos.insert(todo.id.to_string(), todo);
    }

    pub fn remove(&mut self, id: String) {
        self.todos.remove(&id);
    }
}
