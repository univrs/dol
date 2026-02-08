# Tutorial 07: CRDT Schema Design

> **All 7 CRDT strategies with type compatibility and merge patterns**
>
> **Level**: Advanced | **Time**: 60 minutes | **Lines**: 170+

## Overview

DOL supports 7 CRDT strategies for conflict-free distributed data. This tutorial covers:
- All CRDT strategies with complete examples
- Type compatibility rules
- Merge behavior patterns
- Performance characteristics

## CRDT Strategies Reference

| Strategy | Use Case | Merge Behavior | Type Support |
|----------|----------|----------------|--------------|
| **immutable** | IDs, timestamps | First write wins | All types |
| **lww** | Simple fields | Last write wins (timestamp) | All types |
| **or_set** | Collections | Add-wins, union | Set<T> |
| **pn_counter** | Counters | Sum of increments | Int, Float |
| **peritext** | Rich text | Operation-based | String |
| **rga** | Ordered lists | Causal insertion | Vec<T> |
| **mv_register** | Multi-value | Keep all concurrent | All types |

## Complete Examples: All 7 Strategies

### 1. Immutable - Identity Fields

```dol
gen Document {
    @crdt(immutable)
    has id: string

    @crdt(immutable)
    has created_at: Int

    @crdt(immutable)
    has author_id: string
}

docs {
    Immutable fields are set once and never change.
    Perfect for: IDs, creation timestamps, immutable metadata

    Merge behavior:
    - First write wins
    - Subsequent writes ignored
    - No conflicts possible
}
```

**Implementation**:

```rust
#[derive(Clone)]
struct Document {
    id: Option<String>,  // Option tracks if set
    created_at: Option<i64>,
    author_id: Option<String>,
}

impl Document {
    fn merge(&mut self, other: &Self) {
        // Immutable: keep first write
        if self.id.is_none() {
            self.id = other.id.clone();
        }
        if self.created_at.is_none() {
            self.created_at = other.created_at;
        }
        if self.author_id.is_none() {
            self.author_id = other.author_id.clone();
        }
    }
}
```

### 2. LWW (Last-Write-Wins) - Simple Fields

```dol
gen UserProfile {
    @crdt(lww)
    has name: string

    @crdt(lww, tie_break="actor_id")
    has bio: string

    @crdt(lww)
    has avatar_url: string

    @crdt(lww)
    has status: enum {
        Online,
        Away,
        Offline
    }
}

docs {
    Last-Write-Wins uses timestamps to resolve conflicts.
    Perfect for: User preferences, simple updates, status fields

    Merge behavior:
    - Compare timestamps
    - Keep value with latest timestamp
    - Optional tie-break on actor_id
}
```

**Implementation**:

```rust
#[derive(Clone)]
struct LwwField<T> {
    value: T,
    timestamp: u64,
    actor_id: String,
}

impl<T: Clone> LwwField<T> {
    fn merge(&mut self, other: &Self) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
        } else if other.timestamp == self.timestamp {
            // Tie-break on actor_id
            if other.actor_id > self.actor_id {
                self.value = other.value.clone();
                self.actor_id = other.actor_id.clone();
            }
        }
    }
}
```

### 3. OR-Set (Observed-Remove Set) - Collections

```dol
gen BlogPost {
    @crdt(or_set)
    has tags: Set<string>

    @crdt(or_set)
    has collaborators: Set<string>

    @crdt(or_set)
    has comments: Set<Comment>
}

gen Comment {
    has id: string
    has content: string
}

docs {
    OR-Set provides add-wins semantics for sets.
    Perfect for: Tags, memberships, collections

    Merge behavior:
    - Union of all added elements
    - Remove only observed elements
    - Concurrent adds always win over removes
}
```

**Implementation**:

```rust
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct OrSet<T: Clone + Eq + std::hash::Hash> {
    elements: HashMap<T, HashSet<String>>, // element -> set of unique IDs
}

impl<T: Clone + Eq + std::hash::Hash> OrSet<T> {
    fn add(&mut self, value: T, unique_id: String) {
        self.elements.entry(value)
            .or_insert_with(HashSet::new)
            .insert(unique_id);
    }

    fn remove(&mut self, value: &T, observed_ids: &HashSet<String>) {
        if let Some(ids) = self.elements.get_mut(value) {
            ids.retain(|id| !observed_ids.contains(id));
            if ids.is_empty() {
                self.elements.remove(value);
            }
        }
    }

    fn merge(&mut self, other: &Self) {
        for (elem, ids) in &other.elements {
            self.elements.entry(elem.clone())
                .or_insert_with(HashSet::new)
                .extend(ids.clone());
        }
    }

    fn values(&self) -> Vec<T> {
        self.elements.keys().cloned().collect()
    }
}
```

### 4. PN-Counter (Positive-Negative Counter)

```dol
gen PageAnalytics {
    @crdt(pn_counter, min_value=0)
    has view_count: Int

    @crdt(pn_counter)
    has like_count: Int

    @crdt(pn_counter)
    has share_count: Int

    fun increment_views() {
        this.view_count = this.view_count + 1
    }

    fun add_like() {
        this.like_count = this.like_count + 1
    }

    fun remove_like() {
        this.like_count = this.like_count - 1
    }
}

docs {
    PN-Counter supports distributed counter operations.
    Perfect for: View counts, likes, analytics

    Merge behavior:
    - Each replica tracks positive and negative increments
    - Value = sum(positive) - sum(negative)
    - Commutative and associative
}
```

**Implementation**:

```rust
use std::collections::HashMap;

#[derive(Clone)]
struct PnCounter {
    positive: HashMap<String, i64>,  // replica_id -> positive count
    negative: HashMap<String, i64>,  // replica_id -> negative count
    replica_id: String,
}

impl PnCounter {
    fn increment(&mut self, amount: i64) {
        if amount > 0 {
            *self.positive.entry(self.replica_id.clone())
                .or_insert(0) += amount;
        } else {
            *self.negative.entry(self.replica_id.clone())
                .or_insert(0) += amount.abs();
        }
    }

    fn value(&self) -> i64 {
        let pos: i64 = self.positive.values().sum();
        let neg: i64 = self.negative.values().sum();
        pos - neg
    }

    fn merge(&mut self, other: &Self) {
        for (replica, count) in &other.positive {
            *self.positive.entry(replica.clone()).or_insert(0) =
                (*count).max(*self.positive.get(replica).unwrap_or(&0));
        }
        for (replica, count) in &other.negative {
            *self.negative.entry(replica.clone()).or_insert(0) =
                (*count).max(*self.negative.get(replica).unwrap_or(&0));
        }
    }
}
```

### 5. Peritext - Rich Text Editing

```dol
gen CollaborativeDocument {
    @crdt(peritext, formatting="full", conflict_resolution="left_wins")
    has content: string

    @crdt(peritext, formatting="inline_only")
    has abstract: string
}

gen CodeEditor {
    @crdt(peritext, formatting="none", syntax_highlighting=true)
    has code: string

    @crdt(lww)
    has language: string
}

docs {
    Peritext provides conflict-free collaborative text editing.
    Perfect for: Documents, code editors, rich text

    Merge behavior:
    - Operation-based CRDT
    - Preserves formatting and styles
    - Handles concurrent edits
    - Supports inline formatting, block-level, or none
}
```

**Implementation** (conceptual, uses Peritext library):

