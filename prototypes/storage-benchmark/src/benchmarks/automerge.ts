/**
 * Automerge Document Lifecycle Benchmark
 *
 * Tests save/load performance for Automerge CRDT documents.
 * This is critical for VUDO Runtime local-first mode.
 */

import { StorageAdapter } from '../adapters/base.js';
import { generateAutomergeData, calculateDataSize } from '../utils/data-gen.js';
import { BenchmarkResult, getBrowserInfo, getMemoryUsage } from '../utils/metrics.js';

export interface AutomergeBenchmarkConfig {
  adapter: StorageAdapter;
  documentSizes: number[];      // Sizes in bytes: [100KB, 1MB, 10MB, 100MB]
  operations: ('save' | 'load' | 'sync' | 'merge')[];
  iterations?: number;          // Iterations per test (default: 3)
}

export interface AutomergeBenchmarkResults {
  results: BenchmarkResult[];
  summary: {
    totalDocumentsProcessed: number;
    totalDataProcessedMB: number;
    avgThroughputMBps: number;
  };
}

/**
 * Run Automerge document lifecycle benchmark
 */
export async function runAutomergeBenchmark(config: AutomergeBenchmarkConfig): Promise<AutomergeBenchmarkResults> {
  const results: BenchmarkResult[] = [];
  const browserInfo = getBrowserInfo();
  let totalDocuments = 0;
  let totalDataBytes = 0;

  console.log('Starting Automerge benchmark...');
  console.log(`Browser: ${browserInfo.name} ${browserInfo.version}`);
  console.log(`Adapter: ${config.adapter.getName()}`);

  for (const size of config.documentSizes) {
    console.log(`\nTesting with ${(size / 1024 / 1024).toFixed(2)}MB documents...`);

    for (const operation of config.operations) {
      try {
        const result = await runAutomergeOperation(
          config.adapter,
          size,
          operation,
          config.iterations || 3
        );

        results.push(result);
        totalDocuments += config.iterations || 3;
        totalDataBytes += size * (config.iterations || 3);

        console.log(`  ✓ ${operation}: ${result.metrics.avgOperationTime.toFixed(2)}ms avg`);
        console.log(`  ✓ Throughput: ${(result.metrics.bytesPerSecond! / 1024 / 1024).toFixed(2)} MB/s`);

      } catch (error) {
        console.error(`  ✗ Error in ${operation}: ${error}`);
      }
    }
  }

  const totalDataMB = totalDataBytes / 1024 / 1024;
  const avgThroughputMBps = results.reduce((sum, r) => {
    return sum + ((r.metrics.bytesPerSecond || 0) / 1024 / 1024);
  }, 0) / results.length;

  return {
    results,
    summary: {
      totalDocumentsProcessed: totalDocuments,
      totalDataProcessedMB: totalDataMB,
      avgThroughputMBps,
    },
  };
}

/**
 * Run a specific Automerge operation
 */
async function runAutomergeOperation(
  adapter: StorageAdapter,
  documentSize: number,
  operation: 'save' | 'load' | 'sync' | 'merge',
  iterations: number
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();
  const operationTimes: number[] = [];
  const errors: string[] = [];
  let totalBytes = 0;

  switch (operation) {
    case 'save':
      // Test saving Automerge documents
      for (let i = 0; i < iterations; i++) {
        const doc = generateAutomergeData(documentSize);
        const docKey = `automerge-doc-${i}`;

        const startTime = performance.now();
        try {
          const metrics = await adapter.write(docKey, doc);
          operationTimes.push(metrics.operationTime);
          if (metrics.bytesWritten) {
            totalBytes += metrics.bytesWritten;
          }
        } catch (error) {
          errors.push(`Save error for ${docKey}: ${error}`);
        }
      }
      break;

    case 'load':
      // First save documents, then test loading
      const docs: any[] = [];
      for (let i = 0; i < iterations; i++) {
        const doc = generateAutomergeData(documentSize);
        const docKey = `automerge-doc-${i}`;
        await adapter.write(docKey, doc);
        docs.push({ key: docKey, doc });
      }

      // Now load them
      for (let i = 0; i < iterations; i++) {
        const docKey = `automerge-doc-${i}`;

        const startTime = performance.now();
        try {
          const { value, metrics } = await adapter.read(docKey);
          operationTimes.push(metrics.operationTime);
          if (metrics.bytesRead) {
            totalBytes += metrics.bytesRead;
          }

          if (!value) {
            errors.push(`Document not found: ${docKey}`);
          }
        } catch (error) {
          errors.push(`Load error for ${docKey}: ${error}`);
        }
      }
      break;

    case 'sync':
      // Test sync operation (save + load + update + save)
      for (let i = 0; i < iterations; i++) {
        const doc = generateAutomergeData(documentSize);
        const docKey = `automerge-sync-${i}`;

        const startTime = performance.now();
        try {
          // Save
          await adapter.write(docKey, doc);

          // Load
          const { value: loadedDoc } = await adapter.read(docKey);

          // Modify
          if (loadedDoc) {
            loadedDoc.metadata.version++;
            loadedDoc.timestamp = Date.now();

            // Save again
            const metrics = await adapter.write(docKey, loadedDoc);
            operationTimes.push(performance.now() - startTime);
            if (metrics.bytesWritten) {
              totalBytes += metrics.bytesWritten * 2; // Initial + update
            }
          }
        } catch (error) {
          errors.push(`Sync error for ${docKey}: ${error}`);
        }
      }
      break;

    case 'merge':
      // Test merge operation (simulate CRDT merge)
      for (let i = 0; i < iterations; i++) {
        const doc1 = generateAutomergeData(documentSize / 2);
        const doc2 = generateAutomergeData(documentSize / 2);
        const mergeKey = `automerge-merge-${i}`;

        const startTime = performance.now();
        try {
          // Simulate merge by combining documents
          const merged = {
            ...doc1,
            items: [...doc1.items, ...doc2.items],
            metadata: {
              ...doc1.metadata,
              mergedFrom: [doc1.id, doc2.id],
              mergedAt: Date.now(),
            },
          };

          const metrics = await adapter.write(mergeKey, merged);
          operationTimes.push(performance.now() - startTime);
          if (metrics.bytesWritten) {
            totalBytes += metrics.bytesWritten;
          }
        } catch (error) {
          errors.push(`Merge error for ${mergeKey}: ${error}`);
        }
      }
      break;
  }

  const totalTime = operationTimes.reduce((sum, t) => sum + t, 0);
  const avgOperationTime = totalTime / iterations;
  const minOperationTime = Math.min(...operationTimes);
  const maxOperationTime = Math.max(...operationTimes);
  const throughput = iterations / (totalTime / 1000);
  const bytesPerSecond = totalBytes / (totalTime / 1000);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'automerge-lifecycle',
    datasetSize: iterations,
    operationType: operation,
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime,
      maxOperationTime,
      throughput,
      bytesPerSecond,
      errorRate: errors.length / iterations,
    },
    errors: errors.length > 0 ? errors : undefined,
    timestamp: Date.now(),
  };
}

