# DOL ABI Specification

> **Version:** 0.1.0
> **Status:** Stable
> **Last Updated:** 2026-02-04

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Type Definitions](#type-definitions)
4. [Host Functions](#host-functions)
5. [Message Protocol](#message-protocol)
6. [Error Handling](#error-handling)
7. [Memory Model](#memory-model)
8. [Best Practices](#best-practices)
9. [Examples](#examples)

---

## 1. Overview

### Purpose

The DOL Application Binary Interface (ABI) defines the contract between DOL-compiled WASM modules (Spirits) and the VUDO runtime. This ABI enables:

- **Spirit ↔ Host Communication**: Structured messaging between WASM and runtime
- **Resource Management**: Memory allocation, deallocation, and lifecycle
- **Side Effect Handling**: Controlled I/O, time, random, and custom effects
- **Type Safety**: Strongly-typed interfaces across the WASM boundary
- **Interoperability**: Consistent behavior across different runtime implementations

### Relationship Between Compiler and Runtime

The DOL compiler (`dolc`) and VUDO runtime work together through this ABI:

```
┌─────────────────────────────────────────────────────────────────┐
│                     DOL Source Code                             │
│                                                                 │
│   fun greet(name: String) {                                     │
│       println("Hello, " + name)                                 │
│       send("logger", "Greeted " + name)                         │
│   }                                                             │
└────────────────────────┬────────────────────────────────────────┘
                         │
                    dolc compile
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                  DOL Spirit (WASM Module)                       │
│                                                                 │
│   import "vudo" "vudo_println"                                  │
│   import "vudo" "vudo_send"                                     │
│                                                                 │
│   func greet(name_ptr i32, name_len i32) {                     │
│       call $vudo_println(name_ptr, name_len)                   │
│       call $vudo_send(...)                                     │
│   }                                                             │
└────────────────────────┬────────────────────────────────────────┘
                         │
                    WASM imports
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                     VUDO Runtime (Host)                         │
│                                                                 │
│   vudo_println(ptr, len) {                                      │
│       const str = decodeUtf8(memory, ptr, len)                  │
│       console.log(str)                                          │
│   }                                                             │
│                                                                 │
│   vudo_send(target_ptr, target_len, payload_ptr, payload_len) {│
│       const target = decodeUtf8(memory, target_ptr, target_len) │
│       const payload = decodeUtf8(memory, payload_ptr, ...)      │
│       messageBus.send(target, payload)                          │
│       return ResultCode.Ok                                      │
│   }                                                             │
└─────────────────────────────────────────────────────────────────┘
```

### Key Concepts

- **Spirit**: A DOL program compiled to WASM that imports host functions
- **Host**: The VUDO runtime that provides implementations of host functions
- **ABI Contract**: All host functions are imported from the `"vudo"` namespace
- **Effect System**: DOL tracks which functions call host functions (effectful)
- **Linear Memory**: Shared byte array for passing strings and complex data

---

## 2. Architecture

### WASM Module Structure

All DOL-compiled WASM modules follow this structure:

```wasm
(module
  ;; Import host functions from "vudo" namespace
  (import "vudo" "vudo_print" (func $print (param i32 i32)))
  (import "vudo" "vudo_alloc" (func $alloc (param i32) (result i32)))
  (import "vudo" "vudo_now" (func $now (result i64)))

  ;; Linear memory (shared with host)
  (memory (export "memory") 1)

  ;; Spirit functions
  (func (export "main")
    ;; ... calls to imported host functions ...
  )
)
```

### Import Namespace

All 22 host functions are imported from the **`"vudo"`** module namespace. This ensures:

1. No collision with other WASM imports (e.g., WASI)
2. Clear separation between host and guest functions
3. Easy validation of ABI compliance

### Host Function Categories

```
┌───────────────────────────────────────────────────────────────┐
│                      Host Functions (22)                      │
├───────────────┬───────────────┬───────────────┬───────────────┤
│   I/O (4)     │  Memory (3)   │   Time (3)    │ Messaging (5) │
│               │               │               │               │
│ • print       │ • alloc       │ • now         │ • send        │
│ • println     │ • free        │ • sleep       │ • recv        │
│ • log         │ • realloc     │ • monotonic   │ • pending     │
│ • error       │               │               │ • broadcast   │
│               │               │               │ • free_msg    │
├───────────────┼───────────────┼───────────────┼───────────────┤
│  Random (2)   │  Effects (2)  │   Debug (3)   │               │
│               │               │               │               │
│ • random      │ • emit_effect │ • breakpoint  │               │
│ • random_bytes│ • subscribe   │ • assert      │               │
│               │               │ • panic       │               │
└───────────────┴───────────────┴───────────────┴───────────────┘
```

---

## 3. Type Definitions

### WASM Value Types

The ABI uses standard WASM value types:

```rust
/// WASM value types used in function signatures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    /// 32-bit integer (i32)
    I32,
    /// 64-bit integer (i64)
    I64,
    /// 32-bit float (f32)
    F32,
    /// 64-bit float (f64)
    F64,
}
```

**TypeScript Equivalent:**

```typescript
type WasmType = 'i32' | 'i64' | 'f32' | 'f64';
```

### LogLevel

Structured logging severity levels:

```rust
/// Log levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    /// Debug information (development only)
    Debug = 0,
    /// Informational messages
    Info = 1,
    /// Warning conditions
    Warn = 2,
    /// Error conditions
    Error = 3,
}
```

**TypeScript Equivalent:**

```typescript
enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}
```

**Usage:**

```dol
log(LogLevel.Info, "Spirit started")
log(LogLevel.Error, "Failed to connect")
```

### ResultCode

Return codes for fallible host functions:

```rust
/// Result codes returned by host functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ResultCode {
    /// Success
    Ok = 0,
    /// Generic error
    Error = -1,
    /// Invalid argument
    InvalidArg = -2,
    /// Out of memory
    OutOfMemory = -3,
    /// Target not found (for messaging)
    NotFound = -4,
    /// Operation not permitted
    NotPermitted = -5,
    /// Timeout
    Timeout = -6,
    /// Buffer too small
    BufferTooSmall = -7,
}
```

**TypeScript Equivalent:**

```typescript
enum ResultCode {
    Ok = 0,
    Error = -1,
    InvalidArg = -2,
    OutOfMemory = -3,
    NotFound = -4,
    NotPermitted = -5,
    Timeout = -6,
    BufferTooSmall = -7,
}
```

**Checking Results:**

```rust
let result = vudo_send(...);
if result == ResultCode::Ok {
    // Success
} else {
    // Handle error based on result code
}
```

### StandardEffect

Predefined side effects that the runtime handles:

```rust
/// Standard effect types that the runtime handles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StandardEffect {
    /// No-op effect (for testing)
    Noop = 0,
    /// Request to terminate the Spirit
    Terminate = 1,
    /// Request to spawn a new Spirit
    Spawn = 2,
    /// File system read request
    FsRead = 10,
    /// File system write request
    FsWrite = 11,
    /// HTTP GET request
    HttpGet = 20,
    /// HTTP POST request
    HttpPost = 21,
    /// Database query
    DbQuery = 30,
}
```

**TypeScript Equivalent:**

```typescript
enum StandardEffect {
    Noop = 0,
    Terminate = 1,
    Spawn = 2,
    FsRead = 10,
    FsWrite = 11,
    HttpGet = 20,
    HttpPost = 21,
    DbQuery = 30,
}
```

### MessageHeader

Metadata for Spirit-to-Spirit messages:

```rust
/// Message header containing sender information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    /// Spirit ID of the sender
    pub sender: String,
    /// Timestamp when message was sent (ms since epoch)
    pub timestamp: u64,
    /// Optional message ID for tracking
    pub message_id: Option<u64>,
}
```

**TypeScript Equivalent:**

```typescript
interface MessageHeader {
    sender: string;
    timestamp: bigint;
    messageId?: bigint;
}
```

### MessagePayload

Message payload types:

```rust
/// Message payload types
#[derive(Debug, Clone, PartialEq)]
pub enum MessagePayload {
    /// UTF-8 text message
    Text(String),
    /// Binary data
    Binary(Vec<u8>),
    /// Structured JSON-like data
    Structured(StructuredData),
}
```

**TypeScript Equivalent:**

```typescript
type MessagePayload = {
    type: 'text';
    content: string;
} | {
    type: 'binary';
    content: Uint8Array;
} | {
    type: 'structured';
    content: Record<string, unknown>;
};
```

### Message

Complete message structure:

```rust
/// A complete message between Spirits
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Message header with sender info
    pub header: MessageHeader,
    /// Message payload
    pub payload: MessagePayload,
}
```

**TypeScript Equivalent:**

```typescript
interface Message {
    header: MessageHeader;
    payload: MessagePayload;
}
```

### AbiError

Errors that can occur during ABI operations:

```rust
/// Errors that can occur during ABI operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiError {
    /// Unknown host function name
    UnknownHostFunction(String),
    /// Signature mismatch between expected and actual
    SignatureMismatch {
        function: String,
        expected: String,
        actual: String,
    },
    /// ABI version mismatch
    VersionMismatch {
        expected: String,
        actual: String,
    },
    /// Invalid message encoding
    InvalidMessage(String),
    /// Memory access out of bounds
    OutOfBounds {
        offset: u32,
        size: u32,
        memory_size: u32,
    },
    /// Invalid pointer (null or misaligned)
    InvalidPointer(u32),
    /// Buffer too small for operation
    BufferTooSmall {
        required: usize,
        available: usize,
    },
}
```

---

## 4. Host Functions

### I/O Functions

#### `vudo_print`

Print a UTF-8 string without newline.

**Signature:**
```rust
fn vudo_print(ptr: i32, len: i32)
```

**Parameters:**
- `ptr`: Pointer to UTF-8 string in linear memory
- `len`: Length of string in bytes

**Return:** None

**Example (Rust):**
```rust
let msg = "Hello, World!";
let ptr = msg.as_ptr() as i32;
let len = msg.len() as i32;
vudo_print(ptr, len);
```

**Example (DOL):**
```dol
print("Hello, World!")
```

**Error Conditions:**
- Invalid UTF-8 encoding (host logs error, continues)
- Pointer out of bounds (host panics)

---

#### `vudo_println`

Print a UTF-8 string with newline.

**Signature:**
```rust
fn vudo_println(ptr: i32, len: i32)
```

**Parameters:**
- `ptr`: Pointer to UTF-8 string in linear memory
- `len`: Length of string in bytes

**Return:** None

**Example (Rust):**
```rust
let msg = "Hello, World!";
vudo_println(msg.as_ptr() as i32, msg.len() as i32);
```

**Example (DOL):**
```dol
println("Hello, World!")
```

**Error Conditions:**
- Same as `vudo_print`

---

#### `vudo_log`

Structured logging with severity level.

**Signature:**
```rust
fn vudo_log(level: i32, ptr: i32, len: i32)
```

**Parameters:**
- `level`: Log level (0=DEBUG, 1=INFO, 2=WARN, 3=ERROR)
- `ptr`: Pointer to UTF-8 string in linear memory
- `len`: Length of string in bytes

**Return:** None

**Example (Rust):**
```rust
let msg = "Connection established";
vudo_log(LogLevel::Info as i32, msg.as_ptr() as i32, msg.len() as i32);
```

**Example (DOL):**
```dol
log(LogLevel.Info, "Connection established")
log(LogLevel.Error, "Failed to authenticate")
```

**Error Conditions:**
- Invalid log level (host defaults to INFO)
- Invalid UTF-8 (host logs error)

---

#### `vudo_error`

Log an error message (shorthand for `vudo_log` with ERROR level).

**Signature:**
```rust
fn vudo_error(ptr: i32, len: i32)
```

**Parameters:**
- `ptr`: Pointer to UTF-8 string in linear memory
- `len`: Length of string in bytes

**Return:** None

**Example (DOL):**
```dol
error("Critical failure in authentication")
```

---

### Memory Functions

#### `vudo_alloc`

Allocate memory from the host allocator.

**Signature:**
```rust
fn vudo_alloc(size: i32) -> i32
```

**Parameters:**
- `size`: Number of bytes to allocate

**Return:**
- Pointer to allocated memory (non-zero)
- `0` on allocation failure

**Example (Rust):**
```rust
let size = 1024;
let ptr = vudo_alloc(size);
if ptr == 0 {
    panic!("Out of memory");
}
// Use memory at ptr..ptr+size
```

**Example (TypeScript):**
```typescript
const size = 1024;
const ptr = instance.exports.vudo_alloc(size);
if (ptr === 0) {
    throw new Error("Out of memory");
}
```

**Error Conditions:**
- Size is 0 → returns 0
- Size is negative → returns 0
- Allocation fails → returns 0

**Memory Ownership:**
- Caller owns the allocated memory
- Must call `vudo_free(ptr, size)` when done

---

#### `vudo_free`

Free previously allocated memory.

**Signature:**
```rust
fn vudo_free(ptr: i32, size: i32)
```

**Parameters:**
- `ptr`: Pointer to memory to free (from `vudo_alloc`)
- `size`: Size of allocation (must match original)

**Return:** None

**Example (Rust):**
```rust
let ptr = vudo_alloc(1024);
// ... use memory ...
vudo_free(ptr, 1024);
```

**Error Conditions:**
- Freeing null pointer (0) → no-op
- Size mismatch → undefined behavior (may corrupt allocator)
- Double free → undefined behavior

**Important:**
- Always pass the same size used in `vudo_alloc`
- Do not free memory allocated by WASM allocator

---

#### `vudo_realloc`

Reallocate memory (grow or shrink).

**Signature:**
```rust
fn vudo_realloc(ptr: i32, old_size: i32, new_size: i32) -> i32
```

**Parameters:**
- `ptr`: Pointer to existing allocation
- `old_size`: Current size (must match original)
- `new_size`: Desired new size

**Return:**
- Pointer to reallocated memory (may differ from `ptr`)
- `0` on failure (original allocation remains valid)

**Example (Rust):**
```rust
let mut ptr = vudo_alloc(1024);
// ... buffer too small ...
let new_ptr = vudo_realloc(ptr, 1024, 2048);
if new_ptr == 0 {
    // Reallocation failed, ptr still valid
} else {
    ptr = new_ptr;  // Update pointer
}
```

**Error Conditions:**
- `new_size` is 0 → equivalent to `vudo_free(ptr, old_size)`, returns 0
- Allocation fails → returns 0, original memory unchanged
- Invalid `old_size` → undefined behavior

---

### Time Functions

#### `vudo_now`

Get current timestamp in milliseconds since Unix epoch.

**Signature:**
```rust
fn vudo_now() -> i64
```

**Return:**
- Milliseconds since 1970-01-01 00:00:00 UTC

**Example (Rust):**
```rust
let timestamp = vudo_now();
println!("Current time: {}", timestamp);
```

**Example (DOL):**
```dol
let start = now()
// ... do work ...
let elapsed = now() - start
println("Took " + elapsed + " ms")
```

**TypeScript Implementation:**
```typescript
function vudo_now(): bigint {
    return BigInt(Date.now());
}
```

---

#### `vudo_sleep`

Sleep for specified milliseconds (async, yields to runtime).

**Signature:**
```rust
fn vudo_sleep(ms: i32)
```

**Parameters:**
- `ms`: Milliseconds to sleep

**Return:** None (blocks until timeout)

**Example (DOL):**
```dol
println("Starting task")
sleep(1000)  // Sleep for 1 second
println("Task complete")
```

**Error Conditions:**
- Negative `ms` → no-op
- Zero `ms` → yields to scheduler, returns immediately

**Important:**
- This is a cooperative yield, not OS sleep
- Other Spirits can execute during sleep
- Not suitable for real-time guarantees

---

#### `vudo_monotonic_now`

Get monotonic time in nanoseconds for performance measurements.

**Signature:**
```rust
fn vudo_monotonic_now() -> i64
```

**Return:**
- Nanoseconds from an arbitrary reference point
- Guaranteed monotonic (never decreases)

**Example (Rust):**
```rust
let start = vudo_monotonic_now();
// ... perform operation ...
let end = vudo_monotonic_now();
let elapsed_ns = end - start;
println!("Took {} ns", elapsed_ns);
```

**Example (DOL):**
```dol
let start = monotonic_now()
compute_fibonacci(40)
let duration = monotonic_now() - start
println("Computed in " + duration + " ns")
```

**TypeScript Implementation:**
```typescript
function vudo_monotonic_now(): bigint {
    return process.hrtime.bigint();
}
```

---

### Messaging Functions

#### `vudo_send`

Send a message to another Spirit.

**Signature:**
```rust
fn vudo_send(target_ptr: i32, target_len: i32, payload_ptr: i32, payload_len: i32) -> i32
```

**Parameters:**
- `target_ptr`: Pointer to target Spirit ID (UTF-8)
- `target_len`: Length of target ID
- `payload_ptr`: Pointer to message payload (UTF-8)
- `payload_len`: Length of payload

**Return:**
- `ResultCode::Ok` (0) on success
- Error code on failure (see ResultCode)

**Example (Rust):**
```rust
let target = "logger";
let payload = "User logged in";
let result = vudo_send(
    target.as_ptr() as i32,
    target.len() as i32,
    payload.as_ptr() as i32,
    payload.len() as i32,
);
if result != 0 {
    println!("Send failed: {}", result);
}
```

**Example (DOL):**
```dol
let result = send("logger", "User logged in")
if result != ResultCode.Ok {
    error("Failed to send message")
}
```

**Error Conditions:**
- Target not found → `ResultCode::NotFound`
- Payload too large (>1MB) → `ResultCode::InvalidArg`
- Target mailbox full → `ResultCode::Timeout`

---

#### `vudo_recv`

Receive next message from inbox.

**Signature:**
```rust
fn vudo_recv() -> i32
```

**Return:**
- Pointer to message in linear memory (non-zero)
- `0` if no messages pending

**Message Format:**
```
┌─────────────┬────────────────┬───────────┬─────────────┬─────────────┬─────────────┐
│ sender_len  │ sender (UTF-8) │ timestamp │ payload_type│ payload_len │ payload     │
│ (4 bytes)   │ (variable)     │ (8 bytes) │ (1 byte)    │ (4 bytes)   │ (variable)  │
└─────────────┴────────────────┴───────────┴─────────────┴─────────────┴─────────────┘
```

**Example (Rust):**
```rust
let msg_ptr = vudo_recv();
if msg_ptr != 0 {
    // Decode message (see Message Wire Format)
    let message = Message::decode(msg_ptr);
    println!("From {}: {}", message.sender, message.payload);

    // IMPORTANT: Free the message when done
    vudo_free_message(msg_ptr);
}
```

**Example (DOL):**
```dol
if pending() > 0 {
    let msg = recv()
    println("Received: " + msg.payload)
    // Message automatically freed at end of scope
}
```

**Error Conditions:**
- Empty inbox → returns 0
- Message decode error → returns 0, error logged

**Memory Ownership:**
- Caller owns returned message
- Must call `vudo_free_message(ptr)` when done

---

#### `vudo_pending`

Check number of pending messages in inbox.

**Signature:**
```rust
fn vudo_pending() -> i32
```

**Return:**
- Number of messages waiting in inbox

**Example (DOL):**
```dol
println("Inbox: " + pending() + " messages")

while pending() > 0 {
    let msg = recv()
    process(msg)
}
```

---

#### `vudo_broadcast`

Broadcast message to all Spirits in session.

**Signature:**
```rust
fn vudo_broadcast(payload_ptr: i32, payload_len: i32) -> i32
```

**Parameters:**
- `payload_ptr`: Pointer to message payload (UTF-8)
- `payload_len`: Length of payload

**Return:**
- `ResultCode::Ok` (0) on success
- Error code on failure

**Example (DOL):**
```dol
broadcast("System shutting down in 10 seconds")
sleep(10000)
effect(StandardEffect.Terminate, "")
```

**Error Conditions:**
- Payload too large → `ResultCode::InvalidArg`
- No other Spirits → `ResultCode::Ok` (no-op)

---

#### `vudo_free_message`

Free a message received from `vudo_recv`.

**Signature:**
```rust
fn vudo_free_message(msg_ptr: i32)
```

**Parameters:**
- `msg_ptr`: Pointer returned by `vudo_recv`

**Return:** None

**Example:**
```rust
let msg_ptr = vudo_recv();
if msg_ptr != 0 {
    // ... process message ...
    vudo_free_message(msg_ptr);  // CRITICAL: Don't forget!
}
```

---

### Random Functions

#### `vudo_random`

Generate random f64 in range [0, 1).

**Signature:**
```rust
fn vudo_random() -> f64
```

**Return:**
- Random floating-point value in [0, 1)

**Example (DOL):**
```dol
let roll = random() * 6.0 + 1.0  // Random d6: 1-6
println("You rolled: " + roll)
```

**TypeScript Implementation:**
```typescript
function vudo_random(): number {
    return Math.random();
}
```

---

#### `vudo_random_bytes`

Fill buffer with cryptographically secure random bytes.

**Signature:**
```rust
fn vudo_random_bytes(ptr: i32, len: i32)
```

**Parameters:**
- `ptr`: Pointer to buffer to fill
- `len`: Number of bytes to generate

**Return:** None

**Example (Rust):**
```rust
let mut buffer = [0u8; 32];
let ptr = buffer.as_mut_ptr() as i32;
vudo_random_bytes(ptr, 32);
// buffer now contains 32 random bytes
```

**Example (DOL):**
```dol
let key = alloc(32)
random_bytes(key, 32)
// Use key for cryptographic operations
```

**TypeScript Implementation:**
```typescript
function vudo_random_bytes(ptr: number, len: number) {
    const bytes = crypto.getRandomValues(new Uint8Array(len));
    const memory = new Uint8Array(instance.exports.memory.buffer);
    memory.set(bytes, ptr);
}
```

---

### Effect Functions

#### `vudo_emit_effect`

Emit a side effect for host handling.

**Signature:**
```rust
fn vudo_emit_effect(effect_id: i32, payload_ptr: i32, payload_len: i32) -> i32
```

**Parameters:**
- `effect_id`: Effect type (see `StandardEffect`)
- `payload_ptr`: Pointer to effect payload (JSON)
- `payload_len`: Length of payload

**Return:**
- `ResultCode::Ok` (0) if effect handled
- Error code if effect rejected

**Example (DOL):**
```dol
// Read a file
let payload = "{\"path\": \"/data/config.json\"}"
let result = emit_effect(StandardEffect.FsRead, payload)

// Make HTTP request
let http = "{\"url\": \"https://api.example.com/data\"}"
emit_effect(StandardEffect.HttpGet, http)
```

**TypeScript Handler:**
```typescript
function vudo_emit_effect(effect_id: number, payload_ptr: number, payload_len: number): number {
    const payload = decodeString(payload_ptr, payload_len);

    switch (effect_id) {
        case StandardEffect.FsRead:
            const { path } = JSON.parse(payload);
            const data = fs.readFileSync(path, 'utf8');
            // Store result for Spirit to retrieve
            return ResultCode.Ok;

        case StandardEffect.HttpGet:
            const { url } = JSON.parse(payload);
            // Initiate async HTTP request
            return ResultCode.Ok;

        default:
            return ResultCode.NotPermitted;
    }
}
```

**Error Conditions:**
- Unknown effect ID → `ResultCode::InvalidArg`
- Effect not permitted → `ResultCode::NotPermitted`
- Invalid payload → `ResultCode::InvalidArg`

---

#### `vudo_subscribe`

Subscribe to an effect channel.

**Signature:**
```rust
fn vudo_subscribe(channel_ptr: i32, channel_len: i32) -> i32
```

**Parameters:**
- `channel_ptr`: Pointer to channel name (UTF-8)
- `channel_len`: Length of channel name

**Return:**
- `ResultCode::Ok` (0) on success
- Error code on failure

**Example (DOL):**
```dol
// Subscribe to file change notifications
subscribe("fs.watch:/data/config.json")

// Subscribe to HTTP events
subscribe("http.response")
```

---

### Debug Functions

#### `vudo_breakpoint`

Trigger a debugger breakpoint (debug builds only).

**Signature:**
```rust
fn vudo_breakpoint()
```

**Return:** None

**Example (DOL):**
```dol
if DEBUG {
    println("About to process critical section")
    breakpoint()  // Debugger stops here
}
process_data()
```

**TypeScript Implementation:**
```typescript
function vudo_breakpoint() {
    debugger;  // JavaScript debugger statement
}
```

**Production Behavior:**
- No-op in production builds
- Should be compiled out by optimizer

---

#### `vudo_assert`

Assert a condition with message (panics if false).

**Signature:**
```rust
fn vudo_assert(condition: i32, msg_ptr: i32, msg_len: i32)
```

**Parameters:**
- `condition`: 0 = false, non-zero = true
- `msg_ptr`: Pointer to assertion message
- `msg_len`: Length of message

**Return:** None (panics if condition is false)

**Example (DOL):**
```dol
fun divide(a: Int64, b: Int64) -> Int64 {
    assert(b != 0, "Division by zero")
    return a / b
}
```

**TypeScript Implementation:**
```typescript
function vudo_assert(condition: number, msg_ptr: number, msg_len: number) {
    if (condition === 0) {
        const msg = decodeString(msg_ptr, msg_len);
        throw new Error(`Assertion failed: ${msg}`);
    }
}
```

---

#### `vudo_panic`

Panic with message (terminates Spirit immediately).

**Signature:**
```rust
fn vudo_panic(msg_ptr: i32, msg_len: i32)
```

**Parameters:**
- `msg_ptr`: Pointer to panic message
- `msg_len`: Length of message

**Return:** Never returns (terminates execution)

**Example (DOL):**
```dol
fun unreachable() {
    panic("This code should never execute")
}
```

**TypeScript Implementation:**
```typescript
function vudo_panic(msg_ptr: number, msg_len: number): never {
    const msg = decodeString(msg_ptr, msg_len);
    console.error(`PANIC: ${msg}`);
    // Terminate Spirit
    throw new Error(`Spirit panicked: ${msg}`);
}
```

---

## 5. Message Protocol

### Wire Format

Messages in WASM linear memory use this binary layout:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
┌─────────────────────────────────────────────────────────────────┐
│                      sender_len (u32, LE)                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                     sender (UTF-8, variable)                    │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                      timestamp (u64, LE)                        │
│                                                                 │
├───────────────────────────────────┬─────────────────────────────┤
│  payload_type (u8)                │                             │
├───────────────────────────────────┴─────────────────────────────┤
│                      payload_len (u32, LE)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                     payload (variable)                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Field Descriptions

| Field | Type | Size | Description |
|-------|------|------|-------------|
| `sender_len` | u32 | 4 bytes | Length of sender ID in bytes (max 256) |
| `sender` | UTF-8 | variable | Sender Spirit ID |
| `timestamp` | u64 | 8 bytes | Message timestamp (ms since epoch) |
| `payload_type` | u8 | 1 byte | 0=Text, 1=Binary, 2=Structured |
| `payload_len` | u32 | 4 bytes | Length of payload in bytes (max 1MB) |
| `payload` | bytes | variable | Message payload |

### Payload Types

| Type | Value | Encoding | Use Case |
|------|-------|----------|----------|
| Text | 0 | UTF-8 string | Simple text messages |
| Binary | 1 | Raw bytes | File data, images, compressed data |
| Structured | 2 | Custom format | Complex data structures (future) |

### Encoding Example (Rust)

```rust
use std::io::Write;

pub fn encode_message(sender: &str, payload: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // sender_len
    buf.write_all(&(sender.len() as u32).to_le_bytes()).unwrap();

    // sender
    buf.write_all(sender.as_bytes()).unwrap();

    // timestamp (set by runtime, use 0 for now)
    buf.write_all(&0u64.to_le_bytes()).unwrap();

    // payload_type (0 = Text)
    buf.write_all(&[0u8]).unwrap();

    // payload_len
    buf.write_all(&(payload.len() as u32).to_le_bytes()).unwrap();

    // payload
    buf.write_all(payload.as_bytes()).unwrap();

    buf
}
```

### Decoding Example (TypeScript)

```typescript
function decodeMessage(bytes: Uint8Array): Message {
    const view = new DataView(bytes.buffer, bytes.byteOffset);
    const decoder = new TextDecoder();

    let offset = 0;

    // sender_len
    const senderLen = view.getUint32(offset, true);
    offset += 4;

    // sender
    const senderBytes = bytes.slice(offset, offset + senderLen);
    const sender = decoder.decode(senderBytes);
    offset += senderLen;

    // timestamp
    const timestamp = view.getBigUint64(offset, true);
    offset += 8;

    // payload_type
    const payloadType = view.getUint8(offset);
    offset += 1;

    // payload_len
    const payloadLen = view.getUint32(offset, true);
    offset += 4;

    // payload
    const payloadBytes = bytes.slice(offset, offset + payloadLen);
    const payload = payloadType === 0
        ? decoder.decode(payloadBytes)
        : payloadBytes;

    return {
        header: { sender, timestamp },
        payload,
        payloadType,
    };
}
```

### Constraints

- **Sender ID**: Maximum 256 bytes (UTF-8)
- **Payload**: Maximum 1 MB (1,048,576 bytes)
- **Total Message**: Maximum ~1 MB + overhead
- **Encoding**: All integers are little-endian
- **String Encoding**: Always UTF-8

---

## 6. Error Handling

### Error Propagation

Errors propagate across the WASM boundary via:

1. **Return Codes**: Functions return `i32` result codes
2. **Zero Pointers**: Allocation failures return `0`
3. **Panics**: `vudo_panic` terminates execution

### Error Checking Pattern

```rust
// Pattern 1: Check result code
let result = vudo_send(target_ptr, target_len, msg_ptr, msg_len);
if result != ResultCode::Ok as i32 {
    match result {
        ResultCode::NotFound => {
            vudo_error("Target not found".as_ptr() as i32, 16);
        }
        ResultCode::Timeout => {
            vudo_error("Timeout sending message".as_ptr() as i32, 23);
        }
        _ => {
            vudo_panic("Unexpected error".as_ptr() as i32, 16);
        }
    }
}

// Pattern 2: Check null pointer
let ptr = vudo_alloc(1024);
if ptr == 0 {
    vudo_panic("Out of memory".as_ptr() as i32, 13);
}
```

### DOL Error Handling

```dol
// DOL Result type wraps ABI errors
fun send_message(target: String, msg: String) -> Result<(), Error> {
    let result = send(target, msg)

    return if result == ResultCode.Ok {
        Ok(())
    } else {
        Err(Error.SendFailed(result))
    }
}

// Usage with ? operator
fun notify_all(msg: String) -> Result<(), Error> {
    send_message("logger", msg)?
    send_message("monitor", msg)?
    Ok(())
}
```

### Runtime Error Handling (TypeScript)

```typescript
function vudo_send(
    target_ptr: number,
    target_len: number,
    payload_ptr: number,
    payload_len: number
): number {
    try {
        const target = decodeString(target_ptr, target_len);
        const payload = decodeString(payload_ptr, payload_len);

        // Validate
        if (!spirits.has(target)) {
            return ResultCode.NotFound;
        }

        if (payload.length > MAX_PAYLOAD_SIZE) {
            return ResultCode.InvalidArg;
        }

        // Send message
        messageBus.send(target, payload);
        return ResultCode.Ok;

    } catch (error) {
        console.error('vudo_send error:', error);
        return ResultCode.Error;
    }
}
```

### Best Practices

1. **Always check return codes** for fallible functions
2. **Validate pointers** before dereferencing
3. **Use assertions** for invariants
4. **Log errors** before panicking
5. **Clean up resources** even on error paths

---

## 7. Memory Model

### Linear Memory

WASM linear memory is a contiguous byte array shared between Spirit and host:

```
┌─────────────────────────────────────────────────────────────────┐
│                     WASM Linear Memory                          │
│                                                                 │
│  ┌───────────┬───────────┬───────────┬───────────┬───────────┐ │
│  │  WASM     │   Host    │   Stack   │   Heap    │  Unused   │ │
│  │  Globals  │  Allocs   │           │           │           │ │
│  └───────────┴───────────┴───────────┴───────────┴───────────┘ │
│       │            │           │           │           │         │
│      0x0        0x1000     0x10000    0x20000   0x100000       │
└─────────────────────────────────────────────────────────────────┘
```

### Allocation Regions

- **WASM Globals** (0x0-0x1000): WASM global variables
- **Host Allocations** (0x1000-0x10000): `vudo_alloc` allocations
- **WASM Stack** (0x10000-0x20000): Call stack
- **WASM Heap** (0x20000+): WASM allocator (dlmalloc, etc.)

### Ownership Rules

1. **WASM-allocated memory**: Managed by WASM allocator
   - Used for local variables, struct instances
   - Never pass to `vudo_free`

2. **Host-allocated memory**: Managed by host allocator
   - Allocated with `vudo_alloc`
   - Must free with `vudo_free`
   - Example: Received messages from `vudo_recv`

3. **String passing**: Caller owns the buffer
   - DOL compiler allocates string in WASM memory
   - Host reads, does not take ownership
   - WASM deallocates after call returns

### Memory Safety

#### Safe Pattern

```rust
// Allocate from host
let ptr = vudo_alloc(1024);
if ptr == 0 {
    return;  // Handle error
}

// Use memory
let slice = unsafe {
    std::slice::from_raw_parts_mut(ptr as *mut u8, 1024)
};
slice[0] = 42;

// Free when done
vudo_free(ptr, 1024);
```

#### Unsafe Patterns (DON'T DO THIS)

```rust
// ❌ WRONG: Double free
let ptr = vudo_alloc(1024);
vudo_free(ptr, 1024);
vudo_free(ptr, 1024);  // CRASH!

// ❌ WRONG: Size mismatch
let ptr = vudo_alloc(1024);
vudo_free(ptr, 2048);  // Corrupts allocator

// ❌ WRONG: Freeing WASM memory with host allocator
let wasm_ptr = Box::new(42);
vudo_free(Box::into_raw(wasm_ptr) as i32, 8);  // CRASH!

// ❌ WRONG: Use after free
let ptr = vudo_alloc(1024);
vudo_free(ptr, 1024);
vudo_print(ptr, 10);  // Undefined behavior
```

### String Encoding Contract

All strings passed across ABI boundary **must be valid UTF-8**:

**WASM → Host:**
```rust
let msg = "Hello, 世界";  // Valid UTF-8
vudo_println(msg.as_ptr() as i32, msg.len() as i32);
```

**Host → WASM:**
```typescript
function sendStringToWasm(str: string, ptr: number) {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str);  // UTF-8
    const memory = new Uint8Array(instance.exports.memory.buffer);
    memory.set(bytes, ptr);
}
```

**Invalid UTF-8 Handling:**
- Host validates UTF-8 before decoding
- Invalid UTF-8 → error logged, operation fails
- No crashes or security vulnerabilities

---

## 8. Best Practices

### For Spirit Developers

#### 1. Always Check Return Codes

```dol
// ✅ GOOD
let result = send("logger", "event")
if result != ResultCode.Ok {
    error("Send failed: " + result)
}

// ❌ BAD: Ignoring errors
send("logger", "event")
```

#### 2. Free Resources

```dol
// ✅ GOOD
let msg_ptr = recv()
if msg_ptr != 0 {
    process(msg_ptr)
    free_message(msg_ptr)  // Clean up!
}

// ❌ BAD: Memory leak
let msg_ptr = recv()
if msg_ptr != 0 {
    process(msg_ptr)
    // Forgot to free!
}
```

#### 3. Validate Inputs

```dol
fun divide(a: Int64, b: Int64) -> Int64 {
    // ✅ GOOD: Validate before dividing
    assert(b != 0, "Division by zero")
    return a / b
}
```

#### 4. Use Appropriate Log Levels

```dol
// ✅ GOOD: Structured logging
log(LogLevel.Debug, "Processing request #123")
log(LogLevel.Info, "Request completed successfully")
log(LogLevel.Warn, "Rate limit approaching")
log(LogLevel.Error, "Database connection failed")

// ❌ BAD: Everything is an error
error("Processing request")  // Not an error!
```

#### 5. Handle Message Decoding Gracefully

```dol
if pending() > 0 {
    let msg = recv()
    if msg != null {
        try {
            process(msg)
        } catch (e) {
            error("Failed to process message: " + e)
        }
        free_message(msg)
    }
}
```

### For Runtime Implementers

#### 1. Validate All Pointers

```typescript
function decodeString(ptr: number, len: number): string {
    const memory = new Uint8Array(instance.exports.memory.buffer);

    // ✅ GOOD: Bounds checking
    if (ptr + len > memory.length) {
        throw new Error('Pointer out of bounds');
    }

    const bytes = memory.slice(ptr, ptr + len);
    return new TextDecoder().decode(bytes);
}
```

#### 2. Track Allocations (Debug Mode)

```typescript
const allocations = new Map<number, number>();

function vudo_alloc(size: number): number {
    const ptr = allocate(size);
    if (ptr !== 0 && DEBUG) {
        allocations.set(ptr, size);
        console.log(`[ALLOC] ${ptr} (${size} bytes)`);
    }
    return ptr;
}

function vudo_free(ptr: number, size: number) {
    if (DEBUG && allocations.has(ptr)) {
        const allocSize = allocations.get(ptr)!;
        if (allocSize !== size) {
            console.error(`[FREE] Size mismatch: alloc=${allocSize}, free=${size}`);
        }
        allocations.delete(ptr);
    }
    deallocate(ptr, size);
}
```

#### 3. Implement Timeouts

```typescript
function vudo_send(...): number {
    const timeout = 5000;  // 5 seconds

    try {
        const sent = messageBus.send(target, payload, { timeout });
        return sent ? ResultCode.Ok : ResultCode.Timeout;
    } catch (error) {
        return ResultCode.Error;
    }
}
```

#### 4. Rate Limit Effects

```typescript
const effectLimits = {
    [StandardEffect.HttpGet]: 100,  // per minute
    [StandardEffect.FsRead]: 1000,
};

function vudo_emit_effect(effect_id: number, ...): number {
    if (!checkRateLimit(effect_id)) {
        return ResultCode.NotPermitted;
    }

    return handleEffect(effect_id, ...);
}
```

### For Compiler Implementers

#### 1. Minimize String Allocations

```rust
// ✅ GOOD: Reuse buffer
let mut buffer = String::with_capacity(256);
for i in 0..10 {
    buffer.clear();
    write!(buffer, "Iteration {}", i).unwrap();
    vudo_println(buffer.as_ptr() as i32, buffer.len() as i32);
}

// ❌ BAD: New allocation each time
for i in 0..10 {
    let msg = format!("Iteration {}", i);
    vudo_println(msg.as_ptr() as i32, msg.len() as i32);
}
```

#### 2. Mark Effectful Functions

```dol
// Compiler infers println is effectful
fun log_event(msg: String) {  // ← Automatically marked effectful
    println(msg)
}

// Pure function (no host calls)
fun add(a: Int64, b: Int64) -> Int64 {  // ← Pure
    return a + b
}
```

#### 3. Optimize Message Passing

```rust
// ✅ GOOD: Batch messages
let messages = vec!["msg1", "msg2", "msg3"];
let batch = messages.join("\n");
vudo_broadcast(batch.as_ptr() as i32, batch.len() as i32);

// ❌ BAD: Individual sends
for msg in &messages {
    vudo_broadcast(msg.as_ptr() as i32, msg.len() as i32);
}
```

### Common Anti-Patterns

| Anti-Pattern | Why Bad | Better Approach |
|-------------|---------|-----------------|
| Ignoring return codes | Silent failures | Always check, log errors |
| String allocation in loops | Performance | Reuse buffers |
| Polling `pending()` in tight loop | CPU waste | Use `sleep()` between checks |
| Large messages (>100KB) | Latency | Stream or chunk data |
| Frequent `alloc`/`free` | Fragmentation | Pool allocations |

---

## 9. Examples

### Example 1: Hello World

**DOL Source:**
```dol
fun main() {
    println("Hello from DOL Spirit!")
}
```

**Generated WASM:**
```wasm
(module
  (import "vudo" "vudo_println" (func $vudo_println (param i32 i32)))
  (memory (export "memory") 1)

  (data (i32.const 0) "Hello from DOL Spirit!")

  (func (export "main")
    (call $vudo_println
      (i32.const 0)     ;; String pointer
      (i32.const 22))   ;; String length
  )
)
```

### Example 2: Fibonacci with Logging

**DOL Source:**
```dol
fun fib(n: Int64) -> Int64 {
    if n <= 1 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

fun main() {
    let start = monotonic_now()
    let result = fib(40)
    let duration = monotonic_now() - start

    println("fib(40) = " + result)
    log(LogLevel.Info, "Computed in " + duration + " ns")
}
```

**Rust Equivalent:**
```rust
fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main() {
    let start = vudo_monotonic_now();
    let result = fib(40);
    let duration = vudo_monotonic_now() - start;

    let msg = format!("fib(40) = {}", result);
    vudo_println(msg.as_ptr() as i32, msg.len() as i32);

    let log_msg = format!("Computed in {} ns", duration);
    vudo_log(
        LogLevel::Info as i32,
        log_msg.as_ptr() as i32,
        log_msg.len() as i32,
    );
}
```

### Example 3: Message Passing

**DOL Source:**
```dol
spirit Logger {
    fun handle_message(msg: Message) {
        println("[LOG] " + msg.sender + ": " + msg.payload)
    }

    fun run() {
        while true {
            if pending() > 0 {
                let msg = recv()
                handle_message(msg)
            } else {
                sleep(100)  // Wait for messages
            }
        }
    }
}

spirit Client {
    fun run() {
        send("logger", "Client started")

        // Do some work
        let result = compute()

        send("logger", "Computation complete: " + result)
    }
}
```

**TypeScript Runtime:**
```typescript
class MessageBus {
    private mailboxes = new Map<string, Message[]>();

    send(target: string, sender: string, payload: string): ResultCode {
        if (!this.mailboxes.has(target)) {
            return ResultCode.NotFound;
        }

        const message: Message = {
            header: {
                sender,
                timestamp: BigInt(Date.now()),
            },
            payload,
            payloadType: MessagePayloadType.Text,
        };

        this.mailboxes.get(target)!.push(message);
        return ResultCode.Ok;
    }

    recv(spiritId: string): Uint8Array | null {
        const mailbox = this.mailboxes.get(spiritId);
        if (!mailbox || mailbox.length === 0) {
            return null;
        }

        const message = mailbox.shift()!;
        return encodeMessage(message);
    }
}
```

### Example 4: HTTP Request Effect

**DOL Source:**
```dol
fun fetch_data(url: String) -> Result<String, Error> {
    let payload = "{\"url\": \"" + url + "\", \"method\": \"GET\"}"

    let result = emit_effect(StandardEffect.HttpGet, payload)
    if result != ResultCode.Ok {
        return Err(Error.HttpFailed(result))
    }

    // Wait for response (simplified)
    sleep(1000)

    // Response available in effect channel
    let response = read_effect_result()
    return Ok(response)
}
```

**TypeScript Effect Handler:**
```typescript
async function handleHttpGet(payload: string): Promise<ResultCode> {
    const { url } = JSON.parse(payload);

    try {
        const response = await fetch(url);
        const data = await response.text();

        // Store result for Spirit to retrieve
        effectResults.set('http.last_response', data);

        return ResultCode.Ok;
    } catch (error) {
        console.error('HTTP GET failed:', error);
        return ResultCode.Error;
    }
}
```

### Example 5: Memory-Intensive Operation

**DOL Source:**
```dol
fun process_large_file(size: Int64) {
    // Allocate buffer
    let buffer = alloc(size)
    if buffer == 0 {
        panic("Failed to allocate " + size + " bytes")
    }

    // Fill with random data
    random_bytes(buffer, size)

    // Process data
    process_buffer(buffer, size)

    // Clean up
    free(buffer, size)
}
```

**Rust Equivalent:**
```rust
fn process_large_file(size: i32) {
    // Allocate from host
    let buffer = vudo_alloc(size);
    if buffer == 0 {
        vudo_panic("Allocation failed".as_ptr() as i32, 17);
    }

    // Fill with random data
    vudo_random_bytes(buffer, size);

    // Process (unsafe pointer manipulation)
    unsafe {
        let slice = std::slice::from_raw_parts_mut(buffer as *mut u8, size as usize);
        // ... process slice ...
    }

    // CRITICAL: Free the buffer
    vudo_free(buffer, size);
}
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-02-04 | Initial specification with 22 host functions |

---

## Appendix: Complete Function Reference

### Quick Reference Table

| Function | Category | Signature | Returns |
|----------|----------|-----------|---------|
| `vudo_print` | I/O | `(i32, i32)` | void |
| `vudo_println` | I/O | `(i32, i32)` | void |
| `vudo_log` | I/O | `(i32, i32, i32)` | void |
| `vudo_error` | I/O | `(i32, i32)` | void |
| `vudo_alloc` | Memory | `(i32)` | i32 |
| `vudo_free` | Memory | `(i32, i32)` | void |
| `vudo_realloc` | Memory | `(i32, i32, i32)` | i32 |
| `vudo_now` | Time | `()` | i64 |
| `vudo_sleep` | Time | `(i32)` | void |
| `vudo_monotonic_now` | Time | `()` | i64 |
| `vudo_send` | Messaging | `(i32, i32, i32, i32)` | i32 |
| `vudo_recv` | Messaging | `()` | i32 |
| `vudo_pending` | Messaging | `()` | i32 |
| `vudo_broadcast` | Messaging | `(i32, i32)` | i32 |
| `vudo_free_message` | Messaging | `(i32)` | void |
| `vudo_random` | Random | `()` | f64 |
| `vudo_random_bytes` | Random | `(i32, i32)` | void |
| `vudo_emit_effect` | Effects | `(i32, i32, i32)` | i32 |
| `vudo_subscribe` | Effects | `(i32, i32)` | i32 |
| `vudo_breakpoint` | Debug | `()` | void |
| `vudo_assert` | Debug | `(i32, i32, i32)` | void |
| `vudo_panic` | Debug | `(i32, i32)` | never |

---

## License

This specification is part of the DOL project and is licensed under MIT OR Apache-2.0.
