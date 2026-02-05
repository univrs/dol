/**
 * VUDO Host Functions - Messaging Implementation
 *
 * Implements the 5 messaging host functions:
 * - vudo_send: Send message to target Spirit
 * - vudo_recv: Receive message with timeout
 * - vudo_pending: Check pending message count
 * - vudo_broadcast: Broadcast to all Spirits
 * - vudo_free_message: Free a received message
 *
 * @module @vudo/runtime/host/messaging
 */

import { ResultCode } from '../abi/types.js';

// ============================================================================
// Message Types
// ============================================================================

/**
 * A message sent between Spirits
 */
export interface Message {
  /** Unique message identifier */
  id: number;
  /** Source Spirit name */
  sender: string;
  /** Destination Spirit name (empty for broadcast) */
  recipient: string;
  /** Message payload bytes */
  payload: Uint8Array;
  /** Timestamp when message was created */
  timestamp: number;
}

/**
 * Message queue for a single Spirit
 */
interface MessageQueue {
  /** Spirit name */
  name: string;
  /** Queued messages */
  messages: Message[];
}

// ============================================================================
// Message Broker Interface
// ============================================================================

/**
 * Interface for message broker implementations
 *
 * Provides message routing between Spirits in a Séance session
 */
export interface IMessageBroker {
  /**
   * Register a Spirit to receive messages
   * @param name - Spirit name
   */
  register(name: string): void;

  /**
   * Unregister a Spirit
   * @param name - Spirit name
   */
  unregister(name: string): void;

  /**
   * Send a message to a specific Spirit
   * @param from - Sender Spirit name
   * @param to - Recipient Spirit name
   * @param payload - Message payload
   * @returns Result code
   */
  send(from: string, to: string, payload: Uint8Array): ResultCode;

  /**
   * Receive next message for a Spirit (with timeout)
   * @param name - Spirit name
   * @param timeoutMs - Timeout in milliseconds (0 = non-blocking)
   * @returns Message or null if timeout/no messages
   */
  recv(name: string, timeoutMs: number): Message | null;

  /**
   * Get count of pending messages for a Spirit
   * @param name - Spirit name
   * @returns Number of pending messages
   */
  pending(name: string): number;

  /**
   * Broadcast a message to all registered Spirits
   * @param from - Sender Spirit name
   * @param payload - Message payload
   * @returns Result code
   */
  broadcast(from: string, payload: Uint8Array): ResultCode;

  /**
   * Free a message (release memory)
   * @param messageId - Message ID to free
   */
  freeMessage(messageId: number): void;
}

// ============================================================================
// Message Broker Implementation
// ============================================================================

/**
 * Default message broker implementation
 */
export class MessageBroker implements IMessageBroker {
  /** Message queues per Spirit */
  private queues: Map<string, MessageQueue> = new Map();

  /** Allocated messages (for memory tracking) */
  private allocatedMessages: Map<number, Message> = new Map();

  /** Next message ID */
  private nextMessageId = 1;

  /** Debug mode */
  private debug: boolean;

  constructor(options: { debug?: boolean } = {}) {
    this.debug = options.debug ?? false;
  }

  register(name: string): void {
    if (this.queues.has(name)) {
      throw new Error(`Spirit '${name}' is already registered`);
    }

    this.queues.set(name, {
      name,
      messages: [],
    });

    if (this.debug) {
      console.log(`[MessageBroker] Registered Spirit '${name}'`);
    }
  }

  unregister(name: string): void {
    const queue = this.queues.get(name);
    if (queue) {
      // Free all pending messages
      for (const msg of queue.messages) {
        this.allocatedMessages.delete(msg.id);
      }
      this.queues.delete(name);

      if (this.debug) {
        console.log(`[MessageBroker] Unregistered Spirit '${name}'`);
      }
    }
  }

