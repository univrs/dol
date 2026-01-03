/**
 * CLI command tests
 */

import { describe, it, expect } from 'vitest';

// Minimal WASM module that exports an 'add' function
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

describe('parseArg', () => {
  // Test the argument parsing logic
  function parseArg(arg: string): unknown {
    // BigInt (ends with 'n')
    if (/^-?\d+n$/.test(arg)) {
      return BigInt(arg.slice(0, -1));
    }
    // Integer
    if (/^-?\d+$/.test(arg)) {
      const num = parseInt(arg, 10);
      if (num > Number.MAX_SAFE_INTEGER || num < Number.MIN_SAFE_INTEGER) {
        return BigInt(arg);
      }
      return num;
    }
    // Float
    if (/^-?\d+\.\d+$/.test(arg)) {
      return parseFloat(arg);
    }
    // Boolean
    if (arg === 'true') return true;
    if (arg === 'false') return false;
    // String
    return arg;
  }

  it('should parse integers', () => {
    expect(parseArg('123')).toBe(123);
    expect(parseArg('-456')).toBe(-456);
    expect(parseArg('0')).toBe(0);
  });

  it('should parse explicit BigInts', () => {
    expect(parseArg('123n')).toBe(123n);
    expect(parseArg('-456n')).toBe(-456n);
    expect(parseArg('9007199254740993n')).toBe(9007199254740993n);
  });

  it('should parse floats', () => {
    expect(parseArg('3.14')).toBeCloseTo(3.14);
    expect(parseArg('-2.5')).toBeCloseTo(-2.5);
  });

  it('should parse booleans', () => {
    expect(parseArg('true')).toBe(true);
    expect(parseArg('false')).toBe(false);
  });

  it('should parse strings', () => {
    expect(parseArg('hello')).toBe('hello');
    expect(parseArg('hello world')).toBe('hello world');
  });
});

describe('WASM module inspection', () => {
  it('should parse minimal WASM module', async () => {
    const module = await WebAssembly.compile(MINIMAL_ADD_WASM);
    const exports = WebAssembly.Module.exports(module);

    expect(exports).toHaveLength(1);
    expect(exports[0].name).toBe('add');
    expect(exports[0].kind).toBe('function');
  });

  it('should list imports', async () => {
    const module = await WebAssembly.compile(MINIMAL_ADD_WASM);
    const imports = WebAssembly.Module.imports(module);

    // Minimal module has no imports
    expect(imports).toHaveLength(0);
  });
});
