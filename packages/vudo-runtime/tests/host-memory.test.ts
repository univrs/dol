/**
 * WasmMemory interface tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { WasmMemory, MemoryError } from '../src/host/memory.js';

describe('WasmMemory - String Operations', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 2 }); // 128KB
    wasmMem = new WasmMemory(memory);
  });

  it('should read and write ASCII strings', () => {
    const str = 'Hello, World!';
    const ptr = 1024;

    const written = wasmMem.writeString(ptr, str);
    expect(written).toBe(str.length);

    const read = wasmMem.readString(ptr, written);
    expect(read).toBe(str);
  });

  it('should handle UTF-8 characters', () => {
    const str = 'Hello, 世界! 🌍';
    const ptr = 2048;

    const written = wasmMem.writeString(ptr, str);
    const read = wasmMem.readString(ptr, written);

    expect(read).toBe(str);
  });

  it('should handle empty strings', () => {
    const str = '';
    const ptr = 3072;

    const written = wasmMem.writeString(ptr, str);
    expect(written).toBe(0);

    const read = wasmMem.readString(ptr, written);
    expect(read).toBe('');
  });

  it('should handle long strings', () => {
    const str = 'a'.repeat(10000);
    const ptr = 4096;

    const written = wasmMem.writeString(ptr, str);
    const read = wasmMem.readString(ptr, written);

    expect(read).toBe(str);
  });

  it('should handle multi-byte UTF-8 sequences', () => {
    const str = '你好🎉中文✨字符串';
    const ptr = 20480;

    const written = wasmMem.writeString(ptr, str);
    const read = wasmMem.readString(ptr, written);

    expect(read).toBe(str);
  });
});

describe('WasmMemory - Byte Operations', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 2 });
    wasmMem = new WasmMemory(memory);
  });

  it('should read bytes', () => {
    const ptr = 1024;
    const view = new Uint8Array(memory.buffer, ptr, 4);
    view[0] = 1;
    view[1] = 2;
    view[2] = 3;
    view[3] = 4;

    const bytes = wasmMem.readBytes(ptr, 4);
    expect(bytes).toEqual(new Uint8Array([1, 2, 3, 4]));
  });

  it('should write bytes', () => {
    const ptr = 2048;
    const data = new Uint8Array([10, 20, 30, 40, 50]);

    const written = wasmMem.writeBytes(ptr, data);
    expect(written).toBe(5);

    const view = new Uint8Array(memory.buffer, ptr, 5);
    expect(view).toEqual(data);
  });

  it('should handle large byte sequences', () => {
    const ptr = 4096;
    const data = new Uint8Array(1000);
    for (let i = 0; i < 1000; i++) {
      data[i] = i % 256;
    }

    wasmMem.writeBytes(ptr, data);
    const read = wasmMem.readBytes(ptr, 1000);

    expect(read).toEqual(data);
  });
});

describe('WasmMemory - Numeric Operations', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 2 });
    wasmMem = new WasmMemory(memory);
  });

  describe('I32 Operations', () => {
    it('should read and write positive i32', () => {
      const ptr = 1024;
      const value = 42;

      wasmMem.writeI32(ptr, value);
      const read = wasmMem.readI32(ptr);

      expect(read).toBe(value);
    });

    it('should read and write negative i32', () => {
      const ptr = 1028;
      const value = -12345;

      wasmMem.writeI32(ptr, value);
      const read = wasmMem.readI32(ptr);

      expect(read).toBe(value);
    });

    it('should handle i32 boundary values', () => {
      const ptr = 1032;
      const values = [0, 1, -1, 2147483647, -2147483648];

      values.forEach((val, i) => {
        const offset = ptr + i * 4;
        wasmMem.writeI32(offset, val);
        expect(wasmMem.readI32(offset)).toBe(val);
      });
    });
  });

  describe('I64 Operations', () => {
    it('should read and write positive i64', () => {
      const ptr = 2048;
      const value = 9007199254740992n;

      wasmMem.writeI64(ptr, value);
      const read = wasmMem.readI64(ptr);

      expect(read).toBe(value);
    });

    it('should read and write negative i64', () => {
      const ptr = 2056;
      const value = -9007199254740992n;

      wasmMem.writeI64(ptr, value);
      const read = wasmMem.readI64(ptr);

      expect(read).toBe(value);
    });

    it('should handle i64 boundary values', () => {
      const ptr = 2064;
      const values = [
        0n,
        1n,
        -1n,
        9223372036854775807n, // Max i64
        -9223372036854775808n, // Min i64
      ];

      values.forEach((val, i) => {
        const offset = ptr + i * 8;
        wasmMem.writeI64(offset, val);
        expect(wasmMem.readI64(offset)).toBe(val);
      });
    });
  });

  describe('F32 Operations', () => {
    it('should read and write f32', () => {
      const ptr = 3072;
      const value = 3.14159;

      wasmMem.writeF32(ptr, value);
      const read = wasmMem.readF32(ptr);

      expect(read).toBeCloseTo(value, 4);
    });

    it('should handle f32 special values', () => {
      const ptr = 3076;
      const values = [0.0, -0.0, 1.5, -1.5, Infinity, -Infinity];

      values.forEach((val, i) => {
        const offset = ptr + i * 4;
        wasmMem.writeF32(offset, val);
        const read = wasmMem.readF32(offset);
        if (Number.isNaN(val)) {
          expect(Number.isNaN(read)).toBe(true);
        } else {
          expect(read).toBeCloseTo(val, 4);
        }
      });
    });
  });

  describe('F64 Operations', () => {
    it('should read and write f64', () => {
      const ptr = 4096;
      const value = 3.141592653589793;

      wasmMem.writeF64(ptr, value);
      const read = wasmMem.readF64(ptr);

      expect(read).toBeCloseTo(value, 10);
    });

    it('should handle f64 special values', () => {
      const ptr = 4104;
      const values = [0.0, -0.0, 1.5, -1.5, Infinity, -Infinity, Math.PI];

      values.forEach((val, i) => {
        const offset = ptr + i * 8;
        wasmMem.writeF64(offset, val);
        const read = wasmMem.readF64(offset);
        if (Number.isNaN(val)) {
          expect(Number.isNaN(read)).toBe(true);
        } else {
          expect(read).toBeCloseTo(val, 10);
        }
      });
    });
  });
});

describe('WasmMemory - Bounds Checking', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
    wasmMem = new WasmMemory(memory);
  });

  it('should reject negative pointer', () => {
    expect(() => wasmMem.readString(-1, 10)).toThrow(MemoryError);
  });

  it('should reject negative length', () => {
    expect(() => wasmMem.readString(1024, -1)).toThrow(MemoryError);
  });

  it('should reject out-of-bounds read', () => {
    const bufferSize = memory.buffer.byteLength;
    expect(() => wasmMem.readString(bufferSize, 1)).toThrow(MemoryError);
  });

  it('should reject out-of-bounds write', () => {
    const bufferSize = memory.buffer.byteLength;
    expect(() => wasmMem.writeString(bufferSize, 'x')).toThrow(MemoryError);
  });

  it('should reject out-of-bounds isValidRange', () => {
    const bufferSize = memory.buffer.byteLength;
    expect(() => wasmMem.isValidRange(bufferSize, 1)).toThrow(MemoryError);
  });

  it('should include error context in MemoryError', () => {
    try {
      wasmMem.readBytes(1000000, 100);
      expect.fail('Should have thrown MemoryError');
    } catch (error) {
      expect(error).toBeInstanceOf(MemoryError);
      const memErr = error as MemoryError;
      expect(memErr.ptr).toBe(1000000);
      expect(memErr.length).toBe(100);
    }
  });
});

describe('WasmMemory - Memory Growth', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
    wasmMem = new WasmMemory(memory);
  });

  it('should grow memory', () => {
    const initialSize = wasmMem.size();
    const initialPages = wasmMem.pages();

    const result = wasmMem.grow(2);

    expect(result).toBe(initialPages);
    expect(wasmMem.pages()).toBe(initialPages + 2);
    expect(wasmMem.size()).toBe(initialSize + 131072); // 2 * 64KB
  });

  it('should handle multiple growths', () => {
    const page1 = wasmMem.pages();
    wasmMem.grow(1);
    const page2 = wasmMem.pages();
    wasmMem.grow(1);
    const page3 = wasmMem.pages();

    expect(page2).toBe(page1 + 1);
    expect(page3).toBe(page2 + 1);
  });

  it('should reject negative growth', () => {
    expect(() => wasmMem.grow(-1)).toThrow(MemoryError);
  });

  it('should allow zero growth', () => {
    const pages = wasmMem.pages();
    const result = wasmMem.grow(0);
    expect(result).toBe(pages);
    expect(wasmMem.pages()).toBe(pages);
  });
});

describe('WasmMemory - Size Queries', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 2 }); // 128KB
    wasmMem = new WasmMemory(memory);
  });

  it('should report correct size in bytes', () => {
    expect(wasmMem.size()).toBe(131072); // 2 * 64KB
  });

  it('should report correct size in pages', () => {
    expect(wasmMem.pages()).toBe(2);
  });

  it('should update size after growth', () => {
    wasmMem.grow(3);
    expect(wasmMem.pages()).toBe(5);
    expect(wasmMem.size()).toBe(327680); // 5 * 64KB
  });
});

describe('WasmMemory - View Creation', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 2 });
    wasmMem = new WasmMemory(memory);
  });

  it('should create DataView', () => {
    const ptr = 1024;
    const length = 16;

    const view = wasmMem.createDataView(ptr, length);

    expect(view).toBeInstanceOf(DataView);
    expect(view.byteLength).toBe(length);
    expect(view.byteOffset).toBe(ptr);
  });

  it('should create Uint8Array view', () => {
    const ptr = 2048;
    const length = 32;

    const view = wasmMem.createByteView(ptr, length);

    expect(view).toBeInstanceOf(Uint8Array);
    expect(view.length).toBe(length);
    expect(view.byteOffset).toBe(ptr);
  });

  it('should reject invalid views', () => {
    expect(() => wasmMem.createDataView(1000000, 100)).toThrow(MemoryError);
    expect(() => wasmMem.createByteView(1000000, 100)).toThrow(MemoryError);
  });

  it('should allow writing through views', () => {
    const ptr = 3072;
    const view = wasmMem.createByteView(ptr, 4);

    view[0] = 1;
    view[1] = 2;
    view[2] = 3;
    view[3] = 4;

    const read = wasmMem.readBytes(ptr, 4);
    expect(read).toEqual(new Uint8Array([1, 2, 3, 4]));
  });
});

describe('WasmMemory - Access to Raw Memory', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 });
    wasmMem = new WasmMemory(memory);
  });

  it('should provide access to raw WebAssembly.Memory', () => {
    const raw = wasmMem.getRawMemory();
    expect(raw).toBe(memory);
  });

  it('should have consistent buffer between raw and interface', () => {
    const raw = wasmMem.getRawMemory();
    expect(raw.buffer).toBe(memory.buffer);
    expect(wasmMem.size()).toBe(raw.buffer.byteLength);
  });
});

describe('WasmMemory - Mixed Operations', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 2 });
    wasmMem = new WasmMemory(memory);
  });

  it('should handle interleaved reads and writes', () => {
    const ptr1 = 1024;
    const ptr2 = 2048;

    wasmMem.writeI32(ptr1, 42);
    wasmMem.writeF64(ptr2, 3.14);

    expect(wasmMem.readI32(ptr1)).toBe(42);
    expect(wasmMem.readF64(ptr2)).toBeCloseTo(3.14);
    expect(wasmMem.readI32(ptr1)).toBe(42); // Verify still correct
  });

  it('should handle overlapping memory regions', () => {
    const ptr = 1024;
    const str = 'Hello';

    wasmMem.writeString(ptr, str);
    const written = str.length;

    // Read back individual bytes
    const bytes = wasmMem.readBytes(ptr, written);
    expect(bytes).toEqual(new TextEncoder().encode(str));
  });

  it('should preserve data across multiple memory regions', () => {
    const regions = [
      { ptr: 1024, data: 'region1' },
      { ptr: 2048, data: 'region2' },
      { ptr: 3072, data: 'region3' },
    ];

    regions.forEach((r) => wasmMem.writeString(r.ptr, r.data));
    regions.forEach((r) => {
      const len = new TextEncoder().encode(r.data).length;
      expect(wasmMem.readString(r.ptr, len)).toBe(r.data);
    });
  });
});
