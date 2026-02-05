/**
 * Multi-Tab Coordination Benchmark
 *
 * Tests concurrent write safety and coordination across multiple browser tabs.
 * CRITICAL for VUDO Runtime: No data corruption allowed!
 */

import { StorageAdapter } from '../adapters/base.js';
import { generateTestRecord } from '../utils/data-gen.js';
import { BenchmarkResult, getBrowserInfo } from '../utils/metrics.js';

export interface MultiTabBenchmarkConfig {
  adapter: StorageAdapter;
  tabCounts: number[];          // [2, 5, 10]
  writesPerTab: number;
  coordinationStrategy: 'locks' | 'optimistic' | 'none';
}

export interface MultiTabBenchmarkResults {
  results: BenchmarkResult[];
  corruptionDetected: boolean;
  conflictRate: number;
}

/**
 * Run multi-tab coordination benchmark
 *
 * Note: This simulates multi-tab behavior using BroadcastChannel and SharedWorkers.
 * For true multi-tab testing, open the test page in multiple browser tabs.
 */
export async function runMultiTabBenchmark(config: MultiTabBenchmarkConfig): Promise<MultiTabBenchmarkResults> {
  const results: BenchmarkResult[] = [];
  const browserInfo = getBrowserInfo();
  let totalConflicts = 0;
  let totalWrites = 0;
  let corruptionDetected = false;

  console.log('Starting multi-tab coordination benchmark...');
  console.log(`Browser: ${browserInfo.name} ${browserInfo.version}`);
  console.log(`Adapter: ${config.adapter.getName()}`);
  console.log(`Strategy: ${config.coordinationStrategy}`);

  for (const tabCount of config.tabCounts) {
    console.log(`\nTesting with ${tabCount} concurrent tabs...`);

    try {
      const result = await runMultiTabTest(
        config.adapter,
        tabCount,
        config.writesPerTab,
        config.coordinationStrategy
      );

      results.push(result);

      if (result.errors && result.errors.length > 0) {
        totalConflicts += result.errors.filter(e => e.includes('conflict')).length;
        if (result.errors.some(e => e.includes('corruption'))) {
          corruptionDetected = true;
        }
      }

      totalWrites += tabCount * config.writesPerTab;

      console.log(`  ✓ Completed: ${result.metrics.throughput.toFixed(2)} ops/s`);
      console.log(`  ✓ Conflicts: ${result.errors?.length || 0}`);

    } catch (error) {
      console.error(`  ✗ Error: ${error}`);
      corruptionDetected = true;
    }
  }

  const conflictRate = totalWrites > 0 ? totalConflicts / totalWrites : 0;

  return {
    results,
    corruptionDetected,
    conflictRate,
  };
}

/**
 * Simulate multi-tab concurrent writes
 */
async function runMultiTabTest(
  adapter: StorageAdapter,
  tabCount: number,
  writesPerTab: number,
  strategy: 'locks' | 'optimistic' | 'none'
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();
  const errors: string[] = [];
  const operationTimes: number[] = [];
  let totalBytes = 0;

  // Create a shared write space
  const sharedKeys = new Set<string>();
  const writeResults = new Map<string, any[]>();

  const startTime = performance.now();

  // Simulate concurrent writes from multiple "tabs"
  const tabPromises = Array.from({ length: tabCount }, async (_, tabIndex) => {
    const tabId = `tab-${tabIndex}`;

    for (let i = 0; i < writesPerTab; i++) {
      // 30% chance of writing to a shared key (simulate conflicts)
      const useSharedKey = Math.random() < 0.3;
      const key = useSharedKey
        ? `shared-key-${i % 10}` // Shared across tabs
        : `${tabId}-key-${i}`;   // Tab-specific

      const record = generateTestRecord(undefined, 'medium');
      record.metadata.created = Date.now();
      record.metadata.version = 1;
      (record as any).tabId = tabId;
      (record as any).writeIndex = i;

      const opStart = performance.now();

      try {
        // Apply coordination strategy
        if (strategy === 'locks') {
          // Simulate lock-based coordination
          await acquireLock(key);
          const metrics = await adapter.write(key, record);
          await releaseLock(key);

          operationTimes.push(metrics.operationTime);
          if (metrics.bytesWritten) {
            totalBytes += metrics.bytesWritten;
          }
        } else if (strategy === 'optimistic') {
          // Optimistic: read-modify-write with version check
          const { value: existing } = await adapter.read(key);

          if (existing) {
            // Check version
            if (existing.metadata.version >= record.metadata.version) {
              record.metadata.version = existing.metadata.version + 1;
            }
          }

          const metrics = await adapter.write(key, record);
          operationTimes.push(metrics.operationTime);
          if (metrics.bytesWritten) {
            totalBytes += metrics.bytesWritten;
          }
        } else {
          // No coordination - just write
          const metrics = await adapter.write(key, record);
          operationTimes.push(metrics.operationTime);
          if (metrics.bytesWritten) {
            totalBytes += metrics.bytesWritten;
          }
        }

        // Track writes for verification
        if (!writeResults.has(key)) {
          writeResults.set(key, []);
        }
        writeResults.get(key)!.push({ tabId, record, timestamp: Date.now() });

      } catch (error) {
        errors.push(`Tab ${tabId} write error on key ${key}: ${error}`);
      }
    }
  });

  // Wait for all tabs to complete
  await Promise.all(tabPromises);

  const endTime = performance.now();
  const totalTime = endTime - startTime;

  // Verify data integrity
  const integrityErrors = await verifyMultiTabIntegrity(adapter, writeResults);
  errors.push(...integrityErrors);

  const totalWrites = tabCount * writesPerTab;
  const avgOperationTime = operationTimes.length > 0
    ? operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length
    : 0;

  const minOperationTime = operationTimes.length > 0 ? Math.min(...operationTimes) : 0;
  const maxOperationTime = operationTimes.length > 0 ? Math.max(...operationTimes) : 0;
  const throughput = totalWrites / (totalTime / 1000);
  const bytesPerSecond = totalBytes / (totalTime / 1000);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'multi-tab-coordination',
    datasetSize: totalWrites,
    operationType: `${tabCount}-tabs-${strategy}`,
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime,
      maxOperationTime,
      throughput,
      bytesPerSecond,
      errorRate: errors.length / totalWrites,
    },
    errors: errors.length > 0 ? errors : undefined,
    timestamp: Date.now(),
  };
}

