# VUDO State Engine

Platform-agnostic local state management for VUDO Runtime with Automerge CRDT support.

## Overview

The VUDO State Engine is the core local state management layer for the VUDO Runtime (Phase 2: MYCELIUM). It provides a comprehensive solution for managing CRDT-based document state with reactive subscriptions, offline operation queuing, snapshot management, and multi-document transactions.

## Features

- **Automerge Document Store**: In-memory document cache with lifecycle management
- **Reactive Subscriptions**: Observable pattern for change notifications with < 16ms latency
- **Operation Queue**: FIFO queue for offline mutations with persistence and deduplication
- **Snapshot Management**: Periodic compaction with 50%+ storage reduction
- **Multi-Document Transactions**: Atomic operations with commit/rollback support
- **Platform-Agnostic**: Pure Rust core with no browser/desktop dependencies

## Performance Targets

- **< 1ms** local read/write latency (in-memory operations)
- **< 16ms** reactive subscription firing (one animation frame)
- **50%+** storage reduction via snapshot compaction
- **100K+** operations/sec throughput

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vudo-state = "0.1.0"
```

## Quick Start

```rust
use vudo_state::{StateEngine, DocumentId};
use automerge::{transaction::Transactable, ROOT};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize state engine
    let engine = StateEngine::new().await?;

    // Create a document
    let doc_id = DocumentId::new("users", "alice");
    let handle = engine.create_document(doc_id).await?;

    // Update the document
    handle.update(|doc| {
        doc.put(ROOT, "name", "Alice")?;
        doc.put(ROOT, "age", 30i64)?;
        Ok(())
    })?;

    // Read from the document
    handle.read(|doc| {
        // Extract values from doc...
        Ok(())
    })?;

    Ok(())
}
```

## Examples

### Basic CRUD Operations

See `examples/simple_state.rs` for a complete example of creating, reading, updating, and deleting documents.

```bash
cargo run --example simple_state
```

### Reactive Subscriptions

See `examples/reactive_updates.rs` for an example of subscribing to document changes.

```bash
cargo run --example reactive_updates
```

### Offline Operation Queue

See `examples/offline_queue.rs` for an example of queuing operations for offline sync.

```bash
cargo run --example offline_queue
```

### Multi-Document Transactions

See `examples/transactions.rs` for an example of atomic multi-document operations.

```bash
cargo run --example transactions
```

## Architecture

```
StateEngine
├── DocumentStore      - In-memory document cache
├── ChangeObservable   - Reactive subscription system
├── OperationQueue     - Offline operation tracking
├── SnapshotStorage    - Snapshot management
└── TransactionManager - Multi-document transactions
```

## API Documentation

### StateEngine

Main entry point for all state operations.

```rust
let engine = StateEngine::new().await?;
let engine = StateEngine::with_config(config).await?;
```

### DocumentStore

Manages Automerge documents with namespace organization.

```rust
let doc_id = DocumentId::new("namespace", "key");
let handle = store.create(doc_id)?;
let handle = store.get(&doc_id)?;
store.delete(&doc_id)?;
```

### Reactive Subscriptions

Subscribe to document changes with fine-grained filtering.

```rust
let filter = SubscriptionFilter::Document(doc_id);
let mut sub = engine.subscribe(filter).await;

while let Some(event) = sub.recv().await {
    println!("Change: {:?}", event);
}
```

### Operation Queue

Track offline operations for sync.

```rust
let op = Operation::new(OperationType::Create { document_id });
engine.queue.enqueue(op)?;

// Serialize for persistence
let bytes = engine.queue.serialize()?;

// Deserialize on restart
engine.queue.deserialize(&bytes)?;
```

### Transactions

Atomic multi-document operations.

```rust
let tx = engine.begin_transaction();

tx.update(&doc_id1, |doc| { /* ... */ })?;
tx.update(&doc_id2, |doc| { /* ... */ })?;

engine.commit_transaction(tx)?;
// or
engine.rollback_transaction(tx)?;
```

### Snapshots

Periodic compaction for storage efficiency.

```rust
let snapshot = engine.snapshot(&handle).await?;
let result = engine.compact(&handle).await?;

println!("Compaction saved {} bytes ({}%)",
    result.reduction, result.reduction_percent);
```

## Testing

Run all tests:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

## Integration

### Phase 1.3 Output

Uses generated Automerge-backed structs from `dol-codegen-rust`.

### Phase 2.2 (Storage)

Provides hooks for persistence layer integration.

### Phase 2.3 (P2P)

Operation queue feeds the sync protocol.

### Phase 2.5 (Evolution)

Schema evolution hooks in document loading.

## License

MIT OR Apache-2.0

## Contributing

This is part of the VUDO Runtime (MYCELIUM phase) within the univrs-dol project.
