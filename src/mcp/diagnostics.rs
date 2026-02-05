//! CRDT Schema Diagnostics
//!
//! This module provides diagnostic analysis for DOL schemas with CRDT annotations,
//! detecting anti-patterns, performance issues, and suggesting optimizations.
//!
//! # Overview
//!
//! The diagnostics engine analyzes DOL declarations to:
//! - Detect CRDT anti-patterns
//! - Warn about performance implications
//! - Suggest schema optimizations
//! - Integrate with validation errors/warnings
//!
//! # Example
//!
//! ```rust
//! use metadol::mcp::diagnostics::{SchemaDiagnostics, DiagnosticSeverity};
//!
//! let diagnostics = SchemaDiagnostics::new();
//! let issues = diagnostics.analyze_schema(dol_source);
//!
//! for issue in issues {
//!     if issue.severity == DiagnosticSeverity::Error {
//!         eprintln!("Error: {}", issue.message);
//!     }
//! }
//! ```

use crate::ast::{CrdtStrategy, Declaration, Gen, HasField, Statement};
use std::collections::HashSet;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Schema diagnostics analyzer.
///
/// Detects anti-patterns and performance issues in CRDT-annotated schemas.
pub struct SchemaDiagnostics {
    /// Enable strict checking
    strict: bool,
}

impl SchemaDiagnostics {
    /// Creates a new diagnostics analyzer.
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Creates a new diagnostics analyzer with strict checking enabled.
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Analyzes a DOL declaration for CRDT-related issues.
    ///
    /// # Arguments
    ///
    /// * `decl` - The declaration to analyze
    ///
    /// # Returns
    ///
    /// A vector of diagnostic issues found.
    pub fn analyze(&self, decl: &Declaration) -> Vec<DiagnosticIssue> {
        let mut issues = Vec::new();

        match decl {
            Declaration::Gene(gene) => {
                self.analyze_gene(gene, &mut issues);
            }
            _ => {
                // Other declaration types don't have CRDT annotations yet
            }
        }

        issues
    }

    /// Analyzes a gene declaration.
    fn analyze_gene(&self, gene: &Gen, issues: &mut Vec<DiagnosticIssue>) {
        let mut crdt_fields = Vec::new();
        let mut non_crdt_fields = Vec::new();

        // Collect fields (dereference the Box)
        for stmt in &gene.statements {
            if let Statement::HasField(field) = stmt {
                if field.crdt_annotation.is_some() {
                    crdt_fields.push(field.as_ref());
                } else {
                    non_crdt_fields.push(field.as_ref());
                }
            }
        }

        // Check for anti-patterns
        self.check_missing_immutable_id(gene, &crdt_fields, issues);
        self.check_mixed_crdt_non_crdt(gene, &crdt_fields, &non_crdt_fields, issues);
        self.check_lww_on_collections(gene, &crdt_fields, issues);
        self.check_peritext_without_max_length(gene, &crdt_fields, issues);
        self.check_counter_without_bounds(gene, &crdt_fields, issues);
        self.check_set_without_size_limit(gene, &crdt_fields, issues);
        self.check_excessive_mv_registers(gene, &crdt_fields, issues);
        self.check_performance_implications(gene, &crdt_fields, issues);
        self.check_constraint_compatibility(gene, &crdt_fields, issues);
    }

    /// Checks if a gene has an immutable ID field.
    fn check_missing_immutable_id(
        &self,
        gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        let has_immutable_id = crdt_fields.iter().any(|f| {
            matches!(f.name.as_str(), "id" | "uuid" | "identity")
                && f.crdt_annotation
                    .as_ref()
                    .map(|c| matches!(c.strategy, CrdtStrategy::Immutable))
                    .unwrap_or(false)
        });

        if !has_immutable_id && self.strict {
            issues.push(DiagnosticIssue {
                severity: DiagnosticSeverity::Warning,
                category: DiagnosticCategory::BestPractice,
                message: format!(
                    "Gene '{}' should have an immutable ID field for distributed identity",
                    gene.name
                ),
                suggestion: "Add: @crdt(immutable) has id: Uuid".to_string(),
                field: None,
            });
        }
    }