```rust
use peritext::{Peritext, Operation};

#[derive(Clone)]
struct RichTextField {
    peritext: Peritext,
    actor_id: String,
}

impl RichTextField {
    fn insert(&mut self, pos: usize, text: &str) {
        let op = Operation::Insert {
            pos,
            content: text.to_string(),
            actor: self.actor_id.clone(),
            timestamp: current_timestamp(),
        };
        self.peritext.apply(op);
    }

    fn delete(&mut self, pos: usize, len: usize) {
        let op = Operation::Delete {
            pos,
            len,
            actor: self.actor_id.clone(),
            timestamp: current_timestamp(),
        };
        self.peritext.apply(op);
    }

    fn format(&mut self, pos: usize, len: usize, style: &str) {
        let op = Operation::Format {
            pos,
            len,
            style: style.to_string(),
            actor: self.actor_id.clone(),
        };
        self.peritext.apply(op);
    }

    fn merge(&mut self, other: &Self) {
        self.peritext.merge(&other.peritext);
    }

    fn to_string(&self) -> String {
        self.peritext.to_string()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
```

### 6. RGA (Replicated Growable Array) - Ordered Lists

```dol
gen TaskList {
    @crdt(rga)
    has tasks: Vec<Task>

    @crdt(rga)
    has priorities: Vec<Priority>

    fun add_task(task: Task) {
        this.tasks.push(task)
    }

    fun insert_task(index: Int, task: Task) {
        this.tasks.insert(index, task)
    }

    fun remove_task(index: Int) {
        this.tasks.remove(index)
    }
}

gen Task {
    has id: string
    has title: string
    has completed: Bool
}

docs {
    RGA maintains causally ordered sequences.
    Perfect for: Todo lists, timelines, ordered collections

    Merge behavior:
    - Each element has unique ID and causal position
    - Concurrent insertions preserved
    - Maintains relative order
}
```

**Implementation**:

```rust
#[derive(Clone, PartialEq, Eq)]
struct RgaElement<T> {
    id: String,
    value: T,
    after: Option<String>,  // ID of element this comes after
}

#[derive(Clone)]
struct Rga<T: Clone> {
    elements: Vec<RgaElement<T>>,
    tombstones: HashSet<String>,  // Deleted element IDs
}

impl<T: Clone> Rga<T> {
    fn insert(&mut self, index: usize, value: T, id: String) {
        let after = if index == 0 {
            None
        } else {
            self.elements.get(index - 1).map(|e| e.id.clone())
        };

        let elem = RgaElement { id, value, after };
        self.elements.insert(index, elem);
    }

    fn delete(&mut self, index: usize) {
        if let Some(elem) = self.elements.get(index) {
            self.tombstones.insert(elem.id.clone());
            self.elements.remove(index);
        }
    }

    fn merge(&mut self, other: &Self) {
        // Add elements from other that we don't have
        for elem in &other.elements {
            if !self.elements.iter().any(|e| e.id == elem.id)
                && !self.tombstones.contains(&elem.id) {
                self.insert_sorted(elem.clone());
            }
        }

        // Add tombstones
        self.tombstones.extend(other.tombstones.clone());

        // Remove tombstoned elements
        self.elements.retain(|e| !self.tombstones.contains(&e.id));
    }

    fn insert_sorted(&mut self, elem: RgaElement<T>) {
        // Find correct position based on causal order
        let pos = match &elem.after {
            None => 0,
            Some(after_id) => {
                self.elements.iter()
                    .position(|e| &e.id == after_id)
                    .map(|i| i + 1)
                    .unwrap_or(self.elements.len())
            }
        };
        self.elements.insert(pos, elem);
    }

    fn values(&self) -> Vec<T> {
        self.elements.iter().map(|e| e.value.clone()).collect()
    }
}
```

### 7. MV-Register (Multi-Value Register)

```dol
gen Configuration {
    @crdt(mv_register)
    has setting: string

    @crdt(mv_register)
    has theme: enum {
        Light,
        Dark,
        Auto
    }

    fun get_values() -> Vec<string> {
        // Returns all concurrent values
        return this.setting.all_values()
    }

    fun resolve() -> string {
        // Application-specific resolution
        let values = this.setting.all_values()
        return values.max_by(|a, b| a.len().cmp(&b.len()))
    }
}

docs {
    MV-Register keeps all concurrent values.
    Perfect for: Conflict detection, manual resolution

    Merge behavior:
    - Concurrent writes create siblings
    - Application chooses resolution strategy
    - Preserves all conflicts for visibility
}
```

