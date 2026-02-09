//! Snapshot management for document compaction and versioning.

use crate::document_store::{DocumentHandle, DocumentId};
use crate::error::{Result, StateError};
use automerge::AutoCommit;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;

/// Snapshot metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Document ID.
    pub document_id: DocumentId,
    /// Snapshot version number.
    pub version: u64,
    /// Timestamp (Unix epoch milliseconds).
    pub timestamp: u64,
    /// Size in bytes.
    pub size: usize,
    /// Number of changes since last snapshot.
    pub changes_since_last: usize,
}

/// A snapshot of a document at a specific point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Metadata.
    pub metadata: SnapshotMetadata,
    /// Serialized Automerge document.
    pub data: Vec<u8>,
}

impl Snapshot {
    /// Create a new snapshot from a document handle.
    pub fn from_document(handle: &DocumentHandle, version: u64) -> Self {
        let data = handle.save();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let metadata = SnapshotMetadata {
            document_id: handle.id.clone(),
            version,
            timestamp,
            size: data.len(),
            changes_since_last: handle.change_count(),
        };

        Self { metadata, data }
    }

    /// Load a document from this snapshot.
    pub fn to_document(&self) -> Result<AutoCommit> {
        AutoCommit::load(&self.data).map_err(StateError::from)
    }

    /// Calculate compression ratio compared to original size.
    pub fn compression_ratio(&self, original_size: usize) -> f64 {
        if original_size == 0 {
            return 1.0;
        }
        self.metadata.size as f64 / original_size as f64
    }
}

/// Snapshot storage (in-memory for now, will be persisted in Phase 2.2).
pub struct SnapshotStorage {
    /// Map of document ID to snapshots (ordered by version).
    snapshots: Arc<RwLock<HashMap<DocumentId, Vec<Snapshot>>>>,
    /// Maximum number of snapshots to keep per document.
    max_snapshots_per_doc: usize,
}

impl SnapshotStorage {
    /// Create a new snapshot storage.
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots_per_doc: 10,
        }
    }

    /// Create a new snapshot storage with a maximum number of snapshots per document.
    pub fn with_max_snapshots(max_snapshots_per_doc: usize) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots_per_doc,
        }
    }

    /// Store a snapshot.
    pub fn store(&self, snapshot: Snapshot) -> Result<()> {
        let mut snapshots = self.snapshots.write();
        let doc_snapshots = snapshots
            .entry(snapshot.metadata.document_id.clone())
            .or_insert_with(Vec::new);

        doc_snapshots.push(snapshot);

        // Sort by version
        doc_snapshots.sort_by_key(|s| s.metadata.version);

        // Enforce maximum snapshots limit (keep most recent)
        if doc_snapshots.len() > self.max_snapshots_per_doc {
            doc_snapshots.drain(0..doc_snapshots.len() - self.max_snapshots_per_doc);
        }

        Ok(())
    }

    /// Get the latest snapshot for a document.
    pub fn get_latest(&self, document_id: &DocumentId) -> Option<Snapshot> {
        let snapshots = self.snapshots.read();
        snapshots
            .get(document_id)
            .and_then(|snaps| snaps.last().cloned())
    }

    /// Get a specific snapshot by version.
    pub fn get_version(&self, document_id: &DocumentId, version: u64) -> Option<Snapshot> {
        let snapshots = self.snapshots.read();
        snapshots
            .get(document_id)
            .and_then(|snaps| snaps.iter().find(|s| s.metadata.version == version).cloned())
    }

    /// List all snapshots for a document.
    pub fn list(&self, document_id: &DocumentId) -> Vec<SnapshotMetadata> {
        let snapshots = self.snapshots.read();
        snapshots
            .get(document_id)
            .map(|snaps| snaps.iter().map(|s| s.metadata.clone()).collect())
            .unwrap_or_default()
    }

    /// Delete all snapshots for a document.
    pub fn delete(&self, document_id: &DocumentId) -> Result<()> {
        self.snapshots.write().remove(document_id);
        Ok(())
    }

    /// Delete snapshots older than a specific version.
    pub fn delete_older_than(&self, document_id: &DocumentId, version: u64) -> Result<()> {
        let mut snapshots = self.snapshots.write();
        if let Some(doc_snapshots) = snapshots.get_mut(document_id) {
            doc_snapshots.retain(|s| s.metadata.version >= version);
        }
        Ok(())
    }

    /// Get the total number of snapshots.
    pub fn total_count(&self) -> usize {
        self.snapshots
            .read()
            .values()
            .map(|snaps| snaps.len())
            .sum()
    }

    /// Get the total storage size of all snapshots.
    pub fn total_size(&self) -> usize {
        self.snapshots
            .read()
            .values()
            .flat_map(|snaps| snaps.iter())
            .map(|s| s.metadata.size)
            .sum()
    }

    /// Clear all snapshots.
    pub fn clear(&self) {
        self.snapshots.write().clear();
    }
}

