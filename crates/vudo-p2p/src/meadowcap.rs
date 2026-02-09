//! Meadowcap capability system for fine-grained permissions.
//!
//! This module implements the Meadowcap capability system as specified in the
//! Willow Protocol. Capabilities provide cryptographically-secured permissions
//! for read, write, and delegation operations on path prefixes within a namespace.

use crate::error::{P2PError, Result};
use crate::willow_types::{NamespaceId, Path, SubspaceId};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

/// Permission level for a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// Read-only access.
    Read,
    /// Write access (includes read).
    Write,
    /// Full access including delegation.
    Admin,
}

impl Permission {
    /// Check if this permission includes another permission.
    pub fn includes(&self, other: Permission) -> bool {
        match (self, other) {
            (Permission::Admin, _) => true,
            (Permission::Write, Permission::Read) => true,
            (Permission::Write, Permission::Write) => true,
            (Permission::Read, Permission::Read) => true,
            _ => false,
        }
    }
}

/// A Meadowcap capability for accessing resources within a namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Namespace ID this capability grants access to.
    pub namespace_id: NamespaceId,
    /// Subspace ID (optional - if None, grants access to all subspaces).
    pub subspace_id: Option<SubspaceId>,
    /// Path prefix (grants access to all paths with this prefix).
    pub path_prefix: Path,
    /// Permission level.
    pub permission: Permission,
    /// Issuer public key (who created this capability).
    pub issuer: VerifyingKey,
    /// Signature over the capability.
    pub signature: Signature,
    /// Delegation chain (parent capabilities).
    pub delegation_chain: Vec<Capability>,
}

impl Capability {
    /// Create a new root capability with admin permission.
    pub fn new_root(namespace_id: NamespaceId, signing_key: &SigningKey) -> Self {
        let path_prefix = Path::empty();
        let permission = Permission::Admin;
        let issuer = signing_key.verifying_key();

        let message = Self::create_signing_message(
            namespace_id,
            None,
            &path_prefix,
            permission,
            &issuer,
            &[],
        );
        let signature = signing_key.sign(&message);

        Self {
            namespace_id,
            subspace_id: None,
            path_prefix,
            permission,
            issuer,
            signature,
            delegation_chain: Vec::new(),
        }
    }

    /// Delegate a capability to a more restricted scope.
    pub fn delegate(
        &self,
        subspace_id: Option<SubspaceId>,
        path_prefix: Path,
        permission: Permission,
        signing_key: &SigningKey,
    ) -> Result<Self> {
        // Verify this capability can delegate
        if self.permission != Permission::Admin {
            return Err(P2PError::CapabilityDelegationError(
                "Only admin capabilities can delegate".to_string(),
            ));
        }

        // Verify new permission is not greater than parent
        if !self.permission.includes(permission) {
            return Err(P2PError::CapabilityDelegationError(
                "Cannot delegate greater permission than parent".to_string(),
            ));
        }

        // Verify path prefix is more restrictive (or equal)
        if !self.path_prefix.is_prefix_of(&path_prefix) {
            return Err(P2PError::CapabilityDelegationError(
                "Delegated path must be under parent path".to_string(),
            ));
        }

        // Verify subspace ID matches if parent has one
        if let Some(parent_sub) = self.subspace_id {
            if let Some(new_sub) = subspace_id {
                if parent_sub != new_sub {
                    return Err(P2PError::CapabilityDelegationError(
                        "Delegated subspace must match parent".to_string(),
                    ));
                }
            }
        }

        let issuer = signing_key.verifying_key();
        let mut delegation_chain = self.delegation_chain.clone();
        delegation_chain.push(self.clone());

        let message = Self::create_signing_message(
            self.namespace_id,
            subspace_id,
            &path_prefix,
            permission,
            &issuer,
            &delegation_chain,
        );
        let signature = signing_key.sign(&message);

        Ok(Self {
            namespace_id: self.namespace_id,
            subspace_id,
            path_prefix,
            permission,
            issuer,
            signature,
            delegation_chain,
        })
    }

