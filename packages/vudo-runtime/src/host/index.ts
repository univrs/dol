/**
 * Host Functions Module
 *
 * Exports all host function implementations for I/O, memory, time, effects, and debug operations.
 *
 * @module @vudo/runtime/host
 */

// Interfaces and base types
export {
  IWasmMemory,
  ILogger,
  ITimeProvider,
  ConsoleLogger,
  SystemTimeProvider,
  HostFunctionError,
} from './interfaces.js';
export type { IOConfig } from './io.js';
export type { MemoryOpsConfig } from './memory-ops.js';
export type { TimeConfig } from './time.js';

// I/O host functions
export {
  IOHostFunctions,
  createIOHostFunctions,
} from './io.js';

// Memory host functions
export {
  MemoryHostFunctions,
  createMemoryHostFunctions,
} from './memory-ops.js';

// Time host functions
export {
  TimeHostFunctions,
  createTimeHostFunctions,
} from './time.js';

// Effects host functions
export {
  EffectsSystem,
  createEffectsSystem,
  DefaultEffectHandler,
  type IEffectHandler,
} from './effects.js';

// Debug host functions
export {
  DebugSystem,
  createDebugSystem,
  DefaultDebugHandler,
  PanicError,
  type IDebugHandler,
} from './debug.js';

// ============================================================================
// PHASE 9: HOST FUNCTION REGISTRY (Aggregated Interface)
// ============================================================================

// Registry - aggregates all 22 host functions
export {
  HostFunctionRegistry,
  verifyImports,
  type ITimeProvider,
  type ILogger,
  type IMessageBroker,
  type IRandomProvider,
  type IEffectHandler,
} from './registry.js';

// Default provider implementations
export {
  DefaultTimeProvider,
  DefaultLogger,
  DefaultMessageBroker,
  DefaultRandomProvider,
  createDefaultProviders,
} from './providers.js';

// Re-export ABI types for convenience
export { LogLevel, ResultCode } from '../abi/types.js';

// Combined host function utilities
import { createEffectsSystem, type IEffectHandler } from './effects.js';
import { createDebugSystem, type IDebugHandler } from './debug.js';

/**
 * Create all host function implementations
 *
 * Convenience function to create both effects and debug systems
 * and return combined host functions.
 *
 * @param memory - WASM memory instance
 * @param handlers - Optional custom handlers
 * @returns Object with all host functions for WASM imports
 *
 * @example
 * ```typescript
 * const memory = new WebAssembly.Memory({ initial: 1 });
 * const hostFunctions = createAllHostFunctions(memory);
 *
 * const imports = {
 *   vudo: hostFunctions
 * };
 *
 * const instance = await WebAssembly.instantiate(wasmModule, imports);
 * ```
 */
export function createAllHostFunctions(
  memory: WebAssembly.Memory,
  handlers?: {
    effects?: IEffectHandler;
    debug?: IDebugHandler;
  }
): {
  vudo_emit_effect: (effectPtr: number, effectLen: number) => number;
  vudo_subscribe: (patternPtr: number, patternLen: number) => number;
  vudo_breakpoint: () => void;
  vudo_assert: (condition: number, msgPtr: number, msgLen: number) => void;
  vudo_panic: (msgPtr: number, msgLen: number) => never;
} {
  const effects = createEffectsSystem(memory, handlers?.effects);
  const debug = createDebugSystem(memory, handlers?.debug);

  return {
    ...effects.createHostFunctions(),
    ...debug.createHostFunctions(),
  };
}

/**
 * Host systems container
 *
 * Holds references to all host systems for easy management.
 */
export interface HostSystems {
  effects: ReturnType<typeof createEffectsSystem>;
  debug: ReturnType<typeof createDebugSystem>;
}

/**
 * Create all host systems
 *
 * Creates effects and debug systems and returns them in a container.
 *
 * @param memory - WASM memory instance
 * @param handlers - Optional custom handlers
 * @returns Container with all host systems
 *
 * @example
 * ```typescript
 * const memory = new WebAssembly.Memory({ initial: 1 });
 * const systems = createHostSystems(memory);
 *
 * // Access individual systems
 * systems.effects.emitEffect(...);
 * systems.debug.breakpoint();
 *
 * // Get host functions for WASM
 * const imports = {
 *   vudo: {
 *     ...systems.effects.createHostFunctions(),
 *     ...systems.debug.createHostFunctions(),
 *   }
 * };
 * ```
 */
export function createHostSystems(
  memory: WebAssembly.Memory,
  handlers?: {
    effects?: IEffectHandler;
    debug?: IDebugHandler;
  }
): HostSystems {
  return {
    effects: createEffectsSystem(memory, handlers?.effects),
    debug: createDebugSystem(memory, handlers?.debug),
  };
}
