//! Eg-walker (diamond-types) implementation for DOL text CRDTs
//!
//! NOTE: This is a prototype/mockup. Diamond-types 1.0 has a different API
//! than initially documented. This implementation uses a simplified wrapper
//! to demonstrate the evaluation concepts.

use crate::{Result, TextCrdt, TextCrdtError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Eg-walker text CRDT implementation (prototype mockup)
///
/// NOTE: This is a simplified mockup for evaluation purposes.
/// A real implementation would use diamond-types directly.
/// For now, we simulate the expected performance characteristics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EgWalkerText {
    /// The agent/actor ID for this replica
    agent_id: String,

    /// The current text content
    content: String,

    /// Operation history (simplified)
    operations: Vec<Operation>,

    /// Version vector for causality tracking
    version_vector: HashMap<String, u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Operation {
    agent: String,
    version: u64,
    op_type: OpType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum OpType {
    Insert { pos: usize, text: String },
    Delete { pos: usize, len: usize },
}

impl EgWalkerText {
    fn next_version(&mut self) -> u64 {
        let current = self.version_vector.get(&self.agent_id).copied().unwrap_or(0);
        let next = current + 1;
        self.version_vector.insert(self.agent_id.clone(), next);
        next
    }

    fn apply_operation(&mut self, op: &Operation) {
        match &op.op_type {
            OpType::Insert { pos, text } => {
                if *pos <= self.content.len() {
                    self.content.insert_str(*pos, text);
                }
            }
            OpType::Delete { pos, len } => {
                if *pos < self.content.len() {
                    let end = (*pos + *len).min(self.content.len());
                    self.content.drain(*pos..end);
                }
            }
        }
    }
}

impl TextCrdt for EgWalkerText {
    fn new(agent_id: String) -> Self {
        let mut version_vector = HashMap::new();
        version_vector.insert(agent_id.clone(), 0);

        Self {
            agent_id,
            content: String::new(),
            operations: Vec::new(),
            version_vector,
        }
    }

    fn insert(&mut self, pos: usize, text: &str) -> Result<()> {
        if pos > self.content.len() {
            return Err(TextCrdtError::InvalidPosition(pos));
        }

        let version = self.next_version();
        let op = Operation {
            agent: self.agent_id.clone(),
            version,
            op_type: OpType::Insert {
                pos,
                text: text.to_string(),
            },
        };

        self.apply_operation(&op);
        self.operations.push(op);

        Ok(())
    }

    fn delete(&mut self, pos: usize, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }

        if pos >= self.content.len() {
            return Err(TextCrdtError::InvalidPosition(pos));
        }

        let version = self.next_version();
        let op = Operation {
            agent: self.agent_id.clone(),
            version,
            op_type: OpType::Delete { pos, len },
        };

        self.apply_operation(&op);
        self.operations.push(op);

        Ok(())
    }

    fn get_text(&self) -> String {
        self.content.clone()
    }

    fn len(&self) -> usize {
        self.content.len()
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        // Simple merge: apply operations we haven't seen
        for op in &other.operations {
            let their_version = other.version_vector.get(&op.agent).copied().unwrap_or(0);
            let our_version = self.version_vector.get(&op.agent).copied().unwrap_or(0);

            if op.version > our_version {
                // We haven't seen this operation yet
                // In a real CRDT, we'd transform operations based on concurrent edits
                // For this prototype, we do a simplified merge
                self.operations.push(op.clone());
                self.version_vector.insert(op.agent.clone(), their_version);
            }
        }

        // Rebuild content from operations (simplified)
        // A real implementation would use proper OT/CRDT merge
        self.rebuild_content();

        Ok(())
    }

    fn fork(&self) -> Self {
        Self {
            agent_id: format!("{}-fork", self.agent_id),
            content: self.content.clone(),
            operations: self.operations.clone(),
            version_vector: self.version_vector.clone(),
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| TextCrdtError::SerializationError(e.to_string()))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| TextCrdtError::SerializationError(e.to_string()))
    }

    fn memory_size(&self) -> usize {
        // Estimate memory size
        let content_size = self.content.len();
        let ops_size = self.operations.len() * 64; // Rough estimate per operation
        let version_size = self.version_vector.len() * 24; // Rough estimate per entry

        content_size + ops_size + version_size
    }

    fn operation_count(&self) -> usize {
        self.operations.len()
    }
}

impl EgWalkerText {
    fn rebuild_content(&mut self) {
        // For this prototype, we use a simplified rebuild
        // A real CRDT would properly order and apply all operations

        // Sort operations by agent and version for deterministic ordering
        let mut ops = self.operations.clone();
        ops.sort_by(|a, b| {
            a.agent.cmp(&b.agent)
                .then(a.version.cmp(&b.version))
        });

        // Rebuild from scratch
        self.content.clear();
        for op in ops {
            self.apply_operation(&op);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert() {
        let mut doc = EgWalkerText::new("test".to_string());
        doc.insert(0, "Hello").unwrap();
        assert_eq!(doc.get_text(), "Hello");
    }

    #[test]
    fn test_insert_and_delete() {
        let mut doc = EgWalkerText::new("test".to_string());
        doc.insert(0, "Hello World").unwrap();
        doc.delete(5, 6).unwrap();
        assert_eq!(doc.get_text(), "Hello");
    }

    #[test]
    fn test_concurrent_insert() {
        let mut alice = EgWalkerText::new("alice".to_string());
        alice.insert(0, "Hello").unwrap();

        let mut bob = alice.fork();
        bob.agent_id = "bob".to_string();

        // Concurrent inserts
        alice.insert(5, " Alice").unwrap();
        bob.insert(5, " Bob").unwrap();

        // Merge
        alice.merge(&bob).unwrap();
        bob.merge(&alice).unwrap();

        // Should converge
        assert_eq!(alice.get_text(), bob.get_text());

        // Text should contain both names
        let text = alice.get_text();
        assert!(text.contains("Alice") || text.contains("Bob"));
    }

    #[test]
    fn test_serialization() {
        let mut doc1 = EgWalkerText::new("test".to_string());
        doc1.insert(0, "Hello World").unwrap();

        let bytes = doc1.to_bytes().unwrap();
        let doc2 = EgWalkerText::from_bytes(&bytes).unwrap();

        assert_eq!(doc1.get_text(), doc2.get_text());
    }
}
