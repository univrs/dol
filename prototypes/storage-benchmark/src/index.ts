/**
 * Storage Benchmark Suite - Main Entry Point
 *
 * Orchestrates comprehensive storage evaluation for VUDO Runtime.
 */

import { createStorageAdapter, getAvailableAdapters, checkBrowserSupport, StorageAdapterType } from './adapters/index.js';
import { runWriteBenchmark, runUpdateBenchmark } from './benchmarks/write.js';
import { runReadBenchmark, runRangeQueryBenchmark, runCacheBenchmark } from './benchmarks/read.js';
import { runAutomergeBenchmark, runOperationLogBenchmark, runConflictResolutionBenchmark } from './benchmarks/automerge.js';
import { runMultiTabBenchmark, testCrossTabCommunication } from './benchmarks/multi-tab.js';
import { runPersistenceBenchmark } from './benchmarks/persistence.js';
import { runQuotaBenchmark, testCompressionEffectiveness, testEvictionBehavior } from './benchmarks/quota.js';
import { globalMetrics, getBrowserInfo } from './utils/metrics.js';

/**
 * Complete benchmark suite configuration
 */
export interface BenchmarkSuiteConfig {
  adapters?: StorageAdapterType[];       // Adapters to test (default: all available)
  scenarios?: string[];                   // Scenarios to run (default: all)
  datasetSizes?: number[];                // Dataset sizes (default: [1000, 10000, 100000])
  documentSizes?: number[];               // Automerge doc sizes (default: [100KB, 1MB, 10MB, 100MB])
  multiTabCounts?: number[];              // Tab counts (default: [2, 5, 10])
  quotaTargets?: number[];                // Quota targets (default: [50MB, 100MB, 200MB])
  iterations?: number;                    // Iterations per test (default: 3)
}

/**
 * Run complete benchmark suite
 */
