//! Simple offline payment example
//!
//! Demonstrates local spend from escrow without network connectivity.
//!
//! Run with: `cargo run --example simple_payment`

use std::sync::Arc;
use vudo_credit::{
    BftCommittee, CreditAccountHandle, DeviceEscrow, MutualCreditScheduler, ReputationTier,
    TransactionMetadata,
};
use vudo_state::StateEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Simple Offline Payment Example ===\n");

    // 1. Setup state engine
    println!("1. Initializing state engine...");
    let state_engine = Arc::new(StateEngine::new().await?);

    // 2. Create BFT committee (4 members, tolerates 1 Byzantine fault)
    println!("2. Creating BFT committee...");
    let members = vec![
        "did:peer:member1".to_string(),
        "did:peer:member2".to_string(),
        "did:peer:member3".to_string(),
        "did:peer:member4".to_string(),
    ];
    let bft_committee = Arc::new(BftCommittee::new(members)?);
    println!(
        "   Committee size: {}, Quorum: {}, Max Byzantine faults: {}",
        bft_committee.size(),
        bft_committee.quorum(),
        bft_committee.max_byzantine_faults()
    );

    // 3. Create scheduler
    println!("\n3. Creating mutual credit scheduler...");
    let scheduler = MutualCreditScheduler::new(
        Arc::clone(&state_engine),
        Arc::clone(&bft_committee),
        "phone-device-123".to_string(),
    )
    .await?;

    // 4. Create Alice's account with $100.00 initial balance
    println!("\n4. Creating Alice's account...");
    let initial_balance = 10_000; // $100.00 in cents
    let alice_account =
        CreditAccountHandle::create(&state_engine, "did:peer:alice".to_string(), initial_balance)
            .await?;
    println!("   Initial balance: ${:.2}", initial_balance as f64 / 100.0);

    // 5. Allocate escrow for Alice's phone
    println!("\n5. Allocating escrow for Alice's phone...");
    let tier = ReputationTier::new(1)?; // Trusted tier
    let escrow = bft_committee
        .grant_escrow(&alice_account, "phone-device-123", tier)
        .await?;
    println!("   Escrow allocated: ${:.2}", escrow.allocated as f64 / 100.0);
    println!("   Expires in {} days", escrow.time_until_expiry() / 86400);

    // Store escrow in scheduler
    scheduler.set_device_escrow("did:peer:alice", escrow.clone());

    // 6. Make offline payment (< 1ms, no network)
    println!("\n6. Making offline payment...");
    println!("   From: Alice");
    println!("   To: Bob");
    println!("   Amount: $5.00");
    println!("   Description: Coffee at cafe");

    let start = std::time::Instant::now();
    let tx_id = scheduler
        .spend_local(
            "did:peer:alice",
            500, // $5.00 in cents
            "did:peer:bob",
            TransactionMetadata {
                description: "Coffee at cafe".to_string(),
                category: Some("food".to_string()),
                invoice_id: None,
            },
        )
        .await?;
    let elapsed = start.elapsed();

    println!("   ✓ Payment completed in {:?}", elapsed);
    println!("   Transaction ID: {}", tx_id);

    // 7. Check remaining escrow
    println!("\n7. Checking remaining escrow...");
    let updated_escrow = scheduler.get_device_escrow("did:peer:alice")?;
    println!(
        "   Remaining: ${:.2} / ${:.2}",
        updated_escrow.remaining as f64 / 100.0,
        updated_escrow.allocated as f64 / 100.0
    );

    // 8. Make another payment
    println!("\n8. Making another payment...");
    println!("   From: Alice");
    println!("   To: Charlie");
    println!("   Amount: $3.50");

    let tx_id2 = scheduler
        .spend_local(
            "did:peer:alice",
            350, // $3.50 in cents
            "did:peer:charlie",
            TransactionMetadata {
                description: "Bus ticket".to_string(),
                category: Some("transport".to_string()),
                invoice_id: None,
            },
        )
        .await?;

    println!("   ✓ Payment completed");
    println!("   Transaction ID: {}", tx_id2);

    // 9. Final escrow status
    println!("\n9. Final escrow status...");
    let final_escrow = scheduler.get_device_escrow("did:peer:alice")?;
    println!(
        "   Total spent: ${:.2}",
        final_escrow.spent as f64 / 100.0
    );
    println!(
        "   Remaining: ${:.2}",
        final_escrow.remaining as f64 / 100.0
    );

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("- Payments complete in < 1ms (no network required)");
    println!("- Escrow prevents double-spend locally");
    println!("- Transactions queued for reconciliation when online");

    Ok(())
}
