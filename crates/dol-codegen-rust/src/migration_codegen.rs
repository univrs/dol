//! Migration Code Generation from DOL Evolution Declarations
//!
//! This module generates deterministic migration code from DOL `evo` declarations.
//! Generated migrations are CRDT-safe and produce identical operations on all peers.
//!
//! # Example DOL Evolution
//!
//! ```dol
//! evo user.profile @ 2.0.0 > 1.0.0 {
//!   adds email: String
//!   renames username -> display_name
//!   removes legacy_id
//!
//!   because "Email field required for notifications, username renamed for clarity"
//! }
//!
//! docs {
//!   Version 2.0.0 adds email support and renames username for clarity.
//! }
//! ```
//!
//! # Generated Migration
//!
//! ```rust,ignore
//! pub struct UserProfileV1ToV2;
//!
//! #[async_trait]
//! impl Migration for UserProfileV1ToV2 {
//!     async fn migrate(&self, doc: &mut automerge::Automerge) -> Result<()> {
//!         let mut tx = doc.transaction();
//!         tx.set_actor(ActorId::from(vec![0u8; 32]));  // Deterministic
//!
//!         // Add email field
//!         if !tx.get(ROOT, "email")?.is_some() {
//!             tx.put(ROOT, "email", "")?;
//!         }
//!
//!         // Rename username -> display_name
//!         if let Some((value, _)) = tx.get(ROOT, "username")? {
//!             tx.put(ROOT, "display_name", value)?;
//!             tx.delete(ROOT, "username")?;
//!         }
//!
//!         // Remove legacy_id
//!         tx.delete(ROOT, "legacy_id")?;
//!
//!         tx.commit();
//!         Ok(())
//!     }
//!
//!     fn can_migrate(&self, doc: &automerge::Automerge) -> bool {
//!         // Check if migration is needed
//!         doc.get(ROOT, "__schema_version")
//!             .ok()
//!             .flatten()
//!             .is_some()
//!     }
//!
//!     fn metadata(&self) -> &MigrationMetadata {
//!         &MigrationMetadata {
//!             name: "UserProfileV1ToV2".to_string(),
//!             from_version: Version::new(1, 0, 0),
//!             to_version: Version::new(2, 0, 0),
//!             actor_id: ActorId::from(vec![0u8; 32]),
//!         }
//!     }
//! }
//! ```

use dol::ast::{Evo, Statement};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use thiserror::Error;

/// Error type for migration code generation.
#[derive(Error, Debug)]
pub enum MigrationCodegenError {
    /// Evolution declaration is invalid.
    #[error("Invalid evolution declaration: {0}")]
    InvalidEvolution(String),

    /// Failed to parse version.
    #[error("Failed to parse version: {0}")]
    InvalidVersion(String),

    /// Unsupported migration operation.
    #[error("Unsupported migration operation: {0}")]
    UnsupportedOperation(String),

    /// Code generation error.
    #[error("Code generation error: {0}")]
    CodegenError(String),
}

/// Result type for migration codegen.
pub type Result<T> = std::result::Result<T, MigrationCodegenError>;

/// Migration operation detected from DOL evolution.
#[derive(Debug, Clone, PartialEq)]
pub enum MigrationOp {
    /// Add a new field with default value.
    AddField {
        name: String,
        type_name: String,
        default_value: String,
    },

    /// Remove a field.
    RemoveField { name: String },

    /// Rename a field.
    RenameField { old_name: String, new_name: String },

    /// Change field type (requires conversion).
    ChangeType {
        name: String,
        old_type: String,
        new_type: String,
    },
}

/// Migration code generator.
pub struct MigrationCodegen {
    /// Template cache for code generation.
    templates: HashMap<String, String>,
}

