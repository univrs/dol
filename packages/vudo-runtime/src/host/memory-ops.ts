/**
 * Memory Host Functions Implementation
 *
 * Implements the 3 memory management host functions:
 * - vudo_alloc: Allocate memory
 * - vudo_free: Free allocated memory
 * - vudo_realloc: Reallocate (resize) memory
 *
 * These functions provide the allocator interface to WASM modules.
 *
 * @module @vudo/runtime/host/memory-ops
 */

import { ResultCode } from '../abi/types.js';
import type { IWasmMemory } from './interfaces.js';
import { HostFunctionError } from './interfaces.js';

/**
 * Configuration for memory host functions
 */
export interface MemoryOpsConfig {
  /** Memory interface for allocation operations */
  memory: IWasmMemory;
  /** Maximum single allocation size (default 10MB) */
  maxAllocationSize?: number;
  /** Enable debug logging for allocations */
  debug?: boolean;
}

/**
 * Memory host function provider
 *
 * Delegates to the IWasmMemory allocator while providing
 * validation, error handling, and debugging support.
 */
export class MemoryHostFunctions {
  private memory: IWasmMemory;
  private maxAllocationSize: number;
  private debug: boolean;
  private allocationCount = 0;
  private totalAllocated = 0;
  private peakMemoryUsage = 0;

  constructor(config: MemoryOpsConfig) {
    this.memory = config.memory;
    this.maxAllocationSize = config.maxAllocationSize ?? 10 * 1024 * 1024; // 10MB
    this.debug = config.debug ?? false;
  }

  /**
   * Validate allocation size
   * @param size - Requested allocation size
   * @throws HostFunctionError if size is invalid
   */
  private validateSize(size: number): void {
    if (size < 0) {
      throw new HostFunctionError(
        `Invalid allocation size: ${size} (must be non-negative)`,
        ResultCode.InvalidArg
      );
    }

    if (size === 0) {
      throw new HostFunctionError(
        'Cannot allocate zero bytes',
        ResultCode.InvalidArg
      );
    }

    if (size > this.maxAllocationSize) {
      throw new HostFunctionError(
        `Allocation size ${size} exceeds maximum ${this.maxAllocationSize}`,
        ResultCode.InvalidArg
      );
    }
  }

  /**
   * Validate pointer
   * @param ptr - Pointer to validate
   * @throws HostFunctionError if pointer is invalid
   */
  private validatePointer(ptr: number): void {
    if (ptr < 0) {
      throw new HostFunctionError(
        `Invalid pointer: ${ptr} (must be non-negative)`,
        ResultCode.InvalidArg
      );
    }

    if (ptr === 0) {
      throw new HostFunctionError(
        'Null pointer (0) is invalid for memory operations',
        ResultCode.InvalidArg
      );
    }

    if (ptr >= this.memory.buffer.byteLength) {
      throw new HostFunctionError(
        `Pointer ${ptr} exceeds memory bounds ${this.memory.buffer.byteLength}`,
        ResultCode.InvalidArg
      );
    }
  }

  /**
   * Update memory usage statistics
   */
  private updateStats(delta: number): void {
    this.totalAllocated += delta;
    if (this.totalAllocated > this.peakMemoryUsage) {
      this.peakMemoryUsage = this.totalAllocated;
    }
  }

  /**
   * Log debug message if debug mode is enabled
   */
  private debugLog(message: string): void {
    if (this.debug) {
      console.debug(`[MemoryHostFunctions] ${message}`);
    }
  }

  /**
   * Allocate memory from the host allocator
   *
   * Implements vudo_alloc(size: i32) -> i32
   *
   * @param size - Number of bytes to allocate
   * @returns Pointer to allocated memory (0 on failure)
   */
  alloc(size: number): number {
    try {
      this.validateSize(size);

      const ptr = this.memory.alloc(size);

      if (ptr === 0) {
        this.debugLog(`Allocation failed for ${size} bytes (out of memory)`);
        return 0;
      }

      this.allocationCount++;
      this.updateStats(size);
      this.debugLog(
        `Allocated ${size} bytes at 0x${ptr.toString(16)} (total: ${this.totalAllocated} bytes, count: ${this.allocationCount})`
      );

      return ptr;
    } catch (error) {
      if (error instanceof HostFunctionError) {
        this.debugLog(`Allocation error: ${error.message}`);
      } else {
        this.debugLog(
          `Unexpected allocation error: ${error instanceof Error ? error.message : String(error)}`
        );
      }
      return 0; // Return 0 on any error
    }
  }

