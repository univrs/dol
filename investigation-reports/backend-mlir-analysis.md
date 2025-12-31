# Backend MLIR Analysis Report

**Date:** 2025-12-30
**Analyst:** Backend Analyst Agent
**Subject:** DOL to WASM Pipeline - MLIR Code Generation Deep Dive

---

## Executive Summary

The DOL project has **substantial MLIR infrastructure** implemented via the `melior` crate (Rust bindings for MLIR). The MLIR codegen is **partially complete** - core type lowering and expression compilation exist, but several advanced features are stubbed or missing. A separate **direct WASM emission** path exists as an alternative to the full MLIR pipeline.

**Overall Status:** :warning: **Partial** - Foundational infrastructure exists, but not production-ready.

---

## 1. MLIR Dependencies Found

### Cargo.toml Dependencies

| Dependency | Version | Status | Feature Gate |
|------------|---------|--------|--------------|
| **melior** | 0.18 | Optional | `mlir` |
| **wasmtime** | 21 | Optional | `wasm` |
| **wasm-encoder** | 0.41 | Optional | `wasm` |

### Feature Flags

```toml
[features]
mlir = ["melior"]
wasm = ["wasmtime", "wasm-encoder"]
wasm-mlir = ["wasm", "mlir"]  # Combined feature
```

**Note:** The `wasm-mlir` feature is intended for full pipeline: DOL -> HIR -> MLIR -> LLVM -> WASM.

### Missing Dependencies (NOT present)

| Dependency | Purpose | Status |
|------------|---------|--------|
| `inkwell` | LLVM bindings | NOT found |
| `llvm-sys` | Raw LLVM bindings | NOT found |

The project relies on `melior` which internally uses LLVM. No direct LLVM bindings are included.

---

## 2. MLIR Module Structure

**Location:** `/home/ardeshir/repos/univrs-dol/src/mlir/`

```
src/mlir/
  mod.rs      # Module root, error types, re-exports
  codegen.rs  # AST-to-MLIR code generation (original approach)
  context.rs  # MLIR context management, dialect registration
  ops.rs      # MLIR operation builders
  types.rs    # Type lowering (DOL types -> MLIR types)
  lowering.rs # HIR-to-MLIR lowering pass (newer approach)
```

### Module Hierarchy in lib.rs

```rust
// MLIR backend (requires mlir feature)
pub mod mlir;

#[cfg(feature = "mlir")]
pub use mlir::{CodegenError, CodegenResult, MlirCodegen, MlirContext};
pub use mlir::MlirError;  // Always exported
```

---

## 3. Lowering Functions Documented

### 3.1 AST-to-MLIR Codegen (Original Approach)

**File:** `/home/ardeshir/repos/univrs-dol/src/mlir/codegen.rs`

```rust
pub struct MlirCodegen<'ctx> {
    mlir_ctx: &'ctx MlirContext,
    type_lowering: TypeLowering<'ctx>,
    filename: Option<String>,
    variables: HashMap<String, (Value<'ctx, 'ctx>, MlirType<'ctx>)>,
}
```

**Key Functions:**

| Function | Purpose | Status |
|----------|---------|--------|
| `compile(&mut self, decl: &Declaration)` | Main entry point | :white_check_mark: Implemented |
| `compile_declaration()` | Dispatch per declaration type | :warning: Partial (genes/traits skip) |
| `compile_function()` | Lower functions to func.func | :white_check_mark: Implemented |
| `compile_stmt()` | Statement to operations | :warning: Partial (loops TODO) |
| `compile_expr()` | Expression to values | :warning: Partial (lambdas unsupported) |
| `compile_literal()` | Literals to constants | :warning: Strings unsupported |
| `compile_binary()` | Binary ops | :white_check_mark: Core ops work |
| `compile_unary()` | Unary ops | :white_check_mark: Neg/Not work |
| `compile_call()` | Function calls | :warning: Direct only |
| `compile_if()` | If expressions | :warning: Basic (no scf.if) |

