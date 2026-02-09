//! DID (Decentralized Identifier) implementation with did:peer:2 support
//!
//! This module implements the did:peer:2 method specification with Ed25519 authentication
//! keys and X25519 key agreement keys.
//!
//! # Format
//!
//! ```text
//! did:peer:2.Ez<base58btc of Ed25519 pubkey>.S<base58btc of X25519 pubkey>
//! ```
//!
//! # Examples
//!
//! ```
//! use vudo_identity::Did;
//! use ed25519_dalek::SigningKey;
//! use x25519_dalek::{StaticSecret, PublicKey};
//! use rand::rngs::OsRng;
//!
//! // Create DID from keypair
//! let signing_key = SigningKey::generate(&mut OsRng);
//! let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
//! let encryption_public = PublicKey::from(&encryption_secret);
//!
//! let did = Did::from_keys(
//!     signing_key.verifying_key(),
//!     &encryption_public,
//! ).unwrap();
//!
//! println!("DID: {}", did);
//! ```

use crate::error::{Error, Result};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use std::fmt;
use x25519_dalek::PublicKey as X25519PublicKey;

/// DID (Decentralized Identifier)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did {
    /// Full DID string
    did: String,

    /// Ed25519 verification key (authentication)
    #[serde(with = "verifying_key_serde")]
    pub verification_key: VerifyingKey,

    /// X25519 public key (key agreement)
    #[serde(with = "x25519_pubkey_serde")]
    pub encryption_key: X25519PublicKey,
}

impl Did {
    /// Create a DID from Ed25519 and X25519 public keys
    pub fn from_keys(
        verification_key: VerifyingKey,
        encryption_key: &X25519PublicKey,
    ) -> Result<Self> {
        // Encode Ed25519 key with multicodec prefix (0xed01 for Ed25519 public key)
        let mut ed_bytes = vec![0xed, 0x01];
        ed_bytes.extend_from_slice(verification_key.as_bytes());

        // Encode X25519 key with multicodec prefix (0xec01 for X25519 public key)
        let mut x25519_bytes = vec![0xec, 0x01];
        x25519_bytes.extend_from_slice(encryption_key.as_bytes());

        // Base58btc encode (multibase prefix 'z')
        let ed_encoded = format!("z{}", bs58::encode(&ed_bytes).into_string());
        let x25519_encoded = format!("z{}", bs58::encode(&x25519_bytes).into_string());

        // Construct did:peer:2
        let did = format!("did:peer:2.Ez{}.S{}",
            &ed_encoded[1..],  // Remove 'z' prefix
            &x25519_encoded[1..]  // Remove 'z' prefix
        );

        Ok(Self {
            did,
            verification_key,
            encryption_key: *encryption_key,
        })
    }

    /// Parse a did:peer:2 string
    pub fn parse(did_str: &str) -> Result<Self> {
        // Format: did:peer:2.Ez<ed25519>.S<x25519>
        if !did_str.starts_with("did:peer:2.") {
            return Err(Error::Did(format!("Invalid did:peer:2 format: {}", did_str)));
        }

        let rest = &did_str["did:peer:2.".len()..];
        let parts: Vec<&str> = rest.split('.').collect();

        if parts.len() != 2 {
            return Err(Error::Did(format!("Expected 2 key parts, got {}", parts.len())));
        }

        // Parse Ed25519 key (starts with Ez)
        if !parts[0].starts_with("Ez") {
            return Err(Error::Did("Ed25519 key must start with Ez".to_string()));
        }
        let ed_encoded = format!("z{}", &parts[0][2..]);
        let ed_bytes = bs58::decode(&ed_encoded[1..])
            .into_vec()
            .map_err(|e| Error::Encoding(format!("Failed to decode Ed25519 key: {}", e)))?;

        // Check multicodec prefix
        if ed_bytes.len() < 34 || ed_bytes[0] != 0xed || ed_bytes[1] != 0x01 {
            return Err(Error::Did("Invalid Ed25519 multicodec prefix".to_string()));
        }

        let verification_key = VerifyingKey::from_bytes(
            ed_bytes[2..34]
                .try_into()
                .map_err(|_| Error::Did("Invalid Ed25519 key length".to_string()))?,
        )
        .map_err(|e| Error::Did(format!("Invalid Ed25519 key: {}", e)))?;

        // Parse X25519 key (starts with S)
        if !parts[1].starts_with('S') {
            return Err(Error::Did("X25519 key must start with S".to_string()));
        }
        let x25519_encoded = format!("z{}", &parts[1][1..]);
        let x25519_bytes = bs58::decode(&x25519_encoded[1..])
            .into_vec()
            .map_err(|e| Error::Encoding(format!("Failed to decode X25519 key: {}", e)))?;

        // Check multicodec prefix
        if x25519_bytes.len() < 34 || x25519_bytes[0] != 0xec || x25519_bytes[1] != 0x01 {
            return Err(Error::Did("Invalid X25519 multicodec prefix".to_string()));
        }

        let encryption_key: [u8; 32] = x25519_bytes[2..34]
            .try_into()
            .map_err(|_| Error::Did("Invalid X25519 key length".to_string()))?;

        Ok(Self {
            did: did_str.to_string(),
            verification_key,
            encryption_key: X25519PublicKey::from(encryption_key),
        })
    }

