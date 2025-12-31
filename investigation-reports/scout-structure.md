# DOL Repository Structure - Scout Report

**Date**: 2025-12-30
**Agent**: SCOUT
**Objective**: Fast reconnaissance of univrs-dol repository structure

---

## Directory Tree

```
univrs-dol/
├── src/                          # Main Rust source code
│   ├── lib.rs                    # Library entry point (metadol crate)
│   ├── lexer.rs                  # Tokenizer using logos
│   ├── parser.rs                 # Recursive descent parser
│   ├── pratt.rs                  # Pratt expression parser
│   ├── ast.rs                    # AST definitions
│   ├── error.rs                  # Error types with thiserror
│   ├── validator.rs              # Semantic validation
│   ├── typechecker.rs            # Type inference and checking
│   ├── reflect.rs                # Reflection system
│   ├── hir/                      # High-level IR
│   │   ├── mod.rs, types.rs, desugar.rs, validate.rs, visit.rs, span.rs, symbol.rs, print.rs
│   ├── lower/                    # AST -> HIR lowering
│   │   ├── mod.rs, context.rs, decl.rs, expr.rs, stmt.rs, desugar.rs
│   ├── codegen/                  # Code generation backends
│   │   ├── mod.rs, rust.rs, typescript.rs, jsonschema.rs, hir_rust.rs, crate_gen.rs
│   ├── mlir/                     # MLIR backend (feature-gated)
│   │   ├── mod.rs, ops.rs, types.rs, context.rs, lowering.rs, codegen.rs
│   ├── wasm/                     # WASM backend (feature-gated)
│   │   ├── mod.rs, compiler.rs, runtime.rs
│   ├── compiler/                 # Spirit compiler
│   │   ├── mod.rs, spirit.rs
│   ├── eval/                     # Expression evaluation
│   │   ├── mod.rs, interpreter.rs, builtins.rs, value.rs
│   ├── macros/                   # Macro system
│   │   ├── mod.rs, builtin.rs, expand.rs
│   ├── transform/                # AST transformation
│   │   ├── mod.rs, passes.rs, fold.rs, visitor.rs, desugar_idiom.rs
│   ├── sex/                      # Side Effect eXecution tracking
│   │   ├── mod.rs, context.rs, lint.rs, tracking.rs
│   ├── mcp/                      # Model Context Protocol server
│   │   ├── mod.rs, server.rs, tools.rs
│   ├── network/                  # Hyphal network topology
│   │   ├── mod.rs, discovery.rs, growth.rs, topology.rs
│   ├── swarm/                    # Agent swarm coordination
│   │   ├── mod.rs, hyphal_coordinator.rs
│   └── bin/                      # CLI binaries
│       ├── dol-parse.rs, dol-test.rs, dol-check.rs, dol-codegen.rs
│       ├── dol-mcp.rs, dol-build-crate.rs, dol-migrate.rs
├── dol/                          # Self-hosting DOL source
│   ├── main.dol, mod.dol, lexer.dol, parser.dol, ast.dol, token.dol
│   ├── types.dol, typechecker.dol, codegen.dol, bootstrap.dol
├── examples/                     # Example .dol files
│   ├── genes/, traits/, constraints/, systems/, evolutions/
│   ├── stdlib/biology/, stdlib/network/
├── stdlib/                       # Standard library definitions
├── tests/                        # Test suite
│   ├── lexer_tests.rs, parser_tests.rs, integration_tests.rs
│   ├── wasm_execution.rs, codegen_*.rs, hir/, corpus/
├── docs/                         # Documentation
├── stage1/, stage2/              # Bootstrap stages
├── archive/                      # Archived files
└── flywheel/, scripts/, bootstrap-dolv3/  # Build utilities
```

---

## Key Dependencies by Category

### Core Dependencies
| Dependency | Version | Purpose |
|------------|---------|---------|
| `thiserror` | 1.0 | Error type derivation |
| `logos` | 0.14 | Fast lexer generator |
| `home` | =0.5.9 | Pinned (melior dependency) |

