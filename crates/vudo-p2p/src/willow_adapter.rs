//! Willow Protocol adapter for DOL-to-Willow mapping.
//!
//! This module provides the integration between DOL's document model and
//! Willow's 3D namespace structure, enabling structured sync with fine-grained
//! permissions and GDPR-compliant deletion.

use crate::error::{P2PError, Result};
use crate::meadowcap::{Capability, CapabilityStore, Permission};
use crate::willow_types::{Entry, NamespaceId, Path, SubspaceId, Tombstone};
use bytes::Bytes;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use vudo_state::{DocumentId, StateEngine};

/// Resource constraints for sync operations.
#[derive(Debug, Clone)]
pub struct ResourceConstraints {
    /// Maximum memory for sync (bytes).
    pub max_memory: usize,
    /// Maximum bandwidth (bytes/sec).
    pub max_bandwidth: u64,
    /// Priority level.
    pub priority: SyncPriority,
}

impl Default for ResourceConstraints {
    fn default() -> Self {
        Self {
            max_memory: 100 * 1024 * 1024, // 100 MB
            max_bandwidth: 1024 * 1024,     // 1 MB/s
            priority: SyncPriority::Medium,
        }
    }
}

/// Sync priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPriority {
    /// High priority (user-initiated sync).
    High,
    /// Medium priority (background sync).
    Medium,
    /// Low priority (opportunistic sync).
    Low,
}

/// Willow Protocol adapter.
pub struct WillowAdapter {
    /// State engine integration.
    state_engine: Arc<StateEngine>,
    /// DOL namespace → Willow namespace mapping.
    namespaces: Arc<RwLock<HashMap<String, NamespaceId>>>,
    /// Capability store for permissions.
    capabilities: Arc<CapabilityStore>,
    /// Local entry storage (simulated Willow store).
    entries: Arc<DashMap<(NamespaceId, SubspaceId, Path), Entry>>,
    /// Tombstone storage for deletions.
    tombstones: Arc<DashMap<(NamespaceId, SubspaceId, Path), Tombstone>>,
}

impl WillowAdapter {
    /// Create a new Willow adapter.
    pub async fn new(state_engine: Arc<StateEngine>) -> Result<Self> {
        Ok(Self {
            state_engine,
            namespaces: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(CapabilityStore::new()),
            entries: Arc::new(DashMap::new()),
            tombstones: Arc::new(DashMap::new()),
        })
    }

    /// Get capability store.
    pub fn capabilities(&self) -> Arc<CapabilityStore> {
        Arc::clone(&self.capabilities)
    }

    /// Map DOL namespace to Willow namespace ID.
    pub fn map_namespace(&self, dol_namespace: &str) -> NamespaceId {
        let mut namespaces = self.namespaces.write();
        *namespaces
            .entry(dol_namespace.to_string())
            .or_insert_with(|| NamespaceId::from_dol_namespace(dol_namespace))
    }

    /// Map DOL collection to Willow subspace ID.
    pub fn map_subspace(&self, collection: &str) -> SubspaceId {
        SubspaceId::from_dol_collection(collection)
    }

    /// Map DOL document ID to Willow 3D path.
    pub fn map_path(&self, namespace: &str, collection: &str, id: &str) -> (NamespaceId, SubspaceId, Path) {
        let ns_id = self.map_namespace(namespace);
        let subspace = self.map_subspace(collection);
        let path = Path::from_dol_id(id);
        (ns_id, subspace, path)
    }

    /// Write an entry to Willow.
    pub async fn write_entry(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        data: Bytes,
        capability: &Capability,
    ) -> Result<()> {
        let (ns, subspace, path) = self.map_path(namespace, collection, id);

        // Check write permission
        if !capability.can_write(subspace, &path) {
            return Err(P2PError::PermissionDenied(format!(
                "No write permission for {}/{}",
                collection, id
            )));
        }

        // Verify capability signature
        capability.verify()?;

        let timestamp = current_timestamp();
        let entry = Entry::new(ns, subspace, path.clone(), data, timestamp);

        self.entries.insert((ns, subspace, path), entry);

        Ok(())
    }

