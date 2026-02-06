//! Integration tests for VUDO Privacy.

use vudo_privacy::{
    audit::{DataCategory, DeletionMethod},
    crypto::{DataEncryptionKey, PersonalDataCrypto},
    gdpr::{DeletionRequest, GdprComplianceEngine},
    pseudonymous::{ActorIdMapper, PseudonymousActorId},
};

#[test]
fn test_dek_lifecycle() {
    let crypto = PersonalDataCrypto::new();

    // Generate DEK
    let dek = crypto.generate_dek("did:peer:alice").unwrap();
    assert_eq!(dek.owner, "did:peer:alice");
    assert!(!dek.is_deleted());

    // Encrypt data
    let plaintext = b"sensitive data";
    let encrypted = crypto.encrypt_field(&dek, plaintext).unwrap();
    assert_ne!(encrypted.ciphertext, plaintext);

    // Decrypt data
    let decrypted = crypto.decrypt_field(&dek, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);

    // Delete DEK
    let receipt = crypto.delete_dek("did:peer:alice").unwrap();
    assert!(receipt.irreversible);

    // Verify decryption fails
    let deleted_dek = crypto.get_dek("did:peer:alice").unwrap();
    assert!(deleted_dek.is_deleted());
    let result = crypto.decrypt_field(&deleted_dek, &encrypted);
    assert!(result.is_err());
}

#[test]
fn test_multiple_users() {
    let crypto = PersonalDataCrypto::new();

    // Create DEKs for multiple users
    let dek_alice = crypto.generate_dek("did:peer:alice").unwrap();
    let dek_bob = crypto.generate_dek("did:peer:bob").unwrap();

    // Encrypt data for each user
    let alice_data = b"alice's secret";
    let bob_data = b"bob's secret";

    let encrypted_alice = crypto.encrypt_field(&dek_alice, alice_data).unwrap();
    let encrypted_bob = crypto.encrypt_field(&dek_bob, bob_data).unwrap();

    // Each user can decrypt their own data
    assert_eq!(
        crypto.decrypt_field(&dek_alice, &encrypted_alice).unwrap(),
        alice_data
    );
    assert_eq!(
        crypto.decrypt_field(&dek_bob, &encrypted_bob).unwrap(),
        bob_data
    );

    // Users cannot decrypt each other's data
    assert!(crypto.decrypt_field(&dek_alice, &encrypted_bob).is_err());
    assert!(crypto.decrypt_field(&dek_bob, &encrypted_alice).is_err());
}

#[test]
fn test_pseudonymous_actor_ids() {
    let pseudo_alice = PseudonymousActorId::from_did("did:peer:alice").unwrap();
    let pseudo_bob = PseudonymousActorId::from_did("did:peer:bob").unwrap();

    // Different DIDs produce different pseudonyms
    assert_ne!(pseudo_alice.actor_id(), pseudo_bob.actor_id());

    // Same DID produces same pseudonym (deterministic)
    let pseudo_alice2 = PseudonymousActorId::from_did("did:peer:alice").unwrap();
    assert_eq!(pseudo_alice.actor_id(), pseudo_alice2.actor_id());

    // Mapper can resolve pseudonyms
    let mapper = ActorIdMapper::new();
    mapper.register(&pseudo_alice);
    mapper.register(&pseudo_bob);

    assert_eq!(
        mapper.resolve(&pseudo_alice.actor_id()),
        Some("did:peer:alice".to_string())
    );
    assert_eq!(
        mapper.resolve(&pseudo_bob.actor_id()),
        Some("did:peer:bob".to_string())
    );
}

#[tokio::test]
async fn test_gdpr_personal_data_deletion() {
    let engine = GdprComplianceEngine::new().unwrap();

    // Create DEK
    engine.crypto().generate_dek("did:peer:alice").unwrap();

    // Execute deletion
    let request = DeletionRequest::personal_only("app.example".to_string());
    let report = engine.execute_deletion("did:peer:alice", request).await.unwrap();

    // Verify report
    assert!(report.irreversible);
    assert!(report.categories_deleted.contains(&DataCategory::PersonalData));
    assert!(report.crypto_proof.is_some());

    // Verify user is marked as deleted
    assert!(engine.is_deleted("did:peer:alice"));
}

#[tokio::test]
async fn test_gdpr_full_deletion() {
    let engine = GdprComplianceEngine::new().unwrap();

    // Create DEK
    engine.crypto().generate_dek("did:peer:alice").unwrap();

    // Execute full deletion
    let request = DeletionRequest::all_data("app.example".to_string())
        .add_public_path("/profile".to_string())
        .add_public_path("/settings".to_string());

    let report = engine.execute_deletion("did:peer:alice", request).await.unwrap();

    // Verify all categories deleted
    assert!(report.categories_deleted.contains(&DataCategory::PersonalData));
    assert!(report.categories_deleted.contains(&DataCategory::PublicData));
    assert!(report.categories_deleted.contains(&DataCategory::TransactionHistory));
}