### Optional: CLI Support (`cli` feature)
| Dependency | Version | Purpose |
|------------|---------|---------|
| `clap` | 4.4 | CLI argument parsing |
| `anyhow` | 1.0 | Error handling |
| `colored` | 2.1 | Terminal coloring |
| `regex` | 1 | Regular expressions |

### Optional: Serialization (`serde` feature)
| Dependency | Version | Purpose |
|------------|---------|---------|
| `serde` | 1.0 | Serialization framework |
| `serde_json` | 1.0 | JSON serialization |

### Optional: MLIR Backend (`mlir` feature)
| Dependency | Version | Purpose |
|------------|---------|---------|
| `melior` | 0.18 | MLIR Rust bindings |

### Optional: WASM Backend (`wasm` feature)
| Dependency | Version | Purpose |
|------------|---------|---------|
| `wasmtime` | 21 | WASM runtime engine |
| `wasm-encoder` | 0.41 | WASM bytecode emission |

### Dev Dependencies
| Dependency | Version | Purpose |
|------------|---------|---------|
| `pretty_assertions` | 1.4 | Test assertions |
| `insta` | 1.34 | Snapshot testing |
| `criterion` | 0.5 | Benchmarking |
| `tempfile` | 3.9 | Temporary files for tests |

---

## Feature Flags

```toml
[features]
default = []
cli = ["dep:clap", "dep:anyhow", "dep:colored", "dep:regex", "serde"]
serde = ["dep:serde", "dep:serde_json"]
mlir = ["melior"]
wasm = ["wasmtime", "wasm-encoder"]
wasm-mlir = ["wasm", "mlir"]  # Combined WASM+MLIR pipeline
```

---

## Module Structure (from src/lib.rs)

### Public Modules
- `ast` - Abstract Syntax Tree definitions
- `codegen` - Code generation (Rust, TypeScript, JSON Schema)
- `error` - Error types with source location
- `eval` - Expression evaluation/interpreter
- `hir` - High-level Intermediate Representation
- `lexer` - Tokenization (logos-based)
- `lower` - AST to HIR lowering
- `macros` - Compile-time metaprogramming
- `parser` - Recursive descent parser
- `pratt` - Pratt expression parser
- `reflect` - Runtime reflection system
- `sex` - Side Effect eXecution tracking
- `transform` - AST transformation passes
- `typechecker` - Type inference and checking
- `validator` - Semantic validation

### Conditional Modules
- `mcp` - Model Context Protocol server (requires `serde`)
- `mlir` - MLIR code generation (requires `mlir`)
- `wasm` - WASM compilation (requires `wasm`)
- `compiler` - Spirit compiler (requires `wasm`)
- `test_parser` - Test file parser (requires `cli`)

### Hyphal Network Modules
- `network` - Network topology, discovery, growth
- `swarm` - Agent swarm coordination

---

## Binary Targets

| Binary | Path | Required Features |
|--------|------|-------------------|
| `dol-parse` | src/bin/dol-parse.rs | `cli` |
| `dol-test` | src/bin/dol-test.rs | `cli` |
| `dol-check` | src/bin/dol-check.rs | `cli` |
| `dol-codegen` | src/bin/dol-codegen.rs | `cli` |
| `dol-mcp` | src/bin/dol-mcp.rs | `cli` |
| `dol-build-crate` | src/bin/dol-build-crate.rs | `cli` |
| `dol-migrate` | src/bin/dol-migrate.rs | `cli` |

---

## Example Files Found (65 total)

