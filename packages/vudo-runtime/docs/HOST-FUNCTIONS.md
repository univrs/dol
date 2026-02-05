# Host Functions Reference

Complete documentation for all 22 host functions provided by `@vudo/runtime`.

## Overview

Host functions are imported by WASM Spirits from the `"vudo"` namespace. They provide essential capabilities for I/O, memory management, timing, messaging, randomness, effects, and debugging.

All host functions follow these conventions:
- Strings are passed as `(ptr: i32, len: i32)` pointer-length pairs
- Strings are encoded as UTF-8
- Memory pointers are i32 offsets into linear memory
- Return codes use `0` for success, non-zero for errors
- All integers are little-endian

---

## I/O Functions (4)

### vudo_print

**Signature**: `(ptr: i32, len: i32) -> void`

Print a UTF-8 string to the host console without adding a newline.

**Parameters**:
- `ptr` - Pointer to string data in WASM linear memory
- `len` - Length of string in bytes

**Behavior**:
- Reads `len` bytes from memory starting at `ptr`
- Decodes bytes as UTF-8
- Outputs to host console (stdout) without newline
- Invalid UTF-8 sequences cause undefined behavior

**Error Conditions**:
- Out-of-bounds pointer causes memory access error
- Invalid UTF-8 may produce garbled output or throw

**Example (WASM)**:
```wat
(func $print_hello
  ;; String "Hello" at offset 0
  i32.const 0
  i32.const 5
  call $vudo_print
)
```

**Example (TypeScript Test)**:
```typescript
const memory = new WasmMemory(spirit.exports.memory);
memory.writeString(0, "Hello");

spirit.call('print_hello'); // Prints: Hello (no newline)
```

---

### vudo_println

**Signature**: `(ptr: i32, len: i32) -> void`

Print a UTF-8 string to the host console with a trailing newline.

**Parameters**:
- `ptr` - Pointer to string data in WASM linear memory
- `len` - Length of string in bytes

**Behavior**:
- Reads `len` bytes from memory starting at `ptr`
- Decodes bytes as UTF-8
- Outputs to host console (stdout) with newline
- Equivalent to `vudo_print` followed by `\n`

**Error Conditions**:
- Out-of-bounds pointer causes memory access error
- Invalid UTF-8 may produce garbled output or throw

**Example (WASM)**:
```wat
(func $print_line
  ;; String "Hello, World!" at offset 0
  i32.const 0
  i32.const 13
  call $vudo_println
)
```

**Example (TypeScript Test)**:
```typescript
const mockLogger = new MockLogger();
const spirit = await loadSpirit(wasmBytes, { logger: mockLogger });

spirit.call('print_line');

expect(mockLogger.outputs).toContain('Hello, World!\n');
```

---

### vudo_log

**Signature**: `(level: i32, ptr: i32, len: i32) -> void`

Write a structured log message with a severity level.

**Parameters**:
- `level` - Log level (0=DEBUG, 1=INFO, 2=WARN, 3=ERROR)
- `ptr` - Pointer to message data in WASM linear memory
- `len` - Length of message in bytes

**Log Levels**:
```typescript
enum LogLevel {
  DEBUG = 0,  // Development-only messages
  INFO  = 1,  // Informational messages
  WARN  = 2,  // Warning conditions
  ERROR = 3,  // Error conditions
}
```

**Behavior**:
- Reads `len` bytes from memory starting at `ptr`
- Decodes bytes as UTF-8
- Logs message at specified level
- Host may filter logs based on configured level
- Invalid levels are treated as ERROR

**Error Conditions**:
- Out-of-bounds pointer causes memory access error
- Invalid UTF-8 may produce garbled output

**Example (WASM)**:
```wat
(func $log_warning
  ;; LogLevel::WARN = 2
  i32.const 2
  ;; Message "Temperature high" at offset 0
  i32.const 0
  i32.const 16
  call $vudo_log
)
```

