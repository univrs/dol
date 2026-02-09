//! VUDO P2P Integration Layer
//!
//! Iroh-based peer-to-peer networking for VUDO Runtime with:
//! - Peer discovery (DHT + mDNS) via Iroh
//! - Connection management (direct + relay)
//! - Automerge sync protocol over Iroh streams
//! - Willow Protocol adapter for structured data sync
//! - Meadowcap capabilities for fine-grained permissions
//! - Gossip overlay for presence
//! - Bandwidth-aware sync
//! - Background sync in Web Workers/tokio
//! - GDPR-compliant deletion with tombstones
//!
//! # Architecture
//!
//! The P2P layer combines two complementary protocols:
//!
//! ## Iroh P2P Networking
//! - QUIC-based encrypted connections
//! - mDNS for local network discovery
//! - DHT for internet-wide discovery
//! - Relay servers for NAT traversal
//!
//! ## Willow Protocol Data Sync
//! - 3D namespace structure (namespace, subspace, path)
//! - Fine-grained capabilities (Meadowcap)
//! - GDPR-compliant deletion (tombstones)
//! - Resource-aware sync
//!
//! ## DOL Mapping
//! - **DOL System → Willow Namespace**: System identifiers hashed to namespace IDs
//! - **DOL Collection → Willow Subspace**: Collection names map to subspace IDs
//! - **DOL Document → Willow Path**: Document IDs become hierarchical paths
//!
//! # Examples
//!
//! ## Basic P2P Setup
//!
//! ```no_run
//! use vudo_p2p::{VudoP2P, P2PConfig};
//! use vudo_state::StateEngine;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_p2p::error::Result<()> {
//! // Create state engine
//! let state_engine = Arc::new(StateEngine::new().await?);
//!
//! // Create P2P layer with Iroh networking
//! let config = P2PConfig::default();
//! let p2p = VudoP2P::new(state_engine, config).await?;
//!
//! // Start P2P services
//! p2p.start().await?;
//!
//! // Discover peers
//! let peers = p2p.discover_peers().await?;
//! println!("Discovered {} peers", peers.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Willow Protocol with Capabilities
//!
//! ```no_run
//! use vudo_p2p::{WillowAdapter, Capability};
//! use vudo_state::StateEngine;
//! use std::sync::Arc;
//! use bytes::Bytes;
//! use ed25519_dalek::SigningKey;
//!
//! # async fn example() -> vudo_p2p::error::Result<()> {
//! // Initialize Willow adapter
//! let engine = Arc::new(StateEngine::new().await?);
//! let adapter = WillowAdapter::new(Arc::clone(&engine)).await?;
//!
//! // Create root capability
//! let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
//! let namespace_id = adapter.map_namespace("myapp.v1");
//! let root_cap = Capability::new_root(namespace_id, &signing_key);
//!
//! // Write data with capability
//! let data = Bytes::from("test data");
//! adapter.write_entry("myapp.v1", "users", "alice", data, &root_cap).await?;
//! # Ok(())
//! # }
//! ```

// Iroh P2P modules
pub mod background_sync;
pub mod bandwidth;
pub mod discovery;
pub mod gossip;
pub mod iroh_adapter;
pub mod sync_protocol;

// Willow Protocol modules
pub mod error;
pub mod meadowcap;
pub mod willow_adapter;
pub mod willow_types;

// Iroh P2P exports
pub use background_sync::{BackgroundSync, BackgroundSyncConfig};
pub use bandwidth::{BandwidthManager, BandwidthStats, SyncTask};
pub use discovery::{DiscoveredPeer, DiscoveryMethod, PeerDiscovery, PeerPrioritizer};
pub use gossip::{GossipMessage, GossipOverlay, Subscription, Topic};
pub use iroh_adapter::{ConnectionMetadata, IrohAdapter, P2PConfig};
pub use sync_protocol::{PeerId, SyncMessage, SyncProtocol, SyncStats};

// Willow Protocol exports
pub use error::{P2PError, Result};
pub use meadowcap::{Capability, CapabilityStore, Permission};
pub use willow_adapter::{ResourceConstraints, WillowAdapter, WillowStats};
pub use willow_types::{Entry, NamespaceId, Path, SubspaceId, Tombstone};

// Re-export SyncPriority from bandwidth (more general than Willow's)
pub use bandwidth::SyncPriority;

use iroh::net::NodeAddr;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};
use vudo_state::StateEngine;

/// Main P2P coordinator integrating Iroh and Willow.
pub struct VudoP2P {
    /// State engine.
    state_engine: Arc<StateEngine>,
    /// Iroh adapter for P2P networking.
    iroh: Arc<IrohAdapter>,
    /// Sync protocol handler.
    sync_protocol: Arc<SyncProtocol>,
    /// Gossip overlay.
    gossip: Arc<GossipOverlay>,
    /// Peer discovery.
    discovery: Arc<PeerDiscovery>,
    /// Bandwidth manager.
    bandwidth: Arc<BandwidthManager>,
    /// Background sync.
    background_sync: Arc<RwLock<Option<BackgroundSync>>>,
    /// Willow adapter (optional, for structured sync).
    willow: Option<Arc<WillowAdapter>>,
    /// Configuration.
    config: P2PConfig,
}

