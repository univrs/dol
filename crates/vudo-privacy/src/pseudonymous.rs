//! Pseudonymous actor IDs for CRDT operations.
//!
//! This module provides pseudonymous actor IDs for CRDT operations to protect
//! user privacy in CRDT metadata. Instead of using real DIDs as actor IDs
//! (which would expose user identity in every CRDT operation), we use cryptographic
//! hashes to generate pseudonymous IDs.
//!
//! # Problem
//!
//! Automerge (and other CRDTs) store actor IDs in every operation for conflict
//! resolution. If we use real DIDs as actor IDs, user identity is exposed in
//! all CRDT metadata, violating GDPR privacy requirements.
//!
//! # Solution
//!
//! Generate pseudonymous actor IDs by hashing the real DID:
//!
//! ```text
//! pseudonym = BLAKE3(DID)
//! ```
//!
//! This provides:
//! - Privacy: Real identity not exposed in CRDT ops
//! - Consistency: Same DID always maps to same pseudonym
//! - Unlinkability: Cannot reverse pseudonym → DID without rainbow table
//!
//! # Example
//!
//! ```rust
//! use vudo_privacy::pseudonymous::PseudonymousActorId;
//! use automerge::{AutoCommit, transaction::Transactable};
//!
//! # fn example() {
//! // Generate pseudonymous actor ID from DID
//! let pseudo = PseudonymousActorId::from_did("did:peer:alice").unwrap();
//!
//! // Use in Automerge document
//! let mut doc = AutoCommit::new();
//! doc.set_actor(pseudo.actor_id());
//!
//! // Now all CRDT operations use pseudonym instead of real DID
//! doc.put(automerge::ROOT, "field", "value").unwrap();
//! # }
//! ```

use crate::error::{PrivacyError, Result};
use automerge::ActorId;
use blake3;
use serde::{Deserialize, Serialize};

/// Pseudonymous actor ID for CRDT operations.
///
/// Maps a real DID to a pseudonymous actor ID for use in CRDT metadata.
/// The mapping is deterministic (same DID → same pseudonym) but unlinkable
/// (cannot reverse pseudonym → DID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PseudonymousActorId {
    /// Real DID (kept private, not stored in CRDT).
    real_did: String,

    /// Pseudonymous actor ID (used in CRDT ops).
    #[serde(with = "actor_id_serde")]
    pseudonym: ActorId,
}

/// Serde helper for ActorId.
mod actor_id_serde {
    use automerge::ActorId;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(actor_id: &ActorId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        actor_id.to_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ActorId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        Ok(ActorId::from(bytes))
    }
}

impl PseudonymousActorId {
    /// Generate pseudonymous actor ID from DID.
    ///
    /// Uses BLAKE3 hash to create a deterministic but unlinkable pseudonym.
    ///
    /// # Arguments
    ///
    /// * `did` - The real DID to pseudonymize
    ///
    /// # Returns
    ///
    /// A `PseudonymousActorId` containing both the real DID and pseudonym.
    pub fn from_did(did: &str) -> Result<Self> {
        if did.is_empty() {
            return Err(PrivacyError::InvalidDid("DID cannot be empty".to_string()));
        }

        // Hash DID with BLAKE3 to generate pseudonym
        let hash = blake3::hash(did.as_bytes());

        // Use first 16 bytes of hash as actor ID
        let bytes: Vec<u8> = hash.as_bytes()[..16].to_vec();
        let pseudonym = ActorId::from(bytes);

        Ok(Self {
            real_did: did.to_string(),
            pseudonym,
        })
    }

    /// Get the pseudonymous actor ID for CRDT operations.
    ///
    /// This is what should be used in Automerge documents, not the real DID.
    pub fn actor_id(&self) -> ActorId {
        self.pseudonym.clone()
    }

    /// Get the real DID (for internal use only).
    ///
    /// This should only be used when necessary (e.g., key management).
    /// Never expose this in CRDT metadata.
    pub fn real_did(&self) -> &str {
        &self.real_did
    }

