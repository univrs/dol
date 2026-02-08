//! Integration tests for schema evolution.

use automerge::transaction::Transactable;
use automerge::{ActorId, Automerge, ReadDoc, ROOT};
use semver::Version;
use std::sync::Arc;
use vudo_state::schema_evolution::{
    EvolutionEngine, ForwardCompatibleReader, Migration, MigrationConflictResolver,
    MigrationMetadata, SchemaMetadata, SchemaVersion,
};
use vudo_state::{DocumentId, StateEngine};

/// Simple test migration that adds an "email" field.
#[derive(Debug, Clone)]
struct AddEmailField;

#[async_trait::async_trait]
impl Migration for AddEmailField {
    async fn migrate(&self, doc: &mut Automerge) -> vudo_state::Result<()> {
        let mut tx = doc.transaction();
        tx.set_actor(ActorId::from(vec![0u8; 32])); // Deterministic

        // Add email field if it doesn't exist
        if tx.get(&ROOT, "email")?.is_none() {
            tx.put(&ROOT, "email", "")?;
        }

        tx.commit();
        Ok(())
    }

    fn can_migrate(&self, doc: &Automerge) -> bool {
        // Can migrate if email field doesn't exist
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

/// Migration that renames "username" to "display_name".
#[derive(Debug, Clone)]
struct RenameUsername;

#[async_trait::async_trait]
impl Migration for RenameUsername {
    async fn migrate(&self, doc: &mut Automerge) -> vudo_state::Result<()> {
        let mut tx = doc.transaction();
        tx.set_actor(ActorId::from(vec![0u8; 32]));

        // Rename username -> display_name
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

#[tokio::test]
async fn test_schema_version_embedding() {
    let mut doc = Automerge::new();
    let version = SchemaVersion::new(
        "user.profile".to_string(),
        Version::new(1, 0, 0),
        [0u8; 32],
    );

    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let evolution_engine = EvolutionEngine::new(state_engine);

    evolution_engine.embed_version(&mut doc, &version).unwrap();

    // Verify version was embedded
    match doc.get(&ROOT, "__schema_version").unwrap() {
        Some((automerge::Value::Object(_), obj_id)) => {
            match doc.get(obj_id, "version").unwrap() {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(version_str) = s.as_ref() {
                        assert_eq!(version_str.to_string(), "1.0.0");
                    } else {
                        panic!("Version is not a string");
                    }
                }
                _ => panic!("Version field not found"),
            }
        }
        _ => panic!("__schema_version not found"),
    }
}

#[tokio::test]
async fn test_simple_migration() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let doc_id = DocumentId::new("users", "alice");
    let handle = state_engine.create_document(doc_id.clone()).await.unwrap();

    // Create v1 document
    handle
        .update(|tx| {
            tx.put(&ROOT, "username", "alice")?;
            tx.put(&ROOT, "age", 30i64)?;

            // Embed schema version
            let schema_obj = tx.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
            tx.put(&schema_obj, "gen_name", "users")?;
            tx.put(&schema_obj, "version", "1.0.0")?;
            Ok(())
        })
        .unwrap();

    // Register schema with migration
    let evolution_engine = EvolutionEngine::new(Arc::clone(&state_engine));
    let mut metadata = SchemaMetadata::new(SchemaVersion::new(
        "users".to_string(),
        Version::new(2, 0, 0),
        [0u8; 32],
    ));
    metadata.add_migration(Arc::new(AddEmailField));
    evolution_engine.register_schema(metadata);

    // Load with migration
    let migrated_handle = evolution_engine
        .load_with_migration("users", "alice")
        .await
        .unwrap();

    // Verify email field was added
    migrated_handle
        .read(|doc| {
            match doc.get(&ROOT, "email")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(email) = s.as_ref() {
                        assert_eq!(email.to_string(), "");
                    } else {
                        panic!("Email is not a string");
                    }
                }
                _ => panic!("Email field not found"),
            }

            // Verify schema version was updated
            match doc.get(&ROOT, "__schema_version")? {
                Some((automerge::Value::Object(_), obj_id)) => {
                    match doc.get(obj_id, "version")? {
                        Some((automerge::Value::Scalar(s), _)) => {
                            if let automerge::ScalarValue::Str(version_str) = s.as_ref() {
                                assert_eq!(version_str.to_string(), "2.0.0");
                            } else {
                                panic!("Version is not a string");
                            }
                        }
                        _ => panic!("Version field not found"),
                    }
                }
                _ => panic!("__schema_version not found"),
            }

            Ok(())
        })
        .unwrap();
}

