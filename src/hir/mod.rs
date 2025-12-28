//! HIR (High-level Intermediate Representation)
//!
//! Canonical representation for DOL programs.
//! All surface syntax desugars to these 22 node types.
//!
//! # Design Principles
//!
//! - **Minimal**: 22 node types (vs 50+ in AST)
//! - **Canonical**: One representation per concept
//! - **Typed**: All expressions carry type information
//! - **Desugared**: No syntactic sugar remains
//!
//! # Key Types
//!
//! - [`HirModule`] - Top-level compilation unit
//! - [`HirDecl`] - 4 declaration forms (Type, Trait, Function, Module)
//! - [`HirExpr`] - 12 expression forms
//! - [`HirStmt`] - 6 statement forms (Val, Var, Assign, Expr, Return, Break)
//! - [`HirType`] - 8 type forms
//! - [`HirPat`] - 6 pattern forms

pub mod print;
pub mod span;
pub mod symbol;
pub mod types;
pub mod validate;
pub mod visit;

pub use span::{HirId, SpanMap};
pub use symbol::{Symbol, SymbolTable};
pub use types::*;
pub use validate::{validate_module, ValidationContext, ValidationError};
