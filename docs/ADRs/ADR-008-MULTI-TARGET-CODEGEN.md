# ADR-008: Multi-Target Code Generation

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2025-11-10 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Context

DOL must compile to multiple targets to serve different use cases:
- **WASM** for universal sandboxed execution
- **Rust** for native performance and ecosystem integration
- **TypeScript** for JavaScript ecosystem
- **Python** for data science and scripting
- **JSON Schema** for validation in any language

### Requirements

1. **Single Source of Truth** - One .dol file generates all targets
2. **Semantic Preservation** - Same behavior across targets
3. **Idiomatic Output** - Generated code follows target conventions
4. **Incremental Compilation** - Only regenerate changed modules
5. **Debug Support** - Source maps for all targets

### Options Considered

| Approach | Flexibility | Maintenance | Performance |
|----------|-------------|-------------|-------------|
| **Shared HIR → Target Codegen** | ✅ | ✅ | ✅ |
| **Direct AST → Target** | ⚠️ | ❌ | ✅ |
| **Transpile through one target** | ❌ | ✅ | ⚠️ |

## Decision

**We chose a shared High-Level IR (HIR) with pluggable code generators.**

### Architecture

```
                              DOL Source (.dol)
                                     │
                                     ▼
                              ┌─────────────┐
                              │   Parser    │
                              │   (AST)     │
                              └──────┬──────┘
                                     │
                                     ▼
                              ┌─────────────┐
                              │    Type     │
                              │   Checker   │
                              └──────┬──────┘
                                     │
                                     ▼
                              ┌─────────────┐
                              │    HIR      │
                              │  (Shared)   │
                              └──────┬──────┘
                                     │
         ┌───────────┬───────────┬───┴───┬───────────┬───────────┐
         │           │           │       │           │           │
         ▼           ▼           ▼       ▼           ▼           ▼
    ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
    │  WASM   │ │  Rust   │ │   TS    │ │ Python  │ │  JSON   │
    │ Codegen │ │ Codegen │ │ Codegen │ │ Codegen │ │ Schema  │
    └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘
         │           │           │           │           │
         ▼           ▼           ▼           ▼           ▼
      .wasm        .rs         .ts         .py        .json
```

### Why Shared HIR?

1. **Single Optimization Pass** - Optimize once in HIR, benefit all targets
2. **Consistent Semantics** - Type erasure happens once
3. **Easier Maintenance** - New targets only implement codegen
4. **Better Testing** - Test HIR transformations once

## Target Specifications

### WASM (Primary Target)

```bash
dol build --target wasm src/lib.dol -o output/
```

**Output:** `.wasm` binary + JS glue code

**Features:**
- WASI support for system calls
- Component Model compatible
- Source maps for debugging
- Tree-shaking for size

**Type Mapping:**

| DOL Type | WASM Type |
|----------|-----------|
| `i8-i64` | `i32/i64` |
| `u8-u64` | `i32/i64` |
| `f32/f64` | `f32/f64` |
| `bool` | `i32` (0/1) |
| `string` | Linear memory + length |
| `Vec<T>` | Linear memory + length + capacity |
| `gen` | Struct in linear memory |

### Rust

```bash
dol build --target rust src/lib.dol -o output/
```

**Output:** `.rs` files with Cargo.toml

**Features:**
- Idiomatic Rust patterns
- derive macros for common traits
- Proper lifetime annotations
- Integration with Rust ecosystem

**Type Mapping:**

| DOL Type | Rust Type |
|----------|-----------|
| `i32` | `i32` |
| `i64` | `i64` |
| `u64` | `u64` |
| `f64` | `f64` |
| `bool` | `bool` |
| `string` | `String` |
| `Vec<T>` | `Vec<T>` |
| `Option<T>` | `Option<T>` |
| `Result<T,E>` | `Result<T,E>` |
| `gen Foo` | `struct Foo` |
| `rule` | `fn validate_*(&self) -> bool` |

**Example Output:**

```rust
// Generated from: src/container.dol

/// A container for isolated execution.
#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub id: u64,
    pub name: String,
    pub running: bool,
}

impl Container {
    /// Validates the valid_id rule
    pub fn validate_valid_id(&self) -> bool {
        self.id > 0
    }
    
    /// Validates all rules
    pub fn validate(&self) -> bool {
        self.validate_valid_id()
    }
}
```

### TypeScript

```bash
dol build --target typescript src/lib.dol -o output/
```

**Output:** `.ts` files with package.json

**Features:**
- ES modules (ESM)
- Full type definitions
- Runtime validation functions
- Compatible with Deno, Node, Browser

**Type Mapping:**

| DOL Type | TypeScript Type |
|----------|-----------------|
| `i32/i64` | `number` |
| `u32/u64` | `number` |
| `f32/f64` | `number` |
| `bool` | `boolean` |
| `string` | `string` |
| `Vec<T>` | `T[]` |
| `Option<T>` | `T \| null` |
| `Result<T,E>` | `{ ok: true, value: T } \| { ok: false, error: E }` |
| `gen Foo` | `interface Foo` |

**Example Output:**

