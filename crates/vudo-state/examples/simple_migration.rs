//! Simple Migration Example
//!
//! Demonstrates basic schema evolution from v1 to v2.

use automerge::transaction::Transactable;
use automerge::{ActorId, Automerge, ReadDoc, ROOT};
use semver::Version;
use std::sync::Arc;
use vudo_state::schema_evolution::{
    EvolutionEngine, Migration, MigrationMetadata, SchemaMetadata, SchemaVersion,
};
use vudo_state::{DocumentId, StateEngine};

/// Migration from v1 to v2: adds email field
#[derive(Debug, Clone)]
struct AddEmailField;

#[async_trait::async_trait]
impl Migration for AddEmailField {
    async fn migrate(&self, doc: &mut Automerge) -> vudo_state::Result<()> {
        println!("🔄 Migrating: Adding email field");

        let mut tx = doc.transaction();
        tx.set_actor(ActorId::from(vec![0u8; 32])); // Deterministic

        if tx.get(&ROOT, "email")?.is_none() {
            tx.put(&ROOT, "email", "")?;
            println!("  ✓ Email field added");
        } else {
            println!("  ℹ Email field already exists (idempotent)");
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

#[tokio::main]
async fn main() -> vudo_state::Result<()> {
    println!("📚 Simple Migration Example\n");

    // Initialize state engine
    let state_engine = Arc::new(StateEngine::new().await?);
    println!("✓ State engine initialized");

    // Create a v1 document
    let doc_id = DocumentId::new("users", "alice");
    let handle = state_engine.create_document(doc_id.clone()).await?;
    println!("✓ Document created: {}/{}", doc_id.namespace, doc_id.id);

    // Populate v1 document
    handle.update(|tx| {
        tx.put(&ROOT, "username", "alice")?;
        tx.put(&ROOT, "age", 30i64)?;

        // Embed schema version
        let schema_obj = tx.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
        tx.put(&schema_obj, "gen_name", "users")?;
        tx.put(&schema_obj, "version", "1.0.0")?;

        Ok(())
    })?;
    println!("✓ Document populated with v1 schema");

    // Display v1 document
    println!("\n📄 Document (v1.0.0):");
    handle.read(|doc| {
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "username")? {
            if let automerge::ScalarValue::Str(username) = s.as_ref() {
                println!("  username: {}", username);
            }
        }
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "age")? {
            if let automerge::ScalarValue::Int(age) = s.as_ref() {
                println!("  age: {}", age);
            }
        }
        println!("  email: <not present>");
        Ok(())
    })?;

    // Register schema with migration
    println!("\n🔧 Registering schema evolution");
    let evolution_engine = EvolutionEngine::new(Arc::clone(&state_engine));

    let mut metadata = SchemaMetadata::new(SchemaVersion::new(
        "users".to_string(),
        Version::new(2, 0, 0),
        [0u8; 32],
    ));
    metadata.add_migration(Arc::new(AddEmailField));
    evolution_engine.register_schema(metadata);
    println!("✓ Schema registered: users v2.0.0");

    // Load with lazy migration
    println!("\n⚡ Loading document with lazy migration");
    let migrated_handle = evolution_engine
        .load_with_migration("users", "alice")
        .await?;

    // Display v2 document
    println!("\n📄 Document (v2.0.0):");
    migrated_handle.read(|doc| {
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "username")? {
            if let automerge::ScalarValue::Str(username) = s.as_ref() {
                println!("  username: {}", username);
            }
        }
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "age")? {
            if let automerge::ScalarValue::Int(age) = s.as_ref() {
                println!("  age: {}", age);
            }
        }
        if let Some((automerge::Value::Scalar(s), _)) = doc.get(&ROOT, "email")? {
            if let automerge::ScalarValue::Str(email) = s.as_ref() {
                println!("  email: \"{}\" (added by migration)", email);
            }
        }

        // Display schema version
        if let Some((automerge::Value::Object(_), obj_id)) =
            doc.get(&ROOT, "__schema_version")?
        {
            if let Some((automerge::Value::Scalar(s), _)) = doc.get(obj_id, "version")? {
                if let automerge::ScalarValue::Str(version_str) = s.as_ref() {
                    println!("  __schema_version: {}", version_str);
                }
            }
        }

        Ok(())
    })?;

    println!("\n✅ Migration complete!");
    println!("📝 Note: Migration was applied lazily on read (not proactively)");

    Ok(())
}
