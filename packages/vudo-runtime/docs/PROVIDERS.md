# Provider Interfaces

Documentation for all provider interfaces in `@vudo/runtime`.

## Overview

Provider interfaces enable dependency injection and testing by abstracting platform-specific implementations. All host functions are implemented using these provider interfaces.

**Benefits**:
- **Testability**: Mock providers for unit tests
- **Flexibility**: Custom implementations for specific environments
- **Portability**: Abstract platform differences (Node.js, browser, Deno)
- **Isolation**: Host functions don't directly depend on global objects

---

## IWasmMemory

Interface for WASM memory access operations.

### Purpose

Provides safe access to WASM linear memory with typed array views and string encoding/decoding capabilities.

### Interface Definition

```typescript
interface IWasmMemory {
  /**
   * Get raw memory buffer
   */
  readonly buffer: ArrayBuffer;

  /**
   * Get Uint8Array view of memory
   */
  readonly u8: Uint8Array;

  /**
   * Get Int32Array view of memory
   */
  readonly i32: Int32Array;

  /**
   * Get Float64Array view of memory
   */
  readonly f64: Float64Array;

  /**
   * Decode a UTF-8 string from memory
   * @param ptr - Pointer to string data
   * @param len - Length in bytes
   * @returns Decoded string
   * @throws Error if pointer is invalid or string is malformed
   */
  decodeString(ptr: number, len: number): string;

  /**
   * Encode a UTF-8 string into memory
   * @param str - String to encode
   * @returns Pointer to encoded string
   */
  encodeString(str: string): number;

  /**
   * Allocate memory
   * @param size - Number of bytes to allocate
   * @returns Pointer to allocated memory (0 on failure)
   */
  alloc(size: number): number;

  /**
   * Free allocated memory
   * @param ptr - Pointer to free
   * @param size - Size of allocation
   */
  free(ptr: number, size: number): void;

  /**
   * Reallocate memory
   * @param ptr - Current pointer
   * @param oldSize - Current size
   * @param newSize - Desired new size
   * @returns New pointer (0 on failure)
   */
  realloc(ptr: number, oldSize: number, newSize: number): number;
}
```

### Default Implementation: WasmMemory

The `WasmMemory` class provides a complete implementation with bounds checking:

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

### Key Features

- **Bounds Checking**: All operations validate pointer and length
- **UTF-8 Validation**: String operations properly handle UTF-8 encoding
- **Typed Access**: Read/write i32, i64, f32, f64 with proper byte order
- **Memory Growth**: Safe memory expansion with error handling
- **Zero-Copy Views**: DataView and TypedArray creation

### Error Handling

```typescript
try {
  const str = memory.readString(ptr, len);
} catch (e) {
  if (e instanceof MemoryError) {
    console.error(`Memory error at ptr=${e.ptr}, len=${e.length}`);
  }
}
```

### Mock Implementation for Testing

```typescript
class MockWasmMemory implements IWasmMemory {
  private data = new ArrayBuffer(65536); // 64KB
  private offset = 0;

  get buffer() { return this.data; }
  get u8() { return new Uint8Array(this.data); }
  get i32() { return new Int32Array(this.data); }
  get f64() { return new Float64Array(this.data); }

  decodeString(ptr: number, len: number): string {
    const bytes = new Uint8Array(this.data, ptr, len);
    return new TextDecoder().decode(bytes);
  }

  encodeString(str: string): number {
    const encoded = new TextEncoder().encode(str);
    const ptr = this.alloc(encoded.length);
    this.u8.set(encoded, ptr);
    return ptr;
  }

  alloc(size: number): number {
    const ptr = this.offset;
    this.offset += size;
    return ptr;
  }

  free(ptr: number, size: number): void {
    // No-op for bump allocator
  }

  realloc(ptr: number, oldSize: number, newSize: number): number {
    const newPtr = this.alloc(newSize);
    this.u8.copyWithin(newPtr, ptr, ptr + Math.min(oldSize, newSize));
    return newPtr;
  }
}
```

### Used By

- `vudo_print`, `vudo_println`, `vudo_log`, `vudo_error`
- `vudo_alloc`, `vudo_free`, `vudo_realloc`
- `vudo_send`, `vudo_recv`, `vudo_broadcast`
- `vudo_random_bytes`
- `vudo_emit_effect`, `vudo_subscribe`
- `vudo_assert`, `vudo_panic`

---

## ILogger

Interface for logging operations.

### Purpose

Abstraction over console/logging systems to enable testing and custom log routing.