#[tokio::test]
async fn test_audit_log_completeness() {
    let engine = GdprComplianceEngine::new().unwrap();

    // Create DEKs for multiple users
    engine.crypto().generate_dek("did:peer:alice").unwrap();
    engine.crypto().generate_dek("did:peer:bob").unwrap();

    // Delete data
    let request = DeletionRequest::personal_only("app.example".to_string());
    engine.execute_deletion("did:peer:alice", request.clone()).await.unwrap();
    engine.execute_deletion("did:peer:bob", request).await.unwrap();

    // Verify audit log
    let audit = engine.audit_log();
    assert_eq!(audit.total_deletions(), 2);

    let alice_entries = audit.get_entries_for_user("did:peer:alice");
    assert_eq!(alice_entries.len(), 1);
    assert!(alice_entries[0].has_proof());

    let bob_entries = audit.get_entries_for_user("did:peer:bob");
    assert_eq!(bob_entries.len(), 1);
}

#[tokio::test]
async fn test_idempotent_deletion() {
    let engine = GdprComplianceEngine::new().unwrap();

    // Create DEK
    engine.crypto().generate_dek("did:peer:alice").unwrap();

    let request = DeletionRequest::personal_only("app.example".to_string());

    // First deletion
    let report1 = engine.execute_deletion("did:peer:alice", request.clone()).await.unwrap();

    // Second deletion (idempotent)
    let report2 = engine.execute_deletion("did:peer:alice", request).await.unwrap();

    // Should return same report
    assert_eq!(report1.request_id, report2.request_id);
}

#[tokio::test]
async fn test_deletion_without_dek() {
    let engine = GdprComplianceEngine::new().unwrap();

    // Try to delete without creating DEK first
    let request = DeletionRequest::personal_only("app.example".to_string());
    let result = engine.execute_deletion("did:peer:alice", request).await;

    // Should succeed (just skip personal data deletion)
    assert!(result.is_ok());
}

#[test]
fn test_encryption_with_deleted_key() {
    let crypto = PersonalDataCrypto::new();
    let mut dek = crypto.generate_dek("did:peer:alice").unwrap();

    // Delete the key
    dek.mark_deleted();

    // Try to encrypt with deleted key
    let result = crypto.encrypt_field(&dek, b"data");
    assert!(result.is_err());
}

#[test]
fn test_concurrent_encryption() {
    use std::sync::Arc;
    use std::thread;

    let crypto = Arc::new(PersonalDataCrypto::new());
    let dek = Arc::new(crypto.generate_dek("did:peer:alice").unwrap());

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let crypto = Arc::clone(&crypto);
            let dek = Arc::clone(&dek);

            thread::spawn(move || {
                let data = format!("data_{}", i);
                let encrypted = crypto.encrypt_field(&dek, data.as_bytes()).unwrap();
                let decrypted = crypto.decrypt_field(&dek, &encrypted).unwrap();
                assert_eq!(decrypted, data.as_bytes());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[tokio::test]
async fn test_deletion_stats() {
    let engine = GdprComplianceEngine::new().unwrap();

    // Create DEKs
    engine.crypto().generate_dek("did:peer:alice").unwrap();
    engine.crypto().generate_dek("did:peer:bob").unwrap();
    engine.crypto().generate_dek("did:peer:charlie").unwrap();

    // Delete with different methods
    let personal_request = DeletionRequest::personal_only("app.example".to_string());
    engine.execute_deletion("did:peer:alice", personal_request).await.unwrap();

    let full_request = DeletionRequest::all_data("app.example".to_string())
        .add_public_path("/profile".to_string());
    engine.execute_deletion("did:peer:bob", full_request).await.unwrap();

    // Get stats
    let stats = engine.get_stats();
    assert!(stats.total_deletions >= 2);
    assert!(stats.cryptographic_erasures >= 2);
}

#[test]
fn test_audit_log_export_import() {
    let crypto = PersonalDataCrypto::new();
    crypto.generate_dek("did:peer:alice").unwrap();

    let audit = vudo_privacy::audit::DeletionAuditLog::new();
    audit.record_deletion(
        "did:peer:alice",
        vec![DataCategory::PersonalData],
        DeletionMethod::CryptographicErasure,
        None,
    );

    // Export to JSON
    let json = audit.export_json().unwrap();
    assert!(!json.is_empty());

    // Import from JSON
    let imported = vudo_privacy::audit::DeletionAuditLog::import_json(&json).unwrap();
    assert_eq!(imported.total_deletions(), 1);

    let entries = imported.get_entries_for_user("did:peer:alice");
    assert_eq!(entries.len(), 1);
}

#[test]
fn test_deletion_report_serialization() {
    let report = vudo_privacy::gdpr::DeletionReport {
        request_id: "test-123".to_string(),
        completed_at: 1234567890,
        irreversible: true,
        compliance_note: "Test deletion".to_string(),
        categories_deleted: vec![DataCategory::PersonalData],
        crypto_proof: None,
    };

    let json = report.to_json().unwrap();
    assert!(json.contains("test-123"));
    assert!(json.contains("PersonalData"));
}

#[test]
fn test_encrypted_field_serialization() {
    let crypto = PersonalDataCrypto::new();
    let dek = crypto.generate_dek("did:peer:alice").unwrap();

    let plaintext = b"test data";
    let encrypted = crypto.encrypt_field(&dek, plaintext).unwrap();

    // Serialize
    let json = serde_json::to_string(&encrypted).unwrap();

    // Deserialize
    let deserialized: vudo_privacy::crypto::EncryptedField =
        serde_json::from_str(&json).unwrap();

    // Decrypt deserialized field
    let decrypted = crypto.decrypt_field(&dek, &deserialized).unwrap();
    assert_eq!(decrypted, plaintext);
}
