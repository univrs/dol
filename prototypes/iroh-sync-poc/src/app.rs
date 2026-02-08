use anyhow::Result;
use automerge::{transaction::Transactable, AutoCommit, ObjType, ReadDoc, ScalarValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use crate::p2p::IrohNode;
use crate::sync::AutomergeSync;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

/// TodoApp demonstrates P2P sync of a todo list using Automerge CRDT
pub struct TodoApp {
    node_name: String,
    doc: AutoCommit,
    sync: AutomergeSync,
    todos: HashMap<String, Todo>,
}

impl TodoApp {
    pub fn new(node_name: String) -> Self {
        let doc = AutoCommit::new();
        let sync = AutomergeSync::new();

        Self {
            node_name,
            doc,
            sync,
            todos: HashMap::new(),
        }
    }

    /// Add a new todo item
    pub fn add_todo(&mut self, title: String) -> Result<String> {
        let id = format!("todo-{}", uuid::Uuid::new_v4().to_string());

        // Update Automerge document
        let todos_obj = match self.doc.get(automerge::ROOT, "todos")? {
            Some((automerge::Value::Object(ObjType::Map), obj_id)) => obj_id,
            _ => {
                // Create todos map if it doesn't exist
                self.doc
                    .put_object(automerge::ROOT, "todos", ObjType::Map)?
            }
        };

        let todo_obj = self.doc.put_object(&todos_obj, &id, ObjType::Map)?;
        self.doc
            .put(&todo_obj, "id", ScalarValue::Str(id.clone().into()))?;
        self.doc
            .put(&todo_obj, "title", ScalarValue::Str(title.clone().into()))?;
        self.doc
            .put(&todo_obj, "completed", ScalarValue::Boolean(false))?;

        // Update local cache
        let todo = Todo {
            id: id.clone(),
            title,
            completed: false,
        };
        self.todos.insert(id.clone(), todo);

        info!("[{}] Added todo: {}", self.node_name, id);
        Ok(id)
    }

    /// Toggle todo completion status
    pub fn toggle_todo(&mut self, id: &str) -> Result<()> {
        if let Some(todo) = self.todos.get_mut(id) {
            todo.completed = !todo.completed;

            // Update Automerge document
            let todos_obj = match self.doc.get(automerge::ROOT, "todos")? {
                Some((automerge::Value::Object(ObjType::Map), obj_id)) => obj_id,
                _ => anyhow::bail!("Todos object not found"),
            };

            let todo_obj = match self.doc.get(&todos_obj, id)? {
                Some((automerge::Value::Object(ObjType::Map), obj_id)) => obj_id,
                _ => anyhow::bail!("Todo {} not found", id),
            };

            self.doc
                .put(&todo_obj, "completed", ScalarValue::Boolean(todo.completed))?;

            info!(
                "[{}] Toggled todo {}: completed={}",
                self.node_name, id, todo.completed
            );
        }

        Ok(())
    }

    /// Get all todos
    pub fn list_todos(&self) -> Vec<&Todo> {
        let mut todos: Vec<&Todo> = self.todos.values().collect();
        todos.sort_by_key(|t| &t.id);
        todos
    }

    /// Sync with remote peer
    async fn sync_with_peer(&mut self, node: &IrohNode) -> Result<()> {
        // Get sync message from local document
        let sync_msg = self.sync.generate_sync_message(&mut self.doc)?;

        // Send to peers
        node.broadcast_sync_message(&sync_msg).await?;

        // Receive and apply sync messages from peers
        while let Ok(remote_msg) = node.receive_sync_message().await {
            match self.sync.apply_sync_message(&mut self.doc, &remote_msg) {
                Ok(changed) => {
                    if changed {
                        self.reload_from_doc()?;
                        info!("[{}] Applied sync from peer", self.node_name);
                    }
                }
                Err(e) => {
                    warn!("[{}] Failed to apply sync: {}", self.node_name, e);
                }
            }
        }

        Ok(())
    }

    /// Reload local cache from Automerge document
    fn reload_from_doc(&mut self) -> Result<()> {
        self.todos.clear();

        if let Some((automerge::Value::Object(ObjType::Map), todos_obj)) =
            self.doc.get(automerge::ROOT, "todos")?
        {
            for key in self.doc.keys(&todos_obj) {
                if let Some((automerge::Value::Object(ObjType::Map), todo_obj)) =
                    self.doc.get(&todos_obj, &key)?
                {
                    let id = self.get_string(&todo_obj, "id")?;
                    let title = self.get_string(&todo_obj, "title")?;
                    let completed = self.get_bool(&todo_obj, "completed")?;

                    let todo = Todo {
                        id: id.clone(),
                        title,
                        completed,
                    };
                    self.todos.insert(id, todo);
                }
            }
        }

        Ok(())
    }

    fn get_string(&self, obj: &automerge::ObjId, key: &str) -> Result<String> {
        match self.doc.get(obj, key)? {
            Some((automerge::Value::Scalar(v), _)) => match v.as_ref() {
                ScalarValue::Str(s) => Ok(s.to_string()),
                _ => anyhow::bail!("Expected string for {}", key),
            },
            _ => anyhow::bail!("Key {} not found", key),
        }
    }

    fn get_bool(&self, obj: &automerge::ObjId, key: &str) -> Result<bool> {
        match self.doc.get(obj, key)? {
            Some((automerge::Value::Scalar(v), _)) => match v.as_ref() {
                ScalarValue::Boolean(b) => Ok(*b),
                _ => anyhow::bail!("Expected boolean for {}", key),
            },
            _ => anyhow::bail!("Key {} not found", key),
        }
    }

    /// Main run loop
    pub async fn run(&mut self, node: IrohNode) -> Result<()> {
        // Demo: Add some todos
        self.add_todo(format!("Task from {}", self.node_name))?;

        info!("[{}] Starting sync loop...", self.node_name);

        // Main sync loop
        loop {
            // Sync with peers
            if let Err(e) = self.sync_with_peer(&node).await {
                warn!("[{}] Sync error: {}", self.node_name, e);
            }

            // Print current state
            info!("[{}] Current todos:", self.node_name);
            for todo in self.list_todos() {
                info!(
                    "  - [{}] {}",
                    if todo.completed { "x" } else { " " },
                    todo.title
                );
            }

            // Wait before next sync
            sleep(Duration::from_secs(5)).await;
        }
    }
}

// Add uuid dependency
mod uuid {
    use std::fmt;

    pub struct Uuid([u8; 16]);

    impl Uuid {
        pub fn new_v4() -> Self {
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let mut bytes = [0u8; 16];
            bytes[..8].copy_from_slice(&nanos.to_le_bytes()[..8]);
            bytes[8..].copy_from_slice(&nanos.to_le_bytes()[8..]);

            Self(bytes)
        }

        pub fn to_string(&self) -> String {
            format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3],
                self.0[4], self.0[5],
                self.0[6], self.0[7],
                self.0[8], self.0[9],
                self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15]
            )
        }
    }

    impl fmt::Display for Uuid {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.to_string())
        }
    }
}
