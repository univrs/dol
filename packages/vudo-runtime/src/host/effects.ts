/**
 * Effects System - Host Functions
 *
 * Implements the effects host functions for the VUDO runtime:
 * - vudo_emit_effect: Emit an effect for the host to handle
 * - vudo_subscribe: Subscribe to effect pattern notifications
 *
 * Effects enable Spirits to trigger side effects and subscribe to
 * effect channels using pattern matching (exact and wildcard "*").
 *
 * @module @vudo/runtime/host/effects
 */

import type { StandardEffect } from '../abi/types.js';
import { ResultCode } from '../abi/types.js';

/**
 * Subscriber registration for effect patterns
 */
interface EffectSubscriber {
  /** Subscriber ID (for tracking) */
  id: number;
  /** Pattern to match (exact string or wildcard "*") */
  pattern: string;
  /** Callback function to notify on matching effects */
  callback: (effect: StandardEffect) => void;
}

/**
 * Handler interface for effects system
 *
 * Implementors can customize effect handling and subscription behavior.
 *
 * @example
 * ```typescript
 * class MyEffectHandler implements IEffectHandler {
 *   handleEffect(effect: StandardEffect): ResultCode {
 *     console.log(`Effect: ${effect.effect_type}`, effect.payload);
 *     return ResultCode.Success;
 *   }
 *
 *   onSubscribe(pattern: string, subscriptionId: number): void {
 *     console.log(`Subscribed to pattern: ${pattern} (ID: ${subscriptionId})`);
 *   }
 * }
 * ```
 */
export interface IEffectHandler {
  /**
   * Handle an emitted effect
   *
   * @param effect - The effect to handle
   * @returns Result code indicating success or failure
   */
  handleEffect(effect: StandardEffect): ResultCode;

  /**
   * Called when a new subscription is registered
   *
   * @param pattern - The pattern being subscribed to
   * @param subscriptionId - Unique subscription identifier
   */
  onSubscribe(pattern: string, subscriptionId: number): void;
}

/**
 * Default effect handler implementation
 *
 * Logs effects to console and tracks subscriptions.
 */
export class DefaultEffectHandler implements IEffectHandler {
  handleEffect(effect: StandardEffect): ResultCode {
    console.log(`[Effect] ${effect.effect_type}:`, effect.payload);
    return ResultCode.Success;
  }

  onSubscribe(pattern: string, subscriptionId: number): void {
    console.log(`[Effect] Subscribed to pattern: ${pattern} (ID: ${subscriptionId})`);
  }
}

/**
 * Effects system managing effect emission and subscriptions
 *
 * Provides:
 * - Effect emission with JSON parsing
 * - Pattern-based subscriptions (exact match and wildcard "*")
 * - Effect queue with subscriber notifications
 * - Thread-safe subscription management
 */
export class EffectsSystem {
  private memory: WebAssembly.Memory;
  private handler: IEffectHandler;
  private subscribers: Map<number, EffectSubscriber>;
  private nextSubscriberId: number;
  private effectQueue: StandardEffect[];