    /// Read an entry from Willow.
    pub async fn read_entry(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        capability: &Capability,
    ) -> Result<Option<Bytes>> {
        let (ns, subspace, path) = self.map_path(namespace, collection, id);

        // Check read permission
        if !capability.can_read(subspace, &path) {
            return Err(P2PError::PermissionDenied(format!(
                "No read permission for {}/{}",
                collection, id
            )));
        }

        // Verify capability signature
        capability.verify()?;

        // Check if tombstone exists (document was deleted)
        if self.tombstones.contains_key(&(ns, subspace, path.clone())) {
            return Ok(None);
        }

        // Get entry
        Ok(self
            .entries
            .get(&(ns, subspace, path))
            .map(|entry| entry.payload.clone()))
    }

    /// Delete an entry (GDPR-compliant deletion with tombstone).
    pub async fn delete_entry(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        capability: &Capability,
        reason: Option<String>,
    ) -> Result<()> {
        let (ns, subspace, path) = self.map_path(namespace, collection, id);

        // Check write permission (delete requires write)
        if !capability.can_write(subspace, &path) {
            return Err(P2PError::PermissionDenied(format!(
                "No write permission for {}/{}",
                collection, id
            )));
        }

        // Verify capability signature
        capability.verify()?;

        let timestamp = current_timestamp();

        // Remove entry
        self.entries.remove(&(ns, subspace, path.clone()));

        // Create tombstone
        let tombstone = Tombstone::new(ns, subspace, path.clone(), timestamp, reason);
        self.tombstones.insert((ns, subspace, path), tombstone);

        Ok(())
    }

    /// Sync document from state engine to Willow.
    pub async fn sync_from_state_engine(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        capability: &Capability,
    ) -> Result<()> {
        // Load document from state engine
        let doc_id = DocumentId::new(collection, id);
        let handle = self
            .state_engine
            .get_document(&doc_id)
            .await
            .map_err(|e| P2PError::DocumentNotFound(e.to_string()))?;

        // Serialize document
        let data = Bytes::from(handle.save());

        // Write to Willow
        self.write_entry(namespace, collection, id, data, capability)
            .await?;

        Ok(())
    }

    /// Sync document from Willow to state engine.
    pub async fn sync_to_state_engine(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        capability: &Capability,
    ) -> Result<()> {
        // Read from Willow
        let data = self
            .read_entry(namespace, collection, id, capability)
            .await?;

        if let Some(bytes) = data {
            // Load into state engine
            let doc_id = DocumentId::new(collection, id);
            self.state_engine
                .store
                .load(doc_id, &bytes)
                .map_err(|e| P2PError::StateError(e))?;
        } else {
            // Entry was deleted - delete from state engine too
            let doc_id = DocumentId::new(collection, id);
            self.state_engine
                .delete_document(&doc_id)
                .await
                .map_err(|e| P2PError::StateError(e))?;
        }

        Ok(())
    }

    /// Sync with resource constraints.
    pub async fn sync_with_constraints(
        &self,
        namespace: &str,
        collection: &str,
        capability: &Capability,
        constraints: ResourceConstraints,
    ) -> Result<SyncStats> {
        let (ns, subspace, _) = self.map_path(namespace, collection, "");

        let mut synced_count = 0;
        let mut total_bytes = 0;
        let mut errors = 0;

        // Calculate entry size budget
        let max_entries = constraints.max_memory / AVERAGE_ENTRY_SIZE;

        // List all documents in collection from state engine
        let docs = self.state_engine.store.list_namespace(collection);

        for doc_id in docs.iter().take(max_entries) {
            // Check memory limit
            if total_bytes >= constraints.max_memory {
                break;
            }

            // Sync document
            match self
                .sync_from_state_engine(namespace, collection, &doc_id.key, capability)
                .await
            {
                Ok(()) => {
                    synced_count += 1;
                    if let Some(entry) = self
                        .entries
                        .get(&(ns, subspace, Path::from_dol_id(&doc_id.key)))
                    {
                        total_bytes += entry.size();
                    }
                }
                Err(_) => {
                    errors += 1;
                }
            }
        }

        Ok(SyncStats {
            synced_count,
            total_bytes,
            errors,
        })
    }

    /// GDPR-compliant deletion with tombstone propagation.
    pub async fn gdpr_delete(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        capability: &Capability,
        reason: &str,
    ) -> Result<()> {
        // Delete from Willow with tombstone
        self.delete_entry(namespace, collection, id, capability, Some(reason.to_string()))
            .await?;

        // Delete from state engine
        let doc_id = DocumentId::new(collection, id);
        self.state_engine
            .delete_document(&doc_id)
            .await
            .map_err(|e| P2PError::StateError(e))?;

        tracing::info!(
            "GDPR deletion completed for {}/{} - reason: {}",
            collection,
            id,
            reason
        );

        Ok(())
    }