**Example (TypeScript Test)**:
```typescript
const mockLogger = new MockLogger();
const spirit = await loadSpirit(wasmBytes, { logger: mockLogger });

spirit.call('log_warning');

expect(mockLogger.messages).toContainEqual({
  level: LogLevel.WARN,
  message: 'Temperature high'
});
```

---

### vudo_error

**Signature**: `(ptr: i32, len: i32) -> void`

Log an error message to the host error stream.

**Parameters**:
- `ptr` - Pointer to error message in WASM linear memory
- `len` - Length of message in bytes

**Behavior**:
- Equivalent to `vudo_log(LogLevel.ERROR, ptr, len)`
- Logs to stderr or error log stream
- Typically logged with maximum visibility
- Does not terminate execution (use `vudo_panic` for that)

**Error Conditions**:
- Out-of-bounds pointer causes memory access error
- Invalid UTF-8 may produce garbled output

**Example (WASM)**:
```wat
(func $report_error
  ;; Error message "File not found" at offset 0
  i32.const 0
  i32.const 14
  call $vudo_error
)
```

**Example (TypeScript Test)**:
```typescript
const mockLogger = new MockLogger();
const spirit = await loadSpirit(wasmBytes, { logger: mockLogger });

spirit.call('report_error');

expect(mockLogger.errors).toContain('File not found');
```

---

## Memory Functions (3)

### vudo_alloc

**Signature**: `(size: i32) -> i32`

Allocate memory from the host allocator.

**Parameters**:
- `size` - Number of bytes to allocate

**Returns**:
- Pointer to allocated memory on success
- `0` on failure (out of memory)

**Behavior**:
- Allocates a contiguous block of `size` bytes
- Memory is zero-initialized
- Returns pointer to start of block
- Uses bump-pointer allocator (no in-place reuse)
- Allocations are word-aligned (8-byte aligned)

**Error Conditions**:
- Returns `0` if allocation fails
- Negative size is undefined behavior
- Very large allocations may fail

**Example (WASM)**:
```wat
(func $allocate_buffer (result i32)
  ;; Allocate 256 bytes
  i32.const 256
  call $vudo_alloc

  ;; Check for allocation failure
  local.tee $ptr
  i32.const 0
  i32.eq
  if
    ;; Allocation failed - panic
    i32.const 0
    i32.const 16
    call $vudo_panic
  end

  local.get $ptr
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

const ptr = spirit.call('allocate_buffer');

expect(ptr).toBeGreaterThan(0);
expect(ptr % 8).toBe(0); // 8-byte aligned
```

---

### vudo_free

**Signature**: `(ptr: i32, size: i32) -> void`

Free previously allocated memory.

**Parameters**:
- `ptr` - Pointer to previously allocated memory
- `size` - Size of the block (must match allocation)

**Behavior**:
- Returns memory block to host allocator
- For bump allocator, this is typically a no-op
- Must pass exact size from original allocation
- Double-free is undefined behavior
- Freeing invalid pointers is undefined behavior

**Error Conditions**:
- Invalid pointer or wrong size causes undefined behavior
- No error is reported

**Example (WASM)**:
```wat
(func $use_temp_buffer
  ;; Allocate
  i32.const 256
  call $vudo_alloc
  local.set $ptr

  ;; ... use buffer ...

  ;; Free (must pass exact size)
  local.get $ptr
  i32.const 256
  call $vudo_free
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

spirit.call('use_temp_buffer');

// For bump allocator, free is a no-op
// For other allocators, memory is reclaimed
```

---

### vudo_realloc

**Signature**: `(ptr: i32, oldSize: i32, newSize: i32) -> i32`

Reallocate memory (grow or shrink an existing block).

**Parameters**:
- `ptr` - Pointer to previously allocated memory
- `oldSize` - Current size of the block
- `newSize` - Desired new size in bytes

**Returns**:
- New pointer to reallocated memory on success
- `0` on failure (original block remains valid)

