/**
 * Default Provider Implementations
 *
 * Provides default/base implementations of all provider interfaces
 * that can be used directly or extended for custom behavior.
 *
 * @module @vudo/runtime/host/providers
 */

import {
  type ITimeProvider,
  type ILogger,
  type IMessageBroker,
  type IRandomProvider,
  type IEffectHandler,
  type IDebugHandler,
} from './registry.js';
import { LogLevel, ResultCode } from '../abi/host.js';

// ============================================================================
// DEFAULT TIME PROVIDER
// ============================================================================

/**
 * Default time provider using system time
 */
export class DefaultTimeProvider implements ITimeProvider {
  /**
   * Get current timestamp in milliseconds
   */
  now(): bigint {
    return BigInt(Date.now());
  }

  /**
   * Get monotonic time in nanoseconds
   * Falls back to performance.now() if available
   */
  monotonic_now(): bigint {
    if (typeof performance !== 'undefined' && typeof performance.now === 'function') {
      return BigInt(Math.floor(performance.now() * 1000000));
    }
    // Fallback: use current timestamp converted to nanoseconds
    return BigInt(Date.now()) * BigInt(1000000);
  }

  /**
   * Sleep for specified milliseconds
   */
  async sleep(ms: number): Promise<void> {
    return new Promise((resolve) => {
      setTimeout(resolve, Math.max(0, ms));
    });
  }
}

// ============================================================================
// DEFAULT LOGGER
// ============================================================================

/**
 * Default logger that outputs to console
 */
export class DefaultLogger implements ILogger {
  private minLevel: LogLevel;

  /**
   * Create logger with minimum log level
   * @param minLevel - Minimum level to output (default: DEBUG)
   */
  constructor(minLevel: LogLevel = LogLevel.DEBUG) {
    this.minLevel = minLevel;
  }

  /**
   * Log at specified level
   */
  log(level: LogLevel, message: string): void {
    if (level < this.minLevel) {
      return;
    }

    const levelName = LogLevel[level] || 'UNKNOWN';
    const prefix = `[${levelName}]`;

    switch (level) {
      case LogLevel.DEBUG:
        console.debug(prefix, message);
        break;
      case LogLevel.INFO:
        console.info(prefix, message);
        break;
      case LogLevel.WARN:
        console.warn(prefix, message);
        break;
      case LogLevel.ERROR:
        console.error(prefix, message);
        break;
      default:
        console.log(prefix, message);
    }
  }

  debug(message: string): void {
    this.log(LogLevel.DEBUG, message);
  }

  info(message: string): void {
    this.log(LogLevel.INFO, message);
  }

  warn(message: string): void {
    this.log(LogLevel.WARN, message);
  }

  error(message: string): void {
    this.log(LogLevel.ERROR, message);
  }
}

// ============================================================================
// DEFAULT MESSAGE BROKER
// ============================================================================

/**
 * Simple in-memory message broker
 * Messages are stored per Spirit and can be sent/received
 */
export class DefaultMessageBroker implements IMessageBroker {
  private inboxes: Map<string, Uint8Array[]> = new Map();
  private allocatedMessages: Map<number, Uint8Array> = new Map();
  private nextMessagePtr = 10000; // Start message pointers at safe offset

  /**
   * Send a message to target Spirit
   */
  send(target: string, payload: Uint8Array): ResultCode {
    if (!this.inboxes.has(target)) {
      this.inboxes.set(target, []);
    }

    const inbox = this.inboxes.get(target);
    if (!inbox) {
      return ResultCode.NotFound;
    }

    // Copy payload into inbox
    const messageCopy = new Uint8Array(payload);
    inbox.push(messageCopy);

    return ResultCode.Ok;
  }

  /**
   * Receive next message from Spirit's inbox
   */
  recv(_spiritPtr: number, spiritId: string): number | null {
    const inbox = this.inboxes.get(spiritId);
    if (!inbox || inbox.length === 0) {
      return null;
    }

    const payload = inbox.shift();
    if (!payload) {
      return null;
    }

    // Allocate message pointer and store payload
    const ptr = this.nextMessagePtr++;
    this.allocatedMessages.set(ptr, payload);

    return ptr;
  }

  /**
   * Get number of pending messages
   */
  pending(spiritId: string): number {
    const inbox = this.inboxes.get(spiritId);
    return inbox ? inbox.length : 0;
  }

  /**
   * Broadcast message to all Spirits
   */
  broadcast(payload: Uint8Array, _sender: string): ResultCode {
    let broadcastCount = 0;

    const spiritIds = Array.from(this.inboxes.keys());
    for (const spiritId of spiritIds) {
      const result = this.send(spiritId, payload);
      if (result === ResultCode.Ok) {
        broadcastCount++;
      }
    }

    return broadcastCount > 0 ? ResultCode.Ok : ResultCode.Error;
  }

