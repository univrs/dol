"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/index.ts
var index_exports = {};
__export(index_exports, {
  BumpAllocator: () => BumpAllocator,
  LoaRegistry: () => LoaRegistry,
  Seance: () => Seance,
  Spirit: () => Spirit,
  SpiritLoader: () => SpiritLoader,
  SpiritMemoryManager: () => SpiritMemoryManager,
  calculateLayout: () => calculateLayout,
  coreLoa: () => coreLoa,
  createLoa: () => createLoa,
  createLoggingLoa: () => createLoggingLoa,
  createSeance: () => createSeance,
  decodeString: () => decodeString,
  encodeString: () => encodeString,
  getTypeAlignment: () => getTypeAlignment,
  getTypeSize: () => getTypeSize,
  loadSpirit: () => loadSpirit,
  readGene: () => readGene,
  withSeance: () => withSeance,
  writeGene: () => writeGene
});
module.exports = __toCommonJS(index_exports);

// src/utils/type-bridge.ts
function encodeString(memory, allocator, str) {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(str);
  const len = bytes.length;
  const ptr = allocator.alloc(4 + len, 4);
  const view = new DataView(memory.buffer);
  view.setUint32(ptr, len, true);
  new Uint8Array(memory.buffer, ptr + 4, len).set(bytes);
  return ptr;
}
function decodeString(memory, ptr) {
  const view = new DataView(memory.buffer);
  const len = view.getUint32(ptr, true);
  const bytes = new Uint8Array(memory.buffer, ptr + 4, len);
  return new TextDecoder("utf-8").decode(bytes);
}
function writeGene(memory, ptr, layout, values) {
  const view = new DataView(memory.buffer);
  for (const field of layout.fields) {
    const value = values[field.name];
    const fieldPtr = ptr + field.offset;
    writeField(view, fieldPtr, field, value);
  }
}
function readGene(memory, ptr, layout) {
  const view = new DataView(memory.buffer);
  const result = {};
  for (const field of layout.fields) {
    const fieldPtr = ptr + field.offset;
    result[field.name] = readField(view, memory, fieldPtr, field);
  }
  return result;
}
function writeField(view, ptr, field, value) {
  switch (field.type) {
    case "i32":
      view.setInt32(ptr, Number(value), true);
      break;
    case "bool":
      view.setInt32(ptr, value ? 1 : 0, true);
      break;
    case "i64":
      view.setBigInt64(ptr, BigInt(value), true);
      break;
    case "f32":
      view.setFloat32(ptr, Number(value), true);
      break;
    case "f64":
      view.setFloat64(ptr, Number(value), true);
      break;
    case "string":
      view.setInt32(ptr, Number(value), true);
      break;
  }
}
function readField(view, memory, ptr, field) {
  switch (field.type) {
    case "i32":
      return view.getInt32(ptr, true);
    case "bool":
      return view.getInt32(ptr, true) !== 0;
    case "i64":
      return view.getBigInt64(ptr, true);
    case "f32":
      return view.getFloat32(ptr, true);
    case "f64":
      return view.getFloat64(ptr, true);
    case "string": {
      const strPtr = view.getInt32(ptr, true);
      return strPtr !== 0 ? decodeString(memory, strPtr) : "";
    }
  }
}
function getTypeSize(type) {
  switch (type) {
    case "i32":
    case "bool":
    case "f32":
    case "string":
      return 4;
    case "i64":
    case "f64":
      return 8;
  }
}
function getTypeAlignment(type) {
  switch (type) {
    case "i32":
    case "bool":
    case "f32":
    case "string":
      return 4;
    case "i64":
    case "f64":
      return 8;
  }
}
function calculateLayout(name, fields) {
  let offset = 0;
  let maxAlignment = 1;
  const layoutFields = [];
  for (const field of fields) {
    const alignment = getTypeAlignment(field.type);
    const size = getTypeSize(field.type);
    offset = Math.ceil(offset / alignment) * alignment;
    layoutFields.push({
      ...field,
      offset
    });
    offset += size;
    maxAlignment = Math.max(maxAlignment, alignment);
  }
  const totalSize = Math.ceil(offset / maxAlignment) * maxAlignment;
  return {
    name,
    fields: layoutFields,
    size: totalSize,
    alignment: maxAlignment
  };
}

