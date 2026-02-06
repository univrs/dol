//! Example demonstrating concurrent editing with eg-walker

use eg_walker_dol::{EgWalkerText, TextCrdt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Concurrent Editing Example ===\n");

    // Alice creates a document
    let mut alice = EgWalkerText::new("alice".to_string());
    alice.insert(0, "Hello")?;
    println!("Alice creates: {}", alice.get_text());

    // Bob forks the document (simulating sync)
    let mut bob = alice.fork();
    println!("Bob syncs, gets: {}", bob.get_text());

    // Both edit concurrently (offline)
    println!("\n--- Offline editing ---");

    alice.insert(5, " Alice")?;
    println!("Alice adds her name: {}", alice.get_text());

    bob.insert(5, " Bob")?;
    println!("Bob adds his name: {}", bob.get_text());

    // They both merge (coming back online)
    println!("\n--- Coming back online ---");

    alice.merge(&bob)?;
    bob.merge(&alice)?;

    println!("Alice after merge: {}", alice.get_text());
    println!("Bob after merge: {}", bob.get_text());

    // Verify convergence
    assert_eq!(alice.get_text(), bob.get_text());
    println!("\n✓ Documents converged!");

    // More complex scenario: three-way editing
    println!("\n=== Three-Way Concurrent Edits ===\n");

    let mut doc1 = EgWalkerText::new("user1".to_string());
    doc1.insert(0, "The quick brown fox")?;
    println!("Initial: {}", doc1.get_text());

    let mut doc2 = doc1.fork();
    let mut doc3 = doc1.fork();

    // Concurrent edits
    doc1.insert(4, "very ")?;
    println!("User1: {}", doc1.get_text());

    doc2.insert(19, " jumps")?;
    println!("User2: {}", doc2.get_text());

    doc3.delete(10, 6)?; // Delete "brown "
    println!("User3: {}", doc3.get_text());

    // Merge all
    doc1.merge(&doc2)?;
    doc1.merge(&doc3)?;

    doc2.merge(&doc1)?;
    doc3.merge(&doc1)?;

    println!("\nFinal (all): {}", doc1.get_text());

    // Verify all converged
    assert_eq!(doc1.get_text(), doc2.get_text());
    assert_eq!(doc2.get_text(), doc3.get_text());
    println!("✓ All three replicas converged!");

    Ok(())
}
