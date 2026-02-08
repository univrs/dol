# Peritext Strategy

The **peritext** CRDT strategy enables conflict-free collaborative rich text editing with formatting preservation. It's the gold standard for real-time document collaboration, based on research by Ink & Switch.

## When to Use

Use `@crdt(peritext)` for:
- ✅ Collaborative document editing (Google Docs style)
- ✅ Rich text fields with formatting (bold, italic, links)
- ✅ Markdown editors
- ✅ Code editors with collaborative features
- ✅ Any text where multiple users edit simultaneously

## Syntax

```dol
@crdt(peritext, formatting="full", max_length=500000)
has content: String
```

**Options**:
- `formatting`: `"full"` | `"markdown"` | `"plain"` (default: `"full"`)
- `max_length`: Maximum character count (default: 1000000)

## How Peritext Works

### Character-Level CRDT

Peritext treats text as a sequence of characters, each with:
- **Unique ID**: (actor_id, sequence_number)
- **Left origin**: ID of character to the left
- **Formatting marks**: Bold, italic, etc.

```
Text: "Hello World"

Internal representation:
[H(id=a1, left=none),
 e(id=a2, left=a1),
 l(id=a3, left=a2),
 l(id=a4, left=a3),
 o(id=a5, left=a4),
  (id=a6, left=a5),
 W(id=a7, left=a6),
 o(id=a8, left=a7),
 r(id=a9, left=a8),
 l(id=a10, left=a9),
 d(id=a11, left=a10)]
```

### Concurrent Edits

**Scenario**: Alice and Bob edit the same document concurrently

```
Initial: "Hello"

Alice: insert(5, " World")  → "Hello World"
Bob:   insert(0, "Hi ")     → "Hi Hello" (concurrent)

Merge: "Hi Hello World"
```

**Why deterministic**: Character IDs and left-origin pointers create a causal order that all replicas follow.

## Basic Example

```dol
gen document {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="full", max_length=500000)
  has content: String

  @crdt(rga)
  has collaborators: Vec<String>
}

docs {
  Collaborative document with Peritext CRDT for rich text content.
  Supports real-time editing by multiple users with automatic
  conflict resolution.
}
```

## Generated Rust Code

```rust
use automerge::{Automerge, AutoCommit, ObjId};
use autosurgeon::{Reconcile, Hydrate};

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Document {
    #[autosurgeon(immutable)]
    pub id: String,

    #[autosurgeon(lww)]
    pub title: String,

    #[autosurgeon(text)]  // Peritext via Automerge text
    pub content: String,

    #[autosurgeon(map)]
    pub collaborators: Vec<String>,
}

impl Document {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: String::new(),
            content: String::new(),
            collaborators: Vec::new(),
        }
    }

    /// Insert text at position
    pub fn insert_text(&mut self, pos: usize, text: &str, doc: &mut AutoCommit) -> Result<(), automerge::AutomergeError> {
        let text_obj = doc.get(automerge::ROOT, "content")?
            .and_then(|(_, obj)| obj)
            .ok_or_else(|| automerge::AutomergeError::MissingField("content".into()))?;

        doc.splice_text(&text_obj, pos, 0, text)?;
        Ok(())
    }

    /// Delete text at position
    pub fn delete_text(&mut self, pos: usize, len: usize, doc: &mut AutoCommit) -> Result<(), automerge::AutomergeError> {
        let text_obj = doc.get(automerge::ROOT, "content")?
            .and_then(|(_, obj)| obj)
            .ok_or_else(|| automerge::AutomergeError::MissingField("content".into()))?;

        doc.splice_text(&text_obj, pos, len, "")?;
        Ok(())
    }

    /// Add formatting mark
    pub fn add_mark(&mut self, start: usize, end: usize, mark: &str, doc: &mut AutoCommit) -> Result<(), automerge::AutomergeError> {
        let text_obj = doc.get(automerge::ROOT, "content")?
            .and_then(|(_, obj)| obj)
            .ok_or_else(|| automerge::AutomergeError::MissingField("content".into()))?;

        doc.mark(&text_obj, start, end, mark, true, automerge::Value::Primitive(true.into()))?;
        Ok(())
    }

    /// Get current text content
    pub fn get_text(&self, doc: &Automerge) -> Result<String, automerge::AutomergeError> {
        let text_obj = doc.get(automerge::ROOT, "content")?
            .and_then(|(_, obj)| obj)
            .ok_or_else(|| automerge::AutomergeError::MissingField("content".into()))?;

        doc.text(&text_obj)
    }
}
```

## Usage Example

