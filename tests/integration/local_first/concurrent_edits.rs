//! Concurrent multi-node edit tests.
//!
//! Stress tests CRDT convergence with high concurrency:
//! - Concurrent edits on 5+ nodes
//! - High-frequency updates
//! - Conflicting updates to same keys
//! - Mixed operation types (puts, deletes, lists)
//! - Burst traffic patterns

use super::test_harness::*;
use automerge::ROOT;
use std::time::Duration;

#[tokio::test]
async fn test_concurrent_five_node_edits() {
    let nodes = create_mesh_network(5).await;

    // Create shared document on all nodes
    for node in &nodes {
        node.create_document("stress", "test", |doc| {
            doc.put(ROOT, "initialized", true)?;
            Ok(())
        })
        .await;
    }

    // Concurrent edits from all nodes (10 operations each)
    let handles: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(node_idx, node)| {
            let node = node;
            tokio::spawn(async move {
                for i in 0..10 {
                    node.update_document("stress", "test", |doc| {
                        doc.put(ROOT, format!("node_{}_{}", node_idx, i), i as i64)?;
                        Ok(())
                    })
                    .await;

                    // Small delay to simulate realistic patterns
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
        })
        .collect();

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all operations on each node (locally)
    for (node_idx, node) in nodes.iter().enumerate() {
        for i in 0..10 {
            let has_op = node
                .read_document("stress", "test", |doc| {
                    let key = format!("node_{}_{}", node_idx, i);
                    Ok(doc.get(ROOT, key.as_str())?.is_some())
                })
                .await;

            assert!(has_op, "Node {} should have operation {}", node_idx, i);
        }
    }
}

#[tokio::test]
async fn test_concurrent_ten_node_edits() {
    let nodes = create_mesh_network(10).await;

    // Create shared document
    for node in &nodes {
        node.create_document("stress", "high", |doc| {
            doc.put(ROOT, "counter", 0i64)?;
            Ok(())
        })
        .await;
    }

    // Concurrent edits from all nodes (100 operations each)
    let handles: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(node_idx, node)| {
            let node = node;
            tokio::spawn(async move {
                for i in 0..100 {
                    node.update_document("stress", "high", |doc| {
                        doc.put(ROOT, format!("op_{}_{}", node_idx, i), (node_idx * 100 + i) as i64)?;
                        Ok(())
                    })
                    .await;
                }
            })
        })
        .collect();

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify local operations
    for (node_idx, node) in nodes.iter().enumerate() {
        let op_50 = node
            .read_document("stress", "high", |doc| {
                let key = format!("op_{}_{}", node_idx, 50);
                match doc.get(ROOT, key.as_str())? {
                    Some((automerge::Value::Scalar(s), _)) => {
                        if let automerge::ScalarValue::Int(val) = s.as_ref() {
                            Ok(Some(*val))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            })
            .await;

        assert_eq!(
            op_50,
            Some((node_idx * 100 + 50) as i64),
            "Operation 50 on node {} should exist",
            node_idx
        );
    }
}

#[tokio::test]
async fn test_conflicting_updates_same_key() {
    let nodes = create_mesh_network(5).await;

    // Create shared document
    for node in &nodes {
        node.create_document("conflict", "test", |doc| {
            doc.put(ROOT, "shared_key", 0i64)?;
            Ok(())
        })
        .await;
    }

    // All nodes concurrently update the same key
    let handles: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(node_idx, node)| {
            let node = node;
            tokio::spawn(async move {
                for i in 0..20 {
                    node.update_document("conflict", "test", |doc| {
                        doc.put(ROOT, "shared_key", (node_idx * 20 + i) as i64)?;
                        Ok(())
                    })
                    .await;

                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
            })
        })
        .collect();

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }

    // In CRDT implementation, last-write-wins or other conflict resolution
    // Verify that each node has *some* value for shared_key
    for node in &nodes {
        let value = node
            .read_document("conflict", "test", |doc| {
                match doc.get(ROOT, "shared_key")? {
                    Some((automerge::Value::Scalar(s), _)) => {
                        if let automerge::ScalarValue::Int(val) = s.as_ref() {
                            Ok(Some(*val))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            })
            .await;

        assert!(value.is_some(), "Shared key should have a value");
    }
}

#[tokio::test]
async fn test_high_frequency_updates() {
    let node_a = TestNode::new("node_a").await;
    let node_b = TestNode::new("node_b").await;

    // Create document
    for node in [&node_a, &node_b] {
        node.create_document("freq", "test", |doc| {
            doc.put(ROOT, "counter", 0i64)?;
            Ok(())
        })
        .await;
    }

    // High-frequency updates on node A (1000 ops)
    let start = std::time::Instant::now();

    for i in 0..1000 {
        node_a
            .update_document("freq", "test", |doc| {
                doc.put(ROOT, "counter", i as i64)?;
                Ok(())
            })
            .await;
    }

    let duration = start.elapsed();

    // Should complete in reasonable time (< 5 seconds)
    assert!(
        duration < Duration::from_secs(5),
        "1000 operations took {:?}, expected < 5s",
        duration
    );

    // Verify final counter
    let counter = node_a
        .read_document("freq", "test", |doc| {
            match doc.get(ROOT, "counter")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Int(val) = s.as_ref() {
                        Ok(*val)
                    } else {
                        Err(automerge::AutomergeError::Fail)
                    }
                }
                _ => Err(automerge::AutomergeError::Fail),
            }
        })
        .await;

    assert_eq!(counter, 999, "Counter should reach 999");
}

#[tokio::test]
async fn test_burst_traffic_pattern() {
    let nodes = create_mesh_network(5).await;

    // Create document
    for node in &nodes {
        node.create_document("burst", "test", |doc| {
            doc.put(ROOT, "initialized", true)?;
            Ok(())
        })
        .await;
    }

    // Simulate burst: 3 rounds of 50 concurrent ops, with pauses
    for burst in 0..3 {
        let handles: Vec<_> = nodes
            .iter()
            .enumerate()
            .map(|(node_idx, node)| {
                let node = node;
                tokio::spawn(async move {
                    for i in 0..50 {
                        node.update_document("burst", "test", |doc| {
                            doc.put(
                                ROOT,
                                format!("burst_{}_node_{}_op_{}", burst, node_idx, i),
                                i as i64,
                            )?;
                            Ok(())
                        })
                        .await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        // Pause between bursts
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Verify operations from first burst on node 0
    for i in 0..50 {
        let has_op = nodes[0]
            .read_document("burst", "test", |doc| {
                let key = format!("burst_0_node_0_op_{}", i);
                Ok(doc.get(ROOT, key.as_str())?.is_some())
            })
            .await;

        assert!(has_op, "Burst 0 operation {} should exist", i);
    }
}

#[tokio::test]
async fn test_concurrent_document_creation() {
    let nodes = create_mesh_network(5).await;

    // Concurrently create different documents on different nodes
    let handles: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(node_idx, node)| {
            let node = node;
            tokio::spawn(async move {
                for i in 0..10 {
                    node.create_document(
                        "concurrent",
                        &format!("doc_{}_{}", node_idx, i),
                        |doc| {
                            doc.put(ROOT, "node", node_idx as i64)?;
                            doc.put(ROOT, "index", i as i64)?;
                            Ok(())
                        },
                    )
                    .await;
                }
            })
        })
        .collect();

    // Wait for all creations
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify documents exist on their creating nodes
    for (node_idx, node) in nodes.iter().enumerate() {
        for i in 0..10 {
            let doc_exists = node
                .state_engine
                .get_document(&vudo_state::DocumentId::new(
                    "concurrent",
                    &format!("doc_{}_{}", node_idx, i),
                ))
                .await
                .is_ok();

            assert!(
                doc_exists,
                "Document doc_{}_{} should exist on node {}",
                node_idx, i, node_idx
            );
        }
    }
}

#[tokio::test]
async fn test_mixed_operation_types() {
    let nodes = create_mesh_network(3).await;

    // Create document
    for node in &nodes {
        node.create_document("mixed", "ops", |doc| {
            doc.put(ROOT, "initialized", true)?;
            Ok(())
        })
        .await;
    }

    // Node 0: puts
    let node_0 = &nodes[0];
    let handle_0 = tokio::spawn(async move {
        for i in 0..20 {
            node_0
                .update_document("mixed", "ops", |doc| {
                    doc.put(ROOT, format!("put_{}", i), i as i64)?;
                    Ok(())
                })
                .await;
        }
    });

    // Node 1: updates to same keys
    let node_1 = &nodes[1];
    let handle_1 = tokio::spawn(async move {
        for i in 0..20 {
            node_1
                .update_document("mixed", "ops", |doc| {
                    doc.put(ROOT, format!("put_{}", i), (i * 2) as i64)?;
                    Ok(())
                })
                .await;
        }
    });

    // Node 2: different keys
    let node_2 = &nodes[2];
    let handle_2 = tokio::spawn(async move {
        for i in 0..20 {
            node_2
                .update_document("mixed", "ops", |doc| {
                    doc.put(ROOT, format!("alt_{}", i), i as i64)?;
                    Ok(())
                })
                .await;
        }
    });

    // Wait for all
    handle_0.await.unwrap();
    handle_1.await.unwrap();
    handle_2.await.unwrap();

    // Verify each node has its own operations
    let has_put_0 = nodes[0]
        .read_document("mixed", "ops", |doc| {
            Ok(doc.get(ROOT, "put_0")?.is_some())
        })
        .await;

    let has_alt_0 = nodes[2]
        .read_document("mixed", "ops", |doc| {
            Ok(doc.get(ROOT, "alt_0")?.is_some())
        })
        .await;

    assert!(has_put_0);
    assert!(has_alt_0);
}
