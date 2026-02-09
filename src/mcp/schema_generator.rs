//! Schema Generator for DOL
//!
//! This module provides tools for generating complete DOL schemas from
//! high-level descriptions, including field inference, CRDT strategy
//! selection, and constraint generation.
//!
//! # Overview
//!
//! The schema generator:
//! - Infers field types from semantic descriptions
//! - Suggests appropriate CRDT strategies
//! - Generates validation constraints
//! - Produces complete, well-formed DOL definitions
//!
//! # Example
//!
//! ```rust
//! use metadol::mcp::schema_generator::{SchemaGenerator, FieldSpec};
//!
//! let generator = SchemaGenerator::new();
//! let spec = FieldSpec {
//!     name: "title".to_string(),
//!     description: "A collaborative document title".to_string(),
//!     required: true,
//!     unique: false,
//! };
//!
//! let field_def = generator.generate_field_definition(&spec);
//! ```

use super::recommendations::{ConsistencyLevel, CrdtRecommender, UsagePattern};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Field specification for schema generation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldSpec {
    /// Field name
    pub name: String,
    /// Semantic description of the field
    pub description: String,
    /// Whether the field is required
    pub required: bool,
    /// Whether the field must be unique
    pub unique: bool,
}

/// Generated field definition.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldDefinition {
    /// Field name
    pub name: String,
    /// Inferred type
    pub field_type: String,
    /// CRDT strategy
    pub crdt_strategy: String,
    /// Validation constraints
    pub constraints: Vec<String>,
    /// Documentation
    pub documentation: String,
}

/// Schema generation options.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GenerationOptions {
    /// Include audit fields (created_at, updated_at)
    pub include_audit_fields: bool,
    /// Include id field
    pub include_id_field: bool,
    /// Default consistency level
    pub default_consistency: ConsistencyLevel,
    /// Generate exegesis documentation
    pub generate_exegesis: bool,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            include_audit_fields: true,
            include_id_field: true,
            default_consistency: ConsistencyLevel::Eventual,
            generate_exegesis: true,
        }
    }
}

/// Schema generator for DOL.
pub struct SchemaGenerator {
    recommender: CrdtRecommender,
    type_mappings: HashMap<String, String>,
}

impl SchemaGenerator {
    /// Creates a new schema generator.
    pub fn new() -> Self {
        let mut type_mappings = HashMap::new();

        // Common semantic → type mappings
        type_mappings.insert("text".to_string(), "String".to_string());
        type_mappings.insert("string".to_string(), "String".to_string());
        type_mappings.insert("title".to_string(), "String".to_string());
        type_mappings.insert("name".to_string(), "String".to_string());
        type_mappings.insert("description".to_string(), "String".to_string());
        type_mappings.insert("content".to_string(), "String".to_string());
        type_mappings.insert("number".to_string(), "i32".to_string());
        type_mappings.insert("integer".to_string(), "i32".to_string());
        type_mappings.insert("count".to_string(), "i32".to_string());
        type_mappings.insert("counter".to_string(), "i32".to_string());
        type_mappings.insert("float".to_string(), "f64".to_string());
        type_mappings.insert("decimal".to_string(), "f64".to_string());
        type_mappings.insert("boolean".to_string(), "bool".to_string());
        type_mappings.insert("flag".to_string(), "bool".to_string());
        type_mappings.insert("timestamp".to_string(), "i64".to_string());
        type_mappings.insert("time".to_string(), "i64".to_string());
        type_mappings.insert("date".to_string(), "i64".to_string());
        type_mappings.insert("list".to_string(), "Vec<String>".to_string());
        type_mappings.insert("array".to_string(), "Vec<String>".to_string());
        type_mappings.insert("set".to_string(), "Set<String>".to_string());
        type_mappings.insert("collection".to_string(), "Set<String>".to_string());
        type_mappings.insert("tags".to_string(), "Set<String>".to_string());
        type_mappings.insert("labels".to_string(), "Set<String>".to_string());
        type_mappings.insert("id".to_string(), "String".to_string());
        type_mappings.insert("identifier".to_string(), "String".to_string());
        type_mappings.insert("uuid".to_string(), "String".to_string());

        Self {
            recommender: CrdtRecommender::new(),
            type_mappings,
        }
    }

