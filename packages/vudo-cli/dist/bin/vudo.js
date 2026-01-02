#!/usr/bin/env node
/**
 * VUDO CLI - Command line tool for running DOL WASM modules
 */
import { Command } from 'commander';
import { runCommand } from '../commands/run.js';
import { inspectCommand } from '../commands/inspect.js';
import { version } from '../version.js';
const program = new Command();
program
    .name('vudo')
    .description('CLI tool for running DOL WASM modules')
    .version(version);
// vudo run <file.wasm> [function] [args...]
program
    .command('run')
    .description('Execute a DOL WASM module')
    .argument('<file>', 'WASM file to run')
    .argument('[function]', 'Function to call (default: main)')
    .argument('[args...]', 'Arguments to pass to the function')
    .option('-d, --debug', 'Enable debug output')
    .option('-m, --memory <pages>', 'Initial memory pages (default: 16)', '16')
    .option('-M, --max-memory <pages>', 'Maximum memory pages (default: 256)', '256')
    .option('--json', 'Output result as JSON')
    .action(runCommand);
// vudo inspect <file.wasm>
program
    .command('inspect')
    .description('Inspect a DOL WASM module')
    .argument('<file>', 'WASM file to inspect')
    .option('--exports', 'List exported functions only')
    .option('--imports', 'List required imports only')
    .option('--memory', 'Show memory configuration')
    .option('--json', 'Output as JSON')
    .action(inspectCommand);
program.parse();
//# sourceMappingURL=vudo.js.map