// src/memory.ts
var BumpAllocator = class {
  memory;
  currentOffset;
  baseOffset;
  /**
   * Create a new bump allocator
   * @param memory - WASM memory instance
   * @param baseOffset - Starting offset (default 1024 to avoid null pointer region)
   */
  constructor(memory, baseOffset = 1024) {
    this.memory = memory;
    this.currentOffset = baseOffset;
    this.baseOffset = baseOffset;
  }
  /**
   * Allocate `size` bytes with alignment
   * @param size - Number of bytes to allocate
   * @param align - Alignment requirement (default 8 for 64-bit)
   * @returns Pointer to allocated memory
   */
  alloc(size, align = 8) {
    const alignedOffset = Math.ceil(this.currentOffset / align) * align;
    const ptr = alignedOffset;
    this.currentOffset = alignedOffset + size;
    const requiredBytes = this.currentOffset;
    const currentBytes = this.memory.buffer.byteLength;
    if (requiredBytes > currentBytes) {
      const requiredPages = Math.ceil(requiredBytes / 65536);
      const currentPages = currentBytes / 65536;
      const pagesToGrow = requiredPages - currentPages;
      if (pagesToGrow > 0) {
        this.memory.grow(pagesToGrow);
      }
    }
    return ptr;
  }
  /**
   * Free memory at pointer (no-op for bump allocator)
   * Memory is only reclaimed on reset()
   */
  free(_ptr) {
  }
  /**
   * Reset allocator to initial state
   * All previously allocated memory becomes invalid
   */
  reset() {
    this.currentOffset = this.baseOffset;
  }
  /**
   * Get current allocation offset
   */
  get offset() {
    return this.currentOffset;
  }
  /**
   * Get raw memory buffer
   */
  get buffer() {
    return this.memory.buffer;
  }
  /**
   * Get WASM memory instance
   */
  get rawMemory() {
    return this.memory;
  }
};
var SpiritMemoryManager = class {
  allocator;
  wasmMemory;
  constructor(memory, baseOffset = 1024) {
    this.wasmMemory = memory;
    this.allocator = new BumpAllocator(memory, baseOffset);
  }
  alloc(size, align = 8) {
    return this.allocator.alloc(size, align);
  }
  free(ptr) {
    this.allocator.free(ptr);
  }
  reset() {
    this.allocator.reset();
  }
  readGene(ptr, layout) {
    return readGene(this.wasmMemory, ptr, layout);
  }
  writeGene(ptr, values, layout) {
    writeGene(this.wasmMemory, ptr, layout, values);
  }
  encodeString(str) {
    return encodeString(this.wasmMemory, this.allocator, str);
  }
  decodeString(ptr) {
    return decodeString(this.wasmMemory, ptr);
  }
  get buffer() {
    return this.allocator.buffer;
  }
  get offset() {
    return this.allocator.offset;
  }
  /**
   * Get typed array views of memory
   */
  get views() {
    const buffer = this.buffer;
    return {
      i8: new Int8Array(buffer),
      u8: new Uint8Array(buffer),
      i32: new Int32Array(buffer),
      u32: new Uint32Array(buffer),
      i64: new BigInt64Array(buffer),
      u64: new BigUint64Array(buffer),
      f32: new Float32Array(buffer),
      f64: new Float64Array(buffer)
    };
  }
};

