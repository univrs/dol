/**
 * Write Throughput Benchmark
 *
 * Tests write performance across different dataset sizes and patterns.
 */

import { StorageAdapter } from '../adapters/base.js';
import { generateDataset, DatasetConfig } from '../utils/data-gen.js';
import { BenchmarkResult, getBrowserInfo, getMemoryUsage } from '../utils/metrics.js';

export interface WriteBenchmarkConfig {
  adapter: StorageAdapter;
  datasetSizes: number[];       // [1000, 10000, 100000]
  patterns: ('sequential' | 'batch' | 'random')[];
  batchSize?: number;           // Records per batch (default: 100)
}

export interface WriteBenchmarkResults {
  results: BenchmarkResult[];
  summary: {
    totalRecordsWritten: number;
    totalTimeMs: number;
    avgThroughput: number;
    errors: number;
  };
}

/**
 * Run write throughput benchmark
 */
export async function runWriteBenchmark(config: WriteBenchmarkConfig): Promise<WriteBenchmarkResults> {
  const results: BenchmarkResult[] = [];
  const browserInfo = getBrowserInfo();
  let totalRecords = 0;
  let totalTime = 0;
  let totalErrors = 0;

  console.log('Starting write benchmark...');
  console.log(`Browser: ${browserInfo.name} ${browserInfo.version}`);
  console.log(`Adapter: ${config.adapter.getName()}`);

  for (const size of config.datasetSizes) {
    for (const pattern of config.patterns) {
      console.log(`\nTesting ${pattern} writes with ${size} records...`);

      try {
        // Clear storage before each test
        await config.adapter.clear();

        const result = await runWritePattern(
          config.adapter,
          size,
          pattern,
          config.batchSize || 100
        );

        results.push({
          adapter: config.adapter.getName(),
          browser: `${browserInfo.name} ${browserInfo.version}`,
          scenario: 'write-throughput',
          datasetSize: size,
          operationType: pattern,
          metrics: result.metrics,
          errors: result.errors,
          timestamp: Date.now(),
        });

        totalRecords += size;
        totalTime += result.metrics.totalTime;
        if (result.errors && result.errors.length > 0) {
          totalErrors += result.errors.length;
        }

        console.log(`  ✓ Completed in ${result.metrics.totalTime.toFixed(2)}ms`);
        console.log(`  ✓ Throughput: ${result.metrics.throughput.toFixed(2)} ops/s`);
        if (result.metrics.bytesPerSecond) {
          console.log(`  ✓ Data rate: ${(result.metrics.bytesPerSecond / 1024 / 1024).toFixed(2)} MB/s`);
        }

      } catch (error) {
        console.error(`  ✗ Error: ${error}`);
        totalErrors++;
      }
    }
  }

  const avgThroughput = totalRecords / (totalTime / 1000); // ops/s

  return {
    results,
    summary: {
      totalRecordsWritten: totalRecords,
      totalTimeMs: totalTime,
      avgThroughput,
      errors: totalErrors,
    },
  };
}

/**
 * Run a specific write pattern
 */
