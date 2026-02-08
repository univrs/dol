# RFC-001: DOL 2.0 CRDT Annotations

**Status:** Draft
**Version:** 1.0.0
**Date:** 2026-02-05
**Authors:** arch-dol-crdt, researcher-crdt-frontier
**Project:** MYCELIUM-SYNC - Univrs Local-First Architecture

---

## Abstract

This RFC extends DOL 2.0 with native Conflict-free Replicated Data Type (CRDT) annotations, enabling ontology-driven conflict resolution for local-first distributed applications. By embedding CRDT semantics directly in the ontology language, we create a formal, verifiable foundation for distributed state synchronization that preserves DOL's constraint system and evolution model.

The core innovation: **The ontology becomes the authoritative specification for conflict resolution**, compiled into WASM modules that guarantee convergence across distributed peers.

---

## 1. Motivation

### 1.1 The Local-First Software Paradigm

Local-first software prioritizes:
- **Offline-first operation**: Applications work without network connectivity
- **Peer-to-peer synchronization**: No central coordination required
- **Data ownership**: Users control their own data
- **Immediate responsiveness**: No network round-trips for local operations

Traditional centralized architectures cannot satisfy these requirements. CRDTs provide the mathematical foundation for conflict-free distributed state, but lack higher-level semantic specification.

### 1.2 Why Extend DOL?

DOL (Distributed Ontology Language) already provides:
- **Formal type system** (Gene, Trait, Constraint, System, Evolution)
- **Semantic validation** (dol-check ensures invariants)
- **Compilation to WASM** (ontology → executable code)
- **Mandatory documentation** (exegesis/docs blocks)

By extending DOL with CRDT annotations, we gain:
- **Type-safe CRDT specifications**: Compiler enforces CRDT-type compatibility
- **Constraint-aware merging**: DOL constraints guide conflict resolution
- **Version-tracked evolution**: CRDT strategy changes documented via Evolution
- **Cross-platform convergence**: WASM execution guarantees identical semantics

---

## 2. Grammar Extensions

### 2.1 CRDT Annotation Syntax

Extend the DOL grammar to support `@crdt(strategy, options)` annotations on Gene fields:

```ebnf
(* CRDT Annotation *)
crdt_annotation = '@crdt' , '(' , crdt_strategy , [ ',' , crdt_options ] , ')' ;

crdt_strategy = 'immutable'
              | 'lww'           (* Last-Write-Wins *)
              | 'or_set'        (* Observed-Remove Set *)
              | 'pn_counter'    (* Positive-Negative Counter *)
              | 'peritext'      (* Rich Text CRDT *)
              | 'rga'           (* Replicated Growable Array *)
              | 'mv_register' ; (* Multi-Value Register *)

crdt_options = option_pair , { ',' , option_pair } ;
option_pair = identifier , '=' , value ;
value = string_literal | number | 'true' | 'false' ;

(* Extended Gene Field Declaration *)
gene_field = [ crdt_annotation ] , identifier , ':' , type_spec ;

(* Gene Declaration with Fields *)
gene_declaration = kw_gene , qualified_identifier , delim_lbrace ,
                   { gene_field } ,
                   { gene_statement } ,
                   delim_rbrace ;
```

### 2.2 Annotation Placement

CRDT annotations appear **immediately before** field declarations:

```dol
gen message.chat {
  @crdt(immutable)
  id: Uuid

  @crdt(lww)
  author: Identity

  @crdt(peritext)
  content: RichText

  @crdt(lww)
  timestamp: Timestamp

  @crdt(or_set)
  reactions: Set<Reaction>
}

docs {
  A chat message with immutable identity, LWW metadata,
  collaborative rich text content, and add-wins reaction set.
}
```

### 2.3 System-Level Sync Policies

Systems can declare sync policies for P2P coordination:

```dol
system chat.app @1.0.0 {
  requires message.chat >= 1.0.0

  sync_policy {
    strategy: eager          # eager | lazy | on_demand
    conflict_resolution: crdt # crdt | manual | last_writer_wins
    max_peers: 100
    offline_storage: persistent
  }
}
```

---

## 3. CRDT Strategy Semantics

### 3.1 Strategy Overview

| Strategy | Use Case | Merge Rule | Deletion |
|----------|----------|------------|----------|
| **immutable** | IDs, creation timestamps | First-write-wins | No deletion |
| **lww** | Single-valued fields (name, status) | Last timestamp wins | Tombstone |
| **or_set** | Collections (tags, reactions) | Add-wins | Observed-remove |
| **pn_counter** | Numeric counters (likes, votes) | Sum all ops | Decrement |
| **peritext** | Rich text documents | CRDT list + formatting | Character delete |
| **rga** | Ordered sequences (lists, arrays) | Causal ordering | Tombstone |
| **mv_register** | Concurrent values | Keep all versions | Mark deleted |

### 3.2 Immutable Strategy

**Semantics**: Value set exactly once, never modified. Concurrent sets choose first by actor ID (deterministic tie-breaking).

**Type Compatibility**: `Uuid`, `Timestamp`, `Identity`, any type with `#[derive(Immutable)]`.

**Merge Logic**:
```rust
fn merge(local: T, remote: T, meta_local: Meta, meta_remote: Meta) -> T {
    if meta_local.timestamp < meta_remote.timestamp {
        local  // Keep first value
    } else if meta_local.timestamp == meta_remote.timestamp {
        if meta_local.actor_id < meta_remote.actor_id {
            local  // Deterministic tie-break
        } else {
            remote
        }
    } else {
        local
    }
}
```

**DOL Example**:
```dol
gen identity.cryptographic {
  @crdt(immutable)
  id: Uuid  # Set once at creation, never changes

  @crdt(immutable)
  created_at: Timestamp

  @crdt(lww)
  public_key: Ed25519PublicKey  # Can be rotated
}
```

### 3.3 Last-Write-Wins (LWW) Strategy

