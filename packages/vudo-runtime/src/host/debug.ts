/**
 * Debug System - Host Functions
 *
 * Implements debug host functions for the VUDO runtime:
 * - vudo_breakpoint: Trigger debugger breakpoint
 * - vudo_assert: Assert condition with message
 * - vudo_panic: Panic and terminate Spirit (never returns)
 *
 * These functions aid in debugging WASM Spirits by providing
 * breakpoint control, assertion checking, and panic handling.
 *
 * @module @vudo/runtime/host/debug
 */

/**
 * Handler interface for debug operations
 *
 * Implementors can customize debug behavior for breakpoints,
 * assertions, and panics.
 *
 * @example
 * ```typescript
 * class MyDebugHandler implements IDebugHandler {
 *   onBreakpoint(): void {
 *     console.log('Breakpoint hit!');
 *     debugger;
 *   }
 *
 *   onAssertionFailure(message: string): void {
 *     console.error('Assertion failed:', message);
 *     throw new Error(message);
 *   }
 *
 *   onPanic(message: string): never {
 *     console.error('PANIC:', message);
 *     process.exit(1);
 *   }
 * }
 * ```
 */
export interface IDebugHandler {
  /**
   * Called when vudo_breakpoint() is invoked
   *
   * Typically triggers a debugger breakpoint if attached.
   */
  onBreakpoint(): void;

  /**
   * Called when vudo_assert() fails (condition is false)
   *
   * @param message - Assertion failure message
   */
  onAssertionFailure(message: string): void;

  /**
   * Called when vudo_panic() is invoked
   *
   * This should terminate the Spirit and never return.
   *
   * @param message - Panic message
   * @returns Never returns
   */
  onPanic(message: string): never;
}

/**
 * Default debug handler implementation
 *
 * Provides standard debug behavior:
 * - Breakpoints trigger JavaScript debugger statement
 * - Assertions throw Error on failure
 * - Panics throw PanicError and terminate
 */
export class DefaultDebugHandler implements IDebugHandler {
  onBreakpoint(): void {
    // Trigger JavaScript debugger if attached
    // eslint-disable-next-line no-debugger
    debugger;
  }

  onAssertionFailure(message: string): void {
    const error = new Error(`Assertion failed: ${message}`);
    error.name = 'AssertionError';
    console.error('[Debug] Assertion failed:', message);
    throw error;
  }

  onPanic(message: string): never {
    const error = new PanicError(message);
    console.error('[Debug] PANIC:', message);
    throw error;
  }
}

/**
 * Custom error type for panic operations
 *
 * Thrown when vudo_panic() is called, indicating an unrecoverable error.
 */
export class PanicError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'PanicError';
    Object.setPrototypeOf(this, PanicError.prototype);
  }

  toString(): string {
    return `PanicError: ${this.message}`;
  }
}

/**
 * Debug system managing breakpoints, assertions, and panics
 *
 * Provides:
 * - Breakpoint triggering for debugging
 * - Assertion validation with custom messages
 * - Panic handling for unrecoverable errors
 */
export class DebugSystem {
  private memory: WebAssembly.Memory;
  private handler: IDebugHandler;
  private breakpointCount: number;
  private assertionCount: number;
  private assertionFailures: number;

  /**
   * Create a new debug system
   *
   * @param memory - WASM memory instance for reading debug messages
   * @param handler - Handler for debug operations (defaults to DefaultDebugHandler)
   */
  constructor(memory: WebAssembly.Memory, handler?: IDebugHandler) {
    this.memory = memory;
    this.handler = handler || new DefaultDebugHandler();
    this.breakpointCount = 0;
    this.assertionCount = 0;
    this.assertionFailures = 0;
  }

  /**
   * Update the WASM memory reference
   *
   * Called when memory grows or is reassigned.
   *
   * @param memory - New memory instance
   */
  updateMemory(memory: WebAssembly.Memory): void {
    this.memory = memory;
  }

