//! Version evolution example.
//!
//! This example demonstrates:
//! - Linking exegesis to Gene evolution versions
//! - Preserving documentation history across versions
//! - Independent editing of version-specific exegesis

use dol_exegesis::{ExegesisManager, Result};
use std::sync::Arc;
use vudo_state::StateEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Version Evolution Example ===\n");

    // Initialize state engine and manager
    let state_engine = Arc::new(StateEngine::new().await?);
    let manager = ExegesisManager::new(state_engine).await?;

    // Create initial exegesis for v1.0.0
    println!("1. Creating exegesis for 'payment.gateway' v1.0.0...");
    manager
        .create_exegesis(
            "payment.gateway",
            "1.0.0",
            "Payment gateway for processing credit card transactions.\n\
             Supports Visa, Mastercard, and American Express."
        )
        .await?;

    // Add some contributors to v1.0.0
    manager
        .edit_exegesis("payment.gateway", "1.0.0", "did:peer:alice", |content| {
            content.push_str("\n\nImplemented by Alice in Sprint 1.");
        })
        .await?;

    manager
        .edit_exegesis("payment.gateway", "1.0.0", "did:peer:bob", |content| {
            content.push_str("\nReviewed and tested by Bob.");
        })
        .await?;

    let v1_doc = manager.get_exegesis("payment.gateway", "1.0.0").await?;
    println!("   v1.0.0 content:");
    println!("   {}", v1_doc.content);
    println!("   Contributors: {:?}\n", v1_doc.contributors);

    // Evolve to v2.0.0 - link exegesis
    println!("2. Evolving to v2.0.0...");
    let v2_doc = manager
        .link_to_evolution("payment.gateway", "1.0.0", "2.0.0")
        .await?;
    println!("   Linked exegesis from v1.0.0 to v2.0.0");
    println!("   v2.0.0 initial content:");
    println!("   {}", v2_doc.content);
    println!("   Contributors (inherited): {:?}\n", v2_doc.contributors);

    // Update v2.0.0 exegesis with new information
    println!("3. Updating v2.0.0 exegesis with new features...");
    manager
        .edit_exegesis("payment.gateway", "2.0.0", "did:peer:charlie", |content| {
            content.push_str(
                "\n\n## New in v2.0.0\n\
                 - Added cryptocurrency support (Bitcoin, Ethereum)\n\
                 - Implemented 3D Secure authentication\n\
                 - Added fraud detection system"
            );
        })
        .await?;

    let updated_v2 = manager.get_exegesis("payment.gateway", "2.0.0").await?;
    println!("   Updated v2.0.0 content:");
    println!("   {}", updated_v2.content);
    println!("   Contributors: {:?}\n", updated_v2.contributors);

    // Verify v1.0.0 is unchanged
    println!("4. Verifying v1.0.0 remains unchanged...");
    let v1_check = manager.get_exegesis("payment.gateway", "1.0.0").await?;
    println!("   v1.0.0 content:");
    println!("   {}", v1_check.content);
    println!("   (No mention of v2.0.0 features - versions are independent)\n");

    // Evolve to v3.0.0
    println!("5. Evolving to v3.0.0...");
    manager
        .link_to_evolution("payment.gateway", "2.0.0", "3.0.0")
        .await?;

    manager
        .edit_exegesis("payment.gateway", "3.0.0", "did:peer:dave", |content| {
            content.push_str(
                "\n\n## New in v3.0.0\n\
                 - Real-time transaction monitoring\n\
                 - Automatic currency conversion\n\
                 - Webhook support for payment events"
            );
        })
        .await?;

    let v3_doc = manager.get_exegesis("payment.gateway", "3.0.0").await?;
    println!("   v3.0.0 content:");
    println!("   {}", v3_doc.content);
    println!("   Contributors: {:?}\n", v3_doc.contributors);

    // Show version lineage
    println!("6. Version lineage summary:");
    println!("   v1.0.0 → v2.0.0 → v3.0.0");
    println!("   Each version preserves exegesis history while allowing independent updates\n");

    // Demonstrate that all versions coexist
    println!("7. Verifying all versions exist independently:");
    for version in &["1.0.0", "2.0.0", "3.0.0"] {
        let exists = manager.exists("payment.gateway", version).await;
        let doc = manager.get_exegesis("payment.gateway", version).await?;
        println!(
            "   v{} - exists: {}, contributors: {}",
            version,
            exists,
            doc.contributors.len()
        );
    }

    println!("\n=== Example Complete ===");
    println!("Exegesis successfully tracked across Gene evolution!");
    Ok(())
}
