/**
 * Debug system tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  DebugSystem,
  createDebugSystem,
  type IDebugHandler,
  DefaultDebugHandler,
  PanicError,
} from '../src/host/debug.js';

describe('DebugSystem', () => {
  let memory: WebAssembly.Memory;
  let debug: DebugSystem;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
    debug = createDebugSystem(memory);
  });

  describe('Breakpoint', () => {
    it('should trigger breakpoint', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      customDebug.breakpoint();

      expect(handler.onBreakpoint).toHaveBeenCalledTimes(1);
    });

    it('should increment breakpoint count', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      customDebug.breakpoint();
      customDebug.breakpoint();
      customDebug.breakpoint();

      const stats = customDebug.getStats();
      expect(stats.breakpoints).toBe(3);
    });

    it('should reset breakpoint count', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      customDebug.breakpoint();
      customDebug.breakpoint();
      customDebug.resetStats();

      const stats = customDebug.getStats();
      expect(stats.breakpoints).toBe(0);
    });
  });

  describe('Assert', () => {
    it('should pass when condition is true', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'Should pass';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      customDebug.assert(1, ptr, len); // Non-zero = true

      expect(handler.onAssertionFailure).not.toHaveBeenCalled();
    });

    it('should fail when condition is false', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'Should fail';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      customDebug.assert(0, ptr, len); // Zero = false

      expect(handler.onAssertionFailure).toHaveBeenCalledWith('Should fail');
    });

    it('should increment assertion count', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'test';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      customDebug.assert(1, ptr, len);
      customDebug.assert(1, ptr, len);
      customDebug.assert(0, ptr, len); // This one fails

      const stats = customDebug.getStats();
      expect(stats.assertions).toBe(3);
      expect(stats.assertionFailures).toBe(1);
    });

    it('should decode UTF-8 message correctly', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'Buffer size must be > 0';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      customDebug.assert(0, ptr, len);

      expect(handler.onAssertionFailure).toHaveBeenCalledWith(message);
    });
  });

  describe('Panic', () => {
    it('should call panic handler', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(() => {
          throw new PanicError('panic');
        }) as never,
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'Critical error';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      expect(() => customDebug.panic(ptr, len)).toThrow();
      expect(handler.onPanic).toHaveBeenCalledWith('Critical error');
    });

    it('should throw PanicError with default handler', () => {
      const message = 'Fatal error';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      expect(() => debug.panic(ptr, len)).toThrow(PanicError);
      expect(() => debug.panic(ptr, len)).toThrow('Fatal error');
    });

    it('should decode UTF-8 panic message correctly', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(() => {
          throw new PanicError('panic');
        }) as never,
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'Out of memory: allocation failed';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      expect(() => customDebug.panic(ptr, len)).toThrow();
      expect(handler.onPanic).toHaveBeenCalledWith(message);
    });
  });

  describe('Statistics', () => {
    it('should track all debug statistics', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'test';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      customDebug.breakpoint();
      customDebug.breakpoint();
      customDebug.assert(1, ptr, len); // Pass
      customDebug.assert(1, ptr, len); // Pass
      customDebug.assert(0, ptr, len); // Fail

      const stats = customDebug.getStats();
      expect(stats.breakpoints).toBe(2);
      expect(stats.assertions).toBe(3);
      expect(stats.assertionFailures).toBe(1);
    });

    it('should reset all statistics', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);

      const message = 'test';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      customDebug.breakpoint();
      customDebug.assert(0, ptr, len);
      customDebug.resetStats();

      const stats = customDebug.getStats();
      expect(stats.breakpoints).toBe(0);
      expect(stats.assertions).toBe(0);
      expect(stats.assertionFailures).toBe(0);
    });
  });

  describe('Host Functions', () => {
    it('should create host functions for WASM', () => {
      const hostFunctions = debug.createHostFunctions();

      expect(hostFunctions).toHaveProperty('vudo_breakpoint');
      expect(hostFunctions).toHaveProperty('vudo_assert');
      expect(hostFunctions).toHaveProperty('vudo_panic');
      expect(typeof hostFunctions.vudo_breakpoint).toBe('function');
      expect(typeof hostFunctions.vudo_assert).toBe('function');
      expect(typeof hostFunctions.vudo_panic).toBe('function');
    });

    it('should call breakpoint through host function', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);
      const hostFunctions = customDebug.createHostFunctions();

      hostFunctions.vudo_breakpoint();

      expect(handler.onBreakpoint).toHaveBeenCalledTimes(1);
    });

    it('should call assert through host function', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(),
      };

      const customDebug = createDebugSystem(memory, handler);
      const hostFunctions = customDebug.createHostFunctions();

      const message = 'test';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      hostFunctions.vudo_assert(0, ptr, len);

      expect(handler.onAssertionFailure).toHaveBeenCalledWith('test');
    });

    it('should call panic through host function', () => {
      const handler: IDebugHandler = {
        onBreakpoint: vi.fn(),
        onAssertionFailure: vi.fn(),
        onPanic: vi.fn(() => {
          throw new PanicError('panic');
        }) as never,
      };

      const customDebug = createDebugSystem(memory, handler);
      const hostFunctions = customDebug.createHostFunctions();

      const message = 'fatal';
      const ptr = encodeStringToMemory(memory, message);
      const len = new TextEncoder().encode(message).length;

      expect(() => hostFunctions.vudo_panic(ptr, len)).toThrow();
      expect(handler.onPanic).toHaveBeenCalledWith('fatal');
    });
  });

  describe('Memory Updates', () => {
    it('should update memory reference', () => {
      const newMemory = new WebAssembly.Memory({ initial: 2 });
      debug.updateMemory(newMemory);

      const message = 'test';
      const ptr = encodeStringToMemory(newMemory, message);
      const len = new TextEncoder().encode(message).length;

      // Should work with new memory
      expect(() => debug.assert(1, ptr, len)).not.toThrow();
    });
  });
});

describe('DefaultDebugHandler', () => {
  it('should trigger debugger statement on breakpoint', () => {
    const handler = new DefaultDebugHandler();

    // This just ensures the function runs without error
    // (debugger statement is a no-op in tests)
    expect(() => handler.onBreakpoint()).not.toThrow();
  });

  it('should throw Error on assertion failure', () => {
    const handler = new DefaultDebugHandler();

    expect(() => handler.onAssertionFailure('test')).toThrow(Error);
    expect(() => handler.onAssertionFailure('test')).toThrow('Assertion failed: test');
  });

  it('should throw PanicError on panic', () => {
    const handler = new DefaultDebugHandler();

    expect(() => handler.onPanic('fatal')).toThrow(PanicError);
    expect(() => handler.onPanic('fatal')).toThrow('fatal');
  });
});

describe('PanicError', () => {
  it('should be an instance of Error', () => {
    const error = new PanicError('test');

    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(PanicError);
  });

  it('should have correct name', () => {
    const error = new PanicError('test');

    expect(error.name).toBe('PanicError');
  });

  it('should have correct message', () => {
    const error = new PanicError('Out of memory');

    expect(error.message).toBe('Out of memory');
  });

  it('should format toString correctly', () => {
    const error = new PanicError('Critical failure');

    expect(error.toString()).toBe('PanicError: Critical failure');
  });
});

// Helper function to encode strings into WASM memory
function encodeStringToMemory(memory: WebAssembly.Memory, str: string): number {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(str);
  const view = new Uint8Array(memory.buffer);

  // Write at offset 2048 (safe zone)
  const ptr = 2048;
  view.set(bytes, ptr);

  return ptr;
}
