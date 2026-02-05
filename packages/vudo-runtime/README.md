# @vudo/runtime

TypeScript/JavaScript runtime host environment for DOL Spirits (WASM modules).

## Overview

`@vudo/runtime` provides a complete host environment for executing compiled DOL programs as WebAssembly modules. The runtime bridges the gap between WASM Spirits and the host JavaScript/TypeScript environment by providing:

- **22 Host Functions**: Complete API for I/O, memory, time, messaging, randomness, effects, and debugging
- **Type-Safe Memory Interface**: Typed access to WASM linear memory with bounds checking
- **Multi-Spirit Sessions**: Coordinate multiple WASM modules in a single session (Séance)
- **Message Passing**: Inter-Spirit communication via message queues
- **Provider Interfaces**: Dependency injection for testing and customization

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

// Type-safe calls with generated interfaces
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
const result = await seance.invoke('calc', 'multiply', [6n, 7n]);
await seance.dismiss();
```

## Architecture

### Core Concepts

| Term | Description |
|------|-------------|
| **Spirit** | A compiled DOL program running as a WASM module |
| **Host Function** | A function provided by the runtime that Spirits can call |
| **Séance** | A session managing multiple Spirit instances with shared message bus |
| **Loa** | A service providing a group of related host functions |
| **Provider** | An interface abstraction for dependency injection (logging, time, etc.) |

### How WASM Spirits Call Host Functions

When a DOL program is compiled to WASM, all host functions are imported from the `"vudo"` namespace:

```wat
(module
  ;; Import host functions
  (import "vudo" "vudo_print" (func $vudo_print (param i32 i32)))
  (import "vudo" "vudo_alloc" (func $vudo_alloc (param i32) (result i32)))
  (import "vudo" "vudo_now" (func $vudo_now (result i64)))

  ;; Spirit code calls host functions
  (func $main
    ;; Allocate memory
    i32.const 256
    call $vudo_alloc

    ;; Print message
    i32.const 0  ;; string pointer
    i32.const 11 ;; string length
    call $vudo_print
  )
)
```

The runtime provides implementations for all 22 host functions, which are linked when the WASM module is instantiated.

### Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│                   Host Environment                    │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐    │
│  │  ILogger   │  │ITimeProvider│ │IWasmMemory │    │
│  └─────┬──────┘  └──────┬─────┘  └──────┬─────┘    │
│        │                 │                │           │
│        └─────────┬───────┴────────────────┘          │
│                  ▼                                    │
│          ┌───────────────┐                           │
│          │ Host Functions│ (22 functions)            │
│          └───────┬───────┘                           │
│                  │                                    │
│                  │ import "vudo"                     │
├──────────────────┼───────────────────────────────────┤
│                  ▼                                    │
│      ┌──────────────────────┐                        │
│      │   Spirit (WASM)      │                        │
│      │  ┌──────────────┐    │                        │
│      │  │ Linear Memory│    │                        │
│      │  └──────────────┘    │                        │
│      │  ┌──────────────┐    │                        │
│      │  │ DOL Code     │    │                        │
│      │  └──────────────┘    │                        │
│      └──────────────────────┘                        │
└──────────────────────────────────────────────────────┘
```

## Host Functions

The runtime provides **22 host functions** organized into 7 categories:

### I/O Functions (4)

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_print` | `(ptr: i32, len: i32) -> void` | Print string without newline |
| `vudo_println` | `(ptr: i32, len: i32) -> void` | Print string with newline |
| `vudo_log` | `(level: i32, ptr: i32, len: i32) -> void` | Structured logging with level |
| `vudo_error` | `(ptr: i32, len: i32) -> void` | Log error message |

### Memory Functions (3)

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_alloc` | `(size: i32) -> i32` | Allocate memory (returns pointer) |
| `vudo_free` | `(ptr: i32, size: i32) -> void` | Free allocated memory |
| `vudo_realloc` | `(ptr: i32, old: i32, new: i32) -> i32` | Reallocate memory |

### Time Functions (3)

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_now` | `() -> i64` | Current timestamp (milliseconds) |
| `vudo_sleep` | `(ms: i32) -> void` | Sleep for duration |
| `vudo_monotonic_now` | `() -> i64` | Monotonic time (nanoseconds) |

### Messaging Functions (5)

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_send` | `(to_ptr: i32, to_len: i32, payload_ptr: i32, payload_len: i32) -> i32` | Send message to Spirit |
| `vudo_recv` | `() -> i32` | Receive message (returns pointer) |
| `vudo_pending` | `() -> i32` | Count pending messages |
| `vudo_broadcast` | `(ptr: i32, len: i32) -> i32` | Broadcast to all Spirits |
| `vudo_free_message` | `(ptr: i32) -> void` | Free received message |