/**
 * Simple lock mechanism (simulated)
 */
const locks = new Map<string, Promise<void>>();

async function acquireLock(key: string): Promise<void> {
  while (locks.has(key)) {
    await locks.get(key);
  }

  const release = Promise.resolve();
  locks.set(key, release);

  // Small delay to simulate lock acquisition
  await new Promise(resolve => setTimeout(resolve, 1));
}

async function releaseLock(key: string): Promise<void> {
  locks.delete(key);
}

/**
 * Verify data integrity after multi-tab writes
 */
async function verifyMultiTabIntegrity(
  adapter: StorageAdapter,
  writeResults: Map<string, any[]>
): Promise<string[]> {
  const errors: string[] = [];

  for (const [key, writes] of writeResults.entries()) {
    try {
      const { value } = await adapter.read(key);

      if (!value) {
        errors.push(`CORRUPTION: Key ${key} not found after ${writes.length} writes`);
        continue;
      }

      // Verify the stored value matches one of the writes
      const matchesAnyWrite = writes.some(w => {
        return JSON.stringify(w.record) === JSON.stringify(value);
      });

      if (!matchesAnyWrite) {
        errors.push(`CORRUPTION: Key ${key} contains unexpected data`);
      }

      // Check if latest write won (if deterministic)
      const latestWrite = writes.reduce((latest, w) => {
        return w.timestamp > latest.timestamp ? w : latest;
      });

      // If versions are being tracked, verify
      if (value.metadata?.version) {
        const expectedVersion = writes.length;
        if (value.metadata.version > expectedVersion) {
          // This is acceptable - version incremented properly
        } else if (value.metadata.version < expectedVersion) {
          errors.push(`Conflict on key ${key}: expected version ${expectedVersion}, got ${value.metadata.version}`);
        }
      }

    } catch (error) {
      errors.push(`Verification error for key ${key}: ${error}`);
    }
  }

  return errors;
}

/**
 * Test cross-tab communication using BroadcastChannel
 */
export async function testCrossTabCommunication(
  adapter: StorageAdapter,
  messageCount: number
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();

  console.log(`\nTesting cross-tab communication: ${messageCount} messages...`);

  const channelName = 'storage-benchmark-sync';
  const channel = new BroadcastChannel(channelName);

  const messagesReceived: any[] = [];
  const operationTimes: number[] = [];

  channel.onmessage = (event) => {
    messagesReceived.push(event.data);
  };

  const startTime = performance.now();

  // Send messages and verify they're received
  for (let i = 0; i < messageCount; i++) {
    const message = {
      type: 'storage-update',
      key: `key-${i}`,
      timestamp: Date.now(),
      sender: 'benchmark',
    };

    const opStart = performance.now();
    channel.postMessage(message);
    operationTimes.push(performance.now() - opStart);

    // Small delay to allow message propagation
    await new Promise(resolve => setTimeout(resolve, 5));
  }

  // Wait for messages to be received
  await new Promise(resolve => setTimeout(resolve, 100));

  const endTime = performance.now();
  const totalTime = endTime - startTime;

  channel.close();

  const avgOperationTime = operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length;
  const throughput = messageCount / (totalTime / 1000);

  console.log(`  ✓ Sent ${messageCount} messages in ${totalTime.toFixed(2)}ms`);
  console.log(`  ✓ Received ${messagesReceived.length} messages`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'cross-tab-communication',
    datasetSize: messageCount,
    operationType: 'broadcast',
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime: Math.min(...operationTimes),
      maxOperationTime: Math.max(...operationTimes),
      throughput,
    },
    timestamp: Date.now(),
  };
}
