/**
 * Test utilities and mocks for host function tests
 */

import type { IWasmMemory, ILogger, ITimeProvider } from '../../src/host/interfaces.js';
import { LogLevel } from '../../src/abi/types.js';

/**
 * Mock WASM memory for testing
 */
export class MockWasmMemory implements IWasmMemory {
  private _buffer: ArrayBuffer;
  private _u8: Uint8Array;
  private _i32: Int32Array;
  private _f64: Float64Array;
  private allocOffset = 1024; // Start after null pointer region
  private allocations = new Map<number, number>(); // ptr -> size

  constructor(sizeInBytes = 65536) {
    this._buffer = new ArrayBuffer(sizeInBytes);
    this._u8 = new Uint8Array(this._buffer);
    this._i32 = new Int32Array(this._buffer);
    this._f64 = new Float64Array(this._buffer);
  }

  get buffer(): ArrayBuffer {
    return this._buffer;
  }

  get u8(): Uint8Array {
    return this._u8;
  }

  get i32(): Int32Array {
    return this._i32;
  }

  get f64(): Float64Array {
    return this._f64;
  }

  decodeString(ptr: number, len: number): string {
    if (ptr < 0 || len < 0 || ptr + len > this._buffer.byteLength) {
      throw new Error(`Invalid string bounds: ptr=${ptr}, len=${len}`);
    }
    const bytes = this._u8.slice(ptr, ptr + len);
    return new TextDecoder().decode(bytes);
  }

  encodeString(str: string): number {
    const encoded = new TextEncoder().encode(str);

    // Handle empty strings - allocate 1 byte but return valid pointer
    if (encoded.length === 0) {
      const ptr = this.allocOffset;
      this.allocOffset += 1;
      return ptr;
    }

    const ptr = this.alloc(encoded.length);
    if (ptr === 0) {
      throw new Error('Failed to allocate memory for string');
    }
    this._u8.set(encoded, ptr);
    return ptr;
  }

  alloc(size: number): number {
    if (size <= 0) {
      return 0;
    }

    const ptr = this.allocOffset;
    if (ptr + size > this._buffer.byteLength) {
      return 0; // Out of memory
    }

    this.allocations.set(ptr, size);
    this.allocOffset += size;
    return ptr;
  }

  free(ptr: number, size: number): void {
    if (this.allocations.has(ptr)) {
      const allocSize = this.allocations.get(ptr)!;
      if (allocSize !== size) {
        throw new Error(`Free size mismatch: expected ${allocSize}, got ${size}`);
      }
      this.allocations.delete(ptr);
    }
  }

  realloc(ptr: number, oldSize: number, newSize: number): number {
    if (!this.allocations.has(ptr)) {
      throw new Error(`Invalid realloc: pointer ${ptr} not allocated`);
    }

    const allocSize = this.allocations.get(ptr)!;
    if (allocSize !== oldSize) {
      throw new Error(`Realloc size mismatch: expected ${allocSize}, got ${oldSize}`);
    }

    // Allocate new block
    const newPtr = this.alloc(newSize);
    if (newPtr === 0) {
      return 0; // Realloc failed, original remains valid
    }

    // Copy data (min of old and new sizes)
    const copySize = Math.min(oldSize, newSize);
    this._u8.copyWithin(newPtr, ptr, ptr + copySize);

    // Free old block
    this.free(ptr, oldSize);

    return newPtr;
  }

  /**
   * Reset allocator for testing
   */
  reset(): void {
    this.allocOffset = 1024;
    this.allocations.clear();
    // Zero out memory
    this._u8.fill(0);
  }

  /**
   * Get allocation statistics
   */
  getAllocationStats(): { count: number; bytes: number } {
    let bytes = 0;
    for (const size of this.allocations.values()) {
      bytes += size;
    }
    return { count: this.allocations.size, bytes };
  }
}

/**
 * Mock logger for testing
 */
export class MockLogger implements ILogger {
  public messages: Array<{ level: string; message: string }> = [];

  log(level: LogLevel, message: string): void {
    const levelStr = ['DEBUG', 'INFO', 'WARN', 'ERROR'][level];
    this.messages.push({ level: levelStr, message });
  }

  debug(message: string): void {
    this.messages.push({ level: 'DEBUG', message });
  }

  info(message: string): void {
    this.messages.push({ level: 'INFO', message });
  }

  warn(message: string): void {
    this.messages.push({ level: 'WARN', message });
  }

  error(message: string): void {
    this.messages.push({ level: 'ERROR', message });
  }

  print(message: string): void {
    this.messages.push({ level: 'PRINT', message });
  }

  println(message: string): void {
    this.messages.push({ level: 'PRINTLN', message });
  }

  /**
   * Reset captured messages
   */
  reset(): void {
    this.messages = [];
  }

  /**
   * Get messages of a specific level
   */
  getMessages(level: string): string[] {
    return this.messages
      .filter((m) => m.level === level)
      .map((m) => m.message);
  }

  /**
   * Check if a message was logged
   */
  hasMessage(level: string, messagePattern: string | RegExp): boolean {
    const messages = this.getMessages(level);
    if (typeof messagePattern === 'string') {
      return messages.some((m) => m.includes(messagePattern));
    }
    return messages.some((m) => messagePattern.test(m));
  }
}

/**
 * Mock time provider for testing
 */
export class MockTimeProvider implements ITimeProvider {
  private _now = BigInt(Date.now());
  private _monotonicNow = 0n;
  private sleeps: number[] = [];

  now(): bigint {
    return this._now;
  }

  monotonicNow(): bigint {
    return this._monotonicNow;
  }

  async sleep(ms: number): Promise<void> {
    this.sleeps.push(ms);
    // Don't actually sleep in tests
    return Promise.resolve();
  }

  /**
   * Advance time for testing
   */
  advance(ms: number): void {
    this._now += BigInt(ms);
    this._monotonicNow += BigInt(ms) * 1_000_000n; // Convert to nanoseconds
  }

  /**
   * Set absolute time
   */
  setNow(timestamp: bigint): void {
    this._now = timestamp;
  }

  /**
   * Set absolute monotonic time
   */
  setMonotonicNow(timestamp: bigint): void {
    this._monotonicNow = timestamp;
  }

  /**
   * Get sleep history
   */
  getSleeps(): number[] {
    return [...this.sleeps];
  }

  /**
   * Reset for testing
   */
  reset(): void {
    this._now = BigInt(Date.now());
    this._monotonicNow = 0n;
    this.sleeps = [];
  }
}

/**
 * Helper to write a string to memory at a specific location
 */
export function writeStringToMemory(
  memory: MockWasmMemory,
  str: string,
  ptr: number
): void {
  const encoded = new TextEncoder().encode(str);
  memory.u8.set(encoded, ptr);
}

/**
 * Helper to create a test string in memory
 */
export function createTestString(
  memory: MockWasmMemory,
  str: string
): { ptr: number; len: number } {
  const encoded = new TextEncoder().encode(str);
  const ptr = memory.encodeString(str);
  return { ptr, len: encoded.length };
}
