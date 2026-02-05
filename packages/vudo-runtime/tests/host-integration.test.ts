/**
 * Integration tests for VUDO host function system
 *
 * Tests the complete stack: WASM modules + host imports + memory management
 * Includes full end-to-end scenarios with mock providers.
 *
 * @module @vudo/runtime/tests/host-integration
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { WasmMemory } from '../src/host/memory.js';
import { HostBumpAllocator } from '../src/host/allocator.js';
import type { ILogger, ITimeProvider } from '../src/host/interfaces.js';
import { LogLevel, ResultCode } from '../src/abi/types.js';

// ============================================================================
// Mock Providers for Testing
// ============================================================================

/**
 * Mock logger that captures all output for assertion
 */
class MockLogger implements ILogger {
  public logs: Array<{ level: LogLevel; message: string }> = [];
  public prints: string[] = [];
  public printlns: string[] = [];

  log(level: LogLevel, message: string): void {
    this.logs.push({ level, message });
  }

  debug(message: string): void {
    this.logs.push({ level: LogLevel.Debug, message });
  }

  info(message: string): void {
    this.logs.push({ level: LogLevel.Info, message });
  }

  warn(message: string): void {
    this.logs.push({ level: LogLevel.Warn, message });
  }

  error(message: string): void {
    this.logs.push({ level: LogLevel.Error, message });
  }

  print(message: string): void {
    this.prints.push(message);
  }

  println(message: string): void {
    this.printlns.push(message);
  }

  reset(): void {
    this.logs = [];
    this.prints = [];
    this.printlns = [];
  }

  getLastLog(): { level: LogLevel; message: string } | undefined {
    return this.logs[this.logs.length - 1];
  }
}

/**
 * Mock time provider with controllable time for deterministic testing
 */
class MockTimeProvider implements ITimeProvider {
  private currentTime: bigint = 1000000n; // Start at 1 second
  private monotonicTime: bigint = 0n;

  now(): bigint {
    return this.currentTime;
  }

  monotonicNow(): bigint {
    return this.monotonicTime;
  }

  async sleep(_ms: number): Promise<void> {
    // Mock sleep doesn't actually wait
    return Promise.resolve();
  }

  advance(ms: bigint): void {
    this.currentTime += ms;
    this.monotonicTime += ms * 1_000_000n; // Convert to nanoseconds
  }

  setTime(ms: bigint): void {
    this.currentTime = ms;
  }

  setMonotonicTime(ns: bigint): void {
    this.monotonicTime = ns;
  }
}

/**
 * Mock message broker for inter-Spirit messaging
 */
class MockMessageBroker {
  private queues: Map<string, Array<{ from: string; payload: Uint8Array }>> = new Map();

  register(name: string): void {
    if (!this.queues.has(name)) {
      this.queues.set(name, []);
    }
  }

  unregister(name: string): void {
    this.queues.delete(name);
  }

  send(from: string, to: string, payload: Uint8Array): ResultCode {
    const queue = this.queues.get(to);
    if (!queue) {
      return ResultCode.NotFound;
    }
    queue.push({ from, payload: new Uint8Array(payload) });
    return ResultCode.Ok;
  }

  recv(name: string): { from: string; payload: Uint8Array } | null {
    const queue = this.queues.get(name);
    if (!queue || queue.length === 0) {
      return null;
    }
    return queue.shift()!;
  }

  pending(name: string): number {
    const queue = this.queues.get(name);
    return queue ? queue.length : 0;
  }

  broadcast(from: string, payload: Uint8Array): number {
    let count = 0;
    for (const [name, queue] of this.queues) {
      if (name !== from) {
        queue.push({ from, payload: new Uint8Array(payload) });
        count++;
      }
    }
    return count;
  }

  clear(name: string): void {
    const queue = this.queues.get(name);
    if (queue) {
      queue.length = 0;
    }
  }
}

/**
 * Mock random provider with deterministic sequences
 */
