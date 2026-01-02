/**
 * test-counter.ts - Vertical Slice Integration Test
 *
 * This test validates the complete DOL -> WASM -> @vudo/runtime pipeline.
 * Run with: npx tsx vertical-slice-results/test-counter.ts
 */

import { loadSpirit } from '../packages/vudo-runtime/src/index.js';
import { readFile } from 'fs/promises';

interface TestResult {
  name: string;
  expected: bigint | string;
  actual: bigint | string | Error;
  passed: boolean;
}

const results: TestResult[] = [];

function recordTest(name: string, expected: bigint | string, actual: bigint | string | Error) {
  const passed = actual === expected ||
    (typeof actual === 'bigint' && typeof expected === 'bigint' && actual === expected);
  results.push({ name, expected, actual, passed });

  const status = passed ? '\x1b[32mPASS\x1b[0m' : '\x1b[31mFAIL\x1b[0m';
  console.log(`  ${status} ${name}: expected ${expected}, got ${actual}`);
}

async function main() {
  console.log('=== DOL -> WASM -> @vudo/runtime Vertical Slice ===\n');

  // Step 1: Load compiled WASM
  console.log('Step 1: Loading counter.wasm...');
  let wasmBytes: Buffer;
  try {
    wasmBytes = await readFile('./vertical-slice-results/counter.wasm');
    console.log(`  Loaded ${wasmBytes.length} bytes`);
  } catch (e) {
    console.error(`  ERROR: Could not load counter.wasm: ${e}`);
    process.exit(1);
  }

  // Step 2: Create Spirit instance
  console.log('\nStep 2: Creating Spirit instance...');
  let spirit;
  try {
    spirit = await loadSpirit(wasmBytes, { debug: false });
    console.log('  Spirit loaded successfully');
    console.log('  Exports:', Object.keys(spirit.exports).filter(k => typeof spirit.exports[k] === 'function'));
  } catch (e) {
    console.error(`  ERROR: Could not load Spirit: ${e}`);
    process.exit(1);
  }

  // Step 3: Test add_numbers (standalone function, no gene)
  console.log('\nStep 3: Testing add_numbers(3, 4)...');
  try {
    const sum = spirit.call<bigint>('add_numbers', [3n, 4n]);
    recordTest('add_numbers(3, 4)', 7n, sum);
  } catch (e: any) {
    recordTest('add_numbers(3, 4)', 7n, e.message || String(e));
  }

  // Step 4: Allocate Counter gene memory
  console.log('\nStep 4: Allocating Counter gene memory...');
  const counterPtr = spirit.memory.alloc(8); // Int64 = 8 bytes
  // Write value using typed array view (i64 values at byte offset / 8)
  const i64View = new BigInt64Array(spirit.memory.buffer);
  i64View[counterPtr / 8] = 10n;   // value = 10
  console.log(`  Allocated at ptr: ${counterPtr}`);
  console.log(`  Initial value: 10`);

  // Step 5: Call Counter.increment (expects self pointer, returns value + 1)
  console.log('\nStep 5: Calling Counter.increment...');
  try {
    const result = spirit.call<bigint>('Counter.increment', [counterPtr]);
    recordTest('Counter.increment(ptr where value=10)', 11n, result);
  } catch (e: any) {
    recordTest('Counter.increment(ptr where value=10)', 11n, e.message || String(e));
  }

  // Step 6: Call Counter.get_value
  console.log('\nStep 6: Calling Counter.get_value...');
  try {
    const result = spirit.call<bigint>('Counter.get_value', [counterPtr]);
    recordTest('Counter.get_value(ptr where value=10)', 10n, result);
  } catch (e: any) {
    recordTest('Counter.get_value(ptr where value=10)', 10n, e.message || String(e));
  }

  // Step 7: Call Counter.add
  console.log('\nStep 7: Calling Counter.add(5)...');
  try {
    const result = spirit.call<bigint>('Counter.add', [counterPtr, 5n]);
    recordTest('Counter.add(ptr where value=10, 5)', 15n, result);
  } catch (e: any) {
    recordTest('Counter.add(ptr where value=10, 5)', 15n, e.message || String(e));
  }

  // Summary
  console.log('\n=== Vertical Slice Summary ===\n');

  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;

  console.log(`Results: ${passed} passed, ${failed} failed\n`);

  for (const r of results) {
    const icon = r.passed ? '✅' : '❌';
    console.log(`${icon} ${r.name}`);
  }

  if (failed > 0) {
    console.log('\n\x1b[31mVERTICAL SLICE FAILED\x1b[0m');
    console.log('Some tests did not pass. Review the output above.');
    process.exit(1);
  } else {
    console.log('\n\x1b[32mVERTICAL SLICE PASSED\x1b[0m');
    console.log('All tests passed! The DOL -> WASM -> @vudo/runtime pipeline works.');
    process.exit(0);
  }
}

main().catch(e => {
  console.error('Unexpected error:', e);
  process.exit(1);
});
