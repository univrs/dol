/**
 * Tests for memory host functions
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { MemoryHostFunctions } from '../../src/host/memory-ops.js';
import { ResultCode } from '../../src/abi/types.js';
import { MockWasmMemory } from './test-utils.js';

describe('MemoryHostFunctions', () => {
  let memory: MockWasmMemory;
  let memOps: MemoryHostFunctions;

  beforeEach(() => {
    memory = new MockWasmMemory(1024 * 1024); // 1MB
    memOps = new MemoryHostFunctions({ memory });
  });

  describe('vudo_alloc', () => {
    it('should allocate memory successfully', () => {
      const ptr = memOps.alloc(256);

      expect(ptr).toBeGreaterThan(0);
      expect(ptr).toBeLessThan(memory.buffer.byteLength);
    });

    it('should allocate multiple blocks', () => {
      const ptr1 = memOps.alloc(128);
      const ptr2 = memOps.alloc(256);
      const ptr3 = memOps.alloc(64);

      expect(ptr1).toBeGreaterThan(0);
      expect(ptr2).toBeGreaterThan(ptr1);
      expect(ptr3).toBeGreaterThan(ptr2);
    });

    it('should return 0 for negative size', () => {
      const ptr = memOps.alloc(-100);

      expect(ptr).toBe(0);
    });

    it('should return 0 for zero size', () => {
      const ptr = memOps.alloc(0);

      expect(ptr).toBe(0);
    });

    it('should return 0 when size exceeds maximum', () => {
      const ptr = memOps.alloc(11 * 1024 * 1024); // 11MB > 10MB default max

      expect(ptr).toBe(0);
    });

    it('should return 0 when out of memory', () => {
      // Allocate almost all memory
      const largePtr = memOps.alloc(1024 * 1024 - 2048);
      expect(largePtr).toBeGreaterThan(0);

      // Try to allocate more than available
      const failPtr = memOps.alloc(10000);
      expect(failPtr).toBe(0);
    });

    it('should track allocation statistics', () => {
      memOps.alloc(100);
      memOps.alloc(200);
      memOps.alloc(300);

      const stats = memOps.getStats();
      expect(stats.allocationCount).toBe(3);
      expect(stats.totalAllocated).toBe(600);
      expect(stats.peakMemoryUsage).toBe(600);
    });

    it('should handle very small allocations', () => {
      const ptr = memOps.alloc(1);

      expect(ptr).toBeGreaterThan(0);
    });

    it('should handle page-sized allocations', () => {
      const ptr = memOps.alloc(4096); // 4KB page

      expect(ptr).toBeGreaterThan(0);
    });
  });

  describe('vudo_free', () => {
    it('should free allocated memory', () => {
      const ptr = memOps.alloc(256);
      expect(ptr).toBeGreaterThan(0);

      memOps.free(ptr, 256);

      const stats = memOps.getStats();
      expect(stats.allocationCount).toBe(0);
      expect(stats.totalAllocated).toBe(0);
    });

    it('should handle freeing null pointer gracefully', () => {
      // Should not throw
      memOps.free(0, 100);

      const stats = memOps.getStats();
      expect(stats.allocationCount).toBe(0);
    });

    it('should handle multiple frees', () => {
      const ptr1 = memOps.alloc(100);
      const ptr2 = memOps.alloc(200);
      const ptr3 = memOps.alloc(300);

      memOps.free(ptr1, 100);
      memOps.free(ptr2, 200);
      memOps.free(ptr3, 300);

      const stats = memOps.getStats();
      expect(stats.allocationCount).toBe(0);
      expect(stats.totalAllocated).toBe(0);
    });

    it('should handle invalid pointer gracefully', () => {
      // Should not throw - memory ops should be resilient
      memOps.free(-1, 100);
      memOps.free(999999, 100);
    });

    it('should update statistics correctly', () => {
      const ptr = memOps.alloc(500);
      expect(memOps.getStats().totalAllocated).toBe(500);

      memOps.free(ptr, 500);
      expect(memOps.getStats().totalAllocated).toBe(0);
    });
  });

  describe('vudo_realloc', () => {
    it('should reallocate to larger size', () => {
      const ptr = memOps.alloc(100);
      const newPtr = memOps.realloc(ptr, 100, 200);

      expect(newPtr).toBeGreaterThan(0);
      expect(newPtr).not.toBe(ptr); // May move
    });

    it('should reallocate to smaller size', () => {
      const ptr = memOps.alloc(200);
      const newPtr = memOps.realloc(ptr, 200, 100);

      expect(newPtr).toBeGreaterThan(0);
    });

    it('should preserve data when growing', () => {
      const ptr = memOps.alloc(10);
      const testData = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
      memory.u8.set(testData, ptr);

      const newPtr = memOps.realloc(ptr, 10, 20);
      const preserved = Array.from(memory.u8.slice(newPtr, newPtr + 10));

      expect(preserved).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    });

    it('should preserve data when shrinking', () => {
      const ptr = memOps.alloc(20);
      const testData = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
      memory.u8.set(testData, ptr);

      const newPtr = memOps.realloc(ptr, 20, 5);
      const preserved = Array.from(memory.u8.slice(newPtr, newPtr + 5));

      expect(preserved).toEqual([1, 2, 3, 4, 5]);
    });

    it('should return 0 for invalid pointer', () => {
      const newPtr = memOps.realloc(-1, 100, 200);

      expect(newPtr).toBe(0);
    });

    it('should return 0 for invalid old size', () => {
      const ptr = memOps.alloc(100);
      const newPtr = memOps.realloc(ptr, -100, 200);

      expect(newPtr).toBe(0);
    });

    it('should return 0 for invalid new size', () => {
      const ptr = memOps.alloc(100);
      const newPtr = memOps.realloc(ptr, 100, -200);

      expect(newPtr).toBe(0);
    });

    it('should return 0 when new size exceeds maximum', () => {
      const ptr = memOps.alloc(100);
      const newPtr = memOps.realloc(ptr, 100, 11 * 1024 * 1024);

      expect(newPtr).toBe(0);
    });

    it('should update statistics correctly', () => {
      const ptr = memOps.alloc(100);
      expect(memOps.getStats().totalAllocated).toBe(100);

      const newPtr = memOps.realloc(ptr, 100, 200);
      expect(memOps.getStats().totalAllocated).toBe(200);
    });

    it('should handle shrinking statistics', () => {
      const ptr = memOps.alloc(200);
      const newPtr = memOps.realloc(ptr, 200, 100);

      expect(memOps.getStats().totalAllocated).toBe(100);
    });
  });

  describe('statistics', () => {
    it('should track allocation count', () => {
      memOps.alloc(100);
      memOps.alloc(200);

      expect(memOps.getStats().allocationCount).toBe(2);
    });

    it('should track total allocated bytes', () => {
      memOps.alloc(100);
      memOps.alloc(200);
      memOps.alloc(300);

      expect(memOps.getStats().totalAllocated).toBe(600);
    });

    it('should track peak memory usage', () => {
      memOps.alloc(100);
      memOps.alloc(200); // Total: 300
      const ptr = memOps.alloc(400); // Total: 700

      expect(memOps.getStats().peakMemoryUsage).toBe(700);

      memOps.free(ptr, 400); // Total: 300

      expect(memOps.getStats().peakMemoryUsage).toBe(700); // Still 700
    });

    it('should reset statistics', () => {
      memOps.alloc(100);
      memOps.alloc(200);

      memOps.resetStats();

      const stats = memOps.getStats();
      expect(stats.allocationCount).toBe(0);
      expect(stats.totalAllocated).toBe(0);
      expect(stats.peakMemoryUsage).toBe(0);
    });
  });

  describe('buildImports', () => {
    it('should return all memory functions', () => {
      const imports = memOps.buildImports();

      expect(imports).toHaveProperty('vudo_alloc');
      expect(imports).toHaveProperty('vudo_free');
      expect(imports).toHaveProperty('vudo_realloc');
    });

    it('should bind functions correctly', () => {
      const imports = memOps.buildImports();

      const ptr = (imports.vudo_alloc as Function)(256);
      expect(ptr).toBeGreaterThan(0);
    });
  });

  describe('debug mode', () => {
    it('should log allocations in debug mode', () => {
      const debugOps = new MemoryHostFunctions({
        memory,
        debug: true,
      });

      // Capture console output
      const logs: string[] = [];
      const originalDebug = console.debug;
      console.debug = (msg: string) => logs.push(msg);

      debugOps.alloc(100);

      console.debug = originalDebug;

      expect(logs.some((log) => log.includes('Allocated 100 bytes'))).toBe(true);
    });
  });

  describe('custom configuration', () => {
    it('should respect custom max allocation size', () => {
      const customOps = new MemoryHostFunctions({
        memory,
        maxAllocationSize: 1000,
      });

      const ptr = customOps.alloc(1001);

      expect(ptr).toBe(0);
    });

    it('should allow allocations up to max size', () => {
      const customOps = new MemoryHostFunctions({
        memory,
        maxAllocationSize: 1000,
      });

      const ptr = customOps.alloc(1000);

      expect(ptr).toBeGreaterThan(0);
    });
  });

  describe('edge cases', () => {
    it('should handle rapid alloc/free cycles', () => {
      for (let i = 0; i < 100; i++) {
        const ptr = memOps.alloc(10);
        memOps.free(ptr, 10);
      }

      const stats = memOps.getStats();
      expect(stats.allocationCount).toBe(0);
      expect(stats.totalAllocated).toBe(0);
    });

    it('should handle interleaved operations', () => {
      const ptr1 = memOps.alloc(100);
      const ptr2 = memOps.alloc(200);
      memOps.free(ptr1, 100);
      const ptr3 = memOps.alloc(150);
      memOps.free(ptr2, 200);
      memOps.free(ptr3, 150);

      expect(memOps.getStats().allocationCount).toBe(0);
    });

    it('should handle realloc to same size', () => {
      const ptr = memOps.alloc(100);
      const newPtr = memOps.realloc(ptr, 100, 100);

      expect(newPtr).toBeGreaterThan(0);
      expect(memOps.getStats().totalAllocated).toBe(100);
    });
  });
});
