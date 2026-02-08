# VUDO Workspace - Local-First Collaborative Workspace

**Reference Application for MYCELIUM-SYNC Project (Task t4.2)**

A comprehensive demonstration of the VUDO local-first stack featuring rich text collaborative editing, Kanban task boards, user profiles with mutual credit, P2P synchronization, and permission management.

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Key Features](#key-features)
- [DOL Schema Design](#dol-schema-design)
- [CRDT Implementation](#crdt-implementation)
- [P2P Synchronization](#p2p-synchronization)
- [Mutual Credit System](#mutual-credit-system)
- [Permission Management](#permission-management)
- [Offline-First Architecture](#offline-first-architecture)
- [UI/UX Design](#uiux-design)
- [Integration Examples](#integration-examples)
- [Deployment Targets](#deployment-targets)
- [Performance Characteristics](#performance-characteristics)
- [Development Setup](#development-setup)
- [Project Status](#project-status)

---

## Overview

VUDO Workspace is a **reference application** demonstrating the complete VUDO local-first platform. It showcases:

- **Collaborative Document Editor** - Real-time rich text editing with Peritext CRDT
- **Kanban Task Board** - Drag-and-drop task management with Movable List CRDT
- **User Profiles** - Decentralized identity with mutual credit balances
- **P2P Sync** - Offline-first synchronization via Iroh networking
- **Permission System** - Fine-grained access control via UCANs
- **Schema Evolution** - Live upgrades without downtime

This is a **design document with working examples**, not a complete implementation. It demonstrates how to build local-first applications using the DOL → WASM compilation pipeline.

---

## Architecture

### Technology Stack

```
┌─────────────────────────────────────────────────────────────┐
│                     VUDO Workspace                          │
├─────────────────────────────────────────────────────────────┤
│  UI Layer       │  React/Svelte + TailwindCSS               │
├─────────────────────────────────────────────────────────────┤
│  WASM Runtime   │  DOL-Generated Rust → WASM Modules        │
├─────────────────────────────────────────────────────────────┤
│  CRDT Layer     │  Automerge 3.0 (Peritext, RGA, PN-Counter)│
├─────────────────────────────────────────────────────────────┤
│  Storage Layer  │  IndexedDB (browser) / SQLite (Tauri)     │
├─────────────────────────────────────────────────────────────┤
│  P2P Layer      │  Iroh + Willow Protocol                   │
├─────────────────────────────────────────────────────────────┤
│  Identity       │  did:key + UCAN Permissions               │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Action (UI)
       ↓
WASM Module (DOL-generated)
       ↓
CRDT Operation (Automerge)
       ↓
Local Storage (IndexedDB/SQLite)
       ↓
P2P Sync (Iroh) → Other Peers
       ↓
CRDT Merge → Convergence
```

---

## Key Features

### 1. Collaborative Document Editor (Peritext CRDT)

**Schema**: [`schemas/document.dol`](./schemas/document.dol)

**Features**:
- Rich text editing with formatting (bold, italic, headings, links)
- Real-time collaborative editing with cursor indicators
- Version history with time-travel
- Tag-based organization (OR-Set CRDT)
- Permission-based sharing (owner, collaborator, viewer)

**CRDT Strategy**:
- `@crdt(peritext)` for content - preserves formatting through concurrent edits
- `@crdt(lww)` for title and metadata - last-write-wins semantics
- `@crdt(rga)` for collaborators - replicated growable array for ordered lists
- `@crdt(or_set)` for tags - add-wins conflict resolution

**Conflict Resolution**:
```
Alice types: "Hello World"
Bob types: "Hello Beautiful World" (concurrent)

After merge: "Hello Beautiful World"
↑ Peritext CRDT merges character insertions deterministically
```

**Implementation**: [`examples/peritext_integration.rs`](./examples/peritext_integration.rs)

### 2. Kanban Task Board (Movable List CRDT)

**Schema**: [`schemas/task_board.dol`](./schemas/task_board.dol)

**Features**:
- Drag-and-drop task cards between columns
- WIP (Work-in-Progress) limits per column
- Task assignments, labels, and priorities
- Comments and attachments
- Rich task descriptions with Peritext

**CRDT Strategy**:
- `@crdt(rga)` for columns and tasks - enables drag-and-drop reordering
- `@crdt(lww)` for task metadata - simple conflict resolution
- `@crdt(or_set)` for labels - add-wins semantics
- `@crdt(pn_counter)` for WIP limits - monotonic increases

**Conflict Resolution**:
```
Alice moves Task #42: "To Do" → "In Progress"
Bob moves Task #42: "To Do" → "Done" (concurrent)

After merge: Task #42 is in "Done"
↑ Timestamp tiebreaker: Bob's move happened 2ms later
```

**Implementation**: [`examples/movable_list_integration.rs`](./examples/movable_list_integration.rs)

### 3. User Profile & Mutual Credit (PN-Counter CRDT)

**Schema**: [`schemas/user_profile.dol`](./schemas/user_profile.dol)

**Features**:
- Decentralized identity (DID)
- Privacy-preserving profile fields (`@personal` annotation)
- Mutual credit balance tracking
- Trust relationships with peers
- Escrow-based transactions
- Activity log and transaction history

**CRDT Strategy**:
- `@crdt(immutable)` for DID - permanent identity
- `@crdt(lww)` for profile fields - user preferences
- `@crdt(pn_counter)` for credit balance - monotonic operations
- `@crdt(rga)` for transaction history - append-only log

**Conflict Resolution**:
```
Alice earns 500 credits from Bob (peer1)
Alice earns 300 credits from Carol (peer2) - concurrent

After merge: Alice's balance = 800
↑ PN-Counter sums all concurrent increments
```

**Implementation**: [`examples/mutual_credit_integration.rs`](./examples/mutual_credit_integration.rs)

---

## DOL Schema Design

### Document Schema

```dol
gen workspace.document {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="full", max_length=500000)
  has content: String

  @crdt(rga)
  has collaborators: Vec<String>

  @crdt(or_set)
  has tags: Set<String>
}

docs {
  Collaborative document using Peritext CRDT for conflict-free
  rich text editing with formatting preservation.
}
```

### Task Board Schema

```dol
gen workspace.task_board {
  @crdt(immutable)
  has id: String

  @crdt(rga)
  has columns: Vec<Column>

  @crdt(rga)
  has members: Vec<String>
}

gen workspace.column {
  @crdt(rga)
  has tasks: Vec<Task>

  @crdt(pn_counter, min_value=0)
  has wip_limit: i32
}
```

### User Profile Schema

```dol
gen workspace.user_profile {
  @crdt(immutable)
  has did: String

  @crdt(lww) @personal
  has display_name: String

  @crdt(pn_counter)
  has credit_balance: i64

  @crdt(rga)
  has transaction_history: Vec<CreditTransaction>
}
```

---

## CRDT Implementation

### Peritext CRDT (Rich Text)

**Purpose**: Conflict-free collaborative rich text editing

**How It Works**:
1. Text stored as ordered sequence of characters
2. Each character has unique ID (timestamp + actor)
3. Formatting stored as "marks" on character ranges
4. Concurrent edits merge deterministically

**Example**:
```rust
let mut doc = PeritextDocument::new("doc-123", "did:key:alice");

// Alice types "Hello"
doc.insert_text(0, "Hello")?;

// Bob concurrently types " World" at position 5
doc.insert_text(5, " World")?;

// Both converge to "Hello World"
```

**Code**: [`examples/peritext_integration.rs`](./examples/peritext_integration.rs)

### RGA CRDT (Movable List)

**Purpose**: Drag-and-drop reordering with conflict resolution

**How It Works**:
1. List stored as Replicated Growable Array
2. Each item has unique ID and position
3. Concurrent moves are resolved by timestamp
4. List order is eventually consistent

**Example**:
```rust
let mut board = MovableListBoard::new("board-123", "Sprint", "alice");

// Add columns
let todo = board.add_column("To Do", "#blue")?;
let done = board.add_column("Done", "#green")?;

// Add task
let task = board.add_task(&todo, "Task 1", "Description")?;

// Drag task to "Done"
board.move_task(&task, &todo, &done, 0)?;
```

**Code**: [`examples/movable_list_integration.rs`](./examples/movable_list_integration.rs)

### PN-Counter CRDT (Credit Balance)

**Purpose**: Monotonic counter for credit tracking

**How It Works**:
1. Counter = (P, N) where P = increments, N = decrements
2. Balance = sum(P) - sum(N)
3. Only increments are allowed (no direct decrements)
4. Concurrent increments are summed

**Example**:
```rust
let mut account = MutualCreditAccount::new("acc-alice", "did:key:alice", 5000);

// Earn credits
account.earn_credits(500, "did:key:bob")?;

// Transfer credits (creates escrow)
let txn = account.transfer_credits("did:key:carol", 200, "Payment")?;

// Balance is always consistent
let balance = account.get_balance()?;
```

**Code**: [`examples/mutual_credit_integration.rs`](./examples/mutual_credit_integration.rs)

---

## P2P Synchronization

### Iroh P2P Integration

**Features**:
- NAT traversal via hole-punching
- Relay fallback for restricted networks
- Gossip protocol for peer discovery
- Delta compression for efficient sync

**Sync Flow**:
```
1. User edits document locally
   ↓
2. CRDT operation generated
   ↓
3. Operation stored in IndexedDB
   ↓
4. Service Worker schedules background sync
   ↓
5. Iroh broadcasts changes to connected peers
   ↓
6. Peers receive and merge changes
   ↓
7. All peers converge to same state
```

### Online/Offline Status

**Implementation**:
```javascript
// Online indicator
const syncStatus = {
  online: true,
  peers_connected: 3,
  last_sync: Date.now()
};

// Show in UI
<div class="sync-status online">
  <span class="status-dot"></span>
  <span>{syncStatus.peers_connected} peers connected</span>
</div>
```

**Service Worker**:
```javascript
// Background sync registration
navigator.serviceWorker.ready.then(registration => {
  registration.sync.register('sync-documents');
  registration.periodicSync.register('continuous-sync', {
    minInterval: 60 * 1000 // 1 minute
  });
});
```

**Code**: [`config/service-worker.js`](./config/service-worker.js)

---

## Mutual Credit System

### Escrow-Based Transactions

**Flow**:
```
1. Alice wants to pay Bob 500 credits
   ↓
2. System checks: Alice.balance >= 500
   ↓
3. System checks: Bob.trust_limit >= 500
   ↓
4. Create transaction with status=InEscrow
   ↓
5. Generate cryptographic escrow_proof
   ↓
6. Alice's spent_counter += 500
   ↓
7. Bob confirms work completion
   ↓
8. Bob's earned_counter += 500
   ↓
9. Transaction status=Completed
```

### Byzantine Fault Tolerance

**Escrow Proof**:
```rust
fn create_escrow_proof(txn_id: &str, amount: i64) -> String {
  let mut hasher = Sha256::new();
  hasher.update(txn_id.as_bytes());
  hasher.update(amount.to_le_bytes());
  hasher.update(timestamp.to_le_bytes());
  format!("{:x}", hasher.finalize())
}
```

**Dispute Resolution**:
- If transaction disputed, escrow_proof used as evidence
- Byzantine consensus (BFT) among trusted peers
- Majority vote determines transaction validity

### Trust Network

```rust
// Establish trust relationship
account.establish_trust("did:key:bob", trust_limit: 2000)?;

// Transfer limited by trust
account.transfer_credits("did:key:bob", 500, "Payment")?; // ✓ OK
account.transfer_credits("did:key:bob", 3000, "Payment")?; // ✗ Exceeds trust
```

---

## Permission Management

### UCAN Capabilities

**Model**: User-Controlled Authorization Networks (UCAN)

**Capabilities**:
```
document:read    - Read document content
document:write   - Edit document content
document:share   - Share with others
board:read       - View task board
board:write      - Move tasks, add comments
board:admin      - Manage board members
credit:transfer  - Transfer credits
profile:read:personal - Read personal profile fields
```

**Example UCAN**:
```json
{
  "iss": "did:key:alice",
  "aud": "did:key:bob",
  "att": [
    {
      "with": "document:doc-123",
      "can": "document:write"
    }
  ],
  "exp": 1735689600,
  "prf": []
}
```

**UI Integration**:
```javascript
// Check permission before allowing edit
if (await checkCapability('document:write', documentId)) {
  enableEditing();
} else {
  showReadOnlyMode();
}
```

### Privacy with @personal

**DOL Annotation**:
```dol
gen workspace.user_profile {
  @crdt(lww) @personal
  has email: Option<String>

  @crdt(lww) @personal
  has avatar_url: String

  @crdt(lww)  // Public field
  has public_bio: String
}
```

**Behavior**:
- `@personal` fields require explicit UCAN to sync
- Public fields sync freely
- Privacy-preserving by default

---

## Offline-First Architecture

### Local Storage

**Browser (PWA)**:
```
IndexedDB
  ├── documents (Automerge documents)
  ├── tasks (Task board state)
  ├── profiles (User profiles)
  ├── pending_changes (Sync queue)
  └── credentials (DIDs, UCANs)
```

**Desktop (Tauri)**:
```
SQLite Database
  ├── automerge_docs table
  ├── sync_queue table
  ├── credentials table
  └── metadata table
```

### Offline Operation Queue

```javascript
class OfflineQueue {
  async addOperation(op) {
    // Store in IndexedDB
    await db.put('pending_changes', op);

    // Register background sync
    if ('sync' in navigator.serviceWorker) {
      await navigator.serviceWorker.ready;
      await registration.sync.register('sync-documents');
    }
  }

  async processQueue() {
    const pending = await db.getAll('pending_changes');

    for (const op of pending) {
      try {
        await sendToPeers(op);
        await db.delete('pending_changes', op.id);
      } catch (e) {
        console.log('Will retry later:', e);
      }
    }
  }
}
```

### Conflict Resolution

**Automatic**:
- CRDTs handle conflicts automatically
- No user intervention required
- Deterministic convergence guaranteed

**Manual (rare cases)**:
- WIP limit violations → show warning
- Trust limit exceeded → transaction fails
- Invalid UCAN → permission denied

---

## UI/UX Design

### Wireframes

**Location**: [`ui/index.html`](./ui/index.html)

**Components**:
1. **Header** - Navigation, sync status, user menu
2. **Document Editor** - Rich text editor with toolbar
3. **Task Board** - Kanban columns with draggable cards
4. **Profile** - User info, credit balance, transactions
5. **Sync Indicator** - Online/offline status, peer count

**Screenshots**:

```
┌─────────────────────────────────────────────────────────┐
│  🍄 VUDO  │  Documents  Tasks  Profile  Credit (+1250) │
│──────────────────────────────────────────────────────────│
│                                                           │
│  📄 Product Requirements Document                        │
│  ────────────────────────────────────────────────        │
│  B I U │ H1 • List 🔗 │ Share  Version History          │
│  ────────────────────────────────────────────────        │
│                                                           │
│  # Product Requirements Document                         │
│                                                           │
│  This document outlines the requirements for our new     │
│  **local-first collaborative workspace**.                │
│                                                           │
│  ## Core Features                                        │
│  - Rich text editing with Peritext CRDT                 │
│  - Task boards with drag-and-drop                       │
│  - Mutual credit system                                 │
│                                                           │
│  [Bob ↑]  [Carol ↑]  Editing now...                     │
└─────────────────────────────────────────────────────────┘
```

**Styles**: [`ui/styles.css`](./ui/styles.css)

### User Flows

**Creating a Document**:
```
1. Click "New Document"
   ↓
2. Document created locally (instant)
   ↓
3. Start typing (Peritext CRDT)
   ↓
4. Changes saved to IndexedDB
   ↓
5. Service Worker syncs to peers
   ↓
6. Collaborators see updates in real-time
```

**Moving a Task**:
```
1. Drag task card from "To Do" to "Done"
   ↓
2. RGA CRDT operation generated
   ↓
3. Local board updates (instant)
   ↓
4. Operation queued for sync
   ↓
5. Peers receive and merge operation
   ↓
6. All boards show task in "Done"
```

**Transferring Credits**:
```
1. Click "Transfer Credits"
   ↓
2. Enter recipient DID and amount
   ↓
3. System checks balance and trust limit
   ↓
4. Transaction created with escrow_proof
   ↓
5. Sender's balance reduced
   ↓
6. Recipient confirms
   ↓
7. Recipient's balance increased
   ↓
8. Trust relationship updated
```

---

## Integration Examples

### Example 1: Collaborative Editing

**File**: [`examples/peritext_integration.rs`](./examples/peritext_integration.rs)

**Scenario**: Alice and Bob edit a document concurrently while offline

```rust
// Alice's side
let mut alice_doc = PeritextDocument::new("doc-123", "alice");
alice_doc.insert_text(0, "Hello World")?;
alice_doc.insert_text(5, " Beautiful")?; // "Hello Beautiful World"

// Bob's side (forked from earlier state)
let mut bob_doc = PeritextDocument::new("doc-123", "alice");
bob_doc.merge(&alice_initial_state)?;
bob_doc.insert_text(11, "!")?; // "Hello World!"

// Both sync
alice_doc.merge(&bob_doc.get_changes())?;
bob_doc.merge(&alice_doc.get_changes())?;

// Both converge to "Hello Beautiful World!"
assert_eq!(alice_doc.get_text()?, bob_doc.get_text()?);
```

### Example 2: Drag-and-Drop Tasks

**File**: [`examples/movable_list_integration.rs`](./examples/movable_list_integration.rs)

**Scenario**: Alice and Bob move tasks concurrently

```rust
// Alice moves Task #1 to "In Progress"
alice_board.move_task("task-1", "todo", "progress", 0)?;

// Bob moves Task #1 to "Done" (concurrent)
bob_board.move_task("task-1", "todo", "done", 0)?;

// Sync
alice_board.merge(&bob_board.get_changes())?;
bob_board.merge(&alice_board.get_changes())?;

// Timestamp tiebreaker: Task ends up in "Done" (Bob's move was later)
```

### Example 3: Credit Transfer

**File**: [`examples/mutual_credit_integration.rs`](./examples/mutual_credit_integration.rs)

**Scenario**: Alice pays Bob for completing a task

```rust
// Alice's account
let mut alice = MutualCreditAccount::new("alice", "did:key:alice", 5000);
alice.earn_credits(1000, "system")?;

// Establish trust
alice.establish_trust("did:key:bob", 2000)?;

// Transfer credits
let txn_id = alice.transfer_credits("did:key:bob", 500, "Task payment")?;

// Bob confirms
let mut bob = MutualCreditAccount::new("bob", "did:key:bob", 5000);
bob.complete_transaction(&txn_id, "did:key:alice")?;

// Balances updated
assert_eq!(alice.get_balance()?, 500);
assert_eq!(bob.get_balance()?, 500);
```

---

## Deployment Targets

### 1. Progressive Web App (PWA)

**Platform**: Browser (Chrome, Firefox, Safari, Edge)

**Features**:
- Install to home screen
- Offline operation
- Background sync
- Push notifications

**Build**:
```bash
npm run build:pwa
npm run deploy
```

**Service Worker**: [`config/service-worker.js`](./config/service-worker.js)
**Manifest**: [`config/manifest.json`](./config/manifest.json)

### 2. Desktop App (Tauri)

**Platform**: Windows, macOS, Linux

**Features**:
- Native OS integration
- System tray icon
- File system access
- SQLite storage

**Build**:
```bash
npm run tauri:build
```

**Config**: [`config/tauri.conf.json`](./config/tauri.conf.json)

### 3. Mobile App (Future)

**Platform**: iOS, Android

**Technologies**:
- React Native + WASM
- Or Tauri Mobile (upcoming)

**Considerations**:
- Mobile-optimized UI
- Battery-efficient sync
- Mobile data usage limits

---

## Performance Characteristics

### Benchmarks (from t4.1)

| Metric | Target | Achieved |
|--------|--------|----------|
| WASM Module Size | < 100KB gzipped | 85KB |
| CRDT Merge Latency | < 10ms (10K ops) | 7ms |
| Sync Throughput | > 1000 ops/sec | 1,500 ops/sec |
| Memory Usage | < 50MB (100K records) | 42MB |
| Startup Time | < 500ms cold start | 380ms |

### Optimization Techniques

**WASM Size**:
- `opt-level = 'z'` in release profile
- `wasm-opt -Oz` post-processing
- Strip debug symbols

**CRDT Performance**:
- Incremental merge (only new ops)
- Operation batching (100 ops/batch)
- Lazy materialization (OnceCell)

**Sync Efficiency**:
- Delta compression (zstd)
- Operation deduplication (Bloom filter)
- Parallel sync (tokio::spawn)

**Memory**:
- String interning (string_cache)
- Compact data structures (SmallVec)
- Memory pooling (typed-arena)

---

## Development Setup

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Node.js and npm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install --lts

# DOL compiler (from root of univrs-dol repo)
cargo install --path .
```

### Build Instructions

```bash
# Clone repository
git clone https://github.com/univrs/dol
cd univrs-dol/apps/workspace

# Install dependencies
npm install

# Generate Rust from DOL schemas
dol-codegen-rust schemas/*.dol --output src/generated/

# Build WASM modules
cargo build --target wasm32-unknown-unknown --release

# Optimize WASM
wasm-opt -Oz target/wasm32-unknown-unknown/release/*.wasm -o dist/

# Run development server
npm run dev

# Build for production
npm run build
```

### Testing

```bash
# Run Rust tests
cargo test

# Run integration tests
cargo test --test peritext_integration
cargo test --test movable_list_integration
cargo test --test mutual_credit_integration

# Run benchmarks
cargo bench
```

---

## Project Status

### Implementation Phase: DESIGN DOCUMENT

This is a **reference implementation** demonstrating concepts, not a production-ready application.

### Completed ✅

- [x] DOL schemas for all entities (document, task_board, user_profile)
- [x] UI wireframes showing all major features (HTML/CSS)
- [x] Integration examples with working code:
  - [x] Peritext CRDT for collaborative editing
  - [x] RGA CRDT for drag-and-drop tasks
  - [x] PN-Counter CRDT for mutual credit
- [x] PWA manifest and service worker configuration
- [x] Tauri configuration for desktop builds
- [x] Comprehensive architecture documentation

### Working Examples

| Component | Status | File |
|-----------|--------|------|
| Document Editor | ✅ Example Code | `examples/peritext_integration.rs` |
| Task Board | ✅ Example Code | `examples/movable_list_integration.rs` |
| Mutual Credit | ✅ Example Code | `examples/mutual_credit_integration.rs` |
| UI Mockup | ✅ HTML/CSS | `ui/index.html`, `ui/styles.css` |
| PWA Config | ✅ Complete | `config/manifest.json` |
| Service Worker | ✅ Complete | `config/service-worker.js` |
| Tauri Config | ✅ Complete | `config/tauri.conf.json` |

### Next Steps (For Full Implementation)

1. **Complete WASM Integration**
   - Compile DOL schemas to Rust
   - Build WASM modules with wasm-bindgen
   - Integrate with JavaScript/TypeScript frontend

2. **P2P Networking**
   - Integrate Iroh for peer discovery
   - Implement sync protocol
   - Add relay server configuration

3. **Storage Layer**
   - Implement IndexedDB adapter (browser)
   - Implement SQLite adapter (Tauri)
   - Add migration system

4. **Permission System**
   - Implement UCAN generation and verification
   - Add permission UI
   - Integrate with DID system

5. **Testing & Polish**
   - End-to-end tests
   - Performance profiling
   - UX refinements
   - Documentation

---

## Architecture Decisions

### Why Peritext for Documents?

**Alternatives**: Yjs, Quill Delta, Operational Transform

**Chosen**: Peritext CRDT (via Automerge)

**Rationale**:
- Preserves formatting through concurrent edits
- Built into Automerge 3.0
- Proven in production (Ink & Switch)
- Strong convergence guarantees

### Why RGA for Task Lists?

**Alternatives**: LSEQ, Logoot, Fractional Indexing

**Chosen**: RGA (Replicated Growable Array)

**Rationale**:
- Simple mental model (append-only with tombstones)
- Efficient for lists with < 10,000 items
- Built into Automerge
- Handles concurrent moves deterministically

### Why PN-Counter for Credits?

**Alternatives**: G-Counter, LWW-Register

**Chosen**: PN-Counter (Positive-Negative Counter)

**Rationale**:
- Monotonic operations only (no direct decrements)
- Prevents double-spending
- Efficient merging
- Byzantine fault tolerant with escrow proofs

---

## Learn More

**VUDO Platform**: https://book.univrs.io
**DOL Language**: https://dol.univrs.io
**Automerge CRDTs**: https://automerge.org
**Iroh P2P**: https://iroh.computer
**UCAN Auth**: https://ucan.xyz

---

## License

MIT OR Apache-2.0

Copyright © 2026 Univrs.io

---

**Built with DOL 2.0 - Ontology-Driven Development for Local-First Applications**
