/**
 * vudo run - Execute a DOL WASM module
 */

import { readFile } from 'fs/promises';
import { resolve } from 'path';
import chalk from 'chalk';
import { loadSpirit } from '@vudo/runtime';

interface RunOptions {
  debug?: boolean;
  memory?: string;
  maxMemory?: string;
  json?: boolean;
}

/**
 * Parse a CLI argument into a typed value
 */
function parseArg(arg: string): unknown {
  // BigInt (ends with 'n')
  if (/^-?\d+n$/.test(arg)) {
    return BigInt(arg.slice(0, -1));
  }

  // Integer
  if (/^-?\d+$/.test(arg)) {
    const num = parseInt(arg, 10);
    // Use BigInt for large numbers (WASM i64)
    if (num > Number.MAX_SAFE_INTEGER || num < Number.MIN_SAFE_INTEGER) {
      return BigInt(arg);
    }
    return num;
  }

  // Float
  if (/^-?\d+\.\d+$/.test(arg)) {
    return parseFloat(arg);
  }

  // Boolean
  if (arg === 'true') return true;
  if (arg === 'false') return false;

  // String (default)
  return arg;
}

/**
 * Format a result for display
 */
function formatResult(result: unknown, json: boolean): string {
  if (json) {
    return JSON.stringify(result, (_, v) =>
      typeof v === 'bigint' ? v.toString() + 'n' : v
    );
  }

  if (typeof result === 'bigint') {
    return `${result}n`;
  }

  if (result === undefined || result === null) {
    return chalk.dim('(void)');
  }

  return String(result);
}

export async function runCommand(
  file: string,
  func: string | undefined,
  args: string[],
  options: RunOptions
): Promise<void> {
  const functionName = func ?? 'main';
  const debug = options.debug ?? false;
  const initialMemory = parseInt(options.memory ?? '16', 10);
  const maxMemory = parseInt(options.maxMemory ?? '256', 10);

  try {
    // Resolve file path
    const filePath = resolve(process.cwd(), file);

    if (debug) {
      console.log(chalk.dim(`Loading: ${filePath}`));
    }

    // Read WASM bytes
    const wasmBytes = await readFile(filePath);

    // Load Spirit
    const spirit = await loadSpirit(wasmBytes, {
      debug,
      memory: {
        initial: initialMemory,
        maximum: maxMemory,
      },
    });

    // Check if function exists
    if (!spirit.hasFunction(functionName)) {
      const available = spirit.listFunctions();
      console.error(chalk.red(`Error: Function '${functionName}' not found`));
      console.error(chalk.dim(`Available functions: ${available.join(', ')}`));
      process.exit(1);
    }

    // Parse arguments
    const parsedArgs = args.map(parseArg);

    if (debug) {
      console.log(chalk.dim(`Calling: ${functionName}(${parsedArgs.map(a =>
        typeof a === 'bigint' ? `${a}n` : JSON.stringify(a)
      ).join(', ')})`));
    }

    // Call function
    const result = spirit.call(functionName, parsedArgs);

    // Output result
    console.log(formatResult(result, options.json ?? false));

  } catch (error) {
    if (error instanceof Error) {
      console.error(chalk.red(`Error: ${error.message}`));
      if (debug && error.stack) {
        console.error(chalk.dim(error.stack));
      }
    } else {
      console.error(chalk.red('Unknown error occurred'));
    }
    process.exit(1);
  }
}
