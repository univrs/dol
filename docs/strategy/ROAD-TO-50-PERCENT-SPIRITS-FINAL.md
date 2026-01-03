# Road to 50% Spirits: Complete Implementation Guide

> **Version:** 2.0  
> **Date:** January 2026  
> **Status:** Ready for Implementation  
> **Philosophy:** Pure by default, effects explicit via `sex`

---

## Executive Summary

This document provides the complete roadmap to achieve 50% Spirit functionality in the VUDO ecosystem. The key insight: **DOL uses the SEX (Side Effect eXecution) system** for all impure operations—not `extern`. This aligns with DOL's core philosophy of making effects visible and trackable.

### Current State → Target State

```
CURRENT (v0.7.0 - ~20%)                 TARGET (v0.8.0 - 50%)
════════════════════════                ════════════════════════

✅ Pure computation                     ✅ Pure computation
✅ Gene methods                         ✅ Gene methods  
✅ Field READ                           ✅ Field READ + WRITE
✅ Control flow                         ✅ Control flow
✅ vudo CLI (run/compile/check)         ✅ vudo CLI + seance
✅ @vudo/runtime (66 tests)             ✅ Runtime with messaging
                                        
❌ sex fun declarations                 ✅ sex fun (host imports)
❌ sex { } blocks                       ✅ sex { } effect blocks
❌ Effect tracking                      ✅ Compiler enforces purity
❌ String literals                      ✅ Strings in data section
❌ Field mutation                       ✅ self.value = x works
❌ Spirit messaging                     ✅ vudo_send/recv via Loa
```

---

## The SEX System

### Why SEX, Not Extern?

DOL's SEX system is the **unified effect system** for all impure operations:

| Category | Examples | All Use `sex` |
|----------|----------|---------------|
| **I/O** | print, read, write files | `sex fun vudo_print()` |
| **Network** | HTTP, WebSocket, P2P | `sex fun vudo_connect()` |
| **State** | Global variables, mutation | `sex var COUNTER` |
| **FFI** | Host calls, WASM imports | `sex fun` (no body) |
| **IPC** | Spirit messaging | `sex fun vudo_send()` |
| **Database** | SQL, key-value stores | `sex fun db_query()` |

### Three SEX Patterns

```dol
// Pattern 1: sex fun - Effectful function
// No body = WASM import, Has body = local implementation
sex fun vudo_print(ptr: Int32, len: Int32)           // Import
sex fun log_value(x: Int64) { vudo_print(x) }       // Local

// Pattern 2: sex { } - Effect block in pure code
fun mostly_pure(x: Int64) -> Int64 {
    let result = x * 2
    sex { vudo_print("Debug: computed result") }    // Contained effect
    return result
}

// Pattern 3: sex var - Mutable global state
sex var REQUEST_COUNT: Int64 = 0
sex fun track_request() { REQUEST_COUNT = REQUEST_COUNT + 1 }
```

