//! Import management for WASM code generation

pub mod emitter;
pub mod tracker;

pub use emitter::{ImportEmitter, ImportError, ImportInfo, ImportSection};
pub use tracker::{ImportTracker, UsedImports};
