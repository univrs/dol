//! Hyphal Swarm Coordinator
//!
//! Agent coordination using hyphal network topology patterns.
//!
//! This module provides:
//! - Agent management with dynamic role assignment
//! - Message routing through hyphal network
//! - Resource gradient integration
//! - Swarm metrics and monitoring

use crate::network::discovery::{GradientManager, ResourceGradient, ResourceType};
use crate::network::growth::{GrowthParams, GrowthSimulator};
use crate::network::topology::NodeId;
use std::collections::HashMap;

/// Agent role in the hyphal swarm.
///
/// Roles are dynamically assigned based on network topology:
/// - Explorer: Active tips, discovering new territory
/// - Transport: Segment nodes, routing messages
/// - Hub: Fusion points with high connectivity (3+ connections)
/// - Dormant: Inactive nodes, awaiting resources or connections
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentRole {
    /// Active exploration tip, discovering new resources.
    Explorer,
    /// Segment node, responsible for message routing.
    Transport,
    /// High-connectivity fusion point, coordinating multiple paths.
    Hub,
    /// Inactive node, awaiting resources or connections.
    Dormant,
}

impl AgentRole {
    /// Check if this role is active (exploring or routing).
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            AgentRole::Explorer | AgentRole::Transport | AgentRole::Hub
        )
    }

    /// Get the priority of this role (higher = more important).
    pub fn priority(&self) -> u8 {
        match self {
            AgentRole::Hub => 3,
            AgentRole::Explorer => 2,
            AgentRole::Transport => 1,
            AgentRole::Dormant => 0,
        }
    }
}

/// An agent in the hyphal swarm.
///
/// Each agent is associated with a network node and has a dynamic role
/// based on its position in the topology.
#[derive(Debug, Clone)]
pub struct HyphalAgent {
    /// Agent identifier (same as node ID).
    pub id: NodeId,
    /// Current role in the swarm.
    pub role: AgentRole,
    /// Current network position.
    pub position: NodeId,
    /// Resources held by this agent.
    pub resources: HashMap<ResourceType, f64>,
    /// Number of messages routed.
    pub messages_routed: u64,
    /// Age in ticks since creation.
    pub age: u64,
}

impl HyphalAgent {
    /// Create a new agent at a position.
    pub fn new(id: NodeId, position: NodeId, role: AgentRole) -> Self {
        Self {
            id,
            role,
            position,
            resources: HashMap::new(),
            messages_routed: 0,
            age: 0,
        }
    }

    /// Create a new explorer agent.
    pub fn explorer(id: NodeId) -> Self {
        Self::new(id, id, AgentRole::Explorer)
    }

    /// Add resources to the agent.
    pub fn add_resource(&mut self, resource_type: ResourceType, amount: f64) {
        *self.resources.entry(resource_type).or_insert(0.0) += amount;
    }

    /// Get amount of a resource.
    pub fn get_resource(&self, resource_type: &ResourceType) -> f64 {
        *self.resources.get(resource_type).unwrap_or(&0.0)
    }

    /// Get total resources.
    pub fn total_resources(&self) -> f64 {
        self.resources.values().sum()
    }

    /// Increment age.
    pub fn tick(&mut self) {
        self.age += 1;
    }
}

/// Message in the swarm network.
#[derive(Debug, Clone)]
pub struct SwarmMessage {
    /// Source node.
    pub from: NodeId,
    /// Destination node.
    pub to: NodeId,
    /// Message payload.
    pub payload: Vec<u8>,
    /// Priority level (0 = normal, higher = urgent).
    pub priority: u8,
    /// Time-to-live (hops remaining).
    pub ttl: u8,
    /// Message ID for tracking.
    pub id: u64,
}

impl SwarmMessage {
    /// Create a new message.
    pub fn new(from: NodeId, to: NodeId, payload: Vec<u8>) -> Self {
        Self {
            from,
            to,
            payload,
            priority: 0,
            ttl: 64,
            id: 0,
        }
    }

    /// Create a high-priority message.
    pub fn urgent(from: NodeId, to: NodeId, payload: Vec<u8>) -> Self {
        Self {
            from,
            to,
            payload,
            priority: 255,
            ttl: 64,
            id: 0,
        }
    }

    /// Check if message has expired.
    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }

    /// Decrement TTL after a hop.
    pub fn hop(&mut self) {
        if self.ttl > 0 {
            self.ttl -= 1;
        }
    }
}

