/**
 * Spirit - WASM module loader and wrapper
 *
 * A Spirit is a compiled DOL program running in WASM.
 */

import type {
  LoadOptions,
  SpiritInstance,
  MemoryManager,
  LoaContext,
  LoaRegistry as ILoaRegistry,
} from './types.js';
import { SpiritMemoryManager } from './memory.js';
import { LoaRegistry } from './loa.js';

// ============================================================================
// Spirit Class
// ============================================================================

/**
 * Spirit instance wrapping a loaded WASM module
 */
export class Spirit implements SpiritInstance {
  private instance: WebAssembly.Instance;
  private wasmMemory: WebAssembly.Memory;
  private memoryManager: SpiritMemoryManager;
  private debug: boolean;

  constructor(
    instance: WebAssembly.Instance,
    memory: WebAssembly.Memory,
    debug = false
  ) {
    this.instance = instance;
    this.wasmMemory = memory;
    this.memoryManager = new SpiritMemoryManager(memory);
    this.debug = debug;
  }

  /**
   * Call an exported function by name
   */
  call<R = unknown>(name: string, args: unknown[] = []): R {
    const func = this.instance.exports[name];

    if (typeof func !== 'function') {
      throw new Error(`Function '${name}' not found in Spirit exports`);
    }

    if (this.debug) {
      console.log(`[Spirit] Calling ${name}(${args.join(', ')})`);
    }

    const result = (func as (...a: unknown[]) => unknown)(...args);

    if (this.debug) {
      console.log(`[Spirit] ${name} returned:`, result);
    }

    return result as R;
  }

  /**
   * Get a typed interface to the Spirit
   *
   * Allows type-safe calls using generated TypeScript types:
   * ```typescript
   * import type { Calculator } from './generated/calculator.types';
   * const calc = spirit.as<Calculator>();
   * const sum = calc.add(1n, 2n);
   * ```
   */
  as<T extends object>(): T {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this;

    return new Proxy({} as T, {
      get(_target, prop: string) {
        return (...args: unknown[]) => self.call(prop, args);
      },
    }) as T;
  }

  /**
   * Get the memory manager for this Spirit
   */
  get memory(): MemoryManager {
    return this.memoryManager;
  }

  /**
   * Get raw WASM exports
   */
  get exports(): WebAssembly.Exports {
    return this.instance.exports;
  }

  /**
   * Get raw WASM memory
   */
  get rawMemory(): WebAssembly.Memory {
    return this.wasmMemory;
  }

  /**
   * Check if the Spirit exports a function
   */
  hasFunction(name: string): boolean {
    return typeof this.instance.exports[name] === 'function';
  }

  /**
   * List all exported function names
   */
  listFunctions(): string[] {
    return Object.entries(this.instance.exports)
      .filter(([_, v]) => typeof v === 'function')
      .map(([k]) => k);
  }
}

// ============================================================================
// Spirit Loader
// ============================================================================

/**
 * Loader for Spirit WASM modules
 */
export class SpiritLoader {
  private registry: ILoaRegistry;
  private debug: boolean;

  constructor(options: { loas?: ILoaRegistry; debug?: boolean } = {}) {
    this.registry = options.loas ?? new LoaRegistry();
    this.debug = options.debug ?? false;
  }

  /**
   * Load a Spirit from WASM bytes
   */
  async load(
    wasmBytes: ArrayBuffer | Uint8Array,
    options: LoadOptions = {}
  ): Promise<Spirit> {
    const debug = options.debug ?? this.debug;

    // Create WASM memory
    const memory = new WebAssembly.Memory({
      initial: options.memory?.initial ?? 16, // 1MB default
      maximum: options.memory?.maximum ?? 256, // 16MB max
    });

    // Create temporary allocator for building imports
    const tempMemoryManager = new SpiritMemoryManager(memory);

    // Build Loa context
    const loaContext: LoaContext = {
      memory,
      alloc: (size: number) => tempMemoryManager.alloc(size),
      debug,
    };

    // Get registry (use options or default)
    const registry = options.loas ?? this.registry;

    // Build imports from Loas
    const loaImports = registry.buildImports(loaContext);

    // Merge with custom imports (custom takes precedence)
    const imports: WebAssembly.Imports = {
      env: {
        memory,
        ...loaImports,
        ...options.imports,
      },
    };

    if (debug) {
      console.log('[SpiritLoader] Loading WASM module...');
      console.log('[SpiritLoader] Available imports:', Object.keys(imports.env));
    }

    // Compile and instantiate
    const bytes = wasmBytes instanceof ArrayBuffer
      ? wasmBytes
      : new Uint8Array(wasmBytes).buffer;
    const module = await WebAssembly.compile(bytes);
    const instance = await WebAssembly.instantiate(module, imports);

    // Use the module's exported memory if available, otherwise fall back to imported memory
    const exportedMemory = instance.exports.memory as WebAssembly.Memory | undefined;
    const actualMemory = exportedMemory ?? memory;

    if (debug) {
      console.log('[SpiritLoader] Spirit loaded successfully');
      if (exportedMemory) {
        console.log('[SpiritLoader] Using module-exported memory');
      } else {
        console.log('[SpiritLoader] Using imported memory');
      }
      const spirit = new Spirit(instance, actualMemory, debug);
      console.log('[SpiritLoader] Exports:', spirit.listFunctions());
    }

    return new Spirit(instance, actualMemory, debug);
  }

  /**
   * Load a Spirit from a URL (browser) or file path (Node.js)
   */
  async loadFrom(source: string | URL, options: LoadOptions = {}): Promise<Spirit> {
    const bytes = await this.fetchBytes(source);
    return this.load(bytes, options);
  }

  /**
   * Fetch WASM bytes from URL or file
   */
  private async fetchBytes(source: string | URL): Promise<Uint8Array> {
    if (typeof globalThis.fetch !== 'undefined') {
      // Browser or Node.js 18+
      const response = await fetch(source);
      if (!response.ok) {
        throw new Error(`Failed to fetch Spirit: ${response.statusText}`);
      }
      return new Uint8Array(await response.arrayBuffer());
    } else {
      // Fallback for older Node.js
      const fs = await import('fs/promises');
      return fs.readFile(source as string);
    }
  }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Load a Spirit from WASM bytes or URL
 *
 * @example
 * ```typescript
 * // From bytes
 * const spirit = await loadSpirit(wasmBytes);
 *
 * // From URL/path
 * const spirit = await loadSpirit('/spirits/calculator.wasm');
 *
 * // With options
 * const spirit = await loadSpirit(wasmBytes, {
 *   debug: true,
 *   memory: { initial: 32 },
 * });
 * ```
 */
export async function loadSpirit(
  source: string | URL | ArrayBuffer | Uint8Array,
  options: LoadOptions = {}
): Promise<Spirit> {
  const loader = new SpiritLoader({
    loas: options.loas,
    debug: options.debug,
  });

  if (source instanceof ArrayBuffer || source instanceof Uint8Array) {
    return loader.load(source, options);
  }

  return loader.loadFrom(source, options);
}
