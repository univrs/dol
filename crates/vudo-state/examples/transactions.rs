//! Multi-document transaction example demonstrating atomic operations.

use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};
use vudo_state::*;

fn get_i64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<i64> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::Int(val) = s.as_ref() {
                Ok(*val)
            } else {
                panic!("Expected int value")
            }
        }
        _ => panic!("Value not found"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Multi-Document Transaction Demo ===\n");

    // Initialize state engine
    let engine = StateEngine::new().await?;

    // Create bank accounts
    println!("1. Setting up bank accounts...");
    let alice_id = DocumentId::new("accounts", "alice");
    let bob_id = DocumentId::new("accounts", "bob");
    let charlie_id = DocumentId::new("accounts", "charlie");

    let alice = engine.create_document(alice_id.clone()).await?;
    let bob = engine.create_document(bob_id.clone()).await?;
    let charlie = engine.create_document(charlie_id.clone()).await?;

    // Set initial balances
    alice.update(|doc| {
        doc.put(ROOT, "name", "Alice")?;
        doc.put(ROOT, "balance", 1000i64)?;
        Ok(())
    })?;

    bob.update(|doc| {
        doc.put(ROOT, "name", "Bob")?;
        doc.put(ROOT, "balance", 500i64)?;
        Ok(())
    })?;

    charlie.update(|doc| {
        doc.put(ROOT, "name", "Charlie")?;
        doc.put(ROOT, "balance", 250i64)?;
        Ok(())
    })?;

    println!("\nInitial balances:");
    print_balance(&alice, "Alice")?;
    print_balance(&bob, "Bob")?;
    print_balance(&charlie, "Charlie")?;

    // Successful transaction: Alice sends $100 to Bob
    println!("\n2. Transaction 1: Alice sends $100 to Bob");
    let tx1 = engine.begin_transaction();

    tx1.update(&alice_id, |doc| {
        doc.put(ROOT, "balance", 900i64)?; // 1000 - 100
        Ok(())
    })?;

    tx1.update(&bob_id, |doc| {
        doc.put(ROOT, "balance", 600i64)?; // 500 + 100
        Ok(())
    })?;

    println!("  Committing transaction...");
    engine.commit_transaction(tx1)?;
    println!("  ✓ Transaction committed successfully");

    println!("\nBalances after transaction 1:");
    print_balance(&alice, "Alice")?;
    print_balance(&bob, "Bob")?;

    // Failed transaction: Attempt a transfer that will be rolled back
    println!("\n3. Transaction 2: Bob sends $200 to Charlie (will be rolled back)");
    let tx2 = engine.begin_transaction();

    tx2.update(&bob_id, |doc| {
        doc.put(ROOT, "balance", 400i64)?; // 600 - 200
        Ok(())
    })?;

    tx2.update(&charlie_id, |doc| {
        doc.put(ROOT, "balance", 450i64)?; // 250 + 200
        Ok(())
    })?;

    println!("  Rolling back transaction...");
    engine.rollback_transaction(tx2)?;
    println!("  ✓ Transaction rolled back successfully");

    println!("\nBalances after rollback (should be unchanged):");
    print_balance(&bob, "Bob")?;
    print_balance(&charlie, "Charlie")?;

    // Multi-party transaction
    println!("\n4. Transaction 3: Multi-party settlement");
    println!("  Alice: -$50");
    println!("  Bob: -$50");
    println!("  Charlie: +$100");

    let tx3 = engine.begin_transaction();

    tx3.update(&alice_id, |doc| {
        doc.put(ROOT, "balance", 850i64)?; // 900 - 50
        Ok(())
    })?;

    tx3.update(&bob_id, |doc| {
        doc.put(ROOT, "balance", 550i64)?; // 600 - 50
        Ok(())
    })?;

    tx3.update(&charlie_id, |doc| {
        doc.put(ROOT, "balance", 350i64)?; // 250 + 100
        Ok(())
    })?;

    println!("  Committing transaction...");
    engine.commit_transaction(tx3)?;
    println!("  ✓ Transaction committed successfully");

    println!("\nFinal balances:");
    print_balance(&alice, "Alice")?;
    print_balance(&bob, "Bob")?;
    print_balance(&charlie, "Charlie")?;

    // Verify total balance is conserved
    println!("\n5. Verification:");
    let total = get_balance(&alice)? + get_balance(&bob)? + get_balance(&charlie)?;
    println!("  Total balance across all accounts: ${}", total);
    println!("  Expected total: $1750");
    println!("  Balance conserved: {}", total == 1750);

    // Show transaction manager stats
    println!("\n6. Transaction manager stats:");
    println!(
        "  Active transactions: {}",
        engine.transaction_manager.active_count()
    );

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn print_balance(handle: &DocumentHandle, name: &str) -> Result<()> {
    handle.read(|doc| {
        let balance = get_i64(doc, ROOT, "balance")?;
        println!("  {}: ${}", name, balance);
        Ok(())
    })
}

fn get_balance(handle: &DocumentHandle) -> Result<i64> {
    handle.read(|doc| {
        let balance = get_i64(doc, ROOT, "balance")?;
        Ok(balance)
    })
}
