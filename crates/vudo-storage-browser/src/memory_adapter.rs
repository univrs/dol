//! In-memory storage adapter for browser environments.
//!
//! This is a high-performance in-memory implementation suitable for:
//! - Testing and development
//! - Prototype applications
//! - Fallback when persistent storage is not available
//!
//! For production use, consider the OPFS+SQLite backend (planned).

use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;
use vudo_storage::{Operation, QueryFilter, Result, StorageAdapter, StorageStats};

/// Document entry with metadata.
#[derive(Debug, Clone)]
struct DocumentEntry {
    data: Bytes,
    updated_at: u64,
}

/// Snapshot entry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SnapshotEntry {
    version: u64,
    data: Bytes,
    created_at: u64,
}

/// In-memory storage adapter.
///
/// This adapter stores all data in memory using concurrent data structures.
/// It's fast but data is lost when the page reloads.
pub struct MemoryAdapter {
    /// Documents stored by namespace and ID.
    documents: Arc<DashMap<String, DashMap<String, DocumentEntry>>>,
    /// Operations queue.
    operations: Arc<RwLock<Vec<Operation>>>,
    /// Snapshots stored by namespace, ID, and version.
    #[allow(clippy::type_complexity)]
    snapshots: Arc<DashMap<String, DashMap<String, BTreeMap<u64, SnapshotEntry>>>>,
}

impl MemoryAdapter {
    /// Create a new in-memory adapter.
    pub fn new() -> Self {
        Self {
            documents: Arc::new(DashMap::new()),
            operations: Arc::new(RwLock::new(Vec::new())),
            snapshots: Arc::new(DashMap::new()),
        }
    }

    /// Get current timestamp in milliseconds.
    fn timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Get or create namespace map for documents.
    #[allow(clippy::type_complexity)]
    fn get_namespace(&self, namespace: &str) -> dashmap::mapref::one::Ref<'_, String, DashMap<String, DocumentEntry>> {
        if !self.documents.contains_key(namespace) {
            self.documents.insert(namespace.to_string(), DashMap::new());
        }
        self.documents.get(namespace).unwrap()
    }

    /// Get or create namespace map for snapshots.
    #[allow(clippy::type_complexity)]
    fn get_snapshot_namespace(
        &self,
        namespace: &str,
    ) -> dashmap::mapref::one::Ref<'_, String, DashMap<String, BTreeMap<u64, SnapshotEntry>>> {
        if !self.snapshots.contains_key(namespace) {
            self.snapshots
                .insert(namespace.to_string(), DashMap::new());
        }
        self.snapshots.get(namespace).unwrap()
    }
}

impl Default for MemoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageAdapter for MemoryAdapter {
    async fn init(&self) -> Result<()> {
        // No initialization needed for in-memory storage
        Ok(())
    }

    async fn save(&self, namespace: &str, id: &str, data: Bytes) -> Result<()> {
        let ns = self.get_namespace(namespace);
        let entry = DocumentEntry {
            data,
            updated_at: Self::timestamp(),
        };
        ns.insert(id.to_string(), entry);
        Ok(())
    }

