//! Multi-document transactions with atomic commit/rollback.

use crate::document_store::{DocumentHandle, DocumentId, DocumentStore};
use crate::error::{Result, StateError};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Transaction ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(u64);

impl TransactionId {
    /// Generate a new transaction ID.
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Transaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction is active and accepting operations.
    Active,
    /// Transaction is being committed.
    Committing,
    /// Transaction has been committed successfully.
    Committed,
    /// Transaction has been rolled back.
    RolledBack,
}

/// A saved document state for rollback.
#[derive(Clone)]
struct DocumentSnapshot {
    /// Document ID.
    document_id: DocumentId,
    /// Saved document bytes.
    snapshot_bytes: Vec<u8>,
}

/// Transaction operation.
#[derive(Clone)]
enum TransactionOp {
    /// Update operation with a closure.
    Update {
        document_id: DocumentId,
        // We can't store closures, so we'll apply operations immediately
        // and track snapshots for rollback
    },
}

/// A multi-document transaction.
pub struct Transaction {
    /// Transaction ID.
    id: TransactionId,
    /// Transaction state.
    state: Arc<Mutex<TransactionState>>,
    /// Document store reference.
    store: Arc<DocumentStore>,
    /// Snapshots for rollback.
    snapshots: Arc<Mutex<HashMap<DocumentId, DocumentSnapshot>>>,
    /// Document handles involved in the transaction.
    handles: Arc<Mutex<Vec<DocumentHandle>>>,
    /// Transaction log for debugging.
    log: Arc<Mutex<Vec<String>>>,
}

impl Transaction {
    /// Create a new transaction.
    pub fn new(store: Arc<DocumentStore>) -> Self {
        Self {
            id: TransactionId::new(),
            state: Arc::new(Mutex::new(TransactionState::Active)),
            store,
            snapshots: Arc::new(Mutex::new(HashMap::new())),
            handles: Arc::new(Mutex::new(Vec::new())),
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.id
    }

    /// Get the transaction state.
    pub fn state(&self) -> TransactionState {
        *self.state.lock()
    }

    /// Check if the transaction is active.
    pub fn is_active(&self) -> bool {
        matches!(*self.state.lock(), TransactionState::Active)
    }

    /// Add a log entry.
    fn log(&self, message: String) {
        self.log.lock().push(message);
    }

    /// Get the transaction log.
    pub fn get_log(&self) -> Vec<String> {
        self.log.lock().clone()
    }

    /// Update a document within the transaction.
    pub fn update<F, T>(&self, document_id: &DocumentId, f: F) -> Result<T>
    where
        F: FnOnce(&mut automerge::AutoCommit) -> Result<T>,
    {
        // Check if transaction is active
        if !self.is_active() {
            return Err(StateError::TransactionFailed(
                "Transaction is not active".to_string(),
            ));
        }

        // Get or create document
        let handle = self.store.get(document_id)?;

        // Create snapshot if this is the first operation on this document
        {
            let mut snapshots = self.snapshots.lock();
            if !snapshots.contains_key(document_id) {
                let snapshot_bytes = handle.save();
                snapshots.insert(
                    document_id.clone(),
                    DocumentSnapshot {
                        document_id: document_id.clone(),
                        snapshot_bytes,
                    },
                );
                self.log(format!("Created snapshot for {}", document_id));
            }
        }

        // Track the handle
        self.handles.lock().push(handle.clone());

        // Apply the update
        let result = handle.update(f)?;
        self.log(format!("Updated document {}", document_id));

        Ok(result)
    }

    /// Commit the transaction.
    pub fn commit(self) -> Result<()> {
        let mut state = self.state.lock();

        if !matches!(*state, TransactionState::Active) {
            return Err(StateError::TransactionFailed(
                "Transaction is not active".to_string(),
            ));
        }

        *state = TransactionState::Committing;
        drop(state);

        self.log("Committing transaction".to_string());

        // In Automerge, changes are already applied to documents
        // Commit just means we won't roll back
        *self.state.lock() = TransactionState::Committed;
        self.log("Transaction committed successfully".to_string());

        Ok(())
    }

    /// Rollback the transaction.
    pub fn rollback(self) -> Result<()> {
        let mut state = self.state.lock();

        if !matches!(*state, TransactionState::Active) {
            return Err(StateError::TransactionFailed(
                "Transaction is not active".to_string(),
            ));
        }

        *state = TransactionState::RolledBack;
        drop(state);

        self.log("Rolling back transaction".to_string());

        // Restore all documents from snapshots
        let snapshots = self.snapshots.lock();
        for (doc_id, snapshot) in snapshots.iter() {
            // Load the snapshot back into the document
            if let Ok(handle) = self.store.get(doc_id) {
                let restored_doc = automerge::AutoCommit::load(&snapshot.snapshot_bytes)?;
                // Replace the document's content
                *handle.doc.write() = restored_doc;
                self.log(format!("Rolled back document {}", doc_id));
            }
        }

        self.log("Transaction rolled back successfully".to_string());

        Ok(())
    }
}

/// Transaction builder for fluent API.
pub struct TransactionBuilder {
    store: Arc<DocumentStore>,
}

impl TransactionBuilder {
    /// Create a new transaction builder.
    pub fn new(store: Arc<DocumentStore>) -> Self {
        Self { store }
    }

