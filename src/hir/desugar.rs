//! Re-export of lowering/desugaring functions
//!
//! Desugaring happens during AST→HIR lowering.
//! See `crate::lower::desugar` for the implementation.

pub use crate::lower::{lower_file, lower_module};
