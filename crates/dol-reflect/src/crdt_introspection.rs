//! CRDT Introspection for DOL Schemas
//!
//! This module provides CRDT-specific introspection capabilities, allowing
//! programs to query CRDT strategies, inspect constraint-CRDT compatibility,
//! and analyze merge semantics at runtime.
//!
//! # Features
//!
//! - Query CRDT strategy for each field
//! - Inspect constraint-CRDT compatibility
//! - Analyze merge conflict potential
//! - Validate CRDT configurations
//!
//! # Example
//!
//! ```rust
//! use dol_reflect::crdt_introspection::{CrdtIntrospector, MergeSemantics};
//!
//! // Create an introspector
//! let introspector = CrdtIntrospector::new();
//!
//! // Check merge semantics for a field
//! let semantics = MergeSemantics::for_strategy(
//!     metadol::CrdtStrategy::Lww
//! );
//!
//! assert!(semantics.is_commutative());
//! assert!(semantics.is_idempotent());
//! ```

use metadol::ast::{CrdtAnnotation, CrdtOption, CrdtStrategy, TypeExpr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schema_api::{FieldReflection, GenReflection, ReflectionError, SchemaRegistry};

/// Error type for CRDT introspection operations.
#[derive(Debug, thiserror::Error)]
pub enum CrdtError {
    /// CRDT strategy not found
    #[error("CRDT strategy not found for field '{0}'")]
    StrategyNotFound(String),

    /// Incompatible CRDT strategy for type
    #[error("Incompatible CRDT strategy '{strategy:?}' for type '{type_name}'")]
    IncompatibleStrategy {
        strategy: CrdtStrategy,
        type_name: String,
    },

    /// Invalid CRDT configuration
    #[error("Invalid CRDT configuration: {0}")]
    InvalidConfiguration(String),

    /// Constraint incompatible with CRDT
    #[error("Constraint '{constraint}' incompatible with CRDT strategy '{strategy:?}'")]
    IncompatibleConstraint {
        constraint: String,
        strategy: CrdtStrategy,
    },
}

/// Result type for CRDT operations.
pub type CrdtResult<T> = Result<T, CrdtError>;

/// Merge semantics for a CRDT strategy.
///
/// Describes the mathematical properties of a CRDT merge operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeSemantics {
    /// CRDT strategy
    pub strategy: CrdtStrategy,
    /// Is the merge commutative? (a ⊔ b = b ⊔ a)
    pub commutative: bool,
    /// Is the merge associative? ((a ⊔ b) ⊔ c = a ⊔ (b ⊔ c))
    pub associative: bool,
    /// Is the merge idempotent? (a ⊔ a = a)
    pub idempotent: bool,
    /// Does it satisfy Strong Eventual Consistency?
    pub sec: bool,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
}

/// How conflicts are resolved in a CRDT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Last write wins based on timestamp
    LastWriteWins,
    /// Add wins over remove (for sets)
    AddWins,
    /// Keep all concurrent values
    MultiValue,
    /// No conflicts possible (immutable)
    NoConflicts,
    /// Custom resolution logic
    Custom,
}

impl MergeSemantics {
    /// Returns merge semantics for a given CRDT strategy.
    pub fn for_strategy(strategy: CrdtStrategy) -> Self {
        match strategy {
            CrdtStrategy::Immutable => Self {
                strategy,
                commutative: true,
                associative: true,
                idempotent: true,
                sec: true,
                conflict_resolution: ConflictResolution::NoConflicts,
            },
            CrdtStrategy::Lww => Self {
                strategy,
                commutative: true,
                associative: true,
                idempotent: true,
                sec: true,
                conflict_resolution: ConflictResolution::LastWriteWins,
            },
            CrdtStrategy::OrSet => Self {
                strategy,
                commutative: true,
                associative: true,
                idempotent: true,
                sec: true,
                conflict_resolution: ConflictResolution::AddWins,
            },
            CrdtStrategy::PnCounter => Self {
                strategy,
                commutative: true,
                associative: true,
                idempotent: true,
                sec: true,
                conflict_resolution: ConflictResolution::NoConflicts,
            },
            CrdtStrategy::Peritext => Self {
                strategy,
                commutative: true,
                associative: true,
                idempotent: true,
                sec: true,
                conflict_resolution: ConflictResolution::Custom,
            },
            CrdtStrategy::Rga => Self {
                strategy,
                commutative: true,
                associative: true,
                idempotent: true,
                sec: true,
                conflict_resolution: ConflictResolution::Custom,
            },
            CrdtStrategy::MvRegister => Self {
                strategy,
                commutative: true,
                associative: true,
                idempotent: true,
                sec: true,
                conflict_resolution: ConflictResolution::MultiValue,
            },
        }
    }

