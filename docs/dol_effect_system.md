# DOL Effect System Specification

> **Status:** APPROVED  
> **Date:** January 2, 2026  
> **Principle:** Inference over annotation. Simplicity over ceremony.

---

## Core Philosophy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   "Everything is pure by default.                                           │
│    The compiler infers effects.                                             │
│    Users never annotate effects.                                            │
│    Great errors guide when needed."                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Rules

### 1. No Effect Keywords in User Code

```dol
// ✅ CORRECT - Just write code
fun greet(name: String) {
    print("Hello, " + name)
}

// ❌ WRONG - Never in user code
sex fun greet(name: String) {
    print("Hello, " + name)
}
```

### 2. Effects Are Inferred

```dol
fun a() { print("x") }     // Compiler infers: effectful (calls print)
fun b() { a() }            // Compiler infers: effectful (calls a)
fun c() { return 1 + 2 }   // Compiler infers: pure
```

### 3. `sex` Is Internal Only

```dol
// ONLY in stdlib implementation files
// Users never see or write this

// io/_internal.dol
sex fun _host_print(ptr: Int, len: Int)

// io/print.dol (public API)
fun print(s: String) {
    _host_print(s.ptr, s.len)
}
```

### 4. Errors Over Annotations

When effects conflict with requirements, emit clear errors:

```
error[E0312]: effectful call in pure context
  --> app.dol:5:5
   |
 5 |     print("debug")
   |     ^^^^^^^^^^^^^^ `print` has effects
   |
note: `parallel_map` requires a pure function
help: remove the print, or use sequential `map` instead
```

---

## Standard Library API

### Prelude (Auto-Imported)

```dol
// Available everywhere without import

// I/O
print(s: String)
println(s: String)
log(level: Level, s: String)
error(s: String)

// Messaging
send(to: SpiritId, s: String)
send(to: SpiritId, b: Bytes)
recv() -> String
recv_bytes() -> Bytes
pending() -> Int
broadcast(s: String)

// Memory
alloc(size: Int) -> Ptr
free(ptr: Ptr)

// Time
now() -> Time
sleep(ms: Int)
```

### Modules (Require Import)

```dol
use net.http    // HTTP client
use net.ws      // WebSocket
use db          // Database
use fs          // Filesystem
```

---

## Effect Inference Algorithm

```
1. Build call graph
2. Mark known effectful primitives (stdlib host bindings)
3. Propagate effects transitively through call graph
4. Store effect status in HIR (internal flag, not syntax)
5. Use effect info for:
   - Optimization (pure functions can be memoized, parallelized)
   - Error messages (when effects conflict)
   - WASM codegen (import handling)
```

---

## What Users See

### Writing Code
```dol
gene Counter {
    has value: Int
    
    fun increment() {
        self.value = self.value + 1
        println("Counter: " + self.value)
    }
    
    fun get() -> Int {
        return self.value
    }
}

fun main() {
    let c = Counter { value: 0 }
    c.increment()
    c.increment()
    print("Final: " + c.get())
}
```

**No `sex`. No `pure`. No `do`. Just code.**

### IDE/LSP Information

Hover over function shows inferred effect:

```
increment() -> Void
  Effects: io (via println), mutation (via field assignment)
  
get() -> Int
  Effects: none (pure)
```

### Errors (When Needed)

```
error[E0312]: effectful operation in pure context
  --> app.dol:12:9
   |
12 |         print("logging")
   |         ^^^^^^^^^^^^^^^^
   |
note: trait `Eq.equals` must be pure for correctness
help: remove the print call
```

---

## Implementation Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  USER CODE                                                                  │
│  fun main() { print("Hi"); send(1, "msg") }                                │
├─────────────────────────────────────────────────────────────────────────────┤
│  STDLIB (public API)                                                        │
│  fun print(s: String) { _host_print(s.ptr, s.len) }                        │
│  fun send(to: SpiritId, s: String) { _host_send(to, s.ptr, s.len) }       │
├─────────────────────────────────────────────────────────────────────────────┤
│  STDLIB (internal - sex keyword lives here ONLY)                           │
│  sex fun _host_print(ptr: Int, len: Int)                                   │
│  sex fun _host_send(to: Int, ptr: Int, len: Int)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  RUNTIME (@vudo/runtime)                                                    │
│  io: { print: (ptr, len) => console.log(decode(ptr, len)) }               │
│  msg: { send: (to, ptr, len) => bus.send(to, ptr, len) }                  │
├─────────────────────────────────────────────────────────────────────────────┤
│  WASM                                                                       │
│  (import "io" "print" (func ...))                                          │
│  (import "msg" "send" (func ...))                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Keywords Inventory

### User-Facing (After Cleanup)

| Category | Keywords |
|----------|----------|
| Declarations | `gene`, `trait`, `fun`, `val`, `var`, `type`, `mod` |
| Control | `if`, `else`, `match`, `for`, `while`, `loop`, `break`, `continue`, `return` |
| Types | `Int`, `String`, `Bool`, `Bytes`, `Void`, `Self` |
| Operators | `and`, `or`, `not`, `in`, `is`, `as` |
| Other | `use`, `pub`, `impl`, `where`, `forall` |

### Internal Only (Stdlib)

| Keyword | Purpose |
|---------|---------|
| `sex` | Mark host bindings as effectful |

### Removed/Never Added

| Keyword | Reason |
|---------|--------|
| `pure` | Inference handles it |
| `do` | Not needed |
| `extern` | Use `sex` internally |
| `unsafe` | Use `sex` internally |

---

## Migration Path

### Phase 1: Update Examples & Docs
- Remove all `sex fun` from user-facing examples
- Update documentation to show clean code
- Keep `sex` in stdlib implementation only

### Phase 2: HIR Effect Inference
- Implement call graph analysis
- Propagate effects transitively
- Store effect status as internal flag

### Phase 3: Error Messages
- Invest in clear, helpful effect-related errors
- Show propagation path ("effectful because it calls X which calls Y")
- Provide actionable suggestions

### Phase 4: Stdlib Refactor
- Create clean public API (`io.print`, `msg.send`)
- Hide `sex` in internal implementation files
- Remove `vudo_` prefix entirely

---

## Success Criteria

- [ ] Users never write `sex`, `pure`, `do`, or `vudo_`
- [ ] Effect inference works transitively
- [ ] Error messages clearly explain effect conflicts
- [ ] Stdlib has clean, short function names
- [ ] IDE shows inferred effects on hover
- [ ] No new keywords added to language

---

*"Do the right thing. Infer what you can. Guide when you can't."*