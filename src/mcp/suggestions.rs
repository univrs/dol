//! AI-Powered Schema Suggestions
//!
//! This module provides intelligent suggestions for improving DOL schemas,
//! including CRDT strategy optimizations, field additions, and best practices.
//!
//! # Overview
//!
//! The suggestion engine:
//! - Analyzes schema patterns and usage
//! - Recommends field additions for common use cases
//! - Suggests CRDT strategy improvements
//! - Generates exegesis documentation
//!
//! # Example
//!
//! ```rust
//! use metadol::mcp::suggestions::{SuggestionEngine, SuggestionContext};
//!
//! let engine = SuggestionEngine::new();
//! let dol_source = r#"
//! gen user.profile {
//!   user has name: String @crdt(lww)
//! }
//! "#;
//!
//! let suggestions = engine.analyze_and_suggest(dol_source, SuggestionContext::default());
//! ```

use super::recommendations::CrdtRecommender;
use crate::ast::{Declaration, Gen, Statement};
use crate::parse_file;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Suggestion priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SuggestionPriority {
    /// Low priority, nice to have
    Low,
    /// Medium priority, recommended
    Medium,
    /// High priority, strongly recommended
    High,
    /// Critical, should be addressed
    Critical,
}

/// Types of suggestions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SuggestionType {
    /// Add a new field
    AddField,
    /// Modify existing field
    ModifyField,
    /// Change CRDT strategy
    ChangeStrategy,
    /// Add documentation
    AddDocumentation,
    /// Add constraint
    AddConstraint,
    /// Improve structure
    ImproveStructure,
}

/// A schema improvement suggestion.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Suggestion {
    /// Suggestion type
    pub suggestion_type: SuggestionType,
    /// Priority level
    pub priority: SuggestionPriority,
    /// Title of the suggestion
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Rationale for the suggestion
    pub rationale: String,
    /// Code example implementing the suggestion
    pub code_example: String,
    /// Expected impact
    pub impact: String,
}

/// Collection of suggestions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SuggestionSet {
    /// All suggestions
    pub suggestions: Vec<Suggestion>,
    /// Summary of suggestions
    pub summary: String,
    /// Overall schema health score (0-100)
    pub health_score: u8,
}

/// Context for suggestion generation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SuggestionContext {
    /// Target use case (e.g., "user_profile", "document_editing")
    pub use_case: Option<String>,
    /// Expected scale (users, documents, etc.)
    pub expected_scale: Option<String>,
    /// Performance priorities
    pub performance_priority: bool,
    /// Security priorities
    pub security_priority: bool,
}

/// Suggestion engine for DOL schemas.
pub struct SuggestionEngine {
    recommender: CrdtRecommender,
}

impl SuggestionEngine {
    /// Creates a new suggestion engine.
    pub fn new() -> Self {
        Self {
            recommender: CrdtRecommender::new(),
        }
    }

    /// Analyzes a schema and generates suggestions.
    pub fn analyze_and_suggest(&self, source: &str, context: SuggestionContext) -> SuggestionSet {
        let mut suggestions = Vec::new();

        // Parse the schema
        let decl = match parse_file(source) {
            Ok(d) => d,
            Err(_) => {
                return SuggestionSet {
                    suggestions: vec![],
                    summary: "Unable to parse schema. Fix syntax errors first.".to_string(),
                    health_score: 0,
                };
            }
        };

        // Extract Gen declaration
        let gen = match &decl {
            Declaration::Gene(g) => g,
            _ => {
                return SuggestionSet {
                    suggestions: vec![],
                    summary: "Expected a Gen declaration".to_string(),
                    health_score: 0,
                };
            }
        };

        // Generate suggestions
        self.suggest_standard_fields(gen, &mut suggestions);
        self.suggest_audit_fields(gen, &mut suggestions);
        self.suggest_crdt_improvements(gen, &mut suggestions);
        self.suggest_documentation(gen, &mut suggestions);
        self.suggest_use_case_fields(gen, &context, &mut suggestions);

        // Sort by priority
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Calculate health score
        let health_score = self.calculate_health_score(gen, &suggestions);

        // Generate summary
        let summary = self.generate_summary(&suggestions, health_score);

        SuggestionSet {
            suggestions,
            summary,
            health_score,
        }
    }

