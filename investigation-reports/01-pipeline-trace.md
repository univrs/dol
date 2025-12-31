# DOL → WASM Pipeline Trace Report

**Date:** 2025-12-30
**Investigation:** Claude Flow Swarm - DOL WASM Pipeline Analysis
**Status:** COMPLETE

---

## Executive Summary

The DOL project implements **two distinct compilation pipelines** to WASM:

| Pipeline | Status | Output |
|----------|--------|--------|
| **Direct WASM** | WORKING | Valid WASM for simple functions |
| **Spirit (Full)** | PARTIAL | Placeholder WASM (empty module) |

---

## Pipeline 1: Direct WASM Emission

```
┌─────────────┐    ┌─────────────┐    ┌─────────────────┐    ┌─────────────┐
│ DOL Source  │───▶│   Parser    │───▶│  AST (Module)   │───▶│ WasmCompiler│
└─────────────┘    └─────────────┘    └─────────────────┘    └──────┬──────┘
                                                                    │
                   ┌─────────────┐    ┌─────────────────┐           │
                   │ .wasm file  │◀───│  wasm-encoder   │◀──────────┘
                   └─────────────┘    └─────────────────┘
```

### Stage Analysis

| Stage | File | Function | Status |
|-------|------|----------|--------|
| Lexer | `src/lexer.rs` | `Lexer::tokenize()` | COMPLETE |
| Parser | `src/parser.rs` | `Parser::parse_module()` | COMPLETE |
| WASM Gen | `src/wasm/compiler.rs` | `WasmCompiler::compile()` | PARTIAL |
| Runtime | `src/wasm/runtime.rs` | `WasmRuntime::load()` | COMPLETE |

### Entry Point

```rust
// src/wasm/compiler.rs:42
pub fn compile(&self, module: &Declaration) -> Result<Vec<u8>, WasmError>
```

### Supported Features

- Function declarations with typed parameters
- Integer literals (i64), Float literals (f64), Boolean literals (i32)
- Binary operations: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`
- Function calls
- Return statements
- Parameter references

### Unsupported Features

- Genes, Traits, Systems, Constraints
- Local variables (let bindings)
- Control flow (if/else, match, loops)
- Complex types (structs, enums, tuples)
- Closures/lambdas
- String/char literals

---

## Pipeline 2: Spirit Compiler (Full Pipeline)

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ DOL Source  │───▶│   Lexer     │───▶│   Parser    │───▶│     AST     │
└─────────────┘    └─────────────┘    └─────────────┘    └──────┬──────┘
                                                                │
┌─────────────┐    ┌─────────────┐    ┌─────────────┐           │
│    WASM     │◀╌╌╌│    MLIR     │◀───│     HIR     │◀──────────┘
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                  │
       │           NOT CONNECTED
       │                  │
       └──────────────────┘
```

### Stage Analysis

| Stage | File | Function | Status |
|-------|------|----------|--------|
| Lexer | `src/lexer.rs` | `Lexer::tokenize()` | COMPLETE |
| Parser | `src/parser.rs` | `Parser::parse_module()` | COMPLETE |
| AST→HIR | `src/lower/mod.rs` | `lower_module()` | COMPLETE |
| HIR→MLIR | `src/mlir/lowering.rs` | `HirToMlirLowering::lower_module()` | PARTIAL |
| MLIR→WASM | N/A | **NOT IMPLEMENTED** | MISSING |

### Entry Points

```rust
// src/compiler/spirit.rs:45
pub fn compile_file(path: &Path) -> Result<CompiledSpirit, CompilerError>

// src/compiler/spirit.rs:72
pub fn compile_source(source: &str, filename: &str) -> Result<CompiledSpirit, CompilerError>
```

### Current WASM Output

```rust
// src/compiler/spirit.rs:142
fn generate_placeholder_wasm(_hir_module: &HirModule) -> Result<Vec<u8>, CompilerError> {
    let mut wasm = Vec::new();
    wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // Magic: \0asm
    wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Version: 1
    Ok(wasm)
}
```

**Result:** Valid but EMPTY WASM module (8 bytes, no code sections)

---

## Pipeline 3: MLIR Path (Incomplete)

### Available Infrastructure

