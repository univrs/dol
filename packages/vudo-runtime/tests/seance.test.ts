/**
 * Séance (session) tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { Seance, createSeance, withSeance } from '../src/seance.js';
import { Spirit } from '../src/spirit.js';

// Minimal WASM module that exports an 'add' function
const MINIMAL_ADD_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
  0x01, 0x07, 0x01, 0x60, 0x02, 0x7e, 0x7e, 0x01, 0x7e,
  0x03, 0x02, 0x01, 0x00,
  0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00,
  0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x7c, 0x0b,
]);

// Minimal WASM module that exports a 'multiply' function
const MINIMAL_MUL_WASM = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
  0x01, 0x07, 0x01, 0x60, 0x02, 0x7e, 0x7e, 0x01, 0x7e,
  0x03, 0x02, 0x01, 0x00,
  0x07, 0x0c, 0x01, 0x08, 0x6d, 0x75, 0x6c, 0x74, 0x69, 0x70, 0x6c, 0x79, 0x00, 0x00,
  0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x7e, 0x0b,
]);

describe('Seance', () => {
  let seance: Seance;

  beforeEach(() => {
    seance = new Seance();
  });

  it('should summon a Spirit', async () => {
    await seance.summon('calc', MINIMAL_ADD_WASM);

    expect(seance.hasSpirit('calc')).toBe(true);
    expect(seance.spirits()).toContain('calc');
    expect(seance.size).toBe(1);
  });

  it('should invoke Spirit functions', async () => {
    await seance.summon('calc', MINIMAL_ADD_WASM);

    const result = await seance.invoke<bigint>('calc', 'add', [10n, 5n]);
    expect(result).toBe(15n);
  });

  it('should throw when summoning duplicate name', async () => {
    await seance.summon('calc', MINIMAL_ADD_WASM);

    await expect(seance.summon('calc', MINIMAL_ADD_WASM)).rejects.toThrow(
      "Spirit 'calc' is already summoned"
    );
  });

  it('should throw when invoking unknown Spirit', async () => {
    await expect(seance.invoke('unknown', 'add', [])).rejects.toThrow(
      "Spirit 'unknown' not found"
    );
  });

  it('should get Spirit by name', async () => {
    await seance.summon('calc', MINIMAL_ADD_WASM);

    const spirit = seance.getSpirit('calc');
    expect(spirit).toBeInstanceOf(Spirit);
  });

  it('should return undefined for unknown Spirit', () => {
    const spirit = seance.getSpirit('unknown');
    expect(spirit).toBeUndefined();
  });

  it('should release a Spirit', async () => {
    await seance.summon('calc', MINIMAL_ADD_WASM);
    expect(seance.hasSpirit('calc')).toBe(true);

    await seance.release('calc');
    expect(seance.hasSpirit('calc')).toBe(false);
    expect(seance.size).toBe(0);
  });

  it('should throw when releasing unknown Spirit', async () => {
    await expect(seance.release('unknown')).rejects.toThrow(
      "Spirit 'unknown' not found"
    );
  });

  it('should dismiss all Spirits', async () => {
    await seance.summon('add', MINIMAL_ADD_WASM);
    await seance.summon('mul', MINIMAL_MUL_WASM);

    expect(seance.size).toBe(2);

    await seance.dismiss();

    expect(seance.size).toBe(0);
    expect(seance.spirits()).toEqual([]);
  });

  it('should provide Loa registry access', () => {
    expect(seance.loas).toBeDefined();
    expect(seance.loas.has('core')).toBe(true);
  });
});

describe('Multi-Spirit session', () => {
  it('should coordinate multiple Spirits', async () => {
    const seance = new Seance();

    await seance.summon('adder', MINIMAL_ADD_WASM);
    await seance.summon('multiplier', MINIMAL_MUL_WASM);

    const sum = await seance.invoke<bigint>('adder', 'add', [3n, 4n]);
    const product = await seance.invoke<bigint>('multiplier', 'multiply', [sum, 2n]);

    expect(sum).toBe(7n);
    expect(product).toBe(14n);

    await seance.dismiss();
  });
});

describe('createSeance helper', () => {
  it('should create a new Seance', () => {
    const seance = createSeance();
    expect(seance).toBeInstanceOf(Seance);
  });

  it('should accept options', () => {
    const seance = createSeance({ debug: true });
    expect(seance).toBeInstanceOf(Seance);
  });
});

describe('withSeance helper', () => {
  it('should run callback and dismiss', async () => {
    let spiritsCounted = 0;

    await withSeance(async (seance) => {
      await seance.summon('calc', MINIMAL_ADD_WASM);
      spiritsCounted = seance.size;

      const result = await seance.invoke<bigint>('calc', 'add', [1n, 2n]);
      expect(result).toBe(3n);
    });

    expect(spiritsCounted).toBe(1);
    // Session is dismissed, but we can't verify from outside
  });

  it('should dismiss on error', async () => {
    await expect(
      withSeance(async (seance) => {
        await seance.summon('calc', MINIMAL_ADD_WASM);
        throw new Error('Test error');
      })
    ).rejects.toThrow('Test error');
  });

  it('should return callback result', async () => {
    const result = await withSeance(async (seance) => {
      await seance.summon('calc', MINIMAL_ADD_WASM);
      return seance.invoke<bigint>('calc', 'add', [20n, 22n]);
    });

    expect(result).toBe(42n);
  });
});
