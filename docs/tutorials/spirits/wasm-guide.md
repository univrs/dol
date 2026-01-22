# DOL WASM Compilation Guide

> Complete pipeline from DOL source to WebAssembly execution

## Overview

DOL compiles to WebAssembly through a multi-stage pipeline:

```
DOL Source → AST → Rust IR → WASM Binary → Runtime Execution
```

This guide covers each stage of the compilation pipeline and how to work with WASM output.

## Compilation Pipeline

### Stage 1: DOL to AST

The parser converts DOL source code into an Abstract Syntax Tree:

```
┌─────────────────┐
│   DOL Source    │
│   (.dol file)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Lexer (logos)  │
│   Tokenization  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Parser      │
│ Recursive Desc. │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│       AST       │
│   (Typed Tree)  │
└─────────────────┘
```

```bash
# Generate AST from DOL
dol-parse examples/spirits/physics/particles.dol --output particles.ast.json
```

### Stage 2: AST to Rust

The code generator transforms the AST into Rust source code:

```rust
// DOL input:
// pub fun kinetic_energy(mass: f64, velocity: f64) -> f64 {
//     return 0.5 * mass * velocity * velocity
// }

// Generated Rust:
#[no_mangle]
pub extern "C" fn kinetic_energy(mass: f64, velocity: f64) -> f64 {
    0.5 * mass * velocity * velocity
}
```

```bash
# Generate Rust from DOL
vudo build --emit rust

# View generated code
cat target/codegen/particles.rs
```

### Stage 3: Rust to WASM

The Rust compiler produces WebAssembly:

```bash
# Build WASM binary
vudo build --target wasm32-unknown-unknown --release

# Output location
ls target/wasm32-unknown-unknown/release/*.wasm
```

### Stage 4: WASM Execution

The WASM binary runs in browsers or server-side runtimes:

```javascript
// Browser execution
const response = await fetch('physics.wasm');
const wasm = await WebAssembly.instantiate(await response.arrayBuffer());
const result = wasm.instance.exports.kinetic_energy(10.0, 100.0);
console.log(result); // 50000.0
```

## Building WASM from DOL

### Basic Build

```bash
# Build current Spirit to WASM
vudo build

# Build with optimizations
vudo build --release

# Build specific file
vudo build src/mechanics.dol
```

### Build Configuration

Configure WASM builds in `Spirit.dol`:

```dol
spirit Physics {
    has name: "physics"
    has version: "0.9.0"
    has lib: "src/lib.dol"

    // WASM-specific configuration
    has wasm: {
        target: "wasm32-unknown-unknown",
        optimize: true,
        strip: true,
        features: ["simd", "bulk-memory"]
    }
}
```

### Build Targets

| Target | Description | Use Case |
|--------|-------------|----------|
| `wasm32-unknown-unknown` | Minimal WASM | Browser, embedded |
| `wasm32-wasi` | WASI support | Server-side, CLI |
| `wasm32-unknown-emscripten` | Emscripten | Legacy browser support |

```bash
# Build for different targets
vudo build --target wasm32-unknown-unknown    # Default
vudo build --target wasm32-wasi               # With WASI
```

## WASM Code Generation

### Function Export

Public DOL functions are exported from WASM:

```dol
// DOL Source
pub fun add(a: i64, b: i64) -> i64 {
    return a + b
}

// Private (not exported)
fun helper() -> i64 {
    return 42
}
```

```rust
// Generated Rust
#[no_mangle]
pub extern "C" fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn helper() -> i64 {
    42
}
```

### Gene to Struct Translation

DOL genes become Rust structs:

```dol
// DOL Gene
pub gen Particle {
    has name: string
    has mass: f64
    has charge: f64
}

pub fun electron() -> Particle {
    return Particle {
        name: "Electron",
        mass: 9.1093837015e-31,
        charge: -1.602176634e-19
    }
}
```

```rust
// Generated Rust
#[repr(C)]
pub struct Particle {
    pub name: *const u8,
    pub name_len: usize,
    pub mass: f64,
    pub charge: f64,
}

#[no_mangle]
pub extern "C" fn electron() -> Particle {
    Particle {
        name: "Electron\0".as_ptr(),
        name_len: 8,
        mass: 9.1093837015e-31,
        charge: -1.602176634e-19,
    }
}
```

### Type Mapping

| DOL Type | Rust Type | WASM Type |
|----------|-----------|-----------|
| `i32` | `i32` | `i32` |
| `i64` | `i64` | `i64` |
| `f32` | `f32` | `f32` |
| `f64` | `f64` | `f64` |
| `bool` | `i32` | `i32` |
| `string` | `(*const u8, usize)` | `(i32, i32)` |
| `Option<T>` | `(i32, T)` | discriminant + value |
| `Vec<T>` | `(*const T, usize)` | `(i32, i32)` |