    /// Checks for mixed CRDT/non-CRDT fields.
    fn check_mixed_crdt_non_crdt(
        &self,
        gene: &Gen,
        crdt_fields: &[&HasField],
        non_crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        if !crdt_fields.is_empty() && !non_crdt_fields.is_empty() {
            issues.push(DiagnosticIssue {
                severity: DiagnosticSeverity::Info,
                category: DiagnosticCategory::Consistency,
                message: format!(
                    "Gene '{}' has {} CRDT fields and {} non-CRDT fields. \
                     Consider annotating all fields for consistent replication.",
                    gene.name,
                    crdt_fields.len(),
                    non_crdt_fields.len()
                ),
                suggestion: "Add CRDT annotations to remaining fields or use @crdt(lww) as default"
                    .to_string(),
                field: None,
            });
        }
    }

    /// Checks for LWW on collection types (anti-pattern).
    fn check_lww_on_collections(
        &self,
        _gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        for field in crdt_fields {
            if let Some(crdt) = &field.crdt_annotation {
                if matches!(crdt.strategy, CrdtStrategy::Lww) {
                    let type_str = format!("{:?}", field.type_);
                    if type_str.contains("Set")
                        || type_str.contains("Vec")
                        || type_str.contains("List")
                    {
                        issues.push(DiagnosticIssue {
                            severity: DiagnosticSeverity::Warning,
                            category: DiagnosticCategory::AntiPattern,
                            message: format!(
                                "Field '{}' uses LWW on collection type {}. \
                                 Entire collection will be replaced on conflict.",
                                field.name, type_str
                            ),
                            suggestion: format!(
                                "Use @crdt(or_set) for Set or @crdt(rga) for Vec/List instead"
                            ),
                            field: Some(field.name.clone()),
                        });
                    }
                }
            }
        }
    }

    /// Checks for Peritext without max_length.
    fn check_peritext_without_max_length(
        &self,
        _gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        for field in crdt_fields {
            if let Some(crdt) = &field.crdt_annotation {
                if matches!(crdt.strategy, CrdtStrategy::Peritext) {
                    let has_max_length = crdt.options.iter().any(|opt| opt.key == "max_length");
                    if !has_max_length {
                        issues.push(DiagnosticIssue {
                            severity: DiagnosticSeverity::Warning,
                            category: DiagnosticCategory::Performance,
                            message: format!(
                                "Field '{}' uses Peritext without max_length. \
                                 Unbounded text can lead to memory issues.",
                                field.name
                            ),
                            suggestion:
                                "Add max_length option: @crdt(peritext, max_length=1000000)"
                                    .to_string(),
                            field: Some(field.name.clone()),
                        });
                    }
                }
            }
        }
    }

    /// Checks for counters without bounds.
    fn check_counter_without_bounds(
        &self,
        _gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        for field in crdt_fields {
            if let Some(crdt) = &field.crdt_annotation {
                if matches!(crdt.strategy, CrdtStrategy::PnCounter) {
                    let has_bounds = crdt
                        .options
                        .iter()
                        .any(|opt| opt.key == "min_value" || opt.key == "max_value");
                    if !has_bounds && self.strict {
                        issues.push(DiagnosticIssue {
                            severity: DiagnosticSeverity::Info,
                            category: DiagnosticCategory::BestPractice,
                            message: format!(
                                "Counter field '{}' has no bounds. \
                                 Consider adding min_value/max_value for safety.",
                                field.name
                            ),
                            suggestion: "Add bounds: @crdt(pn_counter, min_value=0)".to_string(),
                            field: Some(field.name.clone()),
                        });
                    }
                }
            }
        }
    }

