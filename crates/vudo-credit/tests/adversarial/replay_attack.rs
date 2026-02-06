//! Replay Attack Tests
//!
//! Tests Byzantine fault tolerance against replay attacks where an attacker
//! attempts to replay old transactions to spend credit twice.

use crate::test_harness::*;
use std::time::Duration;
use vudo_credit::TransactionMetadata;

#[tokio::test]
async fn test_replay_attack_prevented() {
    // Setup: Alice and Bob
    let mut alice = HonestNode::new("alice".to_string()).await.unwrap();
    alice.create_account(10_000).await.unwrap();
    alice.start().await.unwrap();

    let mut bob = HonestNode::new("bob".to_string()).await.unwrap();
    bob.create_account(10_000).await.unwrap();
    bob.start().await.unwrap();

    // Alice pays Bob $10.00
    let original_tx = alice.pay("bob", 1000).await.unwrap();

    // Wait for transaction to confirm
    wait_for_bft_confirmation(&original_tx).await;

    let alice_balance_before = alice.get_balance().await;
    let bob_balance_before = bob.get_balance().await;

    // Attack: Mallory intercepts and replays the transaction
    let malicious_node = create_malicious_node(AttackStrategy::ReplayAttack).await;
    let result = malicious_node.execute_attack().await.unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify: Replay is rejected
    let alice_balance_after = alice.get_balance().await;
    let bob_balance_after = bob.get_balance().await;

    assert_eq!(alice_balance_after, alice_balance_before);
    assert_eq!(bob_balance_after, bob_balance_before);

    // Verify: Attack was unsuccessful
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Verify: Replay was detected quickly
    if let Some(detection_time) = result.detection_time {
        assert!(detection_time < Duration::from_millis(10));
    }

    // Cleanup
    alice.stop().await.unwrap();
    bob.stop().await.unwrap();
}

#[tokio::test]
async fn test_transaction_id_uniqueness() {
    // Setup
    let mut alice = HonestNode::new("alice".to_string()).await.unwrap();
    alice.create_account(10_000).await.unwrap();
    alice.start().await.unwrap();

    let mut bob = HonestNode::new("bob".to_string()).await.unwrap();
    bob.create_account(10_000).await.unwrap();
    bob.start().await.unwrap();

    // Create multiple transactions
    let tx1 = alice.pay("bob", 100).await.unwrap();
    let tx2 = alice.pay("bob", 200).await.unwrap();
    let tx3 = alice.pay("bob", 300).await.unwrap();

    // Verify: All transaction IDs are unique
    assert_ne!(tx1, tx2);
    assert_ne!(tx2, tx3);
    assert_ne!(tx1, tx3);

    // Attempt to replay tx1
    let malicious_node = create_malicious_node(AttackStrategy::ReplayAttack).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Duplicate transaction ID is detected
    assert!(!result.successful);
    assert!(matches!(
        result.mitigation,
        Some(Mitigation::OperationRejected { .. })
    ));

    // Cleanup
    alice.stop().await.unwrap();
    bob.stop().await.unwrap();
}

#[tokio::test]
async fn test_replay_after_network_partition() {
    // Setup: Two nodes that will be partitioned
    let mut alice = HonestNode::new("alice".to_string()).await.unwrap();
    alice.create_account(10_000).await.unwrap();
    alice.start().await.unwrap();

    let mut bob = HonestNode::new("bob".to_string()).await.unwrap();
    bob.create_account(10_000).await.unwrap();
    bob.start().await.unwrap();

    // Alice pays Bob before partition
    let tx_id = alice.pay("bob", 500).await.unwrap();
    wait_for_bft_confirmation(&tx_id).await;

    // Simulate network partition
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Attempt replay during partition
    let malicious_node = create_malicious_node(AttackStrategy::ReplayAttack).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Replay is still prevented (transaction ID persists)
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Cleanup
    alice.stop().await.unwrap();
    bob.stop().await.unwrap();
}

#[tokio::test]
async fn test_replay_with_modified_amount() {
    // Setup
    let mut alice = HonestNode::new("alice".to_string()).await.unwrap();
    alice.create_account(10_000).await.unwrap();
    alice.start().await.unwrap();

    let mut bob = HonestNode::new("bob".to_string()).await.unwrap();
    bob.create_account(10_000).await.unwrap();
    bob.start().await.unwrap();

    // Alice pays Bob $5.00
    let original_tx = alice.pay("bob", 500).await.unwrap();
    wait_for_bft_confirmation(&original_tx).await;

    // Attack: Try to replay with modified amount ($50.00)
    // This would change the transaction hash/signature
    let malicious_node = create_malicious_node(AttackStrategy::ReplayAttack).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Modified replay is rejected (signature invalid)
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Cleanup
    alice.stop().await.unwrap();
    bob.stop().await.unwrap();
}

#[tokio::test]
async fn test_replay_from_multiple_sources() {
    // Setup: Alice, Bob, and multiple malicious nodes
    let mut alice = HonestNode::new("alice".to_string()).await.unwrap();
    alice.create_account(10_000).await.unwrap();
    alice.start().await.unwrap();

    let mut bob = HonestNode::new("bob".to_string()).await.unwrap();
    bob.create_account(10_000).await.unwrap();
    bob.start().await.unwrap();

    // Alice pays Bob
    let tx_id = alice.pay("bob", 1000).await.unwrap();
    wait_for_bft_confirmation(&tx_id).await;

    let balance_before = alice.get_balance().await;

    // Attack: Multiple malicious nodes replay the same transaction
    let malicious_nodes = create_malicious_nodes(5, AttackStrategy::ReplayAttack).await;

    let mut results = vec![];
    for node in &malicious_nodes {
        let result = node.execute_attack().await.unwrap();
        results.push(result);
    }

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify: All replays rejected
    for result in &results {
        assert!(!result.successful);
        assert_eq!(result.damage_assessment, DamageLevel::None);
    }

    // Verify: Balance unchanged
    let balance_after = alice.get_balance().await;
    assert_eq!(balance_after, balance_before);

    // Cleanup
    alice.stop().await.unwrap();
    bob.stop().await.unwrap();
}

#[tokio::test]
async fn test_nonce_prevents_replay() {
    // Setup
    let mut alice = HonestNode::new("alice".to_string()).await.unwrap();
    alice.create_account(10_000).await.unwrap();
    alice.start().await.unwrap();

    let mut bob = HonestNode::new("bob".to_string()).await.unwrap();
    bob.create_account(10_000).await.unwrap();
    bob.start().await.unwrap();

    // Create transactions with increasing nonces
    let tx1 = alice.pay("bob", 100).await.unwrap();
    let tx2 = alice.pay("bob", 100).await.unwrap();

    // Wait for confirmation
    wait_for_bft_confirmation(&tx1).await;
    wait_for_bft_confirmation(&tx2).await;

    // Attempt to replay tx1
    let malicious_node = create_malicious_node(AttackStrategy::ReplayAttack).await;
    let result = malicious_node.execute_attack().await.unwrap();

    // Verify: Nonce check prevents replay
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // Verify: Detection was fast
    if let Some(detection_time) = result.detection_time {
        assert!(detection_time < Duration::from_millis(5));
    }

    // Cleanup
    alice.stop().await.unwrap();
    bob.stop().await.unwrap();
}
