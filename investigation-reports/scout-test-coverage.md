# DOL Test Coverage Analysis Report

**Generated**: 2025-12-30
**Scout Agent**: Test Suite Analysis

---

## Summary Statistics

| Metric | Count |
|--------|-------|
| **Total Tests** | 1,755 |
| Tests in `tests/` directory | 1,140 |
| Tests in `src/` directory (inline) | 475 |
| Tests in `archive/` (legacy) | 138 |
| Tests in `stage2/` | 2 |

---

## Tests by Module (src/)

| Module | Test Count |
|--------|------------|
| `codegen/` | 102 |
| `lower/` | 50 |
| `macros/` | 36 |
| `network/` | 29 |
| `mlir/` | 28 |
| `transform/` | 22 |
| `sex/` | 22 |
| `typechecker.rs` | 21 |
| `hir/` | 21 |
| `lexer.rs` | 20 |
| `bin/` | 18 |
| `reflect.rs` | 17 |
| `validator.rs` | 16 |
| `eval/` | 16 |
| `wasm/` | 11 |
| `swarm/` | 10 |
| `pratt.rs` | 7 |
| `compiler/` | 7 |
| `test_parser.rs` | 5 |
| `parser.rs` | 4 |
| `mcp/` | 4 |
| `ast.rs` | 4 |
| `error.rs` | 3 |
| `lib.rs` | 2 |

---

## Tests by File (tests/ directory)

| Test File | Test Count |
|-----------|------------|
| `parser_exhaustive.rs` | 118 |
| `adaptive_tests.rs` | 93 |
| `parser_tests.rs` | 80 |
| `lexer_tests.rs` | 77 |
| `expr_comprehensive.rs` | 73 |
| `parser_stress.rs` | 61 |
| `lexer_stress.rs` | 58 |
| `ast_structure.rs` | 54 |
| `sex_tests.rs` | 46 |
| `typechecker_exhaustive.rs` | 44 |
| `dol2_tests.rs` | 42 |
| `integration_comprehensive.rs` | 39 |
| `codegen_rust_tests.rs` | 36 |
| `quote_tests.rs` | 34 |
| `compiler_e2e.rs` | 32 |
| `codegen_unit.rs` | 31 |
| `error_handling.rs` | 30 |
| `idiom_tests.rs` | 27 |
| `lexer_exhaustive.rs` | 27 |
| `validator_unit.rs` | 27 |
| `integration_tests.rs` | 24 |
| **`wasm_execution.rs`** | **23** |
| `codegen_operators_test.rs` | 18 |
| `reflect_tests.rs` | 17 |
| `biology_tests.rs` | 8 |
| **`compiler_integration.rs`** | **5** |
| `stress_tests.rs` | 5 |
| `codegen_golden.rs` | 1 |

---

## WASM-Related Tests

### Location: `tests/wasm_execution.rs`
- **23 tests** specifically for WASM execution and runtime
- Feature-gated with `#[cfg(feature = "wasm")]`
- Tests include:
  - WASM magic number validation
  - WASM version validation
  - Invalid magic number handling
  - Invalid version handling
  - Truncated module handling
  - Empty module handling
  - Runtime initialization
  - Module loading and execution

### Location: `tests/compiler_integration.rs`
- **5 tests** for DOL-to-WASM compilation pipeline
- Feature-gated with `#[cfg(feature = "wasm")]`
- Tests include:
  - WASM output verification (magic number, version)
  - Compilation error handling

### Location: `src/wasm/`
- `compiler.rs`: **8 inline tests** for WASM compiler
- `runtime.rs`: **3 inline tests** for WASM runtime

**Total WASM Tests: 39**

---

## DOL Fixture Files

