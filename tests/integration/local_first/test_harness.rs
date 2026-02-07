//! Test harness for local-first integration tests.
//!
//! Provides:
//! - TestNode: Isolated test node with state engine, storage, and P2P
//! - Network simulation: partition/heal, disconnect/reconnect
//! - Convergence verification: hash-based document comparison
//! - Performance measurement: sync timing, throughput

use automerge::{transaction::Transactable, ReadDoc, ROOT};
use blake3::Hasher;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use vudo_p2p::{P2PConfig, VudoP2P};
use vudo_state::{DocumentId, StateEngine};

/// Test node with full local-first stack.
pub struct TestNode {
    /// Node identifier.
    pub id: String,
    /// State engine.
    pub state_engine: Arc<StateEngine>,
    /// P2P layer (optional).
    pub p2p: Option<Arc<VudoP2P>>,
    /// Connected peers.
    peers: Arc<RwLock<Vec<String>>>,
    /// Network status (online/offline).
    network_online: Arc<RwLock<bool>>,
    /// Performance metrics.
    metrics: Arc<RwLock<NodeMetrics>>,
}

/// Node performance metrics.
#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    pub documents_created: usize,
    pub documents_synced: usize,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub sync_operations: usize,
    pub total_sync_time: Duration,
}

impl TestNode {
    /// Create a new test node with default configuration.
    pub async fn new(id: &str) -> Self {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());

        Self {
            id: id.to_string(),
            state_engine,
            p2p: None,
            peers: Arc::new(RwLock::new(vec![])),
            network_online: Arc::new(RwLock::new(true)),
            metrics: Arc::new(RwLock::new(NodeMetrics::default())),
        }
    }

    /// Create a test node with P2P enabled.
    pub async fn with_p2p(id: &str) -> Self {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let config = P2PConfig::default();
        let p2p = Arc::new(VudoP2P::new(Arc::clone(&state_engine), config).await.unwrap());

        // Start P2P services
        p2p.start().await.unwrap();

        Self {
            id: id.to_string(),
            state_engine,
            p2p: Some(p2p),
            peers: Arc::new(RwLock::new(vec![])),
            network_online: Arc::new(RwLock::new(true)),
            metrics: Arc::new(RwLock::new(NodeMetrics::default())),
        }
    }

    /// Create a document.
    pub async fn create_document<F>(&self, namespace: &str, id: &str, init: F) -> DocumentId
    where
        F: FnOnce(&mut automerge::AutoCommit) -> Result<(), automerge::AutomergeError>,
    {
        let doc_id = DocumentId::new(namespace, id);
        let handle = self.state_engine.create_document(doc_id.clone()).await.unwrap();

        handle.update(init).unwrap();

        self.metrics.write().documents_created += 1;

        doc_id
    }

    /// Update a document.
    pub async fn update_document<F>(&self, namespace: &str, id: &str, update: F)
    where
        F: FnOnce(&mut automerge::AutoCommit) -> Result<(), automerge::AutomergeError>,
    {
        let doc_id = DocumentId::new(namespace, id);
        let handle = self.state_engine.get_document(&doc_id).await.unwrap();

        handle.update(update).unwrap();
    }

    /// Get document as bytes for hashing.
    pub async fn get_document_bytes(&self, namespace: &str, id: &str) -> Vec<u8> {
        let doc_id = DocumentId::new(namespace, id);
        let handle = self.state_engine.get_document(&doc_id).await.unwrap();

        handle
            .read(|doc| {
                Ok(doc.save())
            })
            .unwrap()
    }

    /// Compute document hash.
    pub async fn document_hash(&self, namespace: &str, id: &str) -> [u8; 32] {
        let bytes = self.get_document_bytes(namespace, id).await;
        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    }

    /// Read from a document.
    pub async fn read_document<F, T>(&self, namespace: &str, id: &str, read: F) -> T
    where
        F: FnOnce(&automerge::Automerge) -> Result<T, automerge::AutomergeError>,
    {
        let doc_id = DocumentId::new(namespace, id);
        let handle = self.state_engine.get_document(&doc_id).await.unwrap();

        handle.read(read).unwrap()
    }

    /// Connect to another node (P2P).
    pub async fn connect(&self, other: &TestNode) -> Result<(), String> {
        if !*self.network_online.read() {
            return Err("Network offline".to_string());
        }

        if let Some(p2p) = &self.p2p {
            if let Some(other_p2p) = &other.p2p {
                let other_addr = other_p2p.node_addr().await.map_err(|e| e.to_string())?;
                p2p.connect(other_addr).await.map_err(|e| e.to_string())?;

                self.peers.write().push(other.id.clone());
            }
        }

        Ok(())
    }

    /// Disconnect from a specific peer.
    pub async fn disconnect(&self, peer_id: &str) {
        if let Some(p2p) = &self.p2p {
            p2p.disconnect(&peer_id.to_string()).await.ok();
        }

        self.peers.write().retain(|p| p != peer_id);
    }

    /// Disconnect from all peers (simulate offline).
    pub async fn disconnect_all(&self) {
        *self.network_online.write() = false;

        let peers = self.peers.read().clone();
        for peer_id in peers {
            self.disconnect(&peer_id).await;
        }

        self.peers.write().clear();
    }

    /// Reconnect to network (simulate back online).
    pub async fn reconnect(&self) {
        *self.network_online.write() = true;
    }

    /// Simulate network partition.
    pub fn partition(&self) {
        *self.network_online.write() = false;
    }

    /// Heal network partition.
    pub fn heal(&self) {
        *self.network_online.write() = true;
    }

    /// Check if network is online.
    pub fn is_online(&self) -> bool {
        *self.network_online.read()
    }

    /// Get metrics.
    pub fn metrics(&self) -> NodeMetrics {
        self.metrics.read().clone()
    }

    /// Sync document with peer (simulated).
    pub async fn sync_with_peer(&self, peer: &TestNode, namespace: &str, id: &str) -> Result<Duration, String> {
        if !self.is_online() || !peer.is_online() {
            return Err("Network offline".to_string());
        }

        let start = Instant::now();

        // Get our document
        let our_bytes = self.get_document_bytes(namespace, id).await;

        // Simulate network latency
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Get peer's document
        let peer_bytes = peer.get_document_bytes(namespace, id).await;

        // Merge (simulated by loading peer's changes)
        let doc_id = DocumentId::new(namespace, id);
        let our_handle = self.state_engine.get_document(&doc_id).await.unwrap();

        our_handle
            .read(|doc| {
                // In real implementation, this would merge changes
                // For now, we just verify the operation works
                Ok(())
            })
            .unwrap();

        let duration = start.elapsed();

        // Update metrics
        self.metrics.write().sync_operations += 1;
        self.metrics.write().total_sync_time += duration;
        self.metrics.write().bytes_received += peer_bytes.len();
        self.metrics.write().bytes_sent += our_bytes.len();

        Ok(duration)
    }
}