### examples/ directory (24 files)
- **genes/**: container.exists.dol, counter.dol, hello.world.dol, identity.cryptographic.dol, network.core.dol
- **traits/**: container.lifecycle.dol, container.networking.dol, countable.dol, greetable.dol, node.discovery.dol
- **constraints/**: container.integrity.dol, counter_bounds.dol, greeting_protocol.dol, identity.immutable.dol
- **systems/**: bounded.counter.dol, greeting.service.dol, univrs.orchestrator.dol, univrs.scheduler.dol
- **evolutions/**: container.lifecycle.v0.0.2.dol, identity.cryptographic.v0.0.2.dol
- **stdlib/biology/**: types.dol, mod.dol, transport.dol, hyphal.dol, ecosystem.dol, mycelium.dol, evolution.dol
- **stdlib/network/**: hyphal_network.dol

### stdlib/ directory (9 files)
- constraint.transfer.dol, constraint.transferability.dol, constraint.transfer_result.dol, constraint.transfer_invariants.dol
- gene.map.dol, gene.map_preserving.dol, gene.map_strict.dol
- traits.applicative.dol, traits.functor.dol, traits.monad.dol

### dol/ directory - Self-hosting (10 files)
- main.dol, mod.dol, lexer.dol, parser.dol, ast.dol, token.dol
- types.dol, typechecker.dol, codegen.dol, bootstrap.dol

### tests/corpus/ (5 files)
- genes/nested_generics.dol, genes/complex_constraints.dol, genes/evolution_chain.dol
- traits/trait_relationships.dol, sex/nested_sex.dol

### tests/codegen/golden/input/ (4 files)
- simple_gene.dol, gene_with_fields.dol, function.dol, pipe_operators.dol

---

## WASM Pipeline Architecture

### Current Implementation (Direct Emission)
```
DOL AST -> WASM Bytecode (via wasm-encoder)
```
- Located in: `src/wasm/compiler.rs`
- Uses: `wasm-encoder` for direct bytecode emission
- Runtime: `wasmtime` for execution

### Alternative Pipeline (MLIR-based, requires `wasm-mlir` feature)
```
DOL AST -> HIR -> MLIR -> LLVM IR -> WASM
```
- Located in: `src/mlir/`
- Uses: `melior` (MLIR Rust bindings)
- Status: Architecture defined, lowering passes partially implemented

### Spirit Compiler
```
DOL Source -> Lexer -> Parser -> AST -> HIR -> [MLIR] -> WASM
```
- Located in: `src/compiler/spirit.rs`
- Currently: Generates placeholder WASM, full pipeline in progress

---

## WASM Features Supported

### Types
- `i32`, `i64`, `int` -> WASM `i64`
- `f32`, `f64`, `float` -> WASM `f64`
- `bool` -> WASM `i32`

### Expressions
- Integer/Float/Boolean literals
- Binary operations: +, -, *, /, %, ==, !=, <, <=, >, >=, &&, ||
- Function parameters (local.get)
- Direct function calls

### Not Yet Supported
- Complex types (structs, enums, tuples, generics)
- Local variables (let bindings)
- Control flow (if, loops, match)
- Closures, unary operations, string/char literals

---

## Key Observations

1. **Mature Lexer/Parser**: Using `logos` for fast tokenization, hand-written recursive descent parser
2. **HIR Layer**: Full HIR implementation for semantic analysis before lowering
3. **Dual WASM Paths**: Direct emission (simple) and MLIR-based (advanced, requires LLVM)
4. **Self-Hosting Effort**: `dol/` directory contains DOL sources for bootstrapping the compiler
5. **Extensive Testing**: Multiple test categories including exhaustive, stress, integration, e2e
6. **Rich Standard Library**: Functor, Applicative, Monad traits defined in DOL
7. **Bio-inspired Patterns**: Hyphal network topology, swarm coordination modules

---

## Next Investigation Steps

1. **WASM Compiler Deep Dive**: Examine `src/wasm/compiler.rs` limitations
2. **HIR Validation**: Check `src/hir/validate.rs` for type checking before WASM
3. **MLIR Lowering**: Investigate `src/mlir/lowering.rs` completion status
4. **Spirit Compiler**: Determine what's blocking full HIR->MLIR->WASM pipeline
5. **Test Coverage**: Run `cargo test --features wasm` to assess current state
