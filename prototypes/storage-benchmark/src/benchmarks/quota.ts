/**
 * Storage Quota Management Benchmark
 *
 * Tests storage quota behavior and handling when approaching limits.
 * Important for VUDO Runtime to handle large datasets gracefully.
 */

import { StorageAdapter } from '../adapters/base.js';
import { generateRecordsWithSize } from '../utils/data-gen.js';
import { BenchmarkResult, getBrowserInfo } from '../utils/metrics.js';

export interface QuotaBenchmarkConfig {
  adapter: StorageAdapter;
  targetSizes: number[];        // [50MB, 100MB, 200MB]
  testQuotaExceeded: boolean;
}

export interface QuotaBenchmarkResults {
  results: BenchmarkResult[];
  quotaInfo: {
    available: number;
    used: number;
    quota: number;
    percentage: number;
  };
  quotaExceededHandling: 'graceful' | 'error' | 'silent' | 'eviction';
}

/**
 * Run storage quota benchmark
 */
export async function runQuotaBenchmark(config: QuotaBenchmarkConfig): Promise<QuotaBenchmarkResults> {
  const results: BenchmarkResult[] = [];
  const browserInfo = getBrowserInfo();

  console.log('Starting quota benchmark...');
  console.log(`Browser: ${browserInfo.name} ${browserInfo.version}`);
  console.log(`Adapter: ${config.adapter.getName()}`);

  // Get initial quota info
  const initialQuota = await getStorageQuota();
  console.log(`\nInitial quota: ${(initialQuota.quota / 1024 / 1024).toFixed(2)}MB`);
  console.log(`Used: ${(initialQuota.used / 1024 / 1024).toFixed(2)}MB`);
  console.log(`Available: ${(initialQuota.available / 1024 / 1024).toFixed(2)}MB`);

  let quotaExceededHandling: QuotaBenchmarkResults['quotaExceededHandling'] = 'graceful';

  // Clear storage before testing
  await config.adapter.clear();

  for (const targetSize of config.targetSizes) {
    console.log(`\nFilling storage to ${(targetSize / 1024 / 1024).toFixed(2)}MB...`);

    try {
      const result = await fillStorage(config.adapter, targetSize);
      results.push(result);

      const quota = await getStorageQuota();
      console.log(`  ✓ Current usage: ${(quota.used / 1024 / 1024).toFixed(2)}MB`);
      console.log(`  ✓ Quota: ${(quota.percentage).toFixed(2)}%`);

    } catch (error) {
      console.error(`  ✗ Error: ${error}`);
    }
  }

  // Test quota exceeded behavior if requested
  if (config.testQuotaExceeded) {
    console.log(`\nTesting quota exceeded behavior...`);
    const handling = await testQuotaExceeded(config.adapter);
    quotaExceededHandling = handling;
    console.log(`  Handling: ${handling}`);
  }

  const finalQuota = await getStorageQuota();

  return {
    results,
    quotaInfo: finalQuota,
    quotaExceededHandling,
  };
}

/**
 * Fill storage to target size
 */
async function fillStorage(
  adapter: StorageAdapter,
  targetBytes: number
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();
  const errors: string[] = [];
  const operationTimes: number[] = [];
  let totalBytesWritten = 0;
  let recordCount = 0;

  const startTime = performance.now();

  // Write records until we reach target size
  const recordSize = 100 * 1024; // 100KB per record
  const targetRecords = Math.floor(targetBytes / recordSize);

  console.log(`  Target: ${targetRecords} records of ~${(recordSize / 1024).toFixed(2)}KB each`);

  for (let i = 0; i < targetRecords; i++) {
    const key = `quota-test-${i}`;
    const record = generateRecordsWithSize(1, recordSize)[0];

    const opStart = performance.now();
    try {
      const metrics = await adapter.write(key, record);
      operationTimes.push(metrics.operationTime);

      if (metrics.bytesWritten) {
        totalBytesWritten += metrics.bytesWritten;
      }

      recordCount++;

      // Report progress every 10%
      if (i > 0 && i % Math.floor(targetRecords / 10) === 0) {
        const progress = (i / targetRecords) * 100;
        console.log(`  Progress: ${progress.toFixed(0)}% (${(totalBytesWritten / 1024 / 1024).toFixed(2)}MB)`);
      }

    } catch (error) {
      errors.push(`Write error at record ${i}: ${error}`);

      // Check if quota exceeded
      if (String(error).includes('quota') || String(error).includes('QuotaExceeded')) {
        console.log(`  Quota exceeded at ${(totalBytesWritten / 1024 / 1024).toFixed(2)}MB`);
        break;
      }
    }
  }

  const endTime = performance.now();
  const totalTime = endTime - startTime;

  const avgOperationTime = operationTimes.length > 0
    ? operationTimes.reduce((sum, t) => sum + t, 0) / operationTimes.length
    : 0;

  const throughput = recordCount / (totalTime / 1000);
  const bytesPerSecond = totalBytesWritten / (totalTime / 1000);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'quota-management',
    datasetSize: recordCount,
    operationType: `fill-to-${(targetBytes / 1024 / 1024).toFixed(0)}MB`,
    metrics: {
      totalTime,
      avgOperationTime,
      minOperationTime: operationTimes.length > 0 ? Math.min(...operationTimes) : 0,
      maxOperationTime: operationTimes.length > 0 ? Math.max(...operationTimes) : 0,
      throughput,
      bytesPerSecond,
      storageUsed: totalBytesWritten,
      errorRate: errors.length / recordCount,
    },
    errors: errors.length > 0 ? errors : undefined,
    timestamp: Date.now(),
  };
}

/**
 * Test quota exceeded behavior
 */