```typescript
// Generated from: src/container.dol

/**
 * A container for isolated execution.
 */
export interface Container {
    id: number;
    name: string;
    running: boolean;
}

/**
 * Validates the valid_id rule
 */
export function validateValidId(self: Container): boolean {
    return self.id > 0;
}

/**
 * Validates all rules
 */
export function validate(self: Container): boolean {
    return validateValidId(self);
}

/**
 * Runtime type guard
 */
export function isContainer(value: unknown): value is Container {
    return (
        typeof value === 'object' &&
        value !== null &&
        typeof (value as Container).id === 'number' &&
        typeof (value as Container).name === 'string' &&
        typeof (value as Container).running === 'boolean'
    );
}
```

### Python

```bash
dol build --target python src/lib.dol -o output/
```

**Output:** `.py` files with pyproject.toml

**Features:**
- Type hints (PEP 484)
- Dataclasses for gen types
- Pydantic validation optional
- Python 3.10+ compatible

**Type Mapping:**

| DOL Type | Python Type |
|----------|-------------|
| `i32/i64` | `int` |
| `u32/u64` | `int` |
| `f32/f64` | `float` |
| `bool` | `bool` |
| `string` | `str` |
| `Vec<T>` | `list[T]` |
| `Option<T>` | `T \| None` |
| `Result<T,E>` | `Result[T, E]` (custom) |
| `gen Foo` | `@dataclass class Foo` |

**Example Output:**

```python
# Generated from: src/container.dol

from dataclasses import dataclass

@dataclass
class Container:
    """A container for isolated execution."""
    id: int
    name: str
    running: bool
    
    def validate_valid_id(self) -> bool:
        """Validates the valid_id rule"""
        return self.id > 0
    
    def validate(self) -> bool:
        """Validates all rules"""
        return self.validate_valid_id()
```

### JSON Schema

```bash
dol build --target jsonschema src/lib.dol -o output/
```

**Output:** `.json` schema files

**Features:**
- JSON Schema Draft 2020-12
- Validation in any language
- OpenAPI compatible
- Documentation included

**Example Output:**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.com/container.json",
  "title": "Container",
  "description": "A container for isolated execution.",
  "type": "object",
  "properties": {
    "id": {
      "type": "integer",
      "minimum": 1,
      "description": "Must be greater than 0 (valid_id rule)"
    },
    "name": {
      "type": "string"
    },
    "running": {
      "type": "boolean"
    }
  },
  "required": ["id", "name", "running"],
  "additionalProperties": false
}
```

## Consequences

### Positive

- **Universal Reach** - DOL code runs everywhere
- **Best of Both Worlds** - Type safety + runtime validation
- **Ecosystem Integration** - Use DOL with existing codebases
- **Gradual Adoption** - Generate types for existing projects

### Negative

- **Maintenance Burden** - Five code generators to maintain
- **Semantic Gaps** - Some DOL features don't map cleanly
- **Testing Complexity** - Must test all target combinations

### Neutral

- **Output Quality** - Generated code is readable but not hand-optimized
- **Build Time** - Multi-target builds take longer

## Implementation Notes

### CodeGenerator Trait

```rust
pub trait CodeGenerator {
    type Output;
    type Error;
    
    fn generate(&self, hir: &HIR) -> Result<Self::Output, Self::Error>;
    fn target_name(&self) -> &'static str;
}

// Implementations
pub struct WasmCodegen { /* ... */ }
pub struct RustCodegen { /* ... */ }
pub struct TypeScriptCodegen { /* ... */ }
pub struct PythonCodegen { /* ... */ }
pub struct JsonSchemaCodegen { /* ... */ }
```

### CLI Interface

```bash
# Build single target
dol build --target rust src/

# Build multiple targets
dol build --target rust,typescript,jsonschema src/

# Build all targets
dol build --all-targets src/

# Watch mode
dol build --target typescript --watch src/
```

### Spirit.dol Configuration

```dol
spirit MySpirit {
    targets {
        wasm: {
            optimize: true
            target: "wasm32-wasi"
        }
        rust: {
            edition: "2024"
            derive: ["Debug", "Clone", "Serialize"]
        }
        typescript: {
            esm: true
            runtime: "deno"
            strict: true
        }
        python: {
            min_version: "3.10"
            use_pydantic: true
        }
        jsonschema: {
            draft: "2020-12"
            include_descriptions: true
        }
    }
}
```

## Test Strategy

### Cross-Target Validation

```rust
#[test]
fn test_semantic_equivalence() {
    let dol_source = include_str!("fixtures/container.dol");
    
    // Generate all targets
    let wasm = WasmCodegen::new().generate(dol_source)?;
    let rust = RustCodegen::new().generate(dol_source)?;
    let ts = TypeScriptCodegen::new().generate(dol_source)?;
    
    // Test with same input
    let input = Container { id: 42, name: "test".into(), running: true };
    
    assert!(wasm_validate(&wasm, &input));
    assert!(rust_validate(&rust, &input));
    assert!(ts_validate(&ts, &input));
}
```

## References

- [LLVM Multi-Target](https://llvm.org/docs/CodeGenerator.html)
- [GraalVM Truffle](https://www.graalvm.org/latest/graalvm-as-a-platform/language-implementation-framework/)
- [Protocol Buffers](https://protobuf.dev/) (multi-language code gen)
- [OpenAPI Generator](https://openapi-generator.tech/)

## Changelog

| Date | Change |
|------|--------|
| 2025-11-10 | Initial multi-target design |
| 2025-12-01 | Added Python target |
| 2026-01-15 | Updated to v0.8.1 examples |
| 2026-02-01 | Added JSON Schema target |