### Interface Definition

```typescript
interface ILogger {
  /**
   * Log a message at the specified level
   * @param level - Log severity level
   * @param message - Message to log
   */
  log(level: LogLevel, message: string): void;

  /**
   * Log a debug message
   * @param message - Debug message
   */
  debug(message: string): void;

  /**
   * Log an info message
   * @param message - Info message
   */
  info(message: string): void;

  /**
   * Log a warning message
   * @param message - Warning message
   */
  warn(message: string): void;

  /**
   * Log an error message
   * @param message - Error message
   */
  error(message: string): void;

  /**
   * Print a message without newline
   * @param message - Message to print
   */
  print(message: string): void;

  /**
   * Print a message with newline
   * @param message - Message to print
   */
  println(message: string): void;
}
```

### Log Levels

```typescript
enum LogLevel {
  Debug = 0,  // Development-only messages
  Info  = 1,  // Informational messages
  Warn  = 2,  // Warning conditions
  Error = 3,  // Error conditions
}
```

### Default Implementation: ConsoleLogger

```typescript
import { ConsoleLogger } from '@vudo/runtime/host/interfaces';

const logger = new ConsoleLogger();

logger.debug('Starting initialization');
logger.info('System ready');
logger.warn('Memory usage high');
logger.error('Connection failed');
logger.print('Progress: ');
logger.println('100%');
```

**Output Format**:
```
[DEBUG] Starting initialization
[INFO] System ready
[WARN] Memory usage high
[ERROR] Connection failed
Progress: 100%
```

### Custom Implementation Example

```typescript
class FileLogger implements ILogger {
  private file: string;

  constructor(file: string) {
    this.file = file;
  }

  log(level: LogLevel, message: string): void {
    const timestamp = new Date().toISOString();
    const levelStr = LogLevel[level].toUpperCase();
    const line = `${timestamp} [${levelStr}] ${message}\n`;
    fs.appendFileSync(this.file, line);
  }

  debug(message: string) { this.log(LogLevel.Debug, message); }
  info(message: string) { this.log(LogLevel.Info, message); }
  warn(message: string) { this.log(LogLevel.Warn, message); }
  error(message: string) { this.log(LogLevel.Error, message); }
  print(message: string) { process.stdout.write(message); }
  println(message: string) { console.log(message); }
}
```

### Mock Implementation for Testing

```typescript
class MockLogger implements ILogger {
  public messages: Array<{ level: LogLevel; message: string }> = [];
  public outputs: string[] = [];
  public errors: string[] = [];

  log(level: LogLevel, message: string): void {
    this.messages.push({ level, message });
  }

  debug(message: string) { this.log(LogLevel.Debug, message); }
  info(message: string) { this.log(LogLevel.Info, message); }
  warn(message: string) { this.log(LogLevel.Warn, message); }
  error(message: string) {
    this.log(LogLevel.Error, message);
    this.errors.push(message);
  }

  print(message: string) { this.outputs.push(message); }
  println(message: string) { this.outputs.push(message + '\n'); }

  // Test helpers
  hasMessage(level: LogLevel, text: string): boolean {
    return this.messages.some(m => m.level === level && m.message.includes(text));
  }

  clear(): void {
    this.messages = [];
    this.outputs = [];
    this.errors = [];
  }
}
```

### Testing Example

```typescript
import { MockLogger } from '@vudo/runtime/testing';

const mockLogger = new MockLogger();
const spirit = await loadSpirit(wasmBytes, { logger: mockLogger });

spirit.call('run_test');

// Assert on logged messages
expect(mockLogger.hasMessage(LogLevel.Info, 'Test passed')).toBe(true);
expect(mockLogger.errors).toHaveLength(0);
```

### Used By

- `vudo_print`, `vudo_println`
- `vudo_log`
- `vudo_error`
- `vudo_assert` (on failure)
- `vudo_panic`

---

## ITimeProvider

Interface for time operations.

### Purpose

Abstraction over system time to enable testing and custom time providers (e.g., simulated time).

### Interface Definition

```typescript
interface ITimeProvider {
  /**
   * Get current timestamp in milliseconds since Unix epoch
   * @returns Timestamp in milliseconds
   */
  now(): bigint;

  /**
   * Get monotonic time in nanoseconds
   * @returns Monotonic time in nanoseconds
   */
  monotonicNow(): bigint;

  /**
   * Sleep for specified milliseconds
   * @param ms - Duration in milliseconds
   * @returns Promise that resolves after sleep duration
   */
  sleep(ms: number): Promise<void>;
}
```

