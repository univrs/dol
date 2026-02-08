# Your First Local-First App

In this tutorial, we'll build a collaborative note-taking app in under 30 minutes. You'll learn:

- How to define schemas in DOL
- Which CRDT strategy to use for each field
- How to compile to WASM
- How to integrate with a web frontend
- How P2P sync works

## What We're Building

A simple note-taking app with:
- ✅ Offline-first operation
- ✅ Real-time collaborative editing
- ✅ Automatic conflict resolution
- ✅ P2P synchronization (no server required)

**Demo**: Two users can edit the same note offline, and changes merge automatically when they reconnect.

## Prerequisites

- DOL toolchain installed ([Installation Guide](./01-installation.md))
- Basic knowledge of Rust and JavaScript/TypeScript
- A text editor

## Step 1: Create a New Project

```bash
# Create project
dol new note-app --template minimal
cd note-app

# Project structure
ls -R
# .
# ├── schemas/
# ├── src/
# ├── web/
# ├── Cargo.toml
# └── dol.toml
```

## Step 2: Define the Schema

Create `schemas/note.dol`:

```dol
gen note {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="markdown")
  has content: String

  @crdt(or_set)
  has tags: Set<String>

  @crdt(lww)
  has created_at: i64

  @crdt(lww)
  has updated_at: i64

  @crdt(lww)
  has author: String
}

constraint note.immutability {
  note never changes id
  note never changes created_at
  note never changes author
}

constraint note.timestamps {
  updated_at always >= created_at
}

docs {
  A collaborative note with:
  - Immutable identity (id, author, created_at)
  - LWW metadata (title, timestamps)
  - Peritext rich text content (Markdown)
  - OR-Set tags (add-wins semantics)

  Conflict Resolution:
  - Title: Last write wins
  - Content: Peritext CRDT merges character-level edits
  - Tags: Concurrent adds preserved, removes win only if observed
}
```

### Why These CRDT Strategies?

| Field | Strategy | Reason |
|-------|----------|--------|
| `id` | immutable | Identity never changes |
| `title` | lww | Simple metadata, last edit wins |
| `content` | peritext | Collaborative text editing |
| `tags` | or_set | Tags can be added concurrently |
| `created_at` | lww | Metadata (set once at creation) |
| `updated_at` | lww | Metadata (updated on each edit) |
| `author` | lww | Metadata |

## Step 3: Validate the Schema

```bash
dol check schemas/note.dol
```

**Expected output**:
```
✅ Syntax valid
✅ Type compatibility: All CRDT strategies match field types
✅ Constraint validation: All constraints are CRDT-safe or eventually consistent
✅ CRDT coverage: 7/7 fields annotated (100%)

✅ All checks passed
```

## Step 4: Generate Rust Code

```bash
dol codegen rust schemas/note.dol --output src/generated/
```

**Generated files**:
- `src/generated/note.rs` - Rust struct with CRDT operations
- `src/generated/note_wasm.rs` - WASM bindings for JavaScript

Inspect the generated code:

```rust
// src/generated/note.rs (simplified)
use automerge::{Automerge, AutoCommit, ReadDoc};
use autosurgeon::{Reconcile, Hydrate};

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Note {
    #[autosurgeon(immutable)]
    pub id: String,

    #[autosurgeon(lww)]
    pub title: String,

    #[autosurgeon(text)]  // Peritext via Automerge text
    pub content: String,

    #[autosurgeon(map)]   // OR-Set via map
    pub tags: HashMap<String, bool>,

    #[autosurgeon(lww)]
    pub created_at: i64,

    #[autosurgeon(lww)]
    pub updated_at: i64,

    #[autosurgeon(lww)]
    pub author: String,
}

impl Note {
    pub fn new(id: String, author: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            title: String::new(),
            content: String::new(),
            tags: HashMap::new(),
            created_at: now,
            updated_at: now,
            author,
        }
    }

    pub fn set_title(&mut self, title: String, doc: &mut AutoCommit) {
        self.title = title;
        self.updated_at = chrono::Utc::now().timestamp();
        autosurgeon::reconcile(doc, self).unwrap();
    }

    pub fn edit_content(&mut self, pos: usize, text: &str, doc: &mut AutoCommit) {
        // Peritext operations on Automerge text
        let text_obj = doc.get(automerge::ROOT, "content")
            .expect("content field")
            .expect_text();
        doc.splice_text(&text_obj, pos, 0, text).unwrap();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn add_tag(&mut self, tag: String, doc: &mut AutoCommit) {
        self.tags.insert(tag, true);
        self.updated_at = chrono::Utc::now().timestamp();
        autosurgeon::reconcile(doc, self).unwrap();
    }

    pub fn remove_tag(&mut self, tag: &str, doc: &mut AutoCommit) {
        self.tags.remove(tag);
        self.updated_at = chrono::Utc::now().timestamp();
        autosurgeon::reconcile(doc, self).unwrap();
    }

    pub fn merge(local: &Automerge, remote: &Automerge) -> Automerge {
        local.merge(remote).unwrap()
    }
}
```

