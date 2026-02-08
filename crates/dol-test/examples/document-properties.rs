//! Property-based tests for a collaborative document
//!
//! This example demonstrates CRDT properties for a document with:
//! - Immutable document ID
//! - RGA-based paragraph list
//! - OR-Set tags
//! - LWW metadata
//!
//! Run with: cargo test --example document-properties

use dol_test::generators::*;
use dol_test::properties::*;
use dol_test::TestResult;
use proptest::prelude::*;
use proptest::collection::vec;
use std::collections::{HashMap, HashSet};

/// A vertex in the RGA (Replicated Growable Array)
#[derive(Debug, Clone, PartialEq, Eq)]
struct RgaVertex {
    id: String,
    content: String,
    left_origin: Option<String>,
    timestamp: u64,
}

/// A collaborative document with CRDT semantics
#[derive(Debug, Clone, PartialEq, Eq)]
struct Document {
    /// Immutable document ID
    id: Option<String>,
    id_timestamp: u64,

    /// RGA for paragraphs
    paragraphs: Vec<RgaVertex>,
    paragraph_tombstones: HashSet<String>,

    /// OR-Set for tags
    tags: HashMap<String, HashSet<String>>, // tag -> unique IDs
    tag_tombstones: HashSet<String>,

    /// LWW for metadata
    metadata: HashMap<String, (String, u64)>, // key -> (value, timestamp)
}

impl Default for Document {
    fn default() -> Self {
        Self {
            id: None,
            id_timestamp: 0,
            paragraphs: Vec::new(),
            paragraph_tombstones: HashSet::new(),
            tags: HashMap::new(),
            tag_tombstones: HashSet::new(),
            metadata: HashMap::new(),
        }
    }
}

impl Mergeable for Document {
    fn merge(&mut self, other: &Self) -> TestResult<()> {
        // Merge immutable ID (keep earliest)
        match (&self.id, &other.id) {
            (None, Some(_)) => {
                self.id = other.id.clone();
                self.id_timestamp = other.id_timestamp;
            }
            (Some(_), None) => {}
            (Some(_), Some(_)) => {
                if other.id_timestamp < self.id_timestamp {
                    self.id = other.id.clone();
                    self.id_timestamp = other.id_timestamp;
                }
            }
            (None, None) => {}
        }

        // Merge RGA paragraphs
        for vertex in &other.paragraphs {
            if !self.paragraphs.iter().any(|v| v.id == vertex.id) {
                self.paragraphs.push(vertex.clone());
            }
        }
        self.paragraph_tombstones
            .extend(other.paragraph_tombstones.clone());

        // Sort paragraphs by timestamp (simplified RGA ordering)
        self.paragraphs.sort_by(|a, b| {
            a.timestamp
                .cmp(&b.timestamp)
                .then_with(|| a.id.cmp(&b.id))
        });

        // Merge tags (OR-Set)
        for (tag, ids) in &other.tags {
            self.tags
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .extend(ids.clone());
        }
        self.tag_tombstones.extend(other.tag_tombstones.clone());

        // Merge metadata (LWW)
        for (key, (value, timestamp)) in &other.metadata {
            self.metadata
                .entry(key.clone())
                .and_modify(|(v, ts)| {
                    if timestamp > ts {
                        *v = value.clone();
                        *ts = *timestamp;
                    }
                })
                .or_insert_with(|| (value.clone(), *timestamp));
        }

        Ok(())
    }

    fn replica_id(&self) -> String {
        "document".to_string()
    }
}

