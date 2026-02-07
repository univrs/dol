//! Error types for procedural macros.

use std::fmt;

/// Result type for procedural macro operations.
pub type ProcMacroResult<T> = Result<T, ProcMacroError>;

/// Errors that can occur during procedural macro execution.
#[derive(Debug, Clone)]
pub struct ProcMacroError {
    /// Error message
    pub message: String,
    /// Optional span information
    pub span: Option<proc_macro2::Span>,
}

impl ProcMacroError {
    /// Creates a new procedural macro error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    /// Creates an error with span information.
    pub fn with_span(message: impl Into<String>, span: proc_macro2::Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates an error for unsupported features.
    pub fn unsupported(feature: &str) -> Self {
        Self::new(format!("unsupported feature: {}", feature))
    }

    /// Creates an error for invalid input.
    pub fn invalid_input(msg: &str) -> Self {
        Self::new(format!("invalid input: {}", msg))
    }

    /// Creates an error for missing attributes.
    pub fn missing_attribute(attr: &str) -> Self {
        Self::new(format!("missing required attribute: {}", attr))
    }

    /// Creates an error for invalid attributes.
    pub fn invalid_attribute(attr: &str, msg: &str) -> Self {
        Self::new(format!("invalid attribute '{}': {}", attr, msg))
    }

    /// Converts this error into a compile error token stream.
    pub fn to_compile_error(&self) -> proc_macro2::TokenStream {
        let msg = &self.message;
        quote::quote! {
            compile_error!(#msg);
        }
    }
}

impl fmt::Display for ProcMacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "procedural macro error: {}", self.message)
    }
}

impl std::error::Error for ProcMacroError {}

impl From<syn::Error> for ProcMacroError {
    fn from(err: syn::Error) -> Self {
        Self::new(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ProcMacroError::new("test error");
        assert_eq!(err.message, "test error");
        assert!(err.span.is_none());
    }

    #[test]
    fn test_unsupported_error() {
        let err = ProcMacroError::unsupported("async");
        assert!(err.message.contains("unsupported"));
        assert!(err.message.contains("async"));
    }

    #[test]
    fn test_invalid_input_error() {
        let err = ProcMacroError::invalid_input("expected struct");
        assert!(err.message.contains("invalid input"));
    }

    #[test]
    fn test_error_display() {
        let err = ProcMacroError::new("test");
        let display = format!("{}", err);
        assert!(display.contains("procedural macro error"));
    }
}
