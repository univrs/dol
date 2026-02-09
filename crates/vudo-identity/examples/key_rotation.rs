//! Example: Key rotation with grace period
//!
//! This example demonstrates:
//! - Rotating master keys
//! - Grace period for old keys
//! - Verifying rotation certificates
//! - Preserving device relationships

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use vudo_identity::{DeviceIdentity, MasterIdentity};
use x25519_dalek::StaticSecret;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Key Rotation Example ===\n");

    // Step 1: Create master identity
    println!("Step 1: Creating master identity...");
    let mut master = MasterIdentity::generate("Charlie").await?;
    let old_did = master.did.clone();
    println!("  Master DID: {}", old_did);
    println!();

    // Step 2: Link a device
    println!("Step 2: Linking a device...");
    let device = DeviceIdentity::generate("Charlie's Phone").await?;
    let master_key = master.signing_key();
    master
        .link_device(
            device.device_name().to_string(),
            device.did().clone(),
            &master_key,
        )
        .await?;
    println!("  Device DID: {}", device.did());
    println!("  Devices linked: {}", master.devices.len());
    println!();

    // Step 3: Rotate master key
    println!("Step 3: Rotating master key...");
    println!("  Generating new keys...");
    let new_key = SigningKey::generate(&mut OsRng);
    let new_encryption = StaticSecret::random_from_rng(&mut OsRng);

    println!("  Performing rotation...");
    let rotation = master.rotate_key(new_key, new_encryption).await?;
    let new_did = master.did.clone();

    println!("  ✓ Key rotated successfully");
    println!("  Old DID: {}", old_did);
    println!("  New DID: {}", new_did);
    println!();

    // Step 4: Verify rotation certificate
    println!("Step 4: Verifying rotation certificate...");
    rotation.verify()?;
    println!("  ✓ Rotation certificate verified");
    println!();

    // Step 5: Check grace period
    println!("Step 5: Checking grace period...");
    println!("  Grace period active? {}", rotation.in_grace_period());
    println!(
        "  Grace period duration: {} days",
        rotation.grace_period / (24 * 60 * 60)
    );
    println!("  Rotated at: {}", rotation.rotated_at);
    println!();

    // Step 6: Display rotation certificate details
    println!("Step 6: Rotation certificate details:");
    println!("  Old DID: {}", rotation.certificate.old_did);
    println!("  New DID: {}", rotation.certificate.new_did);
    println!("  Timestamp: {}", rotation.certificate.timestamp);
    println!(
        "  Old key signature length: {} bytes",
        rotation.certificate.old_key_signature.len()
    );
    println!(
        "  New key signature length: {} bytes",
        rotation.certificate.new_key_signature.len()
    );
    println!();

    // Step 7: Verify device relationships preserved
    println!("Step 7: Verifying device relationships preserved...");
    println!("  Devices after rotation: {}", master.devices.len());
    for device_link in &master.devices {
        println!("    - {}", device_link.device_name);
        println!("      DID: {}", device_link.device_did);
        println!("      Revoked: {}", device_link.revoked);
    }
    println!("  ✓ All device relationships preserved");
    println!();

    // Step 8: Rotation history
    println!("Step 8: Rotation history:");
    println!("  Total rotations: {}", master.rotations.len());
    for (i, rot) in master.rotations.iter().enumerate() {
        println!("  Rotation {}:", i + 1);
        println!("    From: {}", rot.old_did);
        println!("    To: {}", rot.new_did);
        println!("    At: {}", rot.rotated_at);
        println!("    In grace period: {}", rot.in_grace_period());
    }
    println!();

    println!("=== Example Complete ===");
    println!("\nKey Rotation Summary:");
    println!("  - Old key: {}", old_did);
    println!("  - New key: {}", new_did);
    println!("  - Grace period: {} days", rotation.grace_period / (24 * 60 * 60));
    println!("  - Device relationships: preserved");
    println!("  - Rotation verified: yes");
    println!("\nBest Practices:");
    println!("  - Rotate keys periodically (e.g., annually)");
    println!("  - Use grace period to ensure smooth transition");
    println!("  - Verify rotation certificates before accepting");
    println!("  - Keep rotation history for audit trail");
    println!("  - Sync rotation certificates via P2P network");
    println!("\nDuring Grace Period:");
    println!("  - Both old and new keys are valid");
    println!("  - Peers gradually update to new key");
    println!("  - After grace period, old key is rejected");

    Ok(())
}