### Compiler Enforcement

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         EFFECT BOUNDARY ENFORCEMENT                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Pure Context                        Sex Context                           │
│   ════════════                        ═══════════                           │
│   • fun name() { }                    • sex fun name() { }                  │
│   • Cannot call sex functions         • Can call any function               │
│   • Cannot access sex var             • Can access sex var                  │
│   • Referentially transparent         • May have side effects               │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │  // ERROR: sex call in pure context                                 │  │
│   │  fun bad() { vudo_print("fail") }  // Compile error!               │  │
│   │                                                                     │  │
│   │  // OK: sex block enables effects                                   │  │
│   │  fun good() { sex { vudo_print("ok") } }  // Compiles!             │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Four-Week Implementation Plan

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ROAD TO 50% SPIRITS - 4 WEEKS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  WEEK 1: SEX INFRASTRUCTURE                          ████████░░ 80%        │
│  ───────────────────────────                                                │
│  • Parse sex keyword (sex fun, sex { }, sex var)                           │
│  • Track effect context in type checker                                     │
│  • Generate WASM imports for sex fun (no body)                             │
│  • Test: Effect boundary enforcement                                        │
│                                                                             │
│  WEEK 2: STRINGS & I/O                               ████░░░░░░ 40%        │
│  ──────────────────────                                                     │
│  • String literals → WASM data section                                      │
│  • String passing protocol (ptr, len)                                       │
│  • HelloWorld Spirit: sex { vudo_print("Hello!") }                         │
│  • Test: Console output works                                               │
│                                                                             │
│  WEEK 3: STATE & MUTATION                            ██░░░░░░░░ 20%        │
│  ────────────────────────                                                   │
│  • Field assignment (i64.store)                                             │
│  • sex var globals in WASM                                                  │
│  • Stateful Counter Spirit                                                  │
│  • Test: State persists across calls                                        │
│                                                                             │
│  WEEK 4: MESSAGING & SÉANCE                          ░░░░░░░░░░ 0%         │
│  ──────────────────────────                                                 │
│  • MessageBus in @vudo/runtime                                              │
│  • vudo_send/vudo_recv Loa functions                                        │
│  • Multi-Spirit Séance coordination                                         │
│  • Test: Ping/Pong Spirits communicate                                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Week 1: SEX Infrastructure

### Deliverables
- `sex` keyword in lexer
- `sex fun`, `sex { }`, `sex var` in parser
- Effect context tracking in type checker
- WASM import section for `sex fun` without body
- Error: "sex call in pure context"

### Example: sex_basic.dol
```dol
// Host imports (no body = WASM import)
sex fun vudo_print(ptr: Int32, len: Int32)
sex fun vudo_alloc(size: Int32) -> Int32

// Pure function - no effects allowed
fun add(a: Int64, b: Int64) -> Int64 {
    return a + b
}

// Sex function with body - effects allowed
sex fun log_and_add(a: Int64, b: Int64) -> Int64 {
    vudo_print("Adding...")
    return a + b
}

// Sex block in pure function
fun compute_with_debug(x: Int64) -> Int64 {
    let result = x * 2
    sex {
        vudo_print("Debug: result computed")
    }
    return result
}
```

### Example: sex_error.dol (Should Fail)
```dol
sex fun vudo_print(ptr: Int32, len: Int32)

// ERROR: This should produce compile error
fun bad_function() -> Int64 {
    vudo_print("Not allowed in pure context!")  // ← Error here
    return 42
}
```

### WASM Output Structure
```wat
(module
  ;; Import section - from sex fun without body
  (import "loa" "vudo_print" (func $vudo_print (param i32 i32)))
  (import "loa" "vudo_alloc" (func $vudo_alloc (param i32) (result i32)))
  
  ;; Type section
  (type $t0 (func (param i64 i64) (result i64)))
  
  ;; Function section - local functions
  (func $add (param $a i64) (param $b i64) (result i64)
    local.get $a
    local.get $b
    i64.add
  )
  
  ;; Export section
  (export "add" (func $add))
)
```

---

## Week 2: Strings & I/O

### Deliverables
- String literals in WASM data section
- String → (ptr, len) compilation
- `vudo_print("Hello")` works end-to-end
- HelloWorld Spirit runs

### Example: hello_world.dol
```dol
// Standard Loa imports
sex fun vudo_print(ptr: Int32, len: Int32)

// Entry point - implicitly sex context
fun main() {
    sex {
        vudo_print("Hello from DOL Spirit!")
        vudo_print("The SEX system works!")
    }
}

// Greeting function
sex fun greet(name_ptr: Int32, name_len: Int32) {
    vudo_print("Hello, ")
    vudo_print(name_ptr, name_len)
    vudo_print("!")
}
```