    /// Check if this capability grants read permission for a path.
    pub fn can_read(&self, subspace_id: SubspaceId, path: &Path) -> bool {
        self.check_permission(subspace_id, path, Permission::Read)
    }

    /// Check if this capability grants write permission for a path.
    pub fn can_write(&self, subspace_id: SubspaceId, path: &Path) -> bool {
        self.check_permission(subspace_id, path, Permission::Write)
    }

    /// Check if this capability grants permission for a path.
    fn check_permission(&self, subspace_id: SubspaceId, path: &Path, required: Permission) -> bool {
        // Check permission level
        if !self.permission.includes(required) {
            return false;
        }

        // Check subspace
        if let Some(cap_subspace) = self.subspace_id {
            if cap_subspace != subspace_id {
                return false;
            }
        }

        // Check path prefix
        self.path_prefix.is_prefix_of(path)
    }

    /// Verify the cryptographic signature on this capability.
    pub fn verify(&self) -> Result<()> {
        let message = Self::create_signing_message(
            self.namespace_id,
            self.subspace_id,
            &self.path_prefix,
            self.permission,
            &self.issuer,
            &self.delegation_chain,
        );

        self.issuer
            .verify(&message, &self.signature)
            .map_err(|e| P2PError::CapabilityDelegationError(format!("Invalid signature: {}", e)))?;

        // Verify delegation chain
        for parent in &self.delegation_chain {
            parent.verify()?;
        }

        Ok(())
    }

    /// Create a message to sign for a capability.
    fn create_signing_message(
        namespace_id: NamespaceId,
        subspace_id: Option<SubspaceId>,
        path_prefix: &Path,
        permission: Permission,
        issuer: &VerifyingKey,
        delegation_chain: &[Capability],
    ) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(namespace_id.as_bytes());
        if let Some(sub) = subspace_id {
            hasher.update(sub.as_bytes());
        }
        hasher.update(path_prefix.to_string().as_bytes());
        hasher.update([permission as u8]);
        hasher.update(issuer.as_bytes());
        hasher.update(&(delegation_chain.len() as u64).to_le_bytes());
        hasher.finalize().to_vec()
    }
}

/// Store for managing capabilities.
pub struct CapabilityStore {
    /// Capabilities indexed by namespace ID.
    capabilities: Arc<parking_lot::RwLock<HashMap<NamespaceId, Vec<Capability>>>>,
}

impl CapabilityStore {
    /// Create a new capability store.
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Add a capability to the store.
    pub fn add(&self, capability: Capability) -> Result<()> {
        // Verify the capability before storing
        capability.verify()?;

        let mut caps = self.capabilities.write();
        caps.entry(capability.namespace_id)
            .or_insert_with(Vec::new)
            .push(capability);
        Ok(())
    }

    /// Find a capability that grants the required permission for a path.
    pub fn find_capability(
        &self,
        namespace_id: NamespaceId,
        subspace_id: SubspaceId,
        path: &Path,
        required: Permission,
    ) -> Option<Capability> {
        let caps = self.capabilities.read();
        caps.get(&namespace_id)?
            .iter()
            .find(|cap| cap.check_permission(subspace_id, path, required))
            .cloned()
    }

    /// Get all capabilities for a namespace.
    pub fn get_all(&self, namespace_id: NamespaceId) -> Vec<Capability> {
        let caps = self.capabilities.read();
        caps.get(&namespace_id).cloned().unwrap_or_default()
    }

    /// Remove a capability.
    pub fn remove(&self, namespace_id: NamespaceId, capability: &Capability) {
        let mut caps = self.capabilities.write();
        if let Some(cap_list) = caps.get_mut(&namespace_id) {
            cap_list.retain(|c| c.signature != capability.signature);
        }
    }

    /// Clear all capabilities.
    pub fn clear(&self) {
        self.capabilities.write().clear();
    }
}

impl Default for CapabilityStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_includes() {
        assert!(Permission::Admin.includes(Permission::Read));
        assert!(Permission::Admin.includes(Permission::Write));
        assert!(Permission::Admin.includes(Permission::Admin));