**Implementation**:

```rust
use std::collections::HashSet;

#[derive(Clone)]
struct MvRegister<T: Clone + Eq + std::hash::Hash> {
    values: HashMap<String, (T, u64)>,  // replica_id -> (value, timestamp)
}

impl<T: Clone + Eq + std::hash::Hash> MvRegister<T> {
    fn write(&mut self, value: T, replica_id: String, timestamp: u64) {
        self.values.insert(replica_id, (value, timestamp));
    }

    fn merge(&mut self, other: &Self) {
        for (replica, (value, ts)) in &other.values {
            match self.values.get(replica) {
                Some((_, self_ts)) if ts > self_ts => {
                    self.values.insert(replica.clone(), (value.clone(), *ts));
                }
                None => {
                    self.values.insert(replica.clone(), (value.clone(), *ts));
                }
                _ => {}
            }
        }

        // Remove dominated values (causally earlier)
        self.remove_dominated();
    }

    fn all_values(&self) -> Vec<T> {
        self.values.values()
            .map(|(v, _)| v.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn has_conflict(&self) -> bool {
        self.all_values().len() > 1
    }

    fn remove_dominated(&mut self) {
        // Complex causal ordering logic
        // Keep only concurrent (incomparable) values
    }
}
```

## Type Compatibility Matrix

| Strategy | string | Int/Float | Bool | enum | Set<T> | Vec<T> | Option<T> |
|----------|--------|-----------|------|------|--------|--------|-----------|
| immutable | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| lww | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| or_set | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| pn_counter | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| peritext | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| rga | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| mv_register | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |

## Performance Characteristics

| Strategy | Merge | Memory | Metadata | Best For |
|----------|-------|--------|----------|----------|
| immutable | O(1) | Low | None | IDs |
| lww | O(1) | Low | Timestamp + Actor | Simple fields |
| or_set | O(n) | High | Unique IDs per element | Collections |
| pn_counter | O(r) | Medium | Per-replica counts | Counters |
| peritext | O(n²) | High | Operation log | Rich text |
| rga | O(n log n) | High | Causal metadata | Lists |
| mv_register | O(r) | Medium | Per-replica values | Conflicts |

where n = number of elements, r = number of replicas

## Best Practices

### 1. Choose the Right Strategy

```dol
// ✅ Good: Match strategy to use case
gen Product {
    @crdt(immutable)  has id: string           // Never changes
    @crdt(lww)        has price: Float64        // Simple updates
    @crdt(or_set)     has tags: Set<string>    // Collection
    @crdt(pn_counter) has view_count: Int      // Counter
}

// ❌ Bad: Wrong strategies
gen Product {
    @crdt(or_set)     has id: string           // Overkill!
    @crdt(immutable)  has price: Float64       // Can't update!
    @crdt(lww)        has tags: Set<string>    // Loses data!
}
```

### 2. Minimize CRDT Overhead

```dol
// ✅ Good: Only CRDT what's distributed
gen LocalState {
    has session_id: string      // Not distributed, no CRDT
    has cache: Map<string, Int> // Local only

    @crdt(lww)
    has synced_value: string    // Distributed, needs CRDT
}
```

### 3. Document Merge Semantics

```dol
gen ShoppingCart {
    @crdt(or_set)
    has items: Set<CartItem>

    docs {
        Using OR-Set for items ensures:
        - Adding same item on two devices = appears once
        - Removing item only if observed on that device
        - No lost additions from concurrent adds
    }
}
```

---

**Next**: [Tutorial 08: Advanced Reflection Patterns](./08-Advanced-Reflection-Patterns.md)
