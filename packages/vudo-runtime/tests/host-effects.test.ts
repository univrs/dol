/**
 * Effects system tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  EffectsSystem,
  createEffectsSystem,
  type IEffectHandler,
  DefaultEffectHandler,
} from '../src/host/effects.js';
import { ResultCode, type StandardEffect } from '../src/abi/types.js';

describe('EffectsSystem', () => {
  let memory: WebAssembly.Memory;
  let effects: EffectsSystem;

  beforeEach(() => {
    memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
    effects = createEffectsSystem(memory);
  });

  describe('Effect Emission', () => {
    it('should emit a valid effect', () => {
      // Create effect JSON
      const effect: StandardEffect = {
        effect_type: 'log',
        payload: { message: 'test log' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);

      // Encode into memory
      const ptr = encodeStringToMemory(memory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      // Emit effect
      const result = effects.emitEffect(ptr, len);

      expect(result).toBe(ResultCode.Success);
    });

    it('should add emitted effects to the queue', () => {
      const effect: StandardEffect = {
        effect_type: 'state_change',
        payload: { value: 42 },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const ptr = encodeStringToMemory(memory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(ptr, len);

      const queue = effects.getEffectQueue();
      expect(queue).toHaveLength(1);
      expect(queue[0].effect_type).toBe('state_change');
      expect(queue[0].payload).toEqual({ value: 42 });
    });

    it('should return error for invalid JSON', () => {
      const invalidJson = '{ invalid json }';
      const ptr = encodeStringToMemory(memory, invalidJson);
      const len = new TextEncoder().encode(invalidJson).length;

      const result = effects.emitEffect(ptr, len);

      expect(result).toBe(ResultCode.Error);
    });

    it('should return error for missing effect_type', () => {
      const invalidEffect = {
        payload: { data: 'test' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(invalidEffect);
      const ptr = encodeStringToMemory(memory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      const result = effects.emitEffect(ptr, len);

      expect(result).toBe(ResultCode.Error);
    });

    it('should set timestamp if not provided', () => {
      const effect = {
        effect_type: 'test',
        payload: null,
      };
      const effectJson = JSON.stringify(effect);
      const ptr = encodeStringToMemory(memory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(ptr, len);

      const queue = effects.getEffectQueue();
      expect(queue[0].timestamp).toBeGreaterThan(0);
    });
  });

  describe('Subscriptions', () => {
    it('should subscribe to exact pattern', () => {
      const pattern = 'log';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback = vi.fn();
      const subId = effects.subscribe(ptr, len, callback);

      expect(subId).toBeGreaterThan(0);
    });

    it('should subscribe to wildcard pattern', () => {
      const pattern = '*';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback = vi.fn();
      const subId = effects.subscribe(ptr, len, callback);

      expect(subId).toBeGreaterThan(0);
    });

    it('should notify exact pattern subscribers', () => {
      const pattern = 'log';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback = vi.fn();
      effects.subscribe(ptr, len, callback);

      // Emit matching effect
      const effect: StandardEffect = {
        effect_type: 'log',
        payload: { message: 'test' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const effectPtr = encodeStringToMemory(memory, effectJson);
      const effectLen = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(effectPtr, effectLen);

      expect(callback).toHaveBeenCalledTimes(1);
      expect(callback).toHaveBeenCalledWith(
        expect.objectContaining({
          effect_type: 'log',
          payload: { message: 'test' },
        })
      );
    });

    it('should not notify non-matching subscribers', () => {
      const pattern = 'log';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback = vi.fn();
      effects.subscribe(ptr, len, callback);

      // Emit non-matching effect
      const effect: StandardEffect = {
        effect_type: 'error',
        payload: { message: 'test' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const effectPtr = encodeStringToMemory(memory, effectJson);
      const effectLen = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(effectPtr, effectLen);

      expect(callback).not.toHaveBeenCalled();
    });

    it('should notify wildcard subscribers for all effects', () => {
      const pattern = '*';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback = vi.fn();
      effects.subscribe(ptr, len, callback);

      // Emit multiple effects
      const effects1: StandardEffect = {
        effect_type: 'log',
        payload: { message: 'log' },
        timestamp: Date.now(),
      };
      const effects2: StandardEffect = {
        effect_type: 'error',
        payload: { message: 'error' },
        timestamp: Date.now(),
      };

      for (const effect of [effects1, effects2]) {
        const effectJson = JSON.stringify(effect);
        const effectPtr = encodeStringToMemory(memory, effectJson);
        const effectLen = new TextEncoder().encode(effectJson).length;
        effects.emitEffect(effectPtr, effectLen);
      }

      expect(callback).toHaveBeenCalledTimes(2);
    });

    it('should support multiple subscribers to the same pattern', () => {
      const pattern = 'log';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback1 = vi.fn();
      const callback2 = vi.fn();

      effects.subscribe(ptr, len, callback1);
      effects.subscribe(ptr, len, callback2);

      // Emit effect
      const effect: StandardEffect = {
        effect_type: 'log',
        payload: { message: 'test' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const effectPtr = encodeStringToMemory(memory, effectJson);
      const effectLen = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(effectPtr, effectLen);

      expect(callback1).toHaveBeenCalledTimes(1);
      expect(callback2).toHaveBeenCalledTimes(1);
    });

    it('should handle subscriber callback errors gracefully', () => {
      const pattern = 'log';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const throwingCallback = vi.fn(() => {
        throw new Error('Callback error');
      });

      effects.subscribe(ptr, len, throwingCallback);

      // Emit effect - should not throw
      const effect: StandardEffect = {
        effect_type: 'log',
        payload: { message: 'test' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const effectPtr = encodeStringToMemory(memory, effectJson);
      const effectLen = new TextEncoder().encode(effectJson).length;

      expect(() => effects.emitEffect(effectPtr, effectLen)).not.toThrow();
      expect(throwingCallback).toHaveBeenCalled();
    });

    it('should unsubscribe successfully', () => {
      const pattern = 'log';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback = vi.fn();
      const subId = effects.subscribe(ptr, len, callback);

      const removed = effects.unsubscribe(subId);
      expect(removed).toBe(true);

      // Emit effect - callback should not be called
      const effect: StandardEffect = {
        effect_type: 'log',
        payload: { message: 'test' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const effectPtr = encodeStringToMemory(memory, effectJson);
      const effectLen = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(effectPtr, effectLen);

      expect(callback).not.toHaveBeenCalled();
    });

    it('should return false when unsubscribing invalid ID', () => {
      const removed = effects.unsubscribe(999);
      expect(removed).toBe(false);
    });
  });

  describe('Queue Management', () => {
    it('should clear effect queue', () => {
      const effect: StandardEffect = {
        effect_type: 'test',
        payload: null,
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const ptr = encodeStringToMemory(memory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(ptr, len);
      expect(effects.getEffectQueue()).toHaveLength(1);

      effects.clearEffectQueue();
      expect(effects.getEffectQueue()).toHaveLength(0);
    });

    it('should return immutable queue copy', () => {
      const effect: StandardEffect = {
        effect_type: 'test',
        payload: null,
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const ptr = encodeStringToMemory(memory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      effects.emitEffect(ptr, len);

      const queue1 = effects.getEffectQueue();
      const queue2 = effects.getEffectQueue();

      expect(queue1).not.toBe(queue2); // Different array instances
      expect(queue1).toEqual(queue2); // Same contents
    });
  });

  describe('Subscription Management', () => {
    it('should list all active subscriptions', () => {
      const pattern1 = 'log';
      const pattern2 = 'error';

      const ptr1 = 2048; // First offset
      const bytes1 = new TextEncoder().encode(pattern1);
      new Uint8Array(memory.buffer, ptr1).set(bytes1);
      const len1 = bytes1.length;

      const ptr2 = 2048 + 100; // Second offset (offset to avoid overlap)
      const bytes2 = new TextEncoder().encode(pattern2);
      new Uint8Array(memory.buffer, ptr2).set(bytes2);
      const len2 = bytes2.length;

      const callback = vi.fn();
      const id1 = effects.subscribe(ptr1, len1, callback);
      const id2 = effects.subscribe(ptr2, len2, callback);

      const subs = effects.getSubscriptions();
      expect(subs).toHaveLength(2);
      expect(subs).toEqual(
        expect.arrayContaining([
          { id: id1, pattern: 'log' },
          { id: id2, pattern: 'error' },
        ])
      );
    });
  });

  describe('Custom Handler', () => {
    it('should use custom effect handler', () => {
      const handler: IEffectHandler = {
        handleEffect: vi.fn(() => ResultCode.Success),
        onSubscribe: vi.fn(),
      };

      const customEffects = createEffectsSystem(memory, handler);

      const effect: StandardEffect = {
        effect_type: 'test',
        payload: { data: 'test' },
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const ptr = encodeStringToMemory(memory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      customEffects.emitEffect(ptr, len);

      expect(handler.handleEffect).toHaveBeenCalledWith(
        expect.objectContaining({
          effect_type: 'test',
          payload: { data: 'test' },
        })
      );
    });

    it('should notify handler on subscription', () => {
      const handler: IEffectHandler = {
        handleEffect: vi.fn(() => ResultCode.Success),
        onSubscribe: vi.fn(),
      };

      const customEffects = createEffectsSystem(memory, handler);

      const pattern = 'log';
      const ptr = encodeStringToMemory(memory, pattern);
      const len = new TextEncoder().encode(pattern).length;

      const callback = vi.fn();
      const subId = customEffects.subscribe(ptr, len, callback);

      expect(handler.onSubscribe).toHaveBeenCalledWith('log', subId);
    });
  });

  describe('Host Functions', () => {
    it('should create host functions for WASM', () => {
      const hostFunctions = effects.createHostFunctions();

      expect(hostFunctions).toHaveProperty('vudo_emit_effect');
      expect(hostFunctions).toHaveProperty('vudo_subscribe');
      expect(typeof hostFunctions.vudo_emit_effect).toBe('function');
      expect(typeof hostFunctions.vudo_subscribe).toBe('function');
    });
  });

  describe('Memory Updates', () => {
    it('should update memory reference', () => {
      const newMemory = new WebAssembly.Memory({ initial: 2 });
      effects.updateMemory(newMemory);

      // Should work with new memory
      const effect: StandardEffect = {
        effect_type: 'test',
        payload: null,
        timestamp: Date.now(),
      };
      const effectJson = JSON.stringify(effect);
      const ptr = encodeStringToMemory(newMemory, effectJson);
      const len = new TextEncoder().encode(effectJson).length;

      const result = effects.emitEffect(ptr, len);
      expect(result).toBe(ResultCode.Success);
    });
  });
});

describe('DefaultEffectHandler', () => {
  it('should log effects to console', () => {
    const handler = new DefaultEffectHandler();
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const effect: StandardEffect = {
      effect_type: 'test',
      payload: { data: 'test' },
      timestamp: Date.now(),
    };

    const result = handler.handleEffect(effect);

    expect(result).toBe(ResultCode.Success);
    expect(consoleSpy).toHaveBeenCalled();

    consoleSpy.mockRestore();
  });

  it('should log subscription notifications', () => {
    const handler = new DefaultEffectHandler();
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    handler.onSubscribe('log', 1);

    expect(consoleSpy).toHaveBeenCalled();

    consoleSpy.mockRestore();
  });
});

// Helper function to encode strings into WASM memory
function encodeStringToMemory(memory: WebAssembly.Memory, str: string): number {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(str);
  const view = new Uint8Array(memory.buffer);

  // Write at offset 2048 (safe zone)
  const ptr = 2048;
  view.set(bytes, ptr);

  return ptr;
}
