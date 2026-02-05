/**
 * Host-side WASM Memory Allocator
 *
 * Provides memory allocation and management for host functions that need to
 * allocate space in WASM linear memory for passing data to Spirit functions.
 */

import type { WasmMemory } from './memory.js';
import { MemoryError } from './memory.js';

/**
 * Memory region constants for WASM linear memory layout
 */
export const MEMORY_LAYOUT = {
  /** Start of heap region (64KB reserved for null pointer safety) */
  HEAP_BASE: 0x10000, // 65536
  /** Size of stack region below heap */
  STACK_SIZE: 0x8000, // 32768
  /** Minimum heap growth increment (64KB page) */
  PAGE_SIZE: 0x10000, // 65536
} as const;

/**
 * Allocation statistics for monitoring and debugging
 */
export interface AllocationStats {
  /** Total bytes allocated by this allocator */
  totalAllocated: number;
  /** Number of active allocations */
  activeAllocations: number;
  /** Peak memory used since start or reset */
  peakUsage: number;
  /** Current offset into heap */
  currentOffset: number;
  /** Memory growth count */
  growthCount: number;
}

/**
 * Allocation record for tracking individual allocations
 */
interface AllocationRecord {
  ptr: number;
  size: number;
  timestamp: number;
  tag?: string;
}

/**
 * Host-side Bump Allocator for WASM memory
 *
 * Simple and efficient allocation strategy used by host functions to allocate
 * space in WASM linear memory. Allocations are sequential (bump pointer) with
 * support for alignment and automatic memory growth.
 *
 * Memory layout:
 * ```
 * 0x0000 - 0xFFFF:     Null pointer region (reserved)
 * 0x10000+:            Heap (managed by this allocator)
 * ```
 */
export class HostBumpAllocator {
  private wasmMemory: WasmMemory;
  private currentOffset: number;
  private baseOffset: number;
  private allocations: Map<number, AllocationRecord> = new Map();
  private stats: AllocationStats;

  /**
   * Create a new host-side bump allocator
   * @param wasmMemory - WasmMemory interface to manage
   * @param baseOffset - Starting offset (default: HEAP_BASE)
   */
  constructor(
    wasmMemory: WasmMemory,
    baseOffset: number = MEMORY_LAYOUT.HEAP_BASE,
  ) {
    this.wasmMemory = wasmMemory;
    this.baseOffset = baseOffset;
    this.currentOffset = baseOffset;
    this.stats = {
      totalAllocated: 0,
      activeAllocations: 0,
      peakUsage: 0,
      currentOffset: baseOffset,
      growthCount: 0,
    };
  }

  /**
   * Allocate bytes in WASM memory with optional alignment
   * @param size - Number of bytes to allocate
   * @param align - Alignment requirement in bytes (default: 8)
   * @param tag - Optional tag for debugging
   * @returns Pointer to allocated memory
   * @throws MemoryError if allocation fails
   */
  alloc(size: number, align: number = 8, tag?: string): number {
    if (size <= 0) {
      throw new MemoryError(`Invalid allocation size: ${size}`);
    }

    if (align <= 0 || (align & (align - 1)) !== 0) {
      throw new MemoryError(`Alignment must be a power of 2, got ${align}`);
    }

    // Align the current offset
    const alignedOffset = Math.ceil(this.currentOffset / align) * align;
    const ptr = alignedOffset;
    const newOffset = alignedOffset + size;

    // Ensure memory is large enough
    this.ensureCapacity(newOffset);

    // Update tracking
    this.currentOffset = newOffset;
    this.stats.totalAllocated += size;
    this.stats.activeAllocations += 1;
    this.stats.currentOffset = this.currentOffset;

    // Update peak usage
    if (this.currentOffset > this.stats.peakUsage) {
      this.stats.peakUsage = this.currentOffset;
    }

    // Record allocation
    this.allocations.set(ptr, {
      ptr,
      size,
      timestamp: Date.now(),
      tag,
    });

    return ptr;
  }

  /**
   * Allocate bytes that can hold a UTF-8 encoded string
   * @param str - The string to allocate space for
   * @param tag - Optional tag for debugging
   * @returns Pointer to allocated memory
   */
  allocString(str: string, tag?: string): number {
    // UTF-8 encoding can use up to 4 bytes per character
    const maxBytes = str.length * 4;
    return this.alloc(maxBytes, 4, tag);
  }

  /**
   * Free an allocation (no-op for bump allocator)
   *
   * Note: Bump allocators don't support individual frees. Use reset() to
   * reclaim all memory at once.
   *
   * @param _ptr - The pointer to free (ignored)
   */
  free(_ptr: number): void {
    // Bump allocator doesn't support individual frees
    // Memory is reclaimed on reset()
  }

  /**
   * Reset the allocator to initial state
   * All previously allocated memory becomes invalid
   */
  reset(): void {
    this.currentOffset = this.baseOffset;
    this.allocations.clear();
    this.stats.activeAllocations = 0;
  }