### WASM Output with Data Section
```wat
(module
  (import "loa" "vudo_print" (func $vudo_print (param i32 i32)))
  
  ;; Memory
  (memory 1)
  
  ;; Data section - string literals
  (data (i32.const 0) "Hello from DOL Spirit!")   ;; 22 bytes @ offset 0
  (data (i32.const 22) "The SEX system works!")   ;; 21 bytes @ offset 22
  
  ;; Main function
  (func $main
    ;; vudo_print("Hello from DOL Spirit!")
    i32.const 0      ;; ptr
    i32.const 22     ;; len
    call $vudo_print
    
    ;; vudo_print("The SEX system works!")
    i32.const 22     ;; ptr
    i32.const 21     ;; len
    call $vudo_print
  )
  
  (export "main" (func $main))
)
```

### Expected CLI Output
```bash
$ vudo compile examples/hello_world.dol -o hello.wasm
Compiled: hello.wasm (312 bytes)

$ vudo run hello.wasm -f main
[Spirit] Hello from DOL Spirit!
[Spirit] The SEX system works!
```

---

## Week 3: State & Mutation

### Deliverables
- Field assignment (`self.value = x`)
- `sex var` global variables
- Stateful Counter Spirit
- State persists across function calls

### Example: counter_service.dol
```dol
sex fun vudo_print(ptr: Int32, len: Int32)

gene Counter {
    has value: Int64
    
    // Pure read - no sex needed
    fun get() -> Int64 {
        return self.value
    }
    
    // Mutation requires sex
    sex fun increment() {
        self.value = self.value + 1
        vudo_print("Incremented!")
    }
    
    sex fun set(n: Int64) {
        self.value = n
    }
    
    sex fun add(n: Int64) -> Int64 {
        self.value = self.value + n
        return self.value
    }
}
```

### Example: global_state.dol
```dol
sex fun vudo_print(ptr: Int32, len: Int32)

// Global mutable state - must be sex var
sex var TOTAL_REQUESTS: Int64 = 0
sex var LAST_VALUE: Int64 = 0

sex fun track(value: Int64) {
    TOTAL_REQUESTS = TOTAL_REQUESTS + 1
    LAST_VALUE = value
    vudo_print("Request tracked")
}

sex fun get_stats() -> Int64 {
    return TOTAL_REQUESTS
}
```

### Expected CLI Output
```bash
$ vudo compile examples/counter_service.dol -o counter.wasm

$ vudo run counter.wasm -f Counter.set -a '[10]'
# Counter.value = 10

$ vudo run counter.wasm -f Counter.increment
[Spirit] Incremented!
# Counter.value = 11

$ vudo run counter.wasm -f Counter.get
11
```

---

## Week 4: Messaging & Séance

### Deliverables
- `vudo_send`, `vudo_recv`, `vudo_has_message` in Loa
- MessageBus in @vudo/runtime
- `vudo seance` CLI command
- Ping/Pong Spirits communicate

### Standard Loa Library: sex/loa.dol
```dol
// ═══════════════════════════════════════════════════════════════
// VUDO LOA - Host Function Library
// All functions are sex (have side effects)
// ═══════════════════════════════════════════════════════════════

// ─── I/O ───────────────────────────────────────────────────────
sex fun vudo_print(ptr: Int32, len: Int32)
sex fun vudo_log(level: Int32, ptr: Int32, len: Int32)
sex fun vudo_error(ptr: Int32, len: Int32)

// ─── Memory ────────────────────────────────────────────────────
sex fun vudo_alloc(size: Int32) -> Int32
sex fun vudo_free(ptr: Int32)
sex fun vudo_copy(src: Int32, dst: Int32, len: Int32)

// ─── Messaging ─────────────────────────────────────────────────
sex fun vudo_send(target: Int32, ptr: Int32, len: Int32)
sex fun vudo_recv(buf: Int32, max_len: Int32) -> Int32
sex fun vudo_has_message() -> Bool
sex fun vudo_sender() -> Int32
sex fun vudo_broadcast(ptr: Int32, len: Int32)

// ─── Time ──────────────────────────────────────────────────────
sex fun vudo_now() -> Int64
sex fun vudo_sleep(ms: Int32)

// ─── Identity ──────────────────────────────────────────────────
sex fun vudo_self_id() -> Int32
sex fun vudo_spirit_name(buf: Int32, len: Int32) -> Int32
```

