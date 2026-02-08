/**
 * Performance Metrics Collection
 *
 * Utilities for collecting and analyzing benchmark performance data.
 */

export interface BenchmarkResult {
  adapter: string;
  browser: string;
  scenario: string;
  datasetSize: number;
  operationType: string;
  metrics: {
    totalTime: number;           // Total time in ms
    avgOperationTime: number;    // Average time per operation
    minOperationTime: number;    // Fastest operation
    maxOperationTime: number;    // Slowest operation
    throughput: number;          // Operations per second
    bytesPerSecond?: number;     // Data throughput
    memoryUsed?: number;         // Memory footprint
    errorRate?: number;          // Percentage of failed operations
  };
  errors?: string[];
  timestamp: number;
}

export class MetricsCollector {
  private results: BenchmarkResult[] = [];

  /**
   * Record a benchmark result
   */
  record(result: BenchmarkResult): void {
    this.results.push(result);
  }

  /**
   * Get all results
   */
  getResults(): BenchmarkResult[] {
    return this.results;
  }

  /**
   * Get results filtered by criteria
   */
  getFilteredResults(filter: {
    adapter?: string;
    browser?: string;
    scenario?: string;
    operationType?: string;
  }): BenchmarkResult[] {
    return this.results.filter(result => {
      if (filter.adapter && result.adapter !== filter.adapter) return false;
      if (filter.browser && result.browser !== filter.browser) return false;
      if (filter.scenario && result.scenario !== filter.scenario) return false;
      if (filter.operationType && result.operationType !== filter.operationType) return false;
      return true;
    });
  }

  /**
   * Calculate statistics for a set of results
   */
  calculateStats(results: BenchmarkResult[]): {
    count: number;
    avgTotalTime: number;
    avgThroughput: number;
    avgBytesPerSecond: number;
    totalErrors: number;
  } {
    if (results.length === 0) {
      return {
        count: 0,
        avgTotalTime: 0,
        avgThroughput: 0,
        avgBytesPerSecond: 0,
        totalErrors: 0,
      };
    }

    const totalTime = results.reduce((sum, r) => sum + r.metrics.totalTime, 0);
    const totalThroughput = results.reduce((sum, r) => sum + r.metrics.throughput, 0);
    const totalBytes = results.reduce((sum, r) => sum + (r.metrics.bytesPerSecond || 0), 0);
    const totalErrors = results.reduce((sum, r) => sum + (r.errors?.length || 0), 0);

    return {
      count: results.length,
      avgTotalTime: totalTime / results.length,
      avgThroughput: totalThroughput / results.length,
      avgBytesPerSecond: totalBytes / results.length,
      totalErrors,
    };
  }

  /**
   * Generate comparison table
   */
  generateComparisonTable(): string {
    const adapters = [...new Set(this.results.map(r => r.adapter))];
    const scenarios = [...new Set(this.results.map(r => r.scenario))];

    let table = '| Scenario | ' + adapters.join(' | ') + ' |\n';
    table += '|----------|' + adapters.map(() => '------').join('|') + '|\n';

    for (const scenario of scenarios) {
      table += `| ${scenario} |`;
      for (const adapter of adapters) {
        const results = this.getFilteredResults({ adapter, scenario });
        const stats = this.calculateStats(results);
        table += ` ${stats.avgThroughput.toFixed(2)} ops/s |`;
      }
      table += '\n';
    }

    return table;
  }

  /**
   * Export results to JSON
   */
  exportToJSON(): string {
    return JSON.stringify({
      metadata: {
        exportDate: new Date().toISOString(),
        totalResults: this.results.length,
        adapters: [...new Set(this.results.map(r => r.adapter))],
        scenarios: [...new Set(this.results.map(r => r.scenario))],
      },
      results: this.results,
    }, null, 2);
  }

  /**
   * Export results to CSV
   */
  exportToCSV(): string {
    const headers = [
      'Adapter',
      'Browser',
      'Scenario',
      'Dataset Size',
      'Operation Type',
      'Total Time (ms)',
      'Avg Operation Time (ms)',
      'Throughput (ops/s)',
      'Bytes/Second',
      'Memory Used',
      'Error Rate',
      'Timestamp',
    ];

    let csv = headers.join(',') + '\n';

    for (const result of this.results) {
      const row = [
        result.adapter,
        result.browser,
        result.scenario,
        result.datasetSize,
        result.operationType,
        result.metrics.totalTime,
        result.metrics.avgOperationTime,
        result.metrics.throughput,
        result.metrics.bytesPerSecond || 0,
        result.metrics.memoryUsed || 0,
        result.metrics.errorRate || 0,
        result.timestamp,
      ];

      csv += row.join(',') + '\n';
    }

    return csv;
  }

  /**
   * Clear all results
   */
  clear(): void {
    this.results = [];
  }
}

/**
 * Get browser information
 */
export function getBrowserInfo(): {
  name: string;
  version: string;
  platform: string;
} {
  const ua = navigator.userAgent;
  let browserName = 'Unknown';
  let browserVersion = 'Unknown';

  if (ua.indexOf('Firefox') > -1) {
    browserName = 'Firefox';
    const match = ua.match(/Firefox\/(\d+)/);
    browserVersion = match ? match[1] : 'Unknown';
  } else if (ua.indexOf('Chrome') > -1) {
    browserName = 'Chrome';
    const match = ua.match(/Chrome\/(\d+)/);
    browserVersion = match ? match[1] : 'Unknown';
  } else if (ua.indexOf('Safari') > -1) {
    browserName = 'Safari';
    const match = ua.match(/Version\/(\d+)/);
    browserVersion = match ? match[1] : 'Unknown';
  } else if (ua.indexOf('Edge') > -1) {
    browserName = 'Edge';
    const match = ua.match(/Edge\/(\d+)/);
    browserVersion = match ? match[1] : 'Unknown';
  }

  return {
    name: browserName,
    version: browserVersion,
    platform: navigator.platform,
  };
}

/**
 * Get memory usage (if available)
 */
export async function getMemoryUsage(): Promise<number | undefined> {
  if ('memory' in performance && (performance as any).memory) {
    return (performance as any).memory.usedJSHeapSize;
  }

  if ('storage' in navigator && 'estimate' in navigator.storage) {
    const estimate = await navigator.storage.estimate();
    return estimate.usage;
  }

  return undefined;
}

/**
 * Measure operation performance
 */
export async function measurePerformance<T>(
  operation: () => Promise<T>,
  iterations: number = 1
): Promise<{
  result: T;
  totalTime: number;
  avgTime: number;
  minTime: number;
  maxTime: number;
}> {
  const times: number[] = [];
  let result!: T;

  for (let i = 0; i < iterations; i++) {
    const startTime = performance.now();
    result = await operation();
    const endTime = performance.now();
    times.push(endTime - startTime);
  }

  const totalTime = times.reduce((sum, t) => sum + t, 0);
  const avgTime = totalTime / iterations;
  const minTime = Math.min(...times);
  const maxTime = Math.max(...times);

  return {
    result,
    totalTime,
    avgTime,
    minTime,
    maxTime,
  };
}

/**
 * Create a global metrics collector instance
 */
export const globalMetrics = new MetricsCollector();