## Working with Generated WASM

### Inspecting WASM

```bash
# Show WASM structure
wasm2wat target/release/physics.wasm

# Show exports
wasm-objdump -x target/release/physics.wasm

# Show module info
wasm-tools print target/release/physics.wasm
```

### WASM Exports

```bash
$ wasm-objdump -x physics.wasm | grep "Export"

Export[0]: memory -> "memory"
Export[1]: __heap_base -> global[0]
Export[2]: add -> func[0]
Export[3]: kinetic_energy -> func[1]
Export[4]: gravitational_force -> func[2]
Export[5]: electron -> func[3]
```

### Size Optimization

```bash
# Check size
ls -lh target/release/*.wasm

# Optimize with wasm-opt
wasm-opt -Oz target/release/physics.wasm -o physics.opt.wasm

# Strip debug info
wasm-strip physics.opt.wasm

# Compare sizes
ls -lh target/release/physics.wasm physics.opt.wasm
```

## Browser Integration

### Loading WASM

```javascript
// physics-loader.js

async function loadPhysicsSpirit() {
    // Fetch WASM binary
    const response = await fetch('/wasm/physics.wasm');
    const wasmBytes = await response.arrayBuffer();

    // Define imports (for DOL runtime)
    const imports = {
        env: {
            // Memory allocation
            __alloc: (size) => { /* allocate memory */ },
            __dealloc: (ptr, size) => { /* free memory */ },

            // Console output
            __print_i64: (value) => console.log(value),
            __print_f64: (value) => console.log(value),
            __print_str: (ptr, len) => {
                const bytes = new Uint8Array(memory.buffer, ptr, len);
                console.log(new TextDecoder().decode(bytes));
            },
        }
    };

    // Instantiate module
    const { instance } = await WebAssembly.instantiate(wasmBytes, imports);

    return instance.exports;
}

// Usage
const physics = await loadPhysicsSpirit();

const energy = physics.kinetic_energy(10.0, 100.0);
console.log(`Kinetic energy: ${energy} J`);

const force = physics.gravitational_force(5.972e24, 1.0, 6.371e6);
console.log(`Gravitational force: ${force} N`);
```

### TypeScript Bindings

Generate TypeScript definitions for WASM exports:

```bash
# Generate TypeScript bindings
vudo build --emit typescript

# Output
cat target/codegen/physics.d.ts
```

```typescript
// physics.d.ts (generated)

export interface PhysicsExports {
    memory: WebAssembly.Memory;

    // Pure functions
    add(a: bigint, b: bigint): bigint;
    kinetic_energy(mass: number, velocity: number): number;
    gravitational_force(m1: number, m2: number, r: number): number;

    // Particle creation
    electron(): number;  // Returns pointer to Particle
}

export function loadPhysics(): Promise<PhysicsExports>;
```

### React Integration

```tsx
// PhysicsCalculator.tsx
import { useEffect, useState } from 'react';
import { loadPhysics, PhysicsExports } from './physics';

export function PhysicsCalculator() {
    const [physics, setPhysics] = useState<PhysicsExports | null>(null);
    const [result, setResult] = useState<number>(0);

    useEffect(() => {
        loadPhysics().then(setPhysics);
    }, []);

    const calculate = () => {
        if (physics) {
            const energy = physics.kinetic_energy(10.0, 100.0);
            setResult(energy);
        }
    };

    return (
        <div>
            <button onClick={calculate}>Calculate</button>
            <p>Kinetic Energy: {result} J</p>
        </div>
    );
}
```

## Server-Side WASM (WASI)

### Building for WASI

```bash
# Build with WASI target
vudo build --target wasm32-wasi

# Run with wasmtime
wasmtime target/wasm32-wasi/release/physics.wasm
```

### Node.js Integration

```javascript
// server.js
const { WASI } = require('wasi');
const fs = require('fs');

async function runPhysicsWASI() {
    const wasi = new WASI({
        args: ['physics'],
        env: process.env,
    });

    const wasmBytes = fs.readFileSync('target/wasm32-wasi/release/physics.wasm');
    const { instance } = await WebAssembly.instantiate(
        wasmBytes,
        { wasi_snapshot_preview1: wasi.wasiImport }
    );

    wasi.start(instance);

    // Call exported functions
    const result = instance.exports.kinetic_energy(10.0, 100.0);
    console.log(`Result: ${result}`);
}

runPhysicsWASI();
```

## Cloud Compilation API

For environments without a local Rust toolchain, use the cloud compilation API:

