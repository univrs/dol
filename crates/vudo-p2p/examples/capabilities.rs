//! Example: Meadowcap capability delegation.
//!
//! This example demonstrates how to create and delegate capabilities for
//! fine-grained access control.

use ed25519_dalek::SigningKey;
use vudo_p2p::{
    meadowcap::{Capability, Permission},
    willow_types::{NamespaceId, Path, SubspaceId},
};

fn main() {
    println!("=== Meadowcap Capability Delegation ===\n");

    // Create root capability
    println!("1. Creating Root Capability\n");

    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let namespace_id = NamespaceId::from_dol_namespace("myapp.v1");
    let root_cap = Capability::new_root(namespace_id, &signing_key);

    println!("   Namespace: {}", namespace_id);
    println!("   Permission: Admin (full access)");
    println!("   Path prefix: <empty> (all paths)\n");

    // Delegate write capability for users collection
    println!("2. Delegating Write Capability for Users Collection\n");

    let users_admin_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let subspace_id = SubspaceId::from_dol_collection("users");

    let users_cap = root_cap
        .delegate(
            Some(subspace_id),
            Path::empty(),
            Permission::Write,
            &users_admin_key,
        )
        .unwrap();

    println!("   Subspace: users");
    println!("   Permission: Write");
    println!("   Path prefix: <empty> (all users)\n");

    // Delegate read-only capability for Alice's data
    println!("3. Delegating Read-Only Capability for Alice\n");

    let alice_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let alice_path = Path::from_components(["alice"]);

    let alice_cap = users_cap
        .delegate(Some(subspace_id), alice_path.clone(), Permission::Read, &alice_key)
        .unwrap();

    println!("   Subspace: users");
    println!("   Permission: Read");
    println!("   Path prefix: alice/* (only Alice's data)\n");

    // Test permissions
    println!("4. Testing Permissions\n");

    let alice_profile_path = Path::from_components(["alice", "profile"]);
    let alice_posts_path = Path::from_components(["alice", "posts", "1"]);
    let bob_profile_path = Path::from_components(["bob", "profile"]);

    println!("   Alice capability:");
    println!("     Can read alice/profile? {}", alice_cap.can_read(subspace_id, &alice_profile_path));
    println!(
        "     Can read alice/posts/1? {}",
        alice_cap.can_read(subspace_id, &alice_posts_path)
    );
    println!("     Can read bob/profile? {}", alice_cap.can_read(subspace_id, &bob_profile_path));
    println!("     Can write alice/profile? {}", alice_cap.can_write(subspace_id, &alice_profile_path));
    println!();

    // Demonstrate delegation chain
    println!("5. Capability Delegation Chain\n");

    println!("   Root capability:");
    println!("     Delegation chain length: {}", root_cap.delegation_chain.len());
    println!();

    println!("   Users capability:");
    println!("     Delegation chain length: {}", users_cap.delegation_chain.len());
    println!("     Parent: Root capability");
    println!();

    println!("   Alice capability:");
    println!("     Delegation chain length: {}", alice_cap.delegation_chain.len());
    println!("     Parent chain: Root -> Users admin");
    println!();

    // Verify cryptographic signatures
    println!("6. Verifying Cryptographic Signatures\n");

    println!("   Root capability valid? {}", root_cap.verify().is_ok());
    println!("   Users capability valid? {}", users_cap.verify().is_ok());
    println!("   Alice capability valid? {}", alice_cap.verify().is_ok());

    println!("\n=== Capability Delegation Complete ===");
}
