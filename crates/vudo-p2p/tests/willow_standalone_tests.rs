//! Standalone tests for Willow Protocol types and Meadowcap capabilities.
//!
//! These tests don't require the full state engine or Iroh integration.

use ed25519_dalek::SigningKey;
use vudo_p2p::willow_types::{NamespaceId, Path, SubspaceId};
use vudo_p2p::meadowcap::{Capability, CapabilityStore, Permission};

#[test]
fn test_namespace_id_deterministic() {
    let ns1 = NamespaceId::from_dol_namespace("myapp.v1");
    let ns2 = NamespaceId::from_dol_namespace("myapp.v1");
    let ns3 = NamespaceId::from_dol_namespace("myapp.v2");

    // Same namespace should produce same ID (deterministic)
    assert_eq!(ns1, ns2);
    // Different namespace should produce different ID
    assert_ne!(ns1, ns3);
}

#[test]
fn test_subspace_id_deterministic() {
    let sub1 = SubspaceId::from_dol_collection("users");
    let sub2 = SubspaceId::from_dol_collection("users");
    let sub3 = SubspaceId::from_dol_collection("posts");

    assert_eq!(sub1, sub2);
    assert_ne!(sub1, sub3);
}

#[test]
fn test_path_construction() {
    let path1 = Path::from_components(["users", "alice", "posts", "1"]);
    assert_eq!(path1.components().len(), 4);
    assert_eq!(path1.components()[0], "users");
    assert_eq!(path1.components()[3], "1");

    let path2 = Path::from_dol_id("alice/posts/1");
    assert_eq!(path2.components().len(), 3);
    assert_eq!(path2.components()[0], "alice");
    assert_eq!(path2.components()[2], "1");
}

#[test]
fn test_path_prefix_matching() {
    let prefix = Path::from_components(["users", "alice"]);
    let full_path = Path::from_components(["users", "alice", "posts", "1"]);
    let other_path = Path::from_components(["users", "bob", "posts", "1"]);

    assert!(prefix.is_prefix_of(&full_path));
    assert!(!other_path.is_prefix_of(&full_path));
    assert!(!full_path.is_prefix_of(&prefix));
}

#[test]
fn test_empty_path() {
    let empty = Path::empty();
    let non_empty = Path::from_components(["alice"]);

    assert!(empty.is_empty());
    assert!(!non_empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(non_empty.len(), 1);

    // Empty path is prefix of all paths
    assert!(empty.is_prefix_of(&non_empty));
}

#[test]
fn test_permission_hierarchy() {
    // Admin includes all permissions
    assert!(Permission::Admin.includes(Permission::Read));
    assert!(Permission::Admin.includes(Permission::Write));
    assert!(Permission::Admin.includes(Permission::Admin));

    // Write includes read
    assert!(Permission::Write.includes(Permission::Read));
    assert!(Permission::Write.includes(Permission::Write));
    assert!(!Permission::Write.includes(Permission::Admin));

    // Read only includes read
    assert!(Permission::Read.includes(Permission::Read));
    assert!(!Permission::Read.includes(Permission::Write));
    assert!(!Permission::Read.includes(Permission::Admin));
}

#[test]
fn test_root_capability_creation() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    assert_eq!(root_cap.namespace_id, namespace_id);
    assert_eq!(root_cap.permission, Permission::Admin);
    assert!(root_cap.path_prefix.is_empty());
    assert!(root_cap.delegation_chain.is_empty());
}

#[test]
fn test_root_capability_verification() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Root capability should be valid
    assert!(root_cap.verify().is_ok());
}

#[test]
fn test_capability_delegation_basic() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");
    let subspace_id = SubspaceId::from_dol_collection("users");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    let delegated_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let delegated_cap = root_cap
        .delegate(
            Some(subspace_id),
            Path::from_components(["alice"]),
            Permission::Write,
            &delegated_key,
        )
        .unwrap();

    assert_eq!(delegated_cap.namespace_id, namespace_id);
    assert_eq!(delegated_cap.subspace_id, Some(subspace_id));
    assert_eq!(delegated_cap.permission, Permission::Write);
    assert_eq!(delegated_cap.delegation_chain.len(), 1);
}

#[test]
fn test_delegated_capability_verification() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");
    let subspace_id = SubspaceId::from_dol_collection("users");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    let delegated_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let delegated_cap = root_cap
        .delegate(
            Some(subspace_id),
            Path::from_components(["alice"]),
            Permission::Read,
            &delegated_key,
        )
        .unwrap();

    // Delegated capability should be valid
    assert!(delegated_cap.verify().is_ok());
}

#[test]
fn test_capability_read_permission() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");
    let subspace_id = SubspaceId::from_dol_collection("users");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_cap = root_cap
        .delegate(
            Some(subspace_id),
            Path::from_components(["alice"]),
            Permission::Read,
            &alice_key,
        )
        .unwrap();

    // Alice can read her own documents
    assert!(alice_cap.can_read(subspace_id, &Path::from_components(["alice", "profile"])));
    assert!(alice_cap.can_read(subspace_id, &Path::from_components(["alice", "posts", "1"])));

    // Alice cannot read Bob's documents
    assert!(!alice_cap.can_read(subspace_id, &Path::from_components(["bob", "profile"])));
}

