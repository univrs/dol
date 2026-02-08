//! Property-based testing for DOL code generation.
//!
//! This module uses proptest to:
//! - Generate random valid DOL schemas
//! - Verify generated code compiles
//! - Test round-trip transformations
//! - Run 10K+ random test cases

mod codegen;
