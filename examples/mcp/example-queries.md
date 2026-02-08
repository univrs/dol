# DOL MCP Example Queries

This document provides example interactions with the DOL MCP server for AI-assisted CRDT schema design.

## Setup

1. Install the DOL MCP server:
```bash
cargo install --path . --features cli
```

2. Configure Claude Desktop (or another MCP client) with the server configuration from `claude-desktop-config.json`.

3. Start using DOL tools in your AI assistant conversations!

## Example Interactions

### Example 1: Get CRDT Recommendation

**User Query:**
> I need a field for chat message content that multiple users can edit concurrently. What CRDT strategy should I use?

**AI Assistant Uses Tool:**
```json
{
  "tool": "recommend_crdt",
  "parameters": {
    "field_name": "content",
    "field_type": "String",
    "usage_pattern": "collaborative-text",
    "consistency_requirement": "eventual"
  }
}
```

**Expected Response:**
```json
{
  "field_name": "content",
  "field_type": "String",
  "recommended_strategy": "peritext",
  "confidence": "high",
  "reasoning": "Peritext CRDT enables conflict-free collaborative text editing with formatting. Ideal for rich text fields (String). Preserves concurrent edits and user intent. Usage pattern: CollaborativeText. Best-in-class for collaborative documents.",
  "example": "@crdt(peritext, formatting=\"full\") has content: String",
  "alternatives": [
    {
      "strategy": "lww",
      "reason": "Simpler but loses concurrent edits (last writer wins)"
    },
    {
      "strategy": "mv_register",
      "reason": "Keeps all concurrent values for manual resolution"
    }
  ],
  "trade_offs": {
    "pros": [
      "Conflict-free concurrent editing",
      "Preserves formatting",
      "Excellent UX for collaboration"
    ],
    "cons": [
      "Higher storage overhead",
      "More complex merge logic",
      "Larger sync payloads"
    ]
  }
}
```

### Example 2: Validate a CRDT Schema

**User Query:**
> Can you validate this DOL schema and check for CRDT anti-patterns?

```dol
gen document.collaborative {
  @crdt(lww)
  tags: Set<String>

  @crdt(peritext)
  content: String
}
```

**AI Assistant Uses Tool:**
```json
{
  "tool": "validate_schema",
  "parameters": {
    "source": "gen document.collaborative {\n  @crdt(lww)\n  tags: Set<String>\n  \n  @crdt(peritext)\n  content: String\n}\n\nexegesis {\n  Document.\n}"
  }
}
```

**Expected Response:**
```json
{
  "valid": false,
  "errors": [],
  "warnings": [],
  "crdt_issues": [
    {
      "severity": "Warning",
      "category": "AntiPattern",
      "message": "Field 'tags' uses LWW on collection type Set<String>. Entire collection will be replaced on conflict.",
      "suggestion": "Use @crdt(or_set) for Set or @crdt(rga) for Vec/List instead",
      "field": "tags"
    },
    {
      "severity": "Warning",
      "category": "Performance",
      "message": "Field 'content' uses Peritext without max_length. Unbounded text can lead to memory issues.",
      "suggestion": "Add max_length option: @crdt(peritext, max_length=1000000)",
      "field": "content"
    }
  ]
}
```

### Example 3: Explain a CRDT Strategy

**User Query:**
> What is the Peritext CRDT strategy and when should I use it?

**AI Assistant Uses Tool:**
```json
{
  "tool": "explain_strategy",
  "parameters": {
    "strategy": "peritext"
  }
}
```

**Expected Response:**
```
Peritext Strategy:
- Collaborative rich text editing
- Conflict-free concurrent editing
- Preserves formatting and user intent
- Based on RGA + formatting marks
- Trade-off: Higher storage/merge overhead
- Best-in-class for document collaboration
```

### Example 4: Generate Example Schema

**User Query:**
> Show me an example of a CRDT-annotated schema for a task board application.

**AI Assistant Uses Tool:**
```json
{
  "tool": "generate_example",
  "parameters": {
    "use_case": "task_board"
  }
}
```

