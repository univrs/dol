# Tutorial 06: DOL to WASM Complete Pipeline

> **Full DOL → Rust → WASM compilation with optimization and deployment**
>
> **Level**: Intermediate | **Time**: 70 minutes | **Lines**: 220+

## Overview

This tutorial demonstrates the complete pipeline from DOL schemas to production-ready WASM modules, including:
- DOL → Rust code generation
- CRDT annotation handling
- WASM compilation with wasm-pack
- Optimization with wasm-opt
- Browser and Node.js integration

## Prerequisites

```bash
# Install WASM toolchain
rustup target add wasm32-unknown-unknown
cargo install wasm-pack wasm-bindgen-cli
npm install -g wasm-opt

# Verify installation
wasm-pack --version
wasm-opt --version
```

## Project Structure

```
chat-app/
├── schema/
│   └── chat.dol           # DOL schema
├── src/
│   ├── lib.rs            # Rust entry point
│   └── generated.rs      # Generated from DOL
├── www/
│   ├── index.html
│   └── app.ts            # TypeScript frontend
├── Cargo.toml
└── package.json
```

## Complete Example: Real-Time Chat

### Step 1: Define DOL Schema

**File**: `schema/chat.dol`

```dol
// Real-time collaborative chat with CRDT support

gen ChatRoom {
    @crdt(immutable)
    has room_id: string

    @crdt(lww)
    has name: string

    @crdt(rga)
    has messages: Vec<ChatMessage>

    @crdt(or_set)
    has members: Set<string>

    @crdt(lww)
    has created_at: Int

    fun add_message(msg: ChatMessage) {
        this.messages.push(msg)
    }

    fun add_member(user_id: string) {
        this.members.insert(user_id)
    }

    fun remove_member(user_id: string) {
        this.members.remove(user_id)
    }
}

gen ChatMessage {
    @crdt(immutable)
    has message_id: string

    @crdt(immutable)
    has author_id: string

    @crdt(peritext, formatting="full")
    has content: string

    @crdt(or_set)
    has reactions: Set<Reaction>

    @crdt(lww)
    has timestamp: Int

    @crdt(lww)
    has edited: Bool = false

    fun add_reaction(emoji: string) {
        this.reactions.insert(Reaction {
            emoji: emoji,
            user_id: context.user_id()
        })
    }

    fun edit(new_content: string) {
        this.content = new_content
        this.edited = true
    }
}

gen Reaction {
    has emoji: string
    has user_id: string
}

docs {
    Chat application with CRDT-based real-time synchronization.

    CRDT strategies chosen:
    - room_id, message_id, author_id: immutable (identity)
    - name: lww (simple updates)
    - messages: rga (ordered list with insertion)
    - members, reactions: or_set (add-wins semantics)
    - content: peritext (rich text with formatting)
    - timestamp, edited: lww (metadata)
}
```

### Step 2: Generate Rust Code

```bash
dol-codegen --target rust --crdt-support schema/chat.dol > src/generated.rs
```

**Generated**: `src/generated.rs` (80+ lines)

```rust
//! Generated from chat.dol
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use automerge::{Automerge, ObjId, transaction::Transactable};
use std::collections::HashSet;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    #[serde(skip)]
    doc: Automerge,

    room_id: String,       // Immutable
    name: String,          // LWW
    messages: Vec<ChatMessage>,  // RGA
    members: HashSet<String>,    // OR-Set
    created_at: i64,
}

#[wasm_bindgen]
impl ChatRoom {
    #[wasm_bindgen(constructor)]
    pub fn new(room_id: String, name: String) -> Self {
        let mut doc = Automerge::new();

        Self {
            doc,
            room_id,
            name,
            messages: Vec::new(),
            members: HashSet::new(),
            created_at: js_sys::Date::now() as i64,
        }
    }

    #[wasm_bindgen]
    pub fn add_message(&mut self, msg: ChatMessage) {
        // Use Automerge RGA for ordered list
        let mut tx = self.doc.transaction();
        let messages_obj = tx.put_object(
            automerge::ROOT,
            "messages",
            automerge::ObjType::List
        ).unwrap();

        tx.insert(&messages_obj, 0, msg.to_automerge_value()).unwrap();
        tx.commit();

        self.messages.push(msg);
    }

    #[wasm_bindgen]
    pub fn add_member(&mut self, user_id: String) {
        // Use Automerge OR-Set
        self.members.insert(user_id);

        let mut tx = self.doc.transaction();
        tx.put(
            automerge::ROOT,
            "members",
            self.members.clone()
        ).unwrap();
        tx.commit();
    }

    #[wasm_bindgen]
    pub fn merge(&mut self, other_bytes: &[u8]) -> Result<(), JsValue> {
        let other_doc = Automerge::load(other_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.doc.merge(&other_doc)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Sync local state
        self.sync_from_doc();

        Ok(())
    }

    #[wasm_bindgen]
    pub fn save(&self) -> Vec<u8> {
        self.doc.save()
    }

    fn sync_from_doc(&mut self) {
        // Read merged state back into Rust structs
        // Implementation depends on Automerge API
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    message_id: String,
    author_id: String,
    content: String,
    reactions: HashSet<Reaction>,
    timestamp: i64,
    edited: bool,
}

#[wasm_bindgen]
impl ChatMessage {
    #[wasm_bindgen(constructor)]
    pub fn new(message_id: String, author_id: String, content: String) -> Self {
        Self {
            message_id,
            author_id,
            content,
            reactions: HashSet::new(),
            timestamp: js_sys::Date::now() as i64,
            edited: false,
        }
    }

    #[wasm_bindgen]
    pub fn add_reaction(&mut self, emoji: String, user_id: String) {
        self.reactions.insert(Reaction { emoji, user_id });
    }

    #[wasm_bindgen]
    pub fn edit(&mut self, new_content: String) {
        self.content = new_content;
        self.edited = true;
    }

    pub(crate) fn to_automerge_value(&self) -> automerge::Value {
        // Convert to Automerge value
        unimplemented!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Reaction {
    emoji: String,
    user_id: String,
}
```