**Semantics**: Most recent write (by Lamport timestamp + actor ID) wins. Simple and efficient, but loses concurrent updates.

**Type Compatibility**: Any single-valued type (`String`, `Int`, `Bool`, `Enum`, structs).

**Merge Logic**:
```rust
fn merge(local: T, remote: T, ts_local: Timestamp, ts_remote: Timestamp) -> T {
    if ts_remote > ts_local {
        remote
    } else if ts_remote == ts_local {
        if remote.actor_id > local.actor_id {  // Deterministic tie-break
            remote
        } else {
            local
        }
    } else {
        local
    }
}
```

**Options**:
- `tie_break`: `"actor_id"` (default) | `"content_hash"` | `"custom"`

**DOL Example**:
```dol
gen user.profile {
  @crdt(lww)
  display_name: String

  @crdt(lww)
  avatar_url: String

  @crdt(lww, tie_break="content_hash")
  bio: String  # Content-hash tie-break for identical concurrent edits
}
```

### 3.4 Observed-Remove Set (OR-Set) Strategy

**Semantics**: Add-wins semantics. Each element tagged with unique ID. Remove only deletes observed tags. Concurrent add + remove → element present.

**Type Compatibility**: `Set<T>` where `T: Hash + Eq`.

**Merge Logic**:
```rust
struct ORSet<T> {
    elements: HashMap<T, HashSet<Uuid>>,  // element -> set of unique tags
    tombstones: HashSet<Uuid>,            // removed tags
}

fn add(&mut self, element: T) -> Uuid {
    let tag = Uuid::new_v4();
    self.elements.entry(element).or_default().insert(tag);
    tag
}

fn remove(&mut self, element: &T) {
    if let Some(tags) = self.elements.get(element) {
        self.tombstones.extend(tags);
    }
}

fn contains(&self, element: &T) -> bool {
    self.elements.get(element)
        .map(|tags| tags.iter().any(|t| !self.tombstones.contains(t)))
        .unwrap_or(false)
}

fn merge(&mut self, other: &ORSet<T>) {
    for (elem, tags) in &other.elements {
        self.elements.entry(elem.clone()).or_default().extend(tags);
    }
    self.tombstones.extend(&other.tombstones);
}
```

**DOL Example**:
```dol
gen document.collaborative {
  @crdt(or_set)
  tags: Set<String>

  @crdt(or_set)
  collaborators: Set<Identity>

  @crdt(or_set)
  attachments: Set<AttachmentRef>
}
```

### 3.5 Positive-Negative Counter (PN-Counter) Strategy

**Semantics**: Each actor maintains separate increment/decrement counters. Value = sum(increments) - sum(decrements). Commutative and convergent.

**Type Compatibility**: `Int`, `UInt`, `Float` (with precision considerations).

**Merge Logic**:
```rust
struct PNCounter {
    increments: HashMap<ActorId, u64>,
    decrements: HashMap<ActorId, u64>,
}

fn increment(&mut self, actor: ActorId, amount: u64) {
    *self.increments.entry(actor).or_default() += amount;
}

fn decrement(&mut self, actor: ActorId, amount: u64) {
    *self.decrements.entry(actor).or_default() += amount;
}

fn value(&self) -> i64 {
    let inc: u64 = self.increments.values().sum();
    let dec: u64 = self.decrements.values().sum();
    inc as i64 - dec as i64
}

fn merge(&mut self, other: &PNCounter) {
    for (actor, count) in &other.increments {
        *self.increments.entry(*actor).or_default() =
            (*self.increments.get(actor).unwrap_or(&0)).max(*count);
    }
    for (actor, count) in &other.decrements {
        *self.decrements.entry(*actor).or_default() =
            (*self.decrements.get(actor).unwrap_or(&0)).max(*count);
    }
}
```

**Options**:
- `min_value`: Enforce lower bound (e.g., `min_value=0` for natural numbers)
- `max_value`: Enforce upper bound
- `overflow_strategy`: `"saturate"` | `"wrap"` | `"error"`

**DOL Example**:
```dol
gen post.social {
  @crdt(pn_counter, min_value=0)
  likes: Int

  @crdt(pn_counter)
  karma_score: Int

  @crdt(pn_counter, min_value=0, overflow_strategy="saturate")
  view_count: Int
}
```

### 3.6 Peritext Strategy (Rich Text)

**Semantics**: Collaborative rich text editing with formatting. Based on Peritext CRDT (Litt et al.). Combines RGA for character sequences with mark-based formatting.

**Type Compatibility**: `RichText`, `String` (with auto-conversion).

**Architecture**:
- **Character sequence**: RGA (Replicated Growable Array)
- **Formatting marks**: Ranges with expand-left/right semantics
- **Cursor positions**: Causal ordering preserves intent

**Options**:
- `formatting`: `"full"` | `"markdown"` | `"plain"`
- `max_length`: Character limit for memory bounds

**DOL Example**:
```dol
gen document.editor {
  @crdt(peritext, formatting="full", max_length=1000000)
  content: RichText

  @crdt(lww)
  title: String
}
```

**Implementation Note**: Uses Automerge's text CRDT, which implements Peritext semantics.

### 3.7 Replicated Growable Array (RGA) Strategy

**Semantics**: Ordered sequence with causal insertion order. Each element has unique ID and "left origin" reference. Concurrent inserts at same position ordered by actor ID.

**Type Compatibility**: `List<T>`, `Vec<T>`, `Array<T>`.

**Merge Logic**:
```rust
struct RGA<T> {
    sequence: Vec<Vertex<T>>,
    tombstones: HashSet<VertexId>,
}

struct Vertex<T> {
    id: VertexId,
    element: T,
    left_origin: Option<VertexId>,
    timestamp: Timestamp,
}

fn insert(&mut self, position: usize, element: T) -> VertexId {
    let left_origin = self.visible_vertex_at(position.saturating_sub(1));
    let vertex = Vertex {
        id: VertexId::new(self.actor_id, self.counter.increment()),
        element,
        left_origin: left_origin.map(|v| v.id),
        timestamp: Timestamp::now(),
    };
    let insert_idx = self.find_insert_position(&vertex);
    self.sequence.insert(insert_idx, vertex);
    vertex.id
}

fn merge(&mut self, other: &RGA<T>) {
    // Merge sequences preserving causal order
    let merged = topological_sort(
        &self.sequence,
        &other.sequence,
        |v| v.left_origin
    );
    self.sequence = merged;
    self.tombstones.extend(&other.tombstones);
}
```

