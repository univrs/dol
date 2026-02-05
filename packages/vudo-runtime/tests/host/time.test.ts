/**
 * Tests for time host functions
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { TimeHostFunctions } from '../../src/host/time.js';
import { MockTimeProvider } from './test-utils.js';

describe('TimeHostFunctions', () => {
  let timeProvider: MockTimeProvider;
  let timeOps: TimeHostFunctions;

  beforeEach(() => {
    timeProvider = new MockTimeProvider();
    timeOps = new TimeHostFunctions({ timeProvider });
  });

  describe('vudo_now', () => {
    it('should return current timestamp in milliseconds', () => {
      const now = timeOps.now();

      expect(typeof now).toBe('bigint');
      expect(now).toBeGreaterThan(0n);
    });

    it('should return consistent values from time provider', () => {
      timeProvider.setNow(1234567890000n);

      const now = timeOps.now();

      expect(now).toBe(1234567890000n);
    });

    it('should reflect time advancement', () => {
      const start = timeOps.now();

      timeProvider.advance(5000); // Advance 5 seconds

      const end = timeOps.now();

      expect(end - start).toBe(5000n);
    });

    it('should handle very large timestamps', () => {
      const largeTimestamp = 9999999999999n;
      timeProvider.setNow(largeTimestamp);

      const now = timeOps.now();

      expect(now).toBe(largeTimestamp);
    });

    it('should be monotonically increasing in real time', () => {
      const t1 = timeOps.now();
      timeProvider.advance(1);
      const t2 = timeOps.now();
      timeProvider.advance(1);
      const t3 = timeOps.now();

      expect(t2).toBeGreaterThan(t1);
      expect(t3).toBeGreaterThan(t2);
    });
  });

  describe('vudo_sleep', () => {
    it('should accept valid sleep duration', () => {
      timeOps.sleep(1000);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps).toHaveLength(1);
      expect(sleeps[0]).toBe(1000);
    });

    it('should accept zero sleep duration', () => {
      timeOps.sleep(0);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps).toHaveLength(1);
      expect(sleeps[0]).toBe(0);
    });

    it('should handle very short durations', () => {
      timeOps.sleep(1);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps[0]).toBe(1);
    });

    it('should handle long durations within limit', () => {
      const oneHour = 60 * 60 * 1000;
      timeOps.sleep(oneHour);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps[0]).toBe(oneHour);
    });

    it('should treat negative durations as zero', () => {
      timeOps.sleep(-100);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps).toHaveLength(0); // Treated as invalid, no sleep recorded
    });

    it('should reject durations exceeding maximum', () => {
      const twoHours = 2 * 60 * 60 * 1000;
      timeOps.sleep(twoHours);

      // Should not sleep (exceeds 1 hour default max)
      const sleeps = timeProvider.getSleeps();
      expect(sleeps).toHaveLength(0);
    });

    it('should track multiple sleep calls', () => {
      timeOps.sleep(100);
      timeOps.sleep(200);
      timeOps.sleep(300);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps).toEqual([100, 200, 300]);
    });

    it('should update statistics', () => {
      timeOps.sleep(100);
      timeOps.sleep(200);

      const stats = timeOps.getStats();
      expect(stats.sleepCount).toBe(2);
      expect(stats.totalSleepTime).toBe(300);
      expect(stats.averageSleepTime).toBe(150);
    });

    it('should calculate average sleep time', () => {
      timeOps.sleep(100);
      timeOps.sleep(200);
      timeOps.sleep(300);

      const stats = timeOps.getStats();
      expect(stats.averageSleepTime).toBe(200);
    });

    it('should handle zero average when no sleeps', () => {
      const stats = timeOps.getStats();
      expect(stats.averageSleepTime).toBe(0);
    });
  });

  describe('vudo_monotonic_now', () => {
    it('should return monotonic time in nanoseconds', () => {
      const time = timeOps.monotonicNow();

      expect(typeof time).toBe('bigint');
      expect(time).toBeGreaterThanOrEqual(0n);
    });

    it('should return consistent values from time provider', () => {
      timeProvider.setMonotonicNow(123456789000000n);

      const time = timeOps.monotonicNow();

      expect(time).toBe(123456789000000n);
    });

    it('should be monotonically increasing', () => {
      const t1 = timeOps.monotonicNow();
      timeProvider.advance(1); // Advance 1ms = 1,000,000 ns
      const t2 = timeOps.monotonicNow();
      timeProvider.advance(1);
      const t3 = timeOps.monotonicNow();

      expect(t2).toBeGreaterThan(t1);
      expect(t3).toBeGreaterThan(t2);
      expect(t2 - t1).toBe(1_000_000n); // 1ms in nanoseconds
      expect(t3 - t2).toBe(1_000_000n);
    });

    it('should have nanosecond precision', () => {
      timeProvider.setMonotonicNow(0n);

      const start = timeOps.monotonicNow();
      timeProvider.setMonotonicNow(1n); // 1 nanosecond
      const end = timeOps.monotonicNow();

      expect(end - start).toBe(1n);
    });

    it('should handle very large values', () => {
      const largeTime = 999999999999999999n;
      timeProvider.setMonotonicNow(largeTime);

      const time = timeOps.monotonicNow();

      expect(time).toBe(largeTime);
    });

    it('should measure elapsed time accurately', () => {
      const start = timeOps.monotonicNow();

      timeProvider.advance(100); // 100ms

      const end = timeOps.monotonicNow();
      const elapsedMs = (end - start) / 1_000_000n;

      expect(elapsedMs).toBe(100n);
    });
  });

  describe('statistics', () => {
    it('should track sleep count', () => {
      timeOps.sleep(100);
      timeOps.sleep(200);
      timeOps.sleep(300);

      expect(timeOps.getStats().sleepCount).toBe(3);
    });

    it('should track total sleep time', () => {
      timeOps.sleep(100);
      timeOps.sleep(200);
      timeOps.sleep(300);

      expect(timeOps.getStats().totalSleepTime).toBe(600);
    });

    it('should reset statistics', () => {
      timeOps.sleep(100);
      timeOps.sleep(200);

      timeOps.resetStats();

      const stats = timeOps.getStats();
      expect(stats.sleepCount).toBe(0);
      expect(stats.totalSleepTime).toBe(0);
      expect(stats.averageSleepTime).toBe(0);
    });

    it('should maintain accurate stats after resets', () => {
      timeOps.sleep(100);
      timeOps.resetStats();
      timeOps.sleep(200);

      const stats = timeOps.getStats();
      expect(stats.sleepCount).toBe(1);
      expect(stats.totalSleepTime).toBe(200);
    });
  });

  describe('buildImports', () => {
    it('should return all time functions', () => {
      const imports = timeOps.buildImports();

      expect(imports).toHaveProperty('vudo_now');
      expect(imports).toHaveProperty('vudo_sleep');
      expect(imports).toHaveProperty('vudo_monotonic_now');
    });

    it('should bind functions correctly', () => {
      const imports = timeOps.buildImports();

      const now = (imports.vudo_now as Function)();
      expect(typeof now).toBe('bigint');
    });
  });

  describe('debug mode', () => {
    it('should log time calls in debug mode', () => {
      const debugOps = new TimeHostFunctions({
        timeProvider,
        debug: true,
      });

      // Capture console output
      const logs: string[] = [];
      const originalDebug = console.debug;
      console.debug = (msg: string) => logs.push(msg);

      debugOps.now();
      debugOps.sleep(100);
      debugOps.monotonicNow();

      console.debug = originalDebug;

      expect(logs.some((log) => log.includes('now()'))).toBe(true);
      expect(logs.some((log) => log.includes('sleep(100ms)'))).toBe(true);
      expect(logs.some((log) => log.includes('monotonic_now()'))).toBe(true);
    });
  });

  describe('custom configuration', () => {
    it('should respect custom max sleep duration', () => {
      const customOps = new TimeHostFunctions({
        timeProvider,
        maxSleepDuration: 1000,
      });

      customOps.sleep(1001);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps).toHaveLength(0); // Rejected
    });

    it('should allow sleep up to max duration', () => {
      const customOps = new TimeHostFunctions({
        timeProvider,
        maxSleepDuration: 1000,
      });

      customOps.sleep(1000);

      const sleeps = timeProvider.getSleeps();
      expect(sleeps[0]).toBe(1000);
    });
  });

  describe('edge cases', () => {
    it('should handle rapid time queries', () => {
      for (let i = 0; i < 100; i++) {
        const now = timeOps.now();
        expect(now).toBeGreaterThanOrEqual(0n);
      }
    });

    it('should handle rapid monotonic queries', () => {
      const times: bigint[] = [];
      for (let i = 0; i < 100; i++) {
        times.push(timeOps.monotonicNow());
        timeProvider.advance(1);
      }

      // Verify monotonically increasing
      for (let i = 1; i < times.length; i++) {
        expect(times[i]).toBeGreaterThan(times[i - 1]);
      }
    });

    it('should handle sleep(0) correctly', () => {
      timeOps.sleep(0);

      const stats = timeOps.getStats();
      expect(stats.sleepCount).toBe(1);
      expect(stats.totalSleepTime).toBe(0);
    });

    it('should handle time provider errors gracefully', () => {
      // Create a time provider that throws
      const errorProvider: MockTimeProvider = {
        now: () => {
          throw new Error('Time error');
        },
        monotonicNow: () => {
          throw new Error('Monotonic error');
        },
        sleep: async () => {},
        advance: () => {},
        setNow: () => {},
        setMonotonicNow: () => {},
        getSleeps: () => [],
        reset: () => {},
      };

      const errorOps = new TimeHostFunctions({
        timeProvider: errorProvider,
      });

      // Should not throw, should return fallback value
      const now = errorOps.now();
      expect(now).toBeGreaterThan(0n);

      const monotonic = errorOps.monotonicNow();
      expect(monotonic).toBeGreaterThanOrEqual(0n);
    });
  });

  describe('time precision', () => {
    it('should support millisecond precision for now()', () => {
      timeProvider.setNow(1234567890123n);

      const time = timeOps.now();

      expect(time).toBe(1234567890123n);
    });

    it('should support nanosecond precision for monotonic_now()', () => {
      timeProvider.setMonotonicNow(1234567890123456789n);

      const time = timeOps.monotonicNow();

      expect(time).toBe(1234567890123456789n);
    });

    it('should convert between milliseconds and nanoseconds correctly', () => {
      const ms = 1000n;
      const ns = ms * 1_000_000n;

      timeProvider.setMonotonicNow(ns);

      const time = timeOps.monotonicNow();
      const backToMs = time / 1_000_000n;

      expect(backToMs).toBe(ms);
    });
  });
});
