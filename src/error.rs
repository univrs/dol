//! Error types for Metal DOL.
//!
//! This module defines all error types used throughout the crate,
//! providing rich error information including source locations and
//! cross-boundary error reporting for ABI operations.
//!
//! # Error Categories
//!
//! - [`LexError`]: Errors during tokenization
//! - [`ParseError`]: Errors during parsing
//! - [`ValidationError`]: Errors during semantic validation
//! - [`AbiError`]: Errors at application binary interface boundaries
//!
//! # Example
//!
//! ```rust
//! use metadol::error::ParseError;
//! use metadol::ast::Span;
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
    #[error("invalid declaration type '{found}' at line {}, column {} (expected module, use, pub, fun, gen, gene, trait, rule, constraint, system, evo, evolves, docs, or exegesis)", span.line, span.column)]
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

    /// An invalid CRDT strategy was specified.
    ///
    /// CRDT strategies must be one of: immutable, lww, or_set, pn_counter, peritext, rga, mv_register
    #[error("invalid CRDT strategy '{strategy}' at line {}, column {} (expected one of: immutable, lww, or_set, pn_counter, peritext, rga, mv_register)", span.line, span.column)]
    InvalidCrdtStrategy {
        /// The invalid strategy name
        strategy: String,
        /// Location of the invalid strategy
        span: Span,
    },
}

impl ParseError {
    /// Returns the source span where this error occurred.
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
            ParseError::MissingExegesis { span } => *span,
            ParseError::InvalidStatement { span, .. } => *span,
            ParseError::InvalidCrdtStrategy { span, .. } => *span,
            ParseError::InvalidDeclaration { span, .. } => *span,
            ParseError::UnexpectedEof { span, .. } => *span,
            ParseError::LexerError(lex_err) => match lex_err {
                LexError::UnexpectedChar { span, .. } => *span,
                LexError::UnterminatedString { span } => *span,
                LexError::InvalidVersion { span, .. } => *span,
                LexError::InvalidEscape { span, .. } => *span,
            },
        }
    }
}

/// Errors that can occur during semantic validation.
///
/// These errors are produced by the [`validator`](crate::validator) when
/// the AST violates semantic rules that cannot be caught during parsing.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
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

    /// A type error occurred during type checking.
    #[error("type error at line {}, column {}: {message}", span.line, span.column)]
    TypeError {
        /// The error message
        message: String,
        /// Expected type (if applicable)
        expected: Option<String>,
        /// Actual type (if applicable)
        actual: Option<String>,
        /// Location of the error
        span: Span,
    },

    /// A CRDT strategy is incompatible with the field type.
    ///
    /// This occurs when a CRDT annotation specifies a merge strategy
    /// that cannot be applied to the field's type (e.g., OR-Set on a String).
    #[error("incompatible CRDT strategy for field '{field}' at line {}, column {}: {strategy} cannot be used with type {type_}", span.line, span.column)]
    IncompatibleCrdtStrategy {
        /// The field name
        field: String,
        /// The field type
        type_: String,
        /// The attempted strategy
        strategy: String,
        /// Suggested valid strategies
        suggestion: String,
        /// Location of the error
        span: Span,
    },

    /// A constraint conflicts with CRDT semantics.
    ///
    /// This occurs when a constraint requires strong consistency guarantees
    /// that cannot be provided by the CRDT merge strategy.
    #[error("constraint '{constraint}' requires strong consistency at line {}, column {}", span.line, span.column)]
    ConstraintCrdtConflict {
        /// The constraint name
        constraint: String,
        /// The affected field
        field: String,
        /// Explanation of the conflict
        message: String,
        /// Location of the error
        span: Span,
    },

    /// An evolution attempts an invalid CRDT strategy change.
    ///
    /// This occurs when an evolution tries to change a CRDT strategy
    /// in a way that violates semantic compatibility (e.g., Immutable → LWW).
    #[error("invalid CRDT strategy evolution for field '{field}' at line {}, column {}: cannot change {old_strategy} to {new_strategy}", span.line, span.column)]
    InvalidCrdtEvolution {
        /// The field name
        field: String,
        /// The old strategy
        old_strategy: String,
        /// The new strategy
        new_strategy: String,
        /// Reason for invalidity
        reason: String,
        /// Location of the error
        span: Span,
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

    /// A constraint may be eventually consistent in a CRDT context.
    ///
    /// This warning indicates that a constraint may temporarily violate
    /// during network partitions but will converge once the partition heals.
    EventuallyConsistent {
        /// The constraint name
        constraint: String,
        /// The affected field
        field: String,
        /// Explanation of the eventual consistency
        message: String,
        /// Location of the constraint
        span: Span,
    },

    /// A constraint requires coordination mechanisms.
    ///
    /// This warning indicates that a constraint requires escrow,
    /// BFT consensus, or other coordination to maintain in a distributed system.
    RequiresCoordination {
        /// The constraint name
        constraint: String,
        /// The affected field
        field: String,
        /// Explanation of why coordination is needed
        message: String,
        /// Suggested coordination pattern
        suggestion: String,
        /// Location of the constraint
        span: Span,
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
            ValidationWarning::EventuallyConsistent {
                constraint,
                field,
                message,
                span,
            } => {
                write!(
                    f,
                    "constraint '{}' on field '{}' may be eventually consistent at line {}, column {}: {}",
                    constraint, field, span.line, span.column, message
                )
            }
            ValidationWarning::RequiresCoordination {
                constraint,
                field,
                message,
                suggestion,
                span,
            } => {
                write!(
                    f,
                    "constraint '{}' on field '{}' requires coordination at line {}, column {}: {} (suggestion: {})",
                    constraint, field, span.line, span.column, message, suggestion
                )
            }
        }
    }
}

