//! CRDT property definitions and verification
//!
//! This module implements the 13 formal theorems from RFC-001 as testable properties.
//! Each property can be verified using the test harness with generated operation sequences.
//!
//! # Fundamental Properties (Theorems 1-3)
//!
//! - **Commutativity**: `merge(a, b) = merge(b, a)`
//! - **Associativity**: `merge(merge(a, b), c) = merge(a, merge(b, c))`
//! - **Idempotency**: `merge(a, a) = a`
//!
//! # Strategy-Specific Properties (Theorems 2.1-7.1)
//!
//! Each CRDT strategy has specific convergence properties that must hold.
//!
//! # Example
//!
//! ```rust
//! use dol_test::properties::*;
//!
//! let mut state_a = CrdtState::new();
//! let mut state_b = CrdtState::new();
//!
//! // Apply operations...
//!
//! // Test commutativity
//! assert!(verify_commutativity(&state_a, &state_b));
//! ```

use crate::TestResult;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait for types that support CRDT merge semantics
pub trait Mergeable: Clone + PartialEq + Debug {
    /// Merges another state into this state
    fn merge(&mut self, other: &Self) -> TestResult<()>;

    /// Returns a unique identifier for this replica
    fn replica_id(&self) -> String;
}

/// Trait for types that can be queried for their value
pub trait Queryable {
    /// The value type returned by queries
    type Value: PartialEq + Debug;

    /// Queries the current value
    fn query(&self) -> Self::Value;
}

/// Trait for types that support CRDT operations
pub trait Operable<Op> {
    /// Applies an operation to this state
    fn apply(&mut self, operation: Op) -> TestResult<()>;
}

/// Theorem 1: Commutativity
///
/// For any two states s₁ and s₂:
/// `merge(s₁, s₂) = merge(s₂, s₁)`
///
/// This is the fundamental property ensuring that replicas can merge in any order.
pub fn verify_commutativity<T: Mergeable>(state_a: &T, state_b: &T) -> bool {
    let mut ab = state_a.clone();
    let mut ba = state_b.clone();

    // Attempt to merge in both directions
    if ab.merge(state_b).is_err() || ba.merge(state_a).is_err() {
        return false;
    }

    ab == ba
}

/// Theorem 2: Associativity
///
/// For any three states s₁, s₂, s₃:
/// `merge(merge(s₁, s₂), s₃) = merge(s₁, merge(s₂, s₃))`
///
/// This ensures that merge operations can be grouped in any order.
pub fn verify_associativity<T: Mergeable>(state_a: &T, state_b: &T, state_c: &T) -> bool {
    // Left-associative: (a ⊔ b) ⊔ c
    let mut ab = state_a.clone();
    if ab.merge(state_b).is_err() {
        return false;
    }
    let mut ab_c = ab.clone();
    if ab_c.merge(state_c).is_err() {
        return false;
    }

    // Right-associative: a ⊔ (b ⊔ c)
    let mut bc = state_b.clone();
    if bc.merge(state_c).is_err() {
        return false;
    }
    let mut a_bc = state_a.clone();
    if a_bc.merge(&bc).is_err() {
        return false;
    }

    ab_c == a_bc
}

/// Theorem 3: Idempotency
///
/// For any state s:
/// `merge(s, s) = s`
///
/// This ensures that duplicate merge operations have no effect.
pub fn verify_idempotency<T: Mergeable>(state: &T) -> bool {
    let mut merged = state.clone();
    if merged.merge(state).is_err() {
        return false;
    }

    merged == *state
}

