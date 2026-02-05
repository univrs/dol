//! CRDT Strategy Recommendation Engine
//!
//! This module provides AI-assisted recommendations for choosing appropriate
//! CRDT strategies based on field types, usage patterns, and consistency requirements.
//!
//! # Overview
//!
//! The recommendation engine analyzes DOL field declarations and suggests
//! appropriate CRDT strategies based on:
//! - Field type compatibility (from RFC-001 Table 4.1)
//! - Usage pattern (write-once, last-write-wins, collaborative, etc.)
//! - Consistency requirements (eventual, causal, strong)
//! - Performance implications
//!
//! # Example
//!
//! ```rust
//! use metadol::mcp::recommendations::{CrdtRecommender, UsagePattern, ConsistencyLevel};
//!
//! let recommender = CrdtRecommender::new();
//! let recommendation = recommender.recommend(
//!     "content",
//!     "String",
//!     UsagePattern::CollaborativeText,
//!     ConsistencyLevel::Eventual
//! );
//!
//! assert_eq!(recommendation.strategy, "peritext");
//! assert_eq!(recommendation.confidence, Confidence::High);
//! ```

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// CRDT strategy recommender.
///
/// Provides intelligent recommendations for CRDT strategies based on
/// field characteristics and usage patterns.
pub struct CrdtRecommender {
    /// Reference to RFC-001 compatibility matrix
    compatibility_matrix: CompatibilityMatrix,
}

impl CrdtRecommender {
    /// Creates a new CRDT recommender.
    pub fn new() -> Self {
        Self {
            compatibility_matrix: CompatibilityMatrix::default(),
        }
    }