    /// Checks for sets without size limits.
    fn check_set_without_size_limit(
        &self,
        _gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        for field in crdt_fields {
            if let Some(crdt) = &field.crdt_annotation {
                if matches!(crdt.strategy, CrdtStrategy::OrSet) {
                    if let Some(constraint) = &field.constraint {
                        // Check if constraint limits size
                        let constraint_str = format!("{:?}", constraint);
                        if !constraint_str.contains("size") && !constraint_str.contains("count") {
                            if self.strict {
                                issues.push(DiagnosticIssue {
                                    severity: DiagnosticSeverity::Info,
                                    category: DiagnosticCategory::Performance,
                                    message: format!(
                                        "OR-Set field '{}' has no size limit. \
                                         Unbounded sets can grow indefinitely.",
                                        field.name
                                    ),
                                    suggestion: "Add constraint: where size <= 1000".to_string(),
                                    field: Some(field.name.clone()),
                                });
                            }
                        }
                    } else if self.strict {
                        issues.push(DiagnosticIssue {
                            severity: DiagnosticSeverity::Info,
                            category: DiagnosticCategory::Performance,
                            message: format!(
                                "OR-Set field '{}' has no size constraint",
                                field.name
                            ),
                            suggestion: "Add constraint: where size <= 1000".to_string(),
                            field: Some(field.name.clone()),
                        });
                    }
                }
            }
        }
    }

    /// Checks for excessive MV-Registers.
    fn check_excessive_mv_registers(
        &self,
        gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        let mv_count = crdt_fields
            .iter()
            .filter(|f| {
                f.crdt_annotation
                    .as_ref()
                    .map(|c| matches!(c.strategy, CrdtStrategy::MvRegister))
                    .unwrap_or(false)
            })
            .count();

        if mv_count > 3 {
            issues.push(DiagnosticIssue {
                severity: DiagnosticSeverity::Warning,
                category: DiagnosticCategory::Performance,
                message: format!(
                    "Gene '{}' has {} MV-Register fields. \
                     Each requires manual conflict resolution.",
                    gene.name, mv_count
                ),
                suggestion: "Consider using LWW or other automatic merge strategies where possible"
                    .to_string(),
                field: None,
            });
        }
    }

    /// Checks for performance implications.
    fn check_performance_implications(
        &self,
        gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        let mut has_peritext = false;
        let mut has_rga = false;
        let mut collection_count = 0;

        for field in crdt_fields {
            if let Some(crdt) = &field.crdt_annotation {
                match crdt.strategy {
                    CrdtStrategy::Peritext => has_peritext = true,
                    CrdtStrategy::Rga => has_rga = true,
                    CrdtStrategy::OrSet => collection_count += 1,
                    _ => {}
                }
            }
        }

        if has_peritext && has_rga {
            issues.push(DiagnosticIssue {
                severity: DiagnosticSeverity::Info,
                category: DiagnosticCategory::Performance,
                message: format!(
                    "Gene '{}' combines Peritext and RGA, both have O(n) merge complexity",
                    gene.name
                ),
                suggestion: "Monitor sync performance with large documents".to_string(),
                field: None,
            });
        }

        if collection_count > 5 {
            issues.push(DiagnosticIssue {
                severity: DiagnosticSeverity::Info,
                category: DiagnosticCategory::Performance,
                message: format!(
                    "Gene '{}' has {} collection fields with tombstone overhead",
                    gene.name, collection_count
                ),
                suggestion: "Consider periodic garbage collection for tombstones".to_string(),
                field: None,
            });
        }
    }

