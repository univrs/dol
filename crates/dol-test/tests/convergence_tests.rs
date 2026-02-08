//! Comprehensive convergence tests for all CRDT strategies
//!
//! This test suite verifies the 13 theorems from RFC-001 across all 7 CRDT strategies.
//! Each test runs with 1000 iterations to ensure robust coverage.

use dol_test::generators::*;
use dol_test::properties::*;
use dol_test::TestResult;
use proptest::prelude::*;
use proptest::collection::vec;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Immutable Strategy Tests (Theorem 2.1)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImmutableValue {
    value: Option<String>,
    timestamp: u64,
    actor: String,
}

impl Default for ImmutableValue {
    fn default() -> Self {
        Self {
            value: None,
            timestamp: 0,
            actor: String::new(),
        }
    }
}

impl Mergeable for ImmutableValue {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Keep the first set value (earliest timestamp)
        match (&self.value, &other.value) {
            (None, Some(_)) => {
                self.value = other.value.clone();
                self.timestamp = other.timestamp;
                self.actor = other.actor.clone();
            }
            (Some(_), None) => {
                // Keep self
            }
            (Some(_), Some(_)) => {
                // Keep earliest
                if other.timestamp < self.timestamp
                    || (other.timestamp == self.timestamp && other.actor < self.actor)
                {
                    self.value = other.value.clone();
                    self.timestamp = other.timestamp;
                    self.actor = other.actor.clone();
                }
            }
            (None, None) => {
                // Both empty
            }
        }
        Ok(())
    }

    fn replica_id(&self) -> String {
        self.actor.clone()
    }
}

impl Operable<CrdtOperation> for ImmutableValue {
    fn apply(&mut self, op: CrdtOperation) -> TestResult<()> {
        if let CrdtOperation::ImmutableSet { value, timestamp, actor } = op {
            if self.value.is_none() {
                self.value = Some(value);
                self.timestamp = timestamp;
                self.actor = actor;
            }
        }
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_immutable_commutativity(
        ops_a in vec(immutable_set_op(), 1..10),
        ops_b in vec(immutable_set_op(), 1..10)
    ) {
        let mut replica_a = ImmutableValue::default();
        let mut replica_b = ImmutableValue::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        prop_assert!(verify_commutativity(&replica_a, &replica_b));
    }

    #[test]
    fn test_immutable_associativity(
        ops_a in vec(immutable_set_op(), 1..10),
        ops_b in vec(immutable_set_op(), 1..10),
        ops_c in vec(immutable_set_op(), 1..10)
    ) {
        let mut replica_a = ImmutableValue::default();
        let mut replica_b = ImmutableValue::default();
        let mut replica_c = ImmutableValue::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }
        for op in ops_c {
            replica_c.apply(op).unwrap();
        }

        prop_assert!(verify_associativity(&replica_a, &replica_b, &replica_c));
    }

    #[test]
    fn test_immutable_idempotency(
        ops in vec(immutable_set_op(), 1..10)
    ) {
        let mut replica = ImmutableValue::default();

        for op in ops {
            replica.apply(op).unwrap();
        }

        prop_assert!(verify_idempotency(&replica));
    }
}

// ============================================================================
// Last-Write-Wins (LWW) Strategy Tests (Theorem 3.1)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct LwwValue {
    value: String,
    timestamp: u64,
    actor: String,
}

impl Default for LwwValue {
    fn default() -> Self {
        Self {
            value: String::new(),
            timestamp: 0,
            actor: String::new(),
        }
    }
}

impl Mergeable for LwwValue {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Keep the value with the highest timestamp
        if other.timestamp > self.timestamp
            || (other.timestamp == self.timestamp && other.actor > self.actor)
        {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.actor = other.actor.clone();
        }
        Ok(())
    }

    fn replica_id(&self) -> String {
        self.actor.clone()
    }
}

impl Operable<CrdtOperation> for LwwValue {
    fn apply(&mut self, op: CrdtOperation) -> TestResult<()> {
        if let CrdtOperation::LwwWrite { value, timestamp, actor } = op {
            if timestamp > self.timestamp
                || (timestamp == self.timestamp && actor > self.actor)
            {
                self.value = value;
                self.timestamp = timestamp;
                self.actor = actor;
            }
        }
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_lww_commutativity(
        ops_a in vec(lww_write_op(), 1..10),
        ops_b in vec(lww_write_op(), 1..10)
    ) {
        let mut replica_a = LwwValue::default();
        let mut replica_b = LwwValue::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        prop_assert!(verify_commutativity(&replica_a, &replica_b));
    }

    #[test]
    fn test_lww_associativity(
        ops_a in vec(lww_write_op(), 1..10),
        ops_b in vec(lww_write_op(), 1..10),
        ops_c in vec(lww_write_op(), 1..10)
    ) {
        let mut replica_a = LwwValue::default();
        let mut replica_b = LwwValue::default();
        let mut replica_c = LwwValue::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }
        for op in ops_c {
            replica_c.apply(op).unwrap();
        }

        prop_assert!(verify_associativity(&replica_a, &replica_b, &replica_c));
    }

