//! Network Topology Module
//!
//! Hyphal-inspired network topology algorithms for distributed coordination.
//!
//! This module provides graph-based network representation with support for:
//! - Node management (exploration tips, transport nodes, hubs)
//! - Edge connections with capacity and latency metrics
//! - Anastomosis (fusion) operations for network consolidation
//! - Dijkstra's shortest path algorithm for message routing
//! - Network topology metrics

use std::collections::{HashMap, HashSet};

/// Node identifier in the hyphal network.
///
/// Each node represents a point in the distributed network, which can be
/// an active exploration tip, a transport segment, or a fusion hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new NodeId from a u64 value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the underlying u64 value.
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Edge between nodes in the hyphal network.
///
/// Represents a hyphal segment connecting two nodes with defined
/// transport capacity and communication latency.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source node of the edge
    pub source: NodeId,
    /// Target node of the edge
    pub target: NodeId,
    /// Transport capacity (bandwidth) of the edge
    pub capacity: f64,
    /// Communication latency of the edge
    pub latency: f64,
}

impl Edge {
    /// Create a new edge with the given parameters.
    pub fn new(source: NodeId, target: NodeId, capacity: f64, latency: f64) -> Self {
        Self {
            source,
            target,
            capacity,
            latency,
        }
    }

    /// Create an edge with default latency calculated from capacity.
    pub fn with_capacity(source: NodeId, target: NodeId, capacity: f64) -> Self {
        Self {
            source,
            target,
            capacity,
            latency: 1.0 / capacity,
        }
    }
}