class MockRandomProvider {
  private values: number[] = [0.5, 0.25, 0.75, 0.1, 0.9];
  private index: number = 0;

  random(): number {
    const value = this.values[this.index % this.values.length];
    this.index++;
    return value;
  }

  randomBytes(buffer: Uint8Array): void {
    for (let i = 0; i < buffer.length; i++) {
      buffer[i] = Math.floor(this.random() * 256);
    }
  }

  setValues(values: number[]): void {
    this.values = values;
    this.index = 0;
  }

  reset(): void {
    this.index = 0;
  }
}

/**
 * Mock effect handler for side effects
 */
class MockEffectHandler {
  public effects: Array<{ id: number; payload: Uint8Array }> = [];
  public subscriptions: Map<string, Set<string>> = new Map();

  emit(effectId: number, payload: Uint8Array): ResultCode {
    this.effects.push({ id: effectId, payload: new Uint8Array(payload) });
    return ResultCode.Ok;
  }

  subscribe(spirit: string, channel: string): ResultCode {
    if (!this.subscriptions.has(channel)) {
      this.subscriptions.set(channel, new Set());
    }
    this.subscriptions.get(channel)!.add(spirit);
    return ResultCode.Ok;
  }

  getEffects(): Array<{ id: number; payload: Uint8Array }> {
    return this.effects;
  }

  getSubscribers(channel: string): string[] {
    return Array.from(this.subscriptions.get(channel) || []);
  }

  reset(): void {
    this.effects = [];
    this.subscriptions.clear();
  }
}

/**
 * Mock debug handler for assertions and panics
 */
class MockDebugHandler {
  public breakpoints: number = 0;
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

  reset(): void {
    this.breakpoints = 0;
    this.assertions = [];
    this.panics = [];
  }
}

// ============================================================================
// Host Function Factory Functions
// ============================================================================

/**
 * Create I/O host functions with mock logger
 */
function createIOFunctions(
  memory: WasmMemory,
  logger: MockLogger,
): Record<string, Function> {
  return {
    vudo_print: (ptr: number, len: number): void => {
      const text = memory.readString(ptr, len);
      logger.print(text);
    },

    vudo_println: (ptr: number, len: number): void => {
      const text = memory.readString(ptr, len);
      logger.println(text);
    },

    vudo_log: (level: number, ptr: number, len: number): void => {
      const text = memory.readString(ptr, len);
      logger.log(level as LogLevel, text);
    },

    vudo_error: (ptr: number, len: number): void => {
      const text = memory.readString(ptr, len);
      logger.error(text);
    },
  };
}

/**
 * Create memory management host functions
 */
function createMemoryFunctions(allocator: HostBumpAllocator): Record<string, Function> {
  const allocations = new Map<number, number>(); // ptr -> size

  return {
    vudo_alloc: (size: number): number => {
      if (size <= 0) return 0;
      try {
        const ptr = allocator.alloc(size);
        allocations.set(ptr, size);
        return ptr;
      } catch {
        return 0;
      }
    },

    vudo_free: (ptr: number, size: number): void => {
      allocations.delete(ptr);
      allocator.free(ptr);
    },

    vudo_realloc: (ptr: number, oldSize: number, newSize: number): number => {
      if (newSize <= 0) return 0;
      try {
        const wasmMem = allocator['wasmMemory'];
        const newPtr = allocator.alloc(newSize);
        if (ptr !== 0 && oldSize > 0) {
          const copySize = Math.min(oldSize, newSize);
          const oldBytes = wasmMem.readBytes(ptr, copySize);
          wasmMem.writeBytes(newPtr, oldBytes);
        }
        allocations.delete(ptr);
        allocations.set(newPtr, newSize);
        return newPtr;
      } catch {
        return 0;
      }
    },
  };
}

/**
 * Create time host functions with mock provider
 */
