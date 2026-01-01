# @vudo/runtime

JavaScript/TypeScript runtime for DOL Spirits - WASM host environment.

## Installation

```bash
npm install @vudo/runtime
```

## Quick Start

```typescript
import { loadSpirit, Seance } from '@vudo/runtime';

// Load a single Spirit
const spirit = await loadSpirit('./calculator.wasm');
const result = spirit.call('add', [1n, 2n]);
console.log(result); // 3n

// Type-safe calls
interface Calculator {
  add(a: bigint, b: bigint): bigint;
  multiply(a: bigint, b: bigint): bigint;
}
const calc = spirit.as<Calculator>();
const sum = calc.add(10n, 20n); // Type-safe!

// Multi-Spirit session
const seance = new Seance();
await seance.summon('calc', './calculator.wasm');
await seance.summon('logger', './logger.wasm');
await seance.invoke('calc', 'multiply', [6n, 7n]);
await seance.dismiss();
```

## Core Concepts

| Term | Description |
|------|-------------|
| **Spirit** | A compiled DOL program running in WASM |
| **Séance** | Session managing multiple Spirit instances |
| **Loa** | Service providing host functions to Spirits |

## API Reference

### Spirit Loading

```typescript
// From bytes
const spirit = await loadSpirit(wasmBytes);

// From URL/path
const spirit = await loadSpirit('/spirits/math.wasm');

// With options
const spirit = await loadSpirit(wasmBytes, {
  debug: true,
  memory: { initial: 32, maximum: 256 },
});
```

### Séance (Sessions)

```typescript
const seance = new Seance();

await seance.summon('name', source);        // Load Spirit
await seance.invoke('name', 'func', args);  // Call function
seance.getSpirit('name');                   // Get Spirit instance
await seance.release('name');               // Release one Spirit
await seance.dismiss();                     // End session

// Auto-cleanup with helper
await withSeance(async (seance) => {
  await seance.summon('calc', './calc.wasm');
  return seance.invoke('calc', 'add', [1n, 2n]);
});
```

### Loa (Services)

```typescript
import { LoaRegistry, createLoa } from '@vudo/runtime';

// Create custom Loa
const httpLoa = createLoa('http', '1.0.0', {
  http_get: (ctx) => async (url: string) => { /* ... */ },
});

// Register with session
const registry = new LoaRegistry();
registry.register(httpLoa);

const seance = new Seance({ loas: registry });
```

### Memory Management

```typescript
import type { GeneLayout } from '@vudo/runtime';

const PointLayout: GeneLayout = {
  name: 'Point',
  fields: [
    { name: 'x', type: 'i64', offset: 0 },
    { name: 'y', type: 'i64', offset: 8 },
  ],
  size: 16,
  alignment: 8,
};

// Allocate and write
const ptr = spirit.memory.alloc(16);
spirit.memory.writeGene(ptr, { x: 10n, y: 20n }, PointLayout);

// Read back
const point = spirit.memory.readGene(ptr, PointLayout);
```

## Built-in Host Functions

The `coreLoa` provides these functions to all Spirits:

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_print` | `(ptr: i32, len: i32) -> void` | Print string to console |
| `vudo_alloc` | `(size: i32) -> i32` | Allocate memory |
| `vudo_now` | `() -> i64` | Current timestamp (ms) |
| `vudo_random` | `() -> f64` | Random number [0, 1) |

## Integration with DOL

```bash
# Compile DOL to WASM
dol-codegen --wasm calculator.dol -o calculator.wasm

# Generate TypeScript types
dol-codegen --typescript calculator.dol -o calculator.types.ts
```

```typescript
import { loadSpirit } from '@vudo/runtime';
import type { Calculator } from './calculator.types';

const spirit = await loadSpirit('./calculator.wasm');
const calc = spirit.as<Calculator>();
```

## License

MIT