// src/loa.ts
var coreLoa = {
  name: "core",
  version: "1.0.0",
  capabilities: ["print", "alloc", "time", "random", "effects"],
  provides: (context) => ({
    /**
     * Print a string from WASM memory
     * Signature: vudo_print(ptr: i32, len: i32) -> void
     */
    vudo_print: (ptr, len) => {
      const bytes = new Uint8Array(context.memory.buffer, ptr, len);
      const text = new TextDecoder("utf-8").decode(bytes);
      console.log("[Spirit]", text);
    },
    /**
     * Allocate memory
     * Signature: vudo_alloc(size: i32) -> i32
     */
    vudo_alloc: (size) => {
      return context.alloc(size);
    },
    /**
     * Get current timestamp (milliseconds since epoch)
     * Signature: vudo_now() -> i64
     */
    vudo_now: () => {
      return BigInt(Date.now());
    },
    /**
     * Generate a random number between 0 and 1
     * Signature: vudo_random() -> f64
     */
    vudo_random: () => {
      return Math.random();
    },
    /**
     * Emit a side effect for the host to handle
     * Signature: vudo_emit_effect(effect_id: i32, payload_ptr: i32) -> i32
     *
     * Returns: 0 = success, non-zero = error code
     */
    vudo_emit_effect: (effectId, payloadPtr) => {
      if (context.debug) {
        console.log(`[Spirit] Effect ${effectId} emitted with payload at ${payloadPtr}`);
      }
      return 0;
    },
    /**
     * Debug log (only when debug mode is enabled)
     * Signature: vudo_debug(ptr: i32, len: i32) -> void
     */
    vudo_debug: (ptr, len) => {
      if (context.debug) {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder("utf-8").decode(bytes);
        console.debug("[Spirit:debug]", text);
      }
    },
    /**
     * Abort execution with error message
     * Signature: vudo_abort(msg_ptr: i32, msg_len: i32, file_ptr: i32, file_len: i32, line: i32) -> void
     */
    vudo_abort: (msgPtr, msgLen, filePtr, fileLen, line) => {
      const msg = new TextDecoder().decode(
        new Uint8Array(context.memory.buffer, msgPtr, msgLen)
      );
      const file = new TextDecoder().decode(
        new Uint8Array(context.memory.buffer, filePtr, fileLen)
      );
      throw new Error(`Spirit abort: ${msg} at ${file}:${line}`);
    }
  })
};
var LoaRegistry = class {
  loas = /* @__PURE__ */ new Map();
  constructor() {
    this.register(coreLoa);
  }
  /**
   * Register a new Loa
   * @param loa - Loa to register
   * @throws Error if Loa with same name already exists
   */
  register(loa) {
    if (this.loas.has(loa.name)) {
      throw new Error(`Loa '${loa.name}' is already registered`);
    }
    this.loas.set(loa.name, loa);
  }
  /**
   * Get a Loa by name
   */
  get(name) {
    return this.loas.get(name);
  }
  /**
   * Get all registered Loas
   */
  all() {
    return Array.from(this.loas.values());
  }
  /**
   * Check if a Loa is registered
   */
  has(name) {
    return this.loas.has(name);
  }
  /**
   * Unregister a Loa
   */
  unregister(name) {
    if (name === "core") {
      throw new Error("Cannot unregister core Loa");
    }
    return this.loas.delete(name);
  }
  /**
   * Build WASM imports object from all registered Loas
   */
  buildImports(context) {
    const imports = {};
    for (const loa of this.loas.values()) {
      const provided = loa.provides(context);
      Object.assign(imports, provided);
    }
    return imports;
  }
};
function createLoa(name, version, functions) {
  return {
    name,
    version,
    capabilities: Object.keys(functions),
    provides: (context) => {
      const result = {};
      for (const [key, factory] of Object.entries(functions)) {
        result[key] = factory(context);
      }
      return result;
    }
  };
}
function createLoggingLoa(logger) {
  return {
    name: "logging",
    version: "1.0.0",
    capabilities: ["log", "error", "debug"],
    provides: (context) => ({
      vudo_print: (ptr, len) => {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder("utf-8").decode(bytes);
        logger.log(text);
      },
      vudo_error: (ptr, len) => {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder("utf-8").decode(bytes);
        logger.error(text);
      },
      vudo_debug: (ptr, len) => {
        const bytes = new Uint8Array(context.memory.buffer, ptr, len);
        const text = new TextDecoder("utf-8").decode(bytes);
        logger.debug(text);
      }
    })
  };
}

