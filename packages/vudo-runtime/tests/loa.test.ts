/**
 * Loa (service) tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  coreLoa,
  LoaRegistry,
  createLoa,
  createLoggingLoa,
} from '../src/loa.js';
import type { Loa, LoaContext } from '../src/types.js';

describe('coreLoa', () => {
  it('should have correct name and version', () => {
    expect(coreLoa.name).toBe('core');
    expect(coreLoa.version).toBe('1.0.0');
  });

  it('should provide required capabilities', () => {
    expect(coreLoa.capabilities).toContain('print');
    expect(coreLoa.capabilities).toContain('alloc');
    expect(coreLoa.capabilities).toContain('time');
    expect(coreLoa.capabilities).toContain('random');
  });

  it('should provide vudo_now function', () => {
    const memory = new WebAssembly.Memory({ initial: 1 });
    const context: LoaContext = {
      memory,
      alloc: () => 0,
      debug: false,
    };

    const imports = coreLoa.provides(context);
    const now = (imports.vudo_now as () => bigint)();

    expect(typeof now).toBe('bigint');
    expect(now).toBeGreaterThan(0n);
  });

  it('should provide vudo_random function', () => {
    const memory = new WebAssembly.Memory({ initial: 1 });
    const context: LoaContext = {
      memory,
      alloc: () => 0,
      debug: false,
    };

    const imports = coreLoa.provides(context);
    const random = (imports.vudo_random as () => number)();

    expect(typeof random).toBe('number');
    expect(random).toBeGreaterThanOrEqual(0);
    expect(random).toBeLessThan(1);
  });

  it('should provide vudo_alloc that uses context allocator', () => {
    const memory = new WebAssembly.Memory({ initial: 1 });
    const mockAlloc = vi.fn().mockReturnValue(1024);
    const context: LoaContext = {
      memory,
      alloc: mockAlloc,
      debug: false,
    };

    const imports = coreLoa.provides(context);
    const ptr = (imports.vudo_alloc as (size: number) => number)(100);

    expect(mockAlloc).toHaveBeenCalledWith(100);
    expect(ptr).toBe(1024);
  });

  it('should provide vudo_print that reads from memory', () => {
    const memory = new WebAssembly.Memory({ initial: 1 });
    const context: LoaContext = {
      memory,
      alloc: () => 0,
      debug: false,
    };

    // Write a test string to memory
    const testStr = 'Hello, Spirit!';
    const encoder = new TextEncoder();
    const bytes = encoder.encode(testStr);
    new Uint8Array(memory.buffer, 1024, bytes.length).set(bytes);

    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const imports = coreLoa.provides(context);
    (imports.vudo_print as (ptr: number, len: number) => void)(1024, bytes.length);

    expect(consoleSpy).toHaveBeenCalledWith('[Spirit]', testStr);
    consoleSpy.mockRestore();
  });
});

describe('LoaRegistry', () => {
  let registry: LoaRegistry;

  beforeEach(() => {
    registry = new LoaRegistry();
  });

  it('should have core Loa registered by default', () => {
    expect(registry.has('core')).toBe(true);
  });

  it('should register new Loas', () => {
    const customLoa: Loa = {
      name: 'custom',
      version: '1.0.0',
      capabilities: ['test'],
      provides: () => ({}),
    };

    registry.register(customLoa);
    expect(registry.has('custom')).toBe(true);
    expect(registry.get('custom')).toBe(customLoa);
  });

  it('should throw when registering duplicate name', () => {
    const customLoa: Loa = {
      name: 'custom',
      version: '1.0.0',
      capabilities: [],
      provides: () => ({}),
    };

    registry.register(customLoa);

    expect(() => registry.register(customLoa)).toThrow(
      "Loa 'custom' is already registered"
    );
  });

  it('should unregister Loas', () => {
    const customLoa: Loa = {
      name: 'custom',
      version: '1.0.0',
      capabilities: [],
      provides: () => ({}),
    };

    registry.register(customLoa);
    expect(registry.has('custom')).toBe(true);

    registry.unregister('custom');
    expect(registry.has('custom')).toBe(false);
  });

  it('should not allow unregistering core Loa', () => {
    expect(() => registry.unregister('core')).toThrow(
      'Cannot unregister core Loa'
    );
  });

  it('should list all registered Loas', () => {
    const customLoa: Loa = {
      name: 'custom',
      version: '1.0.0',
      capabilities: [],
      provides: () => ({}),
    };

    registry.register(customLoa);

    const all = registry.all();
    expect(all.length).toBe(2); // core + custom
    expect(all.some(l => l.name === 'core')).toBe(true);
    expect(all.some(l => l.name === 'custom')).toBe(true);
  });

  it('should build combined imports', () => {
    const customLoa: Loa = {
      name: 'custom',
      version: '1.0.0',
      capabilities: ['custom_fn'],
      provides: () => ({
        custom_fn: () => 42,
      }),
    };

    registry.register(customLoa);

    const memory = new WebAssembly.Memory({ initial: 1 });
    const context: LoaContext = {
      memory,
      alloc: () => 0,
      debug: false,
    };

    const imports = registry.buildImports(context);

    // Core imports
    expect(imports.vudo_print).toBeDefined();
    expect(imports.vudo_alloc).toBeDefined();
    expect(imports.vudo_now).toBeDefined();

    // Custom imports
    expect(imports.custom_fn).toBeDefined();
    expect((imports.custom_fn as () => number)()).toBe(42);
  });
});

describe('createLoa helper', () => {
  it('should create a Loa from function definitions', () => {
    const loa = createLoa('test', '1.0.0', {
      test_fn: () => () => 123,
    });

    expect(loa.name).toBe('test');
    expect(loa.version).toBe('1.0.0');
    expect(loa.capabilities).toContain('test_fn');

    const memory = new WebAssembly.Memory({ initial: 1 });
    const context: LoaContext = {
      memory,
      alloc: () => 0,
      debug: false,
    };

    const imports = loa.provides(context);
    expect((imports.test_fn as () => number)()).toBe(123);
  });
});

describe('createLoggingLoa helper', () => {
  it('should create a logging Loa with custom logger', () => {
    const mockLogger = {
      log: vi.fn(),
      error: vi.fn(),
      debug: vi.fn(),
    };

    const loa = createLoggingLoa(mockLogger);

    expect(loa.name).toBe('logging');
    expect(loa.capabilities).toContain('log');
    expect(loa.capabilities).toContain('error');
    expect(loa.capabilities).toContain('debug');

    const memory = new WebAssembly.Memory({ initial: 1 });

    // Write test string
    const testStr = 'Test message';
    const bytes = new TextEncoder().encode(testStr);
    new Uint8Array(memory.buffer, 1024, bytes.length).set(bytes);

    const context: LoaContext = {
      memory,
      alloc: () => 0,
      debug: false,
    };

    const imports = loa.provides(context);
    (imports.vudo_print as (ptr: number, len: number) => void)(1024, bytes.length);

    expect(mockLogger.log).toHaveBeenCalledWith(testStr);
  });
});
