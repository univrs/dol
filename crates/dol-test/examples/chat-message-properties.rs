//! Property-based tests for a collaborative chat message
//!
//! This example demonstrates how to use the dol-test framework to verify
//! CRDT convergence properties for a chat message with reactions and likes.
//!
//! Run with: cargo test --example chat-message-properties

use dol_test::generators::*;
use dol_test::properties::*;
use dol_test::TestResult;
use proptest::prelude::*;
use proptest::collection::vec;
use std::collections::{HashMap, HashSet};

/// A collaborative chat message with CRDT semantics
#[derive(Debug, Clone, PartialEq, Eq)]
struct ChatMessage {
    /// Immutable message ID
    id: Option<String>,
    id_timestamp: u64,
    id_actor: String,

    /// Peritext-style content (simplified for example)
    content: String,
    content_timestamp: u64,
    content_actor: String,

    /// OR-Set of reactions
    reactions: HashMap<String, HashSet<String>>, // emoji -> tags
    reaction_tombstones: HashSet<String>,

    /// PN-Counter for likes
    likes_increments: HashMap<String, i64>,
    likes_decrements: HashMap<String, i64>,
}

impl Default for ChatMessage {
    fn default() -> Self {
        Self {
            id: None,
            id_timestamp: 0,
            id_actor: String::new(),
            content: String::new(),
            content_timestamp: 0,
            content_actor: String::new(),
            reactions: HashMap::new(),
            reaction_tombstones: HashSet::new(),
            likes_increments: HashMap::new(),
            likes_decrements: HashMap::new(),
        }
    }
}

impl Mergeable for ChatMessage {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Merge immutable ID (keep earliest)
        match (&self.id, &other.id) {
            (None, Some(_)) => {
                self.id = other.id.clone();
                self.id_timestamp = other.id_timestamp;
                self.id_actor = other.id_actor.clone();
            }
            (Some(_), None) => {
                // Keep self
            }
            (Some(_), Some(_)) => {
                if other.id_timestamp < self.id_timestamp
                    || (other.id_timestamp == self.id_timestamp && other.id_actor < self.id_actor)
                {
                    self.id = other.id.clone();
                    self.id_timestamp = other.id_timestamp;
                    self.id_actor = other.id_actor.clone();
                }
            }
            (None, None) => {}
        }

        // Merge content (LWW - keep latest)
        if other.content_timestamp > self.content_timestamp
            || (other.content_timestamp == self.content_timestamp
                && other.content_actor > self.content_actor)
        {
            self.content = other.content.clone();
            self.content_timestamp = other.content_timestamp;
            self.content_actor = other.content_actor.clone();
        }

        // Merge reactions (OR-Set - union)
        for (emoji, tags) in &other.reactions {
            self.reactions
                .entry(emoji.clone())
                .or_insert_with(HashSet::new)
                .extend(tags.clone());
        }
        self.reaction_tombstones
            .extend(other.reaction_tombstones.clone());

