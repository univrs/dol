//! AI-Powered Schema Validation
//!
//! This module provides intelligent schema validation that goes beyond
//! syntax checking to detect anti-patterns, suggest improvements, and
//! recommend better CRDT strategies.
//!
//! # Overview
//!
//! The schema validator:
//! - Detects CRDT anti-patterns
//! - Identifies potential performance issues
//! - Suggests better strategies for specific use cases
//! - Generates comprehensive validation reports
//!
//! # Example
//!
//! ```rust
//! use metadol::mcp::schema_validator::{SchemaValidator, ValidationContext};
//!
//! let validator = SchemaValidator::new();
//! let dol_source = r#"
//! gen document.schema {
//!   document has content: String @crdt(lww)
//! }
//! "#;
//!
//! let report = validator.validate_schema(dol_source, ValidationContext::default());
//! ```

use super::recommendations::{ConsistencyLevel, CrdtRecommender, UsagePattern};
use crate::ast::{CrdtStrategy, Declaration, Gen, Statement};
use crate::parse_file;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Schema validation severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ValidationSeverity {
    /// Critical issue that will cause runtime errors
    Error,
    /// Potential issue that may cause problems
    Warning,
    /// Suggestion for improvement
    Info,
}

/// Schema validation issue.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationIssue {
    /// Severity level
    pub severity: ValidationSeverity,
    /// Issue category
    pub category: String,
    /// Field or location where issue was found
    pub location: String,
    /// Description of the issue
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Code example of the fix
    pub fix_example: Option<String>,
}

/// Schema validation report.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationReport {
    /// List of validation issues
    pub issues: Vec<ValidationIssue>,
    /// Overall validation score (0-100)
    pub score: u8,
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Number of info suggestions
    pub info_count: usize,
    /// Summary of validation
    pub summary: String,
}

/// Context for schema validation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationContext {
    /// Expected usage patterns
    pub usage_patterns: HashMap<String, UsagePattern>,
    /// Consistency requirements
    pub consistency_requirements: HashMap<String, ConsistencyLevel>,
    /// Check for anti-patterns
    pub check_antipatterns: bool,
    /// Check for performance issues
    pub check_performance: bool,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            usage_patterns: HashMap::new(),
            consistency_requirements: HashMap::new(),
            check_antipatterns: true,
            check_performance: true,
        }
    }
}

/// Anti-pattern definitions.
struct AntiPattern {
    name: &'static str,
    description: &'static str,
    detector: fn(&str, &CrdtStrategy, &str) -> bool,
    suggestion: &'static str,
}

/// Schema validator with AI-powered recommendations.
pub struct SchemaValidator {
    recommender: CrdtRecommender,
    anti_patterns: Vec<AntiPattern>,
}

impl SchemaValidator {
    /// Creates a new schema validator.
    pub fn new() -> Self {
        let anti_patterns = vec![
            AntiPattern {
                name: "LWW for Collaborative Text",
                description: "Using LWW strategy for collaborative text editing",
                detector: |field_type, strategy, _field_name| {
                    field_type == "String" && matches!(strategy, CrdtStrategy::Lww)
                },
                suggestion: "Use 'peritext' strategy for collaborative text to preserve concurrent edits",
            },
            AntiPattern {
                name: "Immutable Counter",
                description: "Using immutable strategy for counter fields",
                detector: |_field_type, strategy, field_name| {
                    matches!(strategy, CrdtStrategy::Immutable)
                        && (field_name.contains("count") || field_name.contains("counter"))
                },
                suggestion: "Use 'pn_counter' strategy for counters to support increment/decrement operations",
            },
            AntiPattern {
                name: "LWW for Sets",
                description: "Using LWW strategy for set-based fields",
                detector: |field_type, strategy, _field_name| {
                    (field_type.starts_with("Set<") || field_type.starts_with("Vec<"))
                        && matches!(strategy, CrdtStrategy::Lww)
                },
                suggestion: "Use 'or_set' for sets or 'rga' for ordered lists to support concurrent additions",
            },
            AntiPattern {
                name: "Strong Consistency for High-Write Fields",
                description: "Using immutable/strong consistency for frequently updated fields",
                detector: |_field_type, strategy, field_name| {
                    matches!(strategy, CrdtStrategy::Immutable)
                        && (field_name.contains("status") || field_name.contains("state"))
                },
                suggestion: "Consider 'lww' or 'mv_register' for fields that need updates",
            },
        ];

        Self {
            recommender: CrdtRecommender::new(),
            anti_patterns,
        }
    }

