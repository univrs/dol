//! # MLIR Code Generation
//!
//! This module provides MLIR (Multi-Level Intermediate Representation) code
//! generation capabilities for DOL declarations. MLIR enables high-performance
//! compilation targets including WASM, LLVM, and specialized hardware backends.
//!
//! ## Overview
//!
//! The MLIR infrastructure transforms DOL's high-level declarations into
//! executable code by:
//!
//! 1. **Type Lowering**: Converting DOL types to MLIR types
//! 2. **Operation Generation**: Mapping DOL semantics to MLIR operations
//! 3. **Optimization**: Applying MLIR's dialect-based optimization passes
//! 4. **Target Generation**: Lowering to platform-specific targets
//!
//! ## Architecture
//!
//! ```text
//! DOL AST → MLIR Dialect → Standard MLIR → Target (LLVM/WASM)
//!     ↓           ↓              ↓               ↓
//!   Types      Operations    Optimization   Code Output
//! ```
//!
//! ## Feature Flags
//!
//! This module requires the `mlir` feature flag:
//!
//! ```toml
//! [dependencies]
//! metadol = { version = "0.0.1", features = ["mlir"] }
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use metadol::mlir::OpBuilder;
//! use melior::Context;
//!
//! // Create MLIR context and operation builder
//! let context = Context::new();
//! let builder = OpBuilder::new(&context);
//!
//! // Build MLIR operations from DOL AST nodes
//! let location = Location::unknown(&context);
//! let add_op = builder.build_binary_arith(BinaryOp::Add, lhs, rhs, location)?;
//! ```
//!
//! ## Modules
//!
//! - [`ops`]: MLIR operation builders for arithmetic, control flow, and functions

// Conditionally compile MLIR modules only when feature is enabled
#[cfg(feature = "mlir")]
pub mod ops;

// Re-export public types when MLIR feature is enabled
#[cfg(feature = "mlir")]
pub use ops::OpBuilder;

use crate::ast::Span;
use std::fmt;

/// Error type for MLIR code generation operations.
///
/// This error type captures issues that can occur during MLIR generation,
/// type lowering, optimization passes, and target code emission.
#[derive(Debug, Clone, PartialEq)]
pub struct MlirError {
    /// Human-readable error message
    pub message: String,
    /// Optional source location where the error occurred
    pub span: Option<Span>,
}

impl MlirError {
    /// Creates a new MLIR error with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of the error
    ///
    /// # Example
    ///
    /// ```
    /// use metadol::mlir::MlirError;
    ///
    /// let err = MlirError::new("unsupported type");
    /// assert_eq!(err.message, "unsupported type");
    /// assert!(err.span.is_none());
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    /// Creates a new MLIR error with a message and source location.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of the error
    /// * `span` - Source location where the error occurred
    ///
    /// # Example
    ///
    /// ```
    /// use metadol::mlir::MlirError;
    /// use metadol::ast::Span;
    ///
    /// let span = Span { start: 10, end: 20 };
    /// let err = MlirError::with_span("type mismatch", span);
    /// assert_eq!(err.message, "type mismatch");
    /// assert_eq!(err.span, Some(span));
    /// ```
    pub fn with_span(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

impl fmt::Display for MlirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = &self.span {
            write!(
                f,
                "MLIR error at {}..{}: {}",
                span.start, span.end, self.message
            )
        } else {
            write!(f, "MLIR error: {}", self.message)
        }
    }
}

impl std::error::Error for MlirError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlir_error_new() {
        let err = MlirError::new("test error");
        assert_eq!(err.message, "test error");
        assert!(err.span.is_none());
    }

    #[test]
    fn test_mlir_error_with_span() {
        let span = Span {
            start: 5,
            end: 10,
            line: 1,
            column: 1,
        };
        let err = MlirError::with_span("test error", span);
        assert_eq!(err.message, "test error");
        assert_eq!(err.span, Some(span));
    }

    #[test]
    fn test_mlir_error_display() {
        let err = MlirError::new("no span");
        assert_eq!(err.to_string(), "MLIR error: no span");

        let span = Span {
            start: 5,
            end: 10,
            line: 1,
            column: 1,
        };
        let err = MlirError::with_span("with span", span);
        assert_eq!(err.to_string(), "MLIR error at 5..10: with span");
    }
}
