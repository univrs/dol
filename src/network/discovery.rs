//! Resource Discovery Module
//!
//! Chemotropic resource discovery using gradients, inspired by how fungal hyphae
//! navigate toward nutrient sources.
//!
//! ## Overview
//!
//! This module provides:
//! - Resource type identification
//! - Gradient field representation for resource distribution
//! - Resource exploration agents that follow gradients
//! - Absorption mechanics for resource collection

use crate::network::topology::{NodeId, Topology};
use std::collections::HashMap;

/// Resource type identifier.
///
/// Represents different types of resources that can be discovered
/// and transported through the hyphal network.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceType(pub String);

impl ResourceType {
    /// Create a new resource type.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the resource type name.
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ResourceType {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ResourceType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Resource gradient field.
///
/// Represents the distribution of a specific resource type across
/// the network nodes. Used for chemotropic navigation.
#[derive(Debug, Clone)]
pub struct ResourceGradient {
    /// The type of resource this gradient represents
    pub resource_type: ResourceType,
    /// Resource concentration at each node
    pub concentrations: HashMap<NodeId, f64>,
}

impl ResourceGradient {
    /// Create a new empty gradient for a resource type.
    pub fn new(resource_type: ResourceType) -> Self {
        Self {
            resource_type,
            concentrations: HashMap::new(),
        }
    }

    /// Set concentration at a node.
    pub fn set(&mut self, node: NodeId, concentration: f64) {
        self.concentrations.insert(node, concentration.max(0.0));
    }

    /// Get concentration at a node (returns 0.0 if not set).
    pub fn get(&self, node: NodeId) -> f64 {
        *self.concentrations.get(&node).unwrap_or(&0.0)
    }

    /// Remove resource from a node.
    pub fn remove(&mut self, node: NodeId) {
        self.concentrations.remove(&node);
    }

    /// Decrease concentration at a node by a specified amount.
    pub fn decrease(&mut self, node: NodeId, amount: f64) {
        let current = self.get(node);
        self.set(node, current - amount);
    }

    /// Increase concentration at a node by a specified amount.
    pub fn increase(&mut self, node: NodeId, amount: f64) {
        let current = self.get(node);
        self.set(node, current + amount);
    }

    /// Find direction toward higher concentration among neighbors.
    ///
    /// Returns the neighbor with the highest concentration that is greater
    /// than the current node's concentration, or None if no such neighbor exists.
    pub fn gradient_direction(&self, from: NodeId, neighbors: &[NodeId]) -> Option<NodeId> {
        let current = self.get(from);

        neighbors
            .iter()
            .filter(|&&n| self.get(n) > current)
            .max_by(|&&a, &&b| {
                self.get(a)
                    .partial_cmp(&self.get(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }

    /// Find the node with the highest concentration among a set of nodes.
    pub fn peak_concentration(&self, nodes: &[NodeId]) -> Option<NodeId> {
        nodes
            .iter()
            .max_by(|&&a, &&b| {
                self.get(a)
                    .partial_cmp(&self.get(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }

    /// Calculate total resource in the gradient.
    pub fn total(&self) -> f64 {
        self.concentrations.values().sum()
    }

    /// Get all nodes with concentration above a threshold.
    pub fn nodes_above_threshold(&self, threshold: f64) -> Vec<NodeId> {
        self.concentrations
            .iter()
            .filter(|(_, &c)| c > threshold)
            .map(|(&n, _)| n)
            .collect()
    }

    /// Diffuse resources between neighboring nodes.
    ///
    /// Simulates resource spreading through the network.
    pub fn diffuse(&mut self, topology: &Topology, rate: f64) {
        let mut changes: HashMap<NodeId, f64> = HashMap::new();

        for &node in self.concentrations.keys() {
            let neighbors = topology.neighbors(node);
            if neighbors.is_empty() {
                continue;
            }

            let current = self.get(node);
            let diffusion = current * rate / neighbors.len() as f64;

            for neighbor in neighbors {
                *changes.entry(neighbor).or_insert(0.0) += diffusion;
                *changes.entry(node).or_insert(0.0) -= diffusion;
            }
        }

        for (node, delta) in changes {
            let current = self.get(node);
            self.set(node, current + delta);
        }
    }
}

/// Resource discovery agent.
///
/// An exploration agent that navigates the network following resource
/// gradients and absorbs resources along the way.
#[derive(Debug)]
pub struct ResourceExplorer {
    /// Current position in the network
    pub position: NodeId,
    /// Resources absorbed by this explorer
    pub absorbed: HashMap<ResourceType, f64>,
    /// History of visited nodes
    pub exploration_history: Vec<NodeId>,
    /// Maximum history length to maintain
    pub max_history: usize,
}

impl ResourceExplorer {
    /// Create a new explorer at a starting position.
    pub fn new(start: NodeId) -> Self {
        Self {
            position: start,
            absorbed: HashMap::new(),
            exploration_history: vec![start],
            max_history: 100,
        }
    }

    /// Create an explorer with custom history limit.
    pub fn with_history_limit(start: NodeId, max_history: usize) -> Self {
        Self {
            position: start,
            absorbed: HashMap::new(),
            exploration_history: vec![start],
            max_history,
        }
    }

    /// Move toward resource gradient.
    ///
    /// Returns true if the explorer moved, false if no better position was found.
    pub fn follow_gradient(&mut self, gradient: &ResourceGradient, topology: &Topology) -> bool {
        let neighbors = topology.neighbors(self.position);

        if let Some(next) = gradient.gradient_direction(self.position, &neighbors) {
            self.move_to(next);
            true
        } else {
            false
        }
    }

    /// Move to a specific node.
    pub fn move_to(&mut self, node: NodeId) {
        self.position = node;
        self.exploration_history.push(node);

        // Trim history if too long
        if self.exploration_history.len() > self.max_history {
            self.exploration_history.remove(0);
        }
    }

    /// Absorb resource at current position.
    ///
    /// Returns the amount actually absorbed.
    pub fn absorb(&mut self, gradient: &mut ResourceGradient, rate: f64) -> f64 {
        let available = gradient.get(self.position);
        let absorbed = available * rate.min(1.0);

        gradient.decrease(self.position, absorbed);

        *self
            .absorbed
            .entry(gradient.resource_type.clone())
            .or_insert(0.0) += absorbed;

        absorbed
    }

    /// Absorb a fixed amount of resource.
    pub fn absorb_amount(&mut self, gradient: &mut ResourceGradient, amount: f64) -> f64 {
        let available = gradient.get(self.position);
        let absorbed = amount.min(available);

        gradient.decrease(self.position, absorbed);

        *self
            .absorbed
            .entry(gradient.resource_type.clone())
            .or_insert(0.0) += absorbed;

        absorbed
    }

    /// Get total absorbed amount of a resource type.
    pub fn total_absorbed(&self, resource_type: &ResourceType) -> f64 {
        *self.absorbed.get(resource_type).unwrap_or(&0.0)
    }

    /// Get total of all absorbed resources.
    pub fn total_all_absorbed(&self) -> f64 {
        self.absorbed.values().sum()
    }

    /// Deposit resources at current position.
    pub fn deposit(&mut self, gradient: &mut ResourceGradient, amount: f64) -> f64 {
        let available = *self.absorbed.get(&gradient.resource_type).unwrap_or(&0.0);
        let deposited = amount.min(available);

        if deposited > 0.0 {
            *self
                .absorbed
                .entry(gradient.resource_type.clone())
                .or_insert(0.0) -= deposited;
            gradient.increase(self.position, deposited);
        }

        deposited
    }

    /// Check if explorer has visited a node.
    pub fn has_visited(&self, node: NodeId) -> bool {
        self.exploration_history.contains(&node)
    }

    /// Get the number of unique nodes visited.
    pub fn unique_visits(&self) -> usize {
        let unique: std::collections::HashSet<_> = self.exploration_history.iter().collect();
        unique.len()
    }

    /// Move to a random unvisited neighbor if available.
    pub fn explore_unvisited(&mut self, topology: &Topology) -> bool {
        let neighbors = topology.neighbors(self.position);
        let unvisited: Vec<_> = neighbors
            .into_iter()
            .filter(|n| !self.has_visited(*n))
            .collect();

        if let Some(&next) = unvisited.first() {
            self.move_to(next);
            true
        } else {
            false
        }
    }
}

/// Multi-resource gradient manager.
///
/// Manages multiple resource gradients simultaneously.
#[derive(Debug, Default)]
pub struct GradientManager {
    /// Map of resource type to gradient
    pub gradients: HashMap<ResourceType, ResourceGradient>,
}

impl GradientManager {
    /// Create a new empty gradient manager.
    pub fn new() -> Self {
        Self {
            gradients: HashMap::new(),
        }
    }

    /// Add or update a gradient.
    pub fn add_gradient(&mut self, gradient: ResourceGradient) {
        self.gradients
            .insert(gradient.resource_type.clone(), gradient);
    }

    /// Get a gradient by resource type.
    pub fn get(&self, resource_type: &ResourceType) -> Option<&ResourceGradient> {
        self.gradients.get(resource_type)
    }

    /// Get a mutable gradient by resource type.
    pub fn get_mut(&mut self, resource_type: &ResourceType) -> Option<&mut ResourceGradient> {
        self.gradients.get_mut(resource_type)
    }

    /// Get or create a gradient for a resource type.
    pub fn get_or_create(&mut self, resource_type: ResourceType) -> &mut ResourceGradient {
        self.gradients
            .entry(resource_type.clone())
            .or_insert_with(|| ResourceGradient::new(resource_type))
    }

    /// Calculate combined gradient direction considering all resources.
    ///
    /// Weights each resource gradient and finds the best overall direction.
    pub fn combined_direction(
        &self,
        from: NodeId,
        neighbors: &[NodeId],
        weights: &HashMap<ResourceType, f64>,
    ) -> Option<NodeId> {
        let mut scores: HashMap<NodeId, f64> = HashMap::new();

        for (res_type, gradient) in &self.gradients {
            let weight = weights.get(res_type).unwrap_or(&1.0);
            let from_concentration = gradient.get(from);

            for &neighbor in neighbors {
                let concentration = gradient.get(neighbor);
                if concentration > from_concentration {
                    *scores.entry(neighbor).or_insert(0.0) +=
                        (concentration - from_concentration) * weight;
                }
            }
        }

        scores
            .into_iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(node, _)| node)
    }

    /// Diffuse all gradients.
    pub fn diffuse_all(&mut self, topology: &Topology, rate: f64) {
        for gradient in self.gradients.values_mut() {
            gradient.diffuse(topology, rate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type() {
        let res = ResourceType::new("nutrient");
        assert_eq!(res.name(), "nutrient");

        let res2: ResourceType = "water".into();
        assert_eq!(res2.name(), "water");
    }

    #[test]
    fn test_gradient_basic() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 0.5);
        gradient.set(NodeId(2), 1.0);

        assert_eq!(gradient.get(NodeId(1)), 0.5);
        assert_eq!(gradient.get(NodeId(2)), 1.0);
        assert_eq!(gradient.get(NodeId(3)), 0.0); // Not set
    }

    #[test]
    fn test_gradient_negative_clamp() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), -0.5);
        assert_eq!(gradient.get(NodeId(1)), 0.0); // Clamped to 0
    }

    #[test]
    fn test_gradient_direction() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 0.1);
        gradient.set(NodeId(2), 0.5);
        gradient.set(NodeId(3), 1.0);

        let neighbors = vec![NodeId(2), NodeId(3)];
        let direction = gradient.gradient_direction(NodeId(1), &neighbors);
        assert_eq!(direction, Some(NodeId(3))); // Highest concentration
    }

    #[test]
    fn test_gradient_direction_none() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 1.0);
        gradient.set(NodeId(2), 0.5);

        let neighbors = vec![NodeId(2)];
        let direction = gradient.gradient_direction(NodeId(1), &neighbors);
        assert_eq!(direction, None); // No neighbor has higher concentration
    }

    #[test]
    fn test_gradient_following() {
        let mut topo = Topology::new();
        topo.add_tip(NodeId(1));
        topo.add_tip(NodeId(2));
        topo.add_tip(NodeId(3));
        topo.connect(NodeId(1), NodeId(2), 1.0);
        topo.connect(NodeId(2), NodeId(3), 1.0);

        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 0.1);
        gradient.set(NodeId(2), 0.5);
        gradient.set(NodeId(3), 1.0);

        let mut explorer = ResourceExplorer::new(NodeId(1));
        assert!(explorer.follow_gradient(&gradient, &topo));
        assert_eq!(explorer.position, NodeId(2));
    }

    #[test]
    fn test_absorption() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 1.0);

        let mut explorer = ResourceExplorer::new(NodeId(1));
        let absorbed = explorer.absorb(&mut gradient, 0.5);

        assert_eq!(absorbed, 0.5);
        assert_eq!(gradient.get(NodeId(1)), 0.5);
        assert_eq!(explorer.total_absorbed(&ResourceType::new("nutrient")), 0.5);
    }