### Default Implementation: SystemTimeProvider

```typescript
import { SystemTimeProvider } from '@vudo/runtime/host/interfaces';

const timeProvider = new SystemTimeProvider();

const timestamp = timeProvider.now();
console.log(`Current time: ${timestamp}ms`);

const startTime = timeProvider.monotonicNow();
// ... do work ...
const elapsed = timeProvider.monotonicNow() - startTime;
console.log(`Elapsed: ${Number(elapsed) / 1_000_000}ms`);

await timeProvider.sleep(1000); // Sleep 1 second
```

**Platform Support**:
- **Browser**: Uses `Date.now()` and `performance.now()`
- **Node.js**: Uses `Date.now()` and `process.hrtime.bigint()`
- **Deno**: Uses `Date.now()` and `performance.now()`

### Mock Implementation for Testing

```typescript
class MockTimeProvider implements ITimeProvider {
  private currentTime = 0n;
  private monotonicTime = 0n;

  now(): bigint {
    return this.currentTime;
  }

  monotonicNow(): bigint {
    return this.monotonicTime;
  }

  async sleep(ms: number): Promise<void> {
    this.currentTime += BigInt(ms);
    this.monotonicTime += BigInt(ms) * 1_000_000n; // Convert to nanoseconds
  }

  // Test helpers
  setTime(time: bigint): void {
    this.currentTime = time;
  }

  advance(ms: number): void {
    this.currentTime += BigInt(ms);
    this.monotonicTime += BigInt(ms) * 1_000_000n;
  }

  reset(): void {
    this.currentTime = 0n;
    this.monotonicTime = 0n;
  }
}
```

### Testing Example

```typescript
import { MockTimeProvider } from '@vudo/runtime/testing';

const mockTime = new MockTimeProvider();
mockTime.setTime(1000000n); // Set to 1000 seconds

const spirit = await loadSpirit(wasmBytes, { timeProvider: mockTime });

const timeout = spirit.call('calculate_timeout'); // Returns now() + 5000
expect(timeout).toBe(1005000n);

// Test sleep
await spirit.call('sleep_100ms');
expect(mockTime.now()).toBe(1000100n);
```

### Advanced: Simulated Time

For testing time-dependent behavior:

```typescript
class SimulatedTimeProvider implements ITimeProvider {
  private time = 0n;
  private timers: Array<{ at: bigint; resolve: () => void }> = [];

  now(): bigint {
    return this.time;
  }

  monotonicNow(): bigint {
    return this.time * 1_000_000n; // ms to ns
  }

  async sleep(ms: number): Promise<void> {
    return new Promise(resolve => {
      this.timers.push({
        at: this.time + BigInt(ms),
        resolve
      });
      this.timers.sort((a, b) => Number(a.at - b.at));
    });
  }

  // Advance time and trigger timers
  tick(ms: number): void {
    this.time += BigInt(ms);
    while (this.timers.length > 0 && this.timers[0].at <= this.time) {
      this.timers.shift()!.resolve();
    }
  }
}
```

### Used By

- `vudo_now`
- `vudo_sleep`
- `vudo_monotonic_now`

---

## IMessageBroker

Interface for inter-Spirit messaging.

### Purpose

Enables Spirits to send and receive messages within a Séance session. Messages are queued per-Spirit and can be consumed synchronously.

### Implementation: MessageBus

The `MessageBus` class provides the message broker implementation:

```typescript
import { MessageBus } from '@vudo/runtime';

const bus = new MessageBus({ debug: true });

// Register Spirits
bus.register('spirit-a');
bus.register('spirit-b');

// Send message
const success = bus.send('spirit-a', 'spirit-b', 1, new Uint8Array([1, 2, 3]));
console.log(`Send ${success ? 'succeeded' : 'failed'}`);

// Receive message
const message = bus.recv('spirit-b', 1);
if (message) {
  console.log(`From: ${message.from}`);
  console.log(`Payload: ${message.payload}`);
}

// Check pending
const count = bus.pending('spirit-b');
console.log(`${count} messages pending`);

// Unregister
bus.unregister('spirit-a');
```

### Message Format

```typescript
interface Message {
  /** Source Spirit name */
  from: string;
  /** Destination Spirit name */
  to: string;
  /** Message type/channel identifier */
  channel: number;
  /** Raw payload bytes */
  payload: Uint8Array;
  /** Timestamp when message was sent */
  timestamp: number;
}
```

