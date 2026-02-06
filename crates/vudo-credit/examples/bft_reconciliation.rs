//! BFT reconciliation example
//!
//! Demonstrates BFT committee consensus for balance reconciliation.
//!
//! Run with: `cargo run --example bft_reconciliation`

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

    println!("=== BFT Reconciliation Example ===\n");

    // 1. Setup BFT committee
    println!("1. Setting up BFT committee...");
    let members = vec![
        "did:peer:validator1".to_string(),
        "did:peer:validator2".to_string(),
        "did:peer:validator3".to_string(),
        "did:peer:validator4".to_string(),
        "did:peer:validator5".to_string(),
        "did:peer:validator6".to_string(),
        "did:peer:validator7".to_string(),
    ];

    let bft_committee = Arc::new(BftCommittee::new(members)?);
    println!("   Committee size: {}", bft_committee.size());
    println!("   Quorum required: {}", bft_committee.quorum());
    println!(
        "   Max Byzantine faults tolerated: {}",
        bft_committee.max_byzantine_faults()
    );

    // 2. Setup state engine and scheduler
    println!("\n2. Initializing state engine...");
    let state_engine = Arc::new(StateEngine::new().await?);
    let scheduler = MutualCreditScheduler::new(
        Arc::clone(&state_engine),
        Arc::clone(&bft_committee),
        "tablet-device-789".to_string(),
    )
    .await?;

    // 3. Create account and allocate escrow
    println!("\n3. Creating account and allocating escrow...");
    let initial_balance = 50_000; // $500.00
    let alice_account =
        CreditAccountHandle::create(&state_engine, "did:peer:alice".to_string(), initial_balance)
            .await?;

    let tier = ReputationTier::new(2)?;
    let escrow = bft_committee
        .grant_escrow(&alice_account, "tablet-device-789", tier)
        .await?;

    scheduler
        .escrow_manager
        .set("did:peer:alice", "tablet-device-789", escrow);

    println!(
        "   Initial confirmed balance: ${:.2}",
        initial_balance as f64 / 100.0
    );

    // 4. Make several offline payments
    println!("\n4. Making offline payments (pending reconciliation)...");
    let payments = vec![
        ("Bob", 5000, "Consulting work"),
        ("Charlie", 3500, "Software license"),
        ("Dave", 2000, "Hosting fees"),
    ];

    for (recipient, amount, description) in payments {
        let tx_id = scheduler
            .spend_local(
                "did:peer:alice",
                amount,
                &format!("did:peer:{}", recipient.to_lowercase()),
                TransactionMetadata {
                    description: description.to_string(),
                    category: Some("business".to_string()),
                    invoice_id: None,
                },
            )
            .await?;

        println!(
            "   → {} paid ${:.2} to {}",
            "Alice",
            amount as f64 / 100.0,
            recipient
        );
    }

    // 5. Check account state before reconciliation
    println!("\n5. Account state before reconciliation:");
    alice_account.read(|acc| {
        println!("   Confirmed balance: ${:.2}", acc.confirmed_balance as f64 / 100.0);
        println!("   Pending credits: ${:.2}", acc.pending_credits as f64 / 100.0);
        println!("   Pending debits: ${:.2}", acc.total_pending_debits() as f64 / 100.0);
        println!("   Pending transactions: {}", acc.transactions.len());
        Ok(())
    })?;

    // 6. Trigger BFT reconciliation
    println!("\n6. Starting BFT reconciliation...");
    println!("   Broadcasting proposal to committee...");
    println!("   Collecting votes from members...");

    scheduler.reconcile_account("did:peer:alice").await?;

    println!("   ✓ Consensus reached!");

    // 7. Check account state after reconciliation
    println!("\n7. Account state after reconciliation:");
    alice_account.read(|acc| {
        println!("   Confirmed balance: ${:.2}", acc.confirmed_balance as f64 / 100.0);
        println!("   Pending credits: ${:.2}", acc.pending_credits as f64 / 100.0);
        println!("   Pending debits: ${:.2}", acc.total_pending_debits() as f64 / 100.0);
        println!("   Confirmed transactions: {}", acc.transactions.iter().filter(|tx| tx.is_confirmed()).count());
        println!("   Last reconciliation: {}", acc.last_reconciliation);
        Ok(())
    })?;

    // 8. Demonstrate reconciliation with pending credits
    println!("\n8. Simulating incoming pending credits...");
    alice_account.update(|acc| {
        acc.pending_credits = 15_000; // $150.00 pending incoming
        Ok(())
    })?;

    println!("   Added ${:.2} in pending credits", 150.0);

    println!("\n9. Reconciling with pending credits...");
    scheduler.reconcile_account("did:peer:alice").await?;

    alice_account.read(|acc| {
        println!("   New confirmed balance: ${:.2}", acc.confirmed_balance as f64 / 100.0);
        println!("   Pending credits cleared: ${:.2}", acc.pending_credits as f64 / 100.0);
        Ok(())
    })?;

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("- BFT committee provides Byzantine fault tolerance");
    println!("- Reconciliation requires quorum (2f+1 votes)");
    println!("- Confirmed balance updated atomically");
    println!("- Pending transactions marked as confirmed");

    Ok(())
}
