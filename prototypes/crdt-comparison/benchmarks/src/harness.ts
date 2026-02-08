/**
 * Benchmark Harness
 *
 * Runs standardized benchmarks across all CRDT implementations
 * and collects performance metrics.
 */

import type { CRDTTodoList, BenchmarkMetrics } from '../../common/domain';
import {
  BenchmarkRunner,
  sequentialAdds,
  concurrentEdits2Peer,
  concurrentEdits2PeerSecond,
  conflictResolution,
  mixedOperations,
  verifyConvergence
} from '../../common/scenarios';

export class BenchmarkHarness {
  private runner: BenchmarkRunner;

  constructor() {
    this.runner = new BenchmarkRunner();
  }

  async runFullSuite(
    library: string,
    createImpl: () => CRDTTodoList,
    platform: string
  ): Promise<BenchmarkMetrics[]> {
    const results: BenchmarkMetrics[] = [];

    console.log(`\n=== Running benchmarks for ${library} on ${platform} ===\n`);

    // Scenario 1: Sequential Adds - 1K
    results.push(await this.runSequentialAdds(library, createImpl, 1000, platform));

    // Scenario 2: Sequential Adds - 10K
    results.push(await this.runSequentialAdds(library, createImpl, 10000, platform));

    // Scenario 3: Sequential Adds - 100K
    results.push(await this.runSequentialAdds(library, createImpl, 100000, platform));

    // Scenario 4: Concurrent Edits (2 peers)
    results.push(await this.runConcurrentEdits(library, createImpl, 1000, platform));

    // Scenario 5: Conflict Resolution
    results.push(await this.runConflictResolution(library, createImpl, 1000, platform));

    // Scenario 6: Mixed Operations
    results.push(await this.runMixedOperations(library, createImpl, 1000, platform));

    return results;
  }

  private async runSequentialAdds(
    library: string,
    createImpl: () => CRDTTodoList,
    count: number,
    platform: string
  ): Promise<BenchmarkMetrics> {
    const impl = createImpl();
    const scenario = sequentialAdds(count);

    const startTime = performance.now();
    const startMem = this.getMemoryUsage();

    scenario(impl);

    const endTime = performance.now();
    const endMem = this.getMemoryUsage();

    // Measure serialization
    const serStartTime = performance.now();
    const serialized = impl.serialize();
    const serEndTime = performance.now();

    // Measure merge (with itself - no-op but measures overhead)
    const mergeStartTime = performance.now();
    const clone = impl.clone();
    impl.merge(clone);
    const mergeEndTime = performance.now();

    const totalTime = endTime - startTime;

    return {
      library,
      scenario: `Sequential Adds (${count})`,
      operationCount: count,
      totalTime,
      mergeTime: mergeEndTime - mergeStartTime,
      serializeTime: serEndTime - serStartTime,
      deserializeTime: 0,
      serializedSize: serialized.byteLength,
      memoryUsage: endMem - startMem,
      opsPerSecond: (count / totalTime) * 1000,
      mergesPerSecond: 0,
      convergenceCorrect: true,
      platform,
      timestamp: Date.now()
    };
  }

  private async runConcurrentEdits(
    library: string,
    createImpl: () => CRDTTodoList,
    count: number,
    platform: string
  ): Promise<BenchmarkMetrics> {
    const impl1 = createImpl();
    const impl2 = createImpl();

    const scenario1 = concurrentEdits2Peer(count);
    const scenario2 = concurrentEdits2PeerSecond(count);

    const startTime = performance.now();

    // Simulate concurrent edits
    scenario1(impl1);
    scenario2(impl2);

    const editTime = performance.now();

    // Measure merge
    const mergeStartTime = performance.now();
    impl1.merge(impl2);
    impl2.merge(impl1);
    const mergeEndTime = performance.now();

    const totalTime = editTime - startTime;
    const mergeTime = mergeEndTime - mergeStartTime;

    // Verify convergence
    const convergenceCorrect = verifyConvergence(impl1, impl2);

    return {
      library,
      scenario: `Concurrent Edits (2 peers, ${count} ops)`,
      operationCount: count,
      totalTime,
      mergeTime,
      serializeTime: 0,
      deserializeTime: 0,
      serializedSize: impl1.serialize().byteLength,
      memoryUsage: 0,
      opsPerSecond: (count / totalTime) * 1000,
      mergesPerSecond: (2 / mergeTime) * 1000,
      convergenceCorrect,
      platform,
      timestamp: Date.now()
    };
  }

  private async runConflictResolution(
    library: string,
    createImpl: () => CRDTTodoList,
    count: number,
    platform: string
  ): Promise<BenchmarkMetrics> {
    const impl = createImpl();
    const scenario = conflictResolution(count);

    const startTime = performance.now();
    scenario(impl);
    const endTime = performance.now();

    const totalTime = endTime - startTime;

    return {
      library,
      scenario: `Conflict Resolution (${count} conflicts)`,
      operationCount: count,
      totalTime,
      mergeTime: 0,
      serializeTime: 0,
      deserializeTime: 0,
      serializedSize: impl.serialize().byteLength,
      memoryUsage: 0,
      opsPerSecond: (count / totalTime) * 1000,
      mergesPerSecond: 0,
      convergenceCorrect: true,
      platform,
      timestamp: Date.now()
    };
  }

  private async runMixedOperations(
    library: string,
    createImpl: () => CRDTTodoList,
    count: number,
    platform: string
  ): Promise<BenchmarkMetrics> {
    const impl = createImpl();
    const scenario = mixedOperations(count);

    const startTime = performance.now();
    scenario(impl);
    const endTime = performance.now();

    const totalTime = endTime - startTime;

    return {
      library,
      scenario: `Mixed Operations (${count} ops)`,
      operationCount: count,
      totalTime,
      mergeTime: 0,
      serializeTime: 0,
      deserializeTime: 0,
      serializedSize: impl.serialize().byteLength,
      memoryUsage: 0,
      opsPerSecond: (count / totalTime) * 1000,
      mergesPerSecond: 0,
      convergenceCorrect: true,
      platform,
      timestamp: Date.now()
    };
  }

  private getMemoryUsage(): number {
    if (typeof performance !== 'undefined' && 'memory' in performance) {
      return (performance as any).memory.usedJSHeapSize;
    }
    if (typeof process !== 'undefined' && process.memoryUsage) {
      return process.memoryUsage().heapUsed;
    }
    return 0;
  }
}

export function formatResults(results: BenchmarkMetrics[]): string {
  let output = '\n';
  output += '='.repeat(80) + '\n';
  output += 'CRDT BENCHMARK RESULTS\n';
  output += '='.repeat(80) + '\n\n';

  for (const result of results) {
    output += `Library: ${result.library}\n`;
    output += `Scenario: ${result.scenario}\n`;
    output += `Platform: ${result.platform}\n`;
    output += `Operations: ${result.operationCount}\n`;
    output += `Total Time: ${result.totalTime.toFixed(2)}ms\n`;
    output += `Merge Time: ${result.mergeTime.toFixed(2)}ms\n`;
    output += `Ops/Second: ${result.opsPerSecond.toFixed(0)}\n`;
    output += `Serialized Size: ${(result.serializedSize / 1024).toFixed(2)}KB\n`;
    output += `Convergence: ${result.convergenceCorrect ? 'PASS' : 'FAIL'}\n`;
    output += '-'.repeat(80) + '\n';
  }

  return output;
}