**Behavior**:
- Resizes existing allocation to `newSize`
- If `newSize > oldSize`, new bytes are zero-initialized
- If `newSize < oldSize`, excess data is discarded
- May move data to new location (update all pointers)
- Original data preserved up to `min(oldSize, newSize)`
- On failure, original block remains valid at `ptr`

**Error Conditions**:
- Returns `0` if reallocation fails
- Original pointer must be from `vudo_alloc` or `vudo_realloc`
- `oldSize` must match current allocation size

**Example (WASM)**:
```wat
(func $grow_buffer
  ;; Allocate 256 bytes
  i32.const 256
  call $vudo_alloc
  local.set $ptr

  ;; ... fill buffer ...

  ;; Need more space - grow to 512 bytes
  local.get $ptr
  i32.const 256
  i32.const 512
  call $vudo_realloc

  ;; Check for failure
  local.tee $new_ptr
  i32.const 0
  i32.eq
  if
    ;; Realloc failed - original ptr still valid
    ;; Handle error...
  else
    ;; Success - update ptr
    local.set $ptr
  end
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

spirit.call('grow_buffer');

// Verify that buffer was grown successfully
const result = spirit.call('get_buffer_size');
expect(result).toBe(512);
```

---

## Time Functions (3)

### vudo_now

**Signature**: `() -> i64`

Get the current timestamp in milliseconds since the Unix epoch.

**Returns**:
- Timestamp in milliseconds (i64)

**Behavior**:
- Returns milliseconds since 00:00:00 UTC on 1 January 1970
- Resolution is milliseconds (not nanoseconds)
- Uses `Date.now()` on JavaScript platforms
- Suitable for timeout calculations and timestamps
- May have platform-dependent precision

**Error Conditions**:
- None (always succeeds)

**Example (WASM)**:
```wat
(func $get_timeout (result i64)
  ;; Get current time
  call $vudo_now

  ;; Add 5 seconds (5000 ms)
  i64.const 5000
  i64.add
)
```

**Example (TypeScript Test)**:
```typescript
const mockTime = new MockTimeProvider();
mockTime.setTime(1000000n);

const spirit = await loadSpirit(wasmBytes, { timeProvider: mockTime });

const timeout = spirit.call('get_timeout');
expect(timeout).toBe(1005000n);
```

---

### vudo_sleep

**Signature**: `(ms: i32) -> void`

Sleep for a specified number of milliseconds.

**Parameters**:
- `ms` - Duration in milliseconds to sleep

**Behavior**:
- Blocks the calling Spirit for `ms` milliseconds
- Implemented as async in runtime (allows other Spirits to run)
- Sleep duration is approximate
- Negative values are treated as 0 (return immediately)
- May be interrupted by host shutdown

**Error Conditions**:
- None (always succeeds)

**Example (WASM)**:
```wat
(func $rate_limited_loop
  (loop $continue
    ;; Do work...

    ;; Sleep 100ms between iterations
    i32.const 100
    call $vudo_sleep

    ;; Continue loop
    br $continue
  )
)
```

**Example (TypeScript Test)**:
```typescript
const mockTime = new MockTimeProvider();

const spirit = await loadSpirit(wasmBytes, { timeProvider: mockTime });

const start = Date.now();
await spirit.call('sleep_one_second'); // Sleeps 1000ms
const elapsed = Date.now() - start;

expect(elapsed).toBeGreaterThanOrEqual(1000);
```

---

### vudo_monotonic_now

**Signature**: `() -> i64`

Get monotonic time in nanoseconds for precise measurements.

**Returns**:
- Nanoseconds since arbitrary epoch (i64)

**Behavior**:
- Returns monotonically increasing time
- Never goes backward (unlike `vudo_now`)
- Starting epoch is arbitrary (not Unix epoch)
- Resolution is nanoseconds
- Uses `performance.now()` or `process.hrtime.bigint()`
- Suitable for high-precision timing and benchmarks