#[tokio::test]
async fn test_migration_chain() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let doc_id = DocumentId::new("users", "bob");
    let handle = state_engine.create_document(doc_id.clone()).await.unwrap();

    // Create v1 document
    handle
        .update(|tx| {
            tx.put(&ROOT, "username", "bob")?;

            let schema_obj = tx.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
            tx.put(&schema_obj, "gen_name", "users")?;
            tx.put(&schema_obj, "version", "1.0.0")?;
            Ok(())
        })
        .unwrap();

    // Register schema with migration chain: v1 → v2 → v3
    let evolution_engine = EvolutionEngine::new(Arc::clone(&state_engine));
    let mut metadata = SchemaMetadata::new(SchemaVersion::new(
        "users".to_string(),
        Version::new(3, 0, 0),
        [0u8; 32],
    ));
    metadata.add_migration(Arc::new(AddEmailField)); // v1 → v2
    metadata.add_migration(Arc::new(RenameUsername)); // v2 → v3
    evolution_engine.register_schema(metadata);

    // Load with migration
    let migrated_handle = evolution_engine
        .load_with_migration("users", "bob")
        .await
        .unwrap();

    // Verify both migrations were applied
    migrated_handle
        .read(|doc| {
            // Email added (v1 → v2)
            assert!(doc.get(&ROOT, "email")?.is_some());

            // Username renamed (v2 → v3)
            assert!(doc.get(&ROOT, "username")?.is_none());
            match doc.get(&ROOT, "display_name")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(name) = s.as_ref() {
                        assert_eq!(name.to_string(), "bob");
                    }
                }
                _ => panic!("display_name not found"),
            }

            // Version updated to v3
            match doc.get(&ROOT, "__schema_version")? {
                Some((automerge::Value::Object(_), obj_id)) => {
                    match doc.get(obj_id, "version")? {
                        Some((automerge::Value::Scalar(s), _)) => {
                            if let automerge::ScalarValue::Str(version_str) = s.as_ref() {
                                assert_eq!(version_str.to_string(), "3.0.0");
                            }
                        }
                        _ => panic!("Version field not found"),
                    }
                }
                _ => panic!("__schema_version not found"),
            }

            Ok(())
        })
        .unwrap();
}

#[tokio::test]
async fn test_migration_idempotence() {
    let mut doc = Automerge::new();

    // Apply migration twice
    let migration = AddEmailField;
    migration.migrate(&mut doc).await.unwrap();
    migration.migrate(&mut doc).await.unwrap();

    // Should still have only one email field
    match doc.get(&ROOT, "email").unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let automerge::ScalarValue::Str(email) = s.as_ref() {
                assert_eq!(email.to_string(), "");
            }
        }
        _ => panic!("Email field not found"),
    }
}

#[tokio::test]
async fn test_deterministic_migration() {
    // Two peers apply the same migration independently
    let mut doc1 = Automerge::new();
    let mut doc2 = Automerge::new();

    // Both documents start with same state
    {
        let mut tx1 = doc1.transaction();
        tx1.put(&ROOT, "username", "alice")?;
        tx1.commit();

        let mut tx2 = doc2.transaction();
        tx2.put(&ROOT, "username", "alice")?;
        tx2.commit();
    }

    // Apply migration on both peers
    let migration = AddEmailField;
    migration.migrate(&mut doc1).await.unwrap();
    migration.migrate(&mut doc2).await.unwrap();

    // Merge the documents
    doc1.merge(&doc2).unwrap();

    // Should have no conflicts (deterministic migrations)
    match doc1.get(&ROOT, "email").unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let automerge::ScalarValue::Str(email) = s.as_ref() {
                assert_eq!(email.to_string(), "");
            }
        }
        _ => panic!("Email field not found"),
    }
}

#[tokio::test]
async fn test_migration_conflict_resolution() {
    let resolver = MigrationConflictResolver::new();

    let mut doc1 = Automerge::new();
    let mut doc2 = Automerge::new();

    // Apply migration to both
    let migration = AddEmailField;
    migration.migrate(&mut doc1).await.unwrap();
    migration.migrate(&mut doc2).await.unwrap();

    // Embed schema versions
    {
        let mut tx1 = doc1.transaction();
        let schema_obj = tx1.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
        tx1.put(&schema_obj, "version", "2.0.0")?;
        tx1.commit();

        let mut tx2 = doc2.transaction();
        let schema_obj = tx2.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
        tx2.put(&schema_obj, "version", "2.0.0")?;
        tx2.commit();
    }

    // Resolve
    let merged = resolver.resolve(&doc1, &doc2).unwrap();

    // Verify version
    let version = resolver.verify_version(&doc1, &doc2).unwrap();
    assert_eq!(version, Version::new(2, 0, 0));

    // Merged document should have email field
    assert!(merged.get(&ROOT, "email").unwrap().is_some());
}

