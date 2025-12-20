//! Scheduling scoring implementation based on Metal DOL scheduling.score trait.
//!
//! This module provides scoring functions for ranking nodes that have passed
//! filtering. Scores help the scheduler select the optimal node for a pod
//! based on various criteria like resource balance, spreading, and affinity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::filter::NodeId;

/// Scoring functions used to evaluate node suitability.
///
/// Each scoring function represents a different optimization strategy that
/// can be applied when selecting a node for a pod.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScoringFunction {
    /// Balances resource utilization across the cluster
    ///
    /// Prefers nodes where CPU and memory utilization would be more balanced
    /// after scheduling the pod.
    ResourceBalance,

    /// Spreads pods across nodes to improve availability
    ///
    /// Prefers nodes that have fewer pods from the same deployment/service.
    Spreading,

    /// Packs pods tightly onto nodes to minimize resource fragmentation
    ///
    /// Prefers nodes with higher utilization to leave other nodes empty
    /// for large workloads.
    BinPacking,

    /// Honors preferred affinity rules
    ///
    /// Prefers nodes that match preferred (soft) affinity constraints.
    PreferredAffinity,

    /// Custom scoring function
    Custom(u32),
}

/// Score assigned to a node for a specific pod.
///
/// Contains individual scores from different scoring functions and a
/// weighted final score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeScore {
    /// Identifier of the scored node
    pub node_id: NodeId,

    /// Individual scores from each scoring function (0.0 to 100.0)
    pub scores: HashMap<ScoringFunction, f64>,

    /// Final weighted score combining all scoring functions
    pub final_score: f64,
}

impl NodeScore {
    /// Creates a new node score with the given node ID.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            scores: HashMap::new(),
            final_score: 0.0,
        }
    }

    /// Adds a score for a specific scoring function.
    ///
    /// # Arguments
    ///
    /// * `function` - The scoring function that generated this score
    /// * `score` - The score value (typically 0.0 to 100.0)
    pub fn add_score(&mut self, function: ScoringFunction, score: f64) {
        self.scores.insert(function, score);
    }

    /// Calculates the final weighted score based on the provided weights.
    ///
    /// The final score is the sum of (individual score * weight) for each
    /// scoring function.
    ///
    /// # Arguments
    ///
    /// * `weights` - The weights to apply to each scoring function
    pub fn calculate_final_score(&mut self, weights: &ScoringWeights) {
        let mut total = 0.0;
        let mut total_weight = 0.0;

        for (function, score) in &self.scores {
            let weight = weights.get_weight(*function);
            total += score * weight;
            total_weight += weight;
        }

        // Normalize by total weight if non-zero
        self.final_score = if total_weight > 0.0 {
            total / total_weight
        } else {
            0.0
        };
    }
}

/// Configurable weights for different scoring functions.
///
/// Weights determine the relative importance of each scoring function
/// when calculating the final node score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoringWeights {
    /// Weight for resource balance scoring (default: 1.0)
    pub resource_balance: f64,

    /// Weight for spreading scoring (default: 1.0)
    pub spreading: f64,

    /// Weight for bin packing scoring (default: 0.0)
    pub bin_packing: f64,

    /// Weight for preferred affinity scoring (default: 1.0)
    pub preferred_affinity: f64,

    /// Custom weights for additional scoring functions
    pub custom: HashMap<u32, f64>,
}

impl ScoringWeights {
    /// Creates a new set of scoring weights with default values.
    pub fn new() -> Self {
        Self {
            resource_balance: 1.0,
            spreading: 1.0,
            bin_packing: 0.0,
            preferred_affinity: 1.0,
            custom: HashMap::new(),
        }
    }

    /// Creates scoring weights optimized for high availability.
    ///
    /// Emphasizes spreading pods across nodes.
    pub fn high_availability() -> Self {
        Self {
            resource_balance: 0.5,
            spreading: 2.0,
            bin_packing: 0.0,
            preferred_affinity: 1.0,
            custom: HashMap::new(),
        }
    }

    /// Creates scoring weights optimized for resource efficiency.
    ///
    /// Emphasizes bin packing to reduce resource fragmentation.
    pub fn resource_efficiency() -> Self {
        Self {
            resource_balance: 0.5,
            spreading: 0.0,
            bin_packing: 2.0,
            preferred_affinity: 1.0,
            custom: HashMap::new(),
        }
    }

    /// Creates scoring weights optimized for balanced resource utilization.
    pub fn balanced() -> Self {
        Self {
            resource_balance: 2.0,
            spreading: 1.0,
            bin_packing: 0.5,
            preferred_affinity: 1.0,
            custom: HashMap::new(),
        }
    }

