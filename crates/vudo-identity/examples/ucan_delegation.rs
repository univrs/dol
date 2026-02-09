//! Example: UCAN delegation chains
//!
//! This example demonstrates:
//! - Creating a root UCAN
//! - Delegating capabilities to another DID
//! - Building a delegation chain (Alice -> Bob -> Carol)
//! - Verifying the entire chain

use chrono::Utc;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use vudo_identity::{Capability, Did, Ucan};
use x25519_dalek::{PublicKey, StaticSecret};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== UCAN Delegation Chain Example ===\n");

    // Create three identities: Alice (root), Bob, and Carol
    println!("Creating identities...");

    let alice_key = SigningKey::generate(&mut OsRng);
    let alice_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let alice_did = Did::from_keys(alice_key.verifying_key(), &alice_enc)?;
    println!("  Alice DID: {}", alice_did);

    let bob_key = SigningKey::generate(&mut OsRng);
    let bob_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let bob_did = Did::from_keys(bob_key.verifying_key(), &bob_enc)?;
    println!("  Bob DID: {}", bob_did);

    let carol_key = SigningKey::generate(&mut OsRng);
    let carol_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let carol_did = Did::from_keys(carol_key.verifying_key(), &carol_enc)?;
    println!("  Carol DID: {}\n", carol_did);

    // Step 1: Alice grants Bob full access to myapp
    println!("Step 1: Alice grants Bob full access to myapp...");
    let alice_to_bob = Ucan::new(
        alice_did.clone(),
        bob_did.clone(),
        vec![Capability::wildcard("vudo://myapp/")],
        Utc::now().timestamp() as u64 + 3600, // 1 hour
        None,
        None,
        vec![],
    )
    .sign(&alice_key)?;

    println!("  Issuer: {}", alice_to_bob.iss);
    println!("  Audience: {}", alice_to_bob.aud);
    println!("  Capabilities:");
    for cap in &alice_to_bob.att {
        println!("    - Resource: {}", cap.resource);
        println!("      Action: {}", cap.action);
    }
    println!("  Expires: {}", alice_to_bob.exp);
    println!();

    // Verify root UCAN
    println!("Verifying root UCAN...");
    alice_to_bob.verify()?;
    println!("  ✓ Root UCAN verified\n");

    // Step 2: Bob delegates read access to Carol
    println!("Step 2: Bob delegates read access to Carol...");
    let bob_to_carol = alice_to_bob.delegate(
        carol_did.clone(),
        vec![Capability::new("vudo://myapp/data", "read")],
        Utc::now().timestamp() as u64 + 1800, // 30 minutes
        &bob_key,
    )?;

    println!("  Issuer: {}", bob_to_carol.iss);
    println!("  Audience: {}", bob_to_carol.aud);
    println!("  Capabilities:");
    for cap in &bob_to_carol.att {
        println!("    - Resource: {}", cap.resource);
        println!("      Action: {}", cap.action);
    }
    println!("  Expires: {}", bob_to_carol.exp);
    println!("  Proof chain length: {}", bob_to_carol.prf.len());
    println!();

    // Verify delegation chain
    println!("Verifying delegation chain...");
    bob_to_carol.verify()?;
    println!("  ✓ Entire delegation chain verified\n");

    // Check if Carol has specific capabilities
    println!("Checking Carol's capabilities...");
    let can_read = bob_to_carol.grants_to(
        &carol_did,
        &[Capability::new("vudo://myapp/data", "read")],
    )?;
    println!("  Can read vudo://myapp/data? {}", can_read);

    let can_write = bob_to_carol.grants_to(
        &carol_did,
        &[Capability::new("vudo://myapp/data", "write")],
    )?;
    println!("  Can write vudo://myapp/data? {}", can_write);
    println!();

    // Step 3: Encode UCANs as JWTs
    println!("Step 3: Encoding UCANs as JWTs...");
    let root_jwt = alice_to_bob.encode()?;
    let delegated_jwt = bob_to_carol.encode()?;

    println!("  Root UCAN JWT length: {} bytes", root_jwt.len());
    println!("  Delegated UCAN JWT length: {} bytes", delegated_jwt.len());
    println!();

    // Step 4: Decode and verify JWTs
    println!("Step 4: Decoding and verifying JWTs...");
    let decoded_root = Ucan::decode(&root_jwt)?;
    let decoded_delegated = Ucan::decode(&delegated_jwt)?;

    decoded_root.verify()?;
    println!("  ✓ Decoded root UCAN verified");

    decoded_delegated.verify()?;
    println!("  ✓ Decoded delegated UCAN verified");
    println!();

    // Step 5: Demonstrate insufficient delegation
    println!("Step 5: Demonstrating insufficient delegation...");
    println!("  Attempting to delegate write access (should fail)...");

    let result = alice_to_bob.delegate(
        carol_did.clone(),
        vec![Capability::new("vudo://otherapp/data", "read")], // Different app!
        Utc::now().timestamp() as u64 + 1800,
        &bob_key,
    );

    match result {
        Ok(_) => println!("  ✗ Delegation succeeded (unexpected)"),
        Err(e) => println!("  ✓ Delegation failed as expected: {}", e),
    }
    println!();

    println!("=== Example Complete ===");
    println!("\nDelegation Chain:");
    println!("  Alice (root) → Bob (full access)");
    println!("                  ↓");
    println!("                Carol (read-only)");
    println!("\nKey Points:");
    println!("  - UCANs enable fine-grained capability delegation");
    println!("  - Delegation chains are verified cryptographically");
    println!("  - Cannot delegate capabilities not granted in parent");
    println!("  - Expiration times cascade down the chain");

    Ok(())
}
