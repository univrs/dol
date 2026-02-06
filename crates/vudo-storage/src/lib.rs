//! Platform-agnostic storage trait for VUDO Runtime persistence.
//!
//! This crate provides the core [`StorageAdapter`] trait that all platform-specific
//! storage implementations must implement. It supports:
//! - Document persistence (save/load/delete)
//! - Operation queue persistence
//! - Snapshot management
//! - Query capabilities
//!
//! # Platform Implementations
//!
//! - **vudo-storage-browser**: Browser storage using OPFS + SQLite WASM
//! - **vudo-storage-native**: Desktop/Mobile/Server storage using native SQLite
//!
//! # Example
//!
//! ```no_run
//! use vudo_storage::{StorageAdapter, QueryFilter};
//! use bytes::Bytes;
//!
//! async fn example(storage: impl StorageAdapter) {
//!     // Initialize storage
//!     storage.init().await.unwrap();
//!
//!     // Save a document
//!     let data = Bytes::from("document content");
//!     storage.save("users", "alice", data.clone()).await.unwrap();
//!
//!     // Load it back
//!     let loaded = storage.load("users", "alice").await.unwrap();
//!     assert_eq!(loaded, Some(data));
//! }
//! ```

pub mod error;
pub mod operation;
pub mod query;

pub use error::{Result, StorageError};
pub use operation::Operation;
pub use query::QueryFilter;

use async_trait::async_trait;
use bytes::Bytes;

/// Platform-agnostic storage adapter trait.
///
/// All platform-specific storage implementations must implement this trait to
/// provide persistent storage for VUDO documents, operations, and snapshots.
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    /// Initialize storage (create tables, indexes, etc.).
    ///
    /// This should be called once before any other operations. It should be
    /// idempotent - calling it multiple times should be safe.
    async fn init(&self) -> Result<()>;

    /// Save a document.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Document namespace (e.g., "users", "posts")
    /// * `id` - Document ID within the namespace
    /// * `data` - Serialized document data
    async fn save(&self, namespace: &str, id: &str, data: Bytes) -> Result<()>;

    /// Load a document.
    ///
    /// Returns `None` if the document doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Document namespace
    /// * `id` - Document ID within the namespace
    async fn load(&self, namespace: &str, id: &str) -> Result<Option<Bytes>>;

    /// Delete a document.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Document namespace
    /// * `id` - Document ID within the namespace
    async fn delete(&self, namespace: &str, id: &str) -> Result<()>;

    /// List document IDs in a namespace.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Document namespace to list
    async fn list(&self, namespace: &str) -> Result<Vec<String>>;

    /// Save operation queue.
    ///
    /// Persists the operation queue for offline mutation tracking. This
    /// completely replaces the existing queue.
    ///
    /// # Arguments
    ///
    /// * `ops` - Operations to persist
    async fn save_operations(&self, ops: &[Operation]) -> Result<()>;

    /// Load operation queue.
    ///
    /// Retrieves all persisted operations in order.
    async fn load_operations(&self) -> Result<Vec<Operation>>;

    /// Save a snapshot.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Document namespace
    /// * `id` - Document ID
    /// * `version` - Snapshot version number
    /// * `data` - Serialized snapshot data
    async fn save_snapshot(
        &self,
        namespace: &str,
        id: &str,
        version: u64,
        data: Bytes,
    ) -> Result<()>;

    /// Load the latest snapshot for a document.
    ///
    /// Returns `None` if no snapshots exist for this document.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Document namespace
    /// * `id` - Document ID
    ///
    /// # Returns
    ///
    /// A tuple of (version, data) if a snapshot exists.
    async fn load_snapshot(&self, namespace: &str, id: &str) -> Result<Option<(u64, Bytes)>>;

    /// Query documents with a filter.
    ///
    /// This is an optional capability for indexed queries. Implementations
    /// may return an error if they don't support certain filter types.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Document namespace to query
    /// * `filter` - Query filter
    ///
    /// # Returns
    ///
    /// A vector of (id, data) tuples matching the filter.
    async fn query(&self, namespace: &str, filter: QueryFilter) -> Result<Vec<(String, Bytes)>>;

    /// Get storage statistics.
    ///
    /// Returns statistics about the storage (sizes, counts, etc.).
    async fn stats(&self) -> Result<StorageStats> {
        // Default implementation - adapters can override for more accurate stats
        Ok(StorageStats::default())
    }

    /// Clear all data.
    ///
    /// This is primarily for testing. It removes all documents, operations,
    /// and snapshots.
    async fn clear(&self) -> Result<()>;
}

/// Storage statistics.
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// Number of documents stored.
    pub document_count: usize,
    /// Total size of all documents in bytes.
    pub total_document_size: usize,
    /// Number of operations in the queue.
    pub operation_count: usize,
    /// Number of snapshots stored.
    pub snapshot_count: usize,
    /// Total size of all snapshots in bytes.
    pub total_snapshot_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock adapter for testing the trait
    struct MockAdapter;

    #[async_trait]
    impl StorageAdapter for MockAdapter {
        async fn init(&self) -> Result<()> {
            Ok(())
        }

        async fn save(&self, _namespace: &str, _id: &str, _data: Bytes) -> Result<()> {
            Ok(())
        }

        async fn load(&self, _namespace: &str, _id: &str) -> Result<Option<Bytes>> {
            Ok(None)
        }

        async fn delete(&self, _namespace: &str, _id: &str) -> Result<()> {
            Ok(())
        }

        async fn list(&self, _namespace: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }

        async fn save_operations(&self, _ops: &[Operation]) -> Result<()> {
            Ok(())
        }

        async fn load_operations(&self) -> Result<Vec<Operation>> {
            Ok(vec![])
        }

        async fn save_snapshot(
            &self,
            _namespace: &str,
            _id: &str,
            _version: u64,
            _data: Bytes,
        ) -> Result<()> {
            Ok(())
        }

        async fn load_snapshot(&self, _namespace: &str, _id: &str) -> Result<Option<(u64, Bytes)>> {
            Ok(None)
        }

        async fn query(&self, _namespace: &str, _filter: QueryFilter) -> Result<Vec<(String, Bytes)>> {
            Ok(vec![])
        }

        async fn clear(&self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_adapter() {
        let adapter = MockAdapter;
        adapter.init().await.unwrap();

        let data = Bytes::from("test");
        adapter.save("test", "id1", data).await.unwrap();

        let result = adapter.load("test", "id1").await.unwrap();
        assert!(result.is_none()); // Mock always returns None
    }

    #[tokio::test]
    async fn test_storage_stats_default() {
        let stats = StorageStats::default();
        assert_eq!(stats.document_count, 0);
        assert_eq!(stats.total_document_size, 0);
    }
}