**DOL Example**:
```dol
gen task.board {
  @crdt(rga)
  task_order: List<TaskId>

  @crdt(rga)
  column_order: List<ColumnId>
}
```

### 3.8 Multi-Value Register (MV-Register) Strategy

**Semantics**: Keeps all concurrent values until explicitly resolved. Application chooses resolution strategy (union, intersection, custom logic). Useful for detecting conflicts.

**Type Compatibility**: Any type `T`.

**Merge Logic**:
```rust
struct MVRegister<T> {
    values: HashMap<VectorClock, T>,
}

fn set(&mut self, value: T, clock: VectorClock) {
    // Remove causally dominated values
    self.values.retain(|vc, _| !clock.dominates(vc));
    self.values.insert(clock, value);
}

fn get(&self) -> Vec<&T> {
    self.values.values().collect()
}

fn merge(&mut self, other: &MVRegister<T>) {
    for (clock, value) in &other.values {
        // Keep value if not dominated by any local value
        if !self.values.keys().any(|local_clock| local_clock.dominates(clock)) {
            self.values.insert(clock.clone(), value.clone());
        }
    }
    // Remove dominated values
    let dominated: Vec<_> = self.values.iter()
        .filter(|(vc1, _)| {
            self.values.iter().any(|(vc2, _)| vc2 != vc1 && vc2.dominates(vc1))
        })
        .map(|(vc, _)| vc.clone())
        .collect();
    for vc in dominated {
        self.values.remove(&vc);
    }
}
```

**DOL Example**:
```dol
gen config.app {
  @crdt(mv_register)
  theme: Theme  # Keep all concurrent theme changes, let user choose

  @crdt(mv_register)
  language: Language
}
```

---

## 4. Type Compatibility Matrix

| DOL Type | immutable | lww | or_set | pn_counter | peritext | rga | mv_register |
|----------|-----------|-----|--------|------------|----------|-----|-------------|
| `Uuid` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `String` | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ |
| `Int`, `UInt` | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ |
| `Float` | ✅ | ✅ | ❌ | ⚠️ | ❌ | ❌ | ✅ |
| `Bool` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Enum` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Timestamp` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `Set<T>` | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| `List<T>`, `Vec<T>` | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| `Map<K, V>` | ❌ | ❌ | ⚠️ | ❌ | ❌ | ❌ | ✅ |
| `RichText` | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Custom Struct | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |

**Legend**:
- ✅ **Recommended**: Natural fit, efficient implementation
- ⚠️ **Caution**: Works but has caveats (e.g., Float with PN-Counter loses precision)
- ❌ **Invalid**: Type mismatch, will fail dol-check validation

---

## 5. Constraint-CRDT Interaction Rules

### 5.1 Constraint Categories

DOL constraints interact with CRDT merges in three ways:

#### **Category A: CRDT-Safe Constraints**
These constraints hold **automatically** under CRDT merge semantics. No additional validation needed.

- **Immutability constraints**: `id never changes` (enforced by `@crdt(immutable)`)
- **Monotonic growth**: `counter always increases` (enforced by PN-Counter)
- **Set membership**: `tags has element` (preserved by OR-Set add-wins)

**Example**:
```dol
gen message.immutable {
  @crdt(immutable)
  id: Uuid

  @crdt(immutable)
  author: Identity
}

constraint message.identity_immutable {
  message never changes id
  message never changes author
}

docs {
  This constraint is CRDT-safe: immutable strategy prevents modification.
  No runtime validation needed.
}
```

#### **Category B: Eventually-Consistent Constraints**
These constraints may be **temporarily violated** during concurrent operations but **eventually converge** to valid state after merge.

- **Uniqueness across peers**: `all users have unique email` (requires conflict detection)
- **Resource limits**: `storage_used <= storage_quota` (bounded by escrow)
- **Referential integrity**: `post.author exists in users` (eventual propagation)

**Example**:
```dol
gen account.balance {
  @crdt(pn_counter, min_value=0)
  confirmed_balance: Int  # BFT-confirmed balance

  @crdt(lww)
  local_escrow: Int  # Pre-allocated for local spends

  @crdt(pn_counter)
  pending_credits: Int  # Eventually consistent
}

constraint account.solvency {
  confirmed_balance always >= 0
  local_escrow always <= confirmed_balance
  local_escrow always >= 0
}

docs {
  Eventually-consistent constraint. Local operations spend from escrow
  (always positive). Periodic BFT reconciliation updates confirmed_balance.
  Constraint may be temporarily violated if escrow exceeds confirmed balance
  due to network partition, but reconciliation resolves it.
}
```

#### **Category C: Strong-Consistency Constraints**
These constraints require **coordination** (BFT consensus, locks, or escrow) to maintain. Cannot be purely CRDT-based.

- **Uniqueness under high cardinality**: `all transactions have unique id` (requires coordination)
- **Cross-account transfers**: `A.balance -= X AND B.balance += X` (atomic, requires escrow)
- **Access control**: `user can_edit document IFF user in collaborators` (requires permission sync)

