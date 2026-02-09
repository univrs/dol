//! Migration Chain Example
//!
//! Demonstrates a migration chain: v1 → v2 → v3

use automerge::transaction::Transactable;
use automerge::{ActorId, Automerge, ReadDoc, ROOT};
use semver::Version;
use std::sync::Arc;
use vudo_state::schema_evolution::{
    EvolutionEngine, Migration, MigrationMetadata, SchemaMetadata, SchemaVersion,
};
use vudo_state::{DocumentId, StateEngine};

/// v1 → v2: Add email field
#[derive(Debug, Clone)]
struct AddEmailField;

#[async_trait::async_trait]
impl Migration for AddEmailField {
    async fn migrate(&self, doc: &mut Automerge) -> vudo_state::Result<()> {
        println!("  🔄 Migration 1/2: Adding email field");
        let mut tx = doc.transaction();
        tx.set_actor(ActorId::from(vec![0u8; 32]));

        if tx.get(&ROOT, "email")?.is_none() {
            tx.put(&ROOT, "email", "")?;
        }

        tx.commit();
        Ok(())
    }

    fn can_migrate(&self, doc: &Automerge) -> bool {
        doc.get(&ROOT, "email").ok().flatten().is_none()
    }

    fn metadata(&self) -> &MigrationMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<MigrationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MigrationMetadata::new(
                "AddEmailField".to_string(),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
            )
        })
    }
}

/// v2 → v3: Rename username to display_name
#[derive(Debug, Clone)]
struct RenameUsername;

#[async_trait::async_trait]
impl Migration for RenameUsername {
    async fn migrate(&self, doc: &mut Automerge) -> vudo_state::Result<()> {
        println!("  🔄 Migration 2/2: Renaming username → display_name");
        let mut tx = doc.transaction();
        tx.set_actor(ActorId::from(vec![0u8; 32]));

        if let Some((value, _)) = tx.get(&ROOT, "username")? {
            tx.put(&ROOT, "display_name", value)?;
            tx.delete(&ROOT, "username")?;
        }

        tx.commit();
        Ok(())
    }

    fn can_migrate(&self, doc: &Automerge) -> bool {
        doc.get(&ROOT, "username").ok().flatten().is_some()
    }

    fn metadata(&self) -> &MigrationMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<MigrationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MigrationMetadata::new(
                "RenameUsername".to_string(),
                Version::new(2, 0, 0),
                Version::new(3, 0, 0),
            )
        })
    }
}

#[tokio::main]
async fn main() -> vudo_state::Result<()> {
    println!("📚 Migration Chain Example\n");

    // Initialize
    let state_engine = Arc::new(StateEngine::new().await?);
    println!("✓ State engine initialized");

    // Create v1 document
    let doc_id = DocumentId::new("users", "bob");
    let handle = state_engine.create_document(doc_id.clone()).await?;

    handle.update(|tx| {
        tx.put(&ROOT, "username", "bob")?;
        tx.put(&ROOT, "age", 25i64)?;

        let schema_obj = tx.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
        tx.put(&schema_obj, "gen_name", "users")?;
        tx.put(&schema_obj, "version", "1.0.0")?;
        Ok(())
    })?;

    println!("✓ Document created with v1.0.0 schema");

    // Display v1
    println!("\n📄 Document (v1.0.0):");
    handle.read(|doc| {
        println!("  username: bob");
        println!("  age: 25");
        println!("  email: <not present>");
        Ok(())
    })?;

    // Register schema with migration chain
    println!("\n🔧 Registering migration chain: v1 → v2 → v3");
    let evolution_engine = EvolutionEngine::new(Arc::clone(&state_engine));

    let mut metadata = SchemaMetadata::new(SchemaVersion::new(
        "users".to_string(),
        Version::new(3, 0, 0),
        [0u8; 32],
    ));
    metadata.add_migration(Arc::new(AddEmailField)); // v1 → v2
    metadata.add_migration(Arc::new(RenameUsername)); // v2 → v3
    evolution_engine.register_schema(metadata);

    println!("✓ Migration chain registered");

    // Load with migration
    println!("\n⚡ Applying migration chain:");
    let migrated_handle = evolution_engine.load_with_migration("users", "bob").await?;

    // Display v3
    println!("\n📄 Document (v3.0.0):");
    migrated_handle.read(|doc| {
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "display_name")? {
            if let automerge::ScalarValue::Str(name) = s.as_ref() {
                println!("  display_name: {} (renamed from username)", name);
            }
        }
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "age")? {
            if let automerge::ScalarValue::Int(age) = s.as_ref() {
                println!("  age: {}", age);
            }
        }
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "email")? {
            if let automerge::ScalarValue::Str(email) = s.as_ref() {
                println!("  email: \"{}\" (added)", email);
            }
        }

        // Verify old field is gone
        if doc.get(&ROOT, "username")?.is_none() {
            println!("  username: <removed>");
        }

        Ok(())
    })?;

    println!("\n✅ Migration chain complete!");
    println!("📝 Two migrations applied sequentially: v1→v2→v3");

    Ok(())
}