  send(from: string, to: string, payload: Uint8Array): ResultCode {
    const queue = this.queues.get(to);
    if (!queue) {
      if (this.debug) {
        console.warn(`[MessageBroker] Target Spirit '${to}' not found`);
      }
      return ResultCode.Error;
    }

    const message: Message = {
      id: this.nextMessageId++,
      sender: from,
      recipient: to,
      payload: new Uint8Array(payload), // Copy payload
      timestamp: Date.now(),
    };

    queue.messages.push(message);
    this.allocatedMessages.set(message.id, message);

    if (this.debug) {
      console.log(
        `[MessageBroker] ${from} -> ${to} (${payload.length} bytes, id=${message.id})`
      );
    }

    return ResultCode.Success;
  }

  recv(name: string, timeoutMs: number): Message | null {
    const queue = this.queues.get(name);
    if (!queue) {
      if (this.debug) {
        console.warn(`[MessageBroker] Spirit '${name}' not registered`);
      }
      return null;
    }

    // Check for immediate message
    if (queue.messages.length > 0) {
      const message = queue.messages.shift()!;
      if (this.debug) {
        console.log(`[MessageBroker] ${name} <- ${message.sender} (id=${message.id})`);
      }
      return message;
    }

    // Non-blocking if timeout is 0
    if (timeoutMs === 0) {
      return null;
    }

    // For timeout > 0, we would need async support
    // For now, just return null (synchronous implementation)
    if (this.debug) {
      console.log(`[MessageBroker] ${name} recv timeout (no async support)`);
    }
    return null;
  }

  pending(name: string): number {
    const queue = this.queues.get(name);
    return queue ? queue.messages.length : 0;
  }

  broadcast(from: string, payload: Uint8Array): ResultCode {
    const spirits = Array.from(this.queues.keys());
    if (spirits.length === 0) {
      return ResultCode.Success; // No spirits to broadcast to
    }

    let allSuccess = true;
    for (const spirit of spirits) {
      const result = this.send(from, spirit, payload);
      if (result !== ResultCode.Success) {
        allSuccess = false;
      }
    }

    if (this.debug) {
      console.log(
        `[MessageBroker] Broadcast from ${from} to ${spirits.length} spirits`
      );
    }

    return allSuccess ? ResultCode.Success : ResultCode.Error;
  }

  freeMessage(messageId: number): void {
    if (this.allocatedMessages.has(messageId)) {
      this.allocatedMessages.delete(messageId);
      if (this.debug) {
        console.log(`[MessageBroker] Freed message id=${messageId}`);
      }
    }
  }

  /**
   * Get all registered Spirit names
   */
  spirits(): string[] {
    return Array.from(this.queues.keys());
  }

  /**
   * Clear all messages for a Spirit
   */
  clear(name: string): void {
    const queue = this.queues.get(name);
    if (queue) {
      // Free all messages
      for (const msg of queue.messages) {
        this.allocatedMessages.delete(msg.id);
      }
      queue.messages = [];
    }
  }

  /**
   * Clear all messages
   */
  clearAll(): void {
    for (const queue of this.queues.values()) {
      for (const msg of queue.messages) {
        this.allocatedMessages.delete(msg.id);
      }
      queue.messages = [];
    }
  }
}

// ============================================================================
// Host Function Factory
// ============================================================================

/**
 * Create host function implementations for messaging
 *
 * @param broker - Message broker instance
 * @param spiritName - Name of the Spirit using these functions
 * @param memory - WASM memory instance
 * @returns Object with host function implementations
 */
