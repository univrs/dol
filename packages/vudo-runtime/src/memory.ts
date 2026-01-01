/**
 * Memory management for Spirit instances
 *
 * Internal implementation based on vudo-runtime-spec.md BumpAllocator
 */

import type { GeneLayout, MemoryManager } from './types.js';
import { decodeString, encodeString, readGene, writeGene } from './utils/type-bridge.js';

/**
 * Bump allocator for WASM linear memory
 *
 * Simple and fast allocation strategy - memory is allocated sequentially
 * and only freed when the entire allocator is reset.
 *
 * @internal
 */
export class BumpAllocator {
  private memory: WebAssembly.Memory;
  private currentOffset: number;
  private readonly baseOffset: number;

  /**
   * Create a new bump allocator
   * @param memory - WASM memory instance
   * @param baseOffset - Starting offset (default 1024 to avoid null pointer region)
   */
  constructor(memory: WebAssembly.Memory, baseOffset = 1024) {
    this.memory = memory;
    this.currentOffset = baseOffset;
    this.baseOffset = baseOffset;
  }

  /**
   * Allocate `size` bytes with alignment
   * @param size - Number of bytes to allocate
   * @param align - Alignment requirement (default 8 for 64-bit)
   * @returns Pointer to allocated memory
   */
  alloc(size: number, align = 8): number {
    // Align offset
    const alignedOffset = Math.ceil(this.currentOffset / align) * align;
    const ptr = alignedOffset;
    this.currentOffset = alignedOffset + size;

    // Grow memory if needed
    const requiredBytes = this.currentOffset;
    const currentBytes = this.memory.buffer.byteLength;

    if (requiredBytes > currentBytes) {
      const requiredPages = Math.ceil(requiredBytes / 65536);
      const currentPages = currentBytes / 65536;
      const pagesToGrow = requiredPages - currentPages;

      if (pagesToGrow > 0) {
        this.memory.grow(pagesToGrow);
      }
    }

    return ptr;
  }

  /**
   * Free memory at pointer (no-op for bump allocator)
   * Memory is only reclaimed on reset()
   */
  free(_ptr: number): void {
    // Bump allocator doesn't support individual frees
    // Memory is reclaimed on reset()
  }

  /**
   * Reset allocator to initial state
   * All previously allocated memory becomes invalid
   */
  reset(): void {
    this.currentOffset = this.baseOffset;
  }

  /**
   * Get current allocation offset
   */
  get offset(): number {
    return this.currentOffset;
  }

  /**
   * Get raw memory buffer
   */
  get buffer(): ArrayBuffer {
    return this.memory.buffer;
  }

  /**
   * Get WASM memory instance
   */
  get rawMemory(): WebAssembly.Memory {
    return this.memory;
  }
}

/**
 * Memory manager implementation wrapping BumpAllocator
 * Provides high-level API for gene operations
 */
export class SpiritMemoryManager implements MemoryManager {
  private allocator: BumpAllocator;
  private wasmMemory: WebAssembly.Memory;

  constructor(memory: WebAssembly.Memory, baseOffset = 1024) {
    this.wasmMemory = memory;
    this.allocator = new BumpAllocator(memory, baseOffset);
  }

  alloc(size: number, align = 8): number {
    return this.allocator.alloc(size, align);
  }

  free(ptr: number): void {
    this.allocator.free(ptr);
  }

  reset(): void {
    this.allocator.reset();
  }

  readGene<T extends GeneLayout>(ptr: number, layout: T): Record<string, unknown> {
    return readGene(this.wasmMemory, ptr, layout);
  }

  writeGene<T extends GeneLayout>(ptr: number, values: Record<string, unknown>, layout: T): void {
    writeGene(this.wasmMemory, ptr, layout, values);
  }

  encodeString(str: string): number {
    return encodeString(this.wasmMemory, this.allocator, str);
  }

  decodeString(ptr: number): string {
    return decodeString(this.wasmMemory, ptr);
  }

  get buffer(): ArrayBuffer {
    return this.allocator.buffer;
  }

  get offset(): number {
    return this.allocator.offset;
  }

  /**
   * Get typed array views of memory
   */
  get views() {
    const buffer = this.buffer;
    return {
      i8: new Int8Array(buffer),
      u8: new Uint8Array(buffer),
      i32: new Int32Array(buffer),
      u32: new Uint32Array(buffer),
      i64: new BigInt64Array(buffer),
      u64: new BigUint64Array(buffer),
      f32: new Float32Array(buffer),
      f64: new Float64Array(buffer),
    };
  }
}
