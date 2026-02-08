//! Arbitrary generators for property-based testing
//!
//! This module provides proptest generators for CRDT operations, network topologies,
//! and operation orderings. These generators are used to create random test cases
//! that explore the space of possible CRDT behaviors.
//!
//! # Example
//!
//! ```rust
//! use dol_test::generators::*;
//! use proptest::prelude::*;
//!
//! proptest! {
//!     #[test]
//!     fn test_with_random_ops(ops in vec(any_crdt_operation(), 1..100)) {
//!         // Test with randomly generated operations
//!     }
//! }
//! ```

use proptest::prelude::*;
use std::collections::HashMap;

/// Generates random actor IDs for CRDT operations
pub fn actor_id() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z]{8}").unwrap()
}

/// Generates random timestamps (logical clocks)
pub fn timestamp() -> impl Strategy<Value = u64> {
    any::<u64>()
}

/// Generates random operation IDs
pub fn operation_id() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-f0-9]{16}").unwrap()
}

/// CRDT operation types for testing
#[derive(Debug, Clone, PartialEq)]
pub enum CrdtOperation {
    /// Immutable set operation
    ImmutableSet {
        value: String,
        timestamp: u64,
        actor: String,
    },

    /// LWW write operation
    LwwWrite {
        value: String,
        timestamp: u64,
        actor: String,
    },

    /// OR-Set add operation
    OrSetAdd {
        element: String,
        tag: String,
    },

    /// OR-Set remove operation
    OrSetRemove {
        element: String,
        observed_tags: Vec<String>,
    },

    /// PN-Counter increment
    PnCounterIncrement {
        actor: String,
        amount: i64,
    },

    /// PN-Counter decrement
    PnCounterDecrement {
        actor: String,
        amount: i64,
    },

    /// RGA insert operation
    RgaInsert {
        position: usize,
        element: String,
        vertex_id: String,
        left_origin: Option<String>,
    },

    /// RGA delete operation
    RgaDelete {
        vertex_id: String,
    },

    /// MV-Register write
    MvRegisterWrite {
        value: String,
        vector_clock: HashMap<String, u64>,
    },

    /// Peritext insert character
    PeritextInsert {
        position: usize,
        character: char,
        timestamp: u64,
    },

    /// Peritext delete character
    PeritextDelete {
        position: usize,
    },

    /// Peritext format mark
    PeritextFormat {
        start: usize,
        end: usize,
        format: String,
    },
}

/// Generates random immutable set operations
pub fn immutable_set_op() -> impl Strategy<Value = CrdtOperation> {
    (
        prop::string::string_regex("[a-zA-Z0-9]{1,20}").unwrap(),
        any::<u64>(),
        actor_id(),
    )
        .prop_map(|(value, timestamp, actor)| CrdtOperation::ImmutableSet {
            value,
            timestamp,
            actor,
        })
}

/// Generates random LWW write operations
pub fn lww_write_op() -> impl Strategy<Value = CrdtOperation> {
    (
        prop::string::string_regex("[a-zA-Z0-9 ]{1,50}").unwrap(),
        any::<u64>(),
        actor_id(),
    )
        .prop_map(|(value, timestamp, actor)| CrdtOperation::LwwWrite {
            value,
            timestamp,
            actor,
        })
}

/// Generates random OR-Set add operations
pub fn or_set_add_op() -> impl Strategy<Value = CrdtOperation> {
    (
        prop::string::string_regex("[a-z]{1,10}").unwrap(),
        operation_id(),
    )
        .prop_map(|(element, tag)| CrdtOperation::OrSetAdd { element, tag })
}

/// Generates random OR-Set remove operations
pub fn or_set_remove_op() -> impl Strategy<Value = CrdtOperation> {
    (
        prop::string::string_regex("[a-z]{1,10}").unwrap(),
        prop::collection::vec(operation_id(), 0..5),
    )
        .prop_map(|(element, observed_tags)| CrdtOperation::OrSetRemove {
            element,
            observed_tags,
        })
}

