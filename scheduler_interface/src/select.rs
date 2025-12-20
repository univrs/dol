//! Selection phase of the scheduling algorithm.
//!
//! This module implements the `scheduling.select` trait from the Metal DOL
//! scheduling ontology. It handles selecting the optimal node from a set of
//! scored candidates and creating resource reservations.
//!
//! # Selection Process
//!
//! The selector receives scored nodes from the scoring phase and applies
//! tiebreaking strategies when multiple nodes have the same score:
//!
//! - **Random**: Randomly select among tied nodes
//! - **LeastLoaded**: Prefer nodes with lower resource utilization
//! - **RoundRobin**: Distribute selections evenly across nodes
//!
//! # Example
//!
//! ```rust
//! use scheduler_interface::select::{Selector, TiebreakerStrategy, SelectionResult};
//! use scheduler_interface::score::NodeScore;
//!
//! // Implement the Selector trait for your scheduler
//! struct MySelector;
//!
//! impl Selector for MySelector {
//!     fn select(
//!         &self,
//!         scored_nodes: &[NodeScore],
//!         strategy: TiebreakerStrategy,
//!     ) -> Result<SelectionResult, scheduler_interface::SchedulerError> {
//!         // Select the highest scoring node
//!         // Apply tiebreaker if needed
//!         // Create reservation
//!         todo!()
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tiebreaking strategy when multiple nodes have the same score.
///
/// When the scoring phase produces multiple nodes with identical scores,
/// the selector uses one of these strategies to make the final choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TiebreakerStrategy {
    /// Randomly select among tied nodes.
    ///
    /// Provides uniform distribution but is non-deterministic.
    /// Useful for load balancing across equivalent nodes.
    Random,

    /// Select the node with the lowest resource utilization.
    ///
    /// Considers current CPU, memory, and storage usage to find
    /// the least loaded node. Helps prevent hotspots.
    LeastLoaded,

    /// Distribute selections evenly using round-robin.
    ///
    /// Maintains a counter to ensure even distribution of containers
    /// across nodes over time. Deterministic and fair.
    RoundRobin,
}

impl Default for TiebreakerStrategy {
    fn default() -> Self {
        TiebreakerStrategy::LeastLoaded
    }
}

/// Resource reservation created when a node is selected.
///
/// Reservations provide optimistic concurrency control, allowing multiple
/// schedulers to work in parallel while preventing double-booking of resources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reservation {
    /// Unique reservation identifier.
    pub id: String,

    /// Node where resources are reserved.
    pub node_id: String,

    /// Timestamp when the reservation expires (Unix timestamp in seconds).
    ///
    /// After this time, the reservation is released automatically.
    /// Typical values: 30-60 seconds for scheduling operations.
    pub expires_at: u64,

    /// Resources reserved on the node.
    ///
    /// Maps resource type (cpu, memory, storage) to reserved quantity.
    /// Example: {"cpu": "2000m", "memory": "4Gi", "storage": "10Gi"}
    pub resources: HashMap<String, String>,
}

impl Reservation {
    /// Creates a new reservation.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique reservation identifier
    /// * `node_id` - Target node identifier
    /// * `expires_at` - Expiration timestamp (Unix seconds)
    /// * `resources` - Resources to reserve
    pub fn new(
        id: String,
        node_id: String,
        expires_at: u64,
        resources: HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            node_id,
            expires_at,
            resources,
        }
    }

    /// Checks if this reservation has expired.
    ///
    /// # Arguments
    ///
    /// * `now` - Current timestamp (Unix seconds)
    pub fn is_expired(&self, now: u64) -> bool {
        now >= self.expires_at
    }
}

/// Result of the node selection process.
///
/// Contains the selected node, its final score, and the resource reservation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionResult {
    /// The selected node identifier.
    pub selected_node: String,

    /// Final score of the selected node.
    ///
    /// This is the score calculated during the scoring phase,
    /// possibly adjusted by the selection algorithm.
    pub score: f64,

    /// Resource reservation on the selected node.
    ///
    /// This reservation must be honored during the binding phase.
    pub reservation: Reservation,
}

impl SelectionResult {
    /// Creates a new selection result.
    ///
    /// # Arguments
    ///
    /// * `selected_node` - Identifier of the selected node
    /// * `score` - Final score
    /// * `reservation` - Resource reservation
    pub fn new(selected_node: String, score: f64, reservation: Reservation) -> Self {
        Self {
            selected_node,
            score,
            reservation,
        }
    }
}

/// Trait for implementing node selection logic.
///
/// The selector is responsible for:
/// 1. Choosing the optimal node from scored candidates
/// 2. Applying tiebreaking strategies when scores are equal
/// 3. Creating resource reservations for the selected node
///
/// Implementations should ensure:
/// - Deterministic behavior for non-Random strategies
/// - Efficient tiebreaking for large node sets
/// - Proper reservation creation with appropriate timeouts
pub trait Selector: Send + Sync {
    /// Selects the optimal node from scored candidates.
    ///
    /// # Arguments
    ///
    /// * `scored_nodes` - Nodes with their calculated scores (sorted by score descending)
    /// * `strategy` - Tiebreaking strategy to use when scores are equal
    ///
    /// # Returns
    ///
    /// A `SelectionResult` containing the chosen node and its reservation,
    /// or a `SchedulerError` if selection fails.
    ///
    /// # Errors
    ///
    /// - `NoViableNodes` - No nodes are available for selection
    /// - `ReservationFailed` - Unable to create resource reservation
    fn select(
        &self,
        scored_nodes: &[super::score::NodeScore],
        strategy: TiebreakerStrategy,
    ) -> Result<SelectionResult, super::SchedulerError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiebreaker_strategy_default() {
        assert_eq!(TiebreakerStrategy::default(), TiebreakerStrategy::LeastLoaded);
    }

    #[test]
    fn test_reservation_creation() {
        let mut resources = HashMap::new();
        resources.insert("cpu".to_string(), "2000m".to_string());
        resources.insert("memory".to_string(), "4Gi".to_string());

        let reservation = Reservation::new(
            "res-123".to_string(),
            "node-01".to_string(),
            1000000,
            resources.clone(),
        );

        assert_eq!(reservation.id, "res-123");
        assert_eq!(reservation.node_id, "node-01");
        assert_eq!(reservation.expires_at, 1000000);
        assert_eq!(reservation.resources.get("cpu").unwrap(), "2000m");
    }

    #[test]
    fn test_reservation_expiration() {
        let reservation = Reservation::new(
            "res-123".to_string(),
            "node-01".to_string(),
            1000,
            HashMap::new(),
        );

        assert!(!reservation.is_expired(999));
        assert!(reservation.is_expired(1000));
        assert!(reservation.is_expired(1001));
    }

    #[test]
    fn test_selection_result_creation() {
        let reservation = Reservation::new(
            "res-123".to_string(),
            "node-01".to_string(),
            1000,
            HashMap::new(),
        );

        let result = SelectionResult::new("node-01".to_string(), 95.5, reservation.clone());

        assert_eq!(result.selected_node, "node-01");
        assert_eq!(result.score, 95.5);
        assert_eq!(result.reservation.id, "res-123");
    }
}
