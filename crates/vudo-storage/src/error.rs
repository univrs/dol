//! Error types for storage operations.

use thiserror::Error;

/// Result type for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;

/// Storage error types.
#[derive(Error, Debug)]
pub enum StorageError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// Document not found.
    #[error("Document not found: {namespace}/{id}")]
    NotFound {
        /// Document namespace.
        namespace: String,
        /// Document ID.
        id: String,
    },

    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Storage is full or quota exceeded.
    #[error("Storage quota exceeded")]
    QuotaExceeded,

    /// Concurrent modification error.
    #[error("Concurrent modification detected")]
    ConcurrentModification,

    /// Unsupported feature.
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = StorageError::NotFound {
            namespace: "users".to_string(),
            id: "alice".to_string(),
        };
        assert_eq!(err.to_string(), "Document not found: users/alice");
    }

    #[test]
    fn test_serialization_error() {
        let err = StorageError::Serialization("test error".to_string());
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = StorageError::from(io_err);
        assert!(err.to_string().contains("IO error"));
    }
}