| Component | File | Status |
|-----------|------|--------|
| MLIR Context | `src/mlir/context.rs` | COMPLETE |
| Dialect Registration | `src/mlir/context.rs` | COMPLETE |
| Type Lowering | `src/mlir/types.rs` | PARTIAL |
| Op Builders | `src/mlir/ops.rs` | COMPLETE |
| HIR→MLIR | `src/mlir/lowering.rs` | PARTIAL |
| MLIR→LLVM | N/A | MISSING |
| LLVM→WASM | N/A | MISSING |

### Dialects Used

| Dialect | Purpose | Operations |
|---------|---------|------------|
| `arith` | Arithmetic | addi, subi, muli, divsi, constant |
| `func` | Functions | func, call, return |
| `scf` | Control flow | if, for, while |

### Missing Connection

The MLIR infrastructure exists but **cannot produce WASM output**:

1. No MLIR pass manager configured
2. No LLVM dialect lowering
3. No WASM target lowering
4. No external toolchain integration (llc, wat2wasm)

---

## Data Flow Summary

### DOL Types → WASM Types

| DOL Type | MLIR Type | WASM Type | Status |
|----------|-----------|-----------|--------|
| Int32 | i32 | i32 | WORKING |
| Int64 | i64 | i64 | WORKING |
| Float32 | f32 | f32 | WORKING |
| Float64 | f64 | f64 | WORKING |
| Bool | i1 | i32 | WORKING |
| String | index | N/A | NOT IMPLEMENTED |
| Void | tuple() | void | PARTIAL |

### Declaration Types → WASM

| Declaration | Direct WASM | Spirit Pipeline |
|-------------|-------------|-----------------|
| Function | SUPPORTED | Placeholder |
| Gene | NOT SUPPORTED | Placeholder |
| Trait | NOT SUPPORTED | Placeholder |
| System | NOT SUPPORTED | Placeholder |
| Constraint | NOT SUPPORTED | Placeholder |
| Evolution | NOT SUPPORTED | Placeholder |

---

## Critical Code Paths

### Working Path: Simple Function Compilation

```
src/parser.rs:parse_module()     Line 156
       ↓
src/wasm/compiler.rs:compile()   Line 42
       ↓
src/wasm/compiler.rs:extract_functions()  Line 89
       ↓
src/wasm/compiler.rs:emit_function_body() Line 234
       ↓
src/wasm/compiler.rs:emit_expression()    Line 312
       ↓
wasm_encoder::Module::finish()   (External crate)
```

### Non-Working Path: Full Pipeline

```
src/compiler/spirit.rs:compile_source()   Line 72
       ↓
src/parser.rs:parse_module()              Line 156
       ↓
src/lower/mod.rs:lower_module()           Line 45
       ↓
src/compiler/spirit.rs:generate_placeholder_wasm()  Line 142
       ↓
Returns empty 8-byte WASM (DEAD END)
```

---

## Evidence: Existing WASM Output

### File: `/home/ardeshir/repos/univrs-dol/add.wasm`

**Size:** 42 bytes

**Hex Analysis:**
```
00000000: 0061 736d 0100 0000 0107 0160 027e 7e01  .asm.......`.~~.
00000010: 7e03 0201 0007 0701 0361 6464 0000 0a0a  ~........add....
00000020: 0108 0020 0020 017c 0f0b                 ... . .|..
```

**Structure:**
- Magic: `\0asm` (valid)
- Version: 1 (valid)
- Type section: Function type with 2 i64 params, 1 i64 result
- Function section: 1 function of type 0
- Export section: Exports "add" as function 0
- Code section: `local.get 0`, `local.get 1`, `i64.add`, `return`

**Verdict:** This is a **valid, working** WASM module produced by the Direct WASM compiler.

---

## Conclusion

| Question | Answer |
|----------|--------|
| Does Pipeline 1 (Direct) work? | **YES** - for simple functions |
| Does Pipeline 2 (Spirit) work? | **NO** - returns placeholder |
| Does Pipeline 3 (MLIR) work? | **NO** - not connected to WASM |
| Can DOL produce .wasm today? | **PARTIAL** - basic functions only |

---

*Generated by Claude Flow Swarm - Synthesizer Agent*