    /// Begin a new transaction.
    pub fn begin(&self) -> Transaction {
        Transaction::new(Arc::clone(&self.store))
    }
}

/// Transaction manager for coordinating multiple transactions.
pub struct TransactionManager {
    /// Document store.
    store: Arc<DocumentStore>,
    /// Active transactions.
    active_transactions: Arc<Mutex<HashMap<TransactionId, Transaction>>>,
}

impl TransactionManager {
    /// Create a new transaction manager.
    pub fn new(store: Arc<DocumentStore>) -> Self {
        Self {
            store,
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Begin a new transaction.
    pub fn begin(&self) -> Transaction {
        let tx = Transaction::new(Arc::clone(&self.store));
        self.active_transactions.lock().insert(tx.id, tx.clone());
        tx
    }

    /// Get an active transaction by ID.
    pub fn get(&self, id: TransactionId) -> Option<Transaction> {
        self.active_transactions.lock().get(&id).cloned()
    }

    /// Commit a transaction.
    pub fn commit(&self, tx: Transaction) -> Result<()> {
        let id = tx.id;
        let result = tx.commit();
        self.active_transactions.lock().remove(&id);
        result
    }

    /// Rollback a transaction.
    pub fn rollback(&self, tx: Transaction) -> Result<()> {
        let id = tx.id;
        let result = tx.rollback();
        self.active_transactions.lock().remove(&id);
        result
    }

    /// Get the number of active transactions.
    pub fn active_count(&self) -> usize {
        self.active_transactions.lock().len()
    }

    /// Rollback all active transactions.
    pub fn rollback_all(&self) -> Result<()> {
        let transactions: Vec<Transaction> = {
            let active = self.active_transactions.lock();
            active.values().cloned().collect()
        };

        for tx in transactions {
            tx.rollback()?;
        }

        self.active_transactions.lock().clear();
        Ok(())
    }
}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: Arc::clone(&self.state),
            store: Arc::clone(&self.store),
            snapshots: Arc::clone(&self.snapshots),
            handles: Arc::clone(&self.handles),
            log: Arc::clone(&self.log),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};