### API Methods

```typescript
class MessageBus {
  /** Register a Spirit to receive messages */
  register(name: string): void;

  /** Unregister a Spirit */
  unregister(name: string): void;

  /** Check if a Spirit is registered */
  isRegistered(name: string): boolean;

  /** Send a message to a Spirit */
  send(from: string, to: string, channel: number, payload: Uint8Array): boolean;

  /** Receive a message */
  recv(name: string, channel?: number): Message | undefined;

  /** Peek at next message without removing */
  peek(name: string, channel?: number): Message | undefined;

  /** Get number of pending messages */
  pending(name: string, channel?: number): number;

  /** Add a message handler */
  onMessage(name: string, handler: (message: Message) => void): void;

  /** Clear messages for a Spirit */
  clear(name: string): void;

  /** Clear all messages */
  clearAll(): void;

  /** Get all registered Spirit names */
  spirits(): string[];
}
```

### Channels

Messages can be filtered by channel (similar to topics or event types):

```typescript
// Channel 0 = any channel (default)
// Channel 1+ = specific channels

bus.send('a', 'b', 1, payload1); // Send on channel 1
bus.send('a', 'b', 2, payload2); // Send on channel 2

// Receive from any channel
const msg1 = bus.recv('b', 0);

// Receive from specific channel
const msg2 = bus.recv('b', 2);
```

### Integration with Séance

The MessageBus is automatically integrated with Séance:

```typescript
const seance = new Seance();

await seance.summon('worker-1', './worker.wasm');
await seance.summon('worker-2', './worker.wasm');

// Spirits can now send messages to each other
// using vudo_send() and vudo_recv()
```

### Testing Example

```typescript
const bus = new MessageBus();
bus.register('sender');
bus.register('receiver');

const spirit = await loadSpirit(wasmBytes, { messageBus: bus });

// Spirit calls vudo_send()
spirit.call('send_message');

// Verify message was delivered
expect(bus.pending('receiver')).toBe(1);

const msg = bus.recv('receiver');
expect(msg?.from).toBe('sender');
```

### Used By

- `vudo_send`
- `vudo_recv`
- `vudo_pending`
- `vudo_broadcast`
- `vudo_free_message`

---

## IRandomProvider

Interface for random number generation.

### Purpose

Provides cryptographically secure random numbers for Spirits.

### Default Implementation

The runtime uses platform-specific secure random sources:

**Browser**:
```typescript
function vudo_random(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0] / 0xFFFFFFFF;
}

function vudo_random_bytes(ptr: number, len: number): void {
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  crypto.getRandomValues(bytes);
}
```

**Node.js**:
```typescript
import crypto from 'crypto';

function vudo_random(): number {
  return crypto.randomBytes(4).readUInt32LE() / 0xFFFFFFFF;
}

function vudo_random_bytes(ptr: number, len: number): void {
  const bytes = crypto.randomBytes(len);
  new Uint8Array(memory.buffer, ptr, len).set(bytes);
}
```

### Mock Implementation for Testing

```typescript
class MockRandomProvider {
  private nextValue = 0.5;
  private bytesSequence: number[] = [];

  setNextValue(value: number): void {
    this.nextValue = Math.max(0, Math.min(value, 0.999999));
  }

  setBytesSequence(bytes: number[]): void {
    this.bytesSequence = bytes;
  }

  random(): number {
    return this.nextValue;
  }

  randomBytes(ptr: number, len: number, memory: IWasmMemory): void {
    const view = new Uint8Array(memory.buffer, ptr, len);
    for (let i = 0; i < len; i++) {
      view[i] = this.bytesSequence[i % this.bytesSequence.length] || 0;
    }
  }
}
```

### Testing Example

```typescript
const mockRandom = new MockRandomProvider();
mockRandom.setNextValue(0.5);
mockRandom.setBytesSequence([0xDE, 0xAD, 0xBE, 0xEF]);

const spirit = await loadSpirit(wasmBytes, { randomProvider: mockRandom });

const dice = spirit.call('roll_dice'); // Uses vudo_random()
expect(dice).toBe(4); // floor(0.5 * 6) + 1

const uuidPtr = spirit.call('generate_uuid'); // Uses vudo_random_bytes()
const bytes = memory.readBytes(uuidPtr, 4);
expect(bytes).toEqual(new Uint8Array([0xDE, 0xAD, 0xBE, 0xEF]));
```

### Security Considerations