impl VudoP2P {
    /// Create a new P2P instance with Iroh networking.
    pub async fn new(state_engine: Arc<StateEngine>, config: P2PConfig) -> Result<Self> {
        info!("Initializing VUDO P2P layer");

        // Create Iroh adapter
        let iroh = Arc::new(IrohAdapter::new(config.clone()).await?);

        // Create sync protocol
        let sync_protocol = Arc::new(SyncProtocol::new(Arc::clone(&state_engine)));

        // Create gossip overlay
        let gossip = Arc::new(GossipOverlay::new());

        // Create peer discovery
        let discovery = Arc::new(PeerDiscovery::new(config.enable_mdns, config.enable_dht));

        // Create bandwidth manager
        let bandwidth = Arc::new(BandwidthManager::new());

        Ok(Self {
            state_engine,
            iroh,
            sync_protocol,
            gossip,
            discovery,
            bandwidth,
            background_sync: Arc::new(RwLock::new(None)),
            willow: None,
            config,
        })
    }

    /// Create a new P2P instance with Willow Protocol integration.
    pub async fn with_willow(state_engine: Arc<StateEngine>, config: P2PConfig) -> Result<Self> {
        let mut p2p = Self::new(Arc::clone(&state_engine), config).await?;

        // Create Willow adapter
        let willow = Arc::new(WillowAdapter::new(Arc::clone(&state_engine)).await?);
        p2p.willow = Some(willow);

        Ok(p2p)
    }

    /// Get Willow adapter if enabled.
    pub fn willow(&self) -> Option<Arc<WillowAdapter>> {
        self.willow.as_ref().map(Arc::clone)
    }

    /// Start P2P services.
    pub async fn start(&self) -> Result<()> {
        info!("Starting VUDO P2P services");

        // Start peer discovery
        self.discovery.start();

        // Start background sync
        let bg_sync = BackgroundSync::new(
            BackgroundSyncConfig::default(),
            Arc::clone(&self.bandwidth),
        );
        bg_sync.start();
        *self.background_sync.write() = Some(bg_sync);

        // Start message handler
        self.start_message_handler();

        // Announce presence
        let node_addr = self.iroh.node_addr().await?;
        self.discovery.announce_presence(node_addr)?;

        info!("VUDO P2P services started");
        Ok(())
    }

    /// Stop P2P services.
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping VUDO P2P services");

        // Stop background sync
        if let Some(bg_sync) = self.background_sync.read().as_ref() {
            bg_sync.stop();
        }

        // Close Iroh endpoint
        self.iroh.close().await?;

