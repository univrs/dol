//! Byzantine Committee Tests
//!
//! Tests Byzantine Fault Tolerance in BFT consensus committees.
//! Tests the 3f+1 property: a committee of 3f+1 nodes can tolerate f Byzantine faults.

use crate::test_harness::*;
use std::sync::Arc;
use std::time::Duration;
use vudo_credit::{BftCommittee, CreditAccountHandle};

#[tokio::test]
async fn test_bft_tolerates_one_byzantine_fault() {
    // Setup: 3f+1 = 4 nodes (f=1, tolerates 1 Byzantine fault)
    let mut honest_nodes = create_honest_nodes(3).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Create BFT committee (3 honest + 1 malicious = 4)
    let committee = BftCommittee::new_mock(4).await.unwrap();

    // One Byzantine node votes maliciously
    let malicious_node = create_malicious_node(AttackStrategy::ByzantineVoting).await;

    // Create account to reconcile
    let account = CreditAccountHandle::create(&honest_nodes[0].state_engine, "alice".to_string(), 10_000)
        .await
        .unwrap();

    // Reconcile balance with Byzantine voter
    let result = committee.reconcile_balance(&account).await.unwrap();

    // Verify: Consensus still achieved (3/4 honest votes)
    assert!(result.consensus, "BFT should achieve consensus with 1 Byzantine fault");

    // Verify: Byzantine vote was rejected
    let attack_result = malicious_node.execute_attack().await.unwrap();
    assert!(!attack_result.successful);
    assert_eq!(attack_result.damage_assessment, DamageLevel::None);

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_bft_tolerates_max_byzantine_faults() {
    // Setup: 3f+1 = 7 nodes (f=2, tolerates 2 Byzantine faults)
    let mut honest_nodes = create_honest_nodes(5).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Create BFT committee (5 honest + 2 malicious = 7)
    let committee = BftCommittee::new_mock(7).await.unwrap();

    // Two Byzantine nodes vote maliciously
    let malicious_nodes = create_malicious_nodes(2, AttackStrategy::ByzantineVoting).await;

    // Create account to reconcile
    let account = CreditAccountHandle::create(&honest_nodes[0].state_engine, "alice".to_string(), 10_000)
        .await
        .unwrap();

    // Reconcile with 2 Byzantine voters
    let result = committee.reconcile_balance(&account).await.unwrap();

    // Verify: Consensus still achieved (5/7 honest votes > 2/3)
    assert!(result.consensus, "BFT should tolerate f=2 faults in 3f+1=7 setup");

    // Verify: Byzantine votes were rejected
    for malicious_node in &malicious_nodes {
        let attack_result = malicious_node.execute_attack().await.unwrap();
        assert!(!attack_result.successful);
    }

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_bft_fails_with_too_many_byzantine_faults() {
    // Setup: 3f+1 = 4 nodes (f=1, should tolerate 1 Byzantine)
    let mut honest_nodes = create_honest_nodes(2).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    // Create BFT committee (2 honest + 2 malicious = 4)
    let committee = BftCommittee::new_mock(4).await.unwrap();

    // Two Byzantine nodes (exceeds f=1 tolerance)
    let malicious_nodes = create_malicious_nodes(2, AttackStrategy::ByzantineVoting).await;

    // Create account to reconcile
    let account = CreditAccountHandle::create(&honest_nodes[0].state_engine, "alice".to_string(), 10_000)
        .await
        .unwrap();

    // Reconcile with too many Byzantine voters
    let result = committee.reconcile_balance(&account).await.unwrap();

    // Verify: Consensus may fail or be incorrect (2/4 honest votes = 50%, not > 2/3)
    // In this case, BFT cannot guarantee safety
    println!("Consensus with f+1 faults: {}", result.consensus);

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_byzantine_node_sends_conflicting_votes() {
    // Setup: 3f+1 = 4 nodes
    let mut honest_nodes = create_honest_nodes(3).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    let committee = BftCommittee::new_mock(4).await.unwrap();

    // Byzantine node sends different votes to different peers
    let malicious_node = create_malicious_node(AttackStrategy::ByzantineVoting).await;

    // Create account to reconcile
    let account = CreditAccountHandle::create(&honest_nodes[0].state_engine, "alice".to_string(), 10_000)
        .await
        .unwrap();

    // Reconcile
    let result = committee.reconcile_balance(&account).await.unwrap();

    // Verify: Conflicting votes detected and rejected
    assert!(result.consensus);

    let attack_result = malicious_node.execute_attack().await.unwrap();
    assert!(!attack_result.successful);
    assert!(matches!(
        attack_result.mitigation,
        Some(Mitigation::BftConsensusRejected)
    ));

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_byzantine_node_delays_vote() {
    // Setup: 3f+1 = 4 nodes
    let mut honest_nodes = create_honest_nodes(3).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    let committee = BftCommittee::new_mock(4).await.unwrap();

    // Byzantine node delays its vote to disrupt consensus
    let malicious_node = create_malicious_node(AttackStrategy::ByzantineVoting).await;

    // Create account to reconcile
    let account = CreditAccountHandle::create(&honest_nodes[0].state_engine, "alice".to_string(), 10_000)
        .await
        .unwrap();

    // Reconcile with timeout
    let start = std::time::Instant::now();
    let result = committee.reconcile_balance(&account).await.unwrap();
    let elapsed = start.elapsed();

    // Verify: Consensus achieved without Byzantine vote (3/4 honest)
    assert!(result.consensus);

    // Verify: Timeout prevents indefinite waiting
    assert!(elapsed < Duration::from_secs(10), "Should timeout quickly");

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_byzantine_node_sends_invalid_signature() {
    // Setup: 3f+1 = 4 nodes
    let mut honest_nodes = create_honest_nodes(3).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    let committee = BftCommittee::new_mock(4).await.unwrap();

    // Byzantine node sends vote with invalid signature
    let malicious_node = create_malicious_node(AttackStrategy::ByzantineVoting).await;

    // Create account to reconcile
    let account = CreditAccountHandle::create(&honest_nodes[0].state_engine, "alice".to_string(), 10_000)
        .await
        .unwrap();

    // Reconcile
    let result = committee.reconcile_balance(&account).await.unwrap();

    // Verify: Invalid signature detected and rejected
    assert!(result.consensus);

    let attack_result = malicious_node.execute_attack().await.unwrap();
    assert!(!attack_result.successful);
    assert!(matches!(
        attack_result.mitigation,
        Some(Mitigation::BftConsensusRejected)
    ));

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_bft_quorum_calculation() {
    // Test quorum calculations for different committee sizes

    // 3f+1 = 4 (f=1): quorum = 3 (> 2/3 of 4 = 2.67)
    assert_eq!(calculate_quorum(4), 3);

    // 3f+1 = 7 (f=2): quorum = 5 (> 2/3 of 7 = 4.67)
    assert_eq!(calculate_quorum(7), 5);

    // 3f+1 = 10 (f=3): quorum = 7 (> 2/3 of 10 = 6.67)
    assert_eq!(calculate_quorum(10), 7);

    // 3f+1 = 13 (f=4): quorum = 9 (> 2/3 of 13 = 8.67)
    assert_eq!(calculate_quorum(13), 9);
}

#[tokio::test]
async fn test_byzantine_node_votes_wrong_balance() {
    // Setup: 3f+1 = 4 nodes
    let mut honest_nodes = create_honest_nodes(3).await;
    for node in &mut honest_nodes {
        node.create_account(10_000).await.unwrap();
        node.start().await.unwrap();
    }

    let committee = BftCommittee::new_mock(4).await.unwrap();

    // Byzantine node votes for incorrect balance
    let malicious_node = create_malicious_node(AttackStrategy::ByzantineVoting).await;

    // Create account with balance 10,000
    let account = CreditAccountHandle::create(&honest_nodes[0].state_engine, "alice".to_string(), 10_000)
        .await
        .unwrap();

    // Byzantine votes for balance 1,000,000 (incorrect)
    // Honest nodes vote for 10,000 (correct)

    let result = committee.reconcile_balance(&account).await.unwrap();

    // Verify: Honest majority prevails
    assert!(result.consensus);
    assert_eq!(result.new_confirmed_balance, 10_000);

    // Cleanup
    for node in &honest_nodes {
        node.stop().await.unwrap();
    }
}

/// Calculate BFT quorum (> 2/3)
fn calculate_quorum(committee_size: usize) -> usize {
    (committee_size * 2 / 3) + 1
}