/// Network topology representing a hyphal graph structure.
///
/// The topology maintains:
/// - A set of all nodes in the network
/// - A list of directed edges connecting nodes
/// - A set of active exploration tips
#[derive(Debug, Clone)]
pub struct Topology {
    /// All nodes in the network
    pub nodes: HashSet<NodeId>,
    /// All edges connecting nodes
    pub edges: Vec<Edge>,
    /// Active exploration tips (growing nodes)
    pub active_tips: HashSet<NodeId>,
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

impl Topology {
    /// Create a new empty topology.
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: vec![],
            active_tips: HashSet::new(),
        }
    }

    /// Add a node as an exploration tip.
    ///
    /// The node is added to both the node set and the active tips set.
    pub fn add_tip(&mut self, id: NodeId) {
        self.nodes.insert(id);
        self.active_tips.insert(id);
    }

    /// Add a node without making it an active tip.
    pub fn add_node(&mut self, id: NodeId) {
        self.nodes.insert(id);
    }

    /// Remove a node from active tips (make it dormant).
    pub fn deactivate_tip(&mut self, id: NodeId) {
        self.active_tips.remove(&id);
    }

    /// Connect two nodes with an edge.
    ///
    /// Both nodes are added to the network if not already present.
    pub fn connect(&mut self, source: NodeId, target: NodeId, capacity: f64) {
        self.nodes.insert(source);
        self.nodes.insert(target);
        self.edges.push(Edge {
            source,
            target,
            capacity,
            latency: 1.0 / capacity,
        });
    }

    /// Connect two nodes with a bidirectional edge.
    pub fn connect_bidirectional(&mut self, a: NodeId, b: NodeId, capacity: f64) {
        self.connect(a, b, capacity);
        self.connect(b, a, capacity);
    }

    /// Get neighbors of a node (nodes reachable via outgoing edges).
    pub fn neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(|e| e.source == node)
            .map(|e| e.target)
            .collect()
    }

    /// Get all edges from a node.
    pub fn edges_from(&self, node: NodeId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.source == node).collect()
    }

    /// Get all edges to a node.
    pub fn edges_to(&self, node: NodeId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.target == node).collect()
    }

    /// Perform anastomosis (fuse nearby tips).
    ///
    /// When two exploration tips merge, they create a stronger connection
    /// and one becomes inactive. Returns the surviving node if fusion succeeds.
    pub fn fuse_tips(&mut self, a: NodeId, b: NodeId) -> Option<NodeId> {
        if !self.active_tips.contains(&a) || !self.active_tips.contains(&b) {
            return None;
        }

        // Create fusion edge with higher capacity
        self.edges.push(Edge {
            source: a,
            target: b,
            capacity: 2.0, // Fused edges have higher capacity
            latency: 0.5,
        });

        // Create reverse edge for bidirectional connectivity
        self.edges.push(Edge {
            source: b,
            target: a,
            capacity: 2.0,
            latency: 0.5,
        });

        // One tip becomes inactive (absorbed into the network)
        self.active_tips.remove(&b);

        Some(a)
    }

    /// Branch a tip into two new tips.
    ///
    /// Creates two new exploration tips from an existing tip,
    /// connecting them to the parent node.
    pub fn branch_tip(
        &mut self,
        tip: NodeId,
        left_id: NodeId,
        right_id: NodeId,
        capacity: f64,
    ) -> (NodeId, NodeId) {
        // Add new nodes as tips
        self.nodes.insert(left_id);
        self.nodes.insert(right_id);
        self.active_tips.insert(left_id);
        self.active_tips.insert(right_id);

        // Deactivate the parent tip
        self.active_tips.remove(&tip);

        // Connect parent to children
        self.edges.push(Edge {
            source: tip,
            target: left_id,
            capacity,
            latency: 1.0 / capacity,
        });
        self.edges.push(Edge {
            source: tip,
            target: right_id,
            capacity,
            latency: 1.0 / capacity,
        });

        (left_id, right_id)
    }

    /// Find shortest path using Dijkstra's algorithm.
    ///
    /// Uses edge latency as the distance metric.
    /// Returns None if no path exists.
    pub fn shortest_path(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
        if !self.nodes.contains(&from) || !self.nodes.contains(&to) {
            return None;
        }

        if from == to {
            return Some(vec![from]);
        }

        let mut distances: HashMap<NodeId, f64> = HashMap::new();
        let mut previous: HashMap<NodeId, NodeId> = HashMap::new();
        let mut unvisited: HashSet<NodeId> = self.nodes.clone();

        distances.insert(from, 0.0);

        while !unvisited.is_empty() {
            // Find minimum distance node among unvisited
            let current = unvisited
                .iter()
                .filter(|n| distances.contains_key(n))
                .min_by(|a, b| {
                    let da = distances.get(a).unwrap_or(&f64::INFINITY);
                    let db = distances.get(b).unwrap_or(&f64::INFINITY);
                    da.partial_cmp(db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied();

            let current = match current {
                Some(c) => c,
                None => break, // No reachable nodes remaining
            };

            if current == to {
                // Reconstruct path
                let mut path = vec![to];
                let mut curr = to;
                while let Some(&prev) = previous.get(&curr) {
                    path.push(prev);
                    curr = prev;
                }
                path.reverse();
                return Some(path);
            }

            unvisited.remove(&current);
            let current_dist = *distances.get(&current).unwrap_or(&f64::INFINITY);

            // Update neighbors
            for edge in &self.edges {
                if edge.source == current && unvisited.contains(&edge.target) {
                    let alt = current_dist + edge.latency;
                    if alt < *distances.get(&edge.target).unwrap_or(&f64::INFINITY) {
                        distances.insert(edge.target, alt);
                        previous.insert(edge.target, current);
                    }
                }
            }
        }

        None
    }

    /// Check if there is any path between two nodes.
    pub fn is_connected(&self, from: NodeId, to: NodeId) -> bool {
        self.shortest_path(from, to).is_some()
    }

    /// Check if the entire network is connected (all nodes reachable from any node).
    pub fn is_fully_connected(&self) -> bool {
        if self.nodes.is_empty() {
            return true;
        }

        let start = *self.nodes.iter().next().unwrap();
        self.nodes.iter().all(|&n| self.is_connected(start, n))
    }

    /// Prune edges below a capacity threshold.
    pub fn prune(&mut self, threshold: f64) {
        self.edges.retain(|e| e.capacity >= threshold);
    }

    /// Calculate network metrics.
    pub fn metrics(&self) -> TopologyMetrics {
        let node_count = self.nodes.len();
        let edge_count = self.edges.len();
        let active_tip_count = self.active_tips.len();

        let total_capacity: f64 = self.edges.iter().map(|e| e.capacity).sum();
        let avg_capacity = if edge_count > 0 {
            total_capacity / edge_count as f64
        } else {
            0.0
        };

        let total_latency: f64 = self.edges.iter().map(|e| e.latency).sum();
        let avg_latency = if edge_count > 0 {
            total_latency / edge_count as f64
        } else {
            0.0
        };

        // Calculate density (ratio of edges to possible edges)
        let density = if node_count > 1 {
            edge_count as f64 / (node_count * (node_count - 1)) as f64
        } else {
            0.0
        };

        TopologyMetrics {
            node_count,
            edge_count,
            active_tip_count,
            avg_capacity,
            avg_latency,
            density,
        }
    }
}

/// Network topology metrics.
#[derive(Debug, Clone)]
pub struct TopologyMetrics {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Number of active exploration tips
    pub active_tip_count: usize,
    /// Average edge capacity
    pub avg_capacity: f64,
    /// Average edge latency
    pub avg_latency: f64,
    /// Graph density (edges / possible edges)
    pub density: f64,
}

impl TopologyMetrics {
    /// Check if the network has exploration capacity.
    pub fn has_explorers(&self) -> bool {
        self.active_tip_count > 0
    }

    /// Check if the network is sparse.
    pub fn is_sparse(&self) -> bool {
        self.density < 0.3
    }

    /// Check if the network is dense.
    pub fn is_dense(&self) -> bool {
        self.density > 0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_topology() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.connect(NodeId(1), NodeId(2), 1.0);

        assert_eq!(topo.nodes.len(), 2);
        assert_eq!(topo.edges.len(), 1);
        assert_eq!(topo.active_tips.len(), 2);
    }

    #[test]
    fn test_shortest_path_simple() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.add_tip(NodeId(3));
        topo.connect(NodeId(1), NodeId(2), 1.0);
        topo.connect(NodeId(2), NodeId(3), 1.0);

        let path = topo.shortest_path(NodeId(1), NodeId(3));
        assert_eq!(path, Some(vec![NodeId(1), NodeId(2), NodeId(3)]));
    }

    #[test]
    fn test_shortest_path_same_node() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));

        let path = topo.shortest_path(NodeId(1), NodeId(1));
        assert_eq!(path, Some(vec![NodeId(1)]));
    }

    #[test]
    fn test_shortest_path_no_path() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        // No edge connecting them

        let path = topo.shortest_path(NodeId(1), NodeId(2));
        assert_eq!(path, None);
    }

    #[test]
    fn test_shortest_path_prefers_lower_latency() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.add_tip(NodeId(3));

        // Direct path with high latency
        topo.edges.push(Edge {
            source: NodeId(1),
            target: NodeId(3),
            capacity: 0.1,
            latency: 10.0,
        });

        // Indirect path with lower total latency
        topo.connect(NodeId(1), NodeId(2), 1.0); // latency = 1.0
        topo.connect(NodeId(2), NodeId(3), 1.0); // latency = 1.0

        let path = topo.shortest_path(NodeId(1), NodeId(3));
        // Should prefer the indirect path (total latency 2.0 < 10.0)
        assert_eq!(path, Some(vec![NodeId(1), NodeId(2), NodeId(3)]));
    }

    #[test]
    fn test_anastomosis() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));

        let fused = topo.fuse_tips(NodeId(1), NodeId(2));
        assert!(fused.is_some());
        assert_eq!(fused, Some(NodeId(1)));
        assert_eq!(topo.active_tips.len(), 1);
        assert!(topo.active_tips.contains(&NodeId(1)));
        assert!(!topo.active_tips.contains(&NodeId(2)));
        assert_eq!(topo.edges.len(), 2); // Bidirectional edges
    }

    #[test]
    fn test_anastomosis_requires_active_tips() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_node(NodeId(2)); // Not an active tip

        let fused = topo.fuse_tips(NodeId(1), NodeId(2));
        assert!(fused.is_none());
    }

    #[test]
    fn test_branch_tip() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));

        let (left, right) = topo.branch_tip(NodeId(1), NodeId(2), NodeId(3), 1.0);

        assert_eq!(left, NodeId(2));
        assert_eq!(right, NodeId(3));
        assert_eq!(topo.nodes.len(), 3);
        assert_eq!(topo.edges.len(), 2);
        assert!(!topo.active_tips.contains(&NodeId(1)));
        assert!(topo.active_tips.contains(&NodeId(2)));
        assert!(topo.active_tips.contains(&NodeId(3)));
    }

    #[test]
    fn test_neighbors() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.add_tip(NodeId(3));
        topo.connect(NodeId(1), NodeId(2), 1.0);
        topo.connect(NodeId(1), NodeId(3), 1.0);

        let neighbors = topo.neighbors(NodeId(1));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&NodeId(2)));
        assert!(neighbors.contains(&NodeId(3)));
    }

    #[test]
    fn test_prune() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.add_tip(NodeId(3));
        topo.connect(NodeId(1), NodeId(2), 0.5);
        topo.connect(NodeId(2), NodeId(3), 1.5);

        topo.prune(1.0);

        assert_eq!(topo.edges.len(), 1);
        assert_eq!(topo.edges[0].source, NodeId(2));
        assert_eq!(topo.edges[0].target, NodeId(3));
    }

    #[test]
    fn test_metrics() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.add_tip(NodeId(3));
        topo.connect(NodeId(1), NodeId(2), 1.0);
        topo.connect(NodeId(2), NodeId(3), 2.0);

        let metrics = topo.metrics();
        assert_eq!(metrics.node_count, 3);
        assert_eq!(metrics.edge_count, 2);
        assert_eq!(metrics.active_tip_count, 3);
        assert!((metrics.avg_capacity - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_bidirectional_connection() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.connect_bidirectional(NodeId(1), NodeId(2), 1.0);

        assert_eq!(topo.edges.len(), 2);

        // Can traverse both directions
        let path1 = topo.shortest_path(NodeId(1), NodeId(2));
        let path2 = topo.shortest_path(NodeId(2), NodeId(1));
        assert!(path1.is_some());
        assert!(path2.is_some());
    }
}