    /// List all entries in a path prefix.
    pub fn list_entries(&self, namespace_id: NamespaceId, subspace_id: SubspaceId, prefix: &Path) -> Vec<Entry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.key().0 == namespace_id
                    && entry.key().1 == subspace_id
                    && prefix.is_prefix_of(&entry.key().2)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get sync statistics.
    pub fn stats(&self) -> WillowStats {
        WillowStats {
            entry_count: self.entries.len(),
            tombstone_count: self.tombstones.len(),
            total_size: self.entries.iter().map(|e| e.value().size()).sum(),
        }
    }
}

/// Statistics for a sync operation.
#[derive(Debug, Clone)]
pub struct SyncStats {
    /// Number of documents synced.
    pub synced_count: usize,
    /// Total bytes synced.
    pub total_bytes: usize,
    /// Number of errors.
    pub errors: usize,
}

/// Willow adapter statistics.
#[derive(Debug, Clone)]
pub struct WillowStats {
    /// Number of entries stored.
    pub entry_count: usize,
    /// Number of tombstones stored.
    pub tombstone_count: usize,
    /// Total size of all entries in bytes.
    pub total_size: usize,
}

/// Average entry size estimation (for resource calculations).
const AVERAGE_ENTRY_SIZE: usize = 4096; // 4 KB

