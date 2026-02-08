//! Integration test runner for local-first stack.
//!
//! This is the main entry point for running integration tests.
//! Tests are organized in the `local_first/` module.
//!
//! Note: The local_first module requires external crates (automerge, vudo-*)
//! that are not available in the main dol package. These integration tests
//! should be run from the full VUDO workspace when available.

// TODO: Enable when running from full VUDO workspace with automerge available
// mod local_first;

// Re-export test modules so they can be discovered by test runner
// This allows running tests with: cargo test --test integration
