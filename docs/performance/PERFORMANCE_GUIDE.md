# Performance Maintenance Guide

Quick reference for maintaining performance budgets in the MYCELIUM-SYNC stack.

## Performance Budgets

Always keep these targets in mind when developing:

| Metric | Budget | Impact |
|--------|--------|--------|
| WASM Module Size | < 100KB gzipped | Download speed, mobile experience |
| CRDT Merge | < 10ms for 10K ops | UI responsiveness |
| Sync Throughput | > 1000 ops/sec | Collaboration efficiency |
| Memory Usage | < 50MB for 100K records | Low-end device support |
| Startup Time | < 500ms cold start | User experience |

## Quick Checks

### Before Submitting PR

```bash
# Run regression tests
cd benchmarks
cargo test --release --test regression_tests

# If any test fails, fix before submitting
```

### Benchmark New Code

```bash
# Run relevant benchmarks
cargo bench --bench wasm_size     # If touching codegen
cargo bench --bench memory_usage   # If adding data structures
cargo bench --bench startup_time   # If touching initialization

# Compare results with baseline
```

### WASM Size Check

```bash
# Build and check size
cargo build --target wasm32-unknown-unknown --release
./scripts/optimize-wasm.sh target/wasm32-unknown-unknown/release/your_module.wasm

# Should see: ✅ Size budget met
```

## Common Performance Pitfalls

### 1. String Allocations

❌ **Bad**:
```rust
for i in 0..1000 {
    let s = format!("item_{}", i);  // 1000 allocations
    map.insert(s, value);
}
```

✅ **Good**:
```rust
use string_cache::DefaultAtom;

for i in 0..1000 {
    let s = DefaultAtom::from(format!("item_{}", i));  // Interned
    map.insert(s, value);
}
```

### 2. Small Vector Allocations

❌ **Bad**:
```rust
struct Document {
    fields: Vec<Field>,  // Always heap-allocated
}
```

✅ **Good**:
```rust
use smallvec::SmallVec;

struct Document {
    fields: SmallVec<[Field; 4]>,  // Stack if < 4 items
}
```

### 3. Eager Computation

❌ **Bad**:
```rust
fn create_document() -> Document {
    let state = compute_full_state();  // Expensive, may not be needed
    Document { state }
}
```

✅ **Good**:
```rust
fn create_document() -> Document {
    Document {
        state: OnceCell::new(),  // Compute only when accessed
    }
}
```

### 4. Synchronous Initialization

❌ **Bad**:
```rust
async fn init() {
    let state = StateEngine::new().await;      // Blocks
    let p2p = P2PNode::new(state).await;       // Blocks
    let credit = CreditSystem::new().await;    // Blocks
    // Total: 1200ms
}
```

✅ **Good**:
```rust
async fn init() {
    let state = StateEngine::new().await;      // Quick

    // Initialize rest in background
    tokio::spawn(async { P2PNode::new(state).await });
    tokio::spawn(async { CreditSystem::new().await });

    // Return immediately
}
```

### 5. Full Document Merge

❌ **Bad**:
```rust
fn merge(&mut self, other: &Document) {
    // Re-merge entire history (expensive)
    self.apply_all_operations(other.operations());
}
```

✅ **Good**:
```rust
fn merge(&mut self, other: &Document) {
    // Only merge new operations
    let new_ops = other.operations_since(&self.last_sync_clock);
    self.apply_operations(new_ops);
    self.last_sync_clock = other.current_clock();
}
```

## Performance Checklist

When adding new features:

### ✅ Data Structures

- [ ] Use `SmallVec` for < 4 items
- [ ] Use `string_cache::DefaultAtom` for repeated strings
- [ ] Use `HashMap` (not `BTreeMap`) unless ordering needed
- [ ] Consider `Arena` allocator for many same-size objects
- [ ] Use `parking_lot` locks (not std::sync)

### ✅ WASM Code

- [ ] Feature-gate unused CRDTs (tree shaking)
- [ ] Avoid generic blanket implementations (bloat)
- [ ] Use `#[inline]` for hot paths (carefully)
- [ ] Test gzipped size with `optimize-wasm.sh`
- [ ] Profile WASM in browser DevTools

### ✅ Async Code

- [ ] Spawn background tasks for non-critical work
- [ ] Use `tokio::spawn` for parallelism
- [ ] Batch operations when possible
- [ ] Use channels for async communication
- [ ] Avoid `block_on` in async context

### ✅ Memory Management

- [ ] Reuse allocations when possible
- [ ] Use `with_capacity` for collections
- [ ] Drop large allocations early
- [ ] Profile with Valgrind on Linux
- [ ] Monitor memory in browser DevTools

### ✅ Initialization

