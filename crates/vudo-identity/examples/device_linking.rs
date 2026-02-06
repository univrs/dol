//! Example: Managing multiple devices with master identity
//!
//! This example demonstrates:
//! - Linking multiple devices to a master identity
//! - Revoking a device
//! - Checking revocation status

use vudo_identity::{DeviceIdentity, MasterIdentity};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Device Linking Example ===\n");

    // Create master identity
    println!("Creating master identity for Bob...");
    let mut master = MasterIdentity::generate("Bob").await?;
    println!("  Master DID: {}\n", master.did);

    // Create multiple devices
    println!("Creating devices...");
    let phone = DeviceIdentity::generate("Bob's Phone").await?;
    let laptop = DeviceIdentity::generate("Bob's Laptop").await?;
    let tablet = DeviceIdentity::generate("Bob's Tablet").await?;
    println!("  Phone DID: {}", phone.did());
    println!("  Laptop DID: {}", laptop.did());
    println!("  Tablet DID: {}\n", tablet.did());

    // Link all devices
    println!("Linking devices to master identity...");
    let master_key = master.signing_key();

    let phone_link = master
        .link_device(
            phone.device_name().to_string(),
            phone.did().clone(),
            &master_key,
        )
        .await?;
    println!("  ✓ Phone linked");

    let laptop_link = master
        .link_device(
            laptop.device_name().to_string(),
            laptop.did().clone(),
            &master_key,
        )
        .await?;
    println!("  ✓ Laptop linked");

    let tablet_link = master
        .link_device(
            tablet.device_name().to_string(),
            tablet.did().clone(),
            &master_key,
        )
        .await?;
    println!("  ✓ Tablet linked\n");

    // Display linked devices
    println!("Linked devices ({}): ", master.devices.len());
    for (i, device) in master.devices.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, device.device_name, device.device_did);
        println!("     Linked at: {}", device.linked_at);
        println!("     Revoked: {}", device.revoked);
    }
    println!();

    // Check revocation status
    println!("Checking revocation status...");
    println!("  Phone revoked? {}", master.is_device_revoked(phone.did()));
    println!(
        "  Laptop revoked? {}",
        master.is_device_revoked(laptop.did())
    );
    println!(
        "  Tablet revoked? {}\n",
        master.is_device_revoked(tablet.did())
    );

    // Revoke laptop (e.g., device lost or stolen)
    println!("Revoking laptop (device lost)...");
    master
        .revoke_device(
            laptop.did(),
            Some("Device lost or stolen".to_string()),
            &master_key,
        )
        .await?;
    println!("  ✓ Laptop revoked\n");

    // Check revocation status again
    println!("Checking revocation status after revocation...");
    println!("  Phone revoked? {}", master.is_device_revoked(phone.did()));
    println!(
        "  Laptop revoked? {}",
        master.is_device_revoked(laptop.did())
    );
    println!(
        "  Tablet revoked? {}\n",
        master.is_device_revoked(tablet.did())
    );

    // Display revocation list
    println!("Revocation list:");
    println!("  Version: {}", master.revocations.version);
    println!("  Revocations: {}", master.revocations.revocations.len());
    for (i, revocation) in master.revocations.revocations.iter().enumerate() {
        println!("  {}. Subject: {}", i + 1, revocation.subject);
        if let Some(ref reason) = revocation.reason {
            println!("     Reason: {}", reason);
        }
        println!("     Revoked at: {}", revocation.revoked_at);
    }
    println!();

    // Verify revocation list signature
    println!("Verifying revocation list signature...");
    master.revocations.verify()?;
    println!("  ✓ Signature verified\n");

    println!("=== Example Complete ===");
    println!("\nSummary:");
    println!("  - Created master identity with 3 devices");
    println!("  - Revoked 1 device (laptop)");
    println!("  - 2 devices remain active (phone, tablet)");
    println!("  - Revocation list signed and verified");
    println!("\nBest Practices:");
    println!("  - Revoke devices immediately if lost or stolen");
    println!("  - Keep revocation list synced across P2P network");
    println!("  - Regularly audit linked devices");

    Ok(())
}