### Random Functions (2)

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_random` | `() -> f64` | Random number [0, 1) |
| `vudo_random_bytes` | `(ptr: i32, len: i32) -> void` | Fill buffer with random bytes |

### Effect Functions (2)

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_emit_effect` | `(id: i32, ptr: i32, len: i32) -> i32` | Emit side effect for host |
| `vudo_subscribe` | `(chan_ptr: i32, chan_len: i32) -> i32` | Subscribe to effect channel |

### Debug Functions (3)

| Function | Signature | Description |
|----------|-----------|-------------|
| `vudo_breakpoint` | `() -> void` | Trigger debugger breakpoint |
| `vudo_assert` | `(cond: i32, ptr: i32, len: i32) -> void` | Assert condition with message |
| `vudo_panic` | `(ptr: i32, len: i32) -> void` | Panic and terminate Spirit |

For detailed documentation of each function, see [HOST-FUNCTIONS.md](./docs/HOST-FUNCTIONS.md).

## Memory Interface

### WasmMemory

The `WasmMemory` class provides type-safe access to WASM linear memory:

```typescript
import { WasmMemory } from '@vudo/runtime/host/memory';

const memory = new WasmMemory(wasmMemory);

// String operations
const str = memory.readString(ptr, len);
const bytesWritten = memory.writeString(ptr, "Hello");

// Typed number access
const i32Value = memory.readI32(ptr);
const i64Value = memory.readI64(ptr); // Returns BigInt
const f64Value = memory.readF64(ptr);

memory.writeI32(ptr, 42);
memory.writeI64(ptr, 1000n);
memory.writeF64(ptr, 3.14159);

// Raw byte operations
const bytes = memory.readBytes(ptr, len);
memory.writeBytes(ptr, new Uint8Array([1, 2, 3]));

// Memory growth
const newPages = memory.grow(10); // Add 10 pages (640KB)
```

### BumpAllocator

The default allocator uses a simple bump-pointer strategy:

```typescript
const ptr = spirit.memory.alloc(256);      // Allocate 256 bytes
spirit.memory.free(ptr);                    // No-op for bump allocator
spirit.memory.reset();                      // Reset allocator to start
```

### Gene Layout System

For structured data access, use Gene layouts generated by the DOL compiler:

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
console.log(point); // { x: 10n, y: 20n }
```

## Provider Interfaces

All host function implementations use provider interfaces for dependency injection.

### IWasmMemory

Interface for memory operations:

```typescript
interface IWasmMemory {
  readonly buffer: ArrayBuffer;
  readonly u8: Uint8Array;
  readonly i32: Int32Array;
  readonly f64: Float64Array;

  decodeString(ptr: number, len: number): string;
  encodeString(str: string): number;
  alloc(size: number): number;
  free(ptr: number, size: number): void;
  realloc(ptr: number, oldSize: number, newSize: number): number;
}
```

### ILogger

Interface for logging operations:

```typescript
interface ILogger {
  log(level: LogLevel, message: string): void;
  debug(message: string): void;
  info(message: string): void;
  warn(message: string): void;
  error(message: string): void;
  print(message: string): void;
  println(message: string): void;
}
```

Default implementation: `ConsoleLogger`

### ITimeProvider

Interface for time operations:

```typescript
interface ITimeProvider {
  now(): bigint;
  monotonicNow(): bigint;
  sleep(ms: number): Promise<void>;
}
```

Default implementation: `SystemTimeProvider`

### IMessageBroker

The `MessageBus` class provides inter-Spirit messaging:

```typescript
const bus = new MessageBus({ debug: true });

bus.register('spirit-a');
bus.register('spirit-b');

bus.send('spirit-a', 'spirit-b', 1, new Uint8Array([1, 2, 3]));