    async fn load(&self, namespace: &str, id: &str) -> Result<Option<Bytes>> {
        if let Some(ns) = self.documents.get(namespace) {
            Ok(ns.get(id).map(|entry| entry.data.clone()))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, namespace: &str, id: &str) -> Result<()> {
        if let Some(ns) = self.documents.get(namespace) {
            ns.remove(id);
        }
        Ok(())
    }

    async fn list(&self, namespace: &str) -> Result<Vec<String>> {
        if let Some(ns) = self.documents.get(namespace) {
            let mut ids: Vec<String> = ns.iter().map(|entry| entry.key().clone()).collect();
            ids.sort();
            Ok(ids)
        } else {
            Ok(vec![])
        }
    }

    async fn save_operations(&self, ops: &[Operation]) -> Result<()> {
        let mut operations = self.operations.write();
        operations.clear();
        operations.extend_from_slice(ops);
        Ok(())
    }

    async fn load_operations(&self) -> Result<Vec<Operation>> {
        Ok(self.operations.read().clone())
    }

    async fn save_snapshot(
        &self,
        namespace: &str,
        id: &str,
        version: u64,
        data: Bytes,
    ) -> Result<()> {
        let ns = self.get_snapshot_namespace(namespace);

        // Get or create the document's snapshot map
        if !ns.contains_key(id) {
            ns.insert(id.to_string(), BTreeMap::new());
        }

        if let Some(mut doc_snapshots) = ns.get_mut(id) {
            let entry = SnapshotEntry {
                version,
                data,
                created_at: Self::timestamp(),
            };
            doc_snapshots.insert(version, entry);
        }

        Ok(())
    }

    async fn load_snapshot(&self, namespace: &str, id: &str) -> Result<Option<(u64, Bytes)>> {
        if let Some(ns) = self.snapshots.get(namespace) {
            if let Some(doc_snapshots) = ns.get(id) {
                // Get the latest snapshot (highest version)
                if let Some((version, entry)) = doc_snapshots.iter().next_back() {
                    return Ok(Some((*version, entry.data.clone())));
                }
            }
        }
        Ok(None)
    }

    async fn query(&self, namespace: &str, filter: QueryFilter) -> Result<Vec<(String, Bytes)>> {
        if let Some(ns) = self.documents.get(namespace) {
            let mut results: Vec<(String, Bytes)> = ns
                .iter()
                .filter(|entry| matches_filter(entry.value(), &filter))
                .map(|entry| (entry.key().clone(), entry.value().data.clone()))
                .collect();

            results.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(results)
        } else {
            Ok(vec![])
        }
    }

    async fn stats(&self) -> Result<StorageStats> {
        let document_count: usize = self
            .documents
            .iter()
            .map(|ns| ns.value().len())
            .sum();

        let total_document_size: usize = self
            .documents
            .iter()
            .map(|ns| {
                ns.value()
                    .iter()
                    .map(|entry| entry.value().data.len())
                    .sum::<usize>()
            })
            .sum();

        let operation_count = self.operations.read().len();

        let snapshot_count: usize = self
            .snapshots
            .iter()
            .map(|ns| {
                ns.value()
                    .iter()
                    .map(|entry| entry.value().len())
                    .sum::<usize>()
            })
            .sum();

        let total_snapshot_size: usize = self
            .snapshots
            .iter()
            .map(|ns| {
                ns.value()
                    .iter()
                    .map(|entry| {
                        entry
                            .value()
                            .values()
                            .map(|snapshot| snapshot.data.len())
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            })
            .sum();

        Ok(StorageStats {
            document_count,
            total_document_size,
            operation_count,
            snapshot_count,
            total_snapshot_size,
        })
    }

    async fn clear(&self) -> Result<()> {
        self.documents.clear();
        self.operations.write().clear();
        self.snapshots.clear();
        Ok(())
    }
}

/// Check if a document entry matches a filter.
fn matches_filter(entry: &DocumentEntry, filter: &QueryFilter) -> bool {
    match filter {
        QueryFilter::All => true,
        QueryFilter::UpdatedAfter(timestamp) => entry.updated_at > *timestamp,
        QueryFilter::UpdatedBefore(timestamp) => entry.updated_at < *timestamp,
        QueryFilter::UpdatedBetween { start, end } => {
            entry.updated_at >= *start && entry.updated_at <= *end
        }
        QueryFilter::And(filters) => filters.iter().all(|f| matches_filter(entry, f)),
        QueryFilter::Or(filters) => filters.iter().any(|f| matches_filter(entry, f)),
        QueryFilter::Not(f) => !matches_filter(entry, f),
        QueryFilter::Field { .. } => {
            // Field filtering not supported in in-memory adapter
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_adapter_new() {
        let adapter = MemoryAdapter::new();
        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.document_count, 0);
    }

    #[tokio::test]
    async fn test_memory_adapter_init() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_adapter_save_and_load() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        let data = Bytes::from("test document");
        adapter.save("users", "alice", data.clone()).await.unwrap();

        let loaded = adapter.load("users", "alice").await.unwrap();
        assert_eq!(loaded, Some(data));
    }

    #[tokio::test]
    async fn test_memory_adapter_load_nonexistent() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        let loaded = adapter.load("users", "bob").await.unwrap();
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_memory_adapter_delete() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        let data = Bytes::from("test document");
        adapter.save("users", "alice", data).await.unwrap();

        adapter.delete("users", "alice").await.unwrap();

        let loaded = adapter.load("users", "alice").await.unwrap();
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_memory_adapter_list() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("data1"))
            .await
            .unwrap();
        adapter
            .save("users", "bob", Bytes::from("data2"))
            .await
            .unwrap();
        adapter
            .save("posts", "post1", Bytes::from("data3"))
            .await
            .unwrap();

        let users = adapter.list("users").await.unwrap();
        assert_eq!(users, vec!["alice", "bob"]);

        let posts = adapter.list("posts").await.unwrap();
        assert_eq!(posts, vec!["post1"]);
    }

    #[tokio::test]
    async fn test_memory_adapter_operations() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        let ops = vec![
            Operation::new(1, "users", "alice", vudo_storage::operation::OperationType::Create),
            Operation::new(2, "users", "bob", vudo_storage::operation::OperationType::Create),
        ];

        adapter.save_operations(&ops).await.unwrap();

        let loaded_ops = adapter.load_operations().await.unwrap();
        assert_eq!(loaded_ops.len(), 2);
        assert_eq!(loaded_ops[0].id, 1);
        assert_eq!(loaded_ops[1].id, 2);
    }