- **Always use cryptographic random sources in production**
- Mock random providers are for testing only
- Predictable random values are a security vulnerability
- Use `crypto.getRandomValues()` or `crypto.randomBytes()`

### Used By

- `vudo_random`
- `vudo_random_bytes`

---

## IEffectHandler

Interface for handling side effects.

### Purpose

Effects provide a controlled way for Spirits to request side effects (file I/O, HTTP, database, etc.) without direct access.

### Standard Effect IDs

```typescript
enum EffectId {
  Noop = 0,           // No operation
  Terminate = 1,      // Terminate Spirit
  Spawn = 2,          // Spawn new Spirit
  FsRead = 10,        // Read file
  FsWrite = 11,       // Write file
  FsDelete = 12,      // Delete file
  FsList = 13,        // List directory
  HttpGet = 20,       // HTTP GET request
  HttpPost = 21,      // HTTP POST request
  HttpPut = 22,       // HTTP PUT request
  HttpDelete = 23,    // HTTP DELETE request
  DbQuery = 30,       // Database query
  DbInsert = 31,      // Database insert
  DbUpdate = 32,      // Database update
  DbDelete = 33,      // Database delete
}
```

### Custom Implementation Example

```typescript
class EffectHandler {
  async handle(effectId: number, payload: Uint8Array): Promise<number> {
    switch (effectId) {
      case EffectId.FsRead:
        return this.handleFsRead(payload);

      case EffectId.HttpGet:
        return this.handleHttpGet(payload);

      case EffectId.DbQuery:
        return this.handleDbQuery(payload);

      default:
        console.warn(`Unknown effect ID: ${effectId}`);
        return ResultCode.Error;
    }
  }

  private async handleFsRead(payload: Uint8Array): Promise<number> {
    const path = new TextDecoder().decode(payload);

    try {
      const content = await fs.promises.readFile(path);
      // Store result for Spirit to retrieve
      this.storeResult(content);
      return ResultCode.Ok;
    } catch (error) {
      console.error(`File read error: ${error}`);
      return ResultCode.NotFound;
    }
  }

  private async handleHttpGet(payload: Uint8Array): Promise<number> {
    const url = new TextDecoder().decode(payload);

    try {
      const response = await fetch(url);
      const data = await response.arrayBuffer();
      this.storeResult(new Uint8Array(data));
      return ResultCode.Ok;
    } catch (error) {
      console.error(`HTTP error: ${error}`);
      return ResultCode.Error;
    }
  }
}
```

### Effect Channels

Spirits can subscribe to effect channels to receive notifications:

```typescript
class EffectHandler {
  private channels = new Map<string, Set<string>>();

  subscribe(spiritName: string, channel: string): number {
    if (!this.channels.has(channel)) {
      this.channels.set(channel, new Set());
    }
    this.channels.get(channel)!.add(spiritName);
    return ResultCode.Ok;
  }

  notify(channel: string, payload: Uint8Array): void {
    const subscribers = this.channels.get(channel);
    if (subscribers) {
      for (const spirit of subscribers) {
        this.sendMessage(spirit, channel, payload);
      }
    }
  }
}
```

### Mock Implementation for Testing

```typescript
class MockEffectHandler {
  public effects: Array<{ id: number; payload: string }> = [];

  async handle(effectId: number, payload: Uint8Array): Promise<number> {
    const payloadStr = new TextDecoder().decode(payload);
    this.effects.push({ id: effectId, payload: payloadStr });
    return ResultCode.Ok;
  }

  // Test helpers
  hasEffect(id: number, payload: string): boolean {
    return this.effects.some(e => e.id === id && e.payload.includes(payload));
  }

  clear(): void {
    this.effects = [];
  }
}
```

### Testing Example

```typescript
const effectHandler = new MockEffectHandler();
const spirit = await loadSpirit(wasmBytes, { effectHandler });

spirit.call('read_config'); // Calls vudo_emit_effect(EffectId.FsRead, ...)

expect(effectHandler.hasEffect(EffectId.FsRead, 'config.json')).toBe(true);
```

### Used By

- `vudo_emit_effect`
- `vudo_subscribe`

---

## IDebugHandler

Interface for debug operations.

### Purpose

Provides debugging capabilities like breakpoints, assertions, and panics.

### Default Implementation