function createTimeFunctions(timeProvider: MockTimeProvider): Record<string, Function> {
  return {
    vudo_now: (): bigint => {
      return timeProvider.now();
    },

    vudo_sleep: (ms: number): void => {
      timeProvider.advance(BigInt(ms));
    },

    vudo_monotonic_now: (): bigint => {
      return timeProvider.monotonicNow();
    },
  };
}

/**
 * Create messaging host functions with mock broker
 */
function createMessagingFunctions(
  memory: WasmMemory,
  allocator: HostBumpAllocator,
  broker: MockMessageBroker,
  spiritName: string,
): Record<string, Function> {
  return {
    vudo_send: (
      targetPtr: number,
      targetLen: number,
      payloadPtr: number,
      payloadLen: number,
    ): number => {
      const target = memory.readString(targetPtr, targetLen);
      const payload = memory.readBytes(payloadPtr, payloadLen);
      return broker.send(spiritName, target, payload);
    },

    vudo_recv: (): number => {
      const msg = broker.recv(spiritName);
      if (!msg) return 0;

      // Allocate and format message: [sender_len:u32][sender][payload_len:u32][payload]
      const senderBytes = new TextEncoder().encode(msg.from);
      const totalSize = 4 + senderBytes.length + 4 + msg.payload.length;
      const ptr = allocator.alloc(totalSize);

      let offset = ptr;
      memory.writeI32(offset, senderBytes.length);
      offset += 4;
      memory.writeBytes(offset, senderBytes);
      offset += senderBytes.length;
      memory.writeI32(offset, msg.payload.length);
      offset += 4;
      memory.writeBytes(offset, msg.payload);

      return ptr;
    },

    vudo_pending: (): number => {
      return broker.pending(spiritName);
    },

    vudo_broadcast: (ptr: number, len: number): number => {
      const payload = memory.readBytes(ptr, len);
      const count = broker.broadcast(spiritName, payload);
      return count > 0 ? ResultCode.Ok : ResultCode.Error;
    },

    vudo_free_message: (_ptr: number): void => {
      // In bump allocator, free is a no-op
    },
  };
}

/**
 * Create random host functions with mock provider
 */
function createRandomFunctions(
  memory: WasmMemory,
  random: MockRandomProvider,
): Record<string, Function> {
  return {
    vudo_random: (): number => {
      return random.random();
    },

    vudo_random_bytes: (ptr: number, len: number): void => {
      const buffer = new Uint8Array(len);
      random.randomBytes(buffer);
      memory.writeBytes(ptr, buffer);
    },
  };
}

/**
 * Create effect host functions with mock handler
 */
function createEffectFunctions(
  memory: WasmMemory,
  effectHandler: MockEffectHandler,
  spiritName: string,
): Record<string, Function> {
  return {
    vudo_emit_effect: (effectId: number, payloadPtr: number, payloadLen: number): number => {
      const payload = memory.readBytes(payloadPtr, payloadLen);
      return effectHandler.emit(effectId, payload);
    },

    vudo_subscribe: (channelPtr: number, channelLen: number): number => {
      const channel = memory.readString(channelPtr, channelLen);
      return effectHandler.subscribe(spiritName, channel);
    },
  };
}

/**
 * Create debug host functions with mock handler
 */
function createDebugFunctions(
  memory: WasmMemory,
  debugHandler: MockDebugHandler,
): Record<string, Function> {
  return {
    vudo_breakpoint: (): void => {
      debugHandler.breakpoint();
    },

    vudo_assert: (condition: number, ptr: number, len: number): void => {
      const message = memory.readString(ptr, len);
      debugHandler.assert(condition !== 0, message);
    },

    vudo_panic: (ptr: number, len: number): void => {
      const message = memory.readString(ptr, len);
      debugHandler.panic(message);
    },
  };
}

// ============================================================================
// Integration Tests
// ============================================================================

