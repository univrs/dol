//! VUDO Local State Engine
//!
//! Platform-agnostic local state management for VUDO Runtime with Automerge CRDT support.
//!
//! This crate provides the core state management layer for the VUDO Runtime, including:
//! - Automerge document store with in-memory caching
//! - Reactive subscriptions for change notifications
//! - Operation queue for offline mutations
//! - Snapshot management for compaction
//! - Multi-document transactions with atomic commit/rollback
//!
//! # Examples
//!
//! ```
//! use vudo_state::{StateEngine, document_store::DocumentId};
//! use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};
//!
//! #[tokio::main]
//! async fn main() -> vudo_state::error::Result<()> {
//!     // Initialize state engine
//!     let engine = StateEngine::new().await?;
//!
//!     // Create a document
//!     let doc_id = DocumentId::new("users", "alice");
//!     let handle = engine.create_document(doc_id).await?;
//!
//!     // Update the document
//!     handle.update(|tx| {
//!         tx.put(ROOT, "name", "Alice")?;
//!         tx.put(ROOT, "age", 30i64)?;
//!         Ok(())
//!     })?;
//!
//!     // Read from the document
//!     handle.read(|doc| {
//!         // Extract name value
//!         if let Some((automerge::Value::Scalar(s), _)) = doc.get(ROOT, "name")? {
//!             if let ScalarValue::Str(smol_str) = s.as_ref() {
//!                 assert_eq!(smol_str.to_string(), "Alice");
//!             }
//!         }
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```

pub mod document_store;
pub mod error;
pub mod operation_queue;
pub mod reactive;
// pub mod schema_evolution; // Disabled - task t2.5
pub mod snapshot;
pub mod transaction;

pub use document_store::{DocumentHandle, DocumentId, DocumentMetadata, DocumentStore};
pub use error::{Result, StateError};
pub use operation_queue::{Operation, OperationId, OperationQueue, OperationType};
pub use reactive::{ChangeEvent, ChangeObservable, ReactiveDocument, Subscription, SubscriptionFilter, SubscriptionId};
// pub use schema_evolution::{
//     EvolutionEngine, ForwardCompatibleReader, Migration, MigrationConflictResolver,
//     MigrationMetadata, SchemaMetadata, SchemaVersion,
// };
pub use snapshot::{CompactionResult, Snapshot, SnapshotManager, SnapshotMetadata, SnapshotStorage};
pub use transaction::{Transaction, TransactionBuilder, TransactionId, TransactionManager, TransactionState};

use std::sync::Arc;

/// Main state engine that coordinates all components.
pub struct StateEngine {
    /// Document store.
    pub store: Arc<DocumentStore>,
    /// Change observable for reactive subscriptions.
    pub observable: Arc<ChangeObservable>,
    /// Operation queue for offline mutations.
    pub queue: Arc<OperationQueue>,
    /// Snapshot storage.
    pub snapshot_storage: Arc<SnapshotStorage>,
    /// Snapshot manager.
    pub snapshot_manager: Arc<SnapshotManager>,
    /// Transaction manager.
    pub transaction_manager: Arc<TransactionManager>,
}

impl StateEngine {
    /// Create a new state engine.
    pub async fn new() -> Result<Self> {
        let store = Arc::new(DocumentStore::new());
        let observable = Arc::new(ChangeObservable::new());
        let queue = Arc::new(OperationQueue::new());
        let snapshot_storage = Arc::new(SnapshotStorage::new());
        let snapshot_manager = Arc::new(SnapshotManager::new(Arc::clone(&snapshot_storage)));
        let transaction_manager = Arc::new(TransactionManager::new(Arc::clone(&store)));

        Ok(Self {
            store,
            observable,
            queue,
            snapshot_storage,
            snapshot_manager,
            transaction_manager,
        })
    }

    /// Create a new state engine with custom configuration.
    pub async fn with_config(config: StateEngineConfig) -> Result<Self> {
        let store = Arc::new(DocumentStore::new());
        let observable = Arc::new(ChangeObservable::new());
        let queue = Arc::new(OperationQueue::with_max_size(config.max_queue_size));
        let snapshot_storage = Arc::new(SnapshotStorage::with_max_snapshots(
            config.max_snapshots_per_doc,
        ));
        let snapshot_manager = Arc::new(SnapshotManager::with_settings(
            Arc::clone(&snapshot_storage),
            config.snapshot_interval,
            config.min_changes_threshold,
        ));
        let transaction_manager = Arc::new(TransactionManager::new(Arc::clone(&store)));

        Ok(Self {
            store,
            observable,
            queue,
            snapshot_storage,
            snapshot_manager,
            transaction_manager,
        })
    }

    /// Create a new document.
    pub async fn create_document(&self, id: DocumentId) -> Result<DocumentHandle> {
        let handle = self.store.create(id.clone())?;

        // Enqueue create operation
        let op = Operation::new(OperationType::Create { document_id: id });
        self.queue.enqueue(op)?;

        Ok(handle)
    }

    /// Get a document by ID.
    pub async fn get_document(&self, id: &DocumentId) -> Result<DocumentHandle> {
        self.store.get(id)
    }

    /// Delete a document.
    pub async fn delete_document(&self, id: &DocumentId) -> Result<()> {
        self.store.delete(id)?;

        // Enqueue delete operation
        let op = Operation::new(OperationType::Delete {
            document_id: id.clone(),
        });
        self.queue.enqueue(op)?;

        Ok(())
    }

    /// Subscribe to document changes.
    pub async fn subscribe(&self, filter: SubscriptionFilter) -> Subscription {
        self.observable.subscribe(filter)
    }

    /// Unsubscribe from changes.
    pub async fn unsubscribe(&self, id: SubscriptionId) -> Result<()> {
        self.observable.unsubscribe(id)
    }

    /// Begin a new transaction.
    pub fn begin_transaction(&self) -> Transaction {
        self.transaction_manager.begin()
    }

    /// Commit a transaction.
    pub fn commit_transaction(&self, tx: Transaction) -> Result<()> {
        self.transaction_manager.commit(tx)
    }

    /// Rollback a transaction.
    pub fn rollback_transaction(&self, tx: Transaction) -> Result<()> {
        self.transaction_manager.rollback(tx)
    }

    /// Create a snapshot of a document.
    pub async fn snapshot(&self, handle: &DocumentHandle) -> Result<Snapshot> {
        self.snapshot_manager.create_snapshot(handle)
    }

    /// Compact a document.
    pub async fn compact(&self, handle: &DocumentHandle) -> Result<CompactionResult> {
        self.snapshot_manager.compact(handle)
    }

    /// Get statistics about the state engine.
    pub fn stats(&self) -> StateEngineStats {
        StateEngineStats {
            document_count: self.store.count(),
            total_document_size: self.store.total_size(),
            subscription_count: self.observable.subscription_count(),
            queue_length: self.queue.len(),
            snapshot_count: self.snapshot_storage.total_count(),
            total_snapshot_size: self.snapshot_storage.total_size(),
            active_transaction_count: self.transaction_manager.active_count(),
        }
    }
}

/// Configuration for the state engine.
#[derive(Debug, Clone)]
pub struct StateEngineConfig {
    /// Maximum operation queue size.
    pub max_queue_size: usize,
    /// Maximum number of snapshots to keep per document.
    pub max_snapshots_per_doc: usize,
    /// Snapshot interval.
    pub snapshot_interval: tokio::time::Duration,
    /// Minimum number of changes before creating a snapshot.
    pub min_changes_threshold: usize,
}

impl Default for StateEngineConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10_000,
            max_snapshots_per_doc: 10,
            snapshot_interval: tokio::time::Duration::from_secs(60),
            min_changes_threshold: 10,
        }
    }
}

/// Statistics about the state engine.
#[derive(Debug, Clone)]
pub struct StateEngineStats {
    /// Number of documents in the store.
    pub document_count: usize,
    /// Total size of all documents in bytes.
    pub total_document_size: usize,
    /// Number of active subscriptions.
    pub subscription_count: usize,
    /// Number of operations in the queue.
    pub queue_length: usize,
    /// Number of snapshots stored.
    pub snapshot_count: usize,
    /// Total size of all snapshots in bytes.
    pub total_snapshot_size: usize,
    /// Number of active transactions.
    pub active_transaction_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};

    fn get_i64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<i64> {
        match doc.get(&obj, key)? {
            Some((automerge::Value::Scalar(s), _)) => {
                if let ScalarValue::Int(val) = s.as_ref() {
                    Ok(*val)
                } else {
                    Err(StateError::Internal("Expected int value".to_string()))
                }
            }
            _ => Err(StateError::Internal("Value not found".to_string())),
        }
    }

    #[tokio::test]
    async fn test_state_engine_new() {
        let engine = StateEngine::new().await.unwrap();
        let stats = engine.stats();
        assert_eq!(stats.document_count, 0);
    }

    #[tokio::test]
    async fn test_state_engine_create_document() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id = DocumentId::new("users", "alice");
        let handle = engine.create_document(doc_id.clone()).await.unwrap();

        assert_eq!(handle.id, doc_id);
        assert_eq!(engine.stats().document_count, 1);
    }

    #[tokio::test]
    async fn test_state_engine_get_document() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id = DocumentId::new("users", "alice");
        engine.create_document(doc_id.clone()).await.unwrap();

        let handle = engine.get_document(&doc_id).await.unwrap();
        assert_eq!(handle.id, doc_id);
    }

    #[tokio::test]
    async fn test_state_engine_delete_document() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id = DocumentId::new("users", "alice");
        engine.create_document(doc_id.clone()).await.unwrap();

        engine.delete_document(&doc_id).await.unwrap();
        assert_eq!(engine.stats().document_count, 0);
    }

    #[tokio::test]
    async fn test_state_engine_subscribe() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Document(doc_id);

        let _sub = engine.subscribe(filter).await;
        assert_eq!(engine.stats().subscription_count, 1);
    }

    #[tokio::test]
    async fn test_state_engine_transaction() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id1 = DocumentId::new("users", "alice");
        let doc_id2 = DocumentId::new("users", "bob");

        engine.create_document(doc_id1.clone()).await.unwrap();
        engine.create_document(doc_id2.clone()).await.unwrap();

        let tx = engine.begin_transaction();

        tx.update(&doc_id1, |doc| {
            doc.put(ROOT, "balance", 100i64)?;
            Ok(())
        })
        .unwrap();

        tx.update(&doc_id2, |doc| {
            doc.put(ROOT, "balance", 100i64)?;
            Ok(())
        })
        .unwrap();

        engine.commit_transaction(tx).unwrap();

        // Verify changes
        let handle1 = engine.get_document(&doc_id1).await.unwrap();
        handle1
            .read(|doc| {
                let balance = get_i64(doc, ROOT, "balance")?;
                assert_eq!(balance, 100);
                Ok(())
            })
            .unwrap();
    }

    #[tokio::test]
    async fn test_state_engine_snapshot() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id = DocumentId::new("users", "alice");
        let handle = engine.create_document(doc_id).await.unwrap();

        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let snapshot = engine.snapshot(&handle).await.unwrap();
        assert_eq!(snapshot.metadata.version, 1);
    }

    #[tokio::test]
    async fn test_state_engine_stats() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id = DocumentId::new("users", "alice");
        let handle = engine.create_document(doc_id.clone()).await.unwrap();

        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        engine.snapshot(&handle).await.unwrap();

        let stats = engine.stats();
        assert_eq!(stats.document_count, 1);
        assert!(stats.total_document_size > 0);
        assert_eq!(stats.snapshot_count, 1);
        assert!(stats.total_snapshot_size > 0);
    }

    #[tokio::test]
    async fn test_state_engine_with_config() {
        let config = StateEngineConfig {
            max_queue_size: 5000,
            max_snapshots_per_doc: 5,
            snapshot_interval: tokio::time::Duration::from_secs(30),
            min_changes_threshold: 5,
        };

        let engine = StateEngine::with_config(config).await.unwrap();
        assert_eq!(engine.stats().document_count, 0);
    }

    #[tokio::test]
    async fn test_state_engine_operation_queue() {
        let engine = StateEngine::new().await.unwrap();
        let doc_id = DocumentId::new("users", "alice");

        engine.create_document(doc_id.clone()).await.unwrap();
        engine.delete_document(&doc_id).await.unwrap();

        // Should have 2 operations (create + delete)
        assert_eq!(engine.stats().queue_length, 2);
    }
}