async function runWritePattern(
  adapter: StorageAdapter,
  count: number,
  pattern: 'sequential' | 'batch' | 'random',
  batchSize: number
): Promise<{
  metrics: BenchmarkResult['metrics'];
  errors?: string[];
}> {
  const datasetConfig: DatasetConfig = {
    count,
    size: 'medium',
    keyPattern: pattern === 'random' ? 'random' : 'sequential',
    distribution: pattern === 'random' ? 'random' : 'uniform',
  };

  const dataset = generateDataset(datasetConfig);
  const errors: string[] = [];
  const operationTimes: number[] = [];
  let totalBytes = 0;

  const memoryBefore = await getMemoryUsage();
  const startTime = performance.now();

  if (pattern === 'batch') {
    // Batch writes
    for (let i = 0; i < dataset.length; i += batchSize) {
      const batch = dataset.slice(i, Math.min(i + batchSize, dataset.length));
      const batchRecords = batch.map(item => ({
        key: item.key,
        value: item.value,
      }));

      const opStart = performance.now();
      try {
        const metrics = await adapter.writeBatch(batchRecords);
        operationTimes.push(metrics.operationTime);
        if (metrics.bytesWritten) {
          totalBytes += metrics.bytesWritten;
        }
        if (metrics.errors) {
          errors.push(...metrics.errors);
        }
      } catch (error) {
        errors.push(`Batch write error: ${error}`);
      }
    }
  } else {
    // Sequential or random individual writes
    for (const item of dataset) {
      const opStart = performance.now();
      try {
        const metrics = await adapter.write(item.key, item.value);
        operationTimes.push(metrics.operationTime);
        if (metrics.bytesWritten) {
          totalBytes += metrics.bytesWritten;
        }
      } catch (error) {
        errors.push(`Write error for key ${item.key}: ${error}`);
      }
    }
  }

  const endTime = performance.now();
  const totalTime = endTime - startTime;
  const memoryAfter = await getMemoryUsage();

  const avgOperationTime = operationTimes.length > 0
    ? operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length
    : 0;

  const minOperationTime = operationTimes.length > 0 ? Math.min(...operationTimes) : 0;
  const maxOperationTime = operationTimes.length > 0 ? Math.max(...operationTimes) : 0;
  const throughput = count / (totalTime / 1000); // ops/s
  const bytesPerSecond = totalBytes / (totalTime / 1000);
  const memoryUsed = memoryAfter && memoryBefore ? memoryAfter - memoryBefore : undefined;
  const errorRate = errors.length / count;

  return {
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime,
      maxOperationTime,
      throughput,
      bytesPerSecond,
      memoryUsed,
      errorRate: errorRate > 0 ? errorRate : undefined,
    },
    errors: errors.length > 0 ? errors : undefined,
  };
}

/**
 * Test write with updates (overwrite existing keys)
 */
export async function runUpdateBenchmark(
  adapter: StorageAdapter,
  count: number,
  updateRounds: number = 3
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();
  const dataset = generateDataset({ count, size: 'medium', keyPattern: 'sequential' });

  console.log(`\nTesting updates: ${count} records, ${updateRounds} rounds...`);

  // Initial write
  await adapter.clear();
  for (const item of dataset) {
    await adapter.write(item.key, item.value);
  }

  const operationTimes: number[] = [];
  let totalBytes = 0;
  const errors: string[] = [];

  const startTime = performance.now();

  // Perform update rounds
  for (let round = 0; round < updateRounds; round++) {
    for (const item of dataset) {
      // Modify the value
      item.value.metadata.updated = Date.now();
      item.value.metadata.version++;

      try {
        const metrics = await adapter.write(item.key, item.value);
        operationTimes.push(metrics.operationTime);
        if (metrics.bytesWritten) {
          totalBytes += metrics.bytesWritten;
        }
      } catch (error) {
        errors.push(`Update error for key ${item.key}: ${error}`);
      }
    }
  }

  const endTime = performance.now();
  const totalTime = endTime - startTime;
  const totalOperations = count * updateRounds;

  const avgOperationTime = operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length;
  const minOperationTime = Math.min(...operationTimes);
  const maxOperationTime = Math.max(...operationTimes);
  const throughput = totalOperations / (totalTime / 1000);
  const bytesPerSecond = totalBytes / (totalTime / 1000);

  console.log(`  ✓ Completed ${totalOperations} updates in ${totalTime.toFixed(2)}ms`);
  console.log(`  ✓ Throughput: ${throughput.toFixed(2)} ops/s`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'write-update',
    datasetSize: count,
    operationType: 'update',
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime,
      maxOperationTime,
      throughput,
      bytesPerSecond,
      errorRate: errors.length / totalOperations,
    },
    errors: errors.length > 0 ? errors : undefined,
    timestamp: Date.now(),
  };
}
