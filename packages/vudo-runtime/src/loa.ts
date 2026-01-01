/**
 * Loa - Service injection for Spirit instances
 *
 * A Loa is a named service that provides host functions to WASM modules.
 * Based on vudo-runtime-spec.md host function implementations.
 */

import type { Loa, LoaContext, LoaRegistry as ILoaRegistry } from './types.js';

// ============================================================================
// Core Loa - Default host functions for all Spirits
// ============================================================================

/**
 * Core Loa providing essential host functions
 *
 * These functions are available to all Spirits by default:
 * - vudo_print: Print a string to console
 * - vudo_alloc: Allocate memory (bound to Spirit's allocator)
 * - vudo_now: Get current timestamp
 * - vudo_random: Generate random number
 * - vudo_emit_effect: Emit a side effect
 */
export const coreLoa: Loa = {
  name: 'core',
  version: '1.0.0',
  capabilities: ['print', 'alloc', 'time', 'random', 'effects'],

  provides: (context: LoaContext) => ({
    /**
     * Print a string from WASM memory
     * Signature: vudo_print(ptr: i32, len: i32) -> void
     */
    vudo_print: (ptr: number, len: number): void => {
      const bytes = new Uint8Array(context.memory.buffer, ptr, len);
      const text = new TextDecoder('utf-8').decode(bytes);
      console.log('[Spirit]', text);
    },

    /**
     * Allocate memory
     * Signature: vudo_alloc(size: i32) -> i32
     */
    vudo_alloc: (size: number): number => {
      return context.alloc(size);
    },

    /**
     * Get current timestamp (milliseconds since epoch)
     * Signature: vudo_now() -> i64
     */
    vudo_now: (): bigint => {
      return BigInt(Date.now());
    },

    /**
     * Generate a random number between 0 and 1
     * Signature: vudo_random() -> f64
     */
    vudo_random: (): number => {
      return Math.random();
    },

    /**
     * Emit a side effect for the host to handle
     * Signature: vudo_emit_effect(effect_id: i32, payload_ptr: i32) -> i32
     *
     * Returns: 0 = success, non-zero = error code
     */
    vudo_emit_effect: (effectId: number, payloadPtr: number): number => {
      if (context.debug) {
        console.log(`[Spirit] Effect ${effectId} emitted with payload at ${payloadPtr}`);
      }
      // Default implementation - effects are logged but not handled
      // Override with custom Loa for actual effect handling
      return 0;
    },

    /**
     * Debug log (only when debug mode is enabled)
     * Signature: vudo_debug(ptr: i32, len: i32) -> void
     */
    vudo_debug: (ptr: number, len: number): void => {
      if (context.debug) {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder('utf-8').decode(bytes);
        console.debug('[Spirit:debug]', text);
      }
    },

    /**
     * Abort execution with error message
     * Signature: vudo_abort(msg_ptr: i32, msg_len: i32, file_ptr: i32, file_len: i32, line: i32) -> void
     */
    vudo_abort: (
      msgPtr: number,
      msgLen: number,
      filePtr: number,
      fileLen: number,
      line: number
    ): void => {
      const msg = new TextDecoder().decode(
        new Uint8Array(context.memory.buffer, msgPtr, msgLen)
      );
      const file = new TextDecoder().decode(
        new Uint8Array(context.memory.buffer, filePtr, fileLen)
      );
      throw new Error(`Spirit abort: ${msg} at ${file}:${line}`);
    },
  }),
};

// ============================================================================
// Loa Registry
// ============================================================================

/**
 * Registry for managing Loa services
 *
 * Allows registration of custom Loas that provide host functions to Spirits.
 */
export class LoaRegistry implements ILoaRegistry {
  private loas: Map<string, Loa> = new Map();

  constructor() {
    // Register core Loa by default
    this.register(coreLoa);
  }

  /**
   * Register a new Loa
   * @param loa - Loa to register
   * @throws Error if Loa with same name already exists
   */
  register(loa: Loa): void {
    if (this.loas.has(loa.name)) {
      throw new Error(`Loa '${loa.name}' is already registered`);
    }
    this.loas.set(loa.name, loa);
  }

  /**
   * Get a Loa by name
   */
  get(name: string): Loa | undefined {
    return this.loas.get(name);
  }

  /**
   * Get all registered Loas
   */
  all(): Loa[] {
    return Array.from(this.loas.values());
  }

  /**
   * Check if a Loa is registered
   */
  has(name: string): boolean {
    return this.loas.has(name);
  }

  /**
   * Unregister a Loa
   */
  unregister(name: string): boolean {
    if (name === 'core') {
      throw new Error('Cannot unregister core Loa');
    }
    return this.loas.delete(name);
  }

  /**
   * Build WASM imports object from all registered Loas
   */
  buildImports(context: LoaContext): Record<string, WebAssembly.ImportValue> {
    const imports: Record<string, WebAssembly.ImportValue> = {};

    for (const loa of this.loas.values()) {
      const provided = loa.provides(context);
      Object.assign(imports, provided);
    }

    return imports;
  }
}

// ============================================================================
// Loa Factory Helpers
// ============================================================================

/**
 * Create a simple Loa from host function definitions
 */
export function createLoa(
  name: string,
  version: string,
  functions: Record<string, (context: LoaContext) => WebAssembly.ImportValue>
): Loa {
  return {
    name,
    version,
    capabilities: Object.keys(functions),
    provides: (context: LoaContext) => {
      const result: Record<string, WebAssembly.ImportValue> = {};
      for (const [key, factory] of Object.entries(functions)) {
        result[key] = factory(context);
      }
      return result;
    },
  };
}

/**
 * Create a logging Loa with custom logger
 */
export function createLoggingLoa(
  logger: {
    log: (msg: string) => void;
    error: (msg: string) => void;
    debug: (msg: string) => void;
  }
): Loa {
  return {
    name: 'logging',
    version: '1.0.0',
    capabilities: ['log', 'error', 'debug'],
    provides: (context: LoaContext) => ({
      vudo_print: (ptr: number, len: number): void => {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder('utf-8').decode(bytes);
        logger.log(text);
      },
      vudo_error: (ptr: number, len: number): void => {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder('utf-8').decode(bytes);
        logger.error(text);
      },
      vudo_debug: (ptr: number, len: number): void => {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder('utf-8').decode(bytes);
        logger.debug(text);
      },
    }),
  };
}