export function createMessagingHost(
  broker: IMessageBroker,
  spiritName: string,
  memory: WebAssembly.Memory
) {
  /**
   * vudo_send: Send a message to another Spirit
   *
   * @param targetPtr - Pointer to target Spirit name (UTF-8)
   * @param targetLen - Length of target name in bytes
   * @param msgPtr - Pointer to message payload data
   * @param msgLen - Length of payload in bytes
   * @returns ResultCode (0 = success, non-zero = error)
   */
  function vudo_send(
    targetPtr: number,
    targetLen: number,
    msgPtr: number,
    msgLen: number
  ): number {
    try {
      const memView = new Uint8Array(memory.buffer);

      // Decode target name
      const targetBytes = memView.slice(targetPtr, targetPtr + targetLen);
      const target = new TextDecoder('utf-8').decode(targetBytes);

      // Copy payload
      const payload = memView.slice(msgPtr, msgPtr + msgLen);

      // Send via broker
      return broker.send(spiritName, target, payload);
    } catch (error) {
      console.error('[vudo_send] Error:', error);
      return ResultCode.Error;
    }
  }

  /**
   * vudo_recv: Receive the next message
   *
   * Message format in memory:
   * [sender_len: u32][sender: bytes][payload_len: u32][payload: bytes]
   *
   * @param timeoutMs - Timeout in milliseconds (0 = non-blocking)
   * @param outPtr - Pointer to output buffer
   * @param outLen - Size of output buffer
   * @returns Number of bytes written, -1 if no message, -2 if buffer too small
   */
  function vudo_recv(timeoutMs: number, outPtr: number, outLen: number): number {
    try {
      // Check if there's a message available first (peek)
      const queue = (broker as any).queues?.get(spiritName);
      if (!queue || queue.messages.length === 0) {
        return -1; // No message available
      }

      // Peek at the first message to check size
      const message = queue.messages[0];
      const senderBytes = new TextEncoder().encode(message.sender);
      const payloadBytes = message.payload;

      // Calculate required size: 4 (sender_len) + sender + 4 (payload_len) + payload
      const requiredSize = 4 + senderBytes.length + 4 + payloadBytes.length;
      if (requiredSize > outLen) {
        return -2; // Buffer too small - message stays in queue
      }

      // Now remove the message since buffer is large enough
      broker.recv(spiritName, timeoutMs);

      // Write to memory
      const memView = new Uint8Array(memory.buffer);
      const dataView = new DataView(memory.buffer);
      let offset = outPtr;

      // Write sender_len (u32 little-endian)
      dataView.setUint32(offset, senderBytes.length, true);
      offset += 4;

      // Write sender bytes
      memView.set(senderBytes, offset);
      offset += senderBytes.length;

      // Write payload_len (u32 little-endian)
      dataView.setUint32(offset, payloadBytes.length, true);
      offset += 4;

      // Write payload bytes
      memView.set(payloadBytes, offset);
      offset += payloadBytes.length;

      return requiredSize;
    } catch (error) {
      console.error('[vudo_recv] Error:', error);
      return ResultCode.Error;
    }
  }

  /**
   * vudo_pending: Get count of pending messages
   *
   * @returns Number of pending messages
   */
  function vudo_pending(): number {
    return broker.pending(spiritName);
  }

  /**
   * vudo_broadcast: Broadcast a message to all Spirits
   *
   * @param msgPtr - Pointer to message payload data
   * @param msgLen - Length of payload in bytes
   * @returns ResultCode (0 = success, non-zero = error)
   */
  function vudo_broadcast(msgPtr: number, msgLen: number): number {
    try {
      const memView = new Uint8Array(memory.buffer);
      const payload = memView.slice(msgPtr, msgPtr + msgLen);
      return broker.broadcast(spiritName, payload);
    } catch (error) {
      console.error('[vudo_broadcast] Error:', error);
      return ResultCode.Error;
    }
  }

  /**
   * vudo_free_message: Free a received message
   *
   * @param msgId - Message ID (from message header)
   */
  function vudo_free_message(msgId: number): void {
    broker.freeMessage(msgId);
  }

  return {
    vudo_send,
    vudo_recv,
    vudo_pending,
    vudo_broadcast,
    vudo_free_message,
  };
}

// ============================================================================
// Exports
// ============================================================================

export { ResultCode };