### Examples Directory (31 files)
| Category | Files |
|----------|-------|
| `examples/genes/` | `counter.dol`, `hello.world.dol`, `network.core.dol`, `identity.cryptographic.dol`, `container.exists.dol` |
| `examples/traits/` | `container.lifecycle.dol`, `greetable.dol`, `countable.dol`, `container.networking.dol`, `node.discovery.dol` |
| `examples/constraints/` | `counter_bounds.dol`, `greeting_protocol.dol`, `container.integrity.dol`, `identity.immutable.dol` |
| `examples/evolutions/` | `container.lifecycle.v0.0.2.dol`, `identity.cryptographic.v0.0.2.dol` |
| `examples/systems/` | `univrs.orchestrator.dol`, `bounded.counter.dol`, `univrs.scheduler.dol`, `greeting.service.dol` |
| `examples/stdlib/biology/` | `evolution.dol`, `ecosystem.dol`, `hyphal.dol`, `types.dol`, `transport.dol`, `mycelium.dol`, `mod.dol` |
| `examples/stdlib/network/` | `hyphal_network.dol` |

### Test Corpus (5 files)
| Path |
|------|
| `tests/corpus/genes/nested_generics.dol` |
| `tests/corpus/genes/evolution_chain.dol` |
| `tests/corpus/genes/complex_constraints.dol` |
| `tests/corpus/sex/nested_sex.dol` |
| `tests/corpus/traits/trait_relationships.dol` |

### Golden Test Input (4 files)
| Path |
|------|
| `tests/codegen/golden/input/simple_gene.dol` |
| `tests/codegen/golden/input/function.dol` |
| `tests/codegen/golden/input/gene_with_fields.dol` |
| `tests/codegen/golden/input/pipe_operators.dol` |

### Standard Library (10 files)
| Path |
|------|
| `stdlib/constraint.transfer_invariants.dol` |
| `stdlib/traits.monad.dol` |
| `stdlib/constraint.transfer_result.dol` |
| `stdlib/constraint.transferability.dol` |
| `stdlib/constraint.transfer.dol` |
| `stdlib/gene.map_preserving.dol` |
| `stdlib/gene.map_strict.dol` |
| `stdlib/traits.applicative.dol` |
| `stdlib/traits.functor.dol` |
| `stdlib/gene.map.dol` |

### DOL Bootstrap/Self-Describing (10 files)
| Path |
|------|
| `dol/main.dol`, `dol/ast.dol`, `dol/types.dol`, `dol/token.dol`, `dol/typechecker.dol` |
| `dol/parser.dol`, `dol/bootstrap.dol`, `dol/codegen.dol`, `dol/mod.dol`, `dol/lexer.dol` |

### Documentation (2 files)
| Path |
|------|
| `docs/ontology/prospective/storage.dol` |
| `docs/ontology/prospective/identity.dol` |

### DOL Test Files (.dol.test)
| Path |
|------|
| `examples/container.lifecycle.dol.test` |

**Total .dol Files: 65**

---

## Integration Test Structure

### tests/ Subdirectories
| Directory | Status |
|-----------|--------|
| `tests/corpus/` | Contains test corpus with genes, sex, systems, traits |
| `tests/codegen/golden/` | Golden test fixtures (input/expected) |
| `tests/e2e/` | Empty (placeholder) |
| `tests/hir/` | Empty (placeholder) |
| `tests/integration/` | Empty (placeholder) |
| `tests/generated/` | Auto-generated test files |

---

## Observations

1. **Strong Test Coverage**: 1,755 total tests is substantial coverage
2. **WASM Tests Present**: 39 WASM-related tests across 4 files
3. **Feature-Gated**: WASM tests require `--features wasm` to run
4. **Inline Tests**: Many modules use inline `#[cfg(test)]` modules
5. **Golden Testing**: Codegen uses golden file testing pattern
6. **Empty Directories**: `e2e/`, `hir/`, `integration/` are placeholders for future tests
7. **Legacy Archive**: 138 tests in archive (not actively compiled)

---

## Recommendations

1. Enable WASM feature in CI: `cargo test --features wasm`
2. Populate empty test directories with actual integration tests
3. Add more golden tests for WASM output validation
4. Consider migrating archive tests or removing them
5. Add coverage reporting to CI pipeline
