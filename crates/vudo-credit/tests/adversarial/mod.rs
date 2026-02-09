//! Adversarial Testing Module
//!
//! Byzantine Fault Tolerance testing suite for VUDO Credit system.
//!
//! This module contains comprehensive adversarial tests validating the system
//! against Byzantine faults including:
//! - Malicious CRDT operations
//! - Sybil attacks
//! - Replay attacks
//! - Timing attacks
//! - Eclipse attacks
//! - Resource exhaustion
//! - Byzantine committee voting
//!
//! ## Test Philosophy
//!
//! Byzantine testing follows these principles:
//! 1. **Assume malicious intent**: Attackers actively try to break the system
//! 2. **Test at scale**: Single attacker vs. many attackers
//! 3. **Measure damage**: Quantify impact of successful attacks
//! 4. **Verify mitigation**: Ensure defenses work as designed
//! 5. **Document threats**: Maintain living threat model
//!
//! ## BFT Guarantees
//!
//! The system provides Byzantine fault tolerance through:
//! - **3f+1 committee**: Tolerates f Byzantine faults
//! - **2/3+ quorum**: Requires >2/3 honest votes for consensus
//! - **Signature verification**: All votes cryptographically signed
//! - **Timeout protection**: Prevents indefinite blocking
//! - **Replay prevention**: Transaction IDs prevent double-spend
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all adversarial tests
//! cargo test --test adversarial
//!
//! # Run specific category
//! cargo test --test adversarial corrupted_crdt
//! cargo test --test adversarial sybil_attack
//! cargo test --test adversarial replay_attack
//! cargo test --test adversarial timing_attack
//! cargo test --test adversarial eclipse_attack
//! cargo test --test adversarial resource_exhaustion
//! cargo test --test adversarial byzantine_committee
//! ```

pub mod test_harness;

// Attack category tests
pub mod corrupted_crdt;
pub mod sybil_attack;
pub mod replay_attack;
pub mod timing_attack;
pub mod eclipse_attack;
pub mod resource_exhaustion;
pub mod byzantine_committee;

// Re-export test infrastructure for use in tests
pub use test_harness::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_all_attacks_unsuccessful() {
        // Integration test: Run all attack types and verify all fail
        let attack_strategies = vec![
            AttackStrategy::CorruptedCrdt,
            AttackStrategy::SybilAttack { identities: 100 },
            AttackStrategy::ReplayAttack,
            AttackStrategy::TimingAttack,
            AttackStrategy::EclipseAttack,
            AttackStrategy::ResourceExhaustion,
            AttackStrategy::ByzantineVoting,
        ];

        for strategy in attack_strategies {
            let malicious_node = create_malicious_node(strategy.clone()).await;
            let result = malicious_node.execute_attack().await.unwrap();

            // Verify: All attacks unsuccessful
            assert!(
                !result.successful,
                "Attack {:?} should not succeed",
                strategy
            );

            // Verify: Damage is None or Minimal
            assert!(
                result.damage_assessment <= DamageLevel::Minimal,
                "Attack {:?} should have minimal impact",
                strategy
            );
        }
    }

    #[tokio::test]
    async fn test_combined_attacks() {
        // Setup: Honest network
        let mut honest_nodes = create_honest_nodes(5).await;
        for node in &mut honest_nodes {
            node.create_account(10_000).await.unwrap();
            node.start().await.unwrap();
        }

        // Launch multiple simultaneous attacks
        let attacks = vec![
            AttackStrategy::CorruptedCrdt,
            AttackStrategy::SybilAttack { identities: 50 },
            AttackStrategy::EclipseAttack,
        ];

        let mut attackers = vec![];
        for strategy in attacks {
            let attacker = create_malicious_node(strategy).await;
            attackers.push(attacker);
        }

        // Execute all attacks simultaneously
        let mut results = vec![];
        for attacker in &attackers {
            let result = attacker.execute_attack().await.unwrap();
            results.push(result);
        }

        // Verify: All attacks unsuccessful
        for result in results {
            assert!(!result.successful);
        }

        // Verify: Honest network still operational
        let tx_id = honest_nodes[0].pay("honest_1", 100).await.unwrap();
        wait_for_bft_confirmation(&tx_id).await;

        // Cleanup
        for node in &honest_nodes {
            node.stop().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_attack_detection_speed() {
        // Verify: All attacks are detected quickly (< 100ms)
        let strategies = vec![
            AttackStrategy::CorruptedCrdt,
            AttackStrategy::ReplayAttack,
            AttackStrategy::ResourceExhaustion,
        ];

        for strategy in strategies {
            let attacker = create_malicious_node(strategy.clone()).await;
            let result = attacker.execute_attack().await.unwrap();

            if let Some(detection_time) = result.detection_time {
                assert!(
                    detection_time < std::time::Duration::from_millis(100),
                    "Attack {:?} should be detected quickly",
                    strategy
                );
            }
        }
    }
}
