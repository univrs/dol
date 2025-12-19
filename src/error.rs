//! Error types for Metal DOL.
//!
//! This module defines all error types used throughout the crate,
//! providing rich error information including source locations.
//!
//! # Error Categories
//!
//! - [`LexError`]: Errors during tokenization
//! - [`ParseError`]: Errors during parsing
//! - [`ValidationError`]: Errors during semantic validation
//!
//! # Example
//!
//! ```rust
//! use metadol::error::{ParseError, Span};
//!
//! let error = ParseError::UnexpectedToken {
//!     expected: "identifier".to_string(),
//!     found: "keyword 'gene'".to_string(),
//!     span: Span::new(10, 14, 1, 11),
//! };
//!
//! // Error messages include location information
//! assert!(error.to_string().contains("expected identifier"));
//! ```

use crate::ast::Span;
use thiserror::Error;

/// Errors that can occur during lexical analysis.
///
/// These errors are produced by the [`Lexer`](crate::lexer::Lexer) when
/// it encounters invalid or unexpected input.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexError {
    /// An unexpected character was encountered.
    ///
    /// This typically occurs when the input contains characters that
    /// are not part of the DOL language syntax.
    #[error("unexpected character '{ch}' at line {}, column {}", span.line, span.column)]
    UnexpectedChar {
        /// The unexpected character
        ch: char,
        /// Location in the source
        span: Span,
    },

    /// A string literal was not properly terminated.
    ///
    /// String literals must end with a closing double quote on the same line.
    #[error("unterminated string literal starting at line {}, column {}", span.line, span.column)]
    UnterminatedString {
        /// Location of the opening quote
        span: Span,
    },

    /// An invalid version number was encountered.
    ///
    /// Version numbers must follow semantic versioning format: `X.Y.Z`
    /// where X, Y, and Z are non-negative integers.
    #[error("invalid version number '{text}' at line {}, column {}", span.line, span.column)]
    InvalidVersion {
        /// The invalid version text
        text: String,
        /// Location in the source
        span: Span,
    },

    /// An invalid escape sequence was found in a string.
    #[error("invalid escape sequence '\\{ch}' at line {}, column {}", span.line, span.column)]
    InvalidEscape {
        /// The character after the backslash
        ch: char,
        /// Location of the escape sequence
        span: Span,
    },
}

/// Errors that can occur during parsing.
///
/// These errors are produced by the [`Parser`](crate::parser::Parser) when
/// the token stream does not match the expected grammar.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// An unexpected token was encountered.
    ///
    /// This is the most common parse error, indicating that the parser
    /// expected one token but found another.
    #[error("expected {expected}, found {found} at line {}, column {}", span.line, span.column)]
    UnexpectedToken {
        /// Description of what was expected
        expected: String,
        /// Description of what was found
        found: String,
        /// Location of the unexpected token
        span: Span,
    },

    /// The required exegesis block is missing.
    ///
    /// Every DOL declaration must have an exegesis block explaining
    /// its purpose and context.
    #[error("missing required exegesis block at line {}, column {}", span.line, span.column)]
    MissingExegesis {
        /// Location where exegesis was expected
        span: Span,
    },

    /// A statement uses an invalid predicate or structure.
    #[error("{message} at line {}, column {}", span.line, span.column)]
    InvalidStatement {
        /// Description of the error
        message: String,
        /// Location of the invalid statement
        span: Span,
    },

    /// An invalid declaration type was encountered.
    #[error("invalid declaration type '{found}' at line {}, column {} (expected gene, trait, constraint, system, or evolves)", span.line, span.column)]
    InvalidDeclaration {
        /// The invalid declaration keyword found
        found: String,
        /// Location of the declaration
        span: Span,
    },

    /// Unexpected end of file.
    #[error("unexpected end of file at line {}, column {}: {context}", span.line, span.column)]
    UnexpectedEof {
        /// Context about what was being parsed
        context: String,
        /// Location at end of file
        span: Span,
    },

    /// A lexer error occurred during parsing.
    #[error("lexer error: {0}")]
    LexerError(#[from] LexError),
}