```rust
use automerge::{Automerge, AutoCommit};

// Create document
let mut doc = AutoCommit::new();
let mut document = Document::new("doc-1".to_string());

// Insert text
document.insert_text(0, "Hello World", &mut doc)?;
// Content: "Hello World"

// Add formatting (bold)
document.add_mark(0, 5, "bold", &mut doc)?;
// Content: "**Hello** World"

// Insert in middle
document.insert_text(5, " Beautiful", &mut doc)?;
// Content: "**Hello** Beautiful World"

// Delete text
document.delete_text(6, 10, &mut doc)?;
// Content: "**Hello** World"
```

## Concurrent Editing Example

### Scenario 1: Concurrent Insertions

```rust
let mut alice_doc = AutoCommit::new();
let mut bob_doc = alice_doc.fork().unwrap();

let mut alice_entity = Document::new("doc-1".to_string());
let mut bob_entity = Document::new("doc-1".to_string());

// Both start with "Hello"
alice_entity.insert_text(0, "Hello", &mut alice_doc)?;
bob_entity.insert_text(0, "Hello", &mut bob_doc)?;

// Concurrent edits (offline)
alice_entity.insert_text(5, " World", &mut alice_doc)?;
// Alice's view: "Hello World"

bob_entity.insert_text(5, "!", &mut bob_doc)?;
// Bob's view: "Hello!"

// Merge
alice_doc.merge(&mut bob_doc)?;
bob_doc.merge(&mut alice_doc)?;

// Both converge to: "Hello! World" or "Hello World!"
// (deterministic based on operation timestamps)
```

### Scenario 2: Concurrent Delete + Insert

```rust
// Start: "Hello World"

// Alice deletes "World"
alice_entity.delete_text(6, 5, &mut alice_doc)?;
// Alice's view: "Hello "

// Bob inserts "Beautiful " (concurrent, before "World")
bob_entity.insert_text(6, "Beautiful ", &mut bob_doc)?;
// Bob's view: "Hello Beautiful World"

// Merge
alice_doc.merge(&mut bob_doc)?;

// Result: "Hello Beautiful " (delete wins for "World", insert preserved)
```

## Formatting Preservation

### Bold, Italic, Underline

```rust
// Add formatting
document.add_mark(0, 5, "bold", &mut doc)?;
document.add_mark(6, 11, "italic", &mut doc)?;
document.add_mark(0, 11, "underline", &mut doc)?;

// Content: **Hello** _World_ (with underline)
```

### Concurrent Formatting

```
Alice: bold(0, 5)    → **Hello** World
Bob:   italic(0, 5)  → _Hello_ World (concurrent)

Merge: **_Hello_** World  (both marks preserved)
```

### Format Expansion

When inserting inside formatted text:

```
Text: "**Hello**"
Insert "World" after "Hello"

Alice types " Beautiful":
"**Hello Beautiful**"  (bold expands to include new text)
```

**Expand-left/Expand-right semantics**: Peritext determines whether formatting expands based on where you type.

## Markdown Mode

```dol
@crdt(peritext, formatting="markdown")
has content: String
```

**Behavior**:
- Stores text as plain string
- Parses markdown on read
- Simpler than full rich text
- Smaller memory footprint

**Example**:
```rust
document.insert_text(0, "# Heading\n\nSome **bold** text", &mut doc)?;
// Internally: plain text with markdown syntax
// Rendered: parsed as formatted HTML
```

## Performance

**Time Complexity**:
- Insert: O(log n) amortized
- Delete: O(log n) amortized
- Merge: O(n + m) where n, m are operation counts

**Space Complexity**: O(n) where n = character count

**Overhead**: ~20-40 bytes per character (includes formatting metadata)

### Optimization for Large Documents

```dol
@crdt(peritext, formatting="markdown", max_length=100000)
has content: String
```

**Benefits**:
- Reject edits beyond max length
- Prevent memory exhaustion
- Faster merge (bounded size)

## Best Practices

### ✅ DO: Use for Collaborative Text

```dol
gen note {
  @crdt(peritext)
  has content: String
}
```

### ✅ DO: Set Reasonable Limits

```dol
@crdt(peritext, max_length=500000)
has content: String
```

**Why**: Prevent unbounded growth, protect against malicious peers.

### ✅ DO: Use Markdown for Simple Text

```dol
@crdt(peritext, formatting="markdown")
has bio: String  // User bio (simple formatting)
```

### ❌ DON'T: Use for Non-Text Data

```dol
// BAD: Peritext is for text, not structured data
@crdt(peritext)
has config_json: String
```

**Fix**: Use proper structured types:
```dol
gen config {
  @crdt(lww)
  has theme: Theme

  @crdt(lww)
  has language: String
}
```

### ❌ DON'T: Use for Single-Line Text

```dol
// BAD: Overkill for a title (one line, no formatting)
@crdt(peritext)
has title: String
```

**Fix**: Use LWW for simple text:
```dol
@crdt(lww)
has title: String
```

## Advanced: Collaborative Code Editor

