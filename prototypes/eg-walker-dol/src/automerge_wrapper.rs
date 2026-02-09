//! Automerge text CRDT wrapper for comparison benchmarking

use crate::{Result, TextCrdt, TextCrdtError};
use automerge::{AutoCommit, ObjType, ReadDoc, ROOT};
use std::cell::RefCell;

/// Automerge text CRDT implementation for benchmarking comparison
///
/// Uses interior mutability (RefCell) to work with the immutable trait methods
#[derive(Clone, Debug)]
pub struct AutomergeText {
    /// The agent/actor ID for this replica
    agent_id: String,

    /// The Automerge document (with interior mutability)
    doc: RefCell<AutoCommit>,
}

impl AutomergeText {
    const TEXT_KEY: &'static str = "text";

    /// Get or create the text object
    fn get_text_obj(&self) -> Result<automerge::ObjId> {
        let doc = self.doc.borrow();
        match doc.get(ROOT, Self::TEXT_KEY) {
            Ok(Some((_, Some(obj_id)))) => Ok(obj_id),
            Ok(Some((_, None))) => Err(TextCrdtError::AutomergeError(
                "Text key exists but has no object ID".to_string()
            )),
            Ok(None) => Err(TextCrdtError::AutomergeError(
                "Text object not initialized".to_string()
            )),
            Err(e) => Err(TextCrdtError::AutomergeError(format!("{:?}", e))),
        }
    }

    /// Initialize the text object if it doesn't exist
    fn ensure_text_obj(&self) -> Result<automerge::ObjId> {
        let mut doc = self.doc.borrow_mut();
        match doc.get(ROOT, Self::TEXT_KEY) {
            Ok(Some((_, Some(obj_id)))) => Ok(obj_id),
            _ => {
                // Create new text object
                let obj_id = doc
                    .put_object(ROOT, Self::TEXT_KEY, ObjType::Text)
                    .map_err(|e| TextCrdtError::AutomergeError(format!("{:?}", e)))?;
                Ok(obj_id)
            }
        }
    }
}

impl TextCrdt for AutomergeText {
    fn new(agent_id: String) -> Self {
        let mut doc = AutoCommit::new();
        doc.set_actor(agent_id.as_bytes().try_into().unwrap_or([0u8; 16]));

        let text = Self {
            agent_id,
            doc: RefCell::new(doc),
        };

        // Initialize text object
        let _ = text.ensure_text_obj();

        text
    }

    fn insert(&mut self, pos: usize, text: &str) -> Result<()> {
        let obj_id = self.ensure_text_obj()?;

        self.doc
            .borrow_mut()
            .splice_text(&obj_id, pos, 0, text)
            .map_err(|e| TextCrdtError::AutomergeError(format!("{:?}", e)))?;

        Ok(())
    }

    fn delete(&mut self, pos: usize, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }

        let obj_id = self.get_text_obj()?;

        self.doc
            .borrow_mut()
            .splice_text(&obj_id, pos, len as isize, "")
            .map_err(|e| TextCrdtError::AutomergeError(format!("{:?}", e)))?;

        Ok(())
    }

    fn get_text(&self) -> String {
        let obj_id = match self.get_text_obj() {
            Ok(id) => id,
            Err(_) => return String::new(),
        };

        self.doc.borrow().text(&obj_id).unwrap_or_default()
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        let mut other_doc = other.doc.borrow().clone();
        self.doc
            .borrow_mut()
            .merge(&mut other_doc)
            .map_err(|e| TextCrdtError::MergeError(format!("{:?}", e)))?;
        Ok(())
    }

    fn fork(&self) -> Self {
        let forked_doc = self.doc.borrow().fork();
        Self {
            agent_id: self.agent_id.clone(),
            doc: RefCell::new(forked_doc),
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.doc.borrow().save())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let doc = AutoCommit::load(bytes)
            .map_err(|e| TextCrdtError::SerializationError(format!("{:?}", e)))?;

        Ok(Self {
            agent_id: "restored".to_string(),
            doc: RefCell::new(doc),
        })
    }

    fn memory_size(&self) -> usize {
        // Approximate memory size based on saved bytes
        self.doc.borrow().save().len()
    }

    fn operation_count(&self) -> usize {
        // Automerge doesn't directly expose operation count
        // Approximate based on document size
        self.doc.borrow().length(ROOT)
    }
}

// Note: Serialization disabled for AutomergeText in prototype
// (Automerge has complex internal state that's hard to serialize)
// In production, use Automerge's native save/load methods

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert() {
        let mut doc = AutomergeText::new("test".to_string());
        doc.insert(0, "Hello").unwrap();
        assert_eq!(doc.get_text(), "Hello");
    }

    #[test]
    fn test_insert_and_delete() {
        let mut doc = AutomergeText::new("test".to_string());
        doc.insert(0, "Hello World").unwrap();
        doc.delete(5, 6).unwrap();
        assert_eq!(doc.get_text(), "Hello");
    }

    #[test]
    fn test_concurrent_insert() {
        let mut alice = AutomergeText::new("alice".to_string());
        alice.insert(0, "Hello").unwrap();

        let mut bob = alice.fork();

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
        assert!(text.contains("Alice"));
        assert!(text.contains("Bob"));
    }

    #[test]
    fn test_serialization() {
        let mut doc1 = AutomergeText::new("test".to_string());
        doc1.insert(0, "Hello World").unwrap();

        let bytes = doc1.to_bytes().unwrap();
        let doc2 = AutomergeText::from_bytes(&bytes).unwrap();

        assert_eq!(doc1.get_text(), doc2.get_text());
    }
}
