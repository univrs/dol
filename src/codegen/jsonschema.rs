//! JSON Schema code generation from Metal DOL declarations.
//!
//! Generates JSON Schema (draft-07) from DOL declarations for validation
//! and documentation purposes.
//!
//! # Type Mapping
//!
//! | DOL Type | JSON Schema Type |
//! |----------|------------------|
//! | `Int8`, `Int16`, `Int32`, `Int64` | `{ "type": "integer" }` |
//! | `UInt8`, `UInt16`, `UInt32`, `UInt64` | `{ "type": "integer", "minimum": 0 }` |
//! | `Float32`, `Float64` | `{ "type": "number" }` |
//! | `String` | `{ "type": "string" }` |
//! | `Bool` | `{ "type": "boolean" }` |
//! | `Void` | `{ "type": "null" }` |
//! | `Option<T>` | `{ "oneOf": [T, { "type": "null" }] }` |
//! | `List<T>` | `{ "type": "array", "items": T }` |
//! | `Map<K, V>` | `{ "type": "object", "additionalProperties": V }` |
//! | `Tuple(A, B)` | `{ "type": "array", "items": [A, B], "minItems": 2, "maxItems": 2 }` |
//!
//! # Example
//!
//! ```rust
//! use metadol::{parse_file, codegen::JsonSchemaCodegen};
//!
//! let source = r#"
//! gene container.exists {
//!   container has id
//!   container has image
//! }
//!
//! exegesis {
//!   A container is the fundamental unit.
//! }
//! "#;
//!
//! let decl = parse_file(source).unwrap();
//! let schema = JsonSchemaCodegen::generate(&decl);
//! println!("{}", schema);
//! ```

use crate::ast::{Declaration, Evo, Gen, Rule, Statement, System, Trait, TypeExpr};
use crate::typechecker::Type;

use super::{to_pascal_case, CodegenOptions, TypeMapper};

/// JSON Schema code generator.
///
/// Transforms DOL declarations into JSON Schema (draft-07) definitions.
#[derive(Debug, Clone, Default)]
pub struct JsonSchemaCodegen {
    /// Configuration options for code generation.
    #[allow(dead_code)]
    options: CodegenOptions,
}

impl JsonSchemaCodegen {
    /// Create a new JSON Schema code generator with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new JSON Schema code generator with custom options.
    pub fn with_options(options: CodegenOptions) -> Self {
        Self { options }
    }

    /// Generate JSON Schema from a declaration.
    pub fn generate(decl: &Declaration) -> String {
        Self::new().generate_declaration(decl)
    }