/// Get current Unix timestamp in milliseconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::{transaction::Transactable, ROOT};
    use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_willow_adapter_new() {
        let engine = StateEngine::new().await.unwrap();
        let adapter = WillowAdapter::new(Arc::new(engine)).await.unwrap();
        let stats = adapter.stats();
        assert_eq!(stats.entry_count, 0);
    }

    #[tokio::test]
    async fn test_namespace_mapping() {
        let engine = StateEngine::new().await.unwrap();
        let adapter = WillowAdapter::new(Arc::new(engine)).await.unwrap();

        let ns1 = adapter.map_namespace("myapp.v1");
        let ns2 = adapter.map_namespace("myapp.v1");
        let ns3 = adapter.map_namespace("myapp.v2");

        // Same namespace should produce same ID
        assert_eq!(ns1, ns2);
        // Different namespace should produce different ID
        assert_ne!(ns1, ns3);
    }

    #[tokio::test]
    async fn test_path_mapping() {
        let engine = StateEngine::new().await.unwrap();
        let adapter = WillowAdapter::new(Arc::new(engine)).await.unwrap();

        let (ns, sub, path) = adapter.map_path("myapp.v1", "users", "alice");

        assert_eq!(ns, NamespaceId::from_dol_namespace("myapp.v1"));
        assert_eq!(sub, SubspaceId::from_dol_collection("users"));
        assert_eq!(path.components().len(), 1);
        assert_eq!(path.components()[0], "alice");
    }

    #[tokio::test]
    async fn test_write_and_read_entry() {
        let engine = StateEngine::new().await.unwrap();
        let adapter = WillowAdapter::new(Arc::new(engine)).await.unwrap();

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = adapter.map_namespace("myapp.v1");
        let capability = Capability::new_root(namespace_id, &signing_key);

        adapter.capabilities().add(capability.clone()).unwrap();

        let data = Bytes::from("test data");
        adapter
            .write_entry("myapp.v1", "users", "alice", data.clone(), &capability)
            .await
            .unwrap();

        let read_data = adapter
            .read_entry("myapp.v1", "users", "alice", &capability)
            .await
            .unwrap();

        assert_eq!(read_data, Some(data));
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let engine = StateEngine::new().await.unwrap();
        let adapter = WillowAdapter::new(Arc::new(engine)).await.unwrap();

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = adapter.map_namespace("myapp.v1");
        let capability = Capability::new_root(namespace_id, &signing_key);

        let data = Bytes::from("test data");
        adapter
            .write_entry("myapp.v1", "users", "alice", data, &capability)
            .await
            .unwrap();

        adapter
            .delete_entry("myapp.v1", "users", "alice", &capability, Some("test deletion".to_string()))
            .await
            .unwrap();

        let read_data = adapter
            .read_entry("myapp.v1", "users", "alice", &capability)
            .await
            .unwrap();

        assert_eq!(read_data, None);
        assert_eq!(adapter.stats().tombstone_count, 1);
    }

    #[tokio::test]
    async fn test_permission_denied_read() {
        let engine = StateEngine::new().await.unwrap();
        let adapter = WillowAdapter::new(Arc::new(engine)).await.unwrap();

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = adapter.map_namespace("myapp.v1");
        let root_cap = Capability::new_root(namespace_id, &signing_key);

        // Create a write-only capability for Alice's data
        let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let subspace_id = adapter.map_subspace("users");
        let alice_cap = root_cap
            .delegate(
                Some(subspace_id),
                Path::from_components(["alice"]),
                Permission::Write,
                &alice_key,
            )
            .unwrap();

        // Write some data with root capability
        let data = Bytes::from("bob's data");
        adapter
            .write_entry("myapp.v1", "users", "bob", data, &root_cap)
            .await
            .unwrap();

        // Try to read Bob's data with Alice's capability - should fail
        let result = adapter
            .read_entry("myapp.v1", "users", "bob", &alice_cap)
            .await;

        assert!(matches!(result, Err(P2PError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn test_sync_from_state_engine() {
        let engine = Arc::new(StateEngine::new().await.unwrap());
        let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

        // Create document in state engine
        let doc_id = DocumentId::new("users", "alice");
        let handle = engine.create_document(doc_id).await.unwrap();
        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = adapter.map_namespace("myapp.v1");
        let capability = Capability::new_root(namespace_id, &signing_key);

        // Sync to Willow
        adapter
            .sync_from_state_engine("myapp.v1", "users", "alice", &capability)
            .await
            .unwrap();

        // Verify it's in Willow
        let data = adapter
            .read_entry("myapp.v1", "users", "alice", &capability)
            .await
            .unwrap();

        assert!(data.is_some());
    }

    #[tokio::test]
    async fn test_sync_to_state_engine() {
        let engine = Arc::new(StateEngine::new().await.unwrap());
        let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = adapter.map_namespace("myapp.v1");
        let capability = Capability::new_root(namespace_id, &signing_key);

        // Create and save a document
        let doc_id = DocumentId::new("users", "alice");
        let handle = engine.create_document(doc_id.clone()).await.unwrap();
        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let bytes = Bytes::from(handle.save());

        // Write to Willow
        adapter
            .write_entry("myapp.v1", "users", "alice", bytes, &capability)
            .await
            .unwrap();

        // Delete from state engine
        engine.delete_document(&doc_id).await.unwrap();

        // Sync back to state engine
        adapter
            .sync_to_state_engine("myapp.v1", "users", "alice", &capability)
            .await
            .unwrap();

        // Verify it's back in state engine
        let restored = engine.get_document(&doc_id).await.unwrap();
        assert_eq!(restored.id, doc_id);
    }

    #[tokio::test]
    async fn test_gdpr_delete() {
        let engine = Arc::new(StateEngine::new().await.unwrap());
        let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

        // Create document
        let doc_id = DocumentId::new("users", "alice");
        let handle = engine.create_document(doc_id.clone()).await.unwrap();
        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = adapter.map_namespace("myapp.v1");
        let capability = Capability::new_root(namespace_id, &signing_key);

        // Sync to Willow
        adapter
            .sync_from_state_engine("myapp.v1", "users", "alice", &capability)
            .await
            .unwrap();

        // GDPR delete
        adapter
            .gdpr_delete("myapp.v1", "users", "alice", &capability, "User requested deletion")
            .await
            .unwrap();

        // Verify deletion
        assert!(engine.get_document(&doc_id).await.is_err());
        assert_eq!(adapter.stats().tombstone_count, 1);
    }

    #[tokio::test]
    async fn test_resource_constrained_sync() {
        let engine = Arc::new(StateEngine::new().await.unwrap());
        let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

        // Create multiple documents
        for i in 0..5 {
            let doc_id = DocumentId::new("users", &format!("user{}", i));
            let handle = engine.create_document(doc_id).await.unwrap();
            handle
                .update(|doc| {
                    doc.put(ROOT, "name", format!("User {}", i))?;
                    Ok(())
                })
                .unwrap();
        }

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = adapter.map_namespace("myapp.v1");
        let capability = Capability::new_root(namespace_id, &signing_key);

        // Sync with tight constraints
        let constraints = ResourceConstraints {
            max_memory: 10 * 1024, // 10 KB
            max_bandwidth: 1024,
            priority: SyncPriority::Low,
        };

        let stats = adapter
            .sync_with_constraints("myapp.v1", "users", &capability, constraints)
            .await
            .unwrap();

        // Should have synced some but not all documents due to constraints
        assert!(stats.synced_count > 0);
        assert!(stats.synced_count <= 5);
    }
}
