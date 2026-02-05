/**
 * Host Function Registry - Aggregates all 22 host functions
 *
 * This registry creates WASM imports object with all host functions bound to providers.
 * It serves as the main entry point for integrating the VUDO host with Spirit instances.
 *
 * @module @vudo/runtime/host/registry
 */

import type { BumpAllocator } from '../memory.js';
import {
  LogLevel,
  ResultCode,
  type HostFunctionMetadata,
  HOST_FUNCTION_REGISTRY,
} from '../abi/host.js';

// ============================================================================
// PROVIDER INTERFACES
// ============================================================================

/**
 * Time provider interface for host-side time functions
 */
export interface ITimeProvider {
  /**
   * Get current timestamp in milliseconds since Unix epoch
   */
  now(): bigint;

  /**
   * Get monotonic time in nanoseconds
   */
  monotonic_now(): bigint;

  /**
   * Sleep for specified milliseconds
   * @returns Promise that resolves after sleep duration
   */
  sleep(ms: number): Promise<void>;
}

/**
 * Logger interface for host-side logging
 */
export interface ILogger {
  /**
   * Log a message at specified level
   */
  log(level: LogLevel, message: string): void;

  /**
   * Log debug message
   */
  debug(message: string): void;

  /**
   * Log info message
   */
  info(message: string): void;

  /**
   * Log warning message
   */
  warn(message: string): void;

  /**
   * Log error message
   */
  error(message: string): void;
}

/**
 * Message broker interface for inter-Spirit communication
 */
export interface IMessageBroker {
  /**
   * Send a message to a target Spirit
   */
  send(target: string, payload: Uint8Array): ResultCode;

  /**
   * Receive next message from inbox
   */
  recv(spiritPtr: number, spiritId: string): number | null;

  /**
   * Get number of pending messages
   */
  pending(spiritId: string): number;

  /**
   * Broadcast message to all Spirits
   */
  broadcast(payload: Uint8Array, sender: string): ResultCode;

  /**
   * Free a message after processing
   */
  freeMessage(ptr: number): void;
}

/**
 * Random provider interface
 */
export interface IRandomProvider {
  /**
   * Generate random f64 in [0, 1)
   */
  random(): number;

  /**
   * Generate random bytes
   */
  randomBytes(len: number): Uint8Array;
}

/**
 * Effect handler interface for side effects
 */
export interface IEffectHandler {
  /**
   * Emit a side effect
   */
  emitEffect(effectId: number, payload: Uint8Array): ResultCode;

  /**
   * Subscribe to effect channel
   */
  subscribe(channel: string, spiritId: string): ResultCode;
}

/**
 * Debug handler interface
 */
export interface IDebugHandler {
  /**
   * Trigger breakpoint
   */
  breakpoint(): void;

  /**
   * Assert condition
   */
  assert(condition: boolean, message: string): void;

  /**
   * Panic with message (terminates Spirit)
   */
  panic(message: string): void;
}

// ============================================================================
// HOST FUNCTION REGISTRY CLASS
// ============================================================================

/**
 * HostFunctionRegistry - Aggregates all 22 host functions
 *
 * Creates WASM imports object with all host functions bound to providers.
 * Handles UTF-8 encoding/decoding, memory allocation, and provider delegation.
 *
 * @example
 * ```typescript
 * const memory = new WebAssembly.Memory({ initial: 256 });
 * const allocator = new BumpAllocator(memory);
 * const providers = {
 *   time: new DefaultTimeProvider(),
 *   logger: new DefaultLogger(),
 *   messageBroker: new DefaultMessageBroker(),
 *   random: new DefaultRandomProvider(),
 *   effectHandler: new DefaultEffectHandler(),
 *   debugHandler: new DefaultDebugHandler(),
 * };
 *
 * const registry = new HostFunctionRegistry(memory, allocator, providers);
 * const imports = registry.getImports();
 * ```
 */
export class HostFunctionRegistry {
  private memory: WebAssembly.Memory;
  private allocator: BumpAllocator;
  private decoder: TextDecoder;
  private encoder: TextEncoder;

  // Providers
  private timeProvider: ITimeProvider;
  private logger: ILogger;
  private messageBroker: IMessageBroker;
  private randomProvider: IRandomProvider;
  private effectHandler: IEffectHandler;
  private debugHandler: IDebugHandler;

  // Current Spirit ID (set when creating imports for specific Spirit)
  private spiritId: string = 'unknown';