export async function runCompleteBenchmarkSuite(config: BenchmarkSuiteConfig = {}): Promise<void> {
  console.log('='.repeat(80));
  console.log('VUDO Runtime Storage Benchmark Suite');
  console.log('='.repeat(80));

  const browserInfo = getBrowserInfo();
  console.log(`\nBrowser: ${browserInfo.name} ${browserInfo.version}`);
  console.log(`Platform: ${browserInfo.platform}`);
  console.log(`Date: ${new Date().toISOString()}\n`);

  // Check browser support
  const support = await checkBrowserSupport();
  console.log('Browser Support:');
  console.log(`  IndexedDB: ${support.indexeddb ? '✓' : '✗'}`);
  console.log(`  OPFS + SQLite: ${support['opfs-sqlite'] ? '✓' : '✗'}`);
  console.log(`  cr-sqlite: ${support['cr-sqlite'] ? '✓' : '✗'}`);
  console.log('');

  // Determine which adapters to test
  const adaptersToTest: StorageAdapterType[] = config.adapters ||
    getAvailableAdapters().filter(type => support[type]);

  if (adaptersToTest.length === 0) {
    console.error('No supported storage adapters available!');
    return;
  }

  console.log(`Testing ${adaptersToTest.length} adapters: ${adaptersToTest.join(', ')}\n`);

  // Default configurations
  const datasetSizes = config.datasetSizes || [1000, 10000, 100000];
  const documentSizes = config.documentSizes || [
    100 * 1024,           // 100KB
    1024 * 1024,          // 1MB
    10 * 1024 * 1024,     // 10MB
    100 * 1024 * 1024,    // 100MB
  ];
  const multiTabCounts = config.multiTabCounts || [2, 5, 10];
  const quotaTargets = config.quotaTargets || [
    50 * 1024 * 1024,     // 50MB
    100 * 1024 * 1024,    // 100MB
    200 * 1024 * 1024,    // 200MB
  ];

  // Run benchmarks for each adapter
  for (const adapterType of adaptersToTest) {
    console.log('\n' + '='.repeat(80));
    console.log(`Testing: ${adapterType}`);
    console.log('='.repeat(80) + '\n');

    try {
      const adapter = await createStorageAdapter({
        type: adapterType,
        dbName: `benchmark-${adapterType}`,
        version: 1,
      });

      // 1. Write Throughput Benchmark
      if (!config.scenarios || config.scenarios.includes('write')) {
        console.log('\n--- Write Throughput Benchmark ---');
        await runWriteBenchmark({
          adapter,
          datasetSizes,
          patterns: ['sequential', 'batch', 'random'],
          batchSize: 100,
        });

        await runUpdateBenchmark(adapter, 10000, 3);
      }

      // 2. Read Throughput Benchmark
      if (!config.scenarios || config.scenarios.includes('read')) {
        console.log('\n--- Read Throughput Benchmark ---');
        await runReadBenchmark({
          adapter,
          datasetSizes,
          patterns: ['sequential', 'random', 'batch'],
          batchSize: 100,
        });

        await runRangeQueryBenchmark(adapter, 10000, 1000);
        await runCacheBenchmark(adapter, 1000, 3);
      }

      // 3. Automerge Lifecycle Benchmark
      if (!config.scenarios || config.scenarios.includes('automerge')) {
        console.log('\n--- Automerge Document Lifecycle Benchmark ---');
        await runAutomergeBenchmark({
          adapter,
          documentSizes,
          operations: ['save', 'load', 'sync', 'merge'],
          iterations: config.iterations || 3,
        });

        await runOperationLogBenchmark(adapter, 'test-doc', 1000);
        await runConflictResolutionBenchmark(adapter, 100);
      }

      // 4. Multi-Tab Coordination Benchmark
      if (!config.scenarios || config.scenarios.includes('multi-tab')) {
        console.log('\n--- Multi-Tab Coordination Benchmark ---');
        await runMultiTabBenchmark({
          adapter,
          tabCounts: multiTabCounts,
          writesPerTab: 100,
          coordinationStrategy: 'optimistic',
        });

        await testCrossTabCommunication(adapter, 100);
      }

      // 5. Persistence & Reliability Benchmark
      if (!config.scenarios || config.scenarios.includes('persistence')) {
        console.log('\n--- Persistence & Reliability Benchmark ---');
        await runPersistenceBenchmark({
          adapter,
          datasetSize: 1000,
          tests: ['restart', 'crash', 'integrity', 'transaction'],
        });
      }

      // 6. Quota Management Benchmark
      if (!config.scenarios || config.scenarios.includes('quota')) {
        console.log('\n--- Storage Quota Benchmark ---');
        await runQuotaBenchmark({
          adapter,
          targetSizes: quotaTargets,
          testQuotaExceeded: true,
        });

        await testCompressionEffectiveness(adapter, 100);
        await testEvictionBehavior(adapter);
      }

      // Cleanup
      await adapter.close();

    } catch (error) {
      console.error(`\n✗ Error testing ${adapterType}:`, error);
    }
  }

  // Generate final report
  console.log('\n' + '='.repeat(80));
  console.log('Benchmark Complete!');
  console.log('='.repeat(80) + '\n');

  generateSummaryReport();
}

/**
 * Generate summary report
 */
function generateSummaryReport(): void {
  const results = globalMetrics.getResults();

  console.log('\nSummary Statistics:');
  console.log(`  Total tests run: ${results.length}`);

  const adapters = [...new Set(results.map(r => r.adapter))];
  console.log(`  Adapters tested: ${adapters.join(', ')}`);

  const scenarios = [...new Set(results.map(r => r.scenario))];
  console.log(`  Scenarios tested: ${scenarios.join(', ')}`);

  console.log('\nPerformance Comparison:');
  console.log(globalMetrics.generateComparisonTable());

  console.log('\nExport results:');
  console.log('  - JSON: globalMetrics.exportToJSON()');
  console.log('  - CSV: globalMetrics.exportToCSV()');
  console.log('\nSave results to results/ directory for analysis.');
}

/**
 * Export results to file
 */
export async function exportResults(format: 'json' | 'csv' = 'json'): Promise<void> {
  const browserInfo = getBrowserInfo();
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const filename = `results/${browserInfo.name}-${timestamp}.${format}`;

  const data = format === 'json'
    ? globalMetrics.exportToJSON()
    : globalMetrics.exportToCSV();

  // In a real implementation, this would use File System Access API or download
  console.log(`Exporting to ${filename}...`);
  console.log(data);
}

// Auto-run on page load if in browser
if (typeof window !== 'undefined') {
  console.log('Storage Benchmark Suite loaded. Run benchmarks with:');
  console.log('  await runCompleteBenchmarkSuite()');
  console.log('  await exportResults("json")');
}

export * from './adapters/index.js';
export * from './benchmarks/index.js';
export * from './utils/metrics.js';
export * from './utils/data-gen.js';
