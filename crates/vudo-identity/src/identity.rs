//! Identity management with master keys, device linking, key rotation, and revocation
//!
//! This module provides hierarchical identity management where a master identity
//! can link multiple device keys, each with their own UCANs.
//!
//! # Examples
//!
//! ```
//! use vudo_identity::{MasterIdentity, DeviceIdentity};
//! use ed25519_dalek::SigningKey;
//! use rand::rngs::OsRng;
//!
//! # async fn example() -> vudo_identity::error::Result<()> {
//! // Create master identity
//! let master = MasterIdentity::generate("Alice's Master Identity").await?;
//!
//! // Create device identity
//! let device = DeviceIdentity::generate("Alice's Phone").await?;
//!
//! // Link device to master
//! let mut master_clone = master.clone();
//! let link = master_clone.link_device(
//!     device.device_name().to_string(),
//!     device.did().clone(),
//!     &master.signing_key(),
//! ).await?;
//!
//! println!("Device linked with UCAN: {}", link.authorization.encode()?);
//! # Ok(())
//! # }
//! ```

use crate::did::Did;
use crate::error::{Error, Result};
use crate::ucan::{Capability, Ucan};
use chrono::Utc;
use ed25519_dalek::{Signature, SigningKey, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

/// Master identity (kept offline/cold storage)
#[derive(Clone, Serialize, Deserialize)]
pub struct MasterIdentity {
    /// Master DID
    pub did: Did,

    /// Display name
    pub name: String,

    /// Master signing key (Ed25519, kept offline)
    #[serde(with = "signing_key_serde")]
    master_key: SigningKey,

    /// Master encryption key (X25519, for key agreement)
    #[serde(with = "static_secret_serde")]
    encryption_key: StaticSecret,

    /// Linked devices
    pub devices: Vec<DeviceLink>,

    /// Revocation list
    pub revocations: RevocationList,

    /// Key rotations
    pub rotations: Vec<KeyRotation>,
}

impl std::fmt::Debug for MasterIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MasterIdentity")
            .field("did", &self.did)
            .field("name", &self.name)
            .field("devices", &self.devices)
            .field("revocations", &self.revocations)
            .field("rotations", &self.rotations)
            .finish_non_exhaustive()
    }
}

impl MasterIdentity {
    /// Generate a new master identity
    pub async fn generate(name: impl Into<String>) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_key = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = X25519PublicKey::from(&encryption_key);

        let did = Did::from_keys(signing_key.verifying_key(), &encryption_public)?;

        Ok(Self {
            did: did.clone(),
            name: name.into(),
            master_key: signing_key,
            encryption_key,
            devices: Vec::new(),
            revocations: RevocationList::new(did),
            rotations: Vec::new(),
        })
    }

    /// Get signing key
    pub fn signing_key(&self) -> SigningKey {
        self.master_key.clone()
    }

    /// Link a new device (offline operation using master key)
    pub async fn link_device(
        &mut self,
        device_name: String,
        device_did: Did,
        master_key: &SigningKey,
    ) -> Result<DeviceLink> {
        // Check if device already linked
        if self
            .devices
            .iter()
            .any(|d| d.device_did == device_did && !d.revoked)
        {
            return Err(Error::DeviceAlreadyLinked(device_name));
        }

        // Create UCAN granting device full capabilities
        let ucan = Ucan::new(
            self.did.clone(),
            device_did.clone(),
            vec![Capability::wildcard("vudo://")],
            Utc::now().timestamp() as u64 + (365 * 24 * 60 * 60), // 1 year
            None,
            Some(Self::random_nonce()),
            vec![],
        )
        .sign(master_key)?;

        let link = DeviceLink {
            device_did,
            device_name,
            authorization: ucan,
            linked_at: Utc::now().timestamp() as u64,
            revoked: false,
        };

        self.devices.push(link.clone());
        Ok(link)
    }

    /// Revoke a device
    pub async fn revoke_device(
        &mut self,
        device_did: &Did,
        reason: Option<String>,
        master_key: &SigningKey,
    ) -> Result<()> {
        // Find and revoke device
        let device = self
            .devices
            .iter_mut()
            .find(|d| &d.device_did == device_did)
            .ok_or_else(|| Error::DeviceNotFound(device_did.to_string()))?;

        device.revoked = true;

        // Add to revocation list
        self.revocations
            .revoke(device_did.to_string(), reason, master_key)?;

        Ok(())
    }

    /// Check if a device is revoked
    pub fn is_device_revoked(&self, device_did: &Did) -> bool {
        self.revocations.is_revoked(&device_did.to_string())
    }

    /// Rotate master key
    pub async fn rotate_key(
        &mut self,
        new_key: SigningKey,
        new_encryption_key: StaticSecret,
    ) -> Result<KeyRotation> {
        let old_key = self.master_key.clone();
        let new_encryption_public = X25519PublicKey::from(&new_encryption_key);
        let new_did = Did::from_keys(new_key.verifying_key(), &new_encryption_public)?;

        let rotation = KeyRotation::create(&old_key, &new_key, &self.did, &new_did)?;

        // Update identity
        self.master_key = new_key;
        self.encryption_key = new_encryption_key;
        self.did = new_did;
        self.rotations.push(rotation.clone());

        Ok(rotation)
    }

    fn random_nonce() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let nonce: [u8; 16] = rng.gen();
        hex::encode(&nonce)
    }
}

