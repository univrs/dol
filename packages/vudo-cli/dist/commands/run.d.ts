/**
 * vudo run - Execute a DOL WASM module
 */
interface RunOptions {
    debug?: boolean;
    memory?: string;
    maxMemory?: string;
    json?: boolean;
}
export declare function runCommand(file: string, func: string | undefined, args: string[], options: RunOptions): Promise<void>;
export {};
//# sourceMappingURL=run.d.ts.map