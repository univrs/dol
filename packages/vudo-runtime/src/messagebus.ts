/**
 * MessageBus - Inter-Spirit communication
 *
 * Enables Spirits to send and receive messages within a Séance session.
 * Messages are queued per-Spirit and can be consumed synchronously.
 */

import type { Loa, LoaContext } from './types.js';

// ============================================================================
// Message Types
// ============================================================================

/**
 * A message sent between Spirits
 */
export interface Message {
  /** Source Spirit name */
  from: string;
  /** Destination Spirit name */
  to: string;
  /** Message type/channel identifier */
  channel: number;
  /** Raw payload bytes */
  payload: Uint8Array;
  /** Timestamp when message was sent */
  timestamp: number;
}

/**
 * Message handler callback type
 */
export type MessageHandler = (message: Message) => void;

// ============================================================================
// MessageBus Class
// ============================================================================

/**
 * Message bus for Spirit-to-Spirit communication
 *
 * @example
 * ```typescript
 * const bus = new MessageBus();
 *
 * // Register Spirits
 * bus.register('ping');
 * bus.register('pong');
 *
 * // Send a message
 * bus.send('ping', 'pong', 1, new Uint8Array([1, 2, 3]));
 *
 * // Receive message
 * const msg = bus.recv('pong', 1);
 * ```
 */
export class MessageBus {
  /** Message queues per Spirit name */
  private queues: Map<string, Message[]> = new Map();
  /** Global message handlers */
  private handlers: Map<string, MessageHandler[]> = new Map();
  /** Debug mode flag */
  private debug: boolean;

  constructor(options: { debug?: boolean } = {}) {
    this.debug = options.debug ?? false;
  }

  /**
   * Register a Spirit to receive messages
   */
  register(name: string): void {
    if (this.queues.has(name)) {
      throw new Error(`Spirit '${name}' is already registered on the message bus`);
    }
    this.queues.set(name, []);
    this.handlers.set(name, []);

    if (this.debug) {
      console.log(`[MessageBus] Registered Spirit '${name}'`);
    }
  }

  /**
   * Unregister a Spirit from the message bus
   */
  unregister(name: string): void {
    this.queues.delete(name);
    this.handlers.delete(name);

    if (this.debug) {
      console.log(`[MessageBus] Unregistered Spirit '${name}'`);
    }
  }

  /**
   * Check if a Spirit is registered
   */
  isRegistered(name: string): boolean {
    return this.queues.has(name);
  }

  /**
   * Send a message to a Spirit
   *
   * @param from - Source Spirit name
   * @param to - Destination Spirit name
   * @param channel - Message channel/type identifier
   * @param payload - Raw bytes to send
   * @returns true if message was delivered, false if destination not found
   */
  send(from: string, to: string, channel: number, payload: Uint8Array): boolean {
    const queue = this.queues.get(to);
    if (!queue) {
      if (this.debug) {
        console.warn(`[MessageBus] Cannot send to unknown Spirit '${to}'`);
      }
      return false;
    }

    const message: Message = {
      from,
      to,
      channel,
      payload,
      timestamp: Date.now(),
    };

    queue.push(message);

    if (this.debug) {
      console.log(
        `[MessageBus] ${from} -> ${to} (channel=${channel}, ${payload.length} bytes)`
      );
    }

    // Notify handlers
    const handlers = this.handlers.get(to);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(message);
        } catch (e) {
          console.error(`[MessageBus] Handler error:`, e);
        }
      }
    }

    return true;
  }

  /**
   * Receive a message for a Spirit
   *
   * @param name - Spirit name to receive for
   * @param channel - Optional channel filter (0 = any channel)
   * @returns Next message or undefined if queue is empty
   */
  recv(name: string, channel: number = 0): Message | undefined {
    const queue = this.queues.get(name);
    if (!queue || queue.length === 0) {
      return undefined;
    }

    if (channel === 0) {
      // Any channel - return first message
      return queue.shift();
    }

    // Filter by channel
    const idx = queue.findIndex((m) => m.channel === channel);
    if (idx === -1) {
      return undefined;
    }

    return queue.splice(idx, 1)[0];
  }

  /**
   * Peek at the next message without removing it
   */
  peek(name: string, channel: number = 0): Message | undefined {
    const queue = this.queues.get(name);
    if (!queue || queue.length === 0) {
      return undefined;
    }

    if (channel === 0) {
      return queue[0];
    }

    return queue.find((m) => m.channel === channel);
  }

  /**
   * Get the number of pending messages for a Spirit
   */
  pending(name: string, channel: number = 0): number {
    const queue = this.queues.get(name);
    if (!queue) {
      return 0;
    }

    if (channel === 0) {
      return queue.length;
    }

    return queue.filter((m) => m.channel === channel).length;
  }

  /**
   * Add a message handler for a Spirit
   */
  onMessage(name: string, handler: MessageHandler): void {
    const handlers = this.handlers.get(name);
    if (handlers) {
      handlers.push(handler);
    }
  }

  /**
   * Clear all messages for a Spirit
   */
  clear(name: string): void {
    const queue = this.queues.get(name);
    if (queue) {
      queue.length = 0;
    }
  }

  /**
   * Clear all messages in the bus
   */
  clearAll(): void {
    for (const queue of this.queues.values()) {
      queue.length = 0;
    }
  }

  /**
   * Get all registered Spirit names
   */
  spirits(): string[] {
    return Array.from(this.queues.keys());
  }
}

