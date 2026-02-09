# Eg-walker Integration Evaluation Prototype

This prototype evaluates Martin Kleppmann's **eg-walker** algorithm (via `diamond-types`) as a potential replacement for Automerge in DOL's text CRDT implementation.

## Background

- **Paper**: "Collaborative Text Editing with Eg-walker: Better, Faster, Smaller"
- **Authors**: Joseph Gentle and Martin Kleppmann
- **Published**: EuroSys 2025
- **Award**: Gilles Muller Best Artifact Award
- **Claims**:
  - Orders of magnitude less memory than existing CRDTs
  - Orders of magnitude faster document loading
  - Orders of magnitude faster merge for diverged branches vs OT
  - Peer-to-peer capable, no central server required

## Project Structure

```
prototypes/eg-walker-dol/
├── src/
│   ├── lib.rs              # Main library with TextCrdt trait
│   ├── egwalker.rs         # Eg-walker implementation via diamond-types
│   ├── automerge_wrapper.rs # Automerge wrapper for comparison
│   ├── correctness.rs      # CRDT property verification
│   └── benchmarks.rs       # Benchmark utilities
├── benches/
│   ├── text_operations.rs  # Insert/delete benchmarks
│   ├── merge_performance.rs # Merge speed benchmarks
│   └── memory_footprint.rs # Memory usage benchmarks
├── examples/
│   ├── basic_usage.rs      # Simple eg-walker usage
│   ├── concurrent_editing.rs # Concurrent editing demo
│   └── comparison_benchmark.rs # Interactive comparison
└── tests/
    └── integration_tests.rs # Integration tests

```

## Building

```bash
cd prototypes/eg-walker-dol
cargo build --release
```

## Running Examples

### Basic Usage
```bash
cargo run --example basic_usage
```

### Concurrent Editing
```bash
cargo run --example concurrent_editing
```

### Full Comparison
```bash
cargo run --example comparison_benchmark
```

## Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench text_operations
cargo bench merge_performance
cargo bench memory_footprint
```

## Running Tests

```bash
# All tests
cargo test

# Correctness tests only
cargo test correctness

# Integration tests
cargo test --test integration_tests
```

## Evaluation Criteria

### 1. Performance
- **Insert/Delete Latency**: Single operation timing
- **Merge Speed**: Two-way and multi-way merge performance
- **Load Time**: Document deserialization speed
- **Memory Usage**: Runtime memory footprint
- **Serialization Size**: Disk/network overhead

### 2. Correctness
- **Convergence**: All replicas reach same state
- **Commutativity**: Merge order independence
- **Associativity**: Merge grouping independence
- **Idempotency**: Multiple merges = single merge
- **Causality**: Causal order preservation

### 3. API Ergonomics
- **Ease of Use**: Learning curve, API simplicity
- **Integration Effort**: Code changes required
- **Type Safety**: Compile-time guarantees
- **Error Handling**: Error reporting quality

### 4. Maintenance
- **Library Maturity**: Production readiness
- **Ecosystem Support**: Community, documentation
- **Long-term Viability**: Active development
- **Breaking Changes**: API stability

## Key Findings (Preliminary)

### Performance

**Eg-walker Advantages:**
- ✓ Significantly faster document loading (deserializa)
- ✓ Lower memory footprint per operation
- ✓ Faster merge for highly diverged branches

**Automerge Advantages:**
- ✓ More mature optimization for common cases
- ✓ Better integration with Rust ecosystem

### Correctness

Both implementations pass all CRDT property tests:
- ✓ Convergence
- ✓ Commutativity
- ✓ Associativity
- ✓ Idempotency
- ✓ Causality preservation

### API Ergonomics

**Eg-walker:**
- Simple, focused API
- Direct character-level operations
- Minimal abstraction overhead

**Automerge:**
- Richer feature set (marks, formatting)
- More complex but more powerful
- Better documentation and examples

### Integration Complexity

**Eg-walker:**
- Requires new wrapper code
- Simpler data model
- Less ecosystem integration

**Automerge:**
- Already integrated in DOL
- Autosurgeon derive macros
- Rich ecosystem (JS/TS bindings, etc.)

## Recommendation

See `docs/research/eg-walker-evaluation.md` for the complete evaluation report and final recommendation.

## References

- [Eg-walker Paper (EuroSys 2025)](https://martin.kleppmann.com/2025/03/30/eg-walker-collaborative-text.html)
- [Diamond-types Rust Implementation](https://crates.io/crates/diamond-types)
- [Eg-walker GitHub Repository](https://github.com/josephg/egwalker-paper)
- [Automerge](https://automerge.org/)
- [DOL CRDT Guide](../../docs/book/local-first/crdt-guide/)
