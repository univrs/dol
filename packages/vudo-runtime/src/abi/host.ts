/**
 * VUDO Host Functions - TypeScript Declarations
 *
 * Type definitions for all 22 host functions imported by WASM modules from the "vudo" namespace.
 * These declarations ensure type safety across the WASM boundary.
 *
 * The host functions are categorized into:
 * - I/O Functions (4): print, println, log, error
 * - Memory Functions (3): alloc, free, realloc
 * - Time Functions (3): now, sleep, monotonic_now
 * - Messaging Functions (5): send, recv, pending, broadcast, free_message
 * - Random Functions (2): random, random_bytes
 * - Effect Functions (2): emit_effect, subscribe
 * - Debug Functions (3): breakpoint, assert, panic
 *
 * @module @vudo/runtime/abi/host
 */

/**
 * Log level enumeration for structured logging.
 *
 * @enum {number}
 * @readonly
 */
export enum LogLevel {
  /** Debug level (0) - Development-only messages */
  DEBUG = 0,
  /** Info level (1) - Informational messages */
  INFO = 1,
  /** Warn level (2) - Warning conditions */
  WARN = 2,
  /** Error level (3) - Error conditions */
  ERROR = 3,
}

/**
 * Result codes returned by host functions.
 *
 * @enum {number}
 * @readonly
 */
export enum ResultCode {
  /** Success (0) */
  Ok = 0,
  /** Generic error (-1) */
  Error = -1,
  /** Invalid argument (-2) */
  InvalidArg = -2,
  /** Out of memory (-3) */
  OutOfMemory = -3,
  /** Target not found (-4) */
  NotFound = -4,
  /** Operation not permitted (-5) */
  NotPermitted = -5,
  /** Timeout (-6) */
  Timeout = -6,
  /** Buffer too small (-7) */
  BufferTooSmall = -7,
}

// ============================================================================
// I/O FUNCTIONS
// ============================================================================

/**
 * Print a UTF-8 string to the host console (without newline).
 *
 * Reads a UTF-8 string from WASM linear memory and outputs it to the host console.
 *
 * @param ptr - Pointer to string data in WASM linear memory
 * @param len - Length of string in bytes
 * @returns void
 *
 * @remarks
 * - The string is decoded as UTF-8
 * - No newline is appended to the output
 * - Invalid UTF-8 sequences cause undefined behavior
 *
 * @example
 * ```typescript
 * // In WASM:
 * vudo_print(string_ptr, string_len);  // Prints without newline
 * ```
 */
export declare function vudo_print(ptr: number, len: number): void;

/**
 * Print a UTF-8 string to the host console (with newline).
 *
 * Reads a UTF-8 string from WASM linear memory and outputs it to the host console
 * with a trailing newline.
 *
 * @param ptr - Pointer to string data in WASM linear memory
 * @param len - Length of string in bytes
 * @returns void
 *
 * @remarks
 * - The string is decoded as UTF-8
 * - A newline character is automatically appended
 * - Invalid UTF-8 sequences cause undefined behavior
 *
 * @example
 * ```typescript
 * // In WASM:
 * vudo_println(string_ptr, string_len);  // Prints with newline
 * ```
 */
export declare function vudo_println(ptr: number, len: number): void;

/**
 * Write a structured log message with a severity level.
 *
 * Logs a message at the specified level with full formatting support.
 *
 * @param level - Log level (0=DEBUG, 1=INFO, 2=WARN, 3=ERROR)
 * @param ptr - Pointer to message data in WASM linear memory
 * @param len - Length of message in bytes
 * @returns void
 *
 * @remarks
 * - Message is decoded as UTF-8
 * - Level must be 0-3; other values are treated as ERROR
 * - Host may filter logs based on configured level
 *
 * @example
 * ```typescript
 * // In WASM:
 * vudo_log(LogLevel.WARN, msg_ptr, msg_len);  // Log a warning
 * ```
 */