**Example**:
```dol
gen transfer.mutual_credit {
  @crdt(immutable)
  id: Uuid

  @crdt(immutable)
  from: AccountId

  @crdt(immutable)
  to: AccountId

  @crdt(immutable)
  amount: Int

  @crdt(lww)
  status: TransferStatus  # pending | confirmed | rejected
}

constraint transfer.double_spend_prevention {
  from.confirmed_balance >= amount  # Checked at creation
  transfer never duplicates          # Enforced by escrow allocation
}

docs {
  Strong-consistency constraint. Double-spend prevented via escrow:
  1. Transfer creation allocates from local_escrow (immediate, local)
  2. If local_escrow sufficient → status=pending
  3. BFT committee confirms → status=confirmed
  4. If escrow insufficient → status=rejected

  This uses eventual consistency (CRDT) for most operations, but
  escrow + BFT for the critical double-spend constraint.
}
```

### 5.2 Constraint Validation Timeline

```
┌─────────────────────────────────────────────────────────────┐
│ Constraint Validation Points in Local-First Architecture    │
└─────────────────────────────────────────────────────────────┘

[Category A: CRDT-Safe]
├─ Validation: COMPILE TIME
├─ Enforced by: Type system + CRDT strategy
└─ Examples: Immutability, monotonic counters

[Category B: Eventually-Consistent]
├─ Validation: MERGE TIME (soft validation)
├─ Enforced by: CRDT semantics + eventual propagation
├─ Action on violation: Flag for reconciliation (not block)
└─ Examples: Uniqueness, resource bounds, referential integrity

[Category C: Strong-Consistency]
├─ Validation: OPERATION TIME (before commit)
├─ Enforced by: Escrow allocation or BFT consensus
├─ Action on violation: Reject operation immediately
└─ Examples: Mutual credit transfers, access control
```

---

## 6. Evolution Compatibility Rules

### 6.1 CRDT Strategy Changes Across Versions

DOL's Evolution construct tracks changes between versions. CRDT strategy changes must be **migration-safe**:

#### **Safe Migrations** (no data loss):
- `immutable` → `lww`: Existing value becomes LWW-timestamped
- `lww` → `mv_register`: Single value becomes multi-value set with one element
- `or_set` → `rga`: Unordered set becomes ordered list (deterministic ordering)
- Adding `@crdt` annotation to unannotated field: Default to `lww`

#### **Unsafe Migrations** (data loss or semantic change):
- `lww` → `immutable`: Cannot "un-mutate" a field (blocks evolution)
- `or_set` → `pn_counter`: Semantic mismatch (set ≠ counter)
- `peritext` → `lww`: Loses rich text structure (requires manual migration)
- Removing `@crdt` annotation: Field becomes non-replicated (data loss)

### 6.2 Evolution Declaration Example

```dol
evolves message.chat @1.1.0 > 1.0.0 {
  adds @crdt(or_set)
  adds reactions: Set<Reaction>

  changes content from @crdt(lww) to @crdt(peritext)

  because "Version 1.1.0 adds collaborative rich text editing and emoji reactions.
           Migrating content from LWW string to Peritext requires importing as
           plain text with no formatting. Existing strings preserved as-is."
}

docs {
  Migration strategy:
  - reactions: New field, default to empty OR-Set
  - content: LWW string → Peritext plain text (no formatting)

  Backward compatibility: v1.0.0 peers can read v1.1.0 documents
  by treating Peritext as plain string (formatting ignored).
}
```

### 6.3 Deterministic Migration Functions

Migrations must be **deterministic** (same input → same output) to preserve CRDT convergence:

```rust
// GOOD: Deterministic migration
fn migrate_lww_to_peritext(lww_string: &str, actor: ActorId) -> Peritext {
    let mut doc = Peritext::new();
    // Use fixed actor ID and timestamp for migration operation
    let migration_actor = ActorId::MIGRATION_SENTINEL;
    let migration_ts = Timestamp::EPOCH;
    doc.insert_text(0, lww_string, migration_actor, migration_ts);
    doc
}

// BAD: Non-deterministic (uses current time)
fn migrate_bad(lww_string: &str, actor: ActorId) -> Peritext {
    let mut doc = Peritext::new();
    doc.insert_text(0, lww_string, actor, Timestamp::now()); // ❌ Non-deterministic!
    doc
}
```

---

## 7. Compilation to Rust/WASM

### 7.1 Code Generation Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│ DOL → Rust → WASM Compilation Pipeline                      │
└─────────────────────────────────────────────────────────────┘

  DOL Source                  dol-parse
  (with @crdt)           ──────────────►  Typed AST
       │                                       │
       │                                       │
       │                    dol-check          │
       │              ◄─────────────────────   │
       │              │ Validate CRDT         │
       │              │ type compat,          │
       │              │ constraints           │
       │              └───────────────────────┘
       │                                       │
       │                  dol-codegen-rust     │
       └──────────────────────────────────────►
                                               │
                                               ▼
                      Rust Code with Automerge Integration
                      ├─ Structs with #[derive(Reconcile, Hydrate)]
                      ├─ CRDT operation methods
                      ├─ Constraint validation
                      └─ Merge functions
                                               │
                                               │ cargo build --target wasm32-unknown-unknown
                                               ▼
                      WASM Module
                      ├─ < 200KB compressed per Gene
                      ├─ wasm-bindgen JS bindings
                      └─ Component Model WIT interface
```

### 7.2 Generated Rust Code Example

**Input DOL**:
```dol
gen message.chat {
  @crdt(immutable)
  id: Uuid

  @crdt(lww)
  author: Identity

  @crdt(peritext)
  content: RichText

  @crdt(or_set)
  reactions: Set<Reaction>
}
```

**Generated Rust**:
```rust
use automerge::{Automerge, AutoCommit, ReadDoc, transaction::Transactable};
use autosurgeon::{Reconcile, Hydrate};

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct MessageChat {
    #[autosurgeon(immutable)]
    pub id: Uuid,

    #[autosurgeon(lww)]
    pub author: Identity,

    #[autosurgeon(text)]  // Peritext implemented via Automerge text
    pub content: String,

    #[autosurgeon(map)]   // OR-Set via Automerge map
    pub reactions: HashMap<String, Reaction>,
}