### Example: ping.dol
```dol
sex fun vudo_print(ptr: Int32, len: Int32)
sex fun vudo_send(target: Int32, ptr: Int32, len: Int32)
sex fun vudo_sleep(ms: Int32)

gene PingSpirit {
    has pings_sent: Int64
    has target: Int32
    
    sex fun init(pong_id: Int32) {
        self.target = pong_id
        self.pings_sent = 0
    }
    
    sex fun tick() {
        // Send ping to pong spirit
        vudo_send(self.target, "ping", 4)
        self.pings_sent = self.pings_sent + 1
        vudo_print("Sent ping #")
        // Note: would need int-to-string for full output
        vudo_sleep(100)
    }
}
```

### Example: pong.dol
```dol
sex fun vudo_print(ptr: Int32, len: Int32)
sex fun vudo_send(target: Int32, ptr: Int32, len: Int32)
sex fun vudo_recv(buf: Int32, len: Int32) -> Int32
sex fun vudo_has_message() -> Bool
sex fun vudo_sender() -> Int32

gene PongSpirit {
    has pongs_sent: Int64
    has buffer: Array<Int8, 256>
    
    sex fun tick() {
        if vudo_has_message() {
            let len = vudo_recv(self.buffer, 256)
            let sender = vudo_sender()
            
            self.pongs_sent = self.pongs_sent + 1
            vudo_print("Received message, sending pong")
            
            // Reply to sender
            vudo_send(sender, "pong", 4)
        }
    }
}
```

### Expected Séance Output
```bash
$ vudo compile examples/ping.dol -o ping.wasm
$ vudo compile examples/pong.dol -o pong.wasm

$ vudo seance ping.wasm pong.wasm --ticks 5
[Séance] Starting session with 2 spirits
[Séance] Spirit 0: ping.wasm
[Séance] Spirit 1: pong.wasm
[Spirit:0] Sent ping #1
[Spirit:1] Received message, sending pong
[Spirit:0] Sent ping #2
[Spirit:1] Received message, sending pong
[Spirit:0] Sent ping #3
[Spirit:1] Received message, sending pong
[Spirit:0] Sent ping #4
[Spirit:1] Received message, sending pong
[Spirit:0] Sent ping #5
[Spirit:1] Received message, sending pong
[Séance] Completed 5 ticks
[Séance] Messages exchanged: 10
```

---

## Claude-Flow Workflows

### Workflow 1: Week 1 - SEX Infrastructure

Save as `~/repos/univrs-dol/flow/week1-sex-infrastructure.yml`:

```yaml
name: "week1-sex-infrastructure"
description: "Parse sex keyword, track effects, generate WASM imports"

config:
  working_directory: "~/repos/univrs-dol"
  branch: "feature/sex-system"
  max_agents: 4

phases:
  - name: "lexer-parser"
    agents:
      - name: "syntax-implementer"
        tasks:
          - name: "add-sex-token"
            prompt: |
              Add 'sex' keyword to lexer in src/lexer.rs:
              1. Add Sex variant to Token enum
              2. Add "sex" to keyword matching
              
          - name: "parse-sex-fun"
            prompt: |
              Add sex fun parsing to parser:
              - sex fun name(params) [-> type] [{ body }]
              - No body = import, has body = local
              
          - name: "parse-sex-block"
            prompt: |
              Add sex { } block parsing:
              - sex { statements }
              - Returns SexBlock AST node
              
          - name: "parse-sex-var"
            prompt: |
              Add sex var parsing:
              - sex var NAME: Type [= init]
              - Only valid at module level

  - name: "effect-tracking"
    depends_on: ["lexer-parser"]
    agents:
      - name: "typechecker-implementer"
        tasks:
          - name: "track-context"
            prompt: |
              Add EffectContext to type checker:
              - Track in_sex: bool
              - enter_sex_context() / exit_sex_context()
              - Apply to sex fun body, sex { } blocks, main()
              
          - name: "enforce-boundary"
            prompt: |
              Check effect boundaries:
              - If calling sex fun from pure context → error
              - Error: "Cannot call sex function from pure context"
              - Hint: "Wrap in sex { } block"

  - name: "wasm-imports"
    depends_on: ["effect-tracking"]
    agents:
      - name: "wasm-implementer"
        tasks:
          - name: "collect-imports"
            prompt: |
              Collect sex fun without body as WASM imports:
              - Module: "loa"
              - Name: function name
              - Track import indices
              
          - name: "emit-import-section"
            prompt: |
              Emit WASM import section (0x02):
              - Before function section
              - Import indices 0, 1, 2...
              - Local functions start after imports
              
          - name: "fix-call-indices"
            prompt: |
              Update compile_call for imports:
              - Check if calling import
              - Use import index or local + offset

  - name: "testing"
    depends_on: ["wasm-imports"]
    agents:
      - name: "tester"
        commands:
          - "cargo build --features cli,wasm"
          - "cargo test"
          - "cargo run --bin vudo -- check examples/sex_basic.dol"
          - "cargo run --bin vudo -- check examples/sex_error.dol 2>&1 | grep -i effect"

success_criteria:
  - "sex fun parses"
  - "sex { } blocks parse"
  - "sex-in-pure produces error"
  - "WASM import section generated"
```

