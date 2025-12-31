# Backend WASM Analysis Report

**Date**: 2025-12-30
**Investigator**: Backend Analyst Agent
**Repository**: univrs-dol

---

## Executive Summary

DOL has a **functional but partial** WASM compilation pipeline. The project provides **two distinct paths** to WASM:

1. **Direct WASM Emission** (via `wasm-encoder`) - **WORKING** for simple functions
2. **MLIR Pipeline** (DOL -> HIR -> MLIR -> LLVM -> WASM) - **PARTIAL**, MLIR->WASM bridge missing

---

## WASM Dependencies

### Found in `/home/ardeshir/repos/univrs-dol/Cargo.toml`

| Dependency | Version | Purpose | Status |
|------------|---------|---------|--------|
| `wasm-encoder` | 0.41 | Direct WASM bytecode emission | Optional (`wasm` feature) |
| `wasmtime` | 21 | WASM runtime execution | Optional (`wasm` feature) |
| `melior` | 0.18 | MLIR bindings (for LLVM path) | Optional (`mlir` feature) |

### Feature Flags

```toml
[features]
wasm = ["wasmtime", "wasm-encoder"]
mlir = ["melior"]
wasm-mlir = ["wasm", "mlir"]
```

**Assessment**: Dependencies are correctly declared. `wasm` feature works independently of MLIR.

---

## WASM Emission Code Locations

### 1. Direct WASM Compiler (PRIMARY PATH)

**File**: `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

**Status**: **WORKING**

**Implementation Details**:
- Uses `wasm_encoder` to build WASM modules directly
- Implements complete WASM section generation:
  - TypeSection (function signatures)
  - FunctionSection (function indices)
  - ExportSection (exported functions)
  - CodeSection (function bodies)

**Key Functions**:
```rust
pub fn compile(&self, module: &Declaration) -> Result<Vec<u8>, WasmError>
fn extract_functions(&self, module: &Declaration) -> Result<Vec<&FunctionDecl>, WasmError>
fn dol_type_to_wasm(&self, type_expr: &TypeExpr) -> Result<ValType, WasmError>
fn emit_function_body(&self, function: &mut Function, func_decl: &FunctionDecl) -> Result<(), WasmError>
fn emit_statement(&self, function: &mut Function, stmt: &Stmt, func_decl: &FunctionDecl) -> Result<(), WasmError>
fn emit_expression(&self, function: &mut Function, expr: &Expr, func_decl: &FunctionDecl) -> Result<(), WasmError>
fn emit_binary_op(&self, function: &mut Function, op: BinaryOp) -> Result<(), WasmError>
```

**Supported Features**:
- Function declarations with typed parameters
- Integer literals (`i64`)
- Float literals (`f64`)
- Boolean literals (`i32`)
- Binary operations: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`
- Function calls
- Return statements
- Parameter references

**Limitations** (explicitly documented):
- No complex types (structs, enums, tuples)
- No local variables (let bindings)
- No control flow (if, loops, match)
- No closures or higher-order functions
- No unary operations

### 2. Spirit Compiler (HIGH-LEVEL PIPELINE)

**File**: `/home/ardeshir/repos/univrs-dol/src/compiler/spirit.rs`

**Status**: **PARTIAL** - Returns placeholder WASM

**Pipeline**:
```
DOL Source -> Lexer -> Parser -> AST -> HIR -> [MLIR] -> [WASM]
                                        |
                                 IMPLEMENTED
                                        v
                              generate_placeholder_wasm()
```

**Current Implementation**:
```rust
fn generate_placeholder_wasm(_hir_module: &HirModule) -> Result<Vec<u8>, CompilerError> {
    let mut wasm = Vec::new();
    wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // Magic
    wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Version
    Ok(wasm)
}
```

**Issue**: Returns valid but **empty** WASM module (no actual code generation from HIR).

### 3. MLIR Lowering (PARTIAL)

