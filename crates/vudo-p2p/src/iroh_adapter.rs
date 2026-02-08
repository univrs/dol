//! Iroh node management and connection handling.

use crate::error::{P2PError, Result};
use crate::sync_protocol::{PeerId, SyncMessage};
use iroh::net::endpoint::{Connection, Incoming};
use iroh::net::{Endpoint, NodeAddr, NodeId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// ALPN protocol identifier for VUDO P2P.
const ALPN: &[u8] = b"vudo-p2p/1";

/// P2P network configuration.
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Node name (for logging).
    pub node_name: String,
    /// Enable relay mode.
    pub enable_relay: bool,
    /// Enable mDNS discovery.
    pub enable_mdns: bool,
    /// Enable DHT discovery.
    pub enable_dht: bool,
    /// Connection timeout.
    pub connection_timeout: Duration,
    /// Maximum concurrent connections.
    pub max_connections: usize,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            node_name: "vudo-node".to_string(),
            enable_relay: true,
            enable_mdns: true,
            enable_dht: true,
            connection_timeout: Duration::from_secs(10),
            max_connections: 100,
        }
    }
}

/// Connection metadata.
#[derive(Debug, Clone)]
pub struct ConnectionMetadata {
    /// Peer ID.
    pub peer_id: PeerId,
    /// Connection established timestamp.
    pub established_at: std::time::Instant,
    /// Is this a direct connection (vs relay)?
    pub is_direct: bool,
    /// Number of messages sent.
    pub messages_sent: u64,
    /// Number of messages received.
    pub messages_received: u64,
    /// Bytes sent.
    pub bytes_sent: u64,
    /// Bytes received.
    pub bytes_received: u64,
}

/// Iroh adapter for P2P networking.
pub struct IrohAdapter {
    /// Iroh endpoint.
    endpoint: Endpoint,
    /// Configuration.
    config: P2PConfig,
    /// Active connections.
    connections: Arc<RwLock<HashMap<PeerId, Connection>>>,
    /// Connection metadata.
    metadata: Arc<RwLock<HashMap<PeerId, ConnectionMetadata>>>,
    /// Incoming message channel.
    message_tx: mpsc::UnboundedSender<(PeerId, SyncMessage)>,
    /// Incoming message receiver.
    message_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<(PeerId, SyncMessage)>>>,
}

impl IrohAdapter {
    /// Create a new Iroh adapter.
    pub async fn new(config: P2PConfig) -> Result<Self> {
        info!("[{}] Initializing Iroh endpoint", config.node_name);

        // Create endpoint
        let endpoint = Endpoint::builder()
            .bind()
            .await
            .map_err(|e| P2PError::IrohError(e.into()))?;

        info!(
            "[{}] Endpoint created with node ID: {}",
            config.node_name,
            endpoint.node_id()
        );

        // Create message channel
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let adapter = Self {
            endpoint,
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            message_tx,
            message_rx: Arc::new(tokio::sync::Mutex::new(message_rx)),
        };

        // Start connection listener
        adapter.start_listener();

        Ok(adapter)
    }

    /// Get this node's ID.
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    /// Get this node's address (for sharing with peers).
    pub async fn node_addr(&self) -> Result<NodeAddr> {
        self.endpoint
            .node_addr()
            .await
            .map_err(|e| P2PError::IrohError(e.into()))
    }

    /// Connect to a peer.
    pub async fn connect(&self, node_addr: NodeAddr) -> Result<PeerId> {
        let peer_id = node_addr.node_id;
        let peer_id_str = peer_id.to_string();

        info!(
            "[{}] Connecting to peer: {}",
            self.config.node_name, peer_id_str
        );

        // Check if already connected
        if self.connections.read().contains_key(&peer_id_str) {
            debug!("Already connected to peer {}", peer_id_str);
            return Ok(peer_id_str);
        }

        // Check connection limit
        if self.connections.read().len() >= self.config.max_connections {
            return Err(P2PError::ConnectionFailed(
                "Maximum connections reached".to_string(),
            ));
        }

        // Connect with timeout
        let conn = tokio::time::timeout(
            self.config.connection_timeout,
            self.endpoint.connect(node_addr, ALPN),
        )
        .await
        .map_err(|_| P2PError::Timeout)?
        .map_err(|e| P2PError::ConnectionFailed(e.to_string()))?;

        info!("[{}] Connected to peer {}", self.config.node_name, peer_id_str);

        // Store connection
        self.connections.write().insert(peer_id_str.clone(), conn.clone());

        // Store metadata
        let metadata = ConnectionMetadata {
            peer_id: peer_id_str.clone(),
            established_at: std::time::Instant::now(),
            is_direct: true, // TODO: Detect if using relay
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
        };
        self.metadata.write().insert(peer_id_str.clone(), metadata);

        // Start receiver for this connection
        self.start_receiver(peer_id_str.clone(), conn);

        Ok(peer_id_str)
    }

