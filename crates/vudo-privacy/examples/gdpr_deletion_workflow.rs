//! Example: Complete GDPR deletion workflow.
//!
//! Demonstrates the full GDPR Article 17 compliance workflow including
//! cryptographic deletion, audit logging, and deletion reports.

use vudo_privacy::gdpr::{DeletionRequest, GdprComplianceEngine};

#[tokio::main]
async fn main() -> vudo_privacy::error::Result<()> {
    println!("=== GDPR Deletion Workflow Example ===\n");

    // Initialize GDPR compliance engine
    let engine = GdprComplianceEngine::new()?;
    println!("1. GDPR Compliance Engine initialized");
    println!();

    // Setup: Create user data
    println!("2. Creating user data...");
    let dek = engine.crypto().generate_dek("did:peer:alice@example.com")?;

    let email = b"alice@example.com";
    let encrypted_email = engine.crypto().encrypt_field(&dek, email)?;

    println!("   ✓ User: did:peer:alice@example.com");
    println!("   ✓ DEK created");
    println!("   ✓ Personal data encrypted");
    println!();

    // Verify data can be decrypted before deletion
    println!("3. Verifying data access...");
    let decrypted = engine.crypto().decrypt_field(&dek, &encrypted_email)?;
    println!("   ✓ Email accessible: {}", String::from_utf8_lossy(&decrypted));
    println!();

    // GDPR deletion request
    println!("4. Executing GDPR deletion request...");
    let request = DeletionRequest::all_data("app.example".to_string())
        .add_public_path("/profile".to_string())
        .add_public_path("/settings".to_string());

    let report = engine.execute_deletion("did:peer:alice@example.com", request).await?;

    println!("   ✓ Request ID: {}", report.request_id);
    println!("   ✓ Completed at: {}", report.completed_at);
    println!("   ✓ Irreversible: {}", report.irreversible);
    println!("   ✓ Categories deleted: {:?}", report.categories_deleted);
    println!();

    // Verify deletion receipt
    if let Some(receipt) = &report.crypto_proof {
        println!("5. Cryptographic proof:");
        println!("   ✓ Owner: {}", receipt.owner);
        println!("   ✓ Deleted at: {}", receipt.deleted_at);
        println!("   ✓ Irreversible: {}", receipt.irreversible);
        println!();
    }

    // Verify data is now inaccessible
    println!("6. Verifying data erasure...");
    let deleted_dek = engine.crypto().get_dek("did:peer:alice@example.com")?;

    match engine.crypto().decrypt_field(&deleted_dek, &encrypted_email) {
        Ok(_) => println!("   ✗ ERROR: Data should be inaccessible!"),
        Err(e) => println!("   ✓ Data permanently erased: {}", e),
    }
    println!();

    // Check deletion status
    println!("7. Checking deletion status...");
    assert!(engine.is_deleted("did:peer:alice@example.com"));
    println!("   ✓ User marked as deleted");

    if let Some(stored_report) = engine.get_deletion_report("did:peer:alice@example.com") {
        println!("   ✓ Deletion report available: {}", stored_report.request_id);
    }
    println!();

    // Export audit log for compliance
    println!("8. Exporting audit log...");
    let audit_json = engine.export_audit_log()?;
    println!("   ✓ Audit log exported ({} bytes)", audit_json.len());
    println!("   ✓ Contains compliance proof for regulatory audits");
    println!();

    // Deletion statistics
    println!("9. Deletion statistics:");
    let stats = engine.get_stats();
    println!("   • Total deletions: {}", stats.total_deletions);
    println!("   • Cryptographic erasures: {}", stats.cryptographic_erasures);
    println!("   • Tombstones: {}", stats.tombstones);
    println!("   • Anonymizations: {}", stats.anonymizations);
    println!();

    // Test idempotency
    println!("10. Testing idempotent deletion...");
    let request2 = DeletionRequest::personal_only("app.example".to_string());
    let report2 = engine.execute_deletion("did:peer:alice@example.com", request2).await?;
    println!("   ✓ Second deletion returned same report: {}", report.request_id == report2.request_id);
    println!();

    println!("=== Example Complete ===");
    println!("\nGDPR Compliance Notes:");
    println!("• Deletion is cryptographically irreversible");
    println!("• Audit trail provides proof of deletion");
    println!("• Deletion reports satisfy regulatory requirements");
    println!("• Idempotent operations prevent duplicate processing");
    println!("• Complies with GDPR Article 17 (Right to Erasure)");

    Ok(())
}
