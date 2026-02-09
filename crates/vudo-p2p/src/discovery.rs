//! Peer discovery mechanisms (mDNS, DHT, relay).

use crate::error::{P2PError, Result};
use crate::sync_protocol::PeerId;
use iroh::net::{NodeAddr, NodeId};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Discovered peer information.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Peer ID.
    pub peer_id: PeerId,
    /// Node address.
    pub node_addr: NodeAddr,
    /// Discovery method.
    pub discovery_method: DiscoveryMethod,
    /// When this peer was discovered.
    pub discovered_at: Instant,
    /// Last seen timestamp.
    pub last_seen: Instant,
}

/// Discovery method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Discovered via mDNS (local network).
    MDNS,
    /// Discovered via DHT (internet-wide).
    DHT,
    /// Discovered via relay server.
    Relay,
    /// Manually added.
    Manual,
}

/// Peer discovery manager.
pub struct PeerDiscovery {
    /// Discovered peers.
    peers: Arc<RwLock<HashMap<PeerId, DiscoveredPeer>>>,
    /// Peer timeout duration.
    peer_timeout: Duration,
    /// Enable mDNS discovery.
    enable_mdns: bool,
    /// Enable DHT discovery.
    enable_dht: bool,
}

impl PeerDiscovery {
    /// Create a new peer discovery manager.
    pub fn new(enable_mdns: bool, enable_dht: bool) -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_timeout: Duration::from_secs(300), // 5 minutes
            enable_mdns,
            enable_dht,
        }
    }

    /// Start peer discovery.
    pub fn start(&self) {
        if self.enable_mdns {
            info!("mDNS discovery enabled");
            self.start_mdns_discovery();
        }

        if self.enable_dht {
            info!("DHT discovery enabled");
            self.start_dht_discovery();
        }

        // Start cleanup task
        self.start_cleanup_task();
    }

    /// Add a manually discovered peer.
    pub fn add_peer(&self, node_addr: NodeAddr) -> Result<PeerId> {
        let peer_id = node_addr.node_id.to_string();

        info!("Adding manual peer: {}", peer_id);

        let peer = DiscoveredPeer {
            peer_id: peer_id.clone(),
            node_addr,
            discovery_method: DiscoveryMethod::Manual,
            discovered_at: Instant::now(),
            last_seen: Instant::now(),
        };

        self.peers.write().insert(peer_id.clone(), peer);

        Ok(peer_id)
    }

    /// Remove a peer.
    pub fn remove_peer(&self, peer_id: &PeerId) {
        self.peers.write().remove(peer_id);
    }

    /// Get all discovered peers.
    pub fn get_peers(&self) -> Vec<DiscoveredPeer> {
        self.peers.read().values().cloned().collect()
    }

    /// Get a specific peer.
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<DiscoveredPeer> {
        self.peers.read().get(peer_id).cloned()
    }

    /// Update peer's last seen timestamp.
    pub fn update_last_seen(&self, peer_id: &PeerId) {
        if let Some(peer) = self.peers.write().get_mut(peer_id) {
            peer.last_seen = Instant::now();
        }
    }

    /// Get peers discovered via specific method.
    pub fn get_peers_by_method(&self, method: DiscoveryMethod) -> Vec<DiscoveredPeer> {
        self.peers
            .read()
            .values()
            .filter(|p| p.discovery_method == method)
            .cloned()
            .collect()
    }

    /// Get number of discovered peers.
    pub fn peer_count(&self) -> usize {
        self.peers.read().len()
    }

    /// Start mDNS discovery (local network).
    fn start_mdns_discovery(&self) {
        let peers = self.peers.clone();

        tokio::spawn(async move {
            info!("Starting mDNS discovery");

            // In a real implementation, this would use Iroh's mDNS discovery
            // For now, this is a placeholder
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;

                // Simulate mDNS discovery
                debug!("mDNS discovery scan");

                // In production, this would:
                // 1. Listen for mDNS announcements
                // 2. Parse discovered peers
                // 3. Add them to the peers map
            }
        });
    }

    /// Start DHT discovery (internet-wide).
    fn start_dht_discovery(&self) {
        let peers = self.peers.clone();

        tokio::spawn(async move {
            info!("Starting DHT discovery");

            // In a real implementation, this would use Iroh's DHT
            // For now, this is a placeholder
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;

                // Simulate DHT discovery
                debug!("DHT discovery scan");

                // In production, this would:
                // 1. Query DHT for peers
                // 2. Add discovered peers to map
            }
        });
    }

    /// Start cleanup task to remove stale peers.
    fn start_cleanup_task(&self) {
        let peers = self.peers.clone();
        let timeout = self.peer_timeout;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;

                let now = Instant::now();
                let mut peers = peers.write();
                let before_count = peers.len();

                // Remove stale peers
                peers.retain(|_, peer| {
                    let elapsed = now.duration_since(peer.last_seen);
                    elapsed < timeout
                });

                let after_count = peers.len();
                if before_count != after_count {
                    info!(
                        "Cleaned up {} stale peers ({} -> {})",
                        before_count - after_count,
                        before_count,
                        after_count
                    );
                }
            }
        });
    }

    /// Announce this node's presence.
    pub fn announce_presence(&self, node_addr: NodeAddr) -> Result<()> {
        info!("Announcing presence: {}", node_addr.node_id);

        // In production, this would:
        // 1. Broadcast mDNS announcement
        // 2. Publish to DHT
        // 3. Register with relay servers

        Ok(())
    }
}