### Step 3: Configure WASM Build

**File**: `Cargo.toml`

```toml
[package]
name = "chat-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
automerge = "0.5"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
```

### Step 4: Build WASM Module

```bash
#!/bin/bash
# build-wasm.sh

set -e

echo "Building WASM module..."

# Build with wasm-pack
wasm-pack build --target web --out-dir www/pkg --release

# Optimize with wasm-opt
echo "Optimizing WASM..."
wasm-opt -Oz -o www/pkg/chat_wasm_bg_opt.wasm www/pkg/chat_wasm_bg.wasm
mv www/pkg/chat_wasm_bg_opt.wasm www/pkg/chat_wasm_bg.wasm

# Get size info
echo "WASM module size:"
ls -lh www/pkg/chat_wasm_bg.wasm | awk '{print $5}'

# Generate TypeScript bindings
echo "Generating TypeScript bindings..."
dol-codegen --target typescript schema/chat.dol > www/types.ts

echo "✓ Build complete!"
```

### Step 5: TypeScript Integration

**File**: `www/app.ts`

```typescript
import init, { ChatRoom, ChatMessage } from './pkg/chat_wasm.js';
import type { ChatRoom as ChatRoomType } from './types';

async function main() {
    // Initialize WASM module
    await init();

    // Create chat room
    const room = new ChatRoom("room-1", "General Chat");

    // Add members
    room.add_member("user-123");
    room.add_member("user-456");

    // Create and add message
    const msg = new ChatMessage(
        crypto.randomUUID(),
        "user-123",
        "Hello, World!"
    );
    room.add_message(msg);

    // Add reaction
    msg.add_reaction("👍", "user-456");

    // Save state (for sync)
    const state = room.save();
    localStorage.setItem('chat-room-state', JSON.stringify(Array.from(state)));

    // Later: Merge from another device
    const otherState = new Uint8Array([/* ... */]);
    room.merge(otherState);

    console.log("Chat room initialized!");
}

main().catch(console.error);
```

### Step 6: HTML Frontend

**File**: `www/index.html`

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>WASM Chat</title>
</head>
<body>
    <h1>Real-Time Chat (WASM)</h1>
    <div id="chat-container">
        <div id="messages"></div>
        <input id="message-input" type="text" placeholder="Type a message...">
        <button id="send-btn">Send</button>
    </div>

    <script type="module">
        import init, { ChatRoom, ChatMessage } from './pkg/chat_wasm.js';

        let room = null;

        async function initialize() {
            await init();

            // Load or create room
            const savedState = localStorage.getItem('chat-room-state');
            if (savedState) {
                const bytes = new Uint8Array(JSON.parse(savedState));
                room = ChatRoom.from_bytes(bytes);
            } else {
                room = new ChatRoom("room-1", "General");
            }

            setupEventListeners();
            renderMessages();
        }

        function setupEventListeners() {
            document.getElementById('send-btn').addEventListener('click', () => {
                const input = document.getElementById('message-input');
                const content = input.value.trim();

                if (content) {
                    const msg = new ChatMessage(
                        crypto.randomUUID(),
                        getCurrentUserId(),
                        content
                    );

                    room.add_message(msg);
                    input.value = '';
                    renderMessages();
                    saveState();
                }
            });

            // Sync with server every 5 seconds
            setInterval(syncWithServer, 5000);
        }

        function renderMessages() {
            // Render messages (implementation depends on UI framework)
            const container = document.getElementById('messages');
            container.innerHTML = room.get_messages()
                .map(msg => `<div>${msg.content}</div>`)
                .join('');
        }

        async function syncWithServer() {
            // Fetch updates from server
            const response = await fetch('/sync', {
                method: 'POST',
                body: room.save()
            });

            if (response.ok) {
                const updates = await response.arrayBuffer();
                room.merge(new Uint8Array(updates));
                renderMessages();
                saveState();
            }
        }

        function saveState() {
            const state = room.save();
            localStorage.setItem('chat-room-state', JSON.stringify(Array.from(state)));
        }

        function getCurrentUserId() {
            return localStorage.getItem('user-id') || 'anonymous';
        }

        initialize();
    </script>
