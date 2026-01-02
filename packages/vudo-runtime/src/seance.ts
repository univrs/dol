/**
 * Séance - Session management for multiple Spirit instances
 *
 * A Séance coordinates multiple Spirits, allowing them to interact
 * within a shared session context.
 */

import type { LoadOptions, SeanceInstance, SpiritInstance } from './types.js';
import { Spirit, loadSpirit } from './spirit.js';
import { LoaRegistry } from './loa.js';
import { MessageBus, createMessagingLoa } from './messagebus.js';

// ============================================================================
// Séance Class
// ============================================================================

/**
 * Session manager for coordinating multiple Spirit instances
 *
 * @example
 * ```typescript
 * const seance = new Seance();
 *
 * // Summon Spirits
 * await seance.summon('calc', '/spirits/calculator.wasm');
 * await seance.summon('logger', '/spirits/logger.wasm');
 *
 * // Invoke functions
 * const result = await seance.invoke('calc', 'add', [1, 2]);
 * await seance.invoke('logger', 'log', [`Result: ${result}`]);
 *
 * // Cleanup
 * await seance.dismiss();
 * ```
 */
export class Seance implements SeanceInstance {
  private spiritMap: Map<string, Spirit> = new Map();
  private registry: LoaRegistry;
  private messageBus: MessageBus;
  private debug: boolean;
  private defaultOptions: LoadOptions;

  constructor(options: {
    loas?: LoaRegistry;
    messageBus?: MessageBus;
    debug?: boolean;
    defaultLoadOptions?: LoadOptions;
  } = {}) {
    this.registry = options.loas ?? new LoaRegistry();
    this.messageBus = options.messageBus ?? new MessageBus({ debug: options.debug });
    this.debug = options.debug ?? false;
    this.defaultOptions = options.defaultLoadOptions ?? {};
  }

  /**
   * Summon a Spirit into the session
   *
   * @param name - Unique name for the Spirit within this session
   * @param source - WASM bytes or URL to load from
   * @param options - Additional load options
   */
  async summon(
    name: string,
    source: string | ArrayBuffer | Uint8Array,
    options: LoadOptions = {}
  ): Promise<void> {
    if (this.spiritMap.has(name)) {
      throw new Error(`Spirit '${name}' is already summoned in this session`);
    }

    // Register Spirit with the MessageBus
    this.messageBus.register(name);

    // Create messaging Loa for this Spirit
    const messagingLoa = createMessagingLoa(this.messageBus, name);

    // Create a registry that includes the messaging Loa
    const spiritRegistry = options.loas ?? this.registry;
    spiritRegistry.register(messagingLoa);

    const mergedOptions: LoadOptions = {
      ...this.defaultOptions,
      ...options,
      loas: spiritRegistry,
      debug: options.debug ?? this.debug,
    };

    if (this.debug) {
      console.log(`[Séance] Summoning Spirit '${name}'...`);
    }

    const spirit = await loadSpirit(source, mergedOptions);
    this.spiritMap.set(name, spirit);

    if (this.debug) {
      console.log(`[Séance] Spirit '${name}' summoned successfully`);
    }
  }

  /**
   * Invoke a function on a summoned Spirit
   *
   * @param spiritName - Name of the Spirit to invoke
   * @param funcName - Function name to call
   * @param args - Arguments to pass
   * @returns Function result
   */
  async invoke<R = unknown>(
    spiritName: string,
    funcName: string,
    args: unknown[] = []
  ): Promise<R> {
    const spirit = this.spiritMap.get(spiritName);

    if (!spirit) {
      throw new Error(`Spirit '${spiritName}' not found in session`);
    }

    if (this.debug) {
      console.log(`[Séance] Invoking ${spiritName}.${funcName}(${args.join(', ')})`);
    }

    // Note: call is synchronous for WASM, but we return Promise for API consistency
    const result = spirit.call<R>(funcName, args);

    if (this.debug) {
      console.log(`[Séance] ${spiritName}.${funcName} returned:`, result);
    }

    return result;
  }

  /**
   * Get a summoned Spirit by name
   */
  getSpirit(name: string): SpiritInstance | undefined {
    return this.spiritMap.get(name);
  }

  /**
   * Check if a Spirit is summoned
   */
  hasSpirit(name: string): boolean {
    return this.spiritMap.has(name);
  }

