//! Property-based tests for migrations.
//!
//! These tests verify that migrations satisfy key properties:
//! - Idempotence: migrate(migrate(doc)) = migrate(doc)
//! - Commutativity: migrate(A) + migrate(B) = migrate(B) + migrate(A)
//! - Determinism: Same input → Same output on all peers

use automerge::transaction::Transactable;
use automerge::{ActorId, Automerge, ReadDoc, ROOT};
use proptest::prelude::*;
use semver::Version;
use vudo_state::schema_evolution::{Migration, MigrationMetadata};

/// Test migration that adds a field with a configurable value.
#[derive(Debug, Clone)]
struct AddFieldMigration {
    field_name: String,
    field_value: String,
}

impl AddFieldMigration {
    fn new(field_name: String, field_value: String) -> Self {
        Self {
            field_name,
            field_value,
        }
    }
}

#[async_trait::async_trait]
impl Migration for AddFieldMigration {
    async fn migrate(&self, doc: &mut Automerge) -> vudo_state::Result<()> {
        let mut tx = doc.transaction();
        tx.set_actor(ActorId::from(vec![0u8; 32]));

        if tx.get(&ROOT, &self.field_name)?.is_none() {
            tx.put(&ROOT, self.field_name.clone(), self.field_value.clone())?;
        }

        tx.commit();
        Ok(())
    }

    fn can_migrate(&self, doc: &Automerge) -> bool {
        doc.get(&ROOT, &self.field_name).ok().flatten().is_none()
    }

    fn metadata(&self) -> &MigrationMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<MigrationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MigrationMetadata::new(
                "AddFieldMigration".to_string(),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
            )
        })
    }
}

proptest! {
    #[test]
    fn test_migration_idempotence(field_name in "[a-z]{1,10}", field_value in "[a-z0-9]{1,20}") {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let migration = AddFieldMigration::new(field_name.clone(), field_value.clone());

            let mut doc = Automerge::new();

            // Apply migration once
            migration.migrate(&mut doc).await.unwrap();
            let value1 = doc.get(&ROOT, &field_name).unwrap();

            // Apply migration again
            migration.migrate(&mut doc).await.unwrap();
            let value2 = doc.get(&ROOT, &field_name).unwrap();

            // Should be idempotent
            prop_assert_eq!(value1, value2);
        });
    }

    #[test]
    fn test_migration_determinism(
        field_name in "[a-z]{1,10}",
        field_value in "[a-z0-9]{1,20}",
        initial_value in "[a-z0-9]{1,20}"
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let migration = AddFieldMigration::new(field_name.clone(), field_value.clone());

            // Create two identical documents
            let mut doc1 = Automerge::new();
            let mut doc2 = Automerge::new();

            // Initialize with same value
            {
                let mut tx1 = doc1.transaction();
                tx1.put(&ROOT, "initial", initial_value.clone()).unwrap();
                tx1.commit();

                let mut tx2 = doc2.transaction();
                tx2.put(&ROOT, "initial", initial_value.clone()).unwrap();
                tx2.commit();
            }

            // Apply migration to both
            migration.migrate(&mut doc1).await.unwrap();
            migration.migrate(&mut doc2).await.unwrap();

            // Extract values
            let value1 = doc1.get(&ROOT, &field_name).unwrap();
            let value2 = doc2.get(&ROOT, &field_name).unwrap();

            // Should be identical (deterministic)
            prop_assert_eq!(value1, value2);

            // Save and compare
            let saved1 = doc1.save();
            let saved2 = doc2.save();

            // Documents should be identical
            prop_assert_eq!(saved1, saved2);
        });
    }
}

/// Test that concurrent migrations produce identical results after merge.
#[tokio::test]
async fn test_concurrent_migration_commutativity() {
    // Create two independent migrations
    let migration_a = AddFieldMigration::new("field_a".to_string(), "value_a".to_string());
    let migration_b = AddFieldMigration::new("field_b".to_string(), "value_b".to_string());

    // Scenario 1: Apply A then B
    let mut doc1 = Automerge::new();
    migration_a.migrate(&mut doc1).await.unwrap();
    migration_b.migrate(&mut doc1).await.unwrap();

    // Scenario 2: Apply B then A
    let mut doc2 = Automerge::new();
    migration_b.migrate(&mut doc2).await.unwrap();
    migration_a.migrate(&mut doc2).await.unwrap();

    // Both should have the same fields
    let value_a1 = doc1.get(&ROOT, "field_a").unwrap();
    let value_a2 = doc2.get(&ROOT, "field_a").unwrap();
    assert_eq!(value_a1, value_a2);

    let value_b1 = doc1.get(&ROOT, "field_b").unwrap();
    let value_b2 = doc2.get(&ROOT, "field_b").unwrap();
    assert_eq!(value_b1, value_b2);
}