    /// Gets the weight for a specific scoring function.
    ///
    /// # Arguments
    ///
    /// * `function` - The scoring function to get the weight for
    ///
    /// # Returns
    ///
    /// The weight value for the specified function
    pub fn get_weight(&self, function: ScoringFunction) -> f64 {
        match function {
            ScoringFunction::ResourceBalance => self.resource_balance,
            ScoringFunction::Spreading => self.spreading,
            ScoringFunction::BinPacking => self.bin_packing,
            ScoringFunction::PreferredAffinity => self.preferred_affinity,
            ScoringFunction::Custom(id) => self.custom.get(&id).copied().unwrap_or(1.0),
        }
    }

    /// Sets the weight for a specific scoring function.
    ///
    /// # Arguments
    ///
    /// * `function` - The scoring function to set the weight for
    /// * `weight` - The new weight value
    pub fn set_weight(&mut self, function: ScoringFunction, weight: f64) {
        match function {
            ScoringFunction::ResourceBalance => self.resource_balance = weight,
            ScoringFunction::Spreading => self.spreading = weight,
            ScoringFunction::BinPacking => self.bin_packing = weight,
            ScoringFunction::PreferredAffinity => self.preferred_affinity = weight,
            ScoringFunction::Custom(id) => {
                self.custom.insert(id, weight);
            }
        }
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for scoring nodes to determine the best placement for a pod.
///
/// Implementors of this trait evaluate filtered nodes and assign scores
/// based on various optimization criteria. Higher scores indicate better
/// placement options.
pub trait Scorer<Node, Pod> {
    /// Scores the provided nodes for the given pod.
    ///
    /// # Arguments
    ///
    /// * `nodes` - Slice of nodes that have passed filtering
    /// * `pod` - The pod that needs to be scheduled
    /// * `weights` - Weights for different scoring functions
    ///
    /// # Returns
    ///
    /// A vector of `NodeScore` objects, one for each input node, sorted by
    /// final score in descending order (best scores first).
    fn score(&self, nodes: &[Node], pod: &Pod, weights: &ScoringWeights) -> Vec<NodeScore>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_score_creation() {
        let mut score = NodeScore::new("node-1".to_string());
        score.add_score(ScoringFunction::ResourceBalance, 75.0);
        score.add_score(ScoringFunction::Spreading, 60.0);

        assert_eq!(score.scores.len(), 2);
        assert_eq!(
            score.scores.get(&ScoringFunction::ResourceBalance),
            Some(&75.0)
        );
    }

    #[test]
    fn test_final_score_calculation() {
        let mut score = NodeScore::new("node-1".to_string());
        score.add_score(ScoringFunction::ResourceBalance, 80.0);
        score.add_score(ScoringFunction::Spreading, 60.0);

        let mut weights = ScoringWeights::new();
        weights.resource_balance = 2.0;
        weights.spreading = 1.0;

        score.calculate_final_score(&weights);

        // (80 * 2 + 60 * 1) / (2 + 1) = 220 / 3 ≈ 73.33
        assert!((score.final_score - 73.33).abs() < 0.01);
    }

    #[test]
    fn test_scoring_weights_presets() {
        let ha_weights = ScoringWeights::high_availability();
        assert_eq!(ha_weights.spreading, 2.0);

        let eff_weights = ScoringWeights::resource_efficiency();
        assert_eq!(eff_weights.bin_packing, 2.0);

        let balanced_weights = ScoringWeights::balanced();
        assert_eq!(balanced_weights.resource_balance, 2.0);
    }

    #[test]
    fn test_custom_weights() {
        let mut weights = ScoringWeights::new();
        weights.set_weight(ScoringFunction::Custom(42), 3.5);

        assert_eq!(weights.get_weight(ScoringFunction::Custom(42)), 3.5);
        assert_eq!(weights.get_weight(ScoringFunction::Custom(99)), 1.0);
    }

    #[test]
    fn test_scoring_function_hash() {
        let mut map = HashMap::new();
        map.insert(ScoringFunction::ResourceBalance, 100.0);
        map.insert(ScoringFunction::Spreading, 80.0);

        assert_eq!(map.get(&ScoringFunction::ResourceBalance), Some(&100.0));
    }

    #[test]
    fn test_serialization() {
        let weights = ScoringWeights::balanced();
        let json = serde_json::to_string(&weights).unwrap();
        let deserialized: ScoringWeights = serde_json::from_str(&json).unwrap();
        assert_eq!(weights, deserialized);
    }

    #[test]
    fn test_empty_scores_calculation() {
        let mut score = NodeScore::new("node-1".to_string());
        let weights = ScoringWeights::new();

        score.calculate_final_score(&weights);
        assert_eq!(score.final_score, 0.0);
    }
}
