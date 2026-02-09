//! JSON Schema Code Generation
//!
//! This module generates JSON Schema definitions from DOL declarations.
//! Features:
//! - Schema generation from Gen declarations
//! - Full JSON Schema Draft 2020-12 support
//! - Validation rules from CRDT annotations
//! - GDPR annotations for personal data

use crate::{CodegenContext, Result};
use dol::ast::{Declaration, DolFile, Gen, HasField, Statement, TypeExpr};
use heck::ToSnakeCase;
use serde_json::{json, Map, Value};

pub const GENERATOR_NAME: &str = "JSON Schema";

/// Generate JSON Schema from a DOL file
pub fn generate(file: &DolFile, context: &CodegenContext) -> Result<String> {
    let mut schemas = Map::new();

    // Generate schema for each declaration
    for decl in &file.declarations {
        if let Some((name, schema)) = generate_declaration(decl, context)? {
            schemas.insert(name, schema);
        }
    }

    // Create root schema
    let root = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://example.com/{}.schema.json",
                       context.module_name.as_deref().unwrap_or("generated")),
        "title": context.module_name.as_deref().unwrap_or("Generated Schema"),
        "description": "Generated from DOL",
        "type": "object",
        "properties": schemas,
    });

    // Pretty print JSON
    serde_json::to_string_pretty(&root).map_err(Into::into)
}

/// Generate schema for a declaration
fn generate_declaration(
    decl: &Declaration,
    context: &CodegenContext,
) -> Result<Option<(String, Value)>> {
    match decl {
        Declaration::Gene(gen) => {
            let schema = generate_gen_schema(gen, context)?;
            Ok(Some((gen.name.clone(), schema)))
        }
        Declaration::Trait(trait_) => {
            // Generate a simple object schema for traits
            let schema = json!({
                "type": "object",
                "description": trait_.exegesis,
            });
            Ok(Some((trait_.name.clone(), schema)))
        }
        _ => Ok(None),
    }
}

/// Generate JSON Schema for a Gen declaration
fn generate_gen_schema(gen: &Gen, context: &CodegenContext) -> Result<Value> {
    let mut properties = Map::new();
    let mut required = Vec::new();

    // Process fields
    for stmt in &gen.statements {
        if let Statement::HasField(field) = stmt {
            let field_name = field.name.to_snake_case();
            let field_schema = generate_field_schema(field.as_ref())?;

            properties.insert(field_name.clone(), field_schema);

            // Add to required if no default value
            if field.default.is_none() {
                required.push(field_name);
            }
        }
    }

    let mut schema = json!({
        "type": "object",
        "properties": properties,
    });

    // Add description if available
    if context.include_docs && !gen.exegesis.is_empty() {
        schema["description"] = json!(gen.exegesis);
    }

    // Add required fields
    if !required.is_empty() {
        schema["required"] = json!(required);
    }

    Ok(schema)
}

/// Generate JSON Schema for a field
fn generate_field_schema(field: &HasField) -> Result<Value> {
    let mut schema = map_type_to_schema(&field.type_);

    // Add CRDT metadata
    if let Some(crdt) = &field.crdt_annotation {
        let mut metadata = Map::new();
        metadata.insert("strategy".to_string(), json!(crdt.strategy.as_str()));

        // Add CRDT options
        if !crdt.options.is_empty() {
            let mut options = Map::new();
            for opt in &crdt.options {
                options.insert(opt.key.clone(), json!(format!("{:?}", opt.value)));
            }
            metadata.insert("options".to_string(), json!(options));
        }

        schema["x-crdt"] = json!(metadata);
    }

    // Add personal data annotation
    if field.personal {
        schema["x-personal-data"] = json!(true);
        schema["x-gdpr"] = json!({
            "category": "personal",
            "retention": "user-controlled",
        });
    }

    // Add default value
    if let Some(_default) = &field.default {
        schema["default"] = json!(null); // TODO: Evaluate default expression
    }

    Ok(schema)
}

/// Map DOL type to JSON Schema type
fn map_type_to_schema(type_expr: &TypeExpr) -> Value {
    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            "String" => json!({
                "type": "string"
            }),
            "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "Int32" | "Int64" => json!({
                "type": "integer"
            }),
            "f32" | "f64" | "Float32" | "Float64" => json!({
                "type": "number"
            }),
            "bool" | "Bool" => json!({
                "type": "boolean"
            }),
            other => json!({
                "$ref": format!("#/$defs/{}", other)
            }),
        },
        TypeExpr::Generic { name, args } => match name.as_str() {
            "Vec" | "List" => {
                let item_schema = if let Some(first_arg) = args.first() {
                    map_type_to_schema(first_arg)
                } else {
                    json!({})
                };
                json!({
                    "type": "array",
                    "items": item_schema
                })
            }
            "Option" => {
                let inner_schema = if let Some(first_arg) = args.first() {
                    map_type_to_schema(first_arg)
                } else {
                    json!({})
                };
                json!({
                    "anyOf": [
                        inner_schema,
                        { "type": "null" }
                    ]
                })
            }
            "Set" | "HashSet" => {
                let item_schema = if let Some(first_arg) = args.first() {
                    map_type_to_schema(first_arg)
                } else {
                    json!({})
                };
                json!({
                    "type": "array",
                    "items": item_schema,
                    "uniqueItems": true
                })
            }
            "Map" | "HashMap" => json!({
                "type": "object",
                "additionalProperties": true
            }),
            _ => json!({
                "type": "object"
            }),
        },
        TypeExpr::Tuple(types) => {
            let schemas: Vec<Value> = types.iter().map(map_type_to_schema).collect();
            json!({
                "type": "array",
                "prefixItems": schemas,
                "minItems": schemas.len(),
                "maxItems": schemas.len()
            })
        }
        TypeExpr::Never => json!({
            "not": {}
        }),
        TypeExpr::Enum { variants } => {
            let enum_values: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
            json!({
                "enum": enum_values
            })
        }
        _ => json!({
            "type": "object"
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dol::ast::{Span, Visibility};

    #[test]
    fn test_type_mapping() {
        let string_type = TypeExpr::Named("String".to_string());
        let schema = map_type_to_schema(&string_type);
        assert_eq!(schema["type"], "string");

        let vec_type = TypeExpr::Generic {
            name: "Vec".to_string(),
            args: vec![TypeExpr::Named("i32".to_string())],
        };
        let schema = map_type_to_schema(&vec_type);
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "integer");
    }

    #[test]
    fn test_generate_simple_gen() {
        let gen = Gen {
            visibility: Visibility::Public,
            name: "Point".to_string(),
            extends: None,
            statements: vec![],
            exegesis: "A 2D point".to_string(),
            span: Span::default(),
        };

        let file = DolFile {
            module: None,
            uses: vec![],
            declarations: vec![Declaration::Gene(gen)],
        };

        let context = CodegenContext::new(crate::Target::JsonSchema);
        let code = generate(&file, &context).unwrap();

        assert!(code.contains("json-schema.org"));
        assert!(code.contains("A 2D point"));
        assert!(code.contains("Point"));
    }
}