  /**
   * List all summoned Spirit names
   */
  spirits(): string[] {
    return Array.from(this.spiritMap.keys());
  }

  /**
   * Dismiss a specific Spirit from the session
   */
  async release(name: string): Promise<void> {
    if (!this.spiritMap.has(name)) {
      throw new Error(`Spirit '${name}' not found in session`);
    }

    if (this.debug) {
      console.log(`[Séance] Releasing Spirit '${name}'...`);
    }

    // Clear any pending messages and unregister from MessageBus
    this.messageBus.clear(name);
    this.messageBus.unregister(name);

    // Reset Spirit's memory allocator
    const spirit = this.spiritMap.get(name);
    if (spirit) {
      spirit.memory.reset();
    }

    this.spiritMap.delete(name);

    if (this.debug) {
      console.log(`[Séance] Spirit '${name}' released`);
    }
  }

  /**
   * Dismiss the session and clean up all Spirits
   */
  async dismiss(): Promise<void> {
    if (this.debug) {
      console.log(`[Séance] Dismissing session with ${this.spiritMap.size} Spirit(s)...`);
    }

    // Clear MessageBus and unregister all Spirits
    this.messageBus.clearAll();

    // Reset all Spirit memory allocators
    for (const [name, spirit] of this.spiritMap) {
      if (this.debug) {
        console.log(`[Séance] Releasing Spirit '${name}'...`);
      }
      this.messageBus.unregister(name);
      spirit.memory.reset();
    }

    this.spiritMap.clear();

    if (this.debug) {
      console.log('[Séance] Session dismissed');
    }
  }

  /**
   * Get the Loa registry for this session
   */
  get loas(): LoaRegistry {
    return this.registry;
  }

  /**
   * Get the MessageBus for this session
   */
  get messages(): MessageBus {
    return this.messageBus;
  }

  /**
   * Get the number of summoned Spirits
   */
  get size(): number {
    return this.spiritMap.size;
  }

  // ===========================================================================
  // Messaging Convenience Methods
  // ===========================================================================

  /**
   * Send a message from one Spirit to another
   *
   * @param from - Source Spirit name
   * @param to - Destination Spirit name
   * @param channel - Message channel identifier
   * @param payload - Data to send
   * @returns true if message was delivered
   *
   * @example
   * ```typescript
   * seance.send('ping', 'pong', 1, new Uint8Array([1, 2, 3]));
   * ```
   */
  send(from: string, to: string, channel: number, payload: Uint8Array): boolean {
    return this.messageBus.send(from, to, channel, payload);
  }

  /**
   * Check number of pending messages for a Spirit
   *
   * @param name - Spirit name
   * @param channel - Optional channel filter (0 = all channels)
   */
  pending(name: string, channel: number = 0): number {
    return this.messageBus.pending(name, channel);
  }

  /**
   * Broadcast a message to all Spirits except the sender
   *
   * @param from - Source Spirit name
   * @param channel - Message channel identifier
   * @param payload - Data to send
   * @returns Number of Spirits that received the message
   *
   * @example
   * ```typescript
   * seance.broadcast('coordinator', 1, new Uint8Array([0xFF]));
   * ```
   */
  broadcast(from: string, channel: number, payload: Uint8Array): number {
    let delivered = 0;
    for (const name of this.spiritMap.keys()) {
      if (name !== from) {
        if (this.messageBus.send(from, name, channel, payload)) {
          delivered++;
        }
      }
    }
    return delivered;
  }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Create a new Séance session
 */
export function createSeance(options?: {
  loas?: LoaRegistry;
  messageBus?: MessageBus;
  debug?: boolean;
}): Seance {
  return new Seance(options);
}

/**
 * Run a Spirit session with automatic cleanup
 *
 * @example
 * ```typescript
 * await withSeance(async (seance) => {
 *   await seance.summon('calc', './calculator.wasm');
 *   const result = await seance.invoke('calc', 'add', [1, 2]);
 *   console.log('Result:', result);
 * });
 * // Session is automatically dismissed
 * ```
 */
export async function withSeance<T>(
  fn: (seance: Seance) => Promise<T>,
  options?: { loas?: LoaRegistry; messageBus?: MessageBus; debug?: boolean }
): Promise<T> {
  const seance = new Seance(options);

  try {
    return await fn(seance);
  } finally {
    await seance.dismiss();
  }
}
