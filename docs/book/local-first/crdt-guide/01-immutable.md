# Immutable Strategy

The **immutable** CRDT strategy ensures a field is set exactly once and never modified. This is essential for permanent identity fields like IDs, creation timestamps, and author information.

## When to Use

Use `@crdt(immutable)` for:
- ✅ Unique identifiers (UUIDs, primary keys)
- ✅ Creation timestamps
- ✅ Author/creator identity
- ✅ Permanent metadata that should never change

## Syntax

```dol
@crdt(immutable)
has field_name: FieldType
```

## Merge Semantics

**Rule**: First write wins. If multiple replicas try to set the value concurrently, the earliest timestamp wins (with actor ID as tiebreaker).

**Mathematical Property**:
```
merge(immutable(v1, t1), immutable(v2, t2)) =
  if t1 < t2:
    immutable(v1, t1)
  elif t1 == t2:
    immutable(min_actor_id(v1, v2), t1)  // Deterministic tiebreaker
  else:
    immutable(v2, t2)
```

## Basic Example

```dol
gen document {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has created_at: i64

  @crdt(immutable)
  has author: String

  @crdt(lww)
  has title: String
}

constraint document.immutability {
  document never changes id
  document never changes created_at
  document never changes author
}

docs {
  Document with immutable identity. The id, creation timestamp,
  and author are set once at creation and never modified.
}
```

## Generated Rust Code

```rust
use automerge::{Automerge, AutoCommit};
use autosurgeon::{Reconcile, Hydrate};

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Document {
    #[autosurgeon(immutable)]
    pub id: String,

    #[autosurgeon(immutable)]
    pub created_at: i64,

    #[autosurgeon(immutable)]
    pub author: String,

    #[autosurgeon(lww)]
    pub title: String,
}

impl Document {
    pub fn new(id: String, author: String) -> Self {
        Self {
            id,
            created_at: chrono::Utc::now().timestamp(),
            author,
            title: String::new(),
        }
    }

    // Note: No setter for immutable fields!
    // pub fn set_id(...) {} ← This method is NOT generated

    pub fn set_title(&mut self, title: String, doc: &mut AutoCommit) {
        self.title = title;
        autosurgeon::reconcile(doc, self).unwrap();
    }
}
```

## Usage Example

```rust
// Create document
let doc = Document::new(
    Uuid::new_v4().to_string(),
    "alice@example.com".to_string(),
);

// ✅ Can modify mutable fields
doc.set_title("My Document");

// ❌ Cannot modify immutable fields (no setter method)
// doc.set_id(...);  // Compile error: method not found
// doc.set_author(...);  // Compile error: method not found
```

## Concurrent Creation Scenario

### Scenario: Two Peers Create Same Entity

```rust
// Alice creates document at t=1000
let mut alice_doc = Automerge::new();
let alice_entity = Document {
    id: "doc-123",
    created_at: 1000,
    author: "alice",
    title: "Alice's Doc",
};
alice_doc.put(&automerge::ROOT, "doc", alice_entity)?;

// Bob creates same document at t=1001 (concurrent, didn't see Alice's)
let mut bob_doc = Automerge::new();
let bob_entity = Document {
    id: "doc-123",
    created_at: 1001,
    author: "bob",
    title: "Bob's Doc",
};
bob_doc.put(&automerge::ROOT, "doc", bob_entity)?;

// Merge
alice_doc.merge(&bob_doc)?;
bob_doc.merge(&alice_doc)?;

// Result: Both converge to Alice's version (t=1000 < t=1001)
assert_eq!(alice_doc.get("doc").author, "alice");
assert_eq!(bob_doc.get("doc").author, "alice");
```

**Why Alice wins**: Her timestamp (1000) is earlier than Bob's (1001), so first-write-wins rule applies.

## Tiebreaker Example

### Scenario: Exact Same Timestamp

```rust
// Alice and Bob create at EXACT same timestamp (rare but possible)
let timestamp = 1000;

let alice_entity = Document {
    id: "doc-123",
    created_at: timestamp,
    author: "alice",  // actor_id: 0xaa...
    title: "Alice's Doc",
};

let bob_entity = Document {
    id: "doc-123",
    created_at: timestamp,
    author: "bob",  // actor_id: 0xbb...
};

// Tiebreaker: actor_id comparison
// 0xaa... < 0xbb... → Alice wins
```

**Deterministic**: All replicas use the same tiebreaker (lexicographic actor ID comparison).

## Best Practices

### ✅ DO: Use for Permanent Identity

```dol
gen user {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has did: String  // Decentralized identifier

  @crdt(immutable)
  has created_at: i64
}
```

### ✅ DO: Combine with Constraints

```dol
constraint user.immutability {
  user never changes id
  user never changes did
  user never changes created_at
}
```

This provides **double protection**: CRDT strategy + constraint validation.

### ❌ DON'T: Use for User-Editable Fields

```dol
// BAD: Users can't change their email!
@crdt(immutable)
has email: String
```

**Fix**: Use `@crdt(lww)` for changeable metadata:

```dol
@crdt(lww)
has email: String
```

### ❌ DON'T: Use for Derived Values

```dol
// BAD: full_name should be computed from first + last
@crdt(immutable)
has full_name: String
```

**Fix**: Compute derived values in application logic:

```rust
impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
```

## Type Compatibility