    /// Suggests standard fields (id, timestamps).
    fn suggest_standard_fields(&self, gen: &Gen, suggestions: &mut Vec<Suggestion>) {
        let field_names: Vec<String> = self.extract_field_names(gen);

        // Check for ID field
        if !field_names.iter().any(|f| f == "id") {
            suggestions.push(Suggestion {
                suggestion_type: SuggestionType::AddField,
                priority: SuggestionPriority::High,
                title: "Add unique identifier field".to_string(),
                description: "Every entity should have a unique identifier for referencing and deduplication".to_string(),
                rationale: "Unique IDs enable entity references, deduplication, and distributed coordination".to_string(),
                code_example: format!("  {} has id: String @crdt(immutable)", gen.name.split('.').next().unwrap_or("entity")),
                impact: "Enables proper entity management and referencing".to_string(),
            });
        }
    }

    /// Suggests audit fields (created_at, updated_at).
    fn suggest_audit_fields(&self, gen: &Gen, suggestions: &mut Vec<Suggestion>) {
        let field_names: Vec<String> = self.extract_field_names(gen);
        let entity = gen.name.split('.').next().unwrap_or("entity");

        let has_created = field_names.iter().any(|f| f.contains("created"));
        let has_updated = field_names.iter().any(|f| f.contains("updated"));

        if !has_created {
            suggestions.push(Suggestion {
                suggestion_type: SuggestionType::AddField,
                priority: SuggestionPriority::Medium,
                title: "Add creation timestamp".to_string(),
                description: "Track when entities are created for audit and debugging".to_string(),
                rationale: "Creation timestamps are essential for audit trails, debugging, and temporal queries".to_string(),
                code_example: format!("  {} has created_at: i64 @crdt(immutable)", entity),
                impact: "Enables temporal tracking and audit capabilities".to_string(),
            });
        }

        if !has_updated {
            suggestions.push(Suggestion {
                suggestion_type: SuggestionType::AddField,
                priority: SuggestionPriority::Medium,
                title: "Add update timestamp".to_string(),
                description: "Track when entities are last modified".to_string(),
                rationale:
                    "Update timestamps help with cache invalidation, sync, and change tracking"
                        .to_string(),
                code_example: format!("  {} has updated_at: i64 @crdt(lww)", entity),
                impact: "Enables change tracking and cache management".to_string(),
            });
        }
    }