  /**
   * Create a new HostFunctionRegistry
   *
   * @param memory - WASM memory instance
   * @param allocator - BumpAllocator instance
   * @param providers - Object containing all provider interfaces
   */
  constructor(
    memory: WebAssembly.Memory,
    allocator: BumpAllocator,
    providers: {
      time: ITimeProvider;
      logger: ILogger;
      messageBroker: IMessageBroker;
      random: IRandomProvider;
      effectHandler: IEffectHandler;
      debugHandler: IDebugHandler;
    },
    spiritId?: string,
  ) {
    this.memory = memory;
    this.allocator = allocator;
    this.decoder = new TextDecoder('utf-8');
    this.encoder = new TextEncoder();

    this.timeProvider = providers.time;
    this.logger = providers.logger;
    this.messageBroker = providers.messageBroker;
    this.randomProvider = providers.random;
    this.effectHandler = providers.effectHandler;
    this.debugHandler = providers.debugHandler;

    if (spiritId) {
      this.spiritId = spiritId;
    }
  }

  /**
   * Get the WASM imports object with all 22 host functions
   *
   * @returns Object with structure { vudo: { vudo_print, vudo_println, ... } }
   */
  getImports(): { vudo: Record<string, (...args: unknown[]) => unknown> } {
    return {
      vudo: {
        // ====================================================================
        // I/O FUNCTIONS (4)
        // ====================================================================

        /**
         * Print a UTF-8 string to console (without newline)
         */
        vudo_print: (ptr: number, len: number): void => {
          const bytes = new Uint8Array(this.memory.buffer, ptr, len);
          const text = this.decoder.decode(bytes);
          process.stdout.write(text);
        },

        /**
         * Print a UTF-8 string to console (with newline)
         */
        vudo_println: (ptr: number, len: number): void => {
          const bytes = new Uint8Array(this.memory.buffer, ptr, len);
          const text = this.decoder.decode(bytes);
          console.log(text);
        },

        /**
         * Structured logging with level
         */
        vudo_log: (level: number, ptr: number, len: number): void => {
          const bytes = new Uint8Array(this.memory.buffer, ptr, len);
          const text = this.decoder.decode(bytes);
          this.logger.log(level as LogLevel, text);
        },

        /**
         * Log an error message
         */
        vudo_error: (ptr: number, len: number): void => {
          const bytes = new Uint8Array(this.memory.buffer, ptr, len);
          const text = this.decoder.decode(bytes);
          this.logger.error(text);
        },

        // ====================================================================
        // MEMORY FUNCTIONS (3)
        // ====================================================================

        /**
         * Allocate memory from host allocator
         */
        vudo_alloc: (size: number): number => {
          return this.allocator.alloc(size, 8);
        },

        /**
         * Free previously allocated memory
         */
        vudo_free: (_ptr: number, _size: number): void => {
          // Bump allocator doesn't support individual frees
          // Memory is reclaimed on reset()
        },

        /**
         * Reallocate memory (grow or shrink)
         */
        vudo_realloc: (ptr: number, oldSize: number, newSize: number): number => {
          // For bump allocator, we allocate new space and copy data
          if (newSize === 0) {
            return 0; // Invalid allocation
          }

          const newPtr = this.allocator.alloc(newSize, 8);
          if (newPtr === 0) {
            return 0; // Allocation failed
          }

          // Copy old data to new location
          const oldBytes = new Uint8Array(this.memory.buffer, ptr, oldSize);
          const newBytes = new Uint8Array(this.memory.buffer, newPtr, newSize);
          const copySize = Math.min(oldSize, newSize);
          newBytes.set(oldBytes.subarray(0, copySize));

          return newPtr;
        },

        // ====================================================================
        // TIME FUNCTIONS (3)
        // ====================================================================

        /**
         * Get current timestamp in milliseconds since Unix epoch
         */
        vudo_now: (): bigint => {
          return this.timeProvider.now();
        },

        /**
         * Sleep for specified milliseconds
         */
        vudo_sleep: (ms: number): void => {
          // Sleep is typically async, but we return void
          // Host should handle this asynchronously
          this.timeProvider.sleep(ms).catch((err) => {
            this.logger.error(`Sleep failed: ${String(err)}`);
          });
        },

        /**
         * Get monotonic time in nanoseconds
         */
        vudo_monotonic_now: (): bigint => {
          return this.timeProvider.monotonic_now();
        },

        // ====================================================================
        // MESSAGING FUNCTIONS (5)
        // ====================================================================

        /**
         * Send a message to another Spirit
         */
        vudo_send: (
          targetPtr: number,
          targetLen: number,
          payloadPtr: number,
          payloadLen: number,
        ): number => {
          const targetBytes = new Uint8Array(this.memory.buffer, targetPtr, targetLen);
          const target = this.decoder.decode(targetBytes);

          const payloadBytes = new Uint8Array(this.memory.buffer, payloadPtr, payloadLen);
          const payload = new Uint8Array(payloadBytes); // Copy

          return this.messageBroker.send(target, payload);
        },

        /**
         * Receive next message from inbox
         */
        vudo_recv: (): number => {
          const ptr = this.messageBroker.recv(0, this.spiritId);
          return ptr ?? 0;
        },

        /**
         * Check number of pending messages
         */
        vudo_pending: (): number => {
          return this.messageBroker.pending(this.spiritId);
        },

        /**
         * Broadcast message to all Spirits
         */
        vudo_broadcast: (ptr: number, len: number): number => {
          const payloadBytes = new Uint8Array(this.memory.buffer, ptr, len);
          const payload = new Uint8Array(payloadBytes); // Copy
          return this.messageBroker.broadcast(payload, this.spiritId);
        },

        /**
         * Free a received message
         */
        vudo_free_message: (ptr: number): void => {
          this.messageBroker.freeMessage(ptr);
        },

        // ====================================================================
        // RANDOM FUNCTIONS (2)
        // ====================================================================

        /**
         * Generate random f64 in [0, 1)
         */
        vudo_random: (): number => {
          return this.randomProvider.random();
        },

        /**
         * Generate random bytes
         */
        vudo_random_bytes: (ptr: number, len: number): void => {
          const randomBytes = this.randomProvider.randomBytes(len);
          const buffer = new Uint8Array(this.memory.buffer, ptr, len);
          buffer.set(randomBytes);
        },

        // ====================================================================
        // EFFECT FUNCTIONS (2)
        // ====================================================================

        /**
         * Emit a side effect for host handling
         */
        vudo_emit_effect: (effectId: number, payloadPtr: number, payloadLen: number): number => {
          const payloadBytes = new Uint8Array(this.memory.buffer, payloadPtr, payloadLen);
          const payload = new Uint8Array(payloadBytes); // Copy
          return this.effectHandler.emitEffect(effectId, payload);
        },

        /**
         * Subscribe to effect channel
         */
        vudo_subscribe: (channelPtr: number, channelLen: number): number => {
          const channelBytes = new Uint8Array(this.memory.buffer, channelPtr, channelLen);
          const channel = this.decoder.decode(channelBytes);
          return this.effectHandler.subscribe(channel, this.spiritId);
        },

        // ====================================================================
        // DEBUG FUNCTIONS (3)
        // ====================================================================

        /**
         * Trigger breakpoint (debug builds only)
         */
        vudo_breakpoint: (): void => {
          this.debugHandler.breakpoint();
        },

        /**
         * Assert condition with message
         */
        vudo_assert: (condition: number, ptr: number, len: number): void => {
          const messageBytes = new Uint8Array(this.memory.buffer, ptr, len);
          const message = this.decoder.decode(messageBytes);
          this.debugHandler.assert(condition !== 0, message);
        },

        /**
         * Panic with message (terminates Spirit)
         */
        vudo_panic: (ptr: number, len: number): void => {
          const messageBytes = new Uint8Array(this.memory.buffer, ptr, len);
          const message = this.decoder.decode(messageBytes);
          this.debugHandler.panic(message);
        },
      },
    };
  }