impl MessageChat {
    pub fn new(id: Uuid, author: Identity) -> Self {
        Self {
            id,
            author,
            content: String::new(),
            reactions: HashMap::new(),
        }
    }

    // Generated CRDT operation methods
    pub fn set_author(&mut self, author: Identity, doc: &mut AutoCommit) {
        self.author = author;
        autosurgeon::reconcile(doc, &self).expect("reconcile failed");
    }

    pub fn edit_content(&mut self, position: usize, text: &str, doc: &mut AutoCommit) {
        // Peritext operations
        let text_obj = doc.get(automerge::ROOT, "content")
            .expect("content field missing")
            .expect_text();
        doc.splice_text(&text_obj, position, 0, text)
            .expect("splice failed");
    }

    pub fn add_reaction(&mut self, reaction: Reaction, doc: &mut AutoCommit) {
        let reaction_id = Uuid::new_v4().to_string();
        self.reactions.insert(reaction_id.clone(), reaction);
        autosurgeon::reconcile(doc, &self).expect("reconcile failed");
    }

    pub fn remove_reaction(&mut self, reaction_id: &str, doc: &mut AutoCommit) {
        self.reactions.remove(reaction_id);
        autosurgeon::reconcile(doc, &self).expect("reconcile failed");
    }

    // Generated merge function
    pub fn merge(local_doc: &Automerge, remote_doc: &Automerge) -> Automerge {
        local_doc.merge(remote_doc).expect("merge failed")
    }
}

// WASM bindings
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;

    #[wasm_bindgen]
    pub struct MessageChatWasm {
        doc: AutoCommit,
        state: MessageChat,
    }

    #[wasm_bindgen]
    impl MessageChatWasm {
        #[wasm_bindgen(constructor)]
        pub fn new(id: String, author: String) -> Self {
            let mut doc = AutoCommit::new();
            let state = MessageChat::new(
                Uuid::parse_str(&id).unwrap(),
                Identity::from(author)
            );
            autosurgeon::reconcile(&mut doc, &state).unwrap();
            Self { doc, state }
        }

        #[wasm_bindgen]
        pub fn edit_content(&mut self, position: usize, text: String) {
            self.state.edit_content(position, &text, &mut self.doc);
        }

        #[wasm_bindgen]
        pub fn add_reaction(&mut self, reaction: JsValue) {
            let reaction: Reaction = serde_wasm_bindgen::from_value(reaction).unwrap();
            self.state.add_reaction(reaction, &mut self.doc);
        }

        #[wasm_bindgen]
        pub fn get_content(&self) -> String {
            self.state.content.clone()
        }

        #[wasm_bindgen]
        pub fn merge(&mut self, remote_bytes: Vec<u8>) -> Result<(), JsValue> {
            let remote_doc = Automerge::load(&remote_bytes)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            self.doc.merge(&remote_doc)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            self.state = autosurgeon::hydrate(&self.doc)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            Ok(())
        }

        #[wasm_bindgen]
        pub fn save(&self) -> Vec<u8> {
            self.doc.save()
        }
    }
}
```

### 7.3 WASM Component Model Integration

Generate WIT (WebAssembly Interface Types) for cross-language composition:

**Generated WIT** (`message-chat.wit`):
```wit
package univrs:message@1.0.0;

interface chat {
    record uuid {
        bytes: list<u8>,
    }

    record identity {
        id: string,
    }

    record reaction {
        emoji: string,
        user: identity,
    }

    resource message-chat {
        constructor(id: uuid, author: identity);

        edit-content: func(position: u32, text: string);
        add-reaction: func(reaction: reaction);
        remove-reaction: func(reaction-id: string);

        get-content: func() -> string;
        get-reactions: func() -> list<reaction>;

        merge: func(remote-bytes: list<u8>) -> result<_, string>;
        save: func() -> list<u8>;
    }
}

world message-chat-world {
    export chat;
}
```

---

## 8. dol-check Validation Rules

### 8.1 Validation Phases

```rust
pub enum CrdtValidationError {
    // Phase 1: Syntax validation
    InvalidStrategy { field: String, strategy: String },
    MissingAnnotation { field: String, context: String },
    InvalidOption { option: String, valid_options: Vec<String> },

    // Phase 2: Type compatibility validation
    TypeStrategyMismatch { field: String, dol_type: String, strategy: String },
    UnsupportedCollectionType { field: String },

    // Phase 3: Constraint compatibility validation
    ConstraintCrdtConflict { constraint: String, field: String, reason: String },
    StrongConsistencyRequired { constraint: String, suggested_fix: String },