    /// Recommends a CRDT strategy for a field.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the field
    /// * `field_type` - Type of the field (e.g., "String", "i32", "Set<String>")
    /// * `usage_pattern` - How the field will be used
    /// * `consistency` - Required consistency level
    ///
    /// # Returns
    ///
    /// A `CrdtRecommendation` with the suggested strategy and rationale.
    pub fn recommend(
        &self,
        field_name: &str,
        field_type: &str,
        usage_pattern: UsagePattern,
        consistency: ConsistencyLevel,
    ) -> CrdtRecommendation {
        // Parse type to determine base type
        let base_type = parse_base_type(field_type);

        // Get compatible strategies for this type
        let compatible = self
            .compatibility_matrix
            .get_compatible_strategies(&base_type);

        // Filter by usage pattern
        let candidates: Vec<_> = compatible
            .into_iter()
            .filter_map(|strategy| {
                self.score_strategy(strategy, &base_type, field_type, usage_pattern, consistency)
            })
            .collect();

        // Pick the best candidate
        let best = candidates
            .iter()
            .max_by_key(|c| c.score)
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to LWW if no good match
                ScoredStrategy {
                    strategy: "lww".to_string(),
                    score: 50,
                    confidence: Confidence::Low,
                }
            });

        // Generate recommendation
        self.build_recommendation(
            field_name,
            field_type,
            best,
            candidates,
            usage_pattern,
            consistency,
        )
    }

    /// Scores a strategy for a given context.
    fn score_strategy(
        &self,
        strategy: &str,
        base_type: &str,
        full_type: &str,
        usage: UsagePattern,
        consistency: ConsistencyLevel,
    ) -> Option<ScoredStrategy> {
        let mut score = 50; // Base score
        let mut confidence = Confidence::Medium;

        // Type-specific scoring
        match (base_type, strategy) {
            // String + Peritext for collaborative text
            ("String", "peritext") if matches!(usage, UsagePattern::CollaborativeText) => {
                score += 40;
                confidence = Confidence::High;
            }
            // String + LWW for simple updates
            ("String", "lww") if matches!(usage, UsagePattern::LastWriteWins) => {
                score += 30;
                confidence = Confidence::High;
            }
            // Integer + PNCounter for counters
            (t, "pn_counter") if is_integer_type(t) && matches!(usage, UsagePattern::Counter) => {
                score += 40;
                confidence = Confidence::High;
            }
            // Set + ORSet for multi-user sets
            ("Set", "or_set") if matches!(usage, UsagePattern::MultiUserSet) => {
                score += 40;
                confidence = Confidence::High;
            }
            // Vec/List + RGA for ordered lists
            (t, "rga")
                if (t == "Vec" || t == "List") && matches!(usage, UsagePattern::OrderedList) =>
            {
                score += 40;
                confidence = Confidence::High;
            }
            // Immutable for write-once
            (_, "immutable") if matches!(usage, UsagePattern::WriteOnce) => {
                score += 50;
                confidence = Confidence::High;
            }
            // LWW as safe default for simple types
            (t, "lww") if is_simple_type(t) => {
                score += 20;
                confidence = Confidence::Medium;
            }
            _ => {
                // Check if strategy is at least compatible
                if !self
                    .compatibility_matrix
                    .is_compatible(base_type, full_type, strategy)
                {
                    return None; // Incompatible
                }
            }
        }

        // Consistency level adjustments
        match consistency {
            ConsistencyLevel::Eventual => {
                // Most CRDTs work well with eventual consistency
                score += 10;
            }
            ConsistencyLevel::Causal => {
                // Prefer strategies with good causal semantics
                if matches!(strategy, "peritext" | "rga" | "mv_register") {
                    score += 15;
                }
            }
            ConsistencyLevel::Strong => {
                // Strong consistency requires coordination (lower CRDT score)
                score -= 20;
                if matches!(strategy, "immutable") {
                    score += 30; // Immutable doesn't need coordination
                }
            }
        }

        Some(ScoredStrategy {
            strategy: strategy.to_string(),
            score,
            confidence,
        })
    }

    /// Builds a complete recommendation from the scored strategy.
    fn build_recommendation(
        &self,
        field_name: &str,
        field_type: &str,
        best: ScoredStrategy,
        candidates: Vec<ScoredStrategy>,
        usage: UsagePattern,
        consistency: ConsistencyLevel,
    ) -> CrdtRecommendation {
        let reasoning = self.generate_reasoning(&best.strategy, field_type, usage, consistency);
        let example = self.generate_example(field_name, field_type, &best.strategy);
        let trade_offs = self.analyze_trade_offs(&best.strategy, field_type);

        // Get alternatives (top 3 excluding the best)
        let mut sorted_candidates = candidates;
        sorted_candidates.sort_by_key(|c| std::cmp::Reverse(c.score));
        let alternatives: Vec<_> = sorted_candidates
            .into_iter()
            .filter(|c| c.strategy != best.strategy)
            .take(3)
            .map(|c| Alternative {
                strategy: c.strategy.clone(),
                reason: self.generate_alternative_reason(&c.strategy, field_type, usage),
            })
            .collect();

        CrdtRecommendation {
            field_name: field_name.to_string(),
            field_type: field_type.to_string(),
            recommended_strategy: best.strategy,
            confidence: best.confidence,
            reasoning,
            example,
            alternatives,
            trade_offs,
        }
    }

    /// Generates reasoning for a recommendation.
    fn generate_reasoning(
        &self,
        strategy: &str,
        field_type: &str,
        usage: UsagePattern,
        consistency: ConsistencyLevel,
    ) -> String {
        match strategy {
            "immutable" => format!(
                "Immutable strategy ensures field is set exactly once and never modified. \
                 Perfect for identity fields like IDs and timestamps. Type {} is compatible. \
                 Consistency requirement: {:?}.",
                field_type, consistency
            ),
            "lww" => format!(
                "Last-Write-Wins strategy resolves conflicts by keeping the most recent write. \
                 Simple and efficient for single-valued fields like {}. Usage pattern: {:?}. \
                 Trade-off: Concurrent updates may be lost.",
                field_type, usage
            ),
            "peritext" => format!(
                "Peritext CRDT enables conflict-free collaborative text editing with formatting. \
                 Ideal for rich text fields ({}). Preserves concurrent edits and user intent. \
                 Usage pattern: {:?}. Best-in-class for collaborative documents.",
                field_type, usage
            ),
            "or_set" => format!(
                "Observed-Remove Set uses add-wins semantics for collections ({}). \
                 Concurrent additions always preserved. Removes only affect observed elements. \
                 Usage pattern: {:?}. Perfect for collaborative tag/member lists.",
                field_type, usage
            ),
            "pn_counter" => format!(
                "Positive-Negative Counter maintains separate increment/decrement operations. \
                 Commutative and convergent for numeric fields ({}). \
                 Usage pattern: {:?}. Ideal for likes, votes, and distributed counters.",
                field_type, usage
            ),
            "rga" => format!(
                "Replicated Growable Array preserves causal ordering for sequences ({}). \
                 Concurrent inserts ordered deterministically. \
                 Usage pattern: {:?}. Great for ordered lists and task sequences.",
                field_type, usage
            ),
            "mv_register" => format!(
                "Multi-Value Register keeps all concurrent values until explicitly resolved. \
                 Useful for detecting conflicts in {} fields. \
                 Usage pattern: {:?}. Allows application-level conflict resolution.",
                field_type, usage
            ),
            _ => format!("Strategy {} recommended for type {}.", strategy, field_type),
        }
    }

    /// Generates example DOL code for a recommendation.
    fn generate_example(&self, field_name: &str, field_type: &str, strategy: &str) -> String {
        let options = match strategy {
            "peritext" => ", formatting=\"full\"",
            "pn_counter" => ", min_value=0",
            _ => "",
        };

        format!(
            "@crdt({}{}) has {}: {}",
            strategy, options, field_name, field_type
        )
    }

    /// Generates alternative recommendation reasoning.
    fn generate_alternative_reason(
        &self,
        strategy: &str,
        _field_type: &str,
        usage: UsagePattern,
    ) -> String {
        match strategy {
            "lww" => "Simpler but loses concurrent edits (last writer wins)".to_string(),
            "immutable" => "Prevents all modifications after initial set".to_string(),
            "mv_register" => "Keeps all concurrent values for manual resolution".to_string(),
            "peritext" => format!(
                "Better for collaborative editing but higher overhead. Usage: {:?}",
                usage
            ),
            "or_set" => "Add-wins semantics, good for collections".to_string(),
            "pn_counter" => "Optimized for numeric increment/decrement operations".to_string(),
            "rga" => "Preserves ordering for sequences".to_string(),
            _ => format!("Alternative strategy: {}", strategy),
        }
    }

    /// Analyzes trade-offs for a strategy.
    fn analyze_trade_offs(&self, strategy: &str, _field_type: &str) -> TradeOffs {
        match strategy {
            "immutable" => TradeOffs {
                pros: vec![
                    "No merge conflicts".to_string(),
                    "Minimal storage overhead".to_string(),
                    "Fast lookups".to_string(),
                ],
                cons: vec![
                    "Cannot modify after creation".to_string(),
                    "Requires careful initial value".to_string(),
                ],
            },
            "lww" => TradeOffs {
                pros: vec![
                    "Simple implementation".to_string(),
                    "Low storage overhead".to_string(),
                    "Fast merges".to_string(),
                ],
                cons: vec![
                    "Loses concurrent updates".to_string(),
                    "Requires accurate timestamps".to_string(),
                ],
            },
            "peritext" => TradeOffs {
                pros: vec![
                    "Conflict-free concurrent editing".to_string(),
                    "Preserves formatting".to_string(),
                    "Excellent UX for collaboration".to_string(),
                ],
                cons: vec![
                    "Higher storage overhead".to_string(),
                    "More complex merge logic".to_string(),
                    "Larger sync payloads".to_string(),
                ],
            },
            "or_set" => TradeOffs {
                pros: vec![
                    "Add-wins semantics (no conflicts)".to_string(),
                    "Preserves all concurrent additions".to_string(),
                    "Intuitive behavior".to_string(),
                ],
                cons: vec![
                    "Tombstone overhead".to_string(),
                    "Larger than simple sets".to_string(),
                ],
            },
            "pn_counter" => TradeOffs {
                pros: vec![
                    "Commutative operations".to_string(),
                    "No coordination needed".to_string(),
                    "Perfect for distributed counters".to_string(),
                ],
                cons: vec![
                    "Per-actor state tracking".to_string(),
                    "Cannot enforce strict bounds without coordination".to_string(),
                ],
            },
            "rga" => TradeOffs {
                pros: vec![
                    "Preserves causal order".to_string(),
                    "Deterministic concurrent inserts".to_string(),
                    "Good for sequences".to_string(),
                ],
                cons: vec![
                    "Tombstone overhead for deletes".to_string(),
                    "More complex than simple arrays".to_string(),
                ],
            },
            "mv_register" => TradeOffs {
                pros: vec![
                    "Detects all conflicts".to_string(),
                    "Application can choose resolution".to_string(),
                    "Flexible conflict handling".to_string(),
                ],
                cons: vec![
                    "Requires manual conflict resolution".to_string(),
                    "Can accumulate values without cleanup".to_string(),
                ],
            },
            _ => TradeOffs {
                pros: vec![],
                cons: vec![],
            },
        }
    }
}

