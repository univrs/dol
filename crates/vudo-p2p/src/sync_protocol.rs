//! Automerge sync protocol over Iroh connections.

use crate::error::{P2PError, Result};
use automerge::{AutoCommit, Change};
use bytes::Bytes;
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
use vudo_state::{DocumentHandle, DocumentId, StateEngine};

/// Peer ID (Iroh node ID).
pub type PeerId = String;

/// Sync message protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Request sync for document.
    SyncRequest {
        /// Document namespace.
        namespace: String,
        /// Document key.
        id: String,
        /// Last sync timestamp (milliseconds since epoch).
        last_sync: Option<u64>,
    },

    /// Send Automerge changes.
    SyncChanges {
        /// Document namespace.
        namespace: String,
        /// Document key.
        id: String,
        /// Serialized Automerge changes.
        changes: Vec<Vec<u8>>,
    },

    /// Acknowledge sync completion.
    SyncComplete {
        /// Document namespace.
        namespace: String,
        /// Document key.
        id: String,
        /// Document version (number of changes).
        version: u64,
    },

    /// Request full document (initial sync).
    FullSync {
        /// Document namespace.
        namespace: String,
        /// Document key.
        id: String,
    },

    /// Send full document.
    FullDocument {
        /// Document namespace.
        namespace: String,
        /// Document key.
        id: String,
        /// Serialized Automerge document.
        document: Vec<u8>,
    },

    /// Heartbeat to keep connection alive.
    Heartbeat,

    /// Error response.
    Error {
        /// Error message.
        message: String,
    },
}

impl SyncMessage {
    /// Serialize message to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(P2PError::from)
    }

    /// Deserialize message from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(P2PError::from)
    }
}

/// Sync metadata for a document.
#[derive(Debug, Clone)]
struct SyncMetadata {
    /// Last sync timestamp.
    last_sync: u64,
    /// Document version at last sync.
    version: u64,
    /// Number of sync operations.
    sync_count: u64,
}

/// Sync state tracker.
struct SyncState {
    /// Per-peer, per-document sync metadata.
    /// Key: (peer_id, namespace, document_id)
    state: HashMap<(PeerId, String, String), SyncMetadata>,
    /// LRU cache to bound memory usage.
    cache: LruCache<(PeerId, String, String), SyncMetadata>,
}