**Files**:
- `/home/ardeshir/repos/univrs-dol/src/mlir/mod.rs`
- `/home/ardeshir/repos/univrs-dol/src/mlir/lowering.rs`
- `/home/ardeshir/repos/univrs-dol/src/mlir/ops.rs`
- `/home/ardeshir/repos/univrs-dol/src/mlir/types.rs`
- `/home/ardeshir/repos/univrs-dol/src/mlir/context.rs`

**Status**: **HIR -> MLIR works, MLIR -> WASM not connected**

**Implemented**:
- `HirToMlirLowering` struct with full expression/statement lowering
- Type mapping (DOL types -> MLIR types)
- Binary/unary operations
- Function calls
- If expressions (using `scf.if`)
- Block expressions
- Literals (bool, int)

**Not Implemented**:
- MLIR -> LLVM lowering
- LLVM -> WASM emission
- No `llc` or `wat2wasm` integration found

### 4. WASM Runtime

**File**: `/home/ardeshir/repos/univrs-dol/src/wasm/runtime.rs`

**Status**: **WORKING**

**Implementation**:
```rust
pub struct WasmRuntime { engine: Engine }
pub struct WasmModule { instance: Instance, store: Store<()> }

impl WasmRuntime {
    pub fn load(&self, wasm_bytes: &[u8]) -> Result<WasmModule, WasmError>
    pub fn load_file(&self, path: impl AsRef<Path>) -> Result<WasmModule, WasmError>
}

impl WasmModule {
    pub fn call(&mut self, name: &str, args: &[Val]) -> Result<Vec<Val>, WasmError>
}
```

---

## Existing .wasm Files

### Found: `/home/ardeshir/repos/univrs-dol/add.wasm`

**Size**: 42 bytes