**Error Conditions**:
- None (always succeeds)

**Example (WASM)**:
```wat
(func $measure_operation (result i64)
  ;; Start timer
  call $vudo_monotonic_now
  local.set $start

  ;; Do work...
  call $expensive_operation

  ;; End timer
  call $vudo_monotonic_now

  ;; Return elapsed nanoseconds
  local.get $start
  i64.sub
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

const elapsed = spirit.call('measure_operation');

// elapsed is in nanoseconds
const elapsedMs = Number(elapsed) / 1_000_000;
console.log(`Operation took ${elapsedMs.toFixed(3)}ms`);
```

---

## Messaging Functions (5)

### vudo_send

**Signature**: `(targetPtr: i32, targetLen: i32, payloadPtr: i32, payloadLen: i32) -> i32`

Send a message to another Spirit.

**Parameters**:
- `targetPtr` - Pointer to target Spirit name (UTF-8)
- `targetLen` - Length of target name in bytes
- `payloadPtr` - Pointer to message payload data
- `payloadLen` - Length of payload in bytes

**Returns**:
- `0` (ResultCode::Ok) on success
- `ResultCode::NotFound` if target Spirit not found
- `ResultCode::Error` if message queue is full

**Behavior**:
- Reads target name from `targetPtr`
- Copies payload to message queue
- Delivery is asynchronous
- Target receives message via `vudo_recv`
- Messages are FIFO ordered

**Error Conditions**:
- Returns non-zero if target not found
- Returns non-zero if queue is full

**Example (WASM)**:
```wat
(func $send_to_worker
  ;; Target: "worker-1" at offset 0
  i32.const 0
  i32.const 8

  ;; Payload: "process" at offset 100
  i32.const 100
  i32.const 7

  call $vudo_send

  ;; Check result
  i32.const 0
  i32.ne
  if
    ;; Send failed
    i32.const 200
    i32.const 11
    call $vudo_error  ;; "Send failed"
  end
)
```

**Example (TypeScript Test)**:
```typescript
const bus = new MessageBus();
const seance = new Seance({ messageBus: bus });

await seance.summon('sender', './sender.wasm');
await seance.summon('receiver', './receiver.wasm');

await seance.invoke('sender', 'send_to_worker');

// Verify message was delivered
const pending = bus.pending('receiver');
expect(pending).toBe(1);
```

---

### vudo_recv

**Signature**: `() -> i32`

Receive the next message from the Spirit's inbox.

**Returns**:
- Pointer to message in WASM memory on success
- `0` if no messages are available

**Message Format**:
```
[sender_len: u32][sender: UTF-8][payload_len: u32][payload: bytes]
```

All integers are little-endian.

**Behavior**:
- Non-blocking: returns immediately
- Returns oldest message in queue
- Message remains in memory until freed
- Caller must call `vudo_free_message` to release
- Returns 0 if inbox is empty

**Error Conditions**:
- Returns `0` if no messages available
- Not an error (just indicates empty inbox)

**Example (WASM)**:
```wat
(func $process_messages
  (loop $continue
    ;; Try to receive a message
    call $vudo_recv
    local.tee $msg_ptr

    ;; Check if message available
    i32.const 0
    i32.eq
    if
      ;; No more messages
      return
    end

    ;; Process message at $msg_ptr
    local.get $msg_ptr
    call $handle_message

    ;; Free message
    local.get $msg_ptr
    call $vudo_free_message

    ;; Check for more messages
    br $continue
  )
)
```

**Example (TypeScript Test)**:
```typescript
const bus = new MessageBus();
bus.register('receiver');
bus.send('sender', 'receiver', 0, new Uint8Array([1, 2, 3]));

const spirit = await loadSpirit(wasmBytes, { messageBus: bus });

const msgPtr = spirit.call('recv_message');
expect(msgPtr).toBeGreaterThan(0);

// Read message format
const memory = new WasmMemory(spirit.exports.memory);
const senderLen = memory.readI32(msgPtr);
const sender = memory.readString(msgPtr + 4, senderLen);
expect(sender).toBe('sender');
```

