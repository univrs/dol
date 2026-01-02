# Spirit Compiler

> End-to-end compilation from DOL source to WebAssembly bytecode

## Overview

The Spirit compiler is a complete compilation pipeline that transforms DOL (Design Ontology Language) source code into executable WebAssembly modules. It orchestrates multiple compilation phases, from lexical analysis through WASM emission.

## Architecture

```
┌─────────────┐
│ DOL Source  │  .dol file or string
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Lexer     │  Tokenization
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │  Syntax Analysis → AST
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     HIR     │  High-level IR (canonical, desugared)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    MLIR     │  Multi-level IR (optimizable) [IN PROGRESS]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    WASM     │  WebAssembly bytecode
└─────────────┘
```

## Module Structure

```
src/compiler/
├── mod.rs      # Module definition and exports
└── spirit.rs   # Spirit compiler implementation
```

## API Reference

### Functions

#### `compile_source(source: &str, filename: &str) -> Result<CompiledSpirit, CompilerError>`

Compiles DOL source code to WASM.

**Arguments:**
- `source` - DOL source code string
- `filename` - Source filename (for error reporting and debug info)

**Returns:**
- `CompiledSpirit` containing WASM bytecode and metadata
- `CompilerError` if compilation fails

**Example:**
```rust
use metadol::compiler::spirit::compile_source;

let source = r#"
module example @ 1.0.0

gene Counter {
    has value: Int64
}

exegesis {
    A simple counter gene.
}
"#;

let result = compile_source(source, "example.dol")?;
println!("Compiled {} bytes of WASM", result.wasm.len());
```

#### `compile_file(path: &Path) -> Result<CompiledSpirit, CompilerError>`

Compiles a DOL file to WASM.

**Arguments:**
- `path` - Path to .dol file

**Returns:**
- `CompiledSpirit` containing WASM bytecode
- `CompilerError` if file cannot be read or compilation fails

**Example:**
```rust
use metadol::compiler::spirit::compile_file;
use std::path::Path;

let compiled = compile_file(Path::new("src/main.dol"))?;
std::fs::write("output.wasm", &compiled.wasm)?;
```

#### `compile_spirit_project(project_dir: &Path) -> Result<CompiledSpirit, CompilerError>`

Compiles a complete Spirit project.

**Arguments:**
- `project_dir` - Path to Spirit project directory

**Returns:**
- `CompiledSpirit` containing compiled WASM
- `CompilerError` if project structure is invalid or compilation fails

**Project Structure:**
```
my-spirit/
├── manifest.toml       # Project metadata
└── src/
    ├── main.dol       # Entry point (required)
    └── helper.dol     # Additional modules (optional)
```

**Example:**
```rust
use metadol::compiler::spirit::compile_spirit_project;
use std::path::Path;

let project = Path::new("examples/my-spirit");
let compiled = compile_spirit_project(project)?;
```

### Types

#### `CompiledSpirit`

Result of successful compilation.

**Fields:**
- `wasm: Vec<u8>` - WebAssembly bytecode
- `source_map: Option<SourceMap>` - Debug source map
- `warnings: Vec<CompilerWarning>` - Non-fatal compilation warnings

#### `SourceMap`

Maps WASM offsets to DOL source locations for debugging.

**Fields:**
- `entries: Vec<SourceMapEntry>` - Individual mapping entries

#### `SourceMapEntry`

Single source location mapping.

**Fields:**
- `wasm_offset: u32` - Offset in WASM bytecode
- `source_file: String` - Source file path
- `line: u32` - Line number (1-indexed)
- `column: u32` - Column number (1-indexed)

#### `CompilerWarning`

Non-fatal compilation warning.

**Fields:**
- `message: String` - Warning message
- `location: Option<(String, u32, u32)>` - Optional (file, line, column)

#### `CompilerError`

Compilation error enum.

**Variants:**
- `LexError(String)` - Lexer error
- `ParseError(ParseError)` - Parser error
- `HirError(String)` - HIR lowering error
- `MlirError(String)` - MLIR lowering error
- `WasmError(String)` - WASM emission error
- `IoError(std::io::Error)` - I/O error
- `ProjectError(String)` - Project structure error

## Compilation Phases

### Phase 1: Lexical Analysis (Complete)

Tokenizes DOL source into a stream of tokens.

**Status:** ✓ Complete
**Module:** `crate::lexer`

### Phase 2: Syntax Analysis (Complete)

Parses token stream into an Abstract Syntax Tree (AST).

**Status:** ✓ Complete
**Module:** `crate::parser`

### Phase 3: HIR Lowering (Complete)

Desugars AST into canonical High-level Intermediate Representation.

**Status:** ✓ Complete
**Module:** `crate::lower`

**Transformations:**
- Desugar `for` loops into `loop` + `match`
- Desugar `while` loops into `loop` + `if`
- Normalize types and expressions
- Emit deprecation warnings

### Phase 4: MLIR Lowering (In Progress)

Lowers HIR to MLIR operations for optimization.

**Status:** ⧗ Skeleton implementation
**Module:** `crate::mlir`

**TODO:**
- HIR → MLIR type mapping
- Expression lowering
- Statement lowering
- Control flow graph construction
- SSA form conversion