**Expected Response:**
```dol
gen task.item {
  @crdt(immutable)
  has id: Uuid

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="markdown")
  has description: RichText

  @crdt(lww)
  has status: TaskStatus

  @crdt(or_set)
  has assignees: Set<Identity>

  @crdt(pn_counter, min_value=0)
  has estimate_hours: Int

  @crdt(lww)
  has due_date: Option<Timestamp>
}

exegesis {
  A collaborative task item with LWW metadata,
  Peritext descriptions, and OR-Set assignees.
}
```

## Advanced Usage Patterns

### Pattern 1: Iterative Schema Design

```
User: I'm building a collaborative whiteboard. What fields do I need?

AI: Let me help design a CRDT schema for a collaborative whiteboard.
    First, what objects will be on the whiteboard? (e.g., shapes, text, images)

User: Rectangles, circles, text boxes, and arrows.

AI: [Uses recommend_crdt for each field type]
    [Generates complete schema with generate_example]
    [Validates with validate_schema]

    Here's a recommended schema:

    gen whiteboard.element {
      @crdt(immutable)
      has id: Uuid

      @crdt(mv_register)  # Multiple users may set type concurrently
      has element_type: ElementType

      @crdt(lww)
      has position: Point

      @crdt(lww)
      has size: Size

      @crdt(peritext)
      has text_content: Option<String>

      @crdt(lww)
      has color: Color
    }
```

### Pattern 2: Schema Migration

```
User: I want to migrate this field from LWW to Peritext. Is that safe?

AI: [Uses validate_schema on both versions]
    [Checks RFC-001 migration rules]

    Migration from @crdt(lww) to @crdt(peritext) on String is SAFE.

    Existing values will be converted to plain Peritext (no formatting).
    This is a one-way migration - you cannot go back to LWW without data loss.

    Recommended migration:
    1. Deploy new schema with peritext
    2. Convert existing LWW strings to Peritext format
    3. Monitor merge performance
```

### Pattern 3: Performance Optimization

```
User: My document sync is slow. Can you analyze my schema?

AI: [Uses validate_schema with diagnostics]

    Found 3 performance issues:

    1. Field 'content' uses Peritext without max_length
       - Unbounded text can cause memory issues
       - Add: max_length=1000000

    2. Gene 'Document' has 6 collection fields with tombstone overhead
       - Consider periodic garbage collection

    3. Combining Peritext + RGA has O(n) merge complexity
       - Monitor sync performance with large documents
       - Consider splitting into separate genes
```

## Tool Reference

### validate_schema

Validates a DOL schema for CRDT correctness and detects anti-patterns.

**Parameters:**
- `source` (required): DOL source code to validate

**Returns:** Validation result with errors, warnings, and CRDT-specific issues

### recommend_crdt

Recommends a CRDT strategy based on usage pattern.

**Parameters:**
- `field_name` (required): Name of the field
- `field_type` (required): Type (e.g., "String", "i32", "Set<String>")
- `usage_pattern` (required): One of:
  - `write-once`: Set once, never modified
  - `last-write-wins`: Simple updates, LWW resolution
  - `collaborative-text`: Real-time text editing
  - `multi-user-set`: Collaborative collections
  - `counter`: Numeric counters
  - `ordered-list`: Sequences with ordering
- `consistency_requirement` (optional): "eventual" (default), "causal", or "strong"

**Returns:** CRDT recommendation with strategy, reasoning, example, alternatives, and trade-offs

### explain_strategy

Explains a CRDT strategy's semantics and trade-offs.

**Parameters:**
- `strategy` (required): One of: immutable, lww, peritext, or_set, pn_counter, rga, mv_register

**Returns:** Detailed explanation of the strategy

### generate_example

Generates example DOL schemas for common use cases.

**Parameters:**
- `use_case` (optional): One of:
  - `chat_message` (default)
  - `task_board`
  - `user_profile`
  - `counter`

**Returns:** Complete DOL schema with CRDT annotations

## Tips for AI Assistants

1. **Always validate** after making schema changes
2. **Check alternatives** - the top recommendation may not always be best for specific constraints
3. **Consider trade-offs** - higher confidence doesn't mean no trade-offs
4. **Reference RFC-001** - the type compatibility matrix is authoritative
5. **Think about consistency** - eventual vs. causal vs. strong affects strategy choice
6. **Monitor performance** - CRDT strategies have different overhead characteristics