// ============================================================================
// Messaging Loa
// ============================================================================

/**
 * Create a messaging Loa that provides vudo_send and vudo_recv functions
 *
 * @param bus - MessageBus instance
 * @param spiritName - Name of the Spirit using this Loa
 */
export function createMessagingLoa(bus: MessageBus, spiritName: string): Loa {
  return {
    name: `messaging:${spiritName}`,
    version: '1.0.0',
    capabilities: ['send', 'recv', 'pending'],

    provides: (context: LoaContext) => ({
      /**
       * Send a message to another Spirit
       * Signature: vudo_send(to_ptr: i32, to_len: i32, channel: i32, payload_ptr: i32, payload_len: i32) -> i32
       * Returns: 1 = success, 0 = destination not found
       */
      vudo_send: (
        toPtr: number,
        toLen: number,
        channel: number,
        payloadPtr: number,
        payloadLen: number
      ): number => {
        const toBytes = new Uint8Array(context.memory.buffer, toPtr, toLen);
        const to = new TextDecoder('utf-8').decode(toBytes);

        const payload = new Uint8Array(
          context.memory.buffer.slice(payloadPtr, payloadPtr + payloadLen)
        );

        const success = bus.send(spiritName, to, channel, payload);
        return success ? 1 : 0;
      },

      /**
       * Receive a message
       * Signature: vudo_recv(channel: i32, from_buf: i32, from_max: i32, payload_buf: i32, payload_max: i32) -> i32
       * Returns: payload length if message received, -1 if no message, -2 if buffer too small
       *
       * Note: from_buf will be filled with null-terminated sender name
       */
      vudo_recv: (
        channel: number,
        fromBuf: number,
        fromMax: number,
        payloadBuf: number,
        payloadMax: number
      ): number => {
        const message = bus.recv(spiritName, channel);
        if (!message) {
          return -1; // No message available
        }

        const fromBytes = new TextEncoder().encode(message.from);

        // Check buffer sizes
        if (fromBytes.length + 1 > fromMax) {
          // Put message back (front of queue)
          const queue = bus['queues'].get(spiritName);
          if (queue) {
            queue.unshift(message);
          }
          return -2; // Buffer too small for sender name
        }

        if (message.payload.length > payloadMax) {
          // Put message back
          const queue = bus['queues'].get(spiritName);
          if (queue) {
            queue.unshift(message);
          }
          return -2; // Buffer too small for payload
        }

        // Copy sender name (null-terminated)
        const memory = new Uint8Array(context.memory.buffer);
        memory.set(fromBytes, fromBuf);
        memory[fromBuf + fromBytes.length] = 0;

        // Copy payload
        memory.set(message.payload, payloadBuf);

        return message.payload.length;
      },

      /**
       * Check number of pending messages
       * Signature: vudo_pending(channel: i32) -> i32
       * Returns: number of pending messages
       */
      vudo_pending: (channel: number): number => {
        return bus.pending(spiritName, channel);
      },
    }),
  };
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Create a new MessageBus
 */
export function createMessageBus(options?: { debug?: boolean }): MessageBus {
  return new MessageBus(options);
}