    #[tokio::test]
    async fn test_memory_adapter_snapshots() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        let data1 = Bytes::from("snapshot v1");
        let data2 = Bytes::from("snapshot v2");

        adapter
            .save_snapshot("users", "alice", 1, data1.clone())
            .await
            .unwrap();
        adapter
            .save_snapshot("users", "alice", 2, data2.clone())
            .await
            .unwrap();

        let snapshot = adapter.load_snapshot("users", "alice").await.unwrap();
        assert_eq!(snapshot, Some((2, data2))); // Should get latest version
    }

    #[tokio::test]
    async fn test_memory_adapter_query_all() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("data1"))
            .await
            .unwrap();
        adapter
            .save("users", "bob", Bytes::from("data2"))
            .await
            .unwrap();

        let results = adapter.query("users", QueryFilter::All).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_adapter_query_updated_after() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        let before_timestamp = MemoryAdapter::timestamp();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        adapter
            .save("users", "alice", Bytes::from("data1"))
            .await
            .unwrap();

        let results = adapter
            .query("users", QueryFilter::updated_after(before_timestamp))
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "alice");
    }

    #[tokio::test]
    async fn test_memory_adapter_query_updated_between() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        let start = MemoryAdapter::timestamp();

        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

        adapter
            .save("users", "alice", Bytes::from("data1"))
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let end = MemoryAdapter::timestamp();

        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

        adapter
            .save("users", "bob", Bytes::from("data2"))
            .await
            .unwrap();

        let results = adapter
            .query("users", QueryFilter::updated_between(start, end))
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "alice");
    }

    #[tokio::test]
    async fn test_memory_adapter_stats() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("test"))
            .await
            .unwrap();

        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.document_count, 1);
        assert!(stats.total_document_size > 0);
    }

    #[tokio::test]
    async fn test_memory_adapter_clear() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("data"))
            .await
            .unwrap();

        adapter.clear().await.unwrap();

        let loaded = adapter.load("users", "alice").await.unwrap();
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_memory_adapter_concurrent_access() {
        let adapter = Arc::new(MemoryAdapter::new());
        adapter.init().await.unwrap();

        let mut handles = vec![];

        for i in 0..100 {
            let adapter_clone = Arc::clone(&adapter);
            let handle = tokio::spawn(async move {
                let data = Bytes::from(format!("data{}", i));
                adapter_clone
                    .save("users", &format!("user{}", i), data)
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let users = adapter.list("users").await.unwrap();
        assert_eq!(users.len(), 100);
    }

    #[tokio::test]
    async fn test_memory_adapter_multiple_namespaces() {
        let adapter = MemoryAdapter::new();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("user1"))
            .await
            .unwrap();
        adapter
            .save("posts", "post1", Bytes::from("post1"))
            .await
            .unwrap();
        adapter
            .save("comments", "comment1", Bytes::from("comment1"))
            .await
            .unwrap();

        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.document_count, 3);
    }
}
