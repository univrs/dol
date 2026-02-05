//! DOL WASM Code Generation
//!
//! This crate provides WASM code generation capabilities for DOL.
//! It handles import emission, type conversions, and module building.

pub mod imports;

// Re-export key types
pub use imports::{ImportEmitter, ImportError, ImportInfo, ImportSection, ImportTracker, UsedImports};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // Basic smoke test
        let _tracker = ImportTracker::new();
        let _used = UsedImports::new();
    }
}
