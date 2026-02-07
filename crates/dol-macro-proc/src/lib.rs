//! # DOL Procedural Macro System
//!
//! This crate provides procedural macros for Metal DOL, enabling
//! Rust-style derive macros, attribute macros, and function-like macros.
//!
//! # Features
//!
//! - **Derive Macros**: Automatically generate trait implementations
//! - **Attribute Macros**: Transform declarations with attributes
//! - **Function-like Macros**: Custom syntax extensions
//! - **AST Manipulation**: Powerful API for transforming DOL AST
//!
//! # Example
//!
//! ```rust,ignore
//! use dol_macro_proc::derive_gen;
//!
//! // Derive macro usage in DOL:
//! // #[derive(Debug, Clone)]
//! // gene container.exists {
//! //   container has identity
//! // }
//! ```
//!
//! # Procedural Macro Types
//!
//! ## Derive Macros
//!
//! Derive macros generate trait implementations based on the structure
//! of a declaration:
//!
//! ```text
//! #[derive(Debug, Clone, PartialEq)]
//! gene container.exists { ... }
//! ```
//!
//! ## Attribute Macros
//!
//! Attribute macros transform declarations:
//!
//! ```text
//! #[cached]
//! spell compute(x: Int) -> Int { ... }
//! ```
//!
//! ## Function-like Macros
//!
//! Function-like macros provide custom syntax:
//!
//! ```text
//! let sql = sql!("SELECT * FROM users WHERE id = ?", user_id);
//! ```

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

pub mod ast_util;
pub mod codegen;
pub mod derive;
pub mod attribute;
pub mod function;
pub mod error;

// Re-export commonly used types
pub use error::{ProcMacroError, ProcMacroResult};
pub use derive::{derive_debug, derive_clone, derive_partial_eq, derive_gen_trait};
pub use attribute::{AttributeMacro, attribute_cached, attribute_async};
pub use function::{FunctionMacro, function_sql, function_format};
pub use ast_util::{AstManipulator, AstTransform};
pub use codegen::{CodeGenerator, generate_rust_code, generate_wit_code};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::ast_util::{AstManipulator, AstTransform};
    pub use crate::attribute::{AttributeMacro, attribute_cached, attribute_async};
    pub use crate::codegen::{CodeGenerator, generate_rust_code, generate_wit_code};
    pub use crate::derive::{derive_clone, derive_debug, derive_gen_trait, derive_partial_eq};
    pub use crate::error::{ProcMacroError, ProcMacroResult};
    pub use crate::function::{FunctionMacro, function_format, function_sql};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_imports() {
        // Just ensure all modules compile and are accessible
        let _ = derive_debug;
        let _ = attribute_cached;
    }
}
