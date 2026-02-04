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
  ABI_VERSION: () => ABI_VERSION,
  AbiCompat: () => AbiCompat,
  AbiError: () => AbiError,
  AbiErrorType: () => AbiErrorType,
  BumpAllocator: () => BumpAllocator,
  IMPORT_MODULE: () => IMPORT_MODULE,
  LoaRegistry: () => LoaRegistry,
  LogLevel: () => LogLevel,
  Message: () => Message,
  MessageBus: () => MessageBus,
  QualifiedId: () => QualifiedId,
  Response: () => Response,
  ResultCode: () => ResultCode,
  Seance: () => Seance,
  Spirit: () => Spirit,
  SpiritLoader: () => SpiritLoader,
  SpiritMemoryManager: () => SpiritMemoryManager,
  calculateLayout: () => calculateLayout,
  coreLoa: () => coreLoa,
  createLoa: () => createLoa,
  createLoggingLoa: () => createLoggingLoa,
  createMessageBus: () => createMessageBus,
  createMessagingLoa: () => createMessagingLoa,
  createSeance: () => createSeance,
  decodeString: () => decodeString,
  deserializeError: () => deserializeError,
  encodeString: () => encodeString,
  err: () => err,
  flatMap: () => flatMap,
  fromPromise: () => fromPromise,
  getTypeAlignment: () => getTypeAlignment,
  getTypeSize: () => getTypeSize,
  isAbiError: () => isAbiError,
  isErr: () => isErr,
  isMessage: () => isMessage,
  isOk: () => isOk,
  isQualifiedId: () => isQualifiedId,
  isResponse: () => isResponse,
  loadSpirit: () => loadSpirit,
  logLevelToString: () => logLevelToString,
  map: () => map,
  ok: () => ok,
  readGene: () => readGene,
  resultCodeToString: () => resultCodeToString,
  serializeError: () => serializeError,
  stringToLogLevel: () => stringToLogLevel,
  stringToResultCode: () => stringToResultCode,
  unwrap: () => unwrap,
  unwrapOr: () => unwrapOr,
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
    const exportedMemory = instance.exports.memory;
    const actualMemory = exportedMemory ?? memory;
    if (debug) {
      console.log("[SpiritLoader] Spirit loaded successfully");
      if (exportedMemory) {
        console.log("[SpiritLoader] Using module-exported memory");
      } else {
        console.log("[SpiritLoader] Using imported memory");
      }
      const spirit = new Spirit(instance, actualMemory, debug);
      console.log("[SpiritLoader] Exports:", spirit.listFunctions());
    }
    return new Spirit(instance, actualMemory, debug);
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

