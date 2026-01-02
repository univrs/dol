/**
 * @vudo/runtime - JavaScript/TypeScript runtime for DOL Spirits
 *
 * Host environment for DOL programs compiled to WebAssembly.
 *
 * @packageDocumentation
 *
 * @example
 * ```typescript
 * import { loadSpirit, Seance } from '@vudo/runtime';
 *
 * // Load a single Spirit
 * const spirit = await loadSpirit('./calculator.wasm');
 * const result = spirit.call('add', [1, 2]);
 *
 * // Multi-Spirit session
 * const seance = new Seance();
 * await seance.summon('calc', './calculator.wasm');
 * await seance.invoke('calc', 'multiply', [6, 7]);
 * await seance.dismiss();
 * ```
 */

// ============================================================================
// Core Types
// ============================================================================

export type {
  // Gene types
  GeneFieldType,
  GeneField,
  GeneLayout,
  GeneValues,

  // Spirit types
  LoadOptions,
  SpiritInstance,

  // Loa types
  Loa,
  LoaContext,
  // LoaRegistry - exported as class from loa.js

  // Séance types
  SeanceInstance,

  // Memory types
  MemoryManager,
} from './types.js';

// ============================================================================
// Spirit Loading
// ============================================================================

export { Spirit, SpiritLoader, loadSpirit } from './spirit.js';

// ============================================================================
// Session Management
// ============================================================================

export { Seance, createSeance, withSeance } from './seance.js';

// ============================================================================
// Loa (Services)
// ============================================================================

export {
  coreLoa,
  LoaRegistry,
  createLoa,
  createLoggingLoa,
} from './loa.js';

// ============================================================================
// Memory Management
// ============================================================================

export { SpiritMemoryManager, BumpAllocator } from './memory.js';

// ============================================================================
// Utilities
// ============================================================================

export {
  // String encoding
  encodeString,
  decodeString,

  // Gene operations
  readGene,
  writeGene,

  // Layout helpers
  calculateLayout,
  getTypeSize,
  getTypeAlignment,
} from './utils/type-bridge.js';
