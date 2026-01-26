/* tslint:disable */
/* eslint-disable */

export function get_alive_count(): bigint;

export function get_cells(): Uint8Array;

export function get_generation(): bigint;

export function get_height(): number;

export function get_width(): number;

export function init(width: number, height: number, wrap: boolean): void;

export function load_pattern(name: string, ox: number, oy: number): boolean;

export function randomize_grid(density: number): void;

export function reset(): void;

export function set_cell_state(x: number, y: number, alive: boolean): void;

export function step(): bigint;

export function toggle_cell(x: number, y: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly init: (a: number, b: number, c: number) => void;
    readonly reset: () => void;
    readonly step: () => bigint;
    readonly set_cell_state: (a: number, b: number, c: number) => void;
    readonly toggle_cell: (a: number, b: number) => void;
    readonly load_pattern: (a: number, b: number, c: number, d: number) => number;
    readonly randomize_grid: (a: number) => void;
    readonly get_generation: () => bigint;
    readonly get_width: () => number;
    readonly get_height: () => number;
    readonly get_alive_count: () => bigint;
    readonly get_cells: () => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