impl MigrationCodegen {
    /// Create a new migration codegen.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Generate migration code from DOL evolution.
    pub fn generate(&self, evolution: &Evo) -> Result<TokenStream> {
        // Parse version strings
        let from_version = self.parse_version(&evolution.parent_version)?;
        let to_version = self.parse_version(&evolution.version)?;

        // Analyze evolution changes
        let operations = self.analyze_evolution(evolution)?;

        // Generate migration struct name
        let migration_name = self.generate_migration_name(&evolution.name, &from_version, &to_version);
        let migration_ident = syn::Ident::new(&migration_name, proc_macro2::Span::call_site());

        // Generate migration operations
        let migration_ops = self.generate_operations(&operations)?;

        // Generate metadata
        let metadata_code = self.generate_metadata(&migration_name, &from_version, &to_version);

        // Generate complete migration implementation
        let code = quote! {
            /// Generated migration from DOL evolution.
            #[derive(Debug, Clone)]
            pub struct #migration_ident;

            #[async_trait::async_trait]
            impl vudo_state::Migration for #migration_ident {
                async fn migrate(&self, doc: &mut automerge::Automerge) -> vudo_state::Result<()> {
                    use automerge::transaction::Transactable;
                    use automerge::{ActorId, ROOT};

                    let mut tx = doc.transaction();

                    // Set deterministic actor ID for CRDT consistency
                    tx.set_actor(ActorId::from(vec![0u8; 32]));

                    #migration_ops

                    tx.commit();
                    Ok(())
                }

                fn can_migrate(&self, doc: &automerge::Automerge) -> bool {
                    // Migration can be applied if document has schema version
                    use automerge::ReadDoc;
                    doc.get(&automerge::ROOT, "__schema_version")
                        .ok()
                        .flatten()
                        .is_some()
                }

                fn metadata(&self) -> &vudo_state::MigrationMetadata {
                    #metadata_code
                }
            }
        };

