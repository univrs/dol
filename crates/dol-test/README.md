# dol-test: Property-Based Testing Framework for CRDT Convergence

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.81%2B-orange.svg)](https://www.rust-lang.org)

`dol-test` provides a comprehensive property-based testing framework for verifying CRDT (Conflict-free Replicated Data Type) convergence guarantees. It implements the 13 formal theorems from RFC-001 and supports testing all 7 CRDT strategies defined in DOL.

## Features

- **Property-based testing** using `proptest` with 1000+ iterations per test
- **13 formal theorems** from RFC-001 implemented as testable properties
- **7 CRDT strategies** supported: Immutable, LWW, OR-Set, PN-Counter, RGA, MV-Register, Peritext
- **Network partition simulation** for testing partition tolerance
- **Automatic test case generation** from DOL files
- **Comprehensive test reports** with violation details and reproducers

## Supported CRDT Properties

### Fundamental Properties (Theorems 1-3)

1. **Commutativity**: `merge(a, b) = merge(b, a)`
2. **Associativity**: `merge(merge(a, b), c) = merge(a, merge(b, c))`
3. **Idempotency**: `merge(a, a) = a`

### Strategy-Specific Properties (Theorems 2.1-7.1)

- **Immutable** (Theorem 2.1): Write-once semantics
- **Last-Write-Wins** (Theorem 3.1): Timestamp-based conflict resolution
- **OR-Set** (Theorem 4.1): Add-wins set semantics with observed-remove
- **PN-Counter** (Theorem 5.1): Distributed counter with increment/decrement
- **RGA** (Theorem 6.1): Replicated Growable Array with causal ordering
- **MV-Register** (Theorem 7.1): Multi-value register preserving concurrent values
- **Peritext**: Collaborative rich text editing (informal verification)

### System-Level Properties (Theorems 9-13)

- **Constrained CRDT Convergence** (Theorem 9.1-9.3): Three-category constraint framework
- **Evolution Compatibility** (Theorem 10.1): Safe CRDT strategy migrations
- **Partition Tolerance** (Theorem 11.1): Operation during and after network partitions
- **Byzantine Fault Tolerance** (Theorem 13.1): Authenticated CRDTs

## Installation

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
dol-test = { path = "crates/dol-test" }
proptest = "1.4"
```

## Quick Start

### Basic Property Test

```rust
use dol_test::generators::*;
use dol_test::properties::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lww_convergence(
        ops_a in vec(lww_write_op(), 1..100),
        ops_b in vec(lww_write_op(), 1..100)
    ) {
        let mut replica_a = LwwValue::default();
        let mut replica_b = LwwValue::default();

        for op in ops_a {
            replica_a.apply(op)?;
        }
        for op in ops_b {
            replica_b.apply(op)?;
        }

        // Verify commutativity
        prop_assert!(verify_commutativity(&replica_a, &replica_b));
    }
}
```

### Using the Test Harness

```rust
use dol_test::harness::*;
use dol_test::TestConfig;

let config = TestConfig {
    num_cases: 1000,
    max_operations: 100,
    num_replicas: 3,
    simulate_partitions: true,
    ..Default::default()
};

let harness = TestHarness::new(config);
let report = harness.test_convergence::<MyCrdt, MyOp>(operations)?;

println!("{}", report.summary());
assert!(report.is_success());
```

### Implementing the Traits

To test your CRDT, implement these traits:

```rust
use dol_test::properties::*;
use dol_test::TestResult;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MyCrdt {
    // Your CRDT state
}

impl Mergeable for MyCrdt {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Your merge logic
        Ok(())
    }

    fn replica_id(&self) -> String {
        "replica".to_string()
    }
}

impl Operable<MyOperation> for MyCrdt {
    fn apply(&mut self, op: MyOperation) -> TestResult<()> {
        // Your operation application logic
        Ok(())
    }
}
```

## Examples

The `examples/property-tests/` directory contains comprehensive examples:

### Chat Message Example

Tests a collaborative chat message with:
- Immutable message ID
- LWW content
- OR-Set reactions
- PN-Counter likes

```bash
cargo test --example chat-message-properties
```

### Document Example

Tests a collaborative document with:
- Immutable document ID
- RGA paragraph list
- OR-Set tags
- LWW metadata

```bash
cargo test --example document-properties
```

### Counter Example

Tests a distributed counter with:
- PN-Counter semantics
- Multi-actor concurrency
- Partition tolerance

```bash
cargo test --example counter-properties
```

## Running Tests

```bash
# Run all convergence tests (1000 cases each)
cargo test --package dol-test --test convergence_tests

