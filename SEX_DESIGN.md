# SEX in DOL: Side Effect eXecution

> **Design Document v0.1**  
> **Status:** RFC  
> **Vocabulary:** `sex` = Side Effect eXecution (or: Shared External eXchange)

---

## The Biological Metaphor

In biology, **sex** is the mechanism for:
- **Genetic recombination** — mixing code across boundaries
- **Mutation** — changing state destructively
- **Crossing barriers** — breaking isolation
- **Creating new combinations** — FFI, interop

In DOL, **sex** represents code that:
- **Mutates global state** — side effects
- **Crosses module boundaries** — unsafe access
- **Performs FFI** — external system calls
- **Breaks referential transparency** — impure functions

---

## Design Overview

### File Convention

| Pattern | Meaning |
|---------|---------|
| `*.sex.dol` | Sex file — contains unsafe/effectful code |
| `sex/` | Sex directory — all files are sex context |
| `sex { }` | Sex block — unsafe scope within pure code |

### Visibility & Safety Model

```
┌─────────────────────────────────────────────────────────────────┐
│                     DOL Safety Hierarchy                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  PURE (default)                                                 │
│  ├── No side effects                                            │
│  ├── Referentially transparent                                  │
│  ├── Private by default                                         │
│  └── Safe to parallelize                                        │
│                                                                 │
│  PUB (public)                                                   │
│  ├── Exported from module                                       │
│  ├── Still pure unless in sex context                          │
│  └── API boundary                                               │
│                                                                 │
│  SEX (side effects)                                             │
│  ├── Can mutate global state                                    │
│  ├── Can perform I/O                                            │
│  ├── Can call FFI                                               │
│  ├── Must be explicitly marked                                  │
│  └── Compiler tracks effect propagation                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Syntax

### Sex Blocks

```dol
gene StatefulService {
    // Pure function — no sex allowed
    fun pure_compute(x: Int64) -> Int64 {
        return x * 2
    }
    
    // Sex block — side effects permitted
    sex fun log_and_compute(x: Int64) -> Int64 {
        println("Computing: " + x)  // I/O side effect
        GLOBAL_COUNTER += 1         // Mutation
        return x * 2
    }
    
    // Inline sex block within pure function
    fun mostly_pure(x: Int64) -> Int64 {
        result = x * 2
        
        sex {
            // This block can have side effects
            debug_log("Result: " + result)
        }
        
        return result
    }
}
```

### Global Mutable State

```dol
// globals.sex.dol — Sex file for global state

// Mutable global — only allowed in sex context
sex var GLOBAL_COUNTER: Int64 = 0

// Global constant — allowed anywhere (immutable)
const MAX_CONNECTIONS: Int64 = 100

// Sex function to modify global
sex fun increment_counter() -> Int64 {
    GLOBAL_COUNTER += 1
    return GLOBAL_COUNTER
}

// Sex function to reset
sex fun reset_counter() {
    GLOBAL_COUNTER = 0
}
```

### FFI (Foreign Function Interface)

```dol
// ffi.sex.dol — External system calls

// Declare external function (FFI)
sex extern fun libc_malloc(size: UInt64) -> Ptr<Void>
sex extern fun libc_free(ptr: Ptr<Void>)

// Wrap unsafe FFI in sex function
sex fun allocate<T>(count: UInt64) -> Ptr<T> {
    size = count * size_of<T>()
    ptr = libc_malloc(size)
    
    if ptr.is_null() {
        panic("Allocation failed")
    }
    
    return ptr.cast<T>()
}

// Raw pointer operations
sex fun unsafe_read<T>(ptr: Ptr<T>, offset: Int64) -> T {
    return ptr.offset(offset).deref()
}
```

### I/O Operations

```dol
// io.sex.dol — I/O is inherently effectful

use std.fs.{ File, OpenMode }

// File operations are sex
sex fun read_file(path: String) -> Result<String, IoError> {
    file = File.open(path, OpenMode.Read)?
    content = file.read_all()?
    file.close()
    return Ok(content)
}

// Network is sex
sex fun http_get(url: String) -> Result<Response, NetError> {
    // Side effect: network request
    return Http.get(url)
}

// Random is sex (non-deterministic)
sex fun random_int(min: Int64, max: Int64) -> Int64 {
    return Random.range(min, max)
}

