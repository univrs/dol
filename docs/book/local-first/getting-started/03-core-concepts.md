# Core Concepts

This guide explains the fundamental concepts behind local-first development with VUDO: DOL, CRDTs, and P2P synchronization.

## DOL (Distributed Ontology Language)

### What is DOL?

DOL is a domain-specific language for defining distributed data models as ontologies. It's like TypeScript for databases, but with:

- **CRDT annotations** for conflict resolution
- **Constraint system** for invariants
- **Evolution tracking** for schema versioning
- **Mandatory documentation** (exegesis blocks)
- **Compilation to WASM** for multi-platform execution

### Basic Syntax

```dol
gen entity_name {
  @crdt(strategy)
  has field_name: FieldType

  // More fields...
}

constraint entity_name.rule_name {
  // Constraints...
}

docs {
  Documentation explaining the entity.
}
```

### Example: User Profile

```dol
gen user.profile {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has display_name: String

  @crdt(lww)
  has avatar_url: String

  @crdt(or_set)
  has interests: Set<String>

  @crdt(pn_counter, min_value=0)
  has reputation: i64
}

constraint user.immutability {
  user never changes id
}

constraint user.reputation_bounds {
  reputation always >= 0
}

docs {
  User profile with immutable identity, mutable metadata,
  and add-wins interest set. Reputation uses PN-Counter
  to prevent negative values.
}
```

### Key Features

**1. Type Safety**

DOL enforces CRDT-type compatibility at compile time:

```dol
// ✅ Valid: OR-Set on Set type
@crdt(or_set)
has tags: Set<String>

// ❌ Invalid: OR-Set on String type
@crdt(or_set)
has name: String  // Error: Type mismatch
```

**2. Constraints**

Express business logic as constraints:

```dol
constraint account.solvency {
  balance always >= 0
  escrow always <= balance
}
```

Constraints are categorized:
- **CRDT-safe**: Enforced by the strategy itself
- **Eventually consistent**: May be temporarily violated
- **Strong consistency**: Requires coordination (escrow/BFT)

**3. Evolution**

Track schema changes over versions:

```dol
evolves user.profile @2.0.0 > 1.0.0 {
  adds @crdt(or_set)
  adds interests: Set<String>

  changes display_name from @crdt(immutable) to @crdt(lww)

  because "Version 2.0 allows users to change their display name
           and adds interest tags for personalization."
}
```

## CRDTs (Conflict-free Replicated Data Types)

### The Problem CRDTs Solve

**Scenario**: Alice and Bob edit a document offline. When they sync, whose changes win?

**Traditional approaches**:
- **Last-write-wins**: Simple but loses data
- **Manual resolution**: Annoying and error-prone
- **Locks**: Requires connectivity, slow

**CRDT approach**:
- Mathematical merge function
- Always produces the same result
- No data loss
- No user intervention needed

### How CRDTs Work

CRDTs guarantee **Strong Eventual Consistency** (SEC):

1. **Eventual Delivery**: All operations eventually reach all replicas
2. **Convergence**: Replicas that receive the same operations converge to the same state
3. **Commutativity**: Order of operations doesn't matter

**Mathematical Property**:
```
merge(A, B) = merge(B, A)  // Commutative
merge(A, merge(B, C)) = merge(merge(A, B), C)  // Associative
```

### The Seven CRDT Strategies

#### 1. Immutable

**Use case**: IDs, creation timestamps, permanent identity

**Merge rule**: First write wins (deterministic tie-breaking)

```dol
@crdt(immutable)
has id: String
```

**Example**:
```
Alice (t=1): id = "abc"
Bob (t=2):   id = "xyz"

Merge: id = "abc"  (first write wins)
```

#### 2. Last-Write-Wins (LWW)

**Use case**: Metadata, single-valued fields

**Merge rule**: Most recent timestamp wins

```dol
@crdt(lww)
has title: String
```

**Example**:
```
Alice (t=1): title = "Draft"
Bob (t=2):   title = "Final"

Merge: title = "Final"  (t=2 > t=1)
```

**Tradeoff**: Simple and efficient, but loses concurrent updates.

#### 3. OR-Set (Observed-Remove Set)

**Use case**: Collections, tags, lists where adds are important

**Merge rule**: Add-wins semantics

```dol
@crdt(or_set)
has tags: Set<String>
```

**Example**:
```
Alice: addTag("rust")
Bob:   addTag("wasm")

Merge: tags = {"rust", "wasm"}  (both preserved)

Alice: addTag("dev")
Bob:   removeTag("dev")  (but Bob hasn't seen Alice's add)

Merge: tags = {..., "dev"}  (add wins, remove only works on observed adds)
```

