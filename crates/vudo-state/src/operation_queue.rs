//! Operation queue for offline mutation tracking and sync.

use crate::document_store::DocumentId;
use crate::error::{Result, StateError};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Operation ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(u64);

impl OperationId {
    /// Generate a new operation ID.
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Operation type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    /// Create a new document.
    Create {
        /// Document ID.
        document_id: DocumentId,
    },
    /// Update an existing document.
    Update {
        /// Document ID.
        document_id: DocumentId,
        /// Automerge change bytes.
        change_bytes: Vec<u8>,
    },
    /// Delete a document.
    Delete {
        /// Document ID.
        document_id: DocumentId,
    },
}

/// Operation metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation ID.
    pub id: OperationId,
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
    pub fn new(op_type: OperationType) -> Self {
        Self {
            id: OperationId::new(),
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
    pub fn new_with_key(op_type: OperationType, idempotency_key: String) -> Self {
        let mut op = Self::new(op_type);
        op.idempotency_key = Some(idempotency_key);
        op
    }

    /// Get the document ID for this operation.
    pub fn document_id(&self) -> &DocumentId {
        match &self.op_type {
            OperationType::Create { document_id } => document_id,
            OperationType::Update { document_id, .. } => document_id,
            OperationType::Delete { document_id } => document_id,
        }
    }
}

/// Operation queue for tracking offline mutations.
pub struct OperationQueue {
    /// FIFO queue of pending operations.
    queue: Arc<RwLock<VecDeque<Operation>>>,
    /// Map of idempotency keys to operation IDs (for deduplication).
    idempotency_map: Arc<RwLock<HashMap<String, OperationId>>>,
    /// Maximum queue size.
    max_size: usize,
}

impl OperationQueue {
    /// Create a new operation queue.
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            idempotency_map: Arc::new(RwLock::new(HashMap::new())),
            max_size: 10_000,
        }
    }

    /// Create a new operation queue with a maximum size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            idempotency_map: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    /// Enqueue an operation.
    pub fn enqueue(&self, operation: Operation) -> Result<OperationId> {
        let mut queue = self.queue.write();

        // Check queue size limit
        if queue.len() >= self.max_size {
            return Err(StateError::OperationQueueError(
                "Queue size limit exceeded".to_string(),
            ));
        }

        // Check for duplicate idempotency key
        if let Some(ref key) = operation.idempotency_key {
            let mut idempotency_map = self.idempotency_map.write();
            if let Some(&existing_id) = idempotency_map.get(key) {
                // Operation with this key already exists
                return Ok(existing_id);
            }
            idempotency_map.insert(key.clone(), operation.id);
        }

        let id = operation.id;
        queue.push_back(operation);
        Ok(id)
    }

    /// Dequeue the next operation.
    pub fn dequeue(&self) -> Option<Operation> {
        let mut queue = self.queue.write();
        let op = queue.pop_front()?;

        // Remove from idempotency map
        if let Some(ref key) = op.idempotency_key {
            self.idempotency_map.write().remove(key);
        }

        Some(op)
    }

    /// Peek at the next operation without removing it.
    pub fn peek(&self) -> Option<Operation> {
        self.queue.read().front().cloned()
    }

    /// Get the queue length.
    pub fn len(&self) -> usize {
        self.queue.read().len()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.read().is_empty()
    }

    /// Clear all operations.
    pub fn clear(&self) {
        self.queue.write().clear();
        self.idempotency_map.write().clear();
    }

    /// Get all operations (without removing them).
    pub fn list(&self) -> Vec<Operation> {
        self.queue.read().iter().cloned().collect()
    }

    /// Retry an operation (increment retry count and re-enqueue).
    pub fn retry(&self, mut operation: Operation) -> Result<OperationId> {
        operation.retry_count += 1;
        self.enqueue(operation)
    }

    /// Serialize the queue to bytes.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let queue = self.queue.read();
        let operations: Vec<Operation> = queue.iter().cloned().collect();
        serde_json::to_vec(&operations).map_err(StateError::from)
    }

    /// Deserialize the queue from bytes.
    pub fn deserialize(&self, bytes: &[u8]) -> Result<()> {
        let operations: Vec<Operation> = serde_json::from_slice(bytes)?;

        let mut queue = self.queue.write();
        let mut idempotency_map = self.idempotency_map.write();

        queue.clear();
        idempotency_map.clear();

        for op in operations {
            if let Some(ref key) = op.idempotency_key {
                idempotency_map.insert(key.clone(), op.id);
            }
            queue.push_back(op);
        }

        Ok(())
    }

    /// Filter operations by document ID.
    pub fn filter_by_document(&self, document_id: &DocumentId) -> Vec<Operation> {
        self.queue
            .read()
            .iter()
            .filter(|op| op.document_id() == document_id)
            .cloned()
            .collect()
    }

    /// Remove operations by document ID.
    pub fn remove_by_document(&self, document_id: &DocumentId) {
        let mut queue = self.queue.write();
        let mut idempotency_map = self.idempotency_map.write();

        queue.retain(|op| {
            if op.document_id() == document_id {
                if let Some(ref key) = op.idempotency_key {
                    idempotency_map.remove(key);
                }
                false
            } else {
                true
            }
        });
    }
}