/// Device identity (used for day-to-day operations)
#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// Device DID
    pub did: Did,

    /// Device name
    pub name: String,

    /// Device signing key
    #[serde(with = "signing_key_serde")]
    signing_key: SigningKey,

    /// Device encryption key
    #[serde(with = "static_secret_serde")]
    encryption_key: StaticSecret,

    /// Master identity DID (if linked)
    pub master_did: Option<Did>,

    /// Device authorization UCAN (if linked)
    pub authorization: Option<Ucan>,
}

impl std::fmt::Debug for DeviceIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceIdentity")
            .field("did", &self.did)
            .field("name", &self.name)
            .field("master_did", &self.master_did)
            .field("authorization", &self.authorization)
            .finish_non_exhaustive()
    }
}

impl DeviceIdentity {
    /// Generate a new device identity
    pub async fn generate(name: impl Into<String>) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_key = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = X25519PublicKey::from(&encryption_key);

        let did = Did::from_keys(signing_key.verifying_key(), &encryption_public)?;

        Ok(Self {
            did,
            name: name.into(),
            signing_key,
            encryption_key,
            master_did: None,
            authorization: None,
        })
    }

    /// Get DID
    pub fn did(&self) -> &Did {
        &self.did
    }

    /// Get device name
    pub fn device_name(&self) -> &str {
        &self.name
    }

    /// Get signing key
    pub fn signing_key(&self) -> SigningKey {
        self.signing_key.clone()
    }

    /// Link to master identity
    pub fn link_to_master(&mut self, master_did: Did, authorization: Ucan) {
        self.master_did = Some(master_did);
        self.authorization = Some(authorization);
    }

    /// Check if linked to master
    pub fn is_linked(&self) -> bool {
        self.master_did.is_some() && self.authorization.is_some()
    }

    /// Verify authorization is still valid
    pub fn verify_authorization(&self) -> Result<()> {
        if let Some(ref ucan) = self.authorization {
            ucan.verify()?;
        }
        Ok(())
    }
}

/// Device link record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLink {
    /// Device DID
    pub device_did: Did,

    /// Device name ("Alice's Phone")
    pub device_name: String,

    /// UCAN granting device full capabilities
    pub authorization: Ucan,

    /// Link timestamp
    pub linked_at: u64,

    /// Revocation status
    pub revoked: bool,
}

/// Key rotation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotation {
    /// Old DID (being rotated out)
    pub old_did: Did,

    /// New DID (rotated in)
    pub new_did: Did,

    /// Rotation timestamp
    pub rotated_at: u64,

    /// Grace period (old key still valid for this duration, in seconds)
    pub grace_period: u64,

    /// Rotation certificate (signed by both keys)
    pub certificate: RotationCertificate,
}

