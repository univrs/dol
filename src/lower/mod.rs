//! AST -> HIR Lowering
//!
//! Converts surface syntax AST to canonical HIR.
//! All desugaring happens here.
//!
//! # Desugaring Rules
//!
//! ## Bindings
//! - `let x = e` -> `Val { name: x, value: e }` (deprecated, emits warning)
//! - `val x = e` -> `Val { name: x, value: e }` (new in v0.3.0)
//! - `let mut x = e` -> `Var { name: x, value: e }` (deprecated)
//! - `var x = e` -> `Var { name: x, value: e }` (new in v0.3.0)
//!
//! ## Control Flow
//! - `for x in xs { body }` -> `HirExpr::For` (pattern, iterable, body)
//! - `while cond { body }` -> `HirExpr::While`; `loop { body }` -> `HirExpr::Loop`
//! - `if a { b }` -> `if a { b } else { () }`
//!
//! ## Operators
//! - `x |> f |> g` -> `g(f(x))`
//! - `a && b` -> `if a { b } else { false }`
//! - `a || b` -> `if a { true } else { b }`
//!
//! ## Quantifiers
//! - `each x in xs` -> `forall x in xs` (deprecated)
//! - `all x in xs` -> `forall x in xs` (deprecated)
//!
//! ## Types
//! - `gene Foo { }` -> `HirDecl::Type { }` (supported)
//! - `type Foo { }` -> `HirDecl::Type { }` (preferred)

mod context;
mod decl;
mod desugar;
mod expr;
mod stmt;

pub use context::LoweringContext;
pub use desugar::{lower_file, lower_module};

/// Diagnostic message from lowering
#[derive(Debug, Clone)]
pub struct LowerDiagnostic {
    /// Kind of diagnostic
    pub kind: DiagnosticKind,
    /// Diagnostic message
    pub message: String,
    /// Source span where the diagnostic occurred
    pub span: Option<crate::ast::Span>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Kind of diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Deprecated syntax that still works
    Deprecation,
    /// Warning about potential issues
    Warning,
    /// Error that prevents lowering
    Error,
}

impl std::fmt::Display for LowerDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.kind {
            DiagnosticKind::Deprecation => "deprecated",
            DiagnosticKind::Warning => "warning",
            DiagnosticKind::Error => "error",
        };
        write!(f, "{}: {}", prefix, self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display() {
        let diag = LowerDiagnostic {
            kind: DiagnosticKind::Deprecation,
            message: "'let' is deprecated".to_string(),
            span: None,
            suggestion: Some("use 'val' instead".to_string()),
        };
        let s = format!("{}", diag);
        assert!(s.contains("deprecated"));
        assert!(s.contains("let"));
        assert!(s.contains("val"));
    }

    #[test]
    fn test_diagnostic_kind_eq() {
        assert_eq!(DiagnosticKind::Error, DiagnosticKind::Error);
        assert_ne!(DiagnosticKind::Error, DiagnosticKind::Warning);
    }
}
