/**
 * Read Throughput Benchmark
 *
 * Tests read performance across different access patterns.
 */

import { StorageAdapter } from '../adapters/base.js';
import { generateDataset, generateReadKeys } from '../utils/data-gen.js';
import { BenchmarkResult, getBrowserInfo, getMemoryUsage } from '../utils/metrics.js';

export interface ReadBenchmarkConfig {
  adapter: StorageAdapter;
  datasetSizes: number[];
  patterns: ('sequential' | 'random' | 'batch')[];
  batchSize?: number;
}

export interface ReadBenchmarkResults {
  results: BenchmarkResult[];
  summary: {
    totalReadsPerformed: number;
    totalTimeMs: number;
    avgThroughput: number;
    cacheHitRate?: number;
  };
}

/**
 * Run read throughput benchmark
 */
export async function runReadBenchmark(config: ReadBenchmarkConfig): Promise<ReadBenchmarkResults> {
  const results: BenchmarkResult[] = [];
  const browserInfo = getBrowserInfo();
  let totalReads = 0;
  let totalTime = 0;

  console.log('Starting read benchmark...');
  console.log(`Browser: ${browserInfo.name} ${browserInfo.version}`);
  console.log(`Adapter: ${config.adapter.getName()}`);

  for (const size of config.datasetSizes) {
    console.log(`\nPreparing dataset with ${size} records...`);

    // Prepare data
    await config.adapter.clear();
    const dataset = generateDataset({ count: size, size: 'medium', keyPattern: 'sequential' });
    const keys = dataset.map(item => item.key);

    // Write data
    for (const item of dataset) {
      await config.adapter.write(item.key, item.value);
    }

    console.log('  ✓ Data prepared');

    for (const pattern of config.patterns) {
      console.log(`\nTesting ${pattern} reads with ${size} records...`);

      try {
        const result = await runReadPattern(
          config.adapter,
          keys,
          pattern,
          config.batchSize || 100
        );

        results.push({
          adapter: config.adapter.getName(),
          browser: `${browserInfo.name} ${browserInfo.version}`,
          scenario: 'read-throughput',
          datasetSize: size,
          operationType: pattern,
          metrics: result.metrics,
          errors: result.errors,
          timestamp: Date.now(),
        });

        totalReads += keys.length;
        totalTime += result.metrics.totalTime;

        console.log(`  ✓ Completed in ${result.metrics.totalTime.toFixed(2)}ms`);
        console.log(`  ✓ Throughput: ${result.metrics.throughput.toFixed(2)} ops/s`);
        if (result.metrics.bytesPerSecond) {
          console.log(`  ✓ Data rate: ${(result.metrics.bytesPerSecond / 1024 / 1024).toFixed(2)} MB/s`);
        }

      } catch (error) {
        console.error(`  ✗ Error: ${error}`);
      }
    }
  }

  const avgThroughput = totalReads / (totalTime / 1000);

  return {
    results,
    summary: {
      totalReadsPerformed: totalReads,
      totalTimeMs: totalTime,
      avgThroughput,
    },
  };
}

/**
 * Run a specific read pattern
 */
