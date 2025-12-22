//! TypeScript code generation from Metal DOL declarations.
//!
//! Generates TypeScript interfaces, types, and type guards from DOL declarations.
//!
//! # Type Mapping
//!
//! | DOL Type | TypeScript Type |
//! |----------|-----------------|
//! | `Int8` | `number` |
//! | `Int16` | `number` |
//! | `Int32` | `number` |
//! | `Int64` | `number` |
//! | `UInt8` | `number` |
//! | `UInt16` | `number` |
//! | `UInt32` | `number` |
//! | `UInt64` | `number` |
//! | `Float32` | `number` |
//! | `Float64` | `number` |
//! | `String` | `string` |
//! | `Bool` | `boolean` |
//! | `Void` | `void` |
//! | `Option<T>` | `T \| undefined` |
//! | `Result<T, E>` | `{ ok: true; value: T } \| { ok: false; error: E }` |
//! | `List<T>` | `T[]` |
//! | `Map<K, V>` | `Map<K, V>` or `Record<K, V>` |
//! | `Tuple(A, B)` | `[A, B]` |
//! | `Function` | `(args) => ReturnType` |

use crate::ast::{Constraint, Declaration, Evolution, Gene, Statement, System, Trait, TypeExpr};
use crate::typechecker::Type;

use super::{to_pascal_case, CodegenOptions, TypeMapper};

/// Convert a DOL identifier to camelCase for TypeScript.
fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}

/// TypeScript code generator.
///
/// Transforms DOL declarations into TypeScript source code.
#[derive(Debug, Clone, Default)]
pub struct TypeScriptCodegen {
    options: CodegenOptions,
}

impl TypeScriptCodegen {
    /// Create a new TypeScript code generator with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new TypeScript code generator with custom options.
    pub fn with_options(options: CodegenOptions) -> Self {
        Self { options }
    }

    /// Generate TypeScript code from a declaration.
    pub fn generate(decl: &Declaration) -> String {
        Self::new().generate_declaration(decl)
    }

