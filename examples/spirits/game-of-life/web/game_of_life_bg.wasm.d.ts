/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const init: (a: number, b: number, c: number) => void;
export const reset: () => void;
export const step: () => bigint;
export const set_cell_state: (a: number, b: number, c: number) => void;
export const toggle_cell: (a: number, b: number) => void;
export const load_pattern: (a: number, b: number, c: number, d: number) => number;
export const randomize_grid: (a: number) => void;
export const get_generation: () => bigint;
export const get_width: () => number;
export const get_height: () => number;
export const get_alive_count: () => bigint;
export const get_cells: () => [number, number];
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_start: () => void;
