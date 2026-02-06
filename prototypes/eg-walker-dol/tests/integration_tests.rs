//! Integration tests for eg-walker DOL prototype

use eg_walker_dol::{
    EgWalkerText, AutomergeText, TextCrdt, correctness,
};

#[test]
fn test_egwalker_all_correctness() {
    assert!(correctness::test_convergence::<EgWalkerText>(5, 10).is_ok());
    assert!(correctness::test_commutativity::<EgWalkerText>().is_ok());
    assert!(correctness::test_associativity::<EgWalkerText>().is_ok());
    assert!(correctness::test_idempotency::<EgWalkerText>().is_ok());
    assert!(correctness::test_causality::<EgWalkerText>().is_ok());
}

#[test]
fn test_automerge_all_correctness() {
    assert!(correctness::test_convergence::<AutomergeText>(5, 10).is_ok());
    assert!(correctness::test_commutativity::<AutomergeText>().is_ok());
    assert!(correctness::test_associativity::<AutomergeText>().is_ok());
    assert!(correctness::test_idempotency::<AutomergeText>().is_ok());
    assert!(correctness::test_causality::<AutomergeText>().is_ok());
}

#[test]
fn test_large_document_handling() {
    // Test with 10K character document
    let mut doc = EgWalkerText::new("large".to_string());

    for i in 0..10_000 {
        doc.insert(doc.len(), "x").unwrap();
    }

    assert_eq!(doc.len(), 10_000);

    // Verify serialization works
    let bytes = doc.to_bytes().unwrap();
    let restored = EgWalkerText::from_bytes(&bytes).unwrap();

    assert_eq!(doc.get_text(), restored.get_text());
}

#[test]
fn test_interleaved_operations() {
    let mut alice = EgWalkerText::new("alice".to_string());
    let mut bob = EgWalkerText::new("bob".to_string());

    // Build up a document with interleaved operations
    for i in 0..100 {
        alice.insert(alice.len(), &format!("A{} ", i)).unwrap();

        if i % 10 == 0 {
            bob.merge(&alice).unwrap();
        }

        bob.insert(bob.len(), &format!("B{} ", i)).unwrap();

        if i % 10 == 5 {
            alice.merge(&bob).unwrap();
        }
    }

    // Final merge
    alice.merge(&bob).unwrap();
    bob.merge(&alice).unwrap();

    assert_eq!(alice.get_text(), bob.get_text());
}

#[test]
fn test_delete_all_then_insert() {
    let mut doc = EgWalkerText::new("test".to_string());

    // Insert initial text
    doc.insert(0, "Hello World").unwrap();
    assert_eq!(doc.get_text(), "Hello World");

    // Delete everything
    let len = doc.len();
    doc.delete(0, len).unwrap();
    assert_eq!(doc.get_text(), "");
    assert!(doc.is_empty());

    // Insert new text
    doc.insert(0, "Goodbye").unwrap();
    assert_eq!(doc.get_text(), "Goodbye");
}

#[test]
fn test_unicode_handling() {
    let mut doc = EgWalkerText::new("unicode".to_string());

    // Test various Unicode characters
    doc.insert(0, "Hello 世界").unwrap();
    assert_eq!(doc.get_text(), "Hello 世界");

    doc.insert(6, "🌍").unwrap();
    assert_eq!(doc.get_text(), "Hello 🌍世界");

    // Test emoji with modifiers
    doc.insert(0, "👨‍👩‍👧‍👦 ").unwrap();
    assert!(doc.get_text().starts_with("👨‍👩‍👧‍👦"));
}

#[test]
fn test_concurrent_delete_same_region() {
    let mut alice = EgWalkerText::new("alice".to_string());
    alice.insert(0, "0123456789").unwrap();

    let mut bob = alice.fork();

    // Both try to delete overlapping regions
    alice.delete(2, 3).unwrap(); // Delete "234"
    bob.delete(4, 3).unwrap();   // Delete "456"

    // Merge
    alice.merge(&bob).unwrap();
    bob.merge(&alice).unwrap();

    // Should converge (exact result depends on CRDT semantics)
    assert_eq!(alice.get_text(), bob.get_text());
}

#[test]
fn test_memory_efficiency() {
    let mut small_doc = EgWalkerText::new("small".to_string());
    small_doc.insert(0, "x").unwrap();
    let small_mem = small_doc.memory_size();

    let mut large_doc = EgWalkerText::new("large".to_string());
    for _ in 0..1000 {
        large_doc.insert(large_doc.len(), "x").unwrap();
    }
    let large_mem = large_doc.memory_size();

    // Memory should scale roughly linearly, not quadratically
    // Allow some overhead, but not excessive
    let ratio = large_mem as f64 / (small_mem as f64 * 1000.0);
    assert!(ratio < 5.0, "Memory scaling is excessive: {:.2}x", ratio);
}

#[test]
fn test_fork_independence() {
    let mut original = EgWalkerText::new("original".to_string());
    original.insert(0, "Original").unwrap();

    let mut fork1 = original.fork();
    let mut fork2 = original.fork();

    // Modify forks independently
    fork1.insert(fork1.len(), " Fork1").unwrap();
    fork2.insert(fork2.len(), " Fork2").unwrap();

    // Original should be unchanged
    assert_eq!(original.get_text(), "Original");
    assert_eq!(fork1.get_text(), "Original Fork1");
    assert_eq!(fork2.get_text(), "Original Fork2");
}

#[test]
fn test_serialize_deserialize_preserves_state() {
    let mut doc = EgWalkerText::new("test".to_string());

    // Perform various operations
    doc.insert(0, "Hello").unwrap();
    doc.insert(5, " World").unwrap();
    doc.delete(6, 5).unwrap();
    doc.insert(6, "Rust").unwrap();

    let original_text = doc.get_text();
    let original_ops = doc.operation_count();

    // Serialize and deserialize
    let bytes = doc.to_bytes().unwrap();
    let restored = EgWalkerText::from_bytes(&bytes).unwrap();

    // Verify state is preserved
    assert_eq!(restored.get_text(), original_text);

    // Operation count should be similar (may vary slightly)
    let ops_diff = (restored.operation_count() as i64 - original_ops as i64).abs();
    assert!(ops_diff < 2, "Operation count differs too much");
}

#[test]
fn test_rapid_concurrent_edits() {
    // Simulate rapid concurrent editing
    let mut replicas: Vec<EgWalkerText> = (0..5)
        .map(|i| EgWalkerText::new(format!("user{}", i)))
        .collect();

    // Each replica makes 50 rapid edits
    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..50 {
            let pos = if replica.len() > 0 { replica.len() / 2 } else { 0 };
            replica.insert(pos, &format!("R{}:{} ", idx, i)).unwrap();
        }
    }

    // Merge all replicas together
    for i in 0..replicas.len() {
        for j in 0..replicas.len() {
            if i != j {
                let other = replicas[j].clone();
                replicas[i].merge(&other).unwrap();
            }
        }
    }

    // All should converge
    let first_text = replicas[0].get_text();
    for replica in &replicas[1..] {
        assert_eq!(replica.get_text(), first_text);
    }
}
