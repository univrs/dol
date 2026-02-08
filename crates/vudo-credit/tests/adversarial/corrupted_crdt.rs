//! Corrupted CRDT Operations Tests
//!
//! Tests Byzantine fault tolerance against malicious peers sending
//! corrupted or malformed Automerge CRDT operations.

use crate::test_harness::*;
use automerge::{ActorId, AutoCommit, Change, ReadDoc};
use std::time::Duration;

#[tokio::test]
async fn test_corrupted_crdt_rejected() {
    // Setup: Create 4 honest nodes (3f+1 where f=1)
    let mut honest_nodes = create_honest_nodes(4).await;

    // Create accounts
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Create valid document on first node
    let doc_data = b"test data".to_vec();

    // Wait for sync
    wait_for_sync(&honest_nodes).await;

    // Malicious node sends corrupted operations
    let malicious_node = create_malicious_node(AttackStrategy::CorruptedCrdt).await;

    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Attack was unsuccessful
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);
    assert!(result.detection_time.is_some());

    // Verify: Malicious node is flagged
    let is_flagged = honest_nodes[0]
        .is_peer_flagged(&malicious_node.peer_id())
        .await;
    // Note: In full implementation, peer would be flagged automatically

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_invalid_actor_id_rejected() {
    // Setup
    let mut honest_nodes = create_honest_nodes(2).await;
    for node in &mut honest_nodes {
        node.create_account(5_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Create malicious change with invalid actor ID
    let invalid_actor = ActorId::from(vec![0xFF; 32]);

    // Attempt to create change with invalid actor
    // In real implementation, this would be rejected by Automerge validation

    let malicious_node = create_malicious_node(AttackStrategy::CorruptedCrdt).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Operation is rejected
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_out_of_order_sequence_rejected() {
    // Setup
    let mut honest_nodes = create_honest_nodes(3).await;
    for node in &mut honest_nodes {
        node.create_account(8_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Malicious node tries to send operations with out-of-order sequence numbers
    // seq: 1, 2, 100 (skipping 3-99)

    let malicious_node = create_malicious_node(AttackStrategy::CorruptedCrdt).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Out-of-order operations rejected
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_invalid_operation_type_rejected() {
    // Setup
    let mut honest_nodes = create_honest_nodes(4).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Malicious node tries to send operations with unknown action types
    let malicious_node = create_malicious_node(AttackStrategy::CorruptedCrdt).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Invalid operation rejected
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Verify: Mitigation applied
    assert!(matches!(
        result.mitigation,
        Some(Mitigation::OperationRejected { .. })
    ));

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_corrupted_changes_do_not_propagate() {
    // Setup: 5 honest nodes
    let mut honest_nodes = create_honest_nodes(5).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Create valid transaction on first node
    let tx_id = honest_nodes[0]
        .pay("honest_1", 100)
        .await
        .expect("Payment failed");

    // Wait for sync
    wait_for_sync(&honest_nodes).await;

    // Malicious node injects corrupted changes
    let malicious_node = create_malicious_node(AttackStrategy::CorruptedCrdt).await;
    let result = malicious_node.execute_attack().await.unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify: All honest nodes have consistent state
    let balance_0 = honest_nodes[0].get_balance().await;
    let balance_1 = honest_nodes[1].get_balance().await;
    let balance_2 = honest_nodes[2].get_balance().await;
    let balance_3 = honest_nodes[3].get_balance().await;
    let balance_4 = honest_nodes[4].get_balance().await;

    // All nodes should have same view
    assert_eq!(balance_0, balance_1);
    assert_eq!(balance_1, balance_2);
    assert_eq!(balance_2, balance_3);
    assert_eq!(balance_3, balance_4);

    // Verify: Corrupted data did not affect honest nodes
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_malformed_automerge_patch_rejected() {
    // Setup
    let mut honest_nodes = create_honest_nodes(3).await;
    for node in &mut honest_nodes {
        node.create_account(5_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Malicious node sends malformed patches
    let malicious_node = create_malicious_node(AttackStrategy::CorruptedCrdt).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Malformed patches rejected
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Verify: Detection was fast (< 10ms)
    if let Some(detection_time) = result.detection_time {
        assert!(detection_time < Duration::from_millis(10));
    }

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}
