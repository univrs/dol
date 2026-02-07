//! Intelligent Code Completion for DOL
//!
//! This module provides context-aware code completion for DOL schemas,
//! including field suggestions, CRDT strategy auto-completion, and
//! type-aware completions with sub-100ms response times.
//!
//! # Overview
//!
//! The completion provider:
//! - Analyzes cursor context to determine completion type
//! - Suggests appropriate field names based on entity type
//! - Auto-completes CRDT strategies with rationale
//! - Provides type-aware completions for field types
//! - Optimized for <100ms response time
//!
//! # Example
//!
//! ```rust
//! use metadol::lsp::completion::{CompletionProvider, CompletionContext};
//!
//! let provider = CompletionProvider::new();
//! let source = "gen document.schema { document has ";
//! let completions = provider.provide_completions(source, source.len());
//!
//! // Returns field name suggestions like "title", "content", "author"
//! assert!(!completions.is_empty());
//! ```

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Completion item kind (following LSP specification).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompletionItemKind {
    /// Text completion
    Text,
    /// Method
    Method,
    /// Function
    Function,
    /// Field or property
    Field,
    /// Variable
    Variable,
    /// Class
    Class,
    /// Interface
    Interface,
    /// Module
    Module,
    /// Property
    Property,
    /// Keyword
    Keyword,
    /// Snippet
    Snippet,
    /// Enum
    Enum,
    /// EnumMember
    EnumMember,
}

/// A completion item.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompletionItem {
    /// Label shown in completion list
    pub label: String,
    /// Kind of completion
    pub kind: CompletionItemKind,
    /// Detailed information
    pub detail: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
    /// Text to insert
    pub insert_text: String,
    /// Sort priority (lower = higher priority)
    pub sort_text: Option<String>,
    /// Filter text for matching
    pub filter_text: Option<String>,
}

/// Context for determining what completions to provide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionContext {
    /// At top level, suggest keywords (gen, trait, rule, etc.)
    TopLevel,
    /// Inside gen/trait body, suggest statement types
    InsideBody,
    /// After "has", suggest field names
    FieldName,
    /// After field name and ":", suggest types
    FieldType,
    /// After "@crdt(", suggest strategies
    CrdtStrategy,
    /// After type, suggest @crdt annotation
    CrdtAnnotation,
    /// Inside exegesis block
    Exegesis,
    /// Unknown context
    Unknown,
}

/// Completion provider for DOL.
pub struct CompletionProvider {
    /// Common field names by entity type
    field_templates: HashMap<String, Vec<FieldTemplate>>,
    /// Common type completions
    type_completions: Vec<TypeCompletion>,
    /// CRDT strategy completions
    crdt_completions: Vec<CrdtStrategyCompletion>,
}

/// Field name template.
#[derive(Debug, Clone)]
struct FieldTemplate {
    name: String,
    field_type: String,
    crdt_strategy: String,
    description: String,
}

/// Type completion.
#[derive(Debug, Clone)]
pub struct TypeCompletion {
    /// Type name
    pub name: String,
    /// Description
    pub description: String,
    /// Suggested CRDT strategy
    pub suggested_crdt: String,
}

/// CRDT strategy completion.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrdtStrategyCompletion {
    /// Strategy name
    pub name: String,
    /// Short description
    pub description: String,
    /// Detailed documentation
    pub documentation: String,
    /// Best for types
    pub best_for: Vec<String>,
    /// Example usage
    pub example: String,
}

/// Field type completion.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldTypeCompletion {
    /// Type name
    pub name: String,
    /// Description
    pub description: String,
    /// Suggested CRDT strategy
    pub suggested_crdt: String,
}

