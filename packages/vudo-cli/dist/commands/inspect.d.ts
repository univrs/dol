/**
 * vudo inspect - Inspect a DOL WASM module
 */
interface InspectOptions {
    exports?: boolean;
    imports?: boolean;
    memory?: boolean;
    json?: boolean;
}
export declare function inspectCommand(file: string, options: InspectOptions): Promise<void>;
export {};
//# sourceMappingURL=inspect.d.ts.map