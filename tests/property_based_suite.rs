//! Property-based testing suite for code generation.
//!
//! This is the entry point for M6.2: Property-Based Testing
//! - Generates random valid DOL schemas using proptest
//! - Verifies generated code compiles
//! - Tests round-trip transformations: DOL → Code → DOL
//! - Runs 10K+ random schemas (with --ignored flag)

mod property_tests;