/// Eventual Consistency Property
///
/// For any two replicas that have delivered the same set of operations:
/// `state(replica_a) ≡ state(replica_b)`
///
/// This is verified by applying the same operations in potentially different orders
/// and checking that merge produces equivalent states.
pub fn verify_eventual_consistency<T, Op>(
    operations: Vec<Op>,
    num_replicas: usize,
) -> bool
where
    T: Mergeable + Operable<Op> + Default,
    Op: Clone,
{
    // Create replicas
    let mut replicas: Vec<T> = (0..num_replicas).map(|_| T::default()).collect();

    // Apply operations in different orders to different replicas
    for (idx, replica) in replicas.iter_mut().enumerate() {
        let mut ops = operations.clone();

        // Shuffle operations based on replica index to simulate different delivery orders
        // In a real implementation, we'd use a proper shuffling algorithm
        if idx > 0 {
            ops.reverse(); // Simple ordering variation for testing
        }

        for op in ops {
            if replica.apply(op).is_err() {
                return false;
            }
        }
    }

    // Merge all replicas together
    let mut final_state = replicas[0].clone();
    for replica in &replicas[1..] {
        if final_state.merge(replica).is_err() {
            return false;
        }
    }

    // Verify all replicas converge to the same state after merging
    for replica in &mut replicas {
        if replica.merge(&final_state).is_err() {
            return false;
        }
        if *replica != final_state {
            return false;
        }
    }

    true
}

/// Monotonicity Property
///
/// For certain CRDT strategies (PN-Counter, OR-Set, RGA), the state should only grow:
/// `∀s ∈ S, ∀op ∈ O: s ≤ apply(s, op)`
///
/// This is verified by checking that operations always increase or maintain the state.
pub fn verify_monotonicity<T, Op>(initial_state: &T, operations: Vec<Op>) -> bool
where
    T: Mergeable + Operable<Op> + MonotonicState,
    Op: Clone,
{
    let mut state = initial_state.clone();

    for op in operations {
        let prev_measure = state.measure();

        if state.apply(op).is_err() {
            return false;
        }

        let new_measure = state.measure();

        // State should not decrease
        if new_measure < prev_measure {
            return false;
        }
    }

    true
}

/// Trait for states that have a monotonic measure
pub trait MonotonicState {
    /// Returns a measure of the state size (must be monotonic)
    fn measure(&self) -> usize;
}

/// Causal Consistency Property
///
/// For operations with happens-before relationships:
/// If `op_a → op_b`, then `op_a` must be applied before `op_b`
///
/// This is critical for RGA and Peritext strategies.
pub fn verify_causal_consistency<T, Op>(
    operations: Vec<(Op, Option<usize>)>, // (operation, causal_dependency_index)
) -> bool
where
    T: Mergeable + Operable<Op> + Default,
    Op: Clone,
{
    let mut state = T::default();
    let mut applied_ops: Vec<bool> = vec![false; operations.len()];

    for (idx, (op, dependency)) in operations.iter().enumerate() {
        // Check if causal dependency is satisfied
        if let Some(dep_idx) = dependency {
            if !applied_ops[*dep_idx] {
                // Dependency not yet applied - this violates causal order
                return false;
            }
        }

        if state.apply(op.clone()).is_err() {
            return false;
        }

        applied_ops[idx] = true;
    }

    true
}

/// Network Partition Tolerance
///
/// Verifies that replicas can operate during partition and converge after healing.
///
/// Simulates a network partition by:
/// 1. Partitioning replicas into two groups
/// 2. Allowing each group to process operations independently
/// 3. Healing the partition by merging
/// 4. Verifying convergence
pub fn verify_partition_tolerance<T, Op>(
    operations_partition_a: Vec<Op>,
    operations_partition_b: Vec<Op>,
) -> bool
where
    T: Mergeable + Operable<Op> + Default,
    Op: Clone,
{
    // Create replicas in each partition
    let mut replica_a = T::default();
    let mut replica_b = T::default();

    // Process operations independently in each partition
    for op in operations_partition_a {
        if replica_a.apply(op).is_err() {
            return false;
        }
    }

    for op in operations_partition_b {
        if replica_b.apply(op).is_err() {
            return false;
        }
    }

    // Heal partition by merging
    let mut final_a = replica_a.clone();
    let mut final_b = replica_b.clone();

    if final_a.merge(&replica_b).is_err() || final_b.merge(&replica_a).is_err() {
        return false;
    }

    // Verify convergence (Theorem 1: Commutativity)
    final_a == final_b
}

