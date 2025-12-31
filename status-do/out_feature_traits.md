# DOL Feature: Traits

**Generated:** $(date)
**Status:** PARSING & VALIDATION WORKING | WASM NOT YET SUPPORTED

---

## Overview

Traits in DOL define behavioral contracts that entities can implement. They specify what capabilities or behaviors a gene or system must provide.

### Trait Syntax

```dol
trait <name> {
    is <capability>
    is <method>(params) -> ReturnType
}
```

---

## Building DOL

```bash
cargo build --features cli 2>&1 | tail -3
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

## Trait Examples

### Example 1: Greetable Trait

A simple behavioral contract for greeting:

```dol
trait entity.greetable {
  uses entity.identity
  greetable can greet
  greetable can receive.greeting
  greeting is polite
}

exegesis {
  The entity.greetable trait defines behavior for entities that can
  participate in greetings. This trait:

  - uses entity.identity: requires the entity to have an identity
  - can greet: the entity can initiate a greeting
  - can receive.greeting: the entity can be greeted
  - is polite: greetings are courteous (invariant)

  The "uses" keyword declares a dependency on another specification.
  The "can" keyword declares capabilities (what the entity can do).
  The "is" keyword declares a quality or invariant.
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/dol-parse examples/traits/greetable.dol`
✓ examples/traits/greetable.dol (entity.greetable)
    entity.greetable trait using 1 dependencies, 1 behaviors

Summary
  Total:    1
  Success:  1
```

### Example 2: Countable Trait

A trait for countable entities:

```dol
trait counter.countable {
  uses counter.state
  countable has increment
  countable has decrement
  countable has reset
  operations is atomic
}

exegesis {
  The counter.countable trait defines operations on a counter. It depends
  on counter.state (which provides value, min, max properties).

  Operations:
  - increment: increase value by 1 (respecting maximum)
  - decrement: decrease value by 1 (respecting minimum)
  - reset: return value to initial state

  The invariant "operations are atomic" ensures that count changes
  happen indivisibly - no partial updates.
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/dol-parse examples/traits/countable.dol`
✓ examples/traits/countable.dol (counter.countable)
    counter.countable trait using 1 dependencies, 4 behaviors

Summary
  Total:    1
  Success:  1
```

### Example 3: Container Lifecycle Trait

A domain-specific trait for container management:

```dol
trait container.lifecycle {
  uses container.exists

  container is created
  container is starting
  container is running
  container is stopping
  container is stopped
  container is removing
  container is removed

  each transition emits event
}

exegesis {
  The container lifecycle defines the state machine that governs
  container execution. Transitions between states are atomic and
  emit events for observability. The lifecycle ensures predictable
  behavior from creation through removal.
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/dol-parse examples/traits/container.lifecycle.dol`
✓ examples/traits/container.lifecycle.dol (container.lifecycle)
    container.lifecycle trait using 1 dependencies, 8 behaviors

Summary
  Total:    1
  Success:  1
```

### Example 4: Node Discovery Trait

```dol
trait node.discovery {
  uses network.core

  node is discovered
  node is registered
  node is healthy
  node is unreachable

  each discovery emits event
}

exegesis {
  Node discovery is a fundamental capability in distributed systems that enables
  dynamic topology management and service coordination. When a new node joins a
  distributed cluster, it must be discovered by existing nodes through mechanisms
  like heartbeat protocols, gossip protocols, or centralized registry services.
  The discovery process involves initial detection, registration with the cluster
  membership, and ongoing health monitoring to track node availability. A node
  transitions through states from discovered to registered, maintains a healthy
  status through periodic liveness checks, and may become unreachable due to
  network partitions or failures. Each discovery event triggers notifications
  that allow other system components to react appropriately, such as load
  balancers updating routing tables or consensus algorithms recalculating quorums.
}
```


---

## Trait Feature Status

| Feature | Status | Notes |
|---------|--------|-------|
| Trait declaration | WORKING | `trait name { }` |
| Capability specs | WORKING | `is capability` |
| Method signatures | WORKING | `is method(args) -> Type` |
| Exegesis docs | WORKING | Full documentation support |
| Parse to AST | WORKING | Complete |
| Validate | WORKING | Semantic checks pass |
| WASM compile | NOT YET | Requires vtable lowering |

---

## WASM Status

Traits do not currently compile to WASM. The error when attempting:

```
WasmError: Only function declarations can be compiled to WASM
```

**Roadmap:** Trait WASM support requires implementing virtual tables and interface dispatch.

---

*Generated by DOL Feature Demo Script*