/// Test that migrations from different peers merge correctly.
#[tokio::test]
async fn test_distributed_migration_convergence() {
    // Three peers all migrate the same document
    let migration = AddFieldMigration::new("shared_field".to_string(), "shared_value".to_string());

    let mut doc1 = Automerge::new();
    let mut doc2 = Automerge::new();
    let mut doc3 = Automerge::new();

    // All peers apply the same migration independently
    migration.migrate(&mut doc1).await.unwrap();
    migration.migrate(&mut doc2).await.unwrap();
    migration.migrate(&mut doc3).await.unwrap();

    // Merge: doc1 <- doc2
    doc1.merge(&doc2).unwrap();

    // Merge: doc1 <- doc3
    doc1.merge(&doc3).unwrap();

    // Result should be deterministic
    let value = doc1.get(&ROOT, "shared_field").unwrap();
    assert!(value.is_some());

    if let Some((automerge::Value::Scalar(s), _)) = value {
        if let automerge::ScalarValue::Str(val) = s.as_ref() {
            assert_eq!(val.to_string(), "shared_value");
        }
    }
}

/// Test migration with concurrent edits.
#[tokio::test]
async fn test_migration_with_concurrent_edits() {
    let migration = AddFieldMigration::new("email".to_string(), "".to_string());

    let mut doc1 = Automerge::new();
    let mut doc2 = doc1.clone();

    // Peer 1: Apply migration
    migration.migrate(&mut doc1).await.unwrap();

    // Peer 2: Make concurrent edit (without migration)
    {
        let mut tx2 = doc2.transaction();
        tx2.put(&ROOT, "username", "alice").unwrap();
        tx2.commit();
    }

    // Merge
    doc1.merge(&doc2).unwrap();

    // Both changes should be present
    assert!(doc1.get(&ROOT, "email").unwrap().is_some());
    assert!(doc1.get(&ROOT, "username").unwrap().is_some());
}

proptest! {
    /// Test that migration order doesn't matter for independent fields.
    #[test]
    fn test_migration_order_independence(
        field1 in "[a-z]{1,10}",
        field2 in "[a-z]{1,10}",
        value1 in "[a-z0-9]{1,20}",
        value2 in "[a-z0-9]{1,20}"
    ) {
        prop_assume!(field1 != field2); // Fields must be different

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let migration1 = AddFieldMigration::new(field1.clone(), value1.clone());
            let migration2 = AddFieldMigration::new(field2.clone(), value2.clone());

            // Order 1: migration1 then migration2
            let mut doc_order1 = Automerge::new();
            migration1.migrate(&mut doc_order1).await.unwrap();
            migration2.migrate(&mut doc_order1).await.unwrap();

            // Order 2: migration2 then migration1
            let mut doc_order2 = Automerge::new();
            migration2.migrate(&mut doc_order2).await.unwrap();
            migration1.migrate(&mut doc_order2).await.unwrap();

            // Results should be equivalent
            let val1_order1 = doc_order1.get(&ROOT, &field1).unwrap();
            let val1_order2 = doc_order2.get(&ROOT, &field1).unwrap();
            prop_assert_eq!(val1_order1, val1_order2);

            let val2_order1 = doc_order1.get(&ROOT, &field2).unwrap();
            let val2_order2 = doc_order2.get(&ROOT, &field2).unwrap();
            prop_assert_eq!(val2_order1, val2_order2);
        });
    }
}

/// Test migration with network partition simulation.
#[tokio::test]
async fn test_migration_network_partition() {
    let migration = AddFieldMigration::new("partition_field".to_string(), "value".to_string());

    // Initial document
    let mut doc_main = Automerge::new();
    {
        let mut tx = doc_main.transaction();
        tx.put(&ROOT, "initial", "data").unwrap();
        tx.commit();
    }

    // Peer A gets a copy (before partition)
    let mut doc_peer_a = doc_main.clone();

    // Peer B gets a copy (before partition)
    let mut doc_peer_b = doc_main.clone();

    // During partition:
    // Peer A applies migration
    migration.migrate(&mut doc_peer_a).await.unwrap();

    // Peer B makes independent edit
    {
        let mut tx = doc_peer_b.transaction();
        tx.put(&ROOT, "concurrent_field", "concurrent_value")
            .unwrap();
        tx.commit();
    }

    // Network heals - merge documents
    doc_peer_a.merge(&doc_peer_b).unwrap();
    doc_peer_b.merge(&doc_peer_a).unwrap();

    // Both peers should converge to same state
    let saved_a = doc_peer_a.save();
    let saved_b = doc_peer_b.save();
    assert_eq!(saved_a, saved_b);

    // Both should have all fields
    assert!(doc_peer_a.get(&ROOT, "partition_field").unwrap().is_some());
    assert!(doc_peer_a.get(&ROOT, "concurrent_field").unwrap().is_some());
    assert!(doc_peer_b.get(&ROOT, "partition_field").unwrap().is_some());
    assert!(doc_peer_b.get(&ROOT, "concurrent_field").unwrap().is_some());
}

/// Test that actor ID is deterministic across migrations.
#[tokio::test]
async fn test_deterministic_actor_id() {
    let migration = AddFieldMigration::new("test_field".to_string(), "test_value".to_string());

    let mut doc1 = Automerge::new();
    let mut doc2 = Automerge::new();

    migration.migrate(&mut doc1).await.unwrap();
    migration.migrate(&mut doc2).await.unwrap();

    // Both documents should use the same actor ID
    // This is verified by checking that the operations are identical
    let saved1 = doc1.save();
    let saved2 = doc2.save();

    assert_eq!(saved1, saved2);
}
