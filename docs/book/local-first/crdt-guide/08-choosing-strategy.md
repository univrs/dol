# Choosing the Right CRDT Strategy

This guide helps you select the optimal CRDT strategy for each field in your schema. Making the right choice ensures efficient synchronization, minimal conflicts, and optimal user experience.

## Decision Tree

```
Start: What type of data is this field?

┌──────────────────────────────────────────────┐
│ Does the field change after creation?       │
├──────────────────────────────────────────────┤
│ NO → @crdt(immutable)                        │
│      • IDs, creation timestamps              │
│      • Author identity                       │
│      • Permanent metadata                    │
└──────────────────────────────────────────────┘
                    ↓ YES

┌──────────────────────────────────────────────┐
│ Is it text content that users edit?         │
├──────────────────────────────────────────────┤
│ YES → @crdt(peritext)                        │
│       • Rich text documents                  │
│       • Collaborative editors                │
│       • Comments with formatting             │
└──────────────────────────────────────────────┘
                    ↓ NO

┌──────────────────────────────────────────────┐
│ Is it a number that only increases/         │
│ decreases (never set to arbitrary values)?  │
├──────────────────────────────────────────────┤
│ YES → @crdt(pn_counter)                      │
│       • Like counts, view counts             │
│       • Reputation scores                    │
│       • Credit balances                      │
└──────────────────────────────────────────────┘
                    ↓ NO

┌──────────────────────────────────────────────┐
│ Is it a collection (set or list)?           │
├──────────────────────────────────────────────┤
│ YES                                          │
│   ├─ Unordered (tags, members)              │
│   │  → @crdt(or_set)                         │
│   │                                          │
│   └─ Ordered (tasks, comments)              │
│      → @crdt(rga)                            │
└──────────────────────────────────────────────┘
                    ↓ NO

┌──────────────────────────────────────────────┐
│ Do you need to detect/show conflicts?       │
├──────────────────────────────────────────────┤
│ YES → @crdt(mv_register)                     │
│       • Settings where conflicts matter      │
│       • AI-assisted conflict resolution      │
│       • Multi-valued preferences             │
└──────────────────────────────────────────────┘
                    ↓ NO

┌──────────────────────────────────────────────┐
│ Simple single value (name, status, etc.)    │
├──────────────────────────────────────────────┤
│ → @crdt(lww)                                 │
│   • Metadata fields                          │
│   • User preferences                         │
│   • Status flags                             │
└──────────────────────────────────────────────┘
```

## Quick Reference Table

| Use Case | Strategy | Example |
|----------|----------|---------|
| **IDs and identity** | immutable | `has id: String` |
| **Creation timestamps** | immutable | `has created_at: i64` |
| **Author information** | immutable | `has author: String` |
| **Simple metadata** | lww | `has title: String` |
| **User preferences** | lww | `has theme: Theme` |
| **Status/state** | lww | `has status: Status` |
| **Tags, labels** | or_set | `has tags: Set<String>` |
| **Members, followers** | or_set | `has members: Set<String>` |
| **Like counts** | pn_counter | `has likes: i64` |
| **View counts** | pn_counter | `has views: i64` |
| **Credit balances** | pn_counter | `has balance: i64` |
| **Rich text documents** | peritext | `has content: String` |
| **Comments** | peritext | `has text: String` |
| **Task lists** | rga | `has tasks: Vec<Task>` |
| **Ordered comments** | rga | `has replies: Vec<Reply>` |
| **Conflict-sensitive settings** | mv_register | `has theme: Theme` |

## Common Patterns

### Pattern 1: Standard Entity

**Use case**: Most domain entities

```dol
gen entity {
  // Permanent identity
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has created_at: i64

  @crdt(immutable)
  has creator: String

  // Mutable metadata
  @crdt(lww)
  has name: String

  @crdt(lww)
  has status: Status

  @crdt(lww)
  has updated_at: i64

  // Collections
  @crdt(or_set)
  has tags: Set<String>

  // Metrics
  @crdt(pn_counter, min_value=0)
  has view_count: i64
}
```

### Pattern 2: Collaborative Document

**Use case**: Documents with real-time editing

```dol
gen document {
  // Identity
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has created_at: i64

  // Metadata
  @crdt(lww)
  has title: String

  @crdt(lww)
  has last_modified: i64

  // Collaborative content
  @crdt(peritext, formatting="full")
  has content: String

  // Participants
  @crdt(rga)
  has collaborators: Vec<String>

  @crdt(or_set)
  has tags: Set<String>

  // Version history
  @crdt(rga)
  has versions: Vec<Version>
}
```

