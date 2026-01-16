//! Sex linting rules
//!
//! This module provides lint rules for enforcing purity constraints and
//! detecting improper use of side effects in DOL code.
//!
//! # Lint Rules
//!
//! ## Errors
//!
//! - **E001**: Sex in pure context - Side effect used outside sex context
//! - **E002**: Mutable global outside sex - Mutable global state accessed in pure context
//! - **E003**: FFI outside sex - Foreign function interface call in pure context
//! - **E004**: I/O outside sex - I/O operation in pure context
//!
//! ## Warnings
//!
//! - **W001**: Large sex block - Sex block exceeds recommended size
//! - **W002**: Sex function without documentation - Sex function lacks exegesis

#[cfg(test)]
use crate::ast::Statement;
use crate::ast::{Declaration, Gene, Span, Trait};
use crate::sex::context::SexContext;
use crate::sex::tracking::{EffectKind, EffectTracker};

/// A sex lint error.
///
/// Represents a critical violation of purity rules that must be fixed.
#[derive(Debug, Clone, PartialEq)]
pub enum SexLintError {
    /// E001: Sex in pure context.
    ///
    /// A side effect was used in a pure context where effects are not allowed.
    SexInPureContext {
        /// The kind of effect that was used
        effect_kind: EffectKind,
        /// Location of the violation
        span: Span,
        /// Additional context
        message: String,
    },

    /// E002: Mutable global outside sex.
    ///
    /// Mutable global state was accessed outside of a sex context.
    MutableGlobalOutsideSex {
        /// Name of the global variable
        name: String,
        /// Location of the access
        span: Span,
    },

    /// E003: FFI outside sex.
    ///
    /// A foreign function interface call was made outside of a sex context.
    FfiOutsideSex {
        /// Name of the FFI call
        name: String,
        /// Location of the call
        span: Span,
    },

    /// E004: I/O outside sex.
    ///
    /// An I/O operation was performed outside of a sex context.
    IoOutsideSex {
        /// Description of the I/O operation
        operation: String,
        /// Location of the operation
        span: Span,
    },
}

impl SexLintError {
    /// Returns the error code for this lint error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::lint::SexLintError;
    /// use metadol::sex::tracking::EffectKind;
    /// use metadol::ast::Span;
    ///
    /// let err = SexLintError::SexInPureContext {
    ///     effect_kind: EffectKind::Io,
    ///     span: Span::default(),
    ///     message: "test".to_string(),
    /// };
    /// assert_eq!(err.code(), "E001");
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            SexLintError::SexInPureContext { .. } => "E001",
            SexLintError::MutableGlobalOutsideSex { .. } => "E002",
            SexLintError::FfiOutsideSex { .. } => "E003",
            SexLintError::IoOutsideSex { .. } => "E004",
        }
    }

    /// Returns the span where this error occurred.
    pub fn span(&self) -> Span {
        match self {
            SexLintError::SexInPureContext { span, .. }
            | SexLintError::MutableGlobalOutsideSex { span, .. }
            | SexLintError::FfiOutsideSex { span, .. }
            | SexLintError::IoOutsideSex { span, .. } => *span,
        }
    }
}

impl std::fmt::Display for SexLintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SexLintError::SexInPureContext {
                effect_kind,
                span,
                message,
            } => write!(
                f,
                "[{}] sex in pure context: {} at line {}, column {} - {}",
                self.code(),
                effect_kind,
                span.line,
                span.column,
                message
            ),
            SexLintError::MutableGlobalOutsideSex { name, span } => write!(
                f,
                "[{}] mutable global '{}' accessed outside sex context at line {}, column {}",
                self.code(),
                name,
                span.line,
                span.column
            ),
            SexLintError::FfiOutsideSex { name, span } => write!(
                f,
                "[{}] FFI call '{}' outside sex context at line {}, column {}",
                self.code(),
                name,
                span.line,
                span.column
            ),
            SexLintError::IoOutsideSex { operation, span } => write!(
                f,
                "[{}] I/O operation '{}' outside sex context at line {}, column {}",
                self.code(),
                operation,
                span.line,
                span.column
            ),
        }
    }
}

/// A sex lint warning.
///
/// Represents a non-critical issue that should be addressed but won't
/// prevent compilation.
#[derive(Debug, Clone, PartialEq)]
pub enum SexLintWarning {
    /// W001: Large sex block.
    ///
    /// A sex block exceeds the recommended size and should be refactored.
    LargeSexBlock {
        /// Number of statements in the block
        size: usize,
        /// Recommended maximum size
        max_size: usize,
        /// Location of the block
        span: Span,
    },

    /// W002: Sex function without documentation.
    ///
    /// A sex function lacks proper exegesis documentation.
    SexFunctionWithoutDocumentation {
        /// Name of the function
        name: String,
        /// Location of the function
        span: Span,
    },
}

