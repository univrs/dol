//! Error types for VUDO state management.

use thiserror::Error;

/// Result type alias for state operations.
pub type Result<T> = std::result::Result<T, StateError>;

/// Error types for state management operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StateError {
    /// Document not found in the store.
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Document already exists.
    #[error("Document already exists: {0}")]
    DocumentAlreadyExists(String),

    /// Invalid document ID format.
    #[error("Invalid document ID: {0}")]
    InvalidDocumentId(String),

    /// Transaction failed.
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Transaction conflict (optimistic concurrency control).
    #[error("Transaction conflict: {0}")]
    TransactionConflict(String),

    /// Automerge error.
    #[error("Automerge error: {0}")]
    AutomergeError(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Invalid path for subscription.
    #[error("Invalid subscription path: {0}")]
    InvalidPath(String),

    /// Subscription not found.
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    /// Operation queue error.
    #[error("Operation queue error: {0}")]
    OperationQueueError(String),

    /// Snapshot error.
    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Lock poisoned.
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    /// Schema not found error.
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),
}

impl From<automerge::AutomergeError> for StateError {
    fn from(err: automerge::AutomergeError) -> Self {
        StateError::AutomergeError(err.to_string())
    }
}

impl From<serde_json::Error> for StateError {
    fn from(err: serde_json::Error) -> Self {
        StateError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for StateError {
    fn from(err: std::io::Error) -> Self {
        StateError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = StateError::DocumentNotFound("test-doc".to_string());
        assert_eq!(err.to_string(), "Document not found: test-doc");
    }

    // Note: Automerge error conversion is tested indirectly through document operations

    #[test]
    fn test_error_from_serde() {
        let serde_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .unwrap_err();
        let state_err: StateError = serde_err.into();
        assert!(matches!(state_err, StateError::SerializationError(_)));
    }

    #[test]
    fn test_error_clone() {
        let err1 = StateError::DocumentNotFound("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
