# VUDO Runtime Testing Guide

Complete guide to running and understanding the VUDO Runtime test suite.

## Quick Reference

```bash
# Integration tests (Rust)
cargo test --test integration

# E2E tests (Browser)
cd tests/e2e && npm test

# All unit tests
cargo test --workspace

# Specific category
cargo test --test integration airplane_mode
```

## Test Suite Overview

### Statistics

- **Total Integration Tests**: 39
  - Airplane Mode: 7
  - Network Partition: 7
  - Concurrent Edits: 8
  - Large Documents: 8
  - Schema Evolution: 9

- **Total E2E Tests**: 15+
  - Crash Recovery: 6
  - Multi-Tab Sync: 7
  - Offline/Online: 8

- **Coverage**: Integration tests cover core local-first stack, E2E tests cover browser-specific behaviors

## Prerequisites

### Rust Integration Tests

```bash
# Rust toolchain (1.81+)
rustup update

# Dependencies installed via Cargo
# (automatic when running tests)
```

### Browser E2E Tests

```bash
# Node.js 18+
node --version

# Install dependencies
cd tests/e2e
npm install

# Install browsers
npx playwright install
```

## Running Tests

### 1. Integration Tests (Primary Quality Gate)

**All integration tests**:
```bash
cargo test --test integration
```

**With detailed output**:
```bash
cargo test --test integration -- --nocapture
```

**With logging**:
```bash
RUST_LOG=debug cargo test --test integration -- --nocapture
```

**Specific category**:
```bash
# Airplane mode tests
cargo test --test integration airplane_mode

# Network partitions
cargo test --test integration network_partition

# Concurrent edits
cargo test --test integration concurrent_edits

# Large documents
cargo test --test integration large_documents

# Schema evolution
cargo test --test integration schema_evolution
```

**Single test**:
```bash
cargo test --test integration test_airplane_mode_basic_cycle -- --exact
```

**Serial execution (debug)**:
```bash
cargo test --test integration -- --test-threads=1
```

### 2. Browser E2E Tests

**All E2E tests**:
```bash
cd tests/e2e
npm test
```

**Specific suite**:
```bash
npm run test:crash         # Crash recovery
npm run test:multi-tab     # Multi-tab sync
npm run test:offline       # Offline/online
```

**Interactive UI**:
```bash
npm run test:ui
```

**Debug mode**:
```bash
npm run test:debug
```

**Headed mode (see browser)**:
```bash
npm run test:headed
```

**Specific browser**:
```bash
npx playwright test --project=chromium
npx playwright test --project=firefox
npx playwright test --project=webkit
```

### 3. Unit Tests

**All crates**:
```bash
cargo test --workspace
```

**Specific crate**:
```bash
# State engine
cd crates/vudo-state && cargo test

# P2P networking
cd crates/vudo-p2p && cargo test

# Storage
cd crates/vudo-storage && cargo test
```

## Test Categories Explained

### Airplane Mode Tests

**Purpose**: Validate offline → online → sync workflows

**Key Tests**:
- `test_airplane_mode_basic_cycle`: Basic offline/online transition
- `test_concurrent_offline_edits`: Both nodes edit while offline
- `test_multiple_offline_online_cycles`: Stability across 10 cycles
- `test_no_data_loss_across_cycles`: 1000 cycles without data loss

**What to watch**:
- Data persistence during offline period
- Correct merge when coming back online
- No lost operations

### Network Partition Tests

**Purpose**: Validate partition tolerance and convergence

**Key Tests**:
- `test_five_node_partition`: 3-node vs 2-node partition
- `test_multiple_partition_heal_cycles`: Repeated partition/heal
- `test_partition_convergence_guarantee`: Always converges

**What to watch**:
- All nodes eventually converge (same hash)
- No data loss during partition
- Edits from both partitions merged correctly

### Concurrent Edits Tests

**Purpose**: Stress test CRDT convergence

**Key Tests**:
- `test_concurrent_ten_node_edits`: 10 nodes, 100 ops each
- `test_conflicting_updates_same_key`: Multiple nodes update same key
- `test_high_frequency_updates`: 1000 rapid updates

**What to watch**:
- All operations complete successfully
- No race conditions
- Final state is deterministic

### Large Documents Tests

**Purpose**: Validate performance with large data

**Key Tests**:
- `test_10mb_document_sync`: 10MB document sync time
- `test_incremental_sync_large_document`: Small change to large doc
- `test_throughput_measurement`: Measure MB/s

**What to watch**:
- Sync completes in target time (< 30s for 10MB)
- Memory usage reasonable
- No crashes with large documents

### Schema Evolution Tests

**Purpose**: Validate version compatibility

**Key Tests**:
- `test_forward_compatible_read`: v1 reads v2 document
- `test_backward_compatible_write`: v2 writes, v1 reads
- `test_three_version_compatibility`: v1, v2, v3 interop