impl SexLintWarning {
    /// Returns the warning code for this lint warning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::lint::SexLintWarning;
    /// use metadol::ast::Span;
    ///
    /// let warn = SexLintWarning::LargeSexBlock {
    ///     size: 100,
    ///     max_size: 50,
    ///     span: Span::default(),
    /// };
    /// assert_eq!(warn.code(), "W001");
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            SexLintWarning::LargeSexBlock { .. } => "W001",
            SexLintWarning::SexFunctionWithoutDocumentation { .. } => "W002",
        }
    }

    /// Returns the span where this warning occurred.
    pub fn span(&self) -> Span {
        match self {
            SexLintWarning::LargeSexBlock { span, .. }
            | SexLintWarning::SexFunctionWithoutDocumentation { span, .. } => *span,
        }
    }
}

impl std::fmt::Display for SexLintWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SexLintWarning::LargeSexBlock {
                size,
                max_size,
                span,
            } => write!(
                f,
                "[{}] large sex block ({} statements, max {}) at line {}, column {}",
                self.code(),
                size,
                max_size,
                span.line,
                span.column
            ),
            SexLintWarning::SexFunctionWithoutDocumentation { name, span } => write!(
                f,
                "[{}] sex function '{}' lacks exegesis at line {}, column {}",
                self.code(),
                name,
                span.line,
                span.column
            ),
        }
    }
}

/// Result of linting a DOL file.
///
/// Contains all errors and warnings found during linting.
#[derive(Debug, Clone, Default)]
pub struct LintResult {
    /// Critical errors that must be fixed
    pub errors: Vec<SexLintError>,
    /// Non-critical warnings
    pub warnings: Vec<SexLintWarning>,
}

impl LintResult {
    /// Create a new empty lint result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns `true` if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns `true` if there are no errors or warnings.
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Add an error to the result.
    pub fn add_error(&mut self, error: SexLintError) {
        self.errors.push(error);
    }

    /// Add a warning to the result.
    pub fn add_warning(&mut self, warning: SexLintWarning) {
        self.warnings.push(warning);
    }
}

/// Sex linter for DOL code.
///
/// Enforces purity constraints and detects improper use of side effects.
///
/// # Example
///
/// ```rust
/// use metadol::sex::lint::SexLinter;
/// use metadol::sex::context::SexContext;
/// use metadol::ast::{Declaration, Gene, Span, Visibility};
///
/// let linter = SexLinter::new(SexContext::Pure);
///
/// let gene = Gene {
///     name: "test.gene".to_string(),
///     extends: None,
///     statements: vec![],
///     exegesis: "Test gene".to_string(),
///     span: Span::default(),
///     visibility: Visibility::default(),
/// };
///
/// let result = linter.lint_declaration(&Declaration::Gene(gene));
/// assert!(result.is_clean());
/// ```
pub struct SexLinter {
    /// The current sex context
    context: SexContext,
    /// Maximum recommended size for sex blocks
    max_sex_block_size: usize,
}

impl SexLinter {
    /// Create a new sex linter.
    ///
    /// # Arguments
    ///
    /// * `context` - The sex context to enforce
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::lint::SexLinter;
    /// use metadol::sex::context::SexContext;
    ///
    /// let linter = SexLinter::new(SexContext::Pure);
    /// ```
    pub fn new(context: SexContext) -> Self {
        Self {
            context,
            max_sex_block_size: 50,
        }
    }

    /// Set the maximum sex block size.
    ///
    /// # Arguments
    ///
    /// * `size` - The maximum number of statements
    pub fn with_max_block_size(mut self, size: usize) -> Self {
        self.max_sex_block_size = size;
        self
    }

    /// Lint a declaration.
    ///
    /// # Arguments
    ///
    /// * `decl` - The declaration to lint
    ///
    /// # Returns
    ///
    /// A [`LintResult`] containing any errors or warnings found.
    pub fn lint_declaration(&self, decl: &Declaration) -> LintResult {
        let mut result = LintResult::new();

        // Track effects in the declaration
        let mut tracker = EffectTracker::new();
        tracker.track_declaration(decl);

        // If we're in a pure context, check for any effects
        if self.context.is_pure() {
            let effects = tracker.get_effects(decl.name());
            for effect in effects {
                match effect.kind {
                    EffectKind::Io => {
                        result.add_error(SexLintError::IoOutsideSex {
                            operation: effect
                                .context
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            span: effect.span,
                        });
                    }
                    EffectKind::Ffi => {
                        result.add_error(SexLintError::FfiOutsideSex {
                            name: effect
                                .context
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            span: effect.span,
                        });
                    }
                    EffectKind::MutableGlobal => {
                        result.add_error(SexLintError::MutableGlobalOutsideSex {
                            name: effect
                                .context
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            span: effect.span,
                        });
                    }
                    _ => {
                        result.add_error(SexLintError::SexInPureContext {
                            effect_kind: effect.kind,
                            span: effect.span,
                            message: effect
                                .context
                                .clone()
                                .unwrap_or_else(|| "unknown effect".to_string()),
                        });
                    }
                }
            }
        }

        // Check for large sex blocks
        self.check_block_size(decl, &mut result);

        // Check for missing exegesis on sex declarations
        if self.context.is_sex() {
            self.check_exegesis(decl, &mut result);
        }

        result
    }