```javascript
// Cloud compilation
async function compileToWasm(dolSource) {
    const response = await fetch('https://compile.univrs.io/api/compile', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            source: dolSource,
            target: 'wasm32-unknown-unknown',
            optimize: true,
        })
    });

    const result = await response.json();

    if (result.success) {
        // Decode base64 WASM
        const wasmBytes = Uint8Array.from(
            atob(result.wasm),
            c => c.charCodeAt(0)
        );
        return wasmBytes;
    } else {
        throw new Error(result.error);
    }
}

// Usage
const dolSource = `
pub fun add(a: i64, b: i64) -> i64 {
    return a + b
}
`;

const wasm = await compileToWasm(dolSource);
const { instance } = await WebAssembly.instantiate(wasm);
console.log(instance.exports.add(2n, 3n)); // 5n
```

## Spirit Runtime

The DOL Spirit runtime provides a high-level API for loading and managing WASM modules:

```javascript
// spirit-runtime.js
class SpiritRuntime {
    constructor() {
        this.spirits = new Map();
        this.memory = new WebAssembly.Memory({ initial: 256 });
    }

    async loadSpirit(name, wasmUrl) {
        const response = await fetch(wasmUrl);
        const wasmBytes = await response.arrayBuffer();

        const imports = this.createImports();
        const { instance, module } = await WebAssembly.instantiate(
            wasmBytes,
            imports
        );

        const spirit = {
            name,
            module,
            instance,
            exports: instance.exports,
        };

        this.spirits.set(name, spirit);
        return spirit;
    }

    createImports() {
        return {
            env: {
                memory: this.memory,
                __print_i64: (v) => console.log(v),
                __print_f64: (v) => console.log(v),
                __print_str: (ptr, len) => {
                    const bytes = new Uint8Array(this.memory.buffer, ptr, len);
                    console.log(new TextDecoder().decode(bytes));
                },
            }
        };
    }

    call(spiritName, funcName, ...args) {
        const spirit = this.spirits.get(spiritName);
        if (!spirit) throw new Error(`Spirit not loaded: ${spiritName}`);

        const func = spirit.exports[funcName];
        if (!func) throw new Error(`Function not found: ${funcName}`);

        return func(...args);
    }
}

// Usage
const runtime = new SpiritRuntime();

await runtime.loadSpirit('physics', '/wasm/physics.wasm');
await runtime.loadSpirit('chemistry', '/wasm/chemistry.wasm');

const energy = runtime.call('physics', 'kinetic_energy', 10.0, 100.0);
const mass = runtime.call('chemistry', 'molar_mass', elementPtr);
```

## Debugging WASM

### Source Maps

```bash
# Build with source maps
vudo build --debug --source-map

# Output
ls target/debug/*.wasm.map
```

### Browser DevTools

1. Open browser DevTools
2. Go to Sources tab
3. Find the WASM file under `wasm://`
4. Set breakpoints in the text format
5. Inspect local variables and memory

### Logging

```dol
// Add logging to DOL code
pub fun debug_kinetic_energy(mass: f64, velocity: f64) -> f64 {
    emit "debug" { message: "Calculating kinetic energy" }
    emit "debug" { mass: mass, velocity: velocity }

    let result = 0.5 * mass * velocity * velocity

    emit "debug" { result: result }
    return result
}
```

## Performance Optimization

### SIMD Support

```dol
// Enable SIMD in Spirit.dol
spirit Physics {
    has wasm: {
        features: ["simd"]
    }
}
```

```dol
// SIMD-optimized vector operations
pub fun dot_product(a: Vec4, b: Vec4) -> f64 {
    // Compiler will use WASM SIMD instructions
    return a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}
```

### Memory Management

```dol
// Pre-allocate buffers for better performance
pub fun batch_kinetic_energy(masses: &[f64], velocities: &[f64], results: &mut [f64]) {
    for i in 0..masses.len() {
        results[i] = 0.5 * masses[i] * velocities[i] * velocities[i]
    }
}
```

## Summary

### Compilation Pipeline

```
DOL → Lexer → Parser → AST → Codegen → Rust → rustc → WASM
```

### Key Commands

```bash
# Build WASM
vudo build --release --target wasm32-unknown-unknown

# Generate Rust code
vudo build --emit rust

# Generate TypeScript bindings
vudo build --emit typescript

# Optimize WASM
wasm-opt -Oz input.wasm -o output.wasm
```

### Browser Loading

```javascript
const wasm = await WebAssembly.instantiate(bytes, imports);
const result = wasm.instance.exports.myFunction(arg1, arg2);
```

### Next Steps

- **[REPL Guide](repl-guide.md)** - Interactive DOL exploration
- **[CLI Guide](cli-guide.md)** - Command-line tools
- **[Spirit Development](spirit-development.md)** - Building Spirits
