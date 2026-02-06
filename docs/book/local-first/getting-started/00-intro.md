# Introduction to Local-First Software

## What is Local-First?

Local-first software is a set of principles for software that prioritizes:

1. **Offline Operation**: Applications work without network connectivity
2. **Data Ownership**: Users control their own data
3. **Peer-to-Peer Sync**: No central coordination required
4. **Immediate Responsiveness**: No network latency for local operations
5. **Multi-Device Support**: Seamless sync across all your devices
6. **Privacy by Default**: Data stays local until you choose to share
7. **Long-Term Access**: Your data remains accessible even if servers shut down

## Why Local-First Matters

### Traditional Cloud Apps Have Problems

**Centralized Architecture**:
- Requires constant internet connection
- Data locked in vendor's servers
- Network latency affects every interaction
- Service shutdown = data loss
- Privacy concerns with third-party control

**Traditional Offline-First**:
- Complex synchronization logic
- Conflict resolution is hard to get right
- Eventual consistency is difficult to reason about
- Race conditions and bugs are common

### Local-First Solves These

**Immediate, Always Available**:
```
User Action → Local Update → Instant UI Response
              ↓ (background)
              Sync to Peers
```

No waiting for server round-trips. The app feels fast because it IS fast.

**True Data Ownership**:
- Your data lives on YOUR devices
- Sync directly between peers (no central server)
- Export and import freely
- Switch providers without lock-in

**Privacy by Design**:
- End-to-end encryption
- Selective sharing with specific peers
- GDPR-compliant cryptographic deletion
- No surveillance by default

## How VUDO Implements Local-First

### The Stack

```
┌─────────────────────────────────────────┐
│  Your Application (React/Svelte/etc)    │
├─────────────────────────────────────────┤
│  WASM Module (compiled from DOL)        │
│  ├─ CRDT Operations                     │
│  ├─ Constraint Validation               │
│  └─ Type-Safe APIs                      │
├─────────────────────────────────────────┤
│  VUDO Runtime                           │
│  ├─ State Engine (Automerge)            │
│  ├─ Storage (IndexedDB/SQLite)          │
│  └─ Sync Protocol (Iroh P2P)            │
├─────────────────────────────────────────┤
│  Platform Layer                         │
│  ├─ Browser (PWA)                       │
│  ├─ Desktop (Tauri)                     │
│  └─ Mobile (React Native)               │
└─────────────────────────────────────────┘
```

### Key Technologies

**DOL (Distributed Ontology Language)**:
- Define your data model as an ontology
- Annotate fields with CRDT strategies
- Compile to Rust/WASM for multi-platform execution

**CRDTs (Conflict-free Replicated Data Types)**:
- Mathematical guarantee of eventual convergence
- No manual conflict resolution needed
- 7 strategies for different data patterns

**Automerge**:
- Production-ready CRDT library
- Compact binary format
- Fast merge algorithms
- Proven at scale (used by Ink & Switch)

**Iroh**:
- Modern P2P networking
- NAT traversal (hole-punching)
- Efficient data transfer (BLAKE3 hashing)
- Relay servers for restricted networks

## Real-World Use Cases

### Collaborative Document Editing
**Example**: Google Docs alternative that works offline

```dol
gen document {
  @crdt(immutable)
  has id: String

  @crdt(peritext)
  has content: String

  @crdt(or_set)
  has collaborators: Set<String>
}
```

**Features**:
- Multiple users edit simultaneously
- Works offline, syncs when online
- No "save" button needed
- Conflict-free merging

### Project Management Tools
**Example**: Trello/Asana alternative with offline support

```dol
gen task_board {
  @crdt(rga)
  has columns: Vec<Column>

  @crdt(or_set)
  has tasks: Set<Task>
}
```

**Features**:
- Drag-and-drop task reordering
- Add tasks offline
- Real-time updates
- No sync conflicts

### Decentralized Marketplace
**Example**: eBay alternative with mutual credit

```dol
gen account {
  @crdt(pn_counter, min_value=0)
  has credit_balance: i64

  @crdt(rga)
  has transaction_history: Vec<Transaction>
}
```

**Features**:
- Spend offline with escrow
- P2P value exchange
- No blockchain fees
- Byzantine fault tolerance

### Personal Knowledge Base
**Example**: Notion alternative that syncs P2P

```dol
gen note {
  @crdt(peritext)
  has content: String

  @crdt(or_set)
  has tags: Set<String>

  @crdt(rga)
  has backlinks: Vec<NoteRef>
}
```

**Features**:
- Rich text notes
- Bidirectional linking
- Full-text search (local)
- Sync across devices without cloud

## The Local-First Workflow

### 1. Define Your Schema in DOL

```dol
gen blog_post {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has title: String

  @crdt(peritext)
  has content: String

  @crdt(or_set)
  has tags: Set<String>
}
```