    /// Convert to bytes for storage.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.pseudonym.to_bytes().to_vec()
    }

    /// Check if two pseudonymous IDs represent the same user.
    pub fn same_user(&self, other: &Self) -> bool {
        self.real_did == other.real_did
    }
}

impl PartialEq for PseudonymousActorId {
    fn eq(&self, other: &Self) -> bool {
        self.pseudonym == other.pseudonym
    }
}

impl Eq for PseudonymousActorId {}

impl std::fmt::Display for PseudonymousActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pseudonym({})", hex::encode(self.pseudonym.to_bytes()))
    }
}

/// Actor ID mapper for managing pseudonym ↔ DID mappings.
///
/// In a real system, you might need to maintain a local mapping to resolve
/// pseudonyms back to DIDs (for UI display, etc.). This is kept local and
/// never synced.
pub struct ActorIdMapper {
    /// Map pseudonym → real DID (local only, never synced).
    pseudonym_to_did: dashmap::DashMap<String, String>,
}

impl ActorIdMapper {
    /// Create a new actor ID mapper.
    pub fn new() -> Self {
        Self {
            pseudonym_to_did: dashmap::DashMap::new(),
        }
    }

    /// Register a pseudonymous actor ID.
    ///
    /// Stores the mapping pseudonym → DID locally (never synced).
    pub fn register(&self, pseudo: &PseudonymousActorId) {
        let key = hex::encode(pseudo.pseudonym.to_bytes());
        self.pseudonym_to_did.insert(key, pseudo.real_did.clone());
    }

    /// Resolve a pseudonym to a DID (if known locally).
    ///
    /// Returns None if the pseudonym is not in the local mapping.
    pub fn resolve(&self, actor_id: &ActorId) -> Option<String> {
        let key = hex::encode(actor_id.to_bytes());
        self.pseudonym_to_did
            .get(&key)
            .map(|entry| entry.value().clone())
    }

    /// Check if a pseudonym is known locally.
    pub fn is_known(&self, actor_id: &ActorId) -> bool {
        let key = hex::encode(actor_id.to_bytes());
        self.pseudonym_to_did.contains_key(&key)
    }

    /// Get the number of registered pseudonyms.
    pub fn count(&self) -> usize {
        self.pseudonym_to_did.len()
    }
}

impl Default for ActorIdMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pseudonym_generation() {
        let pseudo = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        assert_eq!(pseudo.real_did(), "did:peer:alice");
        assert!(!pseudo.actor_id().to_bytes().is_empty());
    }

    #[test]
    fn test_deterministic_pseudonym() {
        let pseudo1 = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        let pseudo2 = PseudonymousActorId::from_did("did:peer:alice").unwrap();

        assert_eq!(pseudo1.actor_id(), pseudo2.actor_id());
    }

    #[test]
    fn test_different_dids_different_pseudonyms() {
        let pseudo1 = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        let pseudo2 = PseudonymousActorId::from_did("did:peer:bob").unwrap();

        assert_ne!(pseudo1.actor_id(), pseudo2.actor_id());
    }

    #[test]
    fn test_empty_did() {
        let result = PseudonymousActorId::from_did("");
        assert!(result.is_err());
    }

    #[test]
    fn test_actor_id_mapper() {
        let mapper = ActorIdMapper::new();

        let pseudo = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        mapper.register(&pseudo);

        let resolved = mapper.resolve(&pseudo.actor_id());
        assert_eq!(resolved, Some("did:peer:alice".to_string()));
    }

    #[test]
    fn test_same_user() {
        let pseudo1 = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        let pseudo2 = PseudonymousActorId::from_did("did:peer:alice").unwrap();
        let pseudo3 = PseudonymousActorId::from_did("did:peer:bob").unwrap();

        assert!(pseudo1.same_user(&pseudo2));
        assert!(!pseudo1.same_user(&pseudo3));
    }
}