export declare function vudo_log(level: number, ptr: number, len: number): void;

/**
 * Log an error message to the host error stream.
 *
 * Convenience function for logging errors, equivalent to vudo_log(ERROR, ptr, len).
 *
 * @param ptr - Pointer to error message in WASM linear memory
 * @param len - Length of message in bytes
 * @returns void
 *
 * @remarks
 * - Message is decoded as UTF-8
 * - Errors are typically logged with maximum visibility
 * - Invalid UTF-8 sequences cause undefined behavior
 *
 * @example
 * ```typescript
 * // In WASM:
 * vudo_error(error_msg_ptr, error_msg_len);
 * ```
 */
export declare function vudo_error(ptr: number, len: number): void;

// ============================================================================
// MEMORY FUNCTIONS
// ============================================================================

/**
 * Allocate memory from the host allocator.
 *
 * Requests a contiguous block of memory from the host allocator and returns a pointer
 * to the allocated block.
 *
 * @param size - Number of bytes to allocate
 * @returns Pointer to allocated memory (0 on failure)
 *
 * @remarks
 * - Returns pointer to start of allocated block on success
 * - Returns 0 if allocation fails (e.g., out of memory)
 * - Allocated memory is zero-initialized
 * - The allocator may use any allocation strategy (bump, arena, free-list, etc.)
 * - Must be paired with vudo_free() for proper cleanup
 *
 * @example
 * ```typescript
 * // In WASM:
 * let ptr = vudo_alloc(256);
 * if (ptr === 0) {
 *     // Allocation failed - out of memory
 *     vudo_panic("Out of memory", 16);
 * }
 * // Use ptr to store data
 * ```
 */
export declare function vudo_alloc(size: number): number;

/**
 * Free previously allocated memory.
 *
 * Returns a block of memory to the host allocator for reuse.
 *
 * @param ptr - Pointer to previously allocated memory
 * @param size - Size of the block (must match allocation)
 * @returns void
 *
 * @remarks
 * - Must pass the exact size from the original allocation
 * - Freeing invalid pointers or wrong sizes causes undefined behavior
 * - Double-free is undefined behavior
 * - The memory is no longer valid after calling this function
 * - Some allocators (e.g., bump) may treat this as a no-op
 *
 * @example
 * ```typescript
 * // In WASM:
 * let ptr = vudo_alloc(256);
 * // ... use ptr ...
 * vudo_free(ptr, 256);  // Must pass exact size
 * ```
 */
export declare function vudo_free(ptr: number, size: number): void;

/**
 * Reallocate memory (grow or shrink an existing block).
 *
 * Resizes a previously allocated block to a new size. The contents are preserved
 * up to the minimum of old and new sizes.
 *
 * @param ptr - Pointer to previously allocated memory
 * @param oldSize - Current size of the block
 * @param newSize - Desired new size in bytes
 * @returns New pointer to reallocated memory (0 on failure)
 *
 * @remarks
 * - If newSize > oldSize, new bytes are zero-initialized
 * - If newSize < oldSize, data beyond newSize is discarded
 * - oldSize must match the original allocation size
 * - Returns new pointer (which may differ from original)
 * - Returns 0 on failure (original block remains valid)
 * - In-place reallocation is not guaranteed
 * - Must update all pointers to the memory after realloc succeeds
 *
 * @example
 * ```typescript
 * // In WASM:
 * let ptr = vudo_alloc(256);
 * let new_ptr = vudo_realloc(ptr, 256, 512);
 * if (new_ptr === 0) {
 *     // Reallocation failed - original ptr still valid
 *     vudo_panic("Realloc failed", 15);
 * }
 * ptr = new_ptr;  // Update pointer
 * ```
 */
export declare function vudo_realloc(
  ptr: number,
  oldSize: number,
  newSize: number
): number;

// ============================================================================
// TIME FUNCTIONS
// ============================================================================

