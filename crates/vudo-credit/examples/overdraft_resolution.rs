//! Overdraft resolution example
//!
//! Demonstrates overdraft detection and resolution strategies.
//!
//! Run with: `cargo run --example overdraft_resolution`

use std::sync::Arc;
use vudo_credit::{
    BftCommittee, CreditAccountHandle, DeviceEscrow, MutualCreditScheduler, OverdraftResolution,
    OverdraftResolver, ReputationTier, Transaction, TransactionMetadata,
};
use vudo_state::StateEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Overdraft Resolution Example ===\n");

    // 1. Setup
    println!("1. Setting up system...");
    let state_engine = Arc::new(StateEngine::new().await?);
    let bft_committee = Arc::new(BftCommittee::new(vec![
        "member1".to_string(),
        "member2".to_string(),
        "member3".to_string(),
        "member4".to_string(),
    ])?);

    let scheduler = MutualCreditScheduler::new(
        Arc::clone(&state_engine),
        Arc::clone(&bft_committee),
        "device-main".to_string(),
    )
    .await?;

    // 2. Scenario: Two devices spend concurrently while offline
    println!("\n2. Scenario: Concurrent spending from two devices");
    println!("   Alice has $50.00 confirmed balance");
    println!("   Device 1 (phone) allocated $30.00 escrow");
    println!("   Device 2 (laptop) allocated $30.00 escrow");
    println!("   Both devices go offline and spend independently\n");

    let balance = 5_000; // $50.00
    let alice_account =
        CreditAccountHandle::create(&state_engine, "did:peer:alice".to_string(), balance)
            .await?;

    // Simulate transactions from both devices
    alice_account.update(|acc| {
        // Device 1 spends $25.00
        acc.add_transaction(Transaction::new(
            "did:peer:alice".to_string(),
            "did:peer:merchant1".to_string(),
            2_500,
            TransactionMetadata {
                description: "Device 1: Electronics".to_string(),
                category: Some("shopping".to_string()),
                invoice_id: Some("INV-001".to_string()),
            },
        ));

        // Device 2 spends $35.00
        acc.add_transaction(Transaction::new(
            "did:peer:alice".to_string(),
            "did:peer:merchant2".to_string(),
            3_500,
            TransactionMetadata {
                description: "Device 2: Furniture".to_string(),
                category: Some("shopping".to_string()),
                invoice_id: Some("INV-002".to_string()),
            },
        ));

        Ok(())
    })?;

    // 3. Detect overdrafts
    println!("3. Detecting overdrafts after devices sync...");
    let overdrafts = scheduler.detect_overdrafts("did:peer:alice").await?;

    println!("   Total pending debits: ${:.2}", 60.0);
    println!("   Confirmed balance: ${:.2}", balance as f64 / 100.0);
    println!("   Overdraft detected: ${:.2}\n", 10.0);

    for (i, overdraft) in overdrafts.iter().enumerate() {
        println!("   Overdraft #{}:", i + 1);
        println!("     Transaction: {}", overdraft.transaction_id);
        println!("     Amount: ${:.2}", overdraft.amount as f64 / 100.0);
        println!("     Deficit: ${:.2}", overdraft.deficit as f64 / 100.0);
    }

    // 4. Analyze and suggest resolutions
    println!("\n4. Analyzing resolution strategies...");
    for (i, overdraft) in overdrafts.iter().enumerate() {
        println!("\n   Overdraft #{}:", i + 1);

        let resolution = OverdraftResolver::suggest_resolution(overdraft, balance);

        match &resolution {
            OverdraftResolution::Reverse => {
                println!("     Suggested: REVERSE");
                println!("     Reason: Large overdraft (>50% of balance)");
                println!("     Action: Reverse transaction, refund sender");
            }
            OverdraftResolution::Approve => {
                println!("     Suggested: APPROVE");
                println!("     Reason: Small overdraft (<10% of balance)");
                println!("     Action: Extend credit, requires BFT vote");
            }
            OverdraftResolution::Split {
                sender_pays,
                receiver_pays,
            } => {
                println!("     Suggested: SPLIT");
                println!("     Reason: Medium overdraft (10-50% of balance)");
                println!(
                    "     Action: Sender pays ${:.2}, Receiver pays ${:.2}",
                    *sender_pays as f64 / 100.0,
                    *receiver_pays as f64 / 100.0
                );
            }
            OverdraftResolution::Defer => {
                println!("     Suggested: DEFER");
                println!("     Action: Mark as disputed, requires manual review");
            }
        }

        // Validate resolution
        let validation = OverdraftResolver::validate_resolution(overdraft, &resolution);
        println!(
            "     Validation: {}",
            if validation.is_ok() { "PASS" } else { "FAIL" }
        );
    }

    // 5. Apply resolution: Split the cost
    println!("\n5. Applying resolution: Split strategy");
    let overdraft = &overdrafts[0];
    let resolution = OverdraftResolution::Split {
        sender_pays: 500,  // $5.00
        receiver_pays: 500, // $5.00
    };

    scheduler
        .resolve_overdraft("did:peer:alice", overdraft, resolution)
        .await?;

    println!("   ✓ Resolution applied");
    println!("   Alice pays: $5.00");
    println!("   Merchant refunds: $5.00");

    // 6. Demonstrate reversal resolution
    println!("\n6. Alternative resolution: Reverse transaction");
    let large_overdraft_account =
        CreditAccountHandle::create(&state_engine, "did:peer:bob".to_string(), 1_000)
            .await?;

    large_overdraft_account.update(|acc| {
        acc.add_transaction(Transaction::new(
            "did:peer:bob".to_string(),
            "did:peer:merchant3".to_string(),
            2_000, // Double the balance
            TransactionMetadata {
                description: "Large purchase".to_string(),
                category: Some("shopping".to_string()),
                invoice_id: None,
            },
        ));
        Ok(())
    })?;

    let bob_overdrafts = scheduler.detect_overdrafts("did:peer:bob").await?;
    let bob_overdraft = &bob_overdrafts[0];

    println!("   Bob's balance: $10.00");
    println!("   Transaction amount: $20.00");
    println!("   Overdraft: $10.00 (100% of balance)");

    let resolution = OverdraftResolver::suggest_resolution(bob_overdraft, 1_000);
    match resolution {
        OverdraftResolution::Reverse => {
            println!("   ✓ Suggested: REVERSE (overdraft too large)");
        }
        _ => {}
    }

    scheduler
        .resolve_overdraft("did:peer:bob", bob_overdraft, OverdraftResolution::Reverse)
        .await?;

    large_overdraft_account.read(|acc| {
        let reversed_tx = acc.transactions.iter().find(|tx| tx.is_reversed());
        if let Some(tx) = reversed_tx {
            println!("   ✓ Transaction {} reversed", tx.id);
        }
        Ok(())
    })?;

    // 7. Summary
    println!("\n7. Resolution strategies summary:");
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │ Strategy  │ Overdraft Range │ Action            │");
    println!("   ├─────────────────────────────────────────────────┤");
    println!("   │ APPROVE   │ < 10%           │ Extend credit     │");
    println!("   │ SPLIT     │ 10-50%          │ Share the cost    │");
    println!("   │ REVERSE   │ > 50%           │ Cancel transaction│");
    println!("   │ DEFER     │ Any             │ Manual review     │");
    println!("   └─────────────────────────────────────────────────┘");

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("- Overdrafts detected via CRDT merge comparison");
    println!("- Resolution strategy depends on severity");
    println!("- Flexible conflict resolution preserves relationships");
    println!("- All resolutions require BFT committee approval");

    Ok(())
}