    /// Checks constraint-CRDT compatibility.
    fn check_constraint_compatibility(
        &self,
        _gene: &Gen,
        crdt_fields: &[&HasField],
        issues: &mut Vec<DiagnosticIssue>,
    ) {
        for field in crdt_fields {
            if let Some(constraint) = &field.constraint {
                let constraint_str = format!("{:?}", constraint);

                // Check for constraints that require strong consistency
                if constraint_str.contains("unique") {
                    issues.push(DiagnosticIssue {
                        severity: DiagnosticSeverity::Warning,
                        category: DiagnosticCategory::Correctness,
                        message: format!(
                            "Field '{}' has uniqueness constraint, which requires coordination",
                            field.name
                        ),
                        suggestion: "Consider using escrow pattern or removing CRDT annotation"
                            .to_string(),
                        field: Some(field.name.clone()),
                    });
                }

                // Check for immutable with mutable constraints
                if let Some(crdt) = &field.crdt_annotation {
                    if matches!(crdt.strategy, CrdtStrategy::Immutable) {
                        if constraint_str.contains("update") || constraint_str.contains("modify") {
                            issues.push(DiagnosticIssue {
                                severity: DiagnosticSeverity::Error,
                                category: DiagnosticCategory::Correctness,
                                message: format!(
                                    "Field '{}' is immutable but has mutable constraint",
                                    field.name
                                ),
                                suggestion: "Remove constraint or change CRDT strategy".to_string(),
                                field: Some(field.name.clone()),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Suggests optimizations for a schema.
    pub fn suggest_optimizations(&self, decl: &Declaration) -> Vec<Optimization> {
        let mut optimizations = Vec::new();

        if let Declaration::Gene(gene) = decl {
            self.suggest_gene_optimizations(gene, &mut optimizations);
        }

        optimizations
    }

    /// Suggests optimizations for a gene.
    fn suggest_gene_optimizations(&self, gene: &Gen, optimizations: &mut Vec<Optimization>) {
        let mut has_timestamp = false;
        let mut lww_fields = HashSet::new();

        for stmt in &gene.statements {
            if let Statement::HasField(field) = stmt {
                if field.name.contains("timestamp") || field.name.contains("time") {
                    has_timestamp = true;
                }

                if let Some(crdt) = &field.crdt_annotation {
                    if matches!(crdt.strategy, CrdtStrategy::Lww) {
                        lww_fields.insert(field.name.clone());
                    }
                }
            }
        }

        // Suggest hybrid logical clock if many LWW fields
        if !has_timestamp && lww_fields.len() > 3 {
            optimizations.push(Optimization {
                category: OptimizationCategory::Timestamp,
                title: "Add Hybrid Logical Clock".to_string(),
                description: format!(
                    "Gene '{}' has {} LWW fields but no timestamp. \
                     Consider adding an HLC for better causality tracking.",
                    gene.name,
                    lww_fields.len()
                ),
                impact: Impact::Medium,
                implementation: "@crdt(immutable) has hlc: HybridLogicalClock".to_string(),
            });
        }

        // Suggest batching for frequent updates
        let high_update_fields: Vec<_> = gene
            .statements
            .iter()
            .filter_map(|s| {
                if let Statement::HasField(field) = s {
                    if let Some(crdt) = &field.crdt_annotation {
                        if matches!(crdt.strategy, CrdtStrategy::PnCounter | CrdtStrategy::Lww) {
                            return Some(field.name.clone());
                        }
                    }
                }
                None
            })
            .collect();

        if high_update_fields.len() > 5 {
            optimizations.push(Optimization {
                category: OptimizationCategory::Sync,
                title: "Batch Updates".to_string(),
                description: format!(
                    "Gene '{}' has {} frequently-updated fields. \
                     Consider batching updates to reduce sync overhead.",
                    gene.name,
                    high_update_fields.len()
                ),
                impact: Impact::High,
                implementation: "Batch operations: defer sync until transaction commit".to_string(),
            });
        }
    }
}

impl Default for SchemaDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// A diagnostic issue found in a schema.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DiagnosticIssue {
    /// Severity of the issue
    pub severity: DiagnosticSeverity,
    /// Category of the issue
    pub category: DiagnosticCategory,
    /// Descriptive message
    pub message: String,
    /// Suggested fix
    pub suggestion: String,
    /// Affected field (if applicable)
    pub field: Option<String>,
}

/// Severity of a diagnostic issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DiagnosticSeverity {
    /// Informational message
    Info,
    /// Warning (should be addressed)
    Warning,
    /// Error (must be fixed)
    Error,
}

/// Category of diagnostic issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DiagnosticCategory {
    /// Anti-pattern detected
    AntiPattern,
    /// Performance concern
    Performance,
    /// Correctness issue
    Correctness,
    /// Consistency concern
    Consistency,
    /// Best practice suggestion
    BestPractice,
}

/// An optimization suggestion.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Optimization {
    /// Category of optimization
    pub category: OptimizationCategory,
    /// Title of the optimization
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Expected impact
    pub impact: Impact,
    /// How to implement
    pub implementation: String,
}

/// Category of optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OptimizationCategory {
    /// Timestamp/clock optimization
    Timestamp,
    /// Sync performance optimization
    Sync,
    /// Storage optimization
    Storage,
    /// Memory optimization
    Memory,
}

/// Expected impact of an optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Impact {
    /// Low impact
    Low,
    /// Medium impact
    Medium,
    /// High impact
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{CrdtAnnotation, Span, TypeExpr, Visibility};

    fn make_gene_with_fields(name: &str, fields: Vec<HasField>) -> Gen {
        Gen {
            visibility: Visibility::default(),
            name: name.to_string(),
            extends: None,
            statements: fields
                .into_iter()
                .map(|f| Statement::HasField(Box::new(f)))
                .collect(),
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        }
    }

    fn make_field(name: &str, type_: &str, strategy: Option<CrdtStrategy>) -> HasField {
        HasField {
            name: name.to_string(),
            type_: TypeExpr::Named(type_.to_string()),
            default: None,
            constraint: None,
            crdt_annotation: strategy.map(|s| CrdtAnnotation {
                strategy: s,
                options: vec![],
                span: Span::default(),
            }),
            span: Span::default(),
        }
    }

    #[test]
    fn test_missing_immutable_id() {
        let gene = make_gene_with_fields(
            "User",
            vec![make_field("name", "String", Some(CrdtStrategy::Lww))],
        );

        let diagnostics = SchemaDiagnostics::strict();
        let issues = diagnostics.analyze(&Declaration::Gene(gene));

        assert!(issues.iter().any(|i| i.message.contains("immutable ID")));
    }

    #[test]
    fn test_lww_on_collection() {
        let gene = make_gene_with_fields(
            "Document",
            vec![make_field("tags", "Set<String>", Some(CrdtStrategy::Lww))],
        );

        let diagnostics = SchemaDiagnostics::new();
        let issues = diagnostics.analyze(&Declaration::Gene(gene));

        assert!(issues
            .iter()
            .any(|i| i.category == DiagnosticCategory::AntiPattern));
    }

    #[test]
    fn test_peritext_without_max_length() {
        let gene = make_gene_with_fields(
            "Article",
            vec![make_field(
                "content",
                "String",
                Some(CrdtStrategy::Peritext),
            )],
        );

        let diagnostics = SchemaDiagnostics::new();
        let issues = diagnostics.analyze(&Declaration::Gene(gene));

        assert!(issues.iter().any(|i| i.message.contains("max_length")));
    }

    #[test]
    fn test_counter_without_bounds() {
        let gene = make_gene_with_fields(
            "Post",
            vec![make_field("likes", "i32", Some(CrdtStrategy::PnCounter))],
        );

        let diagnostics = SchemaDiagnostics::strict();
        let issues = diagnostics.analyze(&Declaration::Gene(gene));

        assert!(issues.iter().any(|i| i.message.contains("bounds")));
    }

    #[test]
    fn test_excessive_mv_registers() {
        let gene = make_gene_with_fields(
            "Config",
            vec![
                make_field("field1", "String", Some(CrdtStrategy::MvRegister)),
                make_field("field2", "String", Some(CrdtStrategy::MvRegister)),
                make_field("field3", "String", Some(CrdtStrategy::MvRegister)),
                make_field("field4", "String", Some(CrdtStrategy::MvRegister)),
            ],
        );

        let diagnostics = SchemaDiagnostics::new();
        let issues = diagnostics.analyze(&Declaration::Gene(gene));

        assert!(issues.iter().any(|i| i.message.contains("MV-Register")));
    }

    #[test]
    fn test_suggest_optimizations() {
        let gene = make_gene_with_fields(
            "User",
            vec![
                make_field("field1", "String", Some(CrdtStrategy::Lww)),
                make_field("field2", "String", Some(CrdtStrategy::Lww)),
                make_field("field3", "String", Some(CrdtStrategy::Lww)),
                make_field("field4", "String", Some(CrdtStrategy::Lww)),
            ],
        );

        let diagnostics = SchemaDiagnostics::new();
        let optimizations = diagnostics.suggest_optimizations(&Declaration::Gene(gene));

        assert!(!optimizations.is_empty());
    }
}
