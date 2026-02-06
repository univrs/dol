//! Basic usage example for VUDO storage adapters.
//!
//! This example demonstrates:
//! - Creating a storage adapter (both native and browser)
//! - Saving and loading documents
//! - Managing operation queues
//! - Working with snapshots

use bytes::Bytes;
use vudo_storage::{StorageAdapter, QueryFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VUDO Storage Adapters Example ===\n");

    // Example 1: Native SQLite storage
    println!("1. Native SQLite Storage:");
    demo_native_storage().await?;

    println!("\n2. Browser In-Memory Storage:");
    demo_browser_storage().await?;

    println!("\n=== All examples completed successfully! ===");

    Ok(())
}

async fn demo_native_storage() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("example.db");

    let storage = vudo_storage_native::SqliteAdapter::new(&db_path).await?;
    storage.init().await?;

    // Save some documents
    storage.save("users", "alice", Bytes::from(b"Alice's data".to_vec())).await?;
    storage.save("users", "bob", Bytes::from(b"Bob's data".to_vec())).await?;
    storage.save("posts", "post1", Bytes::from(b"First post".to_vec())).await?;

    println!("  Saved 3 documents");

    // List documents in namespace
    let users = storage.list("users").await?;
    println!("  Users: {:?}", users);

    // Load a document
    let alice_data = storage.load("users", "alice").await?;
    println!("  Alice's data: {:?}", alice_data.map(|b| String::from_utf8_lossy(&b).to_string()));

    // Query documents
    let all_users = storage.query("users", QueryFilter::All).await?;
    println!("  Query returned {} users", all_users.len());

    // Save a snapshot
    storage.save_snapshot("users", "alice", 1, Bytes::from(b"Snapshot v1".to_vec())).await?;
    let snapshot = storage.load_snapshot("users", "alice").await?;
    println!("  Snapshot version: {:?}", snapshot.map(|(v, _)| v));

    // Get statistics
    let stats = storage.stats().await?;
    println!("  Stats: {} documents, {} bytes", stats.document_count, stats.total_document_size);

    Ok(())
}

async fn demo_browser_storage() -> Result<(), Box<dyn std::error::Error>> {
    let storage = vudo_storage_browser::MemoryAdapter::new();
    storage.init().await?;

    // Save some documents
    storage.save("users", "alice", Bytes::from(b"Alice's data".to_vec())).await?;
    storage.save("users", "bob", Bytes::from(b"Bob's data".to_vec())).await?;

    println!("  Saved 2 documents");

    // List documents
    let users = storage.list("users").await?;
    println!("  Users: {:?}", users);

    // Save operations
    let ops = vec![
        vudo_storage::Operation::new(
            1,
            "users",
            "alice",
            vudo_storage::operation::OperationType::Create,
        ),
    ];
    storage.save_operations(&ops).await?;

    let loaded_ops = storage.load_operations().await?;
    println!("  Loaded {} operations", loaded_ops.len());

    // Get statistics
    let stats = storage.stats().await?;
    println!("  Stats: {} documents, {} operations", stats.document_count, stats.operation_count);

    Ok(())
}
