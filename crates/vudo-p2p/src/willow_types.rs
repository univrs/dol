//! Willow Protocol types and structures.
//!
//! This module defines the core types for the Willow Protocol, including:
//! - 3D namespaces (namespace_id, subspace_id, path)
//! - Entries with payload data
//! - Tombstones for deletion semantics

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A 32-byte namespace identifier.
///
/// Namespaces are derived from DOL System identifiers using BLAKE3 hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespaceId([u8; 32]);

impl NamespaceId {
    /// Create a namespace ID from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        Self(arr)
    }

    /// Get the bytes of the namespace ID.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create a namespace ID from a DOL namespace string.
    pub fn from_dol_namespace(namespace: &str) -> Self {
        let hash = blake3::hash(namespace.as_bytes());
        Self::from_bytes(hash.as_bytes())
    }
}

impl fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0[..8]))
    }
}

/// A 32-byte subspace identifier.
///
/// Subspaces are derived from DOL collection names using BLAKE3 hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubspaceId([u8; 32]);

impl SubspaceId {
    /// Create a subspace ID from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        Self(arr)
    }

    /// Get the bytes of the subspace ID.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create a subspace ID from a DOL collection string.
    pub fn from_dol_collection(collection: &str) -> Self {
        let hash = blake3::hash(collection.as_bytes());
        Self::from_bytes(hash.as_bytes())
    }
}

impl fmt::Display for SubspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0[..8]))
    }
}

/// A path component in the Willow 3D namespace.
///
/// Paths are hierarchical and can have multiple components (e.g., ["users", "alice", "posts", "1"]).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Path {
    /// Path components.
    components: Vec<String>,
}

impl Path {
    /// Create an empty path.
    pub fn empty() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Create a path from components.
    pub fn from_components<I, S>(components: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            components: components.into_iter().map(|s| s.into()).collect(),
        }
    }

    /// Create a path from a DOL document ID.
    pub fn from_dol_id(id: &str) -> Self {
        Self::from_components(id.split('/'))
    }

    /// Get path components.
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// Check if this path is a prefix of another path.
    pub fn is_prefix_of(&self, other: &Path) -> bool {
        if self.components.len() > other.components.len() {
            return false;
        }
        self.components
            .iter()
            .zip(other.components.iter())
            .all(|(a, b)| a == b)
    }

    /// Get the length of the path.
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.components.join("/"))
    }
}

/// A Willow entry representing a document in the 3D namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Namespace ID.
    pub namespace_id: NamespaceId,
    /// Subspace ID.
    pub subspace_id: SubspaceId,
    /// Path within the subspace.
    pub path: Path,
    /// Payload data.
    pub payload: Bytes,
    /// Timestamp (Unix epoch milliseconds).
    pub timestamp: u64,
}

impl Entry {
    /// Create a new entry.
    pub fn new(
        namespace_id: NamespaceId,
        subspace_id: SubspaceId,
        path: Path,
        payload: Bytes,
        timestamp: u64,
    ) -> Self {
        Self {
            namespace_id,
            subspace_id,
            path,
            payload,
            timestamp,
        }
    }

    /// Get the size of the entry in bytes.
    pub fn size(&self) -> usize {
        self.payload.len()
    }
}

/// A deletion tombstone for GDPR-compliant deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    /// Namespace ID.
    pub namespace_id: NamespaceId,
    /// Subspace ID.
    pub subspace_id: SubspaceId,
    /// Path of deleted entry.
    pub path: Path,
    /// Deletion timestamp (Unix epoch milliseconds).
    pub timestamp: u64,
    /// Reason for deletion (optional).
    pub reason: Option<String>,
}

impl Tombstone {
    /// Create a new tombstone.
    pub fn new(
        namespace_id: NamespaceId,
        subspace_id: SubspaceId,
        path: Path,
        timestamp: u64,
        reason: Option<String>,
    ) -> Self {
        Self {
            namespace_id,
            subspace_id,
            path,
            timestamp,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_id_from_dol() {
        let ns1 = NamespaceId::from_dol_namespace("myapp.v1");
        let ns2 = NamespaceId::from_dol_namespace("myapp.v1");
        let ns3 = NamespaceId::from_dol_namespace("myapp.v2");

        // Same namespace should produce same ID
        assert_eq!(ns1, ns2);
        // Different namespace should produce different ID
        assert_ne!(ns1, ns3);
    }

    #[test]
    fn test_subspace_id_from_dol() {
        let sub1 = SubspaceId::from_dol_collection("users");
        let sub2 = SubspaceId::from_dol_collection("users");
        let sub3 = SubspaceId::from_dol_collection("posts");

        assert_eq!(sub1, sub2);
        assert_ne!(sub1, sub3);
    }

    #[test]
    fn test_path_from_components() {
        let path = Path::from_components(["users", "alice", "posts", "1"]);
        assert_eq!(path.components().len(), 4);
        assert_eq!(path.components()[0], "users");
        assert_eq!(path.components()[3], "1");
    }

    #[test]
    fn test_path_from_dol_id() {
        let path = Path::from_dol_id("alice/posts/1");
        assert_eq!(path.components().len(), 3);
        assert_eq!(path.components()[0], "alice");
        assert_eq!(path.components()[2], "1");
    }

    #[test]
    fn test_path_is_prefix() {
        let prefix = Path::from_components(["users", "alice"]);
        let full = Path::from_components(["users", "alice", "posts", "1"]);
        let other = Path::from_components(["users", "bob"]);

        assert!(prefix.is_prefix_of(&full));
        assert!(!other.is_prefix_of(&full));
        assert!(!full.is_prefix_of(&prefix));
    }

    #[test]
    fn test_entry_creation() {
        let ns = NamespaceId::from_dol_namespace("myapp.v1");
        let sub = SubspaceId::from_dol_collection("users");
        let path = Path::from_components(["alice"]);
        let payload = Bytes::from("test data");

        let entry = Entry::new(ns, sub, path, payload, 12345);
        assert_eq!(entry.timestamp, 12345);
        assert_eq!(entry.size(), 9);
    }

    #[test]
    fn test_tombstone_creation() {
        let ns = NamespaceId::from_dol_namespace("myapp.v1");
        let sub = SubspaceId::from_dol_collection("users");
        let path = Path::from_components(["alice"]);

        let tombstone = Tombstone::new(ns, sub, path, 12345, Some("GDPR request".to_string()));
        assert_eq!(tombstone.timestamp, 12345);
        assert_eq!(tombstone.reason, Some("GDPR request".to_string()));
    }
}
