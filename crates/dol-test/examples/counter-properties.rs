//! Property-based tests for distributed counters
//!
//! This example demonstrates CRDT properties for:
//! - PN-Counter (Positive-Negative Counter)
//! - Multi-actor concurrent increments/decrements
//! - Partition tolerance
//!
//! Run with: cargo test --example counter-properties

use dol_test::generators::*;
use dol_test::properties::*;
use dol_test::TestResult;
use proptest::prelude::*;
use proptest::collection::vec;
use std::collections::HashMap;

/// A distributed counter with PN-Counter semantics
#[derive(Debug, Clone, PartialEq, Eq)]
struct DistributedCounter {
    /// Increment counters per actor
    increments: HashMap<String, i64>,

    /// Decrement counters per actor
    decrements: HashMap<String, i64>,

    /// Replica identifier
    replica_id: String,
}

impl Default for DistributedCounter {
    fn default() -> Self {
        Self {
            increments: HashMap::new(),
            decrements: HashMap::new(),
            replica_id: "default".to_string(),
        }
    }
}

impl DistributedCounter {
    /// Creates a new counter for a specific replica
    fn new(replica_id: String) -> Self {
        Self {
            increments: HashMap::new(),
            decrements: HashMap::new(),
            replica_id,
        }
    }

    /// Returns the current value of the counter
    fn value(&self) -> i64 {
        let total_increments: i64 = self.increments.values().sum();
        let total_decrements: i64 = self.decrements.values().sum();
        total_increments - total_decrements
    }

    /// Increments the counter
    fn increment(&mut self, amount: i64) {
        *self.increments.entry(self.replica_id.clone()).or_insert(0) += amount;
    }

    /// Decrements the counter
    fn decrement(&mut self, amount: i64) {
        *self.decrements.entry(self.replica_id.clone()).or_insert(0) += amount;
    }

    /// Returns the number of actors that have modified this counter
    fn actor_count(&self) -> usize {
        let mut actors = std::collections::HashSet::new();
        actors.extend(self.increments.keys().cloned());
        actors.extend(self.decrements.keys().cloned());
        actors.len()
    }
}

