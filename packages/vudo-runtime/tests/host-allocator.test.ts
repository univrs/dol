/**
 * Host Allocator tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  HostBumpAllocator,
  HostStackAllocator,
  MEMORY_LAYOUT,
  type AllocationStats,
} from '../src/host/allocator.js';
import { WasmMemory, MemoryError } from '../src/host/memory.js';

describe('MEMORY_LAYOUT Constants', () => {
  it('should define correct memory layout constants', () => {
    expect(MEMORY_LAYOUT.HEAP_BASE).toBe(0x10000); // 65536
    expect(MEMORY_LAYOUT.STACK_SIZE).toBe(0x8000); // 32768
    expect(MEMORY_LAYOUT.PAGE_SIZE).toBe(0x10000); // 65536
  });

  it('should have heap base greater than null pointer region', () => {
    expect(MEMORY_LAYOUT.HEAP_BASE).toBeGreaterThanOrEqual(0x10000);
  });
});

describe('HostBumpAllocator - Basic Allocation', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostBumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 }); // 256KB
    wasmMem = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMem);
  });

  it('should allocate from heap base', () => {
    const ptr = allocator.alloc(32);
    expect(ptr).toBe(MEMORY_LAYOUT.HEAP_BASE);
  });

  it('should allocate sequentially', () => {
    const ptr1 = allocator.alloc(32);
    const ptr2 = allocator.alloc(32);
    const ptr3 = allocator.alloc(32);

    expect(ptr2).toBe(ptr1 + 32);
    expect(ptr3).toBe(ptr2 + 32);
  });

  it('should respect alignment', () => {
    const ptr1 = allocator.alloc(10, 8); // Allocate 10 bytes with 8-byte alignment
    expect(ptr1 % 8).toBe(0); // Should be aligned to 8

    const ptr2 = allocator.alloc(10, 16); // Next allocation with 16-byte alignment
    expect(ptr2 % 16).toBe(0); // Should be aligned to 16
  });

  it('should handle various alignments', () => {
    const alignments = [1, 2, 4, 8, 16, 32, 64];

    alignments.forEach((align) => {
      allocator.reset();
      const ptr = allocator.alloc(1, align);
      expect(ptr % align).toBe(0);
    });
  });

  it('should reject invalid alignment values', () => {
    expect(() => allocator.alloc(10, 0)).toThrow(MemoryError);
    expect(() => allocator.alloc(10, -1)).toThrow(MemoryError);
    expect(() => allocator.alloc(10, 3)).toThrow(MemoryError); // Not power of 2
    expect(() => allocator.alloc(10, 5)).toThrow(MemoryError); // Not power of 2
  });

  it('should reject invalid size values', () => {
    expect(() => allocator.alloc(0)).toThrow(MemoryError);
    expect(() => allocator.alloc(-1)).toThrow(MemoryError);
  });
});

describe('HostBumpAllocator - Memory Growth', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostBumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
    wasmMem = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMem);
  });

  it('should grow memory when needed', () => {
    const initialPages = wasmMem.pages();

    // Allocate more than initial memory
    allocator.alloc(100000);

    expect(wasmMem.pages()).toBeGreaterThan(initialPages);
  });

  it('should track growth count', () => {
    let stats = allocator.getStats();
    const initialGrowth = stats.growthCount;

    allocator.alloc(100000);

    stats = allocator.getStats();
    expect(stats.growthCount).toBeGreaterThan(initialGrowth);
  });

  it('should handle multiple sequential allocations with growth', () => {
    const ptrs: number[] = [];

    for (let i = 0; i < 10; i++) {
      const ptr = allocator.alloc(30000);
      ptrs.push(ptr);
    }

    // All pointers should be sequential
    for (let i = 1; i < ptrs.length; i++) {
      expect(ptrs[i]).toBeGreaterThan(ptrs[i - 1]);
    }
  });
});

describe('HostBumpAllocator - Statistics', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostBumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMem);
  });

  it('should track total allocated bytes', () => {
    let stats = allocator.getStats();
    expect(stats.totalAllocated).toBe(0);

    allocator.alloc(100);
    stats = allocator.getStats();
    expect(stats.totalAllocated).toBe(100);

    allocator.alloc(50);
    stats = allocator.getStats();
    expect(stats.totalAllocated).toBe(150);
  });

  it('should track active allocations', () => {
    let stats = allocator.getStats();
    expect(stats.activeAllocations).toBe(0);

    allocator.alloc(100);
    stats = allocator.getStats();
    expect(stats.activeAllocations).toBe(1);

    allocator.alloc(50);
    stats = allocator.getStats();
    expect(stats.activeAllocations).toBe(2);
  });

  it('should track peak usage', () => {
    let stats = allocator.getStats();
    const initialPeak = stats.peakUsage;

    allocator.alloc(50000);
    stats = allocator.getStats();
    expect(stats.peakUsage).toBeGreaterThan(initialPeak);

    const peak1 = stats.peakUsage;

    allocator.alloc(100000);
    stats = allocator.getStats();
    expect(stats.peakUsage).toBeGreaterThan(peak1);
  });

  it('should track current offset', () => {
    const offset1 = allocator.offset;
    expect(offset1).toBe(MEMORY_LAYOUT.HEAP_BASE);

    allocator.alloc(1000);
    const offset2 = allocator.offset;
    expect(offset2).toBeGreaterThan(offset1);
  });

  it('should return complete stats object', () => {
    allocator.alloc(100);
    allocator.alloc(50);

    const stats = allocator.getStats();

    expect(stats).toHaveProperty('totalAllocated');
    expect(stats).toHaveProperty('activeAllocations');
    expect(stats).toHaveProperty('peakUsage');
    expect(stats).toHaveProperty('currentOffset');
    expect(stats).toHaveProperty('growthCount');
  });
});

describe('HostBumpAllocator - Tagging', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostBumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMem);
  });

  it('should record allocation with tag', () => {
    const ptr = allocator.alloc(100, 8, 'my-buffer');
    const info = allocator.getAllocationInfo(ptr);

    expect(info).toBeDefined();
    expect(info?.tag).toBe('my-buffer');
    expect(info?.size).toBe(100);
    expect(info?.ptr).toBe(ptr);
  });

  it('should work without tag', () => {
    const ptr = allocator.alloc(100);
    const info = allocator.getAllocationInfo(ptr);

    expect(info).toBeDefined();
    expect(info?.tag).toBeUndefined();
  });

  it('should list all allocations', () => {
    allocator.alloc(100, 8, 'alloc1');
    allocator.alloc(200, 8, 'alloc2');
    allocator.alloc(300, 8, 'alloc3');

    const allocs = allocator.getAllocations();
    expect(allocs.length).toBe(3);
    expect(allocs[0].tag).toBe('alloc1');
    expect(allocs[1].tag).toBe('alloc2');
    expect(allocs[2].tag).toBe('alloc3');
  });

  it('should get null for unknown allocations', () => {
    allocator.alloc(100);
    const info = allocator.getAllocationInfo(99999);
    expect(info).toBeUndefined();
  });
});

describe('HostBumpAllocator - Reset', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostBumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMem);
  });

  it('should reset to base offset', () => {
    allocator.alloc(10000);
    expect(allocator.offset).toBeGreaterThan(MEMORY_LAYOUT.HEAP_BASE);

    allocator.reset();
    expect(allocator.offset).toBe(MEMORY_LAYOUT.HEAP_BASE);
  });

  it('should clear allocation records on reset', () => {
    allocator.alloc(100);
    allocator.alloc(200);

    expect(allocator.getAllocations().length).toBe(2);

    allocator.reset();
    expect(allocator.getAllocations().length).toBe(0);
  });

  it('should allow reallocation after reset', () => {
    const ptr1 = allocator.alloc(100);
    allocator.reset();
    const ptr2 = allocator.alloc(100);

    expect(ptr1).toBe(ptr2);
  });

  it('should reset active allocations count', () => {
    allocator.alloc(100);
    expect(allocator.getStats().activeAllocations).toBe(1);

    allocator.reset();
    expect(allocator.getStats().activeAllocations).toBe(0);
  });
});

describe('HostBumpAllocator - String Allocation', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostBumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMem);
  });

  it('should allocate space for ASCII strings', () => {
    const str = 'Hello';
    const ptr = allocator.allocString(str);

    expect(ptr).toBeDefined();
    expect(ptr).toBe(MEMORY_LAYOUT.HEAP_BASE);
  });

  it('should allocate sufficient space for UTF-8', () => {
    const str = '你好'; // 2 characters, 6 bytes in UTF-8
    const ptr = allocator.allocString(str);

    // Should allocate at least 8 bytes (4 bytes per char max)
    expect(allocator.offset - ptr).toBeGreaterThanOrEqual(8);
  });

  it('should use 4-byte alignment for strings', () => {
    const ptr = allocator.allocString('test');
    expect(ptr % 4).toBe(0);
  });
});

describe('HostBumpAllocator - Custom Base Offset', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
  });

  it('should use custom base offset', () => {
    const customBase = 0x20000;
    const allocator = new HostBumpAllocator(wasmMem, customBase);

    const ptr = allocator.alloc(100);
    expect(ptr).toBe(customBase);
  });

  it('should track used bytes from custom base', () => {
    const customBase = 0x20000;
    const allocator = new HostBumpAllocator(wasmMem, customBase);

    allocator.alloc(100);
    expect(allocator.used).toBe(100);
  });
});

describe('HostBumpAllocator - Free (No-Op)', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostBumpAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostBumpAllocator(wasmMem);
  });

  it('should accept free calls (no-op)', () => {
    const ptr = allocator.alloc(100);
    expect(() => allocator.free(ptr)).not.toThrow();
  });

  it('should not change offset after free', () => {
    allocator.alloc(100);
    const offsetBefore = allocator.offset;

    allocator.free(123);

    expect(allocator.offset).toBe(offsetBefore);
  });
});

describe('HostStackAllocator - Basic Operations', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostStackAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostStackAllocator(wasmMem);
  });

  it('should allocate from stack base', () => {
    const ptr = allocator.alloc(32);
    expect(ptr).toBe(MEMORY_LAYOUT.HEAP_BASE);
  });

  it('should allocate sequentially on stack', () => {
    const ptr1 = allocator.alloc(32);
    const ptr2 = allocator.alloc(32);

    expect(ptr2).toBe(ptr1 + 32);
  });

  it('should respect alignment', () => {
    const ptr1 = allocator.alloc(10, 8);
    expect(ptr1 % 8).toBe(0);

    const ptr2 = allocator.alloc(10, 16);
    expect(ptr2 % 16).toBe(0);
  });

  it('should track used bytes', () => {
    expect(allocator.used).toBe(0);

    allocator.alloc(100);
    expect(allocator.used).toBe(100);

    allocator.alloc(100);
    // Due to alignment, the second allocation may use more space
    expect(allocator.used).toBeGreaterThanOrEqual(200);
  });

  it('should provide capacity', () => {
    expect(allocator.capacity).toBe(MEMORY_LAYOUT.STACK_SIZE);
  });
});

describe('HostStackAllocator - Mark and Pop', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostStackAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostStackAllocator(wasmMem);
  });

  it('should mark stack position', () => {
    allocator.alloc(100);
    expect(() => allocator.mark()).not.toThrow();
  });

  it('should pop back to marked position', () => {
    allocator.alloc(100);
    allocator.mark();

    const usedAfterMark = allocator.used;

    allocator.alloc(200);
    expect(allocator.used).toBeGreaterThan(usedAfterMark);

    allocator.pop();
    expect(allocator.used).toBe(usedAfterMark);
  });

  it('should support nested marks', () => {
    allocator.alloc(100);
    allocator.mark(); // Mark 1

    allocator.alloc(100);
    allocator.mark(); // Mark 2

    allocator.alloc(100);
    const usedBefore = allocator.used;

    allocator.pop(); // Pop to Mark 2
    expect(allocator.used).toBeLessThan(usedBefore);

    allocator.pop(); // Pop to Mark 1
    expect(allocator.used).toBeLessThan(200);
  });

  it('should throw on pop with no marks', () => {
    expect(() => allocator.pop()).toThrow();
  });

  it('should throw when marks exhausted', () => {
    allocator.mark();
    allocator.pop();
    expect(() => allocator.pop()).toThrow();
  });
});

describe('HostStackAllocator - Stack Overflow', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
  });

  it('should detect stack overflow', () => {
    const stackSize = MEMORY_LAYOUT.STACK_SIZE;
    const allocator = new HostStackAllocator(
      wasmMem,
      MEMORY_LAYOUT.HEAP_BASE,
      stackSize,
    );

    // Allocate up to capacity
    allocator.alloc(stackSize - 100);

    // Next allocation should overflow
    expect(() => allocator.alloc(200)).toThrow(MemoryError);
  });

  it('should include overflow context in error', () => {
    const allocator = new HostStackAllocator(
      wasmMem,
      MEMORY_LAYOUT.HEAP_BASE,
      1000,
    );

    allocator.alloc(900);

    try {
      allocator.alloc(200);
      expect.fail('Should have thrown');
    } catch (error) {
      expect(error).toBeInstanceOf(MemoryError);
      expect((error as Error).message).toContain('Stack overflow');
    }
  });
});

describe('HostStackAllocator - Reset', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostStackAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostStackAllocator(wasmMem);
  });

  it('should reset stack position', () => {
    allocator.alloc(1000);
    expect(allocator.used).toBeGreaterThan(0);

    allocator.reset();
    expect(allocator.used).toBe(0);
  });

  it('should clear marks on reset', () => {
    allocator.alloc(100);
    allocator.mark();

    allocator.reset();
    expect(() => allocator.pop()).toThrow();
  });

  it('should allow fresh allocation after reset', () => {
    const ptr1 = allocator.alloc(100);
    allocator.reset();
    const ptr2 = allocator.alloc(100);

    expect(ptr1).toBe(ptr2);
  });
});

describe('HostStackAllocator - Custom Size', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
  });

  it('should use custom stack size', () => {
    const customSize = 16384;
    const allocator = new HostStackAllocator(
      wasmMem,
      MEMORY_LAYOUT.HEAP_BASE,
      customSize,
    );

    expect(allocator.capacity).toBe(customSize);
  });

  it('should enforce custom stack size limit', () => {
    const customSize = 1024;
    const allocator = new HostStackAllocator(
      wasmMem,
      MEMORY_LAYOUT.HEAP_BASE,
      customSize,
    );

    allocator.alloc(900);
    expect(() => allocator.alloc(200)).toThrow(MemoryError);
  });
});

describe('HostStackAllocator - Stack Pointer', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostStackAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostStackAllocator(wasmMem);
  });

  it('should report correct top pointer', () => {
    expect(allocator.top).toBe(MEMORY_LAYOUT.HEAP_BASE);

    allocator.alloc(100);
    expect(allocator.top).toBe(MEMORY_LAYOUT.HEAP_BASE + 100);
  });

  it('should update top on allocation', () => {
    const top1 = allocator.top;
    allocator.alloc(50);
    const top2 = allocator.top;

    expect(top2).toBe(top1 + 50);
  });

  it('should update top on pop', () => {
    const base = allocator.top;
    allocator.alloc(100);
    allocator.mark();
    allocator.alloc(100);

    const topBefore = allocator.top;
    allocator.pop();
    const topAfter = allocator.top;

    expect(topBefore).toBeGreaterThan(topAfter);
    expect(topAfter).toBe(base + 100);
  });
});

describe('HostStackAllocator - Invalid Operations', () => {
  let memory: WebAssembly.Memory;
  let wasmMem: WasmMemory;
  let allocator: HostStackAllocator;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 4 });
    wasmMem = new WasmMemory(memory);
    allocator = new HostStackAllocator(wasmMem);
  });

  it('should reject invalid size', () => {
    expect(() => allocator.alloc(0)).toThrow(MemoryError);
    expect(() => allocator.alloc(-1)).toThrow(MemoryError);
  });

  it('should reject invalid alignment', () => {
    expect(() => allocator.alloc(10, 0)).toThrow(MemoryError);
    expect(() => allocator.alloc(10, 3)).toThrow(MemoryError); // Not power of 2
  });
});