- [ ] Use `OnceCell` for lazy initialization
- [ ] Initialize async (tokio::spawn)
- [ ] Load persisted state in background
- [ ] Progressive enhancement (phased startup)
- [ ] Measure with `startup_time` benchmark

## Profiling Tools

### CPU Profiling

```bash
# Generate flame graph
cargo install flamegraph
cargo flamegraph --bench your_benchmark

# Open flamegraph.svg
```

### Memory Profiling (Linux)

```bash
# Run with Valgrind
valgrind --tool=massif cargo bench --bench memory_usage

# Generate report
ms_print massif.out > memory-report.txt
```

### WASM Profiling (Browser)

1. Open Chrome DevTools
2. Performance tab → Record
3. Run your WASM app
4. Stop recording
5. Analyze flame graph

### Benchmark Profiling

```bash
# Profile specific benchmark
cargo bench --bench wasm_size -- --profile-time=10

# Open Criterion reports
open target/criterion/report/index.html
```

## Optimization Workflow

### 1. Measure Baseline

```bash
cargo bench --bench relevant_benchmark
# Note current performance
```

### 2. Profile

```bash
cargo flamegraph --bench relevant_benchmark
# Identify hotspots
```

### 3. Optimize

Make targeted changes to hot paths.

### 4. Measure Again

```bash
cargo bench --bench relevant_benchmark
# Compare with baseline
```

### 5. Verify Budgets

```bash
cd benchmarks
cargo test --release --test regression_tests
# Should all pass
```

## When to Optimize

### ✅ Do Optimize

- [ ] Hot paths (> 10% CPU in flamegraph)
- [ ] Budget violations (regression tests fail)
- [ ] User-reported performance issues
- [ ] Critical path (startup, sync, merge)

### ❌ Don't Optimize

- [ ] Code that runs once (initialization)
- [ ] Already fast enough (< 1ms)
- [ ] Makes code unreadable
- [ ] Without profiling first

## Performance Review Checklist

For code reviewers:

### Code Changes

- [ ] No obvious performance issues (N² algorithms, etc.)
- [ ] Appropriate data structures used
- [ ] Memory allocations minimized
- [ ] Async code doesn't block unnecessarily

### Testing

- [ ] Benchmarks included for new features
- [ ] Regression tests pass
- [ ] No significant slowdown vs main branch

### Documentation

- [ ] Performance-critical code documented
- [ ] Trade-offs explained
- [ ] Budget impacts noted

## Performance Regression Handling

If CI fails with performance regression:

### 1. Identify Cause

```bash
# Compare benchmarks
git checkout main
cargo bench --bench regression_tests -- --save-baseline main

git checkout your-branch
cargo bench --bench regression_tests -- --baseline main

# Look for regressions
cargo benchcmp main your-branch
```

### 2. Profile Regression

```bash
cargo flamegraph --bench the_failing_benchmark
# Find the hot spot
```

### 3. Fix or Justify

Either:
- Fix the performance regression, OR
- Update budget if regression is acceptable (with justification)

### 4. Document

Add comment to PR explaining:
- What caused the regression
- Why it's necessary (if unfixable)
- What mitigation was attempted

## Maintenance Tasks

### Weekly

- [ ] Review benchmark trends
- [ ] Check for performance regressions
- [ ] Update optimization report if needed

### Monthly

- [ ] Run full benchmark suite
- [ ] Profile on low-end devices
- [ ] Review and tighten budgets if possible

### Quarterly

- [ ] Comprehensive performance audit
- [ ] Evaluate new optimization opportunities
- [ ] Update performance documentation

## Resources

- **Optimization Report**: [optimization-report.md](optimization-report.md)
- **Benchmark Documentation**: [../../benchmarks/README.md](../../benchmarks/README.md)
- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **WASM Size Optimization**: https://rustwasm.github.io/book/reference/code-size.html
- **Criterion.rs Guide**: https://bheisler.github.io/criterion.rs/book/

## Quick Reference Commands

```bash
# Run all benchmarks
cargo bench

# Run regression tests
cd benchmarks && cargo test --release --test regression_tests

# Profile CPU
cargo flamegraph --bench your_benchmark

# Check WASM size
./scripts/optimize-wasm.sh target/wasm32-unknown-unknown/release/module.wasm

# Memory profile (Linux)
valgrind --tool=massif cargo bench --bench memory_usage

# Compare benchmarks
cargo benchcmp baseline current
```

## Getting Help

Performance issues? Try:

1. Check this guide for common pitfalls
2. Run profiling tools to identify bottlenecks
3. Review optimization report for examples
4. Ask in #performance Slack channel
5. Create GitHub issue with profiling data

---

**Remember**: Profile first, optimize second. Measure, don't guess!
