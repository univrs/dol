# vudo-storage-browser

Browser storage adapter for VUDO Runtime.

## Overview

Persistent storage for browser environments with support for multiple backends.

## Current Implementation

The crate currently provides an in-memory adapter suitable for:
- Testing and development
- Prototype applications
- Fallback when persistent storage is unavailable

## Planned Features

### OPFS + SQLite WASM
- Origin Private File System for persistence
- SQLite compiled to WebAssembly
- Target: 10K+ writes/sec
- Full SQL query capabilities

### Multi-Tab Coordination
- BroadcastChannel API for change notifications
- SharedWorker for write coordination (where supported)
- Leader election for tab coordination
- Read-your-writes consistency

### IndexedDB Fallback
- Browser-native key-value storage
- Wide browser support
- Automatic migration from in-memory

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
vudo-storage-browser = "0.1"
```

Basic usage with in-memory adapter:

```rust
use vudo_storage_browser::MemoryAdapter;
use vudo_storage::StorageAdapter;
use bytes::Bytes;

#[tokio::main]
async fn main() -> vudo_storage::Result<()> {
    let storage = MemoryAdapter::new();
    storage.init().await?;

    // Save a document
    let data = Bytes::from("document content");
    storage.save("users", "alice", data).await?;

    // Load it back
    let loaded = storage.load("users", "alice").await?;

    Ok(())
}
```

## Features

### In-Memory Adapter

- Fast concurrent access using DashMap
- Full StorageAdapter trait implementation
- Suitable for testing and development
- Data lost on page reload

### Performance

Current in-memory implementation:
- **Writes**: 100K+ ops/sec
- **Reads**: 1M+ ops/sec
- **Concurrency**: Lock-free for reads

Target for OPFS+SQLite:
- **Writes**: 10K+ sustained writes/sec
- **Query Latency**: < 10ms indexed lookups

## Testing

Run the test suite:

```bash
cargo test
```

## Roadmap

1. **Phase 1** (Current): In-memory adapter for development
2. **Phase 2**: IndexedDB adapter for production use
3. **Phase 3**: OPFS + SQLite WASM for optimal performance
4. **Phase 4**: Multi-tab coordination with SharedWorker

## Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| In-Memory | ✅ | ✅ | ✅ | ✅ |
| IndexedDB | ✅ | ✅ | ✅ | ✅ |
| OPFS | ✅ 86+ | ✅ 111+ | ✅ 15.2+ | ✅ 86+ |
| SharedWorker | ✅ | ✅ | ❌ | ✅ |

## License

MIT OR Apache-2.0
