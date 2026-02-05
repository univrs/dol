# dol-codegen-wasm

WASM code generation for DOL (Design Ontology Language).

## Overview

This crate provides the core infrastructure for generating WebAssembly modules from DOL source code. It handles:

- **Import Emission**: Type-safe generation of WASM import declarations for host functions
- **Import Tracking**: Optimization of imports by tracking which host functions are actually used
- **Type Conversion**: Mapping between DOL ABI types and WASM types

## Architecture

### Import Emitter (`imports/emitter.rs`)

The `ImportEmitter` generates WASM import declarations using the `walrus` crate for module manipulation.

**Key Components:**
- `ImportInfo`: Metadata about an imported function (host function, import ID, function ID)
- `ImportSection`: Collection of imports with efficient lookup
- `ImportEmitter`: Coordinates import emission into a WASM module

**Example:**
```rust
use dol_codegen_wasm::{ImportEmitter, standard_host_functions};
use walrus::Module;

let mut module = Module::default();
let mut emitter = ImportEmitter::new(&mut module);

// Import all standard host functions
let host_functions = standard_host_functions();
let section = emitter.emit_all(&host_functions)?;

// Get function ID for a specific import
let print_func_id = section.get_func_id("print").unwrap();
```

### Import Tracker (`imports/tracker.rs`)

The `ImportTracker` maps DOL prelude function names to host functions and tracks usage.

**Key Components:**
- `ImportTracker`: Maps function names to host functions
- `UsedImports`: Tracks which imports are actually used in generated code

**Example:**
```rust
use dol_codegen_wasm::{ImportTracker, UsedImports};

let tracker = ImportTracker::new();
let mut used = UsedImports::new();

// During code generation, track calls
used.track_call("print");
used.track_call("alloc");

// Filter to only used imports
let all_funcs = standard_host_functions();
let used_funcs = used.filter_used(&all_funcs);
// Only "print" and "alloc" will be in used_funcs
```

## Features

### Type-Safe Bindings

All host function signatures are defined in `dol-abi` and validated during import:

```rust
// From dol-abi
pub struct HostFunction {
    pub name: String,
    pub signature: HostFunctionSignature,
    pub category: HostFunctionCategory,
}

pub struct HostFunctionSignature {
    pub params: Vec<WasmType>,
    pub returns: Option<WasmType>,
}
```

### Deterministic Ordering

Imports are emitted in a deterministic order to ensure reproducible builds:

```rust
let imports = used.get_used_imports();
// Returns: ["alloc", "print", "send"] (alphabetically sorted)
```

### Error Handling

All operations return `Result` types with descriptive errors:

```rust
pub enum ImportError {
    FunctionNotFound(String),
    SignatureMismatch { function, expected, actual },
    DuplicateImport(String),
    ModuleError(String),
}
```

## Testing

The crate includes comprehensive unit tests:

```bash
cargo test
```

**Test Coverage:**
- Import section creation and lookup
- Signature conversion (DOL ABI → WASM types)
- Import emission (single and batch)
- Duplicate import detection
- Usage tracking and filtering
- Deterministic ordering

All tests pass with zero clippy warnings:

```bash
cargo clippy -- -D warnings
```

## Dependencies

- **walrus** (0.21): WASM module manipulation
- **dol-abi**: DOL ABI type definitions
- **thiserror**: Error type derivation

## Integration

This crate is part of the DOL compiler pipeline:

```
DOL Source → Parser → AST → [Codegen] → WASM Module
                              ↑
                   dol-codegen-wasm (this crate)
```

## Phase 2 Implementation Status

✅ **Complete:**
- Import emitter with `ImportInfo`, `ImportSection`, `ImportEmitter`
- Import tracker with `ImportTracker` and `UsedImports`
- Type-safe bindings using `dol-abi` types
- Deterministic import ordering
- Comprehensive error handling with `thiserror`
- Full unit test coverage (15 tests, 100% pass)
- Zero clippy warnings

## Next Steps

Phase 3 will implement:
- Memory layout and allocation
- Function body generation
- Type conversion utilities

## License

MIT OR Apache-2.0