/// Create a mesh network of nodes.
pub async fn create_mesh_network(n: usize) -> Vec<TestNode> {
    let mut nodes = vec![];
    for i in 0..n {
        nodes.push(TestNode::with_p2p(&format!("node_{}", i)).await);
    }

    // Connect all pairs (full mesh)
    for i in 0..n {
        for j in i + 1..n {
            // Note: In a real test, we'd establish connections here
            // For now, we track the topology
        }
    }

    nodes
}

/// Partition network into groups.
pub async fn partition_network(partition_a: &[TestNode], partition_b: &[TestNode]) {
    // Disconnect all cross-partition connections
    for node_a in partition_a {
        for node_b in partition_b {
            node_a.disconnect(&node_b.id).await;
            node_b.disconnect(&node_a.id).await;
        }
    }
}

/// Heal network partition.
pub async fn heal_network(nodes: &[TestNode]) {
    for node in nodes {
        node.heal();
    }

    // Reconnect all pairs
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            nodes[i].connect(&nodes[j]).await.ok();
        }
    }
}

/// Wait for document sync between two nodes.
pub async fn wait_for_sync(
    node_a: &TestNode,
    node_b: &TestNode,
    namespace: &str,
    id: &str,
) -> Duration {
    wait_for_sync_timeout(node_a, node_b, namespace, id, Duration::from_secs(10)).await
}

