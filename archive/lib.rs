//! # dol-parse
//!
//! Metal DOL (Design Ontology Language) parser and AST.
//!
//! DOL is a specification language for declaring system behavior, constraints,
//! and evolution. This crate provides:
//!
//! - Lexer for tokenizing DOL source
//! - Parser for building Abstract Syntax Trees
//! - LLVM-style diagnostics for error reporting
//! - Repository management for multi-file projects
//!
//! ## Example
//!
//! ```rust
//! use dol_parse::{parse, DolFile};
//!
//! let source = r#"
//!     gene container.exists {
//!         container has identity
//!         container has state
//!         container has boundaries
//!     }
//!
//!     exegesis {
//!         A container is the fundamental unit of isolation.
//!     }
//! "#;
//!
//! let file = parse(source).expect("Parse error");
//! println!("Parsed: {}", file.declaration.name());
//! ```
//!
//! ## Design Philosophy
//!
//! DOL embodies the principle that specification precedes implementation.
//! The parser is designed to:
//!
//! - Produce rich AST nodes with full source location tracking
//! - Generate LLVM-quality diagnostic messages
//! - Support incremental parsing for IDE integration
//! - Enable code generation for tests and runtime checks

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod diagnostics;

// Re-exports for convenience
pub use ast::{
    DolFile, DolRepository, Declaration, Gene, Trait, Constraint, System, 
    Evolves, Test, QualifiedName, Version, Statement, Exegesis, Span,
};
pub use lexer::{Token, tokenize};
pub use parser::{parse, parse_repository, ParseError};
pub use diagnostics::{Diagnostic, DiagnosticCollector, Severity};

/// Parse a DOL source file
///
/// # Example
///
/// ```rust
/// use dol_parse::parse_file;
///
/// let source = "gene node.exists { node has identity }";
/// let file = parse_file(source, Some("node.dol")).unwrap();
/// ```
pub fn parse_file(source: &str, path: Option<&str>) -> Result<DolFile, ParseError> {
    let mut file = parse(source)?;
    file.path = path.map(|p| p.to_string());
    Ok(file)
}

/// Validate a DOL file and collect diagnostics
pub fn validate(file: &DolFile) -> DiagnosticCollector {
    let mut collector = DiagnosticCollector::new();
    
    // Validate declaration
    match &file.declaration {
        Declaration::Gene(g) => validate_gene(g, &mut collector),
        Declaration::Trait(t) => validate_trait(t, &mut collector),
        Declaration::Constraint(c) => validate_constraint(c, &mut collector),
        Declaration::System(s) => validate_system(s, &mut collector),
        Declaration::Evolves(e) => validate_evolves(e, &mut collector),
        Declaration::Test(t) => validate_test(t, &mut collector),
    }
    
    // Check for exegesis
    if file.exegesis.is_none() {
        collector.warning(
            "Missing exegesis section",
            file.span,
        );
    }
    
    collector
}

fn validate_gene(gene: &Gene, collector: &mut DiagnosticCollector) {
    if gene.statements.is_empty() {
        collector.warning("Gene has no statements", gene.span);
    }
    
    // Validate naming convention
    if gene.name.segments.len() < 2 {
        collector.warning(
            "Gene name should follow domain.property pattern",
            gene.name.span,
        );
    }
}

fn validate_trait(t: &Trait, collector: &mut DiagnosticCollector) {
    if t.uses.is_empty() && t.statements.is_empty() {
        collector.warning("Trait has no composition or statements", t.span);
    }
}

fn validate_constraint(c: &Constraint, collector: &mut DiagnosticCollector) {
    if c.statements.is_empty() {
        collector.warning("Constraint has no invariants", c.span);
    }
}

fn validate_system(s: &System, collector: &mut DiagnosticCollector) {
    if s.requires.is_empty() {
        collector.warning("System has no requirements", s.span);
    }
}

fn validate_evolves(e: &Evolves, collector: &mut DiagnosticCollector) {
    // Check version ordering
    if e.version <= e.from {
        collector.error(
            format!("Evolution version {} must be greater than {}", e.version, e.from),
            e.span,
        );
    }
    
    if e.changes.is_empty() {
        collector.warning("Evolution has no changes", e.span);
    }
    
    if e.because.is_none() {
        collector.warning("Evolution should include 'because' rationale", e.span);
    }
}

fn validate_test(t: &Test, collector: &mut DiagnosticCollector) {
    if t.given.is_empty() {
        collector.warning("Test has no preconditions (given)", t.span);
    }
    
    if t.when.is_empty() {
        collector.warning("Test has no actions (when)", t.span);
    }
    
    if t.then.is_empty() {
        collector.warning("Test has no assertions (then)", t.span);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_parse_and_validate() {
        let source = r#"
            gene container.exists {
                container has identity
                container has state
            }

            exegesis {
                Containers are isolated execution environments.
            }
        "#;

        let file = parse_file(source, Some("container.dol")).unwrap();
        let diagnostics = validate(&file);
        
        assert!(!diagnostics.has_errors());
    }

    #[test]
    fn missing_exegesis_warning() {
        let source = r#"
            gene container.exists {
                container has identity
            }
        "#;

        let file = parse_file(source, None).unwrap();
        let diagnostics = validate(&file);
        
        assert_eq!(diagnostics.warning_count(), 1);
    }
}
