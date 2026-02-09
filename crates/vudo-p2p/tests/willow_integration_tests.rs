//! Integration tests for Willow Protocol adapter.

use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};
use bytes::Bytes;
use ed25519_dalek::SigningKey;
use std::sync::Arc;
use vudo_p2p::{
    meadowcap::{Capability, Permission},
    ResourceConstraints, SyncPriority, WillowAdapter,
};
use vudo_state::{DocumentId, StateEngine};

fn get_string(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> String {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::Str(smol_str) = s.as_ref() {
                smol_str.to_string()
            } else {
                panic!("Expected string value");
            }
        }
        _ => panic!("Value not found"),
    }
}

#[tokio::test]
async fn test_complete_sync_workflow() {
    // Initialize state engine and adapter
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    // Create root capability
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Create document in state engine
    let doc_id = DocumentId::new("users", "alice");
    let handle = engine.create_document(doc_id.clone()).await.unwrap();
    handle
        .update(|doc| {
            doc.put(ROOT, "name", "Alice")?;
            doc.put(ROOT, "email", "alice@example.com")?;
            doc.put(ROOT, "age", 30i64)?;
            Ok(())
        })
        .unwrap();

    // Sync to Willow
    adapter
        .sync_from_state_engine("myapp.v1", "users", "alice", &root_cap)
        .await
        .unwrap();

    // Delete from state engine
    engine.delete_document(&doc_id).await.unwrap();

    // Sync back from Willow
    adapter
        .sync_to_state_engine("myapp.v1", "users", "alice", &root_cap)
        .await
        .unwrap();

    // Verify document is restored
    let restored = engine.get_document(&doc_id).await.unwrap();
    restored
        .read(|doc| {
            let name = get_string(doc, ROOT, "name");
            assert_eq!(name, "Alice");
            Ok(())
        })
        .unwrap();
}

#[tokio::test]
async fn test_capability_delegation_hierarchy() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Admin delegates write capability for users collection
    let users_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let subspace_id = adapter.map_subspace("users");
    let users_cap = root_cap
        .delegate(
            Some(subspace_id),
            vudo_p2p::Path::empty(),
            Permission::Write,
            &users_key,
        )
        .unwrap();

    // Users admin delegates read-only for Alice's data
    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_cap = users_cap
        .delegate(
            Some(subspace_id),
            vudo_p2p::Path::from_components(["alice"]),
            Permission::Read,
            &alice_key,
        )
        .unwrap();

    // Alice can read her own data
    let data = Bytes::from("alice's data");
    adapter
        .write_entry("myapp.v1", "users", "alice", data.clone(), &users_cap)
        .await
        .unwrap();

    let read_data = adapter
        .read_entry("myapp.v1", "users", "alice", &alice_cap)
        .await
        .unwrap();

    assert_eq!(read_data, Some(data));

    // Alice cannot write
    let result = adapter
        .write_entry("myapp.v1", "users", "alice", Bytes::from("new data"), &alice_cap)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_multi_collection_sync() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Create documents in different collections
    let collections = ["users", "posts", "comments"];
    for collection in &collections {
        for i in 0..3 {
            let doc_id = DocumentId::new(*collection, &format!("doc{}", i));
            let handle = engine.create_document(doc_id).await.unwrap();
            handle
                .update(|doc| {
                    doc.put(ROOT, "data", format!("{} {}", collection, i))?;
                    Ok(())
                })
                .unwrap();
        }
    }

    // Sync all collections
    for collection in &collections {
        for i in 0..3 {
            adapter
                .sync_from_state_engine("myapp.v1", collection, &format!("doc{}", i), &root_cap)
                .await
                .unwrap();
        }
    }

    let stats = adapter.stats();
    assert_eq!(stats.entry_count, 9); // 3 collections × 3 docs
}

#[tokio::test]
async fn test_gdpr_compliant_deletion() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Create user document
    let doc_id = DocumentId::new("users", "alice");
    let handle = engine.create_document(doc_id.clone()).await.unwrap();
    handle
        .update(|doc| {
            doc.put(ROOT, "name", "Alice")?;
            doc.put(ROOT, "email", "alice@example.com")?;
            Ok(())
        })
        .unwrap();

    // Sync to Willow
    adapter
        .sync_from_state_engine("myapp.v1", "users", "alice", &root_cap)
        .await
        .unwrap();

    // GDPR deletion
    adapter
        .gdpr_delete(
            "myapp.v1",
            "users",
            "alice",
            &root_cap,
            "User requested data deletion under GDPR Article 17",
        )
        .await
        .unwrap();

    // Verify document is gone from state engine
    assert!(engine.get_document(&doc_id).await.is_err());

    // Verify tombstone exists in Willow
    let stats = adapter.stats();
    assert_eq!(stats.tombstone_count, 1);
    assert_eq!(stats.entry_count, 0);

    // Reading should return None (deleted)
    let data = adapter
        .read_entry("myapp.v1", "users", "alice", &root_cap)
        .await
        .unwrap();
    assert_eq!(data, None);
}