  /**
   * Free a message after processing
   */
  freeMessage(ptr: number): void {
    this.allocatedMessages.delete(ptr);
  }

  /**
   * Register a new Spirit in the broker
   */
  registerSpirit(spiritId: string): void {
    if (!this.inboxes.has(spiritId)) {
      this.inboxes.set(spiritId, []);
    }
  }

  /**
   * Unregister a Spirit from the broker
   */
  unregisterSpirit(spiritId: string): void {
    this.inboxes.delete(spiritId);
  }

  /**
   * Clear all messages (for testing)
   */
  clear(): void {
    this.inboxes.clear();
    this.allocatedMessages.clear();
    this.nextMessagePtr = 10000;
  }
}

// ============================================================================
// DEFAULT RANDOM PROVIDER
// ============================================================================

/**
 * Default random provider using Math.random()
 */
export class DefaultRandomProvider implements IRandomProvider {
  /**
   * Generate random f64 in [0, 1)
   */
  random(): number {
    return Math.random();
  }

  /**
   * Generate random bytes
   */
  randomBytes(len: number): Uint8Array {
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
      bytes[i] = Math.floor(Math.random() * 256);
    }
    return bytes;
  }
}

// ============================================================================
// DEFAULT EFFECT HANDLER
// ============================================================================

/**
 * Default effect handler (logs but doesn't handle effects)
 */
export class DefaultEffectHandler implements IEffectHandler {
  private logger: ILogger;
  private subscriptions: Map<string, Set<string>> = new Map();

  constructor(logger?: ILogger) {
    this.logger = logger || new DefaultLogger();
  }

  /**
   * Emit a side effect
   * Default implementation logs the effect but doesn't handle it
   */
  emitEffect(effectId: number, payload: Uint8Array): ResultCode {
    this.logger.debug(`Effect ${effectId} emitted with payload (${payload.length} bytes)`);
    return ResultCode.Ok;
  }

  /**
   * Subscribe to effect channel
   */
  subscribe(channel: string, spiritId: string): ResultCode {
    if (!this.subscriptions.has(channel)) {
      this.subscriptions.set(channel, new Set());
    }

    const subscribers = this.subscriptions.get(channel);
    if (!subscribers) {
      return ResultCode.Error;
    }

    subscribers.add(spiritId);
    this.logger.debug(`Spirit ${spiritId} subscribed to channel ${channel}`);

    return ResultCode.Ok;
  }

  /**
   * Get subscribers for a channel
   */
  getSubscribers(channel: string): Set<string> {
    return this.subscriptions.get(channel) || new Set();
  }

  /**
   * Clear all subscriptions
   */
  clear(): void {
    this.subscriptions.clear();
  }
}

// ============================================================================
// DEFAULT DEBUG HANDLER
// ============================================================================

/**
 * Default debug handler (logs but doesn't actually implement debugging)
 */
export class DefaultDebugHandler implements IDebugHandler {
  private logger: ILogger;

  constructor(logger?: ILogger) {
    this.logger = logger || new DefaultLogger();
  }

  /**
   * Trigger breakpoint (no-op in default implementation)
   */
  breakpoint(): void {
    this.logger.debug('Breakpoint triggered');
  }

  /**
   * Assert condition
   */
  assert(condition: boolean, message: string): void {
    if (!condition) {
      this.logger.error(`Assertion failed: ${message}`);
    }
  }

  /**
   * Panic with message (terminates Spirit)
   */
  panic(message: string): void {
    this.logger.error(`PANIC: ${message}`);
    // In a real implementation, this would terminate the Spirit
    throw new Error(`Spirit panic: ${message}`);
  }
}

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

/**
 * Create default providers object
 *
 * @param options - Configuration options
 * @returns Object with all default providers
 *
 * @example
 * ```typescript
 * const providers = createDefaultProviders({ logLevel: LogLevel.INFO });
 * const registry = new HostFunctionRegistry(memory, allocator, providers);
 * ```
 */
export function createDefaultProviders(options?: {
  logLevel?: LogLevel;
  logger?: ILogger;
  messageBroker?: IMessageBroker;
}): {
  time: ITimeProvider;
  logger: ILogger;
  messageBroker: IMessageBroker;
  random: IRandomProvider;
  effectHandler: IEffectHandler;
  debugHandler: IDebugHandler;
} {
  const logger = options?.logger || new DefaultLogger(options?.logLevel ?? LogLevel.DEBUG);
  const messageBroker = options?.messageBroker || new DefaultMessageBroker();

  return {
    time: new DefaultTimeProvider(),
    logger,
    messageBroker,
    random: new DefaultRandomProvider(),
    effectHandler: new DefaultEffectHandler(logger),
    debugHandler: new DefaultDebugHandler(logger),
  };
}
