/**
 * Node.js Benchmark Runner
 *
 * Runs benchmarks in Node.js environment
 */

import { BenchmarkHarness, formatResults } from './harness';
import type { BenchmarkMetrics } from '../../common/domain';
import fs from 'fs/promises';
import path from 'path';

// Import implementations
import { AutomergeTodoList } from '../../automerge-impl/src/todo-list';
import { YrsTodoList } from '../../yrs-impl/src/todo-list';
// Note: Loro and cr-sqlite may have different Node.js setup requirements

async function main() {
  const harness = new BenchmarkHarness();
  const allResults: BenchmarkMetrics[] = [];

  console.log('Starting Node.js benchmarks...\n');

  // Benchmark Automerge
  try {
    const automergeResults = await harness.runFullSuite(
      'Automerge',
      () => new AutomergeTodoList(),
      'Node.js'
    );
    allResults.push(...automergeResults);
  } catch (error) {
    console.error('Automerge benchmark failed:', error);
  }

  // Benchmark Yjs
  try {
    const yrsResults = await harness.runFullSuite(
      'Yjs',
      () => new YrsTodoList(),
      'Node.js'
    );
    allResults.push(...yrsResults);
  } catch (error) {
    console.error('Yjs benchmark failed:', error);
  }

  // Note: Add Loro and cr-sqlite when setup complete
  // const loroResults = await harness.runFullSuite(
  //   'Loro',
  //   () => new LoroTodoList(),
  //   'Node.js'
  // );
  // allResults.push(...loroResults);

  // Output results
  console.log(formatResults(allResults));

  // Save results to file
  const resultsDir = path.join(process.cwd(), '../results');
  await fs.mkdir(resultsDir, { recursive: true });

  const resultsFile = path.join(resultsDir, `node-${Date.now()}.json`);
  await fs.writeFile(resultsFile, JSON.stringify(allResults, null, 2));

  console.log(`\nResults saved to: ${resultsFile}`);
}

main().catch(console.error);
