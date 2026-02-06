//! UCAN (User Controlled Authorization Network) implementation
//!
//! UCANs provide capability-based authorization with delegation chains.
//! They are encoded as JWTs and can be chained together to delegate capabilities.
//!
//! # Examples
//!
//! ```
//! use vudo_identity::{Ucan, Capability, Did};
//! use ed25519_dalek::SigningKey;
//! use x25519_dalek::{StaticSecret, PublicKey};
//! use rand::rngs::OsRng;
//! use chrono::Utc;
//!
//! # fn example() -> vudo_identity::error::Result<()> {
//! // Create issuer and audience DIDs
//! let issuer_key = SigningKey::generate(&mut OsRng);
//! let issuer_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
//! let issuer_did = Did::from_keys(issuer_key.verifying_key(), &issuer_enc)?;
//!
//! let audience_key = SigningKey::generate(&mut OsRng);
//! let audience_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
//! let audience_did = Did::from_keys(audience_key.verifying_key(), &audience_enc)?;
//!
//! // Create root UCAN
//! let capability = Capability::new("vudo://myapp/*", "read");
//! let ucan = Ucan::new(
//!     issuer_did.clone(),
//!     audience_did.clone(),
//!     vec![capability],
//!     Utc::now().timestamp() as u64 + 3600, // 1 hour
//!     None,
//!     None,
//!     vec![],
//! );
//!
//! // Sign UCAN
//! let signed = ucan.sign(&issuer_key)?;
//!
//! // Verify UCAN
//! signed.verify()?;
//! # Ok(())
//! # }
//! ```

use crate::did::Did;
use crate::error::{Error, Result};
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use serde::{Deserialize, Serialize};

/// UCAN (User Controlled Authorization Network)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ucan {
    /// Issuer DID (who grants the capability)
    pub iss: Did,

    /// Audience DID (who receives the capability)
    pub aud: Did,

    /// Capabilities granted
    pub att: Vec<Capability>,

    /// Expiration timestamp (Unix seconds)
    pub exp: u64,

    /// Not before timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,

    /// Nonce (optional, for replay protection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nnc: Option<String>,

    /// Proof: Chain of UCANs for delegation
    #[serde(default)]
    pub prf: Vec<String>,

    /// Signature (added when signed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

impl Ucan {
    /// Create a new UCAN
    pub fn new(
        iss: Did,
        aud: Did,
        att: Vec<Capability>,
        exp: u64,
        nbf: Option<u64>,
        nnc: Option<String>,
        prf: Vec<String>,
    ) -> Self {
        Self {
            iss,
            aud,
            att,
            exp,
            nbf,
            nnc,
            prf,
            sig: None,
        }
    }

    /// Sign the UCAN with a signing key
    pub fn sign(mut self, key: &SigningKey) -> Result<Self> {
        let payload = self.to_payload()?;
        let signature = key.sign(payload.as_bytes());
        self.sig = Some(base64::encode_config(
            &signature.to_bytes(),
            base64::URL_SAFE_NO_PAD,
        ));
        Ok(self)
    }

    /// Verify UCAN is valid (signature, expiry, delegation chain)
    pub fn verify(&self) -> Result<()> {
        // Check signature present
        let sig_str = self
            .sig
            .as_ref()
            .ok_or_else(|| Error::Ucan("UCAN not signed".to_string()))?;

        // Check expiration
        let now = Utc::now().timestamp() as u64;
        if now > self.exp {
            return Err(Error::UcanExpired);
        }

        // Check not-before
        if let Some(nbf) = self.nbf {
            if now < nbf {
                return Err(Error::UcanNotYetValid);
            }
        }

        // Verify signature
        let payload = self.to_payload()?;
        let sig_bytes = base64::decode_config(sig_str, base64::URL_SAFE_NO_PAD)
            .map_err(|e| Error::Encoding(format!("Failed to decode signature: {}", e)))?;

        let signature = Signature::from_bytes(
            sig_bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::SignatureVerification("Invalid signature length".to_string()))?,
        );

        self.iss
            .verification_key
            .verify(payload.as_bytes(), &signature)?;

        // Verify delegation chain
        for proof_jwt in &self.prf {
            let parent = Self::decode(proof_jwt)?;
            parent.verify()?;

            // Check parent grants at least the same capabilities to issuer
            if !parent.grants_to(&self.iss, &self.att)? {
                return Err(Error::InsufficientDelegation(format!(
                    "Parent UCAN does not grant sufficient capabilities to {}",
                    self.iss
                )));
            }
        }

