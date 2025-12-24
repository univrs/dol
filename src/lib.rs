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
//!   container has status
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
//! - [`typechecker`]: DOL 2.0 type inference and checking
//! - [`eval`]: Expression evaluation for DOL 2.0
//! - [`macros`]: Macro system for compile-time metaprogramming
//! - [`transform`]: AST transformation framework with passes
//! - [`codegen`]: Code generation from DOL declarations
//! - [`sex`]: Side Effect eXecution system for purity tracking
//! - [`mcp`]: Model Context Protocol server (requires `serde` feature)
//! - [`mlir`]: MLIR code generation backend (requires `mlir` feature)
//! - [`wasm`]: WebAssembly compilation and runtime (requires `wasm` feature)

#![doc(html_root_url = "https://docs.rs/metadol/0.0.1")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod ast;
pub mod codegen;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod macros;
pub mod parser;
pub mod pratt;
pub mod reflect;
pub mod sex;
pub mod transform;
pub mod typechecker;
pub mod validator;

// MCP server (requires serde feature)
#[cfg(feature = "serde")]
pub mod mcp;

// MLIR backend (requires mlir feature)
pub mod mlir;

// WASM backend (requires wasm feature)
#[cfg(feature = "wasm")]
pub mod wasm;

// Test file parser for .dol.test files
#[cfg(feature = "cli")]
pub mod test_parser;

// Re-exports for convenience
pub use ast::{Constraint, Declaration, Evolution, Gene, Span, Statement, System, Trait};
pub use error::{LexError, ParseError, ValidationError};
pub use eval::{EvalError, Interpreter, Value};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
pub use typechecker::{Type, TypeChecker, TypeEnv, TypeError};
pub use validator::{validate, ValidationResult};

// Codegen re-exports
pub use codegen::{
    Codegen, CodegenOptions, JsonSchemaCodegen, RustCodegen, TypeMapper, TypeScriptCodegen,
    Visibility,
};

// Macro system re-exports
pub use macros::{
    AttributeArg, BuiltinMacros, Macro, MacroAttribute, MacroContext, MacroError, MacroExpander,
    MacroInput, MacroInvocation, MacroOutput,
};

// Transform framework re-exports
pub use transform::{
    ConstantFolding, DeadCodeElimination, Fold, MutVisitor, Pass, PassConfig, PassError,
    PassPipeline, PassResult, PassStats, Visitor,
};

// Reflection system re-exports
pub use reflect::{FieldInfo, MethodInfo, TypeInfo, TypeKind, TypeRegistry};

// SEX (Side Effect eXecution) system re-exports
pub use sex::{
    file_sex_context, is_sex_file, EffectTracker, FileContext, LintResult, SexContext,
    SexLintError, SexLintWarning, SexLinter,
};

// MLIR backend re-exports (requires mlir feature)
#[cfg(feature = "mlir")]
pub use mlir::{CodegenError, CodegenResult, MlirCodegen, MlirContext};

// Always export MlirError since it doesn't require the mlir feature
pub use mlir::MlirError;

// WASM backend re-exports (requires wasm feature)
#[cfg(feature = "wasm")]
pub use wasm::{WasmCompiler, WasmError, WasmModule, WasmRuntime};

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
/// gene example.thing {
///   thing has property
/// }
///
/// exegesis {
///   Example gene for documentation.
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
/// gene example.thing {
///   thing has property
/// }
///
/// exegesis {
///   Example gene for documentation.
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
        // DOL 2.0 tolerant: missing exegesis defaults to empty string
        let source = r#"
gene container.exists {
  container has identity
}
"#;
        let result = parse_file(source);
        assert!(result.is_ok());
        // Exegesis will be empty when not provided
    }
}
