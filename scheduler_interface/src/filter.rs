//! Scheduling filter implementation based on Metal DOL scheduling.filter trait.
//!
//! This module provides filtering predicates for determining which nodes in a cluster
//! are eligible to run a given pod. Filters implement various constraints including
//! node selectors, affinity rules, tolerations, and resource requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a node in the cluster.
pub type NodeId = String;

/// Reasons why a node was filtered out during scheduling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterReason {
    /// Node does not match required label selectors
    LabelMismatch {
        /// Required labels that were not satisfied
        missing_labels: Vec<String>,
    },
    /// Node does not satisfy affinity requirements
    AffinityNotSatisfied {
        /// Description of the affinity rule that failed
        rule: String,
    },
    /// Pod does not have required tolerations for node taints
    TaintNotTolerated {
        /// The taint key that was not tolerated
        taint_key: String,
        /// The taint effect
        taint_effect: String,
    },
    /// Node does not have sufficient resources
    InsufficientResources {
        /// Resource type that is insufficient (cpu, memory, etc.)
        resource_type: String,
        /// Amount requested
        requested: f64,
        /// Amount available
        available: f64,
    },
    /// Custom filter reason
    Custom(String),
}

/// Node selector matches nodes based on label key-value pairs.
///
/// A node must have all the specified labels with matching values to satisfy
/// the node selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSelector {
    /// Map of label keys to required values
    pub labels: HashMap<String, String>,
}

impl NodeSelector {
    /// Creates a new empty node selector.
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Adds a label requirement to the selector.
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
}

impl Default for NodeSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Affinity rules for pod scheduling preferences and requirements.
///
/// Affinity rules can be either required (hard constraints) or preferred
/// (soft constraints with weights).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Affinity {
    /// Hard constraints that must be satisfied during scheduling
    pub required_during_scheduling: Vec<AffinityTerm>,
    /// Soft constraints that are preferred but not required
    pub preferred_during_scheduling: Vec<WeightedAffinityTerm>,
}

impl Affinity {
    /// Creates a new empty affinity specification.
    pub fn new() -> Self {
        Self {
            required_during_scheduling: Vec::new(),
            preferred_during_scheduling: Vec::new(),
        }
    }
}

impl Default for Affinity {
    fn default() -> Self {
        Self::new()
    }
}

/// A single affinity term specifying node selection criteria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffinityTerm {
    /// Label selector for matching nodes
    pub match_expressions: Vec<LabelSelectorRequirement>,
}

/// A weighted affinity term for preferred scheduling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightedAffinityTerm {
    /// Weight of this preference (1-100)
    pub weight: i32,
    /// The affinity term
    pub preference: AffinityTerm,
}

/// Label selector requirement for matching node labels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    /// Label key
    pub key: String,
    /// Operator for matching
    pub operator: LabelOperator,
    /// Values to match against (interpretation depends on operator)
    pub values: Vec<String>,
}

/// Operators for label matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelOperator {
    /// Label value must be in the set of values
    In,
    /// Label value must not be in the set of values
    NotIn,
    /// Label key must exist (values ignored)
    Exists,
    /// Label key must not exist (values ignored)
    DoesNotExist,
}

/// Toleration allows a pod to be scheduled on nodes with matching taints.
///
/// A toleration "tolerates" a taint if the key, operator, value (if applicable),
/// and effect match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Toleration {
    /// Taint key that this toleration applies to
    pub key: String,
    /// How to match the taint
    pub operator: TolerationOperator,
    /// Value to match (only used with Equal operator)
    pub value: Option<String>,
    /// Taint effect to match (empty matches all effects)
    pub effect: Option<TaintEffect>,
}

/// Operators for matching taints with tolerations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TolerationOperator {
    /// Toleration matches if key and value are equal
    Equal,
    /// Toleration matches if key exists (value ignored)
    Exists,
}

/// Effects that a taint can have on pod scheduling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaintEffect {
    /// Pod will not be scheduled on the node unless it tolerates the taint
    NoSchedule,
    /// Scheduler will try to avoid scheduling the pod on the node
    PreferNoSchedule,
    /// Pod will be evicted from the node if already running
    NoExecute,
}

/// Filter predicates that can be applied during scheduling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterPredicate {
    /// Filter based on node label selectors
    NodeSelector(NodeSelector),
    /// Filter based on node affinity rules
    NodeAffinity(Affinity),
    /// Filter based on pod tolerations for node taints
    Toleration(Vec<Toleration>),
    /// Filter based on available node resources
    ResourceFit {
        /// CPU request in cores
        cpu: f64,
        /// Memory request in bytes
        memory: f64,
        /// Custom resource requirements
        custom: HashMap<String, f64>,
    },
}

/// Result of applying filters to a set of nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterResult {
    /// Nodes that passed all filters and are eligible for scheduling
    pub eligible_nodes: Vec<NodeId>,
    /// Nodes that were filtered out with reasons
    pub filtered_out: Vec<(NodeId, FilterReason)>,
}

impl FilterResult {
    /// Creates a new empty filter result.
    pub fn new() -> Self {
        Self {
            eligible_nodes: Vec::new(),
            filtered_out: Vec::new(),
        }
    }

    /// Marks a node as eligible.
    pub fn add_eligible(&mut self, node_id: NodeId) {
        self.eligible_nodes.push(node_id);
    }

    /// Marks a node as filtered out with a reason.
    pub fn add_filtered(&mut self, node_id: NodeId, reason: FilterReason) {
        self.filtered_out.push((node_id, reason));
    }
}

impl Default for FilterResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for filtering nodes based on pod requirements.
///
/// Implementors of this trait evaluate whether nodes in a cluster are suitable
/// for running a given pod, based on various constraints and requirements.
pub trait Filter<Node, Pod> {
    /// Filters the provided nodes and returns which ones are eligible for the pod.
    ///
    /// # Arguments
    ///
    /// * `nodes` - Slice of available nodes to filter
    /// * `pod` - The pod that needs to be scheduled
    ///
    /// # Returns
    ///
    /// A `FilterResult` containing eligible nodes and filtered-out nodes with reasons.
    fn filter(&self, nodes: &[Node], pod: &Pod) -> FilterResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_selector_creation() {
        let selector = NodeSelector::new()
            .with_label("zone".to_string(), "us-west".to_string())
            .with_label("type".to_string(), "compute".to_string());

        assert_eq!(selector.labels.len(), 2);
        assert_eq!(selector.labels.get("zone"), Some(&"us-west".to_string()));
    }

    #[test]
    fn test_affinity_default() {
        let affinity = Affinity::default();
        assert!(affinity.required_during_scheduling.is_empty());
        assert!(affinity.preferred_during_scheduling.is_empty());
    }

    #[test]
    fn test_filter_result_operations() {
        let mut result = FilterResult::new();
        result.add_eligible("node-1".to_string());
        result.add_filtered(
            "node-2".to_string(),
            FilterReason::InsufficientResources {
                resource_type: "cpu".to_string(),
                requested: 4.0,
                available: 2.0,
            },
        );

        assert_eq!(result.eligible_nodes.len(), 1);
        assert_eq!(result.filtered_out.len(), 1);
    }

    #[test]
    fn test_serialization() {
        let reason = FilterReason::LabelMismatch {
            missing_labels: vec!["zone".to_string()],
        };
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: FilterReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }
}