    // Phase 4: Evolution validation
    UnsafeMigration { from_strategy: String, to_strategy: String, reason: String },
    MissingMigrationLogic { evolution: String },
    NonDeterministicMigration { evolution: String, issue: String },
}
```

### 8.2 Validation Rules

#### **Rule V1: Strategy-Type Compatibility**
```rust
fn validate_strategy_type_compat(field: &Field) -> Result<(), CrdtValidationError> {
    match (&field.crdt_strategy, &field.dol_type) {
        (Some(CrdtStrategy::OrSet), DolType::Set(_)) => Ok(()),
        (Some(CrdtStrategy::OrSet), other) => Err(TypeStrategyMismatch {
            field: field.name.clone(),
            dol_type: format!("{:?}", other),
            strategy: "or_set".into(),
        }),
        (Some(CrdtStrategy::PnCounter), DolType::Int | DolType::UInt) => Ok(()),
        (Some(CrdtStrategy::Peritext), DolType::RichText | DolType::String) => Ok(()),
        // ... full compatibility matrix checks
    }
}
```

#### **Rule V2: Immutable Field Constraints**
```rust
fn validate_immutable_constraints(gene: &Gene) -> Result<(), CrdtValidationError> {
    for field in &gene.fields {
        if field.crdt_strategy == Some(CrdtStrategy::Immutable) {
            // Check for conflicting constraints
            if gene.constraints.iter().any(|c| c.allows_modification(&field.name)) {
                return Err(ConstraintCrdtConflict {
                    constraint: c.name.clone(),
                    field: field.name.clone(),
                    reason: "Constraint allows modification of immutable field".into(),
                });
            }
        }
    }
    Ok(())
}
```

#### **Rule V3: Strong-Consistency Detection**
```rust
fn validate_constraint_consistency_level(constraint: &Constraint) -> ValidationWarning {
    if constraint.is_cross_gene_atomic() {
        ValidationWarning::StrongConsistencyRequired {
            constraint: constraint.name.clone(),
            suggested_fix: "Consider using escrow allocation for local operations \
                          with periodic BFT reconciliation.".into(),
        }
    }
}
```

#### **Rule V4: Evolution Migration Safety**
```rust
fn validate_evolution_migration(evolution: &Evolution) -> Result<(), CrdtValidationError> {
    for change in &evolution.changes {
        match (&change.from_strategy, &change.to_strategy) {
            (CrdtStrategy::Lww, CrdtStrategy::Immutable) => {
                return Err(UnsafeMigration {
                    from_strategy: "lww".into(),
                    to_strategy: "immutable".into(),
                    reason: "Cannot make a mutable field immutable without data loss".into(),
                });
            }
            (CrdtStrategy::Lww, CrdtStrategy::Peritext) => {
                // Safe but requires migration logic
                if !evolution.has_migration_fn(&change.field) {
                    return Err(MissingMigrationLogic {
                        evolution: evolution.name.clone(),
                    });
                }
            }
            // ... other migration rules
        }
    }
    Ok(())
}
```

### 8.3 Validation Output

```bash
$ dol-check examples/chat-message.dol

✅ Syntax valid: @crdt annotations parsed successfully
✅ Type compatibility: All CRDT strategies match field types
⚠️  Constraint level: 'message.reactions.uniqueness' requires eventual consistency
    Suggestion: Add @crdt(or_set) to 'reactions' field for add-wins semantics
✅ Evolution safety: All migrations are deterministic
📊 CRDT coverage: 4/5 fields annotated (80%)
    Missing: 'metadata' field (defaulting to @crdt(lww))

✅ All checks passed (1 warning)
```

---

## 9. Complete Examples

### 9.1 Example: Collaborative Chat Message

```dol
gen message.chat {
  @crdt(immutable)
  id: Uuid

  @crdt(immutable)
  created_at: Timestamp

  @crdt(lww)
  author: Identity

  @crdt(peritext, formatting="full", max_length=100000)
  content: RichText

  @crdt(or_set)
  reactions: Set<Reaction>

  @crdt(lww)
  edited_at: Option<Timestamp>
}

constraint message.immutability {
  message never changes id
  message never changes created_at
  message never changes author
}

constraint message.reaction_validity {
  reactions has valid_emoji
  all reactions from authenticated_users
}

docs {
  A collaborative chat message with:
  - Immutable identity (id, created_at, author)
  - Real-time collaborative rich text editing (Peritext CRDT)
  - Add-wins emoji reactions (OR-Set CRDT)
  - Last-write-wins edit timestamp (LWW CRDT)

  Constraint enforcement:
  - Immutability: Enforced by CRDT strategy (Category A)
  - Reaction validity: Eventually consistent (Category B)
}
```

**Generated TypeScript Usage**:
```typescript
import { MessageChat } from './gen/message_chat';

// Create message offline
const msg = new MessageChat(
  crypto.randomUUID(),
  currentUser.identity
);

// Edit collaboratively (Peritext CRDT)
msg.editContent(0, "Hello, ");
msg.editContent(7, "world!");

// Add reactions (OR-Set CRDT)
msg.addReaction({ emoji: "👍", user: currentUser.identity });

// Sync with peer when online
socket.on('message-update', (remoteBytes) => {
  msg.merge(remoteBytes);  // Automatic CRDT merge
  renderMessage(msg);      // UI updates reactively
});
```

### 9.2 Example: Mutual Credit Account with Escrow

```dol
gen account.mutual_credit {
  @crdt(immutable)
  id: Uuid

  @crdt(immutable)
  owner: Identity

  @crdt(pn_counter, min_value=0)
  confirmed_balance: Int  # BFT-confirmed balance (strong consistency)

  @crdt(lww, min_value=0)
  local_escrow: Int  # Pre-allocated for offline spending (eventually consistent)

  @crdt(pn_counter)
  pending_credits: Int  # Unconfirmed incoming credits (eventually consistent)

  @crdt(or_set)
  transaction_history: Set<TransactionRef>

  @crdt(lww)
  reputation_tier: ReputationTier  # Determines escrow allocation limits
}

constraint account.solvency {
  confirmed_balance always >= 0
  local_escrow always >= 0
  local_escrow always <= confirmed_balance
}

constraint account.double_spend_prevention {
  account never spends_more_than local_escrow
  all transactions are atomic
}