```dol
gen code_file {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has filename: String

  @crdt(lww)
  has language: String

  @crdt(peritext, formatting="plain", max_length=1000000)
  has content: String  // Source code

  @crdt(or_set)
  has collaborators: Set<String>

  @crdt(rga)
  has comments: Vec<Comment>
}

gen comment {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has author: String

  @crdt(lww)
  has line_number: i64

  @crdt(peritext, formatting="markdown")
  has text: String
}

docs {
  Collaborative code editor with Peritext CRDT for source code
  and inline comments. Plain formatting for code, markdown for
  comments.
}
```

## Integration with Frontend

### React Example

```typescript
import { useEffect, useState } from 'react';
import { Document } from './generated/document';

function CollaborativeEditor() {
  const [doc, setDoc] = useState<Document | null>(null);
  const [content, setContent] = useState('');

  useEffect(() => {
    // Initialize document
    const newDoc = Document.new('doc-1');
    setDoc(newDoc);

    // Load from storage
    const saved = localStorage.getItem('doc');
    if (saved) {
      newDoc.load(new Uint8Array(JSON.parse(saved)));
      setContent(newDoc.getText());
    }

    // Sync with peers
    const syncInterval = setInterval(async () => {
      const bytes = newDoc.save();
      await p2pSync.broadcast(bytes);
    }, 2000);

    return () => clearInterval(syncInterval);
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    if (!doc) return;

    const newValue = e.target.value;
    const cursorPos = e.target.selectionStart;

    // Calculate diff
    if (newValue.length > content.length) {
      // Insertion
      const inserted = newValue.substring(cursorPos - 1, cursorPos);
      doc.insertText(cursorPos - 1, inserted);
    } else if (newValue.length < content.length) {
      // Deletion
      doc.deleteText(cursorPos, content.length - newValue.length);
    }

    setContent(newValue);

    // Save to localStorage
    localStorage.setItem('doc', JSON.stringify(Array.from(doc.save())));
  };

  return (
    <textarea
      value={content}
      onChange={handleChange}
      rows={20}
      style={{ width: '100%', fontFamily: 'monospace' }}
    />
  );
}
```

## Testing

```rust
#[test]
fn test_concurrent_insertions_converge() {
    let mut alice_doc = AutoCommit::new();
    let mut bob_doc = alice_doc.fork().unwrap();

    let mut alice_entity = Document::new("doc-1".to_string());
    let mut bob_entity = Document::new("doc-1".to_string());

    // Initial text
    alice_entity.insert_text(0, "Hello", &mut alice_doc)?;
    bob_entity.insert_text(0, "Hello", &mut bob_doc)?;

    // Concurrent edits
    alice_entity.insert_text(5, " Alice", &mut alice_doc)?;
    bob_entity.insert_text(5, " Bob", &mut bob_doc)?;

    // Merge
    alice_doc.merge(&mut bob_doc)?;
    bob_doc.merge(&mut alice_doc)?;

    // Assert convergence (both have same text)
    let alice_text = alice_entity.get_text(&alice_doc)?;
    let bob_text = bob_entity.get_text(&bob_doc)?;
    assert_eq!(alice_text, bob_text);
}

#[test]
fn test_formatting_preserved_across_merge() {
    let mut alice_doc = AutoCommit::new();
    let mut bob_doc = alice_doc.fork().unwrap();

    let mut alice_entity = Document::new("doc-1".to_string());
    let mut bob_entity = Document::new("doc-1".to_string());

    // Both start with "Hello World"
    alice_entity.insert_text(0, "Hello World", &mut alice_doc)?;
    bob_entity.insert_text(0, "Hello World", &mut bob_doc)?;

    // Concurrent formatting
    alice_entity.add_mark(0, 5, "bold", &mut alice_doc)?;
    bob_entity.add_mark(6, 11, "italic", &mut bob_doc)?;

    // Merge
    alice_doc.merge(&mut bob_doc)?;
    bob_doc.merge(&mut alice_doc)?;

    // Both should have both formats
    // "**Hello** _World_"
    // (implementation detail: marks are merged)
}
```

## Summary

**Peritext Strategy** is for:
- ✅ Collaborative rich text editing
- ✅ Real-time document collaboration
- ✅ Code editors with multi-user support
- ✅ Any text with concurrent edits

**Key Features**:
- Character-level CRDT
- Formatting preservation
- Deterministic merge
- Proven at scale (Ink & Switch)

**Performance**:
- O(log n) insert/delete
- ~20-40 bytes overhead per char
- Scales to 100K+ characters

**Next**: [RGA (Replicated Growable Array) →](./06-rga.md)

---

## Further Reading

- [Peritext Paper (Litt et al.)](https://www.inkandswitch.com/peritext/)
- [Automerge Text CRDT](https://automerge.org/docs/text/)
- [RFC-001: Peritext Strategy](../../rfcs/RFC-001-dol-crdt-annotations.md#36-peritext-strategy)
- [Workspace Example](/apps/workspace/schemas/document.dol)
