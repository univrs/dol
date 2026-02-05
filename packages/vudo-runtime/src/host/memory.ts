/**
 * WASM Linear Memory Interface
 *
 * Provides typed access to WASM linear memory with bounds checking and proper
 * UTF-8 string handling. This layer bridges host functions and WASM memory.
 */

/**
 * Error thrown when memory operations fail
 */
export class MemoryError extends Error {
  /**
   * Create a new MemoryError
   * @param message - Error description
   * @param ptr - Optional pointer that caused the error
   * @param length - Optional length that caused the error
   */
  constructor(
    message: string,
    readonly ptr?: number,
    readonly length?: number,
  ) {
    super(message);
    this.name = 'MemoryError';
    Object.setPrototypeOf(this, MemoryError.prototype);
  }
}

/**
 * WASM Memory Interface
 *
 * Provides type-safe access to linear memory with:
 * - String read/write with UTF-8 encoding
 * - Raw byte operations
 * - Numeric type access (I32, I64, F32, F64)
 * - Bounds checking
 * - Memory growth
 */
export class WasmMemory {
  private memory: WebAssembly.Memory;
  private textEncoder: TextEncoder;
  private textDecoder: TextDecoder;

  /**
   * Create a new WasmMemory interface
   * @param memory - WebAssembly.Memory instance to manage
   */
  constructor(memory: WebAssembly.Memory) {
    this.memory = memory;
    this.textEncoder = new TextEncoder();
    this.textDecoder = new TextDecoder('utf-8');
  }

  /**
   * Check if a memory range is valid
   * @param ptr - Starting pointer
   * @param length - Number of bytes
   * @returns true if range is within bounds
   * @throws MemoryError if range is out of bounds
   */
  isValidRange(ptr: number, length: number): boolean {
    if (ptr < 0 || length < 0) {
      throw new MemoryError(
        `Invalid pointer or length: ptr=${ptr}, length=${length}`,
        ptr,
        length,
      );
    }

    if (ptr + length > this.memory.buffer.byteLength) {
      throw new MemoryError(
        `Memory access out of bounds: ptr=${ptr}, length=${length}, buffer_size=${this.memory.buffer.byteLength}`,
        ptr,
        length,
      );
    }

    return true;
  }

  /**
   * Read a UTF-8 string from memory
   * @param ptr - Starting pointer
   * @param len - Number of bytes to read
   * @returns Decoded UTF-8 string
   * @throws MemoryError if range is invalid
   */
  readString(ptr: number, len: number): string {
    this.isValidRange(ptr, len);

    const bytes = new Uint8Array(this.memory.buffer, ptr, len);
    return this.textDecoder.decode(bytes);
  }

  /**
   * Write a UTF-8 string to memory
   * @param ptr - Starting pointer
   * @param str - String to write
   * @returns Number of bytes written
   * @throws MemoryError if range is insufficient
   */
  writeString(ptr: number, str: string): number {
    const encoded = this.textEncoder.encode(str);
    const len = encoded.length;

    this.isValidRange(ptr, len);

    const view = new Uint8Array(this.memory.buffer, ptr, len);
    view.set(encoded);

    return len;
  }

  /**
   * Read raw bytes from memory
   * @param ptr - Starting pointer
   * @param len - Number of bytes to read
   * @returns Uint8Array view of memory
   * @throws MemoryError if range is invalid
   */
  readBytes(ptr: number, len: number): Uint8Array {
    this.isValidRange(ptr, len);
    return new Uint8Array(this.memory.buffer, ptr, len);
  }

  /**
   * Write raw bytes to memory
   * @param ptr - Starting pointer
   * @param data - Bytes to write
   * @returns Number of bytes written
   * @throws MemoryError if range is insufficient
   */
  writeBytes(ptr: number, data: Uint8Array): number {
    const len = data.length;
    this.isValidRange(ptr, len);

    const view = new Uint8Array(this.memory.buffer, ptr, len);
    view.set(data);

    return len;
  }

  /**
   * Read a signed 32-bit integer
   * @param ptr - Pointer to i32 value
   * @returns The i32 value
   * @throws MemoryError if pointer is invalid
   */
  readI32(ptr: number): number {
    this.isValidRange(ptr, 4);

    const view = new DataView(this.memory.buffer, ptr, 4);
    return view.getInt32(0, true); // little-endian
  }