/**
 * Get the current timestamp in milliseconds since the Unix epoch.
 *
 * Returns the number of milliseconds that have elapsed since 00:00:00 UTC on 1 January 1970.
 *
 * @returns Timestamp in milliseconds (i64)
 *
 * @remarks
 * - Returns a 64-bit signed integer
 * - Resolution is milliseconds (not nanoseconds)
 * - May have platform-dependent precision
 * - Suitable for timeout calculations and basic timing
 *
 * @example
 * ```typescript
 * // In WASM:
 * let now = vudo_now();  // Returns i64
 * let timeout_at = now + 5000;  // 5 seconds from now
 * ```
 */
export declare function vudo_now(): bigint;

/**
 * Sleep for a specified number of milliseconds.
 *
 * Blocks the calling Spirit for the specified duration. This is typically asynchronous
 * in the runtime, allowing other Spirits to run.
 *
 * @param ms - Duration in milliseconds to sleep
 * @returns void
 *
 * @remarks
 * - Sleep is approximate; actual duration may be slightly longer
 * - During sleep, other Spirits can execute
 * - May be interrupted by host shutdown or explicit cancellation
 * - Negative values are treated as 0 (return immediately)
 *
 * @example
 * ```typescript
 * // In WASM:
 * vudo_sleep(1000);  // Sleep for 1 second
 * ```
 */
export declare function vudo_sleep(ms: number): void;

/**
 * Get monotonic time in nanoseconds (for precise measurements).
 *
 * Returns a monotonically increasing timestamp suitable for measuring elapsed time.
 * Unlike vudo_now(), this cannot go backward.
 *
 * @returns Nanoseconds since an arbitrary epoch (i64)
 *
 * @remarks
 * - Time is monotonic: never goes backward
 * - Starting epoch is arbitrary (not Unix epoch)
 * - Resolution is nanoseconds
 * - Suitable for high-precision timing and performance measurements
 * - May wrap around after very long periods (but very rare)
 *
 * @example
 * ```typescript
 * // In WASM:
 * let start = vudo_monotonic_now();
 * // ... do work ...
 * let elapsed_ns = vudo_monotonic_now() - start;
 * let elapsed_ms = elapsed_ns / 1_000_000;
 * ```
 */
export declare function vudo_monotonic_now(): bigint;

// ============================================================================
// MESSAGING FUNCTIONS
// ============================================================================

/**
 * Send a message to another Spirit.
 *
 * Sends a message to a named target Spirit. The message is delivered to the target's
 * inbox for asynchronous processing.
 *
 * @param targetPtr - Pointer to target Spirit name (UTF-8)
 * @param targetLen - Length of target name in bytes
 * @param payloadPtr - Pointer to message payload data
 * @param payloadLen - Length of payload in bytes
 * @returns Result code (0 on success, non-zero on error)
 *
 * @remarks
 * - Target name is decoded as UTF-8
 * - Payload can be any binary data
 * - Returns 0 (ResultCode::Ok) on success
 * - Returns non-zero error code if target not found or message queue full
 * - Message delivery is asynchronous; use recv() to receive replies
 * - Messages are copied into the host's message queue
 *
 * @example
 * ```typescript
 * // In WASM:
 * let target = "worker-1";
 * let msg = "Hello";
 * let result = vudo_send(target_ptr, target_len, msg_ptr, msg_len);
 * if (result !== 0) {
 *     vudo_error("Send failed", 11);
 * }
 * ```
 */
export declare function vudo_send(
  targetPtr: number,
  targetLen: number,
  payloadPtr: number,
  payloadLen: number
): number;

/**
 * Receive the next message from the Spirit's inbox.
 *
 * Retrieves the oldest message from the Spirit's message queue without blocking.
 *
 * @returns Pointer to message data (0 if inbox is empty)
 *
 * @remarks
 * - Returns pointer to message in WASM memory on success
 * - Returns 0 if no messages are available (non-blocking)
 * - Message format: [sender_len:u32][sender][payload_len:u32][payload]
 * - Caller must use vudo_free_message() to release the message after processing
 * - All integers in message are little-endian
 * - Safe to call multiple times; will return different messages each call
 *
 * @example
 * ```typescript
 * // In WASM:
 * let msg_ptr = vudo_recv();
 * if (msg_ptr === 0) {
 *     // No messages available
 *     return;
 * }
 * // Process message at msg_ptr
 * vudo_free_message(msg_ptr);
 * ```
 */
