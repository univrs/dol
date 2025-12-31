# SCOUT Agent Report: DOL Compilation Entry Points

## Executive Summary

The DOL project has **two distinct WASM compilation paths**:

1. **Direct WASM Emission** (`src/wasm/compiler.rs`) - Uses `wasm-encoder` crate
2. **Spirit Compiler Pipeline** (`src/compiler/spirit.rs`) - Full multi-stage pipeline (currently placeholder)

Both require the `wasm` feature flag to be enabled.

---

## Compilation Entry Points Found

### 1. Direct WASM Compiler (`src/wasm/compiler.rs`)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `WasmCompiler::compile` | `pub fn compile(&self, module: &Declaration) -> Result<Vec<u8>, WasmError>` | **Main entry point** - Compiles DOL AST to WASM bytecode |
| `WasmCompiler::compile_to_file` | `pub fn compile_to_file(&self, module: &Declaration, output_path: impl AsRef<Path>) -> Result<(), WasmError>` | Compile and write to file |

**Pipeline:**
```
DOL AST (Declaration) --> WasmCompiler::compile() --> Vec<u8> (WASM bytecode)
```

**Status:** Functional for basic function declarations with:
- Integer/float/bool types (i32, i64, f32, f64)
- Binary operations (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||)
- Function parameters and return statements
- Literal values

**Limitations:**
- Only supports `Declaration::Function` - genes/traits/systems not supported
- No let bindings, assignments, loops, or control flow
- No string/char literals
- No complex expressions (lambda, match, if, blocks)

### 2. Spirit Compiler (`src/compiler/spirit.rs`)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `compile_file` | `pub fn compile_file(path: &Path) -> Result<CompiledSpirit, CompilerError>` | Compile a `.dol` file |
| `compile_source` | `pub fn compile_source(source: &str, filename: &str) -> Result<CompiledSpirit, CompilerError>` | Compile DOL source text |
| `compile_spirit_project` | `pub fn compile_spirit_project(project_dir: &Path) -> Result<CompiledSpirit, CompilerError>` | Compile a project directory |

**Full Pipeline (Planned):**
```
DOL Source --> Lexer --> Parser --> AST --> HIR --> MLIR --> WASM
```

**Current Status:**
- Phases 1-2 (Parse, HIR lowering) are implemented
- Phases 3-4 (MLIR, WASM emission) return **placeholder WASM** (just magic number + version)
- Returns a valid but empty WASM module

### 3. MLIR Codegen (`src/mlir/codegen.rs`)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `MlirCodegen::compile` | `pub fn compile(&mut self, decl: &Declaration) -> CodegenResult<MlirModule<'ctx>>` | Compile to MLIR module |
| `MlirCodegen::compile_declaration` | `pub fn compile_declaration(&mut self, decl: &Declaration) -> CodegenResult<()>` | Compile a declaration |
| `MlirCodegen::compile_function` | `pub fn compile_function(&mut self, func: &FunctionDecl) -> CodegenResult<()>` | Compile a function |

**Requires:** `mlir` feature flag (depends on `melior` crate / LLVM 18)

### 4. Rust/TypeScript Codegen (`src/codegen/mod.rs`)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `compile_to_rust_via_hir` | `pub fn compile_to_rust_via_hir(source: &str) -> Result<String, ParseError>` | Compile to Rust via HIR |
| `compile_with_diagnostics` | `pub fn compile_with_diagnostics(source: &str) -> Result<(String, Vec<Diagnostic>), ParseError>` | Compile with warnings |

---

## CLI Structure

All CLI binaries use **clap** for argument parsing.