    /// Disconnect from a peer.
    pub async fn disconnect(&self, peer_id: &PeerId) -> Result<()> {
        info!(
            "[{}] Disconnecting from peer {}",
            self.config.node_name, peer_id
        );

        let conn = self
            .connections
            .write()
            .remove(peer_id)
            .ok_or_else(|| P2PError::PeerNotFound(peer_id.clone()))?;

        self.metadata.write().remove(peer_id);

        // Close connection
        conn.close(0u32.into(), b"disconnect");

        Ok(())
    }

    /// Send a message to a peer.
    pub async fn send_message(&self, peer_id: &PeerId, message: &SyncMessage) -> Result<()> {
        let conn = self
            .connections
            .read()
            .get(peer_id)
            .cloned()
            .ok_or_else(|| P2PError::PeerNotFound(peer_id.clone()))?;

        let bytes = message.to_bytes()?;

        debug!(
            "[{}] Sending {} bytes to peer {}",
            self.config.node_name,
            bytes.len(),
            peer_id
        );

        // Open uni-directional stream
        let mut send = conn
            .open_uni()
            .await
            .map_err(|e| P2PError::ConnectionFailed(e.to_string()))?;

        // Send message
        send.write_all(&bytes)
            .await
            .map_err(|e| P2PError::ConnectionFailed(e.to_string()))?;

        send.finish()
            .map_err(|e| P2PError::ConnectionFailed(e.to_string()))?;

        // Update metadata
        if let Some(metadata) = self.metadata.write().get_mut(peer_id) {
            metadata.messages_sent += 1;
            metadata.bytes_sent += bytes.len() as u64;
        }

        Ok(())
    }

    /// Broadcast a message to all connected peers.
    pub async fn broadcast(&self, message: &SyncMessage) -> Result<()> {
        let peer_ids: Vec<PeerId> = self.connections.read().keys().cloned().collect();

        debug!(
            "[{}] Broadcasting message to {} peers",
            self.config.node_name,
            peer_ids.len()
        );

        for peer_id in peer_ids {
            if let Err(e) = self.send_message(&peer_id, message).await {
                warn!("Failed to send to peer {}: {}", peer_id, e);
            }
        }

        Ok(())
    }

    /// Receive the next message.
    pub async fn recv_message(&self) -> Result<(PeerId, SyncMessage)> {
        self.message_rx
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(|| P2PError::Internal("Message channel closed".to_string()))
    }