  /**
   * Decode a UTF-8 string from WASM memory
   *
   * @param ptr - Pointer to string data
   * @param len - Length in bytes
   * @returns Decoded string
   */
  private decodeString(ptr: number, len: number): string {
    const bytes = new Uint8Array(this.memory.buffer, ptr, len);
    return new TextDecoder('utf-8').decode(bytes);
  }

  /**
   * Trigger a breakpoint
   *
   * Host function: vudo_breakpoint()
   *
   * Signals the debugger to pause execution. No-op if no debugger is attached.
   */
  breakpoint(): void {
    this.breakpointCount++;
    console.log(`[Debug] Breakpoint #${this.breakpointCount}`);
    this.handler.onBreakpoint();
  }

  /**
   * Assert a condition with an optional message
   *
   * Host function: vudo_assert(condition, msg_ptr, msg_len)
   *
   * Validates that a condition is true. If false, logs the error message
   * and invokes the handler's assertion failure callback.
   *
   * @param condition - Condition to assert (non-zero = true, zero = false)
   * @param msgPtr - Pointer to assertion message (UTF-8)
   * @param msgLen - Length of message in bytes
   * @throws {Error} If condition is false and handler throws
   */
  assert(condition: number, msgPtr: number, msgLen: number): void {
    this.assertionCount++;

    if (condition !== 0) {
      // Assertion passed
      return;
    }

    // Assertion failed
    this.assertionFailures++;
    const message = this.decodeString(msgPtr, msgLen);

    console.error(`[Debug] Assertion #${this.assertionCount} failed: ${message}`);
    this.handler.onAssertionFailure(message);
  }

  /**
   * Panic with an error message
   *
   * Host function: vudo_panic(msg_ptr, msg_len)
   *
   * Immediately terminates Spirit execution with a panic message.
   * This function never returns.
   *
   * @param msgPtr - Pointer to panic message (UTF-8)
   * @param msgLen - Length of message in bytes
   * @returns Never returns
   * @throws {PanicError} Always throws to terminate execution
   */
  panic(msgPtr: number, msgLen: number): never {
    const message = this.decodeString(msgPtr, msgLen);
    console.error('[Debug] PANIC:', message);
    this.handler.onPanic(message);
  }

  /**
   * Get debug statistics
   *
   * @returns Object with debug counters
   */
  getStats(): {
    breakpoints: number;
    assertions: number;
    assertionFailures: number;
  } {
    return {
      breakpoints: this.breakpointCount,
      assertions: this.assertionCount,
      assertionFailures: this.assertionFailures,
    };
  }

  /**
   * Reset debug statistics
   */
  resetStats(): void {
    this.breakpointCount = 0;
    this.assertionCount = 0;
    this.assertionFailures = 0;
  }

  /**
   * Create WASM host function implementations
   *
   * Returns an object with vudo_breakpoint, vudo_assert, and vudo_panic
   * functions suitable for WASM imports.
   *
   * @returns Host function implementations
   */
  createHostFunctions(): {
    vudo_breakpoint: () => void;
    vudo_assert: (condition: number, msgPtr: number, msgLen: number) => void;
    vudo_panic: (msgPtr: number, msgLen: number) => never;
  } {
    return {
      vudo_breakpoint: (): void => {
        this.breakpoint();
      },
      vudo_assert: (condition: number, msgPtr: number, msgLen: number): void => {
        this.assert(condition, msgPtr, msgLen);
      },
      vudo_panic: (msgPtr: number, msgLen: number): never => {
        return this.panic(msgPtr, msgLen);
      },
    };
  }
}

/**
 * Create a new debug system instance
 *
 * Convenience factory function for creating a debug system.
 *
 * @param memory - WASM memory instance
 * @param handler - Optional custom debug handler
 * @returns New DebugSystem instance
 *
 * @example
 * ```typescript
 * const debug = createDebugSystem(memory);
 * const hostFunctions = debug.createHostFunctions();
 * ```
 */
export function createDebugSystem(
  memory: WebAssembly.Memory,
  handler?: IDebugHandler
): DebugSystem {
  return new DebugSystem(memory, handler);
}