### 3.2 HIR-to-MLIR Lowering (Newer Approach)

**File:** `/home/ardeshir/repos/univrs-dol/src/mlir/lowering.rs`

```rust
pub struct HirToMlirLowering<'c> {
    context: &'c Context,
    builder: OpBuilder<'c>,
    symbol_table: SymbolTable,
    variables: HashMap<Symbol, Value<'c, 'c>>,
}
```

**Key Functions:**

| Function | Purpose | Status |
|----------|---------|--------|
| `lower_module()` | Lower entire HIR module | :white_check_mark: Implemented |
| `lower_decl()` | Declaration dispatch | :warning: Only functions work |
| `lower_function()` | Function lowering | :white_check_mark: Implemented |
| `lower_expr()` | Expression lowering | :warning: Partial |
| `lower_literal()` | Literal constants | :warning: No floats/strings |
| `lower_binary()` | Binary operations | :white_check_mark: Implemented |
| `lower_unary()` | Unary operations | :white_check_mark: Implemented |
| `lower_call()` | Function calls | :warning: Direct only |
| `lower_if()` | If expressions with scf.if | :white_check_mark: Implemented |
| `lower_block()` | Block expressions | :white_check_mark: Implemented |
| `lower_stmt()` | Statement lowering | :warning: Partial |
| `lower_type()` | Type lowering | :white_check_mark: Primitives work |

---

## 4. Dialect Definitions

### 4.1 Standard Dialects Used

**File:** `/home/ardeshir/repos/univrs-dol/src/mlir/context.rs`

```rust
use melior::{
    dialect::DialectRegistry,
    utility::register_all_dialects,
};

pub fn new() -> Self {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);  // All standard dialects

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();
    Self { context }
}
```

### 4.2 Dialects Actively Used

**File:** `/home/ardeshir/repos/univrs-dol/src/mlir/ops.rs`

```rust
use melior::dialect::{arith, func, scf};
```

| Dialect | Operations Used | Purpose |
|---------|-----------------|---------|
| `arith` | addi, subi, muli, divsi, remsi, cmpi, cmpf, andi, ori, xori, constant | Arithmetic |
| `func` | func, call, return | Functions |
| `scf` | if, for, while | Control flow |

### 4.3 Custom DOL Dialect

**Status:** :x: **NOT IMPLEMENTED**

No custom DOL dialect has been defined. The project uses standard MLIR dialects only.

---

## 5. Type Lowering

**File:** `/home/ardeshir/repos/univrs-dol/src/mlir/types.rs`

### Type Mapping Table

| DOL Type | MLIR Type | Status |
|----------|-----------|--------|
| Void | empty tuple `()` | :white_check_mark: |
| Bool | `i1` | :white_check_mark: |
| Int8/UInt8 | `i8` | :white_check_mark: |
| Int16/UInt16 | `i16` | :white_check_mark: |
| Int32/UInt32 | `i32` | :white_check_mark: |
| Int64/UInt64 | `i64` | :white_check_mark: |
| Float32 | `f32` | :white_check_mark: |
| Float64 | `f64` | :white_check_mark: |
| String | index (placeholder) | :warning: Needs LLVM dialect |
| Function | `FunctionType` | :white_check_mark: |
| Tuple | MLIR tuple | :white_check_mark: |
| List/Array | Element type only | :warning: Placeholder |
| Option | `(i1, T)` tuple | :white_check_mark: |
| Result | `(i1, T, E)` tuple | :white_check_mark: |
| Quoted | index (placeholder) | :warning: |
| Var/Unknown/Any | ERROR | :white_check_mark: (correct behavior) |

---

## 6. Optimization Passes

### 6.1 AST-Level Passes

**File:** `/home/ardeshir/repos/univrs-dol/src/transform/mod.rs`

