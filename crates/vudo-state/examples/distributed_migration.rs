//! Distributed Migration Example
//!
//! Demonstrates two peers independently migrating the same document,
//! then syncing without conflicts (deterministic migrations).

use automerge::transaction::Transactable;
use automerge::{ActorId, Automerge, ReadDoc, ROOT};
use semver::Version;
use std::sync::Arc;
use vudo_state::schema_evolution::{
    Migration, MigrationConflictResolver, MigrationMetadata,
};
use vudo_state::{DocumentId, StateEngine};

/// Migration: Add profile_photo field
#[derive(Debug, Clone)]
struct AddProfilePhoto;

#[async_trait::async_trait]
impl Migration for AddProfilePhoto {
    async fn migrate(&self, doc: &mut Automerge) -> vudo_state::Result<()> {
        let mut tx = doc.transaction();
        tx.set_actor(ActorId::from(vec![0u8; 32])); // CRITICAL: Deterministic actor ID

        if tx.get(&ROOT, "profile_photo")?.is_none() {
            tx.put(&ROOT, "profile_photo", "")?;
        }

        tx.commit();
        Ok(())
    }

    fn can_migrate(&self, _doc: &Automerge) -> bool {
        true
    }

    fn metadata(&self) -> &MigrationMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<MigrationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MigrationMetadata::new(
                "AddProfilePhoto".to_string(),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
            )
        })
    }
}

#[tokio::main]
async fn main() -> vudo_state::Result<()> {
    println!("📚 Distributed Migration Example\n");
    println!("Scenario: Two offline peers independently migrate the same document\n");

    // Create two independent state engines (simulating two peers)
    let state_engine_peer1 = Arc::new(StateEngine::new().await?);
    let state_engine_peer2 = Arc::new(StateEngine::new().await?);

    println!("✓ Two peers initialized");

    // Peer 1 creates the document
    let doc_id_peer1 = DocumentId::new("users", "charlie");
    let handle_peer1 = state_engine_peer1
        .create_document(doc_id_peer1.clone())
        .await?;

    handle_peer1.update(|tx| {
        tx.put(&ROOT, "username", "charlie")?;
        tx.put(&ROOT, "bio", "Software engineer")?;

        let schema_obj = tx.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
        tx.put(&schema_obj, "version", "1.0.0")?;
        Ok(())
    })?;

    println!("✓ Peer 1 created document (v1.0.0)");

    // Peer 2 gets a copy (simulating sync)
    let doc_id_peer2 = DocumentId::new("users", "charlie");
    let handle_peer2 = state_engine_peer2
        .create_document(doc_id_peer2.clone())
        .await?;

    let doc_snapshot = handle_peer1.read(|doc| Ok(doc.clone()))?;
    handle_peer2.update(|tx| {
        tx.merge(&doc_snapshot)?;
        Ok(())
    })?;

    println!("✓ Peer 2 received copy via sync");

    // Network partition! Both peers go offline
    println!("\n🔌 Network partition: Peers 1 and 2 go offline");

    // Peer 1 applies migration
    println!("\n👤 Peer 1: Applying migration");
    let migration = Arc::new(AddProfilePhoto);

    handle_peer1.update(|tx| {
        let mut doc = tx.document().clone();
        futures::executor::block_on(migration.migrate(&mut doc))?;
        tx.merge(&doc)?;

        // Update schema version
        match tx.get(&ROOT, "__schema_version")? {
            Some((automerge::Value::Object(_), obj_id)) => {
                tx.put(obj_id, "version", "2.0.0")?;
            }
            _ => {}
        }
        Ok(())
    })?;

    println!("  ✓ Peer 1 migrated to v2.0.0");

    // Peer 2 ALSO applies the same migration (independently!)
    println!("\n👤 Peer 2: Applying migration (independently)");

    handle_peer2.update(|tx| {
        let mut doc = tx.document().clone();
        futures::executor::block_on(migration.migrate(&mut doc))?;
        tx.merge(&doc)?;

        match tx.get(&ROOT, "__schema_version")? {
            Some((automerge::Value::Object(_), obj_id)) => {
                tx.put(obj_id, "version", "2.0.0")?;
            }
            _ => {}
        }
        Ok(())
    })?;

    println!("  ✓ Peer 2 migrated to v2.0.0");

    // Both peers make independent edits
    println!("\n✏️  Concurrent edits during partition:");

    handle_peer1.update(|tx| {
        tx.put(&ROOT, "last_seen", "2024-02-05")?;
        Ok(())
    })?;
    println!("  Peer 1: Added last_seen field");

    handle_peer2.update(|tx| {
        tx.put(&ROOT, "status", "active")?;
        Ok(())
    })?;
    println!("  Peer 2: Added status field");

    // Network heals! Peers sync
    println!("\n🌐 Network reconnects: Syncing peers");

    let doc1 = handle_peer1.read(|doc| Ok(doc.clone()))?;
    let doc2 = handle_peer2.read(|doc| Ok(doc.clone()))?;

    // Merge using conflict resolver
    let resolver = MigrationConflictResolver::new();
    let merged = resolver.resolve(&doc1, &doc2)?;

    println!("  ✓ Documents merged successfully");

    // Verify schema version (should be v2.0.0 on both)
    let version = resolver.verify_version(&doc1, &doc2)?;
    println!("  ✓ Schema version verified: {}", version);

    // Display merged document
    println!("\n📄 Merged Document (v2.0.0):");
    if let Some((automerge::Value::Scalar(s), _)) = merged.get(&ROOT, "username")? {
        if let automerge::ScalarValue::Str(username) = s.as_ref() {
            println!("  username: {}", username);
        }
    }
    if let Some((automerge::Value::Scalar(s), _)) = merged.get(&ROOT, "bio")? {
        if let automerge::ScalarValue::Str(bio) = s.as_ref() {
            println!("  bio: {}", bio);
        }
    }
    if let Some((automerge::Value::Scalar(s), _)) = merged.get(&ROOT, "profile_photo")? {
        if let automerge::ScalarValue::Str(photo) = s.as_ref() {
            println!("  profile_photo: \"{}\" (added by migration)", photo);
        }
    }
    if let Some((automerge::Value::Scalar(s), _)) = merged.get(&ROOT, "last_seen")? {
        if let automerge::ScalarValue::Str(date) = s.as_ref() {
            println!("  last_seen: {} (from Peer 1)", date);
        }
    }
    if let Some((automerge::Value::Scalar(s), _)) = merged.get(&ROOT, "status")? {
        if let automerge::ScalarValue::Str(status) = s.as_ref() {
            println!("  status: {} (from Peer 2)", status);
        }
    }

    println!("\n✅ Distributed migration complete!");
    println!("📝 Key insights:");
    println!("   • Both peers independently migrated to v2.0.0");
    println!("   • Deterministic actor ID prevented conflicts");
    println!("   • Concurrent edits merged successfully");
    println!("   • CRDT semantics maintained consistency");

    Ok(())
}
