# GDPR Compliance for Local-First CRDTs

## Overview

This document describes how VUDO Runtime achieves GDPR Article 17 (Right to Erasure) compliance for local-first CRDT applications through cryptographic deletion.

## The Challenge

**Traditional deletion is impossible in append-only CRDT logs.**

CRDTs (Conflict-free Replicated Data Types) rely on append-only operation logs for conflict resolution. Once data is written to a CRDT and synced to peers, it cannot be physically deleted without breaking CRDT semantics. This creates a fundamental conflict with GDPR Article 17, which requires data controllers to delete personal data upon request.

## The Solution: Cryptographic Deletion

Instead of physically deleting data, we encrypt personal data with user-specific Data Encryption Keys (DEKs). Deleting the key makes data permanently unrecoverable, achieving the same outcome as physical deletion.

### Architecture

```text
Personal Data Flow:
1. User creates document → Generate per-user DEK
2. Encrypt @personal fields with DEK
3. Store encrypted data in CRDT (Automerge)
4. GDPR deletion request → Delete DEK
5. Encrypted data becomes permanently unrecoverable on all peers
```

### Cryptographic Guarantees

- **Algorithm**: ChaCha20-Poly1305 AEAD (Authenticated Encryption with Associated Data)
- **Key Size**: 256 bits (secure against brute force)
- **Nonce**: 96 bits, randomly generated per encryption
- **Key Deletion**: Secure memory zeroing using zeroize crate
- **Irreversibility**: Without the key, data is cryptographically unrecoverable

## DOL Integration

### @personal Annotation

Mark GDPR-sensitive fields with `@personal`:

```dol
gen UserProfile {
  @crdt(immutable) has id: String

  @crdt(lww) @personal has email: String
  @crdt(lww) @personal has full_name: String
  @crdt(lww) @personal has phone: String

  @crdt(lww) has username: String  // Public, not @personal
  @crdt(pn_counter) has reputation: i64  // Public
}

exegesis {
  User profile with GDPR-compliant personal data handling.
}
```

### Code Generation

The DOL code generator automatically handles `@personal` fields:

```rust
// Generated from DOL with @personal annotation
#[derive(Debug, Clone, Reconcile, Hydrate)]
pub struct UserProfile {
    #[autosurgeon(immutable)]
    pub id: String,

    // Encrypted @personal fields
    #[autosurgeon(immutable)]
    pub email: EncryptedField,

    #[autosurgeon(immutable)]
    pub full_name: EncryptedField,

    #[autosurgeon(immutable)]
    pub phone: EncryptedField,

    // Public fields (not encrypted)
    pub username: String,
    pub reputation: i64,
}

impl UserProfile {
    /// GDPR-compliant deletion
    pub fn gdpr_delete(&mut self, engine: &GdprComplianceEngine) -> Result<DeletionReceipt> {
        engine.execute_deletion(&self.id, DeletionRequest::personal_only("app")).await
    }

    /// Access personal data (decrypts if key available)
    pub fn get_email(&self, crypto: &PersonalDataCrypto) -> Result<String> {
        let dek = crypto.get_dek(&self.id)?;
        let plaintext = crypto.decrypt_field(dek, &self.email)?;
        Ok(String::from_utf8(plaintext)?)
    }
}
```

## Privacy-Preserving CRDT Metadata

### The Problem

Automerge (and other CRDTs) store actor IDs in every operation for conflict resolution. If we use real DIDs as actor IDs, user identity is exposed in all CRDT metadata.

### The Solution: Pseudonymous Actor IDs

Generate pseudonymous actor IDs by hashing the real DID:

```rust
use vudo_privacy::pseudonymous::PseudonymousActorId;

// Generate pseudonymous actor ID from DID
let pseudo = PseudonymousActorId::from_did("did:peer:alice")?;

// Use in Automerge document
let mut doc = AutoCommit::new();
doc.set_actor(pseudo.actor_id());

// Now all CRDT operations use pseudonym instead of real DID
doc.put(automerge::ROOT, "field", "value")?;
```

**Properties:**
- **Unlinkable**: Cannot reverse pseudonym → DID without rainbow table
- **Deterministic**: Same DID always maps to same pseudonym
- **Consistent**: Enables proper conflict resolution

## Deletion Workflow

### 1. Personal Data Deletion (Cryptographic Erasure)

```rust
use vudo_privacy::gdpr::{GdprComplianceEngine, DeletionRequest};

let engine = GdprComplianceEngine::new()?;

// Execute GDPR deletion
let request = DeletionRequest::personal_only("app.example".to_string());
let report = engine.execute_deletion("did:peer:alice", request).await?;

// Verify deletion
assert!(report.irreversible);
assert!(report.crypto_proof.is_some());
```

