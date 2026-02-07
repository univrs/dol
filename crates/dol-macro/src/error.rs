//! Error types for the DOL macro system.
//!
//! This module defines error types that can occur during macro
//! definition, expansion, and pattern matching.

use metadol::ast::Span;
use std::fmt;

/// Result type for macro operations.
pub type MacroResult<T> = Result<T, MacroError>;

/// Errors that can occur during macro operations.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroError {
    /// The kind of error
    pub kind: MacroErrorKind,
    /// Error message
    pub message: String,
    /// Source location where the error occurred
    pub span: Option<Span>,
}

/// Different kinds of macro errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroErrorKind {
    /// Macro not found in registry
    UndefinedMacro,
    /// Pattern matching failed
    PatternMismatch,
    /// Invalid number of arguments
    ArityMismatch,
    /// Type mismatch in pattern
    TypeMismatch,
    /// Invalid pattern syntax
    InvalidPattern,
    /// Invalid macro definition
    InvalidDefinition,
    /// Expansion depth exceeded
    RecursionLimit,
    /// Hygiene violation
    HygieneViolation,
    /// Ambiguous macro invocation
    AmbiguousInvocation,
    /// Invalid fragment specifier
    InvalidFragment,
    /// Syntax error in macro body
    SyntaxError,
}

impl MacroError {
    /// Creates a new macro error.
    pub fn new(kind: MacroErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a macro error with source location.
    pub fn with_span(kind: MacroErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates an undefined macro error.
    pub fn undefined(name: &str) -> Self {
        Self::new(
            MacroErrorKind::UndefinedMacro,
            format!("undefined macro: {}", name),
        )
    }

    /// Creates a pattern mismatch error.
    pub fn pattern_mismatch(expected: &str, actual: &str) -> Self {
        Self::new(
            MacroErrorKind::PatternMismatch,
            format!("pattern mismatch: expected {}, got {}", expected, actual),
        )
    }

    /// Creates an arity mismatch error.
    pub fn arity_mismatch(expected: usize, actual: usize) -> Self {
        Self::new(
            MacroErrorKind::ArityMismatch,
            format!("expected {} argument(s), got {}", expected, actual),
        )
    }

    /// Creates a type mismatch error.
    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        Self::new(
            MacroErrorKind::TypeMismatch,
            format!("type mismatch: expected {}, got {}", expected, actual),
        )
    }

    /// Creates an invalid pattern error.
    pub fn invalid_pattern(msg: &str) -> Self {
        Self::new(MacroErrorKind::InvalidPattern, msg)
    }

    /// Creates a recursion limit error.
    pub fn recursion_limit(depth: usize) -> Self {
        Self::new(
            MacroErrorKind::RecursionLimit,
            format!("maximum macro expansion depth ({}) exceeded", depth),
        )
    }

    /// Creates a hygiene violation error.
    pub fn hygiene_violation(msg: &str) -> Self {
        Self::new(MacroErrorKind::HygieneViolation, msg)
    }

    /// Creates an ambiguous invocation error.
    pub fn ambiguous_invocation(msg: &str) -> Self {
        Self::new(MacroErrorKind::AmbiguousInvocation, msg)
    }

    /// Creates an invalid fragment error.
    pub fn invalid_fragment(fragment: &str) -> Self {
        Self::new(
            MacroErrorKind::InvalidFragment,
            format!("invalid fragment specifier: {}", fragment),
        )
    }

    /// Creates a syntax error.
    pub fn syntax_error(msg: &str) -> Self {
        Self::new(MacroErrorKind::SyntaxError, msg)
    }
}

impl fmt::Display for MacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = &self.span {
            write!(
                f,
                "macro error at line {}, column {}: {}",
                span.line, span.column, self.message
            )
        } else {
            write!(f, "macro error: {}", self.message)
        }
    }
}

impl std::error::Error for MacroError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = MacroError::undefined("test_macro");
        assert_eq!(err.kind, MacroErrorKind::UndefinedMacro);
        assert!(err.message.contains("undefined macro"));

        let err = MacroError::arity_mismatch(2, 1);
        assert_eq!(err.kind, MacroErrorKind::ArityMismatch);
        assert!(err.message.contains("expected 2"));
    }

    #[test]
    fn test_error_with_span() {
        let span = Span::new(0, 10, 1, 1);
        let err = MacroError::with_span(
            MacroErrorKind::PatternMismatch,
            "test error",
            span,
        );
        assert!(err.span.is_some());
        assert_eq!(err.span.unwrap(), span);
    }

    #[test]
    fn test_error_display() {
        let err = MacroError::undefined("my_macro");
        let display = format!("{}", err);
        assert!(display.contains("undefined macro"));
    }
}