#[tokio::test]
async fn test_forward_compatible_reader() {
    use std::collections::HashSet;

    let mut known_fields = HashSet::new();
    known_fields.insert("username".to_string());
    known_fields.insert("age".to_string());

    let reader = ForwardCompatibleReader::new(known_fields);

    let mut doc = Automerge::new();
    {
        let mut tx = doc.transaction();
        tx.put(&ROOT, "username", "alice")?;
        tx.put(&ROOT, "age", 30i64)?;
        tx.put(&ROOT, "email", "alice@example.com")?; // Unknown field
        tx.commit();
    }

    // Forward-compatible read (ignores "email")
    #[derive(Debug, serde::Deserialize)]
    struct UserV1 {
        username: String,
        age: i64,
    }

    let user: UserV1 = reader.read_document(&doc).unwrap();
    assert_eq!(user.username, "alice");
    assert_eq!(user.age, 30);
}

#[tokio::test]
async fn test_lazy_migration_only_on_read() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let doc_id = DocumentId::new("users", "charlie");
    let handle = state_engine.create_document(doc_id.clone()).await.unwrap();

    // Create v1 document
    handle
        .update(|tx| {
            tx.put(&ROOT, "username", "charlie")?;

            let schema_obj = tx.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
            tx.put(&schema_obj, "gen_name", "users")?;
            tx.put(&schema_obj, "version", "1.0.0")?;
            Ok(())
        })
        .unwrap();

    // Register schema
    let evolution_engine = EvolutionEngine::new(Arc::clone(&state_engine));
    let mut metadata = SchemaMetadata::new(SchemaVersion::new(
        "users".to_string(),
        Version::new(2, 0, 0),
        [0u8; 32],
    ));
    metadata.add_migration(Arc::new(AddEmailField));
    evolution_engine.register_schema(metadata);

    // Document is NOT migrated yet (lazy migration)
    handle
        .read(|doc| {
            // Version is still 1.0.0
            match doc.get(&ROOT, "__schema_version")? {
                Some((automerge::Value::Object(_), obj_id)) => {
                    match doc.get(obj_id, "version")? {
                        Some((automerge::Value::Scalar(s), _)) => {
                            if let automerge::ScalarValue::Str(version_str) = s.as_ref() {
                                assert_eq!(version_str.to_string(), "1.0.0");
                            }
                        }
                        _ => panic!("Version field not found"),
                    }
                }
                _ => panic!("__schema_version not found"),
            }
            Ok(())
        })
        .unwrap();

    // Migrate on read
    let migrated_handle = evolution_engine
        .load_with_migration("users", "charlie")
        .await
        .unwrap();

    // Now it's migrated
    migrated_handle
        .read(|doc| {
            assert!(doc.get(&ROOT, "email")?.is_some());
            Ok(())
        })
        .unwrap();
}

#[tokio::test]
async fn test_concurrent_migration_same_document() {
    // Simulate two peers migrating the same document concurrently
    let state_engine1 = Arc::new(StateEngine::new().await.unwrap());
    let state_engine2 = Arc::new(StateEngine::new().await.unwrap());

    // Create same document on both peers
    let doc_id1 = DocumentId::new("users", "david");
    let doc_id2 = DocumentId::new("users", "david");

    let handle1 = state_engine1.create_document(doc_id1).await.unwrap();
    let handle2 = state_engine2.create_document(doc_id2).await.unwrap();

    // Both create v1 documents
    for handle in [&handle1, &handle2] {
        handle
            .update(|tx| {
                tx.put(&ROOT, "username", "david")?;

                let schema_obj =
                    tx.put_object(&ROOT, "__schema_version", automerge::ObjType::Map)?;
                tx.put(&schema_obj, "gen_name", "users")?;
                tx.put(&schema_obj, "version", "1.0.0")?;
                Ok(())
            })
            .unwrap();
    }

    // Both peers migrate to v2
    let migration = Arc::new(AddEmailField);

    handle1
        .update(|tx| {
            let mut doc = tx.document().clone();
            futures::executor::block_on(migration.migrate(&mut doc))?;
            tx.merge(&doc)?;

            // Update version
            match tx.get(&ROOT, "__schema_version")? {
                Some((automerge::Value::Object(_), obj_id)) => {
                    tx.put(obj_id, "version", "2.0.0")?;
                }
                _ => panic!("Schema version not found"),
            }
            Ok(())
        })
        .unwrap();

    handle2
        .update(|tx| {
            let mut doc = tx.document().clone();
            futures::executor::block_on(migration.migrate(&mut doc))?;
            tx.merge(&doc)?;

            match tx.get(&ROOT, "__schema_version")? {
                Some((automerge::Value::Object(_), obj_id)) => {
                    tx.put(obj_id, "version", "2.0.0")?;
                }
                _ => panic!("Schema version not found"),
            }
            Ok(())
        })
        .unwrap();

    // Merge documents
    handle1
        .update(|tx| {
            let doc2 = handle2.read(|d| Ok(d.clone()))?;
            tx.merge(&doc2)?;
            Ok(())
        })
        .unwrap();

    // No conflicts - deterministic migration
    handle1
        .read(|doc| {
            assert!(doc.get(&ROOT, "email")?.is_some());
            Ok(())
        })
        .unwrap();
}