// src/spirit.ts
var Spirit = class {
  instance;
  wasmMemory;
  memoryManager;
  debug;
  constructor(instance, memory, debug = false) {
    this.instance = instance;
    this.wasmMemory = memory;
    this.memoryManager = new SpiritMemoryManager(memory);
    this.debug = debug;
  }
  /**
   * Call an exported function by name
   */
  call(name, args = []) {
    const func = this.instance.exports[name];
    if (typeof func !== "function") {
      throw new Error(`Function '${name}' not found in Spirit exports`);
    }
    if (this.debug) {
      console.log(`[Spirit] Calling ${name}(${args.join(", ")})`);
    }
    const result = func(...args);
    if (this.debug) {
      console.log(`[Spirit] ${name} returned:`, result);
    }
    return result;
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
  as() {
    const self = this;
    return new Proxy({}, {
      get(_target, prop) {
        return (...args) => self.call(prop, args);
      }
    });
  }
  /**
   * Get the memory manager for this Spirit
   */
  get memory() {
    return this.memoryManager;
  }
  /**
   * Get raw WASM exports
   */
  get exports() {
    return this.instance.exports;
  }
  /**
   * Get raw WASM memory
   */
  get rawMemory() {
    return this.wasmMemory;
  }
  /**
   * Check if the Spirit exports a function
   */
  hasFunction(name) {
    return typeof this.instance.exports[name] === "function";
  }
  /**
   * List all exported function names
   */
  listFunctions() {
    return Object.entries(this.instance.exports).filter(([_, v]) => typeof v === "function").map(([k]) => k);
  }
};
var SpiritLoader = class {
  registry;
  debug;
  constructor(options = {}) {
    this.registry = options.loas ?? new LoaRegistry();
    this.debug = options.debug ?? false;
  }
  /**
   * Load a Spirit from WASM bytes
   */
  async load(wasmBytes, options = {}) {
    const debug = options.debug ?? this.debug;
    const memory = new WebAssembly.Memory({
      initial: options.memory?.initial ?? 16,
      // 1MB default
      maximum: options.memory?.maximum ?? 256
      // 16MB max
    });
    const tempMemoryManager = new SpiritMemoryManager(memory);
    const loaContext = {
      memory,
      alloc: (size) => tempMemoryManager.alloc(size),
      debug
    };
    const registry = options.loas ?? this.registry;
    const loaImports = registry.buildImports(loaContext);
    const imports = {
      env: {
        memory,
        ...loaImports,
        ...options.imports
      }
    };
    if (debug) {
      console.log("[SpiritLoader] Loading WASM module...");
      console.log("[SpiritLoader] Available imports:", Object.keys(imports.env));
    }
    const bytes = wasmBytes instanceof ArrayBuffer ? wasmBytes : new Uint8Array(wasmBytes).buffer;
    const module2 = await WebAssembly.compile(bytes);
    const instance = await WebAssembly.instantiate(module2, imports);
    if (debug) {
      console.log("[SpiritLoader] Spirit loaded successfully");
      const spirit = new Spirit(instance, memory, debug);
      console.log("[SpiritLoader] Exports:", spirit.listFunctions());
    }
    return new Spirit(instance, memory, debug);
  }
  /**
   * Load a Spirit from a URL (browser) or file path (Node.js)
   */
  async loadFrom(source, options = {}) {
    const bytes = await this.fetchBytes(source);
    return this.load(bytes, options);
  }
  /**
   * Fetch WASM bytes from URL or file
   */
  async fetchBytes(source) {
    if (typeof globalThis.fetch !== "undefined") {
      const response = await fetch(source);
      if (!response.ok) {
        throw new Error(`Failed to fetch Spirit: ${response.statusText}`);
      }
      return new Uint8Array(await response.arrayBuffer());
    } else {
      const fs = await import("fs/promises");
      return fs.readFile(source);
    }
  }
};
async function loadSpirit(source, options = {}) {
  const loader = new SpiritLoader({
    loas: options.loas,
    debug: options.debug
  });
  if (source instanceof ArrayBuffer || source instanceof Uint8Array) {
    return loader.load(source, options);
  }
  return loader.loadFrom(source, options);
}

// src/seance.ts
var Seance = class {
  spiritMap = /* @__PURE__ */ new Map();
  registry;
  debug;
  defaultOptions;
  constructor(options = {}) {
    this.registry = options.loas ?? new LoaRegistry();
    this.debug = options.debug ?? false;
    this.defaultOptions = options.defaultLoadOptions ?? {};
  }
  /**
   * Summon a Spirit into the session
   *
   * @param name - Unique name for the Spirit within this session
   * @param source - WASM bytes or URL to load from
   * @param options - Additional load options
   */
  async summon(name, source, options = {}) {
    if (this.spiritMap.has(name)) {
      throw new Error(`Spirit '${name}' is already summoned in this session`);
    }
    const mergedOptions = {
      ...this.defaultOptions,
      ...options,
      loas: options.loas ?? this.registry,
      debug: options.debug ?? this.debug
    };
    if (this.debug) {
      console.log(`[S\xE9ance] Summoning Spirit '${name}'...`);
    }
    const spirit = await loadSpirit(source, mergedOptions);
    this.spiritMap.set(name, spirit);
    if (this.debug) {
      console.log(`[S\xE9ance] Spirit '${name}' summoned successfully`);
    }
  }
  /**
   * Invoke a function on a summoned Spirit
   *
   * @param spiritName - Name of the Spirit to invoke
   * @param funcName - Function name to call
   * @param args - Arguments to pass
   * @returns Function result
   */
  async invoke(spiritName, funcName, args = []) {
    const spirit = this.spiritMap.get(spiritName);
    if (!spirit) {
      throw new Error(`Spirit '${spiritName}' not found in session`);
    }
    if (this.debug) {
      console.log(`[S\xE9ance] Invoking ${spiritName}.${funcName}(${args.join(", ")})`);
    }
    const result = spirit.call(funcName, args);
    if (this.debug) {
      console.log(`[S\xE9ance] ${spiritName}.${funcName} returned:`, result);
    }
    return result;
  }
  /**
   * Get a summoned Spirit by name
   */
  getSpirit(name) {
    return this.spiritMap.get(name);
  }
  /**
   * Check if a Spirit is summoned
   */
  hasSpirit(name) {
    return this.spiritMap.has(name);
  }
  /**
   * List all summoned Spirit names
   */
  spirits() {
    return Array.from(this.spiritMap.keys());
  }
  /**
   * Dismiss a specific Spirit from the session
   */
  async release(name) {
    if (!this.spiritMap.has(name)) {
      throw new Error(`Spirit '${name}' not found in session`);
    }
    if (this.debug) {
      console.log(`[S\xE9ance] Releasing Spirit '${name}'...`);
    }
    const spirit = this.spiritMap.get(name);
    if (spirit) {
      spirit.memory.reset();
    }
    this.spiritMap.delete(name);
    if (this.debug) {
      console.log(`[S\xE9ance] Spirit '${name}' released`);
    }
  }
  /**
   * Dismiss the session and clean up all Spirits
   */
  async dismiss() {
    if (this.debug) {
      console.log(`[S\xE9ance] Dismissing session with ${this.spiritMap.size} Spirit(s)...`);
    }
    for (const [name, spirit] of this.spiritMap) {
      if (this.debug) {
        console.log(`[S\xE9ance] Releasing Spirit '${name}'...`);
      }
      spirit.memory.reset();
    }
    this.spiritMap.clear();
    if (this.debug) {
      console.log("[S\xE9ance] Session dismissed");
    }
  }
  /**
   * Get the Loa registry for this session
   */
  get loas() {
    return this.registry;
  }
  /**
   * Get the number of summoned Spirits
   */
  get size() {
    return this.spiritMap.size;
  }
};
function createSeance(options) {
  return new Seance(options);
}
async function withSeance(fn, options) {
  const seance = new Seance(options);
  try {
    return await fn(seance);
  } finally {
    await seance.dismiss();
  }
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  BumpAllocator,
  LoaRegistry,
  Seance,
  Spirit,
  SpiritLoader,
  SpiritMemoryManager,
  calculateLayout,
  coreLoa,
  createLoa,
  createLoggingLoa,
  createSeance,
  decodeString,
  encodeString,
  getTypeAlignment,
  getTypeSize,
  loadSpirit,
  readGene,
  withSeance,
  writeGene
});