impl Operable<CrdtOperation> for Document {
    fn apply(&mut self, op: CrdtOperation) -> TestResult<()> {
        match op {
            CrdtOperation::ImmutableSet { value, timestamp, .. } => {
                if self.id.is_none() {
                    self.id = Some(value);
                    self.id_timestamp = timestamp;
                }
            }
            CrdtOperation::RgaInsert {
                element,
                vertex_id,
                left_origin,
                ..
            } => {
                // Simplified RGA insert
                let vertex = RgaVertex {
                    id: vertex_id.clone(),
                    content: element,
                    left_origin,
                    timestamp: self.paragraphs.len() as u64,
                };
                self.paragraphs.push(vertex);
            }
            CrdtOperation::RgaDelete { vertex_id } => {
                self.paragraph_tombstones.insert(vertex_id);
            }
            CrdtOperation::OrSetAdd { element, tag } => {
                self.tags
                    .entry(element)
                    .or_insert_with(HashSet::new)
                    .insert(tag);
            }
            CrdtOperation::OrSetRemove { element, observed_tags } => {
                self.tag_tombstones.extend(observed_tags.clone());
                if let Some(ids) = self.tags.get_mut(&element) {
                    ids.retain(|id| !self.tag_tombstones.contains(id));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Document {
    /// Returns visible paragraphs (not tombstoned)
    fn visible_paragraphs(&self) -> Vec<String> {
        self.paragraphs
            .iter()
            .filter(|v| !self.paragraph_tombstones.contains(&v.id))
            .map(|v| v.content.clone())
            .collect()
    }

    /// Returns active tags (non-empty sets)
    fn active_tags(&self) -> HashSet<String> {
        self.tags
            .iter()
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(tag, _)| tag.clone())
            .collect()
    }
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Test document ID immutability (Theorem 2.1)
    #[test]
    fn test_document_id_immutability(
        first_id in immutable_set_op(),
        other_ids in vec(immutable_set_op(), 1..10)
    ) {
        let mut doc = Document::default();

        doc.apply(first_id.clone()).unwrap();
        let initial_id = doc.id.clone();

        for op in other_ids {
            doc.apply(op).unwrap();
        }

        // ID should remain unchanged
        prop_assert_eq!(doc.id, initial_id);
    }

    /// Test RGA convergence (Theorem 6.1)
    #[test]
    fn test_rga_convergence(
        ops_a in vec(rga_insert_op(), 1..20),
        ops_b in vec(rga_insert_op(), 1..20)
    ) {
        let mut replica_a = Document::default();
        let mut replica_b = Document::default();

        for op in ops_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_b {
            replica_b.apply(op).unwrap();
        }

        // Merge
        replica_a.merge(&replica_b).unwrap();
        replica_b.merge(&replica_a).unwrap();

        // Should converge to same state
        prop_assert_eq!(replica_a, replica_b);
    }

    /// Test tag OR-Set properties (Theorem 4.1)
    #[test]
    fn test_tags_or_set_convergence(
        ops in vec(prop_oneof![or_set_add_op(), or_set_remove_op()], 1..30)
    ) {
        let mut replica_a = Document::default();
        let mut replica_b = Document::default();

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

        // Tags should converge
        prop_assert_eq!(replica_a.active_tags(), replica_b.active_tags());
    }

    /// Test commutativity (Theorem 1)
    #[test]
    fn test_document_commutativity(
        ops_a in vec(prop_oneof![
            immutable_set_op(),
            rga_insert_op(),
            rga_delete_op(),
            or_set_add_op(),
            or_set_remove_op()
        ], 1..20),
        ops_b in vec(prop_oneof![
            immutable_set_op(),
            rga_insert_op(),
            rga_delete_op(),
            or_set_add_op(),
            or_set_remove_op()
        ], 1..20)
    ) {
        let mut replica_a = Document::default();
        let mut replica_b = Document::default();

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
    fn test_document_associativity(
        ops_a in vec(prop_oneof![rga_insert_op(), or_set_add_op()], 1..15),
        ops_b in vec(prop_oneof![rga_insert_op(), or_set_add_op()], 1..15),
        ops_c in vec(prop_oneof![rga_insert_op(), or_set_add_op()], 1..15)
    ) {
        let mut replica_a = Document::default();
        let mut replica_b = Document::default();
        let mut replica_c = Document::default();

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
    fn test_document_idempotency(
        ops in vec(prop_oneof![
            immutable_set_op(),
            rga_insert_op(),
            or_set_add_op()
        ], 1..20)
    ) {
        let mut doc = Document::default();

        for op in ops {
            doc.apply(op).unwrap();
        }

        prop_assert!(verify_idempotency(&doc));
    }

    /// Test partition tolerance (Theorem 11.1)
    #[test]
    fn test_document_partition_tolerance(
        ops_partition_a in vec(rga_insert_op(), 1..20),
        ops_partition_b in vec(rga_insert_op(), 1..20)
    ) {
        prop_assert!(verify_partition_tolerance::<Document, _>(
            ops_partition_a,
            ops_partition_b
        ));
    }

    /// Test concurrent edits convergence
    #[test]
    fn test_concurrent_edits(
        ops_replica_a in vec(rga_insert_op(), 5..15),
        ops_replica_b in vec(rga_insert_op(), 5..15),
        ops_replica_c in vec(rga_insert_op(), 5..15)
    ) {
        let mut replica_a = Document::default();
        let mut replica_b = Document::default();
        let mut replica_c = Document::default();

        // Simulate concurrent edits
        for op in ops_replica_a {
            replica_a.apply(op).unwrap();
        }
        for op in ops_replica_b {
            replica_b.apply(op).unwrap();
        }
        for op in ops_replica_c {
            replica_c.apply(op).unwrap();
        }

        // Merge all together
        replica_a.merge(&replica_b).unwrap();
        replica_a.merge(&replica_c).unwrap();

        replica_b.merge(&replica_a).unwrap();
        replica_b.merge(&replica_c).unwrap();

        replica_c.merge(&replica_a).unwrap();
        replica_c.merge(&replica_b).unwrap();

        // All should converge
        prop_assert_eq!(replica_a, replica_b);
        prop_assert_eq!(replica_b, replica_c);
    }
}

fn main() {
    println!("Run with: cargo test --example document-properties");
}
