//! Swarm Module - Agent Coordination Using Hyphal Topology
//!
//! This module implements agent swarm coordination using hyphal network patterns.
//!
//! ## Overview
//!
//! The hyphal swarm coordinator manages agents using network topology concepts:
//! - Explorer agents: Active tips exploring new territory
//! - Transport agents: Segment nodes routing messages
//! - Hub agents: Fusion points with high connectivity
//! - Dormant agents: Inactive nodes awaiting resources
//!
//! ## Submodules
//!
//! - [`hyphal_coordinator`]: Main swarm coordination logic

pub mod hyphal_coordinator;

// Re-exports for convenience
pub use hyphal_coordinator::{AgentRole, HyphalAgent, HyphalSwarm, SwarmMessage, SwarmMetrics};