**What to watch**:
- No errors when versions differ
- Missing fields handled gracefully
- Data integrity maintained

## Understanding Test Output

### Successful Run

```
running 7 tests
test airplane_mode::test_airplane_mode_basic_cycle ... ok
test airplane_mode::test_concurrent_offline_edits ... ok
test airplane_mode::test_multiple_offline_online_cycles ... ok
test airplane_mode::test_large_number_of_offline_operations ... ok
test airplane_mode::test_offline_delete_and_sync ... ok
test airplane_mode::test_no_data_loss_across_cycles ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Failed Test

```
---- airplane_mode::test_concurrent_offline_edits stdout ----
thread 'airplane_mode::test_concurrent_offline_edits' panicked at 'assertion failed: `(left == right)`
  left: "Alice",
 right: "Bob"', tests/integration/local-first/airplane_mode.rs:123:5
```

**Debugging failed tests**:
1. Read the assertion message
2. Check line number in source
3. Run with `--nocapture` to see println! output
4. Run with `RUST_LOG=debug` for detailed logs

## Performance Benchmarks

### Current Targets (Simulated Environment)

| Metric | Target | Test |
|--------|--------|------|
| 10MB document sync | < 30s | `test_10mb_document_sync` |
| 1000 operations | < 5s | `test_high_frequency_updates` |
| Incremental sync | < 5s | `test_incremental_sync_large_document` |
| Convergence | 100% | All tests |

### Running Benchmarks

```bash
# State engine benchmarks
cd crates/vudo-state
cargo bench

# Generate report
cargo bench -- --save-baseline main
```

## CI/CD Integration

Tests run automatically in CI:

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - name: Run integration tests
        run: cargo test --test integration --verbose
```

**CI Features**:
- Runs on: push, pull_request, schedule
- Multiple OS: Ubuntu, macOS, Windows
- Test timeout: 10 minutes
- Retry: 2 attempts on failure
- Artifacts: Test results, coverage reports

## Troubleshooting

### Integration Tests Won't Compile

```bash
# Clean and rebuild
cargo clean
cargo test --test integration
```

### Tests Timeout

```bash
# Increase timeout
RUST_TEST_TIMEOUT=600 cargo test --test integration
```

### Flaky Tests

```bash
# Run multiple times
for i in {1..10}; do cargo test --test integration test_name -- --exact; done
```

### E2E Tests Fail to Start

```bash
# Reinstall browsers
cd tests/e2e
npx playwright install --force
```

### Browser Tests Timeout

```bash
# Increase timeout in playwright.config.ts
export default defineConfig({
  timeout: 60000, // 60 seconds
});
```

## Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --test integration --out Html

# Open report
open tarpaulin-report.html
```

### Expected Coverage

- Integration tests: > 80% of local-first stack
- Unit tests: > 90% of individual crates
- E2E tests: > 70% of browser features

## Best Practices

### Writing Tests

1. **Use descriptive names**: `test_concurrent_offline_edits`, not `test1`
2. **Test one thing**: Each test should verify one specific behavior
3. **Use test harness**: Leverage `TestNode` for consistency
4. **Verify convergence**: Always check hashes match after sync
5. **Document expectations**: Comment on why test exists

### Running Tests

1. **Run locally first**: Before pushing, run all tests
2. **Check performance**: Monitor test duration, flag slow tests
3. **Review failures**: Don't ignore intermittent failures
4. **Update docs**: Keep README.md test counts current

### Debugging Tests

1. **Start simple**: Run single test with `--exact`
2. **Add logging**: Use `tracing::debug!()` liberally
3. **Use print debugging**: `println!()` works with `--nocapture`
4. **Check assumptions**: Verify setup code executed correctly
5. **Isolate issue**: Binary search to find failing component

## Common Issues

### "no such file or directory"

```bash
# Ensure you're in the right directory
cd tests/integration
cargo test
```

### "dependency not found"

```bash
# Update dependencies
cargo update
```

### "connection refused" (E2E)

```bash
# Ensure dev server is running
# Or configure webServer in playwright.config.ts
```

### "browser not found"

```bash
# Install Playwright browsers
npx playwright install
```

## Additional Resources

- [Integration Test README](./integration/README.md)
- [E2E Test README](./e2e/README.md)
- [Test Harness Documentation](./integration/local-first/test_harness.rs)
- [Playwright Docs](https://playwright.dev/)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)

## Support

For issues or questions:
1. Check this guide
2. Review test source code
3. Check CI logs
4. File an issue with test output

## Success Checklist

Before marking Phase 2 complete:

- [ ] All 39 integration tests pass
- [ ] All 15+ E2E tests pass
- [ ] 100% convergence verified (hash-based)
- [ ] No data loss in 1000 offline/online cycles
- [ ] 10MB document syncs in < 30s
- [ ] Browser crash recovery < 2s
- [ ] All tests pass in CI
- [ ] Coverage > 80% for integration