/// Generates random PN-Counter increment operations
pub fn pn_counter_increment_op() -> impl Strategy<Value = CrdtOperation> {
    (actor_id(), 1i64..100)
        .prop_map(|(actor, amount)| CrdtOperation::PnCounterIncrement { actor, amount })
}

/// Generates random PN-Counter decrement operations
pub fn pn_counter_decrement_op() -> impl Strategy<Value = CrdtOperation> {
    (actor_id(), 1i64..100)
        .prop_map(|(actor, amount)| CrdtOperation::PnCounterDecrement { actor, amount })
}

/// Generates random RGA insert operations
pub fn rga_insert_op() -> impl Strategy<Value = CrdtOperation> {
    (
        any::<usize>(),
        prop::string::string_regex("[a-zA-Z0-9]").unwrap(),
        operation_id(),
        prop::option::of(operation_id()),
    )
        .prop_map(|(position, element, vertex_id, left_origin)| CrdtOperation::RgaInsert {
            position,
            element,
            vertex_id,
            left_origin,
        })
}

/// Generates random RGA delete operations
pub fn rga_delete_op() -> impl Strategy<Value = CrdtOperation> {
    operation_id().prop_map(|vertex_id| CrdtOperation::RgaDelete { vertex_id })
}

/// Generates random MV-Register write operations
pub fn mv_register_write_op() -> impl Strategy<Value = CrdtOperation> {
    (
        prop::string::string_regex("[a-zA-Z0-9 ]{1,30}").unwrap(),
        prop::collection::hash_map(actor_id(), any::<u64>(), 1..5),
    )
        .prop_map(|(value, vector_clock)| CrdtOperation::MvRegisterWrite {
            value,
            vector_clock,
        })
}

/// Generates random Peritext insert operations
pub fn peritext_insert_op() -> impl Strategy<Value = CrdtOperation> {
    (any::<usize>(), any::<char>(), any::<u64>())
        .prop_map(|(position, character, timestamp)| CrdtOperation::PeritextInsert {
            position,
            character,
            timestamp,
        })
}

/// Generates random Peritext delete operations
pub fn peritext_delete_op() -> impl Strategy<Value = CrdtOperation> {
    any::<usize>().prop_map(|position| CrdtOperation::PeritextDelete { position })
}

/// Generates random Peritext format operations
pub fn peritext_format_op() -> impl Strategy<Value = CrdtOperation> {
    (
        any::<usize>(),
        any::<usize>(),
        prop::sample::select(vec!["bold", "italic", "underline"]),
    )
        .prop_map(|(start, end, format)| CrdtOperation::PeritextFormat {
            start,
            end,
            format: format.to_string(),
        })
}

/// Generates random CRDT operations (any strategy)
pub fn any_crdt_operation() -> impl Strategy<Value = CrdtOperation> {
    prop_oneof![
        immutable_set_op(),
        lww_write_op(),
        or_set_add_op(),
        or_set_remove_op(),
        pn_counter_increment_op(),
        pn_counter_decrement_op(),
        rga_insert_op(),
        rga_delete_op(),
        mv_register_write_op(),
        peritext_insert_op(),
        peritext_delete_op(),
        peritext_format_op(),
    ]
}

/// Network topology for partition testing
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkTopology {
    /// Number of replicas
    pub num_replicas: usize,

    /// Partition configuration: replica IDs in each partition
    pub partitions: Vec<Vec<usize>>,

    /// Message delay matrix (milliseconds)
    pub delays: Vec<Vec<u64>>,
}

/// Generates random network topologies
pub fn network_topology(num_replicas: usize) -> impl Strategy<Value = NetworkTopology> {
    let delays = prop::collection::vec(
        prop::collection::vec(0u64..1000, num_replicas),
        num_replicas,
    );

    // Simple partition: split replicas into two groups
    let partitions = Just(vec![
        (0..num_replicas / 2).collect(),
        (num_replicas / 2..num_replicas).collect(),
    ]);

    (delays, partitions).prop_map(move |(delays, partitions)| NetworkTopology {
        num_replicas,
        partitions,
        delays,
    })
}