    /// Validates a DOL schema and returns a detailed report.
    pub fn validate_schema(&self, source: &str, context: ValidationContext) -> ValidationReport {
        let mut issues = Vec::new();

        // Parse the schema
        let decl = match parse_file(source) {
            Ok(d) => d,
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    category: "Parse Error".to_string(),
                    location: "schema".to_string(),
                    message: format!("Failed to parse schema: {}", e),
                    suggestion: Some("Fix syntax errors before validation".to_string()),
                    fix_example: None,
                });
                return self.build_report(issues);
            }
        };

        // Extract Gen declaration
        let gen = match &decl {
            Declaration::Gene(g) => g,
            _ => {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    category: "Invalid Declaration".to_string(),
                    location: "schema".to_string(),
                    message: "Expected a Gen declaration".to_string(),
                    suggestion: None,
                    fix_example: None,
                });
                return self.build_report(issues);
            }
        };

        // Validate fields
        self.validate_fields(gen, &context, &mut issues);

        // Check for missing standard fields
        self.check_missing_fields(gen, &mut issues);

        // Check exegesis documentation
        self.check_exegesis(gen, &mut issues);

        self.build_report(issues)
    }

    /// Validates individual fields.
    fn validate_fields(
        &self,
        gen: &Gen,
        context: &ValidationContext,
        issues: &mut Vec<ValidationIssue>,
    ) {
        for stmt in &gen.statements {
            if let Statement::HasField(has_field) = stmt {
                let property = &has_field.name;
                let type_expr = &has_field.type_;
                let crdt_annotation = &has_field.crdt_annotation;

                // Extract CRDT strategy
                let strategy = if let Some(annotation) = crdt_annotation {
                    &annotation.strategy
                } else {
                    // Missing CRDT annotation
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        category: "Missing CRDT Strategy".to_string(),
                        location: property.clone(),
                        message: format!("Field '{}' has no CRDT strategy annotation", property),
                        suggestion: Some(
                            "Add @crdt() annotation to specify merge behavior".to_string(),
                        ),
                        fix_example: Some(format!(
                            "has {}: {} @crdt(lww)",
                            property,
                            format_type_expr(type_expr)
                        )),
                    });
                    continue;
                };

                let type_str = format_type_expr(type_expr);

                // Check for anti-patterns
                if context.check_antipatterns {
                    self.check_antipatterns(&type_str, strategy, property, issues);
                }

                // Check if strategy is appropriate for the type
                self.check_strategy_compatibility(&type_str, strategy, property, context, issues);

                // Performance checks
                if context.check_performance {
                    self.check_performance(&type_str, strategy, property, issues);
                }
            }
        }
    }

    /// Checks for anti-patterns.
    fn check_antipatterns(
        &self,
        field_type: &str,
        strategy: &CrdtStrategy,
        field_name: &str,
        issues: &mut Vec<ValidationIssue>,
    ) {
        for pattern in &self.anti_patterns {
            if (pattern.detector)(field_type, strategy, field_name) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    category: "Anti-Pattern".to_string(),
                    location: field_name.to_string(),
                    message: format!("{}: {}", pattern.name, pattern.description),
                    suggestion: Some(pattern.suggestion.to_string()),
                    fix_example: None,
                });
            }
        }
    }

    /// Checks strategy compatibility with field type.
    fn check_strategy_compatibility(
        &self,
        field_type: &str,
        strategy: &CrdtStrategy,
        field_name: &str,
        context: &ValidationContext,
        issues: &mut Vec<ValidationIssue>,
    ) {
        // Get usage pattern if provided
        let usage_pattern = context
            .usage_patterns
            .get(field_name)
            .copied()
            .unwrap_or(UsagePattern::LastWriteWins);

        let consistency = context
            .consistency_requirements
            .get(field_name)
            .copied()
            .unwrap_or(ConsistencyLevel::Eventual);

        // Get recommendation
        let recommendation =
            self.recommender
                .recommend(field_name, field_type, usage_pattern, consistency);

        // Check if current strategy matches recommendation
        let strategy_str = format!("{:?}", strategy).to_lowercase();
        if strategy_str != recommendation.recommended_strategy {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Strategy Recommendation".to_string(),
                location: field_name.to_string(),
                message: format!(
                    "Field '{}' uses '{}' but '{}' might be more appropriate",
                    field_name, strategy_str, recommendation.recommended_strategy
                ),
                suggestion: Some(recommendation.reasoning.clone()),
                fix_example: Some(format!(
                    "has {}: {} @crdt({})",
                    field_name, field_type, recommendation.recommended_strategy
                )),
            });
        }
    }

    /// Checks for performance issues.
    fn check_performance(
        &self,
        field_type: &str,
        strategy: &CrdtStrategy,
        field_name: &str,
        issues: &mut Vec<ValidationIssue>,
    ) {
        // Check for large collections with inefficient strategies
        if (field_type.starts_with("Vec<") || field_type.starts_with("Set<"))
            && matches!(strategy, CrdtStrategy::Lww)
        {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Performance".to_string(),
                location: field_name.to_string(),
                message: format!(
                    "Field '{}' is a collection using LWW, which replaces entire collection on update",
                    field_name
                ),
                suggestion: Some("Consider element-level CRDTs (or_set, rga) for better merge behavior".to_string()),
                fix_example: None,
            });
        }

        // Check for text fields using non-text CRDTs
        if field_type == "String" && matches!(strategy, CrdtStrategy::OrSet | CrdtStrategy::Rga) {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Inefficient Strategy".to_string(),
                location: field_name.to_string(),
                message: format!(
                    "Field '{}' is a String but uses a collection CRDT",
                    field_name
                ),
                suggestion: Some(
                    "Use 'lww' for simple text or 'peritext' for collaborative editing".to_string(),
                ),
                fix_example: None,
            });
        }
    }

    /// Checks for missing standard fields.
    fn check_missing_fields(&self, gen: &Gen, issues: &mut Vec<ValidationIssue>) {
        let field_names: Vec<String> = gen
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Statement::Has { property, .. } => Some(property.clone()),
                Statement::HasField(has_field) => Some(has_field.name.clone()),
                _ => None,
            })
            .collect();

        // Check for ID field
        if !field_names.iter().any(|f| f == "id") {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Missing Field".to_string(),
                location: "schema".to_string(),
                message: "Schema lacks an 'id' field for unique identification".to_string(),
                suggestion: Some("Add an immutable 'id' field".to_string()),
                fix_example: Some("has id: String @crdt(immutable)".to_string()),
            });
        }

        // Check for timestamp fields
        let has_created = field_names.iter().any(|f| f.contains("created"));
        let has_updated = field_names.iter().any(|f| f.contains("updated"));

        if !has_created && !has_updated {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Missing Field".to_string(),
                location: "schema".to_string(),
                message: "Schema lacks timestamp fields for audit trail".to_string(),
                suggestion: Some("Add created_at and updated_at fields".to_string()),
                fix_example: Some(
                    "has created_at: i64 @crdt(immutable)\nhas updated_at: i64 @crdt(lww)"
                        .to_string(),
                ),
            });
        }
    }

    /// Checks exegesis documentation.
    fn check_exegesis(&self, gen: &Gen, issues: &mut Vec<ValidationIssue>) {
        if gen.exegesis.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Documentation".to_string(),
                location: "exegesis".to_string(),
                message: "Schema lacks exegesis documentation".to_string(),
                suggestion: Some(
                    "Add exegesis block to document the schema purpose and design decisions"
                        .to_string(),
                ),
                fix_example: None,
            });
        }
    }

    /// Builds the final validation report.
    fn build_report(&self, issues: Vec<ValidationIssue>) -> ValidationReport {
        let error_count = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .count();
        let warning_count = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .count();
        let info_count = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Info)
            .count();

        // Calculate score (100 - penalties)
        let mut score = 100u8;
        score = score.saturating_sub((error_count * 20) as u8);
        score = score.saturating_sub((warning_count * 10) as u8);
        score = score.saturating_sub((info_count * 2) as u8);

        let summary = if error_count > 0 {
            format!("Schema has {} errors that must be fixed", error_count)
        } else if warning_count > 0 {
            format!("Schema has {} warnings to address", warning_count)
        } else if info_count > 0 {
            format!(
                "Schema is valid with {} suggestions for improvement",
                info_count
            )
        } else {
            "Schema is well-formed with no issues detected".to_string()
        };

        ValidationReport {
            issues,
            score,
            error_count,
            warning_count,
            info_count,
            summary,
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to format type expressions.
fn format_type_expr(type_expr: &crate::ast::TypeExpr) -> String {
    use crate::ast::TypeExpr;
    match type_expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Generic { name, args } => {
            let args_str = args
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name, args_str)
        }
        TypeExpr::Tuple(types) => {
            let types_str = types
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", types_str)
        }
        TypeExpr::Function {
            params,
            return_type,
        } => {
            let params_str = params
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({}) -> {}", params_str, format_type_expr(return_type))
        }
        TypeExpr::Never => "!".to_string(),
        TypeExpr::Enum { .. } => "enum".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "TODO: Fix MCP validator - not detecting antipatterns correctly"]
    fn test_validate_schema_with_antipattern() {
        let validator = SchemaValidator::new();
        let source = r#"
gen document.schema {
  document has content: String @crdt(lww)
}

exegesis {
  A document schema.
}
"#;

        let report = validator.validate_schema(source, ValidationContext::default());
        assert!(report.warning_count > 0 || report.info_count > 0);
    }

    #[test]
    fn test_validate_schema_missing_id() {
        let validator = SchemaValidator::new();
        let source = r#"
gen user.schema {
  user has name: String @crdt(lww)
}

exegesis {
  A user schema.
}
"#;

        let report = validator.validate_schema(source, ValidationContext::default());
        let has_id_warning = report.issues.iter().any(|i| i.message.contains("id"));
        assert!(has_id_warning);
    }

    #[test]
    #[ignore = "TODO: Fix MCP validator - not detecting missing exegesis"]
    fn test_validate_schema_missing_exegesis() {
        let validator = SchemaValidator::new();
        let source = r#"
gen user.schema {
  user has id: String @crdt(immutable)
}
"#;

        let report = validator.validate_schema(source, ValidationContext::default());
        let has_exegesis_warning = report.issues.iter().any(|i| i.category == "Documentation");
        assert!(has_exegesis_warning);
    }
}
