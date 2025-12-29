//! Adaptive Growth Module
//!
//! Network growth algorithms inspired by hyphal behavior, implementing
//! branching, extension, fusion, and pruning operations.
//!
//! This module provides:
//! - Growth simulation with configurable parameters
//! - Branch/extend/fuse operations
//! - Automatic network optimization

use super::discovery::ResourceGradient;
use super::topology::{Edge, NodeId, Topology};
use std::collections::HashMap;

/// Growth behavior parameters.
///
/// Configures how the hyphal network grows and optimizes.
#[derive(Debug, Clone)]
pub struct GrowthParams {
    /// Minimum potential required for branching.
    pub branching_threshold: f64,
    /// Minimum capacity for edges to survive pruning.
    pub pruning_threshold: f64,
    /// Maximum distance for tip fusion.
    pub fusion_distance: f64,
    /// Base growth rate per cycle.
    pub base_growth_rate: f64,
    /// Maximum branches per branching event.
    pub max_branches: usize,
    /// Capacity boost for fused edges.
    pub fusion_capacity_boost: f64,
}

impl Default for GrowthParams {
    fn default() -> Self {
        Self {
            branching_threshold: 0.5,
            pruning_threshold: 0.1,
            fusion_distance: 0.5,
            base_growth_rate: 0.1,
            max_branches: 2,
            fusion_capacity_boost: 2.0,
        }
    }
}

impl GrowthParams {
    /// Create parameters for aggressive exploration.
    pub fn exploration() -> Self {
        Self {
            branching_threshold: 0.3,
            pruning_threshold: 0.05,
            max_branches: 3,
            ..Default::default()
        }
    }

    /// Create parameters for efficient transport.
    pub fn transport() -> Self {
        Self {
            branching_threshold: 0.7,
            pruning_threshold: 0.3,
            max_branches: 2,
            ..Default::default()
        }
    }

    /// Create parameters for balanced growth.
    pub fn balanced() -> Self {
        Self::default()
    }
}

/// Network growth simulator.
///
/// Simulates hyphal network growth over discrete time steps,
/// handling branching, extension, fusion, and pruning.
pub struct GrowthSimulator {
    /// Current network topology.
    pub topology: Topology,
    /// Growth parameters.
    pub params: GrowthParams,
    /// Growth potential at each node.
    pub node_potentials: HashMap<NodeId, f64>,
    /// Counter for generating new node IDs.
    pub next_node_id: u64,
    /// Current generation (growth cycle count).
    pub generation: u64,
    /// Statistics about growth.
    pub stats: GrowthStats,
}

/// Statistics about network growth.
#[derive(Debug, Clone, Default)]
pub struct GrowthStats {
    /// Total branching events.
    pub total_branches: usize,
    /// Total extension events.
    pub total_extensions: usize,
    /// Total fusion events.
    pub total_fusions: usize,
    /// Total pruned edges.
    pub total_pruned: usize,
}

impl GrowthSimulator {
    /// Create a new growth simulator with given parameters.
    pub fn new(params: GrowthParams) -> Self {
        Self {
            topology: Topology::new(),
            params,
            node_potentials: HashMap::new(),
            next_node_id: 1,
            generation: 0,
            stats: GrowthStats::default(),
        }
    }

    /// Create with default parameters.
    pub fn with_defaults() -> Self {
        Self::new(GrowthParams::default())
    }

    /// Spawn a new exploration tip at the origin.
    ///
    /// Returns the node ID of the new tip.
    pub fn spawn_tip(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        self.topology.add_tip(id);
        self.node_potentials.insert(id, 1.0);
        id
    }

