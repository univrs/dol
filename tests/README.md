# VUDO Runtime Test Suite

Comprehensive testing infrastructure for the VUDO Runtime local-first stack.

## Overview

This directory contains the complete test suite for Phase 2 (MYCELIUM) of the VUDO Runtime project. The tests validate:

- ✅ **State Engine**: Automerge CRDT state management
- ✅ **Storage**: Platform-agnostic persistence
- ✅ **P2P Networking**: Iroh-based peer discovery and sync
- ✅ **Offline-First**: Complete offline workflows
- ✅ **Convergence**: CRDT merge correctness
- ✅ **Browser Integration**: WASM, IndexedDB, multi-tab sync

## Test Levels

### 1. Integration Tests (`integration/`)

**Rust-based integration tests** validating the local-first stack:

```bash
cd tests/integration
cargo test
```

**Test Coverage**:
- Airplane mode simulation (7 tests)
- Network partition/heal (7 tests)
- Concurrent multi-node edits (8 tests)
- Large document sync (8 tests)
- Schema evolution (9 tests)

**Total**: 39 integration tests

📚 [See integration/README.md](./integration/README.md) for details

### 2. End-to-End Tests (`e2e/`)

**Browser-based E2E tests** using Playwright:

```bash
cd tests/e2e
npm install
npm test
```

**Test Coverage**:
- Browser crash recovery
- Multi-tab synchronization
- Offline/online transitions
- IndexedDB persistence
- Service worker recovery

**Browsers**: Chrome, Firefox, Safari, Mobile Chrome, Mobile Safari

📚 [See e2e/README.md](./e2e/README.md) for details

### 3. Unit Tests (in crates)

**Per-crate unit tests**:

```bash
# State engine
cd crates/vudo-state
cargo test

# P2P networking
cd crates/vudo-p2p
cargo test

# Storage
cd crates/vudo-storage
cargo test
```

## Quick Start

### Run All Tests

```bash
# Rust integration tests
cargo test --test integration

# Browser E2E tests
cd tests/e2e && npm test

# Unit tests (all crates)
cargo test --workspace
```

### Run Specific Categories

```bash
# Airplane mode tests
cargo test --test integration airplane_mode

# Network partition tests
cargo test --test integration network_partition

# Browser crash recovery
cd tests/e2e && npm run test:crash

# Multi-tab sync
cd tests/e2e && npm run test:multi-tab
```

## Test Infrastructure

### Rust Integration Tests

**Test Harness** (`integration/local-first/test_harness.rs`):
- `TestNode`: Simulated node with state engine + P2P
- Network simulation: partition/heal
- Convergence verification: hash-based
- Performance measurement

**Key Abstractions**:
```rust
// Create test node
let node = TestNode::with_p2p("node_a").await;

// Create document
node.create_document("users", "alice", |doc| {
    doc.put(ROOT, "name", "Alice")?;
    Ok(())
}).await;

// Verify convergence
verify_full_convergence(&nodes, "users", "alice").await;
```

### Browser E2E Tests

**Playwright Configuration** (`e2e/playwright.config.ts`):
- Multi-browser support
- Mobile device emulation
- Video recording on failure
- Retry logic for CI

**Test Utilities**:
- Page object models
- Mock VUDO runtime
- IndexedDB inspection
- Network condition simulation

## Performance Targets

| Test Category | Target | Status |
|--------------|--------|--------|
| 10MB document sync | < 30s | ✅ |
| Convergence rate | 100% | ✅ |
| Data loss | 0% | ✅ |
| 1000 offline/online cycles | No loss | ✅ |
| Browser crash recovery | < 2s | ✅ |
| Cross-tab sync | < 500ms | ✅ |

## Success Criteria

Phase 2 (MYCELIUM) quality gate:

- [x] 100% convergence after partition heal (verified by hash)
- [x] No data loss across 1000 offline/online cycles
- [x] Browser crash recovery restores last persisted state
- [x] 10MB document syncs in < 30 seconds on broadband
- [x] All 39+ integration tests pass
- [x] All 15+ E2E tests pass
- [x] Mixed schema versions sync correctly

## CI/CD Integration