/// Hyphal swarm coordinator.
///
/// Manages agents using hyphal network topology for coordination.
/// Agents are nodes in the growth simulation, and their roles are
/// determined by their position in the network.
pub struct HyphalSwarm {
    /// All agents in the swarm.
    pub agents: HashMap<NodeId, HyphalAgent>,
    /// Network growth simulator.
    pub growth: GrowthSimulator,
    /// Resource gradient manager.
    pub gradients: GradientManager,
    /// Pending message queue.
    pub message_queue: Vec<SwarmMessage>,
    /// Messages that were delivered.
    pub delivered_messages: Vec<SwarmMessage>,
    /// Messages that failed to route.
    pub failed_messages: Vec<SwarmMessage>,
    /// Next message ID.
    next_message_id: u64,
    /// Total ticks elapsed.
    pub ticks: u64,
}

impl Default for HyphalSwarm {
    fn default() -> Self {
        Self::new()
    }
}

impl HyphalSwarm {
    /// Create a new empty swarm.
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            growth: GrowthSimulator::new(GrowthParams::default()),
            gradients: GradientManager::new(),
            message_queue: Vec::new(),
            delivered_messages: Vec::new(),
            failed_messages: Vec::new(),
            next_message_id: 1,
            ticks: 0,
        }
    }

    /// Create a swarm with custom growth parameters.
    pub fn with_params(params: GrowthParams) -> Self {
        Self {
            agents: HashMap::new(),
            growth: GrowthSimulator::new(params),
            gradients: GradientManager::new(),
            message_queue: Vec::new(),
            delivered_messages: Vec::new(),
            failed_messages: Vec::new(),
            next_message_id: 1,
            ticks: 0,
        }
    }

    /// Spawn a new explorer agent at a new network tip.
    pub fn spawn_explorer(&mut self) -> NodeId {
        let tip = self.growth.spawn_tip();
        let agent = HyphalAgent::explorer(tip);
        self.agents.insert(tip, agent);
        tip
    }

    /// Spawn a connected explorer from an existing node.
    pub fn spawn_connected(&mut self, parent: NodeId) -> Option<NodeId> {
        if !self.agents.contains_key(&parent) {
            return None;
        }

        let child = self.growth.spawn_connected(parent);
        let agent = HyphalAgent::explorer(child);
        self.agents.insert(child, agent);
        Some(child)
    }

    /// Run one coordination cycle.
    pub fn tick(&mut self) {
        self.ticks += 1;

        // 1. Update resource gradients from agent discoveries
        self.update_gradients();

        // 2. Grow network based on gradients
        self.grow_network();

        // 3. Update agent roles based on topology
        self.update_roles();

        // 4. Age all agents
        for agent in self.agents.values_mut() {
            agent.tick();
        }

        // 5. Route pending messages
        self.route_messages();
    }

    /// Update gradients based on agent resource holdings.
    fn update_gradients(&mut self) {
        for (node, agent) in &self.agents {
            for (res_type, amount) in &agent.resources {
                let gradient = self.gradients.get_or_create(res_type.clone());
                gradient.set(*node, *amount);
            }
        }
    }

    /// Grow the network based on resource gradients.
    fn grow_network(&mut self) {
        // Get all gradients and grow toward them
        let resource_types: Vec<ResourceType> = self.gradients.gradients.keys().cloned().collect();

        for res_type in resource_types {
            if let Some(gradient) = self.gradients.get(&res_type) {
                self.growth.grow(gradient);
            }
        }

        // If no gradients, still run a growth cycle with empty gradient
        if self.gradients.gradients.is_empty() {
            let empty_gradient = ResourceGradient::new(ResourceType::new("default"));
            self.growth.grow(&empty_gradient);
        }
    }

    /// Update agent roles based on network topology.
    fn update_roles(&mut self) {
        let active_tips = &self.growth.topology.active_tips;
        let edges = &self.growth.topology.edges;

        for (node, agent) in &mut self.agents {
            if active_tips.contains(node) {
                agent.role = AgentRole::Explorer;
            } else {
                // Count connections
                let connections = edges
                    .iter()
                    .filter(|e| e.source == *node || e.target == *node)
                    .count();

                agent.role = match connections {
                    0 => AgentRole::Dormant,
                    1..=2 => AgentRole::Transport,
                    _ => AgentRole::Hub,
                };
            }
        }
    }

    /// Route all pending messages.
    fn route_messages(&mut self) {
        let messages = std::mem::take(&mut self.message_queue);

        for mut msg in messages {
            msg.hop();

            if msg.is_expired() {
                self.failed_messages.push(msg);
                continue;
            }

            // Check if destination exists
            if !self.agents.contains_key(&msg.to) {
                self.message_queue.push(msg); // Retry later
                continue;
            }

            // Try to find a path
            if let Some(path) = self.growth.topology.shortest_path(msg.from, msg.to) {
                // Message routed successfully
                // Update routing stats for intermediate nodes
                for hop_node in &path[1..path.len() - 1] {
                    if let Some(agent) = self.agents.get_mut(hop_node) {
                        agent.messages_routed += 1;
                    }
                }

                self.delivered_messages.push(msg);
            } else {
                // No path found, requeue if TTL remains
                self.message_queue.push(msg);
            }
        }
    }

    /// Send a message between agents.
    pub fn send(&mut self, from: NodeId, to: NodeId, payload: Vec<u8>) -> u64 {
        let id = self.next_message_id;
        self.next_message_id += 1;

        let mut msg = SwarmMessage::new(from, to, payload);
        msg.id = id;

        self.message_queue.push(msg);
        id
    }

    /// Send an urgent message.
    pub fn send_urgent(&mut self, from: NodeId, to: NodeId, payload: Vec<u8>) -> u64 {
        let id = self.next_message_id;
        self.next_message_id += 1;

        let mut msg = SwarmMessage::urgent(from, to, payload);
        msg.id = id;

        self.message_queue.push(msg);
        id
    }

    /// Broadcast a message to all agents.
    pub fn broadcast(&mut self, from: NodeId, payload: Vec<u8>) -> Vec<u64> {
        let targets: Vec<NodeId> = self
            .agents
            .keys()
            .filter(|&&id| id != from)
            .copied()
            .collect();

        targets
            .into_iter()
            .map(|to| self.send(from, to, payload.clone()))
            .collect()
    }

    /// Add a resource to the gradient field.
    pub fn add_resource(&mut self, node: NodeId, resource_type: ResourceType, amount: f64) {
        let gradient = self.gradients.get_or_create(resource_type.clone());
        gradient.set(node, amount);

        // Also add to agent if exists
        if let Some(agent) = self.agents.get_mut(&node) {
            agent.add_resource(resource_type, amount);
        }
    }

    /// Get an agent by ID.
    pub fn get_agent(&self, id: NodeId) -> Option<&HyphalAgent> {
        self.agents.get(&id)
    }

    /// Get a mutable agent by ID.
    pub fn get_agent_mut(&mut self, id: NodeId) -> Option<&mut HyphalAgent> {
        self.agents.get_mut(&id)
    }

    /// Get all agents with a specific role.
    pub fn agents_by_role(&self, role: &AgentRole) -> Vec<&HyphalAgent> {
        self.agents.values().filter(|a| &a.role == role).collect()
    }

    /// Get swarm metrics.
    pub fn metrics(&self) -> SwarmMetrics {
        let topo_metrics = self.growth.topology.metrics();

        let mut role_counts: HashMap<AgentRole, usize> = HashMap::new();
        for agent in self.agents.values() {
            *role_counts.entry(agent.role.clone()).or_insert(0) += 1;
        }

        SwarmMetrics {
            agent_count: self.agents.len(),
            explorer_count: *role_counts.get(&AgentRole::Explorer).unwrap_or(&0),
            transport_count: *role_counts.get(&AgentRole::Transport).unwrap_or(&0),
            hub_count: *role_counts.get(&AgentRole::Hub).unwrap_or(&0),
            dormant_count: *role_counts.get(&AgentRole::Dormant).unwrap_or(&0),
            generation: self.growth.generation,
            node_count: topo_metrics.node_count,
            edge_count: topo_metrics.edge_count,
            pending_messages: self.message_queue.len(),
            delivered_messages: self.delivered_messages.len(),
            failed_messages: self.failed_messages.len(),
            ticks: self.ticks,
        }
    }

    /// Reset the swarm to initial state.
    pub fn reset(&mut self) {
        self.agents.clear();
        self.growth.reset();
        self.gradients = GradientManager::new();
        self.message_queue.clear();
        self.delivered_messages.clear();
        self.failed_messages.clear();
        self.next_message_id = 1;
        self.ticks = 0;
    }
}