impl KeyRotation {
    /// Create rotation certificate (requires both old and new keys)
    pub fn create(
        old_key: &SigningKey,
        new_key: &SigningKey,
        old_did: &Did,
        new_did: &Did,
    ) -> Result<Self> {
        let timestamp = Utc::now().timestamp() as u64;
        let message = format!("{}|{}|{}", old_did, new_did, timestamp);

        let old_sig = old_key.sign(message.as_bytes());
        let new_sig = new_key.sign(message.as_bytes());

        Ok(Self {
            old_did: old_did.clone(),
            new_did: new_did.clone(),
            rotated_at: timestamp,
            grace_period: 7 * 24 * 60 * 60, // 7 days
            certificate: RotationCertificate {
                old_did: old_did.clone(),
                new_did: new_did.clone(),
                timestamp,
                old_key_signature: old_sig.to_bytes().to_vec(),
                new_key_signature: new_sig.to_bytes().to_vec(),
            },
        })
    }

    /// Check if rotation is still in grace period
    pub fn in_grace_period(&self) -> bool {
        let now = Utc::now().timestamp() as u64;
        now < self.rotated_at + self.grace_period
    }

    /// Verify rotation certificate
    pub fn verify(&self) -> Result<()> {
        let message = format!("{}|{}|{}", self.old_did, self.new_did, self.certificate.timestamp);

        // Verify old key signature
        let old_sig = Signature::from_bytes(
            self.certificate.old_key_signature.as_slice().try_into()
                .map_err(|_| Error::SignatureVerification("Invalid old key signature length".to_string()))?
        );
        self.old_did
            .verification_key
            .verify(message.as_bytes(), &old_sig)?;

        // Verify new key signature
        let new_sig = Signature::from_bytes(
            self.certificate.new_key_signature.as_slice().try_into()
                .map_err(|_| Error::SignatureVerification("Invalid new key signature length".to_string()))?
        );
        self.new_did
            .verification_key
            .verify(message.as_bytes(), &new_sig)?;

        Ok(())
    }
}

/// Rotation certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationCertificate {
    /// Old DID
    pub old_did: Did,

    /// New DID
    pub new_did: Did,

    /// Rotation timestamp
    pub timestamp: u64,

    /// Signature by old key (proving ownership)
    pub old_key_signature: Vec<u8>,

    /// Signature by new key (proving possession)
    pub new_key_signature: Vec<u8>,
}

/// Revocation list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationList {
    /// Issuer of the revocation list
    pub issuer: Did,

    /// Revoked DIDs/UCANs
    pub revocations: Vec<Revocation>,

    /// Version (increments on each update)
    pub version: u64,

    /// Timestamp of last update
    pub updated_at: u64,

    /// Signature over canonical representation
    pub signature: Option<Vec<u8>>,
}

impl RevocationList {
    /// Create a new revocation list
    pub fn new(issuer: Did) -> Self {
        Self {
            issuer,
            revocations: Vec::new(),
            version: 0,
            updated_at: Utc::now().timestamp() as u64,
            signature: None,
        }
    }

    /// Add revocation (must have master key to sign)
    pub fn revoke(
        &mut self,
        subject: String,
        reason: Option<String>,
        master_key: &SigningKey,
    ) -> Result<()> {
        let revocation = Revocation {
            subject,
            reason,
            revoked_at: Utc::now().timestamp() as u64,
        };

        self.revocations.push(revocation);
        self.version += 1;
        self.updated_at = Utc::now().timestamp() as u64;

        // Re-sign
        let canonical = self.canonical_representation()?;
        let signature = master_key.sign(&canonical);
        self.signature = Some(signature.to_bytes().to_vec());

        Ok(())
    }

    /// Check if DID/UCAN is revoked
    pub fn is_revoked(&self, subject: &str) -> bool {
        self.revocations.iter().any(|r| r.subject == subject)
    }

