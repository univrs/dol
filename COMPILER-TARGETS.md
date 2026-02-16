# DOL Compiler Targets

Side-by-side feature matrix for every DOL compilation backend.

## Backends at a Glance

| | Native (LLVM) | WASM | Rust Codegen |
|---|---|---|---|
| **Location** | `llvm-backend/` | `crates/dol-codegen-wasm/` | `crates/dol-codegen-rust/` |
| **Output** | Machine code (ELF/Mach-O/COFF) | `.wasm` module | Rust source with Automerge |
| **Runtime** | VUDO native runtime | VUDO WASM host | Automerge CRDT |
| **Status** | Beta | Phase 2 (imports only) | Production |
| **Tests** | 15 | 32 | 10 |

## Language Features

| Feature | Native | WASM | Rust |
|---------|--------|------|------|
| Functions (params, return types) | ✅ | - | ✅ |
| Recursive functions | ✅ | - | ✅ |
| Integer arithmetic (`+` `-` `*` `/` `%`) | ✅ | - | ✅ |
| Float arithmetic | ✅ | - | ✅ |
| Comparison operators (`==` `!=` `<` `>` `<=` `>=`) | ✅ | - | ✅ |
| Boolean logic (`and` `or` `not`) | ✅ | - | ✅ |
| Unary operators (`-` `not`) | ✅ | - | ✅ |
| If/else conditionals | ✅ | - | ✅ |
| While/for loops | ❌ | - | ✅ |
| Match expressions | ❌ | - | ✅ |
| Local variables (`let`) | ✅ | - | ✅ |
| Mutable variables (`var`) | ❌ | - | ✅ |
| Compound assignment (`+=` `-=`) | ❌ | - | ✅ |
| String literals | ✅ | - | ✅ |
| String concatenation | ✅ | - | ✅ |
| String + integer concat | ✅ | - | ✅ |
| `println()` / `print()` | ✅ | - | ✅ |
| Gen (struct) declarations | ✅ | - | ✅ |
| Enum declarations | ✅ | - | ✅ |
| Trait/rule/system declarations | ✅ compile-time | - | ✅ |
| Field access (`point.x`) | ❌ | - | ✅ |
| Method calls (`obj.method()`) | ❌ | - | ✅ |
| Closures / lambdas | ❌ | - | ✅ |
| Path syntax (`module::item`) | ❌ | - | ✅ |
| Reference types (`&T`) | ❌ | - | ✅ |
| Multiple functions per file | ✅ | - | ✅ |
| `main()` entry point | ✅ | - | N/A |

**Legend:** ✅ working | ❌ not yet implemented | `-` backend not at this phase yet | N/A not applicable

## Type System

| Type | Native | WASM | Rust |
|------|--------|------|------|
| `i8` `i16` `i32` `i64` `i128` | ✅ | - | ✅ |
| `u8` `u16` `u32` `u64` `u128` | ✅ | - | ✅ |
| `f32` `f64` | ✅ | - | ✅ |
| `bool` | ✅ | - | ✅ |
| `string` | ✅ fat `{ptr, i64}` | - | ✅ `String` |
| `()` (void) | ✅ | - | ✅ |
| Custom structs (gen) | ✅ LLVM struct | - | ✅ Rust struct |
| Enums | ✅ tagged `{i32, i64}` | - | ✅ Rust enum |
| `Option<T>` | ❌ | - | ✅ |
| `Vec<T>` / `List<T>` | ❌ needs stdlib | - | ✅ |
| `Map<K,V>` / `Set<T>` | ❌ needs stdlib | - | ✅ |
| Generics / type params | ❌ | - | ✅ |
| Tuples | ❌ | - | ✅ |

## Runtime / Host Functions

The VUDO ABI defines 24 host functions shared across Native and WASM backends.

| Category | Functions | Native | WASM |
|----------|-----------|--------|------|
| **I/O** | `print` `println` `log` `error` | ✅ | ✅ declared |
| **Memory** | `alloc` `free` `realloc` | ✅ | ✅ declared |
| **Time** | `now` `sleep` `monotonic_now` | ✅ | ✅ declared |
| **Strings** | `string_concat` `i64_to_string` | ✅ | ✅ declared |
| **Messaging** | `send` `recv` `pending` `broadcast` `free_message` | ✅ | ✅ declared |
| **Effects** | `emit_effect` `subscribe` | ✅ | ✅ declared |
| **Random** | `random` `random_bytes` | ✅ | ✅ declared |
| **Debug** | `breakpoint` `assert` `panic` | ✅ | ✅ declared |

The Rust codegen backend does not use VUDO host functions. It generates Automerge-backed structs with CRDT merge semantics.

## Rust Codegen: CRDT Strategies

The Rust backend supports 7 CRDT annotation strategies for local-first collaborative apps:

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
| `x86_64-unknown-linux-gnu` | x86-64 Linux | ✅ Default |
| `aarch64-unknown-linux-gnu` | ARM64 Linux | ✅ Cross-compile |
| `aarch64-apple-darwin` | Apple Silicon Mac | ✅ Cross-compile |
| `riscv64gc-unknown-linux-gnu` | RISC-V 64-bit | ✅ Cross-compile |
| `x86_64-pc-windows-msvc` | x86-64 Windows | ✅ Cross-compile |

## WASM: Implementation Phases

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 1 | ABI types, host function signatures | ✅ Complete |
| Phase 2 | Import emitter, import tracker, test infra | ✅ Complete |
| Phase 3 | Memory layout, function bodies, type conversion | Planned |
| Phase 4 | Optimization, dead code elimination | Planned |

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
   │ HIR → LLVM IR   │    │ HIR → Walrus    │    │ HIR → Rust src  │
   │ → machine code  │    │ → .wasm binary  │    │ → Automerge     │
   └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
            │                      │                      │
            ▼                      ▼                      ▼
   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
   │  cc + libvudo   │    │  WASM runtime   │    │   cargo build   │
   │  → executable   │    │  → browser/node │    │  → Rust binary  │
   └─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Test Counts

| Backend | Unit Tests | Integration Tests | Working Examples |
|---------|-----------|-------------------|-----------------|
| Native (LLVM) | 15 | 5 (in hir_lowering) | 12 `.dol` files |
| WASM | 15 | 17 | Phase 3 |
| Rust Codegen | 10 | 10 | 3 |
| **DOL Core** | **89** | **2,074** (parser, lexer, HIR) | **155+** |

## What's Next

### Native
- Loops (`while`, `for`) and `match` expression codegen
- Field access and method call support
- Standard library (collections, iterators, string utilities)
- Parser extensions: `var`, `+=`, `::`, `&T`

### WASM
- Phase 3: memory layout, function body generation, type conversion
- Full end-to-end `.dol` → `.wasm` compilation
- Browser integration testing

### Rust Codegen
- Schema evolution via `evolution` declarations
- TypeScript definition generation for WASM bindings
- Incremental codegen for changed declarations
