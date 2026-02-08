//! VUDO Privacy - GDPR-compliant cryptographic deletion for local-first CRDTs.
//!
//! This crate provides GDPR Article 17 (Right to Erasure) compliance for VUDO Runtime
//! through cryptographic deletion. Instead of physically deleting CRDT data (impossible
//! in append-only logs), we encrypt personal data with user-specific keys and delete
//! the key to make data permanently unrecoverable.
//!
//! # Features
//!
//! - **Cryptographic Deletion**: Per-user encryption keys (DEKs) for personal data
//! - **@personal Annotation**: DOL annotation for GDPR-sensitive fields
//! - **Pseudonymous Actor IDs**: Privacy-preserving CRDT metadata
//! - **Audit Trail**: Comprehensive logging for compliance
//! - **Willow Integration**: True-deletion for non-personal data
//!
//! # Architecture
//!
//! ```text
//! Personal Data Flow:
//! 1. User creates document → Generate per-user DEK
//! 2. Encrypt @personal fields with DEK
//! 3. Store encrypted data in CRDT (Automerge)
//! 4. GDPR deletion request → Delete DEK
//! 5. Encrypted data becomes permanently unrecoverable
//! ```
//!
//! # Example
//!
//! ```rust
//! use vudo_privacy::{
//!     crypto::{PersonalDataCrypto, DataEncryptionKey},
//!     gdpr::{GdprComplianceEngine, DeletionRequest},
//!     pseudonymous::PseudonymousActorId,
//! };
//!
//! # async fn example() -> vudo_privacy::error::Result<()> {
//! // 1. Personal data encryption
//! let crypto = PersonalDataCrypto::new();
//! let dek = crypto.generate_dek("did:peer:alice")?;
//!
//! let plaintext = b"alice@example.com";
//! let encrypted = crypto.encrypt_field(&dek, plaintext)?;
//!
//! // 2. Pseudonymous actor IDs for CRDT ops
//! let pseudo = PseudonymousActorId::from_did("did:peer:alice")?;
//! // Use pseudo.actor_id() in Automerge documents
//!
//! // 3. GDPR deletion
//! let engine = GdprComplianceEngine::new()?;
//! let request = DeletionRequest::personal_only("app.example".to_string());
//! let report = engine.execute_deletion("did:peer:alice", request).await?;
//!
//! assert!(report.irreversible);
//! assert!(report.crypto_proof.is_some());
//! # Ok(())
//! # }
//! ```
//!
//! # DOL Integration
//!
//! Mark GDPR-sensitive fields with `@personal`:
//!
//! ```dol
//! gen UserProfile {
//!   @crdt(immutable) has id: String
//!
//!   @crdt(lww) @personal has email: String
//!   @crdt(lww) @personal has full_name: String
//!
//!   @crdt(lww) has username: String  // Public
//! }
//! ```
//!
//! Generated Rust code will automatically encrypt `@personal` fields with DEKs.
//!
//! # GDPR Compliance
//!
//! This crate implements GDPR Article 17 (Right to Erasure):
//!
//! - **Cryptographic Deletion**: DEK deletion makes data unrecoverable
//! - **Audit Trail**: Immutable record of all deletions
//! - **Proof**: Deletion receipts for regulatory compliance
//! - **Irreversibility**: Cannot recover data after key deletion
//!
//! # Security Considerations
//!
//! - Uses ChaCha20-Poly1305 AEAD for encryption
//! - 256-bit keys for strong security
//! - Secure memory zeroing (zeroize crate)
//! - Pseudonymous actor IDs prevent identity leakage
//!
//! # References
//!
//! - [GDPR Article 17](https://gdpr-info.eu/art-17-gdpr/)
//! - [Cryptographic Deletion in CRDTs](https://arxiv.org/abs/2103.13108)
//! - [VUDO Privacy Design](docs/compliance/gdpr-local-first.md)

pub mod audit;
pub mod crypto;
pub mod error;
pub mod gdpr;
pub mod pseudonymous;

// Re-export main types
pub use audit::{DataCategory, DeletionAuditLog, DeletionLogEntry, DeletionMethod};
pub use crypto::{DataEncryptionKey, DeletionReceipt, EncryptedField, PersonalDataCrypto};
pub use error::{PrivacyError, Result};
pub use gdpr::{DeletionReport, DeletionRequest, DeletionStats, GdprComplianceEngine};
pub use pseudonymous::{ActorIdMapper, PseudonymousActorId};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_end_to_end_gdpr_deletion() {
        // 1. Create GDPR engine
        let engine = GdprComplianceEngine::new().unwrap();

        // 2. Generate DEK for user using engine's crypto
        let crypto = engine.crypto();
        let dek = crypto.generate_dek("did:peer:alice").unwrap();

        // 3. Encrypt personal data
        let email = b"alice@example.com";
        let encrypted_email = crypto.encrypt_field(&dek, email).unwrap();

        // 4. Verify decryption works
        let decrypted = crypto.decrypt_field(&dek, &encrypted_email).unwrap();
        assert_eq!(decrypted, email);

        // 5. Execute GDPR deletion
        let request = DeletionRequest::personal_only("app.example".to_string());
        let report = engine.execute_deletion("did:peer:alice", request).await.unwrap();

        // 6. Verify deletion
        assert!(report.irreversible);
        assert!(report.crypto_proof.is_some());

        // 7. Get updated DEK and verify decryption now fails
        let deleted_dek = crypto.get_dek("did:peer:alice").unwrap();
        let result = crypto.decrypt_field(&deleted_dek, &encrypted_email);
        assert!(result.is_err());
    }

    #[test]
    fn test_pseudonymous_actor_id() {
        let pseudo = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        assert_eq!(pseudo.real_did(), "did:peer:alice");

        // Pseudonym should be deterministic
        let pseudo2 = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        assert_eq!(pseudo.actor_id(), pseudo2.actor_id());
    }
}
