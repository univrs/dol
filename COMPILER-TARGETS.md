# DOL Compiler Targets

Side-by-side feature matrix for the DOL compilation backends, as of crate v0.8.2.

None of these backends is production-ready. For a verified overview see
[docs/DOL-STATUS-2026-11.md](docs/DOL-STATUS-2026-11.md).

This matrix does not cover the code generator in the published crate (`src/codegen/`, used by
`dol-codegen --target rust|typescript|jsonschema`). That generator works on declarations, but its
Rust output compiles standalone for only 14 of 41 example files, and its TypeScript and JSON Schema
output omits typed `has x: T` fields.

## Backends at a Glance

| | Native (LLVM) | WASM | Rust (Automerge) |
|---|---|---|---|
| **Location** | `llvm-backend/` | `src/wasm/` | `crates/dol-codegen-rust/` |
| **Output** | Machine code (ELF/Mach-O/COFF) | `.wasm` module | Rust source with Automerge |
| **Runtime** | VUDO native runtime | VUDO VM (module `vudo`) | Automerge CRDT |
| **Status** | Beta. Separate workspace, not published, not in CI; needs a manual LLVM 18 setup | Beta. In the published crate (`wasm-compile` feature); covers a subset of the language | Experimental. Separate unpublished crate, not built in CI, not wired into any CLI; its generated code is never compiled by its tests |

## Language Features

| Feature | Native | WASM | Rust |
|---------|--------|------|------|
| Functions (params, return types) | ✅ | ✅ | ✅ |
| Recursive functions | ✅ | ✅ | ✅ |
| Integer arithmetic (`+` `-` `*` `/` `%`) | ✅ | ✅ | ✅ |
| Float arithmetic | ✅ | ⚠️ annotated locals only | ✅ |
| Comparison operators (`==` `!=` `<` `>` `<=` `>=`) | ✅ | ✅ | ✅ |
| Boolean logic (`and` `or` `not`) | ✅ | ✅ | ✅ |
| Unary operators (`-` `not`) | ✅ | ⚠️ no float `-` | ✅ |
| If/else conditionals | ✅ | ✅ | ✅ |
| While/for loops | ❌ | ✅ `for` over ranges | ✅ |
| Match expressions | ❌ | ⚠️ int literals + `_` | ✅ |
| Local variables (`let`) | ✅ | ⚠️ not inside blocks | ✅ |
| Mutable variables (`var`) | ❌ | ✅ | ✅ |
| Compound assignment (`+=` `-=`) | ❌ | ❌ | ❌ not parseable |
| String literals | ✅ | ✅ | ✅ |
| String concatenation | ✅ | ❌ miscompiles | ✅ |
| String + integer concat | ✅ | ❌ | ✅ |
| `println()` / `print()` | ✅ | ⚠️ via `sex fun` import | ✅ |
| Gen (struct) declarations | ✅ | ⚠️ linear memory; `u64` and `string` fields rejected | ✅ |
| Enum declarations | ✅ | ✅ | ✅ |
| Trait/rule/system declarations | ✅ compile-time | ignored | ✅ |
| Field access (`point.x`) | ❌ | ✅ | ✅ |
| Method calls (`obj.method()`) | ❌ | ❌ | ✅ |
| Closures / lambdas | ❌ | ❌ | ✅ |
| Path syntax (`module::item`) | ❌ | ❌ | ✅ |
| Reference types (`&T`) | ❌ | ❌ | ❌ not parseable |
| Multiple functions per file | ✅ | ✅ | ✅ |
| `main()` entry point | ✅ | N/A (exports) | N/A |

**Legend:** ✅ working | ⚠️ partial | ❌ not yet implemented | `-` backend not at this phase yet | N/A not applicable

**Rust (Automerge) column:** ✅ means the generator emits code for the construct. That code is not
compiled by any test, so it is unverified.

**Parser limits apply to every backend:** the v0.8.2 parser rejects `+=`/`-=`, `&T` and generic
functions, and discards generic parameters on `gen`/`trait`. No backend can receive these
constructs from DOL source, whatever the rows below say.