impl Default for OperationQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_id() {
        let id1 = OperationId::new();
        let id2 = OperationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_operation_new() {
        let doc_id = DocumentId::new("users", "alice");
        let op_type = OperationType::Create { document_id: doc_id.clone() };
        let op = Operation::new(op_type);

        assert_eq!(op.document_id(), &doc_id);
        assert_eq!(op.retry_count, 0);
        assert!(op.idempotency_key.is_none());
    }

    #[test]
    fn test_operation_new_with_key() {
        let doc_id = DocumentId::new("users", "alice");
        let op_type = OperationType::Create { document_id: doc_id };
        let op = Operation::new_with_key(op_type, "create-alice".to_string());

        assert_eq!(op.idempotency_key, Some("create-alice".to_string()));
    }

    #[test]
    fn test_queue_enqueue_dequeue() {
        let queue = OperationQueue::new();
        let doc_id = DocumentId::new("users", "alice");
        let op_type = OperationType::Create { document_id: doc_id };
        let op = Operation::new(op_type);

        queue.enqueue(op.clone()).unwrap();
        assert_eq!(queue.len(), 1);

        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, op.id);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_peek() {
        let queue = OperationQueue::new();
        let doc_id = DocumentId::new("users", "alice");
        let op_type = OperationType::Create { document_id: doc_id };
        let op = Operation::new(op_type);

        queue.enqueue(op.clone()).unwrap();

        let peeked = queue.peek().unwrap();
        assert_eq!(peeked.id, op.id);
        assert_eq!(queue.len(), 1); // Peek doesn't remove
    }

    #[test]
    fn test_queue_idempotency() {
        let queue = OperationQueue::new();
        let doc_id = DocumentId::new("users", "alice");
        let op_type = OperationType::Create { document_id: doc_id.clone() };

        let op1 = Operation::new_with_key(op_type.clone(), "create-alice".to_string());
        let op2 = Operation::new_with_key(op_type, "create-alice".to_string());

        let id1 = queue.enqueue(op1).unwrap();
        let id2 = queue.enqueue(op2).unwrap();

        // Should return the same ID (deduplication)
        assert_eq!(id1, id2);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_queue_max_size() {
        let queue = OperationQueue::with_max_size(2);
        let doc_id = DocumentId::new("users", "alice");

        let op1 = Operation::new(OperationType::Create { document_id: doc_id.clone() });
        let op2 = Operation::new(OperationType::Create { document_id: doc_id.clone() });
        let op3 = Operation::new(OperationType::Create { document_id: doc_id });

        queue.enqueue(op1).unwrap();
        queue.enqueue(op2).unwrap();

        let result = queue.enqueue(op3);
        assert!(matches!(result, Err(StateError::OperationQueueError(_))));
    }

    #[test]
    fn test_queue_clear() {
        let queue = OperationQueue::new();
        let doc_id = DocumentId::new("users", "alice");
        let op = Operation::new(OperationType::Create { document_id: doc_id });

        queue.enqueue(op).unwrap();
        assert_eq!(queue.len(), 1);

        queue.clear();
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_list() {
        let queue = OperationQueue::new();
        let doc_id1 = DocumentId::new("users", "alice");
        let doc_id2 = DocumentId::new("users", "bob");

        let op1 = Operation::new(OperationType::Create { document_id: doc_id1 });
        let op2 = Operation::new(OperationType::Create { document_id: doc_id2 });

        queue.enqueue(op1.clone()).unwrap();
        queue.enqueue(op2.clone()).unwrap();

        let ops = queue.list();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].id, op1.id);
        assert_eq!(ops[1].id, op2.id);
    }