### Workflow 2: Week 2 - Strings & I/O

Save as `~/repos/univrs-dol/flow/week2-strings-io.yml`:

```yaml
name: "week2-strings-io"
description: "String literals in data section, HelloWorld Spirit"

config:
  working_directory: "~/repos/univrs-dol"
  branch: "feature/sex-system"
  max_agents: 3

phases:
  - name: "string-literals"
    agents:
      - name: "string-implementer"
        tasks:
          - name: "string-table"
            prompt: |
              Create StringTable for collecting literals:
              - Deduplicate strings
              - Assign offsets
              - Track total size for memory layout
              
          - name: "data-section"
            prompt: |
              Emit WASM data section:
              - Section ID 0x0B
              - (data (i32.const offset) "content")
              - Place after memory section
              
          - name: "compile-string-expr"
            prompt: |
              Compile string literal expression:
              - Look up in string table
              - Push (i32.const ptr, i32.const len)
              - For function args, expand to two params

  - name: "hello-world"
    depends_on: ["string-literals"]
    agents:
      - name: "integration-tester"
        commands:
          - |
            cat > examples/hello_world.dol << 'EOF'
            sex fun vudo_print(ptr: Int32, len: Int32)
            
            fun main() {
                sex {
                    vudo_print("Hello from DOL Spirit!")
                }
            }
            EOF
          - "cargo run --bin vudo -- compile examples/hello_world.dol -o /tmp/hello.wasm"
          - "wasm2wat /tmp/hello.wasm | grep -A5 'data'"
          - "cargo run --bin vudo -- run /tmp/hello.wasm -f main"

success_criteria:
  - "String literals in data section"
  - "vudo_print(\"Hello\") compiles"
  - "Console output: Hello from DOL Spirit!"
```

### Workflow 3: Week 3 - State & Mutation

Save as `~/repos/univrs-dol/flow/week3-state-mutation.yml`:

```yaml
name: "week3-state-mutation"
description: "Field assignment, sex var globals, stateful Spirits"

config:
  working_directory: "~/repos/univrs-dol"
  branch: "feature/sex-system"
  max_agents: 3

phases:
  - name: "field-assignment"
    agents:
      - name: "mutation-implementer"
        tasks:
          - name: "parse-assignment"
            prompt: |
              Parse field assignment:
              - self.field = expr
              - Must be in sex context
              - Create FieldAssign AST node
              
          - name: "compile-store"
            prompt: |
              Compile field assignment to WASM:
              - local.get 0 (self pointer)
              - compile value expression
              - i64.store (offset) or i32.store

  - name: "sex-var"
    depends_on: ["field-assignment"]
    agents:
      - name: "global-implementer"
        tasks:
          - name: "compile-sex-var"
            prompt: |
              Compile sex var to WASM global:
              - (global $NAME (mut i64) (i64.const init))
              - Track in symbol table
              - Read: global.get
              - Write: global.set (only in sex context)

  - name: "stateful-spirit"
    depends_on: ["sex-var"]
    agents:
      - name: "integration-tester"
        commands:
          - |
            cat > examples/counter_stateful.dol << 'EOF'
            sex fun vudo_print(ptr: Int32, len: Int32)
            
            gene Counter {
                has value: Int64
                
                fun get() -> Int64 { return self.value }
                
                sex fun increment() {
                    self.value = self.value + 1
                }
                
                sex fun set(n: Int64) {
                    self.value = n
                }
            }
            EOF
          - "cargo run --bin vudo -- compile examples/counter_stateful.dol -o /tmp/counter.wasm"
          - "cargo run --bin vudo -- run /tmp/counter.wasm -f Counter.set -a '[10]' -f Counter.increment -f Counter.get"

success_criteria:
  - "self.value = x compiles"
  - "sex var compiles to WASM global"
  - "State persists: set(10) + increment() = 11"
```

### Workflow 4: Week 4 - Messaging & Séance

Save as `~/repos/univrs-dol/flow/week4-messaging.yml`:

```yaml
name: "week4-messaging"
description: "Spirit messaging, MessageBus, Séance coordination"

config:
  working_directory: "~/repos/univrs-dol"
  branch: "feature/sex-system"
  max_agents: 4

phases:
  - name: "message-bus"
    agents:
      - name: "runtime-implementer"
        tasks:
          - name: "implement-bus"
            prompt: |
              In packages/vudo-runtime/src/message-bus.ts:
              
              Create MessageBus class:
              - queues: Map<spiritId, Message[]>
              - send(from, to, payload)
              - receive(spiritId) -> Message | null
              - hasMessages(spiritId) -> boolean

  - name: "loa-messaging"
    depends_on: ["message-bus"]
    agents:
      - name: "loa-implementer"
        tasks:
          - name: "add-messaging-loa"
            prompt: |
              Add to coreLoa in packages/vudo-runtime/src/loa.ts:
              
              - vudo_send(target, ptr, len)
              - vudo_recv(buf, maxLen) -> len
              - vudo_has_message() -> bool
              - vudo_sender() -> id

  - name: "seance-coordination"
    depends_on: ["loa-messaging"]
    agents:
      - name: "seance-implementer"
        tasks:
          - name: "multi-spirit"
            prompt: |
              Update Seance class for multi-spirit:
              - summon multiple spirits
              - Each gets unique ID
              - tick() runs all spirits
              - MessageBus routes messages

  - name: "vudo-seance-cli"
    depends_on: ["seance-coordination"]
    agents:
      - name: "cli-implementer"
        tasks:
          - name: "add-seance-command"
            prompt: |
              Add 'vudo seance' command:
              - vudo seance spirit1.wasm spirit2.wasm --ticks N
              - Load all spirits into Séance
              - Run tick loop
              - Report message stats

  - name: "ping-pong-test"
    depends_on: ["vudo-seance-cli"]
    agents:
      - name: "integration-tester"
        commands:
          - "cargo run --bin vudo -- compile examples/ping.dol -o /tmp/ping.wasm"
          - "cargo run --bin vudo -- compile examples/pong.dol -o /tmp/pong.wasm"
          - "cargo run --bin vudo -- seance /tmp/ping.wasm /tmp/pong.wasm --ticks 5"

success_criteria:
  - "MessageBus routes messages"
  - "vudo_send/recv work"
  - "vudo seance runs multiple spirits"
  - "Ping/Pong exchange 10 messages in 5 ticks"
```

---

## Deployment Commands

### Start Week 1 (Immediate)