  /**
   * Get allocation statistics
   * @returns Current allocation statistics
   */
  getStats(): AllocationStats {
    return {
      ...this.stats,
      activeAllocations: this.allocations.size,
    };
  }

  /**
   * Get the current offset (next allocation point)
   * @returns Current offset in bytes
   */
  get offset(): number {
    return this.currentOffset;
  }

  /**
   * Get the base offset where heap starts
   * @returns Base offset in bytes
   */
  get base(): number {
    return this.baseOffset;
  }

  /**
   * Get the total bytes used since start or last reset
   * @returns Total used bytes
   */
  get used(): number {
    return this.currentOffset - this.baseOffset;
  }

  /**
   * Get all current allocations
   * @returns Array of allocation records
   */
  getAllocations(): AllocationRecord[] {
    return Array.from(this.allocations.values());
  }

  /**
   * Get a specific allocation record
   * @param ptr - The pointer to look up
   * @returns Allocation record if found, undefined otherwise
   */
  getAllocationInfo(ptr: number): AllocationRecord | undefined {
    return this.allocations.get(ptr);
  }

  /**
   * Ensure memory has capacity for a given offset
   * @param requiredOffset - The offset we need to reach
   * @throws MemoryError if memory growth fails
   */
  private ensureCapacity(requiredOffset: number): void {
    const currentSize = this.wasmMemory.size();

    if (requiredOffset <= currentSize) {
      return; // Already enough capacity
    }

    // Calculate pages needed
    const requiredPages = Math.ceil(requiredOffset / MEMORY_LAYOUT.PAGE_SIZE);
    const currentPages = Math.ceil(currentSize / MEMORY_LAYOUT.PAGE_SIZE);
    const pagesToGrow = requiredPages - currentPages;

    if (pagesToGrow > 0) {
      try {
        this.wasmMemory.grow(pagesToGrow);
        this.stats.growthCount += 1;
      } catch (error) {
        throw new MemoryError(
          `Failed to grow memory to support offset ${requiredOffset}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
  }
}

/**
 * Stack-based allocation for temporary values
 *
 * Allocates from a separate stack region with push/pop semantics.
 * Useful for temporary host function scratch space.
 */
export class HostStackAllocator {
  private wasmMemory: WasmMemory;
  private stackBase: number;
  private stackTop: number;
  private stackLimit: number;
  private marks: number[] = [];

  /**
   * Create a new stack allocator
   * @param wasmMemory - WasmMemory interface
   * @param stackBase - Where to place the stack (default: start of heap)
   * @param stackSize - Stack size in bytes (default: STACK_SIZE)
   */
  constructor(
    wasmMemory: WasmMemory,
    stackBase: number = MEMORY_LAYOUT.HEAP_BASE,
    stackSize: number = MEMORY_LAYOUT.STACK_SIZE,
  ) {
    this.wasmMemory = wasmMemory;
    this.stackBase = stackBase;
    this.stackTop = stackBase;
    this.stackLimit = stackBase + stackSize;
  }

  /**
   * Allocate bytes from the stack
   * @param size - Number of bytes to allocate
   * @param align - Alignment in bytes (default: 8)
   * @returns Pointer to allocated memory
   * @throws MemoryError if stack overflow occurs or invalid arguments
   */
  alloc(size: number, align: number = 8): number {
    if (size <= 0) {
      throw new MemoryError(`Invalid allocation size: ${size}`);
    }

    if (align <= 0 || (align & (align - 1)) !== 0) {
      throw new MemoryError(`Alignment must be a power of 2, got ${align}`);
    }

    // Align stack top
    const alignedTop = Math.ceil(this.stackTop / align) * align;
    const newTop = alignedTop + size;

    if (newTop > this.stackLimit) {
      throw new MemoryError(
        `Stack overflow: needed ${newTop - this.stackBase} bytes, stack size is ${this.stackLimit - this.stackBase}`,
      );
    }

    const ptr = alignedTop;
    this.stackTop = newTop;
    return ptr;
  }

  /**
   * Mark current stack position for later restoration
   */
  mark(): void {
    this.marks.push(this.stackTop);
  }

  /**
   * Pop stack back to last mark
   * @throws Error if no marks exist
   */
  pop(): void {
    const mark = this.marks.pop();
    if (mark === undefined) {
      throw new Error('No stack marks to pop');
    }
    this.stackTop = mark;
  }

  /**
   * Clear all marks and reset stack
   */
  reset(): void {
    this.stackTop = this.stackBase;
    this.marks = [];
  }

  /**
   * Get current stack usage in bytes
   * @returns Bytes used
   */
  get used(): number {
    return this.stackTop - this.stackBase;
  }

  /**
   * Get stack capacity in bytes
   * @returns Total stack size
   */
  get capacity(): number {
    return this.stackLimit - this.stackBase;
  }

  /**
   * Get current stack top
   * @returns Current stack pointer
   */
  get top(): number {
    return this.stackTop;
  }
}