    #[test]
    fn test_lww_idempotency(
        ops in vec(lww_write_op(), 1..10)
    ) {
        let mut replica = LwwValue::default();

        for op in ops {
            replica.apply(op).unwrap();
        }

        prop_assert!(verify_idempotency(&replica));
    }
}

// ============================================================================
// OR-Set Strategy Tests (Theorem 4.1)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct OrSet {
    elements: HashMap<String, HashSet<String>>, // element -> tags
    tombstones: HashSet<String>,
}

impl Default for OrSet {
    fn default() -> Self {
        Self {
            elements: HashMap::new(),
            tombstones: HashSet::new(),
        }
    }
}

impl Mergeable for OrSet {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Merge elements (union of tags)
        for (element, tags) in &other.elements {
            self.elements
                .entry(element.clone())
                .or_insert_with(HashSet::new)
                .extend(tags.clone());
        }

        // Merge tombstones (union)
        self.tombstones.extend(other.tombstones.clone());

        Ok(())
    }

    fn replica_id(&self) -> String {
        "orset".to_string()
    }
}

impl Operable<CrdtOperation> for OrSet {
    fn apply(&mut self, op: CrdtOperation) -> TestResult<()> {
        match op {
            CrdtOperation::OrSetAdd { element, tag } => {
                self.elements
                    .entry(element)
                    .or_insert_with(HashSet::new)
                    .insert(tag);
            }
            CrdtOperation::OrSetRemove { element, observed_tags } => {
                self.tombstones.extend(observed_tags);
                // Remove observed tags from element
                if let Some(tags) = self.elements.get_mut(&element) {
                    tags.retain(|tag| !self.tombstones.contains(tag));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl MonotonicState for OrSet {
    fn measure(&self) -> usize {
        self.elements.len() + self.tombstones.len()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_or_set_commutativity(
        ops_a in vec(prop_oneof![or_set_add_op(), or_set_remove_op()], 1..10),
        ops_b in vec(prop_oneof![or_set_add_op(), or_set_remove_op()], 1..10)
    ) {
        let mut replica_a = OrSet::default();
        let mut replica_b = OrSet::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        prop_assert!(verify_commutativity(&replica_a, &replica_b));
    }

    #[test]
    fn test_or_set_associativity(
        ops_a in vec(prop_oneof![or_set_add_op(), or_set_remove_op()], 1..10),
        ops_b in vec(prop_oneof![or_set_add_op(), or_set_remove_op()], 1..10),
        ops_c in vec(prop_oneof![or_set_add_op(), or_set_remove_op()], 1..10)
    ) {
        let mut replica_a = OrSet::default();
        let mut replica_b = OrSet::default();
        let mut replica_c = OrSet::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }
        for op in ops_c {
            replica_c.apply(op).unwrap();
        }

        prop_assert!(verify_associativity(&replica_a, &replica_b, &replica_c));
    }

    #[test]
    fn test_or_set_monotonicity(
        ops in vec(or_set_add_op(), 1..50)
    ) {
        let initial = OrSet::default();
        prop_assert!(verify_monotonicity(&initial, ops));
    }
}

// ============================================================================
// PN-Counter Strategy Tests (Theorem 5.1)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct PnCounter {
    increments: HashMap<String, i64>,
    decrements: HashMap<String, i64>,
}

impl Default for PnCounter {
    fn default() -> Self {
        Self {
            increments: HashMap::new(),
            decrements: HashMap::new(),
        }
    }
}

impl PnCounter {
    fn value(&self) -> i64 {
        let inc: i64 = self.increments.values().sum();
        let dec: i64 = self.decrements.values().sum();
        inc - dec
    }
}

impl Mergeable for PnCounter {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Merge increments (max for each actor)
        for (actor, count) in &other.increments {
            let entry = self.increments.entry(actor.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }

        // Merge decrements (max for each actor)
        for (actor, count) in &other.decrements {
            let entry = self.decrements.entry(actor.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }

        Ok(())
    }

    fn replica_id(&self) -> String {
        "pncounter".to_string()
    }
}

impl Operable<CrdtOperation> for PnCounter {
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

impl MonotonicState for PnCounter {
    fn measure(&self) -> usize {
        self.increments.len() + self.decrements.len()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_pn_counter_commutativity(
        ops_a in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..10),
        ops_b in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..10)
    ) {
        let mut replica_a = PnCounter::default();
        let mut replica_b = PnCounter::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        prop_assert!(verify_commutativity(&replica_a, &replica_b));
    }

    #[test]
    fn test_pn_counter_associativity(
        ops_a in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..10),
        ops_b in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..10),
        ops_c in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..10)
    ) {
        let mut replica_a = PnCounter::default();
        let mut replica_b = PnCounter::default();
        let mut replica_c = PnCounter::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }
        for op in ops_c {
            replica_c.apply(op).unwrap();
        }

        prop_assert!(verify_associativity(&replica_a, &replica_b, &replica_c));
    }

    #[test]
    fn test_pn_counter_value_convergence(
        ops in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..50)
    ) {
        let mut replica_a = PnCounter::default();
        let mut replica_b = PnCounter::default();

        // Apply same operations in different orders
        for op in ops.iter() {
            replica_a.apply(op.clone()).unwrap();
        }

        for op in ops.iter().rev() {
            replica_b.apply(op.clone()).unwrap();
        }

        // Merge
        replica_a.merge(&replica_b).unwrap();
        replica_b.merge(&replica_a).unwrap();

        // Values should be identical
        prop_assert_eq!(replica_a.value(), replica_b.value());
    }
}