// src/messagebus.ts
var MessageBus = class {
  /** Message queues per Spirit name */
  queues = /* @__PURE__ */ new Map();
  /** Global message handlers */
  handlers = /* @__PURE__ */ new Map();
  /** Debug mode flag */
  debug;
  constructor(options = {}) {
    this.debug = options.debug ?? false;
  }
  /**
   * Register a Spirit to receive messages
   */
  register(name) {
    if (this.queues.has(name)) {
      throw new Error(`Spirit '${name}' is already registered on the message bus`);
    }
    this.queues.set(name, []);
    this.handlers.set(name, []);
    if (this.debug) {
      console.log(`[MessageBus] Registered Spirit '${name}'`);
    }
  }
  /**
   * Unregister a Spirit from the message bus
   */
  unregister(name) {
    this.queues.delete(name);
    this.handlers.delete(name);
    if (this.debug) {
      console.log(`[MessageBus] Unregistered Spirit '${name}'`);
    }
  }
  /**
   * Check if a Spirit is registered
   */
  isRegistered(name) {
    return this.queues.has(name);
  }
  /**
   * Send a message to a Spirit
   *
   * @param from - Source Spirit name
   * @param to - Destination Spirit name
   * @param channel - Message channel/type identifier
   * @param payload - Raw bytes to send
   * @returns true if message was delivered, false if destination not found
   */
  send(from, to, channel, payload) {
    const queue = this.queues.get(to);
    if (!queue) {
      if (this.debug) {
        console.warn(`[MessageBus] Cannot send to unknown Spirit '${to}'`);
      }
      return false;
    }
    const message = {
      from,
      to,
      channel,
      payload,
      timestamp: Date.now()
    };
    queue.push(message);
    if (this.debug) {
      console.log(
        `[MessageBus] ${from} -> ${to} (channel=${channel}, ${payload.length} bytes)`
      );
    }
    const handlers = this.handlers.get(to);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(message);
        } catch (e) {
          console.error(`[MessageBus] Handler error:`, e);
        }
      }
    }
    return true;
  }
  /**
   * Receive a message for a Spirit
   *
   * @param name - Spirit name to receive for
   * @param channel - Optional channel filter (0 = any channel)
   * @returns Next message or undefined if queue is empty
   */
  recv(name, channel = 0) {
    const queue = this.queues.get(name);
    if (!queue || queue.length === 0) {
      return void 0;
    }
    if (channel === 0) {
      return queue.shift();
    }
    const idx = queue.findIndex((m) => m.channel === channel);
    if (idx === -1) {
      return void 0;
    }
    return queue.splice(idx, 1)[0];
  }
  /**
   * Peek at the next message without removing it
   */
  peek(name, channel = 0) {
    const queue = this.queues.get(name);
    if (!queue || queue.length === 0) {
      return void 0;
    }
    if (channel === 0) {
      return queue[0];
    }
    return queue.find((m) => m.channel === channel);
  }
  /**
   * Get the number of pending messages for a Spirit
   */
  pending(name, channel = 0) {
    const queue = this.queues.get(name);
    if (!queue) {
      return 0;
    }
    if (channel === 0) {
      return queue.length;
    }
    return queue.filter((m) => m.channel === channel).length;
  }
  /**
   * Add a message handler for a Spirit
   */
  onMessage(name, handler) {
    const handlers = this.handlers.get(name);
    if (handlers) {
      handlers.push(handler);
    }
  }
  /**
   * Clear all messages for a Spirit
   */
  clear(name) {
    const queue = this.queues.get(name);
    if (queue) {
      queue.length = 0;
    }
  }
  /**
   * Clear all messages in the bus
   */
  clearAll() {
    for (const queue of this.queues.values()) {
      queue.length = 0;
    }
  }
  /**
   * Get all registered Spirit names
   */
  spirits() {
    return Array.from(this.queues.keys());
  }
};
function createMessagingLoa(bus, spiritName) {
  return {
    name: `messaging:${spiritName}`,
    version: "1.0.0",
    capabilities: ["send", "recv", "pending"],
    provides: (context) => ({
      /**
       * Send a message to another Spirit
       * Signature: vudo_send(to_ptr: i32, to_len: i32, channel: i32, payload_ptr: i32, payload_len: i32) -> i32
       * Returns: 1 = success, 0 = destination not found
       */
      vudo_send: (toPtr, toLen, channel, payloadPtr, payloadLen) => {
        const toBytes = new Uint8Array(context.memory.buffer, toPtr, toLen);
        const to = new TextDecoder("utf-8").decode(toBytes);
        const payload = new Uint8Array(
          context.memory.buffer.slice(payloadPtr, payloadPtr + payloadLen)
        );
        const success = bus.send(spiritName, to, channel, payload);
        return success ? 1 : 0;
      },
      /**
       * Receive a message
       * Signature: vudo_recv(channel: i32, from_buf: i32, from_max: i32, payload_buf: i32, payload_max: i32) -> i32
       * Returns: payload length if message received, -1 if no message, -2 if buffer too small
       *
       * Note: from_buf will be filled with null-terminated sender name
       */
      vudo_recv: (channel, fromBuf, fromMax, payloadBuf, payloadMax) => {
        const message = bus.recv(spiritName, channel);
        if (!message) {
          return -1;
        }
        const fromBytes = new TextEncoder().encode(message.from);
        if (fromBytes.length + 1 > fromMax) {
          const queue = bus["queues"].get(spiritName);
          if (queue) {
            queue.unshift(message);
          }
          return -2;
        }
        if (message.payload.length > payloadMax) {
          const queue = bus["queues"].get(spiritName);
          if (queue) {
            queue.unshift(message);
          }
          return -2;
        }
        const memory = new Uint8Array(context.memory.buffer);
        memory.set(fromBytes, fromBuf);
        memory[fromBuf + fromBytes.length] = 0;
        memory.set(message.payload, payloadBuf);
        return message.payload.length;
      },
      /**
       * Check number of pending messages
       * Signature: vudo_pending(channel: i32) -> i32
       * Returns: number of pending messages
       */
      vudo_pending: (channel) => {
        return bus.pending(spiritName, channel);
      }
    })
  };
}
function createMessageBus(options) {
  return new MessageBus(options);
}