        Ok(())
    }

    /// Check if this UCAN grants specific capabilities to a DID
    pub fn grants_to(&self, did: &Did, capabilities: &[Capability]) -> Result<bool> {
        // Check if audience matches
        if &self.aud != did {
            return Ok(false);
        }

        // Check if all requested capabilities are granted
        for requested in capabilities {
            let mut granted = false;
            for available in &self.att {
                if available.matches(requested) {
                    granted = true;
                    break;
                }
            }
            if !granted {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Encode UCAN as JWT
    pub fn encode(&self) -> Result<String> {
        let header = UcanHeader {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
            ucan_version: "0.10.0".to_string(),
        };

        let header_json = serde_json::to_string(&header)?;
        let header_b64 =
            base64::encode_config(header_json.as_bytes(), base64::URL_SAFE_NO_PAD);

        let payload = self.to_payload()?;
        let payload_b64 = base64::encode_config(payload.as_bytes(), base64::URL_SAFE_NO_PAD);

        let sig = self
            .sig
            .as_ref()
            .ok_or_else(|| Error::Ucan("UCAN not signed".to_string()))?;

        Ok(format!("{}.{}.{}", header_b64, payload_b64, sig))
    }

    /// Decode UCAN from JWT
    pub fn decode(jwt: &str) -> Result<Self> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::Ucan(format!("Invalid JWT format, expected 3 parts, got {}", parts.len())));
        }

        let payload_bytes = base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)
            .map_err(|e| Error::Encoding(format!("Failed to decode payload: {}", e)))?;

        let mut ucan: Ucan = serde_json::from_slice(&payload_bytes)?;
        ucan.sig = Some(parts[2].to_string());

        Ok(ucan)
    }

    /// Create canonical payload for signing
    fn to_payload(&self) -> Result<String> {
        // Create payload without signature
        let mut payload = serde_json::to_value(self)?;
        if let Some(obj) = payload.as_object_mut() {
            obj.remove("sig");
        }

        Ok(serde_json::to_string(&payload)?)
    }

    /// Create a delegation UCAN (child of this UCAN)
    pub fn delegate(
        &self,
        new_audience: Did,
        capabilities: Vec<Capability>,
        exp: u64,
        key: &SigningKey,
    ) -> Result<Self> {
        // Verify capabilities are subset of parent
        for cap in &capabilities {
            let mut granted = false;
            for parent_cap in &self.att {
                if parent_cap.matches(cap) {
                    granted = true;
                    break;
                }
            }
            if !granted {
                return Err(Error::InsufficientDelegation(format!(
                    "Cannot delegate capability not granted in parent: {:?}",
                    cap
                )));
            }
        }

        // Encode this UCAN as proof
        let proof = self.encode()?;

        // Create new UCAN
        let ucan = Ucan::new(
            self.aud.clone(),
            new_audience,
            capabilities,
            exp,
            None,
            Some(random_nonce()),
            vec![proof],
        );

        ucan.sign(key)
    }
}

/// UCAN capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    /// Resource (e.g., "vudo://namespace/collection/document")
    pub resource: String,

    /// Action (e.g., "read", "write", "delete", "delegate")
    pub action: String,
}

impl Capability {
    /// Create a new capability
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Check if this capability matches (grants) another capability
    pub fn matches(&self, other: &Capability) -> bool {
        // Check action match (wildcard or exact)
        let action_matches = self.action == "*" || self.action == other.action;

        // Check resource match (wildcard or exact)
        let resource_matches = if self.resource.ends_with('*') {
            let prefix = &self.resource[..self.resource.len() - 1];
            other.resource.starts_with(prefix)
        } else {
            self.resource == other.resource
        };

        action_matches && resource_matches
    }

    /// Create a wildcard capability for a resource prefix
    pub fn wildcard(resource_prefix: impl Into<String>) -> Self {
        Self {
            resource: format!("{}*", resource_prefix.into()),
            action: "*".to_string(),
        }
    }
}

/// UCAN JWT header
#[derive(Debug, Serialize, Deserialize)]
struct UcanHeader {
    alg: String,
    typ: String,
    #[serde(rename = "ucv")]
    ucan_version: String,
}

/// Generate a random nonce
fn random_nonce() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let nonce: [u8; 16] = rng.gen();
    hex::encode(&nonce)
}