    /// Spawn a connected tip from an existing node.
    pub fn spawn_connected(&mut self, parent: NodeId) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        self.topology.add_tip(id);
        self.topology.connect(parent, id, 1.0);
        self.node_potentials.insert(id, 0.5);
        id
    }

    /// Execute one growth cycle.
    ///
    /// Updates the network based on resource gradients.
    pub fn grow(&mut self, gradient: &ResourceGradient) {
        self.generation += 1;

        let tips: Vec<NodeId> = self.topology.active_tips.iter().copied().collect();

        for tip in tips {
            let potential = *self.node_potentials.get(&tip).unwrap_or(&0.0);
            let resource = gradient.get(tip);

            // Absorb resources to increase potential
            let new_potential = potential + resource * self.params.base_growth_rate;
            self.node_potentials.insert(tip, new_potential);

            if new_potential >= self.params.branching_threshold {
                // Branch into multiple tips
                self.branch(tip);
            } else if resource > 0.1 {
                // Extend toward gradient
                self.extend(tip, gradient);
            }
            // Low resource: tip remains dormant
        }

        // Check for anastomosis (fusion)
        self.check_fusion();

        // Prune low-capacity edges
        self.prune();
    }

    /// Branch a tip into multiple new tips.
    fn branch(&mut self, tip: NodeId) {
        let potential = self.node_potentials.get(&tip).copied().unwrap_or(0.0);

        // Create branch tips
        let num_branches = self.params.max_branches.min(2);
        let potential_per_branch = potential / num_branches as f64;

        for _ in 0..num_branches {
            let new_tip = NodeId(self.next_node_id);
            self.next_node_id += 1;

            self.topology.nodes.insert(new_tip);
            self.topology.active_tips.insert(new_tip);

            self.topology.edges.push(Edge {
                source: tip,
                target: new_tip,
                capacity: 1.0,
                latency: 0.1,
            });

            self.node_potentials.insert(new_tip, potential_per_branch);
        }

        // Original tip becomes inactive
        self.topology.active_tips.remove(&tip);
        self.node_potentials.insert(tip, 0.0);

        self.stats.total_branches += 1;
    }

    /// Extend a tip toward the gradient.
    fn extend(&mut self, tip: NodeId, _gradient: &ResourceGradient) {
        let new_tip = NodeId(self.next_node_id);
        self.next_node_id += 1;

        self.topology.nodes.insert(new_tip);
        self.topology.active_tips.insert(new_tip);
        self.topology.active_tips.remove(&tip);

        self.topology.edges.push(Edge {
            source: tip,
            target: new_tip,
            capacity: 1.0,
            latency: 0.1,
        });

        let potential = self.node_potentials.get(&tip).copied().unwrap_or(0.0);
        self.node_potentials.insert(new_tip, potential);
        self.node_potentials.insert(tip, 0.0);

        self.stats.total_extensions += 1;
    }

    /// Check for and perform tip fusions (anastomosis).
    fn check_fusion(&mut self) {
        let tips: Vec<NodeId> = self.topology.active_tips.iter().copied().collect();

        // In a real implementation, we would use spatial indexing
        // For now, just check all pairs
        let mut fused = Vec::new();

        for i in 0..tips.len() {
            for j in (i + 1)..tips.len() {
                let a = tips[i];
                let b = tips[j];

                // Check if already fused in this cycle
                if fused.contains(&a) || fused.contains(&b) {
                    continue;
                }

                // For demo: fuse if both have high potential
                let pot_a = self.node_potentials.get(&a).unwrap_or(&0.0);
                let pot_b = self.node_potentials.get(&b).unwrap_or(&0.0);

                if *pot_a > 0.3 && *pot_b > 0.3 {
                    // Perform fusion
                    self.topology.edges.push(Edge {
                        source: a,
                        target: b,
                        capacity: self.params.fusion_capacity_boost,
                        latency: 0.05,
                    });

                    // Combine potentials
                    let combined = pot_a + pot_b;
                    self.node_potentials.insert(a, combined);

                    // Deactivate one tip
                    self.topology.active_tips.remove(&b);
                    fused.push(b);

                    self.stats.total_fusions += 1;
                }
            }
        }
    }

    /// Prune low-capacity edges.
    fn prune(&mut self) {
        let before = self.topology.edges.len();
        self.topology.prune(self.params.pruning_threshold);
        let after = self.topology.edges.len();
        self.stats.total_pruned += before - after;
    }

    /// Get current generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Get growth statistics.
    pub fn stats(&self) -> &GrowthStats {
        &self.stats
    }

    /// Get total potential across all nodes.
    pub fn total_potential(&self) -> f64 {
        self.node_potentials.values().sum()
    }

    /// Reset the simulator to initial state.
    pub fn reset(&mut self) {
        self.topology = Topology::new();
        self.node_potentials.clear();
        self.next_node_id = 1;
        self.generation = 0;
        self.stats = GrowthStats::default();
    }
}

/// Growth event for logging/debugging.
#[derive(Debug, Clone)]
pub enum GrowthEvent {
    /// A tip branched into multiple tips.
    Branch {
        /// The parent node that branched
        parent: NodeId,
        /// The child nodes created by branching
        children: Vec<NodeId>,
    },
    /// A tip extended forward.
    Extension {
        /// Source node of the extension
        from: NodeId,
        /// Destination node of the extension
        to: NodeId,
    },
    /// Two tips fused.
    Fusion {
        /// First tip in the fusion
        tip_a: NodeId,
        /// Second tip in the fusion
        tip_b: NodeId,
    },
    /// An edge was pruned.
    Prune {
        /// The edge that was pruned (source, target)
        edge: (NodeId, NodeId),
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::discovery::ResourceType;

    #[test]
    fn test_growth_cycle() {
        let mut sim = GrowthSimulator::with_defaults();
        let root = sim.spawn_tip();

        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(root, 1.0);

        // Run growth cycles
        for _ in 0..10 {
            sim.grow(&gradient);
        }

        // Should have grown
        assert!(sim.topology.nodes.len() > 1);
        assert!(sim.generation() == 10);
    }

    #[test]
    fn test_branching() {
        let mut sim = GrowthSimulator::new(GrowthParams {
            branching_threshold: 0.3,
            ..Default::default()
        });

        let root = sim.spawn_tip();
        sim.node_potentials.insert(root, 1.0);

        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(root, 1.0);

        sim.grow(&gradient);

        // Should have branched
        assert!(sim.stats.total_branches >= 1);
    }

    #[test]
    fn test_connected_spawn() {
        let mut sim = GrowthSimulator::with_defaults();
        let root = sim.spawn_tip();
        let child = sim.spawn_connected(root);

        assert_eq!(sim.topology.nodes.len(), 2);
        assert_eq!(sim.topology.edges.len(), 1);
        assert!(sim.topology.active_tips.contains(&child));
    }

    #[test]
    fn test_growth_params_presets() {
        let exploration = GrowthParams::exploration();
        let transport = GrowthParams::transport();
        let balanced = GrowthParams::balanced();

        assert!(exploration.branching_threshold < balanced.branching_threshold);
        assert!(transport.pruning_threshold > balanced.pruning_threshold);
    }

    #[test]
    fn test_reset() {
        let mut sim = GrowthSimulator::with_defaults();
        sim.spawn_tip();
        sim.generation = 100;
        sim.stats.total_branches = 50;

        sim.reset();

        assert_eq!(sim.topology.nodes.len(), 0);
        assert_eq!(sim.generation, 0);
        assert_eq!(sim.stats.total_branches, 0);
    }
}