impl Default for CrdtRecommender {
    fn default() -> Self {
        Self::new()
    }
}

/// Usage pattern for a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UsagePattern {
    /// Field is set once and never modified
    WriteOnce,
    /// Last write wins, simple updates
    LastWriteWins,
    /// Collaborative text editing
    CollaborativeText,
    /// Multi-user set operations (tags, members)
    MultiUserSet,
    /// Numeric counter (likes, votes)
    Counter,
    /// Ordered list with concurrent edits
    OrderedList,
}

/// Required consistency level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ConsistencyLevel {
    /// Eventual consistency (most CRDTs)
    Eventual,
    /// Causal consistency
    Causal,
    /// Strong consistency (requires coordination)
    Strong,
}

/// Confidence level in a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Confidence {
    /// Low confidence recommendation
    Low,
    /// Medium confidence recommendation
    Medium,
    /// High confidence recommendation
    High,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::Low => write!(f, "low"),
            Confidence::Medium => write!(f, "medium"),
            Confidence::High => write!(f, "high"),
        }
    }
}

/// A scored CRDT strategy candidate.
#[derive(Debug, Clone)]
struct ScoredStrategy {
    strategy: String,
    score: i32,
    confidence: Confidence,
}

/// A CRDT strategy recommendation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrdtRecommendation {
    /// Field name
    pub field_name: String,
    /// Field type
    pub field_type: String,
    /// Recommended CRDT strategy
    pub recommended_strategy: String,
    /// Confidence in the recommendation
    pub confidence: Confidence,
    /// Reasoning for the recommendation
    pub reasoning: String,
    /// Example DOL code
    pub example: String,
    /// Alternative strategies
    pub alternatives: Vec<Alternative>,
    /// Trade-offs analysis
    pub trade_offs: TradeOffs,
}