    /// Suggests CRDT strategy improvements.
    fn suggest_crdt_improvements(&self, gen: &Gen, suggestions: &mut Vec<Suggestion>) {
        let entity = gen.name.split('.').next().unwrap_or("entity");

        for stmt in &gen.statements {
            if let Statement::HasField(has_field) = stmt {
                let property = &has_field.name;
                let type_expr = &has_field.type_;
                if let Some(annotation) = &has_field.crdt_annotation {
                    let type_str = self.format_type_expr(type_expr);
                    let current_strategy = format!("{:?}", annotation.strategy).to_lowercase();

                    // Check for common anti-patterns
                    if type_str == "String" && current_strategy == "lww" {
                        if property.contains("content")
                            || property.contains("text")
                            || property.contains("description")
                        {
                            suggestions.push(Suggestion {
                                suggestion_type: SuggestionType::ChangeStrategy,
                                priority: SuggestionPriority::Medium,
                                title: format!("Consider peritext for '{}' field", property),
                                description: "Text fields benefit from character-level CRDTs for collaborative editing".to_string(),
                                rationale: "Peritext preserves concurrent character-level edits, preventing last-write-wins overwrites".to_string(),
                                code_example: format!("  {} has {}: String @crdt(peritext)", entity, property),
                                impact: "Enables real-time collaborative editing without conflicts".to_string(),
                            });
                        }
                    }

                    if (type_str.starts_with("Set<") || type_str.starts_with("Vec<"))
                        && current_strategy == "lww"
                    {
                        let suggested_strategy = if type_str.starts_with("Set<") {
                            "or_set"
                        } else {
                            "rga"
                        };
                        suggestions.push(Suggestion {
                            suggestion_type: SuggestionType::ChangeStrategy,
                            priority: SuggestionPriority::High,
                            title: format!("Use element-level CRDT for '{}' field", property),
                            description: "Collection fields should use element-level CRDTs instead of LWW".to_string(),
                            rationale: format!("LWW replaces entire collections, losing concurrent additions. {} tracks individual elements.", suggested_strategy),
                            code_example: format!("  {} has {}: {} @crdt({})", entity, property, type_str, suggested_strategy),
                            impact: "Preserves concurrent additions/removals to the collection".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Suggests documentation improvements.
    fn suggest_documentation(&self, gen: &Gen, suggestions: &mut Vec<Suggestion>) {
        if gen.exegesis.trim().is_empty() {
            suggestions.push(Suggestion {
                suggestion_type: SuggestionType::AddDocumentation,
                priority: SuggestionPriority::High,
                title: "Add exegesis documentation".to_string(),
                description: "Document the schema's purpose, design decisions, and CRDT strategies"
                    .to_string(),
                rationale:
                    "Exegesis is mandatory in DOL and provides essential context for maintainers"
                        .to_string(),
                code_example: self.generate_exegesis_template(gen),
                impact: "Improves maintainability and team understanding".to_string(),
            });
        } else if gen.exegesis.len() < 50 {
            suggestions.push(Suggestion {
                suggestion_type: SuggestionType::AddDocumentation,
                priority: SuggestionPriority::Low,
                title: "Expand exegesis documentation".to_string(),
                description:
                    "Current exegesis is very brief. Add more detail about design decisions."
                        .to_string(),
                rationale:
                    "Comprehensive documentation helps future maintainers understand CRDT choices"
                        .to_string(),
                code_example: self.generate_exegesis_template(gen),
                impact: "Better documentation for long-term maintenance".to_string(),
            });
        }
    }

    /// Suggests use-case specific fields.
    fn suggest_use_case_fields(
        &self,
        gen: &Gen,
        context: &SuggestionContext,
        suggestions: &mut Vec<Suggestion>,
    ) {
        let field_names: Vec<String> = self.extract_field_names(gen);
        let entity = gen.name.split('.').next().unwrap_or("entity");

        if let Some(use_case) = &context.use_case {
            match use_case.as_str() {
                "user_profile" => {
                    if !field_names.iter().any(|f| f.contains("email")) {
                        suggestions.push(Suggestion {
                            suggestion_type: SuggestionType::AddField,
                            priority: SuggestionPriority::Medium,
                            title: "Add email field for user profile".to_string(),
                            description: "User profiles typically need email for authentication and communication".to_string(),
                            rationale: "Email is a standard field for user identification and contact".to_string(),
                            code_example: format!("  {} has email: String @crdt(lww)", entity),
                            impact: "Enables user authentication and communication".to_string(),
                        });
                    }
                }
                "document_editing" => {
                    if !field_names.iter().any(|f| f.contains("version")) {
                        suggestions.push(Suggestion {
                            suggestion_type: SuggestionType::AddField,
                            priority: SuggestionPriority::Medium,
                            title: "Add version tracking".to_string(),
                            description: "Document editing benefits from version tracking"
                                .to_string(),
                            rationale: "Version numbers help with document history and rollback"
                                .to_string(),
                            code_example: format!(
                                "  {} has version: i32 @crdt(pn_counter)",
                                entity
                            ),
                            impact: "Enables version history and change tracking".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        if context.security_priority {
            if !field_names
                .iter()
                .any(|f| f.contains("owner") || f.contains("creator"))
            {
                suggestions.push(Suggestion {
                    suggestion_type: SuggestionType::AddField,
                    priority: SuggestionPriority::High,
                    title: "Add owner/creator field for security".to_string(),
                    description: "Track entity ownership for access control".to_string(),
                    rationale: "Owner tracking is essential for authorization and access control"
                        .to_string(),
                    code_example: format!("  {} has owner_id: String @crdt(immutable)", entity),
                    impact: "Enables ownership-based access control".to_string(),
                });
            }
        }
    }

    /// Extracts field names from Gen.
    fn extract_field_names(&self, gen: &Gen) -> Vec<String> {
        gen.statements
            .iter()
            .filter_map(|stmt| match stmt {
                Statement::Has { property, .. } => Some(property.clone()),
                Statement::HasField(has_field) => Some(has_field.name.clone()),
                _ => None,
            })
            .collect()
    }

    /// Formats type expression as string.
    fn format_type_expr(&self, type_expr: &crate::ast::TypeExpr) -> String {
        use crate::ast::TypeExpr;
        match type_expr {
            TypeExpr::Named(name) => name.clone(),
            TypeExpr::Generic { name, args } => {
                let args_str = args
                    .iter()
                    .map(|a| self.format_type_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args_str)
            }
            _ => "Unknown".to_string(),
        }
    }

    /// Generates exegesis template.
    fn generate_exegesis_template(&self, gen: &Gen) -> String {
        let entity = gen.name.split('.').next().unwrap_or("entity");
        format!(
            r#"
exegesis {{
  Schema for {} with CRDT-backed fields.

  Design decisions:
  - Fields use appropriate CRDT strategies for their access patterns
  - Immutable fields ensure consistency across replicas
  - Collaborative fields use conflict-free merge strategies

  Fields:
  [Document each field's purpose and CRDT choice]
}}
"#,
            entity
        )
    }

    /// Calculates overall schema health score.
    fn calculate_health_score(&self, gen: &Gen, suggestions: &[Suggestion]) -> u8 {
        let mut score = 100u8;

        // Deduct points for critical suggestions
        let critical_count = suggestions
            .iter()
            .filter(|s| s.priority == SuggestionPriority::Critical)
            .count();
        score = score.saturating_sub((critical_count * 20) as u8);

        // Deduct points for high priority suggestions
        let high_count = suggestions
            .iter()
            .filter(|s| s.priority == SuggestionPriority::High)
            .count();
        score = score.saturating_sub((high_count * 10) as u8);

        // Deduct points for medium priority suggestions
        let medium_count = suggestions
            .iter()
            .filter(|s| s.priority == SuggestionPriority::Medium)
            .count();
        score = score.saturating_sub((medium_count * 5) as u8);

        // Bonus for good practices
        let field_names: Vec<String> = self.extract_field_names(gen);
        if field_names.contains(&"id".to_string()) {
            score = score.saturating_add(5);
        }
        if !gen.exegesis.trim().is_empty() {
            score = score.saturating_add(5);
        }

        score
    }

    /// Generates summary of suggestions.
    fn generate_summary(&self, suggestions: &[Suggestion], health_score: u8) -> String {
        let critical = suggestions
            .iter()
            .filter(|s| s.priority == SuggestionPriority::Critical)
            .count();
        let high = suggestions
            .iter()
            .filter(|s| s.priority == SuggestionPriority::High)
            .count();
        let medium = suggestions
            .iter()
            .filter(|s| s.priority == SuggestionPriority::Medium)
            .count();
        let low = suggestions
            .iter()
            .filter(|s| s.priority == SuggestionPriority::Low)
            .count();

        format!(
            "Schema health: {}%. Found {} suggestions ({} critical, {} high, {} medium, {} low priority)",
            health_score, suggestions.len(), critical, high, medium, low
        )
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_id_field() {
        let engine = SuggestionEngine::new();
        let source = r#"
gen user.profile {
  user has name: String @crdt(lww)
}

exegesis {
  User profile.
}
"#;

        let suggestions = engine.analyze_and_suggest(source, SuggestionContext::default());
        let has_id_suggestion = suggestions
            .suggestions
            .iter()
            .any(|s| s.title.contains("identifier"));
        assert!(has_id_suggestion);
    }

    #[test]
    fn test_suggest_audit_fields() {
        let engine = SuggestionEngine::new();
        let source = r#"
gen document.schema {
  document has id: String @crdt(immutable)
  document has content: String @crdt(peritext)
}

exegesis {
  Document schema.
}
"#;

        let suggestions = engine.analyze_and_suggest(source, SuggestionContext::default());
        let has_timestamp_suggestion = suggestions.suggestions.iter().any(|s| {
            s.title.contains("timestamp")
                || s.title.contains("created")
                || s.title.contains("updated")
        });
        assert!(has_timestamp_suggestion);
    }

    #[test]
    fn test_health_score_calculation() {
        let engine = SuggestionEngine::new();

        // Good schema
        let good_source = r#"
gen document.schema {
  document has id: String @crdt(immutable)
  document has content: String @crdt(peritext)
  document has created_at: i64 @crdt(immutable)
  document has updated_at: i64 @crdt(lww)
}

exegesis {
  A well-documented collaborative document schema with proper CRDT strategies.
}
"#;

        let good_suggestions =
            engine.analyze_and_suggest(good_source, SuggestionContext::default());
        assert!(good_suggestions.health_score >= 80);

        // Poor schema
        let poor_source = r#"
gen document.schema {
  document has content: String @crdt(lww)
}
"#;

        let poor_suggestions =
            engine.analyze_and_suggest(poor_source, SuggestionContext::default());
        assert!(poor_suggestions.health_score < 70);
    }
}