    /// Check if a declaration's block size exceeds the maximum.
    fn check_block_size(&self, decl: &Declaration, result: &mut LintResult) {
        let (size, span) = match decl {
            Declaration::Gene(Gene {
                statements, span, ..
            })
            | Declaration::Trait(Trait {
                statements, span, ..
            }) => (statements.len(), *span),
            _ => return,
        };

        if size > self.max_sex_block_size {
            result.add_warning(SexLintWarning::LargeSexBlock {
                size,
                max_size: self.max_sex_block_size,
                span,
            });
        }
    }

    /// Check if a sex declaration has proper exegesis.
    fn check_exegesis(&self, decl: &Declaration, result: &mut LintResult) {
        let exegesis = decl.exegesis();
        if exegesis.trim().is_empty() || exegesis.trim().len() < 20 {
            result.add_warning(SexLintWarning::SexFunctionWithoutDocumentation {
                name: decl.name().to_string(),
                span: decl.span(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Visibility;

    #[test]
    fn test_lint_error_codes() {
        let err = SexLintError::SexInPureContext {
            effect_kind: EffectKind::Io,
            span: Span::default(),
            message: "test".to_string(),
        };
        assert_eq!(err.code(), "E001");

        let err = SexLintError::MutableGlobalOutsideSex {
            name: "test".to_string(),
            span: Span::default(),
        };
        assert_eq!(err.code(), "E002");

        let err = SexLintError::FfiOutsideSex {
            name: "test".to_string(),
            span: Span::default(),
        };
        assert_eq!(err.code(), "E003");

        let err = SexLintError::IoOutsideSex {
            operation: "test".to_string(),
            span: Span::default(),
        };
        assert_eq!(err.code(), "E004");
    }

    #[test]
    fn test_lint_warning_codes() {
        let warn = SexLintWarning::LargeSexBlock {
            size: 100,
            max_size: 50,
            span: Span::default(),
        };
        assert_eq!(warn.code(), "W001");

        let warn = SexLintWarning::SexFunctionWithoutDocumentation {
            name: "test".to_string(),
            span: Span::default(),
        };
        assert_eq!(warn.code(), "W002");
    }

    #[test]
    fn test_lint_result() {
        let mut result = LintResult::new();
        assert!(result.is_clean());
        assert!(!result.has_errors());
        assert!(!result.has_warnings());

        result.add_error(SexLintError::IoOutsideSex {
            operation: "test".to_string(),
            span: Span::default(),
        });
        assert!(result.has_errors());
        assert!(!result.is_clean());

        result.add_warning(SexLintWarning::LargeSexBlock {
            size: 100,
            max_size: 50,
            span: Span::default(),
        });
        assert!(result.has_warnings());
    }

    #[test]
    fn test_pure_gene_no_effects() {
        let linter = SexLinter::new(SexContext::Pure);

        let gene = Gene {
            visibility: Visibility::default(),
            name: "test.gene".to_string(),
            extends: None,
            statements: vec![Statement::Has {
                subject: "test".to_string(),
                property: "property".to_string(),
                span: Span::default(),
            }],
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        };

        let result = linter.lint_declaration(&Declaration::Gene(gene));
        assert!(result.is_clean());
    }

    #[test]
    fn test_io_in_pure_context() {
        let linter = SexLinter::new(SexContext::Pure);

        let gene = Gene {
            visibility: Visibility::default(),
            name: "io.gene".to_string(),
            extends: None,
            statements: vec![Statement::Has {
                subject: "io".to_string(),
                property: "file_read".to_string(),
                span: Span::default(),
            }],
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        };

        let result = linter.lint_declaration(&Declaration::Gene(gene));
        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code(), "E004");
    }

    #[test]
    fn test_large_block_warning() {
        let linter = SexLinter::new(SexContext::Pure).with_max_block_size(5);

        let statements: Vec<Statement> = (0..10)
            .map(|i| Statement::Has {
                subject: "test".to_string(),
                property: format!("prop{}", i),
                span: Span::default(),
            })
            .collect();

        let gene = Gene {
            visibility: Visibility::default(),
            name: "test.gene".to_string(),
            extends: None,
            statements,
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        };

        let result = linter.lint_declaration(&Declaration::Gene(gene));
        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].code(), "W001");
    }

    #[test]
    fn test_sex_without_documentation() {
        let linter = SexLinter::new(SexContext::Sex);

        let gene = Gene {
            visibility: Visibility::default(),
            name: "test.gene".to_string(),
            extends: None,
            statements: vec![],
            exegesis: "Short".to_string(), // Too short
            span: Span::default(),
        };

        let result = linter.lint_declaration(&Declaration::Gene(gene));
        assert!(result.has_warnings());
        assert_eq!(result.warnings[0].code(), "W002");
    }
}