impl SyncState {
    fn new(capacity: usize) -> Self {
        Self {
            state: HashMap::new(),
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    fn get(&mut self, peer: &PeerId, namespace: &str, id: &str) -> Option<SyncMetadata> {
        let key = (peer.clone(), namespace.to_string(), id.to_string());
        self.cache.get(&key).cloned()
    }

    fn update(&mut self, peer: &PeerId, namespace: &str, id: &str, metadata: SyncMetadata) {
        let key = (peer.clone(), namespace.to_string(), id.to_string());
        self.cache.put(key.clone(), metadata.clone());
        self.state.insert(key, metadata);
    }

    fn remove(&mut self, peer: &PeerId, namespace: &str, id: &str) {
        let key = (peer.clone(), namespace.to_string(), id.to_string());
        self.cache.pop(&key);
        self.state.remove(&key);
    }
}

/// Sync protocol handler.
pub struct SyncProtocol {
    /// State engine.
    state_engine: Arc<StateEngine>,
    /// Sync state tracker.
    sync_state: Arc<RwLock<SyncState>>,
}

impl SyncProtocol {
    /// Create a new sync protocol handler.
    pub fn new(state_engine: Arc<StateEngine>) -> Self {
        Self {
            state_engine,
            sync_state: Arc::new(RwLock::new(SyncState::new(10_000))),
        }
    }

    /// Handle incoming sync request.
    pub async fn handle_sync_request(
        &self,
        peer: &PeerId,
        namespace: String,
        id: String,
        last_sync: Option<u64>,
    ) -> Result<SyncMessage> {
        debug!(
            "Handling sync request from peer {} for {}/{}",
            peer, namespace, id
        );

        let doc_id = DocumentId::new(&namespace, &id);

        // Get document handle
        let handle = self
            .state_engine
            .get_document(&doc_id)
            .await
            .map_err(|_| P2PError::DocumentNotFound(doc_id.to_string()))?;

        // If this is initial sync or no last_sync timestamp, send full document
        if last_sync.is_none() {
            let document_bytes = handle.save();
            info!(
                "Sending full document {}/{} ({} bytes) to peer {}",
                namespace,
                id,
                document_bytes.len(),
                peer
            );
            return Ok(SyncMessage::FullDocument {
                namespace,
                id,
                document: document_bytes,
            });
        }

        // Get changes since last sync
        let changes = self.get_changes_since(&handle, last_sync.unwrap())?;

        if changes.is_empty() {
            debug!("No changes for {}/{} since last sync", namespace, id);
            return Ok(SyncMessage::SyncComplete {
                namespace,
                id,
                version: handle.metadata().version,
            });
        }

        info!(
            "Sending {} changes for {}/{} to peer {}",
            changes.len(),
            namespace,
            id,
            peer
        );

        Ok(SyncMessage::SyncChanges {
            namespace,
            id,
            changes,
        })
    }

    /// Apply incoming sync changes.
    pub async fn apply_sync_changes(
        &self,
        peer: &PeerId,
        namespace: String,
        id: String,
        changes: Vec<Vec<u8>>,
    ) -> Result<()> {
        info!(
            "Applying {} changes from peer {} for {}/{}",
            changes.len(),
            peer,
            namespace,
            id
        );

        let doc_id = DocumentId::new(&namespace, &id);

        // Get or create document
        let handle = match self.state_engine.get_document(&doc_id).await {
            Ok(h) => h,
            Err(_) => self.state_engine.create_document(doc_id.clone()).await?,
        };

        // Apply changes
        handle.update(|doc| {
            for change_bytes in &changes {
                doc.load_incremental(change_bytes)
                    .map_err(|e| vudo_state::StateError::Internal(e.to_string()))?;
            }
            Ok(())
        })?;

        // Update sync state
        let metadata = SyncMetadata {
            last_sync: current_timestamp(),
            version: handle.metadata().version,
            sync_count: self
                .sync_state
                .read()
                .state
                .get(&(peer.clone(), namespace.clone(), id.clone()))
                .map(|m| m.sync_count + 1)
                .unwrap_or(1),
        };

        self.sync_state
            .write()
            .update(peer, &namespace, &id, metadata);

        info!("Successfully applied changes for {}/{}", namespace, id);
        Ok(())
    }

    /// Apply full document.
    pub async fn apply_full_document(
        &self,
        peer: &PeerId,
        namespace: String,
        id: String,
        document_bytes: Vec<u8>,
    ) -> Result<()> {
        info!(
            "Applying full document from peer {} for {}/{} ({} bytes)",
            peer,
            namespace,
            id,
            document_bytes.len()
        );

        let doc_id = DocumentId::new(&namespace, &id);

        // Create or get document handle
        let handle = match self.state_engine.get_document(&doc_id).await {
            Ok(h) => h,
            Err(_) => self.state_engine.create_document(doc_id.clone()).await?,
        };

        // Load the document bytes into the existing handle
        // Note: This is simplified - in production, you'd use Automerge's load_incremental
        // or merge capabilities to properly integrate the remote document
        handle.update(|doc| {
            // Load incremental changes from the full document bytes
            doc.load_incremental(&document_bytes)
                .map_err(|e| vudo_state::StateError::Internal(e.to_string()))?;
            Ok(())
        })?;

        // Update sync state
        let metadata = SyncMetadata {
            last_sync: current_timestamp(),
            version: 1,
            sync_count: 1,
        };

        self.sync_state
            .write()
            .update(peer, &namespace, &id, metadata);

        info!("Successfully applied full document for {}/{}", namespace, id);
        Ok(())
    }

    /// Get changes since timestamp.
    fn get_changes_since(&self, handle: &DocumentHandle, since: u64) -> Result<Vec<Vec<u8>>> {
        // Simplified implementation: return incremental changes as binary data
        // In production, you'd track which changes correspond to timestamps
        // and only send changes since the last sync

        // For now, we'll export the document as binary and let the receiver
        // use Automerge's merge capabilities
        let doc_bytes = handle.save();

        // Return as a single "change" (full document export)
        // In a real implementation, Automerge supports incremental sync
        // via save_incremental() and load_incremental()
        Ok(vec![doc_bytes])
    }

    /// Request sync for a document.
    pub fn create_sync_request(
        &self,
        peer: &PeerId,
        namespace: &str,
        id: &str,
    ) -> Result<SyncMessage> {
        let mut sync_state = self.sync_state.write();
        let last_sync = sync_state.get(peer, namespace, id).map(|m| m.last_sync);

        Ok(SyncMessage::SyncRequest {
            namespace: namespace.to_string(),
            id: id.to_string(),
            last_sync,
        })
    }

    /// Clear sync state for a peer.
    pub fn clear_peer_state(&self, peer: &PeerId) {
        let mut state = self.sync_state.write();
        state
            .state
            .retain(|(p, _, _), _| p != peer);
    }

    /// Get sync statistics.
    pub fn get_stats(&self) -> SyncStats {
        let state = self.sync_state.read();
        SyncStats {
            tracked_documents: state.state.len(),
            total_sync_count: state.state.values().map(|m| m.sync_count).sum(),
        }
    }
}

/// Sync statistics.
#[derive(Debug, Clone)]
pub struct SyncStats {
    /// Number of tracked documents.
    pub tracked_documents: usize,
    /// Total number of sync operations.
    pub total_sync_count: u64,
}

/// Get current timestamp in milliseconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_serialization() {
        let msg = SyncMessage::SyncRequest {
            namespace: "users".to_string(),
            id: "alice".to_string(),
            last_sync: Some(12345),
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = SyncMessage::from_bytes(&bytes).unwrap();

        match decoded {
            SyncMessage::SyncRequest {
                namespace,
                id,
                last_sync,
            } => {
                assert_eq!(namespace, "users");
                assert_eq!(id, "alice");
                assert_eq!(last_sync, Some(12345));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_sync_state() {
        let mut state = SyncState::new(100);

        let metadata = SyncMetadata {
            last_sync: 12345,
            version: 1,
            sync_count: 1,
        };

        state.update(&"peer1".to_string(), "users", "alice", metadata.clone());

        let retrieved = state.get(&"peer1".to_string(), "users", "alice");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().last_sync, 12345);
    }

    #[tokio::test]
    async fn test_sync_protocol_creation() {
        let engine = Arc::new(StateEngine::new().await.unwrap());
        let protocol = SyncProtocol::new(engine);

        let stats = protocol.get_stats();
        assert_eq!(stats.tracked_documents, 0);
    }
}