Tests run in GitHub Actions:

```yaml
# .github/workflows/ci.yml
jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Run integration tests
        run: cargo test --test integration --verbose

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Install Playwright
        run: cd tests/e2e && npm install && npx playwright install
      - name: Run E2E tests
        run: cd tests/e2e && npm test
```

**CI Features**:
- Parallel test execution
- Test result artifacts
- Coverage reporting
- Performance regression detection

## Directory Structure

```
tests/
├── README.md                    # This file
│
├── integration/                 # Rust integration tests
│   ├── Cargo.toml
│   ├── README.md
│   ├── main.rs                  # Test runner
│   └── local-first/
│       ├── mod.rs
│       ├── test_harness.rs      # Test infrastructure
│       ├── airplane_mode.rs     # 7 tests
│       ├── network_partition.rs # 7 tests
│       ├── concurrent_edits.rs  # 8 tests
│       ├── large_documents.rs   # 8 tests
│       └── schema_evolution.rs  # 9 tests
│
└── e2e/                         # Browser E2E tests
    ├── package.json
    ├── playwright.config.ts
    ├── README.md
    └── browser-sync/
        ├── crash_recovery.test.ts
        ├── multi_tab_sync.test.ts
        └── offline_online.test.ts
```

## Test Development

### Adding Integration Tests

1. Add test to appropriate file in `integration/local-first/`
2. Use `TestNode` from test harness
3. Verify convergence with hash comparison
4. Document performance expectations
5. Update test count in README

Example:
```rust
#[tokio::test]
async fn test_my_scenario() {
    let nodes = create_mesh_network(3).await;

    // Create documents
    for node in &nodes {
        node.create_document("test", "doc", |doc| {
            doc.put(ROOT, "value", 42i64)?;
            Ok(())
        }).await;
    }

    // Verify convergence
    verify_full_convergence(&nodes, "test", "doc").await;
}
```

### Adding E2E Tests

1. Create test in `e2e/browser-sync/`
2. Use Playwright page objects
3. Test across multiple browsers
4. Document browser-specific behavior

Example:
```typescript
test('should sync across tabs', async ({ browser }) => {
    const context = await browser.newContext();
    const tab1 = await context.newPage();
    const tab2 = await context.newPage();

    // Test implementation...

    await context.close();
});
```

## Debugging

### Integration Tests

```bash
# Enable logging
RUST_LOG=debug cargo test --test integration -- --nocapture

# Run single test
cargo test --test integration test_name -- --exact --nocapture

# Run serially (not in parallel)
cargo test --test integration -- --test-threads=1
```

### E2E Tests

```bash
# Debug mode
cd tests/e2e && npm run test:debug

# Headed mode (see browser)
npm run test:headed

# UI mode (interactive)
npm run test:ui

# Show trace
npx playwright show-trace trace.zip
```

## Coverage

Generate coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage for integration tests
cargo tarpaulin --test integration --out Html

# View report
open tarpaulin-report.html
```

## Performance Benchmarks

Run benchmarks:

```bash
# State engine benchmarks
cd crates/vudo-state
cargo bench

# P2P benchmarks (when enabled)
cd crates/vudo-p2p
cargo bench
```

## Related Documentation

- [Integration Test README](./integration/README.md) - Rust integration tests
- [E2E Test README](./e2e/README.md) - Browser E2E tests
- [VUDO State README](../crates/vudo-state/README.md) - State engine
- [VUDO P2P README](../crates/vudo-p2p/README.md) - P2P networking
- [VUDO Storage README](../crates/vudo-storage/README.md) - Storage layer

## Continuous Integration

Tests run on:
- Every commit (push to any branch)
- Every pull request
- Nightly builds (extended test suite)

**Test Matrix**:
- OS: Ubuntu, macOS, Windows
- Rust: stable, beta, nightly
- Browsers: Chrome, Firefox, Safari
- Mobile: iOS Safari, Android Chrome

## Contributing

When contributing tests:

1. Ensure all tests pass locally
2. Add tests for new features
3. Update documentation
4. Follow existing patterns
5. Use meaningful test names
6. Document edge cases

## License

Same as parent project: MIT OR Apache-2.0
