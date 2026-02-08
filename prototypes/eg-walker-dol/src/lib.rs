//! Eg-walker integration prototype for DOL text types
//!
//! This crate evaluates the eg-walker algorithm (via diamond-types) as a potential
//! replacement for Automerge in DOL's peritext/text CRDT implementation.
//!
//! ## Architecture
//!
//! The prototype provides:
//! - `EgWalkerText`: A wrapper around diamond-types for DOL text operations
//! - `AutomergeText`: A comparable wrapper around Automerge for benchmarking
//! - Performance benchmarks comparing both implementations
//! - Correctness tests validating CRDT properties
//!
//! ## Evaluation Criteria
//!
//! 1. **Performance**: Insert/delete latency, merge speed, memory usage
//! 2. **Correctness**: Convergence, commutativity, causality preservation
//! 3. **API Ergonomics**: Ease of use, integration complexity
//! 4. **Maintenance**: Library maturity, ecosystem support

mod egwalker;
mod automerge_wrapper;
mod correctness;
mod benchmarks;

pub use egwalker::EgWalkerText;
pub use automerge_wrapper::AutomergeText;
pub use correctness::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextCrdtError {
    #[error("Invalid position: {0}")]
    InvalidPosition(usize),

    #[error("Empty document")]
    EmptyDocument,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Merge error: {0}")]
    MergeError(String),

    #[error("Diamond-types error: {0}")]
    DiamondTypesError(String),

    #[error("Automerge error: {0}")]
    AutomergeError(String),
}

pub type Result<T> = std::result::Result<T, TextCrdtError>;

/// Common trait for text CRDT implementations
pub trait TextCrdt: Clone + Send + Sync {
    /// Create a new empty document with the given agent/actor ID
    fn new(agent_id: String) -> Self;

    /// Insert text at the given position
    fn insert(&mut self, pos: usize, text: &str) -> Result<()>;

    /// Delete text at the given position with the given length
    fn delete(&mut self, pos: usize, len: usize) -> Result<()>;

    /// Get the current text content
    fn get_text(&self) -> String;

    /// Get the length of the text
    fn len(&self) -> usize {
        self.get_text().len()
    }

    /// Check if the document is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Merge changes from another replica
    fn merge(&mut self, other: &Self) -> Result<()>;

    /// Fork this document to create a new replica
    fn fork(&self) -> Self;

    /// Serialize the document to bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Deserialize the document from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>;

    /// Get the memory size in bytes (approximate)
    fn memory_size(&self) -> usize;

    /// Get the operation count
    fn operation_count(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_basic_operations<T: TextCrdt>() {
        let mut doc = T::new("agent1".to_string());

        // Insert text
        doc.insert(0, "Hello").unwrap();
        assert_eq!(doc.get_text(), "Hello");

        // Append text
        doc.insert(5, " World").unwrap();
        assert_eq!(doc.get_text(), "Hello World");

        // Insert in middle
        doc.insert(5, " Beautiful").unwrap();
        assert_eq!(doc.get_text(), "Hello Beautiful World");

        // Delete text
        doc.delete(6, 10).unwrap();
        assert_eq!(doc.get_text(), "Hello  World");
    }

    fn test_concurrent_edits<T: TextCrdt>() {
        let mut alice = T::new("alice".to_string());
        let mut bob = alice.fork();

        // Initial state
        alice.insert(0, "Hello").unwrap();
        bob.merge(&alice).unwrap();

        // Concurrent edits
        alice.insert(5, " Alice").unwrap();
        bob.insert(5, " Bob").unwrap();

        // Merge
        alice.merge(&bob).unwrap();
        bob.merge(&alice).unwrap();

        // Should converge
        assert_eq!(alice.get_text(), bob.get_text());
    }

    fn test_serialization<T: TextCrdt>() {
        let mut doc1 = T::new("agent1".to_string());
        doc1.insert(0, "Hello World").unwrap();

        // Serialize and deserialize
        let bytes = doc1.to_bytes().unwrap();
        let doc2 = T::from_bytes(&bytes).unwrap();

        assert_eq!(doc1.get_text(), doc2.get_text());
    }

    #[test]
    fn test_egwalker_basic() {
        test_basic_operations::<EgWalkerText>();
    }

    #[test]
    fn test_egwalker_concurrent() {
        test_concurrent_edits::<EgWalkerText>();
    }

    #[test]
    fn test_egwalker_serialization() {
        test_serialization::<EgWalkerText>();
    }

    #[test]
    fn test_automerge_basic() {
        test_basic_operations::<AutomergeText>();
    }

    #[test]
    fn test_automerge_concurrent() {
        test_concurrent_edits::<AutomergeText>();
    }

    #[test]
    fn test_automerge_serialization() {
        test_serialization::<AutomergeText>();
    }
}
