//! AST Transformation Framework for DOL 2.0.
//!
//! This module provides infrastructure for transforming DOL ASTs through
//! a series of passes. It includes visitor patterns for traversal and
//! fold patterns for transformation.
//!
//! # Architecture
//!
//! - **Pass**: A single transformation or analysis pass
//! - **PassPipeline**: Chains multiple passes together
//! - **Visitor**: Immutable traversal of AST nodes
//! - **MutVisitor**: Mutable transformation of AST nodes
//! - **Fold**: Expression-level transformation

pub mod fold;
pub mod passes;
pub mod visitor;

pub use fold::Fold;
pub use passes::{ConstantFolding, DeadCodeElimination};
pub use visitor::{MutVisitor, Visitor};

use crate::ast::Declaration;
use std::fmt;

/// Error that can occur during a transformation pass.
#[derive(Debug, Clone)]
pub struct PassError {
    /// Error message
    pub message: String,
    /// Pass that produced the error
    pub pass_name: String,
    /// Optional source location
    pub location: Option<String>,
}

impl PassError {
    /// Creates a new pass error.
    pub fn new(pass_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            pass_name: pass_name.into(),
            location: None,
        }
    }

    /// Adds location information to the error.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

impl fmt::Display for PassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref loc) = self.location {
            write!(f, "[{}] {} at {}", self.pass_name, self.message, loc)
        } else {
            write!(f, "[{}] {}", self.pass_name, self.message)
        }
    }
}

impl std::error::Error for PassError {}

/// Result type for transformation passes.
pub type PassResult<T> = Result<T, PassError>;

/// A transformation or analysis pass over the AST.
pub trait Pass {
    /// Name of this pass for debugging and error messages.
    fn name(&self) -> &str;

    /// Determines if this pass should run on the given declaration.
    fn should_run(&self, _decl: &Declaration) -> bool {
        true
    }

    /// Runs the pass on a declaration, potentially transforming it.
    fn run(&mut self, decl: Declaration) -> PassResult<Declaration>;
}

/// A pipeline of passes to run in sequence.
pub struct PassPipeline {
    passes: Vec<Box<dyn Pass>>,
}

impl PassPipeline {
    /// Creates a new empty pipeline.
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Adds a pass to the pipeline.
    pub fn add<P: Pass + 'static>(&mut self, pass: P) -> &mut Self {
        self.passes.push(Box::new(pass));
        self
    }

    /// Runs all passes on a declaration.
    pub fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        let mut current = decl;
        for pass in &mut self.passes {
            if pass.should_run(&current) {
                current = pass.run(current)?;
            }
        }
        Ok(current)
    }

    /// Runs all passes on a list of declarations.
    pub fn run_all(&mut self, decls: Vec<Declaration>) -> PassResult<Vec<Declaration>> {
        decls.into_iter().map(|d| self.run(d)).collect()
    }
}

impl Default for PassPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration options for passes.
#[derive(Debug, Clone)]
pub struct PassConfig {
    /// Enable debug output
    pub debug: bool,
    /// Maximum iteration count for fixed-point passes
    pub max_iterations: usize,
}

impl Default for PassConfig {
    fn default() -> Self {
        Self {
            debug: false,
            max_iterations: 100,
        }
    }
}

/// Statistics collected during pass execution.
#[derive(Debug, Clone, Default)]
pub struct PassStats {
    /// Number of nodes visited
    pub nodes_visited: usize,
    /// Number of nodes transformed
    pub nodes_transformed: usize,
    /// Number of expressions folded
    pub expressions_folded: usize,
}

impl PassStats {
    /// Creates new empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges stats from another instance.
    pub fn merge(&mut self, other: &PassStats) {
        self.nodes_visited += other.nodes_visited;
        self.nodes_transformed += other.nodes_transformed;
        self.expressions_folded += other.expressions_folded;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CountingPass {
        count: usize,
    }

    impl Pass for CountingPass {
        fn name(&self) -> &str {
            "counting"
        }

        fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
            self.count += 1;
            Ok(decl)
        }
    }

    #[test]
    fn test_pipeline_runs_passes() {
        use crate::ast::{Gene, Span};

        let gene = Gene {
            name: "test".to_string(),
            statements: vec![],
            exegesis: "Test gene".to_string(),
            span: Span::new(0, 0, 1, 1),
        };
        let decl = Declaration::Gene(gene);

        let mut pipeline = PassPipeline::new();
        let pass1 = CountingPass { count: 0 };
        let pass2 = CountingPass { count: 0 };
        pipeline.add(pass1).add(pass2);

        let result = pipeline.run(decl);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pass_error_display() {
        let err = PassError::new("test_pass", "something went wrong");
        assert_eq!(format!("{}", err), "[test_pass] something went wrong");

        let err_with_loc = err.with_location("line 42");
        assert_eq!(
            format!("{}", err_with_loc),
            "[test_pass] something went wrong at line 42"
        );
    }
}
