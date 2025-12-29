//! Network Module - Hyphal-Inspired Distributed Coordination
//!
//! This module implements network topology, resource discovery, and adaptive
//! growth algorithms inspired by fungal mycelium networks.
//!
//! ## Overview
//!
//! Fungal mycelium networks exhibit remarkable properties:
//! - Adaptive growth toward resources (chemotropism)
//! - Efficient branching and exploration
//! - Self-healing through anastomosis (fusion)
//! - Distributed nutrient transport
//!
//! These patterns apply to distributed agent systems.
//!
//! ## Submodules
//!
//! - [`topology`]: Graph topology algorithms for network representation
//! - [`discovery`]: Resource gradient detection and navigation
//! - [`growth`]: Adaptive network growth simulation

pub mod discovery;
pub mod growth;
pub mod topology;

// Re-exports for convenience
pub use discovery::{GradientManager, ResourceExplorer, ResourceGradient, ResourceType};
pub use growth::{GrowthEvent, GrowthParams, GrowthSimulator, GrowthStats};
pub use topology::{Edge, NodeId, Topology, TopologyMetrics};
