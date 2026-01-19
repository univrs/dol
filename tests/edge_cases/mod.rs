//! Edge Case Tests for DOL
//!
//! This module contains comprehensive edge case tests for the DOL language,
//! created during Phase 5 of the Spirits Jam project.
//!
//! ## Test Categories
//!
//! - **numerical**: Division by zero, overflow, NaN, Infinity, precision edge cases
//! - **parser**: Deep nesting, unicode, escapes, empty constructs, ambiguous grammar
//! - **wasm**: Stack overflow, memory limits, hot-reload, invalid WASM recovery
//! - **modules**: Circular deps, diamond deps, shadowing, visibility boundaries
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all edge case tests
//! cargo test edge_cases
//!
//! # Run specific category
//! cargo test edge_cases::numerical
//! cargo test edge_cases::parser
//! cargo test edge_cases::wasm
//! cargo test edge_cases::modules
//!
//! # Run with output to see diagnostic notes
//! cargo test edge_cases -- --nocapture
//! ```
//!
//! ## Bug Documentation
//!
//! Bugs discovered through these tests are documented in `docs/bugs-discovered.md`.

mod modules;
mod numerical;
mod parser;

#[cfg(feature = "wasm-runtime")]
mod wasm;
