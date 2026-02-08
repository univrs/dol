//! Correctness verification for text CRDT implementations
//!
//! This module tests that implementations satisfy the core CRDT properties:
//! - Convergence: All replicas converge to the same state after merging
//! - Commutativity: Merge order doesn't matter
//! - Associativity: Grouping of merges doesn't matter
//! - Idempotency: Merging multiple times has the same effect as once
//! - Causality: Causal relationships are preserved

use crate::{TextCrdt, Result};

/// Test that concurrent edits converge to the same state
pub fn test_convergence<T: TextCrdt>(
    num_replicas: usize,
    operations_per_replica: usize,
) -> Result<bool> {
    // Create replicas
    let mut replicas: Vec<T> = (0..num_replicas)
        .map(|i| T::new(format!("replica-{}", i)))
        .collect();

    // Perform concurrent operations
    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..operations_per_replica {
            let text = format!("R{}-{} ", idx, i);
            replica.insert(replica.len(), &text)?;
        }
    }

    // Merge all replicas with each other
    for i in 0..num_replicas {
        for j in 0..num_replicas {
            if i != j {
                let other = replicas[j].clone();
                replicas[i].merge(&other)?;
            }
        }
    }

    // Check convergence: all replicas should have the same text
    let first_text = replicas[0].get_text();
    for replica in &replicas[1..] {
        if replica.get_text() != first_text {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Test that merge is commutative (A ∪ B = B ∪ A)
pub fn test_commutativity<T: TextCrdt>() -> Result<bool> {
    // Create two replicas
    let mut alice1 = T::new("alice".to_string());
    let mut bob1 = T::new("bob".to_string());

    alice1.insert(0, "Hello")?;
    bob1.insert(0, "World")?;

    // Clone for different merge orders
    let alice2 = alice1.clone();
    let mut bob2 = bob1.clone();

    // Merge in different orders
    alice1.merge(&bob1)?; // A ∪ B
    bob2.merge(&alice2)?; // B ∪ A

    Ok(alice1.get_text() == bob2.get_text())
}

/// Test that merge is associative ((A ∪ B) ∪ C = A ∪ (B ∪ C))
pub fn test_associativity<T: TextCrdt>() -> Result<bool> {
    // Create three replicas
    let mut alice = T::new("alice".to_string());
    let mut bob = T::new("bob".to_string());
    let mut carol = T::new("carol".to_string());

    alice.insert(0, "A")?;
    bob.insert(0, "B")?;
    carol.insert(0, "C")?;

    // Test (A ∪ B) ∪ C
    let mut left = alice.clone();
    left.merge(&bob)?;
    left.merge(&carol)?;

    // Test A ∪ (B ∪ C)
    let mut right = alice.clone();
    let mut bc = bob.clone();
    bc.merge(&carol)?;
    right.merge(&bc)?;

    Ok(left.get_text() == right.get_text())
}

/// Test that merge is idempotent (A ∪ A = A)
pub fn test_idempotency<T: TextCrdt>() -> Result<bool> {
    let mut doc1 = T::new("test".to_string());
    doc1.insert(0, "Hello World")?;

    let text_before = doc1.get_text();
    let doc2 = doc1.clone();

    // Merge with itself
    doc1.merge(&doc2)?;

    Ok(doc1.get_text() == text_before)
}

/// Test causality preservation
pub fn test_causality<T: TextCrdt>() -> Result<bool> {
    // Create a chain of causally related operations
    let mut alice = T::new("alice".to_string());
    alice.insert(0, "A")?;

    let mut bob = alice.fork();
    bob.insert(1, "B")?; // Causally after A

    let mut carol = bob.fork();
    carol.insert(2, "C")?; // Causally after A and B

    // Merge in causal order
    alice.merge(&bob)?;
    alice.merge(&carol)?;

    let text = alice.get_text();

    // Text should have A, then B, then C in that order
    let a_pos = text.find('A');
    let b_pos = text.find('B');
    let c_pos = text.find('C');

    match (a_pos, b_pos, c_pos) {
        (Some(a), Some(b), Some(c)) => Ok(a < b && b < c),
        _ => Ok(false),
    }
}

/// Run all correctness tests
pub fn run_all_tests<T: TextCrdt>(name: &str) -> Result<()> {
    println!("Running correctness tests for {}", name);

    print!("  Convergence test... ");
    let convergence = test_convergence::<T>(5, 10)?;
    println!("{}", if convergence { "PASS" } else { "FAIL" });

    print!("  Commutativity test... ");
    let commutativity = test_commutativity::<T>()?;
    println!("{}", if commutativity { "PASS" } else { "FAIL" });

    print!("  Associativity test... ");
    let associativity = test_associativity::<T>()?;
    println!("{}", if associativity { "PASS" } else { "FAIL" });

    print!("  Idempotency test... ");
    let idempotency = test_idempotency::<T>()?;
    println!("{}", if idempotency { "PASS" } else { "FAIL" });

    print!("  Causality test... ");
    let causality = test_causality::<T>()?;
    println!("{}", if causality { "PASS" } else { "FAIL" });

    let all_pass = convergence && commutativity && associativity && idempotency && causality;
    println!("\n{} correctness: {}", name, if all_pass { "✓ PASS" } else { "✗ FAIL" });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EgWalkerText, AutomergeText};

    #[test]
    fn test_egwalker_convergence() {
        assert!(test_convergence::<EgWalkerText>(3, 5).unwrap());
    }

    #[test]
    fn test_egwalker_commutativity() {
        assert!(test_commutativity::<EgWalkerText>().unwrap());
    }

    #[test]
    fn test_egwalker_associativity() {
        assert!(test_associativity::<EgWalkerText>().unwrap());
    }

    #[test]
    fn test_egwalker_idempotency() {
        assert!(test_idempotency::<EgWalkerText>().unwrap());
    }

    #[test]
    fn test_automerge_convergence() {
        assert!(test_convergence::<AutomergeText>(3, 5).unwrap());
    }

    #[test]
    fn test_automerge_commutativity() {
        assert!(test_commutativity::<AutomergeText>().unwrap());
    }

    #[test]
    fn test_automerge_associativity() {
        assert!(test_associativity::<AutomergeText>().unwrap());
    }

    #[test]
    fn test_automerge_idempotency() {
        assert!(test_idempotency::<AutomergeText>().unwrap());
    }
}