        // Merge likes (PN-Counter - max per actor)
        for (actor, count) in &other.likes_increments {
            let entry = self.likes_increments.entry(actor.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
        for (actor, count) in &other.likes_decrements {
            let entry = self.likes_decrements.entry(actor.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }

        Ok(())
    }

    fn replica_id(&self) -> String {
        "chat-message".to_string()
    }
}

impl Operable<CrdtOperation> for ChatMessage {
    fn apply(&mut self, op: CrdtOperation) -> TestResult<()> {
        match op {
            CrdtOperation::ImmutableSet { value, timestamp, actor } => {
                if self.id.is_none() {
                    self.id = Some(value);
                    self.id_timestamp = timestamp;
                    self.id_actor = actor;
                }
            }
            CrdtOperation::LwwWrite { value, timestamp, actor } => {
                if timestamp > self.content_timestamp
                    || (timestamp == self.content_timestamp && actor > self.content_actor)
                {
                    self.content = value;
                    self.content_timestamp = timestamp;
                    self.content_actor = actor;
                }
            }
            CrdtOperation::OrSetAdd { element, tag } => {
                self.reactions
                    .entry(element)
                    .or_insert_with(HashSet::new)
                    .insert(tag);
            }
            CrdtOperation::OrSetRemove { element, observed_tags } => {
                self.reaction_tombstones.extend(observed_tags.clone());
                if let Some(tags) = self.reactions.get_mut(&element) {
                    tags.retain(|tag| !self.reaction_tombstones.contains(tag));
                }
            }
            CrdtOperation::PnCounterIncrement { actor, amount } => {
                *self.likes_increments.entry(actor).or_insert(0) += amount;
            }
            CrdtOperation::PnCounterDecrement { actor, amount } => {
                *self.likes_decrements.entry(actor).or_insert(0) += amount;
            }
            _ => {}
        }
        Ok(())
    }
}

impl ChatMessage {
    /// Returns the number of likes (PN-Counter value)
    fn likes_count(&self) -> i64 {
        let inc: i64 = self.likes_increments.values().sum();
        let dec: i64 = self.likes_decrements.values().sum();
        inc - dec
    }

    /// Returns the set of active reactions
    fn active_reactions(&self) -> HashSet<String> {
        self.reactions
            .iter()
            .filter(|(_, tags)| !tags.is_empty())
            .map(|(emoji, _)| emoji.clone())
            .collect()
    }
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Test that message ID is immutable (Theorem 2.1)
    #[test]
    fn test_message_id_immutability(
        initial_id in immutable_set_op(),
        subsequent_writes in vec(immutable_set_op(), 1..10)
    ) {
        let mut message = ChatMessage::default();

        // Set initial ID
        message.apply(initial_id.clone()).unwrap();
        let id_after_first = message.id.clone();

        // Try to write more IDs
        for op in subsequent_writes {
            message.apply(op).unwrap();
        }

        // ID should not have changed
        prop_assert_eq!(message.id, id_after_first);
    }

    /// Test content convergence with LWW (Theorem 3.1)
    #[test]
    fn test_content_convergence(
        ops_a in vec(lww_write_op(), 1..20),
        ops_b in vec(lww_write_op(), 1..20)
    ) {
        let mut replica_a = ChatMessage::default();
        let mut replica_b = ChatMessage::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        // Merge both directions
        let mut final_a = replica_a.clone();
        let mut final_b = replica_b.clone();

        final_a.merge(&replica_b).unwrap();
        final_b.merge(&replica_a).unwrap();

        // Should converge
        prop_assert_eq!(final_a.content, final_b.content);
    }

    /// Test reactions add-wins semantics (Theorem 4.1)
    #[test]
    fn test_reactions_add_wins(
        add_ops in vec(or_set_add_op(), 1..20),
        remove_ops in vec(or_set_remove_op(), 1..10)
    ) {
        let mut replica_a = ChatMessage::default();
        let mut replica_b = ChatMessage::default();

        // Partition A: adds reactions
        for op in add_ops.clone() {
            replica_a.apply(op).unwrap();
        }

        // Partition B: tries to remove (without observing adds)
        for op in remove_ops {
            replica_b.apply(op).unwrap();
        }

        // Merge
        replica_a.merge(&replica_b).unwrap();
        replica_b.merge(&replica_a).unwrap();

        // Verify convergence
        prop_assert_eq!(replica_a, replica_b);
    }

    /// Test likes counter convergence (Theorem 5.1)
    #[test]
    fn test_likes_counter_convergence(
        ops in vec(prop_oneof![pn_counter_increment_op(), pn_counter_decrement_op()], 1..50)
    ) {
        let mut replica_a = ChatMessage::default();
        let mut replica_b = ChatMessage::default();

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

        // Likes count should be identical
        prop_assert_eq!(replica_a.likes_count(), replica_b.likes_count());
    }

    /// Test commutativity (Theorem 1)
    #[test]
    fn test_chat_message_commutativity(
        ops_a in vec(any_crdt_operation(), 1..30),
        ops_b in vec(any_crdt_operation(), 1..30)
    ) {
        let mut replica_a = ChatMessage::default();
        let mut replica_b = ChatMessage::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        prop_assert!(verify_commutativity(&replica_a, &replica_b));
    }

    /// Test associativity (Theorem 2)
    #[test]
    fn test_chat_message_associativity(
        ops_a in vec(any_crdt_operation(), 1..20),
        ops_b in vec(any_crdt_operation(), 1..20),
        ops_c in vec(any_crdt_operation(), 1..20)
    ) {
        let mut replica_a = ChatMessage::default();
        let mut replica_b = ChatMessage::default();
        let mut replica_c = ChatMessage::default();

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

    /// Test idempotency (Theorem 3)
    #[test]
    fn test_chat_message_idempotency(
        ops in vec(any_crdt_operation(), 1..30)
    ) {
        let mut message = ChatMessage::default();

        for op in ops {
            message.apply(op).unwrap();
        }

        prop_assert!(verify_idempotency(&message));
    }

    /// Test partition tolerance (Theorem 11.1)
    #[test]
    fn test_chat_message_partition_tolerance(
        ops_partition_a in vec(any_crdt_operation(), 1..30),
        ops_partition_b in vec(any_crdt_operation(), 1..30)
    ) {
        prop_assert!(verify_partition_tolerance::<ChatMessage, _>(
            ops_partition_a,
            ops_partition_b
        ));
    }
}

fn main() {
    println!("Run with: cargo test --example chat-message-properties");
}
