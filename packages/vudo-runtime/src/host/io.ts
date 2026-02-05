/**
 * I/O Host Functions Implementation
 *
 * Implements the 4 I/O host functions:
 * - vudo_print: Print without newline
 * - vudo_println: Print with newline
 * - vudo_log: Structured logging with level
 * - vudo_error: Error logging
 *
 * These functions bridge WASM memory to host I/O systems.
 *
 * @module @vudo/runtime/host/io
 */

import { LogLevel, ResultCode } from '../abi/types.js';
import type { IWasmMemory, ILogger } from './interfaces.js';
import { HostFunctionError } from './interfaces.js';

/**
 * Configuration for I/O host functions
 */
export interface IOConfig {
  /** Memory interface for reading strings */
  memory: IWasmMemory;
  /** Logger implementation */
  logger: ILogger;
  /** Maximum allowed string length (default 1MB) */
  maxStringLength?: number;
}

/**
 * I/O host function provider
 *
 * Creates host function implementations for I/O operations
 * with proper error handling and validation.
 */
export class IOHostFunctions {
  private memory: IWasmMemory;
  private logger: ILogger;
  private maxStringLength: number;

  constructor(config: IOConfig) {
    this.memory = config.memory;
    this.logger = config.logger;
    this.maxStringLength = config.maxStringLength ?? 1024 * 1024; // 1MB default
  }

  /**
   * Validate string parameters
   * @param ptr - Pointer to string data
   * @param len - Length in bytes
   * @throws HostFunctionError if parameters are invalid
   */
  private validateStringParams(ptr: number, len: number): void {
    if (ptr < 0) {
      throw new HostFunctionError(
        `Invalid pointer: ${ptr} (must be non-negative)`,
        ResultCode.InvalidArg
      );
    }

    if (len < 0) {
      throw new HostFunctionError(
        `Invalid length: ${len} (must be non-negative)`,
        ResultCode.InvalidArg
      );
    }

    if (len > this.maxStringLength) {
      throw new HostFunctionError(
        `String length ${len} exceeds maximum ${this.maxStringLength}`,
        ResultCode.InvalidArg
      );
    }

    if (ptr + len > this.memory.buffer.byteLength) {
      throw new HostFunctionError(
        `String at ${ptr}+${len} exceeds memory bounds ${this.memory.buffer.byteLength}`,
        ResultCode.InvalidArg
      );
    }
  }

  /**
   * Read and decode a UTF-8 string from WASM memory
   * @param ptr - Pointer to string data
   * @param len - Length in bytes
   * @returns Decoded string
   * @throws HostFunctionError if string is invalid
   */
  private readString(ptr: number, len: number): string {
    this.validateStringParams(ptr, len);

    try {
      return this.memory.decodeString(ptr, len);
    } catch (error) {
      throw new HostFunctionError(
        `Failed to decode UTF-8 string at ${ptr}: ${error instanceof Error ? error.message : String(error)}`,
        ResultCode.InvalidArg
      );
    }
  }

  /**
   * Print a UTF-8 string without newline
   *
   * Implements vudo_print(ptr: i32, len: i32) -> void
   *
   * @param ptr - Pointer to string data
   * @param len - Length in bytes
   */
  print(ptr: number, len: number): void {
    try {
      const message = this.readString(ptr, len);
      this.logger.print(message);
    } catch (error) {
      // Log error but don't throw - I/O functions should be resilient
      this.logger.error(
        `vudo_print failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * Print a UTF-8 string with newline
   *
   * Implements vudo_println(ptr: i32, len: i32) -> void
   *
   * @param ptr - Pointer to string data
   * @param len - Length in bytes
   */
  println(ptr: number, len: number): void {
    try {
      const message = this.readString(ptr, len);
      this.logger.println(message);
    } catch (error) {
      this.logger.error(
        `vudo_println failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * Log a structured message with severity level
   *
   * Implements vudo_log(level: i32, ptr: i32, len: i32) -> void
   *
   * @param level - Log level (0=DEBUG, 1=INFO, 2=WARN, 3=ERROR)
   * @param ptr - Pointer to message data
   * @param len - Length in bytes
   */
  log(level: number, ptr: number, len: number): void {
    try {
      // Validate log level
      if (level < LogLevel.Debug || level > LogLevel.Error) {
        this.logger.warn(
          `Invalid log level ${level}, defaulting to ERROR`
        );
        level = LogLevel.Error;
      }

      const message = this.readString(ptr, len);
      this.logger.log(level as LogLevel, message);
    } catch (error) {
      this.logger.error(
        `vudo_log failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * Log an error message
   *
   * Implements vudo_error(ptr: i32, len: i32) -> void
   *
   * Convenience function equivalent to vudo_log(ERROR, ptr, len)
   *
   * @param ptr - Pointer to error message
   * @param len - Length in bytes
   */
  error(ptr: number, len: number): void {
    try {
      const message = this.readString(ptr, len);
      this.logger.error(message);
    } catch (error) {
      // Even if reading the error message fails, try to log something
      this.logger.error(
        `vudo_error failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * Build WebAssembly imports object for I/O functions
   *
   * @returns Object with I/O host functions bound to this instance
   */
  buildImports(): Record<string, Function> {
    return {
      vudo_print: this.print.bind(this),
      vudo_println: this.println.bind(this),
      vudo_log: this.log.bind(this),
      vudo_error: this.error.bind(this),
    };
  }
}

/**
 * Create I/O host functions with default configuration
 *
 * @param memory - WASM memory interface
 * @param logger - Logger implementation
 * @returns Configured IOHostFunctions instance
 */
export function createIOHostFunctions(
  memory: IWasmMemory,
  logger: ILogger
): IOHostFunctions {
  return new IOHostFunctions({ memory, logger });
}