/// Peer connection prioritization.
pub struct PeerPrioritizer {
    /// Peer scores.
    scores: Arc<RwLock<HashMap<PeerId, f64>>>,
}

impl PeerPrioritizer {
    /// Create a new peer prioritizer.
    pub fn new() -> Self {
        Self {
            scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calculate peer score based on connection quality.
    pub fn calculate_score(&self, peer_id: &PeerId, metadata: &crate::iroh_adapter::ConnectionMetadata) -> f64 {
        let mut score = 100.0;

        // Prefer direct connections
        if metadata.is_direct {
            score += 50.0;
        }

        // Penalize for age (older connections may be stale)
        let age = metadata.established_at.elapsed().as_secs() as f64;
        score -= age / 3600.0; // Reduce score by 1 per hour

        // Reward for successful message exchange
        let message_ratio = if metadata.messages_sent > 0 {
            metadata.messages_received as f64 / metadata.messages_sent as f64
        } else {
            1.0
        };
        score += message_ratio * 10.0;

        // Update stored score
        self.scores.write().insert(peer_id.clone(), score);

        score
    }

    /// Get peer score.
    pub fn get_score(&self, peer_id: &PeerId) -> Option<f64> {
        self.scores.read().get(peer_id).copied()
    }

    /// Get top N peers by score.
    pub fn get_top_peers(&self, n: usize) -> Vec<(PeerId, f64)> {
        let scores = self.scores.read();
        let mut peer_scores: Vec<_> = scores.iter().map(|(k, v)| (k.clone(), *v)).collect();
        peer_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        peer_scores.truncate(n);
        peer_scores
    }
}

impl Default for PeerPrioritizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_discovery_creation() {
        let discovery = PeerDiscovery::new(true, true);
        assert_eq!(discovery.peer_count(), 0);
    }

    // TODO: Update test with correct Iroh 0.28 NodeId API
    // #[test]
    // fn test_add_manual_peer() {
    //     let discovery = PeerDiscovery::new(true, true);
    //     // Create a dummy node address
    //     let node_id = NodeId::from_bytes(&[0u8; 32]).unwrap();
    //     let node_addr = NodeAddr::new(node_id);
    //     let peer_id = discovery.add_peer(node_addr).unwrap();
    //     assert_eq!(discovery.peer_count(), 1);
    //     let peer = discovery.get_peer(&peer_id).unwrap();
    //     assert_eq!(peer.discovery_method, DiscoveryMethod::Manual);
    // }

    // TODO: Update test with correct Iroh 0.28 NodeId API
    // #[test]
    // fn test_remove_peer() {
    //     let discovery = PeerDiscovery::new(true, true);
    //     let node_id = NodeId::from_bytes(&[0u8; 32]).unwrap();
    //     let node_addr = NodeAddr::new(node_id);
    //     let peer_id = discovery.add_peer(node_addr).unwrap();
    //     assert_eq!(discovery.peer_count(), 1);
    //     discovery.remove_peer(&peer_id);
    //     assert_eq!(discovery.peer_count(), 0);
    // }

    #[test]
    fn test_peer_prioritizer() {
        let prioritizer = PeerPrioritizer::new();

        let metadata = crate::iroh_adapter::ConnectionMetadata {
            peer_id: "peer1".to_string(),
            established_at: Instant::now(),
            is_direct: true,
            messages_sent: 10,
            messages_received: 10,
            bytes_sent: 1000,
            bytes_received: 1000,
        };

        let score = prioritizer.calculate_score(&"peer1".to_string(), &metadata);
        assert!(score > 100.0); // Direct connection bonus

        let retrieved_score = prioritizer.get_score(&"peer1".to_string()).unwrap();
        assert_eq!(score, retrieved_score);
    }

    // TODO: Update test with correct Iroh 0.28 NodeId API
    // #[test]
    // fn test_get_peers_by_method() {
    //     let discovery = PeerDiscovery::new(true, true);
    //     let node_id1 = NodeId::from_bytes(&[1u8; 32]).unwrap();
    //     let node_addr1 = NodeAddr::new(node_id1);
    //     discovery.add_peer(node_addr1).unwrap();
    //     let manual_peers = discovery.get_peers_by_method(DiscoveryMethod::Manual);
    //     assert_eq!(manual_peers.len(), 1);
    //     let mdns_peers = discovery.get_peers_by_method(DiscoveryMethod::MDNS);
    //     assert_eq!(mdns_peers.len(), 0);
    // }
}