#[test]
fn test_capability_write_permission() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");
    let subspace_id = SubspaceId::from_dol_collection("users");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_cap = root_cap
        .delegate(
            Some(subspace_id),
            Path::from_components(["alice"]),
            Permission::Write,
            &alice_key,
        )
        .unwrap();

    // Alice can write to her own documents
    assert!(alice_cap.can_write(subspace_id, &Path::from_components(["alice", "profile"])));

    // Write permission includes read
    assert!(alice_cap.can_read(subspace_id, &Path::from_components(["alice", "profile"])));

    // Alice cannot write to Bob's documents
    assert!(!alice_cap.can_write(subspace_id, &Path::from_components(["bob", "profile"])));
}

#[test]
fn test_capability_delegation_path_restriction() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Delegate to users/* path
    let users_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let users_cap = root_cap
        .delegate(
            None,
            Path::from_components(["users"]),
            Permission::Write,
            &users_key,
        )
        .unwrap();

    // Try to delegate to a more specific path under users - should succeed
    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_cap = users_cap
        .delegate(
            None,
            Path::from_components(["users", "alice"]),
            Permission::Read,
            &alice_key,
        )
        .unwrap();

    assert!(alice_cap.verify().is_ok());

    // Try to delegate to a path NOT under users - should fail
    let admin_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let result = users_cap.delegate(
        None,
        Path::from_components(["admin", "settings"]),
        Permission::Read,
        &admin_key,
    );

    assert!(result.is_err());
}

#[test]
fn test_capability_delegation_permission_restriction() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Delegate write permission
    let write_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let write_cap = root_cap
        .delegate(
            None,
            Path::from_components(["users"]),
            Permission::Write,
            &write_key,
        )
        .unwrap();

    // Try to delegate admin permission from write capability - should fail
    let admin_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let result = write_cap.delegate(
        None,
        Path::from_components(["users", "alice"]),
        Permission::Admin,
        &admin_key,
    );

    assert!(result.is_err());

    // Delegating read from write should succeed
    let read_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let read_cap = write_cap
        .delegate(
            None,
            Path::from_components(["users", "alice"]),
            Permission::Read,
            &read_key,
        )
        .unwrap();

    assert!(read_cap.verify().is_ok());
}

#[test]
fn test_capability_store_add_and_find() {
    let store = CapabilityStore::new();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");
    let subspace_id = SubspaceId::from_dol_collection("users");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Add capability to store
    store.add(root_cap.clone()).unwrap();

    // Find capability for reading
    let found = store.find_capability(
        namespace_id,
        subspace_id,
        &Path::from_components(["alice"]),
        Permission::Read,
    );

    assert!(found.is_some());
    assert_eq!(found.unwrap().namespace_id, namespace_id);
}

#[test]
fn test_capability_store_multiple_capabilities() {
    let store = CapabilityStore::new();

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");
    let subspace_id = SubspaceId::from_dol_collection("users");

    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Add root capability
    store.add(root_cap.clone()).unwrap();

    // Add delegated capability for Alice
    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_cap = root_cap
        .delegate(
            Some(subspace_id),
            Path::from_components(["alice"]),
            Permission::Read,
            &alice_key,
        )
        .unwrap();

    store.add(alice_cap.clone()).unwrap();

    // Should find both capabilities
    let all_caps = store.get_all(namespace_id);
    assert_eq!(all_caps.len(), 2);
}

#[test]
fn test_multi_level_delegation_chain() {
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("test.v1");
    let subspace_id = SubspaceId::from_dol_collection("users");

    // Level 1: Root
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    // Level 2: Users admin
    let users_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let users_cap = root_cap
        .delegate(
            Some(subspace_id),
            Path::empty(),
            Permission::Write,
            &users_key,
        )
        .unwrap();

    // Level 3: Alice read-only
    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_cap = users_cap
        .delegate(
            Some(subspace_id),
            Path::from_components(["alice"]),
            Permission::Read,
            &alice_key,
        )
        .unwrap();

    // Verify delegation chain length
    assert_eq!(root_cap.delegation_chain.len(), 0);
    assert_eq!(users_cap.delegation_chain.len(), 1);
    assert_eq!(alice_cap.delegation_chain.len(), 2);

    // All capabilities should verify
    assert!(root_cap.verify().is_ok());
    assert!(users_cap.verify().is_ok());
    assert!(alice_cap.verify().is_ok());
}

#[test]
fn test_path_display() {
    let path = Path::from_components(["users", "alice", "posts", "1"]);
    assert_eq!(path.to_string(), "users/alice/posts/1");

    let empty = Path::empty();
    assert_eq!(empty.to_string(), "");
}