```typescript
class DebugHandler {
  breakpoint(): void {
    if (typeof process !== 'undefined' && process.env.NODE_ENV === 'development') {
      debugger; // Triggers debugger
    }
  }

  assert(condition: boolean, message: string): void {
    if (!condition) {
      console.error(`[ASSERTION FAILED] ${message}`);
      throw new Error(`Assertion failed: ${message}`);
    }
  }

  panic(message: string): never {
    console.error(`[PANIC] ${message}`);
    throw new Error(`Spirit panicked: ${message}`);
  }
}
```

### Mock Implementation for Testing

```typescript
class MockDebugHandler {
  public breakpoints = 0;
  public assertions: Array<{ condition: boolean; message: string }> = [];
  public panics: string[] = [];

  breakpoint(): void {
    this.breakpoints++;
  }

  assert(condition: boolean, message: string): void {
    this.assertions.push({ condition, message });
    if (!condition) {
      throw new Error(`Assertion failed: ${message}`);
    }
  }

  panic(message: string): never {
    this.panics.push(message);
    throw new Error(`Panic: ${message}`);
  }

  // Test helpers
  clear(): void {
    this.breakpoints = 0;
    this.assertions = [];
    this.panics = [];
  }
}
```

### Testing Example

```typescript
const debugHandler = new MockDebugHandler();
const spirit = await loadSpirit(wasmBytes, { debugHandler });

// This should succeed
spirit.call('test_with_assertion');
expect(debugHandler.assertions[0].condition).toBe(true);

// This should panic
expect(() => spirit.call('fail_critical')).toThrow('Critical error');
expect(debugHandler.panics).toContain('Critical error');
```

### Used By

- `vudo_breakpoint`
- `vudo_assert`
- `vudo_panic`

---

## Provider Configuration

### Loading a Spirit with Custom Providers

```typescript
import { loadSpirit } from '@vudo/runtime';
import {
  MockLogger,
  MockTimeProvider,
  MockRandomProvider
} from '@vudo/runtime/testing';

const spirit = await loadSpirit(wasmBytes, {
  logger: new MockLogger(),
  timeProvider: new MockTimeProvider(),
  randomProvider: new MockRandomProvider(),
  messageBus: new MessageBus(),
  effectHandler: new MockEffectHandler(),
  debugHandler: new MockDebugHandler(),
});
```

### Provider Registry

For complex scenarios, use a provider registry:

```typescript
class ProviderRegistry {
  private providers = new Map<string, any>();

  register<T>(name: string, provider: T): void {
    this.providers.set(name, provider);
  }

  get<T>(name: string): T | undefined {
    return this.providers.get(name) as T | undefined;
  }

  buildImports(memory: WebAssembly.Memory): WebAssembly.Imports {
    const logger = this.get<ILogger>('logger') || new ConsoleLogger();
    const timeProvider = this.get<ITimeProvider>('time') || new SystemTimeProvider();
    // ... build import object
  }
}
```

---

## Testing Best Practices

### 1. Use Mock Providers

Always use mock providers in unit tests for deterministic behavior:

```typescript
describe('Spirit tests', () => {
  let mockLogger: MockLogger;
  let mockTime: MockTimeProvider;
  let spirit: SpiritInstance;

  beforeEach(async () => {
    mockLogger = new MockLogger();
    mockTime = new MockTimeProvider();

    spirit = await loadSpirit(wasmBytes, {
      logger: mockLogger,
      timeProvider: mockTime,
    });
  });

  it('should log startup message', () => {
    spirit.call('initialize');
    expect(mockLogger.hasMessage(LogLevel.Info, 'Started')).toBe(true);
  });

  it('should calculate timeout', () => {
    mockTime.setTime(1000n);
    const timeout = spirit.call('get_timeout');
    expect(timeout).toBe(6000n); // 1000 + 5000
  });
});
```

### 2. Verify Provider Interactions

```typescript
it('should allocate and free memory', () => {
  const spy = jest.spyOn(spirit.memory, 'alloc');

  spirit.call('allocate_buffer');

  expect(spy).toHaveBeenCalledWith(256);
});
```

### 3. Test Error Conditions

```typescript
it('should handle allocation failure', () => {
  // Force allocation to fail
  jest.spyOn(spirit.memory, 'alloc').mockReturnValue(0);

  expect(() => spirit.call('allocate_buffer')).toThrow('Out of memory');
});
```

### 4. Isolate Tests

```typescript
afterEach(() => {
  mockLogger.clear();
  mockTime.reset();
  mockRandom.clear();
});
```

---

## See Also

- [README](../README.md) - Runtime overview and quick start
- [HOST-FUNCTIONS](./HOST-FUNCTIONS.md) - Complete host function documentation