/// Swarm metrics for monitoring.
#[derive(Debug, Clone)]
pub struct SwarmMetrics {
    /// Total number of agents.
    pub agent_count: usize,
    /// Number of explorer agents.
    pub explorer_count: usize,
    /// Number of transport agents.
    pub transport_count: usize,
    /// Number of hub agents.
    pub hub_count: usize,
    /// Number of dormant agents.
    pub dormant_count: usize,
    /// Current network generation.
    pub generation: u64,
    /// Total network nodes.
    pub node_count: usize,
    /// Total network edges.
    pub edge_count: usize,
    /// Messages pending in queue.
    pub pending_messages: usize,
    /// Successfully delivered messages.
    pub delivered_messages: usize,
    /// Failed/expired messages.
    pub failed_messages: usize,
    /// Total ticks elapsed.
    pub ticks: u64,
}

impl SwarmMetrics {
    /// Check if the swarm is healthy.
    pub fn is_healthy(&self) -> bool {
        self.explorer_count > 0 || self.hub_count > 0
    }

    /// Get the message delivery rate.
    pub fn delivery_rate(&self) -> f64 {
        let total = self.delivered_messages + self.failed_messages;
        if total == 0 {
            1.0
        } else {
            self.delivered_messages as f64 / total as f64
        }
    }

