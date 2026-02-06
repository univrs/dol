# Gen Registry Architecture

## Overview

Gen Registry is a local-first P2P registry for DOL Gen modules, built on the VUDO platform with CRDT-based synchronization.

## System Architecture

### Layered Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  - CLI (Clap-based command interface)                       │
│  - Web UI (HTML5 + WASM bindings)                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                     Business Logic Layer                    │
│  - Registry (module management)                             │
│  - Search (Tantivy full-text search)                        │
│  - Version Resolver (dependency resolution)                 │
│  - WASM Validator (binary validation)                       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                        CRDT Layer                           │
│  - Automerge (conflict-free replication)                    │
│  - OR-Set (tags, authorized publishers)                     │
│  - RGA (versions, dependencies)                             │
│  - PN-Counter (downloads, ratings)                          │
│  - LWW (metadata fields)                                    │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                      Storage Layer                          │
│  - VUDO Storage (platform-agnostic)                         │
│  - IndexedDB (browser)                                      │
│  - SQLite (native)                                          │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                         P2P Layer                           │
│  - Iroh (QUIC-based networking)                             │
│  - Willow Protocol (structured namespaces)                  │
│  - Meadowcap (capability-based permissions)                 │
└─────────────────────────────────────────────────────────────┘
```

## Component Design

### Registry Core

**Responsibility**: Manage module lifecycle, versioning, and dependencies

**Key Types**:
- `Registry`: Main registry struct
- `GenModule`: Module metadata
- `ModuleVersion`: Version information
- `Dependency`: Dependency declaration

**Operations**:
- `publish()`: Publish new module version
- `install()`: Install module with dependencies
- `search()`: Search for modules
- `rate()`: Submit rating

### Search Engine

**Technology**: Tantivy (Rust full-text search)

**Index Schema**:
```rust
{
  "module_id": TEXT | STORED,
  "name": TEXT | STORED,
  "description": TEXT | STORED,
  "keywords": TEXT
}
```

**Query Types**:
- Free-text search
- Tag filtering
- Sort by popularity/rating/recency

### Version Resolver

**Algorithm**: Topological sort with cycle detection

**Features**:
- Semver rule matching
- Dependency DAG resolution
- Circular dependency detection
- Optimal install order

**Example**:
```
A depends on B ^1.0.0
B depends on C >=2.0.0
C depends on D *

Install order: D → C → B → A
```

### WASM Validator

**Checks**:
1. Magic number (`\0asm`)
2. Version (1)
3. Binary structure
4. Import whitelist
5. Size limits
6. Hash verification

**Security**:
- Cryptographic signatures
- Hash-based content addressing
- Import sandboxing

## CRDT Design

### Module Metadata

```dol
gen registry.GenModule {
    @crdt(immutable) has id: String
    @crdt(lww) has name: String
    @crdt(lww) has description: String
    @crdt(or_set) has tags: Set<string>
    @crdt(rga) has versions: Vec<ModuleVersion>
    @crdt(pn_counter) has download_count: i64
}
```

**Conflict Resolution**:
- `id`: Immutable, set once
- `name`, `description`: Last-write-wins
- `tags`: Add-wins (OR-Set)
- `versions`: Replicated growable array
- `download_count`: Monotonic increment

### Ratings

```dol
gen registry.Rating {
    @crdt(immutable) has module_id: String
    @crdt(immutable) has user_did: String
    @crdt(lww) has stars: UInt8
    @crdt(lww) has review: String
}
```

**Uniqueness**: One rating per (module_id, user_did) pair

## P2P Synchronization

### Willow Protocol Mapping

```
DOL System → Willow Namespace
DOL Collection → Willow Subspace
DOL Document → Willow Path
```

**Example**:
```
Namespace: 0x1a2b3c... (hash of "registry.gen.community")
Subspace: "io.univrs.user"
Paths:
  - metadata.json
  - versions/1.0.0.json
  - wasm/1.0.0.wasm
  - ratings/did:key:alice.json
```

### Sync Protocol

1. **Peer Discovery**
   - mDNS (local network)
   - DHT (internet-wide)
   - Relay servers (fallback)

2. **Capability Exchange**
   - Meadowcap tokens
   - Read/write permissions
   - Delegation chains

3. **Data Sync**
   - Delta compression
   - Incremental updates
   - Bloom filter deduplication

4. **Conflict Resolution**
   - Automerge CRDT merge
   - Deterministic convergence

## Storage Architecture

### Browser (IndexedDB)

```javascript
{
  "modules": ObjectStore<GenModule>,
  "installed": ObjectStore<InstalledModule>,
  "ratings": ObjectStore<Rating>,
  "search_index": ObjectStore<SearchIndex>,
  "sync_queue": ObjectStore<PendingOp>
}
```

### Native (SQLite)

```sql
CREATE TABLE modules (
  id TEXT PRIMARY KEY,
  data BLOB  -- Automerge document
);

CREATE TABLE installed (
  module_id TEXT PRIMARY KEY,
  version TEXT,
  installed_at INTEGER
);

CREATE TABLE ratings (
  module_id TEXT,
  user_did TEXT,
  stars INTEGER,
  PRIMARY KEY (module_id, user_did)
);
```

## Security Model

### Cryptographic Signing

```rust
fn sign_module(module_id: &str, version: &str, wasm_hash: &str) -> Signature {
    let msg = [module_id, version, wasm_hash].concat();
    signing_key.sign(msg.as_bytes())
}
```

### Hash Verification

```rust
fn verify_wasm(wasm_bytes: &[u8], expected_hash: &str) -> Result<()> {
    let actual_hash = sha256(wasm_bytes);
    if actual_hash != expected_hash {
        return Err(HashMismatch);
    }
    Ok(())
}
```

### Permission Model

**Meadowcap Capabilities**:
- `publish`: Can publish modules
- `read`: Can download modules
- `rate`: Can submit ratings
- `admin`: Can revoke capabilities

## Performance Characteristics

### Benchmarks (Target)

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Search | < 50ms | 1000 qps |
| CRDT Merge | < 10ms | 100 ops/s |
| WASM Validation | < 100ms | 50 modules/s |
| Sync | < 1s | 10 modules/s |

### Optimization Techniques

**Search**:
- Inverted index (Tantivy)
- Query caching
- Incremental indexing

**CRDT**:
- Operation batching
- Lazy materialization
- Incremental sync

**Storage**:
- String interning
- Compact encoding (bincode)
- LRU cache

**P2P**:
- Delta compression (zstd)
- Parallel downloads
- Bandwidth throttling

## Deployment Targets

### Browser (PWA)

- WASM compilation
- IndexedDB storage
- Service Worker sync
- WebRTC P2P

### Desktop (Native)

- SQLite storage
- OS integration
- System tray
- Auto-update

### Server (Relay)

- Iroh relay
- Registry cache
- Analytics
- Monitoring

## Future Enhancements

1. **Content-Addressable Storage**
   - IPFS integration
   - Deduplication

2. **Advanced Search**
   - Type-aware search
   - Capability filtering
   - Dependency graph visualization

3. **Analytics**
   - Usage statistics
   - Popularity trends
   - Dependency analysis

4. **Security**
   - Malware scanning
   - Vulnerability database
   - Security advisories

---

**Next Steps**: See [API Documentation](api.md) for implementation details.