// Time is sex (non-deterministic)
sex fun now() -> Timestamp {
    return Timestamp.now()
}
```

---

## Effect Tracking

### The Sex Type

Functions with side effects have a **sex type annotation**:

```dol
// Pure function type
fun add(a: Int64, b: Int64) -> Int64

// Sex function type — effect is part of the signature
sex fun log(msg: String) -> Void

// In type annotations
type PureCompute = Fun<Int64, Int64>
type SexCompute = Sex<Fun<Int64, Int64>>
```

### Effect Propagation

The compiler tracks sex propagation:

```dol
// ❌ ERROR: Cannot call sex function from pure context
fun pure_caller() -> Int64 {
    log("hello")  // Compile error: sex in pure context
    return 42
}

// ✅ OK: Sex propagates up
sex fun sex_caller() -> Int64 {
    log("hello")  // OK: we're in sex context
    return 42
}

// ✅ OK: Explicit sex block
fun mixed_caller() -> Int64 {
    result = 42
    sex {
        log("hello")  // OK: inside sex block
    }
    return result
}
```

### Sex Boundaries

```dol
// At module boundary, sex must be declared
pub sex fun dangerous_operation() -> Result<Void, Error> {
    // Callers know this has side effects
}

// Pure public function
pub fun safe_operation(x: Int64) -> Int64 {
    // Callers know this is pure
    return x * 2
}
```

---

## Directory Structure

```
my-spirit/
├── Spirit.dol
├── src/
│   ├── lib.dol           # Pure library root
│   ├── main.dol          # Entry point (can use sex)
│   ├── genes/
│   │   └── container.dol # Pure gene definitions
│   ├── spells/
│   │   └── math.dol      # Pure functions
│   └── sex/              # ⚠️ Sex directory
│       ├── globals.dol   # Global mutable state
│       ├── io.dol        # I/O operations
│       ├── ffi.dol       # Foreign function interface
│       └── unsafe.dol    # Pointer operations
```

Or using `.sex.dol` extension:

```
my-spirit/
├── Spirit.dol
├── src/
│   ├── lib.dol
│   ├── container.dol       # Pure
│   ├── container.sex.dol   # Sex operations for Container
│   ├── network.sex.dol     # Network I/O
│   └── state.sex.dol       # Global state
```

---

## Use Cases

### 1. Database Access

```dol
// db.sex.dol

sex var DB_CONNECTION: Option<Connection> = None

sex fun connect(url: String) -> Result<Void, DbError> {
    DB_CONNECTION = Some(Connection.open(url)?)
    return Ok(())
}

sex fun query(sql: String) -> Result<Rows, DbError> {
    conn = DB_CONNECTION.expect("Not connected")
    return conn.execute(sql)
}

sex fun disconnect() {
    if DB_CONNECTION.is_some() {
        DB_CONNECTION.unwrap().close()
        DB_CONNECTION = None
    }
}
```

### 2. Caching

```dol
// cache.sex.dol

sex var CACHE: Map<String, Any> = Map.new()

sex fun get_cached<T>(key: String) -> Option<T> {
    return CACHE.get(key).map(|v| v.cast<T>())
}

sex fun set_cached<T>(key: String, value: T) {
    CACHE.insert(key, value.as_any())
}

sex fun invalidate(key: String) {
    CACHE.remove(key)
}

// Memoization helper
sex fun memoize<A, B>(key: String, compute: Fun<A, B>, arg: A) -> B {
    cached = get_cached<B>(key)
    if cached.is_some() {
        return cached.unwrap()
    }
    
    result = compute(arg)
    set_cached(key, result)
    return result
}
```

### 3. Logging & Telemetry

```dol
// telemetry.sex.dol

sex var LOG_LEVEL: LogLevel = LogLevel.Info

sex fun set_log_level(level: LogLevel) {
    LOG_LEVEL = level
}

sex fun log(level: LogLevel, msg: String) {
    if level >= LOG_LEVEL {
        timestamp = now()
        println("[" + timestamp + "] " + level.to_string() + ": " + msg)
    }
}

sex fun debug(msg: String) { log(LogLevel.Debug, msg) }
sex fun info(msg: String) { log(LogLevel.Info, msg) }
sex fun warn(msg: String) { log(LogLevel.Warn, msg) }
sex fun error(msg: String) { log(LogLevel.Error, msg) }
```

### 4. FFI to System Libraries

```dol
// system.sex.dol