impl CompletionProvider {
    /// Creates a new completion provider.
    pub fn new() -> Self {
        let mut field_templates = HashMap::new();

        // Document entity templates
        field_templates.insert(
            "document".to_string(),
            vec![
                FieldTemplate {
                    name: "id".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "immutable".to_string(),
                    description: "Unique document identifier".to_string(),
                },
                FieldTemplate {
                    name: "title".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "lww".to_string(),
                    description: "Document title".to_string(),
                },
                FieldTemplate {
                    name: "content".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "peritext".to_string(),
                    description: "Collaborative document content".to_string(),
                },
                FieldTemplate {
                    name: "author".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "immutable".to_string(),
                    description: "Document author".to_string(),
                },
                FieldTemplate {
                    name: "tags".to_string(),
                    field_type: "Set<String>".to_string(),
                    crdt_strategy: "or_set".to_string(),
                    description: "Document tags".to_string(),
                },
                FieldTemplate {
                    name: "created_at".to_string(),
                    field_type: "i64".to_string(),
                    crdt_strategy: "immutable".to_string(),
                    description: "Creation timestamp".to_string(),
                },
                FieldTemplate {
                    name: "updated_at".to_string(),
                    field_type: "i64".to_string(),
                    crdt_strategy: "lww".to_string(),
                    description: "Last update timestamp".to_string(),
                },
            ],
        );

        // User entity templates
        field_templates.insert(
            "user".to_string(),
            vec![
                FieldTemplate {
                    name: "id".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "immutable".to_string(),
                    description: "Unique user identifier".to_string(),
                },
                FieldTemplate {
                    name: "name".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "lww".to_string(),
                    description: "User name".to_string(),
                },
                FieldTemplate {
                    name: "email".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "lww".to_string(),
                    description: "User email address".to_string(),
                },
                FieldTemplate {
                    name: "role".to_string(),
                    field_type: "String".to_string(),
                    crdt_strategy: "lww".to_string(),
                    description: "User role".to_string(),
                },
                FieldTemplate {
                    name: "created_at".to_string(),
                    field_type: "i64".to_string(),
                    crdt_strategy: "immutable".to_string(),
                    description: "Account creation timestamp".to_string(),
                },
            ],
        );

        // Common types
        let type_completions = vec![
            TypeCompletion {
                name: "String".to_string(),
                description: "UTF-8 text".to_string(),
                suggested_crdt: "lww".to_string(),
            },
            TypeCompletion {
                name: "i32".to_string(),
                description: "32-bit signed integer".to_string(),
                suggested_crdt: "lww".to_string(),
            },
            TypeCompletion {
                name: "i64".to_string(),
                description: "64-bit signed integer".to_string(),
                suggested_crdt: "lww".to_string(),
            },
            TypeCompletion {
                name: "f64".to_string(),
                description: "64-bit floating point".to_string(),
                suggested_crdt: "lww".to_string(),
            },
            TypeCompletion {
                name: "bool".to_string(),
                description: "Boolean value".to_string(),
                suggested_crdt: "lww".to_string(),
            },
            TypeCompletion {
                name: "Set<String>".to_string(),
                description: "Set of unique strings".to_string(),
                suggested_crdt: "or_set".to_string(),
            },
            TypeCompletion {
                name: "Vec<String>".to_string(),
                description: "Ordered list of strings".to_string(),
                suggested_crdt: "rga".to_string(),
            },
            TypeCompletion {
                name: "Map<String, String>".to_string(),
                description: "Key-value map".to_string(),
                suggested_crdt: "lww".to_string(),
            },
        ];

        // CRDT strategies
        let crdt_completions = vec![
            CrdtStrategyCompletion {
                name: "immutable".to_string(),
                description: "Write-once, never changes".to_string(),
                documentation: "Immutable values that cannot be modified after creation. Best for IDs and creation timestamps.".to_string(),
                best_for: vec!["String (ID)".to_string(), "i64 (timestamps)".to_string()],
                example: "@crdt(immutable)".to_string(),
            },
            CrdtStrategyCompletion {
                name: "lww".to_string(),
                description: "Last-Write-Wins".to_string(),
                documentation: "Uses timestamps to resolve conflicts; last write wins. Best for simple fields with single writers.".to_string(),
                best_for: vec!["String".to_string(), "i32".to_string(), "bool".to_string()],
                example: "@crdt(lww)".to_string(),
            },
            CrdtStrategyCompletion {
                name: "peritext".to_string(),
                description: "Character-level collaborative text".to_string(),
                documentation: "Peritext CRDT for rich collaborative text editing. Preserves concurrent character-level edits.".to_string(),
                best_for: vec!["String (collaborative)".to_string()],
                example: "@crdt(peritext)".to_string(),
            },
            CrdtStrategyCompletion {
                name: "or_set".to_string(),
                description: "Observed-Remove Set".to_string(),
                documentation: "Set CRDT that supports concurrent add/remove operations. Elements can be re-added after removal.".to_string(),
                best_for: vec!["Set<T>".to_string()],
                example: "@crdt(or_set)".to_string(),
            },
            CrdtStrategyCompletion {
                name: "rga".to_string(),
                description: "Replicated Growable Array".to_string(),
                documentation: "List CRDT that preserves insertion order with concurrent edits. Best for ordered collections.".to_string(),
                best_for: vec!["Vec<T>".to_string(), "List<T>".to_string()],
                example: "@crdt(rga)".to_string(),
            },
            CrdtStrategyCompletion {
                name: "pn_counter".to_string(),
                description: "Positive-Negative Counter".to_string(),
                documentation: "Counter CRDT supporting increment and decrement. Converges to correct sum across replicas.".to_string(),
                best_for: vec!["i32".to_string(), "i64".to_string()],
                example: "@crdt(pn_counter)".to_string(),
            },
            CrdtStrategyCompletion {
                name: "mv_register".to_string(),
                description: "Multi-Value Register".to_string(),
                documentation: "Preserves all concurrent writes, returning multiple values. Application resolves conflicts.".to_string(),
                best_for: vec!["String".to_string(), "any type".to_string()],
                example: "@crdt(mv_register)".to_string(),
            },
        ];

        Self {
            field_templates,
            type_completions,
            crdt_completions,
        }
    }

