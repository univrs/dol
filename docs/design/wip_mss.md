# Module and Spirits System Design

**Status**: Design Document
**Author**: DOL Language Team
**Date**: 2026-01-15
**Version**: 0.1.0

## Overview

This document specifies the DOL module system and Spirit compilation model. It defines how DOL source files are organized into modules, how modules relate to each other through visibility and imports, and how Spirits (compiled DOL artifacts) are produced and consumed.

## Table of Contents

1. [Module System](#1-module-system)
2. [Spirit Compilation](#2-spirit-compilation)
3. [Visibility Rules](#3-visibility-rules)
4. [Import System](#4-import-system)
5. [Spirit Manifest](#5-spirit-manifest)
6. [Project Structure](#6-project-structure)
7. [Compilation Pipeline](#7-compilation-pipeline)
8. [Implementation Phases](#8-implementation-phases)
9. [Test Specifications](#9-test-specifications)

---

## 1. Module System

### 1.1 Module Declaration

Every DOL file begins with an optional module declaration that establishes its identity and version:

```dol
module container.lifecycle @ 1.0.0
```

**Syntax:**
```ebnf
module_decl = "module" qualified_identifier "@" version
version     = digit+ "." digit+ "." digit+
```

**Semantics:**
- Module name establishes the namespace for all declarations
- Version follows semantic versioning (MAJOR.MINOR.PATCH)
- Files without module declaration inherit from parent directory name

### 1.2 Module Hierarchy

Modules form a tree structure based on qualified names:

```
univrs                    # Root module
├── container             # Sub-module
│   ├── exists           # Leaf module (gene)
│   ├── lifecycle        # Leaf module (trait)
│   └── integrity        # Leaf module (constraint)
├── identity
│   └── cryptographic
└── orchestrator          # System module
    ├── scheduler
    └── coordinator
```

### 1.3 Module Resolution Rules

1. **Explicit module**: Use declared module name
2. **File-based**: Derive from file path (`src/container/exists.dol` → `container.exists`)
3. **Directory index**: `src/container/mod.dol` → `container`

---

## 2. Spirit Compilation

### 2.1 What is a Spirit?

A **Spirit** is the compiled artifact of a DOL project. Spirits are:

- **Versioned**: Carry semantic version from module declaration
- **Portable**: Compile to WebAssembly for cross-platform execution
- **Self-describing**: Include metadata about types, exports, and dependencies

### 2.2 Spirit Structure

```
┌─────────────────────────────────────┐
│           Spirit Binary             │
├─────────────────────────────────────┤
│  Magic: "SPRT" (0x53505254)        │
│  Version: u32                       │
├─────────────────────────────────────┤
│  Metadata Section                   │
│  - Module name                      │
│  - Semantic version                 │
│  - Dependencies                     │
│  - Export signatures                │
├─────────────────────────────────────┤
│  WASM Binary                        │
│  - Standard WASM module             │
│  - Custom sections for debug info   │
├─────────────────────────────────────┤
│  Source Map (optional)              │
│  - WASM offset → DOL location       │
└─────────────────────────────────────┘
```

### 2.3 Spirit Compilation Pipeline

```
DOL Source → Lexer → Parser → AST → HIR → MLIR → WASM → Spirit
                                      ↓
                              Type Checking
                              Validation
                              Optimization
```

**Phase Details:**

| Phase | Input | Output | Responsibility |
|-------|-------|--------|----------------|
| Lexing | Source text | Token stream | Tokenization, span tracking |
| Parsing | Tokens | AST | Syntax analysis, error recovery |
| Lowering | AST | HIR | Desugaring, canonicalization |
| Type Check | HIR | Typed HIR | Type inference, validation |
| MLIR Gen | Typed HIR | MLIR | IR generation, optimization |
| WASM Emit | MLIR | WASM | Code generation |
| Packaging | WASM + Meta | Spirit | Bundle creation |

---

## 3. Visibility Rules

### 3.1 Visibility Levels

DOL supports four visibility levels:

| Level | Keyword | Scope | Rust Equivalent |
|-------|---------|-------|-----------------|
| Private | (default) | Current module only | (private) |
| Public | `pub` | All modules | `pub` |
| Spirit | `pub(spirit)` | Current spirit/crate | `pub(crate)` |
| Parent | `pub(parent)` | Parent module | `pub(super)` |

### 3.2 Visibility Syntax

```dol
// Private (default)
gene container.internal {
    container has secret_state
}

// Public - visible everywhere
pub gene container.exists {
    container has identity
}

// Spirit-level - visible within compiled spirit
pub(spirit) trait internal.helper {
    uses container.exists
}

// Parent-level - visible to parent module
pub(parent) fun helper_function() -> Int64 {
    return 42
}
```

### 3.3 Visibility Rules by Declaration Type

| Declaration | Default | Allowed Modifiers |
|-------------|---------|-------------------|
| Gene | Private | `pub`, `pub(spirit)`, `pub(parent)` |
| Trait | Private | `pub`, `pub(spirit)`, `pub(parent)` |
| Constraint | Private | `pub`, `pub(spirit)`, `pub(parent)` |
| System | Private | `pub`, `pub(spirit)` |
| Function | Private | `pub`, `pub(spirit)`, `pub(parent)` |
| Gene Field | Private | `pub`, `pub(spirit)`, `pub(parent)` |

### 3.4 Visibility Enforcement

The type checker enforces visibility at:

1. **Import sites**: Cannot import private items from other modules
2. **Use sites**: Cannot reference items outside their visibility scope
3. **Field access**: Gene fields respect visibility modifiers
4. **Method calls**: Methods inherit gene visibility unless overridden

---

## 4. Import System

### 4.1 Use Declarations

```dol
// Import specific declaration
use container.exists

// Import with alias
use identity.cryptographic as crypto

// Import all from module (future)
use container.*
```

### 4.2 Qualified References

Without imports, use fully qualified names:

```dol
trait container.lifecycle {
    uses container.exists           // Explicit dependency

    fun create() -> container.exists.Container {
        // Fully qualified type reference
    }
}
```

### 4.3 Import Resolution

1. Check current module
2. Check explicitly imported modules
3. Check spirit dependencies
4. Check standard library (std)

---

## 5. Spirit Manifest

### 5.1 manifest.toml Structure

```toml
[spirit]
name = "univrs-container"
version = "1.0.0"
authors = ["Univrs Team"]
description = "Container management for Univrs"
license = "MIT"

[dependencies]
univrs-identity = "^0.5.0"
univrs-network = ">=0.3.0, <1.0.0"

[dev-dependencies]
univrs-test = "0.1.0"

[build]
entry = "src/main.dol"
target = "wasm32-unknown-unknown"
optimize = true

[exports]
# Explicitly exported items (in addition to pub items)
functions = ["create_container", "destroy_container"]
genes = ["Container", "ContainerConfig"]
```

### 5.2 Version Constraints

| Syntax | Meaning |
|--------|---------|
| `"1.0.0"` | Exact version |
| `"^1.0.0"` | Compatible (>=1.0.0, <2.0.0) |
| `"~1.0.0"` | Patch updates (>=1.0.0, <1.1.0) |
| `">=1.0.0"` | Greater or equal |
| `">=1.0.0, <2.0.0"` | Range |

---

## 6. Project Structure

### 6.1 Standard Layout

```
my-spirit/
├── manifest.toml           # Project manifest
├── src/
│   ├── main.dol           # Entry point
│   ├── lib.dol            # Library root (alternative)
│   ├── container/
│   │   ├── mod.dol        # Module index
│   │   ├── exists.dol     # Gene definition
│   │   └── lifecycle.dol  # Trait definition
│   └── util/
│       └── helpers.dol
├── tests/
│   ├── container_test.dol
│   └── integration_test.dol
├── examples/
│   └── basic_usage.dol
└── target/
    └── wasm/
        └── my-spirit.wasm
```

### 6.2 Module Discovery

The compiler discovers modules by:

1. Reading `src/main.dol` or `src/lib.dol`
2. Following `use` declarations
3. Scanning `src/` directory structure
4. Matching file paths to module names

---

## 7. Compilation Pipeline

### 7.1 Single File Compilation

```rust
// compile_source(source, filename) -> CompiledSpirit
let source = std::fs::read_to_string("src/main.dol")?;
let spirit = compile_source(&source, "main.dol")?;
```

### 7.2 Project Compilation

```rust
// compile_spirit_project(project_dir) -> CompiledSpirit
let project = Path::new("my-spirit/");
let spirit = compile_spirit_project(project)?;
```

### 7.3 Incremental Compilation

Future enhancement for large projects:

1. Hash source files
2. Compare with cached hashes
3. Recompile only changed modules
4. Link incrementally

### 7.4 Compilation Errors

```rust
pub enum CompilerError {
    LexError(String),
    ParseError(ParseError),
    HirError(String),
    MlirError(String),
    WasmError(String),
    IoError(std::io::Error),
    ProjectError(String),
}
```

---

## 8. Implementation Phases

### Phase 1: Module Declaration Parsing (Complete)
- [x] Parse `module` keyword and version
- [x] Track module context through compilation
- [x] Module name inference from file path
- [x] Tests: Module parsing, version validation

### Phase 2: Visibility System (Complete)
- [x] Visibility enum in AST
- [x] Parser support for visibility modifiers
- [x] HIR visibility threading
- [x] Rust codegen visibility mapping
- [ ] Type checker visibility enforcement
- [x] Tests: Visibility rules, error cases

### Phase 3: Import System (Planned)
- [ ] `use` declaration parsing
- [ ] Import resolution algorithm
- [ ] Qualified name resolution
- [ ] Cyclic dependency detection
- [ ] Tests: Import resolution, cycles

### Phase 4: Spirit Packaging (Planned)
- [ ] Spirit binary format
- [ ] Metadata section encoding
- [ ] Source map generation
- [ ] manifest.toml parsing
- [ ] Tests: Round-trip encoding

### Phase 5: Multi-Module Compilation (Planned)
- [ ] Project discovery
- [ ] Module dependency graph
- [ ] Compilation ordering (topological sort)
- [ ] Cross-module type checking
- [ ] Tests: Multi-file projects

### Phase 6: Incremental & Optimization (Planned)
- [ ] Source hashing
- [ ] Incremental recompilation
- [ ] Dead code elimination
- [ ] Spirit optimization
- [ ] Tests: Incremental builds

---

## 9. Test Specifications

See `tests/spirits/` directory for comprehensive test fixtures.

### 9.1 Test Categories

| Category | Directory | Purpose |
|----------|-----------|---------|
| Module Parsing | `tests/spirits/modules/` | Module declaration parsing |
| Visibility | `tests/spirits/visibility/` | Visibility enforcement |
| Imports | `tests/spirits/imports/` | Import resolution |
| Compilation | `tests/spirits/compilation/` | End-to-end spirit compilation |
| Errors | `tests/spirits/errors/` | Error case validation |

### 9.2 Test Execution

```bash
# Run all spirit tests
./scripts/run_spirit_tests.sh

# Run specific category
./scripts/run_spirit_tests.sh --category visibility

# Generate test report
./scripts/run_spirit_tests.sh --report
```

---

## Appendix A: Grammar Extensions

### Module Declaration
```ebnf
file           = [module_decl] {use_decl} declaration exegesis_block
module_decl    = "module" qualified_id "@" version
use_decl       = "use" qualified_id ["as" identifier]
```

### Visibility Modifiers
```ebnf
visibility     = "pub" ["(" visibility_scope ")"]
visibility_scope = "spirit" | "parent"
```

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **Module** | Namespace container for DOL declarations |
| **Spirit** | Compiled DOL artifact (WASM + metadata) |
| **Visibility** | Access control modifier |
| **Import** | Declaration bringing items into scope |
| **Manifest** | Project configuration file (manifest.toml) |
| **HIR** | High-level Intermediate Representation |
| **MLIR** | Multi-level Intermediate Representation |

---

## References

- [DOL Specification](../specification.md)
- [Self-Presence Inference](./self-presence-inference.md)
- [Option Type Design](./option-type.md)
- [Spirit Compiler Source](../../src/compiler/spirit.rs)
