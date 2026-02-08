/**
 * Results Analysis Tool
 *
 * Analyzes benchmark results and generates comparison tables
 */

import fs from 'fs/promises';
import path from 'path';
import type { BenchmarkMetrics } from '../../common/domain';

interface ComparisonRow {
  library: string;
  scenario: string;
  totalTime: number;
  mergeTime: number;
  opsPerSecond: number;
  serializedSize: number;
  convergence: string;
}

async function loadResults(resultsDir: string): Promise<BenchmarkMetrics[]> {
  const files = await fs.readdir(resultsDir);
  const jsonFiles = files.filter(f => f.endsWith('.json'));

  const allResults: BenchmarkMetrics[] = [];

  for (const file of jsonFiles) {
    const content = await fs.readFile(path.join(resultsDir, file), 'utf-8');
    const results = JSON.parse(content) as BenchmarkMetrics[];
    allResults.push(...results);
  }

  return allResults;
}

function generateComparisonTable(results: BenchmarkMetrics[]): string {
  let output = '\n';
  output += '='.repeat(100) + '\n';
  output += 'CRDT BENCHMARK COMPARISON\n';
  output += '='.repeat(100) + '\n\n';

  // Group by scenario
  const byScenario = new Map<string, BenchmarkMetrics[]>();
  for (const result of results) {
    if (!byScenario.has(result.scenario)) {
      byScenario.set(result.scenario, []);
    }
    byScenario.get(result.scenario)!.push(result);
  }

  for (const [scenario, metrics] of byScenario) {
    output += `\n## ${scenario}\n\n`;
    output += '| Library | Total Time | Merge Time | Ops/Sec | Size (KB) | Convergence |\n';
    output += '|---------|------------|------------|---------|-----------|-------------|\n';

    // Sort by merge time (fastest first)
    const sorted = metrics.sort((a, b) => a.mergeTime - b.mergeTime);

    for (const m of sorted) {
      const sizeKB = (m.serializedSize / 1024).toFixed(2);
      const conv = m.convergenceCorrect ? '✅' : '❌';
      output += `| ${m.library.padEnd(12)} | ${m.totalTime.toFixed(2).padStart(8)}ms | ${m.mergeTime.toFixed(2).padStart(8)}ms | ${m.opsPerSecond.toFixed(0).padStart(8)} | ${sizeKB.padStart(8)} | ${conv} |\n`;
    }
  }

  return output;
}

function generateSummary(results: BenchmarkMetrics[]): string {
  let output = '\n';
  output += '='.repeat(100) + '\n';
  output += 'EXECUTIVE SUMMARY\n';
  output += '='.repeat(100) + '\n\n';

  // Find 10K scenario results (most representative)
  const tenK = results.filter(r => r.scenario.includes('10K'));

  if (tenK.length === 0) {
    return output + 'No 10K benchmark results found.\n';
  }

  // Fastest merge
  const fastestMerge = tenK.reduce((a, b) => a.mergeTime < b.mergeTime ? a : b);
  output += `🏆 Fastest Merge: ${fastestMerge.library} (${fastestMerge.mergeTime.toFixed(2)}ms for 10K ops)\n`;

  // Highest throughput
  const highestOps = tenK.reduce((a, b) => a.opsPerSecond > b.opsPerSecond ? a : b);
  output += `🏆 Highest Throughput: ${highestOps.library} (${highestOps.opsPerSecond.toFixed(0)} ops/sec)\n`;

  // Smallest size
  const smallest = tenK.reduce((a, b) => a.serializedSize < b.serializedSize ? a : b);
  output += `🏆 Smallest Serialized Size: ${smallest.library} (${(smallest.serializedSize / 1024).toFixed(2)}KB for 10K todos)\n`;

  // Average by library
  output += '\n### Average Performance (10K operations)\n\n';
  output += '| Library | Avg Merge Time | Avg Throughput | Avg Size |\n';
  output += '|---------|----------------|----------------|----------|\n';

  const byLibrary = new Map<string, BenchmarkMetrics[]>();
  for (const result of tenK) {
    if (!byLibrary.has(result.library)) {
      byLibrary.set(result.library, []);
    }
    byLibrary.get(result.library)!.push(result);
  }

  for (const [library, metrics] of byLibrary) {
    const avgMerge = metrics.reduce((sum, m) => sum + m.mergeTime, 0) / metrics.length;
    const avgOps = metrics.reduce((sum, m) => sum + m.opsPerSecond, 0) / metrics.length;
    const avgSize = metrics.reduce((sum, m) => sum + m.serializedSize, 0) / metrics.length;

    output += `| ${library.padEnd(12)} | ${avgMerge.toFixed(2).padStart(12)}ms | ${avgOps.toFixed(0).padStart(14)} | ${(avgSize / 1024).toFixed(2).padStart(8)}KB |\n`;
  }

  return output;
}

async function main() {
  const resultsDir = path.join(process.cwd(), '../results');

  try {
    const results = await loadResults(resultsDir);

    if (results.length === 0) {
      console.log('No benchmark results found.');
      console.log(`Expected location: ${resultsDir}`);
      console.log('\nRun benchmarks first:');
      console.log('  pnpm benchmark:node');
      console.log('  pnpm benchmark:browser');
      return;
    }

    console.log(`Loaded ${results.length} benchmark results from ${resultsDir}`);

    const summary = generateSummary(results);
    const comparison = generateComparisonTable(results);

    console.log(summary);
    console.log(comparison);

    // Save to file
    const reportPath = path.join(resultsDir, 'analysis.md');
    await fs.writeFile(reportPath, summary + '\n' + comparison);
    console.log(`\n✅ Analysis saved to: ${reportPath}`);

  } catch (error) {
    console.error('Error analyzing results:', error);
    process.exit(1);
  }
}

main().catch(console.error);
