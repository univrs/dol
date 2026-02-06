//! Example: Audit trail for GDPR deletions.
//!
//! Demonstrates comprehensive audit logging for regulatory compliance.

use vudo_privacy::audit::{DataCategory, DeletionAuditLog, DeletionMethod};
use vudo_privacy::crypto::{DeletionReceipt, PersonalDataCrypto};

fn main() -> vudo_privacy::error::Result<()> {
    println!("=== Audit Trail Example ===\n");

    // Create audit log and crypto manager
    let audit_log = DeletionAuditLog::new();
    let crypto = PersonalDataCrypto::new();

    // Simulate multiple GDPR deletions
    println!("1. Recording multiple GDPR deletions...\n");

    // Alice: Personal data deletion
    println!("   User: Alice");
    crypto.generate_dek("did:peer:alice")?;
    let receipt_alice = crypto.delete_dek("did:peer:alice")?;

    let request_id_alice = audit_log.record_deletion(
        "did:peer:alice",
        vec![DataCategory::PersonalData],
        DeletionMethod::CryptographicErasure,
        Some(receipt_alice),
    );
    println!("   • Request ID: {}", request_id_alice);
    println!("   • Method: Cryptographic Erasure");
    println!("   • Proof: Yes");
    println!();

    // Bob: Full deletion (personal + public)
    println!("   User: Bob");
    crypto.generate_dek("did:peer:bob")?;
    let receipt_bob = crypto.delete_dek("did:peer:bob")?;

    audit_log.record_deletion(
        "did:peer:bob",
        vec![DataCategory::PersonalData],
        DeletionMethod::CryptographicErasure,
        Some(receipt_bob),
    );

    let request_id_bob = audit_log.record_deletion(
        "did:peer:bob",
        vec![DataCategory::PublicData],
        DeletionMethod::Tombstone,
        None,
    );
    println!("   • Request ID: {}", request_id_bob);
    println!("   • Methods: Cryptographic Erasure + Tombstone");
    println!("   • Categories: Personal + Public data");
    println!();

    // Charlie: Transaction anonymization
    println!("   User: Charlie");
    let request_id_charlie = audit_log.record_deletion(
        "did:peer:charlie",
        vec![DataCategory::TransactionHistory],
        DeletionMethod::Anonymization,
        None,
    );
    println!("   • Request ID: {}", request_id_charlie);
    println!("   • Method: Anonymization (legal retention)");
    println!();

    // Query audit log
    println!("2. Querying audit log...\n");

    println!("   Total deletions: {}", audit_log.total_deletions());
    println!();

    // Get entries for specific user
    println!("   Alice's deletion history:");
    let alice_entries = audit_log.get_entries_for_user("did:peer:alice");
    for entry in &alice_entries {
        println!("     • Request: {}", entry.request_id);
        println!("     • Method: {:?}", entry.method);
        println!("     • Has proof: {}", entry.has_proof());
        println!("     • Age: {} seconds", entry.age_seconds());
    }
    println!();

    // Get entries by method
    println!("   Cryptographic erasures: {}",
        audit_log.get_entries_by_method(DeletionMethod::CryptographicErasure).len()
    );
    println!("   Tombstones: {}",
        audit_log.get_entries_by_method(DeletionMethod::Tombstone).len()
    );
    println!("   Anonymizations: {}",
        audit_log.get_entries_by_method(DeletionMethod::Anonymization).len()
    );
    println!();

    // Get entries by category
    println!("   Personal data deletions: {}",
        audit_log.get_entries_by_category(DataCategory::PersonalData).len()
    );
    println!("   Public data deletions: {}",
        audit_log.get_entries_by_category(DataCategory::PublicData).len()
    );
    println!();

    // Export audit log for compliance
    println!("3. Exporting audit log for regulatory compliance...\n");

    let json = audit_log.export_json()?;
    println!("   ✓ Audit log exported ({} bytes)", json.len());
    println!("   ✓ Format: JSON");
    println!("   ✓ Includes cryptographic proofs");
    println!();

    // Show sample entry
    if let Some(entry) = audit_log.get_entry(&request_id_alice) {
        println!("4. Sample audit entry (Alice):\n");
        let entry_json = serde_json::to_string_pretty(&entry)?;
        println!("{}", entry_json);
        println!();
    }

    // Import/export test
    println!("5. Testing audit log persistence...\n");

    let imported = DeletionAuditLog::import_json(&json)?;
    println!("   ✓ Audit log imported successfully");
    println!("   ✓ Entries preserved: {}", imported.total_deletions());
    assert_eq!(imported.total_deletions(), audit_log.total_deletions());
    println!();

    println!("=== Example Complete ===");
    println!("\nAudit Trail Benefits:");
    println!("• Immutable record of all deletions");
    println!("• Cryptographic proofs for erasure");
    println!("• Query by user, method, or category");
    println!("• Export for regulatory audits");
    println!("• Satisfies GDPR record-keeping requirements");

    Ok(())
}
