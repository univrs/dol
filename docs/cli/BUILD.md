# DOL Build CLI Documentation

## Overview

`dol-build` compiles DOL Spirit projects to WebAssembly in a single command, orchestrating the complete pipeline from DOL source to deployable WASM modules with JavaScript bindings.

## Status: Alpha (v0.8.1)

**Current State**: Core pipeline functional for pure functions and basic types.

✅ **Working**: Pure functions, primitives (i32, bool), arithmetic, comparisons
⚠️ **Limited**: Complex types, enums, pattern matching, advanced features
📋 **Roadmap**: See [Path to 100%](#path-to-100-specification) below

## Installation

```bash
cd /path/to/univrs-dol
cargo install --path . --features cli
```

## Quick Start

### 1. Create a Spirit Project

```bash
mkdir my-spirit
cd my-spirit
```

Create `Spirit.dol`:
```dol
spirit my_spirit @ 0.1.0

docs "My first DOL Spirit"

config {
    entry: "src/lib.dol"
    target: wasm32
}

targets {
    rust: { edition: "2021" }
    wasm: { optimize: true, target: "wasm32-unknown-unknown" }
}
```

Create `src/lib.dol`:
```dol
// Pure functions (currently supported)
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun is_even(n: i32) -> bool {
    (n % 2) == 0
}

exegesis {
    Basic math functions for WASM.
}
```

### 2. Build to WASM

```bash
dol-build
```

Output:
```
info: Building Spirit: my_spirit @ 0.1.0

Stage 1: Resolving modules
✓ Resolved 1 modules

Stage 2: Generating Rust code
✓ Generated 2 Rust files

Stage 3: Generating Cargo.toml
✓ Generated Cargo.toml

Stage 4: Compiling to WASM
✓ Built WASM: target/spirit/target/wasm32-unknown-unknown/release/my_spirit.wasm

Stage 5: Generating JS bindings
✓ Generated JS bindings: target/spirit/pkg

Stage 6: Packaging output
✓ Packaged output

✓ Spirit build complete: target/spirit
```

### 3. Use in JavaScript

```javascript
import init, { add, is_even } from './target/spirit/pkg/my_spirit.js';

await init();

console.log(add(5, 3));        // 8
console.log(is_even(4));       // true
console.log(is_even(7));       // false
```

## CLI Reference

### Usage

```bash
dol-build [OPTIONS] [SPIRIT_DIR]
```

### Arguments

- `SPIRIT_DIR` - Path to Spirit project directory (default: current directory)

### Options

| Option | Description |
|--------|-------------|
| `-o, --output <DIR>` | Output directory (default: `target/spirit`) |
| `-r, --release` | Build with optimizations |
| `--no-bindgen` | Skip wasm-bindgen JS generation |
| `--clean` | Clean build (remove existing artifacts) |
| `-v, --verbose` | Verbose output |
| `-q, --quiet` | Quiet mode (errors only) |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

### Examples

```bash
# Build current directory
dol-build

# Build specific project with verbose output
dol-build ./my-spirit -v

# Release build with custom output
dol-build --release -o ./dist

# Build without JS bindings
dol-build --no-bindgen

# Clean build
dol-build --clean
```

## Build Pipeline (6 Stages)

### Stage 1: Module Resolution
- Scans `src/` directory for all `.dol` files
- Builds module dependency graph
- Skips `mod.dol` files (directory markers)

### Stage 2: Rust Code Generation
- Lowers DOL AST → HIR (High-level IR)
- Generates Rust code from HIR
- Creates `src/generated/*.rs` files
- Generates `src/lib.rs` with module declarations

**Current Capabilities**:
- ✅ Pure functions
- ✅ Primitive types (i32, i64, u8, u16, u32, u64, bool, String)
- ✅ Keyword escaping (e.g., `type` → `r#type`)
- ✅ Basic type inference from names
- ✅ Arithmetic and comparison operators
- ⚠️ Limited: Complex types, pattern matching, enums

### Stage 3: Cargo.toml Generation
- Creates WASM-compatible Cargo manifest
- Sets `crate-type = ["cdylib"]` for WASM
- Adds `wasm-bindgen` dependency
- Configures release optimizations

### Stage 4: WASM Compilation
- Runs `cargo build --target wasm32-unknown-unknown`
- Produces `.wasm` binary
- Applies LTO and size optimizations in release mode

### Stage 5: JS Bindings (wasm-bindgen)
- Generates JavaScript wrapper
- Creates TypeScript definitions
- Outputs to `pkg/` directory
- Enables web target for browser use

### Stage 6: Output Packaging
- Organizes artifacts in `target/spirit/`
- Creates manifest.json with Spirit metadata
- Ready for deployment

## Output Structure

```
target/spirit/
├── {spirit_name}.wasm              # Raw WASM binary
├── pkg/                            # wasm-bindgen output
│   ├── {spirit_name}.js           # JavaScript module
│   ├── {spirit_name}.d.ts         # TypeScript definitions
│   ├── {spirit_name}_bg.wasm     # Optimized WASM
│   └── {spirit_name}_bg.wasm.d.ts
├── src/
│   ├── generated/
│   │   └── *.rs                   # Generated Rust modules
│   └── lib.rs                     # Main library file
├── Cargo.toml                      # Build configuration
└── manifest.json                   # Spirit metadata
```

## What Works Today

### ✅ Fully Supported

**Functions**:
```dol
fun add(a: i32, b: i32) -> i32 { a + b }
fun subtract(a: i32, b: i32) -> i32 { a - b }
fun multiply(a: i32, b: i32) -> i32 { a * b }
fun divide(a: i32, b: i32) -> i32 { a / b }
```

**Comparisons**:
```dol
fun is_positive(n: i32) -> bool { n > 0 }
fun is_zero(n: i32) -> bool { n == 0 }
fun max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
```

**Primitive Types**:
- `i8`, `i16`, `i32`, `i64`
- `u8`, `u16`, `u32`, `u64`
- `bool`
- `String` (basic)

**Type Inference**:
```dol
gen Point {
    self has x: i32    // Inferred from name
    self has y: i32    // Inferred from name
    self has neighbors: u8  // Inferred from "neighbors"
}
```

## What Doesn't Work Yet

### ❌ Not Yet Supported

**1. Complex Struct Constructors**
```dol
// ❌ Doesn't work yet
gen Point {
    self has x: i32
    self has y: i32
}

fun make_point(x: i32, y: i32) -> Point {
    Point { x: x, y: y }  // Constructor syntax not yet supported
}
```

**Workaround**: Use separate functions, avoid struct construction in DOL.

**2. Enum Pattern Matching**
```dol
// ❌ Doesn't work yet
gen State {
    is Alive
    is Dead
}

fun next_state(current: State, neighbors: u8) -> State {
    match current {
        State.Alive => { /* ... */ }   // Pattern syntax issues
        State.Dead => { /* ... */ }
    }
}
```

**Issue**: Patterns generate `State.Alive` instead of `State::Alive`.

**3. Collections**
```dol
// ❌ Not implemented
fun make_list() -> List<i32> {
    list(1, 2, 3, 4, 5)
}
```

**4. Advanced Features**
- Loops (`for`, `while`)
- Mutable variables
- References and borrowing
- Generics
- Traits
- Effects
- Systems and constraints

## Path to 100% Specification

### Phase 1: Core Type System (2-3 weeks)

**Goal**: Support all DOL types in WASM compilation

**Tasks**:
1. **Fix struct constructors**
   - Generate proper struct literal syntax
   - Handle field shorthand
   - Support nested construction

2. **Fix enum variants**
   - HIR: Represent enum variants as Constructor patterns
   - Codegen: Generate `::` instead of `.` for enum access
   - Support variant payloads

3. **Collections**
   - Map DOL `List<T>` → Rust `Vec<T>`
   - Implement `list()` constructor
   - Add iteration support

**Deliverables**:
```dol
// All of this should work:
gen Point { self has x: i32, self has y: i32 }
gen State { is Alive, is Dead }

fun make_point(x: i32, y: i32) -> Point {
    Point { x, y }  ✅
}

fun next_state(state: State) -> State {
    match state {
        State::Alive => State::Dead   ✅
        State::Dead => State::Alive    ✅
    }
}

fun make_numbers() -> List<i32> {
    list(1, 2, 3, 4, 5)  ✅
}
```

### Phase 2: Control Flow (1-2 weeks)

**Goal**: Loops, mutable state, breaks

**Tasks**:
1. **Loops**
   - `for x in list { ... }`
   - `while condition { ... }`
   - `break` and `continue`

2. **Mutable variables**
   - `var x = 0` → `let mut x = 0`
   - Assignment statements
   - Rebinding

**Deliverables**:
```dol
fun sum_list(numbers: List<i32>) -> i32 {
    var total = 0
    for n in numbers {
        total = total + n
    }
    total
}  ✅
```

### Phase 3: Advanced Types (2-3 weeks)

**Goal**: References, generics, traits

**Tasks**:
1. **References**
   - Borrowed references (`&T`)
   - Mutable references (`&mut T`)
   - Lifetime inference

2. **Generics**
   - Generic functions
   - Generic types
   - Type constraints

3. **Traits**
   - Trait definitions
   - Trait implementations
   - Trait bounds

**Deliverables**:
```dol
// Generic function
fun identity<T>(value: T) -> T { value }  ✅

// Trait
trait Drawable {
    fun draw(self: &Self) -> String
}  ✅

// Implementation
impl Drawable for Point {
    fun draw(self: &Self) -> String {
        format("({}, {})", self.x, self.y)
    }
}  ✅
```

### Phase 4: DOL-Specific Features (3-4 weeks)

**Goal**: Full DOL semantics in WASM

**Tasks**:
1. **Effects**
   - Effect declarations
   - Effect handlers
   - WASM-compatible effect runtime

2. **Constraints**
   - Runtime constraint checking
   - Compile-time verification where possible

3. **Systems**
   - System declarations
   - Evolution tracking

**Deliverables**:
```dol
// Effect
effect State<T> {
    fun get() -> T
    fun set(value: T)
}  ✅

// Constraint
constraint NonNegative {
    self requires { self >= 0 }
}  ✅
```

### Phase 5: Optimization & Testing (2 weeks)

**Goal**: Production-ready WASM output

**Tasks**:
1. **Optimizations**
   - Dead code elimination
   - Inline small functions
   - WASM size optimization

2. **Testing**
   - Comprehensive test suite
   - Integration tests for all features
   - Performance benchmarks

3. **Documentation**
   - Complete API documentation
   - Tutorial series
   - Best practices guide

### Timeline Summary

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Phase 1: Core Type System | 2-3 weeks | 3 weeks |
| Phase 2: Control Flow | 1-2 weeks | 5 weeks |
| Phase 3: Advanced Types | 2-3 weeks | 8 weeks |
| Phase 4: DOL Features | 3-4 weeks | 12 weeks |
| Phase 5: Optimization | 2 weeks | 14 weeks |

**Total**: ~3-4 months to 100% specification coverage

## Troubleshooting

### Build Fails at Stage 2 (Codegen)

**Symptom**: Parse errors or unsupported syntax

**Solution**:
1. Check DOL syntax (use v0.8.0 keywords: `gen`, `fun`, not `gene`, `spell`)
2. Verify types are supported (see [What Works Today](#what-works-today))
3. Simplify to pure functions first

### Build Fails at Stage 4 (WASM)

**Symptom**: Rust compilation errors

**Solution**:
1. Check generated Rust: `cat target/spirit/src/generated/*.rs`
2. Look for unsupported constructs (struct constructors, enum patterns)
3. Report issues with generated code samples

### WASM Binary Too Large

**Solution**:
```bash
dol-build --release  # Enables LTO and opt-level="z"
```

Release builds typically 10-50x smaller than debug.

### Missing JS Bindings

**Solution**:
1. Ensure `wasm-bindgen` is installed: `cargo install wasm-bindgen-cli`
2. Check `--no-bindgen` flag wasn't used
3. Look for errors in Stage 5 output

## Performance

### Minimal Example (3 functions)
- **Source**: 10 lines DOL
- **Generated**: 26 lines Rust
- **WASM**: 363 bytes (debug), ~100 bytes (release + wasm-opt)
- **Build time**: ~20 seconds (first build), ~2 seconds (incremental)

### Complex Example (game-of-life, 7 modules)
- **Source**: ~200 lines DOL
- **Generated**: ~500 lines Rust
- **WASM**: ~8KB (release)
- **Build time**: ~30 seconds

## Best Practices

### 1. Start Simple
Begin with pure functions and primitives before adding complexity:
```dol
// Good: Start here
fun double(n: i32) -> i32 { n * 2 }

// Later: Add types when needed
gen Point { self has x: i32, self has y: i32 }
```

### 2. Use Verbose Mode
```bash
dol-build -v
```
Shows detailed progress and helps debug issues.

### 3. Incremental Development
Test frequently as you add features:
```bash
# After each change
dol-build && node test.js
```

### 4. Check Generated Code
```bash
cat target/spirit/src/generated/*.rs
```
Verify the Rust output looks correct.

### 5. Use Release Builds for Production
```bash
dol-build --release
```
Significantly reduces WASM size.

## Examples

### Example 1: Math Library (Working Today)
```dol
// Spirit.dol
spirit math @ 1.0.0

config {
    entry: "src/lib.dol"
    target: wasm32
}

targets {
    rust: { edition: "2021" }
    wasm: { optimize: true }
}

// src/lib.dol
fun abs(n: i32) -> i32 {
    if n < 0 { -n } else { n }
}

fun min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

fun max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

fun clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min { min }
    else if value > max { max }
    else { value }
}
```

**Status**: ✅ Compiles to WASM successfully

### Example 2: Game Logic (Partially Working)
```dol
// ⚠️ This example has limitations
gen Cell {
    self has alive: bool
    self has neighbors: u8
}

// ✅ This works
fun count_neighbors(alive: bool, neighbors: u8) -> u8 {
    if alive {
        if neighbors == 2 { 2 }
        else if neighbors == 3 { 3 }
        else { 0 }
    } else {
        if neighbors == 3 { 3 }
        else { 0 }
    }
}

// ❌ This doesn't work yet (struct constructor)
fun make_cell(alive: bool, neighbors: u8) -> Cell {
    Cell { alive: alive, neighbors: neighbors }
}
```

**Status**: ⚠️ Partial (pure functions work, constructors don't)

## Contributing

Found a bug or want to contribute? See [CONTRIBUTING.md](../../CONTRIBUTING.md)

## License

MIT OR Apache-2.0
