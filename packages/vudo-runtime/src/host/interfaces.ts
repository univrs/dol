/**
 * Core interfaces for host function implementations
 *
 * These interfaces enable dependency injection and testing by abstracting
 * platform-specific implementations (console, memory, time, etc.)
 *
 * @module @vudo/runtime/host/interfaces
 */

import type { LogLevel, ResultCode } from '../abi/types.js';

/**
 * Interface for WASM memory access operations
 *
 * Provides safe access to WASM linear memory with typed array views
 * and string encoding/decoding capabilities.
 */
export interface IWasmMemory {
  /**
   * Get raw memory buffer
   */
  readonly buffer: ArrayBuffer;

  /**
   * Get Uint8Array view of memory
   */
  readonly u8: Uint8Array;

  /**
   * Get Int32Array view of memory
   */
  readonly i32: Int32Array;

  /**
   * Get Float64Array view of memory
   */
  readonly f64: Float64Array;

  /**
   * Decode a UTF-8 string from memory
   * @param ptr - Pointer to string data
   * @param len - Length in bytes
   * @returns Decoded string
   * @throws Error if pointer is invalid or string is malformed
   */
  decodeString(ptr: number, len: number): string;

  /**
   * Encode a UTF-8 string into memory
   * @param str - String to encode
   * @returns Pointer to encoded string
   */
  encodeString(str: string): number;

  /**
   * Allocate memory
   * @param size - Number of bytes to allocate
   * @returns Pointer to allocated memory (0 on failure)
   */
  alloc(size: number): number;

  /**
   * Free allocated memory
   * @param ptr - Pointer to free
   * @param size - Size of allocation
   */
  free(ptr: number, size: number): void;

  /**
   * Reallocate memory
   * @param ptr - Current pointer
   * @param oldSize - Current size
   * @param newSize - Desired new size
   * @returns New pointer (0 on failure)
   */
  realloc(ptr: number, oldSize: number, newSize: number): number;
}

/**
 * Interface for logging operations
 *
 * Abstraction over console/logging systems to enable testing
 * and custom log routing.
 */
export interface ILogger {
  /**
   * Log a message at the specified level
   * @param level - Log severity level
   * @param message - Message to log
   */
  log(level: LogLevel, message: string): void;

  /**
   * Log a debug message
   * @param message - Debug message
   */
  debug(message: string): void;

  /**
   * Log an info message
   * @param message - Info message
   */
  info(message: string): void;

  /**
   * Log a warning message
   * @param message - Warning message
   */
  warn(message: string): void;

  /**
   * Log an error message
   * @param message - Error message
   */
  error(message: string): void;

  /**
   * Print a message without newline
   * @param message - Message to print
   */
  print(message: string): void;

  /**
   * Print a message with newline
   * @param message - Message to print
   */
  println(message: string): void;
}

/**
 * Interface for time operations
 *
 * Abstraction over system time to enable testing and
 * custom time providers (e.g., simulated time).
 */
export interface ITimeProvider {
  /**
   * Get current timestamp in milliseconds since Unix epoch
   * @returns Timestamp in milliseconds
   */
  now(): bigint;

  /**
   * Get monotonic time in nanoseconds
   * @returns Monotonic time in nanoseconds
   */
  monotonicNow(): bigint;

  /**
   * Sleep for specified milliseconds
   * @param ms - Duration in milliseconds
   * @returns Promise that resolves after sleep duration
   */
  sleep(ms: number): Promise<void>;
}

/**
 * Default logger implementation using console
 */
export class ConsoleLogger implements ILogger {
  log(level: LogLevel, message: string): void {
    switch (level) {
      case LogLevel.Debug:
        this.debug(message);
        break;
      case LogLevel.Info:
        this.info(message);
        break;
      case LogLevel.Warn:
        this.warn(message);
        break;
      case LogLevel.Error:
        this.error(message);
        break;
    }
  }

  debug(message: string): void {
    console.debug(`[DEBUG] ${message}`);
  }

  info(message: string): void {
    console.info(`[INFO] ${message}`);
  }

  warn(message: string): void {
    console.warn(`[WARN] ${message}`);
  }

  error(message: string): void {
    console.error(`[ERROR] ${message}`);
  }

  print(message: string): void {
    process.stdout.write(message);
  }

  println(message: string): void {
    console.log(message);
  }
}

/**
 * Default time provider using system time
 */
export class SystemTimeProvider implements ITimeProvider {
  now(): bigint {
    return BigInt(Date.now());
  }

  monotonicNow(): bigint {
    // Use performance.now() for monotonic time
    // Convert to nanoseconds (performance.now() returns milliseconds with sub-ms precision)
    if (typeof performance !== 'undefined') {
      return BigInt(Math.floor(performance.now() * 1_000_000));
    }
    // Fallback to process.hrtime.bigint() in Node.js
    if (typeof process !== 'undefined' && process.hrtime?.bigint) {
      return process.hrtime.bigint();
    }
    // Last resort: use Date.now() in milliseconds converted to nanoseconds
    return BigInt(Date.now()) * 1_000_000n;
  }

  async sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

/**
 * Error class for host function failures
 */
export class HostFunctionError extends Error {
  constructor(
    message: string,
    public readonly code: ResultCode = ResultCode.Error
  ) {
    super(message);
    this.name = 'HostFunctionError';
  }
}