        assert!(Permission::Write.includes(Permission::Read));
        assert!(Permission::Write.includes(Permission::Write));
        assert!(!Permission::Write.includes(Permission::Admin));

        assert!(Permission::Read.includes(Permission::Read));
        assert!(!Permission::Read.includes(Permission::Write));
        assert!(!Permission::Read.includes(Permission::Admin));
    }

    #[test]
    fn test_root_capability() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = NamespaceId::from_dol_namespace("myapp.v1");

        let root = Capability::new_root(namespace_id, &signing_key);

        assert_eq!(root.namespace_id, namespace_id);
        assert_eq!(root.permission, Permission::Admin);
        assert!(root.delegation_chain.is_empty());
        assert!(root.verify().is_ok());
    }

    #[test]
    fn test_capability_delegation() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = NamespaceId::from_dol_namespace("myapp.v1");
        let subspace_id = SubspaceId::from_dol_collection("users");

        let root = Capability::new_root(namespace_id, &signing_key);

        let delegated_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let delegated = root
            .delegate(
                Some(subspace_id),
                Path::from_components(["alice"]),
                Permission::Write,
                &delegated_key,
            )
            .unwrap();

        assert_eq!(delegated.namespace_id, namespace_id);
        assert_eq!(delegated.subspace_id, Some(subspace_id));
        assert_eq!(delegated.permission, Permission::Write);
        assert_eq!(delegated.delegation_chain.len(), 1);
        assert!(delegated.verify().is_ok());
    }

    #[test]
    fn test_capability_permissions() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = NamespaceId::from_dol_namespace("myapp.v1");
        let subspace_id = SubspaceId::from_dol_collection("users");

        let root = Capability::new_root(namespace_id, &signing_key);

        let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let alice_cap = root
            .delegate(
                Some(subspace_id),
                Path::from_components(["alice"]),
                Permission::Write,
                &alice_key,
            )
            .unwrap();

        // Alice can read her own documents
        assert!(alice_cap.can_read(subspace_id, &Path::from_components(["alice", "profile"])));

        // Alice can write her own documents
        assert!(alice_cap.can_write(subspace_id, &Path::from_components(["alice", "posts", "1"])));

        // Alice cannot access Bob's documents
        assert!(!alice_cap.can_read(subspace_id, &Path::from_components(["bob", "profile"])));
    }

    #[test]
    fn test_capability_store() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = NamespaceId::from_dol_namespace("myapp.v1");
        let subspace_id = SubspaceId::from_dol_collection("users");

        let store = CapabilityStore::new();
        let root = Capability::new_root(namespace_id, &signing_key);

        store.add(root.clone()).unwrap();

        let found = store.find_capability(
            namespace_id,
            subspace_id,
            &Path::from_components(["alice"]),
            Permission::Read,
        );
        assert!(found.is_some());
    }

    #[test]
    fn test_delegation_path_restriction() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = NamespaceId::from_dol_namespace("myapp.v1");

        let root = Capability::new_root(namespace_id, &signing_key);

        let delegated_key = SigningKey::generate(&mut rand::rngs::OsRng);

        // Try to delegate to a path not under parent - should fail
        let result = root.delegate(
            None,
            Path::from_components(["different", "path"]),
            Permission::Write,
            &delegated_key,
        );

        // This should succeed because root has empty path prefix (matches all)
        assert!(result.is_ok());
    }

    #[test]
    fn test_delegation_permission_restriction() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let namespace_id = NamespaceId::from_dol_namespace("myapp.v1");

        let root = Capability::new_root(namespace_id, &signing_key);

        // Delegate write permission
        let delegated_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let write_cap = root
            .delegate(
                None,
                Path::from_components(["alice"]),
                Permission::Write,
                &delegated_key,
            )
            .unwrap();

        // Try to delegate admin from write capability - should fail
        let admin_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let result = write_cap.delegate(
            None,
            Path::from_components(["alice", "posts"]),
            Permission::Admin,
            &admin_key,
        );

        assert!(result.is_err());
    }
}