#### 4. PN-Counter (Positive-Negative Counter)

**Use case**: Metrics, likes, votes, reputation

**Merge rule**: Sum all increments and decrements

```dol
@crdt(pn_counter, min_value=0)
has like_count: i64
```

**Example**:
```
Alice: increment(+5)  → 5
Bob:   increment(+3)  → 3 (concurrent)

Merge: 5 + 3 = 8  (both increments preserved)
```

**Why it works**: Addition is commutative and associative.

#### 5. Peritext (Rich Text)

**Use case**: Collaborative text editing with formatting

**Merge rule**: Character-level CRDT with mark-based formatting

```dol
@crdt(peritext, formatting="full")
has content: String
```

**Example**:
```
Initial: "Hello"

Alice: insert(5, " World")  → "Hello World"
Bob:   insert(0, "Hi ")     → "Hi Hello" (concurrent)

Merge: "Hi Hello World"  (deterministic character ordering)
```

**Advanced**: Preserves formatting (bold, italic) across concurrent edits.

#### 6. RGA (Replicated Growable Array)

**Use case**: Ordered lists, task boards, sequences

**Merge rule**: Causal ordering with timestamp tiebreaker

```dol
@crdt(rga)
has tasks: Vec<Task>
```

**Example**:
```
Alice: insert(0, "Task A")
Bob:   insert(0, "Task B") (concurrent)

Merge: ["Task B", "Task A"]  (timestamp tiebreaker)
```

**Use case**: Kanban board columns maintain order even with concurrent reordering.

#### 7. MV-Register (Multi-Value Register)

**Use case**: Conflict detection, showing all concurrent values

**Merge rule**: Keep all causally concurrent values

```dol
@crdt(mv_register)
has theme: Theme
```

**Example**:
```
Alice: theme = "dark"
Bob:   theme = "light" (concurrent)

Merge: theme = ["dark", "light"]  (app decides resolution)
```

**Use case**: Detect conflicts and let user or AI choose resolution.

## Data Flow Architecture

### Operational Transform vs CRDT

**Operational Transform (OT)**:
```
Client A → Operation → Server → Transform → Client B
         ← Acknowledgment ←
```

**Pros**: Widely used (Google Docs)
**Cons**: Requires central server, complex transformation functions

**CRDT**:
```
Client A → Operation → Local Apply → Sync → Client B → Merge
```

**Pros**: No server needed, simpler, proven convergence
**Cons**: Larger metadata overhead (but acceptable with modern CRDT designs)

### VUDO Data Flow

```
┌──────────────────────────────────────────────────┐
│ User Action (e.g., edit text)                    │
└────────────────┬─────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────┐
│ WASM Module (compiled from DOL)                  │
│  - Validate operation against constraints        │
│  - Apply CRDT operation                          │
│  - Update local state                            │
└────────────────┬─────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────┐
│ Storage Layer (IndexedDB/SQLite)                 │
│  - Persist CRDT operations                       │
│  - Store Automerge document                      │
└────────────────┬─────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────┐
│ UI Update (reactive state management)            │
│  - Re-render components                          │
│  - Show instant feedback (< 1ms)                 │
└──────────────────────────────────────────────────┘
                 ↓ (background)
┌──────────────────────────────────────────────────┐
│ P2P Sync (Iroh)                                  │
│  - Discover peers                                │
│  - Exchange operations                           │
│  - Merge remote changes                          │
└──────────────────────────────────────────────────┘
```

### Key Insights

1. **Local-first**: User actions apply locally immediately
2. **Optimistic UI**: UI updates before sync
3. **Background sync**: Network operations don't block UI
4. **Automatic merge**: CRDTs handle conflicts automatically

## P2P Synchronization

### Architecture

```
┌──────────┐         ┌──────────┐
│ Alice    │ ←─────→ │ Bob      │  (direct P2P)
│ (Peer 1) │         │ (Peer 2) │
└─────┬────┘         └────┬─────┘
      │                   │
      │    ┌──────────┐   │
      └───→│ Relay    │←──┘  (fallback for NAT)
           │ Server   │
           └──────────┘
```

### Peer Discovery

**1. Local Network (mDNS)**:
- Discover peers on same WiFi/LAN
- Fast, no internet required

**2. DHT (Distributed Hash Table)**:
- Global peer discovery
- Uses public DHT (n0.computer)