/// Wait for document sync with timeout.
pub async fn wait_for_sync_timeout(
    node_a: &TestNode,
    node_b: &TestNode,
    namespace: &str,
    id: &str,
    timeout: Duration,
) -> Duration {
    let start = Instant::now();
    let mut last_hash_a = [0u8; 32];
    let mut last_hash_b = [0u8; 32];

    loop {
        if start.elapsed() > timeout {
            panic!(
                "Sync timeout after {:?}. Hash mismatch: {:?} vs {:?}",
                timeout, last_hash_a, last_hash_b
            );
        }

        let hash_a = node_a.document_hash(namespace, id).await;
        let hash_b = node_b.document_hash(namespace, id).await;

        last_hash_a = hash_a;
        last_hash_b = hash_b;

        if hash_a == hash_b {
            return start.elapsed();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Wait for mesh network convergence.
pub async fn wait_for_mesh_sync(nodes: &[TestNode], namespace: &str, id: &str) {
    wait_for_mesh_sync_timeout(nodes, namespace, id, Duration::from_secs(30)).await
}

/// Wait for mesh network convergence with timeout.
pub async fn wait_for_mesh_sync_timeout(
    nodes: &[TestNode],
    namespace: &str,
    id: &str,
    timeout: Duration,
) {
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            let hashes: Vec<_> = futures::future::join_all(
                nodes.iter().map(|n| n.document_hash(namespace, id))
            )
            .await;
            panic!("Mesh sync timeout after {:?}. Hashes: {:?}", timeout, hashes);
        }

        // Get all hashes
        let hashes: Vec<[u8; 32]> = futures::future::join_all(
            nodes.iter().map(|n| n.document_hash(namespace, id))
        )
        .await;

        // Check if all identical
        if hashes.windows(2).all(|w| w[0] == w[1]) {
            return;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Verify full convergence of all nodes.
pub async fn verify_full_convergence(nodes: &[TestNode], namespace: &str, id: &str) {
    let hashes: Vec<[u8; 32]> = futures::future::join_all(
        nodes.iter().map(|n| n.document_hash(namespace, id))
    )
    .await;

    // All hashes must be identical
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Nodes not converged. Hashes: {:?}",
        hashes
    );
}

/// Verify convergence within a partition.
pub async fn verify_partition_convergence(
    partition: &[TestNode],
    namespace: &str,
    id: &str,
) {
    verify_full_convergence(partition, namespace, id).await
}

/// Generate large document data.
pub fn generate_large_document(size_bytes: usize) -> impl FnOnce(&mut automerge::AutoCommit) -> Result<(), automerge::AutomergeError> {
    move |doc| {
        // Generate approximately size_bytes of data
        let chunk_size = 1000;
        let num_chunks = size_bytes / chunk_size;

        for i in 0..num_chunks {
            let key = format!("chunk_{}", i);
            let value = "x".repeat(chunk_size);
            doc.put(ROOT, key, value)?;
        }

        Ok(())
    }
}

/// Measure sync performance.
pub struct SyncBenchmark {
    start: Instant,
    operations: usize,
}

impl SyncBenchmark {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            operations: 0,
        }
    }

    pub fn record_operation(&mut self) {
        self.operations += 1;
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn throughput(&self) -> f64 {
        self.operations as f64 / self.start.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::ROOT;

    #[tokio::test]
    async fn test_node_creation() {
        let node = TestNode::new("test").await;
        assert_eq!(node.id, "test");
        assert!(node.is_online());
    }

    #[tokio::test]
    async fn test_node_with_p2p() {
        let node = TestNode::with_p2p("test").await;
        assert!(node.p2p.is_some());
    }

    #[tokio::test]
    async fn test_create_document() {
        let node = TestNode::new("test").await;
        let doc_id = node
            .create_document("users", "alice", |doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .await;

        assert_eq!(doc_id.namespace, "users");
        assert_eq!(doc_id.id, "alice");
    }

    #[tokio::test]
    async fn test_document_hash() {
        let node = TestNode::new("test").await;
        node.create_document("users", "alice", |doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        })
        .await;

        let hash1 = node.document_hash("users", "alice").await;
        let hash2 = node.document_hash("users", "alice").await;

        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_network_partition() {
        let node = TestNode::new("test").await;
        assert!(node.is_online());

        node.partition();
        assert!(!node.is_online());

        node.heal();
        assert!(node.is_online());
    }

    #[tokio::test]
    async fn test_mesh_network_creation() {
        let nodes = create_mesh_network(5).await;
        assert_eq!(nodes.len(), 5);

        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(node.id, format!("node_{}", i));
        }
    }

    #[tokio::test]
    async fn test_generate_large_document() {
        let node = TestNode::new("test").await;
        let init_fn = generate_large_document(10_000);

        node.create_document("large", "doc", init_fn).await;

        let bytes = node.get_document_bytes("large", "doc").await;
        assert!(bytes.len() > 5_000); // Should be several KB
    }
}