export declare function vudo_recv(): number;

/**
 * Check the number of pending messages in the inbox.
 *
 * Returns the count of messages waiting to be received by this Spirit.
 *
 * @returns Number of pending messages (i32)
 *
 * @remarks
 * - Returns 0 if inbox is empty
 * - Returns non-zero for each message in queue
 * - Non-blocking; does not wait for messages
 * - The count may change after this call if other Spirits send messages
 * - Useful for busy-waiting or conditional receive logic
 *
 * @example
 * ```typescript
 * // In WASM:
 * while (vudo_pending() > 0) {
 *     let msg = vudo_recv();
 *     if (msg !== 0) {
 *         // Process message
 *         vudo_free_message(msg);
 *     }
 * }
 * ```
 */
export declare function vudo_pending(): number;

/**
 * Broadcast a message to all Spirits in the session.
 *
 * Sends a message to every Spirit in the current Séance (session), including self.
 *
 * @param ptr - Pointer to message payload data
 * @param len - Length of payload in bytes
 * @returns Result code (0 on success, non-zero on error)
 *
 * @remarks
 * - All Spirits receive the message under their own Spirit name as sender
 * - Payload can be any binary data
 * - Returns 0 (ResultCode::Ok) on success
 * - Returns non-zero if broadcast failed (e.g., out of queue space)
 * - Messages are delivered to all Spirits simultaneously
 * - The sender receives a copy of their own broadcast
 *
 * @example
 * ```typescript
 * // In WASM:
 * let msg = "System shutdown";
 * let result = vudo_broadcast(msg_ptr, msg_len);
 * if (result !== 0) {
 *     vudo_error("Broadcast failed", 16);
 * }
 * ```
 */
export declare function vudo_broadcast(ptr: number, len: number): number;

/**
 * Free a received message after processing.
 *
 * Releases a message returned by vudo_recv() back to the host's memory pool.
 *
 * @param ptr - Pointer to message (returned from vudo_recv)
 * @returns void
 *
 * @remarks
 * - Must be called exactly once for each message received
 * - Using a freed message pointer causes undefined behavior
 * - Some allocators may treat this as a no-op
 * - Messages are not automatically freed; must be released explicitly
 * - The message data is no longer valid after calling this function
 *
 * @example
 * ```typescript
 * // In WASM:
 * let msg = vudo_recv();
 * if (msg !== 0) {
 *     // Read and process message
 *     vudo_free_message(msg);  // Release after done
 * }
 * ```
 */
export declare function vudo_free_message(ptr: number): void;

// ============================================================================
// RANDOM FUNCTIONS
// ============================================================================

/**
 * Generate a random floating-point number.
 *
 * Returns a cryptographically suitable random number in the range [0, 1).
 *
 * @returns Random f64 in [0, 1)
 *
 * @remarks
 * - Range is [0.0, 1.0) (excludes 1.0)
 * - Should use a cryptographic random source for security-sensitive use
 * - Each call returns a new random value
 * - Thread-safe in the host
 *
 * @example
 * ```typescript
 * // In WASM:
 * let random_value = vudo_random();  // f64 in [0, 1)
 * let dice_roll = (random_value * 6.0) as i32 + 1;  // 1-6
 * ```
 */
export declare function vudo_random(): number;