## Step 5: Build WASM Module

```bash
# Add WASM target (if not already added)
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings
wasm-bindgen target/wasm32-unknown-unknown/release/note_app.wasm \
  --out-dir web/src/generated \
  --target web

# Optimize WASM
wasm-opt web/src/generated/note_app_bg.wasm -Oz -o web/src/generated/note_app_bg.wasm
```

**Output**:
- `web/src/generated/note_app.js` - JavaScript wrapper
- `web/src/generated/note_app_bg.wasm` - Optimized WASM module (~85KB)

## Step 6: Create the Frontend

Create `web/src/App.tsx`:

```typescript
import { useEffect, useState } from 'react';
import { Note } from './generated/note_app';

function App() {
  const [note, setNote] = useState<Note | null>(null);
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [newTag, setNewTag] = useState('');

  useEffect(() => {
    // Create note on mount
    const newNote = Note.new(
      crypto.randomUUID(),
      'user@example.com'  // In production, use actual identity
    );
    setNote(newNote);

    // Load from local storage
    const saved = localStorage.getItem('note');
    if (saved) {
      newNote.load(new Uint8Array(JSON.parse(saved)));
      setTitle(newNote.getTitle());
      setContent(newNote.getContent());
      setTags(newNote.getTags());
    }

    // Save every 2 seconds
    const interval = setInterval(() => {
      if (newNote) {
        const bytes = newNote.save();
        localStorage.setItem('note', JSON.stringify(Array.from(bytes)));
      }
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  const handleTitleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setTitle(value);
    note?.setTitle(value);
  };

  const handleContentChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    const cursorPos = e.target.selectionStart;

    // Calculate diff and apply to CRDT
    if (content.length < value.length) {
      // Insertion
      const inserted = value.substring(cursorPos - 1, cursorPos);
      note?.editContent(cursorPos - 1, inserted);
    } else if (content.length > value.length) {
      // Deletion
      note?.deleteContent(cursorPos, 1);
    }

    setContent(value);
  };

  const handleAddTag = () => {
    if (newTag.trim()) {
      note?.addTag(newTag.trim());
      setTags([...tags, newTag.trim()]);
      setNewTag('');
    }
  };

  const handleRemoveTag = (tag: string) => {
    note?.removeTag(tag);
    setTags(tags.filter(t => t !== tag));
  };

  return (
    <div style={{ maxWidth: '800px', margin: '0 auto', padding: '20px' }}>
      <h1>Local-First Notes</h1>

      <div style={{ marginBottom: '20px' }}>
        <input
          type="text"
          value={title}
          onChange={handleTitleChange}
          placeholder="Note title..."
          style={{
            width: '100%',
            fontSize: '24px',
            padding: '10px',
            border: '1px solid #ddd',
            borderRadius: '4px',
          }}
        />
      </div>

      <div style={{ marginBottom: '20px' }}>
        <textarea
          value={content}
          onChange={handleContentChange}
          placeholder="Start typing..."
          rows={15}
          style={{
            width: '100%',
            fontSize: '16px',
            padding: '10px',
            border: '1px solid #ddd',
            borderRadius: '4px',
            fontFamily: 'monospace',
          }}
        />
      </div>

      <div style={{ marginBottom: '20px' }}>
        <h3>Tags</h3>
        <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
          {tags.map(tag => (
            <span
              key={tag}
              style={{
                background: '#007bff',
                color: 'white',
                padding: '5px 10px',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
              onClick={() => handleRemoveTag(tag)}
            >
              {tag} ×
            </span>
          ))}
        </div>
        <div style={{ display: 'flex', gap: '10px' }}>
          <input
            type="text"
            value={newTag}
            onChange={e => setNewTag(e.target.value)}
            onKeyPress={e => e.key === 'Enter' && handleAddTag()}
            placeholder="Add tag..."
            style={{
              flex: 1,
              padding: '8px',
              border: '1px solid #ddd',
              borderRadius: '4px',
            }}
          />
          <button
            onClick={handleAddTag}
            style={{
              padding: '8px 16px',
              background: '#28a745',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Add
          </button>
        </div>
      </div>

      <div style={{
        padding: '10px',
        background: '#f0f0f0',
        borderRadius: '4px',
        fontSize: '12px',
      }}>
        <strong>Status:</strong> Working offline
        (auto-saved to localStorage)
      </div>
    </div>
  );
}

export default App;
```

## Step 7: Run the App

```bash
cd web
npm install
npm run dev
```

Visit `http://localhost:5173` and start typing!

### Test Offline Operation

1. Open the app in your browser
2. Type some text in the note
3. Disconnect from the internet (or use DevTools offline mode)
4. Continue editing - it still works!
5. Close the tab and reopen - your changes persist

## Step 8: Add P2P Sync (Optional)