WASM partial/❌ entries are reproduced by `known_bugs` tests in `tests/wasm_execution.rs` (run with `-- --ignored`).

## Type System

| Type | Native | WASM | Rust |
|------|--------|------|------|
| `i8` `i16` `i32` `i64` `i128` | ✅ | ⚠️ `i32`/`i64` only, as i64 | ✅ |
| `u8` `u16` `u32` `u64` `u128` | ✅ | ❌ | ✅ |
| `f32` `f64` | ✅ | ⚠️ both as f64 | ✅ |
| `bool` | ✅ | ✅ i32 | ✅ |
| `string` | ✅ fat `{ptr, i64}` | ✅ ptr to len-prefixed | ✅ `String` |
| `()` (void) | ✅ | ✅ | ✅ |
| Custom structs (gen) | ✅ LLVM struct | ✅ linear memory | ✅ Rust struct |
| Enums | ✅ tagged `{i32, i64}` | ✅ i32 tag | ✅ Rust enum |
| `Option<T>` | ❌ | ❌ pointer only | ✅ |
| `Vec<T>` / `List<T>` | ❌ needs stdlib | ❌ pointer only | ✅ |
| `Map<K,V>` / `Set<T>` | ❌ needs stdlib | ❌ pointer only | ✅ |
| Generics / type params | ❌ | ❌ | ❌ discarded by the parser |
| Tuples | ❌ | ❌ | ✅ |

## Runtime / Host Functions

### WASM (v0.8.2)

Since v0.8.2 the WASM compiler targets the VUDO VM host ABI (`src/wasm/host_abi.rs`). A bodiless
`sex fun host_*` declaration compiles to an import from module `vudo` with the VM linker's exact
signature. There are 15 such functions:

| Category | Functions |
|----------|-----------|
| **Time / random** | `host_time_now` `host_random_bytes` |
| **Logging** | `host_log` |
| **Storage** | `host_storage_read` `host_storage_write` `host_storage_delete` |
| **Network** | `host_network_connect` `host_network_listen` `host_network_broadcast` |
| **Credit** | `host_credit_balance` `host_credit_transfer` `host_credit_reserve` `host_credit_release` `host_credit_consume` `host_credit_available` |

`dol-abi::vm_host_functions()` lists the same 15 functions. Declaring a `host_*` function the VM does
not provide, or with the wrong arity or parameter types, is a compile error.

### Native

The native runtime (`llvm-backend/crates/vudo-runtime-native`) implements an older set of `vudo_*`
functions:

| Category | Functions |
|----------|-----------|
| **I/O** | `print` `println` `log` `error` |
| **Memory** | `alloc` `free` `realloc` |
| **Time** | `now` `sleep` `monotonic_now` |
| **Strings** | `string_concat` `i64_to_string` |
| **Messaging** | `send` `recv` `pending` `broadcast` `free_message` |
| **Effects** | `emit_effect` `subscribe` |
| **Random** | `random` `random_bytes` |
| **Debug** | `breakpoint` `assert` `panic` |

The WASM compiler does not use these names. `dol-abi::standard_host_functions()` (the 22 `vudo_*`
names) is deprecated in favour of `vm_host_functions()`.

The Rust codegen backend does not use VUDO host functions. It generates Automerge-backed structs with CRDT merge semantics.

## Rust (Automerge) Codegen: CRDT Strategies

The parser and validator in the published crate accept 7 `@crdt(...)` strategies. The experimental
Automerge generator in `crates/dol-codegen-rust` maps them as follows:

| Annotation | Automerge Type | Use Case |
|------------|----------------|----------|
| `@crdt(immutable)` | Immutable | Set-once fields (IDs, timestamps) |
| `@crdt(lww)` | Last-Write-Wins | Simple values (names, status) |
| `@crdt(peritext)` | Rich Text | Collaborative text editing |
| `@crdt(or_set)` | OR-Set | Tags, reactions, collections |
| `@crdt(pn_counter)` | PN-Counter | Distributed counters (likes, votes) |
| `@crdt(rga)` | RGA | Ordered lists |
| `@crdt(mv_register)` | Multi-Value Register | Conflict-preserving fields |