// src/seance.ts
var Seance = class {
  spiritMap = /* @__PURE__ */ new Map();
  registry;
  messageBus;
  debug;
  defaultOptions;
  constructor(options = {}) {
    this.registry = options.loas ?? new LoaRegistry();
    this.messageBus = options.messageBus ?? new MessageBus({ debug: options.debug });
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
    this.messageBus.register(name);
    const messagingLoa = createMessagingLoa(this.messageBus, name);
    const spiritRegistry = options.loas ?? this.registry;
    spiritRegistry.register(messagingLoa);
    const mergedOptions = {
      ...this.defaultOptions,
      ...options,
      loas: spiritRegistry,
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
    this.messageBus.clear(name);
    this.messageBus.unregister(name);
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
    this.messageBus.clearAll();
    for (const [name, spirit] of this.spiritMap) {
      if (this.debug) {
        console.log(`[S\xE9ance] Releasing Spirit '${name}'...`);
      }
      this.messageBus.unregister(name);
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
   * Get the MessageBus for this session
   */
  get messages() {
    return this.messageBus;
  }
  /**
   * Get the number of summoned Spirits
   */
  get size() {
    return this.spiritMap.size;
  }
  // ===========================================================================
  // Messaging Convenience Methods
  // ===========================================================================
  /**
   * Send a message from one Spirit to another
   *
   * @param from - Source Spirit name
   * @param to - Destination Spirit name
   * @param channel - Message channel identifier
   * @param payload - Data to send
   * @returns true if message was delivered
   *
   * @example
   * ```typescript
   * seance.send('ping', 'pong', 1, new Uint8Array([1, 2, 3]));
   * ```
   */
  send(from, to, channel, payload) {
    return this.messageBus.send(from, to, channel, payload);
  }
  /**
   * Check number of pending messages for a Spirit
   *
   * @param name - Spirit name
   * @param channel - Optional channel filter (0 = all channels)
   */
  pending(name, channel = 0) {
    return this.messageBus.pending(name, channel);
  }
  /**
   * Broadcast a message to all Spirits except the sender
   *
   * @param from - Source Spirit name
   * @param channel - Message channel identifier
   * @param payload - Data to send
   * @returns Number of Spirits that received the message
   *
   * @example
   * ```typescript
   * seance.broadcast('coordinator', 1, new Uint8Array([0xFF]));
   * ```
   */
  broadcast(from, channel, payload) {
    let delivered = 0;
    for (const name of this.spiritMap.keys()) {
      if (name !== from) {
        if (this.messageBus.send(from, name, channel, payload)) {
          delivered++;
        }
      }
    }
    return delivered;
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

// src/abi/types.ts
var QualifiedId = class _QualifiedId {
  /**
   * Domain part of the identifier
   */
  domain;
  /**
   * Property part of the identifier
   */
  property;
  /**
   * Optional version part (semantic version)
   */
  version;
  /**
   * Create a new qualified identifier
   * @param domain - Domain part
   * @param property - Property part
   * @param version - Optional version part
   */
  constructor(domain, property, version) {
    this.domain = domain;
    this.property = property;
    this.version = version;
  }
  /**
   * Convert to string representation (e.g., "domain.property.1.0.0")
   */
  toString() {
    if (this.version) {
      return `${this.domain}.${this.property}.${this.version}`;
    }
    return `${this.domain}.${this.property}`;
  }
  /**
   * Parse a qualified identifier string into a QualifiedId
   * @param input - String in format "domain.property" or "domain.property.version"
   * @returns Parsed QualifiedId
   */
  static parse(input) {
    const parts = input.split(".");
    if (parts.length < 2) {
      throw new Error(`Invalid qualified identifier: ${input}`);
    }
    const domain = parts[0];
    const property = parts[1];
    const version = parts.length > 2 ? parts.slice(2).join(".") : void 0;
    return new _QualifiedId(domain, property, version);
  }
};
var LogLevel = /* @__PURE__ */ ((LogLevel2) => {
  LogLevel2[LogLevel2["Debug"] = 0] = "Debug";
  LogLevel2[LogLevel2["Info"] = 1] = "Info";
  LogLevel2[LogLevel2["Warn"] = 2] = "Warn";
  LogLevel2[LogLevel2["Error"] = 3] = "Error";
  return LogLevel2;
})(LogLevel || {});
var ResultCode = /* @__PURE__ */ ((ResultCode2) => {
  ResultCode2[ResultCode2["Success"] = 0] = "Success";
  ResultCode2[ResultCode2["Error"] = 1] = "Error";
  ResultCode2[ResultCode2["Pending"] = 2] = "Pending";
  ResultCode2[ResultCode2["Timeout"] = 3] = "Timeout";
  return ResultCode2;
})(ResultCode || {});
function logLevelToString(level) {
  switch (level) {
    case 0 /* Debug */:
      return "DEBUG";
    case 1 /* Info */:
      return "INFO";
    case 2 /* Warn */:
      return "WARN";
    case 3 /* Error */:
      return "ERROR";
    default:
      return "UNKNOWN";
  }
}
function stringToLogLevel(str) {
  switch (str.toUpperCase()) {
    case "DEBUG":
      return 0 /* Debug */;
    case "INFO":
      return 1 /* Info */;
    case "WARN":
      return 2 /* Warn */;
    case "ERROR":
      return 3 /* Error */;
    default:
      throw new Error(`Unknown log level: ${str}`);
  }
}
function resultCodeToString(code) {
  switch (code) {
    case 0 /* Success */:
      return "SUCCESS";
    case 1 /* Error */:
      return "ERROR";
    case 2 /* Pending */:
      return "PENDING";
    case 3 /* Timeout */:
      return "TIMEOUT";
    default:
      return "UNKNOWN";
  }
}
function stringToResultCode(str) {
  switch (str.toUpperCase()) {
    case "SUCCESS":
      return 0 /* Success */;
    case "ERROR":
      return 1 /* Error */;
    case "PENDING":
      return 2 /* Pending */;
    case "TIMEOUT":
      return 3 /* Timeout */;
    default:
      throw new Error(`Unknown result code: ${str}`);
  }
}

// src/abi/message.ts
var Message = class _Message {
  /** Message header with metadata */
  header;
  /** Message payload wrapper */
  payload;
  /**
   * Create a new message
   * @param id - Unique message identifier
   * @param msg_type - Type of message
   * @param data - Payload data
   * @param options - Optional message configuration
   */
  constructor(id, msg_type, data, options) {
    const now = Date.now();
    this.header = {
      id,
      msg_type,
      source: options?.source || "unknown",
      destination: options?.destination || "unknown",
      timestamp: options?.timestamp || Math.floor(now / 1e3),
      version: options?.version || "1.0.0",
      correlation_id: options?.correlation_id,
      priority: options?.priority,
      timeout_ms: options?.timeout_ms
    };
    this.payload = {
      data,
      encoding: options?.encoding,
      content_type: options?.content_type,
      compression: options?.compression
    };
  }
  /**
   * Serialize to JSON (for transmission)
   * @returns JSON string representation
   */
  toJSON() {
    return JSON.stringify({
      header: this.header,
      payload: this.payload
    });
  }
  /**
   * Deserialize from JSON string
   * @param json - JSON string representation
   * @returns Parsed Message
   */
  static fromJSON(json) {
    const parsed = JSON.parse(json);
    const msg = new _Message(parsed.header.id, parsed.header.msg_type, parsed.payload.data, {
      source: parsed.header.source,
      destination: parsed.header.destination,
      timestamp: parsed.header.timestamp,
      version: parsed.header.version,
      correlation_id: parsed.header.correlation_id,
      priority: parsed.header.priority,
      timeout_ms: parsed.header.timeout_ms,
      encoding: parsed.payload.encoding,
      content_type: parsed.payload.content_type,
      compression: parsed.payload.compression
    });
    return msg;
  }
  /**
   * Create a response to this message
   * @param success - Whether the operation succeeded
   * @param data - Response data
   * @param error - Optional error message
   * @returns Response message
   */
  response(success, data, error) {
    return new Response(this.header.id, success, data, {
      correlation_id: this.header.id,
      source: this.header.destination,
      destination: this.header.source,
      error
    });
  }
};
var Response = class _Response {
  /** Message header with metadata */
  header;
  /** Response status: true for success, false for error */
  success;
  /** Response data payload */
  data;
  /** Optional error message */
  error;
  /**
   * Create a new response
   * @param id - Unique response identifier (should match request)
   * @param success - Whether the operation succeeded
   * @param data - Response data
   * @param options - Optional response configuration
   */
  constructor(id, success, data, options) {
    const now = Date.now();
    this.header = {
      id,
      msg_type: "response",
      source: options?.source || "unknown",
      destination: options?.destination || "unknown",
      timestamp: options?.timestamp || Math.floor(now / 1e3),
      version: options?.version || "1.0.0",
      correlation_id: options?.correlation_id || id,
      priority: options?.priority
    };
    this.success = success;
    this.data = data;
    this.error = options?.error;
  }
  /**
   * Create a successful response with data
   * @param id - Response identifier
   * @param data - Response data
   * @param options - Optional configuration
   * @returns Response instance
   */
  static success(id, data, options) {
    return new _Response(id, true, data, {
      source: options?.source,
      destination: options?.destination,
      version: options?.version,
      priority: options?.priority
    });
  }
  /**
   * Create a failed response with error message
   * @param id - Response identifier
   * @param error - Error message
   * @param options - Optional configuration
   * @returns Response instance
   */
  static error(id, error, options) {
    return new _Response(id, false, null, {
      source: options?.source,
      destination: options?.destination,
      version: options?.version,
      priority: options?.priority,
      error
    });
  }
  /**
   * Serialize to JSON (for transmission)
   * @returns JSON string representation
   */
  toJSON() {
    return JSON.stringify({
      header: this.header,
      success: this.success,
      data: this.data,
      error: this.error
    });
  }
  /**
   * Deserialize from JSON string
   * @param json - JSON string representation
   * @returns Parsed Response
   */
  static fromJSON(json) {
    const parsed = JSON.parse(json);
    const response = new _Response(parsed.header.id, parsed.success, parsed.data, {
      source: parsed.header.source,
      destination: parsed.header.destination,
      timestamp: parsed.header.timestamp,
      version: parsed.header.version,
      correlation_id: parsed.header.correlation_id,
      priority: parsed.header.priority,
      error: parsed.error
    });
    return response;
  }
};

// src/abi/error.ts
var AbiErrorType = /* @__PURE__ */ ((AbiErrorType3) => {
  AbiErrorType3["InvalidConfig"] = "InvalidConfig";
  AbiErrorType3["InvalidMessage"] = "InvalidMessage";
  AbiErrorType3["HostError"] = "HostError";
  AbiErrorType3["TypeMismatch"] = "TypeMismatch";
  AbiErrorType3["Other"] = "Other";
  return AbiErrorType3;
})(AbiErrorType || {});
var AbiError = class _AbiError extends Error {
  /** Type of error (for serialization and matching) */
  type;
  /** Error code for programmatic handling */
  code;
  /** Nested error details (if any) */
  details;
  /** Stack trace context */
  context;
  /**
   * Create a new AbiError
   * @param message - Human-readable error message
   * @param type - Error type discriminant
   * @param code - Error code for programmatic matching
   * @param details - Optional error details
   */
  constructor(message, type = "Other" /* Other */, code = "UNKNOWN_ERROR", details) {
    super(message);
    this.name = "AbiError";
    this.type = type;
    this.code = code;
    this.details = details;
    this.context = new Error().stack;
    Object.setPrototypeOf(this, _AbiError.prototype);
  }
  /**
   * Serialize error to JSON-compatible object
   * Suitable for transmission in Response messages
   */
  toJSON() {
    return {
      type: this.type,
      code: this.code,
      message: this.message,
      details: this.details,
      stack: this.context
    };
  }
  /**
   * Serialize error to string for logging
   */
  toString() {
    const parts = [`[${this.type}:${this.code}]`, this.message];
    if (this.details) {
      parts.push(`Details: ${JSON.stringify(this.details)}`);
    }
    return parts.join(" ");
  }
  /**
   * Create an InvalidConfig error
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static invalidConfig(message, details) {
    return new _AbiError(message, "InvalidConfig" /* InvalidConfig */, "INVALID_CONFIG", details);
  }
  /**
   * Create an InvalidMessage error
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static invalidMessage(message, details) {
    return new _AbiError(message, "InvalidMessage" /* InvalidMessage */, "INVALID_MESSAGE", details);
  }
  /**
   * Create a HostError
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static hostError(message, details) {
    return new _AbiError(message, "HostError" /* HostError */, "HOST_ERROR", details);
  }
  /**
   * Create a TypeMismatch error
   * @param expected - Expected type
   * @param received - Received type
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static typeMismatch(expected, received, details) {
    const message = `Expected ${expected} but received ${received}`;
    const detailsObj = {
      expected,
      received,
      ...typeof details === "object" && details !== null ? details : {}
    };
    return new _AbiError(message, "TypeMismatch" /* TypeMismatch */, "TYPE_MISMATCH", detailsObj);
  }
  /**
   * Create a generic error
   * @param message - Error message
   * @param details - Optional error details
   * @returns AbiError instance
   */
  static other(message, details) {
    return new _AbiError(message, "Other" /* Other */, "ERROR", details);
  }
};
function ok(value) {
  return { ok: true, value };
}
function err(error) {
  const abiError = typeof error === "string" ? AbiError.other(error) : error;
  return { ok: false, error: abiError };
}
function isOk(result) {
  return result.ok === true;
}
function isErr(result) {
  return result.ok === false;
}
function unwrap(result) {
  if (isOk(result)) {
    return result.value;
  }
  throw result.error;
}
function unwrapOr(result, defaultValue) {
  if (isOk(result)) {
    return result.value;
  }
  return defaultValue;
}
function map(result, fn) {
  if (isOk(result)) {
    return ok(fn(result.value));
  }
  return result;
}
function flatMap(result, fn) {
  if (isOk(result)) {
    return fn(result.value);
  }
  return result;
}
async function fromPromise(promise) {
  try {
    return ok(await promise);
  } catch (error) {
    const abiError = error instanceof AbiError ? error : AbiError.other(error instanceof Error ? error.message : String(error));
    return err(abiError);
  }
}