    /// Generate TypeScript code from multiple declarations.
    pub fn generate_all(decls: &[Declaration]) -> String {
        let generator = Self::new();
        decls
            .iter()
            .map(|d| generator.generate_declaration(d))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Generate code for a single declaration.
    fn generate_declaration(&self, decl: &Declaration) -> String {
        match decl {
            Declaration::Gene(gene) => self.generate_gene(gene),
            Declaration::Trait(trait_decl) => self.generate_trait(trait_decl),
            Declaration::Constraint(constraint) => self.generate_constraint(constraint),
            Declaration::System(system) => self.generate_system(system),
            Declaration::Evolution(evolution) => self.generate_evolution(evolution),
        }
    }

    /// Generate a TypeScript interface from a gene declaration.
    fn generate_gene(&self, gene: &Gene) -> String {
        let interface_name = to_pascal_case(&gene.name);

        // Collect properties from "has" statements
        let fields = self.extract_fields(&gene.statements);

        let mut output = String::new();

        // JSDoc comment from exegesis
        output.push_str(&self.format_jsdoc(&gene.exegesis));

        // Export keyword
        let export = if self.is_public() { "export " } else { "" };

        // Interface definition
        output.push_str(&format!("{export}interface {interface_name} {{\n"));

        for (field_name, field_type) in &fields {
            let ts_field = to_camel_case(field_name);
            output.push_str(&format!("  {ts_field}: {field_type};\n"));
        }

        output.push_str("}\n");

        output
    }

    /// Generate a TypeScript interface from a trait declaration.
    fn generate_trait(&self, trait_decl: &Trait) -> String {
        let interface_name = to_pascal_case(&trait_decl.name);

        // Find "uses" statements for extends clause
        let extends: Vec<String> = trait_decl
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

        // Extract methods from "is" statements
        let methods = self.extract_methods(&trait_decl.statements);

        let mut output = String::new();

        // JSDoc comment from exegesis
        output.push_str(&self.format_jsdoc(&trait_decl.exegesis));

        // Export keyword
        let export = if self.is_public() { "export " } else { "" };

        // Interface with extends
        if extends.is_empty() {
            output.push_str(&format!("{export}interface {interface_name} {{\n"));
        } else {
            output.push_str(&format!(
                "{export}interface {interface_name} extends {} {{\n",
                extends.join(", ")
            ));
        }

        // Generate method signatures
        for (method_name, return_type) in &methods {
            let ts_method = to_camel_case(method_name);
            output.push_str(&format!("  /** Get the {} state. */\n", method_name));
            output.push_str(&format!("  {ts_method}(): {return_type};\n"));
        }

        output.push_str("}\n");

        output
    }

    /// Generate a TypeScript type guard from a constraint declaration.
    fn generate_constraint(&self, constraint: &Constraint) -> String {
        let mut output = String::new();

        // JSDoc comment from exegesis
        output.push_str(&self.format_jsdoc(&constraint.exegesis));

        // Export keyword
        let export = if self.is_public() { "export " } else { "" };

        // Type guard function
        output.push_str(&format!(
            "{export}function validate{}<T>(value: T): boolean {{\n",
            to_pascal_case(&constraint.name)
        ));

        // Generate constraint checks from statements
        for stmt in &constraint.statements {
            match stmt {
                Statement::Matches {
                    subject, target, ..
                } => {
                    output.push_str(&format!("  // {subject} matches {target}\n"));
                }
                Statement::Never {
                    subject, action, ..
                } => {
                    output.push_str(&format!("  // {subject} never {action}\n"));
                }
                _ => {}
            }
        }

        output.push_str("  return true; // TODO: Implement constraint validation\n");
        output.push_str("}\n");

        output
    }

    /// Generate a TypeScript module from a system declaration.
    fn generate_system(&self, system: &System) -> String {
        let namespace_name = to_pascal_case(&system.name);

        let mut output = String::new();

        // JSDoc comment from exegesis
        output.push_str(&self.format_jsdoc(&system.exegesis));

        // Export keyword
        let export = if self.is_public() { "export " } else { "" };

        // Namespace declaration
        output.push_str(&format!("{export}namespace {namespace_name} {{\n"));
        output.push_str(&format!("  export const VERSION = '{}';\n", system.version));

        if !system.requirements.is_empty() {
            output.push_str("\n  // Requirements:\n");
            for req in &system.requirements {
                output.push_str(&format!(
                    "  // - {} {} {}\n",
                    req.name, req.constraint, req.version
                ));
            }
        }

        output.push_str("}\n");

        output
    }

    /// Generate TypeScript documentation from an evolution declaration.
    fn generate_evolution(&self, evolution: &Evolution) -> String {
        let type_name = to_pascal_case(&evolution.name);

        let mut output = String::new();

        // JSDoc block as changelog
        output.push_str("/**\n");
        output.push_str(&format!(
            " * Evolution: {} -> {}\n",
            evolution.parent_version, evolution.version
        ));
        output.push_str(" *\n");

        for line in evolution.exegesis.lines() {
            output.push_str(&format!(" * {}\n", line.trim()));
        }

        if !evolution.additions.is_empty() {
            output.push_str(" *\n");
            output.push_str(" * Additions:\n");
            for addition in &evolution.additions {
                match addition {
                    Statement::Has {
                        subject, property, ..
                    } => {
                        output.push_str(&format!(" * - {} has {}\n", subject, property));
                    }
                    Statement::Is { subject, state, .. } => {
                        output.push_str(&format!(" * - {} is {}\n", subject, state));
                    }
                    _ => {}
                }
            }
        }

        output.push_str(" */\n");

        // Export a type alias for documentation purposes
        let export = if self.is_public() { "export " } else { "" };
        output.push_str(&format!(
            "{export}type {type_name}Evolution = '{}';\n",
            evolution.version
        ));

        output
    }

    /// Extract fields from "has" statements.
    fn extract_fields(&self, statements: &[Statement]) -> Vec<(String, String)> {
        statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Has { property, .. } = stmt {
                    // Since AST doesn't have type annotations, use "unknown"
                    Some((property.clone(), "unknown".to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract methods from "is" statements.
    fn extract_methods(&self, statements: &[Statement]) -> Vec<(String, String)> {
        statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Is { state, .. } = stmt {
                    // "is" statements map to boolean methods
                    Some((state.clone(), "boolean".to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Format exegesis as JSDoc comment.
    fn format_jsdoc(&self, exegesis: &str) -> String {
        let mut output = String::new();
        output.push_str("/**\n");
        for line in exegesis.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                output.push_str(" *\n");
            } else {
                output.push_str(&format!(" * {}\n", trimmed));
            }
        }
        output.push_str(" */\n");
        output
    }

    /// Check if we should export items.
    fn is_public(&self) -> bool {
        matches!(
            self.options.visibility,
            super::Visibility::Public | super::Visibility::Crate
        )
    }
}

impl TypeMapper for TypeScriptCodegen {
    fn map_type(ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 => "number".to_string(),
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 => "number".to_string(),
            Type::Float32 | Type::Float64 => "number".to_string(),
            Type::String => "string".to_string(),
            Type::Function {
                params,
                return_type,
            } => {
                let param_types: Vec<_> = params
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("arg{}: {}", i, Self::map_type(t)))
                    .collect();
                let ret = Self::map_type(return_type);
                format!("({}) => {}", param_types.join(", "), ret)
            }
            Type::Tuple(types) => {
                let mapped: Vec<_> = types.iter().map(Self::map_type).collect();
                format!("[{}]", mapped.join(", "))
            }
            Type::Generic { name, args } => {
                let mapped_args: Vec<_> = args.iter().map(Self::map_type).collect();
                // Map DOL generic types to TypeScript equivalents
                match name.as_str() {
                    "List" => {
                        if args.len() == 1 {
                            format!("{}[]", mapped_args[0])
                        } else {
                            "unknown[]".to_string()
                        }
                    }
                    "Map" => {
                        if args.len() == 2 {
                            format!("Map<{}, {}>", mapped_args[0], mapped_args[1])
                        } else {
                            "Map<unknown, unknown>".to_string()
                        }
                    }
                    "Option" => {
                        if args.len() == 1 {
                            format!("{} | undefined", mapped_args[0])
                        } else {
                            "unknown | undefined".to_string()
                        }
                    }
                    "Result" => {
                        if args.len() == 2 {
                            format!(
                                "{{ ok: true; value: {} }} | {{ ok: false; error: {} }}",
                                mapped_args[0], mapped_args[1]
                            )
                        } else {
                            "{ ok: boolean; value?: unknown; error?: unknown }".to_string()
                        }
                    }
                    _ => {
                        if args.is_empty() {
                            to_pascal_case(name)
                        } else {
                            format!("{}<{}>", to_pascal_case(name), mapped_args.join(", "))
                        }
                    }
                }
            }
            Type::Var(id) => format!("T{}", id),
            Type::Any => "any".to_string(),
            Type::Unknown => "unknown".to_string(),
            Type::Error => "never".to_string(),
        }
    }

    fn map_type_expr(ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(name) => match name.as_str() {
                "Int8" | "Int16" | "Int32" | "Int64" => "number".to_string(),
                "UInt8" | "UInt16" | "UInt32" | "UInt64" => "number".to_string(),
                "Float32" | "Float64" => "number".to_string(),
                "String" => "string".to_string(),
                "Bool" => "boolean".to_string(),
                "Void" => "void".to_string(),
                "Any" => "any".to_string(),
                _ => to_pascal_case(name),
            },
            TypeExpr::Generic { name, args } => {
                let mapped_args: Vec<_> = args.iter().map(Self::map_type_expr).collect();
                match name.as_str() {
                    "List" => {
                        if args.len() == 1 {
                            format!("{}[]", mapped_args[0])
                        } else {
                            "unknown[]".to_string()
                        }
                    }
                    "Map" => {
                        if args.len() == 2 {
                            format!("Map<{}, {}>", mapped_args[0], mapped_args[1])
                        } else {
                            "Map<unknown, unknown>".to_string()
                        }
                    }
                    "Option" => {
                        if args.len() == 1 {
                            format!("{} | undefined", mapped_args[0])
                        } else {
                            "unknown | undefined".to_string()
                        }
                    }
                    "Result" => {
                        if args.len() == 2 {
                            format!(
                                "{{ ok: true; value: {} }} | {{ ok: false; error: {} }}",
                                mapped_args[0], mapped_args[1]
                            )
                        } else {
                            "{ ok: boolean; value?: unknown; error?: unknown }".to_string()
                        }
                    }
                    _ => format!("{}<{}>", to_pascal_case(name), mapped_args.join(", ")),
                }
            }
            TypeExpr::Function {
                params,
                return_type,
            } => {
                let param_types: Vec<_> = params
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("arg{}: {}", i, Self::map_type_expr(t)))
                    .collect();
                let ret = Self::map_type_expr(return_type);
                format!("({}) => {}", param_types.join(", "), ret)
            }
            TypeExpr::Tuple(types) => {
                let mapped: Vec<_> = types.iter().map(Self::map_type_expr).collect();
                format!("[{}]", mapped.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    #[test]
    fn test_generate_gene_interface() {
        let gene = Gene {
            name: "container.exists".to_string(),
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

        let code = TypeScriptCodegen::generate(&Declaration::Gene(gene));

        assert!(code.contains("export interface ContainerExists"));
        assert!(code.contains("id: unknown;"));
        assert!(code.contains("image: unknown;"));
    }

    #[test]
    fn test_generate_trait_interface() {
        let trait_decl = Trait {
            name: "container.lifecycle".to_string(),
            statements: vec![
                Statement::Uses {
                    reference: "container.exists".to_string(),
                    span: Span::default(),
                },
                Statement::Is {
                    subject: "container".to_string(),
                    state: "created".to_string(),
                    span: Span::default(),
                },
            ],
            exegesis: "Container lifecycle management.".to_string(),
            span: Span::default(),
        };

        let code = TypeScriptCodegen::generate(&Declaration::Trait(trait_decl));

        assert!(code.contains("export interface ContainerLifecycle extends ContainerExists"));
        assert!(code.contains("created(): boolean;"));
    }

    #[test]
    fn test_map_type() {
        assert_eq!(TypeScriptCodegen::map_type(&Type::Int32), "number");
        assert_eq!(TypeScriptCodegen::map_type(&Type::String), "string");
        assert_eq!(TypeScriptCodegen::map_type(&Type::Bool), "boolean");
        assert_eq!(
            TypeScriptCodegen::map_type(&Type::Generic {
                name: "Option".to_string(),
                args: vec![Type::String]
            }),
            "string | undefined"
        );
        assert_eq!(
            TypeScriptCodegen::map_type(&Type::Generic {
                name: "List".to_string(),
                args: vec![Type::Int32]
            }),
            "number[]"
        );
        assert_eq!(
            TypeScriptCodegen::map_type(&Type::Generic {
                name: "Map".to_string(),
                args: vec![Type::String, Type::Int64]
            }),
            "Map<string, number>"
        );
    }

    #[test]
    fn test_map_type_expr() {
        assert_eq!(
            TypeScriptCodegen::map_type_expr(&TypeExpr::Named("Int32".to_string())),
            "number"
        );
        assert_eq!(
            TypeScriptCodegen::map_type_expr(&TypeExpr::Generic {
                name: "List".to_string(),
                args: vec![TypeExpr::Named("String".to_string())]
            }),
            "string[]"
        );
        assert_eq!(
            TypeScriptCodegen::map_type_expr(&TypeExpr::Generic {
                name: "Result".to_string(),
                args: vec![
                    TypeExpr::Named("String".to_string()),
                    TypeExpr::Named("String".to_string())
                ]
            }),
            "{ ok: true; value: string } | { ok: false; error: string }"
        );
    }

    #[test]
    fn test_generate_constraint_type_guard() {
        let constraint = Constraint {
            name: "container.integrity".to_string(),
            statements: vec![Statement::Matches {
                subject: "state".to_string(),
                target: "declared_state".to_string(),
                span: Span::default(),
            }],
            exegesis: "Container integrity constraints.".to_string(),
            span: Span::default(),
        };

        let code = TypeScriptCodegen::generate(&Declaration::Constraint(constraint));

        assert!(code.contains("export function validateContainerIntegrity"));
        assert!(code.contains("value: T"));
        assert!(code.contains("boolean"));
    }

    #[test]
    fn test_generate_system_namespace() {
        let system = System {
            name: "container.runtime".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            statements: vec![],
            exegesis: "Container runtime system.".to_string(),
            span: Span::default(),
        };

        let code = TypeScriptCodegen::generate(&Declaration::System(system));

        assert!(code.contains("export namespace ContainerRuntime"));
        assert!(code.contains("VERSION = '1.0.0'"));
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("container.exists"), "containerExists");
        assert_eq!(
            to_camel_case("identity.cryptographic"),
            "identityCryptographic"
        );
        assert_eq!(to_camel_case("Simple"), "simple");
    }
}
