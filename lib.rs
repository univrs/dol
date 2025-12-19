//! # Metal DOL - Design Ontology Language
//!
//! Metal DOL is a declarative specification language that serves as the
//! source of truth for ontology-first software development.
//!
//! ## Overview
//!
//! In the DOL-first paradigm, design contracts are written before tests,
//! and tests are written before code. Nothing changes without first being
//! declared in the ontology.
//!
//! ```text
//! Traditional:  Code → Tests → Documentation
//! DOL-First:    Design Ontology → Tests → Code
//! ```
//!
//! ## Quick Start
//!
//! Parse a DOL file:
//!
//! ```rust
//! use metadol::{Parser, parse_file};
//!
//! let input = r#"
//! gene container.exists {
//!   container has identity
//!   container has state
//! }
//!
//! exegesis {
//!   A container is the fundamental unit of workload isolation.
//! }
//! "#;
//!
//! let result = parse_file(input);
//! assert!(result.is_ok());
//! ```
//!
//! ## Core Concepts
//!
//! - **Gene**: Atomic unit declaring fundamental truths
//! - **Trait**: Composable behaviors built from genes
//! - **Constraint**: Invariants that must always hold
//! - **System**: Top-level composition of a complete subsystem
//! - **Evolution**: Lineage record of ontology changes
//!
//! ## Modules
//!
//! - [`ast`]: Abstract Syntax Tree definitions
//! - [`lexer`]: Tokenization of DOL source text
//! - [`parser`]: Recursive descent parser producing AST
//! - [`error`]: Error types with source location information
//! - [`validator`]: Semantic validation rules

#![doc(html_root_url = "https://docs.rs/metadol/0.0.1")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod validator;

#[cfg(feature = "codegen")]
pub mod codegen;

#[cfg(feature = "codegen")]
pub mod test_parser;

// Re-exports for convenience
pub use ast::{Declaration, Gene, Trait, Constraint, System, Evolution, Statement, Span};
pub use error::{LexError, ParseError, ValidationError};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
pub use validator::{validate, ValidationResult};

/// Parse a DOL source string into an AST.
///
/// This is the primary entry point for parsing DOL files.
///
/// # Arguments
///
/// * `source` - The DOL source text to parse
///
/// # Returns
///
/// A `Declaration` AST node on success, or a `ParseError` on failure.
///
/// # Example
///
/// ```rust
/// use metadol::parse_file;
///
/// let source = r#"
/// gene test.example {
///   test has property
/// }
///
/// exegesis {
///   Example gene for testing.
/// }
/// "#;
///
/// let result = parse_file(source);
/// assert!(result.is_ok());
/// ```
pub fn parse_file(source: &str) -> Result<Declaration, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse()
}

/// Parse and validate a DOL source string.
///
/// Combines parsing and validation into a single operation.
///
/// # Arguments
///
/// * `source` - The DOL source text to parse and validate
///
/// # Returns
///
/// A tuple of the parsed `Declaration` and `ValidationResult` on success,
/// or a `ParseError` if parsing fails.
///
/// # Example
///
/// ```rust
/// use metadol::parse_and_validate;
///
/// let source = r#"
/// gene test.example {
///   test has property
/// }
///
/// exegesis {
///   Example gene for testing.
/// }
/// "#;
///
/// let (decl, validation) = parse_and_validate(source)?;
/// assert!(validation.is_valid());
/// # Ok::<(), metadol::ParseError>(())
/// ```
pub fn parse_and_validate(source: &str) -> Result<(Declaration, ValidationResult), ParseError> {
    let decl = parse_file(source)?;
    let validation = validate(&decl);
    Ok((decl, validation))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_gene() {
        let source = r#"
gene container.exists {
  container has identity
}

exegesis {
  A container is the fundamental unit.
}
"#;
        let result = parse_file(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_missing_exegesis() {
        let source = r#"
gene container.exists {
  container has identity
}
"#;
        let result = parse_file(source);
        assert!(result.is_err());
    }
}
