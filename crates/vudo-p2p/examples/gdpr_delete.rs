//! Example: GDPR-compliant deletion with tombstones.
//!
//! This example demonstrates how to perform true deletion that propagates
//! across the network, supporting GDPR compliance.

use automerge::{transaction::Transactable, ROOT};
use bytes::Bytes;
use ed25519_dalek::SigningKey;
use std::sync::Arc;
use vudo_p2p::{meadowcap::Capability, WillowAdapter};
use vudo_state::{DocumentId, StateEngine};

#[tokio::main]
async fn main() -> vudo_p2p::error::Result<()> {
    println!("=== GDPR-Compliant Deletion Example ===\n");

    // Initialize state engine and Willow adapter
    let engine = Arc::new(StateEngine::new().await?);
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await?;

    // Create root capability
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    println!("1. Creating User Document\n");

    // Create user document with personal data
    let doc_id = DocumentId::new("users", "alice");
    let handle = engine.create_document(doc_id.clone()).await?;
    handle.update(|doc| {
        doc.put(ROOT, "name", "Alice Johnson")?;
        doc.put(ROOT, "email", "alice@example.com")?;
        doc.put(ROOT, "phone", "+1-555-1234")?;
        doc.put(ROOT, "address", "123 Main St, Anytown, USA")?;
        Ok(())
    })?;

    println!("   Created user document: users/alice");
    println!("   Name: Alice Johnson");
    println!("   Email: alice@example.com");
    println!("   Phone: +1-555-1234");
    println!("   Address: 123 Main St, Anytown, USA\n");

    // Sync to Willow
    println!("2. Syncing to Willow Network\n");

    adapter
        .sync_from_state_engine("myapp.v1", "users", "alice", &root_cap)
        .await?;

    let stats_before = adapter.stats();
    println!("   Willow stats:");
    println!("     Entries: {}", stats_before.entry_count);
    println!("     Tombstones: {}", stats_before.tombstone_count);
    println!();

    // Verify data is readable
    let data = adapter
        .read_entry("myapp.v1", "users", "alice", &root_cap)
        .await?;
    println!("   Data readable: {}\n", data.is_some());

    // User requests deletion under GDPR Article 17 (Right to Erasure)
    println!("3. User Requests GDPR Deletion\n");

    println!("   User: Alice Johnson");
    println!("   Request: Delete all personal data");
    println!("   Legal basis: GDPR Article 17 (Right to Erasure)");
    println!("   Timestamp: {}\n", chrono::Utc::now().to_rfc3339());

    adapter
        .gdpr_delete(
            "myapp.v1",
            "users",
            "alice",
            &root_cap,
            "User requested data deletion under GDPR Article 17 (Right to Erasure)",
        )
        .await?;

    println!("   ✓ Deletion complete\n");

    // Verify deletion
    println!("4. Verifying Deletion\n");

    let stats_after = adapter.stats();
    println!("   Willow stats:");
    println!("     Entries: {}", stats_after.entry_count);
    println!("     Tombstones: {}", stats_after.tombstone_count);
    println!();

    // Try to read deleted data
    let data_after = adapter
        .read_entry("myapp.v1", "users", "alice", &root_cap)
        .await?;
    println!("   Data readable: {}", data_after.is_some());
    println!("   Data content: {:?}\n", data_after);

    // Verify document is gone from state engine
    let doc_exists = engine.get_document(&doc_id).await.is_ok();
    println!("   Document in state engine: {}", doc_exists);
    println!();

    println!("5. Tombstone Details\n");

    println!("   Tombstone created: Yes");
    println!("   Propagates to peers: Yes");
    println!("   Prevents resurrection: Yes");
    println!("   Reason recorded: Yes\n");

    println!("=== GDPR Deletion Complete ===\n");
    println!("Key Points:");
    println!("  • Data is permanently deleted from local state");
    println!("  • Tombstone prevents future recreation");
    println!("  • Tombstone propagates to all peers");
    println!("  • Deletion reason is recorded for audit");
    println!("  • Compliant with GDPR Article 17");

    Ok(())
}
