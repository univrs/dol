/**
 * Integration Tests for HostFunctionRegistry
 *
 * Verifies that all 22 host functions are present, callable, and properly bound to providers.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  HostFunctionRegistry,
  verifyImports,
  LogLevel,
  ResultCode,
  type ITimeProvider,
  type ILogger,
  type IMessageBroker,
  type IRandomProvider,
  type IEffectHandler,
  type IDebugHandler,
} from '../src/host/index.js';
import { BumpAllocator } from '../src/memory.js';

// ============================================================================
// MOCK PROVIDERS
// ============================================================================

class MockTimeProvider implements ITimeProvider {
  now(): bigint {
    return 1000n;
  }

  monotonic_now(): bigint {
    return 2000n;
  }

  async sleep(_ms: number): Promise<void> {
    // No-op for testing
  }
}

class MockLogger implements ILogger {
  logs: Array<{ level: LogLevel; message: string }> = [];

  log(level: LogLevel, message: string): void {
    this.logs.push({ level, message });
  }

  debug(message: string): void {
    this.log(LogLevel.DEBUG, message);
  }

  info(message: string): void {
    this.log(LogLevel.INFO, message);
  }

  warn(message: string): void {
    this.log(LogLevel.WARN, message);
  }

  error(message: string): void {
    this.log(LogLevel.ERROR, message);
  }
}

class MockMessageBroker implements IMessageBroker {
  messages: Map<string, Uint8Array[]> = new Map();

  send(target: string, payload: Uint8Array): ResultCode {
    if (!this.messages.has(target)) {
      this.messages.set(target, []);
    }
    this.messages.get(target)!.push(new Uint8Array(payload));
    return ResultCode.Ok;
  }

  recv(_spiritPtr: number, _spiritId: string): number | null {
    return null;
  }

  pending(_spiritId: string): number {
    return 0;
  }

  broadcast(payload: Uint8Array, _sender: string): ResultCode {
    return ResultCode.Ok;
  }

  freeMessage(_ptr: number): void {
    // No-op
  }
}

class MockRandomProvider implements IRandomProvider {
  random(): number {
    return 0.5;
  }

  randomBytes(len: number): Uint8Array {
    return new Uint8Array(len).fill(42);
  }
}

class MockEffectHandler implements IEffectHandler {
  effects: Array<{ effectId: number; payload: Uint8Array }> = [];

  emitEffect(effectId: number, payload: Uint8Array): ResultCode {
    this.effects.push({ effectId, payload: new Uint8Array(payload) });
    return ResultCode.Ok;
  }

  subscribe(_channel: string, _spiritId: string): ResultCode {
    return ResultCode.Ok;
  }
}

class MockDebugHandler implements IDebugHandler {
  breakpoints: number = 0;
  assertions: Array<{ condition: boolean; message: string }> = [];
  panics: string[] = [];

  breakpoint(): void {
    this.breakpoints++;
  }

  assert(condition: boolean, message: string): void {
    this.assertions.push({ condition, message });
  }

  panic(message: string): void {
    this.panics.push(message);
    throw new Error(`Panic: ${message}`);
  }
}

// ============================================================================
// TESTS
// ============================================================================

describe('HostFunctionRegistry', () => {
  let memory: WebAssembly.Memory;
  let allocator: BumpAllocator;
  let mockTimeProvider: MockTimeProvider;
  let mockLogger: MockLogger;
  let mockMessageBroker: MockMessageBroker;
  let mockRandomProvider: MockRandomProvider;
  let mockEffectHandler: MockEffectHandler;
  let mockDebugHandler: MockDebugHandler;
  let registry: HostFunctionRegistry;
  let imports: ReturnType<HostFunctionRegistry['getImports']>;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 256 });
    allocator = new BumpAllocator(memory);

    mockTimeProvider = new MockTimeProvider();
    mockLogger = new MockLogger();
    mockMessageBroker = new MockMessageBroker();
    mockRandomProvider = new MockRandomProvider();
    mockEffectHandler = new MockEffectHandler();
    mockDebugHandler = new MockDebugHandler();

    registry = new HostFunctionRegistry(memory, allocator, {
      time: mockTimeProvider,
      logger: mockLogger,
      messageBroker: mockMessageBroker,
      random: mockRandomProvider,
      effectHandler: mockEffectHandler,
      debugHandler: mockDebugHandler,
    });

    imports = registry.getImports();
  });

  // ========================================================================
  // VERIFICATION TESTS
  // ========================================================================

  it('should create registry with all required providers', () => {
    expect(registry).toBeDefined();
    expect(registry.getSpiritId()).toBe('unknown');
  });

  it('should have all 22 host functions', () => {
    const required = [
      // I/O Functions (4)
      'vudo_print',
      'vudo_println',
      'vudo_log',
      'vudo_error',

      // Memory Functions (3)
      'vudo_alloc',
      'vudo_free',
      'vudo_realloc',

      // Time Functions (3)
      'vudo_now',
      'vudo_sleep',
      'vudo_monotonic_now',

      // Messaging Functions (5)
      'vudo_send',
      'vudo_recv',
      'vudo_pending',
      'vudo_broadcast',
      'vudo_free_message',

      // Random Functions (2)
      'vudo_random',
      'vudo_random_bytes',

      // Effect Functions (2)
      'vudo_emit_effect',
      'vudo_subscribe',

      // Debug Functions (3)
      'vudo_breakpoint',
      'vudo_assert',
      'vudo_panic',
    ];

    expect(Object.keys(imports.vudo)).toHaveLength(required.length);
    required.forEach((name) => {
      expect(imports.vudo[name]).toBeDefined();
      expect(typeof imports.vudo[name]).toBe('function');
    });
  });

  it('should verify imports with verifyImports helper', () => {
    expect(verifyImports(imports)).toBe(true);
  });

  // ========================================================================
  // I/O FUNCTION TESTS
  // ========================================================================

  describe('I/O Functions', () => {
    beforeEach(() => {
      mockLogger.logs = [];
    });

    it('vudo_print should print to stdout', () => {
      const text = 'Hello';
      const ptr = allocator.alloc(text.length);
      const view = new Uint8Array(memory.buffer, ptr, text.length);
      view.set(new TextEncoder().encode(text));

      (imports.vudo.vudo_print as (ptr: number, len: number) => void)(ptr, text.length);
      // Note: This test verifies it doesn't throw, actual output goes to stdout
    });

    it('vudo_println should print with newline', () => {
      const text = 'Hello';
      const ptr = allocator.alloc(text.length);
      const view = new Uint8Array(memory.buffer, ptr, text.length);
      view.set(new TextEncoder().encode(text));

      (imports.vudo.vudo_println as (ptr: number, len: number) => void)(ptr, text.length);
      // Note: This test verifies it doesn't throw, actual output goes to console
    });

    it('vudo_log should log at specified level', () => {
      const text = 'Test message';
      const ptr = allocator.alloc(text.length);
      const view = new Uint8Array(memory.buffer, ptr, text.length);
      view.set(new TextEncoder().encode(text));

      (imports.vudo.vudo_log as (level: number, ptr: number, len: number) => void)(
        LogLevel.INFO,
        ptr,
        text.length,
      );

      expect(mockLogger.logs.length).toBe(1);
      expect(mockLogger.logs[0].level).toBe(LogLevel.INFO);
      expect(mockLogger.logs[0].message).toBe(text);
    });

    it('vudo_error should log error', () => {
      const text = 'Error occurred';
      const ptr = allocator.alloc(text.length);
      const view = new Uint8Array(memory.buffer, ptr, text.length);
      view.set(new TextEncoder().encode(text));

      (imports.vudo.vudo_error as (ptr: number, len: number) => void)(ptr, text.length);

      expect(mockLogger.logs.length).toBe(1);
      expect(mockLogger.logs[0].level).toBe(LogLevel.ERROR);
    });
  });

  // ========================================================================
  // MEMORY FUNCTION TESTS
  // ========================================================================

  describe('Memory Functions', () => {
    it('vudo_alloc should allocate memory', () => {
      const size = 256;
      const ptr = (imports.vudo.vudo_alloc as (size: number) => number)(size);

      expect(ptr).toBeGreaterThan(0);
      expect(ptr).toBeLessThan(memory.buffer.byteLength);
    });

    it('vudo_alloc with multiple calls should allocate different pointers', () => {
      const ptr1 = (imports.vudo.vudo_alloc as (size: number) => number)(256);
      const ptr2 = (imports.vudo.vudo_alloc as (size: number) => number)(256);

      expect(ptr1).toBeGreaterThan(0);
      expect(ptr2).toBeGreaterThan(0);
      expect(ptr1).not.toBe(ptr2);
    });

    it('vudo_free should not throw', () => {
      const ptr = (imports.vudo.vudo_alloc as (size: number) => number)(256);
      expect(() => {
        (imports.vudo.vudo_free as (ptr: number, size: number) => void)(ptr, 256);
      }).not.toThrow();
    });

    it('vudo_realloc should resize memory', () => {
      const ptr1 = (imports.vudo.vudo_alloc as (size: number) => number)(256);
      const ptr2 = (imports.vudo.vudo_realloc as (
        ptr: number,
        oldSize: number,
        newSize: number
      ) => number)(ptr1, 256, 512);

      expect(ptr2).toBeGreaterThan(0);
      expect(typeof ptr2).toBe('number');
    });
  });

  // ========================================================================
  // TIME FUNCTION TESTS
  // ========================================================================

  describe('Time Functions', () => {
    it('vudo_now should return timestamp', () => {
      const now = (imports.vudo.vudo_now as () => bigint)();

      expect(now).toBe(1000n);
      expect(typeof now).toBe('bigint');
    });

    it('vudo_monotonic_now should return monotonic time', () => {
      const time = (imports.vudo.vudo_monotonic_now as () => bigint)();

      expect(time).toBe(2000n);
      expect(typeof time).toBe('bigint');
    });

    it('vudo_sleep should not throw', () => {
      expect(() => {
        (imports.vudo.vudo_sleep as (ms: number) => void)(100);
      }).not.toThrow();
    });
  });

  // ========================================================================
  // RANDOM FUNCTION TESTS
  // ========================================================================

  describe('Random Functions', () => {
    it('vudo_random should return f64 between 0 and 1', () => {
      const value = (imports.vudo.vudo_random as () => number)();

      expect(value).toBe(0.5);
      expect(typeof value).toBe('number');
    });

    it('vudo_random_bytes should fill buffer', () => {
      const len = 16;
      const ptr = (imports.vudo.vudo_alloc as (size: number) => number)(len);

      (imports.vudo.vudo_random_bytes as (ptr: number, len: number) => void)(ptr, len);

      const buffer = new Uint8Array(memory.buffer, ptr, len);
      expect(buffer).toEqual(new Uint8Array(len).fill(42));
    });
  });

  // ========================================================================
  // MESSAGING FUNCTION TESTS
  // ========================================================================

  describe('Messaging Functions', () => {
    beforeEach(() => {
      registry.setSpiritId('spirit-1');
    });

    it('vudo_send should send message to target', () => {
      const target = 'spirit-2';
      const payload = 'Hello';

      const targetPtr = allocator.alloc(target.length);
      const targetView = new Uint8Array(memory.buffer, targetPtr, target.length);
      targetView.set(new TextEncoder().encode(target));

      const payloadPtr = allocator.alloc(payload.length);
      const payloadView = new Uint8Array(memory.buffer, payloadPtr, payload.length);
      payloadView.set(new TextEncoder().encode(payload));

      const result = (imports.vudo.vudo_send as (
        targetPtr: number,
        targetLen: number,
        payloadPtr: number,
        payloadLen: number
      ) => number)(targetPtr, target.length, payloadPtr, payload.length);

      expect(result).toBe(ResultCode.Ok);
      expect(mockMessageBroker.messages.get(target)).toBeDefined();
    });

    it('vudo_pending should return pending message count', () => {
      const count = (imports.vudo.vudo_pending as () => number)();

      expect(count).toBe(0);
      expect(typeof count).toBe('number');
    });

    it('vudo_recv should return null when no messages', () => {
      const ptr = (imports.vudo.vudo_recv as () => number)();

      expect(ptr).toBe(0);
    });

    it('vudo_free_message should not throw', () => {
      expect(() => {
        (imports.vudo.vudo_free_message as (ptr: number) => void)(100);
      }).not.toThrow();
    });

    it('vudo_broadcast should broadcast to all spirits', () => {
      const payload = 'Broadcast message';
      const payloadPtr = allocator.alloc(payload.length);
      const payloadView = new Uint8Array(memory.buffer, payloadPtr, payload.length);
      payloadView.set(new TextEncoder().encode(payload));

      const result = (imports.vudo.vudo_broadcast as (ptr: number, len: number) => number)(
        payloadPtr,
        payload.length,
      );

      expect(result).toBe(ResultCode.Ok);
    });
  });

  // ========================================================================
  // EFFECT FUNCTION TESTS
  // ========================================================================

  describe('Effect Functions', () => {
    it('vudo_emit_effect should emit effect', () => {
      const payload = 'Effect payload';
      const payloadPtr = allocator.alloc(payload.length);
      const payloadView = new Uint8Array(memory.buffer, payloadPtr, payload.length);
      payloadView.set(new TextEncoder().encode(payload));

      const result = (imports.vudo.vudo_emit_effect as (
        effectId: number,
        ptr: number,
        len: number
      ) => number)(1, payloadPtr, payload.length);

      expect(result).toBe(ResultCode.Ok);
      expect(mockEffectHandler.effects.length).toBe(1);
      expect(mockEffectHandler.effects[0].effectId).toBe(1);
    });

    it('vudo_subscribe should subscribe to channel', () => {
      const channel = 'test-channel';
      const channelPtr = allocator.alloc(channel.length);
      const channelView = new Uint8Array(memory.buffer, channelPtr, channel.length);
      channelView.set(new TextEncoder().encode(channel));

      const result = (imports.vudo.vudo_subscribe as (
        channelPtr: number,
        channelLen: number
      ) => number)(channelPtr, channel.length);

      expect(result).toBe(ResultCode.Ok);
    });
  });

  // ========================================================================
  // DEBUG FUNCTION TESTS
  // ========================================================================

  describe('Debug Functions', () => {
    it('vudo_breakpoint should trigger', () => {
      (imports.vudo.vudo_breakpoint as () => void)();

      expect(mockDebugHandler.breakpoints).toBe(1);
    });

    it('vudo_assert with true condition should pass silently', () => {
      const message = 'All good';
      const messagePtr = allocator.alloc(message.length);
      const messageView = new Uint8Array(memory.buffer, messagePtr, message.length);
      messageView.set(new TextEncoder().encode(message));

      (imports.vudo.vudo_assert as (condition: number, ptr: number, len: number) => void)(
        1,
        messagePtr,
        message.length,
      );

      expect(mockDebugHandler.assertions.length).toBe(1);
      expect(mockDebugHandler.assertions[0].condition).toBe(true);
    });

    it('vudo_assert with false condition should be logged', () => {
      const message = 'Something wrong';
      const messagePtr = allocator.alloc(message.length);
      const messageView = new Uint8Array(memory.buffer, messagePtr, message.length);
      messageView.set(new TextEncoder().encode(message));

      (imports.vudo.vudo_assert as (condition: number, ptr: number, len: number) => void)(
        0,
        messagePtr,
        message.length,
      );

      expect(mockDebugHandler.assertions.length).toBe(1);
      expect(mockDebugHandler.assertions[0].condition).toBe(false);
    });

    it('vudo_panic should throw', () => {
      const message = 'Fatal error';
      const messagePtr = allocator.alloc(message.length);
      const messageView = new Uint8Array(memory.buffer, messagePtr, message.length);
      messageView.set(new TextEncoder().encode(message));

      expect(() => {
        (imports.vudo.vudo_panic as (ptr: number, len: number) => void)(messagePtr, message.length);
      }).toThrow();

      expect(mockDebugHandler.panics.length).toBe(1);
    });
  });

  // ========================================================================
  // SPIRIT ID MANAGEMENT
  // ========================================================================

  describe('Spirit ID Management', () => {
    it('should set and get spirit ID', () => {
      expect(registry.getSpiritId()).toBe('unknown');

      registry.setSpiritId('test-spirit');
      expect(registry.getSpiritId()).toBe('test-spirit');
    });
  });

  // ========================================================================
  // REGISTRY METADATA
  // ========================================================================

  describe('Registry Metadata', () => {
    it('should provide function metadata', () => {
      const metadata = registry.getMetadata();

      expect(metadata).toBeDefined();
      expect(metadata.length).toBe(22);
      expect(metadata.every((m) => m.name && m.category && m.params)).toBe(true);
    });
  });
});