---

### vudo_pending

**Signature**: `() -> i32`

Check the number of pending messages in the inbox.

**Returns**:
- Number of messages waiting in queue (i32)
- `0` if inbox is empty

**Behavior**:
- Non-blocking
- Returns count of messages
- Does not consume messages
- Count may change after this call
- Useful for busy-waiting or conditional logic

**Error Conditions**:
- None (always succeeds)

**Example (WASM)**:
```wat
(func $wait_for_message
  (loop $poll
    ;; Check if messages available
    call $vudo_pending
    i32.const 0
    i32.gt_u
    if
      ;; Messages available
      return
    end

    ;; Sleep briefly before polling again
    i32.const 10
    call $vudo_sleep

    br $poll
  )
)
```

**Example (TypeScript Test)**:
```typescript
const bus = new MessageBus();
bus.register('receiver');

const spirit = await loadSpirit(wasmBytes, { messageBus: bus });

// No messages initially
expect(spirit.call('check_pending')).toBe(0);

// Send 3 messages
bus.send('sender', 'receiver', 0, new Uint8Array([1]));
bus.send('sender', 'receiver', 0, new Uint8Array([2]));
bus.send('sender', 'receiver', 0, new Uint8Array([3]));

expect(spirit.call('check_pending')).toBe(3);
```

---

### vudo_broadcast

**Signature**: `(ptr: i32, len: i32) -> i32`

Broadcast a message to all Spirits in the session.

**Parameters**:
- `ptr` - Pointer to message payload data
- `len` - Length of payload in bytes

**Returns**:
- `0` (ResultCode::Ok) on success
- `ResultCode::Error` if broadcast failed

**Behavior**:
- Sends message to every Spirit in current Séance
- Including the sender (sender receives own broadcast)
- All Spirits receive identical payload
- Delivery is asynchronous
- Efficient implementation (message copied once)

**Error Conditions**:
- Returns non-zero if broadcast fails
- May fail if queue space insufficient

**Example (WASM)**:
```wat
(func $announce_shutdown
  ;; Payload: "SHUTDOWN" at offset 0
  i32.const 0
  i32.const 8
  call $vudo_broadcast

  ;; Check result
  i32.const 0
  i32.ne
  if
    ;; Broadcast failed
    i32.const 100
    i32.const 16
    call $vudo_error  ;; "Broadcast failed"
  end
)
```

**Example (TypeScript Test)**:
```typescript
const bus = new MessageBus();
const seance = new Seance({ messageBus: bus });

await seance.summon('spirit-a', './a.wasm');
await seance.summon('spirit-b', './b.wasm');
await seance.summon('spirit-c', './c.wasm');

await seance.invoke('spirit-a', 'announce_shutdown');

// All spirits received the broadcast
expect(bus.pending('spirit-a')).toBe(1);
expect(bus.pending('spirit-b')).toBe(1);
expect(bus.pending('spirit-c')).toBe(1);
```

---

### vudo_free_message

**Signature**: `(ptr: i32) -> void`

Free a received message after processing.

**Parameters**:
- `ptr` - Pointer to message (returned from `vudo_recv`)

**Behavior**:
- Releases message back to memory pool
- Must be called exactly once per message
- Message data is invalid after calling
- Double-free is undefined behavior
- Some allocators may treat as no-op

**Error Conditions**:
- Invalid pointer causes undefined behavior
- No error is reported

**Example (WASM)**:
```wat
(func $receive_and_process
  ;; Receive message
  call $vudo_recv
  local.tee $msg_ptr

  ;; Check if available
  i32.const 0
  i32.eq
  if
    return
  end

  ;; Process message
  local.get $msg_ptr
  call $process_message

  ;; MUST free after use
  local.get $msg_ptr
  call $vudo_free_message
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

spirit.call('receive_and_process');

// Message should be freed (no memory leak)
// Use memory profiler to verify
```

