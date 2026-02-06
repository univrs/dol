# vudo-storage

Platform-agnostic storage trait for VUDO Runtime persistence.

## Overview

This crate provides the core `StorageAdapter` trait that all platform-specific storage implementations must implement. It defines a unified interface for:

- Document persistence (save/load/delete/list)
- Operation queue persistence (for offline mutations)
- Snapshot management (for document versioning)
- Query capabilities (time-based and filter-based queries)

## Platform Implementations

- **vudo-storage-browser**: Browser storage using in-memory adapter (IndexedDB and OPFS+SQLite WASM planned)
- **vudo-storage-native**: Desktop/Mobile/Server storage using native SQLite

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
vudo-storage = "0.1"
```

The trait is designed to be implemented by platform-specific adapters:

```rust
use vudo_storage::{StorageAdapter, QueryFilter};
use bytes::Bytes;

async fn example(storage: impl StorageAdapter) -> vudo_storage::Result<()> {
    // Initialize storage
    storage.init().await?;

    // Save a document
    let data = Bytes::from("document content");
    storage.save("users", "alice", data.clone()).await?;

    // Load it back
    let loaded = storage.load("users", "alice").await?;
    assert_eq!(loaded, Some(data));

    // Query documents
    let recent = storage.query(
        "users",
        QueryFilter::updated_after(timestamp)
    ).await?;

    Ok(())
}
```

## Features

### Document Operations

- `save`: Store or update a document
- `load`: Retrieve a document
- `delete`: Remove a document
- `list`: List all document IDs in a namespace

### Operation Queue

- `save_operations`: Persist operation queue for offline sync
- `load_operations`: Restore operation queue

### Snapshots

- `save_snapshot`: Store a versioned snapshot
- `load_snapshot`: Retrieve the latest snapshot

### Queries

- `query`: Filter documents by various criteria
  - `QueryFilter::All`: All documents
  - `QueryFilter::UpdatedAfter`: Documents updated after timestamp
  - `QueryFilter::UpdatedBefore`: Documents updated before timestamp
  - `QueryFilter::UpdatedBetween`: Documents in time range
  - `QueryFilter::And/Or/Not`: Combine filters

### Statistics

- `stats`: Get storage statistics (document count, sizes, etc.)

## Testing

Run the test suite:

```bash
cargo test
```

## License

MIT OR Apache-2.0