    /// Generates a complete schema from specifications.
    pub fn generate_schema(
        &self,
        entity_name: &str,
        fields: Vec<FieldSpec>,
        options: GenerationOptions,
    ) -> Result<String, String> {
        let mut field_defs = Vec::new();

        // Generate ID field if requested
        if options.include_id_field {
            field_defs.push(FieldDefinition {
                name: "id".to_string(),
                field_type: "String".to_string(),
                crdt_strategy: "immutable".to_string(),
                constraints: vec!["required".to_string(), "unique".to_string()],
                documentation: "Unique identifier for the entity".to_string(),
            });
        }

        // Generate user-specified fields
        for spec in fields {
            let field_def = self.generate_field_definition(&spec);
            field_defs.push(field_def);
        }

        // Generate audit fields if requested
        if options.include_audit_fields {
            field_defs.push(FieldDefinition {
                name: "created_at".to_string(),
                field_type: "i64".to_string(),
                crdt_strategy: "immutable".to_string(),
                constraints: vec!["required".to_string()],
                documentation: "Timestamp when the entity was created".to_string(),
            });

            field_defs.push(FieldDefinition {
                name: "updated_at".to_string(),
                field_type: "i64".to_string(),
                crdt_strategy: "lww".to_string(),
                constraints: vec!["required".to_string()],
                documentation: "Timestamp when the entity was last updated".to_string(),
            });
        }

        // Build DOL source
        self.build_dol_source(entity_name, &field_defs, &options)
    }

    /// Generates a field definition from a specification.
    pub fn generate_field_definition(&self, spec: &FieldSpec) -> FieldDefinition {
        // Infer type from description and name
        let field_type = self.infer_type(&spec.name, &spec.description);

        // Infer usage pattern from description
        let usage_pattern = self.infer_usage_pattern(&spec.description);

        // Determine consistency level
        let consistency = if spec.unique {
            ConsistencyLevel::Strong
        } else {
            ConsistencyLevel::Eventual
        };

        // Get CRDT recommendation
        let recommendation =
            self.recommender
                .recommend(&spec.name, &field_type, usage_pattern, consistency);

        // Generate constraints
        let mut constraints = Vec::new();
        if spec.required {
            constraints.push("required".to_string());
        }
        if spec.unique {
            constraints.push("unique".to_string());
        }

        // Add type-specific constraints
        if field_type == "String" {
            constraints.push("non_empty".to_string());
        }

        FieldDefinition {
            name: spec.name.clone(),
            field_type,
            crdt_strategy: recommendation.recommended_strategy,
            constraints,
            documentation: spec.description.clone(),
        }
    }

    /// Infers field type from name and description.
    fn infer_type(&self, name: &str, description: &str) -> String {
        let lower_name = name.to_lowercase();
        let lower_desc = description.to_lowercase();

        // Check name first
        for (keyword, type_name) in &self.type_mappings {
            if lower_name.contains(keyword) {
                return type_name.clone();
            }
        }

        // Check description
        for (keyword, type_name) in &self.type_mappings {
            if lower_desc.contains(keyword) {
                return type_name.clone();
            }
        }

        // Default to String
        "String".to_string()
    }