---

## Random Functions (2)

### vudo_random

**Signature**: `() -> f64`

Generate a random floating-point number.

**Returns**:
- Random f64 in the range [0.0, 1.0)

**Behavior**:
- Returns uniform random value
- Range excludes 1.0 (0.0 ≤ value < 1.0)
- Uses cryptographic random source if available
- Falls back to Math.random() if not
- Thread-safe in host

**Error Conditions**:
- None (always succeeds)

**Example (WASM)**:
```wat
(func $roll_dice (result i32)
  ;; Get random [0, 1)
  call $vudo_random

  ;; Scale to [0, 6)
  f64.const 6.0
  f64.mul

  ;; Floor and add 1 for [1, 6]
  i32.trunc_f64_u
  i32.const 1
  i32.add
)
```

**Example (TypeScript Test)**:
```typescript
const mockRandom = new MockRandomProvider();
mockRandom.setNextValue(0.5);

const spirit = await loadSpirit(wasmBytes, { randomProvider: mockRandom });

const dice = spirit.call('roll_dice');
expect(dice).toBe(4); // floor(0.5 * 6) + 1 = 3 + 1 = 4
```

---

### vudo_random_bytes

**Signature**: `(ptr: i32, len: i32) -> void`

Fill a buffer with random bytes.

**Parameters**:
- `ptr` - Pointer to buffer in WASM linear memory
- `len` - Number of random bytes to generate

**Behavior**:
- Generates cryptographically secure random bytes
- Uses `crypto.getRandomValues()` on web platforms
- Uses `crypto.randomBytes()` on Node.js
- Writes exactly `len` bytes starting at `ptr`
- Buffer must be pre-allocated

**Error Conditions**:
- Out-of-bounds pointer causes memory access error
- Invalid length causes undefined behavior

**Example (WASM)**:
```wat
(func $generate_uuid
  ;; Allocate 16 bytes for UUID
  i32.const 16
  call $vudo_alloc
  local.set $uuid_ptr

  ;; Fill with random bytes
  local.get $uuid_ptr
  i32.const 16
  call $vudo_random_bytes

  ;; Set version and variant bits (UUID v4)
  ;; ... bit manipulation ...

  local.get $uuid_ptr
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

const uuidPtr = spirit.call('generate_uuid');

const memory = new WasmMemory(spirit.exports.memory);
const bytes = memory.readBytes(uuidPtr, 16);

// Verify UUID v4 format
expect(bytes[6] & 0xF0).toBe(0x40); // Version 4
expect(bytes[8] & 0xC0).toBe(0x80); // Variant 10
```

---

## Effect Functions (2)

### vudo_emit_effect

**Signature**: `(effectId: i32, payloadPtr: i32, payloadLen: i32) -> i32`

Emit a side effect for the host to handle.

**Parameters**:
- `effectId` - Effect type identifier
- `payloadPtr` - Pointer to effect payload data
- `payloadLen` - Length of payload in bytes

**Returns**:
- `0` (ResultCode::Ok) on success
- `ResultCode::NotPermitted` if effect not allowed
- `ResultCode::Error` if effect failed

**Standard Effect IDs**:
```typescript
const EFFECT_NOOP = 0;          // No operation
const EFFECT_TERMINATE = 1;     // Terminate Spirit
const EFFECT_SPAWN = 2;         // Spawn new Spirit
const EFFECT_FS_READ = 10;      // Read file
const EFFECT_FS_WRITE = 11;     // Write file
const EFFECT_HTTP_GET = 20;     // HTTP GET request
const EFFECT_HTTP_POST = 21;    // HTTP POST request
const EFFECT_DB_QUERY = 30;     // Database query
```

**Behavior**:
- Sends effect request to host
- Host processes effect asynchronously
- Custom effects (ID > 255) are supported
- Effect handling is host-dependent
- Results may be delivered via messages