// Add missing base64 and hex imports
mod base64 {
    pub use base64::prelude::*;

    pub fn encode_config(data: &[u8], config: base64::engine::GeneralPurpose) -> String {
        config.encode(data)
    }

    pub fn decode_config(data: &str, config: base64::engine::GeneralPurpose) -> Result<Vec<u8>, base64::DecodeError> {
        config.decode(data)
    }

    pub const URL_SAFE_NO_PAD: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;
}

mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use x25519_dalek::{PublicKey, StaticSecret};

    fn create_test_did() -> (Did, SigningKey) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = PublicKey::from(&encryption_secret);
        let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();
        (did, signing_key)
    }

    #[test]
    fn test_ucan_creation_and_signing() {
        let (issuer_did, issuer_key) = create_test_did();
        let (audience_did, _) = create_test_did();

        let capability = Capability::new("vudo://myapp/data", "read");
        let ucan = Ucan::new(
            issuer_did,
            audience_did,
            vec![capability],
            Utc::now().timestamp() as u64 + 3600,
            None,
            None,
            vec![],
        );

        let signed = ucan.sign(&issuer_key).unwrap();
        assert!(signed.sig.is_some());
    }

    #[test]
    fn test_ucan_verification() {
        let (issuer_did, issuer_key) = create_test_did();
        let (audience_did, _) = create_test_did();

        let capability = Capability::new("vudo://myapp/data", "read");
        let ucan = Ucan::new(
            issuer_did,
            audience_did,
            vec![capability],
            Utc::now().timestamp() as u64 + 3600,
            None,
            None,
            vec![],
        );

        let signed = ucan.sign(&issuer_key).unwrap();
        assert!(signed.verify().is_ok());
    }

    #[test]
    fn test_ucan_expiration() {
        let (issuer_did, issuer_key) = create_test_did();
        let (audience_did, _) = create_test_did();

        let capability = Capability::new("vudo://myapp/data", "read");
        let ucan = Ucan::new(
            issuer_did,
            audience_did,
            vec![capability],
            Utc::now().timestamp() as u64 - 1, // Already expired
            None,
            None,
            vec![],
        );

        let signed = ucan.sign(&issuer_key).unwrap();
        assert!(matches!(signed.verify(), Err(Error::UcanExpired)));
    }

    #[test]
    fn test_capability_matching() {
        let cap1 = Capability::new("vudo://myapp/*", "*");
        let cap2 = Capability::new("vudo://myapp/data", "read");
        assert!(cap1.matches(&cap2));

        let cap3 = Capability::new("vudo://myapp/data", "write");
        assert!(cap1.matches(&cap3));

        let cap4 = Capability::new("vudo://otherapp/data", "read");
        assert!(!cap1.matches(&cap4));
    }

    #[test]
    fn test_ucan_delegation() {
        let (issuer_did, issuer_key) = create_test_did();
        let (audience_did, audience_key) = create_test_did();
        let (delegate_did, _) = create_test_did();

        // Create root UCAN
        let root_ucan = Ucan::new(
            issuer_did,
            audience_did.clone(),
            vec![Capability::wildcard("vudo://myapp/")],
            Utc::now().timestamp() as u64 + 3600,
            None,
            None,
            vec![],
        )
        .sign(&issuer_key)
        .unwrap();

        // Delegate to another DID
        let delegated = root_ucan
            .delegate(
                delegate_did,
                vec![Capability::new("vudo://myapp/data", "read")],
                Utc::now().timestamp() as u64 + 1800,
                &audience_key,
            )
            .unwrap();

        assert!(delegated.verify().is_ok());
        assert_eq!(delegated.prf.len(), 1);
    }

    #[test]
    fn test_ucan_encoding_decoding() {
        let (issuer_did, issuer_key) = create_test_did();
        let (audience_did, _) = create_test_did();

        let ucan = Ucan::new(
            issuer_did,
            audience_did,
            vec![Capability::new("vudo://myapp/data", "read")],
            Utc::now().timestamp() as u64 + 3600,
            None,
            None,
            vec![],
        )
        .sign(&issuer_key)
        .unwrap();

        let jwt = ucan.encode().unwrap();
        let decoded = Ucan::decode(&jwt).unwrap();

        assert_eq!(ucan.iss, decoded.iss);
        assert_eq!(ucan.aud, decoded.aud);
        assert_eq!(ucan.att, decoded.att);
        assert_eq!(ucan.exp, decoded.exp);
    }
}
