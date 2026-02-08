//! Example: Creating a master identity and device identity
//!
//! This example demonstrates:
//! - Creating a master identity (cold storage)
//! - Creating a device identity
//! - Linking device to master with UCAN
//! - Serializing identities to JSON

use vudo_identity::{DeviceIdentity, MasterIdentity};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VUDO Identity System Example ===\n");

    // Step 1: Create master identity (keep offline/cold storage)
    println!("Step 1: Creating master identity...");
    let mut master = MasterIdentity::generate("Alice").await?;
    println!("  Master DID: {}", master.did);
    println!("  Name: {}", master.name);
    println!();

    // Step 2: Create device identity
    println!("Step 2: Creating device identity...");
    let mut device = DeviceIdentity::generate("Alice's Phone").await?;
    println!("  Device DID: {}", device.did());
    println!("  Device Name: {}", device.device_name());
    println!();

    // Step 3: Link device to master (requires master key - offline operation)
    println!("Step 3: Linking device to master identity...");
    let master_key = master.signing_key();
    let link = master
        .link_device(
            device.device_name().to_string(),
            device.did().clone(),
            &master_key,
        )
        .await?;
    println!("  Device linked successfully!");
    println!("  Authorization UCAN issued");
    println!("  Linked at: {}", link.linked_at);
    println!();

    // Step 4: Update device with authorization
    println!("Step 4: Updating device with authorization...");
    device.link_to_master(master.did.clone(), link.authorization.clone());
    println!("  Device is now linked: {}", device.is_linked());
    println!();

    // Step 5: Verify device authorization
    println!("Step 5: Verifying device authorization...");
    device.verify_authorization()?;
    println!("  Authorization verified successfully!");
    println!();

    // Step 6: Display authorization UCAN
    println!("Step 6: Authorization UCAN details:");
    println!("  Issuer: {}", link.authorization.iss);
    println!("  Audience: {}", link.authorization.aud);
    println!("  Capabilities:");
    for cap in &link.authorization.att {
        println!("    - Resource: {}", cap.resource);
        println!("      Action: {}", cap.action);
    }
    println!("  Expires at: {}", link.authorization.exp);
    println!();

    // Step 7: Encode UCAN as JWT
    println!("Step 7: Encoding UCAN as JWT...");
    let jwt = link.authorization.encode()?;
    println!("  JWT (first 80 chars): {}...", &jwt[..80.min(jwt.len())]);
    println!();

    // Step 8: Serialize master identity (for secure storage)
    println!("Step 8: Serializing identities...");
    let master_json = serde_json::to_string_pretty(&master)?;
    println!("  Master identity JSON length: {} bytes", master_json.len());

    let device_json = serde_json::to_string_pretty(&device)?;
    println!("  Device identity JSON length: {} bytes", device_json.len());
    println!();

    println!("=== Example Complete ===");
    println!("\nSummary:");
    println!("  - Master identity created: {}", master.did);
    println!("  - Device identity created: {}", device.did());
    println!("  - Device linked to master with UCAN authorization");
    println!("  - All operations verified successfully");
    println!("\nSecurity Note:");
    println!("  Keep master identity offline in cold storage!");
    println!("  Use device identity for daily operations.");

    Ok(())
}