async function runReadPattern(
  adapter: StorageAdapter,
  keys: string[],
  pattern: 'sequential' | 'random' | 'batch',
  batchSize: number
): Promise<{
  metrics: BenchmarkResult['metrics'];
  errors?: string[];
}> {
  const errors: string[] = [];
  const operationTimes: number[] = [];
  let totalBytes = 0;

  // Generate read keys based on pattern
  const readKeys = generateReadKeys(keys, keys.length, pattern === 'random' ? 'random' : 'sequential');

  const memoryBefore = await getMemoryUsage();
  const startTime = performance.now();

  if (pattern === 'batch') {
    // Batch reads
    for (let i = 0; i < readKeys.length; i += batchSize) {
      const batch = readKeys.slice(i, Math.min(i + batchSize, readKeys.length));

      const opStart = performance.now();
      try {
        const { values, metrics } = await adapter.readBatch(batch);
        operationTimes.push(metrics.operationTime);
        if (metrics.bytesRead) {
          totalBytes += metrics.bytesRead;
        }
        if (metrics.errors) {
          errors.push(...metrics.errors);
        }
      } catch (error) {
        errors.push(`Batch read error: ${error}`);
      }
    }
  } else {
    // Sequential or random individual reads
    for (const key of readKeys) {
      const opStart = performance.now();
      try {
        const { value, metrics } = await adapter.read(key);
        operationTimes.push(metrics.operationTime);
        if (metrics.bytesRead) {
          totalBytes += metrics.bytesRead;
        }

        if (!value) {
          errors.push(`Key not found: ${key}`);
        }
      } catch (error) {
        errors.push(`Read error for key ${key}: ${error}`);
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
  const throughput = readKeys.length / (totalTime / 1000);
  const bytesPerSecond = totalBytes / (totalTime / 1000);
  const memoryUsed = memoryAfter && memoryBefore ? memoryAfter - memoryBefore : undefined;
  const errorRate = errors.length / readKeys.length;

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
 * Test range query performance
 */
export async function runRangeQueryBenchmark(
  adapter: StorageAdapter,
  datasetSize: number,
  rangeSize: number
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();

  console.log(`\nTesting range query: ${rangeSize} out of ${datasetSize} records...`);

  // Prepare data with sequential keys
  await adapter.clear();
  const dataset = generateDataset({
    count: datasetSize,
    size: 'medium',
    keyPattern: 'sequential',
  });

  for (const item of dataset) {
    await adapter.write(item.key, item.value);
  }

  // Read a range
  const startIndex = Math.floor((datasetSize - rangeSize) / 2);
  const rangeKeys = dataset.slice(startIndex, startIndex + rangeSize).map(item => item.key);

  const operationTimes: number[] = [];
  let totalBytes = 0;
  const errors: string[] = [];

  const startTime = performance.now();

  for (const key of rangeKeys) {
    try {
      const { value, metrics } = await adapter.read(key);
      operationTimes.push(metrics.operationTime);
      if (metrics.bytesRead) {
        totalBytes += metrics.bytesRead;
      }
    } catch (error) {
      errors.push(`Range query error for key ${key}: ${error}`);
    }
  }

  const endTime = performance.now();
  const totalTime = endTime - startTime;

  const avgOperationTime = operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length;
  const minOperationTime = Math.min(...operationTimes);
  const maxOperationTime = Math.max(...operationTimes);
  const throughput = rangeSize / (totalTime / 1000);
  const bytesPerSecond = totalBytes / (totalTime / 1000);

  console.log(`  ✓ Completed in ${totalTime.toFixed(2)}ms`);
  console.log(`  ✓ Throughput: ${throughput.toFixed(2)} ops/s`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'read-range',
    datasetSize,
    operationType: 'range',
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime,
      maxOperationTime,
      throughput,
      bytesPerSecond,
      errorRate: errors.length / rangeSize,
    },
    errors: errors.length > 0 ? errors : undefined,
    timestamp: Date.now(),
  };
}

/**
 * Test cache effectiveness (read same data multiple times)
 */
export async function runCacheBenchmark(
  adapter: StorageAdapter,
  datasetSize: number,
  iterations: number = 3
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();

  console.log(`\nTesting cache: ${datasetSize} records, ${iterations} iterations...`);

  // Prepare data
  await adapter.clear();
  const dataset = generateDataset({
    count: datasetSize,
    size: 'medium',
    keyPattern: 'sequential',
  });

  for (const item of dataset) {
    await adapter.write(item.key, item.value);
  }

  const keys = dataset.map(item => item.key);
  const roundTimes: number[] = [];

  // Read multiple times to test caching
  for (let round = 0; round < iterations; round++) {
    const roundStart = performance.now();

    for (const key of keys) {
      await adapter.read(key);
    }

    const roundEnd = performance.now();
    roundTimes.push(roundEnd - roundStart);
    console.log(`  Round ${round + 1}: ${(roundEnd - roundStart).toFixed(2)}ms`);
  }

  const totalTime = roundTimes.reduce((sum, t) => sum + t, 0);
  const avgOperationTime = totalTime / (datasetSize * iterations);
  const throughput = (datasetSize * iterations) / (totalTime / 1000);

  // Check if later rounds are faster (indicates caching)
  const firstRoundTime = roundTimes[0];
  const lastRoundTime = roundTimes[roundTimes.length - 1];
  const improvement = ((firstRoundTime - lastRoundTime) / firstRoundTime) * 100;

  console.log(`  ✓ Cache improvement: ${improvement.toFixed(2)}%`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'read-cache',
    datasetSize,
    operationType: 'cached-read',
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime: Math.min(...roundTimes) / datasetSize,
      maxOperationTime: Math.max(...roundTimes) / datasetSize,
      throughput,
    },
    timestamp: Date.now(),
  };
}