    fn get_string(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> crate::error::Result<String> {
        match doc.get(&obj, key)? {
            Some((automerge::Value::Scalar(s), _)) => {
                if let ScalarValue::Str(smol_str) = s.as_ref() {
                    Ok(smol_str.to_string())
                } else {
                    Err(StateError::Internal("Expected string value".to_string()))
                }
            }
            _ => Err(StateError::Internal("Value not found".to_string())),
        }
    }

    fn get_i64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> crate::error::Result<i64> {
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

    #[test]
    fn test_transaction_id() {
        let id1 = TransactionId::new();
        let id2 = TransactionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_transaction_new() {
        let store = Arc::new(DocumentStore::new());
        let tx = Transaction::new(store);
        assert!(tx.is_active());
        assert_eq!(tx.state(), TransactionState::Active);
    }

    #[test]
    fn test_transaction_update() {
        let store = Arc::new(DocumentStore::new());
        let doc_id = DocumentId::new("users", "alice");
        store.create(doc_id.clone()).unwrap();

        let tx = Transaction::new(store);

        tx.update(&doc_id, |doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        })
        .unwrap();

        assert!(tx.is_active());
    }

    #[test]
    fn test_transaction_commit() {
        let store = Arc::new(DocumentStore::new());
        let doc_id = DocumentId::new("users", "alice");
        store.create(doc_id.clone()).unwrap();

        let tx = Transaction::new(Arc::clone(&store));

        tx.update(&doc_id, |doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        })
        .unwrap();

        tx.commit().unwrap();

        // Verify the change persisted
        let handle = store.get(&doc_id).unwrap();
        handle
            .read(|doc| {
                let name = get_string(doc, ROOT, "name")?;
                assert_eq!(name, "Alice");
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_transaction_rollback() {
        let store = Arc::new(DocumentStore::new());
        let doc_id = DocumentId::new("users", "alice");
        let handle = store.create(doc_id.clone()).unwrap();

        // Set initial value
        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let tx = Transaction::new(Arc::clone(&store));

        // Update in transaction
        tx.update(&doc_id, |doc| {
            doc.put(ROOT, "name", "Bob")?;
            Ok(())
        })
        .unwrap();

        // Rollback
        tx.rollback().unwrap();

        // Verify the change was rolled back
        let handle = store.get(&doc_id).unwrap();
        handle
            .read(|doc| {
                let name = get_string(doc, ROOT, "name")?;
                assert_eq!(name, "Alice");
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_transaction_multi_document() {
        let store = Arc::new(DocumentStore::new());
        let doc_id1 = DocumentId::new("users", "alice");
        let doc_id2 = DocumentId::new("users", "bob");

        store.create(doc_id1.clone()).unwrap();
        store.create(doc_id2.clone()).unwrap();

        let tx = Transaction::new(Arc::clone(&store));

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

        tx.commit().unwrap();

        // Verify both documents were updated
        let handle1 = store.get(&doc_id1).unwrap();
        handle1
            .read(|doc| {
                let balance = get_i64(doc, ROOT, "balance")?;
                assert_eq!(balance, 100);
                Ok(())
            })
            .unwrap();

        let handle2 = store.get(&doc_id2).unwrap();
        handle2
            .read(|doc| {
                let balance = get_i64(doc, ROOT, "balance")?;
                assert_eq!(balance, 100);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_transaction_multi_document_rollback() {
        let store = Arc::new(DocumentStore::new());
        let doc_id1 = DocumentId::new("users", "alice");
        let doc_id2 = DocumentId::new("users", "bob");

        let handle1 = store.create(doc_id1.clone()).unwrap();
        let handle2 = store.create(doc_id2.clone()).unwrap();

        // Set initial values
        handle1
            .update(|doc| {
                doc.put(ROOT, "balance", 100i64)?;
                Ok(())
            })
            .unwrap();

        handle2
            .update(|doc| {
                doc.put(ROOT, "balance", 100i64)?;
                Ok(())
            })
            .unwrap();

        let tx = Transaction::new(Arc::clone(&store));

        // Transfer money (should be atomic)
        tx.update(&doc_id1, |doc| {
            doc.put(ROOT, "balance", 50i64)?;
            Ok(())
        })
        .unwrap();

        tx.update(&doc_id2, |doc| {
            doc.put(ROOT, "balance", 150i64)?;
            Ok(())
        })
        .unwrap();

        // Rollback the transfer
        tx.rollback().unwrap();

        // Verify both documents were rolled back
        let handle1 = store.get(&doc_id1).unwrap();
        handle1
            .read(|doc| {
                let balance = get_i64(doc, ROOT, "balance")?;
                assert_eq!(balance, 100);
                Ok(())
            })
            .unwrap();

        let handle2 = store.get(&doc_id2).unwrap();
        handle2
            .read(|doc| {
                let balance = get_i64(doc, ROOT, "balance")?;
                assert_eq!(balance, 100);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_transaction_builder() {
        let store = Arc::new(DocumentStore::new());
        let builder = TransactionBuilder::new(Arc::clone(&store));

        let tx = builder.begin();
        assert!(tx.is_active());
    }

    #[test]
    fn test_transaction_manager() {
        let store = Arc::new(DocumentStore::new());
        let manager = TransactionManager::new(Arc::clone(&store));

        let tx1 = manager.begin();
        let tx2 = manager.begin();

        assert_eq!(manager.active_count(), 2);

        manager.commit(tx1).unwrap();
        assert_eq!(manager.active_count(), 1);

        manager.rollback(tx2).unwrap();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_transaction_manager_rollback_all() {
        let store = Arc::new(DocumentStore::new());
        let manager = TransactionManager::new(Arc::clone(&store));

        let _tx1 = manager.begin();
        let _tx2 = manager.begin();

        assert_eq!(manager.active_count(), 2);

        manager.rollback_all().unwrap();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_transaction_log() {
        let store = Arc::new(DocumentStore::new());
        let doc_id = DocumentId::new("users", "alice");
        store.create(doc_id.clone()).unwrap();

        let tx = Transaction::new(store);

        tx.update(&doc_id, |doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        })
        .unwrap();

        let log_before_commit = tx.get_log();
        tx.commit().unwrap();

        assert!(!log_before_commit.is_empty());
        assert!(log_before_commit.iter().any(|entry| entry.contains("snapshot")));
    }

    #[test]
    fn test_transaction_inactive_error() {
        let store = Arc::new(DocumentStore::new());
        let doc_id = DocumentId::new("users", "alice");
        store.create(doc_id.clone()).unwrap();

        let tx = Transaction::new(Arc::clone(&store));
        let tx_clone = tx.clone();
        tx.commit().unwrap();

        // Try to update after commit
        let result = tx_clone.update(&doc_id, |doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        });

        assert!(matches!(result, Err(StateError::TransactionFailed(_))));
    }

    #[test]
    fn test_transaction_document_not_found() {
        let store = Arc::new(DocumentStore::new());
        let doc_id = DocumentId::new("users", "alice");

        let tx = Transaction::new(store);

        let result = tx.update(&doc_id, |doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        });

        assert!(matches!(result, Err(StateError::DocumentNotFound(_))));
    }
}