  /**
   * Free previously allocated memory
   *
   * Implements vudo_free(ptr: i32, size: i32) -> void
   *
   * @param ptr - Pointer to memory to free
   * @param size - Size of allocation (must match original)
   */
  free(ptr: number, size: number): void {
    try {
      // Allow freeing null pointer (common idiom)
      if (ptr === 0) {
        this.debugLog('Attempted to free null pointer (ignored)');
        return;
      }

      this.validatePointer(ptr);
      this.validateSize(size);

      this.memory.free(ptr, size);

      this.allocationCount--;
      this.updateStats(-size);
      this.debugLog(
        `Freed ${size} bytes at 0x${ptr.toString(16)} (total: ${this.totalAllocated} bytes, count: ${this.allocationCount})`
      );
    } catch (error) {
      // Log but don't throw - free operations should be resilient
      this.debugLog(
        `Free error at ${ptr}: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * Reallocate memory (grow or shrink)
   *
   * Implements vudo_realloc(ptr: i32, old_size: i32, new_size: i32) -> i32
   *
   * @param ptr - Pointer to existing allocation
   * @param oldSize - Current size of allocation
   * @param newSize - Desired new size
   * @returns New pointer (0 on failure)
   */
  realloc(ptr: number, oldSize: number, newSize: number): number {
    try {
      this.validatePointer(ptr);
      this.validateSize(oldSize);
      this.validateSize(newSize);

      const newPtr = this.memory.realloc(ptr, oldSize, newSize);

      if (newPtr === 0) {
        this.debugLog(
          `Reallocation failed from ${oldSize} to ${newSize} bytes at 0x${ptr.toString(16)}`
        );
        return 0;
      }

      // Update stats (net change in allocated bytes)
      this.updateStats(newSize - oldSize);
      this.debugLog(
        `Reallocated ${oldSize} -> ${newSize} bytes: 0x${ptr.toString(16)} -> 0x${newPtr.toString(16)} (total: ${this.totalAllocated} bytes)`
      );

      return newPtr;
    } catch (error) {
      if (error instanceof HostFunctionError) {
        this.debugLog(`Reallocation error: ${error.message}`);
      } else {
        this.debugLog(
          `Unexpected reallocation error: ${error instanceof Error ? error.message : String(error)}`
        );
      }
      return 0; // Return 0 on any error, original ptr remains valid
    }
  }

  /**
   * Get allocation statistics
   *
   * @returns Object with allocation metrics
   */
  getStats(): {
    allocationCount: number;
    totalAllocated: number;
    peakMemoryUsage: number;
  } {
    return {
      allocationCount: this.allocationCount,
      totalAllocated: this.totalAllocated,
      peakMemoryUsage: this.peakMemoryUsage,
    };
  }

  /**
   * Reset allocation statistics
   */
  resetStats(): void {
    this.allocationCount = 0;
    this.totalAllocated = 0;
    this.peakMemoryUsage = 0;
    this.debugLog('Statistics reset');
  }

  /**
   * Build WebAssembly imports object for memory functions
   *
   * @returns Object with memory host functions bound to this instance
   */
  buildImports(): Record<string, Function> {
    return {
      vudo_alloc: this.alloc.bind(this),
      vudo_free: this.free.bind(this),
      vudo_realloc: this.realloc.bind(this),
    };
  }
}

/**
 * Create memory host functions with default configuration
 *
 * @param memory - WASM memory interface
 * @param debug - Enable debug logging
 * @returns Configured MemoryHostFunctions instance
 */
export function createMemoryHostFunctions(
  memory: IWasmMemory,
  debug = false
): MemoryHostFunctions {
  return new MemoryHostFunctions({ memory, debug });
}
