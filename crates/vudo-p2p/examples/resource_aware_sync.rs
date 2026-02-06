//! Example: Resource-aware sync with constraints.
//!
//! This example demonstrates how to sync data with memory and bandwidth
//! constraints, useful for mobile devices or constrained environments.

use automerge::{transaction::Transactable, ROOT};
use ed25519_dalek::SigningKey;
use std::sync::Arc;
use vudo_p2p::{meadowcap::Capability, ResourceConstraints, SyncPriority, WillowAdapter};
use vudo_state::{DocumentId, StateEngine};

#[tokio::main]
async fn main() -> vudo_p2p::error::Result<()> {
    println!("=== Resource-Aware Sync Example ===\n");

    // Initialize state engine and Willow adapter
    let engine = Arc::new(StateEngine::new().await?);
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await?;

    // Create root capability
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    println!("1. Creating Sample Documents\n");

    // Create many documents to demonstrate resource constraints
    let num_docs = 50;
    for i in 0..num_docs {
        let doc_id = DocumentId::new("users", &format!("user{:04}", i));
        let handle = engine.create_document(doc_id).await?;
        handle.update(|doc| {
            doc.put(ROOT, "id", i as i64)?;
            doc.put(ROOT, "name", format!("User {}", i))?;
            doc.put(ROOT, "email", format!("user{}@example.com", i))?;
            doc.put(ROOT, "bio", "Lorem ipsum dolor sit amet, consectetur adipiscing elit.")?;
            Ok(())
        })?;
    }

    println!("   Created {} user documents", num_docs);
    println!("   Total size: ~{} KB\n", engine.stats().total_document_size / 1024);

    // Scenario 1: High-priority sync (user-initiated)
    println!("2. High-Priority Sync (User-Initiated)\n");

    let high_priority_constraints = ResourceConstraints {
        max_memory: 200 * 1024,      // 200 KB
        max_bandwidth: 1024 * 1024,  // 1 MB/s
        priority: SyncPriority::High,
    };

    println!("   Constraints:");
    println!("     Max memory: {} KB", high_priority_constraints.max_memory / 1024);
    println!("     Max bandwidth: {} KB/s", high_priority_constraints.max_bandwidth / 1024);
    println!("     Priority: High");
    println!();

    let start = std::time::Instant::now();
    let stats = adapter
        .sync_with_constraints("myapp.v1", "users", &root_cap, high_priority_constraints)
        .await?;
    let duration = start.elapsed();

    println!("   Results:");
    println!("     Synced: {} documents", stats.synced_count);
    println!("     Total size: {} KB", stats.total_bytes / 1024);
    println!("     Errors: {}", stats.errors);
    println!("     Duration: {:?}\n", duration);

    // Clear Willow storage
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await?;

    // Scenario 2: Low-priority background sync
    println!("3. Low-Priority Background Sync\n");

    let low_priority_constraints = ResourceConstraints {
        max_memory: 50 * 1024,       // 50 KB (very constrained)
        max_bandwidth: 128 * 1024,   // 128 KB/s
        priority: SyncPriority::Low,
    };

    println!("   Constraints:");
    println!("     Max memory: {} KB", low_priority_constraints.max_memory / 1024);
    println!("     Max bandwidth: {} KB/s", low_priority_constraints.max_bandwidth / 1024);
    println!("     Priority: Low");
    println!();

    let start = std::time::Instant::now();
    let stats = adapter
        .sync_with_constraints("myapp.v1", "users", &root_cap, low_priority_constraints)
        .await?;
    let duration = start.elapsed();

    println!("   Results:");
    println!("     Synced: {} documents", stats.synced_count);
    println!("     Total size: {} KB", stats.total_bytes / 1024);
    println!("     Errors: {}", stats.errors);
    println!("     Duration: {:?}\n", duration);

    // Clear again
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await?;

    // Scenario 3: Medium-priority opportunistic sync
    println!("4. Medium-Priority Opportunistic Sync\n");

    let medium_priority_constraints = ResourceConstraints {
        max_memory: 100 * 1024,      // 100 KB
        max_bandwidth: 512 * 1024,   // 512 KB/s
        priority: SyncPriority::Medium,
    };

    println!("   Constraints:");
    println!("     Max memory: {} KB", medium_priority_constraints.max_memory / 1024);
    println!("     Max bandwidth: {} KB/s", medium_priority_constraints.max_bandwidth / 1024);
    println!("     Priority: Medium");
    println!();

    let start = std::time::Instant::now();
    let stats = adapter
        .sync_with_constraints("myapp.v1", "users", &root_cap, medium_priority_constraints)
        .await?;
    let duration = start.elapsed();

    println!("   Results:");
    println!("     Synced: {} documents", stats.synced_count);
    println!("     Total size: {} KB", stats.total_bytes / 1024);
    println!("     Errors: {}", stats.errors);
    println!("     Duration: {:?}\n", duration);

    println!("=== Resource-Aware Sync Complete ===\n");
    println!("Key Observations:");
    println!("  • Higher memory limits allow more documents to sync");
    println!("  • Priority affects scheduling (not shown in this example)");
    println!("  • Sync respects memory constraints");
    println!("  • Partial sync allows incremental synchronization");
    println!("  • Ideal for mobile devices with limited resources");

    Ok(())
}
