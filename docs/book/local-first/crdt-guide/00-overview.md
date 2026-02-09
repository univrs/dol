# CRDT Guide: Overview

This guide provides comprehensive documentation for all seven CRDT strategies supported by VUDO, with real-world examples and best practices.

## What Are CRDTs?

**Conflict-free Replicated Data Types** (CRDTs) are data structures that can be replicated across multiple nodes and updated independently without coordination. They mathematically guarantee that all replicas will eventually converge to the same state.

## Why CRDTs Matter for Local-First

### The Problem

```
Alice (offline):  doc.title = "My Report"
Bob (offline):    doc.title = "Final Report"

When they sync, what should title be?
```

### Traditional Solutions

**1. Last-Write-Wins** (simple but lossy):
```
Result: "Final Report"  ← Alice's edit is lost!
```

**2. Manual Conflict Resolution** (annoying):
```
[Conflict Dialog]
Choose version:
○ My Report (Alice, 2:30 PM)
○ Final Report (Bob, 2:35 PM)
```

**3. Locking** (requires connectivity):
```
Alice: "Acquiring lock..."  [TIMEOUT ERROR]
```

### The CRDT Solution

```
Alice: title = "My Report" (timestamp: t1)
Bob:   title = "Final Report" (timestamp: t2)

CRDT merge: "Final Report"  (deterministic: t2 > t1)
NO user intervention needed!
```

## The Seven Strategies

VUDO supports seven CRDT strategies, each optimized for different use cases:

### Quick Reference

| Strategy | Use Case | Data Loss | Concurrency Handling |
|----------|----------|-----------|---------------------|
| **immutable** | IDs, timestamps | None | First-write-wins |
| **lww** | Metadata, settings | ⚠️ Concurrent updates | Last timestamp wins |
| **or_set** | Tags, collections | None | Add-wins semantics |
| **pn_counter** | Metrics, scores | None | Sum all operations |
| **peritext** | Rich text | None | Character-level merge |
| **rga** | Ordered lists | None | Causal ordering |
| **mv_register** | Conflict detection | None | Keep all values |

### Strategy Selection Flowchart

```
Does the field need to change after creation?
├─ NO → @crdt(immutable)
└─ YES
   ├─ Is it text content?
   │  └─ YES → @crdt(peritext)
   │
   ├─ Is it a number that only increases/decreases?
   │  └─ YES → @crdt(pn_counter)
   │
   ├─ Is it a collection (set, list)?
   │  ├─ Unordered (tags, members)
   │  │  └─ @crdt(or_set)
   │  └─ Ordered (tasks, comments)
   │     └─ @crdt(rga)
   │
   ├─ Do you need to detect conflicts?
   │  └─ YES → @crdt(mv_register)
   │
   └─ Simple single value (name, status)
      └─ @crdt(lww)
```

## Mathematical Guarantees

### Strong Eventual Consistency (SEC)

CRDTs guarantee three properties:

**1. Eventual Delivery**:
```
∀ operation o, ∀ replica r:
  o will eventually be delivered to r
```

**2. Convergence**:
```
∀ replicas r1, r2:
  if received(r1) = received(r2)
  then state(r1) = state(r2)
```

**3. Commutativity**:
```
merge(A, B) = merge(B, A)
```

### Idempotence

Applying the same operation twice has no effect:

```
apply(state, op)
apply(apply(state, op), op) = apply(state, op)
```

This allows safe retransmission without deduplication.

## CRDT Families

### State-Based (CvRDT)

Replicas exchange **entire state**:
```rust
fn merge(local: State, remote: State) -> State {
    // Merge function (commutative, associative, idempotent)
}
```

**Pros**: Simple, automatically convergent
**Cons**: Bandwidth overhead for large states

**Used in**: LWW, OR-Set, PN-Counter

### Operation-Based (CmRDT)

Replicas exchange **operations**:
```rust
fn apply(state: &mut State, op: Operation) {
    // Apply operation to local state
}
```

**Pros**: Lower bandwidth (send operations, not state)
**Cons**: Requires reliable delivery or operation logging

**Used in**: Peritext, RGA

## Type Compatibility Matrix

| DOL Type | immutable | lww | or_set | pn_counter | peritext | rga | mv_register |
|----------|-----------|-----|--------|------------|----------|-----|-------------|
| `String` | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ |
| `Int`, `i64` | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ |
| `Bool` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Enum` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Set<T>` | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| `Vec<T>` | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Custom Struct | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |

## Performance Characteristics

### Time Complexity

| Strategy | Insert | Remove | Merge | Space |
|----------|--------|--------|-------|-------|
| immutable | O(1) | N/A | O(1) | O(1) |
| lww | O(1) | O(1) | O(1) | O(1) |
| or_set | O(1) | O(n) | O(n) | O(n·m) |
| pn_counter | O(1) | O(1) | O(n) | O(n) |
| peritext | O(log n) | O(log n) | O(n+m) | O(n) |
| rga | O(1) | O(1) | O(n·m) | O(n) |
| mv_register | O(1) | O(1) | O(n) | O(k) |