### Pattern 3: Social Media Post

**Use case**: Posts with engagement metrics

```dol
gen post {
  // Identity
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has author: String

  @crdt(immutable)
  has created_at: i64

  // Content
  @crdt(peritext, formatting="markdown")
  has content: String

  @crdt(or_set)
  has media: Set<MediaRef>

  // Engagement
  @crdt(pn_counter, min_value=0)
  has like_count: i64

  @crdt(or_set)
  has likers: Set<String>

  @crdt(pn_counter, min_value=0)
  has share_count: i64

  // Comments
  @crdt(rga)
  has comments: Vec<Comment>
}
```

### Pattern 4: Task Management

**Use case**: Kanban boards, task lists

```dol
gen task_board {
  // Identity
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has name: String

  // Columns
  @crdt(rga)
  has columns: Vec<Column>

  // Members
  @crdt(or_set)
  has members: Set<String>
}

gen column {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has name: String

  @crdt(lww)
  has color: String

  // Tasks in this column
  @crdt(rga)
  has tasks: Vec<Task>

  @crdt(lww, min_value=0)
  has wip_limit: i64
}

gen task {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="markdown")
  has description: String

  @crdt(or_set)
  has assignees: Set<String>

  @crdt(pn_counter, min_value=0)
  has estimate_hours: i64

  @crdt(lww)
  has due_date: Option<i64>
}
```

### Pattern 5: User Profile

**Use case**: User accounts and profiles

```dol
gen user_profile {
  // Immutable identity
  @crdt(immutable)
  has did: String

  @crdt(immutable)
  has created_at: i64

  // Mutable profile
  @crdt(lww)
  has display_name: String

  @crdt(lww)
  has avatar_url: String

  @crdt(peritext, formatting="markdown", max_length=5000)
  has bio: String

  // Preferences (detect conflicts)
  @crdt(mv_register)
  has theme: Theme

  @crdt(lww)
  has language: String

  // Social
  @crdt(or_set)
  has interests: Set<String>

  @crdt(pn_counter, min_value=0)
  has reputation: i64

  // Mutual credit
  @crdt(pn_counter, min_value=0)
  has credit_balance: i64

  @crdt(rga)
  has transaction_history: Vec<Transaction>
}
```

## Strategy Comparison

### Performance

| Strategy | Insert | Remove | Merge | Memory Overhead |
|----------|--------|--------|-------|-----------------|
| immutable | O(1) | N/A | O(1) | Minimal |
| lww | O(1) | O(1) | O(1) | Minimal |
| or_set | O(1) | O(n) | O(n) | Moderate (tags) |
| pn_counter | O(1) | O(1) | O(n) | Low (per actor) |
| peritext | O(log n) | O(log n) | O(n+m) | High (per char) |
| rga | O(1) | O(1) | O(n·m) | Moderate (per elem) |
| mv_register | O(1) | O(1) | O(n) | Low (per value) |

### Data Preservation

| Strategy | Preserves All Updates | Data Loss Risk |
|----------|----------------------|----------------|
| immutable | ✅ | None |
| lww | ❌ | ⚠️ Concurrent updates |
| or_set | ✅ | None |
| pn_counter | ✅ | None |
| peritext | ✅ | None |
| rga | ✅ | None |
| mv_register | ✅ | None |

### When to Accept Data Loss

**LWW loses concurrent updates** - but that's okay when:
- ✅ Edits are rare
- ✅ User doesn't expect concurrent edits (settings, profile name)
- ✅ Last value is usually best value (status updates)

**Example**: User profile name
```dol
@crdt(lww)
has display_name: String
```

**Why LWW is fine**: Users rarely change their name concurrently from multiple devices.

## Anti-Patterns

### ❌ Using LWW for Collaborative Text

```dol
// BAD: Concurrent edits lose data
@crdt(lww)
has content: String
```

**Problem**: If Alice writes "Hello" and Bob writes "World" concurrently, only one survives.

**Fix**:
```dol
@crdt(peritext)
has content: String
```

### ❌ Using OR-Set for Ordered Lists

```dol
// BAD: Set is unordered
@crdt(or_set)
has tasks: Set<Task>
```

**Problem**: Task order is lost.

**Fix**:
```dol
@crdt(rga)
has tasks: Vec<Task>
```