docs {
  Mutual credit account operating under eventual consistency with escrow.

  Architecture:
  1. Strong consistency (BFT): confirmed_balance
     - Maintained by BFT committee (3f+1 nodes)
     - Updated during periodic reconciliation

  2. Local operations (immediate, no coordination): local_escrow
     - Pre-allocated from confirmed_balance
     - Enables offline spending up to escrow limit
     - Spent amount decrements escrow immediately

  3. Eventually consistent: pending_credits
     - Incoming credits from other peers
     - Merged via PN-Counter CRDT
     - Converted to confirmed_balance during reconciliation

  Double-spend prevention:
  - Local spends: Escrow allocation prevents overdraft
  - Cross-account transfers: Atomic via CRDT (debit from sender's escrow,
    credit to receiver's pending_credits)
  - Reconciliation: BFT confirms escrow exhaustion, allocates new escrow
    from pending_credits

  Constraint enforcement:
  - Solvency: Category B (eventually consistent)
    - May be temporarily violated during network partition
    - Reconciliation resolves violations
  - Double-spend: Category C (strong consistency via escrow)
    - Prevented by escrow allocation (local check)
    - BFT verifies escrow validity during reconciliation
}
```

**Generated Rust Usage**:
```rust
// Create account offline
let account = AccountMutualCredit::new(user_id, initial_balance);

// Allocate escrow for offline spending
let escrow_amount = account.confirmed_balance / 2;  // 50% escrow allocation
account.set_local_escrow(escrow_amount);

// Spend offline (immediate, no network)
if account.local_escrow >= spend_amount {
    account.spend_from_escrow(spend_amount);  // ✅ Succeeds
} else {
    return Err("Insufficient escrow");  // ❌ Rejected locally
}

// Receive pending credit (CRDT merge, no coordination)
account.receive_pending_credit(incoming_amount);

// Periodic BFT reconciliation (requires network + consensus)
if online && reconciliation_due() {
    let bft_result = bft_committee.reconcile_accounts(&[account.id]).await;
    if bft_result.confirmed {
        account.confirmed_balance += account.pending_credits;
        account.pending_credits = 0;
        account.local_escrow = min(
            account.confirmed_balance / 2,
            escrow_limit_for_tier(account.reputation_tier)
        );
    }
}
```

### 9.3 Example: Collaborative Task Board

```dol
gen task.item {
  @crdt(immutable)
  id: Uuid

  @crdt(lww)
  title: String

  @crdt(peritext, formatting="markdown")
  description: RichText

  @crdt(lww)
  status: TaskStatus  # todo | in_progress | done

  @crdt(or_set)
  assignees: Set<Identity>

  @crdt(pn_counter, min_value=0)
  estimate_hours: Int

  @crdt(lww)
  due_date: Option<Timestamp>
}

gen board.kanban {
  @crdt(immutable)
  id: Uuid

  @crdt(lww)
  name: String

  @crdt(rga)
  column_order: List<ColumnId>

  @crdt(or_set)
  tasks: Set<TaskItem>

  @crdt(or_set)
  collaborators: Set<Identity>
}

constraint board.task_ownership {
  all tasks belong_to exactly_one column
  all assignees are collaborators
}

constraint board.column_consistency {
  column_order has no_duplicates
  all columns in column_order exist
}

docs {
  Collaborative Kanban board with:
  - Immutable IDs (consistent across peers)
  - LWW task metadata (title, status, due date)
  - Peritext descriptions (collaborative Markdown editing)
  - OR-Set assignees and tasks (add-wins semantics)
  - PN-Counter estimates (additive time tracking)
  - RGA column ordering (causal ordering preserved)

  Concurrent operations:
  1. Two users add different tasks → Both appear (OR-Set)
  2. Two users assign to task → Both assignees added (OR-Set)
  3. Two users reorder columns → Causal order preserved (RGA)
  4. Two users edit description → Peritext merges text (CRDT)
  5. Two users update status → Last write wins (LWW)
}
```

---

## 10. Implementation Checklist

### Phase 1: Parser Extensions (t1.1)
- [ ] Extend lexer to recognize `@crdt`, `(`, `)`, strategy keywords
- [ ] Parse CRDT annotation syntax: `@crdt(strategy, options)`
- [ ] Validate strategy keywords: `immutable`, `lww`, `or_set`, etc.
- [ ] Parse optional key=value options
- [ ] Attach CRDT annotations to AST field nodes
- [ ] Test: Parse all 7 strategies with various options

### Phase 2: Type Validation (t1.2)
- [ ] Implement type-strategy compatibility matrix
- [ ] Validate field type matches CRDT strategy
- [ ] Error on invalid combinations (e.g., `or_set` on `Int`)
- [ ] Warn on suboptimal combinations (e.g., `Float` with `pn_counter`)
- [ ] Test: All valid/invalid type-strategy pairs

### Phase 3: Constraint Analysis (t1.2)
- [ ] Categorize constraints: CRDT-safe, eventually-consistent, strong-consistency
- [ ] Detect constraint-CRDT conflicts (e.g., immutable field with mutation constraint)
- [ ] Warn on strong-consistency requirements
- [ ] Suggest escrow patterns for Category C constraints
- [ ] Test: All constraint categories validated correctly

### Phase 4: Evolution Validation (t1.2)
- [ ] Detect CRDT strategy changes in Evolution declarations
- [ ] Validate migration safety (safe vs unsafe migrations)
- [ ] Require migration functions for unsafe changes
- [ ] Validate migration determinism (no Timestamp::now(), no random)
- [ ] Test: All migration scenarios (safe, unsafe, missing logic)

### Phase 5: Code Generation (t1.3)
- [ ] Generate Rust structs with `#[derive(Reconcile, Hydrate)]`
- [ ] Generate CRDT operation methods per strategy
- [ ] Generate merge functions using Automerge API
- [ ] Generate constraint validation hooks
- [ ] Generate WASM bindings via wasm-bindgen
- [ ] Test: DOL → Rust → WASM → JS round-trip

### Phase 6: WIT Interface Generation (t1.4)
- [ ] Generate WIT resource types from DOL Genes
- [ ] Map CRDT operations to WIT methods
- [ ] Generate Component Model worlds
- [ ] Test: WASM Component composition with wasmtime

### Phase 7: Property-Based Testing (t1.5)
- [ ] Implement convergence tests (N replicas → same state)
- [ ] Test commutativity (merge(A,B) == merge(B,A))
- [ ] Test constraint preservation across merges
- [ ] Simulate network partitions and healing
- [ ] Test: 1M+ random operation sequences converge

---

## 11. Future Extensions

### 11.1 Advanced CRDT Strategies (Phase 5)

**eg-walker** (EuroSys 2025 Best Artifact):
```dol
gen document.advanced {
  @crdt(eg_walker, algorithm="list")  # Future: eg-walker for text
  content: RichText
}
```

**Hybrid Logical Clocks**:
```dol
gen event.distributed {
  @crdt(lww, clock="hlc")  # Hybrid Logical Clock for better causality
  timestamp: Timestamp
}
```

### 11.2 CRDT Composition

Nested CRDTs with independent merge semantics:
```dol
gen workspace.collaborative {
  @crdt(or_set)
  documents: Set<DocumentRef>

  // Each document is itself a CRDT
  document DocumentRef {
    @crdt(peritext)
    content: RichText

    @crdt(or_set)
    comments: Set<Comment>
  }
}
```

### 11.3 AI-Assisted Conflict Resolution

```dol
gen merge.policy {
  @crdt(mv_register, resolver="ai_suggest")
  theme: Theme
}

docs {
  When concurrent theme changes occur, MV-Register keeps both values.
  AI resolver suggests best theme based on user preferences and context.
}
```

---

## 12. Security Considerations

### 12.1 Byzantine Fault Tolerance

CRDT strategies assume **honest-but-curious** peers. For **Byzantine** environments (malicious actors):

- **Signature verification**: All CRDT operations signed by author's keypair
- **Operation validation**: Reject ops that violate type constraints
- **Quorum-based confirmation**: Critical operations require BFT committee approval
- **Reputation-based filtering**: Ignore ops from low-reputation peers

### 12.2 GDPR Compliance

**Right to Erasure**:
```dol
gen user.profile {
  @crdt(lww)
  @personal  # New annotation for GDPR-sensitive data
  email: String

  @crdt(lww)
  display_name: String
}

docs {
  @personal fields encrypted with user-specific key.
  Right-to-erasure: Delete encryption key → data unrecoverable across all peers.
}
```

**Key Deletion Protocol**:
1. User requests deletion
2. System deletes user's encryption key from keyring
3. CRDT document still contains encrypted bytes, but unreadable
4. Eventual GC removes orphaned encrypted data

---

## 13. Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| WASM module size | < 200KB compressed | wasm-opt -Oz |
| CRDT merge latency | < 10ms for 10K ops | criterion benchmarks |
| Local operation latency | < 1ms | wasm-bindgen perf trace |
| Sync throughput | > 1000 ops/sec | P2P benchmark harness |
| Convergence time | < 5 seconds after network heal | Integration tests |
| Memory per document | < 50MB for 100K records | Chrome DevTools |

---

## 14. References

### Academic Papers
- Shapiro et al. (2011): "Conflict-free Replicated Data Types" (INRIA)
- Kleppmann & Beresford (2017): "A Conflict-Free Replicated JSON Datatype" (Automerge)
- Litt et al. (2022): "Peritext: A CRDT for Collaborative Rich Text Editing"
- Kleppmann (2025): "eg-walker: An Efficient Approach for Collaborative Text Editing" (EuroSys)

### Implementations
- Automerge 3.0: https://github.com/automerge/automerge
- Loro: https://loro.dev (Rust CRDT library)
- Yrs: https://github.com/y-crdt/y-crdt (Yjs in Rust)
- cr-sqlite: https://github.com/vlcn-io/cr-sqlite (SQLite with CRDTs)

### Protocols
- Willow Protocol: https://willowprotocol.org
- Iroh: https://iroh.computer
- IPFS: https://ipfs.tech

---

## 15. Authors & Acknowledgments

**RFC Authors**:
- arch-dol-crdt: DOL language design and formal semantics
- researcher-crdt-frontier: CRDT theory and convergence proofs

**Contributors**:
- arch-p2p-network: P2P sync protocol design
- arch-wasm-runtime: WASM compilation strategy
- security-auditor: Byzantine fault tolerance review

**Special Thanks**:
- Martin Kleppmann (Automerge, eg-walker)
- Marc Shapiro (CRDT foundations)
- The Ink & Switch team (local-first research)

---

## Appendix A: Grammar Summary

```ebnf
(* Complete CRDT annotation grammar *)

crdt_annotation = '@crdt' , '(' , crdt_strategy , [ ',' , crdt_options ] , ')' ;

crdt_strategy = 'immutable'
              | 'lww'
              | 'or_set'
              | 'pn_counter'
              | 'peritext'
              | 'rga'
              | 'mv_register' ;

crdt_options = option_pair , { ',' , option_pair } ;
option_pair = identifier , '=' , value ;
value = string_literal | number | 'true' | 'false' ;

gene_field = [ crdt_annotation ] , identifier , ':' , type_spec ;

gene_declaration = kw_gene , qualified_identifier , delim_lbrace ,
                   { gene_field } ,
                   { gene_statement } ,
                   delim_rbrace ;
```

---

## Appendix B: Type Compatibility Reference

See **Section 4: Type Compatibility Matrix** for full table.

**Quick Reference**:
- **Immutable**: Any type (one-time set)
- **LWW**: Scalar types (String, Int, Bool, Enum)
- **OR-Set**: `Set<T>`
- **PN-Counter**: `Int`, `UInt` (avoid `Float`)
- **Peritext**: `RichText`, `String`
- **RGA**: `List<T>`, `Vec<T>`
- **MV-Register**: Any type (keeps all concurrent values)

---

## Appendix C: Constraint Categories

### Category A: CRDT-Safe (Compile-Time)
- Immutability: `field never changes`
- Monotonicity: `counter always increases`
- Set membership: `set has element`

### Category B: Eventually-Consistent (Merge-Time)
- Uniqueness: `all entities have unique key`
- Resource bounds: `used <= quota`
- Referential integrity: `foreign_key exists`

### Category C: Strong-Consistency (Operation-Time)
- Atomic transfers: `A.balance -= X AND B.balance += X`
- Uniqueness under high cardinality: `transaction_id is globally unique`
- Access control: `user can_edit IFF user in collaborators`

---

**End of RFC-001: DOL 2.0 CRDT Annotations**

**Status:** Draft
**Next Steps:**
1. Review by arch-wasm-runtime
2. Proof-of-concept implementation (Phase 1)
3. Formal convergence proofs (RFC-001-formal-proofs.md)

**Feedback:** Submit issues to https://github.com/univrs/dol/issues