  /**
   * Get registry metadata
   *
   * @returns Array of all 22 host function metadata
   */
  getMetadata(): HostFunctionMetadata[] {
    return Object.values(HOST_FUNCTION_REGISTRY);
  }

  /**
   * Set the current Spirit ID for messaging context
   *
   * @param spiritId - The ID of the Spirit using this registry
   */
  setSpiritId(spiritId: string): void {
    this.spiritId = spiritId;
  }

  /**
   * Get the current Spirit ID
   */
  getSpiritId(): string {
    return this.spiritId;
  }
}

/**
 * Helper function to verify all 22 functions are present
 *
 * @param imports - Imports object to verify
 * @returns True if all 22 functions are present
 */
export function verifyImports(imports: { vudo: Record<string, unknown> }): boolean {
  const required = [
    // I/O Functions
    'vudo_print',
    'vudo_println',
    'vudo_log',
    'vudo_error',

    // Memory Functions
    'vudo_alloc',
    'vudo_free',
    'vudo_realloc',

    // Time Functions
    'vudo_now',
    'vudo_sleep',
    'vudo_monotonic_now',

    // Messaging Functions
    'vudo_send',
    'vudo_recv',
    'vudo_pending',
    'vudo_broadcast',
    'vudo_free_message',

    // Random Functions
    'vudo_random',
    'vudo_random_bytes',

    // Effect Functions
    'vudo_emit_effect',
    'vudo_subscribe',

    // Debug Functions
    'vudo_breakpoint',
    'vudo_assert',
    'vudo_panic',
  ];

  return required.every((name) => name in imports.vudo && typeof imports.vudo[name] === 'function');
}
