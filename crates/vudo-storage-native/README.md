# vudo-storage-native

Native SQLite storage adapter for VUDO Runtime.

## Overview

High-performance persistent storage for Desktop, Mobile, and Server platforms using native SQLite with Write-Ahead Logging (WAL).

## Features

- **Native SQLite**: Bundled SQLite with full feature set
- **WAL Mode**: Write-Ahead Logging for better concurrency
- **Connection Pooling**: Optimized for concurrent access
- **High Performance**: 100K+ writes/sec target on desktop
- **Async API**: Built on Tokio for async/await support

## Performance

Target benchmarks:
- **Desktop**: 100K+ writes/sec
- **Query Latency**: < 10ms for indexed lookups
- **Concurrent Access**: Multiple readers, optimized writer

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
vudo-storage-native = "0.1"
```

Basic usage:

```rust
use vudo_storage_native::SqliteAdapter;
use vudo_storage::StorageAdapter;
use bytes::Bytes;

#[tokio::main]
async fn main() -> vudo_storage::Result<()> {
    // Create adapter with file path
    let storage = SqliteAdapter::new("./data/vudo.db").await?;
    storage.init().await?;

    // Save a document
    let data = Bytes::from("document content");
    storage.save("users", "alice", data).await?;

    // Load it back
    let loaded = storage.load("users", "alice").await?;

    // Get statistics
    let stats = storage.stats().await?;
    println!("Documents: {}, Size: {} bytes",
        stats.document_count,
        stats.total_document_size);

    Ok(())
}
```

## Database Schema

The adapter creates three tables:

### documents
```sql
CREATE TABLE documents (
    namespace TEXT NOT NULL,
    id TEXT NOT NULL,
    data BLOB NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (namespace, id)
);
CREATE INDEX idx_documents_updated ON documents(namespace, updated_at);
```

### operations
```sql
CREATE TABLE operations (
    id INTEGER PRIMARY KEY,
    data BLOB NOT NULL,
    timestamp INTEGER NOT NULL
);
CREATE INDEX idx_operations_timestamp ON operations(timestamp);
```

### snapshots
```sql
CREATE TABLE snapshots (
    namespace TEXT NOT NULL,
    id TEXT NOT NULL,
    version INTEGER NOT NULL,
    data BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (namespace, id, version)
);
```

## Testing

Run the test suite:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

## License

MIT OR Apache-2.0
