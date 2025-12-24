//! SEX (Side Effect eXecution) System
//!
//! This module handles:
//! - Effect tracking and propagation
//! - File-based sex context detection
//! - FFI wrapper generation
//!
//! # Overview
//!
//! The SEX system enforces purity constraints in DOL code. By default, all DOL
//! code is pure (side-effect free). Code that performs side effects (I/O, FFI,
//! mutable globals) must be explicitly marked as "sex" code.
//!
//! # File Context Detection
//!
//! Files are automatically considered to be in sex context if:
//! - They have a `.sex.dol` extension (e.g., `io.sex.dol`)
//! - They are located in a `sex/` directory
//!
//! # Example
//!
//! ```rust
//! use metadol::sex::{is_sex_file, file_sex_context, SexContext};
//! use std::path::Path;
//!
//! let pure_file = Path::new("container.dol");
//! assert_eq!(file_sex_context(pure_file), SexContext::Pure);
//!
//! let sex_file = Path::new("io.sex.dol");
//! assert_eq!(file_sex_context(sex_file), SexContext::Sex);
//!
//! let sex_dir_file = Path::new("src/sex/globals.dol");
//! assert_eq!(file_sex_context(sex_dir_file), SexContext::Sex);
//! ```

pub mod context;
pub mod lint;
pub mod tracking;

pub use context::{FileContext, SexContext};
pub use lint::{LintResult, SexLintError, SexLintWarning, SexLinter};
pub use tracking::EffectTracker;

use std::path::Path;

/// Determine if a file is in sex context based on its path.
///
/// A file is considered in sex context if:
/// - It has a `.sex.dol` extension, or
/// - It is located in a directory named `sex/`
///
/// # Arguments
///
/// * `path` - The file path to check
///
/// # Returns
///
/// `true` if the file should be treated as sex context, `false` otherwise.
///
/// # Example
///
/// ```rust
/// use metadol::sex::is_sex_file;
/// use std::path::Path;
///
/// assert!(is_sex_file(Path::new("io.sex.dol")));
/// assert!(is_sex_file(Path::new("src/sex/globals.dol")));
/// assert!(!is_sex_file(Path::new("container.dol")));
/// ```
pub fn is_sex_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check for .sex.dol extension
    if path_str.ends_with(".sex.dol") {
        return true;
    }

    // Check if in sex/ directory
    for component in path.components() {
        if component.as_os_str() == "sex" {
            return true;
        }
    }

    false
}

/// Get the sex context for a file.
///
/// Convenience function that returns the appropriate [`SexContext`] for a given file path.
///
/// # Arguments
///
/// * `path` - The file path to check
///
/// # Returns
///
/// [`SexContext::Sex`] if the file is in sex context, [`SexContext::Pure`] otherwise.
///
/// # Example
///
/// ```rust
/// use metadol::sex::{file_sex_context, SexContext};
/// use std::path::Path;
///
/// let ctx = file_sex_context(Path::new("io.sex.dol"));
/// assert_eq!(ctx, SexContext::Sex);
/// ```
pub fn file_sex_context(path: &Path) -> SexContext {
    if is_sex_file(path) {
        SexContext::Sex
    } else {
        SexContext::Pure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sex_file_detection_with_extension() {
        assert!(is_sex_file(Path::new("io.sex.dol")));
        assert!(is_sex_file(Path::new("globals.sex.dol")));
        assert!(is_sex_file(Path::new("src/ffi.sex.dol")));
    }

    #[test]
    fn test_sex_file_detection_in_directory() {
        assert!(is_sex_file(Path::new("src/sex/globals.dol")));
        assert!(is_sex_file(Path::new("sex/ffi.dol")));
        assert!(is_sex_file(Path::new("modules/sex/io.dol")));
    }

    #[test]
    fn test_pure_file_detection() {
        assert!(!is_sex_file(Path::new("container.dol")));
        assert!(!is_sex_file(Path::new("src/genes/process.dol")));
        assert!(!is_sex_file(Path::new("traits/lifecycle.dol")));
    }

    #[test]
    fn test_file_sex_context() {
        assert_eq!(file_sex_context(Path::new("io.sex.dol")), SexContext::Sex);
        assert_eq!(
            file_sex_context(Path::new("container.dol")),
            SexContext::Pure
        );
    }
}
