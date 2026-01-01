/**
 * Memory management tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { BumpAllocator, SpiritMemoryManager } from '../src/memory.js';
import {
  encodeString,
  decodeString,
  readGene,
  writeGene,
  calculateLayout,
  getTypeSize,
  getTypeAlignment,
} from '../src/utils/type-bridge.js';
import type { GeneLayout } from '../src/types.js';

describe('BumpAllocator', () => {
  let memory: WebAssembly.Memory;
  let allocator: BumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
    allocator = new BumpAllocator(memory, 1024);
  });

  it('should allocate memory with correct alignment', () => {
    const ptr1 = allocator.alloc(10, 8);
    expect(ptr1).toBe(1024); // Base offset, already aligned

    const ptr2 = allocator.alloc(10, 8);
    expect(ptr2).toBe(1040); // 1024 + 10 = 1034, aligned to 1040
  });

  it('should allocate memory sequentially', () => {
    const ptr1 = allocator.alloc(16);
    const ptr2 = allocator.alloc(16);
    const ptr3 = allocator.alloc(16);

    expect(ptr2).toBe(ptr1 + 16);
    expect(ptr3).toBe(ptr2 + 16);
  });

  it('should track offset correctly', () => {
    expect(allocator.offset).toBe(1024);

    allocator.alloc(100);
    expect(allocator.offset).toBe(1024 + 100);

    allocator.alloc(50);
    expect(allocator.offset).toBe(1024 + 100 + 56); // aligned to 8
  });

  it('should reset to base offset', () => {
    allocator.alloc(1000);
    expect(allocator.offset).toBeGreaterThan(1024);

    allocator.reset();
    expect(allocator.offset).toBe(1024);
  });

  it('should grow memory when needed', () => {
    const initialSize = memory.buffer.byteLength;

    // Allocate more than initial memory
    allocator.alloc(100000);

    expect(memory.buffer.byteLength).toBeGreaterThan(initialSize);
  });
});

describe('SpiritMemoryManager', () => {
  let memory: WebAssembly.Memory;
  let manager: SpiritMemoryManager;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 });
    manager = new SpiritMemoryManager(memory);
  });

  it('should allocate and track memory', () => {
    const ptr1 = manager.alloc(32);
    const ptr2 = manager.alloc(32);

    expect(ptr1).toBe(1024);
    expect(ptr2).toBe(1056);
  });

  it('should provide buffer access', () => {
    expect(manager.buffer).toBe(memory.buffer);
  });

  it('should provide typed views', () => {
    const views = manager.views;

    expect(views.i8).toBeInstanceOf(Int8Array);
    expect(views.u8).toBeInstanceOf(Uint8Array);
    expect(views.i32).toBeInstanceOf(Int32Array);
    expect(views.i64).toBeInstanceOf(BigInt64Array);
    expect(views.f64).toBeInstanceOf(Float64Array);
  });
});

describe('String encoding/decoding', () => {
  let memory: WebAssembly.Memory;
  let allocator: BumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 });
    allocator = new BumpAllocator(memory, 1024);
  });

  it('should encode and decode ASCII strings', () => {
    const original = 'Hello, World!';
    const ptr = encodeString(memory, allocator, original);
    const decoded = decodeString(memory, ptr);

    expect(decoded).toBe(original);
  });

  it('should encode and decode UTF-8 strings', () => {
    const original = 'Hello, 世界! 🌍';
    const ptr = encodeString(memory, allocator, original);
    const decoded = decodeString(memory, ptr);

    expect(decoded).toBe(original);
  });

  it('should handle empty strings', () => {
    const ptr = encodeString(memory, allocator, '');
    const decoded = decodeString(memory, ptr);

    expect(decoded).toBe('');
  });

  it('should store length prefix correctly', () => {
    const str = 'Test';
    const ptr = encodeString(memory, allocator, str);

    const view = new DataView(memory.buffer);
    const len = view.getUint32(ptr, true);

    expect(len).toBe(4); // 'Test' is 4 bytes
  });
});

describe('Gene read/write', () => {
  let memory: WebAssembly.Memory;

  const PointLayout: GeneLayout = {
    name: 'Point',
    fields: [
      { name: 'x', type: 'i64', offset: 0 },
      { name: 'y', type: 'i64', offset: 8 },
    ],
    size: 16,
    alignment: 8,
  };

  const MixedLayout: GeneLayout = {
    name: 'Mixed',
    fields: [
      { name: 'count', type: 'i32', offset: 0 },
      { name: 'active', type: 'bool', offset: 4 },
      { name: 'value', type: 'f64', offset: 8 },
    ],
    size: 16,
    alignment: 8,
  };

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 });
  });

  it('should write and read i64 fields', () => {
    const ptr = 1024;
    const values = { x: 42n, y: -100n };

    writeGene(memory, ptr, PointLayout, values);
    const result = readGene(memory, ptr, PointLayout);

    expect(result.x).toBe(42n);
    expect(result.y).toBe(-100n);
  });

  it('should write and read mixed field types', () => {
    const ptr = 1024;
    const values = { count: 123, active: true, value: 3.14 };

    writeGene(memory, ptr, MixedLayout, values);
    const result = readGene(memory, ptr, MixedLayout);

    expect(result.count).toBe(123);
    expect(result.active).toBe(true);
    expect(result.value).toBeCloseTo(3.14);
  });

  it('should handle boolean false', () => {
    const ptr = 1024;
    const values = { count: 0, active: false, value: 0.0 };

    writeGene(memory, ptr, MixedLayout, values);
    const result = readGene(memory, ptr, MixedLayout);

    expect(result.active).toBe(false);
  });

  it('should handle large i64 values', () => {
    const ptr = 1024;
    const large = 9007199254740993n; // Larger than MAX_SAFE_INTEGER
    const values = { x: large, y: -large };

    writeGene(memory, ptr, PointLayout, values);
    const result = readGene(memory, ptr, PointLayout);

    expect(result.x).toBe(large);
    expect(result.y).toBe(-large);
  });
});

describe('Layout calculation', () => {
  it('should calculate simple layout', () => {
    const layout = calculateLayout('Point', [
      { name: 'x', type: 'i64' },
      { name: 'y', type: 'i64' },
    ]);

    expect(layout.name).toBe('Point');
    expect(layout.size).toBe(16);
    expect(layout.alignment).toBe(8);
    expect(layout.fields[0].offset).toBe(0);
    expect(layout.fields[1].offset).toBe(8);
  });

  it('should handle mixed types with proper alignment', () => {
    const layout = calculateLayout('Mixed', [
      { name: 'a', type: 'i32' },  // 4 bytes
      { name: 'b', type: 'i64' },  // 8 bytes, needs alignment
      { name: 'c', type: 'i32' },  // 4 bytes
    ]);

    expect(layout.fields[0].offset).toBe(0);
    expect(layout.fields[1].offset).toBe(8);  // Aligned to 8
    expect(layout.fields[2].offset).toBe(16);
    expect(layout.size).toBe(24); // Aligned to 8
  });

  it('should return correct type sizes', () => {
    expect(getTypeSize('i32')).toBe(4);
    expect(getTypeSize('i64')).toBe(8);
    expect(getTypeSize('f32')).toBe(4);
    expect(getTypeSize('f64')).toBe(8);
    expect(getTypeSize('bool')).toBe(4);
    expect(getTypeSize('string')).toBe(4); // pointer
  });

  it('should return correct type alignments', () => {
    expect(getTypeAlignment('i32')).toBe(4);
    expect(getTypeAlignment('i64')).toBe(8);
    expect(getTypeAlignment('f32')).toBe(4);
    expect(getTypeAlignment('f64')).toBe(8);
  });
});