To sync between devices, add Iroh P2P networking:

```bash
cargo add iroh-net
```

Update `src/lib.rs`:

```rust
use iroh_net::{MagicEndpoint, NodeAddr};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NoteSync {
    endpoint: MagicEndpoint,
    note: Arc<RwLock<Automerge>>,
}

impl NoteSync {
    pub async fn new(note: Automerge) -> anyhow::Result<Self> {
        let endpoint = MagicEndpoint::builder()
            .discovery_n0()
            .relay_mode(RelayMode::Default)
            .bind(0)
            .await?;

        Ok(Self {
            endpoint,
            note: Arc::new(RwLock::new(note)),
        })
    }

    pub async fn broadcast_changes(&self) -> anyhow::Result<()> {
        let note = self.note.read().await;
        let bytes = note.save();

        // Send to all connected peers
        for peer in self.endpoint.remote_info_iter() {
            self.endpoint.connect(peer.node_addr.clone(), "note-sync")
                .await?
                .send(&bytes)
                .await?;
        }

        Ok(())
    }

    pub async fn receive_changes(&self) -> anyhow::Result<()> {
        while let Some(msg) = self.endpoint.accept().await {
            let mut recv = msg.await?;
            let bytes = recv.read_to_end(1024 * 1024).await?;

            let remote_doc = Automerge::load(&bytes)?;
            let mut note = self.note.write().await;
            note.merge(&remote_doc)?;
        }

        Ok(())
    }
}
```

Now your notes sync automatically when devices connect!

## Step 9: Test Concurrent Editing

To see CRDTs in action:

1. Open the app in two browser tabs (Tab A and Tab B)
2. **Tab A**: Type "Hello" in the content
3. **Tab B**: Type "World" at the beginning
4. Both tabs merge: "WorldHello" (deterministic order)

### Understanding the Merge

```
Initial state: ""

Tab A (offline):
  - editContent(0, "Hello")
  - local state: "Hello"

Tab B (offline, concurrent):
  - editContent(0, "World")
  - local state: "World"

After sync (Peritext CRDT merge):
  - Both converge to: "WorldHello"
  - Character-level CRDT ensures no data loss
```

## What You've Learned

✅ **DOL Schema Design**: Define data models with CRDT annotations
✅ **CRDT Strategy Selection**: Choose the right strategy for each field
✅ **Code Generation**: Compile DOL to Rust/WASM
✅ **WASM Integration**: Use generated WASM in web apps
✅ **Offline Operation**: Apps work without network
✅ **Automatic Conflict Resolution**: CRDTs handle concurrent edits
✅ **P2P Sync**: Optional networking with Iroh

## Next Steps

### Extend Your App

Add more features:

```dol
gen note {
  // Existing fields...

  @crdt(pn_counter, min_value=0)
  has view_count: i64

  @crdt(rga)
  has comments: Vec<Comment>

  @crdt(or_set)
  has attachments: Set<Attachment>
}

gen comment {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has author: String

  @crdt(peritext)
  has text: String

  @crdt(lww)
  has timestamp: i64
}
```

### Deploy Your App

**As PWA (Progressive Web App)**:
```bash
npm run build
npx serve -s dist
```

**As Desktop App (Tauri)**:
```bash
cargo install tauri-cli
cargo tauri dev
cargo tauri build
```

### Add Features

- **Search**: Full-text search using local index
- **Encryption**: E2E encryption with did:key
- **Permissions**: UCAN-based access control
- **Backups**: Export/import notes as JSON
- **Themes**: Dark mode support

## Troubleshooting

### WASM module too large

**Solution**: Enable optimizations in `Cargo.toml`:

```toml
[profile.release]
opt-level = 'z'
lto = true
codegen-units = 1
```

Then run:
```bash
wasm-opt -Oz input.wasm -o output.wasm
```

### Changes don't persist

**Solution**: Check localStorage quota and implement fallback:

```typescript
try {
  localStorage.setItem('note', data);
} catch (e) {
  if (e.name === 'QuotaExceededError') {
    // Implement IndexedDB fallback
    await indexedDB.save('note', data);
  }
}
```

### Sync not working

**Solution**: Check firewall and relay server connectivity:

```bash
# Test relay server
curl https://relay.univrs.io:4433/health

# Check NAT traversal
dol test p2p-connectivity
```

## Further Reading

- [Core Concepts](./03-core-concepts.md) - Deep dive into DOL and CRDTs
- [CRDT Guide](../crdt-guide/00-overview.md) - All CRDT strategies explained
- [P2P Networking](../p2p-networking/00-overview.md) - Advanced sync patterns
- [Workspace Example](/apps/workspace/README.md) - Full-featured reference app

## Complete Code

The complete source code for this tutorial is available at:
- [GitHub: dol-examples/note-app](https://github.com/univrs/dol-examples/tree/main/note-app)

---

**Next**: [Core Concepts →](./03-core-concepts.md)