Where:
- `n` = number of elements
- `m` = number of replicas
- `k` = number of concurrent values

### Memory Overhead

**Minimal overhead** (< 10 bytes per operation):
- immutable, lww, pn_counter

**Moderate overhead** (10-50 bytes per operation):
- peritext, rga

**Higher overhead** (> 50 bytes per operation):
- or_set (tracks unique tags per element)
- mv_register (stores all concurrent values)

## Common Patterns

### Pattern 1: Immutable Identity + Mutable Metadata

```dol
gen entity {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has created_at: i64

  @crdt(lww)
  has name: String

  @crdt(lww)
  has status: Status
}
```

**Use case**: Most entities need permanent identity with changeable properties.

### Pattern 2: Collaborative Content + Metadata

```dol
gen document {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has title: String

  @crdt(peritext)
  has content: String

  @crdt(or_set)
  has tags: Set<String>
}
```

**Use case**: Documents, notes, articles with collaborative editing.

### Pattern 3: Ordered Collection with Metadata

```dol
gen task_board {
  @crdt(immutable)
  has id: String

  @crdt(rga)
  has tasks: Vec<Task>

  @crdt(or_set)
  has members: Set<String>

  @crdt(pn_counter, min_value=0)
  has total_points: i64
}
```

**Use case**: Kanban boards, playlists, ordered workflows.

### Pattern 4: Metrics and Counters

```dol
gen post {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has content: String

  @crdt(pn_counter, min_value=0)
  has like_count: i64

  @crdt(pn_counter, min_value=0)
  has view_count: i64

  @crdt(or_set)
  has likers: Set<String>
}
```

**Use case**: Social media, analytics, voting systems.

## Anti-Patterns

### ❌ Anti-Pattern 1: Using LWW for Collaborative Text

```dol
// DON'T DO THIS
@crdt(lww)
has content: String
```

**Problem**: Concurrent edits lose data (only last write survives)

**Solution**: Use peritext for text:
```dol
@crdt(peritext)
has content: String
```

### ❌ Anti-Pattern 2: Using OR-Set for Ordered Data

```dol
// DON'T DO THIS
@crdt(or_set)
has task_order: Set<Task>
```

**Problem**: Sets are unordered; task order is lost

**Solution**: Use RGA for ordered lists:
```dol
@crdt(rga)
has tasks: Vec<Task>
```

### ❌ Anti-Pattern 3: Using Immutable for User-Editable Fields

```dol
// DON'T DO THIS
@crdt(immutable)
has email: String  // Users should be able to change email!
```

**Problem**: Field can never be updated

**Solution**: Use LWW for changeable metadata:
```dol
@crdt(lww)
has email: String
```

### ❌ Anti-Pattern 4: Using PN-Counter for Non-Monotonic Values

```dol
// DON'T DO THIS
@crdt(pn_counter)
has temperature: i64  // Temperature goes up AND down arbitrarily
```

**Problem**: PN-Counter is for monotonic operations (increments/decrements), not arbitrary values

**Solution**: Use LWW for arbitrary numeric values:
```dol
@crdt(lww)
has temperature: i64
```

## Testing CRDTs

### Property-Based Testing

Use `dol-test` for property-based CRDT testing:

```bash
dol test schemas/document.dol --property convergence
```

**Tests**:
1. **Convergence**: All replicas reach same state
2. **Commutativity**: Order of operations doesn't matter
3. **Idempotence**: Duplicate operations are safe

### Example Test

```rust
#[test]
fn test_concurrent_edits_converge() {
    let mut alice_doc = Automerge::new();
    let mut bob_doc = alice_doc.fork();

    // Concurrent edits
    alice_doc.edit_content(0, "Alice");
    bob_doc.edit_content(0, "Bob");

    // Merge
    alice_doc.merge(&bob_doc).unwrap();
    bob_doc.merge(&alice_doc).unwrap();

    // Assert convergence
    assert_eq!(alice_doc.get_content(), bob_doc.get_content());
}
```

## Next Steps

Ready to learn each CRDT strategy in detail?

### Strategy Guides

1. [Immutable](./01-immutable.md) - IDs and permanent identity
2. [Last-Write-Wins (LWW)](./02-lww.md) - Metadata and settings
3. [OR-Set](./03-or-set.md) - Tags and collections
4. [PN-Counter](./04-pn-counter.md) - Metrics and counters
5. [Peritext](./05-peritext.md) - Rich text editing
6. [RGA](./06-rga.md) - Ordered lists
7. [MV-Register](./07-mv-register.md) - Conflict detection

### Practical Guide

- [Choosing a Strategy](./08-choosing-strategy.md) - Decision tree and best practices

### Reference

- [RFC-001: DOL CRDT Annotations](/rfcs/RFC-001-dol-crdt-annotations.md)
- [ADR-001: CRDT Library Selection](/docs/adr/ADR-001-crdt-library.md)

---

**Next**: [Immutable Strategy →](./01-immutable.md)