    /// Verify revocation list signature
    pub fn verify(&self) -> Result<()> {
        let sig_bytes = self
            .signature
            .as_ref()
            .ok_or_else(|| Error::Revocation("Revocation list not signed".to_string()))?;

        let signature = Signature::from_bytes(
            sig_bytes.as_slice().try_into()
                .map_err(|_| Error::SignatureVerification("Invalid signature length".to_string()))?
        );

        let canonical = self.canonical_representation()?;
        self.issuer
            .verification_key
            .verify(&canonical, &signature)?;

        Ok(())
    }

    /// Create canonical representation for signing
    fn canonical_representation(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        data.extend_from_slice(self.issuer.as_str().as_bytes());
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&self.updated_at.to_le_bytes());

        for revocation in &self.revocations {
            data.extend_from_slice(revocation.subject.as_bytes());
            data.extend_from_slice(&revocation.revoked_at.to_le_bytes());
        }

        Ok(data)
    }
}

/// Revocation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revocation {
    /// DID or UCAN ID being revoked
    pub subject: String,

    /// Reason (optional)
    pub reason: Option<String>,

    /// Revocation timestamp
    pub revoked_at: u64,
}

// Serde helpers for cryptographic keys
mod signing_key_serde {
    use ed25519_dalek::SigningKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(key: &SigningKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        key.to_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SigningKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        Ok(SigningKey::from_bytes(&bytes))
    }
}

mod static_secret_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use x25519_dalek::StaticSecret;

    pub fn serialize<S>(secret: &StaticSecret, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        secret.to_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StaticSecret, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        Ok(StaticSecret::from(bytes))
    }
}

mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_master_identity_creation() {
        let master = MasterIdentity::generate("Alice").await.unwrap();
        assert_eq!(master.name, "Alice");
        assert!(master.devices.is_empty());
    }

    #[tokio::test]
    async fn test_device_linking() {
        let mut master = MasterIdentity::generate("Alice").await.unwrap();
        let device = DeviceIdentity::generate("Alice's Phone").await.unwrap();

        let master_key = master.signing_key();
        let link = master
            .link_device("Alice's Phone".to_string(), device.did.clone(), &master_key)
            .await
            .unwrap();

        assert_eq!(link.device_name, "Alice's Phone");
        assert!(!link.revoked);
        assert_eq!(master.devices.len(), 1);
    }

    #[tokio::test]
    async fn test_device_revocation() {
        let mut master = MasterIdentity::generate("Alice").await.unwrap();
        let device = DeviceIdentity::generate("Alice's Phone").await.unwrap();

        let master_key = master.signing_key();
        master
            .link_device("Alice's Phone".to_string(), device.did.clone(), &master_key)
            .await
            .unwrap();

        master
            .revoke_device(&device.did, Some("Lost device".to_string()), &master_key)
            .await
            .unwrap();

        assert!(master.is_device_revoked(&device.did));
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let mut master = MasterIdentity::generate("Alice").await.unwrap();

        let new_key = SigningKey::generate(&mut OsRng);
        let new_encryption = StaticSecret::random_from_rng(&mut OsRng);

        let rotation = master.rotate_key(new_key, new_encryption).await.unwrap();

        assert!(rotation.verify().is_ok());
        assert!(rotation.in_grace_period());
    }

    #[tokio::test]
    async fn test_revocation_list() {
        let (did, key) = {
            let signing_key = SigningKey::generate(&mut OsRng);
            let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
            let encryption_public = X25519PublicKey::from(&encryption_secret);
            let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();
            (did, signing_key)
        };

        let mut revocation_list = RevocationList::new(did.clone());

        revocation_list
            .revoke("did:peer:abc123".to_string(), Some("Test".to_string()), &key)
            .unwrap();

        assert!(revocation_list.is_revoked("did:peer:abc123"));
        assert!(!revocation_list.is_revoked("did:peer:xyz789"));
        assert!(revocation_list.verify().is_ok());
    }
}
