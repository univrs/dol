use anyhow::Result;
use iroh::net::{endpoint::Connection, Endpoint, NodeAddr, NodeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

use crate::metrics::ConnectionMetrics;

const ALPN: &[u8] = b"iroh-sync-poc/1";

/// IrohNode manages P2P connections using Iroh
pub struct IrohNode {
    node_name: String,
    endpoint: Endpoint,
    connections: Arc<Mutex<Vec<Connection>>>,
    connection_count: Arc<AtomicU64>,
    sync_tx: mpsc::Sender<Vec<u8>>,
    sync_rx: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,
    metrics: Arc<Mutex<ConnectionMetrics>>,
}

impl IrohNode {
    /// Create a new Iroh node
    pub async fn new(node_name: String, port: Option<u16>, relay: bool) -> Result<Self> {
        info!("[{}] Creating Iroh endpoint...", node_name);

        // Create endpoint
        let endpoint = Endpoint::builder()
            .bind()
            .await?;

        info!("[{}] Endpoint created", node_name);

        // Note: Port binding is handled automatically by Iroh in this version
        let _ = port; // Suppress unused variable warning

        // Enable relay if requested
        if relay {
            info!("[{}] Relay mode enabled", node_name);
            // Note: Iroh 0.28 handles relay configuration automatically
        }

        // Create sync message channels
        let (sync_tx, sync_rx) = mpsc::channel::<Vec<u8>>(100);

        let node = Self {
            node_name,
            endpoint,
            connections: Arc::new(Mutex::new(Vec::new())),
            connection_count: Arc::new(AtomicU64::new(0)),
            sync_tx,
            sync_rx: Arc::new(Mutex::new(sync_rx)),
            metrics: Arc::new(Mutex::new(ConnectionMetrics::new())),
        };

        // Start connection listener
        node.start_listener();

        Ok(node)
    }

    /// Get this node's ID
    pub fn node_id(&self) -> String {
        self.endpoint.node_id().to_string()
    }

    /// Get node address for sharing
    pub async fn node_addr(&self) -> NodeAddr {
        self.endpoint.node_addr().await.unwrap()
    }

    /// Connect to a peer
    pub async fn connect_to_peer(&self, peer_str: &str) -> Result<()> {
        let start = std::time::Instant::now();
        info!("[{}] Connecting to peer: {}", self.node_name, peer_str);

        // Parse peer NodeId
        let peer_id: NodeId = peer_str.parse()?;

        // Create NodeAddr (in real scenario, would include relay info)
        let node_addr = NodeAddr::new(peer_id);

        // Connect
        let conn = self.endpoint.connect(node_addr, ALPN).await?;

        let duration = start.elapsed();
        info!(
            "[{}] Connected to {} in {:?}",
            self.node_name,
            peer_id,
            duration
        );

        // Track connection
        self.connections.lock().await.push(conn.clone());
        self.connection_count.fetch_add(1, Ordering::SeqCst);
        self.metrics.lock().await.record_connection_time(duration);

        // Start receiving from this connection
        self.start_receiver(conn);

        Ok(())
    }

    /// Start listening for incoming connections
    fn start_listener(&self) {
        let node_name = self.node_name.clone();
        let endpoint = self.endpoint.clone();
        let connections = self.connections.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            info!("[{}] Listening for incoming connections...", node_name);

            loop {
                match endpoint.accept().await {
                    Some(incoming) => {
                        let node_name = node_name.clone();
                        let connections = connections.clone();
                        let metrics = metrics.clone();

                        tokio::spawn(async move {
                            match incoming.await {
                                Ok(conn) => {
                                    info!("[{}] Accepted incoming connection", node_name);

                                    connections.lock().await.push(conn.clone());
                                    metrics.lock().await.record_connection_time(
                                        std::time::Duration::from_millis(0),
                                    );

                                    // Receiver will be started by the connection handler
                                }
                                Err(e) => {
                                    warn!("[{}] Failed to accept connection: {}", node_name, e);
                                }
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

    /// Start receiving messages from a connection
    fn start_receiver(&self, conn: Connection) {
        let node_name = self.node_name.clone();
        let sync_tx = self.sync_tx.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            info!("[{}] Starting receiver for connection", node_name);

            loop {
                match conn.accept_uni().await {
                    Ok(mut recv) => {
                        let start = std::time::Instant::now();

                        match recv.read_to_end(1024 * 1024).await {
                            Ok(data) => {
                                let duration = start.elapsed();
                                info!(
                                    "[{}] Received {} bytes (latency: {:?})",
                                    node_name,
                                    data.len(),
                                    duration
                                );

                                metrics.lock().await.record_sync_latency(duration);
                                metrics.lock().await.record_bytes_received(data.len());

                                if let Err(e) = sync_tx.send(data).await {
                                    warn!("[{}] Failed to forward sync message: {}", node_name, e);
                                }
                            }
                            Err(e) => {
                                warn!("[{}] Failed to read data: {}", node_name, e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("[{}] Connection closed: {}", node_name, e);
                        break;
                    }
                }
            }
        });
    }

    /// Broadcast a sync message to all connected peers
    pub async fn broadcast_sync_message(&self, data: &[u8]) -> Result<()> {
        let connections = self.connections.lock().await;

        for (idx, conn) in connections.iter().enumerate() {
            match conn.open_uni().await {
                Ok(mut send) => {
                    if let Err(e) = send.write_all(data).await {
                        warn!(
                            "[{}] Failed to send to peer {}: {}",
                            self.node_name, idx, e
                        );
                    } else {
                        send.finish()?;
                        self.metrics.lock().await.record_bytes_sent(data.len());
                        info!(
                            "[{}] Sent {} bytes to peer {}",
                            self.node_name,
                            data.len(),
                            idx
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "[{}] Failed to open stream to peer {}: {}",
                        self.node_name, idx, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Receive a sync message (non-blocking)
    pub async fn receive_sync_message(&self) -> Result<Vec<u8>> {
        match self.sync_rx.lock().await.try_recv() {
            Ok(data) => Ok(data),
            Err(mpsc::error::TryRecvError::Empty) => {
                anyhow::bail!("No messages available")
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                anyhow::bail!("Sync channel disconnected")
            }
        }
    }

    /// Get connection metrics
    pub async fn get_metrics(&self) -> ConnectionMetrics {
        self.metrics.lock().await.clone()
    }

    /// Get number of active connections
    pub async fn connection_count(&self) -> usize {
        self.connections.lock().await.len()
    }
}
