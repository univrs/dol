# Host Functions - Effects and Debug Implementation

## Overview

This document describes the implementation of Phases 7-8 of the VUDO runtime host functions:
- **Phase 7: Effects System** (2 host functions)
- **Phase 8: Debug System** (3 host functions)

## Implementation Summary

### Files Created

#### Source Files
1. **packages/vudo-runtime/src/host/effects.ts** - Effects system implementation
2. **packages/vudo-runtime/src/host/debug.ts** - Debug system implementation  
3. **packages/vudo-runtime/src/host/index.ts** - Updated with new exports

#### Test Files
1. **packages/vudo-runtime/tests/host-effects.test.ts** - 23 test cases
2. **packages/vudo-runtime/tests/host-debug.test.ts** - 24 test cases
3. **packages/vudo-runtime/tests/host-integration.test.ts** - 11 integration tests

**Total: 58 new test cases, all passing**

## Effects System (Phase 7)

### Host Functions

#### 1. `vudo_emit_effect(effect_ptr, effect_len) -> ResultCode`

Emits an effect from WASM for the host to handle.

**Parameters:**
- `effect_ptr` (i32): Pointer to effect JSON data
- `effect_len` (i32): Length of JSON data in bytes

**Returns:**
- `ResultCode` (i32): 0 = Success, non-zero = Error

**Features:**
- Parses StandardEffect from JSON
- Validates effect structure (requires `effect_type`)
- Adds timestamp if not provided
- Processes through IEffectHandler
- Notifies matching subscribers
- Queues effects for later retrieval

#### 2. `vudo_subscribe(pattern_ptr, pattern_len) -> subscription_id`

Subscribes to effect pattern notifications.

**Parameters:**
- `pattern_ptr` (i32): Pointer to pattern string
- `pattern_len` (i32): Length of pattern string

**Returns:**
- `subscription_id` (i32): Positive number on success, negative on error

**Pattern Matching:**
- **Exact match**: `"log"` matches only `effect_type: "log"`
- **Wildcard**: `"*"` matches all effect types

### Architecture

```typescript
// Effect Handler Interface
interface IEffectHandler {
  handleEffect(effect: StandardEffect): ResultCode;
  onSubscribe(pattern: string, subscriptionId: number): void;
}

// StandardEffect Type
interface StandardEffect {
  effect_type: string;
  payload: unknown;
  timestamp: number;
}
```

### Key Features

- **Effect Queue**: Stores all emitted effects for inspection
- **Pattern-Based Subscriptions**: Supports exact and wildcard patterns
- **Subscriber Notifications**: Callbacks invoked when matching effects are emitted
- **Error Handling**: Gracefully handles invalid JSON and subscriber errors
- **Memory Management**: Updates memory reference when WASM memory grows

### Usage Example

```typescript
import { createEffectsSystem } from '@vudo/runtime/host';

const memory = new WebAssembly.Memory({ initial: 1 });
const effects = createEffectsSystem(memory);

// Create WASM imports
const imports = {
  vudo: effects.createHostFunctions()
};

// Subscribe to effects programmatically
const callback = (effect) => {
  console.log('Effect received:', effect);
};

effects.subscribe(patternPtr, patternLen, callback);
```

## Debug System (Phase 8)

### Host Functions

#### 1. `vudo_breakpoint()`

Triggers a debugger breakpoint.

**Parameters:** None
**Returns:** void

**Behavior:**
- Calls JavaScript `debugger` statement if debugger attached
- No-op if no debugger present
- Increments breakpoint counter for statistics

#### 2. `vudo_assert(condition, msg_ptr, msg_len)`

Asserts a condition with an error message.

**Parameters:**
- `condition` (i32): Non-zero = true, zero = false
- `msg_ptr` (i32): Pointer to assertion message (UTF-8)
- `msg_len` (i32): Length of message in bytes

**Returns:** void (throws on failure)