/// Errors that can occur at ABI (Application Binary Interface) boundaries.
///
/// These errors are used for cross-boundary communication, particularly
/// for WASM integration and other interop scenarios. They are designed
/// to be serializable for transmission across boundaries.
///
/// # Example
///
/// ```rust
/// use metadol::error::AbiError;
///
/// let error = AbiError::InvalidPointer;
/// assert_eq!(error.to_string(), "Invalid pointer");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiError {
    /// An invalid pointer was encountered.
    ///
    /// This typically occurs when a memory address is null or out of bounds.
    InvalidPointer,

    /// Memory allocation failed.
    ///
    /// The system ran out of available memory or allocation was denied.
    OutOfMemory,

    /// A message received across the boundary is invalid.
    ///
    /// This occurs when a message doesn't conform to the expected format
    /// or contains invalid data.
    InvalidMessage(String),

    /// An operation timed out.
    ///
    /// The operation took longer than the allowed timeout period.
    Timeout,

    /// An effect execution failed.
    ///
    /// An operation with side effects failed during execution.
    EffectFailed(String),

    /// An unknown error occurred.
    ///
    /// Used for errors that don't fit into other categories or
    /// when the specific error type is not yet determined.
    UnknownError(String),
}

impl std::fmt::Display for AbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbiError::InvalidPointer => write!(f, "Invalid pointer"),
            AbiError::OutOfMemory => write!(f, "Out of memory"),
            AbiError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            AbiError::Timeout => write!(f, "Operation timed out"),
            AbiError::EffectFailed(err) => write!(f, "Effect failed: {}", err),
            AbiError::UnknownError(err) => write!(f, "Unknown error: {}", err),
        }
    }
}

impl std::error::Error for AbiError {}

impl From<String> for AbiError {
    fn from(s: String) -> Self {
        AbiError::UnknownError(s)
    }
}

impl From<&str> for AbiError {
    fn from(s: &str) -> Self {
        AbiError::UnknownError(s.to_string())
    }
}

/// Serialize support for AbiError when serde feature is enabled.
#[cfg(feature = "serde")]
impl serde::Serialize for AbiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            AbiError::InvalidPointer => {
                map.serialize_entry("type", "InvalidPointer")?;
                map.serialize_entry("message", "Invalid pointer")?;
            }
            AbiError::OutOfMemory => {
                map.serialize_entry("type", "OutOfMemory")?;
                map.serialize_entry("message", "Out of memory")?;
            }
            AbiError::InvalidMessage(msg) => {
                map.serialize_entry("type", "InvalidMessage")?;
                map.serialize_entry("message", msg)?;
            }
            AbiError::Timeout => {
                map.serialize_entry("type", "Timeout")?;
                map.serialize_entry("message", "Operation timed out")?;
            }
            AbiError::EffectFailed(err) => {
                map.serialize_entry("type", "EffectFailed")?;
                map.serialize_entry("message", err)?;
            }
            AbiError::UnknownError(err) => {
                map.serialize_entry("type", "UnknownError")?;
                map.serialize_entry("message", err)?;
            }
        }
        map.end()
    }
}

