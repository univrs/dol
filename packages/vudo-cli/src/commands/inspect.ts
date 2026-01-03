/**
 * vudo inspect - Inspect a DOL WASM module
 */

import { readFile } from 'fs/promises';
import { resolve } from 'path';
import chalk from 'chalk';

interface InspectOptions {
  exports?: boolean;
  imports?: boolean;
  memory?: boolean;
  json?: boolean;
}

interface ModuleInfo {
  exports: ExportInfo[];
  imports: ImportInfo[];
  memory: MemoryInfo | null;
}

interface ExportInfo {
  name: string;
  kind: string;
}

interface ImportInfo {
  module: string;
  name: string;
  kind: string;
}

interface MemoryInfo {
  initial: number;
  maximum?: number;
}

/**
 * Parse WASM module sections for introspection
 */
async function parseWasmModule(wasmBytes: Uint8Array): Promise<ModuleInfo> {
  const bytes = new Uint8Array(wasmBytes).buffer;
  const module = await WebAssembly.compile(bytes);

  // Get exports
  const exports: ExportInfo[] = WebAssembly.Module.exports(module).map((exp) => ({
    name: exp.name,
    kind: exp.kind,
  }));

  // Get imports
  const imports: ImportInfo[] = WebAssembly.Module.imports(module).map((imp) => ({
    module: imp.module,
    name: imp.name,
    kind: imp.kind,
  }));

  // Find memory info from imports or exports
  let memory: MemoryInfo | null = null;

  const memExport = exports.find((e) => e.kind === 'memory');
  const memImport = imports.find((i) => i.kind === 'memory');

  if (memExport || memImport) {
    // We can't easily get the initial/max pages from the WebAssembly API
    // without instantiating, so we provide basic info
    memory = { initial: 0 }; // Placeholder
  }

  return { exports, imports, memory };
}

/**
 * Format export info for display
 */
function formatExport(exp: ExportInfo): string {
  const kindColor = {
    function: chalk.green,
    memory: chalk.blue,
    table: chalk.magenta,
    global: chalk.yellow,
  }[exp.kind] ?? chalk.white;

  return `  ${kindColor(exp.kind.padEnd(10))} ${exp.name}`;
}

/**
 * Format import info for display
 */
function formatImport(imp: ImportInfo): string {
  const kindColor = {
    function: chalk.green,
    memory: chalk.blue,
    table: chalk.magenta,
    global: chalk.yellow,
  }[imp.kind] ?? chalk.white;

  return `  ${kindColor(imp.kind.padEnd(10))} ${imp.module}.${imp.name}`;
}

export async function inspectCommand(
  file: string,
  options: InspectOptions
): Promise<void> {
  try {
    // Resolve file path
    const filePath = resolve(process.cwd(), file);

    // Read WASM bytes
    const wasmBytes = await readFile(filePath);

    // Parse module
    const info = await parseWasmModule(new Uint8Array(wasmBytes));

    // JSON output
    if (options.json) {
      const output: Partial<ModuleInfo> = {};
      if (!options.imports && !options.memory) output.exports = info.exports;
      if (options.imports || (!options.exports && !options.memory)) output.imports = info.imports;
      if (options.memory || (!options.exports && !options.imports)) output.memory = info.memory;
      if (!options.exports && !options.imports && !options.memory) {
        console.log(JSON.stringify(info, null, 2));
      } else {
        console.log(JSON.stringify(output, null, 2));
      }
      return;
    }

    // Text output
    console.log(chalk.bold(`\nModule: ${file}\n`));

    // Show exports
    if (!options.imports && !options.memory) {
      const functions = info.exports.filter((e) => e.kind === 'function');
      const others = info.exports.filter((e) => e.kind !== 'function');

      console.log(chalk.bold.underline('Exports:'));

      if (functions.length > 0) {
        console.log(chalk.dim('\n  Functions:'));
        functions.forEach((exp) => console.log(formatExport(exp)));
      }

      if (others.length > 0) {
        console.log(chalk.dim('\n  Other:'));
        others.forEach((exp) => console.log(formatExport(exp)));
      }

      if (info.exports.length === 0) {
        console.log(chalk.dim('  (none)'));
      }
      console.log();
    }

    // Show imports
    if (options.imports || (!options.exports && !options.memory)) {
      console.log(chalk.bold.underline('Imports:'));

      if (info.imports.length > 0) {
        // Group by module
        const byModule = new Map<string, ImportInfo[]>();
        info.imports.forEach((imp) => {
          const list = byModule.get(imp.module) ?? [];
          list.push(imp);
          byModule.set(imp.module, list);
        });

        byModule.forEach((imps, mod) => {
          console.log(chalk.dim(`\n  ${mod}:`));
          imps.forEach((imp) => {
            const kindColor = {
              function: chalk.green,
              memory: chalk.blue,
            }[imp.kind] ?? chalk.white;
            console.log(`    ${kindColor(imp.kind.padEnd(10))} ${imp.name}`);
          });
        });
      } else {
        console.log(chalk.dim('  (none)'));
      }
      console.log();
    }

    // Show memory
    if (options.memory || (!options.exports && !options.imports)) {
      console.log(chalk.bold.underline('Memory:'));
      if (info.memory) {
        console.log(`  Initial: ${info.memory.initial} pages`);
        if (info.memory.maximum !== undefined) {
          console.log(`  Maximum: ${info.memory.maximum} pages`);
        }
      } else {
        console.log(chalk.dim('  (no memory section)'));
      }
      console.log();
    }

  } catch (error) {
    if (error instanceof Error) {
      console.error(chalk.red(`Error: ${error.message}`));
    } else {
      console.error(chalk.red('Unknown error occurred'));
    }
    process.exit(1);
  }
}