### 2. Compile to WASM

```bash
dol-codegen-rust schemas/blog_post.dol --output src/generated/
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/*.wasm
```

### 3. Use in Your App

```typescript
import { BlogPost } from './generated/blog_post';

// Create a post offline
const post = new BlogPost(crypto.randomUUID());
post.setTitle("My First Post");
post.editContent(0, "Hello, world!");

// Add tags
post.addTag("tutorial");
post.addTag("local-first");

// Save to local storage
await storage.save(post);

// Sync with peers (when online)
await p2pSync.broadcast(post);
```

### 4. Automatic Conflict Resolution

```typescript
// Alice and Bob both edit offline
// Alice's device:
post.editContent(0, "Hello, Alice!");

// Bob's device (concurrent):
post.editContent(0, "Hello, Bob!");

// When they sync, Peritext CRDT merges:
// Result: "Hello, Alice!Hello, Bob!"
// (deterministic, no user intervention)
```

## Key Concepts

### CRDTs Explained Simply

**Problem**: Two users edit the same document offline. Who wins?

**Traditional Solution**: Last-write-wins (data loss!) or manual conflict resolution (annoying!)

**CRDT Solution**: Mathematical merge function that always produces the same result, regardless of order.

**Example - Counter**:
```
Alice's device: counter = 5 → increment → 6
Bob's device:   counter = 5 → increment → 6

Traditional merge: 6 (one increment lost!)
CRDT merge:        7 (both increments preserved)
```

### The Seven CRDT Strategies

| Strategy | Use Case | Example |
|----------|----------|---------|
| **immutable** | IDs, creation timestamps | `has id: String` |
| **lww** | Metadata, single values | `has title: String` |
| **or_set** | Tags, collections | `has tags: Set<String>` |
| **pn_counter** | Likes, votes, metrics | `has like_count: Int` |
| **peritext** | Rich text documents | `has content: String` |
| **rga** | Ordered lists | `has tasks: Vec<Task>` |
| **mv_register** | Conflict detection | `has theme: Theme` |

### P2P Networking Without Servers

**How it Works**:
1. **Discovery**: Find peers using DHT (Distributed Hash Table)
2. **Connect**: Direct peer-to-peer connection (NAT traversal)
3. **Sync**: Exchange CRDT operations
4. **Merge**: Apply operations locally (guaranteed convergence)

**Fallback**: Relay servers for firewalled networks (data still encrypted)

## Comparison with Other Approaches

### vs. Traditional Client-Server

| Feature | Client-Server | Local-First (VUDO) |
|---------|---------------|-------------------|
| Works offline | ❌ No | ✅ Yes |
| Latency | 50-200ms | < 1ms |
| Data ownership | Server | User |
| Scalability | Expensive servers | P2P (free) |
| Conflict resolution | Manual | Automatic (CRDT) |
| Privacy | Server sees all | E2E encrypted |

### vs. Blockchain/Web3

| Feature | Blockchain | Local-First (VUDO) |
|---------|-----------|-------------------|
| Works offline | ❌ No | ✅ Yes |
| Transaction cost | Gas fees ($$$) | Free |
| Latency | 10+ seconds | < 1ms |
| Privacy | Public ledger | Private by default |
| Scalability | Limited TPS | Unlimited local ops |
| Consensus | Global (slow) | Local (fast) + optional BFT |

### vs. Firebase/Supabase

| Feature | Firebase | Local-First (VUDO) |
|---------|----------|-------------------|
| Works offline | ⚠️ Limited | ✅ Full |
| Vendor lock-in | ✅ Yes | ❌ No |
| Cost at scale | High (per read/write) | Low (P2P) |
| Data residency | Vendor servers | User devices |
| Open source | ❌ No | ✅ Yes |

## Getting Started

Ready to build your first local-first app?

→ **Next**: [Installation](./01-installation.md) - Set up the DOL toolchain

## Further Reading

### Papers
- [Local-First Software (Ink & Switch)](https://www.inkandswitch.com/local-first/)
- [Conflict-Free Replicated Data Types (Shapiro et al.)](https://hal.inria.fr/inria-00609399v1/document)
- [Automerge: Making Servers Optional (Kleppmann)](https://martin.kleppmann.com/2019/11/27/data-access-patterns-distributed-systems.html)

### Related Projects
- [Automerge](https://automerge.org) - CRDT library
- [Iroh](https://iroh.computer) - P2P networking
- [Willow Protocol](https://willowprotocol.org) - Sync protocol
- [ElectricSQL](https://electric-sql.com) - Local-first Postgres

### Blog Posts
- [Why Local-First?](https://bricolage.io/some-notes-on-local-first-development/)
- [CRDT Primer](https://crdt.tech)

---

**Next**: [Installation →](./01-installation.md)
