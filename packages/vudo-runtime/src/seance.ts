/**
 * Séance - Session management for multiple Spirit instances
 *
 * A Séance coordinates multiple Spirits, allowing them to interact
 * within a shared session context.
 */

import type { LoadOptions, SeanceInstance, SpiritInstance } from './types.js';
import { Spirit, loadSpirit } from './spirit.js';
import { LoaRegistry } from './loa.js';

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
  private debug: boolean;
  private defaultOptions: LoadOptions;

  constructor(options: {
    loas?: LoaRegistry;
    debug?: boolean;
    defaultLoadOptions?: LoadOptions;
  } = {}) {
    this.registry = options.loas ?? new LoaRegistry();
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

    const mergedOptions: LoadOptions = {
      ...this.defaultOptions,
      ...options,
      loas: options.loas ?? this.registry,
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

    // Reset all Spirit memory allocators
    for (const [name, spirit] of this.spiritMap) {
      if (this.debug) {
        console.log(`[Séance] Releasing Spirit '${name}'...`);
      }
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
   * Get the number of summoned Spirits
   */
  get size(): number {
    return this.spiritMap.size;
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
  options?: { loas?: LoaRegistry; debug?: boolean }
): Promise<T> {
  const seance = new Seance(options);

  try {
    return await fn(seance);
  } finally {
    await seance.dismiss();
  }
}
