# DOL Composition & Runtime Architecture

> **Design Document v0.1**  
> **Status:** RFC (Request for Comments)  
> **Last Updated:** December 22, 2025

---

## Table of Contents

1. [Overview](#overview)
2. [File Organization](#file-organization)
3. [Module System](#module-system)
4. [Spirit Packages](#spirit-packages)
5. [Entry Points](#entry-points)
6. [Compilation Targets](#compilation-targets)
7. [Runtime Models](#runtime-models)
8. [Séance Sessions](#séance-sessions)
9. [Examples](#examples)

---

## Overview

DOL 2.0 needs a clear model for:

| Concept | Question |
|---------|----------|
| **Composition** | How do .dol files relate to each other? |
| **Packaging** | How are Spirits (packages) structured? |
| **Entry Points** | What's "main" vs library code? |
| **Compilation** | What targets exist beyond VUDO OS? |
| **Execution** | JIT vs AOT? Interpreted vs compiled? |

### Design Principles

1. **Ontology First** — Specification before implementation
2. **Multi-Target** — One source, many outputs
3. **Progressive** — Start simple, scale to distributed
4. **Interoperable** — Play nice with existing ecosystems

---

## File Organization

### File Extensions

| Extension | Purpose | Example |
|-----------|---------|---------|
| `.dol` | DOL source file | `container.dol` |
| `.spirit` | Spirit package manifest | `Spirit.toml` or `spirit.dol` |
| `.seance` | Session state snapshot | `session.seance` |

### File Types by Purpose

```
┌─────────────────────────────────────────────────────────────────┐
│                        DOL File Types                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ONTOLOGY FILES (Specification)                                 │
│  ├── gene.*.dol      - Type definitions                         │
│  ├── trait.*.dol     - Interface contracts                      │
│  ├── constraint.*.dol - Validation rules                        │
│  └── system.*.dol    - Composed modules                         │
│                                                                 │
│  IMPLEMENTATION FILES (Behavior)                                │
│  ├── *.impl.dol      - Trait implementations                    │
│  ├── *.spell.dol     - Function libraries (Spells)              │
│  └── *.main.dol      - Entry point (main)                       │
│                                                                 │
│  META FILES (Configuration)                                     │
│  ├── Spirit.dol      - Package manifest                         │
│  ├── test.*.dol      - Test files                               │
│  └── bench.*.dol     - Benchmark files                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Naming Conventions

```dol
// Ontology: noun-based, describes "what it is"
gene.container.dol        // Container gene
trait.runnable.dol        // Runnable trait
constraint.positive.dol   // Positive constraint

// Implementation: verb-based, describes "what it does"
container.impl.dol        // Container implementation
math.spell.dol            // Math spells (functions)
scheduler.main.dol        // Scheduler entry point
```

---

## Module System

### Module Declaration

Every .dol file implicitly declares a module based on its path:

```dol
// File: src/container/lifecycle.dol
// Implicit module: container.lifecycle

// Explicit module override (optional)
module container.lifecycle {
    // contents
}
```

### Imports and Exports

```dol
// ═══════════════════════════════════════════════════════════════
// IMPORTS
// ═══════════════════════════════════════════════════════════════

// Import specific items
use container.{ Container, ContainerState }

// Import all public items
use container.*

// Import with alias
use container.Container as C

// Import from Spirit (external package)
use @univrs/scheduler.{ Scheduler, Task }

// Import from standard library
use std.collections.{ List, Map }
use std.io.{ read, write }

// ═══════════════════════════════════════════════════════════════
// EXPORTS
// ═══════════════════════════════════════════════════════════════

// Public by default (can be accessed from other modules)
pub gene Container { ... }

// Private (module-internal only)
priv gene InternalHelper { ... }

// Re-export from another module
pub use container.Container
```

### Visibility Modifiers

| Modifier | Scope | Example |
|----------|-------|---------|
| `pub` | Public, accessible everywhere | `pub gene Container` |
| `priv` | Private to current module | `priv fun helper()` |
| `pub(spirit)` | Public within Spirit only | `pub(spirit) gene Internal` |
| `pub(parent)` | Public to parent module | `pub(parent) fun util()` |

---

## Spirit Packages

### What is a Spirit?

A **Spirit** is DOL's package unit — a shareable collection of modules with:
- Manifest (Spirit.dol)
- Ontology (genes, traits)
- Implementation (spells)
- Optional entry point (main)

### Spirit Structure

```
my-spirit/
├── Spirit.dol              # Package manifest
├── src/
│   ├── lib.dol             # Library root (re-exports)
│   ├── main.dol            # Entry point (optional)
│   ├── genes/
│   │   ├── container.dol
│   │   └── process.dol
│   ├── traits/
│   │   └── runnable.dol
│   ├── spells/
│   │   ├── lifecycle.dol
│   │   └── network.dol
│   └── impl/
│       └── container.impl.dol
├── tests/
│   └── container_test.dol
├── examples/
│   └── basic.dol
└── target/                 # Build outputs
    ├── wasm/
    ├── rust/
    └── typescript/
```

### Spirit Manifest (Spirit.dol)

```dol
spirit MySpirit {
    // ═══════════════════════════════════════════════════════════
    // METADATA
    // ═══════════════════════════════════════════════════════════
    
    name: "my-spirit"
    version: "0.1.0"
    authors: ["Your Name <you@example.com>"]
    license: "MIT"
    
    exegesis {
        A Spirit for container orchestration.
        Provides genes for container lifecycle management.
    }
    
    // ═══════════════════════════════════════════════════════════
    // DEPENDENCIES
    // ═══════════════════════════════════════════════════════════
    
    requires {
        @univrs/std: "^1.0"        // Standard library
        @univrs/network: "^0.5"    // Network utilities
        @community/logging: "^2.0" // Third-party Spirit
    }
    
    // ═══════════════════════════════════════════════════════════
    // BUILD CONFIGURATION
    // ═══════════════════════════════════════════════════════════
    
    targets {
        wasm: { optimize: true }
        rust: { edition: "2024" }
        typescript: { esm: true }
    }
    
    // ═══════════════════════════════════════════════════════════
    // ENTRY POINTS
    // ═══════════════════════════════════════════════════════════
    
    // Library entry (for use as dependency)
    lib: "src/lib.dol"
    
    // Binary entry (for standalone execution)
    bin: [
        { name: "my-cli", path: "src/main.dol" },
        { name: "my-daemon", path: "src/daemon.dol" }
    ]
}
```

---

## Entry Points

### Library vs Binary

| Type | Purpose | Has `main`? | Output |
|------|---------|-------------|--------|
| **Library** | Reusable code | No | Importable module |
| **Binary** | Executable | Yes | Runnable program |
| **Hybrid** | Both | Optional | Either |

### The `main` Function

```dol
// src/main.dol

use std.io.{ println }
use std.env.{ args }

// Entry point for binary execution
pub fun main(args: List<String>) -> Int32 {
    println("Hello from DOL!")
    
    for arg in args {
        println("Arg: " + arg)
    }
    
    return 0  // Exit code
}
```

### Library Root

```dol
// src/lib.dol

// Re-export public API
pub use genes.container.Container
pub use genes.process.Process
pub use traits.runnable.Runnable
pub use spells.lifecycle.{ start, stop, restart }

// Library-level exegesis
exegesis {
    Container orchestration library.
    
    Quick start:
    ```dol
    use @myorg/containers.{ Container, start }
    
    container = Container { id: 1, name: "web" }
    start(container)
    ```
}
```

### Spell Files (Function Libraries)

```dol
// src/spells/math.spell.dol

pub fun add(a: Int64, b: Int64) -> Int64 {
    return a + b
}

pub fun multiply(a: Int64, b: Int64) -> Int64 {
    return a * b
}

pub fun fibonacci(n: Int64) -> Int64 {
    match n {
        0 { return 0 }
        1 { return 1 }
        _ { return fibonacci(n - 1) + fibonacci(n - 2) }
    }
}

// Higher-order spell
pub fun twice(f: Fun<Int64, Int64>, x: Int64) -> Int64 {
    return f(f(x))
}
```

---

## Compilation Targets

### Multi-Target Architecture

```
                              DOL Source
                                  │
                                  ▼
                         ┌───────────────┐
                         │   DOL Parser  │
                         │   + TypeCheck │
                         └───────┬───────┘
                                 │
                          Typed AST
                                 │
              ┌──────────────────┼──────────────────┐
              │                  │                  │
              ▼                  ▼                  ▼
      ┌───────────┐      ┌───────────┐      ┌───────────┐
      │   Rust    │      │   WASM    │      │TypeScript │
      │  Codegen  │      │  Codegen  │      │  Codegen  │
      └─────┬─────┘      └─────┬─────┘      └─────┬─────┘
            │                  │                  │
            ▼                  ▼                  ▼
      ┌───────────┐      ┌───────────┐      ┌───────────┐
      │   .rs     │      │   .wasm   │      │   .ts     │
      │  files    │      │  binary   │      │  files    │
      └─────┬─────┘      └───────────┘      └─────┬─────┘
            │                  │                  │
            ▼                  ▼                  ▼
      ┌───────────┐      ┌───────────┐      ┌───────────┐
      │  Native   │      │  Browser  │      │   Node    │
      │  Binary   │      │   WASM    │      │    or     │
      │ (via gcc) │      │  Runtime  │      │   Deno    │
      └───────────┘      └───────────┘      └───────────┘
```

### Target Matrix

| Target | Command | Output | Runtime |
|--------|---------|--------|---------|
| **Rust** | `dol build --target rust` | `.rs` files | Native via `cargo` |
| **WASM** | `dol build --target wasm` | `.wasm` binary | Browser, Wasmtime, VUDO |
| **TypeScript** | `dol build --target typescript` | `.ts` files | Node, Deno, Browser |
| **Python** | `dol build --target python` | `.py` files | Python 3.x |
| **JSON Schema** | `dol build --target jsonschema` | `.json` schemas | Validation |
| **MLIR** | `dol build --target mlir` | `.mlir` IR | LLVM toolchain |

### Target-Specific Configuration

```dol
// In Spirit.dol

targets {
    rust {
        edition: "2024"
        features: ["async", "serde"]
        derive: ["Debug", "Clone", "Serialize"]
    }
    
    wasm {
        optimize: true
        target: "wasm32-wasi"  // or "wasm32-unknown-unknown"
        features: ["simd", "threads"]
    }
    
    typescript {
        esm: true
        strict: true
        runtime: "deno"  // or "node", "browser"
    }
    
    python {
        version: "3.11"
        type_hints: true
        dataclasses: true
    }
}
```

---

## Runtime Models

### 1. VUDO OS (Full Platform)

The complete VUDO ecosystem with Spirit orchestration:

```
┌─────────────────────────────────────────────────────────────────┐
│                          VUDO OS                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Spirit    │  │   Spirit    │  │   Spirit    │             │
│  │  (web-ui)   │  │  (api-gw)   │  │  (worker)   │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                     │
│         └────────────────┼────────────────┘                     │
│                          │                                      │
│                    ┌─────┴─────┐                                │
│                    │  Mycelium │  ← P2P Network                 │
│                    │  Network  │                                │
│                    └─────┬─────┘                                │
│                          │                                      │
│  ┌───────────────────────┼───────────────────────┐             │
│  │                   VUDO VM                      │             │
│  │  ┌─────────┐  ┌─────────────┐  ┌───────────┐  │             │
│  │  │  WASM   │  │   Spirit    │  │ Identity  │  │             │
│  │  │ Runtime │  │  Scheduler  │  │ (Ed25519) │  │             │
│  │  └─────────┘  └─────────────┘  └───────────┘  │             │
│  └───────────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Standalone WASM

Run DOL code anywhere WASM runs:

```bash
# Compile to WASM
dol build --target wasm src/main.dol -o app.wasm

# Run with Wasmtime
wasmtime app.wasm

# Run in browser
<script type="module">
  const { instance } = await WebAssembly.instantiateStreaming(
    fetch('app.wasm')
  );
  instance.exports.main();
</script>
```

### 3. Native via Rust

Compile DOL to Rust, then to native binary:

```bash
# Generate Rust
dol build --target rust src/main.dol -o generated/

# Compile with Cargo
cd generated && cargo build --release

# Run native binary
./target/release/my-spirit
```

### 4. JIT Interpretation

For rapid development and REPL:

```bash
# Start DOL REPL
dol repl

DOL> x = 42
42

DOL> fun double(n: Int64) -> Int64 { n * 2 }
<function double>

DOL> double(x)
84

DOL> :type double
Fun<Int64, Int64>
```

### 5. Embedded in Host Language

```rust
// Rust host
use dol_runtime::Runtime;

fn main() {
    let mut rt = Runtime::new();
    
    rt.load_spirit("./my-spirit").unwrap();
    
    let result: i64 = rt.call("math.add", (1, 2)).unwrap();
    println!("Result: {}", result);
}
```

```typescript
// TypeScript host
import { Runtime } from '@dol/runtime';

const rt = new Runtime();
await rt.loadSpirit('./my-spirit');

const result = await rt.call('math.add', [1, 2]);
console.log(`Result: ${result}`);
```

---

## Séance Sessions

### What is a Séance?

A **Séance** is a collaborative editing/execution session where multiple participants can:
- Edit DOL code in real-time
- See live compilation results
- Share execution state
- Invoke Spirits together

### Séance Model

```
┌─────────────────────────────────────────────────────────────────┐
│                         Séance Session                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Participants:                                                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                         │
│  │  Alice  │  │   Bob   │  │  Carol  │                         │
│  │ (Mambo) │  │ (Editor)│  │(Viewer) │                         │
│  └────┬────┘  └────┬────┘  └────┬────┘                         │
│       │            │            │                               │
│       └────────────┼────────────┘                               │
│                    │                                            │
│            ┌───────┴───────┐                                    │
│            │  Shared State │                                    │
│            │  ┌─────────┐  │                                    │
│            │  │  Code   │  │  ← Live DOL source                 │
│            │  │ Buffer  │  │                                    │
│            │  └─────────┘  │                                    │
│            │  ┌─────────┐  │                                    │
│            │  │  AST    │  │  ← Incrementally updated           │
│            │  │  Cache  │  │                                    │
│            │  └─────────┘  │                                    │
│            │  ┌─────────┐  │                                    │
│            │  │ Runtime │  │  ← Live execution                  │
│            │  │  State  │  │                                    │
│            │  └─────────┘  │                                    │
│            └───────────────┘                                    │
│                                                                 │
│  Actions:                                                       │
│  • Edit code  → Recompile → Update all views                   │
│  • Run spell  → Execute  → Broadcast result                    │
│  • Summon spirit → Load → Available to all                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Séance API

```dol
use std.seance.{ Seance, Participant, Role }

// Create a new séance
seance = Seance.create({
    name: "Container Workshop",
    spirits: ["@univrs/containers"],
    access: Role.Invite
})

// Join existing séance
seance = Seance.join("seance://abc123")

// Collaborative editing
seance.on_edit(fun (change) {
    println("Edit by " + change.author + ": " + change.delta)
})

// Execute spell together
result = seance.invoke("containers.start", container)

// Snapshot session state
seance.save("workshop-checkpoint.seance")
```

---

## Examples

### Example 1: Simple CLI Tool

```dol
// Spirit.dol
spirit Greeter {
    name: "greeter"
    version: "1.0.0"
    bin: [{ name: "greet", path: "src/main.dol" }]
}
```

```dol
// src/main.dol
use std.io.println
use std.env.args

pub fun main(args: List<String>) -> Int32 {
    name = if args.length > 1 { args[1] } else { "World" }
    println("Hello, " + name + "!")
    return 0
}
```

```bash
# Build and run
dol build --target rust
cargo run -- "DOL Developer"
# Output: Hello, DOL Developer!
```

### Example 2: Library Spirit

```dol
// Spirit.dol
spirit MathLib {
    name: "@myorg/math"
    version: "1.0.0"
    lib: "src/lib.dol"
}
```

```dol
// src/lib.dol
pub use spells.arithmetic.{ add, subtract, multiply, divide }
pub use spells.trig.{ sin, cos, tan }
pub use genes.complex.Complex
```

```dol
// src/genes/complex.dol
pub gene Complex {
    has real: Float64
    has imag: Float64
    
    constraint valid {
        !this.real.is_nan && !this.imag.is_nan
    }
}
```

```dol
// Consumer in another Spirit
use @myorg/math.{ add, Complex }

result = add(1, 2)
c = Complex { real: 3.0, imag: 4.0 }
```

### Example 3: Multi-Target Build

```bash
# Build for all targets
dol build --target rust src/lib.dol -o target/rust/
dol build --target typescript src/lib.dol -o target/ts/
dol build --target wasm src/main.dol -o target/wasm/
dol build --target jsonschema src/genes/ -o target/schemas/

# Result:
# target/
# ├── rust/
# │   └── lib.rs
# ├── ts/
# │   └── lib.ts
# ├── wasm/
# │   └── main.wasm
# └── schemas/
#     ├── container.schema.json
#     └── process.schema.json
```

### Example 4: REPL Development

```bash
$ dol repl

DOL 2.0 REPL (Q2 Meta-Programming)
Type :help for commands

DOL> gene Point { has x: Int64, has y: Int64 }
Defined gene Point

DOL> p = Point { x: 10, y: 20 }
Point { x: 10, y: 20 }

DOL> fun distance(p: Point) -> Float64 {
...>   return sqrt(p.x * p.x + p.y * p.y)
...> }
Defined fun distance

DOL> distance(p)
22.360679774997898

DOL> expr = '(p.x + p.y)
Quoted<Int64>

DOL> !expr
30

DOL> :type expr
Quoted<Int64>

DOL> :ast expr
Quote(Binary(Add, Member(p, x), Member(p, y)))

DOL> :save session.seance
Session saved to session.seance

DOL> :quit
Goodbye! 🍄
```

---

## Summary: The DOL Execution Model

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                      DOL Source (.dol)                          │
│                            │                                    │
│                            ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    DOL Compiler                          │   │
│  │  Parse → TypeCheck → [Optimize] → Codegen               │   │
│  └─────────────────────────────────────────────────────────┘   │
│                            │                                    │
│         ┌──────────────────┼──────────────────┐                │
│         │                  │                  │                 │
│         ▼                  ▼                  ▼                 │
│    ┌─────────┐       ┌─────────┐       ┌─────────┐             │
│    │  Rust   │       │  WASM   │       │   TS    │             │
│    └────┬────┘       └────┬────┘       └────┬────┘             │
│         │                 │                 │                   │
│         ▼                 ▼                 ▼                   │
│    ┌─────────┐       ┌─────────┐       ┌─────────┐             │
│    │ Native  │       │ VUDO OS │       │  Node   │             │
│    │ Binary  │       │ Browser │       │  Deno   │             │
│    │  CLI    │       │Wasmtime │       │ Browser │             │
│    └─────────┘       └─────────┘       └─────────┘             │
│                                                                 │
│  Entry Points:                                                  │
│  • main.dol  → Executable binary                               │
│  • lib.dol   → Importable library                              │
│  • *.spell.dol → Function collections                          │
│                                                                 │
│  Packaging:                                                     │
│  • Spirit.dol → Package manifest                               │
│  • Mycelium   → Package registry (P2P)                         │
│                                                                 │
│  Collaboration:                                                 │
│  • Séance     → Live editing session                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Open Questions

1. **Module Resolution:** How to resolve `@org/spirit` references? DNS? DHT? Registry?

2. **Version Pinning:** Semantic versioning? Lock files? Content-addressed?

3. **Cross-Target Compatibility:** How to handle target-specific code? `#cfg(target)`?

4. **FFI:** How do Spirits call native code? WASM imports? Rust `extern`?

5. **Hot Reload:** How to update running Spirits in VUDO OS?

---

*"Systems that can become, what you can imagine!"*