    #[test]
    fn test_gradient_total() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 1.0);
        gradient.set(NodeId(2), 2.0);
        gradient.set(NodeId(3), 3.0);

        assert_eq!(gradient.total(), 6.0);
    }

    #[test]
    fn test_explorer_history() {
        let mut explorer = ResourceExplorer::new(NodeId(1));
        explorer.move_to(NodeId(2));
        explorer.move_to(NodeId(3));

        assert!(explorer.has_visited(NodeId(1)));
        assert!(explorer.has_visited(NodeId(2)));
        assert!(explorer.has_visited(NodeId(3)));
        assert!(!explorer.has_visited(NodeId(4)));
        assert_eq!(explorer.unique_visits(), 3);
    }

    #[test]
    fn test_explorer_deposit() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 1.0);

        let mut explorer = ResourceExplorer::new(NodeId(1));
        explorer.absorb(&mut gradient, 1.0);

        // Move and deposit
        gradient.set(NodeId(2), 0.0);
        explorer.move_to(NodeId(2));
        let deposited = explorer.deposit(&mut gradient, 0.5);

        assert_eq!(deposited, 0.5);
        assert_eq!(gradient.get(NodeId(2)), 0.5);
        assert_eq!(explorer.total_absorbed(&ResourceType::new("nutrient")), 0.5);
    }

    #[test]
    fn test_gradient_manager() {
        let mut manager = GradientManager::new();

        let mut nutrient = ResourceGradient::new(ResourceType::new("nutrient"));
        nutrient.set(NodeId(1), 1.0);
        manager.add_gradient(nutrient);

        let mut water = ResourceGradient::new(ResourceType::new("water"));
        water.set(NodeId(2), 2.0);
        manager.add_gradient(water);

        assert!(manager.get(&ResourceType::new("nutrient")).is_some());
        assert!(manager.get(&ResourceType::new("water")).is_some());
        assert!(manager.get(&ResourceType::new("minerals")).is_none());
    }

    #[test]
    fn test_nodes_above_threshold() {
        let mut gradient = ResourceGradient::new(ResourceType::new("nutrient"));
        gradient.set(NodeId(1), 0.1);
        gradient.set(NodeId(2), 0.5);
        gradient.set(NodeId(3), 1.0);

        let above = gradient.nodes_above_threshold(0.3);
        assert_eq!(above.len(), 2);
        assert!(above.contains(&NodeId(2)));
        assert!(above.contains(&NodeId(3)));
    }
}
