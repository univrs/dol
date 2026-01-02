# DOL 2.0 Phase 3: Code Generation & Semantic Analysis

## Background

**Completed:**
- **DOL 2.0 Phase 1**: Extended Syntax (272 tests)
  - Functional pipelines (`|>`, `>>`)
  - Pattern matching with guards
  - Lambda expressions
  - Control flow (`if/else`, `for`, `while`, `loop`)
  - Block expressions
  - Rich type annotations
  - Meta-programming primitives (`'`, `!`, `?`)

- **DOL 2.0 Phase 2**: Type Checker (87+ tests)
  - Bidirectional type inference
  - Type environment with scoped bindings
  - Numeric type promotion
  - Function type checking
  - Gradual typing support

## Phase 3 Objective

Build semantic analysis and code generation capabilities that leverage the type system to produce validated, executable artifacts.

---

## Task Queue

### Task 3.1: Type-Aware Validator Integration
**Priority**: High
**File**: `src/validator.rs` (modify), `src/typechecker.rs` (extend)

**Description**: Integrate type checker with existing validator for semantic validation.

**Subtasks**:
- [ ] Add type checking as validation pass
- [ ] Report type errors as validation errors
- [ ] Validate `has property: Type` declarations match DOL 2.0 types
- [ ] Validate `uses` references resolve to valid typed declarations
- [ ] Add `--typecheck` flag to dol-check CLI

**Acceptance Criteria**:
- Type errors surface during validation
- Integration tests for type validation

---

### Task 3.2: Rust Code Generator
**Priority**: High
**File**: `src/codegen.rs` (new), `src/codegen/rust.rs` (new)

**Description**: Generate Rust code from typed DOL declarations.

**Subtasks**:
- [ ] Create codegen module structure
- [ ] Generate Rust structs from `gene` declarations
- [ ] Generate trait definitions from `trait` declarations
- [ ] Generate type aliases from DOL 2.0 types
- [ ] Generate function signatures from lambda types
- [ ] Map DOL types to Rust types:
  - `Int32` → `i32`
  - `String` → `String`
  - `Option<T>` → `Option<T>`
  - `Result<T, E>` → `Result<T, E>`
  - `List<T>` → `Vec<T>`
  - `Map<K, V>` → `HashMap<K, V>`

**Output Example**:
```rust
// Generated from: gene container.exists @1.0.0
pub struct ContainerExists {
    pub id: String,
    pub image: String,
}

// Generated from: trait container.lifecycle @1.0.0
pub trait ContainerLifecycle: ContainerExists {
    fn current_state(&self) -> &str;
    fn state_history(&self) -> &[String];
}
```

**Acceptance Criteria**:
- Generated code compiles
- Tests for each declaration type
- CLI command: `dol-codegen --target rust file.dol`

---

### Task 3.3: TypeScript Code Generator
**Priority**: Medium
**File**: `src/codegen/typescript.rs` (new)

**Description**: Generate TypeScript interfaces and types from DOL.

**Subtasks**:
- [ ] Generate interfaces from gene declarations
- [ ] Generate type unions from trait compositions
- [ ] Generate type guards for pattern matching
- [ ] Map DOL types to TypeScript types:
  - `Int32` → `number`
  - `String` → `string`
  - `Bool` → `boolean`
  - `Option<T>` → `T | undefined`
  - `Result<T, E>` → custom union type

**Output Example**:
```typescript
// Generated from: gene container.exists @1.0.0
export interface ContainerExists {
  id: string;
  image: string;
}

// Generated from: trait container.lifecycle @1.0.0
export interface ContainerLifecycle extends ContainerExists {
  currentState: string;
  stateHistory: string[];
}
```

**Acceptance Criteria**:
- Generated TypeScript is valid
- Tests for each declaration type
- CLI command: `dol-codegen --target typescript file.dol`

---