    /// Infers usage pattern from description.
    fn infer_usage_pattern(&self, description: &str) -> UsagePattern {
        let lower = description.to_lowercase();

        if lower.contains("collaborative")
            || lower.contains("edit together")
            || lower.contains("real-time")
        {
            UsagePattern::CollaborativeText
        } else if lower.contains("counter")
            || lower.contains("increment")
            || lower.contains("decrement")
        {
            UsagePattern::Counter
        } else if lower.contains("set")
            || lower.contains("collection")
            || lower.contains("unique items")
        {
            UsagePattern::MultiUserSet
        } else if lower.contains("list") || lower.contains("ordered") || lower.contains("sequence")
        {
            UsagePattern::OrderedList
        } else if lower.contains("immutable")
            || lower.contains("write once")
            || lower.contains("read-only")
        {
            UsagePattern::WriteOnce
        } else {
            UsagePattern::LastWriteWins
        }
    }

    /// Builds DOL source code from field definitions.
    fn build_dol_source(
        &self,
        entity_name: &str,
        fields: &[FieldDefinition],
        options: &GenerationOptions,
    ) -> Result<String, String> {
        let mut source = String::new();
        let entity_lower = entity_name.to_lowercase();

        // Gen declaration
        source.push_str(&format!("gen {}.schema {{\n", entity_lower));

        // Fields
        for field in fields {
            let constraints_str = if field.constraints.is_empty() {
                String::new()
            } else {
                format!(" // {}", field.constraints.join(", "))
            };

            source.push_str(&format!(
                "  {} has {}: {} @crdt({}){}\n",
                entity_lower, field.name, field.field_type, field.crdt_strategy, constraints_str
            ));
        }

        source.push_str("}\n\n");

        // Exegesis
        if options.generate_exegesis {
            source.push_str("exegesis {\n");
            source.push_str(&format!("  CRDT-backed schema for {}.\n\n", entity_name));

            source.push_str("  Fields:\n");
            for field in fields {
                source.push_str(&format!(
                    "  - {}: {} ({})\n",
                    field.name, field.field_type, field.crdt_strategy
                ));
                if !field.documentation.is_empty() {
                    source.push_str(&format!("    {}\n", field.documentation));
                }
            }

            source.push_str("}\n");
        }

        Ok(source)
    }
}

impl Default for SchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_type() {
        let generator = SchemaGenerator::new();

        assert_eq!(generator.infer_type("title", ""), "String");
        assert_eq!(generator.infer_type("count", ""), "i32");
        assert_eq!(generator.infer_type("price", "a decimal value"), "f64");
        assert_eq!(generator.infer_type("tags", ""), "Set<String>");
    }

    #[test]
    fn test_infer_usage_pattern() {
        let generator = SchemaGenerator::new();

        let pattern = generator.infer_usage_pattern("collaborative editing");
        assert!(matches!(pattern, UsagePattern::CollaborativeText));

        let pattern = generator.infer_usage_pattern("counter value");
        assert!(matches!(pattern, UsagePattern::Counter));

        let pattern = generator.infer_usage_pattern("unique items in a set");
        assert!(matches!(pattern, UsagePattern::MultiUserSet));
    }

    #[test]
    fn test_generate_field_definition() {
        let generator = SchemaGenerator::new();
        let spec = FieldSpec {
            name: "title".to_string(),
            description: "Document title".to_string(),
            required: true,
            unique: false,
        };

        let field = generator.generate_field_definition(&spec);
        assert_eq!(field.name, "title");
        assert_eq!(field.field_type, "String");
        assert!(field.constraints.contains(&"required".to_string()));
    }

    #[test]
    fn test_generate_schema() {
        let generator = SchemaGenerator::new();
        let fields = vec![
            FieldSpec {
                name: "title".to_string(),
                description: "Document title".to_string(),
                required: true,
                unique: false,
            },
            FieldSpec {
                name: "content".to_string(),
                description: "Collaborative document content".to_string(),
                required: true,
                unique: false,
            },
        ];

        let result = generator.generate_schema("Document", fields, GenerationOptions::default());

        assert!(result.is_ok());
        let source = result.unwrap();
        assert!(source.contains("gen document.schema"));
        assert!(source.contains("has title"));
        assert!(source.contains("has content"));
        assert!(source.contains("exegesis"));
    }
}