  /**
   * Create a new effects system
   *
   * @param memory - WASM memory instance for reading effect data
   * @param handler - Handler for processing effects (defaults to DefaultEffectHandler)
   */
  constructor(memory: WebAssembly.Memory, handler?: IEffectHandler) {
    this.memory = memory;
    this.handler = handler || new DefaultEffectHandler();
    this.subscribers = new Map();
    this.nextSubscriberId = 1;
    this.effectQueue = [];
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
   * Parse effect JSON from WASM memory
   *
   * @param ptr - Pointer to JSON data
   * @param len - Length in bytes
   * @returns Parsed StandardEffect object
   * @throws {Error} If JSON is invalid
   */
  private parseEffect(ptr: number, len: number): StandardEffect {
    const jsonStr = this.decodeString(ptr, len);
    const effect = JSON.parse(jsonStr) as StandardEffect;

    // Validate required fields
    if (!effect.effect_type || typeof effect.effect_type !== 'string') {
      throw new Error('Invalid effect: missing or invalid effect_type');
    }

    // Set timestamp if not provided
    if (!effect.timestamp) {
      effect.timestamp = Date.now();
    }

    return effect;
  }

  /**
   * Check if an effect matches a subscription pattern
   *
   * Patterns:
   * - Exact match: "log" matches effect_type "log"
   * - Wildcard: "*" matches any effect_type
   *
   * @param effectType - The effect type to match
   * @param pattern - The subscription pattern
   * @returns True if the effect matches the pattern
   */
  private matchesPattern(effectType: string, pattern: string): boolean {
    if (pattern === '*') {
      return true;
    }
    return effectType === pattern;
  }

  /**
   * Notify subscribers of an effect
   *
   * Calls subscriber callbacks for patterns that match the effect type.
   *
   * @param effect - The effect to notify about
   */
  private notifySubscribers(effect: StandardEffect): void {
    for (const subscriber of this.subscribers.values()) {
      if (this.matchesPattern(effect.effect_type, subscriber.pattern)) {
        try {
          subscriber.callback(effect);
        } catch (error) {
          console.error(`[Effect] Subscriber ${subscriber.id} callback error:`, error);
        }
      }
    }
  }

  /**
   * Emit an effect
   *
   * Host function: vudo_emit_effect(effect_ptr, effect_len)
   *
   * Parses JSON effect from WASM memory, processes it through the handler,
   * and notifies matching subscribers.
   *
   * @param effectPtr - Pointer to effect JSON data
   * @param effectLen - Length of JSON data in bytes
   * @returns ResultCode (0 = Success, non-zero = Error)
   */
  emitEffect(effectPtr: number, effectLen: number): ResultCode {
    try {
      // Parse effect from JSON
      const effect = this.parseEffect(effectPtr, effectLen);

      // Add to effect queue
      this.effectQueue.push(effect);

      // Process through handler
      const result = this.handler.handleEffect(effect);

      // Notify subscribers
      this.notifySubscribers(effect);

      return result;
    } catch (error) {
      console.error('[Effect] Failed to emit effect:', error);
      return ResultCode.Error;
    }
  }

  /**
   * Subscribe to effect pattern
   *
   * Host function: vudo_subscribe(pattern_ptr, pattern_len)
   *
   * Registers a subscription to receive notifications when effects
   * matching the pattern are emitted.
   *
   * @param patternPtr - Pointer to pattern string
   * @param patternLen - Length of pattern string in bytes
   * @param callback - Callback function to invoke on matching effects
   * @returns Subscription ID (positive number) or error code (negative)
   */
  subscribe(
    patternPtr: number,
    patternLen: number,
    callback: (effect: StandardEffect) => void
  ): number {
    try {
      // Decode pattern string
      const pattern = this.decodeString(patternPtr, patternLen);

      // Create subscriber
      const subscriberId = this.nextSubscriberId++;
      const subscriber: EffectSubscriber = {
        id: subscriberId,
        pattern,
        callback,
      };

      // Register subscriber
      this.subscribers.set(subscriberId, subscriber);

      // Notify handler
      this.handler.onSubscribe(pattern, subscriberId);

      return subscriberId;
    } catch (error) {
      console.error('[Effect] Failed to subscribe:', error);
      return ResultCode.Error;
    }
  }

  /**
   * Unsubscribe from effect notifications
   *
   * @param subscriptionId - ID returned from subscribe()
   * @returns True if subscription was removed, false if not found
   */
  unsubscribe(subscriptionId: number): boolean {
    return this.subscribers.delete(subscriptionId);
  }

  /**
   * Get all effects in the queue
   *
   * @returns Array of effects
   */
  getEffectQueue(): readonly StandardEffect[] {
    return [...this.effectQueue];
  }

  /**
   * Clear the effect queue
   */
  clearEffectQueue(): void {
    this.effectQueue = [];
  }

  /**
   * Get all active subscriptions
   *
   * @returns Array of subscriber info
   */
  getSubscriptions(): Array<{ id: number; pattern: string }> {
    return Array.from(this.subscribers.values()).map((sub) => ({
      id: sub.id,
      pattern: sub.pattern,
    }));
  }

  /**
   * Create WASM host function implementations
   *
   * Returns an object with vudo_emit_effect and vudo_subscribe functions
   * suitable for WASM imports.
   *
   * @returns Host function implementations
   */
  createHostFunctions(): {
    vudo_emit_effect: (effectPtr: number, effectLen: number) => number;
    vudo_subscribe: (patternPtr: number, patternLen: number) => number;
  } {
    return {
      vudo_emit_effect: (effectPtr: number, effectLen: number): number => {
        return this.emitEffect(effectPtr, effectLen);
      },
      vudo_subscribe: (patternPtr: number, patternLen: number): number => {
        // Default callback logs to console
        const callback = (effect: StandardEffect) => {
          console.log(`[Subscription] Effect received:`, effect);
        };
        return this.subscribe(patternPtr, patternLen, callback);
      },
    };
  }
}

/**
 * Create a new effects system instance
 *
 * Convenience factory function for creating an effects system.
 *
 * @param memory - WASM memory instance
 * @param handler - Optional custom effect handler
 * @returns New EffectsSystem instance
 *
 * @example
 * ```typescript
 * const effects = createEffectsSystem(memory);
 * const hostFunctions = effects.createHostFunctions();
 * ```
 */
export function createEffectsSystem(
  memory: WebAssembly.Memory,
  handler?: IEffectHandler
): EffectsSystem {
  return new EffectsSystem(memory, handler);
}
