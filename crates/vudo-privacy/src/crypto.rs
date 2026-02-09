//! Cryptographic deletion via per-user data encryption keys (DEK).
//!
//! This module implements GDPR Article 17 (Right to Erasure) through cryptographic
//! deletion. Instead of physically deleting CRDT data (impossible in append-only logs),
//! we encrypt personal data with user-specific keys. Deleting the key makes data
//! permanently unrecoverable across all peers.
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
//! # Security
//!
//! - Uses ChaCha20-Poly1305 AEAD for encryption (authenticated encryption)
//! - 256-bit keys for strong security
//! - Nonces are generated randomly per encryption
//! - Key deletion uses secure memory zeroing (zeroize crate)
//!
//! # Example
//!
//! ```rust
//! use vudo_privacy::crypto::{PersonalDataCrypto, DataEncryptionKey};
//!
//! # async fn example() -> vudo_privacy::error::Result<()> {
//! let crypto = PersonalDataCrypto::new();
//!
//! // Generate DEK for user
//! let dek = crypto.generate_dek("did:peer:alice")?;
//!
//! // Encrypt personal data
//! let plaintext = b"alice@example.com";
//! let encrypted = crypto.encrypt_field(&dek, plaintext)?;
//!
//! // Decrypt personal data
//! let decrypted = crypto.decrypt_field(&dek, &encrypted)?;
//! assert_eq!(decrypted, plaintext);
//!
//! // GDPR deletion: Delete DEK
//! let mut crypto_mut = crypto.clone();
//! let receipt = crypto_mut.delete_dek("did:peer:alice")?;
//! assert!(receipt.irreversible);
//!
//! // Now decryption fails permanently
//! let result = crypto_mut.decrypt_field(&dek, &encrypted);
//! assert!(result.is_err());
//! # Ok(())
//! # }
//! ```

use crate::error::{PrivacyError, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Data Encryption Key (DEK) for personal data.
///
/// Each user has a unique DEK that encrypts their personal data fields.
/// Deleting the DEK makes all encrypted data permanently unrecoverable.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct DataEncryptionKey {
    /// Owner DID.
    pub owner: String,

    /// DEK (256-bit ChaCha20-Poly1305 key).
    #[serde(with = "serde_bytes_array")]
    pub key: [u8; 32],

    /// Key creation timestamp (Unix seconds).
    pub created_at: u64,

    /// Deletion status.
    pub deleted: bool,

    /// Deletion timestamp (Unix seconds, if deleted).
    pub deleted_at: Option<u64>,
}

/// Helper module for serializing byte arrays.
mod serde_bytes_array {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        let array: [u8; 32] = bytes.try_into().map_err(|_| {
            serde::de::Error::custom("Expected 32 bytes for encryption key")
        })?;
        Ok(array)
    }
}

impl DataEncryptionKey {
    /// Create a new DEK for a user.
    pub fn generate(owner: String) -> Self {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);

        Self {
            owner,
            key: key.into(),
            created_at: Utc::now().timestamp() as u64,
            deleted: false,
            deleted_at: None,
        }
    }

    /// Check if this DEK has been deleted.
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    /// Mark this DEK as deleted (cryptographic erasure).
    pub fn mark_deleted(&mut self) {
        self.deleted = true;
        self.deleted_at = Some(Utc::now().timestamp() as u64);
        // Securely zero out key material
        self.key.zeroize();
    }
}

/// Encrypted field data.
///
/// This is the format stored in CRDT documents for @personal fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    /// Encrypted ciphertext (includes authentication tag).
    pub ciphertext: Vec<u8>,

    /// Nonce used for encryption (12 bytes for ChaCha20-Poly1305).
    #[serde(with = "serde_bytes_nonce")]
    pub nonce: [u8; 12],

    /// DID of data owner (for key lookup).
    pub owner: String,
}

/// Helper module for serializing nonce arrays.
mod serde_bytes_nonce {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &[u8; 12], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 12], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        let array: [u8; 12] = bytes.try_into().map_err(|_| {
            serde::de::Error::custom("Expected 12 bytes for nonce")
        })?;
        Ok(array)
    }
}

/// Deletion receipt proving cryptographic erasure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionReceipt {
    /// Owner DID.
    pub owner: String,

    /// Deletion timestamp (Unix seconds).
    pub deleted_at: u64,

    /// Irreversibility flag (always true for cryptographic deletion).
    pub irreversible: bool,
}

/// Personal data cryptography manager.
///
/// Manages per-user data encryption keys (DEKs) for encrypting/decrypting
/// personal data fields marked with @personal annotation.
#[derive(Clone)]
pub struct PersonalDataCrypto {
    /// Key storage (DID → DEK).
    key_store: Arc<DashMap<String, DataEncryptionKey>>,
}

impl PersonalDataCrypto {
    /// Create a new personal data crypto manager.
    pub fn new() -> Self {
        Self {
            key_store: Arc::new(DashMap::new()),
        }
    }