**Error Conditions**:
- Returns non-zero if effect unknown
- Returns non-zero if not permitted
- Returns non-zero if execution fails

**Example (WASM)**:
```wat
(func $read_config_file (result i32)
  ;; Effect: FS_READ (10)
  i32.const 10

  ;; Payload: "config.json" at offset 0
  i32.const 0
  i32.const 11

  call $vudo_emit_effect

  ;; Check result
  local.tee $result
  i32.const 0
  i32.ne
  if
    ;; Effect failed
    i32.const 100
    i32.const 15
    call $vudo_error  ;; "File read error"
  end

  local.get $result
)
```

**Example (TypeScript Test)**:
```typescript
const effectHandler = new MockEffectHandler();

const spirit = await loadSpirit(wasmBytes, { effectHandler });

spirit.call('read_config_file');

expect(effectHandler.effects).toContainEqual({
  id: 10, // EFFECT_FS_READ
  payload: 'config.json'
});
```

---

### vudo_subscribe

**Signature**: `(channelPtr: i32, channelLen: i32) -> i32`

Subscribe to an effect channel.

**Parameters**:
- `channelPtr` - Pointer to channel name (UTF-8)
- `channelLen` - Length of channel name in bytes

**Returns**:
- `0` (ResultCode::Ok) on success
- `ResultCode::NotFound` if channel doesn't exist
- `ResultCode::Error` if subscription failed

**Behavior**:
- Registers Spirit to receive channel notifications
- Multiple Spirits can subscribe to same channel
- Notifications delivered as messages to inbox
- Unsubscribe by terminating Spirit
- Channels are host-defined

**Error Conditions**:
- Returns non-zero if channel not found
- Returns non-zero if subscription fails

**Example (WASM)**:
```wat
(func $watch_file_changes
  ;; Subscribe to "file-events" channel
  i32.const 0
  i32.const 11
  call $vudo_subscribe

  ;; Check result
  i32.const 0
  i32.ne
  if
    ;; Subscribe failed
    i32.const 100
    i32.const 16
    call $vudo_error  ;; "Subscribe failed"
  end

  ;; Now recv() will receive file change notifications
)
```

**Example (TypeScript Test)**:
```typescript
const effectHandler = new MockEffectHandler();
effectHandler.registerChannel('file-events');

const spirit = await loadSpirit(wasmBytes, { effectHandler });

spirit.call('watch_file_changes');

// Emit event on channel
effectHandler.emit('file-events', 'config.json changed');

// Spirit receives notification
const msgPtr = spirit.call('recv_message');
expect(msgPtr).toBeGreaterThan(0);
```

---

## Debug Functions (3)

### vudo_breakpoint

**Signature**: `() -> void`

Trigger a breakpoint (debug builds only).

**Behavior**:
- Signals debugger to pause execution
- Only effective if debugger is attached
- No-op in release builds or without debugger
- Does not affect execution if no debugger present
- Useful for conditional breakpoints

**Error Conditions**:
- None (always succeeds, may be no-op)

**Example (WASM)**:
```wat
(func $debug_on_error
  ;; Check error condition
  call $check_invariant
  i32.eqz
  if
    ;; Invariant violated - break here
    call $vudo_breakpoint

    ;; Continue with error handling
    call $handle_error
  end
)
```

**Example (TypeScript Test)**:
```typescript
// Not testable in unit tests
// Requires debugger attachment
// In practice, this is no-op in test environment
```

---

### vudo_assert

**Signature**: `(condition: i32, ptr: i32, len: i32) -> void`

Assert a condition with an optional message.

**Parameters**:
- `condition` - Condition to assert (0=false, non-zero=true)
- `ptr` - Pointer to assertion message (UTF-8)
- `len` - Length of message in bytes

**Behavior**:
- If `condition` is non-zero (true), succeeds silently
- If `condition` is zero (false), logs error message
- Host may terminate Spirit on assertion failure
- May throw exception depending on host config
- Message is decoded as UTF-8

