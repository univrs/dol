//! Multi-Target Code Generation
//!
//! This module provides code generation for multiple target languages:
//! - Rust
//! - TypeScript
//! - WebAssembly Interface Types (WIT)
//! - Python
//! - JSON Schema

pub mod rust;
pub mod typescript;
pub mod wit;
pub mod python;
pub mod json_schema;

use crate::{CodegenContext, Result, Target};
use dol::ast::DolFile;

/// Generate code for a specific target
pub fn generate(file: &DolFile, context: &CodegenContext) -> Result<String> {
    match context.target {
        Target::Rust => rust::generate(file, context),
        Target::TypeScript => typescript::generate(file, context),
        Target::Wit => wit::generate(file, context),
        Target::Python => python::generate(file, context),
        Target::JsonSchema => json_schema::generate(file, context),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_generation() {
        // Test that all targets are accessible
        assert!(rust::GENERATOR_NAME.len() > 0);
        assert!(typescript::GENERATOR_NAME.len() > 0);
        assert!(wit::GENERATOR_NAME.len() > 0);
        assert!(python::GENERATOR_NAME.len() > 0);
        assert!(json_schema::GENERATOR_NAME.len() > 0);
    }
}
