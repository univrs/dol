//! Code generation from Metal DOL declarations.
//!
//! This module provides code generation capabilities for transforming DOL
//! declarations into executable code in various target languages.
//!
//! # Supported Targets
//!
//! - **Rust**: Generate structs, traits, and type aliases
//! - **TypeScript**: Generate interfaces and type definitions
//! - **JSON Schema**: Generate JSON Schema from types (planned)
//!
//! # Example
//!
//! ```rust
//! use metadol::{parse_file, codegen::RustCodegen};
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
//! let rust_code = RustCodegen::generate(&decl);
//! println!("{}", rust_code);
//! ```

mod rust;
mod typescript;

pub use rust::RustCodegen;
pub use typescript::TypeScriptCodegen;

use crate::ast::{Declaration, TypeExpr};
use crate::typechecker::Type;

/// Trait for code generation backends.
///
/// Implement this trait to add support for new target languages.
pub trait Codegen {
    /// Generate code from a DOL declaration.
    fn generate(decl: &Declaration) -> String;

    /// Generate code from multiple declarations.
    fn generate_all(decls: &[Declaration]) -> String {
        decls
            .iter()
            .map(Self::generate)
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

/// Configuration options for code generation.
#[derive(Debug, Clone, Default)]
pub struct CodegenOptions {
    /// Include documentation comments in output
    pub include_docs: bool,

    /// Generate derive macros (Rust-specific)
    pub derive_macros: Vec<String>,

    /// Visibility modifier for generated items
    pub visibility: Visibility,

    /// Generate builder pattern methods
    pub generate_builders: bool,
}

/// Visibility level for generated code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Private (no modifier)
    Private,
    /// Public
    #[default]
    Public,
    /// Crate-visible (Rust pub(crate))
    Crate,
}

/// Convert a DOL type to a target language type string.
pub trait TypeMapper {
    /// Map a DOL Type to the target language type string.
    fn map_type(ty: &Type) -> String;

    /// Map a DOL TypeExpr to the target language type string.
    fn map_type_expr(ty: &TypeExpr) -> String;
}

/// Convert a DOL identifier to a valid identifier in the target language.
pub fn to_pascal_case(s: &str) -> String {
    s.split('.')
        .flat_map(|part| part.split('_'))
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Convert a DOL identifier to snake_case.
pub fn to_snake_case(s: &str) -> String {
    s.split('.')
        .collect::<Vec<_>>()
        .join("_")
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                vec!['_', c.to_lowercase().next().unwrap()]
            } else {
                vec![c.to_lowercase().next().unwrap()]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("container.exists"), "ContainerExists");
        assert_eq!(
            to_pascal_case("identity.cryptographic"),
            "IdentityCryptographic"
        );
        assert_eq!(to_pascal_case("simple"), "Simple");
        assert_eq!(to_pascal_case("snake_case"), "SnakeCase");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("container.exists"), "container_exists");
        assert_eq!(to_snake_case("ContainerExists"), "container_exists");
        assert_eq!(to_snake_case("simple"), "simple");
    }
}