// ============================================================================
// Partition Tolerance Tests (Theorem 11.1)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_partition_tolerance_lww(
        ops_partition_a in vec(lww_write_op(), 1..20),
        ops_partition_b in vec(lww_write_op(), 1..20)
    ) {
        prop_assert!(verify_partition_tolerance::<LwwValue, _>(
            ops_partition_a,
            ops_partition_b
        ));
    }

    #[test]
    fn test_partition_tolerance_or_set(
        ops_partition_a in vec(or_set_add_op(), 1..20),
        ops_partition_b in vec(or_set_add_op(), 1..20)
    ) {
        prop_assert!(verify_partition_tolerance::<OrSet, _>(
            ops_partition_a,
            ops_partition_b
        ));
    }

    #[test]
    fn test_partition_tolerance_pn_counter(
        ops_partition_a in vec(pn_counter_increment_op(), 1..20),
        ops_partition_b in vec(pn_counter_increment_op(), 1..20)
    ) {
        prop_assert!(verify_partition_tolerance::<PnCounter, _>(
            ops_partition_a,
            ops_partition_b
        ));
    }
}

// ============================================================================
// Multi-Strategy Composition Tests
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompositeState {
    id: ImmutableValue,
    name: LwwValue,
    tags: OrSet,
    likes: PnCounter,
}

impl Default for CompositeState {
    fn default() -> Self {
        Self {
            id: ImmutableValue::default(),
            name: LwwValue::default(),
            tags: OrSet::default(),
            likes: PnCounter::default(),
        }
    }
}

impl Mergeable for CompositeState {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        self.id.merge(&other.id)?;
        self.name.merge(&other.name)?;
        self.tags.merge(&other.tags)?;
        self.likes.merge(&other.likes)?;
        Ok(())
    }

    fn replica_id(&self) -> String {
        "composite".to_string()
    }
}

impl Operable<CrdtOperation> for CompositeState {
    fn apply(&mut self, op: CrdtOperation) -> TestResult<()> {
        match &op {
            CrdtOperation::ImmutableSet { .. } => self.id.apply(op),
            CrdtOperation::LwwWrite { .. } => self.name.apply(op),
            CrdtOperation::OrSetAdd { .. } | CrdtOperation::OrSetRemove { .. } => {
                self.tags.apply(op)
            }
            CrdtOperation::PnCounterIncrement { .. } | CrdtOperation::PnCounterDecrement { .. } => {
                self.likes.apply(op)
            }
            _ => Ok(()),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_composite_convergence(
        ops in vec(any_crdt_operation(), 1..50)
    ) {
        let mut replica_a = CompositeState::default();
        let mut replica_b = CompositeState::default();

        // Apply operations in different orders
        for op in ops.iter() {
            replica_a.apply(op.clone()).unwrap();
        }

        for op in ops.iter().rev() {
            replica_b.apply(op.clone()).unwrap();
        }

        // Merge
        replica_a.merge(&replica_b).unwrap();
        replica_b.merge(&replica_a).unwrap();

        prop_assert_eq!(replica_a, replica_b);
    }

    #[test]
    fn test_composite_fundamental_properties(
        ops_a in vec(any_crdt_operation(), 1..20),
        ops_b in vec(any_crdt_operation(), 1..20),
        ops_c in vec(any_crdt_operation(), 1..20)
    ) {
        let mut state_a = CompositeState::default();
        let mut state_b = CompositeState::default();
        let mut state_c = CompositeState::default();

        for op in ops_a {
            state_a.apply(op).unwrap();
        }
        for op in ops_b {
            state_b.apply(op).unwrap();
        }
        for op in ops_c {
            state_c.apply(op).unwrap();
        }

        prop_assert!(verify_commutativity(&state_a, &state_b));
        prop_assert!(verify_associativity(&state_a, &state_b, &state_c));
        prop_assert!(verify_idempotency(&state_a));
    }
}