/// Errors that can occur during semantic validation.
///
/// These errors are produced by the [`validator`](crate::validator) when
/// the AST violates semantic rules that cannot be caught during parsing.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// The exegesis block is empty or contains only whitespace.
    #[error("exegesis block is empty at line {}, column {}", span.line, span.column)]
    EmptyExegesis {
        /// Location of the empty exegesis
        span: Span,
    },

    /// An identifier does not follow naming conventions.
    #[error("invalid identifier '{name}': {reason}")]
    InvalidIdentifier {
        /// The invalid identifier
        name: String,
        /// Explanation of what's wrong
        reason: String,
    },

    /// A reference to another declaration could not be resolved.
    #[error("unresolved reference to '{reference}' at line {}, column {}", span.line, span.column)]
    UnresolvedReference {
        /// The unresolved reference
        reference: String,
        /// Location of the reference
        span: Span,
    },

    /// A version number is invalid.
    #[error("invalid version '{version}': {reason}")]
    InvalidVersion {
        /// The invalid version string
        version: String,
        /// Explanation of what's wrong
        reason: String,
    },

    /// A duplicate definition was found.
    #[error("duplicate {kind} '{name}'")]
    DuplicateDefinition {
        /// What kind of thing is duplicated (e.g., "statement", "uses")
        kind: String,
        /// The duplicated name
        name: String,
    },

    /// An evolution references a non-existent parent version.
    #[error("evolution references non-existent parent version '{parent}' for '{name}'")]
    InvalidEvolutionLineage {
        /// The declaration name
        name: String,
        /// The referenced parent version
        parent: String,
    },
}

/// A collection of validation errors and warnings.
///
/// This struct aggregates multiple validation issues that may be found
/// during a single validation pass.
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    /// Critical errors that must be fixed
    pub errors: Vec<ValidationError>,
    /// Warnings that should be addressed
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationErrors {
    /// Creates a new empty error collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns true if there are no errors or warnings.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Adds an error to the collection.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Adds a warning to the collection.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

/// A non-critical validation warning.
///
/// Warnings indicate potential issues that don't prevent the DOL
/// from being valid but should be reviewed.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationWarning {
    /// The exegesis is unusually short.
    ShortExegesis {
        /// Number of characters in the exegesis
        length: usize,
        /// Location of the exegesis
        span: Span,
    },

    /// An identifier doesn't follow recommended naming conventions.
    NamingConvention {
        /// The identifier in question
        name: String,
        /// Recommended format
        suggestion: String,
    },

    /// A deprecated feature is being used.
    DeprecatedFeature {
        /// Description of the deprecated feature
        feature: String,
        /// Suggested alternative
        alternative: String,
    },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationWarning::ShortExegesis { length, span } => {
                write!(
                    f,
                    "exegesis is unusually short ({} chars) at line {}, column {}",
                    length, span.line, span.column
                )
            }
            ValidationWarning::NamingConvention { name, suggestion } => {
                write!(
                    f,
                    "identifier '{}' doesn't follow naming convention; consider: {}",
                    name, suggestion
                )
            }
            ValidationWarning::DeprecatedFeature {
                feature,
                alternative,
            } => {
                write!(
                    f,
                    "deprecated feature '{}'; use '{}' instead",
                    feature, alternative
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_error_display() {
        let error = LexError::UnexpectedChar {
            ch: '$',
            span: Span::new(10, 11, 2, 5),
        };
        let msg = error.to_string();
        assert!(msg.contains("$"));
        assert!(msg.contains("line 2"));
        assert!(msg.contains("column 5"));
    }

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::UnexpectedToken {
            expected: "identifier".to_string(),
            found: "'gene'".to_string(),
            span: Span::new(0, 4, 1, 1),
        };
        let msg = error.to_string();
        assert!(msg.contains("expected identifier"));
        assert!(msg.contains("'gene'"));
    }

    #[test]
    fn test_validation_errors_collection() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());
        
        errors.add_error(ValidationError::EmptyExegesis {
            span: Span::default(),
        });
        assert!(errors.has_errors());
        assert!(!errors.has_warnings());
        
        errors.add_warning(ValidationWarning::ShortExegesis {
            length: 10,
            span: Span::default(),
        });
        assert!(errors.has_warnings());
    }
}