/// Operation ordering strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingStrategy {
    /// Sequential: operations applied in order
    Sequential,

    /// Concurrent: operations applied in arbitrary order
    Concurrent,

    /// Causal: operations respect happens-before relationships
    Causal,

    /// Reversed: operations applied in reverse order
    Reversed,
}

/// Generates random ordering strategies
pub fn ordering_strategy() -> impl Strategy<Value = OrderingStrategy> {
    prop_oneof![
        Just(OrderingStrategy::Sequential),
        Just(OrderingStrategy::Concurrent),
        Just(OrderingStrategy::Causal),
        Just(OrderingStrategy::Reversed),
    ]
}

/// Test scenario combining operations, topology, and ordering
#[derive(Debug, Clone)]
pub struct TestScenario {
    /// Operations to execute
    pub operations: Vec<CrdtOperation>,

    /// Network topology
    pub topology: NetworkTopology,

    /// Ordering strategy
    pub ordering: OrderingStrategy,

    /// Random seed for reproducibility
    pub seed: u64,
}

/// Generates comprehensive test scenarios
pub fn test_scenario(
    num_operations: usize,
    num_replicas: usize,
) -> impl Strategy<Value = TestScenario> {
    (
        prop::collection::vec(any_crdt_operation(), num_operations),
        network_topology(num_replicas),
        ordering_strategy(),
        any::<u64>(),
    )
        .prop_map(|(operations, topology, ordering, seed)| TestScenario {
            operations,
            topology,
            ordering,
            seed,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;

    #[test]
    fn test_actor_id_generation() {
        let strategy = actor_id();
        let mut runner = proptest::test_runner::TestRunner::default();
        let mut value_tree = strategy.new_tree(&mut runner).unwrap();
        let actor = value_tree.current();
        assert_eq!(actor.len(), 8);
        assert!(actor.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_operation_id_generation() {
        let strategy = operation_id();
        let mut runner = proptest::test_runner::TestRunner::default();
        let mut value_tree = strategy.new_tree(&mut runner).unwrap();
        let op_id = value_tree.current();
        assert_eq!(op_id.len(), 16);
        assert!(op_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_immutable_set_op_generation() {
        let strategy = immutable_set_op();
        let mut runner = proptest::test_runner::TestRunner::default();
        let mut value_tree = strategy.new_tree(&mut runner).unwrap();
        let op = value_tree.current();

        match op {
            CrdtOperation::ImmutableSet { value, timestamp, actor } => {
                assert!(!value.is_empty());
                assert!(timestamp > 0 || timestamp == 0);
                assert_eq!(actor.len(), 8);
            }
            _ => panic!("Expected ImmutableSet operation"),
        }
    }

    #[test]
    fn test_network_topology_generation() {
        let strategy = network_topology(4);
        let mut runner = proptest::test_runner::TestRunner::default();
        let mut value_tree = strategy.new_tree(&mut runner).unwrap();
        let topology = value_tree.current();

        assert_eq!(topology.num_replicas, 4);
        assert_eq!(topology.delays.len(), 4);
        assert_eq!(topology.delays[0].len(), 4);
        assert_eq!(topology.partitions.len(), 2);
    }

    #[test]
    fn test_ordering_strategy_generation() {
        let strategy = ordering_strategy();
        let mut runner = proptest::test_runner::TestRunner::default();
        let mut value_tree = strategy.new_tree(&mut runner).unwrap();
        let ordering = value_tree.current();

        assert!(matches!(
            ordering,
            OrderingStrategy::Sequential
                | OrderingStrategy::Concurrent
                | OrderingStrategy::Causal
                | OrderingStrategy::Reversed
        ));
    }
}
