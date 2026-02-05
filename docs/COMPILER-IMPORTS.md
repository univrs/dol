# DOL Compiler: WASM Import Generation

> **Status:** Implementation Guide
> **Version:** 0.8.1
> **Depends On:** dol-abi (via host.rs)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Import Emitter](#import-emitter)
4. [Call Generation](#call-generation)
5. [Prelude Functions](#prelude-functions)
6. [Examples](#examples)
7. [Testing](#testing)

---

## Overview

The DOL compiler's import generation system transforms DOL source code into WebAssembly bytecode that interfaces with the VUDO runtime through host function imports. This system ensures type-safe, efficient communication between WASM guest code and the host environment.

### Purpose

- **Type Safety**: Ensures correct parameter and return types for all host function calls
- **Optimization**: Emits only the imports actually used by the compiled code
- **ABI Compliance**: Maintains strict adherence to the VUDO host function ABI
- **Memory Safety**: Handles string encoding and memory layout correctly

### Compilation Flow

```text
┌─────────────┐
│ DOL Source  │  println("Hello")
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │  AST: Call(println, [String("Hello")])
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Compiler   │  Tracks: vudo_println needed
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    WASM     │  (import "vudo" "vudo_println" (func (param i32 i32)))
└─────────────┘
```

---

## Architecture

### AST to WASM Import Mapping

The compiler bridges DOL's high-level constructs to low-level WASM imports through several stages:

#### 1. AST Node Recognition

DOL function calls are parsed into AST `Call` expressions:

```rust
// DOL source
println("Hello, World!")

// Becomes AST
Expr::Call {
    callee: Box::new(Expr::Identifier("println")),
    args: vec![Expr::String("Hello, World!")]
}
```

#### 2. Import Detection

The compiler identifies host function calls by name:

```rust
// In WasmCompiler::extract_imports()
match callee_name {
    "println" | "print" | "log" | "error" => {
        // I/O host functions
        imports.push(WasmImport::from_name(callee_name))
    }
    "send" | "recv" | "broadcast" | "pending" => {
        // Messaging host functions
        imports.push(WasmImport::from_name(callee_name))
    }
    // ... more categories
}
```

#### 3. Import Section Generation

Detected imports are emitted in the WASM import section:

```rust
let mut import_section = ImportSection::new();
for import in &imports {
    import_section.import(
        &import.module,        // "vudo"
        &import.name,          // "vudo_println"
        EntityType::Function(import.type_idx)
    );
}
```

### Import Section Flow

```text
┌─────────────────────────────────────────────────────────────┐
│                  WASM Module Structure                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. TYPE SECTION                                            │
│     ┌─────────────────────────────────────┐                │
│     │ (type $vudo_println (param i32 i32))│                │
│     │ (type $vudo_send (param i32 i32 i32 i32) (result i32))
│     │ ...                                  │                │
│     └─────────────────────────────────────┘                │
│                                                             │
│  2. IMPORT SECTION                                          │
│     ┌──────────────────────────────────────┐               │
│     │ (import "vudo" "vudo_println"        │               │
│     │   (func $vudo_println (type 0)))     │               │
│     │ (import "vudo" "vudo_send"           │               │
│     │   (func $vudo_send (type 1)))        │               │
│     │ ...                                   │               │
│     └──────────────────────────────────────┘               │
│                                                             │
│  3. FUNCTION SECTION                                        │
│     ┌──────────────────────────────────────┐               │
│     │ (func $main ...)                     │               │
│     │ (func $helper ...)                   │               │
│     └──────────────────────────────────────┘               │
│                                                             │
│  4. MEMORY SECTION                                          │
│     ┌──────────────────────────────────────┐               │
│     │ (memory 1)                           │               │
│     └──────────────────────────────────────┘               │
│                                                             │
│  5. DATA SECTION                                            │
│     ┌──────────────────────────────────────┐               │
│     │ (data (i32.const 0) "Hello, World!") │               │
│     └──────────────────────────────────────┘               │
│                                                             │
│  6. CODE SECTION                                            │
│     ┌──────────────────────────────────────┐               │
│     │ (func $main                          │               │
│     │   i32.const 0    ;; string ptr       │               │
│     │   i32.const 13   ;; string len       │               │
│     │   call $vudo_println                 │               │
│     │ )                                    │               │
│     └──────────────────────────────────────┘               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Prelude Function Mapping

DOL provides a standard prelude of built-in functions that map to VUDO host functions:

```rust
// Internal mapping table (conceptual)
static PRELUDE_MAP: &[(&str, &str)] = &[
    // DOL name    →  VUDO host function
    ("println",     "vudo_println"),
    ("print",       "vudo_print"),
    ("log",         "vudo_log"),
    ("error",       "vudo_error"),
    ("alloc",       "vudo_alloc"),
    ("free",        "vudo_free"),
    ("now",         "vudo_now"),
    ("sleep",       "vudo_sleep"),
    ("send",        "vudo_send"),
    ("recv",        "vudo_recv"),
    // ... 12 more functions
];
```

---

## Import Emitter

### WasmImport Structure

The `WasmImport` struct represents a single host function import:

```rust
#[derive(Debug, Clone)]
struct WasmImport {
    /// Module name (always "vudo" for host functions)
    module: String,

    /// Function name (e.g., "vudo_println")
    name: String,

    /// Parameter types
    params: Vec<wasm_encoder::ValType>,

    /// Return type (None for void functions)
    result: Option<wasm_encoder::ValType>,

    /// Type index in the type section (set during compilation)
    type_idx: u32,
}
```

### Import Section API

#### Building Imports

```rust
impl WasmCompiler {
    /// Extract required imports from declarations
    fn extract_imports(
        &self,
        declarations: &[Declaration]
    ) -> Result<Vec<WasmImport>, WasmError> {
        let mut imports = Vec::new();

        // Traverse AST to find function calls
        for decl in declarations {
            self.visit_declaration(decl, &mut imports)?;
        }

        // Deduplicate (each import only once)
        imports.sort_by(|a, b| a.name.cmp(&b.name));
        imports.dedup_by(|a, b| a.name == b.name);

        Ok(imports)
    }
}
```

#### Emitting Import Section

```rust
// In compile() method:
if !imports.is_empty() {
    let mut import_section = ImportSection::new();

    for import in &imports {
        import_section.import(
            &import.module,              // "vudo"
            &import.name,                // "vudo_println"
            EntityType::Function(import.type_idx)
        );
    }

    wasm_module.section(&import_section);
}
```

### Import Tracking

The compiler maintains import tracking state during code generation:

```rust
struct ImportTracker {
    /// Map from function name to import index
    func_indices: HashMap<String, u32>,

    /// Next available function index
    next_idx: u32,
}

impl ImportTracker {
    /// Register an import and get its index
    fn register(&mut self, name: &str) -> u32 {
        *self.func_indices.entry(name.to_string())
            .or_insert_with(|| {
                let idx = self.next_idx;
                self.next_idx += 1;
                idx
            })
    }

    /// Look up function index by name
    fn get_index(&self, name: &str) -> Option<u32> {
        self.func_indices.get(name).copied()
    }
}
```

### Import Optimization

Only functions actually used in the code are imported:

```rust
// Example: Code uses only println and send
// Generated imports:
(import "vudo" "vudo_println" (func $vudo_println (param i32 i32)))
(import "vudo" "vudo_send" (func $vudo_send (param i32 i32 i32 i32) (result i32)))

// NOT imported: vudo_recv, vudo_alloc, etc.
```

---

## Call Generation

### CallGenerator Methods

The compiler generates WASM call instructions for each category of host functions.

#### I/O Functions (4 functions)

##### vudo_print

**Signature:** `(ptr: i32, len: i32) -> void`

```rust
// DOL: print("Hello")
// Generated WASM:
fn gen_print_call(
    &self,
    builder: &mut InstrSeqBuilder,
    string: &str
) {
    let (ptr, len) = self.string_pool.add(string);

    builder.i32_const(ptr as i32);     // Push pointer
    builder.i32_const(len as i32);     // Push length
    builder.call(self.get_import_idx("vudo_print"));
}
```

**WAT Output:**
```wasm
i32.const 4096    ;; pointer to "Hello" in data section
i32.const 5       ;; length
call $vudo_print
```

##### vudo_println

**Signature:** `(ptr: i32, len: i32) -> void`

Similar to `vudo_print`, but adds a newline.

```rust
// DOL: println("Hello")
// Generated WASM:
i32.const 4096
i32.const 5
call $vudo_println
```

##### vudo_log

**Signature:** `(level: u32, ptr: i32, len: i32) -> void`

```rust
// DOL: log(LogLevel::Info, "System initialized")
// Generated WASM:
fn gen_log_call(
    &self,
    builder: &mut InstrSeqBuilder,
    level: LogLevel,
    string: &str
) {
    let (ptr, len) = self.string_pool.add(string);

    builder.i32_const(level as i32);   // Log level (0-4)
    builder.i32_const(ptr as i32);     // String pointer
    builder.i32_const(len as i32);     // String length
    builder.call(self.get_import_idx("vudo_log"));
}
```

**WAT Output:**
```wasm
i32.const 2       ;; LogLevel::Info
i32.const 4096    ;; pointer
i32.const 18      ;; length
call $vudo_log
```

##### vudo_error

**Signature:** `(ptr: i32, len: i32) -> u32`

Returns an error code.

```rust
// DOL: let code = error("Fatal error occurred")
// Generated WASM:
i32.const 4096
i32.const 20
call $vudo_error    ;; Returns error code on stack
local.set $code     ;; Store in local variable
```

#### Memory Functions (3 functions)

##### vudo_alloc

**Signature:** `(size: u32) -> i32`

Allocates memory and returns a pointer (or 0 on failure).

```rust
// DOL: let ptr = alloc(1024)
// Generated WASM:
fn gen_alloc_call(
    &self,
    builder: &mut InstrSeqBuilder,
    size: u32
) {
    builder.i32_const(size as i32);
    builder.call(self.get_import_idx("vudo_alloc"));
    // Pointer now on stack
}
```

**WAT Output:**
```wasm
i32.const 1024
call $vudo_alloc
local.set $ptr
```

##### vudo_free

**Signature:** `(ptr: i32, size: u32) -> void`

```rust
// DOL: free(ptr, 1024)
// Generated WASM:
local.get $ptr
i32.const 1024
call $vudo_free
```

##### vudo_realloc

**Signature:** `(ptr: i32, old_size: u32, new_size: u32) -> i32`

```rust
// DOL: let new_ptr = realloc(ptr, 1024, 2048)
// Generated WASM:
local.get $ptr
i32.const 1024
i32.const 2048
call $vudo_realloc
local.set $new_ptr
```

#### Time Functions (3 functions)

##### vudo_now

**Signature:** `() -> f64`

Returns current time in milliseconds since Unix epoch.

```rust
// DOL: let timestamp = now()
// Generated WASM:
call $vudo_now
local.set $timestamp  ;; f64 value
```

##### vudo_sleep

**Signature:** `(ms: u32) -> void`

```rust
// DOL: sleep(1000)
// Generated WASM:
i32.const 1000
call $vudo_sleep
```

##### vudo_monotonic_now

**Signature:** `() -> u64`

Returns monotonic timestamp in nanoseconds.

```rust
// DOL: let t = monotonic_now()
// Generated WASM:
call $vudo_monotonic_now
local.set $t  ;; u64 value
```

#### Messaging Functions (5 functions)

##### vudo_send

**Signature:** `(target_ptr: i32, target_len: u32, msg_ptr: i32, msg_len: u32) -> u32`

```rust
// DOL: send("agent-42", "Hello")
// Generated WASM:
fn gen_send_call(
    &self,
    builder: &mut InstrSeqBuilder,
    target: &str,
    message: &str
) {
    let (target_ptr, target_len) = self.string_pool.add(target);
    let (msg_ptr, msg_len) = self.string_pool.add(message);

    builder.i32_const(target_ptr as i32);
    builder.i32_const(target_len as i32);
    builder.i32_const(msg_ptr as i32);
    builder.i32_const(msg_len as i32);
    builder.call(self.get_import_idx("vudo_send"));
    // Returns result code (0 = success)
}
```

**WAT Output:**
```wasm
i32.const 4096    ;; target pointer
i32.const 8       ;; target length
i32.const 4104    ;; message pointer
i32.const 5       ;; message length
call $vudo_send
;; Result code on stack
```

##### vudo_recv

**Signature:** `(timeout_ms: u32, out_ptr: i32, out_len: u32) -> i32`

Returns bytes read, 0 if no message, negative on error.

```rust
// DOL: let msg = recv(1000)
// Generated WASM (simplified):
i32.const 1000         ;; timeout
local.get $buffer_ptr  ;; output buffer
i32.const 1024         ;; buffer size
call $vudo_recv
local.set $bytes_read
```

##### vudo_pending

**Signature:** `() -> u32`

Returns number of pending messages.

```rust
// DOL: if pending() > 0 { ... }
// Generated WASM:
call $vudo_pending
i32.const 0
i32.gt_u
if
  ;; handle messages
end
```

##### vudo_broadcast

**Signature:** `(msg_ptr: i32, msg_len: u32) -> u32`

Returns number of agents message was sent to.

```rust
// DOL: broadcast("System shutdown")
// Generated WASM:
i32.const 4096
i32.const 15
call $vudo_broadcast
;; Returns recipient count
```

##### vudo_free_message

**Signature:** `(msg_id: u32) -> void`

Frees a received message.

```rust
// DOL: free_message(msg_id)
// Generated WASM:
local.get $msg_id
call $vudo_free_message
```

#### Random Functions (2 functions)

##### vudo_random

**Signature:** `() -> f64`

Returns random value in [0.0, 1.0).

```rust
// DOL: let r = random()
// Generated WASM:
call $vudo_random
local.set $r
```

##### vudo_random_bytes

**Signature:** `(ptr: i32, len: u32) -> void`

Fills buffer with random bytes.

```rust
// DOL: random_bytes(buffer, 32)
// Generated WASM:
local.get $buffer
i32.const 32
call $vudo_random_bytes
```

#### Effect Functions (2 functions)

##### vudo_emit_effect

**Signature:** `(effect_ptr: i32, effect_len: u32) -> u32`

Returns effect ID.

```rust
// DOL: emit_effect(effect_json)
// Generated WASM:
i32.const 4096
i32.const 42
call $vudo_emit_effect
local.set $effect_id
```

##### vudo_subscribe

**Signature:** `(pattern_ptr: i32, pattern_len: u32) -> u32`

Returns subscription ID.

```rust
// DOL: subscribe("state_change.*")
// Generated WASM:
i32.const 4096
i32.const 14
call $vudo_subscribe
local.set $sub_id
```

#### Debug Functions (3 functions)

##### vudo_breakpoint

**Signature:** `() -> void`

Triggers debugger breakpoint.

```rust
// DOL: breakpoint()
// Generated WASM:
call $vudo_breakpoint
```

##### vudo_assert

**Signature:** `(condition: u32, msg_ptr: i32, msg_len: u32) -> void`

Panics if condition is false (0).

```rust
// DOL: assert(value > 0, "Value must be positive")
// Generated WASM:
local.get $value
i32.const 0
i32.gt_s
i32.const 4096
i32.const 23
call $vudo_assert
```

##### vudo_panic

**Signature:** `(msg_ptr: i32, msg_len: i32) -> !`

Never returns (terminates execution).

```rust
// DOL: panic("Fatal error!")
// Generated WASM:
i32.const 4096
i32.const 12
call $vudo_panic
unreachable  ;; Never reached
```

### String Encoding

Strings are stored in the WASM data section and passed as (pointer, length) pairs.

#### StringPool Structure

```rust
struct StringPool {
    /// Stored strings: (offset, content)
    strings: Vec<(u32, String)>,

    /// Current offset in data section
    current_offset: u32,

    /// Base offset (typically 0 or after heap)
    base_offset: u32,
}

impl StringPool {
    fn add(&mut self, s: &str) -> (u32, u32) {
        // Check for existing string (deduplication)
        for (offset, existing) in &self.strings {
            if existing == s {
                return (*offset, s.len() as u32);
            }
        }

        // Add new string
        let offset = self.current_offset;
        let len = s.len() as u32;

        self.strings.push((offset, s.to_string()));
        self.current_offset += len;

        (offset, len)
    }

    fn get_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for (_, s) in &self.strings {
            data.extend_from_slice(s.as_bytes());
        }
        data
    }
}
```

#### Memory Layout

```text
WASM Linear Memory:
┌────────────────────────────────────────┐
│ 0x0000 - 0x0FFF: Reserved              │  4KB
├────────────────────────────────────────┤
│ 0x1000 - ????: Data Section (strings)  │  Variable
├────────────────────────────────────────┤
│ ????: Heap Base (dynamic allocation)   │  Grows up
└────────────────────────────────────────┘
```

#### Data Section Emission

```rust
// After code generation:
if !string_pool.is_empty() {
    let mut data = DataSection::new();

    data.active(
        0,                                    // Memory index
        &ConstExpr::i32_const(0),            // Offset
        string_pool.get_data().iter().copied()
    );

    wasm_module.section(&data);
}
```

### Type Conversions

#### DOL Type → WASM Type

```rust
fn dol_type_to_wasm(&self, ty: &Type) -> Result<ValType, WasmError> {
    match ty {
        Type::Int32 | Type::Int => ValType::I32,
        Type::Int64 => ValType::I64,
        Type::Float32 => ValType::F32,
        Type::Float64 | Type::Float => ValType::F64,
        Type::Bool => ValType::I32,  // 0 = false, 1 = true
        Type::String => {
            // Strings are (i32 ptr, i32 len) pairs
            // Not a single type - handled at call site
            return Err(WasmError::InvalidType("String requires ptr+len".into()));
        }
        _ => Err(WasmError::UnsupportedType(format!("{:?}", ty)))
    }
}
```

#### Parameter Widening

DOL may require widening smaller types to WASM types:

```rust
// DOL: let x: i8 = 42
// WASM: i32 (with sign extension)

match param_type {
    Type::Int8 | Type::Int16 | Type::Int32 => {
        // All represented as i32 in WASM
        builder.i32_const(value as i32);
    }
    Type::Int64 => {
        builder.i64_const(value);
    }
    // ...
}
```

---

## Prelude Functions

Complete mapping of all 22 VUDO host functions:

| # | DOL Prelude | Host Function | Signature | Category | Returns |
|---|-------------|---------------|-----------|----------|---------|
| 1 | `print(s)` | `vudo_print` | `(i32, i32)` | I/O | void |
| 2 | `println(s)` | `vudo_println` | `(i32, i32)` | I/O | void |
| 3 | `log(level, s)` | `vudo_log` | `(u32, i32, i32)` | I/O | void |
| 4 | `error(s)` | `vudo_error` | `(i32, i32) -> u32` | I/O | error code |
| 5 | `alloc(size)` | `vudo_alloc` | `(u32) -> i32` | Memory | pointer |
| 6 | `free(ptr, size)` | `vudo_free` | `(i32, u32)` | Memory | void |
| 7 | `realloc(ptr, old, new)` | `vudo_realloc` | `(i32, u32, u32) -> i32` | Memory | pointer |
| 8 | `now()` | `vudo_now` | `() -> f64` | Time | timestamp ms |
| 9 | `sleep(ms)` | `vudo_sleep` | `(u32)` | Time | void |
| 10 | `monotonic_now()` | `vudo_monotonic_now` | `() -> u64` | Time | timestamp ns |
| 11 | `send(target, msg)` | `vudo_send` | `(i32, u32, i32, u32) -> u32` | Messaging | result code |
| 12 | `recv(timeout)` | `vudo_recv` | `(u32, i32, u32) -> i32` | Messaging | bytes read |
| 13 | `pending()` | `vudo_pending` | `() -> u32` | Messaging | count |
| 14 | `broadcast(msg)` | `vudo_broadcast` | `(i32, u32) -> u32` | Messaging | recipient count |
| 15 | `free_message(id)` | `vudo_free_message` | `(u32)` | Messaging | void |
| 16 | `random()` | `vudo_random` | `() -> f64` | Random | [0.0, 1.0) |
| 17 | `random_bytes(buf, len)` | `vudo_random_bytes` | `(i32, u32)` | Random | void |
| 18 | `emit_effect(data)` | `vudo_emit_effect` | `(i32, u32) -> u32` | Effects | effect ID |
| 19 | `subscribe(pattern)` | `vudo_subscribe` | `(i32, u32) -> u32` | Effects | sub ID |
| 20 | `breakpoint()` | `vudo_breakpoint` | `()` | Debug | void |
| 21 | `assert(cond, msg)` | `vudo_assert` | `(u32, i32, i32)` | Debug | void |
| 22 | `panic(msg)` | `vudo_panic` | `(i32, i32) -> !` | Debug | never |

### Category Breakdown

**I/O (4 functions):** Communication with standard output and logging
- Print without newline
- Print with newline
- Structured logging with levels
- Error reporting

**Memory (3 functions):** Dynamic memory management
- Allocation
- Deallocation
- Reallocation

**Time (3 functions):** Time measurement and delays
- Wall-clock time
- Monotonic time
- Sleep/yield

**Messaging (5 functions):** Inter-agent communication
- Point-to-point send
- Receive with timeout
- Check pending messages
- Broadcast to all agents
- Free message resources

**Random (2 functions):** Random number generation
- Random float [0.0, 1.0)
- Random bytes (cryptographically secure)

**Effects (2 functions):** Side effect observation
- Emit observable effects
- Subscribe to effect patterns

**Debug (3 functions):** Debugging and assertions
- Breakpoint trigger
- Conditional assertion
- Unconditional panic

---

## Examples

### Example 1: Hello World

**DOL Source:**
```dol
gen HelloWorld {
    fun main() {
        println("Hello, VUDO!")
    }
}
```

**Generated WASM (text format):**
```wasm
(module
  ;; Import section
  (import "vudo" "vudo_println"
    (func $vudo_println (param i32 i32)))

  ;; Memory section
  (memory 1)

  ;; Data section
  (data (i32.const 0) "Hello, VUDO!")

  ;; Function section
  (func $main (export "main")
    i32.const 0      ;; pointer to "Hello, VUDO!"
    i32.const 12     ;; length of string
    call $vudo_println
  )
)
```

**Transformation Steps:**

1. **Parse**: Recognize `println` call with string literal argument
2. **Track Import**: Add `vudo_println` to import list
3. **Add String**: Store "Hello, VUDO!" in string pool at offset 0
4. **Generate Type**: `(param i32 i32)` function type
5. **Emit Import**: Create import entry in import section
6. **Emit Data**: Write string to data section
7. **Generate Call**: Push pointer (0), length (12), call function

### Example 2: Echo Server

**DOL Source:**
```dol
gen EchoServer {
    fun main() {
        loop {
            if pending() > 0 {
                let buffer = [0u8; 1024]
                let bytes = recv(1000, buffer, 1024)

                if bytes > 0 {
                    println("Received message")
                    // Echo back (simplified)
                    broadcast(buffer)
                }
            }

            sleep(10)
        }
    }
}
```

**Generated Imports:**
```wasm
(import "vudo" "vudo_pending"
  (func $vudo_pending (result i32)))
(import "vudo" "vudo_recv"
  (func $vudo_recv (param i32 i32 i32) (result i32)))
(import "vudo" "vudo_println"
  (func $vudo_println (param i32 i32)))
(import "vudo" "vudo_broadcast"
  (func $vudo_broadcast (param i32 i32) (result i32)))
(import "vudo" "vudo_sleep"
  (func $vudo_sleep (param i32)))
```

**Key Call Sites:**

```wasm
;; pending() > 0
call $vudo_pending
i32.const 0
i32.gt_u

;; recv(1000, buffer, 1024)
i32.const 1000           ;; timeout
local.get $buffer_ptr
i32.const 1024
call $vudo_recv
local.set $bytes

;; println("Received message")
i32.const 4096           ;; pointer to string
i32.const 16             ;; length
call $vudo_println

;; broadcast(buffer)
local.get $buffer_ptr
local.get $bytes
call $vudo_broadcast

;; sleep(10)
i32.const 10
call $vudo_sleep
```

### Example 3: Memory Allocator

**DOL Source:**
```dol
gen Allocator {
    fun allocate_buffer(size: Int32) -> Int32 {
        let ptr = alloc(size)

        if ptr == 0 {
            error("Allocation failed!")
            return 0
        }

        println("Allocated memory")
        return ptr
    }

    fun deallocate_buffer(ptr: Int32, size: Int32) {
        free(ptr, size)
        println("Freed memory")
    }
}
```

**Generated WASM Function:**
```wasm
(func $allocate_buffer (param $size i32) (result i32)
  (local $ptr i32)

  ;; let ptr = alloc(size)
  local.get $size
  call $vudo_alloc
  local.set $ptr

  ;; if ptr == 0
  local.get $ptr
  i32.const 0
  i32.eq
  if (result i32)
    ;; error("Allocation failed!")
    i32.const 4096    ;; error message ptr
    i32.const 18      ;; error message len
    call $vudo_error
    drop              ;; drop error code

    ;; return 0
    i32.const 0
  else
    ;; println("Allocated memory")
    i32.const 4114    ;; success message ptr
    i32.const 16      ;; success message len
    call $vudo_println

    ;; return ptr
    local.get $ptr
  end
)

(func $deallocate_buffer (param $ptr i32) (param $size i32)
  ;; free(ptr, size)
  local.get $ptr
  local.get $size
  call $vudo_free

  ;; println("Freed memory")
  i32.const 4130
  i32.const 12
  call $vudo_println
)
```

### Example 4: Timer

**DOL Source:**
```dol
gen Timer {
    fun measure_duration() {
        let start = monotonic_now()

        // Do some work
        sleep(100)

        let end = monotonic_now()
        let duration = end - start

        log(LogLevel::Info, "Duration: " + duration + " ns")
    }
}
```

**Generated WASM (simplified):**
```wasm
(func $measure_duration
  (local $start i64)
  (local $end i64)
  (local $duration i64)

  ;; let start = monotonic_now()
  call $vudo_monotonic_now
  local.set $start

  ;; sleep(100)
  i32.const 100
  call $vudo_sleep

  ;; let end = monotonic_now()
  call $vudo_monotonic_now
  local.set $end

  ;; let duration = end - start
  local.get $end
  local.get $start
  i64.sub
  local.set $duration

  ;; log(Info, message)
  ;; (String formatting omitted for brevity)
  i32.const 2           ;; LogLevel::Info
  i32.const 4096        ;; message ptr
  i32.const 25          ;; message len
  call $vudo_log
)
```

### Example 5: All Categories Combined

**DOL Source:**
```dol
gen FullDemo {
    fun demo() {
        // I/O
        println("Starting demo")
        log(LogLevel::Debug, "Debug enabled")

        // Memory
        let buffer = alloc(1024)

        // Time
        let t1 = now()
        sleep(50)
        let t2 = now()

        // Messaging
        send("agent-1", "ping")
        let count = pending()

        // Random
        let r = random()
        random_bytes(buffer, 32)

        // Effects
        let eid = emit_effect("{\"type\":\"demo\"}")
        let sid = subscribe("demo.*")

        // Debug
        assert(buffer != 0, "Buffer allocated")

        // Cleanup
        free(buffer, 1024)
        println("Demo complete")
    }
}
```

**All 14 Imports Generated:**
```wasm
(import "vudo" "vudo_println" (func $vudo_println (param i32 i32)))
(import "vudo" "vudo_log" (func $vudo_log (param i32 i32 i32)))
(import "vudo" "vudo_alloc" (func $vudo_alloc (param i32) (result i32)))
(import "vudo" "vudo_now" (func $vudo_now (result f64)))
(import "vudo" "vudo_sleep" (func $vudo_sleep (param i32)))
(import "vudo" "vudo_send" (func $vudo_send (param i32 i32 i32 i32) (result i32)))
(import "vudo" "vudo_pending" (func $vudo_pending (result i32)))
(import "vudo" "vudo_random" (func $vudo_random (result f64)))
(import "vudo" "vudo_random_bytes" (func $vudo_random_bytes (param i32 i32)))
(import "vudo" "vudo_emit_effect" (func $vudo_emit_effect (param i32 i32) (result i32)))
(import "vudo" "vudo_subscribe" (func $vudo_subscribe (param i32 i32) (result i32)))
(import "vudo" "vudo_assert" (func $vudo_assert (param i32 i32 i32)))
(import "vudo" "vudo_free" (func $vudo_free (param i32 i32)))
```

---

## Testing

### Unit Tests

Test import generation in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_deduplication() {
        let mut imports = vec![
            WasmImport::println(),
            WasmImport::println(), // Duplicate
            WasmImport::print(),
        ];

        dedup_imports(&mut imports);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].name, "vudo_print");
        assert_eq!(imports[1].name, "vudo_println");
    }

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::new();

        let (ptr1, len1) = pool.add("Hello");
        let (ptr2, len2) = pool.add("World");
        let (ptr3, len3) = pool.add("Hello"); // Duplicate

        assert_eq!(len1, 5);
        assert_eq!(len2, 5);
        assert_eq!(ptr1, ptr3); // Same offset for duplicate

        let data = pool.get_data();
        assert_eq!(&data[0..5], b"Hello");
        assert_eq!(&data[5..10], b"World");
    }

    #[test]
    fn test_type_mapping() {
        let compiler = WasmCompiler::new();

        assert_eq!(
            compiler.dol_type_to_wasm(&Type::Int32).unwrap(),
            ValType::I32
        );
        assert_eq!(
            compiler.dol_type_to_wasm(&Type::Float64).unwrap(),
            ValType::F64
        );
    }
}
```

### Integration Tests

Test end-to-end compilation:

```rust
#[test]
fn test_compile_hello_world() {
    let source = r#"
        gen Hello {
            fun main() {
                println("Hello, World!")
            }
        }
    "#;

    let wasm = compile(source).unwrap();

    // Validate WASM
    assert!(wasmparser::validate(&wasm).is_ok());

    // Check imports
    let imports = extract_imports(&wasm);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module, "vudo");
    assert_eq!(imports[0].name, "vudo_println");
}

#[test]
fn test_compile_with_multiple_imports() {
    let source = r#"
        gen Multi {
            fun main() {
                let t = now()
                println("Time: " + t)
                send("other", "message")
            }
        }
    "#;

    let wasm = compile(source).unwrap();
    let imports = extract_imports(&wasm);

    // Should have: now, println, send
    assert_eq!(imports.len(), 3);
    assert!(imports.iter().any(|i| i.name == "vudo_now"));
    assert!(imports.iter().any(|i| i.name == "vudo_println"));
    assert!(imports.iter().any(|i| i.name == "vudo_send"));
}
```

### Validation Tests

Ensure generated WASM is valid:

```rust
#[test]
fn test_wasm_validation() {
    let source = r#"
        gen Valid {
            fun test() {
                alloc(1024)
                free(0, 1024)
                println("Done")
            }
        }
    "#;

    let wasm = compile(source).unwrap();

    // Use wasmparser to validate
    let mut validator = wasmparser::Validator::new();
    assert!(validator.validate_all(&wasm).is_ok());
}

#[test]
fn test_import_signatures() {
    let wasm = compile_simple_program().unwrap();

    // Parse and check type section
    for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
        if let Ok(wasmparser::Payload::TypeSection(reader)) = payload {
            for ty in reader {
                let ty = ty.unwrap();
                // Verify function types match ABI
                validate_function_type(&ty);
            }
        }
    }
}
```

### Test Fixtures

Example DOL files for testing:

**tests/fixtures/io.dol:**
```dol
gen IoTest {
    fun test() {
        print("A")
        println("B")
        log(LogLevel::Info, "C")
        error("D")
    }
}
```

**tests/fixtures/memory.dol:**
```dol
gen MemoryTest {
    fun test() {
        let p1 = alloc(100)
        let p2 = realloc(p1, 100, 200)
        free(p2, 200)
    }
}
```

**tests/fixtures/messaging.dol:**
```dol
gen MessagingTest {
    fun test() {
        send("target", "msg")
        let c = pending()
        let buf = [0u8; 100]
        recv(1000, buf, 100)
        broadcast("all")
    }
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test compiler_imports

# With output
cargo test -- --nocapture

# Specific test
cargo test test_compile_hello_world

# Check generated WASM
cargo test -- --show-output validate_wasm
```

---

## Implementation Checklist

- [x] Define `WasmImport` structure
- [x] Implement import extraction from AST
- [x] Add import deduplication logic
- [x] Create `StringPool` for data section
- [x] Implement call generation for all 22 functions
- [x] Add type mapping (DOL → WASM)
- [x] Handle string encoding (ptr, len)
- [x] Emit type section with function signatures
- [x] Emit import section
- [x] Emit data section for strings
- [x] Generate call instructions
- [x] Add unit tests for each component
- [x] Add integration tests for full pipeline
- [x] Validate generated WASM modules
- [x] Document all host function mappings
- [x] Create example DOL programs
- [x] Write comprehensive documentation

---

## Next Steps

After completing this phase:

1. **Runtime Integration**: Implement host function handlers in the VUDO runtime
2. **Error Handling**: Add proper error codes and messages for import failures
3. **Optimization**: Implement dead code elimination for unused imports
4. **Debugging**: Add source maps for better debugging experience
5. **Extended ABI**: Support for additional host functions as needed

---

## References

- **WASM Specification**: https://webassembly.github.io/spec/
- **wasm-encoder docs**: https://docs.rs/wasm-encoder/
- **wasmparser docs**: https://docs.rs/wasmparser/
- **Host Functions**: `src/host.rs`
- **WASM Compiler**: `src/wasm/compiler.rs`
- **ABI Specification**: `docs/ABI-SPECIFICATION.md`

---

**Document Version:** 1.0
**Last Updated:** 2026-02-04
**Status:** Complete