    /// Returns true if the merge is commutative.
    pub fn is_commutative(&self) -> bool {
        self.commutative
    }

    /// Returns true if the merge is associative.
    pub fn is_associative(&self) -> bool {
        self.associative
    }

    /// Returns true if the merge is idempotent.
    pub fn is_idempotent(&self) -> bool {
        self.idempotent
    }

    /// Returns true if the CRDT satisfies Strong Eventual Consistency.
    pub fn is_sec(&self) -> bool {
        self.sec
    }

    /// Returns the conflict resolution strategy.
    pub fn conflict_resolution(&self) -> ConflictResolution {
        self.conflict_resolution
    }
}

/// Type compatibility information for CRDT strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCompatibility {
    /// Type name
    pub type_name: String,
    /// Compatible CRDT strategies
    pub compatible_strategies: Vec<CrdtStrategy>,
    /// Recommended strategy
    pub recommended: Option<CrdtStrategy>,
}

impl TypeCompatibility {
    /// Returns type compatibility for a given type.
    pub fn for_type(type_name: &str) -> Self {
        match type_name {
            "String" | "string" => Self {
                type_name: type_name.to_string(),
                compatible_strategies: vec![
                    CrdtStrategy::Immutable,
                    CrdtStrategy::Lww,
                    CrdtStrategy::Peritext,
                    CrdtStrategy::MvRegister,
                ],
                recommended: Some(CrdtStrategy::Lww),
            },
            "Int32" | "Int64" | "U32" | "U64" | "i32" | "i64" | "u32" | "u64" => Self {
                type_name: type_name.to_string(),
                compatible_strategies: vec![
                    CrdtStrategy::Immutable,
                    CrdtStrategy::Lww,
                    CrdtStrategy::PnCounter,
                    CrdtStrategy::MvRegister,
                ],
                recommended: Some(CrdtStrategy::PnCounter),
            },
            "Bool" | "bool" => Self {
                type_name: type_name.to_string(),
                compatible_strategies: vec![
                    CrdtStrategy::Immutable,
                    CrdtStrategy::Lww,
                    CrdtStrategy::MvRegister,
                ],
                recommended: Some(CrdtStrategy::Lww),
            },
            t if t.starts_with("Set<") || t.starts_with("Vec<") => Self {
                type_name: type_name.to_string(),
                compatible_strategies: vec![
                    CrdtStrategy::Immutable,
                    CrdtStrategy::OrSet,
                    CrdtStrategy::Rga,
                    CrdtStrategy::MvRegister,
                ],
                recommended: Some(CrdtStrategy::OrSet),
            },
            t if t.starts_with("List<") || t.starts_with("Array<") => Self {
                type_name: type_name.to_string(),
                compatible_strategies: vec![
                    CrdtStrategy::Immutable,
                    CrdtStrategy::Rga,
                    CrdtStrategy::MvRegister,
                ],
                recommended: Some(CrdtStrategy::Rga),
            },
            _ => Self {
                type_name: type_name.to_string(),
                compatible_strategies: vec![
                    CrdtStrategy::Immutable,
                    CrdtStrategy::Lww,
                    CrdtStrategy::MvRegister,
                ],
                recommended: Some(CrdtStrategy::Lww),
            },
        }
    }

    /// Checks if a strategy is compatible with this type.
    pub fn is_compatible(&self, strategy: CrdtStrategy) -> bool {
        self.compatible_strategies.contains(&strategy)
    }
}

/// CRDT field analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrdtFieldAnalysis {
    /// Field name
    pub field_name: String,
    /// Field type
    pub field_type: String,
    /// CRDT strategy
    pub strategy: CrdtStrategy,
    /// Merge semantics
    pub semantics: MergeSemantics,
    /// Type compatibility
    pub compatible: bool,
    /// Configuration options
    pub options: HashMap<String, String>,
    /// Potential issues
    pub issues: Vec<String>,
}