### Phase 5: WASM Emission (In Progress)

Generates WebAssembly bytecode from MLIR.

**Status:** ⧗ Placeholder (generates valid WASM header)
**Module:** `crate::wasm`

**TODO:**
- MLIR → WASM instruction mapping
- Type section generation
- Function section generation
- Code section generation
- Export section generation

## Current Implementation Status

### Working

- ✓ Complete parsing pipeline (Lexer → Parser → AST)
- ✓ AST to HIR lowering with desugaring
- ✓ Error handling and reporting
- ✓ Compilation API (`compile_source`, `compile_file`, `compile_spirit_project`)
- ✓ Valid WASM header generation (magic number + version)
- ✓ Comprehensive test coverage

### In Progress

- ⧗ HIR → MLIR lowering
- ⧗ MLIR optimization passes
- ⧗ MLIR → WASM emission
- ⧗ Source map generation
- ⧗ Full WASM section generation

### Planned

- ☐ Multi-file compilation
- ☐ Dependency resolution
- ☐ Incremental compilation
- ☐ Debug info (DWARF)
- ☐ Optimization levels (-O0, -O1, -O2, -O3)

## Usage Examples

### Example 1: Compile and Run

```rust
use metadol::compiler::spirit::compile_source;

let source = r#"
module calculator @ 1.0.0

fun add(a: Int64, b: Int64) -> Int64 {
    return a + b
}
"#;

let compiled = compile_source(source, "calculator.dol")?;

// Write to file
std::fs::write("calculator.wasm", &compiled.wasm)?;

// Or execute with wasmtime
use wasmtime::*;
let engine = Engine::default();
let module = Module::from_binary(&engine, &compiled.wasm)?;
// ... instantiate and run
```

### Example 2: Error Handling

```rust
use metadol::compiler::spirit::{compile_source, CompilerError};

let source = r#"
module broken @ 1.0.0

gene Invalid {
    this is not valid syntax
}
"#;

match compile_source(source, "broken.dol") {
    Ok(_) => println!("Success"),
    Err(CompilerError::ParseError(e)) => {
        eprintln!("Syntax error: {}", e);
    }
    Err(e) => {
        eprintln!("Compilation failed: {}", e);
    }
}
```

### Example 3: Collect Warnings

```rust
use metadol::compiler::spirit::compile_source;

let source = r#"
module example @ 1.0.0

gene Test {
    has field: Int64
}

exegesis {
    Test gene.
}
"#;

let compiled = compile_source(source, "example.dol")?;

for warning in &compiled.warnings {
    println!("Warning: {}", warning.message);
    if let Some((file, line, col)) = &warning.location {
        println!("  at {}:{}:{}", file, line, col);
    }
}
```

## Feature Flags

The Spirit compiler requires the `wasm` feature:

```toml
[dependencies]
metadol = { version = "0.5.0", features = ["wasm"] }
```

This feature enables:
- WASM compilation support
- Wasmtime runtime integration
- wasm-encoder for binary generation

## Testing

### Unit Tests

Located in `src/compiler/spirit.rs`:

```bash
cargo test --lib --features wasm compiler::spirit
```

### Integration Tests

Located in `tests/compiler_integration.rs`:

```bash
cargo test --test compiler_integration --features wasm
```

### Demo

Run the interactive demo:

```bash
cargo run --example compiler_demo --features wasm
```

## Performance Considerations

### Compilation Time

Current implementation focuses on correctness over speed. Optimization opportunities:

- Parallel parsing of multiple files
- Incremental compilation
- Caching of HIR and MLIR
- Lazy MLIR optimization

### Output Size

Current WASM output is minimal (8 bytes: header only). Full implementation will include:

- Type section (~100-500 bytes typical)
- Function section (~50-200 bytes per function)
- Code section (~20-1000 bytes per function)
- Export section (~50-100 bytes per export)

Optimization opportunities:
- Dead code elimination
- Inlining
- Constant folding
- WASM binary compression

## Known Limitations

1. **Incomplete Pipeline**: MLIR → WASM lowering is stubbed
2. **Single File**: Multi-file compilation not yet implemented
3. **No Optimization**: No MLIR optimization passes yet
4. **No Source Maps**: Debug info not yet generated
5. **Limited Types**: Complex types (genes, traits) not yet mapped to WASM

## Future Enhancements

### Short Term (v0.5.x)

- Complete HIR → MLIR lowering
- Basic MLIR → WASM emission
- Simple function compilation

### Medium Term (v0.6.x)

- Multi-file compilation
- Dependency resolution
- Optimization passes (constant folding, DCE)
- Source map generation

### Long Term (v1.0+)

- Full gene/trait compilation
- Constraint enforcement in WASM
- Advanced optimizations
- Debug info (DWARF)
- Incremental compilation
- WASM GC support

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on:

- Code style
- Testing requirements
- Documentation standards
- Review process

## References

- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [MLIR Documentation](https://mlir.llvm.org/)
- [DOL Language Specification](./HIR-SPECIFICATION.md)
- [HIR v0.4.0 Documentation](./HIR-SPECIFICATION.md)

## License

See [LICENSE](../LICENSE) for details.