### Task 3.4: JSON Schema Generator
**Priority**: Medium
**File**: `src/codegen/jsonschema.rs` (new)

**Description**: Generate JSON Schema from DOL type declarations.

**Subtasks**:
- [ ] Generate schema from gene properties
- [ ] Handle nested types and generics
- [ ] Generate $ref for trait compositions
- [ ] Map DOL types to JSON Schema types

**Output Example**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ContainerExists",
  "type": "object",
  "properties": {
    "id": { "type": "string" },
    "image": { "type": "string" }
  },
  "required": ["id", "image"]
}
```

**Acceptance Criteria**:
- Generated schemas validate correctly
- CLI command: `dol-codegen --target jsonschema file.dol`

---

### Task 3.5: Expression Evaluator (Interpreter)
**Priority**: Low
**File**: `src/eval.rs` (new)

**Description**: Interpret DOL 2.0 expressions for testing and REPL.

**Subtasks**:
- [ ] Implement value representation
- [ ] Evaluate literals, arithmetic, comparisons
- [ ] Evaluate lambda application
- [ ] Evaluate pipe and compose operators
- [ ] Evaluate pattern matching
- [ ] Evaluate control flow
- [ ] Evaluate block expressions

**Acceptance Criteria**:
- Comprehensive expression evaluation tests
- REPL command: `dol-repl`

---

### Task 3.6: dol-codegen CLI
**Priority**: High (after 3.2)
**File**: `src/bin/dol-codegen.rs` (new)

**Description**: CLI tool for code generation.

**Commands**:
```bash
dol-codegen --target rust file.dol          # Generate Rust
dol-codegen --target typescript file.dol    # Generate TypeScript
dol-codegen --target jsonschema file.dol    # Generate JSON Schema
dol-codegen --output src/gen/ --target rust dir/  # Output to directory
dol-codegen --dry-run --target rust file.dol      # Preview output
```

**Acceptance Criteria**:
- All targets work
- Directory traversal
- CI-friendly exit codes

---

## Implementation Order

```
┌─────────────────────────────────────────────────────────┐
│  Phase 3 Critical Path                                   │
│                                                          │
│  3.1 Type-Aware Validator ────┬──→ 3.2 Rust Codegen     │
│         (foundation)          │           │              │
│                               │           ↓              │
│                               └──→ 3.6 dol-codegen CLI  │
│                                           │              │
│  ─────────────────────────────────────────┼───────────  │
│                                           │              │
│  3.3 TypeScript Codegen  ←────────────────┤              │
│  3.4 JSON Schema Codegen ←────────────────┘              │
│                                                          │
│  ─────────────────────────────────────────────────────  │
│                                                          │
│  3.5 Expression Evaluator (independent, can parallelize)│
└─────────────────────────────────────────────────────────┘
```

---

## Ready for Queue

The following tasks have no blockers and can start immediately:

| Task | Priority | Estimated Effort | Dependencies |
|------|----------|------------------|--------------|
| 3.1 Type-Aware Validator | High | 4 hours | None |
| 3.2 Rust Code Generator | High | 8 hours | None (or after 3.1) |
| 3.5 Expression Evaluator | Low | 12 hours | None |

---

## Success Criteria

| Check | Target |
|-------|--------|
| `cargo test` | All tests pass |
| `dol-parse --typecheck file.dol` | Reports type errors |
| `dol-codegen --target rust examples/` | Generates valid Rust |
| `dol-codegen --target typescript examples/` | Generates valid TS |
| `dol-codegen --target jsonschema examples/` | Generates valid JSON Schema |

---

## Files to Create

```
src/
├── codegen/
│   ├── mod.rs           # Codegen module root
│   ├── rust.rs          # Rust code generation
│   ├── typescript.rs    # TypeScript generation
│   └── jsonschema.rs    # JSON Schema generation
├── eval.rs              # Expression evaluator
└── bin/
    └── dol-codegen.rs   # Codegen CLI
```
