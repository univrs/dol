/**
 * Spirit loading tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { Spirit, SpiritLoader, loadSpirit } from '../src/spirit.js';
import { LoaRegistry } from '../src/loa.js';

// Minimal WASM module that exports an 'add' function
// (module
//   (func $add (export "add") (param i64 i64) (result i64)
//     local.get 0
//     local.get 1
//     i64.add
//   )
// )
const MINIMAL_ADD_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, // magic
  0x01, 0x00, 0x00, 0x00, // version
  0x01, 0x07,             // type section
  0x01, 0x60, 0x02, 0x7e, 0x7e, 0x01, 0x7e, // (func (param i64 i64) (result i64))
  0x03, 0x02,             // func section
  0x01, 0x00,             // function 0 uses type 0
  0x07, 0x07,             // export section
  0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // export "add" func 0
  0x0a, 0x09,             // code section
  0x01, 0x07, 0x00,       // function body
  0x20, 0x00,             // local.get 0
  0x20, 0x01,             // local.get 1
  0x7c,                   // i64.add
  0x0b,                   // end
]);

// Minimal WASM module that exports a 'get42' function
const MINIMAL_CONST_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, // magic
  0x01, 0x00, 0x00, 0x00, // version
  0x01, 0x05,             // type section
  0x01, 0x60, 0x00, 0x01, 0x7e, // (func (result i64))
  0x03, 0x02,             // func section
  0x01, 0x00,             // function 0 uses type 0
  0x07, 0x09,             // export section
  0x01, 0x05, 0x67, 0x65, 0x74, 0x34, 0x32, 0x00, 0x00, // export "get42" func 0
  0x0a, 0x08,             // code section
  0x01, 0x06, 0x00,       // function body
  0x42, 0x2a,             // i64.const 42
  0x0b,                   // end
]);

describe('Spirit', () => {
  let spirit: Spirit;

  beforeEach(async () => {
    spirit = await loadSpirit(MINIMAL_ADD_WASM);
  });

  it('should call exported functions', () => {
    const result = spirit.call<bigint>('add', [10n, 20n]);
    expect(result).toBe(30n);
  });

  it('should check if function exists', () => {
    expect(spirit.hasFunction('add')).toBe(true);
    expect(spirit.hasFunction('nonexistent')).toBe(false);
  });

  it('should list exported functions', () => {
    const funcs = spirit.listFunctions();
    expect(funcs).toContain('add');
  });

  it('should throw for nonexistent function', () => {
    expect(() => spirit.call('nonexistent')).toThrow(
      "Function 'nonexistent' not found"
    );
  });

  it('should provide memory manager', () => {
    expect(spirit.memory).toBeDefined();
    expect(spirit.memory.alloc).toBeDefined();
    expect(spirit.memory.readGene).toBeDefined();
  });

  it('should provide raw exports', () => {
    expect(spirit.exports).toBeDefined();
    expect(typeof spirit.exports.add).toBe('function');
  });
});

describe('Spirit.as<T>()', () => {
  it('should create typed proxy', async () => {
    const spirit = await loadSpirit(MINIMAL_ADD_WASM);

    interface Calculator {
      add(a: bigint, b: bigint): bigint;
    }

    const calc = spirit.as<Calculator>();
    const result = calc.add(5n, 3n);

    expect(result).toBe(8n);
  });

  it('should work with constant function', async () => {
    const spirit = await loadSpirit(MINIMAL_CONST_WASM);

    interface Constant {
      get42(): bigint;
    }

    const c = spirit.as<Constant>();
    expect(c.get42()).toBe(42n);
  });
});

describe('SpiritLoader', () => {
  it('should load from bytes', async () => {
    const loader = new SpiritLoader();
    const spirit = await loader.load(MINIMAL_ADD_WASM);

    expect(spirit).toBeInstanceOf(Spirit);
    expect(spirit.hasFunction('add')).toBe(true);
  });

  it('should accept custom Loa registry', async () => {
    const registry = new LoaRegistry();
    const loader = new SpiritLoader({ loas: registry });

    const spirit = await loader.load(MINIMAL_ADD_WASM);
    expect(spirit).toBeInstanceOf(Spirit);
  });

  it('should accept debug option', async () => {
    const loader = new SpiritLoader({ debug: true });
    const spirit = await loader.load(MINIMAL_ADD_WASM);

    expect(spirit).toBeInstanceOf(Spirit);
  });

  it('should accept memory options', async () => {
    const loader = new SpiritLoader();
    const spirit = await loader.load(MINIMAL_ADD_WASM, {
      memory: { initial: 32, maximum: 128 },
    });

    expect(spirit).toBeInstanceOf(Spirit);
  });
});

describe('loadSpirit convenience function', () => {
  it('should load from ArrayBuffer', async () => {
    const buffer = MINIMAL_ADD_WASM.buffer;
    const spirit = await loadSpirit(buffer);

    expect(spirit).toBeInstanceOf(Spirit);
  });

  it('should load from Uint8Array', async () => {
    const spirit = await loadSpirit(MINIMAL_ADD_WASM);

    expect(spirit).toBeInstanceOf(Spirit);
  });

  it('should accept load options', async () => {
    const spirit = await loadSpirit(MINIMAL_ADD_WASM, {
      debug: false,
      memory: { initial: 16 },
    });

    expect(spirit).toBeInstanceOf(Spirit);
    expect(spirit.call<bigint>('add', [1n, 2n])).toBe(3n);
  });
});