### 2. Public Data Deletion (Willow Tombstones)

For non-personal data, use Willow tombstones:

```rust
let request = DeletionRequest::all_data("app.example".to_string())
    .add_public_path("/profile")
    .add_public_path("/settings");

let report = engine.execute_deletion("did:peer:alice", request).await?;
```

### 3. Transaction History (Anonymization)

For data that must be retained for legal/tax reasons:

```rust
let request = DeletionRequest {
    namespace: "app.example".to_string(),
    personal_data: true,
    public_data: false,
    public_data_paths: vec![],
    transaction_history: true,  // Anonymize instead of delete
};

let report = engine.execute_deletion("did:peer:alice", request).await?;
```

## Audit Trail

### Requirements

GDPR requires maintaining records of:
- What data was deleted
- When it was deleted
- Who requested the deletion
- How it was deleted (method)
- Proof that deletion occurred

### Implementation

```rust
use vudo_privacy::audit::{DeletionAuditLog, DataCategory, DeletionMethod};

let audit_log = engine.audit_log();

// Query audit log
let alice_entries = audit_log.get_entries_for_user("did:peer:alice");

for entry in alice_entries {
    println!("Request ID: {}", entry.request_id);
    println!("Method: {:?}", entry.method);
    println!("Has proof: {}", entry.has_proof());
}

// Export for regulatory compliance
let json = audit_log.export_json()?;
```

### Audit Log Entry

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_did": "did:peer:alice",
  "deleted_at": 1704067200,
  "categories": ["PersonalData"],
  "method": "CryptographicErasure",
  "proof": {
    "owner": "did:peer:alice",
    "deleted_at": 1704067200,
    "irreversible": true
  }
}
```

## Data Categories

### Personal Data
- **Definition**: Data that can identify an individual (email, name, phone, etc.)
- **Deletion Method**: Cryptographic erasure (DEK deletion)
- **DOL Annotation**: `@personal`
- **Legal Requirement**: Must be deleted on request (GDPR Article 17)

### Public Data
- **Definition**: Non-personal data visible to others (username, public posts, etc.)
- **Deletion Method**: Willow tombstones
- **DOL Annotation**: None (default)
- **Legal Requirement**: May be deleted on request

### Transaction History
- **Definition**: Financial records required for tax/legal compliance
- **Deletion Method**: Anonymization (replace DID with pseudonym)
- **DOL Annotation**: Custom handling
- **Legal Requirement**: May be retained for legal periods

## Legal Compliance

### GDPR Article 17 - Right to Erasure

> "The data subject shall have the right to obtain from the controller the erasure of personal data concerning him or her without undue delay."

**VUDO Compliance:**
- ✅ Personal data is deleted (cryptographically) without undue delay
- ✅ Deletion is irreversible and permanent
- ✅ Audit trail proves deletion occurred
- ✅ Deletion receipts provide cryptographic proof

### Exceptions (GDPR Article 17, Paragraph 3)

Deletion may be refused when data is necessary for:
- Compliance with legal obligations
- Archiving purposes in the public interest
- Establishment, exercise, or defense of legal claims

**VUDO Implementation:**
- Transaction history can be anonymized instead of deleted
- System can distinguish between personal data and legal records
- Configurable retention policies per data category

## Security Considerations

### Key Management

- DEKs are stored locally, never synced
- Key deletion uses secure memory zeroing
- No key recovery mechanism (by design)
- Master keys for key backup (user responsibility)

### Cryptography

- ChaCha20-Poly1305 AEAD (industry standard)
- 256-bit keys (quantum-resistant for foreseeable future)
- Random nonces prevent replay attacks
- Authenticated encryption prevents tampering

### Privacy

- Pseudonymous actor IDs prevent identity leakage
- CRDT metadata does not expose real DIDs
- Local mapping (pseudonym → DID) never synced
- Personal data encrypted at rest and in transit

## Implementation Checklist

- [x] Personal data encryption with DEKs
- [x] Key deletion renders data unrecoverable
- [x] @personal annotation in DOL pipeline
- [x] CRDT metadata uses pseudonymized actor IDs
- [x] Willow true-deletion for non-personal data
- [x] Audit trail for deletion requests
- [x] Deletion receipts with cryptographic proof
- [x] GDPR Article 17 compliance verified
- [x] Code generation for @personal fields
- [x] Examples and documentation

## References

- [GDPR Article 17](https://gdpr-info.eu/art-17-gdpr/)
- [Cryptographic Deletion in CRDTs](https://arxiv.org/abs/2103.13108)
- [ChaCha20-Poly1305 RFC 8439](https://datatracker.ietf.org/doc/html/rfc8439)
- [VUDO Privacy Crate](../../crates/vudo-privacy/)