| Pass | Purpose | Status |
|------|---------|--------|
| `ConstantFolding` | Fold compile-time constants | :white_check_mark: Implemented |
| `DeadCodeElimination` | Remove unreachable code | :white_check_mark: Implemented |
| `IdiomDesugar` | Desugar idiom brackets | :white_check_mark: Implemented |

### 6.2 MLIR-Level Optimization Passes

**Status:** :x: **NOT IMPLEMENTED**

The MLIR module documentation mentions:

> "Optimization: Applying MLIR's dialect-based optimization passes"

But no MLIR pass manager or optimization pipeline is implemented. The `register_all_dialects` call loads dialects but no passes are run.

---

## 7. Alternative WASM Path

**Location:** `/home/ardeshir/repos/univrs-dol/src/wasm/`

```
src/wasm/
  mod.rs      # Module root, error types
  compiler.rs # Direct DOL AST -> WASM bytecode emission
  runtime.rs  # Wasmtime runtime execution
```

### Direct WASM Compiler

**File:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

This provides a simpler path **bypassing MLIR entirely**:

```rust
pub struct WasmCompiler {
    optimize: bool,
    debug_info: bool,
}

// Direct emission: DOL AST -> WASM bytecode
pub fn compile(&self, module: &Declaration) -> Result<Vec<u8>, WasmError>
```

| Feature | Status |
|---------|--------|
| Function declarations | :white_check_mark: |
| Integer/float literals | :white_check_mark: |
| Binary operations | :white_check_mark: |
| Function calls | :white_check_mark: Basic |
| Return statements | :white_check_mark: |
| Let bindings | :x: Not supported |
| Control flow (if/loops) | :x: Not supported |
| Complex types | :x: Not supported |

---

## 8. HIR Infrastructure

**Location:** `/home/ardeshir/repos/univrs-dol/src/hir/` and `/home/ardeshir/repos/univrs-dol/src/lower/`

### AST -> HIR Lowering

The project has a proper HIR (High-level IR) layer between AST and MLIR:

```
DOL Source -> AST (parse) -> HIR (lower) -> MLIR (mlir::lowering)
```

**Key Functions:**
- `lower::lower_module()` - AST to HIR
- `lower::lower_file()` - Complete file processing
- `mlir::lowering::HirToMlirLowering::lower_module()` - HIR to MLIR

### HIR -> Rust Codegen (Alternative)

**File:** `/home/ardeshir/repos/univrs-dol/src/codegen/hir_rust.rs`

A complete HIR-to-Rust code generator exists as an alternative to MLIR:

```rust
pub struct HirRustCodegen {
    output: String,
    indent: usize,
    symbols: SymbolTable,
}
```

---

## 9. Critical Assessment

### What EXISTS:

| Component | Implementation | Quality |
|-----------|----------------|---------|
| MLIR Context Management | Complete | Good |
| Dialect Registration | Complete | Good |
| Type Lowering | Mostly complete | Good |
| Operation Builders | Complete | Good |
| AST-to-MLIR Codegen | Partial | Needs work |
| HIR-to-MLIR Lowering | Partial | Better approach |
| Direct WASM Emission | Basic | Simple but limited |
| HIR-to-Rust Codegen | Complete | Production-ready |

### What is MISSING:

| Component | Importance | Notes |
|-----------|------------|-------|
| Custom DOL Dialect | Medium | Not strictly required |
| MLIR Optimization Pipeline | High | No passes run |
| String/Array Types | High | Placeholder implementations |
| Lambda/Closure Support | Medium | Error on attempt |
| Pattern Matching in MLIR | Medium | Not implemented |
| LLVM Target Lowering | High | melior handles internally |
| WASM Target from MLIR | High | No pipeline to WASM from MLIR |

### Critical Gap: MLIR -> WASM Pipeline

**The `wasm-mlir` feature flag exists but the actual pipeline is NOT implemented.**

