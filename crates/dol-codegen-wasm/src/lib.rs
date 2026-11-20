//! DOL WASM Code Generation
//!
//! This crate provides WASM code generation capabilities for DOL.
//! It handles import emission, type conversions, and module building.

// This crate emits the legacy `vudo_*` host imports (dol_abi::standard_host_functions),
// which the VUDO VM does not provide. Migrating it to dol_abi::vm_host_functions
// (module `vudo`, `host_*`) is tracked by Spirit Hosting Plan H0.
#![allow(deprecated)]

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
