//! Basic exegesis CRUD operations example.
//!
//! This example demonstrates:
//! - Creating exegesis for a Gene
//! - Reading exegesis content
//! - Editing exegesis
//! - Tracking contributors

use dol_exegesis::{ExegesisManager, Result};
use std::sync::Arc;
use vudo_state::StateEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Basic Exegesis Example ===\n");

    // Initialize state engine
    let state_engine = Arc::new(StateEngine::new().await?);
    let manager = ExegesisManager::new(state_engine).await?;

    // Create exegesis for a Gene
    println!("1. Creating exegesis for 'user.profile' v1.0.0...");
    let doc = manager
        .create_exegesis(
            "user.profile",
            "1.0.0",
            "A user profile contains identity, preferences, and settings."
        )
        .await?;

    println!("   Created: {}", doc.document_id());
    println!("   Content: {}", doc.content);
    println!();

    // Read the exegesis
    println!("2. Reading exegesis...");
    let retrieved = manager.get_exegesis("user.profile", "1.0.0").await?;
    println!("   Gene: {}", retrieved.gene_id);
    println!("   Version: {}", retrieved.gene_version);
    println!("   Content: {}", retrieved.content);
    println!();

    // Edit the exegesis
    println!("3. Editing exegesis (by Alice)...");
    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
            content.push_str("\n\nUpdated by Alice: Profiles support multiple authentication methods.");
        })
        .await?;

    let updated = manager.get_exegesis("user.profile", "1.0.0").await?;
    println!("   Updated content: {}", updated.content);
    println!("   Contributors: {:?}", updated.contributors);
    println!();

    // Multiple edits by different users
    println!("4. Adding edits from Bob and Charlie...");
    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:bob", |content| {
            content.push_str("\n\nBob's note: Profiles can be exported as JSON.");
        })
        .await?;

    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:charlie", |content| {
            content.push_str("\n\nCharlie's addition: Profiles support GDPR compliance.");
        })
        .await?;

    let final_doc = manager.get_exegesis("user.profile", "1.0.0").await?;
    println!("   Final content:\n{}", final_doc.content);
    println!("\n   All contributors: {:?}", final_doc.contributors);
    println!();

    // Check existence
    println!("5. Checking existence...");
    println!("   user.profile@1.0.0 exists: {}", manager.exists("user.profile", "1.0.0").await);
    println!("   user.profile@2.0.0 exists: {}", manager.exists("user.profile", "2.0.0").await);
    println!();

    println!("=== Example Complete ===");
    Ok(())
}
