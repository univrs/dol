//! Operation types for queue persistence.
//!
//! These types mirror the operation types from vudo-state but are simplified
//! for storage purposes.

use serde::{Deserialize, Serialize};

/// A persisted operation from the operation queue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Operation {
    /// Operation ID.
    pub id: u64,
    /// Document namespace.
    pub namespace: String,
    /// Document ID.
    pub document_id: String,
    /// Operation type.
    pub op_type: OperationType,
    /// Timestamp (Unix epoch milliseconds).
    pub timestamp: u64,
    /// Idempotency key (for deduplication).
    pub idempotency_key: Option<String>,
    /// Number of retry attempts.
    pub retry_count: u32,
}

impl Operation {
    /// Create a new operation.
    pub fn new(
        id: u64,
        namespace: impl Into<String>,
        document_id: impl Into<String>,
        op_type: OperationType,
    ) -> Self {
        Self {
            id,
            namespace: namespace.into(),
            document_id: document_id.into(),
            op_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            idempotency_key: None,
            retry_count: 0,
        }
    }

    /// Create a new operation with an idempotency key.
    pub fn with_idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }
}

/// Operation type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OperationType {
    /// Create a new document.
    Create,
    /// Update an existing document.
    Update {
        /// Automerge change bytes.
        change_bytes: Vec<u8>,
    },
    /// Delete a document.
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_new() {
        let op = Operation::new(1, "users", "alice", OperationType::Create);
        assert_eq!(op.id, 1);
        assert_eq!(op.namespace, "users");
        assert_eq!(op.document_id, "alice");
        assert!(matches!(op.op_type, OperationType::Create));
        assert!(op.idempotency_key.is_none());
        assert_eq!(op.retry_count, 0);
    }

    #[test]
    fn test_operation_with_idempotency_key() {
        let op = Operation::new(1, "users", "alice", OperationType::Create)
            .with_idempotency_key("create-alice");
        assert_eq!(op.idempotency_key, Some("create-alice".to_string()));
    }

    #[test]
    fn test_operation_serialization() {
        let op = Operation::new(1, "users", "alice", OperationType::Create);
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: Operation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_operation_type_update() {
        let op_type = OperationType::Update {
            change_bytes: vec![1, 2, 3],
        };
        let op = Operation::new(1, "users", "alice", op_type.clone());

        if let OperationType::Update { change_bytes } = op.op_type {
            assert_eq!(change_bytes, vec![1, 2, 3]);
        } else {
            panic!("Expected Update operation");
        }
    }

    #[test]
    fn test_operation_type_equality() {
        let op1 = OperationType::Create;
        let op2 = OperationType::Create;
        assert_eq!(op1, op2);

        let op3 = OperationType::Delete;
        assert_ne!(op1, op3);
    }
}
