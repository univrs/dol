//! VUDO Identity - Decentralized Identity System
//!
//! This crate provides a decentralized identity system for VUDO Runtime with:
//! - **Peer DIDs (did:peer:2)**: For pairwise node authentication
//! - **UCANs**: User Controlled Authorization Networks for capability delegation
//! - **Ed25519 keypairs**: For digital signatures
//! - **Master → Device linking**: Hierarchical identity management
//! - **Key rotation**: With grace periods and revocation lists
//! - **DID resolution**: For P2P peer verification
//!
//! # Architecture
//!
//! The identity system follows a hierarchical model:
//!
//! ```text
//! Master Identity (Cold storage, offline)
//!   ├── Device Key 1 (Phone) ─→ UCAN
//!   ├── Device Key 2 (Laptop) ─→ UCAN
//!   └── Device Key 3 (Tablet) ─→ UCAN
//! ```
//!
//! Each device receives a UCAN from the master identity, granting it capabilities.
//! Devices can further delegate capabilities to apps or services.
//!
//! # Examples
//!
//! ## Creating a Master Identity
//!
//! ```
//! use vudo_identity::MasterIdentity;
//!
//! # async fn example() -> vudo_identity::error::Result<()> {
//! // Create master identity (keep offline!)
//! let master = MasterIdentity::generate("Alice").await?;
//! println!("Master DID: {}", master.did);
//! # Ok(())
//! # }
//! ```
//!
//! ## Linking a Device
//!
//! ```
//! use vudo_identity::{MasterIdentity, DeviceIdentity};
//!
//! # async fn example() -> vudo_identity::error::Result<()> {
//! let mut master = MasterIdentity::generate("Alice").await?;
//! let device = DeviceIdentity::generate("Alice's Phone").await?;
//!
//! // Link device to master (requires master key)
//! let master_key = master.signing_key();
//! let link = master.link_device(
//!     "Alice's Phone".to_string(),
//!     device.did().clone(),
//!     &master_key,
//! ).await?;
//!
//! println!("Device linked with authorization: {}", link.authorization.encode()?);
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating and Verifying UCANs
//!
//! ```
//! use vudo_identity::{Did, Ucan, Capability};
//! use ed25519_dalek::SigningKey;
//! use x25519_dalek::{StaticSecret, PublicKey};
//! use rand::rngs::OsRng;
//! use chrono::Utc;
//!
//! # fn example() -> vudo_identity::error::Result<()> {
//! // Create DIDs
//! let issuer_key = SigningKey::generate(&mut OsRng);
//! let issuer_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
//! let issuer_did = Did::from_keys(issuer_key.verifying_key(), &issuer_enc)?;
//!
//! let audience_key = SigningKey::generate(&mut OsRng);
//! let audience_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
//! let audience_did = Did::from_keys(audience_key.verifying_key(), &audience_enc)?;
//!
//! // Create UCAN
//! let ucan = Ucan::new(
//!     issuer_did,
//!     audience_did,
//!     vec![Capability::new("vudo://myapp/*", "read")],
//!     Utc::now().timestamp() as u64 + 3600, // 1 hour
//!     None,
//!     None,
//!     vec![],
//! ).sign(&issuer_key)?;
//!
//! // Verify UCAN
//! ucan.verify()?;
//! println!("UCAN verified successfully");
//! # Ok(())
//! # }
//! ```
//!
//! ## Key Rotation
//!
//! ```
//! use vudo_identity::MasterIdentity;
//! use ed25519_dalek::SigningKey;
//! use x25519_dalek::StaticSecret;
//! use rand::rngs::OsRng;
//!
//! # async fn example() -> vudo_identity::error::Result<()> {
//! let mut master = MasterIdentity::generate("Alice").await?;
//!
//! // Generate new keys
//! let new_key = SigningKey::generate(&mut OsRng);
//! let new_encryption = StaticSecret::random_from_rng(&mut OsRng);
//!
//! // Rotate keys
//! let rotation = master.rotate_key(new_key, new_encryption).await?;
//!
//! // Verify rotation
//! rotation.verify()?;
//! println!("Key rotated, grace period active: {}", rotation.in_grace_period());
//! # Ok(())
//! # }
//! ```
//!
//! ## DID Resolution
//!
//! ```
//! use vudo_identity::{DidResolver, Did};
//! use ed25519_dalek::SigningKey;
//! use x25519_dalek::{StaticSecret, PublicKey};
//! use rand::rngs::OsRng;
//!
//! # async fn example() -> vudo_identity::error::Result<()> {
//! let resolver = DidResolver::new();
//!
//! // Create DID
//! let signing_key = SigningKey::generate(&mut OsRng);
//! let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
//! let encryption_public = PublicKey::from(&encryption_secret);
//! let did = Did::from_keys(signing_key.verifying_key(), &encryption_public)?;
//!
//! // Resolve DID to document
//! let doc = resolver.resolve(&did).await?;
//! println!("Resolved DID: {}", doc.id);
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - **Peer DID creation**: < 50ms (local key generation)
//! - **UCAN delegation verification**: < 10ms (Ed25519 signature check)
//! - **Key rotation**: Preserves existing sync relationships
//! - **Revocation propagation**: Within 1 sync cycle
//!
//! # Security Considerations
//!
//! - Master keys should be kept offline in cold storage
//! - Device keys are used for daily operations
//! - UCANs have expiration times to limit exposure
//! - Revocation lists are synced via P2P gossip
//! - Key rotation includes grace periods to prevent service disruption
//!
//! # References
//!
//! - [did:peer spec](https://identity.foundation/peer-did-method-spec/)
//! - [UCAN spec](https://ucan.xyz/)
//! - [DID Core](https://www.w3.org/TR/did-core/)

pub mod did;
pub mod error;
pub mod identity;
pub mod resolver;
pub mod ucan;

// Re-export main types
pub use did::{Did, DidDocument, VerificationMethod};
pub use error::{Error, Result};
pub use identity::{
    DeviceIdentity, DeviceLink, KeyRotation, MasterIdentity, Revocation, RevocationList,
    RotationCertificate,
};
pub use resolver::{BatchDidResolver, DidResolver};
pub use ucan::{Capability, Ucan};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