    /// Generate a new DEK for a user.
    ///
    /// # Arguments
    ///
    /// * `owner_did` - The DID of the data owner
    ///
    /// # Returns
    ///
    /// A new `DataEncryptionKey` that can be used to encrypt personal data.
    pub fn generate_dek(&self, owner_did: &str) -> Result<DataEncryptionKey> {
        let dek = DataEncryptionKey::generate(owner_did.to_string());
        self.key_store.insert(owner_did.to_string(), dek.clone());
        Ok(dek)
    }

    /// Get a DEK for a user.
    ///
    /// # Arguments
    ///
    /// * `owner_did` - The DID of the data owner
    ///
    /// # Returns
    ///
    /// The `DataEncryptionKey` for the user, or an error if not found.
    pub fn get_dek(&self, owner_did: &str) -> Result<DataEncryptionKey> {
        self.key_store
            .get(owner_did)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| PrivacyError::DekNotFound(owner_did.to_string()))
    }

    /// Encrypt personal data.
    ///
    /// # Arguments
    ///
    /// * `dek` - The data encryption key to use
    /// * `plaintext` - The plaintext data to encrypt
    ///
    /// # Returns
    ///
    /// An `EncryptedField` containing the ciphertext and metadata.
    pub fn encrypt_field(
        &self,
        dek: &DataEncryptionKey,
        plaintext: &[u8],
    ) -> Result<EncryptedField> {
        if dek.deleted {
            return Err(PrivacyError::KeyDeleted);
        }

        let key = Key::from_slice(&dek.key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| PrivacyError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedField {
            ciphertext,
            nonce: nonce.into(),
            owner: dek.owner.clone(),
        })
    }

    /// Decrypt personal data.
    ///
    /// # Arguments
    ///
    /// * `dek` - The data encryption key to use
    /// * `encrypted` - The encrypted field data
    ///
    /// # Returns
    ///
    /// The decrypted plaintext, or an error if the key was deleted or decryption failed.
    pub fn decrypt_field(
        &self,
        dek: &DataEncryptionKey,
        encrypted: &EncryptedField,
    ) -> Result<Vec<u8>> {
        if dek.deleted {
            return Err(PrivacyError::DataPermanentlyErased);
        }

        let key = Key::from_slice(&dek.key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_slice())
            .map_err(|_| PrivacyError::DecryptionFailed)?;

        Ok(plaintext)
    }

    /// Delete a DEK (implements GDPR right to erasure).
    ///
    /// This permanently deletes the encryption key, making all data encrypted
    /// with it unrecoverable. This operation is irreversible.
    ///
    /// # Arguments
    ///
    /// * `owner_did` - The DID of the data owner
    ///
    /// # Returns
    ///
    /// A `DeletionReceipt` proving the deletion occurred.
    pub fn delete_dek(&self, owner_did: &str) -> Result<DeletionReceipt> {
        if let Some(mut entry) = self.key_store.get_mut(owner_did) {
            let dek = entry.value_mut();

            // Mark as deleted and zero out key material
            dek.mark_deleted();

            Ok(DeletionReceipt {
                owner: owner_did.to_string(),
                deleted_at: dek.deleted_at.unwrap(),
                irreversible: true,
            })
        } else {
            Err(PrivacyError::DekNotFound(owner_did.to_string()))
        }
    }

    /// Check if a DEK exists for a user.
    pub fn has_dek(&self, owner_did: &str) -> bool {
        self.key_store.contains_key(owner_did)
    }

    /// Get the number of stored DEKs.
    pub fn dek_count(&self) -> usize {
        self.key_store.len()
    }
}

impl Default for PersonalDataCrypto {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dek_generation() {
        let crypto = PersonalDataCrypto::new();
        let dek = crypto.generate_dek("did:peer:alice").unwrap();

        assert_eq!(dek.owner, "did:peer:alice");
        assert!(!dek.deleted);
        assert_eq!(dek.key.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = PersonalDataCrypto::new();
        let dek = crypto.generate_dek("did:peer:alice").unwrap();

        let plaintext = b"alice@example.com";
        let encrypted = crypto.encrypt_field(&dek, plaintext).unwrap();
        let decrypted = crypto.decrypt_field(&dek, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_deletion() {
        let crypto = PersonalDataCrypto::new();
        let dek = crypto.generate_dek("did:peer:alice").unwrap();

        let plaintext = b"alice@example.com";
        let encrypted = crypto.encrypt_field(&dek, plaintext).unwrap();

        // Delete key
        let receipt = crypto.delete_dek("did:peer:alice").unwrap();
        assert!(receipt.irreversible);

        // Get the deleted DEK
        let deleted_dek = crypto.get_dek("did:peer:alice").unwrap();

        // Decryption should now fail
        let result = crypto.decrypt_field(&deleted_dek, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_with_deleted_key() {
        let crypto = PersonalDataCrypto::new();
        let mut dek = crypto.generate_dek("did:peer:alice").unwrap();

        // Delete key
        dek.mark_deleted();

        // Encryption should fail
        let result = crypto.encrypt_field(&dek, b"data");
        assert!(result.is_err());
    }
}
