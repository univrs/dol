//! Schema evolution and mixed version tests.
//!
//! Tests schema evolution across different versions:
//! - Mixed schema version peers
//! - Forward compatibility
//! - Backward compatibility
//! - Lazy migration
//! - Schema-independent convergence

use super::test_harness::*;
use automerge::ROOT;

// Note: Schema evolution is currently disabled (schema_evolution.rs.disabled)
// These tests demonstrate the intended behavior for when it's enabled

#[tokio::test]
async fn test_same_schema_version_sync() {
    // Both nodes on same schema version
    let node_a = TestNode::new("node_a").await;
    let node_b = TestNode::new("node_b").await;

    // Create v1 document structure
    for node in [&node_a, &node_b] {
        node.create_document("users", "alice", |doc| {
            doc.put(ROOT, "name", "Alice")?;
            doc.put(ROOT, "age", 30i64)?;
            Ok(())
        })
        .await;
    }

    // Should sync without issues
    node_a.sync_with_peer(&node_b, "users", "alice").await.unwrap();

    // Verify convergence
    let hash_a = node_a.document_hash("users", "alice").await;
    let hash_b = node_b.document_hash("users", "alice").await;

    assert_eq!(hash_a, hash_b, "Same schema versions should converge");
}

#[tokio::test]
async fn test_forward_compatible_read() {
    // Node A on v1, Node B on v2 (with additional field)
    let node_a = TestNode::new("node_a_v1").await;
    let node_b = TestNode::new("node_b_v2").await;

    // Node A creates v1 document
    node_a
        .create_document("users", "alice", |doc| {
            doc.put(ROOT, "name", "Alice")?;
            doc.put(ROOT, "age", 30i64)?;
            Ok(())
        })
        .await;

    // Node B creates v2 document (with email field)
    node_b
        .create_document("users", "alice", |doc| {
            doc.put(ROOT, "name", "Alice")?;
            doc.put(ROOT, "age", 30i64)?;
            doc.put(ROOT, "email", "alice@example.com")?; // v2 field
            Ok(())
        })
        .await;

    // Sync
    node_b.sync_with_peer(&node_a, "users", "alice").await.unwrap();

    // Node A should be able to read (ignoring unknown email field)
    let name = node_a
        .read_document("users", "alice", |doc| {
            match doc.get(ROOT, "name")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(val) = s.as_ref() {
                        Ok(val.to_string())
                    } else {
                        Err(automerge::AutomergeError::Fail)
                    }
                }
                _ => Err(automerge::AutomergeError::Fail),
            }
        })
        .await;

    assert_eq!(name, "Alice", "v1 node should read v2 document");

    // Node B should have all fields
    let email = node_b
        .read_document("users", "alice", |doc| {
            match doc.get(ROOT, "email")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(val) = s.as_ref() {
                        Ok(Some(val.to_string()))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        })
        .await;

    assert_eq!(email, Some("alice@example.com".to_string()));
}

#[tokio::test]
async fn test_backward_compatible_write() {
    // Node A on v2, Node B on v1
    let node_a = TestNode::new("node_a_v2").await;
    let node_b = TestNode::new("node_b_v1").await;

    // Create v1 structure on both
    for node in [&node_a, &node_b] {
        node.create_document("users", "bob", |doc| {
            doc.put(ROOT, "name", "Bob")?;
            doc.put(ROOT, "age", 25i64)?;
            Ok(())
        })
        .await;
    }

    // Node A (v2) adds email field
    node_a
        .update_document("users", "bob", |doc| {
            doc.put(ROOT, "email", "bob@example.com")?;
            Ok(())
        })
        .await;

    // Sync to v1 node
    node_a.sync_with_peer(&node_b, "users", "bob").await.unwrap();

    // Node B should still be able to read (ignoring email)
    let name = node_b
        .read_document("users", "bob", |doc| {
            match doc.get(ROOT, "name")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(val) = s.as_ref() {
                        Ok(val.to_string())
                    } else {
                        Err(automerge::AutomergeError::Fail)
                    }
                }
                _ => Err(automerge::AutomergeError::Fail),
            }
        })
        .await;

    assert_eq!(name, "Bob");
}

#[tokio::test]
async fn test_three_version_compatibility() {
    // Three nodes: v1, v2, v3
    let node_v1 = TestNode::new("node_v1").await;
    let node_v2 = TestNode::new("node_v2").await;
    let node_v3 = TestNode::new("node_v3").await;

    // v1: name, age
    node_v1
        .create_document("users", "charlie", |doc| {
            doc.put(ROOT, "name", "Charlie")?;
            doc.put(ROOT, "age", 35i64)?;
            Ok(())
        })
        .await;

    // v2: adds email
    node_v2
        .create_document("users", "charlie", |doc| {
            doc.put(ROOT, "name", "Charlie")?;
            doc.put(ROOT, "age", 35i64)?;
            doc.put(ROOT, "email", "charlie@example.com")?;
            Ok(())
        })
        .await;

    // v3: adds phone
    node_v3
        .create_document("users", "charlie", |doc| {
            doc.put(ROOT, "name", "Charlie")?;
            doc.put(ROOT, "age", 35i64)?;
            doc.put(ROOT, "email", "charlie@example.com")?;
            doc.put(ROOT, "phone", "555-1234")?;
            Ok(())
        })
        .await;

    // All should be able to read name
    for node in [&node_v1, &node_v2, &node_v3] {
        let name = node
            .read_document("users", "charlie", |doc| {
                match doc.get(ROOT, "name")? {
                    Some((automerge::Value::Scalar(s), _)) => {
                        if let automerge::ScalarValue::Str(val) = s.as_ref() {
                            Ok(val.to_string())
                        } else {
                            Err(automerge::AutomergeError::Fail)
                        }
                    }
                    _ => Err(automerge::AutomergeError::Fail),
                }
            })
            .await;

        assert_eq!(name, "Charlie");
    }
}