    #[test]
    fn test_queue_retry() {
        let queue = OperationQueue::new();
        let doc_id = DocumentId::new("users", "alice");
        let op = Operation::new(OperationType::Create { document_id: doc_id });

        queue.enqueue(op.clone()).unwrap();
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.retry_count, 0);

        queue.retry(dequeued).unwrap();
        let retried = queue.dequeue().unwrap();
        assert_eq!(retried.retry_count, 1);
    }

    #[test]
    fn test_queue_serialize_deserialize() {
        let queue1 = OperationQueue::new();
        let doc_id1 = DocumentId::new("users", "alice");
        let doc_id2 = DocumentId::new("users", "bob");

        let op1 = Operation::new(OperationType::Create { document_id: doc_id1 });
        let op2 = Operation::new(OperationType::Create { document_id: doc_id2 });

        queue1.enqueue(op1.clone()).unwrap();
        queue1.enqueue(op2.clone()).unwrap();

        let bytes = queue1.serialize().unwrap();

        let queue2 = OperationQueue::new();
        queue2.deserialize(&bytes).unwrap();

        assert_eq!(queue2.len(), 2);
        let ops = queue2.list();
        assert_eq!(ops[0].id, op1.id);
        assert_eq!(ops[1].id, op2.id);
    }

    #[test]
    fn test_queue_filter_by_document() {
        let queue = OperationQueue::new();
        let doc_id1 = DocumentId::new("users", "alice");
        let doc_id2 = DocumentId::new("users", "bob");

        let op1 = Operation::new(OperationType::Create { document_id: doc_id1.clone() });
        let op2 = Operation::new(OperationType::Create { document_id: doc_id2 });
        let op3 = Operation::new(OperationType::Update {
            document_id: doc_id1.clone(),
            change_bytes: vec![],
        });

        queue.enqueue(op1).unwrap();
        queue.enqueue(op2).unwrap();
        queue.enqueue(op3).unwrap();

        let filtered = queue.filter_by_document(&doc_id1);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_queue_remove_by_document() {
        let queue = OperationQueue::new();
        let doc_id1 = DocumentId::new("users", "alice");
        let doc_id2 = DocumentId::new("users", "bob");

        let op1 = Operation::new(OperationType::Create { document_id: doc_id1.clone() });
        let op2 = Operation::new(OperationType::Create { document_id: doc_id2 });

        queue.enqueue(op1).unwrap();
        queue.enqueue(op2).unwrap();

        assert_eq!(queue.len(), 2);

        queue.remove_by_document(&doc_id1);
        assert_eq!(queue.len(), 1);

        let remaining = queue.list();
        assert_eq!(remaining[0].document_id().key, "bob");
    }

    #[test]
    fn test_operation_type_equality() {
        let doc_id = DocumentId::new("users", "alice");
        let op1 = OperationType::Create { document_id: doc_id.clone() };
        let op2 = OperationType::Create { document_id: doc_id.clone() };
        assert_eq!(op1, op2);

        let op3 = OperationType::Update {
            document_id: doc_id.clone(),
            change_bytes: vec![1, 2, 3],
        };
        let op4 = OperationType::Update {
            document_id: doc_id,
            change_bytes: vec![1, 2, 3],
        };
        assert_eq!(op3, op4);
    }
}
