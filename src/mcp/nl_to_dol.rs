//! Natural Language to DOL Translation
//!
//! This module provides AI-powered translation from natural language
//! requirements to well-formed DOL schema definitions.
//!
//! # Overview
//!
//! The NL-to-DOL system:
//! - Parses natural language requirements
//! - Identifies entities, properties, and relationships
//! - Suggests appropriate CRDT strategies based on usage patterns
//! - Generates complete Gen definitions with constraints
//!
//! # Example
//!
//! ```rust
//! use metadol::mcp::nl_to_dol::{NlToDolConverter, NlRequirement};
//!
//! let converter = NlToDolConverter::new();
//! let requirement = NlRequirement {
//!     description: "A collaborative document with a title that users can edit together".to_string(),
//!     entity_name: Some("Document".to_string()),
//!     constraints: vec![],
//! };
//!
//! let result = converter.convert(requirement);
//! assert!(result.is_ok());
//! ```

use super::recommendations::{ConsistencyLevel, CrdtRecommender, UsagePattern};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Natural language requirement for DOL schema generation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NlRequirement {
    /// Natural language description of the requirement
    pub description: String,
    /// Optional entity name (inferred if not provided)
    pub entity_name: Option<String>,
    /// Additional constraints or rules
    pub constraints: Vec<String>,
}

/// Extracted field information from natural language.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExtractedField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: String,
    /// Suggested CRDT strategy
    pub crdt_strategy: String,
    /// Confidence in the suggestion
    pub confidence: String,
    /// Rationale for the suggestion
    pub rationale: String,
}

/// Generated DOL schema from natural language.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeneratedSchema {
    /// Entity name
    pub entity_name: String,
    /// Generated DOL source code
    pub dol_source: String,
    /// Extracted fields with CRDT strategies
    pub fields: Vec<ExtractedField>,
    /// Additional constraints or rules
    pub constraints: Vec<String>,
    /// Generation metadata
    pub metadata: SchemaMetadata,
}

/// Metadata about schema generation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SchemaMetadata {
    /// Confidence level (0-100)
    pub confidence: u8,
    /// Warnings or notes
    pub warnings: Vec<String>,
    /// Suggested improvements
    pub suggestions: Vec<String>,
}

/// Natural language to DOL converter.
pub struct NlToDolConverter {
    recommender: CrdtRecommender,
    patterns: HashMap<String, (UsagePattern, ConsistencyLevel)>,
}

