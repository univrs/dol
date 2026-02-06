# VUDO Privacy

GDPR-compliant cryptographic deletion for local-first CRDTs.

## Overview

`vudo-privacy` implements GDPR Article 17 (Right to Erasure) for VUDO Runtime through **cryptographic deletion**. Instead of physically deleting CRDT data (impossible in append-only logs), we encrypt personal data with user-specific keys and delete the key to make data permanently unrecoverable.

## Features

- **Cryptographic Deletion**: Per-user encryption keys (DEKs) for personal data
- **@personal Annotation**: DOL annotation for GDPR-sensitive fields
- **Pseudonymous Actor IDs**: Privacy-preserving CRDT metadata
- **Audit Trail**: Comprehensive logging for compliance
- **Willow Integration**: True-deletion for non-personal data
- **GDPR Article 17 Compliant**: Irreversible data erasure

## Quick Start

### Installation

```toml
[dependencies]
vudo-privacy = { path = "path/to/vudo-privacy" }
```

### Basic Usage

```rust
use vudo_privacy::{
    crypto::PersonalDataCrypto,
    gdpr::{GdprComplianceEngine, DeletionRequest},
    pseudonymous::PseudonymousActorId,
};

// 1. Personal data encryption
let crypto = PersonalDataCrypto::new();
let dek = crypto.generate_dek("did:peer:alice")?;

let email = b"alice@example.com";
let encrypted = crypto.encrypt_field(&dek, email)?;

// 2. GDPR deletion
let engine = GdprComplianceEngine::new()?;
let request = DeletionRequest::personal_only("app.example".to_string());
let report = engine.execute_deletion("did:peer:alice", request).await?;

assert!(report.irreversible);
```

## DOL Integration

Mark GDPR-sensitive fields with `@personal`:

```dol
gen UserProfile {
  @crdt(immutable) has id: String

  @crdt(lww) @personal has email: String
  @crdt(lww) @personal has full_name: String
  @crdt(lww) @personal has phone: String

  @crdt(lww) has username: String  // Public
}

exegesis {
  User profile with GDPR-compliant personal data handling.
}
```

The DOL code generator automatically encrypts `@personal` fields and generates access methods.

## Architecture

### Personal Data Flow

```text
1. User creates document → Generate per-user DEK
2. Encrypt @personal fields with DEK
3. Store encrypted data in CRDT (Automerge)
4. GDPR deletion request → Delete DEK
5. Encrypted data becomes permanently unrecoverable
```

### Cryptographic Security

- **Algorithm**: ChaCha20-Poly1305 AEAD
- **Key Size**: 256 bits
- **Nonce**: Random 96 bits per encryption
- **Key Deletion**: Secure memory zeroing (zeroize crate)

### Privacy Protection

- **Pseudonymous Actor IDs**: BLAKE3 hash of DID
- **Unlinkable**: Cannot reverse pseudonym → DID
- **Deterministic**: Same DID → same pseudonym

## Examples

### Personal Data Encryption

```rust
use vudo_privacy::crypto::PersonalDataCrypto;

let crypto = PersonalDataCrypto::new();
let dek = crypto.generate_dek("did:peer:alice")?;

// Encrypt
let plaintext = b"sensitive data";
let encrypted = crypto.encrypt_field(&dek, plaintext)?;

// Decrypt
let decrypted = crypto.decrypt_field(&dek, &encrypted)?;
assert_eq!(decrypted, plaintext);

// GDPR deletion
let receipt = crypto.delete_dek("did:peer:alice")?;
assert!(receipt.irreversible);

// Now decryption fails permanently
let result = crypto.decrypt_field(&dek, &encrypted);
assert!(result.is_err());
```

### GDPR Deletion Workflow

```rust
use vudo_privacy::gdpr::{GdprComplianceEngine, DeletionRequest};

let engine = GdprComplianceEngine::new()?;

// Create user data
engine.crypto().generate_dek("did:peer:alice")?;

// Execute deletion
let request = DeletionRequest::all_data("app.example".to_string())
    .add_public_path("/profile")
    .add_public_path("/settings");

let report = engine.execute_deletion("did:peer:alice", request).await?;

// Verify deletion
assert!(report.irreversible);
assert!(report.crypto_proof.is_some());
```

### Pseudonymous Actor IDs

```rust
use vudo_privacy::pseudonymous::PseudonymousActorId;
use automerge::{AutoCommit, transaction::Transactable};

// Generate pseudonymous actor ID
let pseudo = PseudonymousActorId::from_did("did:peer:alice")?;

// Use in Automerge document
let mut doc = AutoCommit::new();
doc.set_actor(pseudo.actor_id());

// Now all CRDT operations use pseudonym
doc.put(automerge::ROOT, "field", "value")?;
```

### Audit Trail

```rust
use vudo_privacy::audit::DeletionAuditLog;

let audit = engine.audit_log();

// Query deletions
let entries = audit.get_entries_for_user("did:peer:alice");
for entry in entries {
    println!("Request ID: {}", entry.request_id);
    println!("Method: {:?}", entry.method);
    println!("Has proof: {}", entry.has_proof());
}

// Export for compliance
let json = audit.export_json()?;
```

## GDPR Compliance

### Article 17 - Right to Erasure

✅ **Compliant**: Data is cryptographically erased without undue delay

- Personal data encrypted with user-specific keys
- Key deletion makes data permanently unrecoverable
- Irreversible deletion with cryptographic proof
- Audit trail for regulatory compliance

### Data Categories

- **Personal Data**: Encrypted with DEK, cryptographic erasure
- **Public Data**: Willow tombstones for true-deletion
- **Transaction History**: Anonymized (legal retention)

## Performance

Benchmarks on modern hardware (2024 MacBook):

- **DEK Generation**: < 1ms
- **Encryption (1KB)**: < 0.1ms
- **Decryption (1KB)**: < 0.1ms
- **Pseudonym Generation**: < 0.05ms

Performance overhead is negligible for most applications.

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_tests

# Run examples
cargo run --example personal_data_encryption
cargo run --example gdpr_deletion_workflow
cargo run --example audit_trail
```

## Documentation

- [GDPR Compliance Guide](../../docs/compliance/gdpr-local-first.md)
- [Privacy Audit](../../docs/compliance/privacy-audit.md)
- [API Documentation](https://docs.rs/vudo-privacy)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## References

- [GDPR Article 17](https://gdpr-info.eu/art-17-gdpr/)
- [Cryptographic Deletion in CRDTs](https://arxiv.org/abs/2103.13108)
- [ChaCha20-Poly1305 RFC 8439](https://datatracker.ietf.org/doc/html/rfc8439)
- [BLAKE3 Hash Function](https://github.com/BLAKE3-team/BLAKE3)