**Error Conditions**:
- Assertion failure logs error and may terminate
- Out-of-bounds pointer causes memory access error

**Example (WASM)**:
```wat
(func $checked_divide (param $a i64) (param $b i64) (result i64)
  ;; Assert divisor is not zero
  local.get $b
  i64.const 0
  i64.ne
  i32.const 0
  i32.const 19
  call $vudo_assert  ;; "Divisor must be > 0"

  ;; Safe to divide
  local.get $a
  local.get $b
  i64.div_s
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

// This should succeed
expect(() => spirit.call('checked_divide', [10n, 2n])).not.toThrow();

// This should fail assertion
expect(() => spirit.call('checked_divide', [10n, 0n])).toThrow('Divisor must be > 0');
```

---

### vudo_panic

**Signature**: `(ptr: i32, len: i32) -> void`

Panic with an error message (terminates the Spirit).

**Parameters**:
- `ptr` - Pointer to panic message (UTF-8)
- `len` - Length of message in bytes

**Behavior**:
- Immediately terminates Spirit execution
- Logs panic message
- This function never returns
- Host cleans up Spirit resources
- Other Spirits in Séance continue running
- Unrecoverable error (cannot be caught)

**Error Conditions**:
- Always terminates (no recovery)
- Out-of-bounds pointer causes memory access error before panic

**Example (WASM)**:
```wat
(func $fail_critical
  ;; Critical error - cannot continue
  i32.const 0
  i32.const 20
  call $vudo_panic  ;; "Critical system error"

  ;; This code never executes
  unreachable
)
```

**Example (TypeScript Test)**:
```typescript
const spirit = await loadSpirit(wasmBytes);

expect(() => spirit.call('fail_critical')).toThrow('Critical system error');

// Spirit is now terminated
expect(() => spirit.call('any_function')).toThrow('Spirit terminated');
```

---

## Summary Table

| Category | Function | Signature | Returns |
|----------|----------|-----------|---------|
| **I/O** | `vudo_print` | `(ptr: i32, len: i32)` | void |
| | `vudo_println` | `(ptr: i32, len: i32)` | void |
| | `vudo_log` | `(level: i32, ptr: i32, len: i32)` | void |
| | `vudo_error` | `(ptr: i32, len: i32)` | void |
| **Memory** | `vudo_alloc` | `(size: i32)` | i32 (ptr) |
| | `vudo_free` | `(ptr: i32, size: i32)` | void |
| | `vudo_realloc` | `(ptr: i32, old: i32, new: i32)` | i32 (ptr) |
| **Time** | `vudo_now` | `()` | i64 |
| | `vudo_sleep` | `(ms: i32)` | void |
| | `vudo_monotonic_now` | `()` | i64 |
| **Messaging** | `vudo_send` | `(to_ptr: i32, to_len: i32, payload_ptr: i32, payload_len: i32)` | i32 |
| | `vudo_recv` | `()` | i32 (ptr) |
| | `vudo_pending` | `()` | i32 |
| | `vudo_broadcast` | `(ptr: i32, len: i32)` | i32 |
| | `vudo_free_message` | `(ptr: i32)` | void |
| **Random** | `vudo_random` | `()` | f64 |
| | `vudo_random_bytes` | `(ptr: i32, len: i32)` | void |
| **Effects** | `vudo_emit_effect` | `(id: i32, ptr: i32, len: i32)` | i32 |
| | `vudo_subscribe` | `(chan_ptr: i32, chan_len: i32)` | i32 |
| **Debug** | `vudo_breakpoint` | `()` | void |
| | `vudo_assert` | `(cond: i32, ptr: i32, len: i32)` | void |
| | `vudo_panic` | `(ptr: i32, len: i32)` | void (never returns) |

---

## See Also

- [README](../README.md) - Runtime overview and quick start
- [PROVIDERS](./PROVIDERS.md) - Provider interfaces documentation
