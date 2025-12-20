//! Main scheduler orchestration.
//!
//! This module implements the `scheduling.scheduler` system from the Metal DOL
//! scheduling ontology. It composes the filter, score, select, and bind phases
//! into a complete scheduling pipeline.
//!
//! # Scheduling Pipeline
//!
//! The scheduler executes the following phases in order:
//!
//! 1. **Filter**: Remove unsuitable nodes based on constraints
//! 2. **Score**: Calculate scores for remaining nodes
//! 3. **Select**: Choose optimal node and create reservation
//! 4. **Bind**: Commit container to selected node
//!
//! # Example
//!
//! ```rust
//! use scheduler_interface::scheduler::{Scheduler, SchedulerConfig, ScheduleRequest};
//! use scheduler_interface::filter::Filter;
//! use scheduler_interface::score::Scorer;
//! use scheduler_interface::select::Selector;
//! use scheduler_interface::bind::Binder;
//!
//! // Create scheduler with all phases
//! let config = SchedulerConfig::default();
//! // let scheduler = Scheduler::new(config, filter, scorer, selector, binder);
//!
//! // Schedule a container
//! // let result = scheduler.schedule(&nodes, request)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::bind::{BindRequest, BindResult, Binder, BindingMode};
use crate::filter::{Filter, FilterRequest};
use crate::score::{NodeScore, Scorer};
use crate::select::{SelectionResult, Selector, TiebreakerStrategy};
use crate::{Node, SchedulerError};

/// Configuration for the scheduler.
///
/// Controls scoring weights, default strategies, and binding behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Weights for scoring calculation.
    ///
    /// Maps scoring dimension to its weight (0.0 - 1.0).
    /// Example: {"resource_balance": 0.4, "affinity": 0.3, "spread": 0.3}
    pub scoring_weights: HashMap<String, f64>,

    /// Default tiebreaker strategy when scores are equal.
    pub default_tiebreaker: TiebreakerStrategy,

    /// Default binding mode for resource commitment.
    pub binding_mode: BindingMode,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        let mut scoring_weights = HashMap::new();
        scoring_weights.insert("resource_balance".to_string(), 0.4);
        scoring_weights.insert("affinity".to_string(), 0.3);
        scoring_weights.insert("spread".to_string(), 0.3);

        Self {
            scoring_weights,
            default_tiebreaker: TiebreakerStrategy::default(),
            binding_mode: BindingMode::default(),
        }
    }
}

/// Request to schedule a container.
///
/// Contains the pod/container specification and any scheduling constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleRequest {
    /// Pod/container specification.
    pub pod: PodSpec,

    /// Scheduling constraints to apply during filtering.
    pub constraints: Vec<Constraint>,
}

impl ScheduleRequest {
    /// Creates a new schedule request.
    ///
    /// # Arguments
    ///
    /// * `pod` - Pod specification
    /// * `constraints` - Scheduling constraints
    pub fn new(pod: PodSpec, constraints: Vec<Constraint>) -> Self {
        Self { pod, constraints }
    }
}

/// Pod/container specification.
///
/// Defines resource requirements and scheduling preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PodSpec {
    /// Pod identifier.
    pub id: String,

    /// Pod name.
    pub name: String,

    /// Namespace for the pod.
    pub namespace: String,

    /// Resource requirements.
    ///
    /// Maps resource type to required quantity.
    /// Example: {"cpu": "2000m", "memory": "4Gi"}
    pub resources: HashMap<String, String>,

    /// Node affinity labels.
    ///
    /// Preferences for node selection based on labels.
    pub affinity: HashMap<String, String>,

    /// Anti-affinity rules.
    ///
    /// Avoid co-locating with pods matching these labels.
    pub anti_affinity: HashMap<String, String>,
}

/// Scheduling constraint applied during filtering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    /// Constraint type (e.g., "node_selector", "taint_toleration").
    pub constraint_type: String,

    /// Constraint parameters.
    pub parameters: HashMap<String, String>,
}

/// Result of a scheduling operation.
///
/// Contains the selected node, reservation details, and binding result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleResult {
    /// Selected node identifier.
    pub node_id: String,

    /// Reservation ID from selection phase.
    pub reservation_id: String,

    /// Result of the binding operation.
    pub bind_result: BindResult,
}

impl ScheduleResult {
    /// Creates a new schedule result.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Selected node
    /// * `reservation_id` - Reservation identifier
    /// * `bind_result` - Binding operation result
    pub fn new(node_id: String, reservation_id: String, bind_result: BindResult) -> Self {
        Self {
            node_id,
            reservation_id,
            bind_result,
        }
    }

    /// Returns true if scheduling succeeded.
    pub fn is_success(&self) -> bool {
        self.bind_result.success
    }
}