    /// Provides completions at a given position.
    pub fn provide_completions(&self, source: &str, position: usize) -> Vec<CompletionItem> {
        let context = self.analyze_context(source, position);

        match context {
            CompletionContext::TopLevel => self.complete_keywords(),
            CompletionContext::InsideBody => self.complete_statement_keywords(),
            CompletionContext::FieldName => self.complete_field_names(source),
            CompletionContext::FieldType => self.complete_types(),
            CompletionContext::CrdtStrategy => self.complete_crdt_strategies(source),
            CompletionContext::CrdtAnnotation => self.complete_crdt_annotation(),
            CompletionContext::Exegesis => self.complete_exegesis(),
            CompletionContext::Unknown => vec![],
        }
    }

    /// Analyzes context to determine what kind of completions to provide.
    fn analyze_context(&self, source: &str, position: usize) -> CompletionContext {
        let before_cursor = &source[..position];
        let last_line = before_cursor.lines().last().unwrap_or("");

        // Check for CRDT strategy context
        if last_line.contains("@crdt(") && !last_line.contains(")") {
            return CompletionContext::CrdtStrategy;
        }

        // Check for type context (after ":")
        if last_line.contains(" has ") && last_line.contains(':') && !last_line.contains('@') {
            return CompletionContext::CrdtAnnotation;
        }

        // Check for field type context
        if last_line.contains(" has ") && last_line.contains(':') {
            return CompletionContext::FieldType;
        }

        // Check for field name context
        if last_line.contains(" has ") && !last_line.contains(':') {
            return CompletionContext::FieldName;
        }

        // Check if inside exegesis
        if before_cursor.contains("exegesis {") && !before_cursor.ends_with("}") {
            return CompletionContext::Exegesis;
        }

        // Check if inside body
        if before_cursor.contains("gen ") || before_cursor.contains("trait ") {
            if before_cursor.contains('{') && !before_cursor.ends_with("}") {
                return CompletionContext::InsideBody;
            }
        }

        // Top level
        if !before_cursor.contains('{') || before_cursor.ends_with("}") {
            return CompletionContext::TopLevel;
        }

        CompletionContext::Unknown
    }