# Run with specific number of cases
PROPTEST_CASES=10000 cargo test --package dol-test

# Run a specific test
cargo test --package dol-test test_lww_commutativity

# Run with verbose output
cargo test --package dol-test -- --nocapture
```

## Test Configuration

Configure property tests using `TestConfig`:

```rust
let config = TestConfig {
    num_cases: 1000,              // Number of test cases
    max_operations: 100,          // Max operations per test
    num_replicas: 3,              // Number of replicas to simulate
    simulate_partitions: true,    // Enable partition testing
    simulate_byzantine: false,    // Enable Byzantine fault injection
    seed: Some(42),               // Reproducible random seed
};
```

## Property Generators

The `generators` module provides proptest generators for:

- **CRDT Operations**: `immutable_set_op()`, `lww_write_op()`, `or_set_add_op()`, etc.
- **Network Topologies**: `network_topology(num_replicas)`
- **Ordering Strategies**: `ordering_strategy()`
- **Test Scenarios**: `test_scenario(num_operations, num_replicas)`

## Architecture

```
dol-test/
├── src/
│   ├── lib.rs          # Framework entry point
│   ├── properties.rs   # CRDT property definitions (13 theorems)
│   ├── generators.rs   # Proptest generators
│   └── harness.rs      # Test harness utilities
├── tests/
│   └── convergence_tests.rs  # Comprehensive test suite (50+ tests)
└── examples/
    └── property-tests/ # Example test suites
```

## Performance Targets

From RFC-001 Section 12:

- **Test Cases**: 1000 iterations per property (default)
- **Operation Sequences**: Up to 10K operations
- **Merge Time**: < 10ms for 10K operation histories
- **Coverage**: All 7 CRDT strategies, 13 theorems

## Integration with DOL

`dol-test` integrates with DOL code generation:

```rust
use dol::parse_dol_file;
use dol_codegen_rust::generate_rust;
use dol_test::harness::TestHarness;

// Parse DOL file
let file = parse_dol_file(dol_source)?;

// Generate Rust code with CRDT annotations
let code = generate_rust(&file, &options)?;

// Run property tests on generated code
let harness = TestHarness::default_config();
let report = harness.test_fundamental_properties(states)?;
```

## Test Reports

Test reports include:

- Total cases executed
- Pass/fail counts
- Detailed violation information
- Minimal reproducing sequences
- Configuration summary
- Execution duration

```rust
let report = harness.test_convergence(ops)?;
println!("{}", report.summary());

// Example output:
// Test Report: 1000 total, 1000 passed, 0 failed (100.00% pass rate)
// Config: cases=1000, max_ops=100, replicas=3
// Duration: 1234ms
```

## Troubleshooting

### Test Failures

When a property test fails, proptest will:

1. Shrink the failing input to a minimal reproducer
2. Display the smallest failing case
3. Save the seed for reproduction

To reproduce a failure:

```rust
// Use the seed from the failure output
let config = TestConfig {
    seed: Some(1234567890),
    ..Default::default()
};
```

### Performance Issues

For large test suites, consider:

- Reducing `num_cases` for faster iteration
- Using `--release` mode for performance
- Running tests in parallel with `cargo test --jobs 4`

## Contributing

Contributions are welcome! Please ensure:

- All tests pass: `cargo test --package dol-test`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy -- -D warnings`
- New properties include tests

## References

- [RFC-001: Formal Proofs](../../rfcs/RFC-001-formal-proofs.md) - 13 theorems
- [DOL CRDT Annotations](../../rfcs/RFC-001-dol-crdt-annotations.md) - Language specification
- [Shapiro et al. 2011](https://hal.inria.fr/inria-00555588/) - CRDT foundations
- [proptest documentation](https://docs.rs/proptest/) - Property-based testing

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