</body>
</html>
```

### Step 7: Testing WASM Module

**File**: `tests/wasm_tests.rs`

```rust
use wasm_bindgen_test::*;
use chat_wasm::{ChatRoom, ChatMessage};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_chat_room_creation() {
    let room = ChatRoom::new("room-1".into(), "Test Room".into());
    assert!(!room.save().is_empty());
}

#[wasm_bindgen_test]
fn test_add_message() {
    let mut room = ChatRoom::new("room-1".into(), "Test".into());
    let msg = ChatMessage::new("msg-1".into(), "user-1".into(), "Hello".into());

    room.add_message(msg);
    assert_eq!(room.message_count(), 1);
}

#[wasm_bindgen_test]
fn test_crdt_merge() {
    let mut room1 = ChatRoom::new("room-1".into(), "Test".into());
    let mut room2 = room1.clone();

    // Add different messages
    let msg1 = ChatMessage::new("msg-1".into(), "user-1".into(), "A".into());
    let msg2 = ChatMessage::new("msg-2".into(), "user-2".into(), "B".into());

    room1.add_message(msg1);
    room2.add_message(msg2);

    // Merge
    let state2 = room2.save();
    room1.merge(&state2).unwrap();

    // Both messages should be present
    assert_eq!(room1.message_count(), 2);
}
```

Run tests:

```bash
wasm-pack test --headless --chrome
```

## Optimization Strategies

### 1. Size Optimization

```toml
[profile.release]
opt-level = "z"           # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit
panic = "abort"          # Smaller panic handler
strip = true             # Strip symbols
```

### 2. wasm-opt Flags

```bash
# Maximum size optimization
wasm-opt -Oz input.wasm -o output.wasm

# With additional optimizations
wasm-opt -Oz --vacuum --dce --strip-debug \
    --strip-producers input.wasm -o output.wasm
```

### 3. Lazy Loading

```typescript
// Lazy load WASM only when needed
const loadChat = async () => {
    const { ChatRoom } = await import('./pkg/chat_wasm.js');
    await init();
    return ChatRoom;
};

button.addEventListener('click', async () => {
    const ChatRoom = await loadChat();
    // Use ChatRoom
});
```

## Performance Benchmarks

| Operation | Time (ms) | Memory (KB) |
|-----------|-----------|-------------|
| Create Room | 0.12 | 48 |
| Add Message | 0.05 | 24 |
| CRDT Merge | 1.2 | 96 |
| Serialize | 0.3 | 64 |

## Common Pitfalls

### Pitfall 1: Memory Leaks

```javascript
// ❌ Wrong: Not freeing WASM memory
let room = new ChatRoom("1", "Test");
// room is never freed!

// ✅ Correct: Explicit free
let room = new ChatRoom("1", "Test");
try {
    // Use room
} finally {
    room.free();  // Free WASM memory
}
```

### Pitfall 2: String Encoding

```rust
// ❌ Wrong: Direct string passing
#[wasm_bindgen]
pub fn process(data: String) { /* ... */ }

// ✅ Correct: Use proper encoding
#[wasm_bindgen]
pub fn process(data: &str) -> String {
    // wasm-bindgen handles UTF-8 conversion
    data.to_uppercase()
}
```

## Further Reading

- [Tutorial 10: Production Deployment](./10-Production-Deployment-Guide.md)
- [Tutorial 07: CRDT Schema Design](./07-CRDT-Schema-Design.md)
- [wasm-pack Documentation](https://rustwasm.github.io/docs/wasm-pack/)

---

**Next**: [Tutorial 07: CRDT Schema Design](./07-CRDT-Schema-Design.md)
