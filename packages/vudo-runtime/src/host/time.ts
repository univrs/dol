/**
 * Time Host Functions Implementation
 *
 * Implements the 3 time-related host functions:
 * - vudo_now: Get current timestamp in milliseconds
 * - vudo_sleep: Sleep for specified duration
 * - vudo_monotonic_now: Get monotonic time in nanoseconds
 *
 * These functions provide time and sleep capabilities to WASM modules.
 *
 * @module @vudo/runtime/host/time
 */

import { ResultCode } from '../abi/types.js';
import type { ITimeProvider } from './interfaces.js';
import { HostFunctionError } from './interfaces.js';

/**
 * Configuration for time host functions
 */
export interface TimeConfig {
  /** Time provider implementation */
  timeProvider: ITimeProvider;
  /** Maximum sleep duration in milliseconds (default 1 hour) */
  maxSleepDuration?: number;
  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Time host function provider
 *
 * Provides time-related operations with validation and
 * configurable time sources for testing.
 */
export class TimeHostFunctions {
  private timeProvider: ITimeProvider;
  private maxSleepDuration: number;
  private debug: boolean;
  private sleepCount = 0;
  private totalSleepTime = 0;

  constructor(config: TimeConfig) {
    this.timeProvider = config.timeProvider;
    this.maxSleepDuration = config.maxSleepDuration ?? 60 * 60 * 1000; // 1 hour
    this.debug = config.debug ?? false;
  }

  /**
   * Validate sleep duration
   * @param ms - Sleep duration in milliseconds
   * @throws HostFunctionError if duration is invalid
   */
  private validateSleepDuration(ms: number): void {
    if (ms < 0) {
      throw new HostFunctionError(
        `Invalid sleep duration: ${ms} (must be non-negative)`,
        ResultCode.InvalidArg
      );
    }

    if (ms > this.maxSleepDuration) {
      throw new HostFunctionError(
        `Sleep duration ${ms}ms exceeds maximum ${this.maxSleepDuration}ms`,
        ResultCode.InvalidArg
      );
    }
  }

  /**
   * Log debug message if debug mode is enabled
   */
  private debugLog(message: string): void {
    if (this.debug) {
      console.debug(`[TimeHostFunctions] ${message}`);
    }
  }

  /**
   * Get current timestamp in milliseconds since Unix epoch
   *
   * Implements vudo_now() -> i64
   *
   * Returns the number of milliseconds elapsed since
   * 00:00:00 UTC on 1 January 1970.
   *
   * @returns Timestamp in milliseconds as bigint
   */
  now(): bigint {
    try {
      const timestamp = this.timeProvider.now();
      this.debugLog(`now() = ${timestamp}ms`);
      return timestamp;
    } catch (error) {
      this.debugLog(
        `now() error: ${error instanceof Error ? error.message : String(error)}`
      );
      // Return current time as fallback
      return BigInt(Date.now());
    }
  }

  /**
   * Sleep for specified number of milliseconds
   *
   * Implements vudo_sleep(ms: i32) -> void
   *
   * This is an asynchronous operation that blocks the Spirit
   * while allowing other Spirits to execute. In the WASM context,
   * this is typically handled by the runtime's event loop.
   *
   * Note: This implementation is synchronous in the host function
   * but the runtime should handle it asynchronously.
   *
   * @param ms - Duration to sleep in milliseconds
   */
  sleep(ms: number): void {
    try {
      // Treat negative values as 0 (immediate return)
      if (ms < 0) {
        this.debugLog(`sleep(${ms}) treated as sleep(0) - invalid negative duration`);
        return;
      }

      // Validate against maximum
      this.validateSleepDuration(ms);

      // Track statistics
      this.sleepCount++;
      this.totalSleepTime += ms;

      this.debugLog(
        `sleep(${ms}ms) requested (total sleeps: ${this.sleepCount}, total time: ${this.totalSleepTime}ms)`
      );

      // Call time provider's sleep (though we can't await it in a sync function)
      // This allows the mock to track sleep calls for testing
      this.timeProvider.sleep(ms);

      // Note: Actual sleep implementation would be handled by the runtime
      // using async/await or other scheduling mechanism
    } catch (error) {
      this.debugLog(
        `sleep(${ms}) error: ${error instanceof Error ? error.message : String(error)}`
      );
      // Sleep errors are non-fatal - just return
    }
  }

  /**
   * Get monotonic time in nanoseconds
   *
   * Implements vudo_monotonic_now() -> i64
   *
   * Returns a monotonically increasing timestamp suitable for
   * measuring elapsed time. Unlike now(), this never goes backward.
   *
   * @returns Monotonic time in nanoseconds as bigint
   */
  monotonicNow(): bigint {
    try {
      const timestamp = this.timeProvider.monotonicNow();
      this.debugLog(`monotonic_now() = ${timestamp}ns`);
      return timestamp;
    } catch (error) {
      this.debugLog(
        `monotonic_now() error: ${error instanceof Error ? error.message : String(error)}`
      );
      // Fallback to performance.now() converted to nanoseconds
      if (typeof performance !== 'undefined') {
        return BigInt(Math.floor(performance.now() * 1_000_000));
      }
      // Last resort: Date.now() in nanoseconds
      return BigInt(Date.now()) * 1_000_000n;
    }
  }

  /**
   * Get sleep statistics
   *
   * @returns Object with sleep metrics
   */
  getStats(): {
    sleepCount: number;
    totalSleepTime: number;
    averageSleepTime: number;
  } {
    return {
      sleepCount: this.sleepCount,
      totalSleepTime: this.totalSleepTime,
      averageSleepTime:
        this.sleepCount > 0 ? this.totalSleepTime / this.sleepCount : 0,
    };
  }

  /**
   * Reset sleep statistics
   */
  resetStats(): void {
    this.sleepCount = 0;
    this.totalSleepTime = 0;
    this.debugLog('Statistics reset');
  }

  /**
   * Build WebAssembly imports object for time functions
   *
   * @returns Object with time host functions bound to this instance
   */
  buildImports(): Record<string, Function> {
    return {
      vudo_now: this.now.bind(this),
      vudo_sleep: this.sleep.bind(this),
      vudo_monotonic_now: this.monotonicNow.bind(this),
    };
  }
}

/**
 * Create time host functions with default configuration
 *
 * @param timeProvider - Time provider implementation
 * @param debug - Enable debug logging
 * @returns Configured TimeHostFunctions instance
 */
export function createTimeHostFunctions(
  timeProvider: ITimeProvider,
  debug = false
): TimeHostFunctions {
  return new TimeHostFunctions({ timeProvider, debug });
}