```bash
# 1. Create branch
cd ~/repos/univrs-dol
git checkout -b feature/sex-system

# 2. Copy workflow
mkdir -p flow
# (copy week1-sex-infrastructure.yml content above)

# 3. Deploy swarm
claude-flow swarm read ./flow/week1-sex-infrastructure.yml --workflow

# Or run manually:
# Start with src/lexer.rs - add Sex token
```

### Validate Progress

```bash
# After each week, run validation:

# Week 1
vudo check examples/sex_basic.dol && echo "✅ Week 1 Complete"

# Week 2
vudo run hello.wasm -f main | grep "Hello" && echo "✅ Week 2 Complete"

# Week 3
vudo run counter.wasm -f Counter.set -a '[10]' -f Counter.increment -f Counter.get | grep "11" && echo "✅ Week 3 Complete"

# Week 4
vudo seance ping.wasm pong.wasm --ticks 5 | grep "Messages exchanged: 10" && echo "✅ Week 4 Complete"
```

### Full 50% Validation

```bash
#!/bin/bash
# validate-50-percent.sh

echo "═══════════════════════════════════════════"
echo "     50% SPIRIT VALIDATION SUITE           "
echo "═══════════════════════════════════════════"

PASS=0
FAIL=0

# Test 1: SEX parsing
echo -n "Test 1: SEX syntax... "
if vudo check examples/sex_basic.dol 2>/dev/null; then
    echo "✅ PASS"; ((PASS++))
else
    echo "❌ FAIL"; ((FAIL++))
fi

# Test 2: Effect enforcement
echo -n "Test 2: Effect boundary... "
if vudo check examples/sex_error.dol 2>&1 | grep -qi "pure\|effect"; then
    echo "✅ PASS"; ((PASS++))
else
    echo "❌ FAIL"; ((FAIL++))
fi

# Test 3: HelloWorld
echo -n "Test 3: HelloWorld Spirit... "
if vudo run /tmp/hello.wasm -f main 2>&1 | grep -q "Hello"; then
    echo "✅ PASS"; ((PASS++))
else
    echo "❌ FAIL"; ((FAIL++))
fi

# Test 4: Stateful Counter
echo -n "Test 4: Stateful mutation... "
RESULT=$(vudo run /tmp/counter.wasm -f Counter.set -a '[10]' -f Counter.increment -f Counter.get 2>&1)
if echo "$RESULT" | grep -q "11"; then
    echo "✅ PASS"; ((PASS++))
else
    echo "❌ FAIL"; ((FAIL++))
fi

# Test 5: Messaging
echo -n "Test 5: Spirit messaging... "
if vudo seance /tmp/ping.wasm /tmp/pong.wasm --ticks 5 2>&1 | grep -qi "message"; then
    echo "✅ PASS"; ((PASS++))
else
    echo "❌ FAIL"; ((FAIL++))
fi

echo "═══════════════════════════════════════════"
echo "Results: $PASS passed, $FAIL failed"
if [ $FAIL -eq 0 ]; then
    echo "🎉 50% SPIRITS ACHIEVED!"
else
    echo "⚠️  Some tests failed"
fi
```

---

## Success Metrics

| Metric | Current | Week 1 | Week 2 | Week 3 | Week 4 |
|--------|---------|--------|--------|--------|--------|
| Spirit Capability | 20% | 30% | 40% | 45% | **50%** |
| Tests Passing | 1,839 | +50 | +30 | +30 | +40 |
| Example Spirits | 1 | 2 | 4 | 6 | 8 |
| CLI Commands | 3 | 3 | 3 | 3 | **4** |
| WASM Sections | 4 | **5** | **6** | 6 | 6 |

---

## Next Steps After 50%

Once 50% is achieved, the path to 100%:

```
50% → 75%                              75% → 100%
═══════════                            ════════════
• Trait compilation                    • Full module system
• Gene inheritance in WASM             • Package resolution
• Array/Slice types                    • Spirit.dol manifest
• Match expressions                    • Mycelium network
• For/while loops                      • Browser playground
• String concatenation                 • Self-hosting POC
```

---

*"Pure by default. Effects explicit. Systems describe what they ARE."*
