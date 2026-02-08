# dol-exegesis

CRDT-backed exegesis preservation for DOL in local-first mode with Peritext rich text editing.

## Overview

This crate provides the infrastructure for preserving and synchronizing DOL exegesis (documentation) in local-first environments. It implements CRDT-based storage and collaborative editing, allowing multiple developers to edit documentation offline with automatic conflict resolution on sync.

## Features

- **Peritext CRDT**: Rich text collaborative editing for exegesis content
- **Version Linking**: Exegesis versions tied to Gene evolution versions
- **Offline Authoring**: Full support for offline editing with merge on sync
- **Concurrent Editing**: Multiple developers can edit exegesis simultaneously
- **Contributor Tracking**: Automatic tracking of all contributors via DIDs

## Architecture

### Core Components

1. **ExegesisDocument**: CRDT-backed document model
   - `gene_id`: Immutable (set once, never changed)
   - `gene_version`: Immutable (set once, never changed)
   - `content`: Peritext CRDT (rich text with concurrent editing support)
   - `last_modified`: LWW (last-write-wins with timestamp)
   - `contributors`: RGA (replicated growable array for ordered list)

2. **ExegesisManager**: High-level API for creating, editing, and versioning exegesis
   - `create_exegesis()`: Create new exegesis for a Gene
   - `edit_exegesis()`: Edit existing exegesis (concurrent-safe)
   - `get_exegesis()`: Retrieve exegesis by gene ID and version
   - `link_to_evolution()`: Copy exegesis to a new Gene version

3. **CollaborativeEditor**: Real-time collaborative editing
   - `subscribe_changes()`: Subscribe to exegesis change notifications
   - `sync_exegesis()`: Sync exegesis with a specific peer
   - `broadcast()`: Broadcast changes to all connected peers

## Usage

### Basic CRUD Operations

```rust
use dol_exegesis::ExegesisManager;
use vudo_state::StateEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize state engine
    let state_engine = Arc::new(StateEngine::new().await?);
    let manager = ExegesisManager::new(state_engine).await?;

    // Create exegesis for a Gene
    let doc = manager.create_exegesis(
        "user.profile",
        "1.0.0",
        "A user profile contains identity and preferences."
    ).await?;

    // Edit exegesis
    manager.edit_exegesis(
        "user.profile",
        "1.0.0",
        "did:peer:alice",
        |content| {
            content.push_str("\nUpdated by Alice.");
        }
    ).await?;

    // Retrieve exegesis
    let doc = manager.get_exegesis("user.profile", "1.0.0").await?;
    println!("Content: {}", doc.content);
    println!("Contributors: {:?}", doc.contributors);

    Ok(())
}
```

### Collaborative Editing

```rust
use dol_exegesis::{CollaborativeEditor, ExegesisManager};
use vudo_state::StateEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state_engine = Arc::new(StateEngine::new().await?);
    let manager = Arc::new(ExegesisManager::new(state_engine).await?);

    // Create initial exegesis
    manager.create_exegesis(
        "api.authentication",
        "1.0.0",
        "Authentication API for user login."
    ).await?;

    // Set up collaborative editor
    let editor = CollaborativeEditor::new(manager.clone());

    // Subscribe to changes
    let mut subscription = editor
        .subscribe_changes("api.authentication", "1.0.0")
        .await?;

    // Spawn concurrent editors
    tokio::spawn(async move {
        manager.edit_exegesis(
            "api.authentication",
            "1.0.0",
            "did:peer:alice",
            |content| {
                content.push_str("\n\n## OAuth2 Support");
                content.push_str("\n- Supports OAuth2 authorization code flow");
            }
        ).await.unwrap();
    });

    Ok(())
}
```

### Version Evolution

```rust
use dol_exegesis::ExegesisManager;
use vudo_state::StateEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state_engine = Arc::new(StateEngine::new().await?);
    let manager = ExegesisManager::new(state_engine).await?;

    // Create exegesis for v1.0.0
    manager.create_exegesis(
        "payment.gateway",
        "1.0.0",
        "Payment gateway for credit cards."
    ).await?;

    // Evolve to v2.0.0 - link exegesis
    let v2_doc = manager.link_to_evolution(
        "payment.gateway",
        "1.0.0",
        "2.0.0"
    ).await?;

    // Update v2.0.0 with new features
    manager.edit_exegesis(
        "payment.gateway",
        "2.0.0",
        "did:peer:developer",
        |content| {
            content.push_str("\n\n## New in v2.0.0\n- Added cryptocurrency support");
        }
    ).await?;

    Ok(())
}
```

## Examples

Run the examples to see the crate in action:

```bash
# Basic CRUD operations
cargo run --example basic_exegesis

# Collaborative editing
cargo run --example collaborative_editing

# Version evolution
cargo run --example version_evolution
```

## Testing

The crate includes comprehensive tests:

```bash
# Run all tests (51 total)
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run benchmarks
cargo bench
```

### Test Coverage

- **22 unit tests**: Core functionality (model, manager, collaborative)
- **15 manager tests**: CRUD operations, concurrent editing, version linking
- **10 collaborative tests**: Real-time editing, subscriptions, conflict resolution
- **4 doc tests**: Documentation examples

## Performance

Benchmarks are available for:
- `create_exegesis`: Document creation
- `edit_exegesis`: Single edit operation
- `get_exegesis`: Document retrieval
- `concurrent_edits_3_users`: Multi-user concurrent editing

Run benchmarks with:
```bash
cargo bench
```

## Dependencies

- **vudo-state**: CRDT state engine with Automerge support
- **vudo-p2p**: P2P networking for synchronization (optional)
- **automerge**: CRDT backend
- **tokio**: Async runtime
- **chrono**: Timestamp handling

## Success Criteria

✅ Exegesis fields use Peritext CRDT by default
✅ Concurrent exegesis edits merge correctly
✅ Exegesis version linked to Gene evolution version
✅ All tests pass (51 tests total)
✅ Examples demonstrate collaborative editing

## License

MIT OR Apache-2.0
