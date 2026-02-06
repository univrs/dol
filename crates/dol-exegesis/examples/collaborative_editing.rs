//! Collaborative editing example.
//!
//! This example demonstrates:
//! - Multiple users editing the same exegesis concurrently
//! - CRDT-based conflict resolution
//! - Change subscriptions
//! - Real-time collaboration

use dol_exegesis::{CollaborativeEditor, ExegesisManager, Result};
use std::sync::Arc;
use vudo_state::StateEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Collaborative Editing Example ===\n");

    // Initialize state engine and manager
    let state_engine = Arc::new(StateEngine::new().await?);
    let manager = Arc::new(ExegesisManager::new(Arc::clone(&state_engine)).await?);

    // Create initial exegesis
    println!("1. Creating initial exegesis for 'api.authentication'...");
    manager
        .create_exegesis(
            "api.authentication",
            "1.0.0",
            "Authentication API for user login and session management."
        )
        .await?;
    println!("   Created successfully\n");

    // Set up collaborative editor
    let editor = CollaborativeEditor::new(Arc::clone(&manager));

    // Subscribe to changes (in a real app, this would trigger UI updates)
    println!("2. Setting up change subscription...");
    let _subscription = editor
        .subscribe_changes("api.authentication", "1.0.0")
        .await?;
    println!("   Subscription active\n");

    // Simulate concurrent edits by multiple developers
    println!("3. Simulating concurrent edits by Alice, Bob, and Charlie...\n");

    let manager_alice = Arc::clone(&manager);
    let manager_bob = Arc::clone(&manager);
    let manager_charlie = Arc::clone(&manager);

    // Spawn concurrent editing tasks
    let alice = tokio::spawn(async move {
        println!("   [Alice] Adding OAuth2 support...");
        manager_alice
            .edit_exegesis(
                "api.authentication",
                "1.0.0",
                "did:peer:alice",
                |content| {
                    content.push_str("\n\n## OAuth2 Support");
                    content.push_str("\n- Supports OAuth2 authorization code flow");
                    content.push_str("\n- Configurable client credentials");
                },
            )
            .await
    });

    let bob = tokio::spawn(async move {
        println!("   [Bob] Adding JWT documentation...");
        manager_bob
            .edit_exegesis(
                "api.authentication",
                "1.0.0",
                "did:peer:bob",
                |content| {
                    content.push_str("\n\n## JWT Tokens");
                    content.push_str("\n- Issues JWT tokens on successful authentication");
                    content.push_str("\n- Tokens expire after 1 hour");
                },
            )
            .await
    });

    let charlie = tokio::spawn(async move {
        println!("   [Charlie] Adding security notes...");
        manager_charlie
            .edit_exegesis(
                "api.authentication",
                "1.0.0",
                "did:peer:charlie",
                |content| {
                    content.push_str("\n\n## Security");
                    content.push_str("\n- All endpoints require HTTPS");
                    content.push_str("\n- Implements rate limiting (100 requests/minute)");
                },
            )
            .await
    });

    // Wait for all edits to complete
    alice.await.unwrap()?;
    bob.await.unwrap()?;
    charlie.await.unwrap()?;

    println!("\n4. All edits completed. Fetching final document...\n");

    // Retrieve the merged document
    let final_doc = manager
        .get_exegesis("api.authentication", "1.0.0")
        .await?;

    println!("=== Final Exegesis ===");
    println!("{}\n", final_doc.content);
    println!("Contributors: {:?}", final_doc.contributors);
    println!(
        "Last modified: {}",
        final_doc
            .last_modified_datetime()
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Unknown".to_string())
    );

    println!("\n=== Example Complete ===");
    println!("All three developers' contributions were successfully merged!");
    Ok(())
}