/// Property test result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyResult {
    /// Property holds
    Pass,
    /// Property violated
    Fail { reason: String },
    /// Property test inconclusive (e.g., due to error)
    Inconclusive { reason: String },
}

impl PropertyResult {
    /// Returns true if the property passed
    pub fn is_pass(&self) -> bool {
        matches!(self, PropertyResult::Pass)
    }

    /// Returns true if the property failed
    pub fn is_fail(&self) -> bool {
        matches!(self, PropertyResult::Fail { .. })
    }
}

/// Comprehensive property test suite for a CRDT implementation
///
/// This runs all applicable property tests and returns a detailed report.
pub struct PropertyTestSuite<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> PropertyTestSuite<T>
where
    T: Mergeable,
{
    /// Tests all fundamental properties (commutativity, associativity, idempotency)
    pub fn test_fundamental_properties(
        state_a: &T,
        state_b: &T,
        state_c: &T,
    ) -> Vec<(String, PropertyResult)> {
        let mut results = Vec::new();

        // Test commutativity
        let comm_result = if verify_commutativity(state_a, state_b) {
            PropertyResult::Pass
        } else {
            PropertyResult::Fail {
                reason: "merge(a,b) != merge(b,a)".to_string(),
            }
        };
        results.push(("Commutativity".to_string(), comm_result));

        // Test associativity
        let assoc_result = if verify_associativity(state_a, state_b, state_c) {
            PropertyResult::Pass
        } else {
            PropertyResult::Fail {
                reason: "merge(merge(a,b),c) != merge(a,merge(b,c))".to_string(),
            }
        };
        results.push(("Associativity".to_string(), assoc_result));

        // Test idempotency
        let idem_result = if verify_idempotency(state_a) {
            PropertyResult::Pass
        } else {
            PropertyResult::Fail {
                reason: "merge(a,a) != a".to_string(),
            }
        };
        results.push(("Idempotency".to_string(), idem_result));

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test CRDT for demonstration
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestCounter {
        value: i64,
        replica: String,
    }

    impl Default for TestCounter {
        fn default() -> Self {
            Self {
                value: 0,
                replica: "test".to_string(),
            }
        }
    }

    impl Mergeable for TestCounter {
        fn merge(&mut self, other: &Self) -> TestResult<()> {
            // Max merge (simplified LWW for testing)
            self.value = self.value.max(other.value);
            Ok(())
        }

        fn replica_id(&self) -> String {
            self.replica.clone()
        }
    }

    #[derive(Debug, Clone)]
    struct Increment;

    impl Operable<Increment> for TestCounter {
        fn apply(&mut self, _op: Increment) -> TestResult<()> {
            self.value += 1;
            Ok(())
        }
    }

    impl MonotonicState for TestCounter {
        fn measure(&self) -> usize {
            self.value.max(0) as usize
        }
    }

    #[test]
    fn test_commutativity_simple() {
        let state_a = TestCounter { value: 10, replica: "test".to_string() };
        let state_b = TestCounter { value: 20, replica: "test".to_string() };

        assert!(verify_commutativity(&state_a, &state_b));
    }

    #[test]
    fn test_associativity_simple() {
        let state_a = TestCounter { value: 10, replica: "test".to_string() };
        let state_b = TestCounter { value: 20, replica: "test".to_string() };
        let state_c = TestCounter { value: 15, replica: "test".to_string() };

        assert!(verify_associativity(&state_a, &state_b, &state_c));
    }

    #[test]
    fn test_idempotency_simple() {
        let state = TestCounter { value: 42, replica: "test".to_string() };
        assert!(verify_idempotency(&state));
    }

    #[test]
    fn test_monotonicity_simple() {
        let initial = TestCounter::default();
        let ops = vec![Increment, Increment, Increment];
        assert!(verify_monotonicity(&initial, ops));
    }

    #[test]
    fn test_property_result_constructors() {
        let pass = PropertyResult::Pass;
        assert!(pass.is_pass());
        assert!(!pass.is_fail());

        let fail = PropertyResult::Fail { reason: "test".to_string() };
        assert!(!fail.is_pass());
        assert!(fail.is_fail());
    }
}