**Hex Dump**:
```
00000000: 0061 736d 0100 0000 0107 0160 027e 7e01  .asm.......`.~~.
00000010: 7e03 0201 0007 0701 0361 6464 0000 0a0a  ~........add....
00000020: 0108 0020 0020 017c 0f0b                 ... . .|..
```

**Analysis**:
- Magic number: `0x00 0x61 0x73 0x6d` ("\0asm") - VALID
- Version: `0x01 0x00 0x00 0x00` (version 1) - VALID
- Type section: `0x01` - Function type with 2 i64 params, 1 i64 result
- Function section: `0x03` - 1 function of type 0
- Export section: `0x07` - Exports "add" as function 0
- Code section: `0x0a` - Function body with `local.get`, `i64.add`, `return`

**Conclusion**: This is a **valid, working** WASM module generated by the direct compiler.

---

## Integration Tests

### Test Files Found

| File | Purpose | Status |
|------|---------|--------|
| `/home/ardeshir/repos/univrs-dol/tests/wasm_execution.rs` | Runtime tests | Working (feature-gated) |
| `/home/ardeshir/repos/univrs-dol/tests/compiler_integration.rs` | Spirit compiler | Tests pass, but verify placeholder WASM |
| `/home/ardeshir/repos/univrs-dol/tests/compiler_e2e.rs` | End-to-end | Parse/HIR tests work, WASM tests expect NotImplemented |

### Key Test Coverage

**wasm_execution.rs**:
- `test_wasm_magic_number` - Validates header
- `test_wasm_version` - Validates version
- `test_wasm_runtime_new` - Creates runtime successfully
- `test_wasm_load_minimal_module` - Loads valid WASM
- `test_wasm_module_with_function` - Hand-coded WASM with function (works)
- `test_wasm_call_nonexistent_function` - Error handling

**compiler.rs (unit tests)**:
- `test_compile_simple_function` - Compiles `add(a, b) -> a + b` (PASSES)
- `test_compile_function_with_literals` - Compiles constant returns (PASSES)
- `test_compile_non_function_declaration_fails` - Error on Gene (PASSES)

**Ignored Tests** (awaiting full implementation):
- `test_compile_and_execute_simple_function`
- `test_compile_and_execute_gene_method`
- `test_compile_with_control_flow`
- `test_compile_with_pattern_matching`

---

## Alternative WASM Paths

### Searched For

| Path | Found | Notes |
|------|-------|-------|
| MLIR -> LLVM IR -> WASM | NO | MLIR lowering exists, but no LLVM emission |
| WAT text emission | NO | No `.wat` files or `wat2wasm` calls |
| `llc --target=wasm32` | NO | No LLVM CLI integration |
| External toolchain | NO | No calls to external tools |

### Documentation Claims

From `/home/ardeshir/repos/univrs-dol/docs/wasm-compiler-implementation.md`:

> The implementation uses **direct WASM emission** via the `wasm-encoder` crate, bypassing the MLIR -> LLVM -> WASM pipeline for a simpler, more self-contained approach.

This is accurate for the current working implementation.

---

## Summary Table

| Component | File | Status | Notes |
|-----------|------|--------|-------|
| WASM Dependencies | Cargo.toml | Complete | wasm-encoder, wasmtime |
| Direct Compiler | src/wasm/compiler.rs | **WORKING** | Simple functions only |
| WASM Runtime | src/wasm/runtime.rs | **WORKING** | Load & execute |
| Spirit Compiler | src/compiler/spirit.rs | PARTIAL | Returns placeholder |
| HIR -> MLIR | src/mlir/lowering.rs | PARTIAL | Lowering works |
| MLIR -> WASM | N/A | **MISSING** | Not implemented |
| .wasm output | add.wasm | VALID | 42 bytes, works |
| Integration tests | tests/*.rs | PARTIAL | Some #[ignore] |

---

## CRITICAL VERDICT

### Does DOL produce valid WASM today?

## **PARTIAL** (with caveats)

### What Works

1. **Direct WASM Compilation** via `WasmCompiler`:
   - Simple functions with basic arithmetic
   - Generates valid, executable WASM bytes
   - Runtime execution via Wasmtime works

2. **Generated WASM** (`add.wasm`):
   - Valid WASM binary
   - Correct magic number and version
   - Exports working function

### What Doesn't Work

1. **Spirit Compiler Pipeline**:
   - Returns placeholder (empty) WASM
   - HIR -> MLIR exists but MLIR -> WASM not connected

2. **Advanced Language Features**:
   - Control flow (if/else, loops)
   - Local variables
   - Complex types (structs, enums)
   - Pattern matching

3. **Gene/Trait Declarations**:
   - Cannot compile to WASM (function declarations only)

---

## Status Assessment

| Capability | Status |
|------------|--------|
| Basic function -> WASM | Functional |
| Runtime execution | Functional |
| Full language -> WASM | Partial |
| Production-ready | Not Ready |
| Test coverage | Partial |

### Legend

- **Functional**: Works as intended
- **Partial**: Some functionality present, gaps remain
- **Not Ready**: Requires significant work

---

## Recommendations

1. **Connect HIR -> MLIR -> WASM pipeline** in Spirit compiler
2. **Add control flow support** to direct compiler
3. **Implement local variables** for let bindings
4. **Remove #[ignore] from tests** as features are implemented
5. **Add CI verification** of `add.wasm` output

---

## Files Analyzed

- `/home/ardeshir/repos/univrs-dol/Cargo.toml`
- `/home/ardeshir/repos/univrs-dol/src/wasm/mod.rs`
- `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`
- `/home/ardeshir/repos/univrs-dol/src/wasm/runtime.rs`
- `/home/ardeshir/repos/univrs-dol/src/compiler/spirit.rs`
- `/home/ardeshir/repos/univrs-dol/src/mlir/mod.rs`
- `/home/ardeshir/repos/univrs-dol/src/mlir/lowering.rs`
- `/home/ardeshir/repos/univrs-dol/tests/wasm_execution.rs`
- `/home/ardeshir/repos/univrs-dol/tests/compiler_integration.rs`
- `/home/ardeshir/repos/univrs-dol/tests/compiler_e2e.rs`
- `/home/ardeshir/repos/univrs-dol/docs/wasm-compiler-implementation.md`
- `/home/ardeshir/repos/univrs-dol/add.wasm`
