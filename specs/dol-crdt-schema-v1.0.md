# DOL CRDT Schema Specification v1.0

**Status:** Proposed Standard
**Version:** 1.0.0
**Date:** 2026-02-05
**Authors:** DOL Working Group, Univrs Foundation
**License:** CC BY 4.0

---

## Abstract

This document defines the DOL CRDT Schema specification, an open standard for annotating data structures with Conflict-free Replicated Data Type (CRDT) merge strategies. The specification enables automatic conflict resolution in distributed systems through declarative schema definitions, providing a formal, machine-readable format for CRDT semantics.

The DOL CRDT Schema format is designed to be:
- **Language-agnostic**: Implementable in any programming language
- **Platform-independent**: Usable across web, mobile, desktop, and embedded systems
- **Formally verifiable**: Mathematically grounded in CRDT theory
- **Interoperable**: Enabling cross-library data exchange and collaboration

## Table of Contents

1. [Introduction](#1-introduction)
2. [Notation and Conventions](#2-notation-and-conventions)
3. [Core Concepts](#3-core-concepts)
4. [CRDT Annotation Syntax](#4-crdt-annotation-syntax)
5. [CRDT Strategy Specifications](#5-crdt-strategy-specifications)
6. [Type Compatibility Matrix](#6-type-compatibility-matrix)
7. [Constraint Interaction Model](#7-constraint-interaction-model)
8. [Schema Evolution Rules](#8-schema-evolution-rules)
9. [Serialization Format](#9-serialization-format)
10. [Validation Requirements](#10-validation-requirements)
11. [Security Considerations](#11-security-considerations)
12. [Interoperability Guidelines](#12-interoperability-guidelines)
13. [Conformance](#13-conformance)
14. [References](#14-references)

---

## 1. Introduction

### 1.1 Purpose

The DOL CRDT Schema specification provides a standard way to annotate data structures with CRDT merge strategies, enabling:

1. **Automatic conflict resolution** in distributed systems
2. **Offline-first application development** with guaranteed convergence
3. **Interoperability** between different CRDT implementations
4. **Type-safe CRDT usage** with compile-time validation

### 1.2 Scope

This specification defines:

- Syntax for CRDT annotations in schema definitions
- Semantics for seven core CRDT strategies
- Type compatibility rules
- Evolution and migration patterns
- Validation and conformance requirements

This specification does NOT define:

- Wire protocols for CRDT synchronization
- Storage formats for CRDT state
- Implementation details of CRDT algorithms
- User interface patterns for conflict presentation

### 1.3 Design Goals

**G1. Declarative Simplicity**: Schema authors should express merge intent without understanding CRDT implementation details.

**G2. Mathematical Correctness**: All specified strategies guarantee Strong Eventual Consistency (SEC).

**G3. Type Safety**: Invalid CRDT-type combinations must be detectable at schema validation time.

**G4. Composability**: CRDT-annotated types should compose naturally into larger structures.

**G5. Evolvability**: Schemas should evolve over time with safe migration paths.

### 1.4 Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

**Additional terms:**

- **CRDT**: Conflict-free Replicated Data Type
- **Replica**: An independent copy of a data structure
- **Merge**: The operation combining two replica states
- **Strategy**: A specific CRDT algorithm (LWW, OR-Set, etc.)
- **Annotation**: Metadata attached to a field specifying its CRDT strategy
- **Convergence**: The property that all replicas reach identical state after receiving all operations

---

## 2. Notation and Conventions

### 2.1 EBNF Grammar

Grammar definitions use Extended Backus-Naur Form (EBNF) as defined in ISO/IEC 14977.

Conventions:
- `"literal"` - terminal symbols
- `rule = expression;` - production rules
- `|` - alternation
- `[]` - optional
- `{}` - repetition (zero or more)
- `()` - grouping

### 2.2 JSON Schema

JSON examples and schemas follow RFC 8259 and JSON Schema Draft 2020-12.

### 2.3 Type Notation

Type signatures use a Rust-inspired notation:
- `T` - generic type parameter
- `Option<T>` - optional value
- `Set<T>` - unordered collection
- `List<T>` or `Vec<T>` - ordered sequence
- `Map<K, V>` - key-value mapping

---

## 3. Core Concepts

### 3.1 Conflict-Free Replicated Data Types

A CRDT is a data type that guarantees **Strong Eventual Consistency** (SEC):

**Definition (SEC)**: A replicated object satisfies SEC if:

1. **Eventual Delivery**: All operations eventually reach all replicas
2. **Convergence**: Replicas receiving the same operations reach identical states
3. **Termination**: Merge operations complete in finite time

**Theorem (CRDT Correctness)**: A data type is a CRDT if its merge function is:
- **Commutative**: merge(A, B) = merge(B, A)
- **Associative**: merge(merge(A, B), C) = merge(A, merge(B, C))
- **Idempotent**: merge(A, A) = A

### 3.2 CRDT Families

#### 3.2.1 State-Based CRDTs (CvRDT)

Replicas exchange entire states. Merge combines states.

**Requirements:**
- Merge function MUST be commutative, associative, and idempotent
- State MUST form a join-semilattice

#### 3.2.2 Operation-Based CRDTs (CmRDT)

Replicas exchange operations. Apply function updates state.

**Requirements:**
- Operations MUST be commutative
- Delivery MUST be exactly-once or idempotent

### 3.3 DOL CRDT Annotation Model

DOL extends schema definitions with `@crdt` annotations:

```
@crdt(strategy[, option=value]*)
field_name: type
```

**Semantics:**
1. The annotation declares the merge strategy for `field_name`
2. Implementations MUST use the specified strategy for conflict resolution
3. Writes to the field MUST be translated to CRDT operations
4. Merges MUST follow the strategy's convergence rules

---

## 4. CRDT Annotation Syntax

### 4.1 Grammar

```ebnf
(* CRDT Annotation *)
crdt_annotation = '@crdt', '(', crdt_strategy, [ ',', crdt_options ], ')' ;

(* CRDT Strategies *)
crdt_strategy = 'immutable'
              | 'lww'           (* Last-Write-Wins *)
              | 'or_set'        (* Observed-Remove Set *)
              | 'pn_counter'    (* Positive-Negative Counter *)
              | 'peritext'      (* Rich Text CRDT *)
              | 'rga'           (* Replicated Growable Array *)
              | 'mv_register' ; (* Multi-Value Register *)

(* Options *)
crdt_options = option_pair, { ',', option_pair } ;
option_pair = identifier, '=', value ;
value = string_literal | number | boolean ;

(* Field Declaration *)
annotated_field = [ crdt_annotation ], field_name, ':', type_spec ;
```

### 4.2 Annotation Placement

CRDT annotations MUST appear immediately before the field declaration they annotate.

**Valid:**
```
@crdt(lww)
name: String
```

**Invalid:**
```
name: String @crdt(lww)  // ✗ Annotation after field
```

### 4.3 Strategy Identifiers

Strategy identifiers are case-sensitive and MUST be one of:

| Identifier | Full Name | Section |
|------------|-----------|---------|
| `immutable` | Immutable Register | §5.1 |
| `lww` | Last-Write-Wins Register | §5.2 |
| `or_set` | Observed-Remove Set | §5.3 |
| `pn_counter` | Positive-Negative Counter | §5.4 |
| `peritext` | Peritext Rich Text | §5.5 |
| `rga` | Replicated Growable Array | §5.6 |
| `mv_register` | Multi-Value Register | §5.7 |

### 4.4 Options

Options are strategy-specific key-value pairs:

**General Options** (supported by multiple strategies):
- `tie_break`: `"actor_id"` | `"content_hash"` | `"custom"`
  - Deterministic tie-breaking for concurrent operations
  - Default: `"actor_id"`

**Strategy-Specific Options** (see §5 for details):
- `min_value`: Integer (pn_counter)
- `max_value`: Integer (pn_counter)
- `overflow_strategy`: `"saturate"` | `"wrap"` | `"error"` (pn_counter)
- `formatting`: `"full"` | `"markdown"` | `"plain"` (peritext)
- `max_length`: Integer (peritext)

---

## 5. CRDT Strategy Specifications

### 5.1 Immutable Strategy

**Identifier:** `immutable`

**Semantics:** Value is set exactly once at creation time and never modified.

**Use Cases:**
- Unique identifiers (UUIDs, database IDs)
- Creation timestamps
- Immutable metadata (author, origin)

**Merge Rule:**

```
merge(local, remote, meta_local, meta_remote) =
  if meta_local.timestamp < meta_remote.timestamp then
    local  // Keep earlier value (first-write-wins)
  else if meta_local.timestamp = meta_remote.timestamp then
    if meta_local.actor_id < meta_remote.actor_id then
      local  // Deterministic tie-break
    else
      remote
  else
    local
```

**Properties:**
- First-write-wins (earliest timestamp)
- Deterministic tie-breaking by actor ID
- Idempotent: merge(A, A) = A
- Commutative: merge(A, B) = merge(B, A)

**Type Compatibility:** Any type `T`

**Options:** None

**Example:**
```
@crdt(immutable)
id: Uuid

@crdt(immutable)
created_at: Timestamp
```

**Validation Rules:**
- V1.1: Field MUST be set exactly once
- V1.2: Subsequent write attempts MUST be rejected
- V1.3: Schema evolution MUST NOT change immutable fields

---

### 5.2 Last-Write-Wins (LWW) Strategy

**Identifier:** `lww`

**Semantics:** Most recent write (by Lamport timestamp) wins. Concurrent writes resolved by actor ID.

**Use Cases:**
- User metadata (name, email, avatar)
- Configuration settings
- Status flags
- Simple attributes

**Merge Rule:**

```
merge(local, remote, ts_local, ts_remote) =
  if ts_remote > ts_local then
    remote
  else if ts_remote = ts_local then
    // Tie-break by actor_id or content_hash
    tie_break(local, remote)
  else
    local
```

**Properties:**
- Last-write-wins (latest timestamp)
- Data loss possible (concurrent updates)
- Idempotent and commutative
- Low overhead (timestamp + value)

**Type Compatibility:**
- Scalar types: `String`, `Int`, `Float`, `Bool`, `Enum`
- Composite types: Custom structs
- NOT compatible: Collections (`Set`, `List`, `Map`)

**Options:**
- `tie_break`: `"actor_id"` (default) | `"content_hash"` | `"custom"`

**Example:**
```
@crdt(lww)
name: String

@crdt(lww, tie_break="content_hash")
bio: String  // Content-based tie-breaking
```

**Validation Rules:**
- V2.1: Type MUST be scalar or composite struct
- V2.2: Timestamps MUST be monotonic within each replica
- V2.3: Tie-break MUST be deterministic

**Warning:** LWW can lose concurrent updates. Use `mv_register` to detect conflicts or `peritext` for text collaboration.

---

### 5.3 Observed-Remove Set (OR-Set) Strategy

**Identifier:** `or_set`

**Semantics:** Add-wins semantics. Each element tagged with unique ID. Remove only deletes observed tags.

**Use Cases:**
- Tags and labels
- Collaborator lists
- Membership sets
- Attachments

**Data Structure:**

```
struct ORSet<T> {
  elements: Map<T, Set<UniqueTag>>  // Element → set of unique tags
  tombstones: Set<UniqueTag>        // Removed tags
}
```

**Operations:**

```
add(element: T) -> UniqueTag:
  tag = generate_unique_tag()
  elements[element].add(tag)
  return tag

remove(element: T):
  if element in elements:
    tombstones.add_all(elements[element])

contains(element: T) -> bool:
  if element not in elements:
    return false
  return exists tag in elements[element] where tag not in tombstones
```

**Merge Rule:**

```
merge(local: ORSet<T>, remote: ORSet<T>) -> ORSet<T>:
  result.elements = union(local.elements, remote.elements)
  result.tombstones = union(local.tombstones, remote.tombstones)
  return result
```

**Properties:**
- Add-wins: concurrent add + remove → element present
- Idempotent and commutative
- Preserves all additions across merges
- Space overhead: O(n × m) where n=elements, m=operations per element

**Type Compatibility:**
- `Set<T>` where `T` is hashable and comparable

**Options:** None

**Example:**
```
@crdt(or_set)
tags: Set<String>

@crdt(or_set)
collaborators: Set<UserId>
```

**Validation Rules:**
- V3.1: Type MUST be `Set<T>`
- V3.2: Element type `T` MUST be hashable
- V3.3: Unique tags MUST be globally unique (UUID or (ActorId, Counter))

---

### 5.4 Positive-Negative Counter (PN-Counter) Strategy

**Identifier:** `pn_counter`

**Semantics:** Commutative counter supporting increment and decrement. Each replica maintains separate increment/decrement counters.

**Use Cases:**
- Like counts
- Vote scores
- Inventory quantities
- Metrics and analytics

**Data Structure:**

```
struct PNCounter {
  increments: Map<ActorId, UInt>  // Per-actor increments
  decrements: Map<ActorId, UInt>  // Per-actor decrements
}
```

**Operations:**

```
increment(amount: UInt):
  increments[actor_id] += amount

decrement(amount: UInt):
  decrements[actor_id] += amount

value() -> Int:
  return sum(increments.values()) - sum(decrements.values())
```

**Merge Rule:**

```
merge(local: PNCounter, remote: PNCounter) -> PNCounter:
  for each actor in union(local.actors, remote.actors):
    result.increments[actor] = max(local.increments[actor], remote.increments[actor])
    result.decrements[actor] = max(local.decrements[actor], remote.decrements[actor])
  return result
```

**Properties:**
- Commutative: operations apply in any order
- Monotonic per-replica: each counter only grows
- Eventual consistency: all replicas converge to same value
- Space overhead: O(n) where n=number of replicas

**Type Compatibility:**
- `Int`, `i64`, `i32`
- `UInt`, `u64`, `u32` (with `min_value=0`)
- NOT recommended: `Float` (precision loss)

**Options:**
- `min_value`: Integer (enforce lower bound, default: none)
- `max_value`: Integer (enforce upper bound, default: none)
- `overflow_strategy`: `"saturate"` | `"wrap"` | `"error"` (default: `"saturate"`)

**Example:**
```
@crdt(pn_counter, min_value=0)
likes: Int

@crdt(pn_counter)
karma_score: Int

@crdt(pn_counter, min_value=0, max_value=100, overflow_strategy="saturate")
completion_percentage: Int
```

**Validation Rules:**
- V4.1: Type MUST be integer
- V4.2: If `min_value` specified, value MUST be >= min_value
- V4.3: If `max_value` specified, value MUST be <= max_value
- V4.4: Overflow MUST be handled according to `overflow_strategy`

---

### 5.5 Peritext Strategy

**Identifier:** `peritext`

**Semantics:** Collaborative rich text editing with formatting. Combines character-level CRDT (RGA) with mark-based formatting.

**Use Cases:**
- Document content
- Comments and notes
- Rich text fields
- Markdown editing

**Architecture:**
- Character sequence: RGA (Replicated Growable Array)
- Formatting marks: Range-based with expand semantics
- Cursor positions: Causally ordered

**Operations:**

```
insert(position: UInt, text: String)
delete(position: UInt, length: UInt)
format(range: Range, mark: FormatMark)  // bold, italic, link, etc.
```

**Merge Rule:**

Uses RGA merge for character sequence plus mark conflict resolution.

**Properties:**
- Character-level merge (no data loss)
- Formatting preserves user intent (expand-left/right semantics)
- Causal consistency for cursor positions
- Space overhead: O(n) where n=character count

**Type Compatibility:**
- `String` (plain text, auto-converted)
- `RichText` (native rich text type)

**Options:**
- `formatting`: `"full"` | `"markdown"` | `"plain"` (default: `"full"`)
- `max_length`: Integer (character limit, default: unlimited)

**Example:**
```
@crdt(peritext, formatting="full")
content: String

@crdt(peritext, formatting="markdown", max_length=100000)
description: String
```

**Validation Rules:**
- V5.1: Type MUST be `String` or `RichText`
- V5.2: If `max_length` specified, length MUST be <= max_length
- V5.3: Formatting marks MUST be valid for specified formatting mode

**Implementation Note:** Typically implemented using Automerge.Text, Yjs.Text, or Loro.Text.

---

### 5.6 Replicated Growable Array (RGA) Strategy

**Identifier:** `rga`

**Semantics:** Ordered sequence with causal insertion order. Each element has unique ID and "left origin" reference.

**Use Cases:**
- Task lists
- Ordered collections
- Playlists
- Timeline events

**Data Structure:**

```
struct RGA<T> {
  sequence: Vec<Vertex<T>>
  tombstones: Set<VertexId>
}

struct Vertex<T> {
  id: VertexId              // Unique (ActorId, Counter)
  element: T
  left_origin: Option<VertexId>  // Predecessor
  timestamp: Timestamp
}
```

**Operations:**

```
insert(position: UInt, element: T) -> VertexId:
  left = visible_vertex_at(position - 1)
  vertex = Vertex {
    id: new_vertex_id(),
    element: element,
    left_origin: left.id,
    timestamp: now()
  }
  insert_by_causal_order(vertex)
  return vertex.id

delete(position: UInt):
  vertex = visible_vertex_at(position)
  tombstones.add(vertex.id)
```

**Merge Rule:**

```
merge(local: RGA<T>, remote: RGA<T>) -> RGA<T>:
  all_vertices = union(local.sequence, remote.sequence)
  result.sequence = topological_sort(all_vertices, by_causal_order)
  result.tombstones = union(local.tombstones, remote.tombstones)
  return result

by_causal_order(v1, v2):
  if v1.id = v2.left_origin:
    return v1 < v2
  else if v2.id = v1.left_origin:
    return v2 < v1
  else if v1.timestamp < v2.timestamp:
    return v1 < v2
  else if v1.timestamp = v2.timestamp:
    return v1.id < v2.id  // Deterministic tie-break
  else:
    return v2 < v1
```

**Properties:**
- Causal ordering preserved
- Concurrent inserts at same position ordered deterministically
- Tombstone-based deletion
- Space overhead: O(n) where n=total insertions (including deleted)

**Type Compatibility:**
- `List<T>`, `Vec<T>`, `Array<T>`

**Options:** None

**Example:**
```
@crdt(rga)
tasks: List<TaskId>

@crdt(rga)
column_order: List<ColumnId>
```

**Validation Rules:**
- V6.1: Type MUST be ordered sequence (`List`, `Vec`, `Array`)
- V6.2: Vertex IDs MUST be unique across all replicas
- V6.3: Causal order MUST be deterministic

**Garbage Collection:** Tombstones MAY be removed after all replicas observe deletion (requires coordination).

---

### 5.7 Multi-Value Register (MV-Register) Strategy

**Identifier:** `mv_register`

**Semantics:** Keeps all concurrent values until explicitly resolved. Detects conflicts for application-level resolution.

**Use Cases:**
- Conflict detection
- User-driven conflict resolution
- Configuration with divergent preferences
- Any scenario requiring conflict awareness

**Data Structure:**

```
struct MVRegister<T> {
  values: Map<VectorClock, T>
}
```

**Operations:**

```
set(value: T, clock: VectorClock):
  // Remove causally dominated values
  values.retain(|vc, _| !clock.dominates(vc))
  values.insert(clock, value)

get() -> Vec<T>:
  return values.values()

has_conflict() -> bool:
  return values.len() > 1
```

**Merge Rule:**

```
merge(local: MVRegister<T>, remote: MVRegister<T>) -> MVRegister<T>:
  result.values = {}

  for (clock, value) in union(local.values, remote.values):
    // Keep value if not dominated by any other value
    is_dominated = exists (other_clock, _) in result.values
                   where other_clock.dominates(clock) and other_clock != clock

    if not is_dominated:
      result.values.insert(clock, value)

  // Remove dominated values
  result.values.retain(|vc1, _| {
    !exists (vc2, _) in result.values where vc2 != vc1 and vc2.dominates(vc1)
  })

  return result
```

**Properties:**
- No data loss (all concurrent values preserved)
- Conflict detection (multiple values = conflict)
- Application chooses resolution strategy
- Space overhead: O(k) where k=concurrent value count

**Type Compatibility:** Any type `T`

**Options:** None

**Example:**
```
@crdt(mv_register)
theme: Theme  // Keep all concurrent theme changes

@crdt(mv_register)
config: AppConfig
```

**Validation Rules:**
- V7.1: Applications MUST handle multi-value case
- V7.2: Vector clocks MUST be properly maintained
- V7.3: Dominated values MUST be removed during merge

**Resolution Strategies:**
- User selection (present choices to user)
- Union (merge all values if applicable)
- Custom logic (application-specific rules)

---

## 6. Type Compatibility Matrix

The following matrix specifies which CRDT strategies are compatible with which type categories.

| Type Category | immutable | lww | or_set | pn_counter | peritext | rga | mv_register |
|---------------|-----------|-----|--------|------------|----------|-----|-------------|
| `String` | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ |
| `Int`, `UInt` | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ |
| `Float` | ✅ | ✅ | ❌ | ⚠️ | ❌ | ❌ | ✅ |
| `Bool` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Enum` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Uuid`, `Id` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Timestamp` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Set<T>` | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| `List<T>`, `Vec<T>` | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| `Map<K,V>` | ❌ | ❌ | ⚠️ | ❌ | ❌ | ❌ | ✅ |
| `RichText` | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Custom Struct | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Option<T>` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |

**Legend:**
- ✅ **Compatible**: Natural fit, recommended
- ⚠️ **Caution**: Works but has limitations (see notes)
- ❌ **Incompatible**: Type mismatch, MUST NOT be used

**Notes:**

1. **Float with pn_counter** (⚠️): Floating-point precision issues may cause convergence problems. Use integer counters when possible.

2. **Map with or_set** (⚠️): Maps can be modeled as OR-Set of key-value pairs, but per-key strategies are often more appropriate. Consider using nested CRDT annotations for map values.

3. **Nested Collections**: For types like `List<Set<T>>`, apply CRDT annotations at each level:
   ```
   @crdt(rga)  // Outer list
   items: List<@crdt(or_set) Set<Tag>>  // Inner sets
   ```

### 6.1 Validation Rule

**V6.1**: Schema validators MUST reject incompatible strategy-type combinations based on this matrix.

**Example Error:**
```
Error: Invalid CRDT strategy
  Field: tags
  Type: Set<String>
  Strategy: lww
  Reason: LWW strategy is incompatible with Set types. Use or_set instead.
```

---

## 7. Constraint Interaction Model

CRDT strategies interact with schema constraints in three categories:

### 7.1 Category A: CRDT-Safe Constraints

**Definition**: Constraints that are automatically preserved by CRDT merge semantics.

**Examples:**
- Immutability: `field never changes`
- Monotonicity: `counter always increases`
- Set membership: `element in set`

**Enforcement**: Compile-time (type system + CRDT strategy)

**Validation Requirement**: None (guaranteed by CRDT semantics)

### 7.2 Category B: Eventually-Consistent Constraints

**Definition**: Constraints that may be temporarily violated during concurrent operations but eventually converge to valid state.

**Examples:**
- Uniqueness: `all entities have unique email`
- Resource limits: `used <= quota`
- Referential integrity: `foreign_key exists`

**Enforcement**: Merge-time (soft validation, flag for reconciliation)

**Validation Requirement**: Implementations SHOULD detect violations but MUST NOT block merges.

### 7.3 Category C: Strong-Consistency Constraints

**Definition**: Constraints requiring coordination (consensus, escrow, locks) to maintain.

**Examples:**
- Atomic transfers: `A.balance -= X AND B.balance += X`
- Global uniqueness: `all ids are globally unique`
- Access control: `user can_edit IFF user in permissions`

**Enforcement**: Operation-time (before commit, requires coordination)

**Validation Requirement**: Implementations MUST reject operations violating these constraints before applying to local state.

### 7.4 Interaction Rules

**Rule C1**: CRDT strategies MUST preserve Category A constraints.

**Rule C2**: Implementations SHOULD detect Category B constraint violations and provide reconciliation hooks.

**Rule C3**: Category C constraints REQUIRE explicit coordination mechanisms (escrow, BFT consensus, etc.) outside CRDT merge.

---

## 8. Schema Evolution Rules

### 8.1 Safe Migrations

The following CRDT strategy changes are SAFE (no data loss):

| From | To | Migration Rule |
|------|-----|----------------|
| `immutable` | `lww` | Existing value becomes LWW-timestamped |
| `lww` | `mv_register` | Single value becomes multi-value set |
| `or_set` | `rga` | Unordered set becomes ordered list (deterministic ordering) |
| (none) | any strategy | Field becomes CRDT-replicated (default: `lww`) |

### 8.2 Unsafe Migrations

The following migrations are UNSAFE (data loss or semantic change):

| From | To | Reason |
|------|-----|--------|
| `lww` | `immutable` | Cannot "un-mutate" a field |
| `or_set` | `pn_counter` | Semantic mismatch (set ≠ counter) |
| `peritext` | `lww` | Loses rich text structure |
| any | (none) | Field becomes non-replicated (data loss) |

### 8.3 Migration Requirements

**Requirement M1**: Schema evolution MUST declare strategy changes explicitly.

**Requirement M2**: Unsafe migrations MUST provide custom migration functions.

**Requirement M3**: Migration functions MUST be deterministic (same input → same output).

**Requirement M4**: Migration functions MUST preserve CRDT convergence properties.

### 8.4 Deterministic Migration

**Rule D1**: Migration functions MUST NOT use:
- Current timestamp (`now()`)
- Random values (`random()`)
- External state (environment variables, files)

**Rule D2**: Migration functions SHOULD use:
- Fixed sentinel values
- Content hashes
- Deterministic ordering (sort by field)

**Example (Deterministic):**
```rust
// ✓ Deterministic migration (lww → peritext)
fn migrate(lww_value: String) -> Peritext {
  let mut doc = Peritext::new();
  doc.insert_text(
    0,
    &lww_value,
    ActorId::MIGRATION_SENTINEL,  // Fixed sentinel
    Timestamp::EPOCH              // Fixed timestamp
  );
  doc
}
```

**Example (Non-Deterministic):**
```rust
// ✗ Non-deterministic (uses current time)
fn migrate(lww_value: String) -> Peritext {
  let mut doc = Peritext::new();
  doc.insert_text(0, &lww_value, ActorId::new(), Timestamp::now());
  doc
}
```

---

## 9. Serialization Format

### 9.1 JSON Schema Representation

CRDT-annotated schemas SHOULD be serializable to JSON for machine processing:

```json
{
  "name": "ChatMessage",
  "version": "1.0.0",
  "fields": [
    {
      "name": "id",
      "type": "String",
      "crdt": {
        "strategy": "immutable"
      }
    },
    {
      "name": "content",
      "type": "String",
      "crdt": {
        "strategy": "peritext",
        "options": {
          "formatting": "full"
        }
      }
    },
    {
      "name": "reactions",
      "type": "Set<String>",
      "crdt": {
        "strategy": "or_set"
      }
    }
  ]
}
```

### 9.2 Wire Format

This specification does NOT mandate a wire format. Implementations MAY use:
- Automerge binary format
- JSON patches (RFC 6902)
- Custom binary protocols
- Operational transform deltas

**Requirement W1**: Wire format MUST preserve CRDT semantics.

**Requirement W2**: Wire format SHOULD support incremental sync (delta compression).

---

## 10. Validation Requirements

### 10.1 Schema Validators

Implementations MUST provide schema validation with the following checks:

**V1: Syntax Validation**
- V1.1: CRDT annotation syntax is well-formed
- V1.2: Strategy identifiers are recognized
- V1.3: Options are valid for the strategy

**V2: Type Compatibility**
- V2.1: Strategy is compatible with field type (see §6)
- V2.2: Nested types have consistent annotations

**V3: Constraint Compatibility**
- V3.1: CRDT strategies preserve Category A constraints
- V3.2: Category B constraints flagged for eventual consistency
- V3.3: Category C constraints flagged for coordination requirement

**V4: Evolution Safety**
- V4.1: Strategy changes are safe or have migration functions
- V4.2: Migration functions are deterministic

### 10.2 Runtime Validation

Implementations SHOULD provide runtime validation:

**R1: Merge Correctness**
- R1.1: Merge operations are commutative
- R1.2: Merge operations are idempotent
- R1.3: Merged state satisfies Category A constraints

**R2: Convergence Testing**
- R2.1: Multiple replicas converge to same state
- R2.2: Convergence occurs within bounded time

---

## 11. Security Considerations

### 11.1 Byzantine Fault Tolerance

CRDT strategies assume **honest-but-curious** peers. For Byzantine (malicious) environments:

**S1**: Implementations SHOULD verify operation signatures.

**S2**: Implementations SHOULD validate operations against schema constraints before applying.

**S3**: Critical operations (e.g., financial) SHOULD require BFT consensus, not pure CRDTs.

### 11.2 Data Privacy

**S4**: Sensitive fields SHOULD be encrypted before CRDT replication.

**S5**: GDPR "right to erasure" can be implemented via key deletion (encrypt field, delete key).

### 11.3 Denial of Service

**S6**: Implementations SHOULD enforce:
- Maximum operation size
- Rate limiting
- Storage quotas
- Garbage collection of tombstones

---

## 12. Interoperability Guidelines

### 12.1 Cross-Library Compatibility

To enable interoperability between CRDT libraries:

**I1**: Libraries SHOULD support this annotation format in schema definitions.

**I2**: Libraries SHOULD provide JSON schema export (§9.1).

**I3**: Libraries MAY provide conversion utilities between formats.

### 12.2 Versioning

**I4**: Schema definitions SHOULD include version numbers.

**I5**: Incompatible schema changes SHOULD increment major version.

**I6**: Backward-compatible changes SHOULD increment minor version.

### 12.3 Extension Points

**I7**: Implementations MAY add custom strategies (e.g., `@crdt(custom:my_strategy)`).

**I8**: Custom strategies MUST use namespaced identifiers (e.g., `custom:*`, `vendor:*`).

**I9**: Standard strategies (§5) MUST NOT be overridden.

---

## 13. Conformance

### 13.1 Conformance Levels

**Level 1 (Basic Conformance):**
- Supports at least 4 strategies: `immutable`, `lww`, `or_set`, `pn_counter`
- Implements type compatibility validation (§6)
- Provides JSON schema export (§9.1)

**Level 2 (Full Conformance):**
- Supports all 7 strategies (§5)
- Implements constraint categorization (§7)
- Supports schema evolution (§8)
- Passes convergence test suite

**Level 3 (Advanced Conformance):**
- Byzantine fault tolerance (§11.1)
- Cross-library interoperability (§12)
- Custom strategy extensions (§12.3)

### 13.2 Test Suite

Conforming implementations SHOULD pass the DOL CRDT Test Suite (available at https://github.com/univrs/dol-crdt-test-suite):

- Syntax parsing tests (100+ cases)
- Type compatibility tests
- Convergence property tests (commutativity, idempotence)
- Constraint preservation tests
- Evolution migration tests

---

## 14. References

### 14.1 Normative References

- **[RFC2119]** Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997.

- **[RFC8259]** Bray, T., "The JavaScript Object Notation (JSON) Data Interchange Format", RFC 8259, December 2017.

- **[ISO14977]** ISO/IEC 14977:1996, "Information technology - Syntactic metalanguage - Extended BNF"

### 14.2 Informative References

- **[Shapiro2011]** Shapiro, M., et al., "Conflict-free Replicated Data Types", INRIA Research Report 7687, 2011.

- **[Kleppmann2017]** Kleppmann, M. and Beresford, A., "A Conflict-Free Replicated JSON Datatype", IEEE Transactions on Parallel and Distributed Systems, 2017.

- **[Litt2022]** Litt, G., et al., "Peritext: A CRDT for Collaborative Rich Text Editing", PaPoC 2022.

- **[Kleppmann2024]** Kleppmann, M., et al., "An Efficient Approach for Collaborative Text Editing", EuroSys 2025.

- **[Automerge]** Automerge CRDT Library, https://automerge.org

- **[Yjs]** Yjs CRDT Framework, https://github.com/yjs/yjs

- **[Loro]** Loro CRDT Library, https://loro.dev

---

## Appendix A: Complete Example

```dol
gen ChatMessage v1.0.0 {
  @crdt(immutable)
  id: Uuid

  @crdt(immutable)
  created_at: Timestamp

  @crdt(lww)
  author: Identity

  @crdt(peritext, formatting="full", max_length=100000)
  content: String

  @crdt(or_set)
  reactions: Set<Reaction>

  @crdt(lww)
  edited_at: Option<Timestamp>
}

constraint ChatMessage.immutability {
  message never changes id
  message never changes created_at
  message never changes author
}

constraint ChatMessage.reaction_validity {
  all reactions from authenticated_users
}

docs {
  A collaborative chat message demonstrating all major CRDT strategies:

  - Immutable identity (id, created_at, author)
  - Last-write-wins metadata (edited_at)
  - Collaborative rich text (content via Peritext)
  - Add-wins reactions (reactions via OR-Set)

  Constraint enforcement:
  - Immutability: Category A (enforced by CRDT)
  - Reaction validity: Category B (eventually consistent)
}
```

---

## Appendix B: Implementation Checklist

- [ ] Parse CRDT annotation syntax
- [ ] Validate strategy identifiers
- [ ] Check type-strategy compatibility
- [ ] Implement all 7 CRDT strategies
- [ ] Verify merge commutativity
- [ ] Verify merge idempotence
- [ ] Test convergence properties
- [ ] Implement constraint categorization
- [ ] Support schema evolution
- [ ] Validate migration determinism
- [ ] Export JSON schema format
- [ ] Pass conformance test suite

---

## Appendix C: Glossary

**Actor ID**: Unique identifier for a replica/peer in the distributed system.

**Causal Order**: Ordering based on happens-before relationships between operations.

**Convergence**: The property that all replicas eventually reach the same state.

**CRDT**: Conflict-free Replicated Data Type.

**Idempotent**: Operation that can be applied multiple times with the same effect as applying once.

**Lamport Timestamp**: Logical clock value ensuring causal ordering.

**Merge**: Combining two replica states into a single state.

**Replica**: Independent copy of a data structure.

**Strong Eventual Consistency (SEC)**: Guarantee that replicas receiving the same operations reach identical states without coordination.

**Tombstone**: Marker indicating a deleted element (preserves causal structure).

**Vector Clock**: Logical clock tracking causality across multiple actors.

---

**Document Status:** Proposed Standard
**Version:** 1.0.0
**Published:** 2026-02-05
**License:** Creative Commons Attribution 4.0 International (CC BY 4.0)
**Copyright:** Univrs Foundation, 2026

For questions, feedback, or implementation support, contact: standards@univrs.org