/**
 * Fill a buffer with random bytes.
 *
 * Generates random bytes and writes them to the specified memory location.
 *
 * @param ptr - Pointer to buffer in WASM linear memory
 * @param len - Number of random bytes to generate
 * @returns void
 *
 * @remarks
 * - Generates cryptographically suitable random bytes
 * - Writes exactly len bytes starting at ptr
 * - Buffer must be pre-allocated and valid
 * - Overwriting beyond allocated space causes undefined behavior
 * - Suitable for generating random UUIDs, salts, nonces, etc.
 *
 * @example
 * ```typescript
 * // In WASM:
 * let buffer_ptr = vudo_alloc(32);
 * vudo_random_bytes(buffer_ptr, 32);  // Fill with 32 random bytes
 * // Use bytes from buffer
 * vudo_free(buffer_ptr, 32);
 * ```
 */
export declare function vudo_random_bytes(ptr: number, len: number): void;

// ============================================================================
// EFFECT FUNCTIONS
// ============================================================================

/**
 * Emit a side effect for the host to handle.
 *
 * Sends a side effect request to the host runtime. Common effects include file I/O,
 * HTTP requests, database operations, and process spawning.
 *
 * @param effectId - Effect type identifier
 * @param payloadPtr - Pointer to effect payload data
 * @param payloadLen - Length of payload in bytes
 * @returns Result code (0 on success, non-zero on error)
 *
 * @remarks
 * - Effect handling is entirely host-dependent
 * - Standard effect IDs:
 *   - 0 = Noop
 *   - 1 = Terminate
 *   - 2 = Spawn
 *   - 10 = FsRead
 *   - 11 = FsWrite
 *   - 20 = HttpGet
 *   - 21 = HttpPost
 *   - 30 = DbQuery
 * - Custom effect IDs (>255) can be used
 * - Returns 0 (ResultCode::Ok) on success
 * - Returns non-zero if effect is unknown or fails
 * - Effects are processed asynchronously
 *
 * @example
 * ```typescript
 * // In WASM:
 * let effect_id = 20;  // HttpGet
 * let url = "https://api.example.com/data";
 * let result = vudo_emit_effect(effect_id, url_ptr, url_len);
 * if (result !== 0) {
 *     vudo_error("Effect failed", 14);
 * }
 * ```
 */
export declare function vudo_emit_effect(
  effectId: number,
  payloadPtr: number,
  payloadLen: number
): number;

/**
 * Subscribe to an effect channel.
 *
 * Registers the Spirit to receive notifications when a specific effect channel
 * changes or completes.
 *
 * @param channelPtr - Pointer to channel name (UTF-8)
 * @param channelLen - Length of channel name in bytes
 * @returns Result code (0 on success, non-zero on error)
 *
 * @remarks
 * - Channel name is decoded as UTF-8
 * - Returns 0 (ResultCode::Ok) on successful subscription
 * - Returns non-zero if channel does not exist or subscription fails
 * - Multiple Spirits can subscribe to the same channel
 * - Notifications are delivered as messages to the inbox
 * - Unsubscribe by terminating the Spirit
 *
 * @example
 * ```typescript
 * // In WASM:
 * let channel = "file-operations";
 * let result = vudo_subscribe(channel_ptr, channel_len);
 * if (result !== 0) {
 *     vudo_error("Subscribe failed", 16);
 * }
 * // Now receive() will return notifications from this channel
 * ```
 */
export declare function vudo_subscribe(
  channelPtr: number,
  channelLen: number
): number;

// ============================================================================
// DEBUG FUNCTIONS
// ============================================================================

/**
 * Trigger a breakpoint (debug builds only).
 *
 * Signals the debugger to pause execution if a debugger is attached.
 *
 * @returns void
 *
 * @remarks
 * - Only effective if the host debugger supports breakpoints
 * - No-op in release builds or if no debugger is attached
 * - Does not affect execution if no debugger is present
 * - Useful for debugging specific conditions in WASM code
 *
 * @example
 * ```typescript
 * // In WASM:
 * if (error_condition) {
 *     vudo_breakpoint();  // Pause here if debugger attached
 * }
 * ```
 */
export declare function vudo_breakpoint(): void;

