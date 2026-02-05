/**
 * Tests for random host functions
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  CryptoRandomProvider,
  MathRandomProvider,
  DeterministicRandomProvider,
  createRandomHost,
  createCryptoProvider,
  createMathProvider,
  createDeterministicProvider,
  createDefaultProvider,
  type IRandomProvider,
} from '../src/host/random.js';

describe('CryptoRandomProvider', () => {
  let provider: IRandomProvider;

  beforeEach(() => {
    try {
      provider = new CryptoRandomProvider();
    } catch (error) {
      // Skip tests if crypto not available
      return;
    }
  });

  it('should generate random numbers in [0, 1)', () => {
    if (!provider) return;

    for (let i = 0; i < 100; i++) {
      const value = provider.random();
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThan(1);
    }
  });

  it('should generate different random numbers', () => {
    if (!provider) return;

    const values = new Set<number>();
    for (let i = 0; i < 100; i++) {
      values.add(provider.random());
    }

    // Should have many unique values (not deterministic)
    expect(values.size).toBeGreaterThan(90);
  });

  it('should generate random bytes', () => {
    if (!provider) return;

    const buffer = new Uint8Array(32);
    provider.randomBytes(buffer);

    // Check that buffer was filled (not all zeros)
    const sum = buffer.reduce((a, b) => a + b, 0);
    expect(sum).toBeGreaterThan(0);
  });

  it('should fill entire buffer with random bytes', () => {
    if (!provider) return;

    const buffer = new Uint8Array(1024);
    provider.randomBytes(buffer);

    // Check each byte is in valid range
    for (let i = 0; i < buffer.length; i++) {
      expect(buffer[i]).toBeGreaterThanOrEqual(0);
      expect(buffer[i]).toBeLessThanOrEqual(255);
    }

    // Check for randomness (not all same value)
    const firstByte = buffer[0];
    const allSame = buffer.every((b) => b === firstByte);
    expect(allSame).toBe(false);
  });

  it('should handle empty buffer', () => {
    if (!provider) return;

    const buffer = new Uint8Array(0);
    provider.randomBytes(buffer);
    // Should not throw
  });
});

describe('MathRandomProvider', () => {
  let provider: IRandomProvider;

  beforeEach(() => {
    provider = new MathRandomProvider();
  });

  it('should generate random numbers in [0, 1)', () => {
    for (let i = 0; i < 100; i++) {
      const value = provider.random();
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThan(1);
    }
  });

  it('should generate different random numbers', () => {
    const values = new Set<number>();
    for (let i = 0; i < 100; i++) {
      values.add(provider.random());
    }

    // Should have many unique values
    expect(values.size).toBeGreaterThan(90);
  });

  it('should generate random bytes', () => {
    const buffer = new Uint8Array(32);
    provider.randomBytes(buffer);

    // Check that buffer was filled
    const sum = buffer.reduce((a, b) => a + b, 0);
    expect(sum).toBeGreaterThan(0);
  });

  it('should generate bytes in valid range', () => {
    const buffer = new Uint8Array(256);
    provider.randomBytes(buffer);

    for (let i = 0; i < buffer.length; i++) {
      expect(buffer[i]).toBeGreaterThanOrEqual(0);
      expect(buffer[i]).toBeLessThanOrEqual(255);
    }
  });
});

describe('DeterministicRandomProvider', () => {
  it('should generate reproducible sequences', () => {
    const provider1 = new DeterministicRandomProvider(12345);
    const provider2 = new DeterministicRandomProvider(12345);

    // Generate sequences
    const seq1 = Array.from({ length: 10 }, () => provider1.random());
    const seq2 = Array.from({ length: 10 }, () => provider2.random());

    // Should be identical
    expect(seq1).toEqual(seq2);
  });

  it('should generate different sequences for different seeds', () => {
    const provider1 = new DeterministicRandomProvider(12345);
    const provider2 = new DeterministicRandomProvider(54321);

    const seq1 = Array.from({ length: 10 }, () => provider1.random());
    const seq2 = Array.from({ length: 10 }, () => provider2.random());

    // Should be different
    expect(seq1).not.toEqual(seq2);
  });

  it('should generate numbers in [0, 1)', () => {
    const provider = new DeterministicRandomProvider();

    for (let i = 0; i < 100; i++) {
      const value = provider.random();
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThan(1);
    }
  });

  it('should generate reproducible byte sequences', () => {
    const provider1 = new DeterministicRandomProvider(99999);
    const provider2 = new DeterministicRandomProvider(99999);

    const buffer1 = new Uint8Array(64);
    const buffer2 = new Uint8Array(64);

    provider1.randomBytes(buffer1);
    provider2.randomBytes(buffer2);

    expect(Array.from(buffer1)).toEqual(Array.from(buffer2));
  });

  it('should reset to same sequence', () => {
    const provider = new DeterministicRandomProvider(12345);

    const seq1 = Array.from({ length: 5 }, () => provider.random());

    provider.reset(12345);
    const seq2 = Array.from({ length: 5 }, () => provider.random());

    expect(seq1).toEqual(seq2);
  });

  it('should use default seed', () => {
    const provider1 = new DeterministicRandomProvider();
    const provider2 = new DeterministicRandomProvider();

    const val1 = provider1.random();
    const val2 = provider2.random();

    expect(val1).toBe(val2);
  });
});

describe('createRandomHost', () => {
  let provider: IRandomProvider;
  let memory: WebAssembly.Memory;

  beforeEach(() => {
    provider = new DeterministicRandomProvider(42);
    memory = new WebAssembly.Memory({ initial: 1 });
  });

  describe('vudo_random', () => {
    it('should generate random f64', () => {
      const host = createRandomHost(provider, memory);

      const value = host.vudo_random();

      expect(typeof value).toBe('number');
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThan(1);
    });

    it('should use deterministic provider', () => {
      const detProvider = new DeterministicRandomProvider(12345);
      const host = createRandomHost(detProvider, memory);

      const value1 = host.vudo_random();
      const value2 = host.vudo_random();

      // Reset and verify reproducibility
      detProvider.reset(12345);
      const value1again = host.vudo_random();

      expect(value1again).toBe(value1);
      expect(value1).not.toBe(value2); // Different values in sequence
    });

    it('should generate many different values', () => {
      const mathProvider = new MathRandomProvider();
      const host = createRandomHost(mathProvider, memory);

      const values = new Set<number>();
      for (let i = 0; i < 100; i++) {
        values.add(host.vudo_random());
      }

      expect(values.size).toBeGreaterThan(90);
    });
  });

  describe('vudo_random_bytes', () => {
    it('should fill buffer with random bytes', () => {
      const host = createRandomHost(provider, memory);
      const memView = new Uint8Array(memory.buffer);

      // Fill 32 bytes starting at offset 100
      host.vudo_random_bytes(100, 32);

      // Check that bytes were written
      const buffer = memView.slice(100, 132);
      const sum = buffer.reduce((a, b) => a + b, 0);
      expect(sum).toBeGreaterThan(0);
    });

    it('should write to correct memory location', () => {
      const host = createRandomHost(provider, memory);
      const memView = new Uint8Array(memory.buffer);

      // Zero out test area
      for (let i = 0; i < 200; i++) {
        memView[i] = 0;
      }

      // Fill 16 bytes at offset 50
      host.vudo_random_bytes(50, 16);

      // Check that only target range was modified
      const before = memView.slice(0, 50);
      const target = memView.slice(50, 66);
      const after = memView.slice(66, 100);

      expect(before.every((b) => b === 0)).toBe(true);
      expect(after.every((b) => b === 0)).toBe(true);
      expect(target.some((b) => b !== 0)).toBe(true);
    });

    it('should handle large buffers', () => {
      const host = createRandomHost(provider, memory);
      const memView = new Uint8Array(memory.buffer);

      // Fill 4KB
      host.vudo_random_bytes(0, 4096);

      const buffer = memView.slice(0, 4096);

      // Check all bytes are in valid range
      for (let i = 0; i < buffer.length; i++) {
        expect(buffer[i]).toBeGreaterThanOrEqual(0);
        expect(buffer[i]).toBeLessThanOrEqual(255);
      }
    });

    it('should generate reproducible bytes with deterministic provider', () => {
      const detProvider1 = new DeterministicRandomProvider(99999);
      const detProvider2 = new DeterministicRandomProvider(99999);

      const memory1 = new WebAssembly.Memory({ initial: 1 });
      const memory2 = new WebAssembly.Memory({ initial: 1 });

      const host1 = createRandomHost(detProvider1, memory1);
      const host2 = createRandomHost(detProvider2, memory2);

      host1.vudo_random_bytes(0, 128);
      host2.vudo_random_bytes(0, 128);

      const buffer1 = new Uint8Array(memory1.buffer).slice(0, 128);
      const buffer2 = new Uint8Array(memory2.buffer).slice(0, 128);

      expect(Array.from(buffer1)).toEqual(Array.from(buffer2));
    });

    it('should handle zero-length buffer', () => {
      const host = createRandomHost(provider, memory);

      // Should not throw
      host.vudo_random_bytes(0, 0);
    });

    it('should support subarray efficiently', () => {
      const host = createRandomHost(provider, memory);
      const memView = new Uint8Array(memory.buffer);

      // Fill at different offsets
      host.vudo_random_bytes(0, 100);
      host.vudo_random_bytes(200, 100);
      host.vudo_random_bytes(400, 100);

      // Check that all areas were filled
      const area1 = memView.slice(0, 100);
      const area2 = memView.slice(200, 300);
      const area3 = memView.slice(400, 500);

      expect(area1.some((b) => b !== 0)).toBe(true);
      expect(area2.some((b) => b !== 0)).toBe(true);
      expect(area3.some((b) => b !== 0)).toBe(true);
    });
  });

  describe('integration scenarios', () => {
    it('should generate random UUID-like bytes', () => {
      const host = createRandomHost(provider, memory);

      // Generate 16 bytes for UUID
      host.vudo_random_bytes(0, 16);

      const memView = new Uint8Array(memory.buffer);
      const uuid = Array.from(memView.slice(0, 16))
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('');

      expect(uuid.length).toBe(32);
    });

    it('should generate random salt', () => {
      const mathProvider = new MathRandomProvider();
      const host = createRandomHost(mathProvider, memory);

      // Generate 32-byte salt
      host.vudo_random_bytes(0, 32);

      const memView = new Uint8Array(memory.buffer);
      const salt = memView.slice(0, 32);

      // Should be filled with random data
      const uniqueBytes = new Set(Array.from(salt));
      expect(uniqueBytes.size).toBeGreaterThan(10); // At least some variety
    });

    it('should simulate dice rolls', () => {
      const mathProvider = new MathRandomProvider();
      const host = createRandomHost(mathProvider, memory);

      const rolls: number[] = [];
      for (let i = 0; i < 100; i++) {
        const value = host.vudo_random();
        const diceRoll = Math.floor(value * 6) + 1;
        rolls.push(diceRoll);
      }

      // Check all rolls are valid (1-6)
      expect(rolls.every((r) => r >= 1 && r <= 6)).toBe(true);

      // Check for variety (should have at least 5 different values)
      const uniqueRolls = new Set(rolls);
      expect(uniqueRolls.size).toBeGreaterThanOrEqual(5);
    });

    it('should generate random coordinates', () => {
      const host = createRandomHost(provider, memory);

      const points: Array<{ x: number; y: number }> = [];
      for (let i = 0; i < 50; i++) {
        const x = host.vudo_random() * 100; // 0-100
        const y = host.vudo_random() * 100; // 0-100
        points.push({ x, y });
      }

      // Check all points are in valid range
      expect(points.every((p) => p.x >= 0 && p.x < 100)).toBe(true);
      expect(points.every((p) => p.y >= 0 && p.y < 100)).toBe(true);
    });
  });
});

describe('convenience functions', () => {
  it('should create crypto provider', () => {
    try {
      const provider = createCryptoProvider();
      expect(provider).toBeInstanceOf(CryptoRandomProvider);

      const value = provider.random();
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThan(1);
    } catch {
      // Skip if crypto not available
    }
  });

  it('should create math provider', () => {
    const provider = createMathProvider();
    expect(provider).toBeInstanceOf(MathRandomProvider);

    const value = provider.random();
    expect(value).toBeGreaterThanOrEqual(0);
    expect(value).toBeLessThan(1);
  });

  it('should create deterministic provider with default seed', () => {
    const provider = createDeterministicProvider();
    expect(provider).toBeInstanceOf(DeterministicRandomProvider);

    const value = provider.random();
    expect(value).toBeGreaterThanOrEqual(0);
    expect(value).toBeLessThan(1);
  });

  it('should create deterministic provider with custom seed', () => {
    const provider1 = createDeterministicProvider(42);
    const provider2 = createDeterministicProvider(42);

    expect(provider1.random()).toBe(provider2.random());
  });

  it('should create default provider', () => {
    const provider = createDefaultProvider();
    expect(provider).toBeDefined();

    const value = provider.random();
    expect(value).toBeGreaterThanOrEqual(0);
    expect(value).toBeLessThan(1);
  });
});

describe('statistical properties', () => {
  it('should have approximately uniform distribution', () => {
    const provider = new MathRandomProvider();
    const host = createRandomHost(provider, new WebAssembly.Memory({ initial: 1 }));

    const buckets = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // 10 buckets
    const samples = 10000;

    for (let i = 0; i < samples; i++) {
      const value = host.vudo_random();
      const bucket = Math.floor(value * 10);
      buckets[bucket]++;
    }

    // Each bucket should have roughly samples/10 values
    const expected = samples / 10;
    const tolerance = 0.2; // 20% tolerance

    for (const count of buckets) {
      expect(count).toBeGreaterThan(expected * (1 - tolerance));
      expect(count).toBeLessThan(expected * (1 + tolerance));
    }
  });

  it('should generate bytes with approximately uniform distribution', () => {
    const provider = new MathRandomProvider();
    const memory = new WebAssembly.Memory({ initial: 1 });
    const host = createRandomHost(provider, memory);

    // Generate many random bytes
    host.vudo_random_bytes(0, 10000);

    const memView = new Uint8Array(memory.buffer);
    const buffer = memView.slice(0, 10000);

    // Count byte frequencies
    const freq = new Array(256).fill(0);
    for (const byte of buffer) {
      freq[byte]++;
    }

    // Check that most byte values appear
    const nonZeroFreq = freq.filter((f) => f > 0).length;
    expect(nonZeroFreq).toBeGreaterThan(200); // At least 200 different byte values

    // Average should be around 10000/256 ≈ 39
    const avg = 10000 / 256;
    const counts = freq.filter((f) => f > 0);
    const avgCount = counts.reduce((a, b) => a + b, 0) / counts.length;

    expect(avgCount).toBeGreaterThan(avg * 0.5);
    expect(avgCount).toBeLessThan(avg * 1.5);
  });
});
