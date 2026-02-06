//! Integration test runner for local-first stack.
//!
//! This is the main entry point for running integration tests.
//! Tests are organized in the `local-first/` module.

mod local_first;

// Re-export test modules so they can be discovered by test runner
// This allows running tests with: cargo test --test integration