async function testQuotaExceeded(
  adapter: StorageAdapter
): Promise<'graceful' | 'error' | 'silent' | 'eviction'> {
  try {
    // Try to write a large chunk that should exceed quota
    const hugeRecord = generateRecordsWithSize(1, 500 * 1024 * 1024)[0]; // 500MB

    try {
      await adapter.write('quota-exceeded-test', hugeRecord);
      // If this succeeds, check if old data was evicted
      const oldKey = 'quota-test-0';
      const { value } = await adapter.read(oldKey);

      if (!value) {
        return 'eviction'; // Browser evicted old data to make room
      }

      return 'silent'; // Write succeeded without error or eviction (unlikely)

    } catch (error) {
      const errorStr = String(error);

      if (errorStr.includes('quota') || errorStr.includes('QuotaExceeded')) {
        return 'graceful'; // Proper quota exceeded error
      }

      return 'error'; // Other error
    }

  } catch (error) {
    return 'error';
  }
}

/**
 * Get storage quota information
 */
async function getStorageQuota(): Promise<{
  available: number;
  used: number;
  quota: number;
  percentage: number;
}> {
  if ('storage' in navigator && 'estimate' in navigator.storage) {
    const estimate = await navigator.storage.estimate();
    const used = estimate.usage || 0;
    const quota = estimate.quota || 0;
    const available = quota - used;
    const percentage = quota > 0 ? (used / quota) * 100 : 0;

    return { available, used, quota, percentage };
  }

  // Fallback if StorageManager API not available
  return {
    available: 0,
    used: 0,
    quota: 0,
    percentage: 0,
  };
}

/**
 * Test storage compression effectiveness
 */
export async function testCompressionEffectiveness(
  adapter: StorageAdapter,
  datasetSize: number
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();

  console.log(`\nTesting compression effectiveness: ${datasetSize} records...`);

  // Generate highly compressible data (repeated patterns)
  const compressibleRecords = Array.from({ length: datasetSize }, (_, i) => ({
    key: `compress-${i}`,
    value: {
      id: `record-${i}`,
      type: 'compressible',
      data: {
        // Repeated pattern - highly compressible
        content: 'A'.repeat(10000),
        repeatedArray: Array(100).fill({ field: 'value', number: 123 }),
      },
      metadata: {
        created: Date.now(),
        updated: Date.now(),
        version: 1,
      },
    },
  }));

  const quotaBefore = await getStorageQuota();

  const startTime = performance.now();

  // Write compressible data
  for (const record of compressibleRecords) {
    await adapter.write(record.key, record.value);
  }

  const quotaAfter = await getStorageQuota();
  const totalTime = performance.now() - startTime;

  const actualStorageUsed = quotaAfter.used - quotaBefore.used;
  const uncompressedSize = compressibleRecords.reduce((sum, record) => {
    return sum + new Blob([JSON.stringify(record.value)]).size;
  }, 0);

  const compressionRatio = actualStorageUsed > 0
    ? uncompressedSize / actualStorageUsed
    : 1;

  console.log(`  Uncompressed size: ${(uncompressedSize / 1024 / 1024).toFixed(2)}MB`);
  console.log(`  Actual storage used: ${(actualStorageUsed / 1024 / 1024).toFixed(2)}MB`);
  console.log(`  Compression ratio: ${compressionRatio.toFixed(2)}x`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'quota-compression',
    datasetSize,
    operationType: 'compression-test',
    metrics: {
      totalTime,
      avgOperationTime: totalTime / datasetSize,
      minOperationTime: 0,
      maxOperationTime: 0,
      throughput: datasetSize / (totalTime / 1000),
      bytesPerSecond: uncompressedSize / (totalTime / 1000),
      storageUsed: actualStorageUsed,
    },
    timestamp: Date.now(),
  };
}

/**
 * Test storage eviction behavior
 */
export async function testEvictionBehavior(
  adapter: StorageAdapter
): Promise<BenchmarkResult> {
  const browserInfo = getBrowserInfo();

  console.log(`\nTesting storage eviction behavior...`);

  // Request persistent storage
  let persistent = false;
  if ('storage' in navigator && 'persist' in navigator.storage) {
    persistent = await navigator.storage.persist();
    console.log(`  Persistent storage: ${persistent}`);
  }

  // Fill storage with timestamped records
  const recordCount = 1000;
  const records: Array<{ key: string; timestamp: number }> = [];

  for (let i = 0; i < recordCount; i++) {
    const key = `eviction-${i}`;
    const timestamp = Date.now();
    const record = {
      id: key,
      timestamp,
      data: 'X'.repeat(10000), // 10KB per record
    };

    await adapter.write(key, record);
    records.push({ key, timestamp });

    // Small delay to ensure different timestamps
    await new Promise(resolve => setTimeout(resolve, 1));
  }

  // Check which records survived (simulate eviction pressure)
  let survivedCount = 0;
  const evictedKeys: string[] = [];

  for (const record of records) {
    const { value } = await adapter.read(record.key);
    if (value) {
      survivedCount++;
    } else {
      evictedKeys.push(record.key);
    }
  }

  const evictionRate = (evictedKeys.length / recordCount) * 100;

  console.log(`  Survived: ${survivedCount}/${recordCount}`);
  console.log(`  Eviction rate: ${evictionRate.toFixed(2)}%`);

  return {
    adapter: adapter.getName(),
    browser: `${browserInfo.name} ${browserInfo.version}`,
    scenario: 'quota-eviction',
    datasetSize: recordCount,
    operationType: 'eviction-test',
    metrics: {
      totalTime: 0,
      avgOperationTime: 0,
      minOperationTime: 0,
      maxOperationTime: 0,
      throughput: 0,
      errorRate: evictionRate / 100,
    },
    timestamp: Date.now(),
  };
}