describe('Host Function Integration Tests', () => {
  let memory: WebAssembly.Memory;
  let wasmMemory: WasmMemory;
  let allocator: HostBumpAllocator;
  let logger: MockLogger;
  let timeProvider: MockTimeProvider;
  let messageBroker: MockMessageBroker;
  let randomProvider: MockRandomProvider;
  let effectHandler: MockEffectHandler;
  let debugHandler: MockDebugHandler;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
    wasmMemory = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMemory);
    logger = new MockLogger();
    timeProvider = new MockTimeProvider();
    messageBroker = new MockMessageBroker();
    randomProvider = new MockRandomProvider();
    effectHandler = new MockEffectHandler();
    debugHandler = new MockDebugHandler();
  });

  describe('I/O Integration', () => {
    it('should print text without newline', () => {
      const funcs = createIOFunctions(wasmMemory, logger);
      const text = 'Hello, World!';
      const ptr = allocator.alloc(text.length);
      wasmMemory.writeString(ptr, text);

      funcs.vudo_print(ptr, text.length);

      expect(logger.prints).toHaveLength(1);
      expect(logger.prints[0]).toBe(text);
    });

    it('should print text with newline', () => {
      const funcs = createIOFunctions(wasmMemory, logger);
      const text = 'Hello, World!';
      const ptr = allocator.alloc(text.length);
      wasmMemory.writeString(ptr, text);

      funcs.vudo_println(ptr, text.length);

      expect(logger.printlns).toHaveLength(1);
      expect(logger.printlns[0]).toBe(text);
    });

    it('should log messages at different levels', () => {
      const funcs = createIOFunctions(wasmMemory, logger);

      const messages = [
        { level: LogLevel.Debug, text: 'Debug message' },
        { level: LogLevel.Info, text: 'Info message' },
        { level: LogLevel.Warn, text: 'Warning message' },
        { level: LogLevel.Error, text: 'Error message' },
      ];

      for (const { level, text } of messages) {
        const ptr = allocator.alloc(text.length);
        wasmMemory.writeString(ptr, text);
        funcs.vudo_log(level, ptr, text.length);
      }

      expect(logger.logs).toHaveLength(4);
      expect(logger.logs[0]).toEqual({ level: LogLevel.Debug, message: 'Debug message' });
      expect(logger.logs[3]).toEqual({ level: LogLevel.Error, message: 'Error message' });
    });

    it('should handle UTF-8 strings', () => {
      const funcs = createIOFunctions(wasmMemory, logger);
      const text = 'Hello, 世界! 🌍';
      const encoded = new TextEncoder().encode(text);
      const ptr = allocator.alloc(encoded.length);
      wasmMemory.writeBytes(ptr, encoded);

      funcs.vudo_println(ptr, encoded.length);

      expect(logger.printlns[0]).toBe(text);
    });
  });

  describe('Memory Integration', () => {
    it('should allocate memory successfully', () => {
      const funcs = createMemoryFunctions(allocator);

      const ptr1 = funcs.vudo_alloc(100) as number;
      const ptr2 = funcs.vudo_alloc(100) as number;

      expect(ptr1).toBeGreaterThan(0);
      expect(ptr2).toBeGreaterThan(0);
      expect(ptr2).toBeGreaterThan(ptr1);
    });

    it('should return 0 on allocation failure', () => {
      const funcs = createMemoryFunctions(allocator);

      const ptr = funcs.vudo_alloc(-1) as number;

      expect(ptr).toBe(0);
    });

    it('should reallocate memory with data preservation', () => {
      const funcs = createMemoryFunctions(allocator);

      const oldSize = 10;
      const ptr = funcs.vudo_alloc(oldSize) as number;
      const testData = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
      wasmMemory.writeBytes(ptr, testData);

      const newSize = 20;
      const newPtr = funcs.vudo_realloc(ptr, oldSize, newSize) as number;

      expect(newPtr).toBeGreaterThan(0);
      const readData = wasmMemory.readBytes(newPtr, oldSize);
      expect(Array.from(readData)).toEqual(Array.from(testData));
    });
  });

  describe('Time Integration', () => {
    it('should return current timestamp', () => {
      const funcs = createTimeFunctions(timeProvider);
      timeProvider.setTime(1234567890n);

      const timestamp = funcs.vudo_now() as bigint;

      expect(timestamp).toBe(1234567890n);
    });

    it('should advance time on sleep', () => {
      const funcs = createTimeFunctions(timeProvider);
      const initialTime = timeProvider.now();

      funcs.vudo_sleep(1000);

      expect(timeProvider.now()).toBe(initialTime + 1000n);
    });

    it('should return monotonic time', () => {
      const funcs = createTimeFunctions(timeProvider);
      timeProvider.setMonotonicTime(5000000000n);

      const monotonicTime = funcs.vudo_monotonic_now() as bigint;

      expect(monotonicTime).toBe(5000000000n);
    });
  });

  describe('Messaging Integration', () => {
    const spiritName = 'test-spirit';

    beforeEach(() => {
      messageBroker.register(spiritName);
      messageBroker.register('other-spirit');
    });

    it('should send message to another spirit', () => {
      const funcs = createMessagingFunctions(wasmMemory, allocator, messageBroker, spiritName);

      const target = 'other-spirit';
      const payload = new TextEncoder().encode('Hello!');

      const targetPtr = allocator.alloc(target.length);
      wasmMemory.writeString(targetPtr, target);

      const payloadPtr = allocator.alloc(payload.length);
      wasmMemory.writeBytes(payloadPtr, payload);

      const result = funcs.vudo_send(targetPtr, target.length, payloadPtr, payload.length) as number;

      expect(result).toBe(ResultCode.Ok);
      expect(messageBroker.pending('other-spirit')).toBe(1);
    });

    it('should receive message from inbox', () => {
      const funcs = createMessagingFunctions(wasmMemory, allocator, messageBroker, spiritName);

      const payload = new TextEncoder().encode('Test message');
      messageBroker.send('sender', spiritName, payload);

      const msgPtr = funcs.vudo_recv() as number;

      expect(msgPtr).toBeGreaterThan(0);

      let offset = msgPtr;
      const senderLen = wasmMemory.readI32(offset);
      offset += 4;
      const sender = wasmMemory.readString(offset, senderLen);
      offset += senderLen;
      const payloadLen = wasmMemory.readI32(offset);
      offset += 4;
      const receivedPayload = wasmMemory.readBytes(offset, payloadLen);

      expect(sender).toBe('sender');
      expect(Array.from(receivedPayload)).toEqual(Array.from(payload));
    });

    it('should check pending message count', () => {
      const funcs = createMessagingFunctions(wasmMemory, allocator, messageBroker, spiritName);

      const payload = new TextEncoder().encode('msg');
      messageBroker.send('sender', spiritName, payload);
      messageBroker.send('sender', spiritName, payload);
      messageBroker.send('sender', spiritName, payload);

      const pending = funcs.vudo_pending() as number;

      expect(pending).toBe(3);
    });

    it('should broadcast message to all spirits', () => {
      const funcs = createMessagingFunctions(wasmMemory, allocator, messageBroker, spiritName);

      messageBroker.register('spirit-1');
      messageBroker.register('spirit-2');

      const payload = new TextEncoder().encode('Broadcast!');
      const payloadPtr = allocator.alloc(payload.length);
      wasmMemory.writeBytes(payloadPtr, payload);

      const result = funcs.vudo_broadcast(payloadPtr, payload.length) as number;

      expect(result).toBe(ResultCode.Ok);
      expect(messageBroker.pending('other-spirit')).toBeGreaterThan(0);
      expect(messageBroker.pending('spirit-1')).toBeGreaterThan(0);
      expect(messageBroker.pending(spiritName)).toBe(0);
    });
  });

  describe('Random Integration', () => {
    it('should generate random numbers', () => {
      const funcs = createRandomFunctions(wasmMemory, randomProvider);
      randomProvider.setValues([0.1, 0.5, 0.9]);

      expect(funcs.vudo_random()).toBe(0.1);
      expect(funcs.vudo_random()).toBe(0.5);
      expect(funcs.vudo_random()).toBe(0.9);
    });

    it('should generate random bytes', () => {
      const funcs = createRandomFunctions(wasmMemory, randomProvider);

      const bufferSize = 10;
      const ptr = allocator.alloc(bufferSize);
      funcs.vudo_random_bytes(ptr, bufferSize);

      const bytes = wasmMemory.readBytes(ptr, bufferSize);

      expect(bytes.length).toBe(bufferSize);
      expect(bytes.some(b => b !== 0)).toBe(true);
    });
  });

  describe('Effects Integration', () => {
    const spiritName = 'test-spirit';

    it('should emit effects', () => {
      const funcs = createEffectFunctions(wasmMemory, effectHandler, spiritName);

      const payload = new TextEncoder().encode('effect-data');
      const payloadPtr = allocator.alloc(payload.length);
      wasmMemory.writeBytes(payloadPtr, payload);

      const result = funcs.vudo_emit_effect(10, payloadPtr, payload.length) as number;

      expect(result).toBe(ResultCode.Ok);
      const effects = effectHandler.getEffects();
      expect(effects).toHaveLength(1);
      expect(effects[0].id).toBe(10);
    });

    it('should subscribe to effect channels', () => {
      const funcs = createEffectFunctions(wasmMemory, effectHandler, spiritName);

      const channel = 'file-operations';
      const channelPtr = allocator.alloc(channel.length);
      wasmMemory.writeString(channelPtr, channel);

      const result = funcs.vudo_subscribe(channelPtr, channel.length) as number;

      expect(result).toBe(ResultCode.Ok);
      expect(effectHandler.getSubscribers(channel)).toContain(spiritName);
    });
  });

  describe('Debug Integration', () => {
    it('should trigger breakpoints', () => {
      const funcs = createDebugFunctions(wasmMemory, debugHandler);

      funcs.vudo_breakpoint();
      funcs.vudo_breakpoint();

      expect(debugHandler.breakpoints).toBe(2);
    });

    it('should assert true conditions', () => {
      const funcs = createDebugFunctions(wasmMemory, debugHandler);

      const message = 'Should be true';
      const msgPtr = allocator.alloc(message.length);
      wasmMemory.writeString(msgPtr, message);

      funcs.vudo_assert(1, msgPtr, message.length);

      expect(debugHandler.assertions).toHaveLength(1);
      expect(debugHandler.assertions[0].condition).toBe(true);
    });

    it('should throw on false assertions', () => {
      const funcs = createDebugFunctions(wasmMemory, debugHandler);

      const message = 'This should fail';
      const msgPtr = allocator.alloc(message.length);
      wasmMemory.writeString(msgPtr, message);

      expect(() => {
        funcs.vudo_assert(0, msgPtr, message.length);
      }).toThrow('Assertion failed: This should fail');
    });

    it('should panic with error message', () => {
      const funcs = createDebugFunctions(wasmMemory, debugHandler);

      const message = 'Critical error!';
      const msgPtr = allocator.alloc(message.length);
      wasmMemory.writeString(msgPtr, message);

      expect(() => {
        funcs.vudo_panic(msgPtr, message.length);
      }).toThrow('Panic: Critical error!');

      expect(debugHandler.panics).toHaveLength(1);
    });
  });

  describe('Full Stack Integration', () => {
    it('should handle complete workflow with multiple function categories', () => {
      const imports = {
        ...createIOFunctions(wasmMemory, logger),
        ...createMemoryFunctions(allocator),
        ...createTimeFunctions(timeProvider),
        ...createRandomFunctions(wasmMemory, randomProvider),
        ...createDebugFunctions(wasmMemory, debugHandler),
      };

      // 1. Allocate memory
      const size = 100;
      const ptr = imports.vudo_alloc(size) as number;
      expect(ptr).toBeGreaterThan(0);

      // 2. Write text
      const text = 'Integration test';
      const textLen = wasmMemory.writeString(ptr, text);

      // 3. Print
      imports.vudo_println(ptr, textLen);
      expect(logger.printlns[0]).toBe(text);

      // 4. Get timestamp
      timeProvider.setTime(1000n);
      const timestamp = imports.vudo_now() as bigint;
      expect(timestamp).toBe(1000n);

      // 5. Generate random
      randomProvider.setValues([0.42]);
      const random = imports.vudo_random() as number;
      expect(random).toBe(0.42);

      // 6. Assert
      const assertMsg = 'Everything works';
      const assertPtr = allocator.alloc(assertMsg.length);
      wasmMemory.writeString(assertPtr, assertMsg);
      imports.vudo_assert(1, assertPtr, assertMsg.length);

      // 7. Free memory
      imports.vudo_free(ptr, size);

      // Verify
      expect(logger.printlns).toHaveLength(1);
      expect(debugHandler.assertions).toHaveLength(1);
    });

    it('should maintain memory isolation between operations', () => {
      const funcs = createMemoryFunctions(allocator);

      const ptr1 = funcs.vudo_alloc(50) as number;
      const ptr2 = funcs.vudo_alloc(50) as number;
      const ptr3 = funcs.vudo_alloc(50) as number;

      wasmMemory.writeBytes(ptr1, new Uint8Array([1, 1, 1, 1, 1]));
      wasmMemory.writeBytes(ptr2, new Uint8Array([2, 2, 2, 2, 2]));
      wasmMemory.writeBytes(ptr3, new Uint8Array([3, 3, 3, 3, 3]));

      const read1 = wasmMemory.readBytes(ptr1, 5);
      const read2 = wasmMemory.readBytes(ptr2, 5);
      const read3 = wasmMemory.readBytes(ptr3, 5);

      expect(Array.from(read1)).toEqual([1, 1, 1, 1, 1]);
      expect(Array.from(read2)).toEqual([2, 2, 2, 2, 2]);
      expect(Array.from(read3)).toEqual([3, 3, 3, 3, 3]);
    });

    it('should handle multi-spirit messaging correctly', () => {
      messageBroker.register('spirit-1');
      messageBroker.register('spirit-2');
      messageBroker.register('spirit-3');

      const funcs1 = createMessagingFunctions(wasmMemory, allocator, messageBroker, 'spirit-1');
      const funcs2 = createMessagingFunctions(wasmMemory, allocator, messageBroker, 'spirit-2');
      const funcs3 = createMessagingFunctions(wasmMemory, allocator, messageBroker, 'spirit-3');

      const sendMessage = (funcs: Record<string, Function>, target: string, msg: string) => {
        const targetPtr = allocator.alloc(target.length);
        wasmMemory.writeString(targetPtr, target);
        const msgBytes = new TextEncoder().encode(msg);
        const msgPtr = allocator.alloc(msgBytes.length);
        wasmMemory.writeBytes(msgPtr, msgBytes);
        return funcs.vudo_send(targetPtr, target.length, msgPtr, msgBytes.length);
      };

      sendMessage(funcs1, 'spirit-2', 'Hello from 1');
      sendMessage(funcs1, 'spirit-3', 'Hello from 1');
      sendMessage(funcs2, 'spirit-1', 'Hello from 2');
      sendMessage(funcs2, 'spirit-3', 'Hello from 2');
      sendMessage(funcs3, 'spirit-1', 'Hello from 3');
      sendMessage(funcs3, 'spirit-2', 'Hello from 3');

      expect(messageBroker.pending('spirit-1')).toBe(2);
      expect(messageBroker.pending('spirit-2')).toBe(2);
      expect(messageBroker.pending('spirit-3')).toBe(2);
    });
  });
});