        info!("VUDO P2P services stopped");
        Ok(())
    }

    /// Get this node's ID.
    pub fn node_id(&self) -> String {
        self.iroh.node_id().to_string()
    }

    /// Get this node's address.
    pub async fn node_addr(&self) -> Result<NodeAddr> {
        self.iroh.node_addr().await
    }

    /// Discover peers.
    pub async fn discover_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        Ok(self.discovery.get_peers())
    }

    /// Connect to a peer.
    pub async fn connect(&self, node_addr: NodeAddr) -> Result<PeerId> {
        info!("Connecting to peer: {}", node_addr.node_id);

        // Add to discovery
        let peer_id = self.discovery.add_peer(node_addr.clone())?;

        // Connect via Iroh
        self.iroh.connect(node_addr).await?;

        Ok(peer_id)
    }

    /// Disconnect from a peer.
    pub async fn disconnect(&self, peer_id: &PeerId) -> Result<()> {
        info!("Disconnecting from peer: {}", peer_id);

        self.iroh.disconnect(peer_id).await?;
        self.discovery.remove_peer(peer_id);
        self.sync_protocol.clear_peer_state(peer_id);

        Ok(())
    }

    /// Sync a document with a peer.
    pub async fn sync_document(&self, peer_id: &PeerId, namespace: &str, id: &str) -> Result<()> {
        info!("Syncing document {}/{} with peer {}", namespace, id, peer_id);

        // Create sync request
        let request = self
            .sync_protocol
            .create_sync_request(peer_id, namespace, id)?;

        // Send request
        self.iroh.send_message(peer_id, &request).await?;

        Ok(())
    }

    /// Subscribe to document updates.
    pub async fn subscribe_document(&self, namespace: &str, id: &str) -> Result<Subscription> {
        self.gossip.subscribe_document(namespace, id).await
    }

    /// Announce presence with available documents.
    pub async fn announce_presence(&self, documents: Vec<(String, String)>) -> Result<()> {
        let peer_id = self.node_id();
        self.gossip.announce_presence(peer_id, documents).await
    }

    /// Announce document update.
    pub async fn announce_update(&self, namespace: &str, id: &str, version: u64) -> Result<()> {
        let peer_id = self.node_id();
        self.gossip
            .announce_update(peer_id, namespace, id, version)
            .await
    }

    /// Get connected peers.
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.iroh.connected_peers()
    }

    /// Get connection metadata.
    pub fn get_connection_metadata(&self, peer_id: &PeerId) -> Option<ConnectionMetadata> {
        self.iroh.get_metadata(peer_id)
    }

    /// Get bandwidth statistics.
    pub fn bandwidth_stats(&self) -> BandwidthStats {
        self.bandwidth.stats()
    }

    /// Get sync statistics.
    pub fn sync_stats(&self) -> SyncStats {
        self.sync_protocol.get_stats()
    }

    /// Add document to background sync.
    pub fn add_to_background_sync(&self, peer_id: PeerId, namespace: String, doc_id: String) {
        if let Some(bg_sync) = self.background_sync.read().as_ref() {
            bg_sync.add_document(peer_id, namespace, doc_id);
        }
    }

    /// Start message handler.
    fn start_message_handler(&self) {
        let iroh = Arc::clone(&self.iroh);
        let sync_protocol = Arc::clone(&self.sync_protocol);
        let bandwidth = Arc::clone(&self.bandwidth);
        let discovery = Arc::clone(&self.discovery);

        tokio::spawn(async move {
            info!("Starting message handler");

            loop {
                match iroh.recv_message().await {
                    Ok((peer_id, message)) => {
                        debug!("Received message from peer {}", peer_id);

                        // Update peer last seen
                        discovery.update_last_seen(&peer_id);

                        // Handle message
                        if let Err(e) = Self::handle_message(
                            &peer_id,
                            message,
                            &sync_protocol,
                            &iroh,
                            &bandwidth,
                        )
                        .await
                        {
                            warn!("Failed to handle message from peer {}: {}", peer_id, e);
                        }
                    }
                    Err(e) => {
                        warn!("Error receiving message: {}", e);
                        // Don't break on error, keep listening
                    }
                }
            }
        });
    }

    /// Handle an incoming message.
    async fn handle_message(
        peer_id: &PeerId,
        message: SyncMessage,
        sync_protocol: &Arc<SyncProtocol>,
        iroh: &Arc<IrohAdapter>,
        bandwidth: &Arc<BandwidthManager>,
    ) -> Result<()> {
        match message {
            SyncMessage::SyncRequest {
                namespace,
                id,
                last_sync,
            } => {
                let response = sync_protocol
                    .handle_sync_request(peer_id, namespace, id, last_sync)
                    .await?;

                iroh.send_message(peer_id, &response).await?;
            }

            SyncMessage::SyncChanges {
                namespace,
                id,
                changes,
            } => {
                // Record bandwidth
                let total_bytes: usize = changes.iter().map(|c| c.len()).sum();
                bandwidth.record_received(total_bytes);

                sync_protocol
                    .apply_sync_changes(peer_id, namespace, id, changes)
                    .await?;
            }

            SyncMessage::FullDocument {
                namespace,
                id,
                document,
            } => {
                // Record bandwidth
                bandwidth.record_received(document.len());

                sync_protocol
                    .apply_full_document(peer_id, namespace, id, document)
                    .await?;
            }

            SyncMessage::SyncComplete {
                namespace,
                id,
                version,
            } => {
                debug!(
                    "Sync complete for {}/{} at version {}",
                    namespace, id, version
                );
            }

            SyncMessage::FullSync { namespace, id } => {
                let response = sync_protocol
                    .handle_sync_request(peer_id, namespace, id, None)
                    .await?;

                iroh.send_message(peer_id, &response).await?;
            }

            SyncMessage::Heartbeat => {
                debug!("Received heartbeat from peer {}", peer_id);
            }

            SyncMessage::Error { message } => {
                warn!("Received error from peer {}: {}", peer_id, message);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vudo_p2p_creation() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let config = P2PConfig::default();

        let p2p = VudoP2P::new(state_engine, config).await.unwrap();
        assert!(!p2p.node_id().is_empty());
    }

    #[tokio::test]
    async fn test_vudo_p2p_with_willow() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let config = P2PConfig::default();

        let p2p = VudoP2P::with_willow(state_engine, config).await.unwrap();
        assert!(p2p.willow().is_some());
    }

    #[tokio::test]
    async fn test_start_stop() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let config = P2PConfig::default();

        let p2p = VudoP2P::new(state_engine, config).await.unwrap();

        p2p.start().await.unwrap();
        assert_eq!(p2p.connected_peers().len(), 0);

        p2p.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_node_addr() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let config = P2PConfig::default();

        let p2p = VudoP2P::new(state_engine, config).await.unwrap();

        let addr = p2p.node_addr().await.unwrap();
        assert!(!addr.node_id.to_string().is_empty());
    }
}
