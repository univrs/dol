# VUDO Performance Benchmarks

Comprehensive benchmark suite for the MYCELIUM-SYNC local-first stack, measuring performance against strict budget targets.

## Performance Budgets

| Metric | Target | Purpose |
|--------|--------|---------|
| WASM Module Size | < 100KB gzipped | Fast downloads, good mobile experience |
| CRDT Merge Latency | < 10ms for 10K ops | Responsive sync, no UI blocking |
| Sync Throughput | > 1000 ops/sec | Efficient collaboration |
| Memory Usage | < 50MB for 100K records | Runs on low-end devices |
| Startup Time | < 500ms cold start | Fast app initialization |

## Benchmark Structure

```
benchmarks/
├── wasm_size.rs           # WASM module size benchmarks
├── memory_usage.rs        # Memory profiling benchmarks
├── startup_time.rs        # Initialization benchmarks
└── regression_tests.rs    # CI regression tests (fail on budget violations)
```

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark

```bash
# WASM size benchmarks
cargo bench --bench wasm_size

# Memory usage benchmarks
cargo bench --bench memory_usage

# Startup time benchmarks
cargo bench --bench startup_time

# Regression tests (CI mode)
cargo test --release --test regression_tests
```

### Run with HTML Reports

Criterion generates HTML reports in `target/criterion/`:

```bash
cargo bench
open target/criterion/report/index.html
```

## Benchmark Categories

### 1. WASM Size (`wasm_size.rs`)

Measures compiled WASM module size with various optimizations:

- **Simple Gen Module**: Basic Gen with 3 CRDT fields
- **Multi-CRDT Module**: Gen with 5-50 CRDT fields (scalability)
- **wasm-opt Optimization**: Before/after wasm-opt -Oz
- **Tree Shaking**: Impact of feature flags
- **Dead Code Elimination**: Impact of LTO

**Usage**:
```bash
cargo bench --bench wasm_size
```

### 2. Memory Usage (`memory_usage.rs`)

Measures memory consumption for various dataset sizes:

- **100K Records**: Verify < 50MB budget
- **Memory Scaling**: 1K, 10K, 50K, 100K records
- **String Interning**: Impact on memory usage
- **Compact Structures**: SmallVec vs Vec
- **Memory Pooling**: Arena allocation vs heap
- **Lazy Materialization**: Eager vs lazy state computation

**Usage**:
```bash
cargo bench --bench memory_usage
```

**Note**: Memory profiling requires Linux (uses `/proc/self/status`). On other platforms, memory measurements return 0.

### 3. Startup Time (`startup_time.rs`)

Measures initialization time from cold start:

- **Cold Start**: Full initialization < 500ms
- **WASM Instantiation**: Module loading time
- **Lazy Initialization**: OnceCell vs eager init
- **Async Initialization**: Blocking vs background init
- **State Engine Init**: Isolated state engine startup
- **P2P Node Init**: Isolated P2P startup
- **Persisted State Loading**: Empty, small, large state
- **WASM Streaming**: Streaming vs blocking compilation
- **Incremental Startup**: Progressive enhancement

**Usage**:
```bash
cargo bench --bench startup_time
```

### 4. Regression Tests (`regression_tests.rs`)

CI-friendly tests that fail if budgets are exceeded:

- `test_crdt_merge_latency_budget` - Fails if merge > 10ms
- `test_sync_throughput_budget` - Fails if throughput < 1000 ops/sec
- `test_memory_usage_budget` - Fails if memory > 50MB (Linux only)
- `test_startup_time_budget` - Fails if startup > 500ms
- `test_state_engine_operations_performance` - Individual operation budgets
- `test_batch_operations_performance` - Batch operation budgets
- `test_concurrent_operations_performance` - Concurrent operation budgets

**Usage**:
```bash
# Run all regression tests
cargo test --release --test regression_tests

# Run single test
cargo test --release --test regression_tests test_crdt_merge_latency_budget

# With verbose output
cargo test --release --test regression_tests -- --nocapture
```

**CI Integration**:
```yaml
- name: Performance Regression Tests
  run: cargo test --release --test regression_tests
```

## Crate-Level Benchmarks

### vudo-state

Located in `crates/vudo-state/benches/`:

- **state_engine.rs**: Document create/read/write, subscriptions, transactions
- **document_store.rs**: Document storage operations
- **reactive.rs**: Reactive subscriptions and notifications

```bash
cargo bench -p vudo-state
```

### vudo-p2p

Located in `crates/vudo-p2p/benches/`:

- **sync_performance.rs**: Sync throughput, merge latency, delta sync, parallel sync