impl Mergeable for DistributedCounter {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Merge increments (max per actor)
        for (actor, count) in &other.increments {
            let entry = self.increments.entry(actor.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }

        // Merge decrements (max per actor)
        for (actor, count) in &other.decrements {
            let entry = self.decrements.entry(actor.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }

        Ok(())
    }

    fn replica_id(&self) -> String {
        self.replica_id.clone()
    }
}

impl Operable<CrdtOperation> for DistributedCounter {
    fn apply(&mut self, op: CrdtOperation) -> TestResult<()> {
        match op {
            CrdtOperation::PnCounterIncrement { actor, amount } => {
                *self.increments.entry(actor).or_insert(0) += amount;
            }
            CrdtOperation::PnCounterDecrement { actor, amount } => {
                *self.decrements.entry(actor).or_insert(0) += amount;
            }
            _ => {}
        }
        Ok(())
    }
}

impl MonotonicState for DistributedCounter {
    fn measure(&self) -> usize {
        self.increments.len() + self.decrements.len()
    }
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Test PN-Counter commutativity (Theorem 1)
    #[test]
    fn test_counter_commutativity(
        ops_a in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..30),
        ops_b in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..30)
    ) {
        let mut counter_a = DistributedCounter::default();
        let mut counter_b = DistributedCounter::default();

        for op in ops_a {
            counter_a.apply(op).unwrap();
        }
        for op in ops_b {
            counter_b.apply(op).unwrap();
        }

        prop_assert!(verify_commutativity(&counter_a, &counter_b));
    }

    /// Test PN-Counter associativity (Theorem 2)
    #[test]
    fn test_counter_associativity(
        ops_a in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..20),
        ops_b in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..20),
        ops_c in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..20)
    ) {
        let mut counter_a = DistributedCounter::default();
        let mut counter_b = DistributedCounter::default();
        let mut counter_c = DistributedCounter::default();

        for op in ops_a {
            counter_a.apply(op).unwrap();
        }
        for op in ops_b {
            counter_b.apply(op).unwrap();
        }
        for op in ops_c {
            counter_c.apply(op).unwrap();
        }

        prop_assert!(verify_associativity(&counter_a, &counter_b, &counter_c));
    }

    /// Test PN-Counter idempotency (Theorem 3)
    #[test]
    fn test_counter_idempotency(
        ops in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..30)
    ) {
        let mut counter = DistributedCounter::default();

        for op in ops {
            counter.apply(op).unwrap();
        }

        prop_assert!(verify_idempotency(&counter));
    }

    /// Test value convergence after merge
    #[test]
    fn test_value_convergence(
        ops_a in vec(pn_counter_increment_op(), 1..50),
        ops_b in vec(pn_counter_decrement_op(), 1..50)
    ) {
        let mut replica_a = DistributedCounter::new("replica_a".to_string());
        let mut replica_b = DistributedCounter::new("replica_b".to_string());

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        // Merge both ways
        let mut final_a = replica_a.clone();
        let mut final_b = replica_b.clone();

        final_a.merge(&replica_b).unwrap();
        final_b.merge(&replica_a).unwrap();

        // Values should be identical
        prop_assert_eq!(final_a.value(), final_b.value());
    }

    /// Test monotonicity of state size (Theorem 5.1)
    #[test]
    fn test_counter_monotonicity(
        ops in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..100)
    ) {
        let initial = DistributedCounter::default();
        prop_assert!(verify_monotonicity(&initial, ops));
    }

    /// Test partition tolerance (Theorem 11.1)
    #[test]
    fn test_counter_partition_tolerance(
        ops_partition_a in vec(pn_counter_increment_op(), 1..30),
        ops_partition_b in vec(pn_counter_decrement_op(), 1..30)
    ) {
        let mut replica_a = DistributedCounter::new("partition_a".to_string());
        let mut replica_b = DistributedCounter::new("partition_b".to_string());

        // Simulate partition: each partition processes operations independently
        for op in ops_partition_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_partition_b {
            replica_b.apply(op).unwrap();
        }

        // Heal partition by merging
        let mut healed_a = replica_a.clone();
        let mut healed_b = replica_b.clone();

        healed_a.merge(&replica_b).unwrap();
        healed_b.merge(&replica_a).unwrap();

        // Should converge after healing
        prop_assert_eq!(healed_a.value(), healed_b.value());
        prop_assert_eq!(healed_a, healed_b);
    }

    /// Test concurrent increments from multiple actors
    #[test]
    fn test_multi_actor_concurrent_increments(
        num_actors in 2usize..10,
        increments_per_actor in 1i64..50
    ) {
        // Create replicas for each actor
        let mut replicas: Vec<DistributedCounter> = (0..num_actors)
            .map(|i| DistributedCounter::new(format!("actor_{}", i)))
            .collect();

        // Each actor increments their local replica
        for replica in replicas.iter_mut() {
            replica.increment(increments_per_actor);
        }

        // Merge all replicas together
        let mut final_state = replicas[0].clone();
        for replica in &replicas[1..] {
            final_state.merge(replica).unwrap();
        }

        // Expected value: sum of all increments
        let expected_value = num_actors as i64 * increments_per_actor;
        prop_assert_eq!(final_state.value(), expected_value);
    }

    /// Test concurrent increments and decrements
    #[test]
    fn test_concurrent_inc_dec(
        increment_ops in vec(pn_counter_increment_op(), 10..30),
        decrement_ops in vec(pn_counter_decrement_op(), 10..30)
    ) {
        let mut replica_a = DistributedCounter::new("a".to_string());
        let mut replica_b = DistributedCounter::new("b".to_string());
        let mut replica_c = DistributedCounter::new("c".to_string());

        // Replica A: only increments
        for op in increment_ops.clone() {
            replica_a.apply(op).unwrap();
        }

        // Replica B: only decrements
        for op in decrement_ops.clone() {
            replica_b.apply(op).unwrap();
        }

        // Replica C: both (in mixed order)
        for op in increment_ops {
            replica_c.apply(op).unwrap();
        }
        for op in decrement_ops {
            replica_c.apply(op).unwrap();
        }

        // Merge all
        replica_a.merge(&replica_b).unwrap();
        replica_a.merge(&replica_c).unwrap();

        replica_b.merge(&replica_a).unwrap();
        replica_c.merge(&replica_a).unwrap();

        // All should converge to same value
        prop_assert_eq!(replica_a.value(), replica_b.value());
        prop_assert_eq!(replica_b.value(), replica_c.value());
    }

    /// Test that merge is deterministic regardless of order
    #[test]
    fn test_merge_determinism(
        ops in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 5..20)
    ) {
        // Create 3 replicas with same operations in different orders
        let mut replica_a = DistributedCounter::default();
        let mut replica_b = DistributedCounter::default();
        let mut replica_c = DistributedCounter::default();

        for op in ops.iter() {
            replica_a.apply(op.clone()).unwrap();
        }
        for op in ops.iter().rev() {
            replica_b.apply(op.clone()).unwrap();
        }
        for op in ops.iter() {
            replica_c.apply(op.clone()).unwrap();
        }

        // Merge in different orders
        let mut result_abc = replica_a.clone();
        result_abc.merge(&replica_b).unwrap();
        result_abc.merge(&replica_c).unwrap();

        let mut result_bca = replica_b.clone();
        result_bca.merge(&replica_c).unwrap();
        result_bca.merge(&replica_a).unwrap();

        let mut result_cab = replica_c.clone();
        result_cab.merge(&replica_a).unwrap();
        result_cab.merge(&replica_b).unwrap();

        // All merge orders should produce same result
        prop_assert_eq!(result_abc.value(), result_bca.value());
        prop_assert_eq!(result_bca.value(), result_cab.value());
    }
}

fn main() {
    println!("Run with: cargo test --example counter-properties");
}