/// An alternative CRDT strategy.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Alternative {
    /// Strategy name
    pub strategy: String,
    /// Reason to consider this alternative
    pub reason: String,
}

/// Trade-offs for a CRDT strategy.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TradeOffs {
    /// Pros (advantages)
    pub pros: Vec<String>,
    /// Cons (disadvantages)
    pub cons: Vec<String>,
}

/// CRDT type compatibility matrix (from RFC-001 Table 4.1).
#[derive(Debug, Clone)]
struct CompatibilityMatrix {
    // This encapsulates the RFC-001 compatibility rules
}

impl Default for CompatibilityMatrix {
    fn default() -> Self {
        Self {}
    }
}

impl CompatibilityMatrix {
    /// Gets compatible strategies for a base type.
    fn get_compatible_strategies(&self, base_type: &str) -> Vec<&'static str> {
        match base_type {
            "String" | "string" => vec!["immutable", "lww", "peritext", "mv_register"],
            t if is_integer_type(t) => vec!["immutable", "lww", "pn_counter", "mv_register"],
            t if is_float_type(t) => vec!["immutable", "lww", "mv_register"],
            "Bool" | "bool" => vec!["immutable", "lww", "mv_register"],
            "Set" => vec!["immutable", "or_set", "mv_register"],
            "Vec" | "List" => vec!["immutable", "lww", "rga", "mv_register"],
            "Map" => vec!["immutable", "lww", "mv_register"],
            _ => vec!["immutable", "lww", "mv_register"], // Default for custom types
        }
    }

    /// Checks if a strategy is compatible with a type.
    fn is_compatible(&self, base_type: &str, _full_type: &str, strategy: &str) -> bool {
        self.get_compatible_strategies(base_type)
            .contains(&strategy)
    }
}

/// Parses a type string to extract the base type.
fn parse_base_type(type_str: &str) -> String {
    // Handle generic types: "Set<String>" -> "Set"
    if let Some(pos) = type_str.find('<') {
        type_str[..pos].trim().to_string()
    } else {
        type_str.trim().to_string()
    }
}