    /// Completes top-level keywords.
    fn complete_keywords(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "gen".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Gen declaration".to_string()),
                documentation: Some("Define a gen with CRDT-backed fields".to_string()),
                insert_text: "gen ${1:entity}.${2:name} {\n  $0\n}\n\nexegesis {\n  \n}"
                    .to_string(),
                sort_text: Some("0".to_string()),
                filter_text: None,
            },
            CompletionItem {
                label: "trait".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Trait declaration".to_string()),
                documentation: Some("Define a composable trait".to_string()),
                insert_text: "trait ${1:name} {\n  $0\n}\n\nexegesis {\n  \n}".to_string(),
                sort_text: Some("1".to_string()),
                filter_text: None,
            },
            CompletionItem {
                label: "rule".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Rule (constraint) declaration".to_string()),
                documentation: Some("Define a validation rule".to_string()),
                insert_text: "rule ${1:name} {\n  $0\n}\n\nexegesis {\n  \n}".to_string(),
                sort_text: Some("2".to_string()),
                filter_text: None,
            },
        ]
    }

    /// Completes statement keywords inside body.
    fn complete_statement_keywords(&self) -> Vec<CompletionItem> {
        vec![CompletionItem {
            label: "has".to_string(),
            kind: CompletionItemKind::Keyword,
            detail: Some("Field declaration".to_string()),
            documentation: Some("Declare a field with type and CRDT strategy".to_string()),
            insert_text: "has ${1:field}: ${2:String} @crdt(${3:lww})".to_string(),
            sort_text: Some("0".to_string()),
            filter_text: None,
        }]
    }

    /// Completes field names based on entity type.
    fn complete_field_names(&self, source: &str) -> Vec<CompletionItem> {
        // Extract entity name from gen declaration
        let entity_name = self.extract_entity_name(source);

        let templates = if let Some(name) = entity_name {
            self.field_templates.get(&name)
        } else {
            None
        };

        let templates = templates.unwrap_or_else(|| {
            // Default templates
            self.field_templates.get("document").unwrap()
        });

        templates
            .iter()
            .enumerate()
            .map(|(i, template)| CompletionItem {
                label: template.name.clone(),
                kind: CompletionItemKind::Field,
                detail: Some(format!(
                    "{} ({})",
                    template.field_type, template.crdt_strategy
                )),
                documentation: Some(template.description.clone()),
                insert_text: format!(
                    "{}: {} @crdt({})",
                    template.name, template.field_type, template.crdt_strategy
                ),
                sort_text: Some(format!("{:02}", i)),
                filter_text: Some(template.name.clone()),
            })
            .collect()
    }

    /// Completes type names.
    fn complete_types(&self) -> Vec<CompletionItem> {
        self.type_completions
            .iter()
            .enumerate()
            .map(|(i, type_comp)| CompletionItem {
                label: type_comp.name.clone(),
                kind: CompletionItemKind::Class,
                detail: Some(type_comp.description.clone()),
                documentation: Some(format!("Suggested CRDT: {}", type_comp.suggested_crdt)),
                insert_text: type_comp.name.clone(),
                sort_text: Some(format!("{:02}", i)),
                filter_text: Some(type_comp.name.clone()),
            })
            .collect()
    }

    /// Completes CRDT strategies.
    fn complete_crdt_strategies(&self, _source: &str) -> Vec<CompletionItem> {
        self.crdt_completions
            .iter()
            .enumerate()
            .map(|(i, crdt)| CompletionItem {
                label: crdt.name.clone(),
                kind: CompletionItemKind::EnumMember,
                detail: Some(crdt.description.clone()),
                documentation: Some(format!(
                    "{}\n\nBest for: {}",
                    crdt.documentation,
                    crdt.best_for.join(", ")
                )),
                insert_text: crdt.name.clone(),
                sort_text: Some(format!("{:02}", i)),
                filter_text: Some(crdt.name.clone()),
            })
            .collect()
    }

    /// Completes @crdt annotation.
    fn complete_crdt_annotation(&self) -> Vec<CompletionItem> {
        vec![CompletionItem {
            label: "@crdt()".to_string(),
            kind: CompletionItemKind::Snippet,
            detail: Some("Add CRDT strategy annotation".to_string()),
            documentation: Some("Specify how this field merges across replicas".to_string()),
            insert_text: "@crdt(${1:lww})".to_string(),
            sort_text: Some("0".to_string()),
            filter_text: None,
        }]
    }

    /// Completes exegesis content.
    fn complete_exegesis(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "exegesis template".to_string(),
                kind: CompletionItemKind::Snippet,
                detail: Some("Exegesis documentation template".to_string()),
                documentation: Some("Standard exegesis structure".to_string()),
                insert_text: "Schema for ${1:entity}.\n\nDesign decisions:\n- ${2:decision}\n\nFields:\n- ${3:field}: ${4:description}".to_string(),
                sort_text: Some("0".to_string()),
                filter_text: None,
            },
        ]
    }

    /// Extracts entity name from gen declaration.
    fn extract_entity_name(&self, source: &str) -> Option<String> {
        // Look for "gen <entity>."
        if let Some(start) = source.find("gen ") {
            let after_gen = &source[start + 4..];
            if let Some(dot_pos) = after_gen.find('.') {
                let entity = after_gen[..dot_pos].trim();
                return Some(entity.to_lowercase());
            }
        }
        None
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_context_top_level() {
        let provider = CompletionProvider::new();
        let context = provider.analyze_context("", 0);
        assert_eq!(context, CompletionContext::TopLevel);
    }

    #[test]
    fn test_analyze_context_field_name() {
        let provider = CompletionProvider::new();
        let source = "gen document.schema { document has ";
        let context = provider.analyze_context(source, source.len());
        assert_eq!(context, CompletionContext::FieldName);
    }

    #[test]
    fn test_analyze_context_crdt_strategy() {
        let provider = CompletionProvider::new();
        let source = "gen doc.schema { doc has title: String @crdt(";
        let context = provider.analyze_context(source, source.len());
        assert_eq!(context, CompletionContext::CrdtStrategy);
    }

    #[test]
    fn test_complete_keywords() {
        let provider = CompletionProvider::new();
        let completions = provider.complete_keywords();
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "gen"));
    }

    #[test]
    fn test_complete_field_names() {
        let provider = CompletionProvider::new();
        let source = "gen document.schema { document has ";
        let completions = provider.complete_field_names(source);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "title"));
        assert!(completions.iter().any(|c| c.label == "content"));
    }

    #[test]
    fn test_complete_crdt_strategies() {
        let provider = CompletionProvider::new();
        let completions = provider.complete_crdt_strategies("");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "lww"));
        assert!(completions.iter().any(|c| c.label == "peritext"));
        assert!(completions.iter().any(|c| c.label == "or_set"));
    }

    #[test]
    fn test_extract_entity_name() {
        let provider = CompletionProvider::new();

        let entity = provider.extract_entity_name("gen document.schema {");
        assert_eq!(entity, Some("document".to_string()));

        let entity = provider.extract_entity_name("gen user.profile {");
        assert_eq!(entity, Some("user".to_string()));
    }

    #[test]
    fn test_provide_completions_performance() {
        use std::time::Instant;

        let provider = CompletionProvider::new();
        let source = "gen document.schema { document has ";

        let start = Instant::now();
        let _completions = provider.provide_completions(source, source.len());
        let duration = start.elapsed();

        // Should complete in under 100ms (target: < 10ms typically)
        assert!(
            duration.as_millis() < 100,
            "Completion took {:?}, expected < 100ms",
            duration
        );
    }
}
