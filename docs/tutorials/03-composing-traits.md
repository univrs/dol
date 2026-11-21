# Tutorial 3: Composing Traits

> Applies to `dol` **0.8.2**. Every `dol` code block in this tutorial parses as a complete file with
> `metadol::parse_file_all`.

## What are traits?

Genes are atomic units. **Traits** compose genes (and other traits) into behaviours: lifecycles, event
contracts, capabilities.

- **Genes** are the elementary particles.
- **Traits** are atoms built from them.
- **Systems** are molecules ([Tutorial 4](04-system-design.md)).

## Trait syntax

```dol
trait container.lifecycle {
    uses container.exists
    uses container.state

    has current_phase: string

    container is created
    container is started
    container is stopped

    transition emits lifecycle.event
    each transition emits event

    law idempotent_stop(x: i64) { x == x }
}

docs {
    Lifecycle management for containers: created, started, stopped.
}
```

A trait body accepts:

| Form | Meaning |
|---|---|
| `uses name` | Composes another gene or trait. There is no version on the reference. |
| `has name: type [= default]` | Typed field, as in genes |
| `subject has property` | Untyped property |
| `subject is state` | State or category. The subject is required. |
| `subject emits event` | Event contract. The subject is required. |
| `subject requires dep`, `subject derives from origin` | Dependencies and origins |
| `each ...` / `all ...` | Quantified statement, kept as a phrase |
| `law name(params) { expr }` | Algebraic law. Parsed, but laws are not stored in the trait's AST in 0.8.2. |
| `fun ...` | Function (see [05-dol2-features.md](05-dol2-features.md)) |

A few forms from older tutorials don't parse:
- **Versioned references** (`uses x @1.0.0`).
- **Bare predicates** (`emits x`, `is lifecycle`).
- **`quantified by ...`** (there is no such predicate).

Also, `state` is a keyword, so don't use it as a plain word in statements like `all state is valid`.

## The `uses` statement

```dol
trait container.lifecycle {
    uses container.exists
    uses container.lifecycle.started
    uses container.lifecycle.stopped
    uses container.lifecycle.failed

    lifecycle is state_machine
}

docs {
    Complete lifecycle management for containers.
}
```

`uses` records a composition edge in the AST. The validator reports a duplicate `uses` of the same name
as an error. It also rejects `uses` inside a `gen`, because only traits compose.

## Events: `subject emits event`

```dol
trait container.monitored {
    uses container.exists

    container emits container.started
    container emits container.stopped
    container emits container.health_check
    container emits container.error

    has monitoring_interval: u32
    has alert_threshold: f64

    container is observable
}

docs {
    A container that emits monitoring events.
}
```

## Building a trait from genes

### Step 1: define the genes

```dol
gen container.exists {
    has id: string
    has image: string
    container is entity
}

docs {
    A container exists in the system.
}

gen container.created {
    container requires container.exists
    has created_at: u64
}

docs {
    The container has been created but not started.
}

gen container.running {
    container requires container.exists
    has pid: u32
    has started_at: u64
}

docs {
    The container is actively running.
}

gen container.stopped {
    container requires container.exists
    has exit_code: i32
    has stopped_at: u64
}

docs {
    The container has been stopped.
}
```

### Step 2: compose them

```dol
trait container.lifecycle {
    uses container.exists
    uses container.created
    uses container.running
    uses container.stopped

    lifecycle emits lifecycle.created
    lifecycle emits lifecycle.started
    lifecycle emits lifecycle.stopped

    has current_phase: enum { Created, Running, Stopped }
    has phase_history: Vec<string>

    lifecycle is state_machine
    lifecycle is auditable
}

docs {
    Complete lifecycle management for containers.

    This trait models the container state machine: created, then running,
    then stopped. It emits an event at each transition. Implementations
    must keep exactly one active phase at a time and emit events in order.
}
```

### Step 3: layer more behaviour on top

```dol
trait container.lifecycle.monitored {
    uses container.lifecycle

    health emits health.check
    health emits health.failed
    health emits health.recovered

    has health_check_interval: u32
    has health_check_timeout: u32
    has health_check_retries: u8

    lifecycle is monitored
}

docs {
    Lifecycle management with health monitoring.
}
```

## Laws

Traits can state algebraic laws as boolean expressions over typed parameters:

```dol
trait semigroup {
    uses combinable
    law associative(a: i64, b: i64, c: i64) {
        (a + b) + c == a + (b + c)
    }
}

docs {
    A semigroup: combining values is associative.
}
```

In 0.8.2, laws are parsed and type-annotated, but they are not kept on the `Trait` AST node and
nothing checks them.

## Composition patterns

### Layers

```dol
trait entity.basic {
    uses entity.exists
}

docs {
    Layer 1: the base entity.
}

trait entity.persistent {
    uses entity.basic
    uses entity.storable
}

docs {
    Layer 2: an entity that can be stored.
}

trait entity.managed {
    uses entity.persistent
    uses entity.auditable
    uses entity.versioned
}

docs {
    Layer 3: a fully managed entity.
}
```

### Aspects

Model cross-cutting concerns as traits, then compose them:

```dol
trait aspect.auditable {
    audit emits audit.created
    audit emits audit.updated
    audit emits audit.deleted
    has audit_log: Vec<string>
}

docs {
    Audit trail tracking for any entity.
}

trait aspect.cacheable {
    has cache_key: string
    has cache_ttl: u32
    entity is cacheable
}

docs {
    Caching behaviour for any entity.
}

trait user.profile.managed {
    uses user.profile
    uses aspect.auditable
    uses aspect.cacheable
}

docs {
    A user profile with auditing and caching applied.
}
```

### State machines

```dol
trait order.lifecycle {
    uses order.pending
    uses order.processing
    uses order.completed
    uses order.cancelled

    order emits order.created
    order emits order.processing_started
    order emits order.completed
    order emits order.cancelled

    has current_phase: enum { Pending, Processing, Completed, Cancelled }

    order is state_machine
}

docs {
    Order processing state machine.
}
```

## Best practices

1. **One behavioural concern per trait.** Prefer `user.auth` and `user.personalization` over a single
   `user.everything`.
2. **Document event contracts in `docs`.** List what each event means:

```dol
trait payment.processed {
    payment emits payment.initiated
    payment emits payment.authorized
    payment emits payment.captured
    payment emits payment.failed
}

docs {
    Payment processing with an event-driven workflow.

    Events:
    - payment.initiated: payment request received
    - payment.authorized: payment authorized by the provider
    - payment.captured: funds captured
    - payment.failed: payment failed at any stage
}
```

3. **Avoid cycles.** If trait `a` uses `b` and `b` uses `a`, the composition is meaningless.
   The 0.8.2 validator does **not** detect cycles, so keep the graph acyclic yourself.

## Checking your traits

```bash
cargo run --features cli --bin dol-parse -- my-traits.dol
cargo run --features cli --bin dol-check -- --require-exegesis my-traits.dol
```

Both tools only parse the **first declaration** of each file in 0.8.2. There is no
`--check-dependencies` flag, and `uses` references are not resolved against other files.

## Key takeaways

1. **Traits compose genes and traits** with `uses name`.
2. **Events and states need a subject:** `x emits e`, `x is s`.
3. **Laws** express algebraic properties. They are parsed but not yet checked.
4. **Document** each trait and its events in `docs`.