| Type | Compatible | Notes |
|------|-----------|-------|
| `String` | ✅ | Most common use case |
| `Int`, `i64` | ✅ | For timestamps, sequential IDs |
| `Bool` | ✅ | For permanent flags |
| `Enum` | ✅ | For immutable status |
| `Uuid` | ✅ | Recommended for IDs |
| `Set<T>` | ❌ | Use `@crdt(or_set)` instead |
| `Vec<T>` | ❌ | Use `@crdt(rga)` instead |
| Custom Struct | ✅ | For composite identities |

## Performance

**Time Complexity**:
- Set: O(1)
- Merge: O(1)

**Space Complexity**: O(1) per field

**Overhead**: Minimal (~8-16 bytes for timestamp + actor ID)

## Advanced: Composite Immutable Identity

```dol
gen composite_identity {
  @crdt(immutable)
  has user_id: String

  @crdt(immutable)
  has device_id: String

  @crdt(immutable)
  has session_id: String
}

docs {
  Composite immutable identity for multi-device sessions.
  Combines user, device, and session for unique identification.
}
```

## Integration with Schema Evolution

When evolving schemas, immutable fields **cannot change strategy**:

```dol
// ❌ INVALID EVOLUTION
evolves user @2.0.0 > 1.0.0 {
  changes id from @crdt(immutable) to @crdt(lww)  // ERROR!

  because "Cannot make immutable field mutable"
}
```

**Error**:
```
Error: Unsafe migration
  ├─ From: @crdt(immutable)
  └─ To: @crdt(lww)
  Reason: Cannot make an immutable field mutable without data loss
```

**Allowed evolutions**:
- Add new immutable fields ✅
- Remove immutable fields ✅
- Change immutable field type (with migration) ✅

## Testing

```rust
#[test]
fn test_immutable_field_first_write_wins() {
    let mut doc1 = Automerge::new();
    let mut doc2 = doc1.fork();

    // Concurrent sets (simulated by setting before merge)
    let entity1 = Entity { id: "id-1", timestamp: 1000 };
    let entity2 = Entity { id: "id-2", timestamp: 1001 };

    doc1.put(&ROOT, "entity", entity1)?;
    doc2.put(&ROOT, "entity", entity2)?;

    // Merge
    doc1.merge(&doc2)?;
    doc2.merge(&doc1)?;

    // First write (lower timestamp) wins
    assert_eq!(doc1.get("entity").id, "id-1");
    assert_eq!(doc2.get("entity").id, "id-1");
}

#[test]
fn test_immutable_field_cannot_be_modified() {
    let mut doc = Automerge::new();
    let entity = Entity { id: "id-1", timestamp: 1000 };
    doc.put(&ROOT, "entity", entity)?;

    // Attempt to modify (should be rejected or no-op)
    let result = doc.put(&ROOT, "entity.id", "id-2");
    assert!(result.is_err() || doc.get("entity").id == "id-1");
}
```

## Common Pitfalls

### Pitfall 1: Using Mutable IDs

```dol
// ❌ DON'T
@crdt(lww)
has id: String  // ID can change! Bad for references
```

**Fix**:
```dol
@crdt(immutable)
has id: String  // ID is permanent
```

### Pitfall 2: Forgetting Creation Timestamp

```dol
// ❌ Incomplete
gen entity {
  @crdt(immutable)
  has id: String
  // Missing created_at!
}
```

**Fix**:
```dol
gen entity {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has created_at: i64  // Track when entity was created
}
```

### Pitfall 3: Immutable User Preferences

```dol
// ❌ Users can't change theme!
@crdt(immutable)
has theme: Theme
```

**Fix**:
```dol
@crdt(lww)  // or @crdt(mv_register) for conflict detection
has theme: Theme
```

## Real-World Example: Blog Post

```dol
gen blog_post {
  // Immutable identity
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has created_at: i64

  @crdt(immutable)
  has author: String

  @crdt(immutable)
  has slug: String  // Permanent URL slug

  // Mutable content
  @crdt(lww)
  has title: String

  @crdt(peritext)
  has content: String

  @crdt(or_set)
  has tags: Set<String>

  @crdt(lww)
  has published_at: Option<i64>

  @crdt(pn_counter, min_value=0)
  has view_count: i64
}

constraint blog_post.immutability {
  blog_post never changes id
  blog_post never changes created_at
  blog_post never changes author
  blog_post never changes slug
}

constraint blog_post.slug_format {
  slug matches "^[a-z0-9-]+$"
  slug has length between 3 and 100
}

docs {
  Blog post with immutable identity (id, author, slug, created_at)
  and mutable content. The slug provides a permanent URL even if
  the title changes.

  Example URLs:
  - /posts/my-first-post  (slug never changes)
  - Title can change from "My First Post" to "Updated Post"
}
```

## Summary

**Immutable Strategy** is for:
- ✅ Permanent identity (IDs, DIDs, slugs)
- ✅ Creation timestamps
- ✅ Author/creator information
- ✅ Any field that should never change

**Key Properties**:
- First write wins (deterministic tiebreaker)
- Minimal overhead
- Enforced at type level (no setter methods)
- Essential for referential integrity

**Next**: [Last-Write-Wins (LWW) →](./02-lww.md)

---

## Further Reading

- [RFC-001: Immutable Strategy](../../rfcs/RFC-001-dol-crdt-annotations.md#32-immutable-strategy)
- [Automerge Immutable Fields](https://automerge.org/docs/immutable)
- [CRDT Type System](../../docs/specification.md#crdt-types)
