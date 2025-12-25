# SEX System: Side Effect eXecution

The SEX (Side Effect eXecution) system is DOL 2.0's approach to managing impurity in a functional-first language. By default, all DOL code is pure. Side effects must be explicitly marked, tracked, and isolated.

## Table of Contents

1. [Philosophy](#philosophy)
2. [Core Concepts](#core-concepts)
3. [Syntax](#syntax)
4. [File-Based Context](#file-based-context)
5. [Effect Tracking](#effect-tracking)
6. [Linting Rules](#linting-rules)
7. [Code Generation](#code-generation)
8. [Best Practices](#best-practices)
9. [API Reference](#api-reference)

---

## Philosophy

DOL follows the principle of **explicit effects**:

> "Pure by default, impure by choice."

This design:
- Makes side effects visible in function signatures
- Prevents accidental impurity in pure code
- Enables aggressive optimization of pure code
- Simplifies reasoning about program behavior
- Improves testability through effect isolation

---

## Core Concepts

### Purity

DOL distinguishes between two kinds of code:

| Context | Description | Allowed Operations |
|---------|-------------|-------------------|
| **Pure** | Side-effect free | Computation, pure functions |
| **Sex** | Side-effects permitted | I/O, FFI, mutable globals |

### Effect Kinds

The SEX system tracks six categories of effects:

| Effect | Description | Examples |
|--------|-------------|----------|
| `Io` | I/O operations | File read/write, network, console |
| `Ffi` | Foreign function calls | C library calls, system calls |
| `MutableGlobal` | Mutable global state | Global counters, caches |
| `NonDeterministic` | Non-deterministic ops | Random, current time |
| `Unsafe` | Unsafe operations | Raw pointers, unchecked casts |
| `General` | Generic side effect | Explicitly marked `sex` code |

---

## Syntax

### SEX Functions

Functions with side effects must be marked with `sex`:

```dol
// Pure function (default) - no side effects
fun add(a: Int32, b: Int32) -> Int32 {
    return a + b
}

// Sex function - has side effects
sex fun print(message: String) -> Void {
    // Can perform I/O
    console_write(message)
}

// Public sex function
pub sex fun write_file(path: String, data: String) -> Result<(), Error> {
    // Can perform file I/O
}
```

### SEX Variables

Mutable global state requires `sex var`:

```dol
// Immutable constant (pure)
const PI: Float64 = 3.14159

// Mutable global (requires sex context)
sex var counter: Int32 = 0
sex var cache: Map<String, Value> = Map::new()
```

### SEX Blocks

Isolate side effects within otherwise pure code:

```dol
fun compute_with_logging(x: Int32) -> Int32 {
    let result = x * 2  // Pure computation

    sex {
        // Side effects isolated here
        log("computed: " + result.to_string())
        metrics.increment("computations")
    }

    return result  // Pure return
}
```

### Extern (FFI)

Foreign function declarations are implicitly `sex`:

```dol
// FFI declarations (implicitly sex)
extern fun malloc(size: UInt64) -> *Void
extern fun free(ptr: *Void) -> Void
extern fun printf(format: *Int8, ...) -> Int32

// Using FFI (requires sex context)
sex fun allocate_buffer(size: UInt64) -> *Void {
    let ptr = malloc(size)
    if ptr == null {
        panic("allocation failed")
    }
    return ptr
}
```

---

## File-Based Context

DOL automatically determines context from file paths.

### Sex Context Detection

Files are in **sex context** if:

1. **Extension**: File has `.sex.dol` extension
2. **Directory**: File is in a `sex/` directory

```
src/
├── genes/
│   └── container.dol      # Pure context
├── traits/
│   └── lifecycle.dol      # Pure context
├── sex/
│   └── io.dol             # Sex context (directory)
└── ffi.sex.dol            # Sex context (extension)
```

### Example Usage

```rust
use metadol::sex::{is_sex_file, file_sex_context, SexContext};
use std::path::Path;

// Check file context
let pure = Path::new("container.dol");
assert_eq!(file_sex_context(pure), SexContext::Pure);

let sex_ext = Path::new("io.sex.dol");
assert_eq!(file_sex_context(sex_ext), SexContext::Sex);

let sex_dir = Path::new("src/sex/globals.dol");
assert_eq!(file_sex_context(sex_dir), SexContext::Sex);
```

---

## Effect Tracking

The `EffectTracker` analyzes DOL code to identify side effects.

### Automatic Detection

Effects are detected through:

1. **Property names**: `file_read`, `network_socket`, `global_state`
2. **Keyword usage**: `sex`, `var`, `extern`
3. **Function calls**: Calls to known effectful functions

### Detection Patterns

| Pattern | Detected Effect |
|---------|-----------------|
| `*input*`, `*output*`, `*file*`, `*network*` | `Io` |
| `*ffi*`, `*extern*`, `*native*`, `*syscall*` | `Ffi` |
| `*global*`, `*static*`, `*singleton*` | `MutableGlobal` |

### Tracking API

```rust
use metadol::sex::tracking::{EffectTracker, EffectKind};
use metadol::ast::Declaration;

let mut tracker = EffectTracker::new();

// Track effects in a declaration
tracker.track_declaration(&declaration);

// Check for effects
if tracker.has_effects("my.gene") {
    let effects = tracker.get_effects("my.gene");
    for effect in effects {
        println!("{}: {}", effect.kind, effect.span);
    }
}
```

---

## Linting Rules

The SEX linter enforces purity constraints.

### Errors (Must Fix)

| Code | Name | Description |
|------|------|-------------|
| E001 | Sex in pure context | Side effect in pure context |
| E002 | Mutable global outside sex | Global state access in pure code |
| E003 | FFI outside sex | FFI call in pure context |
| E004 | I/O outside sex | I/O operation in pure context |

### Warnings (Should Fix)

| Code | Name | Description |
|------|------|-------------|
| W001 | Large sex block | Sex block exceeds 50 statements |
| W002 | Sex without documentation | Sex function lacks exegesis |

### Using the Linter

```rust
use metadol::sex::lint::SexLinter;
use metadol::sex::context::SexContext;

// Create linter for pure context
let linter = SexLinter::new(SexContext::Pure)
    .with_max_block_size(30);  // Custom block size limit

// Lint a declaration
let result = linter.lint_declaration(&decl);

if result.has_errors() {
    for error in &result.errors {
        eprintln!("Error {}: {}", error.code(), error);
    }
}

if result.has_warnings() {
    for warning in &result.warnings {
        eprintln!("Warning {}: {}", warning.code(), warning);
    }
}
```

### Example Violations

```dol
// E004: I/O outside sex
gene pure.example {
    processor has file_read  // ERROR: I/O in pure context

    exegesis { Pure gene with I/O - will fail linting }
}

// Correct version
gene pure.example {
    processor has capability

    exegesis { Pure gene without I/O }
}

// Or use sex file: io.sex.dol
gene io.example {
    processor has file_read  // OK: sex context

    exegesis { Sex gene with I/O }
}
```

---

## Code Generation

### Rust Output

Sex functions generate documented Rust code:

**DOL Input:**
```dol
pub sex fun mutate(x: Int32) -> Int32 {
    return x + 1
}
```

**Rust Output:**
```rust
/// Side-effectful function: mutate
///
/// WARNING: This function has side effects.
pub fn mutate(x: i32) -> i32 {
    return (x + 1);
}
```

### Sex Blocks

Sex blocks generate annotated code:

**DOL Input:**
```dol
fun compute(x: Int32) -> Int32 {
    let y = x * 2
    sex {
        log(y)
    }
    return y
}
```

**Rust Output:**
```rust
fn compute(x: i32) -> i32 {
    let y = x * 2;
    /* sex block */ {
        log(y);
    }
    return y;
}
```

---

## Best Practices

### 1. Minimize Sex Scope

Keep side effects as small and isolated as possible:

```dol
// Bad: Large sex function
sex fun process_all(items: [Item]) -> [Result] {
    let results = []
    for item in items {
        let processed = transform(item)  // Pure!
        results.push(processed)
        log("processed: " + item.id)     // Impure
    }
    return results
}

// Good: Isolated sex blocks
fun process_all(items: [Item]) -> [Result] {
    let results = items.map(|item| transform(item))  // Pure

    sex {
        for item in items {
            log("processed: " + item.id)
        }
    }

    return results
}
```

### 2. Document Sex Functions

Always provide thorough exegesis for sex functions:

```dol
sex fun write_config(path: String, config: Config) -> Result<(), Error> {
    // Implementation

    exegesis {
        Writes configuration to the filesystem.

        Side Effects:
        - Writes to file at `path`
        - Creates parent directories if needed
        - Overwrites existing file

        Error Conditions:
        - Returns Err if path is invalid
        - Returns Err if disk is full
        - Returns Err if permissions denied
    }
}
```

### 3. Use Sex Files for I/O Modules

Group related I/O operations in `.sex.dol` files:

```
src/
├── models/
│   └── user.dol           # Pure data structures
├── logic/
│   └── validation.dol     # Pure validation
└── io/
    ├── database.sex.dol   # Database I/O
    ├── http.sex.dol       # HTTP client
    └── files.sex.dol      # File system
```

### 4. Prefer Pure Functions

When possible, restructure to minimize impurity:

```dol
// Instead of: sex fun that does computation + I/O
sex fun fetch_and_process(url: String) -> Data {
    let raw = http_get(url)
    let parsed = parse(raw)
    let transformed = transform(parsed)
    return transformed
}

// Prefer: Pure processing + minimal sex
fun process(raw: String) -> Data {
    let parsed = parse(raw)
    return transform(parsed)
}

sex fun fetch_and_process(url: String) -> Data {
    let raw = http_get(url)  // Only I/O
    return process(raw)      // Pure processing
}
```

---

## API Reference

### Module: `metadol::sex`

```rust
// Context detection
pub fn is_sex_file(path: &Path) -> bool;
pub fn file_sex_context(path: &Path) -> SexContext;

// Re-exports
pub use context::{FileContext, SexContext};
pub use lint::{LintResult, SexLintError, SexLintWarning, SexLinter};
pub use tracking::EffectTracker;
```

### Enum: `SexContext`

```rust
pub enum SexContext {
    Pure,  // Default - no side effects
    Sex,   // Side effects permitted
}

impl SexContext {
    pub fn is_pure(&self) -> bool;
    pub fn is_sex(&self) -> bool;
}
```

### Struct: `FileContext`

```rust
pub struct FileContext {
    pub path: PathBuf,
    pub sex_context: SexContext,
}

impl FileContext {
    pub fn new(path: PathBuf) -> Self;
    pub fn with_context(path: PathBuf, context: SexContext) -> Self;
    pub fn is_sex(&self) -> bool;
    pub fn is_pure(&self) -> bool;
}
```

### Enum: `EffectKind`

```rust
pub enum EffectKind {
    Io,              // I/O operations
    Ffi,             // Foreign function calls
    MutableGlobal,   // Mutable global state
    NonDeterministic, // Random, time, etc.
    Unsafe,          // Unsafe operations
    General,         // Generic side effect
}
```

### Struct: `EffectTracker`

```rust
pub struct EffectTracker { /* ... */ }

impl EffectTracker {
    pub fn new() -> Self;
    pub fn track_declaration(&mut self, decl: &Declaration);
    pub fn track_expr(&mut self, expr: &Expr, effects: &mut Vec<Effect>);
    pub fn has_effects(&self, name: &str) -> bool;
    pub fn get_effects(&self, name: &str) -> &[Effect];
    pub fn set_purity(&mut self, name: String, purity: Purity);
    pub fn get_purity(&self, name: &str) -> Option<Purity>;
}
```

### Struct: `SexLinter`

```rust
pub struct SexLinter { /* ... */ }

impl SexLinter {
    pub fn new(context: SexContext) -> Self;
    pub fn with_max_block_size(self, size: usize) -> Self;
    pub fn lint_declaration(&self, decl: &Declaration) -> LintResult;
}
```

---

## Version

This documentation covers the SEX system as implemented in DOL v0.1.0.

For syntax details, see [syntax-reference.md](./syntax-reference.md).
For the full specification, see [specification.md](./specification.md).
