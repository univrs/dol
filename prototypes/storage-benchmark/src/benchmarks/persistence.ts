/**
 * Persistence & Reliability Benchmark
 *
 * Tests data persistence across browser restarts and crash recovery.
 * Critical for VUDO Runtime local-first mode data safety.
 */

import { StorageAdapter } from '../adapters/base.js';
import { generateTestRecords, TestRecord } from '../utils/data-gen.js';
import { BenchmarkResult, getBrowserInfo } from '../utils/metrics.js';

export interface PersistenceBenchmarkConfig {
  adapter: StorageAdapter;
  datasetSize: number;
  tests: ('restart' | 'crash' | 'integrity' | 'transaction')[];
}

export interface PersistenceBenchmarkResults {
  results: BenchmarkResult[];
  dataIntegrityScore: number;   // 0-100, higher is better
  recoverySuccess: boolean;
}

/**
 * Run persistence benchmark
 */
export async function runPersistenceBenchmark(config: PersistenceBenchmarkConfig): Promise<PersistenceBenchmarkResults> {
  const results: BenchmarkResult[] = [];
  const browserInfo = getBrowserInfo();
  let integrityScores: number[] = [];
  let recoverySuccess = true;

  console.log('Starting persistence benchmark...');
  console.log(`Browser: ${browserInfo.name} ${browserInfo.version}`);
  console.log(`Adapter: ${config.adapter.getName()}`);

  for (const test of config.tests) {
    console.log(`\nTesting ${test}...`);

    try {
      let result: BenchmarkResult;
      let score: number;

      switch (test) {
        case 'restart':
          ({ result, score } = await testBrowserRestart(config.adapter, config.datasetSize));
          break;
        case 'crash':
          ({ result, score } = await testCrashRecovery(config.adapter, config.datasetSize));
          break;
        case 'integrity':
          ({ result, score } = await testDataIntegrity(config.adapter, config.datasetSize));
          break;
        case 'transaction':
          ({ result, score } = await testTransactionRollback(config.adapter, config.datasetSize));
          break;
      }

      results.push(result);
      integrityScores.push(score);

      if (score < 100) {
        recoverySuccess = false;
      }

      console.log(`  ✓ ${test}: ${score.toFixed(2)}% integrity`);

    } catch (error) {
      console.error(`  ✗ Error in ${test}: ${error}`);
      recoverySuccess = false;
      integrityScores.push(0);
    }
  }

  const avgIntegrityScore = integrityScores.reduce((sum, s) => sum + s, 0) / integrityScores.length;

  return {
    results,
    dataIntegrityScore: avgIntegrityScore,
    recoverySuccess,
  };
}

/**
 * Test data persistence across simulated browser restart
 */
async function testBrowserRestart(
  adapter: StorageAdapter,
  datasetSize: number
): Promise<{ result: BenchmarkResult; score: number }> {
  const browserInfo = getBrowserInfo();
  const errors: string[] = [];

  // Phase 1: Write data
  console.log('  Phase 1: Writing data...');
  const records = generateTestRecords(datasetSize, 'medium', 'restart-test');

  const writeStart = performance.now();
  for (const record of records) {
    const key = `restart-${record.id}`;
    try {
      await adapter.write(key, record);
    } catch (error) {
      errors.push(`Write error for ${key}: ${error}`);
    }
  }
  const writeTime = performance.now() - writeStart;

  // Phase 2: Close and reopen connection (simulate restart)
  console.log('  Phase 2: Simulating restart...');
  const reopenStart = performance.now();
  await adapter.close();
  await adapter.initialize();
  const reopenTime = performance.now() - reopenStart;

  // Phase 3: Verify all data persisted
  console.log('  Phase 3: Verifying data...');
  const verifyStart = performance.now();
  let foundCount = 0;
  let corruptedCount = 0;

  for (const record of records) {
    const key = `restart-${record.id}`;
    try {
      const { value } = await adapter.read(key);

      if (value) {
        foundCount++;

        // Verify data integrity
        if (JSON.stringify(value) !== JSON.stringify(record)) {
          corruptedCount++;
          errors.push(`Data corruption detected for ${key}`);
        }
      } else {
        errors.push(`Missing data for ${key}`);
      }
    } catch (error) {
      errors.push(`Read error for ${key}: ${error}`);
    }
  }

  const verifyTime = performance.now() - verifyStart;
  const totalTime = writeTime + reopenTime + verifyTime;

  const persistenceRate = (foundCount / datasetSize) * 100;
  const integrityScore = corruptedCount === 0 ? persistenceRate : persistenceRate * 0.5;

  console.log(`  Found: ${foundCount}/${datasetSize} (${persistenceRate.toFixed(2)}%)`);
  console.log(`  Corrupted: ${corruptedCount}`);

  return {
    result: {
      adapter: adapter.getName(),
      browser: `${browserInfo.name} ${browserInfo.version}`,
      scenario: 'persistence-restart',
      datasetSize,
      operationType: 'restart-recovery',
      metrics: {
        totalTime,
        avgOperationTime: totalTime / datasetSize,
        minOperationTime: 0,
        maxOperationTime: 0,
        throughput: datasetSize / (totalTime / 1000),
        errorRate: errors.length / datasetSize,
      },
      errors: errors.length > 0 ? errors : undefined,
      timestamp: Date.now(),
    },
    score: integrityScore,
  };
}