/// CRDT introspector for analyzing schemas.
///
/// Provides tools for inspecting CRDT annotations and validating
/// CRDT configurations in DOL schemas.
pub struct CrdtIntrospector {
    /// Cached type compatibility information
    type_cache: HashMap<String, TypeCompatibility>,
}

impl CrdtIntrospector {
    /// Creates a new CRDT introspector.
    pub fn new() -> Self {
        Self {
            type_cache: HashMap::new(),
        }
    }

    /// Analyzes a field's CRDT configuration.
    pub fn analyze_field(&mut self, field: &FieldReflection) -> CrdtResult<CrdtFieldAnalysis> {
        let strategy = field
            .crdt_strategy()
            .ok_or_else(|| CrdtError::StrategyNotFound(field.name().to_string()))?;

        let semantics = MergeSemantics::for_strategy(strategy);

        // Get type compatibility
        let type_compat = self.get_type_compatibility(field.type_name());
        let compatible = type_compat.is_compatible(strategy);

        // Extract options
        let options = if let Some(annotation) = field.crdt_annotation() {
            annotation
                .options
                .iter()
                .map(|opt| {
                    (
                        opt.key.clone(),
                        format!("{:?}", opt.value), // Convert Expr to string
                    )
                })
                .collect()
        } else {
            HashMap::new()
        };

        // Check for potential issues
        let mut issues = Vec::new();
        if !compatible {
            issues.push(format!(
                "Strategy {:?} may not be optimal for type {}",
                strategy,
                field.type_name()
            ));
        }

        if field.constraint().is_some() {
            // Check constraint-CRDT compatibility
            if !self.is_constraint_compatible(field, strategy) {
                issues.push("Constraint may conflict with CRDT semantics".to_string());
            }
        }

        Ok(CrdtFieldAnalysis {
            field_name: field.name().to_string(),
            field_type: field.type_name().to_string(),
            strategy,
            semantics,
            compatible,
            options,
            issues,
        })
    }

    /// Analyzes all CRDT fields in a Gen.
    pub fn analyze_gen(&mut self, gen: &GenReflection) -> Vec<CrdtFieldAnalysis> {
        gen.crdt_fields()
            .iter()
            .filter_map(|field| self.analyze_field(field).ok())
            .collect()
    }

    /// Analyzes all CRDT-annotated Gens in a registry.
    pub fn analyze_registry(
        &mut self,
        registry: &SchemaRegistry,
    ) -> HashMap<String, Vec<CrdtFieldAnalysis>> {
        registry
            .gens_with_crdt()
            .into_iter()
            .map(|gen| (gen.name().to_string(), self.analyze_gen(gen)))
            .collect()
    }

    /// Checks if a constraint is compatible with a CRDT strategy.
    fn is_constraint_compatible(&self, field: &FieldReflection, strategy: CrdtStrategy) -> bool {
        // Immutable fields can have any constraint
        if strategy == CrdtStrategy::Immutable {
            return true;
        }

        // LWW can have constraints but they may be violated during merges
        if strategy == CrdtStrategy::Lww {
            // Constraint compatibility depends on the constraint type
            // For now, we accept all constraints with a warning
            return true;
        }

        // Counters typically don't support range constraints
        if strategy == CrdtStrategy::PnCounter && field.constraint().is_some() {
            return false;
        }

        // Sets and sequences generally support constraints
        true
    }

    /// Gets or computes type compatibility.
    fn get_type_compatibility(&mut self, type_name: &str) -> &TypeCompatibility {
        self.type_cache
            .entry(type_name.to_string())
            .or_insert_with(|| TypeCompatibility::for_type(type_name))
    }