| Binary | File | Purpose | WASM-related? |
|--------|------|---------|---------------|
| `dol-parse` | `src/bin/dol-parse.rs` | Parse DOL files, output JSON AST | No |
| `dol-test` | `src/bin/dol-test.rs` | Test runner for .dol.test files | No |
| `dol-check` | `src/bin/dol-check.rs` | Validation and linting | No |
| `dol-codegen` | `src/bin/dol-codegen.rs` | Generate Rust/TS/JSON Schema | No (targets Rust/TS/JSON) |
| `dol-build-crate` | `src/bin/dol-build-crate.rs` | Generate Rust crate from DOL | No |
| `dol-migrate` | `src/bin/dol-migrate.rs` | Migrate DOL syntax versions | No |
| `dol-mcp` | `src/bin/dol-mcp.rs` | Model Context Protocol server | Has `compile_wasm` stub |

**Finding:** There is **NO dedicated CLI binary for WASM compilation**. The `dol-mcp` binary has a `compile_wasm` tool but it returns "not yet implemented".

---

## Path from .dol Input to .wasm Output

### Current Working Path (Direct Emission)

```rust
// 1. Parse DOL source to AST
let source = std::fs::read_to_string("input.dol")?;
let module = metadol::parse_file(&source)?;

// 2. Create compiler and compile
let compiler = metadol::wasm::WasmCompiler::new()
    .with_optimization(true);
let wasm_bytes = compiler.compile(&module)?;

// 3. Write output
std::fs::write("output.wasm", &wasm_bytes)?;
```

**Key Function:** `WasmCompiler::compile(&self, module: &Declaration) -> Result<Vec<u8>, WasmError>`

### Planned Full Pipeline Path (Spirit Compiler)

```rust
// Single function call
let compiled = metadol::compiler::spirit::compile_source(source, "input.dol")?;
std::fs::write("output.wasm", &compiled.wasm)?;
```

**Current Status:** Returns placeholder WASM (empty module with just magic number)

---

## WASM-Related Code Locations

| File | Lines | Content |
|------|-------|---------|
| `src/wasm/mod.rs` | 1-155 | WASM module, `WasmError` type, exports |
| `src/wasm/compiler.rs` | 1-772 | `WasmCompiler` with direct WASM emission |
| `src/wasm/runtime.rs` | 1-266 | `WasmRuntime`, `WasmModule` for execution via Wasmtime |
| `src/compiler/mod.rs` | 1-80 | Compiler module documentation |
| `src/compiler/spirit.rs` | 1-598 | Full pipeline compiler (placeholder WASM) |
| `src/mcp/server.rs` | 141-143 | `tool_compile_wasm` stub (returns error) |
| `src/mcp/mod.rs` | 59 | `CompileWasm` tool enum variant |
| `src/mlir/mod.rs` | 5,20 | MLIR target mentions WASM |
| `src/lib.rs` | 97-103, 161-166 | WASM module exports (behind feature flag) |

---

## Feature Flags

| Feature | Dependencies | Purpose |
|---------|--------------|---------|
| `wasm` | `wasmtime`, `wasm-encoder` | Enable WASM compilation and runtime |
| `wasm-mlir` | `wasm` + `mlir` | Enable MLIR-based WASM compilation |
| `mlir` | `melior` (LLVM 18) | Enable MLIR code generation |

To enable WASM:
```toml
[dependencies]
metadol = { version = "0.4.0", features = ["wasm"] }
```

---

## Key Findings

1. **No CLI for WASM compilation** - Must use library API directly
2. **Two compilation paths** - Direct emission works, Spirit pipeline is placeholder
3. **Limited AST support** - Only function declarations compile to WASM
4. **Feature-gated** - Requires `wasm` feature in Cargo.toml
5. **Runtime included** - Wasmtime-based runtime for executing compiled WASM

---

## Recommended Next Steps

1. **Add CLI binary** - Create `dol-wasm` or add `--target wasm` to `dol-codegen`
2. **Complete Spirit pipeline** - Implement MLIR-to-WASM lowering
3. **Expand WASM support** - Add gene/trait compilation support
4. **Implement MCP tool** - Make `compile_wasm` functional in dol-mcp

---

*Generated by SCOUT Agent - 2025-12-30*
