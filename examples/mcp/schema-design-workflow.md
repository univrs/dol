# CRDT Schema Design Workflow

This guide walks through the complete workflow for designing DOL schemas with CRDT annotations using the MCP server.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Workflow Overview](#workflow-overview)
3. [Step-by-Step Guide](#step-by-step-guide)
4. [Common Patterns](#common-patterns)
5. [Troubleshooting](#troubleshooting)

## Prerequisites

- DOL MCP server installed and configured
- AI assistant with MCP support (e.g., Claude Desktop)
- Basic understanding of DOL syntax
- Familiarity with CRDT concepts (optional but helpful)

## Workflow Overview

```
┌─────────────────────────────────────────────────────────────┐
│ AI-Assisted CRDT Schema Design Workflow                     │
└─────────────────────────────────────────────────────────────┘

1. Define Requirements
   ├─ What data needs to be replicated?
   ├─ What are the usage patterns?
   ├─ What consistency level is needed?
   └─ What are the performance constraints?
          │
          ▼
2. Get CRDT Recommendations
   ├─ Use recommend_crdt for each field
   ├─ Review alternatives and trade-offs
   └─ Choose appropriate strategies
          │
          ▼
3. Generate Initial Schema
   ├─ Use generate_example for similar use cases
   ├─ Or write schema manually with annotations
   └─ Include exegesis documentation
          │
          ▼
4. Validate Schema
   ├─ Use validate_schema
   ├─ Fix errors and warnings
   └─ Address anti-patterns
          │
          ▼
5. Optimize & Refine
   ├─ Review diagnostics
   ├─ Apply optimizations
   └─ Re-validate
          │
          ▼
6. Document & Deploy
   ├─ Document strategy choices
   ├─ Note constraints and limits
   └─ Deploy with confidence!
```

## Step-by-Step Guide

### Step 1: Define Requirements

Start by clearly articulating what you're building:

**Example Conversation:**

```
User: I'm building a collaborative note-taking app. Users should be able to:
- Create notes with titles and rich text content
- Tag notes with categories
- Share notes with other users
- Track creation and modification timestamps
- Add comments to notes

AI: Great! Let's design a CRDT schema for your note-taking app. I'll analyze
    each requirement:

    1. Note title: Simple text field that users update
    2. Rich text content: Needs concurrent editing support
    3. Tags: Multi-user collection that can be modified
    4. Shared users: Collection of identities
    5. Timestamps: Immutable after creation
    6. Comments: Ordered list of comment items

    Let me get recommendations for each field...
```

### Step 2: Get CRDT Recommendations

For each field, use the `recommend_crdt` tool:

**Note Title:**
```
Usage Pattern: last-write-wins (simple updates)
Field Type: String
Consistency: eventual

Recommendation: lww
Reasoning: Simple single-valued field, LWW is efficient
```

**Rich Text Content:**
```
Usage Pattern: collaborative-text
Field Type: String
Consistency: eventual

Recommendation: peritext
Reasoning: Real-time collaborative editing requires CRDT text
```

**Tags:**
```
Usage Pattern: multi-user-set
Field Type: Set<String>
Consistency: eventual

Recommendation: or_set
Reasoning: Add-wins semantics for collaborative collections
```

### Step 3: Generate Initial Schema

Based on recommendations, create the schema:

```dol
gen note.collaborative {
  @crdt(immutable)
  has id: Uuid

  @crdt(immutable)
  has created_at: Timestamp

  @crdt(immutable)
  has created_by: Identity

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="full", max_length=1000000)
  has content: RichText

  @crdt(or_set)
  has tags: Set<String>

  @crdt(or_set)
  has shared_with: Set<Identity>

  @crdt(lww)
  has last_modified: Timestamp

  @crdt(rga)
  has comments: List<CommentRef>
}

exegesis {
  A collaborative note with immutable identity and timestamps,
  LWW title and last_modified, Peritext rich text content,
  OR-Set tags and sharing, and RGA ordered comments.

  Designed for multi-user concurrent editing with eventual
  consistency and conflict-free merging.
}
```

### Step 4: Validate Schema

Use `validate_schema` to check for issues:

```json
{
  "tool": "validate_schema",
  "parameters": {
    "source": "<paste schema here>"
  }
}
```

**Review Results:**

```json
{
  "valid": true,
  "errors": [],
  "warnings": [],
  "crdt_issues": [
    {
      "severity": "Info",
      "category": "BestPractice",
      "message": "OR-Set field 'tags' has no size constraint",
      "suggestion": "Add constraint: where size <= 1000"
    }
  ]
}
```

### Step 5: Optimize & Refine

Apply suggested optimizations:

```dol
gen note.collaborative {
  @crdt(immutable)
  has id: Uuid

  @crdt(immutable)
  has created_at: Timestamp

  @crdt(immutable)
  has created_by: Identity

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="full", max_length=1000000)
  has content: RichText

  @crdt(or_set)
  has tags: Set<String> where size <= 100

  @crdt(or_set)
  has shared_with: Set<Identity> where size <= 50

  @crdt(lww)
  has last_modified: Timestamp

  @crdt(rga)
  has comments: List<CommentRef> where size <= 1000
}

exegesis {
  A collaborative note with immutable identity and timestamps,
  LWW title and last_modified, Peritext rich text content,
  OR-Set tags and sharing (with size limits), and RGA ordered comments.

  Size constraints prevent unbounded growth:
  - Tags: max 100 (reasonable for categorization)
  - Shared users: max 50 (prevents abuse)
  - Comments: max 1000 (consider pagination for larger threads)

  Designed for multi-user concurrent editing with eventual
  consistency and conflict-free merging.
}
```

### Step 6: Document & Deploy

Add comprehensive documentation:

1. **Strategy Rationale**: Why each CRDT strategy was chosen
2. **Constraints**: Size limits, performance considerations
3. **Migration Path**: How to evolve the schema
4. **Known Issues**: Any edge cases or limitations

**Example Documentation:**

```markdown
# Note Schema Design

## CRDT Strategy Choices

### Immutable Fields (id, created_at, created_by)
- Rationale: Identity fields never change
- No merge conflicts
- Minimal overhead

### LWW Fields (title, last_modified)
- Rationale: Single-valued fields with simple updates
- Trade-off: Concurrent edits lost (last write wins)
- Acceptable because: Titles rarely edited concurrently

### Peritext (content)
- Rationale: Rich text requires conflict-free concurrent editing
- Trade-off: Higher storage/merge overhead
- Necessary for: Good UX in collaborative editing

### OR-Set (tags, shared_with)
- Rationale: Collections with add-wins semantics
- Trade-off: Tombstone overhead
- Necessary for: Conflict-free collaborative collections

### RGA (comments)
- Rationale: Ordered list preserves causal ordering
- Trade-off: Tombstone overhead for deletes
- Necessary for: Comment thread chronology

## Performance Considerations

- Peritext content limited to 1MB
- Tags limited to 100 to prevent abuse
- Comments limited to 1000; pagination recommended
- Tombstone GC recommended every 30 days

## Migration Path

Current version: 1.0.0

Planned evolutions:
- v1.1.0: Add reactions to comments (OR-Set)
- v1.2.0: Add formatting marks to titles (Peritext)
- v2.0.0: Migrate to Hybrid Logical Clocks for causality
```

## Common Patterns

### Pattern 1: User Profile

```dol
gen user.profile {
  @crdt(immutable) has id: Uuid
  @crdt(lww) has display_name: String
  @crdt(lww) has avatar_url: String
  @crdt(peritext, max_length=10000) has bio: String
  @crdt(or_set) has interests: Set<String> where size <= 50
  @crdt(pn_counter, min_value=0) has follower_count: Int
}
```

**Rationale:**
- Immutable ID for distributed identity
- LWW for simple metadata (names, URLs)
- Peritext for bio (allows formatting)
- OR-Set for interests (collaborative)
- PN-Counter for follower count (distributed increment)

### Pattern 2: Chat Message

```dol
gen message.chat {
  @crdt(immutable) has id: Uuid
  @crdt(immutable) has timestamp: Timestamp
  @crdt(lww) has author: Identity
  @crdt(peritext, formatting="markdown") has content: String
  @crdt(or_set) has reactions: Set<Reaction>
  @crdt(lww) has edited_at: Option<Timestamp>
}
```

**Rationale:**
- Immutable message identity
- LWW for author (shouldn't change, but LWW ensures convergence)
- Peritext for collaborative editing
- OR-Set for reactions (add-wins)
- LWW for edit timestamp

### Pattern 3: Task Board

```dol
gen task.item {
  @crdt(immutable) has id: Uuid
  @crdt(lww) has title: String
  @crdt(peritext, formatting="markdown") has description: String
  @crdt(lww) has status: TaskStatus
  @crdt(or_set) has assignees: Set<Identity>
  @crdt(pn_counter, min_value=0) has estimate_hours: Int
  @crdt(lww) has due_date: Option<Timestamp>
}
```

**Rationale:**
- Immutable task ID
- LWW for metadata (title, status, due date)
- Peritext for rich descriptions
- OR-Set for multiple assignees
- PN-Counter for time tracking

## Troubleshooting

### Issue: Schema validation fails with type compatibility error

**Problem:**
```
Error: IncompatibleCrdtStrategy
Field 'count' uses pn_counter with type String
```

**Solution:**
PN-Counter requires numeric types. Change field type to `i32` or use `lww` strategy.

### Issue: Warning about missing immutable ID

**Problem:**
```
Warning: Gene should have an immutable ID field for distributed identity
```

**Solution:**
Add an immutable UUID field:
```dol
@crdt(immutable)
has id: Uuid
```

### Issue: Performance concerns with large collections

**Problem:**
```
Warning: OR-Set field 'members' has no size constraint
```

**Solution:**
Add size constraints to prevent unbounded growth:
```dol
@crdt(or_set)
has members: Set<Identity> where size <= 1000
```

### Issue: Concurrent text edits lost with LWW

**Problem:**
Users report losing their edits when editing simultaneously.

**Solution:**
Migrate from `@crdt(lww)` to `@crdt(peritext)` for text fields that need collaborative editing:

```dol
// Before (problematic)
@crdt(lww)
has description: String

// After (collaborative)
@crdt(peritext, max_length=100000)
has description: String
```

## Best Practices

1. **Always start with immutable IDs**
   - Every replicated entity needs a unique, immutable identifier
   - Use `Uuid` or `Identity` types

2. **Use LWW as default for simple fields**
   - Good for metadata that rarely conflicts
   - Simple and efficient

3. **Reserve Peritext for actual collaboration**
   - Don't use Peritext for fields that don't need concurrent editing
   - Storage overhead is significant

4. **Add size constraints to collections**
   - Prevent unbounded growth
   - Make limits explicit

5. **Document your strategy choices**
   - Explain why each strategy was chosen
   - Note trade-offs and constraints

6. **Validate early and often**
   - Use `validate_schema` during development
   - Catch issues before deployment

7. **Consider eventual consistency**
   - All CRDT strategies are eventually consistent
   - Design for temporary inconsistency

8. **Plan for evolution**
   - Document migration paths
   - Test strategy changes in staging