    /// Get DID method ("peer")
    pub fn method(&self) -> &str {
        "peer"
    }

    /// Get full DID string
    pub fn as_str(&self) -> &str {
        &self.did
    }

    /// Generate DID document
    pub fn to_document(&self) -> DidDocument {
        let mut ed_bytes = vec![0xed, 0x01];
        ed_bytes.extend_from_slice(self.verification_key.as_bytes());
        let ed_encoded = format!(
            "z{}",
            bs58::encode(&ed_bytes).into_string()
        );

        let mut x25519_bytes = vec![0xec, 0x01];
        x25519_bytes.extend_from_slice(self.encryption_key.as_bytes());
        let x25519_encoded = format!(
            "z{}",
            bs58::encode(&x25519_bytes).into_string()
        );

        DidDocument {
            context: "https://www.w3.org/ns/did/v1".to_string(),
            id: self.did.clone(),
            authentication: vec![VerificationMethod {
                id: format!("{}#key-1", self.did),
                method_type: "Ed25519VerificationKey2020".to_string(),
                controller: self.did.clone(),
                public_key_multibase: ed_encoded,
            }],
            key_agreement: vec![VerificationMethod {
                id: format!("{}#key-2", self.did),
                method_type: "X25519KeyAgreementKey2020".to_string(),
                controller: self.did.clone(),
                public_key_multibase: x25519_encoded,
            }],
        }
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.did)
    }
}

/// DID Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: String,
    pub id: String,
    pub authentication: Vec<VerificationMethod>,
    #[serde(rename = "keyAgreement")]
    pub key_agreement: Vec<VerificationMethod>,
}

/// Verification method in DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub method_type: String,
    pub controller: String,
    #[serde(rename = "publicKeyMultibase")]
    pub public_key_multibase: String,
}

// Serde helpers for cryptographic keys
mod verifying_key_serde {
    use ed25519_dalek::VerifyingKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(key: &VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        key.as_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        VerifyingKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

mod x25519_pubkey_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use x25519_dalek::PublicKey;

    pub fn serialize<S>(key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        key.as_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        Ok(PublicKey::from(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use x25519_dalek::StaticSecret;

    #[test]
    fn test_did_creation() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = X25519PublicKey::from(&encryption_secret);

        let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();

        assert!(did.as_str().starts_with("did:peer:2.Ez"));
        assert!(did.as_str().contains(".S"));
    }

    #[test]
    fn test_did_roundtrip() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = X25519PublicKey::from(&encryption_secret);

        let did1 = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();
        let did_str = did1.as_str();
        let did2 = Did::parse(did_str).unwrap();

        assert_eq!(did1, did2);
    }

    #[test]
    fn test_did_document_generation() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = X25519PublicKey::from(&encryption_secret);

        let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();
        let doc = did.to_document();

        assert_eq!(doc.context, "https://www.w3.org/ns/did/v1");
        assert_eq!(doc.id, did.as_str());
        assert_eq!(doc.authentication.len(), 1);
        assert_eq!(doc.key_agreement.len(), 1);
        assert_eq!(doc.authentication[0].method_type, "Ed25519VerificationKey2020");
        assert_eq!(doc.key_agreement[0].method_type, "X25519KeyAgreementKey2020");
    }

    #[test]
    fn test_did_method() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = X25519PublicKey::from(&encryption_secret);

        let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();

        assert_eq!(did.method(), "peer");
    }
}