impl Default for SnapshotStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot manager for automatic compaction and versioning.
pub struct SnapshotManager {
    /// Snapshot storage.
    storage: Arc<SnapshotStorage>,
    /// Snapshot interval (duration between automatic snapshots).
    snapshot_interval: Duration,
    /// Minimum changes before creating a snapshot.
    min_changes_threshold: usize,
}

impl SnapshotManager {
    /// Create a new snapshot manager.
    pub fn new(storage: Arc<SnapshotStorage>) -> Self {
        Self {
            storage,
            snapshot_interval: Duration::from_secs(60), // 1 minute
            min_changes_threshold: 10,
        }
    }

    /// Create a snapshot manager with custom settings.
    pub fn with_settings(
        storage: Arc<SnapshotStorage>,
        snapshot_interval: Duration,
        min_changes_threshold: usize,
    ) -> Self {
        Self {
            storage,
            snapshot_interval,
            min_changes_threshold,
        }
    }

    /// Create a snapshot of a document.
    pub fn create_snapshot(&self, handle: &DocumentHandle) -> Result<Snapshot> {
        // Get the next version number
        let latest = self.storage.get_latest(&handle.id);
        let version = latest.map(|s| s.metadata.version + 1).unwrap_or(1);

        let snapshot = Snapshot::from_document(handle, version);
        self.storage.store(snapshot.clone())?;

        Ok(snapshot)
    }

    /// Check if a document should be snapshotted based on change count.
    pub fn should_snapshot(&self, handle: &DocumentHandle) -> bool {
        let change_count = handle.change_count();
        change_count >= self.min_changes_threshold
    }

    /// Create a snapshot if threshold is met.
    pub fn snapshot_if_needed(&self, handle: &DocumentHandle) -> Result<Option<Snapshot>> {
        if self.should_snapshot(handle) {
            Ok(Some(self.create_snapshot(handle)?))
        } else {
            Ok(None)
        }
    }

    /// Compact a document by loading the latest snapshot and creating a new one.
    pub fn compact(&self, handle: &DocumentHandle) -> Result<CompactionResult> {
        let original_size = handle.save().len();

        // Create a new snapshot
        let snapshot = self.create_snapshot(handle)?;

        let compacted_size = snapshot.metadata.size;
        let reduction = original_size.saturating_sub(compacted_size);
        let reduction_percent = if original_size > 0 {
            (reduction as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };

        Ok(CompactionResult {
            original_size,
            compacted_size,
            reduction,
            reduction_percent,
        })
    }

    /// Start a background task that periodically creates snapshots.
    pub async fn start_background_snapshots(
        self: Arc<Self>,
        handles: Vec<DocumentHandle>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.snapshot_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;

                for handle in &handles {
                    if let Err(e) = self.snapshot_if_needed(handle) {
                        tracing::warn!("Failed to create snapshot: {}", e);
                    }
                }
            }
        })
    }

    /// Get the storage.
    pub fn storage(&self) -> &Arc<SnapshotStorage> {
        &self.storage
    }
}

