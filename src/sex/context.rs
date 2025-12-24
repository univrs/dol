//! Sex context management
//!
//! This module provides types for tracking the purity context of DOL files
//! and compilation units.

use std::path::PathBuf;

/// Sex context for a compilation unit.
///
/// Represents whether code is operating in a pure (side-effect free) or
/// sex (side-effect allowed) context.
///
/// # Default
///
/// The default context is [`SexContext::Pure`], enforcing functional purity
/// by default.
///
/// # Example
///
/// ```rust
/// use metadol::sex::SexContext;
///
/// let ctx = SexContext::default();
/// assert_eq!(ctx, SexContext::Pure);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SexContext {
    /// Pure context - no side effects allowed.
    ///
    /// In pure context:
    /// - No I/O operations
    /// - No mutable global state
    /// - No FFI calls
    /// - All functions must be deterministic
    #[default]
    Pure,

    /// Sex context - side effects permitted.
    ///
    /// In sex context:
    /// - I/O operations allowed
    /// - Mutable global state allowed
    /// - FFI calls allowed
    /// - Non-deterministic operations allowed
    Sex,
}

impl SexContext {
    /// Returns `true` if this is a pure context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::SexContext;
    ///
    /// assert!(SexContext::Pure.is_pure());
    /// assert!(!SexContext::Sex.is_pure());
    /// ```
    pub fn is_pure(&self) -> bool {
        matches!(self, SexContext::Pure)
    }

    /// Returns `true` if this is a sex context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::SexContext;
    ///
    /// assert!(SexContext::Sex.is_sex());
    /// assert!(!SexContext::Pure.is_sex());
    /// ```
    pub fn is_sex(&self) -> bool {
        matches!(self, SexContext::Sex)
    }
}

impl std::fmt::Display for SexContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SexContext::Pure => write!(f, "pure"),
            SexContext::Sex => write!(f, "sex"),
        }
    }
}

/// Context for a specific file being compiled.
///
/// Associates a file path with its determined sex context, providing
/// a complete picture of the compilation environment for that file.
///
/// # Example
///
/// ```rust
/// use metadol::sex::FileContext;
/// use std::path::PathBuf;
///
/// let ctx = FileContext::new(PathBuf::from("io.sex.dol"));
/// assert!(ctx.is_sex());
/// assert_eq!(ctx.path.to_str().unwrap(), "io.sex.dol");
/// ```
#[derive(Debug, Clone)]
pub struct FileContext {
    /// The file path
    pub path: PathBuf,
    /// The sex context for this file
    pub sex_context: SexContext,
}

impl FileContext {
    /// Create a new file context, automatically detecting sex context from the path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path
    ///
    /// # Returns
    ///
    /// A new [`FileContext`] with the appropriate sex context determined
    /// from the file path.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::{FileContext, SexContext};
    /// use std::path::PathBuf;
    ///
    /// let ctx = FileContext::new(PathBuf::from("io.sex.dol"));
    /// assert_eq!(ctx.sex_context, SexContext::Sex);
    /// ```
    pub fn new(path: PathBuf) -> Self {
        let sex_context = super::file_sex_context(&path);
        Self { path, sex_context }
    }

    /// Create a file context with an explicit sex context.
    ///
    /// This allows overriding the automatic detection when needed.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path
    /// * `sex_context` - The sex context to use
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::{FileContext, SexContext};
    /// use std::path::PathBuf;
    ///
    /// let ctx = FileContext::with_context(
    ///     PathBuf::from("test.dol"),
    ///     SexContext::Sex
    /// );
    /// assert!(ctx.is_sex());
    /// ```
    pub fn with_context(path: PathBuf, sex_context: SexContext) -> Self {
        Self { path, sex_context }
    }

    /// Returns `true` if this file is in sex context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::FileContext;
    /// use std::path::PathBuf;
    ///
    /// let ctx = FileContext::new(PathBuf::from("io.sex.dol"));
    /// assert!(ctx.is_sex());
    /// ```
    pub fn is_sex(&self) -> bool {
        self.sex_context.is_sex()
    }

    /// Returns `true` if this file is in pure context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::FileContext;
    /// use std::path::PathBuf;
    ///
    /// let ctx = FileContext::new(PathBuf::from("container.dol"));
    /// assert!(ctx.is_pure());
    /// ```
    pub fn is_pure(&self) -> bool {
        self.sex_context.is_pure()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sex_context_default() {
        let ctx = SexContext::default();
        assert_eq!(ctx, SexContext::Pure);
        assert!(ctx.is_pure());
        assert!(!ctx.is_sex());
    }

    #[test]
    fn test_sex_context_display() {
        assert_eq!(SexContext::Pure.to_string(), "pure");
        assert_eq!(SexContext::Sex.to_string(), "sex");
    }

    #[test]
    fn test_file_context_new() {
        let ctx = FileContext::new(PathBuf::from("io.sex.dol"));
        assert_eq!(ctx.sex_context, SexContext::Sex);
        assert!(ctx.is_sex());
        assert!(!ctx.is_pure());
    }

    #[test]
    fn test_file_context_pure() {
        let ctx = FileContext::new(PathBuf::from("container.dol"));
        assert_eq!(ctx.sex_context, SexContext::Pure);
        assert!(ctx.is_pure());
        assert!(!ctx.is_sex());
    }

    #[test]
    fn test_file_context_with_context() {
        let ctx = FileContext::with_context(PathBuf::from("test.dol"), SexContext::Sex);
        assert_eq!(ctx.sex_context, SexContext::Sex);
        assert!(ctx.is_sex());
    }
}