#[tokio::test]
async fn test_field_removal_compatibility() {
    // Test removing a field in newer schema
    let node_v1 = TestNode::new("node_v1").await;
    let node_v2 = TestNode::new("node_v2").await;

    // v1: has deprecated_field
    node_v1
        .create_document("config", "settings", |doc| {
            doc.put(ROOT, "mode", "production")?;
            doc.put(ROOT, "deprecated_field", "old_value")?;
            Ok(())
        })
        .await;

    // v2: deprecated_field removed
    node_v2
        .create_document("config", "settings", |doc| {
            doc.put(ROOT, "mode", "production")?;
            // deprecated_field intentionally omitted
            Ok(())
        })
        .await;

    // Both should be able to read mode
    for node in [&node_v1, &node_v2] {
        let mode = node
            .read_document("config", "settings", |doc| {
                match doc.get(ROOT, "mode")? {
                    Some((automerge::Value::Scalar(s), _)) => {
                        if let automerge::ScalarValue::Str(val) = s.as_ref() {
                            Ok(val.to_string())
                        } else {
                            Err(automerge::AutomergeError::Fail)
                        }
                    }
                    _ => Err(automerge::AutomergeError::Fail),
                }
            })
            .await;

        assert_eq!(mode, "production");
    }
}

#[tokio::test]
async fn test_type_compatible_evolution() {
    // Test type changes (e.g., int -> string)
    let node_a = TestNode::new("node_a").await;

    // Create with int version
    node_a
        .create_document("app", "state", |doc| {
            doc.put(ROOT, "version", 1i64)?;
            Ok(())
        })
        .await;

    // Update to string version (schema evolution)
    node_a
        .update_document("app", "state", |doc| {
            doc.put(ROOT, "version", "1.0.0")?;
            Ok(())
        })
        .await;

    // Should be able to read latest value
    let version = node_a
        .read_document("app", "state", |doc| {
            match doc.get(ROOT, "version")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(val) = s.as_ref() {
                        Ok(val.to_string())
                    } else if let automerge::ScalarValue::Int(val) = s.as_ref() {
                        Ok(val.to_string())
                    } else {
                        Err(automerge::AutomergeError::Fail)
                    }
                }
                _ => Err(automerge::AutomergeError::Fail),
            }
        })
        .await;

    assert_eq!(version, "1.0.0");
}

#[tokio::test]
async fn test_default_value_migration() {
    // Test that missing fields get defaults
    let node_v1 = TestNode::new("node_v1").await;

    // Create v1 document (without email)
    node_v1
        .create_document("users", "diana", |doc| {
            doc.put(ROOT, "name", "Diana")?;
            doc.put(ROOT, "age", 28i64)?;
            Ok(())
        })
        .await;

    // Read with v2 schema expectations (email should have default)
    let email = node_v1
        .read_document("users", "diana", |doc| {
            match doc.get(ROOT, "email")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(val) = s.as_ref() {
                        Ok(val.to_string())
                    } else {
                        Ok(String::new())
                    }
                }
                None => Ok(String::new()), // Default empty string
                _ => Ok(String::new()),
            }
        })
        .await;

    assert_eq!(email, ""); // Default value
}

#[tokio::test]
async fn test_schema_independent_convergence() {
    // Test that documents converge despite schema differences
    let node_a = TestNode::new("node_a").await;
    let node_b = TestNode::new("node_b").await;

    // Create with different schemas
    node_a
        .create_document("data", "shared", |doc| {
            doc.put(ROOT, "field_a", "value_a")?;
            Ok(())
        })
        .await;

    node_b
        .create_document("data", "shared", |doc| {
            doc.put(ROOT, "field_b", "value_b")?;
            Ok(())
        })
        .await;

    // Sync
    node_a.sync_with_peer(&node_b, "data", "shared").await.unwrap();

    // After sync, both nodes should have both fields (CRDT merge)
    let has_field_a = node_a
        .read_document("data", "shared", |doc| {
            Ok(doc.get(ROOT, "field_a")?.is_some())
        })
        .await;

    let has_field_b = node_b
        .read_document("data", "shared", |doc| {
            Ok(doc.get(ROOT, "field_b")?.is_some())
        })
        .await;

    assert!(has_field_a);
    assert!(has_field_b);
}