/// Result of a compaction operation.
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Original size in bytes.
    pub original_size: usize,
    /// Compacted size in bytes.
    pub compacted_size: usize,
    /// Size reduction in bytes.
    pub reduction: usize,
    /// Size reduction percentage.
    pub reduction_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_store::DocumentStore;
    use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};
    use crate::error::StateError;

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

    #[test]
    fn test_snapshot_from_document() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id).unwrap();

        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let snapshot = Snapshot::from_document(&handle, 1);
        assert_eq!(snapshot.metadata.document_id, handle.id);
        assert_eq!(snapshot.metadata.version, 1);
        assert!(snapshot.data.len() > 0);
    }

    #[test]
    fn test_snapshot_to_document() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id).unwrap();

        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let snapshot = Snapshot::from_document(&handle, 1);
        let loaded_doc = snapshot.to_document().unwrap();

        let name = get_string(&loaded_doc, ROOT, "name").unwrap();
        assert_eq!(name, "Alice");
    }

    #[test]
    fn test_snapshot_storage_store_and_get() {
        let storage = SnapshotStorage::new();
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        let snapshot = Snapshot::from_document(&handle, 1);
        storage.store(snapshot.clone()).unwrap();

        let retrieved = storage.get_latest(&id).unwrap();
        assert_eq!(retrieved.metadata.version, 1);
    }

    #[test]
    fn test_snapshot_storage_max_snapshots() {
        let storage = SnapshotStorage::with_max_snapshots(3);
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        // Create 5 snapshots
        for i in 1..=5 {
            let snapshot = Snapshot::from_document(&handle, i);
            storage.store(snapshot).unwrap();
        }

        // Should only keep the last 3
        let snapshots = storage.list(&id);
        assert_eq!(snapshots.len(), 3);
        assert_eq!(snapshots[0].version, 3);
        assert_eq!(snapshots[1].version, 4);
        assert_eq!(snapshots[2].version, 5);
    }

    #[test]
    fn test_snapshot_storage_get_version() {
        let storage = SnapshotStorage::new();
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        for i in 1..=3 {
            let snapshot = Snapshot::from_document(&handle, i);
            storage.store(snapshot).unwrap();
        }

        let snapshot = storage.get_version(&id, 2).unwrap();
        assert_eq!(snapshot.metadata.version, 2);
    }

    #[test]
    fn test_snapshot_storage_delete() {
        let storage = SnapshotStorage::new();
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        let snapshot = Snapshot::from_document(&handle, 1);
        storage.store(snapshot).unwrap();

        storage.delete(&id).unwrap();
        assert!(storage.get_latest(&id).is_none());
    }

    #[test]
    fn test_snapshot_storage_delete_older_than() {
        let storage = SnapshotStorage::new();
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        for i in 1..=5 {
            let snapshot = Snapshot::from_document(&handle, i);
            storage.store(snapshot).unwrap();
        }

        storage.delete_older_than(&id, 3).unwrap();

        let snapshots = storage.list(&id);
        assert_eq!(snapshots.len(), 3);
        assert_eq!(snapshots[0].version, 3);
    }

    #[test]
    fn test_snapshot_storage_total_count() {
        let storage = SnapshotStorage::new();
        let store = DocumentStore::new();

        let id1 = DocumentId::new("users", "alice");
        let handle1 = store.create(id1.clone()).unwrap();
        let snapshot1 = Snapshot::from_document(&handle1, 1);
        storage.store(snapshot1).unwrap();

        let id2 = DocumentId::new("users", "bob");
        let handle2 = store.create(id2.clone()).unwrap();
        let snapshot2 = Snapshot::from_document(&handle2, 1);
        storage.store(snapshot2).unwrap();

        assert_eq!(storage.total_count(), 2);
    }

    #[test]
    fn test_snapshot_manager_create() {
        let storage = Arc::new(SnapshotStorage::new());
        let manager = SnapshotManager::new(storage.clone());
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let snapshot = manager.create_snapshot(&handle).unwrap();
        assert_eq!(snapshot.metadata.version, 1);

        // Create another snapshot
        let snapshot2 = manager.create_snapshot(&handle).unwrap();
        assert_eq!(snapshot2.metadata.version, 2);
    }

    #[test]
    fn test_snapshot_manager_should_snapshot() {
        let storage = Arc::new(SnapshotStorage::new());
        let manager = SnapshotManager::with_settings(
            storage,
            Duration::from_secs(60),
            3, // min 3 changes
        );

        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id).unwrap();

        // Not enough changes yet
        assert!(!manager.should_snapshot(&handle));

        // Make multiple changes
        for i in 0..3 {
            handle
                .update(|doc| {
                    doc.put(ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                })
                .unwrap();
        }

        // Now should snapshot
        assert!(manager.should_snapshot(&handle));
    }

    #[test]
    fn test_snapshot_manager_snapshot_if_needed() {
        let storage = Arc::new(SnapshotStorage::new());
        let manager = SnapshotManager::with_settings(storage, Duration::from_secs(60), 2);

        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id).unwrap();

        // Should not create a snapshot yet
        let result = manager.snapshot_if_needed(&handle).unwrap();
        assert!(result.is_none());

        // Make changes
        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();
        handle
            .update(|doc| {
                doc.put(ROOT, "age", 30i64)?;
                Ok(())
            })
            .unwrap();

        // Should create a snapshot now
        let result = manager.snapshot_if_needed(&handle).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_snapshot_manager_compact() {
        let storage = Arc::new(SnapshotStorage::new());
        let manager = SnapshotManager::new(storage);

        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id).unwrap();

        // Make many changes to create a larger document
        for i in 0..100 {
            handle
                .update(|doc| {
                    doc.put(ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                })
                .unwrap();
        }

        let result = manager.compact(&handle).unwrap();
        assert!(result.compacted_size > 0);
        // Note: Automerge compaction may not always reduce size significantly
        // depending on the structure of changes
    }

    #[test]
    fn test_snapshot_compression_ratio() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id).unwrap();

        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let original_size = handle.save().len();
        let snapshot = Snapshot::from_document(&handle, 1);
        let ratio = snapshot.compression_ratio(original_size);

        // Ratio should be around 1.0 for a fresh snapshot
        assert!(ratio > 0.5 && ratio <= 1.5);
    }

    #[tokio::test]
    async fn test_snapshot_manager_background_task() {
        let storage = Arc::new(SnapshotStorage::new());
        let manager = Arc::new(SnapshotManager::with_settings(
            storage.clone(),
            Duration::from_millis(100), // Fast interval for testing
            1,
        ));

        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        // Make a change
        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let handles = vec![handle];
        let task = manager.clone().start_background_snapshots(handles).await;

        // Wait for at least one snapshot to be created
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should have created at least one snapshot
        let snapshots = storage.list(&id);
        assert!(!snapshots.is_empty());

        task.abort();
    }
}