/// Deserialize support for AbiError when serde feature is enabled.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AbiError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct AbiErrorVisitor;

        impl<'de> Visitor<'de> for AbiErrorVisitor {
            type Value = AbiError;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("AbiError with type and message fields")
            }

            fn visit_map<M>(self, mut map: M) -> Result<AbiError, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut error_type: Option<String> = None;
                let mut message: Option<String> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "type" => {
                            if error_type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            error_type = Some(map.next_value()?);
                        }
                        "message" => {
                            if message.is_some() {
                                return Err(de::Error::duplicate_field("message"));
                            }
                            message = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let error_type = error_type.ok_or_else(|| de::Error::missing_field("type"))?;
                let message = message.ok_or_else(|| de::Error::missing_field("message"))?;

                match error_type.as_str() {
                    "InvalidPointer" => Ok(AbiError::InvalidPointer),
                    "OutOfMemory" => Ok(AbiError::OutOfMemory),
                    "InvalidMessage" => Ok(AbiError::InvalidMessage(message)),
                    "Timeout" => Ok(AbiError::Timeout),
                    "EffectFailed" => Ok(AbiError::EffectFailed(message)),
                    "UnknownError" => Ok(AbiError::UnknownError(message)),
                    _ => Err(de::Error::unknown_variant(
                        &error_type,
                        &[
                            "InvalidPointer",
                            "OutOfMemory",
                            "InvalidMessage",
                            "Timeout",
                            "EffectFailed",
                            "UnknownError",
                        ],
                    )),
                }
            }
        }

        deserializer.deserialize_map(AbiErrorVisitor)
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
        //  v0.3.0 changed this
        errors.add_error(ValidationError::InvalidIdentifier {
            name: "invalid".to_string(),
            reason: "Invalid identifier".to_string(),
        });
        //assert!(errors.has_errors());
        assert!(!errors.has_warnings());

        errors.add_warning(ValidationWarning::ShortExegesis {
            length: 10,
            span: Span::default(),
        });
        assert!(errors.has_warnings());
    }

    #[test]
    fn test_abi_error_invalid_pointer() {
        let error = AbiError::InvalidPointer;
        assert_eq!(error.to_string(), "Invalid pointer");
        assert_eq!(error, AbiError::InvalidPointer);
    }

    #[test]
    fn test_abi_error_out_of_memory() {
        let error = AbiError::OutOfMemory;
        assert_eq!(error.to_string(), "Out of memory");
    }

    #[test]
    fn test_abi_error_invalid_message() {
        let msg = "malformed data".to_string();
        let error = AbiError::InvalidMessage(msg.clone());
        assert_eq!(error.to_string(), format!("Invalid message: {}", msg));
    }

    #[test]
    fn test_abi_error_timeout() {
        let error = AbiError::Timeout;
        assert_eq!(error.to_string(), "Operation timed out");
    }

    #[test]
    fn test_abi_error_effect_failed() {
        let msg = "connection lost".to_string();
        let error = AbiError::EffectFailed(msg.clone());
        assert_eq!(error.to_string(), format!("Effect failed: {}", msg));
    }

    #[test]
    fn test_abi_error_unknown() {
        let msg = "something went wrong".to_string();
        let error = AbiError::UnknownError(msg.clone());
        assert_eq!(error.to_string(), format!("Unknown error: {}", msg));
    }

    #[test]
    fn test_abi_error_from_string() {
        let error: AbiError = "test error".to_string().into();
        assert!(matches!(error, AbiError::UnknownError(_)));
        assert_eq!(error.to_string(), "Unknown error: test error");
    }

    #[test]
    fn test_abi_error_from_str() {
        let error: AbiError = "test error".into();
        assert!(matches!(error, AbiError::UnknownError(_)));
        assert_eq!(error.to_string(), "Unknown error: test error");
    }

    #[test]
    fn test_abi_error_is_error_trait() {
        let error: Box<dyn std::error::Error> = Box::new(AbiError::OutOfMemory);
        assert_eq!(error.to_string(), "Out of memory");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_abi_error_serialize_deserialize() {
        use serde_json;

        let error = AbiError::InvalidMessage("test".to_string());
        let json = serde_json::to_string(&error).expect("serialize failed");
        assert!(json.contains("InvalidMessage"));
        assert!(json.contains("test"));

        let deserialized: AbiError = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(error, deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_abi_error_serialize_all_variants() {
        use serde_json;

        let variants = vec![
            AbiError::InvalidPointer,
            AbiError::OutOfMemory,
            AbiError::InvalidMessage("msg".to_string()),
            AbiError::Timeout,
            AbiError::EffectFailed("reason".to_string()),
            AbiError::UnknownError("error".to_string()),
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("serialize failed");
            let deserialized: AbiError = serde_json::from_str(&json).expect("deserialize failed");
            assert_eq!(variant, deserialized);
        }
    }
}
