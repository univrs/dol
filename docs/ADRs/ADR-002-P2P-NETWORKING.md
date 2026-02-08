# ADR-002: P2P Networking Stack

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2025-11-01 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Context

DOL's local-first architecture requires peer-to-peer networking for:
- Spirit package distribution
- Real-time collaborative editing (Séance sessions)
- Decentralized gen registry
- Offline-first sync when connectivity resumes

### Requirements

1. **NAT Traversal** - Work across firewalls and NAT
2. **Content Addressing** - Identify data by hash, not location
3. **Partial Sync** - Sync subsets of data efficiently
4. **WASM Support** - Run in browser via WebRTC/WebSocket
5. **Rust-Native** - First-class Rust implementation
6. **Privacy** - Support for encrypted, capability-based access

### Options Considered

| Stack | NAT | Content-Addr | Partial Sync | WASM | Privacy |
|-------|-----|--------------|--------------|------|---------|
| **Iroh + Willow** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **libp2p** | ✅ | ✅ | ❌ | ⚠️ | ⚠️ |
| **IPFS** | ✅ | ✅ | ❌ | ⚠️ | ❌ |
| **WebRTC raw** | ✅ | ❌ | ❌ | ✅ | ❌ |
| **Hypercore** | ✅ | ✅ | ✅ | ❌ | ⚠️ |

## Decision

**We chose Iroh for transport + Willow Protocol for data sync.**

### Iroh (Transport Layer)

[Iroh](https://iroh.computer/) provides:
- **QUIC-based transport** with automatic NAT traversal
- **Content-addressed blobs** for efficient data transfer
- **Rust-native** with excellent WASM support
- **Relay servers** for difficult network conditions
- **Ed25519 identity** (matches our Univrs Node pattern)

### Willow Protocol (Sync Layer)

[Willow](https://willowprotocol.org/) provides:
- **3D Range Queries** - Sync by (namespace, subspace, time) ranges
- **Partial Sync** - Only fetch relevant data subsets
- **Capability-based Security** - Fine-grained read/write permissions
- **CRDT-friendly** - Designed for eventual consistency

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     DOL Application                      │
├─────────────────────────────────────────────────────────┤
│                  Automerge Documents                     │
│              (CRDT-enabled gen instances)                │
├─────────────────────────────────────────────────────────┤
│                   Willow Protocol                        │
│        (Namespace/Subspace/Time range queries)           │
├─────────────────────────────────────────────────────────┤
│                        Iroh                              │
│         (QUIC transport, NAT traversal, blobs)           │
├─────────────────────────────────────────────────────────┤
│              Network (UDP/TCP/WebRTC)                    │
└─────────────────────────────────────────────────────────┘
```

### Why Not libp2p?

While libp2p is mature, it has:
- Higher complexity for our use case
- Less efficient partial sync (requires custom implementation)
- Heavier WASM bundle size
- More configuration overhead

Iroh + Willow is purpose-built for local-first applications.

## Consequences

### Positive

- **Efficient Sync** - 3D range queries minimize data transfer
- **Privacy by Design** - Capabilities control access granularly
- **Modern Stack** - Built on QUIC, not TCP
- **Aligned Vision** - Both projects share local-first philosophy
- **Growing Ecosystem** - Active development, good documentation

### Negative

- **Newer Technology** - Less battle-tested than libp2p
- **Smaller Community** - Fewer Stack Overflow answers
- **Rapid Evolution** - APIs may change before 1.0

### Neutral

- **Dual Dependency** - Two projects to track vs one
- **Custom Integration** - We built the Automerge ↔ Willow bridge

## Implementation Notes

### Namespace Mapping

DOL concepts map to Willow's 3D space:

| DOL Concept | Willow Dimension |
|-------------|------------------|
| Spirit package | Namespace |
| User/Device | Subspace |
| Document version | Timestamp |

### Example: Spirit Registry Sync

```rust
// Sync all gen definitions in a Spirit, from a specific user, after a timestamp
let query = WillowQuery {
    namespace: spirit_id,           // e.g., "@univrs/containers"
    subspace: Some(user_pubkey),    // Optional: specific author
    time_range: (last_sync, now),   // Only new changes
};

let entries = willow.sync_range(peer, query).await?;
for entry in entries {
    let doc = Automerge::load(&entry.payload)?;
    registry.merge(doc)?;
}
```

### Capability Delegation

```rust
// Grant read-only access to a collaborator
let read_cap = Capability::new(
    namespace: spirit_id,
    subspace: SubspaceRange::All,
    time: TimeRange::All,
    access: Access::Read,
);

let delegated = read_cap.delegate(collaborator_pubkey)?;
send_to_peer(collaborator, delegated);
```

## References

- [Iroh Documentation](https://iroh.computer/docs)
- [Willow Protocol Specification](https://willowprotocol.org/specs)
- [Willow Rust Implementation](https://github.com/earthstar-project/willow-rs)
- [Local-First Software Principles](https://www.inkandswitch.com/local-first/)

## Changelog

| Date | Change |
|------|--------|
| 2025-11-01 | Initial decision |
| 2025-12-15 | Added capability delegation examples |
| 2026-01-20 | Updated for Iroh 0.20 API changes |