## Native: Supported Targets

| Target Triple | Platform | Status |
|---------------|----------|--------|
| `x86_64-unknown-linux-gnu` | x86-64 Linux | ✅ Default; the 12 `examples/native` programs build and run |
| `aarch64-unknown-linux-gnu` | ARM64 Linux | Accepted by `targets.rs`; not tested |
| `aarch64-apple-darwin` | Apple Silicon Mac | Accepted by `targets.rs`; not tested |
| `riscv64gc-unknown-linux-gnu` | RISC-V 64-bit | Accepted by `targets.rs`; not tested |
| `x86_64-pc-windows-msvc` | x86-64 Windows | Accepted by `targets.rs`; not tested |

## WASM: Implementation Phases

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 1 | ABI types, host function signatures | ✅ Complete |
| Phase 2 | Import emitter, import tracker, test infra | ✅ Complete |
| Phase 3 | Memory layout, function bodies, type conversion (`src/wasm/`) | ✅ Function bodies, gen layouts, bump allocator; ⚠️ type fidelity (8 known bugs) |
| Phase 3b | Spirit pipeline: `compile_source` → `WasmCompiler` | ✅ Complete |
| Phase 4a | Host ABI unification (`loa` → `vudo`) | ✅ Done in v0.8.2 |
| Phase 4b | Typed codegen, optimization | Next (tree shaking available, opt-in) |

Phases 1–2 live in `dol-abi` and `crates/dol-codegen-wasm/` (walrus, imports only). Phase 3 is the
`wasm-encoder` compiler in `src/wasm/`, which emits directly from the AST.

## Compilation Pipeline

```
                    ┌──────────────────────────────────────────┐
                    │           DOL Source (.dol)               │
                    └──────────────┬───────────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────────┐
                    │       Parser → AST → HIR Lowering        │
                    │   (4 decl, 12 expr, 6 stmt, 8 type)      │
                    └──────┬──────────┬──────────┬─────────────┘
                           │          │          │
              ┌────────────┘          │          └────────────┐
              ▼                       ▼                       ▼
   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
   │  Native (LLVM)  │    │      WASM       │    │   Rust Codegen  │
   │                 │    │                 │    │                 │
   │ HIR → LLVM IR   │    │ AST → wasm-enc. │    │ HIR → Rust src  │
   │ → machine code  │    │ → .wasm binary  │    │ → Automerge     │
   └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
            │                      │                      │
            ▼                      ▼                      ▼
   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
   │  cc + libvudo   │    │  WASM runtime   │    │   cargo build   │
   │  → executable   │    │  → browser/node │    │  → Rust binary  │
   └─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Tests

- Native (LLVM): 15 tests in `llvm-backend/`, run manually (not in CI).
- WASM: `tests/wasm_execution.rs`, `tests/wasm_host_abi.rs` and unit tests in `src/wasm/`, run in CI.
- Rust (Automerge): tests in `crates/dol-codegen-rust/` check generated text with string matching; not in CI.
- For whole-crate test counts see the README.

## What's Next

### Native
- Loops (`while`, `for`) and `match` expression codegen
- Field access and method call support
- Standard library (collections, iterators, string utilities)
- Parser extensions: `var`, `+=`, `::`, `&T`

### WASM
- Strings and `u64` fields in gen layouts (currently rejected), lambdas, tuples, lists, `sex { }` blocks
- Typed codegen: use the typechecker instead of `infer_expression_type` (defaults to i64), scoped locals
- Fix heap base (hardcoded 1024; overlaps string data > 1 KB)
- Make stubbed collection/Option methods hard errors until runtime support exists
- Browser integration testing

### Rust (Automerge) Codegen
- Wire into a CLI and compile the generated code in tests
- Schema evolution via `evolution` declarations
- TypeScript definition generation for WASM bindings
- Incremental codegen for changed declarations
