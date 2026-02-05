/**
 * VUDO Host Functions - Random Implementation
 *
 * Implements the 2 random host functions:
 * - vudo_random: Generate random f64 in [0, 1)
 * - vudo_random_bytes: Fill buffer with random bytes
 *
 * Uses crypto.getRandomValues for cryptographically secure randomness
 * with Math.random as fallback.
 *
 * @module @vudo/runtime/host/random
 */

// ============================================================================
// Random Provider Interface
// ============================================================================

/**
 * Interface for random number providers
 */
export interface IRandomProvider {
  /**
   * Generate a random f64 in [0, 1)
   * @returns Random number in range [0.0, 1.0)
   */
  random(): number;

  /**
   * Fill a buffer with random bytes
   * @param buffer - Uint8Array to fill with random bytes
   */
  randomBytes(buffer: Uint8Array): void;
}

// ============================================================================
// Crypto Random Provider (Secure)
// ============================================================================

/**
 * Cryptographically secure random provider using crypto.getRandomValues
 *
 * This is the preferred provider for production use.
 */
export class CryptoRandomProvider implements IRandomProvider {
  private crypto: Crypto;

  constructor() {
    // Check for crypto support (Node.js or browser)
    if (typeof globalThis.crypto !== 'undefined') {
      this.crypto = globalThis.crypto;
    } else if (typeof require !== 'undefined') {
      // Node.js without globalThis.crypto
      try {
        const nodeCrypto = require('crypto');
        this.crypto = nodeCrypto.webcrypto as Crypto;
      } catch {
        throw new Error('crypto module not available');
      }
    } else {
      throw new Error('No crypto implementation available');
    }
  }

  random(): number {
    // Generate random u32 and convert to f64 in [0, 1)
    const bytes = new Uint32Array(1);
    this.crypto.getRandomValues(bytes);
    // Divide by 2^32 to get [0, 1)
    return bytes[0] / 0x100000000;
  }

  randomBytes(buffer: Uint8Array): void {
    this.crypto.getRandomValues(buffer);
  }
}

// ============================================================================
// Math Random Provider (Fallback)
// ============================================================================

/**
 * Non-cryptographic random provider using Math.random
 *
 * This is a fallback for environments without crypto support.
 * NOT suitable for security-sensitive operations.
 */
export class MathRandomProvider implements IRandomProvider {
  random(): number {
    return Math.random();
  }

  randomBytes(buffer: Uint8Array): void {
    for (let i = 0; i < buffer.length; i++) {
      // Generate random byte [0, 255]
      buffer[i] = Math.floor(Math.random() * 256);
    }
  }
}

// ============================================================================
// Deterministic Random Provider (Testing)
// ============================================================================

/**
 * Deterministic random provider for testing
 *
 * Uses a simple LCG (Linear Congruential Generator) with a seed.
 * Produces reproducible sequences for testing.
 */
export class DeterministicRandomProvider implements IRandomProvider {
  private state: number;

  /**
   * Create a deterministic random provider
   * @param seed - Random seed (default: 12345)
   */
  constructor(seed: number = 12345) {
    this.state = seed >>> 0; // Ensure unsigned 32-bit
  }

  /**
   * LCG parameters (same as glibc)
   */
  private next(): number {
    // LCG: next = (a * prev + c) mod m
    // a = 1103515245, c = 12345, m = 2^31
    this.state = ((this.state * 1103515245 + 12345) >>> 0) & 0x7fffffff;
    return this.state;
  }

  random(): number {
    return this.next() / 0x7fffffff; // Divide by 2^31 - 1
  }

  randomBytes(buffer: Uint8Array): void {
    for (let i = 0; i < buffer.length; i++) {
      buffer[i] = this.next() & 0xff;
    }
  }

  /**
   * Reset the seed
   * @param seed - New seed value
   */
  reset(seed: number): void {
    this.state = seed >>> 0;
  }
}

// ============================================================================
// Host Function Factory
// ============================================================================

/**
 * Create host function implementations for random number generation
 *
 * @param provider - Random provider implementation
 * @param memory - WASM memory instance
 * @returns Object with host function implementations
 */
export function createRandomHost(
  provider: IRandomProvider,
  memory: WebAssembly.Memory
) {
  /**
   * vudo_random: Generate a random f64 in [0, 1)
   *
   * @returns Random number in range [0.0, 1.0)
   */
  function vudo_random(): number {
    return provider.random();
  }

  /**
   * vudo_random_bytes: Fill a buffer with random bytes
   *
   * @param ptr - Pointer to buffer in WASM linear memory
   * @param len - Number of random bytes to generate
   */
  function vudo_random_bytes(ptr: number, len: number): void {
    const memView = new Uint8Array(memory.buffer);
    const buffer = memView.subarray(ptr, ptr + len);
    provider.randomBytes(buffer);
  }

  return {
    vudo_random,
    vudo_random_bytes,
  };
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Create a crypto random provider (secure)
 */
export function createCryptoProvider(): IRandomProvider {
  return new CryptoRandomProvider();
}

/**
 * Create a Math.random provider (fallback)
 */
export function createMathProvider(): IRandomProvider {
  return new MathRandomProvider();
}

/**
 * Create a deterministic provider (testing)
 * @param seed - Random seed
 */
export function createDeterministicProvider(seed?: number): IRandomProvider {
  return new DeterministicRandomProvider(seed);
}

/**
 * Create the default random provider
 * Tries crypto first, falls back to Math.random
 */
export function createDefaultProvider(): IRandomProvider {
  try {
    return new CryptoRandomProvider();
  } catch {
    console.warn('[Random] Falling back to Math.random (not cryptographically secure)');
    return new MathRandomProvider();
  }
}