/**
 * Assert a condition with an optional message.
 *
 * Verifies that a condition is true. If false, logs an error message and
 * may terminate the Spirit depending on host configuration.
 *
 * @param condition - Condition to assert (non-zero = true, zero = false)
 * @param ptr - Pointer to assertion message (UTF-8)
 * @param len - Length of message in bytes
 * @returns void
 *
 * @remarks
 * - If condition is non-zero (true), assert succeeds silently
 * - If condition is zero (false), logs the message as an error
 * - Host may terminate the Spirit or raise an exception on assertion failure
 * - Message is decoded as UTF-8
 * - Invalid UTF-8 causes undefined behavior
 * - Useful for validating invariants during development
 *
 * @example
 * ```typescript
 * // In WASM:
 * let buffer_size = 256;
 * let msg = "Buffer size must be positive";
 * vudo_assert(buffer_size > 0, msg_ptr, msg_len);
 * ```
 */
export declare function vudo_assert(
  condition: number,
  ptr: number,
  len: number
): void;

/**
 * Panic with an error message (terminates the Spirit).
 *
 * Immediately terminates the Spirit execution and logs a panic message.
 * This is an unrecoverable error that cannot be caught.
 *
 * @param ptr - Pointer to panic message (UTF-8)
 * @param len - Length of message in bytes
 * @returns void (does not return)
 *
 * @remarks
 * - This function never returns; it terminates execution
 * - Message is decoded as UTF-8 and logged
 * - Host may clean up resources associated with the Spirit
 * - Other Spirits in the session continue running
 * - Invalid UTF-8 causes undefined behavior
 * - Use only for truly unrecoverable errors
 *
 * @example
 * ```typescript
 * // In WASM:
 * if (critical_error) {
 *     let msg = "Critical error occurred";
 *     vudo_panic(msg_ptr, msg_len);  // Never returns
 * }
 * ```
 */
export declare function vudo_panic(ptr: number, len: number): void;

// ============================================================================
// HELPER TYPES
// ============================================================================

/**
 * Metadata for a host function (for introspection).
 *
 * @interface
 */
export interface HostFunctionMetadata {
  /** Function name as imported */
  name: string;
  /** Function category */
  category: HostFunctionCategory;
  /** WASM parameter types */
  params: WasmType[];
  /** WASM return type, if any */
  result?: WasmType;
  /** Human-readable description */
  description: string;
}

/**
 * Category of host function.
 *
 * @enum {string}
 */
export enum HostFunctionCategory {
  IO = 'io',
  Memory = 'memory',
  Time = 'time',
  Messaging = 'messaging',
  Random = 'random',
  Effects = 'effects',
  Debug = 'debug',
}

/**
 * WASM value types.
 *
 * @enum {string}
 */
export enum WasmType {
  I32 = 'i32',
  I64 = 'i64',
  F32 = 'f32',
  F64 = 'f64',
}

/**
 * Registry of all host functions.
 *
 * @constant
 */
