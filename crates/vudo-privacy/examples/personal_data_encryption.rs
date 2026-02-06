//! Example: Personal data encryption with DEKs.
//!
//! Demonstrates how to encrypt and decrypt personal data using per-user
//! data encryption keys (DEKs).

use vudo_privacy::crypto::{DataEncryptionKey, PersonalDataCrypto};

fn main() -> vudo_privacy::error::Result<()> {
    println!("=== Personal Data Encryption Example ===\n");

    // Create crypto manager
    let crypto = PersonalDataCrypto::new();

    // Generate DEK for user
    println!("1. Generating DEK for user...");
    let dek = crypto.generate_dek("did:peer:alice@example.com")?;
    println!("   ✓ DEK created for: {}", dek.owner);
    println!("   ✓ Created at: {}", dek.created_at);
    println!();

    // Encrypt personal data fields
    println!("2. Encrypting personal data...");

    let email = b"alice@example.com";
    let encrypted_email = crypto.encrypt_field(&dek, email)?;
    println!("   ✓ Email encrypted: {} bytes", encrypted_email.ciphertext.len());

    let full_name = b"Alice Johnson";
    let encrypted_name = crypto.encrypt_field(&dek, full_name)?;
    println!("   ✓ Name encrypted: {} bytes", encrypted_name.ciphertext.len());

    let phone = b"+1-555-0123";
    let encrypted_phone = crypto.encrypt_field(&dek, phone)?;
    println!("   ✓ Phone encrypted: {} bytes", encrypted_phone.ciphertext.len());
    println!();

    // Decrypt personal data
    println!("3. Decrypting personal data...");

    let decrypted_email = crypto.decrypt_field(&dek, &encrypted_email)?;
    println!("   ✓ Email: {}", String::from_utf8_lossy(&decrypted_email));

    let decrypted_name = crypto.decrypt_field(&dek, &encrypted_name)?;
    println!("   ✓ Name: {}", String::from_utf8_lossy(&decrypted_name));

    let decrypted_phone = crypto.decrypt_field(&dek, &encrypted_phone)?;
    println!("   ✓ Phone: {}", String::from_utf8_lossy(&decrypted_phone));
    println!();

    // GDPR deletion: Delete DEK
    println!("4. Executing GDPR deletion (deleting DEK)...");
    let receipt = crypto.delete_dek("did:peer:alice@example.com")?;
    println!("   ✓ DEK deleted at: {}", receipt.deleted_at);
    println!("   ✓ Irreversible: {}", receipt.irreversible);
    println!();

    // Try to decrypt after deletion
    println!("5. Attempting to decrypt after deletion...");
    let deleted_dek = crypto.get_dek("did:peer:alice@example.com")?;

    match crypto.decrypt_field(&deleted_dek, &encrypted_email) {
        Ok(_) => println!("   ✗ ERROR: Decryption should have failed!"),
        Err(e) => println!("   ✓ Decryption failed as expected: {}", e),
    }
    println!();

    println!("=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("• Personal data is encrypted with user-specific DEKs");
    println!("• Deleting the DEK makes data permanently unrecoverable");
    println!("• This implements GDPR Article 17 (Right to Erasure)");
    println!("• Data remains encrypted in CRDT, but cannot be decrypted");

    Ok(())
}