#[tokio::test]
async fn test_resource_constrained_sync_high_priority() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Create many documents
    for i in 0..20 {
        let doc_id = DocumentId::new("users", &format!("user{}", i));
        let handle = engine.create_document(doc_id).await.unwrap();
        handle
            .update(|doc| {
                doc.put(ROOT, "name", format!("User {}", i))?;
                Ok(())
            })
            .unwrap();
    }

    // Sync with high priority and tight constraints
    let constraints = ResourceConstraints {
        max_memory: 50 * 1024, // 50 KB
        max_bandwidth: 10 * 1024,
        priority: SyncPriority::High,
    };

    let stats = adapter
        .sync_with_constraints("myapp.v1", "users", &root_cap, constraints)
        .await
        .unwrap();

    assert!(stats.synced_count > 0);
    assert!(stats.total_bytes > 0);
    assert!(stats.total_bytes <= 50 * 1024);
}

#[tokio::test]
async fn test_hierarchical_path_permissions() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    let subspace_id = adapter.map_subspace("users");

    // Delegate read access to alice/* path
    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_cap = root_cap
        .delegate(
            Some(subspace_id),
            vudo_p2p::Path::from_components(["alice"]),
            Permission::Read,
            &alice_key,
        )
        .unwrap();

    // Write data at different paths
    adapter
        .write_entry(
            "myapp.v1",
            "users",
            "alice/profile",
            Bytes::from("alice profile"),
            &root_cap,
        )
        .await
        .unwrap();

    adapter
        .write_entry(
            "myapp.v1",
            "users",
            "alice/posts/1",
            Bytes::from("alice post 1"),
            &root_cap,
        )
        .await
        .unwrap();

    adapter
        .write_entry("myapp.v1", "users", "bob/profile", Bytes::from("bob profile"), &root_cap)
        .await
        .unwrap();

    // Alice can read alice/profile
    let data = adapter
        .read_entry("myapp.v1", "users", "alice/profile", &alice_cap)
        .await
        .unwrap();
    assert_eq!(data, Some(Bytes::from("alice profile")));

    // Alice can read alice/posts/1 (nested path)
    let data = adapter
        .read_entry("myapp.v1", "users", "alice/posts/1", &alice_cap)
        .await
        .unwrap();
    assert_eq!(data, Some(Bytes::from("alice post 1")));

    // Alice cannot read bob/profile
    let result = adapter
        .read_entry("myapp.v1", "users", "bob/profile", &alice_cap)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_namespace_isolation() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    let key1 = SigningKey::generate(&mut rand::rngs::OsRng);
    let ns1 = adapter.map_namespace("app1.v1");
    let cap1 = Capability::new_root(ns1, &key1);

    let key2 = SigningKey::generate(&mut rand::rngs::OsRng);
    let ns2 = adapter.map_namespace("app2.v1");
    let cap2 = Capability::new_root(ns2, &key2);

    // Write to app1
    adapter
        .write_entry("app1.v1", "users", "alice", Bytes::from("app1 data"), &cap1)
        .await
        .unwrap();

    // Write to app2
    adapter
        .write_entry("app2.v1", "users", "alice", Bytes::from("app2 data"), &cap2)
        .await
        .unwrap();

    // Reading from different namespaces should give different data
    let data1 = adapter
        .read_entry("app1.v1", "users", "alice", &cap1)
        .await
        .unwrap();
    let data2 = adapter
        .read_entry("app2.v1", "users", "alice", &cap2)
        .await
        .unwrap();

    assert_eq!(data1, Some(Bytes::from("app1 data")));
    assert_eq!(data2, Some(Bytes::from("app2 data")));
    assert_ne!(data1, data2);
}

#[tokio::test]
async fn test_concurrent_sync_operations() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = Arc::new(WillowAdapter::new(Arc::clone(&engine)).await.unwrap());

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Create documents
    for i in 0..10 {
        let doc_id = DocumentId::new("users", &format!("user{}", i));
        let handle = engine.create_document(doc_id).await.unwrap();
        handle
            .update(|doc| {
                doc.put(ROOT, "id", i as i64)?;
                Ok(())
            })
            .unwrap();
    }

    // Sync concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let adapter_clone = Arc::clone(&adapter);
        let cap_clone = root_cap.clone();
        let handle = tokio::spawn(async move {
            adapter_clone
                .sync_from_state_engine("myapp.v1", "users", &format!("user{}", i), &cap_clone)
                .await
        });
        handles.push(handle);
    }

    // Wait for all syncs
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let stats = adapter.stats();
    assert_eq!(stats.entry_count, 10);
}

#[tokio::test]
async fn test_tombstone_propagation() {
    let engine = Arc::new(StateEngine::new().await.unwrap());
    let adapter = WillowAdapter::new(Arc::clone(&engine)).await.unwrap();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = adapter.map_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Write data
    adapter
        .write_entry("myapp.v1", "users", "alice", Bytes::from("data"), &root_cap)
        .await
        .unwrap();

    assert_eq!(adapter.stats().entry_count, 1);

    // Delete with tombstone
    adapter
        .delete_entry("myapp.v1", "users", "alice", &root_cap, Some("test".to_string()))
        .await
        .unwrap();

    // Entry should be gone, tombstone should exist
    assert_eq!(adapter.stats().entry_count, 0);
    assert_eq!(adapter.stats().tombstone_count, 1);

    // Reading should return None
    let data = adapter
        .read_entry("myapp.v1", "users", "alice", &root_cap)
        .await
        .unwrap();
    assert_eq!(data, None);
}