/// Main scheduler implementation.
///
/// Orchestrates the complete scheduling pipeline by composing
/// the filter, score, select, and bind phases.
pub struct Scheduler<F, S, L, B>
where
    F: Filter,
    S: Scorer,
    L: Selector,
    B: Binder,
{
    /// Scheduler configuration.
    config: SchedulerConfig,

    /// Filter phase implementation.
    filter: F,

    /// Score phase implementation.
    scorer: S,

    /// Select phase implementation.
    selector: L,

    /// Bind phase implementation.
    binder: B,
}

impl<F, S, L, B> Scheduler<F, S, L, B>
where
    F: Filter,
    S: Scorer,
    L: Selector,
    B: Binder,
{
    /// Creates a new scheduler.
    ///
    /// # Arguments
    ///
    /// * `config` - Scheduler configuration
    /// * `filter` - Filter phase implementation
    /// * `scorer` - Score phase implementation
    /// * `selector` - Select phase implementation
    /// * `binder` - Bind phase implementation
    pub fn new(config: SchedulerConfig, filter: F, scorer: S, selector: L, binder: B) -> Self {
        Self {
            config,
            filter,
            scorer,
            selector,
            binder,
        }
    }

    /// Schedules a pod/container to a node.
    ///
    /// Executes the complete scheduling pipeline:
    /// 1. Filter nodes based on constraints
    /// 2. Score remaining nodes
    /// 3. Select optimal node
    /// 4. Bind container to node
    ///
    /// # Arguments
    ///
    /// * `nodes` - Available nodes to schedule on
    /// * `request` - Scheduling request with pod spec and constraints
    ///
    /// # Returns
    ///
    /// A `ScheduleResult` with the selected node and binding details,
    /// or a `SchedulerError` if scheduling fails.
    ///
    /// # Errors
    ///
    /// - `NoViableNodes` - No nodes pass filtering
    /// - `ScoringFailed` - Error during scoring phase
    /// - `SelectionFailed` - Error during selection phase
    /// - `BindingFailed` - Error during binding phase
    pub fn schedule(
        &self,
        nodes: &[Node],
        request: ScheduleRequest,
    ) -> Result<ScheduleResult, SchedulerError> {
        // Phase 1: Filter
        let filter_request = FilterRequest {
            pod_resources: request.pod.resources.clone(),
            constraints: request.constraints.clone(),
            affinity: request.pod.affinity.clone(),
            anti_affinity: request.pod.anti_affinity.clone(),
        };

        let filtered_nodes = self.filter.filter(nodes, &filter_request)?;

        if filtered_nodes.is_empty() {
            return Err(SchedulerError::NoViableNodes {
                reason: "All nodes filtered out".to_string(),
            });
        }

        // Phase 2: Score
        let mut scored_nodes = self.scorer.score(&filtered_nodes, &request.pod)?;

        if scored_nodes.is_empty() {
            return Err(SchedulerError::ScoringFailed {
                reason: "No nodes produced valid scores".to_string(),
            });
        }

        // Sort by score descending
        scored_nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Phase 3: Select
        let selection = self
            .selector
            .select(&scored_nodes, self.config.default_tiebreaker)?;

        // Phase 4: Bind
        let bind_request = BindRequest::new(
            request.pod.id.clone(),
            selection.selected_node.clone(),
            selection.reservation.id.clone(),
            request.pod.resources.clone(),
        );

        let bind_result = self.binder.bind(bind_request, self.config.binding_mode)?;

        Ok(ScheduleResult::new(
            selection.selected_node,
            selection.reservation.id,
            bind_result,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();

        assert!(config.scoring_weights.contains_key("resource_balance"));
        assert_eq!(config.default_tiebreaker, TiebreakerStrategy::LeastLoaded);
        assert_eq!(config.binding_mode, BindingMode::Pessimistic);
    }

    #[test]
    fn test_schedule_request_creation() {
        let pod = PodSpec {
            id: "pod-123".to_string(),
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            resources: HashMap::new(),
            affinity: HashMap::new(),
            anti_affinity: HashMap::new(),
        };

        let request = ScheduleRequest::new(pod.clone(), vec![]);

        assert_eq!(request.pod.id, "pod-123");
        assert!(request.constraints.is_empty());
    }

    #[test]
    fn test_schedule_result_success() {
        let bind_result = BindResult::success(1000000, HashMap::new());
        let result = ScheduleResult::new(
            "node-01".to_string(),
            "res-123".to_string(),
            bind_result,
        );

        assert!(result.is_success());
        assert_eq!(result.node_id, "node-01");
        assert_eq!(result.reservation_id, "res-123");
    }

    #[test]
    fn test_schedule_result_failure() {
        let bind_result = BindResult::failure(HashMap::new());
        let result = ScheduleResult::new(
            "node-01".to_string(),
            "res-123".to_string(),
            bind_result,
        );

        assert!(!result.is_success());
    }
}