```bash
cargo bench -p vudo-p2p
```

### vudo-credit

Located in `crates/vudo-credit/benches/`:

- **credit_benchmarks.rs**: Local spend, escrow allocation, overdraft detection, BFT reconciliation

```bash
cargo bench -p vudo-credit
```

## Interpreting Results

### Criterion Output

```
wasm_size/simple_gen    time:   [1.234 ms 1.245 ms 1.256 ms]
                        change: [-5.2% -3.8% -2.4%] (p = 0.00 < 0.05)
                        Performance has improved.
```

- **time**: Mean, median, and standard deviation
- **change**: Performance change from previous run
- **p-value**: Statistical significance (< 0.05 = significant)

### Regression Test Output

```
✅ CRDT merge latency: 7ms (budget: 10ms)
✅ Sync throughput: 1500.00 ops/sec (budget: 1000 ops/sec)
✅ Memory usage: 42MB (budget: 50MB)
✅ Startup time: 380ms (budget: 500ms)
```

Green checkmarks indicate budgets are met. Tests fail with descriptive errors if budgets are exceeded.

## Optimization Tips

### Before Optimizing

1. **Profile First**: Use `cargo flamegraph` to identify hotspots
2. **Measure Baseline**: Run benchmarks before making changes
3. **Set Goals**: Know what "fast enough" means
4. **One Change at a Time**: Isolate the impact of each optimization

### Common Optimizations

1. **Reduce Allocations**: Use `SmallVec`, `String::with_capacity`
2. **Batch Operations**: Process multiple items at once
3. **Lazy Computation**: Compute only when needed
4. **Cache Results**: Memoize expensive computations
5. **Use Efficient Data Structures**: HashMap vs BTreeMap, Vec vs LinkedList
6. **Parallel Processing**: Use `tokio::spawn` for independent work
7. **Compression**: Reduce network payload sizes
8. **String Interning**: Deduplicate common strings

### WASM-Specific Optimizations

1. **LTO**: Enable link-time optimization in `Cargo.toml`
2. **wasm-opt**: Run `wasm-opt -Oz` on compiled modules
3. **Tree Shaking**: Use feature flags to exclude unused code
4. **Strip Symbols**: Remove debug symbols in release builds
5. **Small Allocator**: Use `wee_alloc` for smaller WASM size (if applicable)

## Performance Budgets in CI

Add to `.github/workflows/ci.yml`:

```yaml
name: Performance Tests

on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run performance regression tests
        run: cargo test --release --test regression_tests
      - name: Upload benchmark results
        uses: actions/upload-artifact@v2
        if: always()
        with:
          name: benchmark-results
          path: target/criterion
```

## Profiling Tools

### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile benchmarks
cargo flamegraph --bench wasm_size

# Open flamegraph.svg
open flamegraph.svg
```

### Memory Profiling

```bash
# Linux: Use Valgrind
valgrind --tool=massif cargo bench --bench memory_usage

# macOS: Use Instruments
instruments -t "Allocations" cargo bench --bench memory_usage
```

### WASM Profiling

Use browser DevTools:

1. Open Chrome DevTools
2. Performance tab → Record
3. Run WASM application
4. Stop recording → Analyze flame graph

## Continuous Performance Monitoring

### Set Up Benchmarking CI

1. Run benchmarks on every PR
2. Compare against main branch
3. Fail if performance regresses > 10%
4. Store historical benchmark data

### Example: GitHub Actions

```yaml
- name: Benchmark PR
  run: |
    cargo bench --bench regression_tests -- --save-baseline pr
    cargo bench --bench regression_tests -- --baseline main
    cargo benchcmp main pr
```

## Troubleshooting

### Benchmarks Too Slow

- Reduce dataset sizes in benchmarks
- Use `--sample-size` to reduce iterations
- Run specific benchmarks instead of all

### Inconsistent Results

- Close other applications
- Disable CPU frequency scaling
- Run multiple times and average

### CI Failures

- Check if budgets are too strict
- Verify platform differences (Linux vs macOS)
- Check for non-deterministic behavior

## Resources

- [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [WASM Optimization Guide](https://rustwasm.github.io/book/reference/code-size.html)
- [Performance Optimization Report](../docs/performance/optimization-report.md)

## Contributing

When adding new benchmarks:

1. Follow existing naming conventions
2. Include budget assertions for regression tests
3. Document the purpose and expected results
4. Add to CI pipeline if critical

---

**Questions?** See the [Performance Optimization Report](../docs/performance/optimization-report.md) for detailed analysis.