const message = bus.recv('spirit-b', 1);
console.log(message); // { from: 'spirit-a', to: 'spirit-b', ... }
```

### IRandomProvider

Random number generation (uses `crypto.getRandomValues` by default):

```typescript
// vudo_random() uses Math.random() or crypto-secure source
// vudo_random_bytes() uses crypto.getRandomValues()
```

### IEffectHandler

Effects are handled via custom effect handlers:

```typescript
// Standard effect IDs:
const EFFECT_NOOP = 0;
const EFFECT_TERMINATE = 1;
const EFFECT_SPAWN = 2;
const EFFECT_FS_READ = 10;
const EFFECT_FS_WRITE = 11;
const EFFECT_HTTP_GET = 20;
const EFFECT_HTTP_POST = 21;
const EFFECT_DB_QUERY = 30;
```

### IDebugHandler

Debug operations:

```typescript
// vudo_breakpoint() - Triggers debugger if attached
// vudo_assert() - Checks condition, throws on failure
// vudo_panic() - Terminates Spirit with error message
```

For detailed provider documentation, see [PROVIDERS.md](./docs/PROVIDERS.md).

## Usage Examples

### Loading a Spirit

```typescript
import { loadSpirit } from '@vudo/runtime';

// From file path
const spirit = await loadSpirit('./calculator.wasm');

// From bytes
const wasmBytes = await fetch('./calculator.wasm').then(r => r.arrayBuffer());
const spirit = await loadSpirit(wasmBytes);

// With custom options
const spirit = await loadSpirit(wasmBytes, {
  debug: true,
  memory: { initial: 32, maximum: 256 },
});
```

### Calling Spirit Functions

```typescript
// Untyped call
const result = spirit.call('add', [10n, 20n]);

// Type-safe interface
interface Calculator {
  add(a: bigint, b: bigint): bigint;
  multiply(a: bigint, b: bigint): bigint;
}
const calc = spirit.as<Calculator>();
const sum = calc.add(10n, 20n);
```

### Working with Memory

```typescript
// Allocate memory
const ptr = spirit.memory.alloc(256);

// Write data
spirit.memory.encodeString('Hello, World!');

// Read data
const str = spirit.memory.decodeString(ptr);
```

### Multi-Spirit Sessions

```typescript
import { Seance } from '@vudo/runtime';

const seance = new Seance();

// Load multiple Spirits
await seance.summon('calculator', './calculator.wasm');
await seance.summon('logger', './logger.wasm');
await seance.summon('database', './database.wasm');

// Invoke functions
const result = await seance.invoke('calculator', 'add', [1n, 2n]);
await seance.invoke('logger', 'log', ['Calculation complete']);

// Access individual Spirits
const calc = seance.getSpirit('calculator');
if (calc) {
  calc.call('multiply', [6n, 7n]);
}

// Clean up
await seance.dismiss();
```

### Inter-Spirit Messaging

```typescript
// Spirit A sends to Spirit B
// In WASM:
//   vudo_send(target_ptr, target_len, payload_ptr, payload_len)

// Spirit B receives
// In WASM:
//   let msg_ptr = vudo_recv()
//   if msg_ptr != 0:
//     // Process message
//     vudo_free_message(msg_ptr)
```

### Custom Loa (Service Provider)

```typescript
import { createLoa, LoaRegistry } from '@vudo/runtime';

// Create custom HTTP Loa
const httpLoa = createLoa('http', '1.0.0', {
  http_get: (ctx) => async (url: string) => {
    const response = await fetch(url);
    return response.text();
  },
});

// Register with session
const registry = new LoaRegistry();
registry.register(httpLoa);

const seance = new Seance({ loas: registry });
```

## Testing

All host functions support mock providers for testing:

```typescript
import { MockLogger, MockTimeProvider } from '@vudo/runtime/testing';

const mockLogger = new MockLogger();
const mockTime = new MockTimeProvider();

// Configure time
mockTime.setTime(1000n);

// Load Spirit with mocks
const spirit = await loadSpirit(wasmBytes, {
  logger: mockLogger,
  timeProvider: mockTime,
});

// Assert on log calls
expect(mockLogger.messages).toContain('[INFO] Test message');
```

## Integration with DOL

The runtime works seamlessly with DOL-compiled WASM:

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

// Fully type-safe calls
const sum = calc.add(10n, 20n);
```

## API Reference

- [Host Functions](./docs/HOST-FUNCTIONS.md) - Complete documentation of all 22 host functions
- [Providers](./docs/PROVIDERS.md) - Provider interfaces and implementations
- [API Documentation](https://vudo-docs.example.com) - Full API reference

## Performance

- Zero-copy string passing where possible
- Bump allocator for fast allocations
- Typed array views for efficient memory access
- Message passing uses ArrayBuffer for zero-copy transfers

## Security

- All memory accesses are bounds-checked
- UTF-8 validation on string operations
- Effect system for controlled side effects
- Sandboxed WASM execution environment

## License

MIT