/**
 * Test crash recovery (interrupted writes)
 */
async function testCrashRecovery(
  adapter: StorageAdapter,
  datasetSize: number
): Promise<{ result: BenchmarkResult; score: number }> {
  const browserInfo = getBrowserInfo();
  const errors: string[] = [];

  console.log('  Simulating crash during writes...');

  const records = generateTestRecords(datasetSize, 'medium', 'crash-test');
  const writeStart = performance.now();

  // Write only half the records, then "crash" (close without proper cleanup)
  const halfwayPoint = Math.floor(datasetSize / 2);
  const writtenKeys: string[] = [];

  for (let i = 0; i < halfwayPoint; i++) {
    const record = records[i];
    const key = `crash-${record.id}`;
    try {
      await adapter.write(key, record);
      writtenKeys.push(key);
    } catch (error) {
      errors.push(`Write error for ${key}: ${error}`);
    }
  }

  // Simulate crash - abrupt close
  await adapter.close();

  // Reopen and verify recovery
  const recoveryStart = performance.now();
  await adapter.initialize();

  let recoveredCount = 0;
  for (const key of writtenKeys) {
    try {
      const { value } = await adapter.read(key);
      if (value) {
        recoveredCount++;
      }
    } catch (error) {
      errors.push(`Recovery read error for ${key}: ${error}`);
    }
  }

  const recoveryTime = performance.now() - recoveryStart;
  const totalTime = (performance.now() - writeStart);

  const recoveryRate = (recoveredCount / writtenKeys.length) * 100;

  console.log(`  Written before crash: ${writtenKeys.length}`);
  console.log(`  Recovered: ${recoveredCount}/${writtenKeys.length} (${recoveryRate.toFixed(2)}%)`);

  return {
    result: {
      adapter: adapter.getName(),
      browser: `${browserInfo.name} ${browserInfo.version}`,
      scenario: 'persistence-crash',
      datasetSize: writtenKeys.length,
      operationType: 'crash-recovery',
      metrics: {
        totalTime,
        avgOperationTime: recoveryTime / writtenKeys.length,
        minOperationTime: 0,
        maxOperationTime: 0,
        throughput: recoveredCount / (recoveryTime / 1000),
        errorRate: errors.length / writtenKeys.length,
      },
      errors: errors.length > 0 ? errors : undefined,
      timestamp: Date.now(),
    },
    score: recoveryRate,
  };
}

/**
 * Test data integrity verification
 */
async function testDataIntegrity(
  adapter: StorageAdapter,
  datasetSize: number
): Promise<{ result: BenchmarkResult; score: number }> {
  const browserInfo = getBrowserInfo();
  const errors: string[] = [];

  console.log('  Testing data integrity...');

  // Write data with checksums
  const records = generateTestRecords(datasetSize, 'medium', 'integrity-test');
  const checksums = new Map<string, string>();

  const writeStart = performance.now();
  for (const record of records) {
    const key = `integrity-${record.id}`;
    const checksum = simpleChecksum(JSON.stringify(record));
    checksums.set(key, checksum);

    try {
      await adapter.write(key, { ...record, __checksum: checksum });
    } catch (error) {
      errors.push(`Write error for ${key}: ${error}`);
    }
  }

  // Read back and verify checksums
  let validCount = 0;
  let corruptedCount = 0;

  for (const [key, expectedChecksum] of checksums.entries()) {
    try {
      const { value } = await adapter.read(key);

      if (value) {
        const actualChecksum = simpleChecksum(JSON.stringify({
          ...value,
          __checksum: undefined,
        }));

        if (actualChecksum === expectedChecksum) {
          validCount++;
        } else {
          corruptedCount++;
          errors.push(`Checksum mismatch for ${key}`);
        }
      } else {
        errors.push(`Missing data for ${key}`);
      }
    } catch (error) {
      errors.push(`Read error for ${key}: ${error}`);
    }
  }

  const totalTime = performance.now() - writeStart;
  const integrityScore = (validCount / datasetSize) * 100;

  console.log(`  Valid: ${validCount}/${datasetSize}`);
  console.log(`  Corrupted: ${corruptedCount}`);

  return {
    result: {
      adapter: adapter.getName(),
      browser: `${browserInfo.name} ${browserInfo.version}`,
      scenario: 'persistence-integrity',
      datasetSize,
      operationType: 'integrity-check',
      metrics: {
        totalTime,
        avgOperationTime: totalTime / (datasetSize * 2), // Read + write
        minOperationTime: 0,
        maxOperationTime: 0,
        throughput: (datasetSize * 2) / (totalTime / 1000),
        errorRate: errors.length / datasetSize,
      },
      errors: errors.length > 0 ? errors : undefined,
      timestamp: Date.now(),
    },
    score: integrityScore,
  };
}