// src/abi/index.ts
var ABI_VERSION = "0.1.0";
var IMPORT_MODULE = "vudo";
function isQualifiedId(value) {
  return value instanceof QualifiedId;
}
function isMessage(value) {
  return value instanceof Message;
}
function isResponse(value) {
  return value instanceof Response;
}
function isAbiError(value) {
  return value instanceof AbiError;
}
var AbiCompat = class {
  /**
   * Check if two ABI versions are compatible
   * Uses semantic versioning rules
   * @param hostVersion - Host ABI version
   * @param guestVersion - Guest ABI version
   * @returns True if versions are compatible
   */
  static compatible(hostVersion, guestVersion) {
    const [hostMajor, hostMinor] = hostVersion.split(".").map(Number);
    const [guestMajor, guestMinor] = guestVersion.split(".").map(Number);
    if (hostMajor !== guestMajor) {
      return false;
    }
    return hostMinor >= guestMinor;
  }
  /**
   * Get a version negotiation message
   * @param version - Version string
   * @returns Message requesting version negotiation
   */
  static versionMessage(version) {
    return new Message("version-check", "version_check", { abi_version: version }, {
      source: "host",
      destination: "guest"
    });
  }
};
function serializeError(error) {
  return error.toJSON();
}
function deserializeError(data) {
  return new AbiError(data.message, data.type, data.code, data.details);
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  ABI_VERSION,
  AbiCompat,
  AbiError,
  AbiErrorType,
  BumpAllocator,
  IMPORT_MODULE,
  LoaRegistry,
  LogLevel,
  Message,
  MessageBus,
  QualifiedId,
  Response,
  ResultCode,
  Seance,
  Spirit,
  SpiritLoader,
  SpiritMemoryManager,
  calculateLayout,
  coreLoa,
  createLoa,
  createLoggingLoa,
  createMessageBus,
  createMessagingLoa,
  createSeance,
  decodeString,
  deserializeError,
  encodeString,
  err,
  flatMap,
  fromPromise,
  getTypeAlignment,
  getTypeSize,
  isAbiError,
  isErr,
  isMessage,
  isOk,
  isQualifiedId,
  isResponse,
  loadSpirit,
  logLevelToString,
  map,
  ok,
  readGene,
  resultCodeToString,
  serializeError,
  stringToLogLevel,
  stringToResultCode,
  unwrap,
  unwrapOr,
  withSeance,
  writeGene
});