#cfg(target.linux)
sex extern "C" {
    fun getpid() -> Int32
    fun fork() -> Int32
    fun execve(path: Ptr<Char>, argv: Ptr<Ptr<Char>>, envp: Ptr<Ptr<Char>>) -> Int32
}

#cfg(target.wasm)
sex extern "wasi" {
    fun fd_write(fd: Int32, iovs: Ptr<IoVec>, iovs_len: Int32, nwritten: Ptr<Int32>) -> Int32
}

// Safe wrapper
sex fun get_process_id() -> Int32 {
    #cfg(target.linux) {
        return getpid()
    }
    #cfg(target.wasm) {
        return 0  // WASM doesn't have PIDs
    }
}
```

---

## Compiler Enforcement

### Sex Lint Rules

```
┌─────────────────────────────────────────────────────────────────┐
│                     Sex Compiler Checks                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  E001: Sex in pure context                                      │
│  → Cannot call sex function from pure function                  │
│                                                                 │
│  E002: Mutable global outside sex                               │
│  → sex var must be in .sex.dol or sex/ directory               │
│                                                                 │
│  E003: FFI outside sex                                          │
│  → extern declarations require sex context                      │
│                                                                 │
│  E004: I/O outside sex                                          │
│  → File, Network, Random, Time require sex                     │
│                                                                 │
│  W001: Large sex block                                          │
│  → Consider extracting to sex function                          │
│                                                                 │
│  W002: Sex function without documentation                       │
│  → Sex functions should document side effects                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Escape Hatches

```dol
// For when you REALLY need to break the rules

// Trust me, I know what I'm doing
#[allow(sex_in_pure)]
fun technically_pure_but_logs() -> Int64 {
    sex { debug("shhh") }
    return 42
}

// Suppress all sex warnings in this module
#![allow(sex_warnings)]
```

---

## Runtime Behavior

### Sex in Different Targets

| Target | Sex Implementation |
|--------|-------------------|
| **WASM** | WASI imports, linear memory |
| **Rust** | `unsafe` blocks, `static mut` |
| **TypeScript** | Side effects allowed (JS is impure) |
| **Python** | Global variables, I/O |

### Generated Code

**DOL:**
```dol
sex var COUNTER: Int64 = 0

sex fun increment() -> Int64 {
    COUNTER += 1
    return COUNTER
}
```

**Rust output:**
```rust
static mut COUNTER: i64 = 0;

pub fn increment() -> i64 {
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}
```

**TypeScript output:**
```typescript
let COUNTER: number = 0;

export function increment(): number {
    COUNTER += 1;
    return COUNTER;
}
```

---

## Vocabulary Summary

| Term | Meaning | Rust Equivalent |
|------|---------|-----------------|
| `sex` | Side Effect eXecution | `unsafe` |
| `sex fun` | Function with side effects | `fn` with mutation |
| `sex var` | Mutable global variable | `static mut` |
| `sex { }` | Effectful block | `unsafe { }` |
| `sex extern` | FFI declaration | `extern "C"` |
| `.sex.dol` | File with sex code | — |
| `sex/` | Directory of sex files | — |

---

## Philosophy

> "In pure functional programming, sex is forbidden.
> In DOL, sex is acknowledged, contained, and tracked.
> Because sometimes, to create something new,
> boundaries must be crossed."

The sex system ensures:
1. **Explicit** — Side effects are visible in types and files
2. **Contained** — Sex code is isolated from pure code
3. **Tracked** — Compiler knows where effects can occur
4. **Documented** — Developers know what's dangerous

---

## Open Questions

1. **Effect polymorphism?** — Can functions be generic over sex?
   ```dol
   fun map<F: Fun | Sex>(f: F, list: List<A>) -> List<B>
   ```

2. **Sex regions?** — Different kinds of effects?
   ```dol
   sex[IO] fun read_file() -> String
   sex[State] fun increment() -> Int64
   sex[IO, State] fun log_and_count() -> Void
   ```

3. **Sex isolation?** — Can sex code run in sandbox?
   ```dol
   sandbox sex {
       // Effects are captured, not executed
   }
   ```

---

*"To evolve, sometimes you need a little sex."* 🍄
