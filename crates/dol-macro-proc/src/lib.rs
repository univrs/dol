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

// Internal modules for proc-macro implementation
// Note: proc-macro crates cannot export public modules or items
// except for proc-macro functions themselves
mod ast_util;
mod codegen;
mod derive;
mod attribute;
mod function;
mod error;

#[cfg(test)]
mod tests {
    use crate::derive::derive_debug;
    use crate::attribute::attribute_cached;

    #[test]
    fn test_basic_imports() {
        // Just ensure all modules compile and are accessible
        let _ = derive_debug;
        let _ = attribute_cached;
    }
}