/// Checks if a type is an integer type.
fn is_integer_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "Int"
            | "int"
    )
}

/// Checks if a type is a float type.
fn is_float_type(type_name: &str) -> bool {
    matches!(type_name, "f32" | "f64" | "Float" | "float")
}

/// Checks if a type is a simple scalar type.
fn is_simple_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "String" | "string" | "Bool" | "bool" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommend_collaborative_text() {
        let recommender = CrdtRecommender::new();
        let rec = recommender.recommend(
            "content",
            "String",
            UsagePattern::CollaborativeText,
            ConsistencyLevel::Eventual,
        );

        assert_eq!(rec.recommended_strategy, "peritext");
        assert_eq!(rec.confidence, Confidence::High);
        assert!(rec.reasoning.contains("collaborative"));
    }

    #[test]
    fn test_recommend_counter() {
        let recommender = CrdtRecommender::new();
        let rec = recommender.recommend(
            "likes",
            "i32",
            UsagePattern::Counter,
            ConsistencyLevel::Eventual,
        );

        assert_eq!(rec.recommended_strategy, "pn_counter");
        assert_eq!(rec.confidence, Confidence::High);
    }

    #[test]
    fn test_recommend_immutable() {
        let recommender = CrdtRecommender::new();
        let rec = recommender.recommend(
            "id",
            "String",
            UsagePattern::WriteOnce,
            ConsistencyLevel::Strong,
        );

        assert_eq!(rec.recommended_strategy, "immutable");
        assert_eq!(rec.confidence, Confidence::High);
    }

    #[test]
    fn test_recommend_or_set() {
        let recommender = CrdtRecommender::new();
        let rec = recommender.recommend(
            "tags",
            "Set<String>",
            UsagePattern::MultiUserSet,
            ConsistencyLevel::Eventual,
        );

        assert_eq!(rec.recommended_strategy, "or_set");
        assert_eq!(rec.confidence, Confidence::High);
    }

    #[test]
    fn test_recommend_rga() {
        let recommender = CrdtRecommender::new();
        let rec = recommender.recommend(
            "items",
            "Vec<String>",
            UsagePattern::OrderedList,
            ConsistencyLevel::Causal,
        );

        assert_eq!(rec.recommended_strategy, "rga");
        assert_eq!(rec.confidence, Confidence::High);
    }

    #[test]
    fn test_parse_base_type() {
        assert_eq!(parse_base_type("String"), "String");
        assert_eq!(parse_base_type("Set<String>"), "Set");
        assert_eq!(parse_base_type("Vec<i32>"), "Vec");
        assert_eq!(parse_base_type("Map<String, User>"), "Map");
    }

    #[test]
    fn test_is_integer_type() {
        assert!(is_integer_type("i32"));
        assert!(is_integer_type("u64"));
        assert!(is_integer_type("Int"));
        assert!(!is_integer_type("String"));
    }

    #[test]
    fn test_compatibility_matrix() {
        let matrix = CompatibilityMatrix::default();

        assert!(matrix.is_compatible("String", "String", "lww"));
        assert!(matrix.is_compatible("String", "String", "peritext"));
        assert!(!matrix.is_compatible("String", "String", "pn_counter"));

        assert!(matrix.is_compatible("i32", "i32", "pn_counter"));
        assert!(!matrix.is_compatible("i32", "i32", "peritext"));

        assert!(matrix.is_compatible("Set", "Set<String>", "or_set"));
        assert!(!matrix.is_compatible("Set", "Set<String>", "peritext"));
    }

    #[test]
    fn test_recommendation_has_alternatives() {
        let recommender = CrdtRecommender::new();
        let rec = recommender.recommend(
            "name",
            "String",
            UsagePattern::LastWriteWins,
            ConsistencyLevel::Eventual,
        );

        assert!(!rec.alternatives.is_empty());
    }

    #[test]
    fn test_trade_offs_analysis() {
        let recommender = CrdtRecommender::new();
        let trade_offs = recommender.analyze_trade_offs("peritext", "String");

        assert!(!trade_offs.pros.is_empty());
        assert!(!trade_offs.cons.is_empty());
        assert!(trade_offs
            .pros
            .iter()
            .any(|p| p.contains("collaboration") || p.contains("concurrent")));
    }
}
