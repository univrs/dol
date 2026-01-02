# @vudo/runtime - TypeScript/JavaScript WASM Host Package

## Overview

`@vudo/runtime` is a TypeScript/JavaScript package that hosts DOL-compiled WASM modules (Spirits) and provides type-safe bindings for interacting with them.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         @vudo/runtime                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │
│  │   Spirit    │  │   Séance    │  │    Loa      │  │  Memory    │  │
│  │   Loader    │  │   Manager   │  │  Registry   │  │  Manager   │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │
│         │                │                │               │          │
│         └────────────────┴────────────────┴───────────────┘          │
│                                  │                                   │
│                    ┌─────────────┴─────────────┐                     │
│                    │     WASM Instance Pool     │                    │
│                    └───────────────────────────┘                     │
│                                  │                                   │
├──────────────────────────────────┼──────────────────────────────────┤
│                    ┌─────────────┴─────────────┐                     │
│                    │   WebAssembly Runtime      │                    │
│                    │   (Browser / Node.js)      │                    │
│                    └───────────────────────────┘                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Concepts (from HyphalNetwork)

| VUDO Term | Description | Implementation |
|-----------|-------------|----------------|
| **Spirit** | Compiled DOL WASM module | `Spirit` class wrapping WASM instance |
| **Séance** | Session/execution context | `Seance` class managing Spirit lifecycle |
| **Loa** | Service interface | `Loa` interface for Spirit capabilities |
| **Ghost** | Composition of Spirits | `Ghost` class for Spirit orchestration |

## Package Structure

```
packages/
└── vudo-runtime/
    ├── package.json
    ├── tsconfig.json
    ├── src/
    │   ├── index.ts              # Main exports
    │   ├── spirit.ts             # Spirit loader and wrapper
    │   ├── seance.ts             # Session management
    │   ├── loa.ts                # Service registry
    │   ├── memory.ts             # WASM memory management
    │   ├── types.ts              # Core type definitions
    │   └── utils/
    │       ├── wasm-loader.ts    # WASM loading utilities
    │       └── type-bridge.ts    # DOL ↔ JS type conversion
    ├── tests/
    │   ├── spirit.test.ts
    │   ├── seance.test.ts
    │   └── integration.test.ts
    └── examples/
        ├── basic-spirit.ts
        └── multi-spirit.ts
```

## API Design

### 1. Spirit Loader

```typescript
import { Spirit, loadSpirit } from '@vudo/runtime';

// Load a Spirit from WASM bytes
const spirit = await loadSpirit(wasmBytes);

// Or from URL/path
const spirit = await Spirit.load('/spirits/calculator.wasm');

// Call exported functions
const result = await spirit.call('add', [1, 2]);
// => 3

// Type-safe calls with generated types
import type { Calculator } from './generated/calculator.types';
const calc = spirit.as<Calculator>();
const sum = await calc.add(1, 2);
```

### 2. Séance (Session Management)

```typescript
import { Seance } from '@vudo/runtime';

// Create a session
const seance = new Seance();

// Load multiple Spirits into session
await seance.summon('calc', '/spirits/calculator.wasm');
await seance.summon('logger', '/spirits/logger.wasm');

// Spirits can interact within session
const result = await seance.invoke('calc', 'add', [5, 3]);
await seance.invoke('logger', 'log', [`Result: ${result}`]);

// Session cleanup
await seance.dismiss();
```

### 3. Loa (Service Registry)

```typescript
import { Loa, LoaRegistry } from '@vudo/runtime';

// Define a Loa (service interface)
const logLoa: Loa = {
  name: 'logging',
  version: '1.0.0',
  capabilities: ['log', 'error', 'debug'],
  provides: (imports) => ({
    log: (msg: string) => console.log(`[Spirit] ${msg}`),
    error: (msg: string) => console.error(`[Spirit] ${msg}`),
    debug: (msg: string) => console.debug(`[Spirit] ${msg}`),
  }),
};

// Register Loa
const registry = new LoaRegistry();
registry.register(logLoa);

// Spirits can request Loa capabilities
const spirit = await loadSpirit(wasmBytes, { loas: registry });
```