export const HOST_FUNCTION_REGISTRY: Record<string, HostFunctionMetadata> = {
  vudo_print: {
    name: 'vudo_print',
    category: HostFunctionCategory.IO,
    params: [WasmType.I32, WasmType.I32],
    description: 'Print a UTF-8 string (no newline)',
  },
  vudo_println: {
    name: 'vudo_println',
    category: HostFunctionCategory.IO,
    params: [WasmType.I32, WasmType.I32],
    description: 'Print a UTF-8 string with newline',
  },
  vudo_log: {
    name: 'vudo_log',
    category: HostFunctionCategory.IO,
    params: [WasmType.I32, WasmType.I32, WasmType.I32],
    description: 'Structured logging with level',
  },
  vudo_error: {
    name: 'vudo_error',
    category: HostFunctionCategory.IO,
    params: [WasmType.I32, WasmType.I32],
    description: 'Log an error message',
  },
  vudo_alloc: {
    name: 'vudo_alloc',
    category: HostFunctionCategory.Memory,
    params: [WasmType.I32],
    result: WasmType.I32,
    description: 'Allocate memory from host allocator',
  },
  vudo_free: {
    name: 'vudo_free',
    category: HostFunctionCategory.Memory,
    params: [WasmType.I32, WasmType.I32],
    description: 'Free previously allocated memory',
  },
  vudo_realloc: {
    name: 'vudo_realloc',
    category: HostFunctionCategory.Memory,
    params: [WasmType.I32, WasmType.I32, WasmType.I32],
    result: WasmType.I32,
    description: 'Reallocate memory (grow or shrink)',
  },
  vudo_now: {
    name: 'vudo_now',
    category: HostFunctionCategory.Time,
    params: [],
    result: WasmType.I64,
    description: 'Get current timestamp (milliseconds since epoch)',
  },
  vudo_sleep: {
    name: 'vudo_sleep',
    category: HostFunctionCategory.Time,
    params: [WasmType.I32],
    description: 'Sleep for specified milliseconds',
  },
  vudo_monotonic_now: {
    name: 'vudo_monotonic_now',
    category: HostFunctionCategory.Time,
    params: [],
    result: WasmType.I64,
    description: 'Get monotonic time in nanoseconds',
  },
  vudo_send: {
    name: 'vudo_send',
    category: HostFunctionCategory.Messaging,
    params: [WasmType.I32, WasmType.I32, WasmType.I32, WasmType.I32],
    result: WasmType.I32,
    description: 'Send a message to another Spirit',
  },
  vudo_recv: {
    name: 'vudo_recv',
    category: HostFunctionCategory.Messaging,
    params: [],
    result: WasmType.I32,
    description: 'Receive next message from inbox',
  },
  vudo_pending: {
    name: 'vudo_pending',
    category: HostFunctionCategory.Messaging,
    params: [],
    result: WasmType.I32,
    description: 'Check number of pending messages',
  },
  vudo_broadcast: {
    name: 'vudo_broadcast',
    category: HostFunctionCategory.Messaging,
    params: [WasmType.I32, WasmType.I32],
    result: WasmType.I32,
    description: 'Broadcast message to all Spirits',
  },
  vudo_free_message: {
    name: 'vudo_free_message',
    category: HostFunctionCategory.Messaging,
    params: [WasmType.I32],
    description: 'Free a received message',
  },
  vudo_random: {
    name: 'vudo_random',
    category: HostFunctionCategory.Random,
    params: [],
    result: WasmType.F64,
    description: 'Generate random f64 in [0, 1)',
  },
  vudo_random_bytes: {
    name: 'vudo_random_bytes',
    category: HostFunctionCategory.Random,
    params: [WasmType.I32, WasmType.I32],
    description: 'Generate random bytes',
  },
  vudo_emit_effect: {
    name: 'vudo_emit_effect',
    category: HostFunctionCategory.Effects,
    params: [WasmType.I32, WasmType.I32, WasmType.I32],
    result: WasmType.I32,
    description: 'Emit a side effect for host handling',
  },
  vudo_subscribe: {
    name: 'vudo_subscribe',
    category: HostFunctionCategory.Effects,
    params: [WasmType.I32, WasmType.I32],
    result: WasmType.I32,
    description: 'Subscribe to an effect channel',
  },
  vudo_breakpoint: {
    name: 'vudo_breakpoint',
    category: HostFunctionCategory.Debug,
    params: [],
    description: 'Trigger a breakpoint (debug builds only)',
  },
  vudo_assert: {
    name: 'vudo_assert',
    category: HostFunctionCategory.Debug,
    params: [WasmType.I32, WasmType.I32, WasmType.I32],
    description: 'Assert condition with message',
  },
  vudo_panic: {
    name: 'vudo_panic',
    category: HostFunctionCategory.Debug,
    params: [WasmType.I32, WasmType.I32],
    description: 'Panic with message (terminates Spirit)',
  },
};