        Ok(code)
    }

    /// Analyze evolution declaration to extract migration operations.
    fn analyze_evolution(&self, evolution: &Evo) -> Result<Vec<MigrationOp>> {
        let mut operations = Vec::new();

        // Analyze additions
        for addition in &evolution.additions {
            match addition {
                Statement::HasField(field) => {
                    let default_value = if let Some(default) = &field.default {
                        format!("{:?}", default)
                    } else {
                        self.default_value_for_type(&format!("{:?}", field.type_))
                    };

                    operations.push(MigrationOp::AddField {
                        name: field.name.clone(),
                        type_name: format!("{:?}", field.type_),
                        default_value,
                    });
                }
                _ => {
                    return Err(MigrationCodegenError::UnsupportedOperation(
                        "Only HasField additions are supported".to_string(),
                    ))
                }
            }
        }

        // Analyze removals
        for removal in &evolution.removals {
            operations.push(MigrationOp::RemoveField {
                name: removal.clone(),
            });
        }

        // TODO: Parse rationale for rename operations
        // For now, we'd need to parse the rationale string to detect renames

        Ok(operations)
    }

    /// Generate code for migration operations.
    fn generate_operations(&self, operations: &[MigrationOp]) -> Result<TokenStream> {
        let mut op_tokens = Vec::new();

        for op in operations {
            let tokens = match op {
                MigrationOp::AddField {
                    name,
                    type_name: _,
                    default_value,
                } => {
                    let field_name = name.as_str();
                    let default = self.parse_default_value(default_value)?;

                    quote! {
                        // Add field: #field_name
                        if tx.get(&ROOT, #field_name)?.is_none() {
                            tx.put(&ROOT, #field_name, #default)?;
                        }
                    }
                }
                MigrationOp::RemoveField { name } => {
                    let field_name = name.as_str();
                    quote! {
                        // Remove field: #field_name
                        tx.delete(&ROOT, #field_name)?;
                    }
                }
                MigrationOp::RenameField { old_name, new_name } => {
                    let old_field = old_name.as_str();
                    let new_field = new_name.as_str();
                    quote! {
                        // Rename field: #old_field -> #new_field
                        if let Some((value, _)) = tx.get(&ROOT, #old_field)? {
                            tx.put(&ROOT, #new_field, value)?;
                            tx.delete(&ROOT, #old_field)?;
                        }
                    }
                }
                MigrationOp::ChangeType {
                    name,
                    old_type: _,
                    new_type: _,
                } => {
                    return Err(MigrationCodegenError::UnsupportedOperation(
                        format!("Type changes not yet supported for field: {}", name),
                    ))
                }
            };

            op_tokens.push(tokens);
        }

        Ok(quote! {
            #(#op_tokens)*
        })
    }

    /// Generate metadata code.
    fn generate_metadata(&self, name: &str, from: &str, to: &str) -> TokenStream {
        quote! {
            use std::sync::OnceLock;
            static METADATA: OnceLock<vudo_state::MigrationMetadata> = OnceLock::new();

            METADATA.get_or_init(|| {
                vudo_state::MigrationMetadata::new(
                    #name.to_string(),
                    semver::Version::parse(#from).unwrap(),
                    semver::Version::parse(#to).unwrap(),
                )
            })
        }
    }

    /// Generate migration struct name.
    fn generate_migration_name(&self, gen_name: &str, from: &str, to: &str) -> String {
        // Convert "user.profile" to "UserProfile"
        let struct_name = gen_name
            .split('.')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join("");

        // Convert "1.0.0" to "V1", "2.0.0" to "V2"
        let from_short = format!("V{}", from.split('.').next().unwrap_or("0"));
        let to_short = format!("V{}", to.split('.').next().unwrap_or("0"));

        format!("{}{}To{}", struct_name, from_short, to_short)
    }

    /// Parse version string.
    fn parse_version(&self, version: &str) -> Result<String> {
        // Validate version format
        semver::Version::parse(version).map_err(|e| {
            MigrationCodegenError::InvalidVersion(format!("Invalid version {}: {}", version, e))
        })?;

        Ok(version.to_string())
    }

    /// Get default value for a type.
    fn default_value_for_type(&self, type_name: &str) -> String {
        match type_name {
            "String" => r#""""#.to_string(),
            "I32" | "I64" | "U32" | "U64" => "0".to_string(),
            "F32" | "F64" => "0.0".to_string(),
            "Bool" => "false".to_string(),
            _ => r#""""#.to_string(), // Default to empty string
        }
    }

    /// Parse default value string to token stream.
    fn parse_default_value(&self, value: &str) -> Result<TokenStream> {
        // Simple parsing for common types
        if value.starts_with('"') && value.ends_with('"') {
            // String literal
            Ok(quote! { #value })
        } else if value == "true" || value == "false" {
            // Boolean
            let bool_val = value.parse::<bool>().unwrap();
            Ok(quote! { #bool_val })
        } else if let Ok(int_val) = value.parse::<i64>() {
            // Integer
            Ok(quote! { #int_val })
        } else if let Ok(float_val) = value.parse::<f64>() {
            // Float
            Ok(quote! { #float_val })
        } else {
            // Unknown - use as string
            Ok(quote! { #value })
        }
    }
}

impl Default for MigrationCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate migration code for a DOL evolution.
///
/// This is the main entry point for migration code generation.
pub fn generate_migration(evolution: &Evo) -> Result<String> {
    let codegen = MigrationCodegen::new();
    let tokens = codegen.generate(evolution)?;
    Ok(tokens.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dol::ast::{Evo, HasField, Span, TypeExpr};

    fn create_test_evolution() -> Evo {
        Evo {
            name: "user.profile".to_string(),
            version: "2.0.0".to_string(),
            parent_version: "1.0.0".to_string(),
            additions: vec![Statement::HasField(Box::new(HasField {
                name: "email".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: Some(dol::ast::Expr::Literal(dol::ast::Literal::String(
                    "".to_string(),
                ))),
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            }))],
            deprecations: vec![],
            removals: vec!["legacy_id".to_string()],
            rationale: Some("Add email support, remove legacy ID".to_string()),
            exegesis: "Version 2.0.0 adds email field.".to_string(),
            span: Span::default(),
        }
    }

    #[test]
    fn test_migration_name_generation() {
        let codegen = MigrationCodegen::new();
        let name = codegen.generate_migration_name("user.profile", "1.0.0", "2.0.0");
        assert_eq!(name, "UserProfileV1ToV2");
    }

    #[test]
    fn test_version_parsing() {
        let codegen = MigrationCodegen::new();
        assert!(codegen.parse_version("1.0.0").is_ok());
        assert!(codegen.parse_version("invalid").is_err());
    }

    #[test]
    fn test_default_values() {
        let codegen = MigrationCodegen::new();
        assert_eq!(codegen.default_value_for_type("String"), r#""""#);
        assert_eq!(codegen.default_value_for_type("I32"), "0");
        assert_eq!(codegen.default_value_for_type("Bool"), "false");
    }

    #[test]
    fn test_analyze_evolution() {
        let codegen = MigrationCodegen::new();
        let evolution = create_test_evolution();
        let ops = codegen.analyze_evolution(&evolution).unwrap();

        assert_eq!(ops.len(), 2); // Add + Remove
        assert!(matches!(ops[0], MigrationOp::AddField { .. }));
        assert!(matches!(ops[1], MigrationOp::RemoveField { .. }));
    }

    #[test]
    fn test_generate_migration() {
        let evolution = create_test_evolution();
        let result = generate_migration(&evolution);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("UserProfileV1ToV2"));
        assert!(code.contains("migrate"));
        assert!(code.contains("email"));
    }
}
