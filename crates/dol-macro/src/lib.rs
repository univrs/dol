//! # DOL Macro System
//!
//! This crate provides the declarative macro system for Metal DOL,
//! enabling compile-time metaprogramming through pattern-based
//! code generation and transformation.
//!
//! # Features
//!
//! - **Declarative Macros**: macro_rules!-style pattern matching
//! - **Hygienic Expansion**: Automatic name resolution and scoping
//! - **Compile-time Code Generation**: Transform AST at compile time
//! - **Standard Library**: Common macros for everyday use
//!
//! # Example
//!
//! ```rust
//! use dol_macro::{MacroRegistry, MacroPattern, MacroRule};
//! use metadol::ast::{Expr, Literal};
//!
//! // Create a macro registry
//! let mut registry = MacroRegistry::new();
//!
//! // Register a simple macro
//! // macro my_const! {
//! //     () => { 42 }
//! // }
//! let rule = MacroRule::new(
//!     vec![MacroPattern::Empty],
//!     vec![Expr::Literal(Literal::Int(42))],
//! );
//! registry.register_declarative("my_const", vec![rule]);
//!
//! // Use the macro
//! // let result = my_const!();
//! ```
//!
//! # Modules
//!
//! - [`declarative`]: Declarative macro implementation
//! - [`pattern`]: Pattern matching for macro rules
//! - [`hygiene`]: Hygienic macro expansion
//! - [`expand`]: Macro expansion engine
//! - [`stdlib`]: Standard library of common macros
//! - [`error`]: Error types

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod declarative;
pub mod error;
pub mod expand;
pub mod hygiene;
pub mod pattern;
pub mod registry;
pub mod stdlib;

// Re-export commonly used types
pub use declarative::{DeclarativeMacro, MacroRule};
pub use error::{MacroError, MacroResult};
pub use expand::MacroExpander;
pub use hygiene::{HygieneContext, SyntaxContext};
pub use pattern::{MacroFragment, MacroPattern, PatternMatcher};
pub use registry::MacroRegistry;

/// Prelude module for convenient imports.
///
/// # Example
///
/// ```rust
/// use dol_macro::prelude::*;
/// ```
pub mod prelude {
    pub use crate::declarative::{DeclarativeMacro, MacroRule};
    pub use crate::error::{MacroError, MacroResult};
    pub use crate::expand::MacroExpander;
    pub use crate::hygiene::{HygieneContext, SyntaxContext};
    pub use crate::pattern::{MacroFragment, MacroPattern, PatternMatcher};
    pub use crate::registry::MacroRegistry;
    pub use crate::stdlib::register_stdlib_macros;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_registry() {
        let registry = MacroRegistry::new();
        assert_eq!(registry.len(), 0);
    }
}
