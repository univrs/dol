//! # Scheduler Interface
//!
//! This crate provides a Rust implementation of the DOL scheduling ontology,
//! defining the core abstractions and interfaces for building pluggable
//! Kubernetes schedulers.
//!
//! ## Ontology Structure
//!
//! The scheduler interface is organized according to the DOL ontology:
//!
//! - **scheduling.resources** (gene): Defines the resource model including
//!   Pods, Nodes, and their properties
//! - **scheduling.filter** (trait): Filtering logic to determine which nodes
//!   can host a pod
//! - **scheduling.score** (trait): Scoring logic to rank suitable nodes
//! - **scheduling.select** (trait): Selection logic to choose the best node
//!   from scored candidates
//! - **scheduling.bind** (trait): Binding logic to assign a pod to a node
//! - **scheduling.scheduler** (system): The complete scheduler system that
//!   orchestrates the scheduling pipeline
//!
//! ## Usage
//!
//! ```rust,ignore
//! use scheduler_interface::{
//!     resources::{Pod, Node},
//!     filter::Filter,
//!     score::Score,
//!     scheduler::Scheduler,
//! };
//!
//! // Create a custom scheduler with specific filter and score implementations
//! let scheduler = Scheduler::builder()
//!     .filter(MyCustomFilter)
//!     .score(MyCustomScorer)
//!     .build();
//!
//! // Schedule a pod
//! let pod = Pod::new("my-pod");
//! let nodes = vec![Node::new("node-1"), Node::new("node-2")];
//! let result = scheduler.schedule(&pod, &nodes)?;
//! ```
//!
//! ## Design Philosophy
//!
//! This crate follows the ontology-first development approach defined in DOL:
//!
//! 1. **Declarative Definitions**: Each module corresponds to a DOL declaration
//!    (gene, trait, or system)
//! 2. **Type Safety**: Rust's type system enforces the contracts defined in
//!    the ontology
//! 3. **Composability**: Traits can be composed to build complex scheduling
//!    behaviors
//! 4. **Testability**: Each component can be tested independently
//!
//! ## Feature Flags
//!
//! - `kubernetes` (default): Enables Kubernetes-specific implementations
//! - `metrics`: Enables Prometheus metrics collection
//! - `tracing`: Enables distributed tracing support

// Re-export error types at the crate root for convenience
pub use error::{SchedulerError, SchedulerResult};

/// Resource definitions for scheduling (scheduling.resources gene).
///
/// This module defines the core resource types used in scheduling:
/// - `Pod`: Workload to be scheduled
/// - `Node`: Compute resource that can host pods
/// - `ResourceRequirements`: CPU, memory, and other resource specifications
pub mod resources;

/// Filter trait and implementations (scheduling.filter trait).
///
/// Filters determine which nodes are eligible to host a specific pod.
/// Common filters include:
/// - Node affinity/anti-affinity
/// - Resource availability
/// - Taints and tolerations
/// - Topology constraints
pub mod filter;

/// Score trait and implementations (scheduling.score trait).
///
/// Scorers assign numeric scores to filtered nodes, ranking them by
/// suitability for hosting a pod. Higher scores indicate better matches.
/// Common scoring strategies include:
/// - Resource utilization balancing
/// - Pod affinity/anti-affinity
/// - Image locality
/// - Custom business logic
pub mod score;

// /// Select trait and implementations (scheduling.select trait).
// ///
// /// Selectors choose the final node from the set of scored candidates.
// /// This is typically the highest-scoring node, but can implement more
// /// sophisticated selection logic like randomization or fairness guarantees.
// pub mod select;

// /// Bind trait and implementations (scheduling.bind trait).
// ///
// /// Binders perform the actual assignment of a pod to a node, including:
// /// - Creating the binding in the Kubernetes API
// /// - Updating pod status
// /// - Recording scheduling events
// /// - Handling binding failures and retries
// pub mod bind;

// /// Scheduler system (scheduling.scheduler system).
// ///
// /// The scheduler orchestrates the complete scheduling pipeline:
// /// 1. Filter nodes using configured filters
// /// 2. Score filtered nodes using configured scorers
// /// 3. Select the best node using the selector
// /// 4. Bind the pod to the selected node
// ///
// /// This module provides the main `Scheduler` type and builder pattern
// /// for composing custom schedulers from pluggable components.
// pub mod scheduler;

/// Error types and result aliases.
///
/// Defines the error hierarchy for scheduling operations:
/// - `FilterError`: Errors during node filtering
/// - `ScoreError`: Errors during node scoring
/// - `SelectError`: Errors during node selection
/// - `BindError`: Errors during pod binding
/// - `SchedulerError`: High-level scheduling errors
pub mod error;

// Re-export commonly used types for convenience
pub use filter::{
    Affinity, Filter, FilterPredicate, FilterReason, FilterResult, NodeSelector, Toleration,
};
pub use resources::{AllocatableResources, ContainerResources, NodeResources, QoSClass};
pub use score::{NodeScore, Scorer, ScoringFunction, ScoringWeights};
// pub use select::Select;
// pub use bind::Bind;
// pub use scheduler::Scheduler;

/// Library version information.
///
/// This should match the version in Cargo.toml and follows semver.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_valid() {
        // Ensure version string is non-empty and follows semver pattern
        assert!(!VERSION.is_empty());
        assert!(VERSION.split('.').count() >= 2);
    }
}