    /// Get list of connected peers.
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.connections.read().keys().cloned().collect()
    }

    /// Get connection metadata for a peer.
    pub fn get_metadata(&self, peer_id: &PeerId) -> Option<ConnectionMetadata> {
        self.metadata.read().get(peer_id).cloned()
    }

    /// Get connection count.
    pub fn connection_count(&self) -> usize {
        self.connections.read().len()
    }

    /// Start listening for incoming connections.
    fn start_listener(&self) {
        let endpoint = self.endpoint.clone();
        let node_name = self.config.node_name.clone();
        let connections = self.connections.clone();
        let metadata = self.metadata.clone();
        let message_tx = self.message_tx.clone();
        let max_connections = self.config.max_connections;

        tokio::spawn(async move {
            info!("[{}] Listening for incoming connections", node_name);

            loop {
                match endpoint.accept().await {
                    Some(incoming) => {
                        let node_name = node_name.clone();
                        let connections = connections.clone();
                        let metadata = metadata.clone();
                        let message_tx = message_tx.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_incoming(
                                incoming,
                                &node_name,
                                connections,
                                metadata,
                                message_tx,
                                max_connections,
                            )
                            .await
                            {
                                warn!("[{}] Failed to handle incoming connection: {}", node_name, e);
                            }
                        });
                    }
                    None => {
                        warn!("[{}] Endpoint closed", node_name);
                        break;
                    }
                }
            }
        });
    }

    /// Handle an incoming connection.
    async fn handle_incoming(
        incoming: Incoming,
        node_name: &str,
        connections: Arc<RwLock<HashMap<PeerId, Connection>>>,
        metadata: Arc<RwLock<HashMap<PeerId, ConnectionMetadata>>>,
        message_tx: mpsc::UnboundedSender<(PeerId, SyncMessage)>,
        max_connections: usize,
    ) -> Result<()> {
        let conn = incoming
            .await
            .map_err(|e| P2PError::ConnectionFailed(e.to_string()))?;

        // Get peer ID from connection - in Iroh 0.28 this might be different
        // For now, generate a temporary peer ID based on connection
        let peer_id = format!("peer-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

        info!("[{}] Accepted connection from peer {}", node_name, peer_id);

        // Check connection limit
        if connections.read().len() >= max_connections {
            warn!(
                "[{}] Rejecting connection from {}: max connections reached",
                node_name, peer_id
            );
            conn.close(0u32.into(), b"max connections reached");
            return Ok(());
        }

        // Store connection
        connections.write().insert(peer_id.clone(), conn.clone());

        // Store metadata
        let conn_metadata = ConnectionMetadata {
            peer_id: peer_id.clone(),
            established_at: std::time::Instant::now(),
            is_direct: true,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
        };
        metadata.write().insert(peer_id.clone(), conn_metadata);

        // Start receiver
        Self::spawn_receiver(peer_id, conn, node_name.to_string(), metadata, message_tx);

        Ok(())
    }

    /// Start receiver for a connection.
    fn start_receiver(&self, peer_id: PeerId, conn: Connection) {
        Self::spawn_receiver(
            peer_id,
            conn,
            self.config.node_name.clone(),
            self.metadata.clone(),
            self.message_tx.clone(),
        );
    }

    /// Spawn receiver task.
    fn spawn_receiver(
        peer_id: PeerId,
        conn: Connection,
        node_name: String,
        metadata: Arc<RwLock<HashMap<PeerId, ConnectionMetadata>>>,
        message_tx: mpsc::UnboundedSender<(PeerId, SyncMessage)>,
    ) {
        tokio::spawn(async move {
            debug!("[{}] Starting receiver for peer {}", node_name, peer_id);

            loop {
                match conn.accept_uni().await {
                    Ok(mut recv) => {
                        match recv.read_to_end(10 * 1024 * 1024).await {
                            // 10 MB max
                            Ok(bytes) => {
                                debug!(
                                    "[{}] Received {} bytes from peer {}",
                                    node_name,
                                    bytes.len(),
                                    peer_id
                                );

                                // Update metadata
                                if let Some(meta) = metadata.write().get_mut(&peer_id) {
                                    meta.messages_received += 1;
                                    meta.bytes_received += bytes.len() as u64;
                                }

                                // Deserialize message
                                match SyncMessage::from_bytes(&bytes) {
                                    Ok(message) => {
                                        if message_tx.send((peer_id.clone(), message)).is_err() {
                                            warn!(
                                                "[{}] Failed to forward message from peer {}",
                                                node_name, peer_id
                                            );
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        warn!(
                                            "[{}] Failed to deserialize message from peer {}: {}",
                                            node_name, peer_id, e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(
                                    "[{}] Failed to read from peer {}: {}",
                                    node_name, peer_id, e
                                );
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        debug!("[{}] Connection closed from peer {}: {}", node_name, peer_id, e);
                        break;
                    }
                }
            }

            info!("[{}] Receiver stopped for peer {}", node_name, peer_id);
        });
    }

    /// Close the endpoint.
    pub async fn close(&self) -> Result<()> {
        info!("[{}] Closing endpoint", self.config.node_name);

        // Close all connections
        let peer_ids: Vec<PeerId> = self.connections.read().keys().cloned().collect();
        for peer_id in peer_ids {
            let _ = self.disconnect(&peer_id).await;
        }

        // Close endpoint (non-async in Iroh 0.28)
        // The endpoint will be closed when dropped

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iroh_adapter_creation() {
        let config = P2PConfig::default();
        let adapter = IrohAdapter::new(config).await.unwrap();

        assert_eq!(adapter.connection_count(), 0);
    }

    #[tokio::test]
    async fn test_node_id() {
        let config = P2PConfig::default();
        let adapter = IrohAdapter::new(config).await.unwrap();

        let node_id = adapter.node_id();
        assert!(!node_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_connected_peers_empty() {
        let config = P2PConfig::default();
        let adapter = IrohAdapter::new(config).await.unwrap();

        let peers = adapter.connected_peers();
        assert_eq!(peers.len(), 0);
    }
}
