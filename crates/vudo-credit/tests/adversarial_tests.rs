//! Adversarial Testing Suite Entry Point
//!
//! This is the main entry point for running Byzantine Fault Tolerance tests.
//! It imports and runs all adversarial test categories.

mod adversarial;

// Re-export for convenience
pub use adversarial::*;