impl NlToDolConverter {
    /// Creates a new NL-to-DOL converter.
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Populate common patterns
        patterns.insert(
            "collaborative".to_string(),
            (UsagePattern::CollaborativeText, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "edit together".to_string(),
            (UsagePattern::CollaborativeText, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "real-time editing".to_string(),
            (UsagePattern::CollaborativeText, ConsistencyLevel::Causal),
        );
        patterns.insert(
            "counter".to_string(),
            (UsagePattern::Counter, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "increment".to_string(),
            (UsagePattern::Counter, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "collection".to_string(),
            (UsagePattern::MultiUserSet, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "set of".to_string(),
            (UsagePattern::MultiUserSet, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "list of".to_string(),
            (UsagePattern::OrderedList, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "ordered".to_string(),
            (UsagePattern::OrderedList, ConsistencyLevel::Eventual),
        );
        patterns.insert(
            "unique".to_string(),
            (UsagePattern::WriteOnce, ConsistencyLevel::Strong),
        );
        patterns.insert(
            "immutable".to_string(),
            (UsagePattern::WriteOnce, ConsistencyLevel::Strong),
        );

        Self {
            recommender: CrdtRecommender::new(),
            patterns,
        }
    }

    /// Converts a natural language requirement to DOL schema.
    pub fn convert(&self, requirement: NlRequirement) -> Result<GeneratedSchema, String> {
        // Extract entity name
        let entity_name = requirement
            .entity_name
            .clone()
            .or_else(|| self.extract_entity_name(&requirement.description))
            .ok_or("Could not determine entity name")?;

        // Extract fields from description
        let fields = self.extract_fields(&requirement.description, &entity_name)?;

        // Generate DOL source
        let dol_source = self.generate_dol_source(&entity_name, &fields, &requirement.constraints);

        // Calculate confidence and generate metadata
        let metadata = self.generate_metadata(&fields, &requirement);

        Ok(GeneratedSchema {
            entity_name,
            dol_source,
            fields,
            constraints: requirement.constraints.clone(),
            metadata,
        })
    }

    /// Extracts entity name from description.
    fn extract_entity_name(&self, description: &str) -> Option<String> {
        // Look for common patterns: "A <entity>", "An <entity>", "The <entity>"
        let words: Vec<&str> = description.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            if matches!(word.to_lowercase().as_str(), "a" | "an" | "the") && i + 1 < words.len() {
                let next = words[i + 1];
                // Capitalize first letter
                if let Some(first) = next.chars().next() {
                    return Some(format!("{}{}", first.to_uppercase(), &next[1..]));
                }
            }
        }

        None
    }

    /// Extracts fields from description.
    fn extract_fields(
        &self,
        description: &str,
        _entity: &str,
    ) -> Result<Vec<ExtractedField>, String> {
        let mut fields = Vec::new();
        let lower_desc = description.to_lowercase();

        // Common field patterns (field_name, field_type, keywords)
        let field_patterns: Vec<(&str, &str, Vec<&str>)> = vec![
            ("title", "String", vec!["title", "name", "heading"]),
            (
                "content",
                "String",
                vec!["content", "text", "body", "description"],
            ),
            ("author", "String", vec!["author", "creator", "user"]),
            ("created_at", "i64", vec!["created", "timestamp", "time"]),
            ("updated_at", "i64", vec!["updated", "modified", "changed"]),
            ("tags", "Set<String>", vec!["tags", "labels", "categories"]),
            ("status", "String", vec!["status", "state"]),
            ("count", "i32", vec!["count", "counter", "number"]),
            ("items", "Vec<String>", vec!["items", "list", "elements"]),
            (
                "members",
                "Set<String>",
                vec!["members", "users", "participants"],
            ),
        ];

        // Detect fields based on patterns
        for (field_name, field_type, keywords) in field_patterns {
            if keywords.iter().any(|k| lower_desc.contains(k)) {
                // Determine usage pattern and consistency level
                let (usage, consistency) = self.detect_usage_pattern(&lower_desc, field_name);

                // Get CRDT recommendation
                let recommendation =
                    self.recommender
                        .recommend(field_name, field_type, usage, consistency);

                fields.push(ExtractedField {
                    name: field_name.to_string(),
                    field_type: field_type.to_string(),
                    crdt_strategy: recommendation.recommended_strategy.clone(),
                    confidence: format!("{:?}", recommendation.confidence),
                    rationale: recommendation.reasoning.clone(),
                });
            }
        }

        // Ensure at least one field
        if fields.is_empty() {
            // Add a default 'id' field
            fields.push(ExtractedField {
                name: "id".to_string(),
                field_type: "String".to_string(),
                crdt_strategy: "immutable".to_string(),
                confidence: "High".to_string(),
                rationale: "Unique identifier for the entity".to_string(),
            });
        }

        Ok(fields)
    }

    /// Detects usage pattern from description.
    fn detect_usage_pattern(
        &self,
        description: &str,
        field_name: &str,
    ) -> (UsagePattern, ConsistencyLevel) {
        // Check for explicit patterns in description
        for (pattern, &(usage, consistency)) in &self.patterns {
            if description.contains(pattern) {
                return (usage, consistency);
            }
        }

        // Field-specific defaults
        match field_name {
            "title" | "content"
                if description.contains("collaborative") || description.contains("together") =>
            {
                (UsagePattern::CollaborativeText, ConsistencyLevel::Eventual)
            }
            "count" | "counter" => (UsagePattern::Counter, ConsistencyLevel::Eventual),
            "tags" | "labels" | "members" => {
                (UsagePattern::MultiUserSet, ConsistencyLevel::Eventual)
            }
            "items" | "list" => (UsagePattern::OrderedList, ConsistencyLevel::Eventual),
            "id" | "created_at" => (UsagePattern::WriteOnce, ConsistencyLevel::Strong),
            _ => (UsagePattern::LastWriteWins, ConsistencyLevel::Eventual),
        }
    }

    /// Generates DOL source code.
    fn generate_dol_source(
        &self,
        entity: &str,
        fields: &[ExtractedField],
        constraints: &[String],
    ) -> String {
        let mut source = String::new();

        // Gen declaration
        source.push_str(&format!("gen {}.schema {{\n", entity.to_lowercase()));

        // Fields
        for field in fields {
            source.push_str(&format!(
                "  {} has {}: {} @crdt({})\n",
                entity.to_lowercase(),
                field.name,
                field.field_type,
                field.crdt_strategy
            ));
        }

        // Constraints
        for constraint in constraints {
            source.push_str(&format!("  {}\n", constraint));
        }

        source.push_str("}\n\n");

        // Exegesis
        source.push_str("exegesis {\n");
        source.push_str(&format!(
            "  Schema for {} with CRDT-backed fields.\n",
            entity
        ));
        source.push_str("  \n");
        source.push_str("  Fields:\n");
        for field in fields {
            source.push_str(&format!(
                "  - {}: {} ({})\n",
                field.name, field.field_type, field.crdt_strategy
            ));
            source.push_str(&format!("    {}\n", field.rationale));
        }
        source.push_str("}\n");

        source
    }

    /// Generates metadata about the schema.
    fn generate_metadata(
        &self,
        fields: &[ExtractedField],
        requirement: &NlRequirement,
    ) -> SchemaMetadata {
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Calculate average confidence
        let confidence_scores: Vec<u8> = fields
            .iter()
            .map(|f| match f.confidence.as_str() {
                "High" => 90,
                "Medium" => 70,
                "Low" => 50,
                _ => 60,
            })
            .collect();

        let avg_confidence = if confidence_scores.is_empty() {
            50
        } else {
            let sum: u32 = confidence_scores.iter().map(|&s| s as u32).sum();
            (sum / confidence_scores.len() as u32) as u8
        };

        // Check for potential issues
        if fields.len() < 2 {
            warnings
                .push("Schema has very few fields. Consider adding more properties.".to_string());
        }

        let has_id = fields.iter().any(|f| f.name == "id");
        if !has_id {
            suggestions
                .push("Consider adding an 'id' field for unique identification.".to_string());
        }

        let has_timestamp = fields
            .iter()
            .any(|f| f.name.contains("created") || f.name.contains("updated"));
        if !has_timestamp {
            suggestions.push(
                "Consider adding timestamp fields (created_at, updated_at) for audit trails."
                    .to_string(),
            );
        }

        // Check for collaborative fields without proper CRDT
        for field in fields {
            if field.field_type == "String" && field.crdt_strategy == "lww" {
                if requirement
                    .description
                    .to_lowercase()
                    .contains("collaborative")
                {
                    suggestions.push(format!(
                        "Field '{}' uses LWW but collaborative editing detected. Consider 'peritext' strategy.",
                        field.name
                    ));
                }
            }
        }

        SchemaMetadata {
            confidence: avg_confidence,
            warnings,
            suggestions,
        }
    }
}

impl Default for NlToDolConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_entity_name() {
        let converter = NlToDolConverter::new();

        let name = converter.extract_entity_name("A collaborative document with a title");
        assert_eq!(name, Some("Collaborative".to_string()));

        let name = converter.extract_entity_name("The user profile has fields");
        assert_eq!(name, Some("User".to_string()));
    }

    #[test]
    fn test_convert_simple_requirement() {
        let converter = NlToDolConverter::new();
        let requirement = NlRequirement {
            description: "A document with a title".to_string(),
            entity_name: Some("Document".to_string()),
            constraints: vec![],
        };

        let result = converter.convert(requirement);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.entity_name, "Document");
        assert!(!schema.fields.is_empty());
        assert!(schema.dol_source.contains("gen document.schema"));
    }

    #[test]
    fn test_convert_collaborative_document() {
        let converter = NlToDolConverter::new();
        let requirement = NlRequirement {
            description:
                "A collaborative document with a title and content that users can edit together"
                    .to_string(),
            entity_name: Some("Document".to_string()),
            constraints: vec![],
        };

        let result = converter.convert(requirement);
        assert!(result.is_ok());

        let schema = result.unwrap();
        let content_field = schema.fields.iter().find(|f| f.name == "content");
        assert!(content_field.is_some());

        // Should recommend peritext for collaborative text
        if let Some(field) = content_field {
            assert_eq!(field.crdt_strategy, "peritext");
        }
    }

    #[test]
    fn test_detect_usage_pattern() {
        let converter = NlToDolConverter::new();

        let (usage, _) = converter.detect_usage_pattern("collaborative editing", "content");
        assert!(matches!(usage, UsagePattern::CollaborativeText));

        let (usage, _) = converter.detect_usage_pattern("counter value", "count");
        assert!(matches!(usage, UsagePattern::Counter));

        let (usage, _) = converter.detect_usage_pattern("set of tags", "tags");
        assert!(matches!(usage, UsagePattern::MultiUserSet));
    }
}
