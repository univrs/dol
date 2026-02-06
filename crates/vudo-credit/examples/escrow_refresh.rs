//! Escrow refresh example
//!
//! Demonstrates automatic escrow refresh when running low.
//!
//! Run with: `cargo run --example escrow_refresh`

use std::sync::Arc;
use vudo_credit::{
    BftCommittee, CreditAccountHandle, DeviceEscrow, MutualCreditScheduler, ReputationManager,
    ReputationTier, TransactionMetadata,
};
use vudo_state::StateEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Escrow Refresh Example ===\n");

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
        "laptop-device-456".to_string(),
    )
    .await?;

    // 2. Create account with good reputation
    println!("\n2. Creating account with reputation tier 2 (Established)...");
    let alice_account =
        CreditAccountHandle::create(&state_engine, "did:peer:alice".to_string(), 100_000)
            .await?;

    // Set reputation tier
    alice_account.update(|acc| {
        acc.reputation_tier = ReputationTier::new(2)?;
        Ok(())
    })?;

    let tier = alice_account.read(|acc| Ok(acc.reputation_tier))?;
    println!("   Reputation: {}", tier);
    println!(
        "   Credit limit: {}",
        ReputationManager::format_credit_limit(tier)
    );
    println!(
        "   Escrow limit: {}",
        ReputationManager::format_escrow_limit(tier)
    );

    // 3. Initial escrow allocation
    println!("\n3. Allocating initial escrow...");
    let escrow = bft_committee
        .grant_escrow(&alice_account, "laptop-device-456", tier)
        .await?;
    println!("   Allocated: ${:.2}", escrow.allocated as f64 / 100.0);

    scheduler
        .escrow_manager
        .set("did:peer:alice", "laptop-device-456", escrow.clone());

    // 4. Spend most of the escrow
    println!("\n4. Making multiple payments to deplete escrow...");
    let num_payments = 8;
    let payment_amount = escrow.allocated / 10; // 10% per payment

    for i in 1..=num_payments {
        let tx_id = scheduler
            .spend_local(
                "did:peer:alice",
                payment_amount,
                "did:peer:merchant",
                TransactionMetadata {
                    description: format!("Payment #{}", i),
                    category: Some("shopping".to_string()),
                    invoice_id: None,
                },
            )
            .await?;

        let current_escrow = scheduler.get_device_escrow("did:peer:alice")?;
        let percent_remaining =
            (current_escrow.remaining as f64 / current_escrow.allocated as f64) * 100.0;

        println!(
            "   Payment #{}: ${:.2} ({}% remaining)",
            i,
            payment_amount as f64 / 100.0,
            percent_remaining as u32
        );
    }

    // 5. Check if escrow is low
    println!("\n5. Checking escrow status...");
    let current_escrow = scheduler.get_device_escrow("did:peer:alice")?;
    let is_low = current_escrow.is_low(20);
    println!(
        "   Remaining: ${:.2}",
        current_escrow.remaining as f64 / 100.0
    );
    println!("   Is low (< 20%)? {}", if is_low { "YES" } else { "NO" });

    if is_low {
        // 6. Request escrow refresh
        println!("\n6. Requesting escrow refresh...");
        println!("   Contacting BFT committee...");

        scheduler
            .request_escrow_refresh("did:peer:alice")
            .await?;

        println!("   ✓ Escrow refresh approved");

        // Check new escrow
        let new_escrow = scheduler.get_device_escrow("did:peer:alice")?;
        println!("   New allocation: ${:.2}", new_escrow.allocated as f64 / 100.0);
        println!("   Expires in {} days", new_escrow.time_until_expiry() / 86400);
    }

    // 7. Continue spending with refreshed escrow
    println!("\n7. Making payment with refreshed escrow...");
    let tx_id = scheduler
        .spend_local(
            "did:peer:alice",
            1000, // $10.00
            "did:peer:merchant",
            TransactionMetadata {
                description: "Payment after refresh".to_string(),
                category: Some("shopping".to_string()),
                invoice_id: None,
            },
        )
        .await?;
    println!("   ✓ Payment successful: {}", tx_id);

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("- Escrow automatically refreshes when running low");
    println!("- Refresh requires BFT committee consensus");
    println!("- No interruption to local operations");

    Ok(())
}