/**
 * Test operation log persistence (CRDT operation history)
 */
export async function runOperationLogBenchmark(
  adapter: StorageAdapter,
  documentId: string,
  operationCount: number
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();

  console.log(`\nTesting operation log: ${operationCount} operations...`);

  const operations: any[] = [];
  const operationTimes: number[] = [];
  let totalBytes = 0;

  // Generate and save operations
  const startTime = performance.now();

  for (let i = 0; i < operationCount; i++) {
    const operation = {
      docId: documentId,
      opId: `op-${i}`,
      type: ['insert', 'update', 'delete'][i % 3],
      timestamp: Date.now(),
      actor: `actor-${i % 3}`,
      path: ['items', i % 10],
      value: generateAutomergeData(1000), // Small operation
    };

    const opKey = `${documentId}:op:${i}`;
    const opStart = performance.now();

    try {
      const metrics = await adapter.write(opKey, operation);
      operationTimes.push(metrics.operationTime);
      if (metrics.bytesWritten) {
        totalBytes += metrics.bytesWritten;
      }
      operations.push(operation);
    } catch (error) {
      console.error(`Error saving operation ${i}: ${error}`);
    }
  }

  const endTime = performance.now();
  const totalTime = endTime - startTime;

  const avgOperationTime = operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length;
  const minOperationTime = Math.min(...operationTimes);
  const maxOperationTime = Math.max(...operationTimes);
  const throughput = operationCount / (totalTime / 1000);
  const bytesPerSecond = totalBytes / (totalTime / 1000);

  console.log(`  ✓ Saved ${operationCount} operations in ${totalTime.toFixed(2)}ms`);
  console.log(`  ✓ Throughput: ${throughput.toFixed(2)} ops/s`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'automerge-oplog',
    datasetSize: operationCount,
    operationType: 'oplog-persist',
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime,
      maxOperationTime,
      throughput,
      bytesPerSecond,
    },
    timestamp: Date.now(),
  };
}

/**
 * Test conflict resolution performance
 */
export async function runConflictResolutionBenchmark(
  adapter: StorageAdapter,
  conflictCount: number
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();

  console.log(`\nTesting conflict resolution: ${conflictCount} conflicts...`);

  const operationTimes: number[] = [];
  let totalBytes = 0;
  const errors: string[] = [];

  const startTime = performance.now();

  for (let i = 0; i < conflictCount; i++) {
    const docKey = `conflict-doc-${i}`;

    // Create two conflicting versions
    const version1 = generateAutomergeData(10000);
    version1.metadata.version = 1;
    version1.metadata.actor = 'actor-1';

    const version2 = generateAutomergeData(10000);
    version2.metadata.version = 1;
    version2.metadata.actor = 'actor-2';

    const conflictStart = performance.now();

    try {
      // Save version 1
      await adapter.write(docKey, version1);

      // Simulate concurrent write (version 2)
      // In real CRDT, this would trigger merge
      const merged = {
        ...version1,
        ...version2,
        metadata: {
          version: 2,
          mergedFrom: ['actor-1', 'actor-2'],
          mergedAt: Date.now(),
        },
      };

      const metrics = await adapter.write(docKey, merged);
      operationTimes.push(performance.now() - conflictStart);
      if (metrics.bytesWritten) {
        totalBytes += metrics.bytesWritten;
      }
    } catch (error) {
      errors.push(`Conflict resolution error for ${docKey}: ${error}`);
    }
  }

  const endTime = performance.now();
  const totalTime = endTime - startTime;

  const avgOperationTime = operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length;
  const minOperationTime = Math.min(...operationTimes);
  const maxOperationTime = Math.max(...operationTimes);
  const throughput = conflictCount / (totalTime / 1000);
  const bytesPerSecond = totalBytes / (totalTime / 1000);

  console.log(`  ✓ Resolved ${conflictCount} conflicts in ${totalTime.toFixed(2)}ms`);
  console.log(`  ✓ Throughput: ${throughput.toFixed(2)} ops/s`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'automerge-conflict',
    datasetSize: conflictCount,
    operationType: 'conflict-resolution',
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime,
      maxOperationTime,
      throughput,
      bytesPerSecond,
      errorRate: errors.length / conflictCount,
    },
    errors: errors.length > 0 ? errors : undefined,
    timestamp: Date.now(),
  };
}