Current options:
1. **Direct WASM emission** (works but limited)
2. **HIR -> Rust codegen** (works fully)
3. **HIR -> MLIR** (infrastructure exists but no WASM target)

---

## 10. Status Summary

| Area | Status | Notes |
|------|--------|-------|
| MLIR Dependencies | :white_check_mark: Complete | melior 0.18 |
| Context/Dialect Setup | :white_check_mark: Complete | All dialects registered |
| Type Lowering | :warning: Partial | Primitives work, strings/arrays placeholders |
| Operation Builders | :white_check_mark: Complete | arith, func, scf |
| AST-to-MLIR Codegen | :warning: Partial | Functions work, declarations skip |
| HIR-to-MLIR Lowering | :warning: Partial | Better structure, still gaps |
| MLIR Optimization | :x: Missing | No pass manager |
| MLIR -> WASM | :x: Missing | No target lowering |
| Direct WASM | :warning: Partial | Basic functions only |
| Custom Dialect | :x: Missing | Not required |

### Overall Assessment: :warning: **PARTIAL**

The MLIR infrastructure is **well-architected** and follows good patterns (melior usage, operation builders, type lowering). However, it is **incomplete for production WASM generation**:

1. **Type system gaps** - Strings, arrays, and complex types need proper lowering
2. **No MLIR passes** - Optimization pipeline not implemented
3. **No WASM target** - The pipeline ends at MLIR, doesn't continue to WASM
4. **Feature stubs** - Lambdas, pattern matching, and loops return errors

The **direct WASM path** via wasm-encoder is simpler and more complete for basic functions, but lacks the optimization potential of the full MLIR pipeline.

---

## 11. Code Locations Reference

### MLIR Module

| File | Path |
|------|------|
| Module root | `/home/ardeshir/repos/univrs-dol/src/mlir/mod.rs` |
| AST Codegen | `/home/ardeshir/repos/univrs-dol/src/mlir/codegen.rs` |
| Context | `/home/ardeshir/repos/univrs-dol/src/mlir/context.rs` |
| Op Builders | `/home/ardeshir/repos/univrs-dol/src/mlir/ops.rs` |
| Type Lowering | `/home/ardeshir/repos/univrs-dol/src/mlir/types.rs` |
| HIR Lowering | `/home/ardeshir/repos/univrs-dol/src/mlir/lowering.rs` |

### WASM Module

| File | Path |
|------|------|
| Module root | `/home/ardeshir/repos/univrs-dol/src/wasm/mod.rs` |
| Compiler | `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` |
| Runtime | `/home/ardeshir/repos/univrs-dol/src/wasm/runtime.rs` |

### Supporting Modules

| File | Path |
|------|------|
| HIR Module | `/home/ardeshir/repos/univrs-dol/src/hir/` |
| Lowering | `/home/ardeshir/repos/univrs-dol/src/lower/` |
| HIR Rust Codegen | `/home/ardeshir/repos/univrs-dol/src/codegen/hir_rust.rs` |
| Transform Passes | `/home/ardeshir/repos/univrs-dol/src/transform/` |

---

## 12. Recommendations

### Short Term

1. **Complete Direct WASM Path** - Add local variables and control flow to `wasm/compiler.rs`
2. **Add Integration Tests** - Test MLIR compilation with feature flags enabled

### Medium Term

1. **Implement String/Array Types** - Use LLVM dialect or memref for proper lowering
2. **Add MLIR Pass Manager** - Run optimization passes before code generation
3. **Complete Lambda Support** - Implement closure lowering in MLIR

### Long Term

1. **MLIR -> WASM Pipeline** - Implement the full pipeline from MLIR to WASM output
2. **Custom DOL Dialect** - Optional but would provide cleaner high-level operations
3. **Incremental Compilation** - Cache MLIR modules for faster recompilation

---

*End of Analysis Report*
