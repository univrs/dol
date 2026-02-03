# DOL → WASM Quick Start

Build DOL Spirit projects to WebAssembly in one command.

## Installation

```bash
cargo install --path . --features cli
```

## 5-Minute Tutorial

### 1. Create a Spirit

```bash
mkdir hello-wasm && cd hello-wasm
```

**Spirit.dol**:
```dol
spirit hello_wasm @ 0.1.0
docs "My first WASM module"
config { entry: "src/lib.dol", target: wasm32 }
targets {
    rust: { edition: "2021" }
    wasm: { optimize: true }
}
```

**src/lib.dol**:
```dol
fun greet(name: String) -> String {
    "Hello, " + name + "!"
}

fun add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 2. Build

```bash
dol-build
```

### 3. Use in JavaScript

```javascript
import init, { greet, add } from './target/spirit/pkg/hello_wasm.js';

await init();
console.log(greet("World"));  // "Hello, World!"
console.log(add(5, 3));        // 8
```

## What Works

✅ Pure functions
✅ Primitives (i32, bool, String)
✅ If/else, comparisons
✅ Arithmetic operators

## What Doesn't (Yet)

❌ Struct constructors
❌ Enum pattern matching
❌ Loops
❌ Collections

See [BUILD.md](cli/BUILD.md) for full details and roadmap.

## CLI Reference

```bash
dol-build [OPTIONS] [PROJECT_DIR]

Options:
  -o, --output <DIR>   Output directory (default: target/spirit)
  -r, --release        Optimized build
  -v, --verbose        Show detailed output
  --no-bindgen         Skip JS binding generation
  --clean              Clean build
```

## Examples

```bash
# Basic build
dol-build

# Release build with custom output
dol-build --release -o ./dist

# Verbose mode (for debugging)
dol-build -v

# Build specific project
dol-build ./my-spirit
```

## Next Steps

- [Full CLI Documentation](cli/BUILD.md)
- [DOL Specification](SPECIFICATION.md)
- [Examples](../examples/)
