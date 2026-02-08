# vudo-p2p

Iroh P2P Integration Layer for VUDO Runtime

## Overview

`vudo-p2p` provides peer-to-peer networking capabilities for the VUDO Runtime, combining:

- **Iroh P2P Networking**: QUIC-based encrypted connections with peer discovery and NAT traversal
- **Willow Protocol**: Structured data sync with fine-grained capabilities
- **Automerge CRDT**: Conflict-free replicated data types for distributed state

## Features

### Iroh P2P Networking

- **Peer Discovery**
  - mDNS for local network discovery
  - DHT for internet-wide discovery
  - Relay servers for NAT traversal

- **Connection Management**
  - Direct QUIC connections (best case)
  - Relay fallback for NAT/firewall scenarios
  - Connection pooling and reuse
  - Peer scoring and prioritization

- **Automerge Sync Protocol**
  - Incremental sync (send only diffs)
  - Multi-document sync
  - Sync state tracking per peer
  - Conflict-free merge guarantees

- **Gossip Overlay**
  - Document presence announcements
  - Peer capability discovery
  - Topic-based routing

- **Bandwidth Management**
  - Metered connection detection
  - Adaptive sync rate
  - Prioritization (user-initiated > background)
  - Compression

- **Background Sync**
  - Non-blocking UI thread
  - Web Worker support (browser)
  - Tokio task support (native)

### Willow Protocol Integration

- **3D Namespace Structure**: Namespace → Subspace → Path
- **Meadowcap Capabilities**: Fine-grained permissions and delegation
- **GDPR-Compliant Deletion**: Tombstones for permanent deletion
- **Resource-Aware Sync**: Bandwidth and memory constraints

## Usage

### Basic P2P Setup

```rust
use vudo_p2p::{VudoP2P, P2PConfig};
use vudo_state::StateEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create state engine
    let state_engine = Arc::new(StateEngine::new().await?);

    // Create P2P layer
    let config = P2PConfig::default();
    let p2p = VudoP2P::new(state_engine, config).await?;

    // Start P2P services
    p2p.start().await?;

    // Discover peers
    let peers = p2p.discover_peers().await?;
    println!("Discovered {} peers", peers.len());

    Ok(())
}
```

### Document Synchronization

```rust
// Sync a document with a peer
p2p.sync_document(&peer_id, "users", "alice").await?;

// Subscribe to document updates
let mut subscription = p2p.subscribe_document("users", "alice").await?;
while let Some(message) = subscription.recv().await {
    println!("Document updated: {:?}", message);
}
```

### Willow Protocol with Capabilities

```rust
use vudo_p2p::{VudoP2P, Capability};
use ed25519_dalek::SigningKey;

// Create P2P with Willow support
let p2p = VudoP2P::with_willow(state_engine, config).await?;

// Get Willow adapter
let willow = p2p.willow().unwrap();

// Create root capability
let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
let namespace_id = willow.map_namespace("myapp.v1");
let root_cap = Capability::new_root(namespace_id, &signing_key);

// Write data with capability
let data = bytes::Bytes::from("test data");
willow.write_entry("myapp.v1", "users", "alice", data, &root_cap).await?;
```

## Performance Targets

- **Peer discovery**: < 5 seconds on local network (mDNS)
- **Sync latency**: < 100ms after connection established
- **Concurrent peers**: 50+ simultaneous connections
- **Bandwidth**: Graceful degradation on slow/metered networks

## Architecture

```
┌─────────────────────────────────────────┐
│          VudoP2P (Coordinator)          │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────┐  ┌──────────┐            │
│  │  Iroh    │  │ Willow   │            │
│  │ Adapter  │  │ Adapter  │            │
│  └──────────┘  └──────────┘            │
│       │              │                  │
│  ┌────▼────┐   ┌────▼────┐             │
│  │ Sync    │   │Meadowcap│             │
│  │Protocol │   │  Store  │             │
│  └─────────┘   └─────────┘             │
│                                         │
│  ┌─────────┐  ┌──────────┐             │
│  │ Gossip  │  │Bandwidth │             │
│  │ Overlay │  │ Manager  │             │
│  └─────────┘  └──────────┘             │
│                                         │
│  ┌──────────┐  ┌──────────┐            │
│  │Discovery │  │Background│            │
│  │  Engine  │  │   Sync   │            │
│  └──────────┘  └──────────┘            │
└─────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│       VUDO State Engine                 │
│   (Automerge CRDT Documents)            │
└─────────────────────────────────────────┘
```

## Browser Support

Iroh P2P requires UDP/QUIC which is not available in browsers. For browser clients:

- **WebSocket Relay**: Browser connects to relay server, relay bridges to Iroh nodes
- **WebRTC**: Direct P2P using WebRTC (requires signaling server)
- **Hybrid**: Native clients use Iroh, browser clients use WebSocket/WebRTC

## Examples

- `simple_sync.rs` - Two nodes sync a document
- `mesh_network.rs` - Multiple nodes in mesh topology (planned)
- `offline_online.rs` - Offline edits → come online → sync (planned)

## Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --lib sync_protocol

# Run with logging
RUST_LOG=debug cargo test

# Run example
cargo run --example simple_sync -- node1
```

## License

MIT OR Apache-2.0