/**
 * Test transaction rollback
 */
async function testTransactionRollback(
  adapter: StorageAdapter,
  datasetSize: number
): Promise<{ result: BenchmarkResult; score: number }> {
  const browserInfo = getBrowserInfo();
  const errors: string[] = [];

  if (!adapter.supportsTransactions()) {
    console.log('  Adapter does not support transactions, skipping...');
    return {
      result: {
        adapter: adapter.getName(),
        browser: `${browserInfo.name} ${browserInfo.version}`,
        scenario: 'persistence-transaction',
        datasetSize: 0,
        operationType: 'rollback',
        metrics: {
          totalTime: 0,
          avgOperationTime: 0,
          minOperationTime: 0,
          maxOperationTime: 0,
          throughput: 0,
        },
        timestamp: Date.now(),
      },
      score: 100, // N/A
    };
  }

  console.log('  Testing transaction rollback...');

  const records = generateTestRecords(datasetSize, 'medium', 'txn-test');
  const startTime = performance.now();

  // Test 1: Successful transaction
  const txn1 = await adapter.beginTransaction();
  for (let i = 0; i < datasetSize / 2; i++) {
    const record = records[i];
    const key = `txn-success-${record.id}`;
    try {
      await adapter.write(key, record);
    } catch (error) {
      errors.push(`Transaction write error for ${key}: ${error}`);
    }
  }
  await adapter.commitTransaction(txn1);

  // Verify committed data exists
  let committedCount = 0;
  for (let i = 0; i < datasetSize / 2; i++) {
    const key = `txn-success-${records[i].id}`;
    const { value } = await adapter.read(key);
    if (value) committedCount++;
  }

  // Test 2: Rolled back transaction
  const txn2 = await adapter.beginTransaction();
  for (let i = datasetSize / 2; i < datasetSize; i++) {
    const record = records[i];
    const key = `txn-rollback-${record.id}`;
    try {
      await adapter.write(key, record);
    } catch (error) {
      errors.push(`Transaction write error for ${key}: ${error}`);
    }
  }
  await adapter.rollbackTransaction(txn2);

  // Verify rolled back data does NOT exist
  let rolledBackCount = 0;
  for (let i = datasetSize / 2; i < datasetSize; i++) {
    const key = `txn-rollback-${records[i].id}`;
    try {
      const { value } = await adapter.read(key);
      if (value) {
        rolledBackCount++;
        errors.push(`Rollback failed: ${key} still exists`);
      }
    } catch (error) {
      // Expected - key should not exist
    }
  }

  const totalTime = performance.now() - startTime;

  const commitScore = (committedCount / (datasetSize / 2)) * 100;
  const rollbackScore = ((datasetSize / 2 - rolledBackCount) / (datasetSize / 2)) * 100;
  const avgScore = (commitScore + rollbackScore) / 2;

  console.log(`  Committed: ${committedCount}/${datasetSize / 2}`);
  console.log(`  Properly rolled back: ${datasetSize / 2 - rolledBackCount}/${datasetSize / 2}`);

  return {
    result: {
      adapter: adapter.getName(),
      browser: `${browserInfo.name} ${browserInfo.version}`,
      scenario: 'persistence-transaction',
      datasetSize,
      operationType: 'rollback',
      metrics: {
        totalTime,
        avgOperationTime: totalTime / datasetSize,
        minOperationTime: 0,
        maxOperationTime: 0,
        throughput: datasetSize / (totalTime / 1000),
        errorRate: errors.length / datasetSize,
      },
      errors: errors.length > 0 ? errors : undefined,
      timestamp: Date.now(),
    },
    score: avgScore,
  };
}

/**
 * Simple checksum function
 */
function simpleChecksum(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return hash.toString(16);
}
