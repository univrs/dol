//! Integration tests for VUDO state engine.

use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};
use vudo_state::*;

fn get_string(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<String> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::Str(smol_str) = s.as_ref() {
                Ok(smol_str.to_string())
            } else {
                panic!("Expected string value")
            }
        }
        _ => panic!("Value not found"),
    }
}

fn get_i64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<i64> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::Int(val) = s.as_ref() {
                Ok(*val)
            } else {
                panic!("Expected int value")
            }
        }
        _ => panic!("Value not found"),
    }
}

fn get_f64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<f64> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::F64(val) = s.as_ref() {
                Ok(*val)
            } else {
                panic!("Expected f64 value")
            }
        }
        _ => panic!("Value not found"),
    }
}

#[tokio::test]
async fn test_full_workflow() {
    // Initialize engine
    let engine = StateEngine::new().await.unwrap();

    // Create documents
    let user_id = DocumentId::new("users", "alice");
    let handle = engine.create_document(user_id.clone()).await.unwrap();

    // Update document
    handle
        .update(|doc| {
            doc.put(ROOT, "name", "Alice")?;
            doc.put(ROOT, "email", "alice@example.com")?;
            doc.put(ROOT, "age", 30i64)?;
            Ok(())
        })
        .unwrap();

    // Subscribe to changes
    let filter = SubscriptionFilter::Document(user_id.clone());
    let mut subscription = engine.subscribe(filter).await;

    // Make a change
    handle
        .update_reactive(&engine.observable, |doc| {
            doc.put(ROOT, "age", 31i64)?;
            Ok(())
        })
        .unwrap();

    // Flush and receive notification
    engine.observable.flush_batch();
    let event = tokio::time::timeout(
        tokio::time::Duration::from_secs(1),
        subscription.recv(),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(event.document_id, user_id);

    // Create snapshot
    let snapshot = engine.snapshot(&handle).await.unwrap();
    assert_eq!(snapshot.metadata.version, 1);

    // Verify stats
    let stats = engine.stats();
    assert_eq!(stats.document_count, 1);
    assert!(stats.total_document_size > 0);
    assert_eq!(stats.snapshot_count, 1);
}

#[tokio::test]
async fn test_transaction_workflow() {
    let engine = StateEngine::new().await.unwrap();

    // Create accounts
    let alice_id = DocumentId::new("accounts", "alice");
    let bob_id = DocumentId::new("accounts", "bob");

    engine.create_document(alice_id.clone()).await.unwrap();
    engine.create_document(bob_id.clone()).await.unwrap();

    // Set initial balances
    let alice = engine.get_document(&alice_id).await.unwrap();
    let bob = engine.get_document(&bob_id).await.unwrap();

    alice
        .update(|doc| {
            doc.put(ROOT, "balance", 100i64)?;
            Ok(())
        })
        .unwrap();

    bob.update(|doc| {
        doc.put(ROOT, "balance", 50i64)?;
        Ok(())
    })
    .unwrap();

    // Transfer money using a transaction
    let tx = engine.begin_transaction();

    tx.update(&alice_id, |doc| {
        doc.put(ROOT, "balance", 75i64)?; // -25
        Ok(())
    })
    .unwrap();

    tx.update(&bob_id, |doc| {
        doc.put(ROOT, "balance", 75i64)?; // +25
        Ok(())
    })
    .unwrap();

    engine.commit_transaction(tx).unwrap();

    // Verify final balances
    alice
        .read(|doc| {
            let balance = get_i64(doc, ROOT, "balance")?;
            assert_eq!(balance, 75);
            Ok(())
        })
        .unwrap();

    bob.read(|doc| {
        let balance = get_i64(doc, ROOT, "balance")?;
        assert_eq!(balance, 75);
        Ok(())
    })
    .unwrap();
}

#[tokio::test]
async fn test_transaction_rollback_workflow() {
    let engine = StateEngine::new().await.unwrap();

    let alice_id = DocumentId::new("accounts", "alice");
    let bob_id = DocumentId::new("accounts", "bob");

    engine.create_document(alice_id.clone()).await.unwrap();
    engine.create_document(bob_id.clone()).await.unwrap();

    let alice = engine.get_document(&alice_id).await.unwrap();
    let bob = engine.get_document(&bob_id).await.unwrap();

    alice
        .update(|doc| {
            doc.put(ROOT, "balance", 100i64)?;
            Ok(())
        })
        .unwrap();

    bob.update(|doc| {
        doc.put(ROOT, "balance", 50i64)?;
        Ok(())
    })
    .unwrap();

    // Start a transaction but roll it back
    let tx = engine.begin_transaction();

    tx.update(&alice_id, |doc| {
        doc.put(ROOT, "balance", 75i64)?;
        Ok(())
    })
    .unwrap();

    tx.update(&bob_id, |doc| {
        doc.put(ROOT, "balance", 75i64)?;
        Ok(())
    })
    .unwrap();

    // Rollback instead of commit
    engine.rollback_transaction(tx).unwrap();

    // Verify balances are unchanged
    alice
        .read(|doc| {
            let balance = get_i64(doc, ROOT, "balance")?;
            assert_eq!(balance, 100);
            Ok(())
        })
        .unwrap();

    bob.read(|doc| {
        let balance = get_i64(doc, ROOT, "balance")?;
        assert_eq!(balance, 50);
        Ok(())
    })
    .unwrap();
}

#[tokio::test]
async fn test_operation_queue_persistence() {
    let engine = StateEngine::new().await.unwrap();

    // Create operations
    let doc_id1 = DocumentId::new("users", "alice");
    let doc_id2 = DocumentId::new("users", "bob");

    engine.create_document(doc_id1.clone()).await.unwrap();
    engine.create_document(doc_id2.clone()).await.unwrap();

    assert_eq!(engine.queue.len(), 2);

    // Serialize queue
    let bytes = engine.queue.serialize().unwrap();

    // Create new queue and deserialize
    let new_queue = OperationQueue::new();
    new_queue.deserialize(&bytes).unwrap();

    assert_eq!(new_queue.len(), 2);

    let ops = new_queue.list();
    assert_eq!(ops[0].document_id(), &doc_id1);
    assert_eq!(ops[1].document_id(), &doc_id2);
}

#[tokio::test]
async fn test_snapshot_compaction() {
    let engine = StateEngine::new().await.unwrap();
    let doc_id = DocumentId::new("data", "large");
    let handle = engine.create_document(doc_id).await.unwrap();

    // Create many changes
    for i in 0..50 {
        handle
            .update(|doc| {
                doc.put(ROOT, format!("key{}", i), i as i64)?;
                Ok(())
            })
            .unwrap();
    }

    // Compact the document
    let result = engine.compact(&handle).await.unwrap();

    assert!(result.original_size > 0);
    assert!(result.compacted_size > 0);
}

#[tokio::test]
async fn test_concurrent_subscriptions() {
    let engine = StateEngine::new().await.unwrap();
    let doc_id = DocumentId::new("users", "alice");
    let handle = engine.create_document(doc_id.clone()).await.unwrap();

    // Create multiple subscriptions
    let filter = SubscriptionFilter::Document(doc_id);
    let mut sub1 = engine.subscribe(filter.clone()).await;
    let mut sub2 = engine.subscribe(filter.clone()).await;
    let mut sub3 = engine.subscribe(filter).await;

    // Make a change
    handle
        .update_reactive(&engine.observable, |doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        })
        .unwrap();

    engine.observable.flush_batch();

    // All subscriptions should receive the event
    let timeout = tokio::time::Duration::from_secs(1);

    let event1 = tokio::time::timeout(timeout, sub1.recv())
        .await
        .unwrap()
        .unwrap();
    let event2 = tokio::time::timeout(timeout, sub2.recv())
        .await
        .unwrap()
        .unwrap();
    let event3 = tokio::time::timeout(timeout, sub3.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(event1.document_id.key, "alice");
    assert_eq!(event2.document_id.key, "alice");
    assert_eq!(event3.document_id.key, "alice");
}

#[tokio::test]
async fn test_document_lifecycle() {
    let engine = StateEngine::new().await.unwrap();
    let doc_id = DocumentId::new("users", "alice");

    // Create
    let handle = engine.create_document(doc_id.clone()).await.unwrap();
    assert_eq!(engine.stats().document_count, 1);

    // Update
    handle
        .update(|doc| {
            doc.put(ROOT, "name", "Alice")?;
            Ok(())
        })
        .unwrap();

    // Read
    handle
        .read(|doc| {
            let name = get_string(doc, ROOT, "name")?;
            assert_eq!(name, "Alice");
            Ok(())
        })
        .unwrap();

    // Snapshot
    let snapshot = engine.snapshot(&handle).await.unwrap();
    assert!(snapshot.data.len() > 0);

    // Delete
    engine.delete_document(&doc_id).await.unwrap();
    assert_eq!(engine.stats().document_count, 0);
}

#[tokio::test]
async fn test_multiple_namespaces() {
    let engine = StateEngine::new().await.unwrap();

    // Create documents in different namespaces
    engine
        .create_document(DocumentId::new("users", "alice"))
        .await
        .unwrap();
    engine
        .create_document(DocumentId::new("users", "bob"))
        .await
        .unwrap();
    engine
        .create_document(DocumentId::new("posts", "1"))
        .await
        .unwrap();
    engine
        .create_document(DocumentId::new("posts", "2"))
        .await
        .unwrap();
    engine
        .create_document(DocumentId::new("comments", "1"))
        .await
        .unwrap();

    // List by namespace
    let users = engine.store.list_namespace("users");
    assert_eq!(users.len(), 2);

    let posts = engine.store.list_namespace("posts");
    assert_eq!(posts.len(), 2);

    let comments = engine.store.list_namespace("comments");
    assert_eq!(comments.len(), 1);
}

#[tokio::test]
async fn test_operation_queue_deduplication() {
    let engine = StateEngine::new().await.unwrap();
    let doc_id = DocumentId::new("users", "alice");

    // Create operations with the same idempotency key
    let op1 = Operation::new_with_key(
        OperationType::Create {
            document_id: doc_id.clone(),
        },
        "create-alice".to_string(),
    );

    let op2 = Operation::new_with_key(
        OperationType::Create {
            document_id: doc_id,
        },
        "create-alice".to_string(),
    );

    let id1 = engine.queue.enqueue(op1).unwrap();
    let id2 = engine.queue.enqueue(op2).unwrap();

    // Should return the same ID (deduplicated)
    assert_eq!(id1, id2);
    assert_eq!(engine.queue.len(), 1);
}

#[tokio::test]
async fn test_snapshot_versioning() {
    let engine = StateEngine::new().await.unwrap();
    let doc_id = DocumentId::new("data", "versioned");
    let handle = engine.create_document(doc_id.clone()).await.unwrap();

    // Make changes and create multiple snapshots
    for i in 0..3 {
        handle
            .update(|doc| {
                doc.put(ROOT, "value", i as i64)?;
                Ok(())
            })
            .unwrap();

        engine.snapshot(&handle).await.unwrap();
    }

    // Should have 3 snapshots
    let snapshots = engine.snapshot_storage.list(&doc_id);
    assert_eq!(snapshots.len(), 3);
    assert_eq!(snapshots[0].version, 1);
    assert_eq!(snapshots[1].version, 2);
    assert_eq!(snapshots[2].version, 3);
}

#[tokio::test]
async fn test_concurrent_document_access() {
    use std::sync::Arc;
    use tokio::task;

    let engine = Arc::new(StateEngine::new().await.unwrap());
    let doc_id = DocumentId::new("counter", "shared");
    engine.create_document(doc_id.clone()).await.unwrap();

    let mut handles = vec![];

    for i in 0..10 {
        let engine_clone = Arc::clone(&engine);
        let doc_id_clone = doc_id.clone();

        let handle = task::spawn(async move {
            let doc = engine_clone.get_document(&doc_id_clone).await.unwrap();
            doc.update(|d| {
                d.put(ROOT, format!("key{}", i), i as i64)?;
                Ok(())
            })
            .unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all keys were written
    let doc = engine.get_document(&doc_id).await.unwrap();
    doc.read(|d| {
        for i in 0..10 {
            let val = get_i64(d, ROOT, &format!("key{}", i))?;
            assert_eq!(val, i);
        }
        Ok(())
    })
    .unwrap();
}