### 4. Memory Manager

```typescript
import { MemoryManager } from '@vudo/runtime';

// Create memory manager for a Spirit
const memory = new MemoryManager(spirit.memory);

// Read/write DOL types from WASM memory
const point = memory.readGene<Point>(ptr, PointLayout);
memory.writeGene(ptr, { x: 10, y: 20 }, PointLayout);

// Allocate memory for passing complex types
const ptr = memory.alloc(PointLayout.size);
memory.writeGene(ptr, myPoint, PointLayout);
const result = await spirit.call('processPoint', [ptr]);
memory.free(ptr);
```

## Type Bridge (DOL ↔ JavaScript)

Generated from DOL source, matching TypeScript codegen:

```typescript
// DOL source:
// gene Point { x: i64, y: i64 }
// fun distance(p1: Point, p2: Point) -> f64

// Generated types (from dol-codegen --typescript):
export interface Point {
  x: bigint;  // i64 → bigint
  y: bigint;
}

// Generated layout (for memory access):
export const PointLayout = {
  size: 16,  // 8 bytes per i64
  alignment: 8,
  fields: {
    x: { offset: 0, type: 'i64' },
    y: { offset: 8, type: 'i64' },
  },
};
```

## Implementation Phases

### Phase 1: Core Spirit Loading (Week 1)

- [ ] Project setup (package.json, tsconfig, build)
- [ ] `Spirit` class with WASM instantiation
- [ ] Basic function calls (primitives only)
- [ ] Error handling and validation
- [ ] Unit tests for Spirit loading

### Phase 2: Memory Management (Week 2)

- [ ] `MemoryManager` for WASM linear memory
- [ ] Gene layout reading/writing
- [ ] Bump allocator bridge (for DOL's alloc)
- [ ] String passing (pointer + length)
- [ ] Tests for memory operations

### Phase 3: Session & Services (Week 3)

- [ ] `Seance` class for session management
- [ ] Multi-Spirit coordination
- [ ] `Loa` interface and `LoaRegistry`
- [ ] Host function injection
- [ ] Integration tests

### Phase 4: Developer Experience (Week 4)

- [ ] Type generation integration (dol-codegen → types)
- [ ] CLI tool: `vudo run <spirit.wasm>`
- [ ] Browser bundle (ESM + UMD)
- [ ] Documentation and examples
- [ ] npm publish as `@vudo/runtime`

## Dependencies

```json
{
  "name": "@vudo/runtime",
  "version": "0.1.0",
  "type": "module",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "dependencies": {},
  "devDependencies": {
    "typescript": "^5.3",
    "vitest": "^1.0",
    "tsup": "^8.0"
  },
  "peerDependencies": {}
}
```

Note: No runtime dependencies - uses built-in WebAssembly APIs.

## Integration with DOL Toolchain

```bash
# Compile DOL to WASM
dol-codegen --wasm --output calculator.wasm calculator.dol

# Generate TypeScript types
dol-codegen --typescript --output calculator.types.ts calculator.dol

# Use in application
```

```typescript
import { loadSpirit } from '@vudo/runtime';
import type { Calculator } from './calculator.types';
import wasmBytes from './calculator.wasm?binary';

const calc = await loadSpirit(wasmBytes);
const typed = calc.as<Calculator>();
const result = await typed.add(40, 2);  // => 42n (bigint)
```

## Success Criteria

- [ ] Load and execute DOL-compiled WASM in Node.js
- [ ] Load and execute DOL-compiled WASM in browsers
- [ ] Type-safe function calls matching DOL signatures
- [ ] Memory management for gene instances
- [ ] Multi-Spirit sessions with shared state
- [ ] Host function injection (Loa services)
- [ ] Published to npm as `@vudo/runtime`
- [ ] Documentation with examples

## References

- [WASM Spec](https://webassembly.github.io/spec/)
- [DOL WASM Backend](../src/wasm/mod.rs)
- [TypeScript Codegen](../src/codegen/typescript.rs)
- [HyphalNetwork Architecture](./HyphalNetwork.md)