    /// Validates a CRDT annotation.
    pub fn validate_annotation(
        &mut self,
        field_type: &str,
        annotation: &CrdtAnnotation,
    ) -> CrdtResult<()> {
        let compat = self.get_type_compatibility(field_type);

        if !compat.is_compatible(annotation.strategy) {
            return Err(CrdtError::IncompatibleStrategy {
                strategy: annotation.strategy,
                type_name: field_type.to_string(),
            });
        }

        // Validate strategy-specific options
        match annotation.strategy {
            CrdtStrategy::Lww => {
                // LWW can have optional tie_break option
                for opt in &annotation.options {
                    if opt.key != "tie_break" {
                        return Err(CrdtError::InvalidConfiguration(format!(
                            "Unknown option '{}' for LWW strategy",
                            opt.key
                        )));
                    }
                }
            }
            CrdtStrategy::PnCounter => {
                // PnCounter can have min/max bounds
                for opt in &annotation.options {
                    if !matches!(opt.key.as_str(), "min_value" | "max_value") {
                        return Err(CrdtError::InvalidConfiguration(format!(
                            "Unknown option '{}' for PnCounter strategy",
                            opt.key
                        )));
                    }
                }
            }
            CrdtStrategy::Peritext => {
                // Peritext can have formatting options
                for opt in &annotation.options {
                    if !matches!(opt.key.as_str(), "formatting" | "rich_text") {
                        return Err(CrdtError::InvalidConfiguration(format!(
                            "Unknown option '{}' for Peritext strategy",
                            opt.key
                        )));
                    }
                }
            }
            _ => {
                // Other strategies don't have specific options yet
            }
        }

        Ok(())
    }

    /// Returns recommended CRDT strategy for a field type.
    pub fn recommend_strategy(&mut self, type_name: &str) -> Option<CrdtStrategy> {
        let compat = self.get_type_compatibility(type_name);
        compat.recommended
    }
}

impl Default for CrdtIntrospector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadol::ast::{CrdtOption, Expr, Literal, Span};

    #[test]
    fn test_merge_semantics() {
        let semantics = MergeSemantics::for_strategy(CrdtStrategy::Lww);

        assert!(semantics.is_commutative());
        assert!(semantics.is_associative());
        assert!(semantics.is_idempotent());
        assert!(semantics.is_sec());
        assert_eq!(
            semantics.conflict_resolution(),
            ConflictResolution::LastWriteWins
        );
    }

    #[test]
    fn test_type_compatibility() {
        let compat = TypeCompatibility::for_type("String");

        assert!(compat.is_compatible(CrdtStrategy::Lww));
        assert!(compat.is_compatible(CrdtStrategy::Peritext));
        assert!(!compat.is_compatible(CrdtStrategy::PnCounter));
    }

    #[test]
    fn test_counter_compatibility() {
        let compat = TypeCompatibility::for_type("Int32");

        assert!(compat.is_compatible(CrdtStrategy::PnCounter));
        assert!(compat.is_compatible(CrdtStrategy::Lww));
        assert_eq!(compat.recommended, Some(CrdtStrategy::PnCounter));
    }

    #[test]
    fn test_set_compatibility() {
        let compat = TypeCompatibility::for_type("Set<String>");

        assert!(compat.is_compatible(CrdtStrategy::OrSet));
        assert!(!compat.is_compatible(CrdtStrategy::PnCounter));
        assert_eq!(compat.recommended, Some(CrdtStrategy::OrSet));
    }

    #[test]
    fn test_validate_annotation() {
        let mut introspector = CrdtIntrospector::new();

        let annotation = CrdtAnnotation {
            strategy: CrdtStrategy::Lww,
            options: vec![],
            span: Span::default(),
        };

        assert!(introspector.validate_annotation("String", &annotation).is_ok());
    }

    #[test]
    fn test_validate_invalid_strategy() {
        let mut introspector = CrdtIntrospector::new();

        let annotation = CrdtAnnotation {
            strategy: CrdtStrategy::PnCounter,
            options: vec![],
            span: Span::default(),
        };

        assert!(introspector.validate_annotation("String", &annotation).is_err());
    }

    #[test]
    fn test_validate_invalid_option() {
        let mut introspector = CrdtIntrospector::new();

        let annotation = CrdtAnnotation {
            strategy: CrdtStrategy::Lww,
            options: vec![CrdtOption {
                key: "invalid_option".to_string(),
                value: Expr::Literal(Literal::String("value".to_string())),
                span: Span::default(),
            }],
            span: Span::default(),
        };

        assert!(introspector.validate_annotation("String", &annotation).is_err());
    }

    #[test]
    fn test_recommend_strategy() {
        let mut introspector = CrdtIntrospector::new();

        assert_eq!(
            introspector.recommend_strategy("String"),
            Some(CrdtStrategy::Lww)
        );
        assert_eq!(
            introspector.recommend_strategy("Int32"),
            Some(CrdtStrategy::PnCounter)
        );
        assert_eq!(
            introspector.recommend_strategy("Set<String>"),
            Some(CrdtStrategy::OrSet)
        );
    }
}