**Behavior:**
- If condition is true (non-zero): Silent success
- If condition is false (zero): Logs error and invokes handler
- Handler may throw Error to terminate execution

#### 3. `vudo_panic(msg_ptr, msg_len) -> !`

Panics and terminates Spirit (never returns).

**Parameters:**
- `msg_ptr` (i32): Pointer to panic message (UTF-8)
- `msg_len` (i32): Length of message in bytes

**Returns:** Never returns

**Behavior:**
- Logs panic message
- Invokes handler's onPanic
- Throws PanicError to terminate
- Unrecoverable error

### Architecture

```typescript
// Debug Handler Interface
interface IDebugHandler {
  onBreakpoint(): void;
  onAssertionFailure(message: string): void;
  onPanic(message: string): never;
}

// Custom Error Type
class PanicError extends Error {
  name: 'PanicError';
}
```

### Key Features

- **Statistics Tracking**: Counts breakpoints, assertions, and failures
- **Custom Handlers**: Pluggable behavior for breakpoints, assertions, panics
- **UTF-8 Decoding**: Properly decodes messages from WASM memory
- **Error Recovery**: Assertions can be caught; panics cannot
- **Default Handler**: Provides standard JavaScript debugging behavior

### Usage Example

```typescript
import { createDebugSystem } from '@vudo/runtime/host';

const memory = new WebAssembly.Memory({ initial: 1 });
const debug = createDebugSystem(memory);

// Create WASM imports
const imports = {
  vudo: debug.createHostFunctions()
};

// Check statistics
const stats = debug.getStats();
console.log(`Breakpoints: ${stats.breakpoints}`);
console.log(`Assertions: ${stats.assertions}`);
console.log(`Failures: ${stats.assertionFailures}`);
```

## Integrated Usage

### Creating All Host Functions

```typescript
import { createAllHostFunctions } from '@vudo/runtime/host';

const memory = new WebAssembly.Memory({ initial: 1 });
const hostFunctions = createAllHostFunctions(memory);

const imports = {
  vudo: hostFunctions
};

const instance = await WebAssembly.instantiate(wasmModule, imports);
```

### Using Host Systems

```typescript
import { createHostSystems } from '@vudo/runtime/host';

const memory = new WebAssembly.Memory({ initial: 1 });
const systems = createHostSystems(memory);

// Access effects system
systems.effects.emitEffect(ptr, len);
const queue = systems.effects.getEffectQueue();

// Access debug system
systems.debug.breakpoint();
const stats = systems.debug.getStats();

// Create WASM imports
const imports = {
  vudo: {
    ...systems.effects.createHostFunctions(),
    ...systems.debug.createHostFunctions(),
  }
};
```

### Custom Handlers

```typescript
import {
  createHostSystems,
  type IEffectHandler,
  type IDebugHandler,
} from '@vudo/runtime/host';

const customEffectHandler: IEffectHandler = {
  handleEffect(effect) {
    // Custom effect processing
    return ResultCode.Success;
  },
  onSubscribe(pattern, id) {
    // Custom subscription handling
  }
};

const customDebugHandler: IDebugHandler = {
  onBreakpoint() {
    // Custom breakpoint behavior
  },
  onAssertionFailure(message) {
    // Custom assertion failure handling
  },
  onPanic(message): never {
    // Custom panic handling
    throw new Error(message);
  }
};

const systems = createHostSystems(memory, {
  effects: customEffectHandler,
  debug: customDebugHandler,
});
```

## Test Coverage

### Effects System Tests (23 tests)

**Effect Emission (5 tests)**
- ✓ Should emit valid effects
- ✓ Should add effects to queue
- ✓ Should return error for invalid JSON
- ✓ Should return error for missing effect_type
- ✓ Should set timestamp if not provided

**Subscriptions (8 tests)**
- ✓ Should subscribe to exact pattern
- ✓ Should subscribe to wildcard pattern
- ✓ Should notify exact pattern subscribers
- ✓ Should not notify non-matching subscribers
- ✓ Should notify wildcard subscribers for all effects
- ✓ Should support multiple subscribers to same pattern
- ✓ Should handle subscriber callback errors gracefully
- ✓ Should unsubscribe successfully