    /// Get the percentage of active agents.
    pub fn active_percentage(&self) -> f64 {
        if self.agent_count == 0 {
            0.0
        } else {
            let active = self.explorer_count + self.transport_count + self.hub_count;
            active as f64 / self.agent_count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_creation() {
        let mut swarm = HyphalSwarm::new();
        let agent1 = swarm.spawn_explorer();
        let agent2 = swarm.spawn_explorer();

        assert_eq!(swarm.agents.len(), 2);
        assert!(swarm.agents.contains_key(&agent1));
        assert!(swarm.agents.contains_key(&agent2));
    }

    #[test]
    fn test_swarm_tick() {
        let mut swarm = HyphalSwarm::new();
        swarm.spawn_explorer();
        swarm.spawn_explorer();

        // Run a few ticks
        for _ in 0..5 {
            swarm.tick();
        }

        let metrics = swarm.metrics();
        assert_eq!(metrics.ticks, 5);
    }

    #[test]
    fn test_message_sending() {
        let mut swarm = HyphalSwarm::new();
        let agent1 = swarm.spawn_explorer();
        let agent2 = swarm.spawn_explorer();

        // Connect them
        swarm.growth.topology.connect(agent1, agent2, 1.0);

        // Send message
        let msg_id = swarm.send(agent1, agent2, vec![1, 2, 3]);
        assert!(msg_id > 0);

        // Route
        swarm.tick();

        let metrics = swarm.metrics();
        assert!(metrics.delivered_messages > 0 || metrics.pending_messages > 0);
    }

    #[test]
    fn test_agent_roles() {
        let mut swarm = HyphalSwarm::new();
        let agent = swarm.spawn_explorer();

        // Initially should be explorer
        assert_eq!(swarm.get_agent(agent).unwrap().role, AgentRole::Explorer);

        // After tick, role might change based on topology
        swarm.tick();
    }

    #[test]
    fn test_resource_management() {
        let mut swarm = HyphalSwarm::new();
        let agent = swarm.spawn_explorer();

        swarm.add_resource(agent, ResourceType::new("compute"), 10.0);

        let agent_ref = swarm.get_agent(agent).unwrap();
        assert_eq!(agent_ref.get_resource(&ResourceType::new("compute")), 10.0);
    }

    #[test]
    fn test_connected_spawn() {
        let mut swarm = HyphalSwarm::new();
        let parent = swarm.spawn_explorer();
        let child = swarm.spawn_connected(parent);

        assert!(child.is_some());
        assert_eq!(swarm.agents.len(), 2);
        assert_eq!(swarm.growth.topology.edges.len(), 1);
    }

    #[test]
    fn test_broadcast() {
        let mut swarm = HyphalSwarm::new();
        let sender = swarm.spawn_explorer();
        swarm.spawn_explorer();
        swarm.spawn_explorer();

        let msg_ids = swarm.broadcast(sender, vec![42]);
        assert_eq!(msg_ids.len(), 2); // 2 other agents
    }

    #[test]
    fn test_swarm_metrics() {
        let mut swarm = HyphalSwarm::new();
        swarm.spawn_explorer();
        swarm.spawn_explorer();

        let metrics = swarm.metrics();
        assert_eq!(metrics.agent_count, 2);
        assert!(metrics.is_healthy());
    }

    #[test]
    fn test_delivery_rate() {
        let metrics = SwarmMetrics {
            agent_count: 2,
            explorer_count: 2,
            transport_count: 0,
            hub_count: 0,
            dormant_count: 0,
            generation: 0,
            node_count: 2,
            edge_count: 1,
            pending_messages: 0,
            delivered_messages: 8,
            failed_messages: 2,
            ticks: 10,
        };

        assert_eq!(metrics.delivery_rate(), 0.8);
    }

    #[test]
    fn test_reset() {
        let mut swarm = HyphalSwarm::new();
        swarm.spawn_explorer();
        swarm.tick();

        swarm.reset();

        assert_eq!(swarm.agents.len(), 0);
        assert_eq!(swarm.ticks, 0);
    }
}