**3. Relay Servers**:
- Fallback for firewalled networks
- Data stays encrypted (relay can't read)

### Sync Protocol

**1. Connection Establishment**:
```rust
let endpoint = MagicEndpoint::builder()
    .discovery_n0()
    .relay_mode(RelayMode::Default)
    .bind(0)
    .await?;
```

**2. Peer Connection**:
```rust
let conn = endpoint.connect(peer_addr, "my-app")
    .await?;
```

**3. Sync Messages**:
```rust
// Send local changes
let changes = doc.get_changes(&[]);
conn.send(&changes).await?;

// Receive remote changes
let remote_changes = conn.recv().await?;
doc.apply_changes(remote_changes)?;
```

**4. CRDT Merge**:
```rust
// Automatic convergence
let merged = Automerge::merge(local_doc, remote_doc)?;
```

### Sync Strategies

**Eager Sync** (default):
- Sync immediately on change
- Low latency, high bandwidth

**Lazy Sync**:
- Batch changes, sync periodically
- Lower bandwidth, higher latency

**On-Demand Sync**:
- Sync only when user requests
- Lowest bandwidth, manual control

Configure in `dol.toml`:
```toml
[sync]
strategy = "eager"  # or "lazy", "on_demand"
batch_interval_ms = 1000
max_batch_size = 1000
```

## State Management

### Local State

**IndexedDB** (browser):
```typescript
const db = await openDB('my-app', 1, {
  upgrade(db) {
    db.createObjectStore('documents');
  },
});

await db.put('documents', docBytes, docId);
```

**SQLite** (desktop):
```rust
let conn = Connection::open("my-app.db")?;
conn.execute(
    "CREATE TABLE IF NOT EXISTS documents (
        id TEXT PRIMARY KEY,
        data BLOB NOT NULL
    )",
    [],
)?;
```

### State Hydration

On app startup:
```typescript
// 1. Load from local storage
const saved = await storage.load(docId);

// 2. Deserialize Automerge document
const doc = Automerge.load(saved);

// 3. Hydrate app state
const state = autosurgeon::hydrate(&doc)?;

// 4. Render UI
render(state);
```

### State Updates

On user action:
```typescript
// 1. User edits
editor.onChange((text) => {
  // 2. Update CRDT
  doc.editContent(cursor, text);

  // 3. Save locally
  storage.save(docId, doc.save());

  // 4. Sync in background
  p2p.broadcast(doc);
});
```

## Performance Considerations

### WASM Module Size

**Target**: < 100KB compressed

**Optimization**:
```toml
[profile.release]
opt-level = 'z'  # Optimize for size
lto = true       # Link-time optimization
codegen-units = 1
```

### CRDT Merge Latency

**Target**: < 10ms for 10K operations

**Optimization**:
- Incremental merge (don't re-merge everything)
- Operation batching
- Parallel merge for independent fields

### Memory Usage

**Target**: < 50MB for 100K records

**Optimization**:
- String interning
- Compact binary format (Automerge)
- Lazy loading of old history

### Sync Throughput

**Target**: > 1000 ops/sec

**Optimization**:
- Delta compression
- Parallel sync for multiple documents
- Bloom filters for change detection

## Security Model

### Identity

**did:key** (decentralized identifiers):
```
did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH
```

No central authority, cryptographically verifiable.

### Permissions

**UCANs** (User-Controlled Authorization Networks):
```json
{
  "aud": "did:key:z6Mkr...",
  "att": [{
    "with": "doc:abc123",
    "can": "edit"
  }],
  "exp": 1234567890,
  "proof": "..."
}
```

Capability-based, delegatable, revocable.

### Encryption

**E2E Encryption**:
- Each document encrypted with unique key
- Keys shared only with authorized peers
- GDPR-compliant deletion (delete key = unreadable data)

**Threat Model**:
- ✅ Protects against: Relay server snooping, network eavesdropping
- ❌ Doesn't protect against: Malicious peers with valid capabilities

## Next Steps

You now understand the core concepts of local-first development with VUDO. Ready to dive deeper?

### Learn by Doing

- **CRDT Guide**: [Detailed guide for each CRDT strategy](../crdt-guide/00-overview.md)
- **P2P Networking**: [Set up peer-to-peer sync](../p2p-networking/00-overview.md)
- **Mutual Credit**: [Add value exchange to your app](../mutual-credit/00-overview.md)

### Reference

- **DOL Syntax**: [Complete language reference](../api-reference/dol-syntax.md)
- **API Docs**: [Generated API documentation](../api-reference/rust-api.md)
- **Examples**: [Real-world applications](/apps/workspace)

---

**Next**: [CRDT Guide →](../crdt-guide/00-overview.md)