**Queue Management (2 tests)**
- ✓ Should clear effect queue
- ✓ Should return immutable queue copy

**Subscription Management (1 test)**
- ✓ Should list all active subscriptions

**Custom Handler (2 tests)**
- ✓ Should use custom effect handler
- ✓ Should notify handler on subscription

**Host Functions (1 test)**
- ✓ Should create host functions for WASM

**Memory Updates (1 test)**
- ✓ Should update memory reference

**DefaultEffectHandler (2 tests)**
- ✓ Should log effects to console
- ✓ Should log subscription notifications

### Debug System Tests (24 tests)

**Breakpoint (3 tests)**
- ✓ Should trigger breakpoint
- ✓ Should increment breakpoint count
- ✓ Should reset breakpoint count

**Assert (4 tests)**
- ✓ Should pass when condition is true
- ✓ Should fail when condition is false
- ✓ Should increment assertion count
- ✓ Should decode UTF-8 message correctly

**Panic (3 tests)**
- ✓ Should call panic handler
- ✓ Should throw PanicError with default handler
- ✓ Should decode UTF-8 panic message correctly

**Statistics (2 tests)**
- ✓ Should track all debug statistics
- ✓ Should reset all statistics

**Host Functions (4 tests)**
- ✓ Should create host functions for WASM
- ✓ Should call breakpoint through host function
- ✓ Should call assert through host function
- ✓ Should call panic through host function

**Memory Updates (1 test)**
- ✓ Should update memory reference

**DefaultDebugHandler (3 tests)**
- ✓ Should trigger debugger statement on breakpoint
- ✓ Should throw Error on assertion failure
- ✓ Should throw PanicError on panic

**PanicError (4 tests)**
- ✓ Should be an instance of Error
- ✓ Should have correct name
- ✓ Should have correct message
- ✓ Should format toString correctly

### Integration Tests (11 tests)

**createAllHostFunctions (2 tests)**
- ✓ Should create all host functions
- ✓ Should work with custom handlers

**createHostSystems (2 tests)**
- ✓ Should create all host systems
- ✓ Should allow direct system access

**Effects and Debug Together (5 tests)**
- ✓ Should handle effect emission with debug assertions
- ✓ Should subscribe to effects with debug breakpoints
- ✓ Should panic on invalid effect format
- ✓ Should track multiple effects with assertions
- ✓ Should handle complex effect subscription patterns

**WASM Import Object (2 tests)**
- ✓ Should create valid WASM import object
- ✓ Should support multiple systems with shared memory

## Performance Characteristics

### Effects System
- **O(1)** effect emission
- **O(n)** subscriber notification (n = number of subscribers)
- **O(1)** subscription/unsubscription
- **Memory**: Queue grows with number of effects (use clearEffectQueue() periodically)

### Debug System
- **O(1)** all operations
- **Memory**: Minimal (only statistics counters)

## Best Practices

1. **Clear Effect Queue**: Call `clearEffectQueue()` periodically to prevent memory growth
2. **Unsubscribe**: Always unsubscribe when no longer needed
3. **Error Handling**: Wrap panic-prone code in try-catch blocks
4. **Custom Handlers**: Use custom handlers for production logging/monitoring
5. **Pattern Matching**: Use specific patterns when possible (avoid wildcard "*" unless needed)

## Next Steps

The effects and debug systems complete Phases 7-8. The remaining phases to implement are:

- **Phase 9**: Host Function Registry (aggregates all 22 functions)
- **Phase 10**: Example WASM modules demonstrating usage

## Related Documentation

- [VUDO Host Functions Specification](./vudo-host-functions.md)
- [DOL ABI Types](../src/abi/types.ts)
- [Host Function Registry](./host-registry.md)