### ❌ Using Peritext for Simple Text

```dol
// BAD: Overkill for a one-line title
@crdt(peritext)
has title: String
```

**Problem**: Unnecessary overhead (~40 bytes per char)

**Fix**:
```dol
@crdt(lww)
has title: String
```

### ❌ Using PN-Counter for Non-Monotonic Values

```dol
// BAD: Temperature isn't monotonic
@crdt(pn_counter)
has temperature: i64
```

**Problem**: PN-Counter is for increments/decrements, not arbitrary sets.

**Fix**:
```dol
@crdt(lww)
has temperature: i64
```

## Special Cases

### Case 1: URLs and External IDs

**Question**: immutable or lww?

**Answer**: Depends on whether it can change.

```dol
// If URL should never change (permalink)
@crdt(immutable)
has canonical_url: String

// If URL can be updated (redirect target)
@crdt(lww)
has redirect_url: String
```

### Case 2: Timestamps

**Question**: Which timestamp fields need immutable?

**Answer**: Only creation timestamps.

```dol
@crdt(immutable)
has created_at: i64  // Never changes

@crdt(lww)
has updated_at: i64  // Changes on every edit

@crdt(lww)
has published_at: Option<i64>  // Set when published
```

### Case 3: Boolean Flags

**Question**: lww or mv_register?

**Answer**: Usually lww, unless conflicts matter.

```dol
// Simple flag (last value wins)
@crdt(lww)
has is_archived: Bool

// Conflict detection needed
@crdt(mv_register)
has is_pinned: Bool  // Show user if devices disagree
```

### Case 4: Enums

**Question**: How to handle enum fields?

**Answer**: lww for simple state, mv_register for conflicts.

```dol
// Status changes (last wins)
@crdt(lww)
has status: Status  // enum { Draft, Published, Archived }

// Conflict-sensitive state
@crdt(mv_register)
has priority: Priority  // Show if devices set different priorities
```

### Case 5: Optional Fields

**Question**: Can CRDT annotations work with Option<T>?

**Answer**: Yes, treat as nullable.

```dol
@crdt(lww)
has deleted_at: Option<i64>  // None = not deleted, Some = deleted

@crdt(peritext)
has notes: Option<String>  // None = no notes
```

## Validation

### Type Compatibility Check

Run `dol check` to validate CRDT-type compatibility:

```bash
$ dol check schemas/entity.dol
```

**Checks**:
- ✅ immutable on any type
- ✅ lww on scalar types
- ✅ or_set on Set<T>
- ✅ pn_counter on Int, i64
- ✅ peritext on String
- ✅ rga on Vec<T>, List<T>
- ✅ mv_register on any type

**Common errors**:
```
❌ Error: Type-strategy mismatch
  ├─ Field: tags
  ├─ Type: Set<String>
  ├─ Strategy: @crdt(lww)
  └─ Suggestion: Use @crdt(or_set) for Set types
```

## Testing Your Choices

### Convergence Test

Verify that your CRDT choices guarantee convergence:

```bash
dol test schemas/entity.dol --property convergence
```

### Concurrent Edit Test

Simulate concurrent edits:

```rust
#[test]
fn test_concurrent_edits() {
    let mut doc1 = Automerge::new();
    let mut doc2 = doc1.fork();

    // Apply operations to both docs
    entity1.edit(...);
    entity2.edit(...);

    // Merge
    doc1.merge(&doc2)?;
    doc2.merge(&doc1)?;

    // Assert convergence
    assert_eq!(doc1.save(), doc2.save());
}
```

## Summary

**Choosing the right CRDT strategy**:
1. Start with the decision tree
2. Use immutable for identity
3. Use peritext for text content
4. Use pn_counter for metrics
5. Use or_set/rga for collections
6. Use lww as default for simple fields
7. Use mv_register for conflict detection

**Validate your choices**:
```bash
dol check schemas/*.dol
dol test schemas/*.dol --property convergence
```

**When in doubt**: Start with lww (safe default) and optimize later.

---

**Next**: [VUDO Runtime Architecture →](../vudo-runtime/00-architecture.md)

## Further Reading

- [CRDT Overview](./00-overview.md)
- [Strategy-specific guides](./01-immutable.md)
- [RFC-001: CRDT Annotations](../../rfcs/RFC-001-dol-crdt-annotations.md)
- [Type Compatibility Matrix](../../docs/specification.md#crdt-type-matrix)