    /// Generate JSON Schema from multiple declarations.
    ///
    /// Returns a schema with `$defs` containing all declarations.
    pub fn generate_all(decls: &[Declaration]) -> String {
        let generator = Self::new();
        let mut defs = Vec::new();

        for decl in decls {
            let name = match decl {
                Declaration::Gene(g) => to_pascal_case(&g.name),
                Declaration::Trait(t) => to_pascal_case(&t.name),
                Declaration::Constraint(c) => to_pascal_case(&c.name),
                Declaration::System(s) => to_pascal_case(&s.name),
                Declaration::Evolution(e) => to_pascal_case(&e.name),
                Declaration::Function(f) => to_pascal_case(&f.name),
                Declaration::Const(c) => to_pascal_case(&c.name),
                Declaration::SexVar(v) => to_pascal_case(&v.name),
            };
            let schema = generator.generate_declaration_inner(decl);
            defs.push(format!("    \"{}\": {}", name, schema));
        }

        format!(
            r#"{{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$defs": {{
{}
  }}
}}"#,
            defs.join(",\n")
        )
    }

    /// Generate schema for a single declaration.
    fn generate_declaration(&self, decl: &Declaration) -> String {
        let title = match decl {
            Declaration::Gene(g) => to_pascal_case(&g.name),
            Declaration::Trait(t) => to_pascal_case(&t.name),
            Declaration::Constraint(c) => to_pascal_case(&c.name),
            Declaration::System(s) => to_pascal_case(&s.name),
            Declaration::Evolution(e) => to_pascal_case(&e.name),
            Declaration::Function(f) => to_pascal_case(&f.name),
            Declaration::Const(c) => to_pascal_case(&c.name),
            Declaration::SexVar(v) => to_pascal_case(&v.name),
        };

        let inner = self.generate_declaration_inner(decl);

        format!(
            r#"{{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "{}",
  {}
}}"#,
            title,
            // Remove leading '{' and trailing '}' from inner to merge
            &inner[1..inner.len() - 1].trim()
        )
    }

    /// Generate the inner schema without $schema wrapper.
    fn generate_declaration_inner(&self, decl: &Declaration) -> String {
        match decl {
            Declaration::Gene(gene) => self.generate_gene(gene),
            Declaration::Trait(trait_decl) => self.generate_trait(trait_decl),
            Declaration::Constraint(constraint) => self.generate_constraint(constraint),
            Declaration::System(system) => self.generate_system(system),
            Declaration::Evolution(evolution) => self.generate_evolution(evolution),
            Declaration::Function(func) => self.generate_function(func),
            Declaration::Const(c) => self.generate_const(c),
            Declaration::SexVar(v) => self.generate_sex_var(v),
        }
    }

    /// Generate schema for a constant declaration.
    fn generate_const(&self, c: &crate::ast::ConstDecl) -> String {
        format!(
            r#"{{
    "type": "object",
    "description": "Constant {}",
    "properties": {{}}
}}"#,
            c.name
        )
    }

    /// Generate schema for a SEX mutable variable declaration.
    fn generate_sex_var(&self, v: &crate::ast::VarDecl) -> String {
        format!(
            r#"{{
    "type": "object",
    "description": "Mutable SEX variable {}",
    "properties": {{}}
}}"#,
            v.name
        )
    }

    /// Generate schema for a function (placeholder - functions don't map well to JSON Schema).
    fn generate_function(&self, func: &crate::ast::FunctionDecl) -> String {
        format!(
            r#"{{
    "type": "object",
    "description": "Function {}",
    "properties": {{}}
}}"#,
            func.name
        )
    }

    /// Generate a JSON Schema object from a gene declaration.
    fn generate_gene(&self, gene: &Gen) -> String {
        let properties = self.extract_properties(&gene.statements);
        let required = self.extract_required(&gene.statements);

        let mut schema = String::from("{\n");

        // Description from exegesis
        // Always include description from exegesis
        {
            let escaped = escape_json_string(&gene.exegesis);
            schema.push_str(&format!("    \"description\": \"{}\",\n", escaped));
        }

        schema.push_str("    \"type\": \"object\",\n");

        // Properties
        if !properties.is_empty() {
            schema.push_str("    \"properties\": {\n");
            let prop_strs: Vec<String> = properties
                .iter()
                .map(|(name, type_schema)| format!("      \"{}\": {}", name, type_schema))
                .collect();
            schema.push_str(&prop_strs.join(",\n"));
            schema.push_str("\n    },\n");
        } else {
            schema.push_str("    \"properties\": {},\n");
        }

        // Required fields
        if !required.is_empty() {
            let req_strs: Vec<String> = required.iter().map(|r| format!("\"{}\"", r)).collect();
            schema.push_str(&format!("    \"required\": [{}]\n", req_strs.join(", ")));
        } else {
            schema.push_str("    \"required\": []\n");
        }

        schema.push_str("  }");
        schema
    }

    /// Generate a JSON Schema with allOf for trait composition.
    fn generate_trait(&self, trait_decl: &Trait) -> String {
        // Find "uses" statements for $ref
        let refs: Vec<String> = trait_decl
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Uses { reference, .. } = stmt {
                    Some(to_pascal_case(reference))
                } else {
                    None
                }
            })
            .collect();

        let properties = self.extract_properties(&trait_decl.statements);

        let mut schema = String::from("{\n");

        // Description from exegesis
        // Always include description from exegesis
        {
            let escaped = escape_json_string(&trait_decl.exegesis);
            schema.push_str(&format!("    \"description\": \"{}\",\n", escaped));
        }

        if refs.is_empty() {
            // No composition, just an object
            schema.push_str("    \"type\": \"object\",\n");

            if !properties.is_empty() {
                schema.push_str("    \"properties\": {\n");
                let prop_strs: Vec<String> = properties
                    .iter()
                    .map(|(name, type_schema)| format!("      \"{}\": {}", name, type_schema))
                    .collect();
                schema.push_str(&prop_strs.join(",\n"));
                schema.push_str("\n    }\n");
            } else {
                schema.push_str("    \"properties\": {}\n");
            }
        } else {
            // Use allOf for composition
            schema.push_str("    \"allOf\": [\n");

            // Add $ref for each used trait/gene
            for ref_name in &refs {
                schema.push_str(&format!(
                    "      {{ \"$ref\": \"#/$defs/{}\" }},\n",
                    ref_name
                ));
            }

            // Add own properties if any
            if !properties.is_empty() {
                schema.push_str("      {\n");
                schema.push_str("        \"type\": \"object\",\n");
                schema.push_str("        \"properties\": {\n");
                let prop_strs: Vec<String> = properties
                    .iter()
                    .map(|(name, type_schema)| format!("          \"{}\": {}", name, type_schema))
                    .collect();
                schema.push_str(&prop_strs.join(",\n"));
                schema.push_str("\n        }\n");
                schema.push_str("      }\n");
            } else {
                // Remove trailing comma from last $ref
                schema = schema.trim_end_matches(",\n").to_string();
                schema.push('\n');
            }

            schema.push_str("    ]\n");
        }

        schema.push_str("  }");
        schema
    }

    /// Generate a schema for constraint (validation rules as metadata).
    fn generate_constraint(&self, constraint: &Rule) -> String {
        let mut schema = String::from("{\n");

        // Description from exegesis
        // Always include description from exegesis
        {
            let escaped = escape_json_string(&constraint.exegesis);
            schema.push_str(&format!("    \"description\": \"{}\",\n", escaped));
        }

        schema.push_str("    \"type\": \"object\",\n");
        schema.push_str("    \"x-dol-constraint\": true,\n");

        // Extract constraint rules as custom extensions
        let mut rules = Vec::new();
        for stmt in &constraint.statements {
            match stmt {
                Statement::Matches {
                    subject, target, ..
                } => {
                    rules.push(format!(
                        "{{ \"type\": \"matches\", \"subject\": \"{}\", \"target\": \"{}\" }}",
                        subject, target
                    ));
                }
                Statement::Never {
                    subject, action, ..
                } => {
                    rules.push(format!(
                        "{{ \"type\": \"never\", \"subject\": \"{}\", \"action\": \"{}\" }}",
                        subject, action
                    ));
                }
                _ => {}
            }
        }

        if !rules.is_empty() {
            schema.push_str(&format!(
                "    \"x-dol-rules\": [\n      {}\n    ]\n",
                rules.join(",\n      ")
            ));
        } else {
            schema.push_str("    \"x-dol-rules\": []\n");
        }

        schema.push_str("  }");
        schema
    }

    /// Generate a schema for system (metadata about system composition).
    fn generate_system(&self, system: &System) -> String {
        let mut schema = String::from("{\n");

        // Description from exegesis
        // Always include description from exegesis
        {
            let escaped = escape_json_string(&system.exegesis);
            schema.push_str(&format!("    \"description\": \"{}\",\n", escaped));
        }

        schema.push_str("    \"type\": \"object\",\n");
        schema.push_str(&format!("    \"x-dol-version\": \"{}\",\n", system.version));

        // Requirements as custom extension
        if !system.requirements.is_empty() {
            let reqs: Vec<String> = system
                .requirements
                .iter()
                .map(|r| {
                    format!(
                        "{{ \"name\": \"{}\", \"constraint\": \"{}\", \"version\": \"{}\" }}",
                        r.name, r.constraint, r.version
                    )
                })
                .collect();
            schema.push_str(&format!(
                "    \"x-dol-requirements\": [\n      {}\n    ]\n",
                reqs.join(",\n      ")
            ));
        } else {
            schema.push_str("    \"x-dol-requirements\": []\n");
        }

        schema.push_str("  }");
        schema
    }

    /// Generate a schema for evolution (version history metadata).
    fn generate_evolution(&self, evolution: &Evo) -> String {
        let mut schema = String::from("{\n");

        // Description from exegesis
        // Always include description from exegesis
        {
            let escaped = escape_json_string(&evolution.exegesis);
            schema.push_str(&format!("    \"description\": \"{}\",\n", escaped));
        }

        schema.push_str("    \"type\": \"object\",\n");
        schema.push_str(&format!(
            "    \"x-dol-version\": \"{}\",\n",
            evolution.version
        ));
        schema.push_str(&format!(
            "    \"x-dol-parent-version\": \"{}\",\n",
            evolution.parent_version
        ));

        // Track additions
        let additions: Vec<String> = evolution
            .additions
            .iter()
            .filter_map(|stmt| match stmt {
                Statement::Has {
                    subject, property, ..
                } => Some(format!("\"{}:has:{}\"", subject, property)),
                Statement::Is { subject, state, .. } => {
                    Some(format!("\"{}:is:{}\"", subject, state))
                }
                _ => None,
            })
            .collect();

        if !additions.is_empty() {
            schema.push_str(&format!(
                "    \"x-dol-additions\": [{}]\n",
                additions.join(", ")
            ));
        } else {
            schema.push_str("    \"x-dol-additions\": []\n");
        }

        schema.push_str("  }");
        schema
    }

    /// Extract properties from "has" statements.
    fn extract_properties(&self, statements: &[Statement]) -> Vec<(String, String)> {
        statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Has { property, .. } = stmt {
                    // Since AST doesn't have type annotations, use generic schema
                    Some((property.clone(), "{ \"type\": \"string\" }".to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract required field names from "has" statements.
    fn extract_required(&self, statements: &[Statement]) -> Vec<String> {
        statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Has { property, .. } = stmt {
                    Some(property.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl TypeMapper for JsonSchemaCodegen {
    fn map_type(ty: &Type) -> String {
        match ty {
            Type::Void => r#"{ "type": "null" }"#.to_string(),
            Type::Bool => r#"{ "type": "boolean" }"#.to_string(),
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 => {
                r#"{ "type": "integer" }"#.to_string()
            }
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 => {
                r#"{ "type": "integer", "minimum": 0 }"#.to_string()
            }
            Type::Float32 | Type::Float64 => r#"{ "type": "number" }"#.to_string(),
            Type::String => r#"{ "type": "string" }"#.to_string(),
            Type::Function { .. } => {
                // Functions can't be represented in JSON Schema
                r#"{ "type": "object", "x-dol-function": true }"#.to_string()
            }
            Type::Tuple(types) => {
                let items: Vec<_> = types.iter().map(Self::map_type).collect();
                format!(
                    r#"{{ "type": "array", "items": [{}], "minItems": {}, "maxItems": {} }}"#,
                    items.join(", "),
                    types.len(),
                    types.len()
                )
            }
            Type::Generic { name, args } => {
                let mapped_args: Vec<_> = args.iter().map(Self::map_type).collect();
                match name.as_str() {
                    "List" => {
                        if args.len() == 1 {
                            format!(r#"{{ "type": "array", "items": {} }}"#, mapped_args[0])
                        } else {
                            r#"{ "type": "array" }"#.to_string()
                        }
                    }
                    "Map" => {
                        if args.len() == 2 {
                            format!(
                                r#"{{ "type": "object", "additionalProperties": {} }}"#,
                                mapped_args[1]
                            )
                        } else {
                            r#"{ "type": "object" }"#.to_string()
                        }
                    }
                    "Option" => {
                        if args.len() == 1 {
                            format!(
                                r#"{{ "oneOf": [{}, {{ "type": "null" }}] }}"#,
                                mapped_args[0]
                            )
                        } else {
                            r#"{ }"#.to_string()
                        }
                    }
                    "Result" => {
                        if args.len() == 2 {
                            format!(
                                r#"{{ "oneOf": [{{ "type": "object", "properties": {{ "ok": {{ "const": true }}, "value": {} }}, "required": ["ok", "value"] }}, {{ "type": "object", "properties": {{ "ok": {{ "const": false }}, "error": {} }}, "required": ["ok", "error"] }}] }}"#,
                                mapped_args[0], mapped_args[1]
                            )
                        } else {
                            r#"{ "type": "object" }"#.to_string()
                        }
                    }
                    _ => {
                        // Reference to another type
                        format!(r##"{{ "$ref": "#/$defs/{}" }}"##, to_pascal_case(name))
                    }
                }
            }
            Type::Var(id) => format!(r#"{{ "$comment": "Type variable T{}" }}"#, id),
            Type::Any => r#"{ }"#.to_string(),
            Type::Unknown => r#"{ }"#.to_string(),
            Type::Never => r#"{ "not": {} }"#.to_string(),
            Type::Error => r#"{ "not": {} }"#.to_string(),
        }
    }

    fn map_type_expr(ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(name) => match name.as_str() {
                "Int8" | "Int16" | "Int32" | "Int64" => r#"{ "type": "integer" }"#.to_string(),
                "UInt8" | "UInt16" | "UInt32" | "UInt64" => {
                    r#"{ "type": "integer", "minimum": 0 }"#.to_string()
                }
                "Float32" | "Float64" => r#"{ "type": "number" }"#.to_string(),
                "String" => r#"{ "type": "string" }"#.to_string(),
                "Bool" => r#"{ "type": "boolean" }"#.to_string(),
                "Void" => r#"{ "type": "null" }"#.to_string(),
                "Any" => r#"{ }"#.to_string(),
                _ => format!(r##"{{ "$ref": "#/$defs/{}" }}"##, to_pascal_case(name)),
            },
            TypeExpr::Generic { name, args } => {
                let mapped_args: Vec<_> = args.iter().map(Self::map_type_expr).collect();
                match name.as_str() {
                    "List" => {
                        if args.len() == 1 {
                            format!(r#"{{ "type": "array", "items": {} }}"#, mapped_args[0])
                        } else {
                            r#"{ "type": "array" }"#.to_string()
                        }
                    }
                    "Map" => {
                        if args.len() == 2 {
                            format!(
                                r#"{{ "type": "object", "additionalProperties": {} }}"#,
                                mapped_args[1]
                            )
                        } else {
                            r#"{ "type": "object" }"#.to_string()
                        }
                    }
                    "Option" => {
                        if args.len() == 1 {
                            format!(
                                r#"{{ "oneOf": [{}, {{ "type": "null" }}] }}"#,
                                mapped_args[0]
                            )
                        } else {
                            r#"{ }"#.to_string()
                        }
                    }
                    "Result" => {
                        if args.len() == 2 {
                            format!(
                                r#"{{ "oneOf": [{{ "type": "object", "properties": {{ "ok": {{ "const": true }}, "value": {} }}, "required": ["ok", "value"] }}, {{ "type": "object", "properties": {{ "ok": {{ "const": false }}, "error": {} }}, "required": ["ok", "error"] }}] }}"#,
                                mapped_args[0], mapped_args[1]
                            )
                        } else {
                            r#"{ "type": "object" }"#.to_string()
                        }
                    }
                    _ => format!(r##"{{ "$ref": "#/$defs/{}" }}"##, to_pascal_case(name)),
                }
            }
            TypeExpr::Function { .. } => {
                r#"{ "type": "object", "x-dol-function": true }"#.to_string()
            }
            TypeExpr::Tuple(types) => {
                let items: Vec<_> = types.iter().map(Self::map_type_expr).collect();
                format!(
                    r#"{{ "type": "array", "items": [{}], "minItems": {}, "maxItems": {} }}"#,
                    items.join(", "),
                    types.len(),
                    types.len()
                )
            }
            TypeExpr::Never => r#"{ "not": {} }"#.to_string(),
            TypeExpr::Enum { variants } => {
                let enum_values: Vec<_> = variants
                    .iter()
                    .map(|v| format!(r#""{}""#, v.name))
                    .collect();
                format!(r#"{{ "enum": [{}] }}"#, enum_values.join(", "))
            }
        }
    }
}

/// Escape special characters for JSON string.
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Span, Visibility};

    #[test]
    fn test_generate_gene_schema() {
        let gene = Gen {
            visibility: Visibility::default(),
            name: "container.exists".to_string(),
            extends: None,
            statements: vec![
                Statement::Has {
                    subject: "container".to_string(),
                    property: "id".to_string(),
                    span: Span::default(),
                },
                Statement::Has {
                    subject: "container".to_string(),
                    property: "image".to_string(),
                    span: Span::default(),
                },
            ],
            exegesis: "A container is the fundamental unit.".to_string(),
            span: Span::default(),
        };

        let schema = JsonSchemaCodegen::generate(&Declaration::Gene(gene));

        assert!(schema.contains("\"$schema\": \"http://json-schema.org/draft-07/schema#\""));
        assert!(schema.contains("\"title\": \"ContainerExists\""));
        assert!(schema.contains("\"type\": \"object\""));
        assert!(schema.contains("\"id\":"));
        assert!(schema.contains("\"image\":"));
        assert!(schema.contains("\"required\": [\"id\", \"image\"]"));
    }

    #[test]
    fn test_generate_trait_with_refs() {
        let trait_decl = Trait {
            visibility: Visibility::default(),
            name: "container.lifecycle".to_string(),
            statements: vec![Statement::Uses {
                reference: "container.exists".to_string(),
                span: Span::default(),
            }],
            exegesis: "Container lifecycle management.".to_string(),
            span: Span::default(),
        };

        let schema = JsonSchemaCodegen::generate(&Declaration::Trait(trait_decl));

        assert!(schema.contains("\"allOf\""));
        assert!(schema.contains("\"$ref\": \"#/$defs/ContainerExists\""));
    }

    #[test]
    fn test_generate_constraint_schema() {
        let constraint = Rule {
            visibility: Visibility::default(),
            name: "container.integrity".to_string(),
            statements: vec![Statement::Matches {
                subject: "state".to_string(),
                target: "declared_state".to_string(),
                span: Span::default(),
            }],
            exegesis: "Container integrity constraints.".to_string(),
            span: Span::default(),
        };

        let schema = JsonSchemaCodegen::generate(&Declaration::Constraint(constraint));

        assert!(schema.contains("\"x-dol-constraint\": true"));
        assert!(schema.contains("\"x-dol-rules\":"));
        assert!(schema.contains("\"type\": \"matches\""));
    }

    #[test]
    fn test_generate_system_schema() {
        let system = System {
            visibility: Visibility::default(),
            name: "container.runtime".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            statements: vec![],
            exegesis: "Container runtime system.".to_string(),
            span: Span::default(),
        };

        let schema = JsonSchemaCodegen::generate(&Declaration::System(system));

        assert!(schema.contains("\"x-dol-version\": \"1.0.0\""));
    }

    #[test]
    fn test_map_type() {
        assert!(JsonSchemaCodegen::map_type(&Type::Int32).contains("\"type\": \"integer\""));
        assert!(JsonSchemaCodegen::map_type(&Type::String).contains("\"type\": \"string\""));
        assert!(JsonSchemaCodegen::map_type(&Type::Bool).contains("\"type\": \"boolean\""));
        assert!(JsonSchemaCodegen::map_type(&Type::UInt32).contains("\"minimum\": 0"));

        let list_schema = JsonSchemaCodegen::map_type(&Type::Generic {
            name: "List".to_string(),
            args: vec![Type::String],
        });
        assert!(list_schema.contains("\"type\": \"array\""));
        assert!(list_schema.contains("\"items\":"));

        let option_schema = JsonSchemaCodegen::map_type(&Type::Generic {
            name: "Option".to_string(),
            args: vec![Type::Int32],
        });
        assert!(option_schema.contains("\"oneOf\""));
        assert!(option_schema.contains("\"type\": \"null\""));
    }

    #[test]
    fn test_map_type_expr() {
        assert!(
            JsonSchemaCodegen::map_type_expr(&TypeExpr::Named("Int32".to_string()))
                .contains("\"type\": \"integer\"")
        );

        let list_schema = JsonSchemaCodegen::map_type_expr(&TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        });
        assert!(list_schema.contains("\"type\": \"array\""));
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json_string("say \"hi\""), "say \\\"hi\\\"");
    }
}