  /**
   * Write a signed 32-bit integer
   * @param ptr - Pointer to write i32 value
   * @param value - The i32 value to write
   * @throws MemoryError if pointer is invalid
   */
  writeI32(ptr: number, value: number): void {
    this.isValidRange(ptr, 4);

    const view = new DataView(this.memory.buffer, ptr, 4);
    view.setInt32(0, value, true); // little-endian
  }

  /**
   * Read a signed 64-bit integer
   * @param ptr - Pointer to i64 value
   * @returns The i64 value as BigInt
   * @throws MemoryError if pointer is invalid
   */
  readI64(ptr: number): bigint {
    this.isValidRange(ptr, 8);

    const view = new DataView(this.memory.buffer, ptr, 8);
    return view.getBigInt64(0, true); // little-endian
  }

  /**
   * Write a signed 64-bit integer
   * @param ptr - Pointer to write i64 value
   * @param value - The i64 value to write as BigInt
   * @throws MemoryError if pointer is invalid
   */
  writeI64(ptr: number, value: bigint): void {
    this.isValidRange(ptr, 8);

    const view = new DataView(this.memory.buffer, ptr, 8);
    view.setBigInt64(0, value, true); // little-endian
  }

  /**
   * Read a signed 32-bit floating point number
   * @param ptr - Pointer to f32 value
   * @returns The f32 value
   * @throws MemoryError if pointer is invalid
   */
  readF32(ptr: number): number {
    this.isValidRange(ptr, 4);

    const view = new DataView(this.memory.buffer, ptr, 4);
    return view.getFloat32(0, true); // little-endian
  }

  /**
   * Write a signed 32-bit floating point number
   * @param ptr - Pointer to write f32 value
   * @param value - The f32 value to write
   * @throws MemoryError if pointer is invalid
   */
  writeF32(ptr: number, value: number): void {
    this.isValidRange(ptr, 4);

    const view = new DataView(this.memory.buffer, ptr, 4);
    view.setFloat32(0, value, true); // little-endian
  }

  /**
   * Read a signed 64-bit floating point number
   * @param ptr - Pointer to f64 value
   * @returns The f64 value
   * @throws MemoryError if pointer is invalid
   */
  readF64(ptr: number): number {
    this.isValidRange(ptr, 8);

    const view = new DataView(this.memory.buffer, ptr, 8);
    return view.getFloat64(0, true); // little-endian
  }

  /**
   * Write a signed 64-bit floating point number
   * @param ptr - Pointer to write f64 value
   * @param value - The f64 value to write
   * @throws MemoryError if pointer is invalid
   */
  writeF64(ptr: number, value: number): void {
    this.isValidRange(ptr, 8);

    const view = new DataView(this.memory.buffer, ptr, 8);
    view.setFloat64(0, value, true); // little-endian
  }

  /**
   * Grow memory by a specified number of pages
   * @param pages - Number of 64KB pages to add
   * @returns Number of pages after growth, or -1 on failure
   */
  grow(pages: number): number {
    if (pages < 0) {
      throw new MemoryError(`Invalid number of pages to grow: ${pages}`);
    }

    try {
      return this.memory.grow(pages);
    } catch (error) {
      throw new MemoryError(
        `Failed to grow memory by ${pages} pages: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  /**
   * Get the underlying WebAssembly.Memory instance
   * @returns The WebAssembly.Memory object
   */
  getRawMemory(): WebAssembly.Memory {
    return this.memory;
  }

  /**
   * Get the current size of memory in bytes
   * @returns Number of bytes in linear memory
   */
  size(): number {
    return this.memory.buffer.byteLength;
  }

  /**
   * Get the current size of memory in 64KB pages
   * @returns Number of 64KB pages
   */
  pages(): number {
    return this.memory.buffer.byteLength / 65536;
  }

  /**
   * Get a DataView for direct memory access
   * @param ptr - Starting pointer
   * @param length - Number of bytes
   * @returns DataView for the memory range
   * @throws MemoryError if range is invalid
   */
  createDataView(ptr: number, length: number): DataView {
    this.isValidRange(ptr, length);
    return new DataView(this.memory.buffer, ptr, length);
  }

  /**
   * Get a Uint8Array view for direct byte access
   * @param ptr - Starting pointer
   * @param length - Number of bytes
   * @returns Uint8Array view of the memory range
   * @throws MemoryError if range is invalid
   */
  createByteView(ptr: number, length: number): Uint8Array {
    this.isValidRange(ptr, length);
    return new Uint8Array(this.memory.buffer, ptr, length);
  }
}
