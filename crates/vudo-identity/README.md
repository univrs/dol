# VUDO Identity

Decentralized identity system for VUDO Runtime with Peer DIDs (did:peer:2), UCANs, and key management.

## Features

- **Peer DIDs (did:peer:2)**: Decentralized identifiers for pairwise node authentication
- **UCANs**: User Controlled Authorization Networks for capability delegation
- **Ed25519 Signatures**: Fast, secure digital signatures
- **X25519 Key Agreement**: Secure key exchange for encryption
- **Master → Device Linking**: Hierarchical identity management
- **Key Rotation**: With grace periods and smooth transitions
- **Revocation Lists**: Cryptographically signed device revocations
- **DID Resolution**: Fast local and P2P resolution

## Architecture

```
Master Identity (Cold storage, offline)
  ├── Device Key 1 (Phone) ─→ UCAN
  ├── Device Key 2 (Laptop) ─→ UCAN
  └── Device Key 3 (Tablet) ─→ UCAN
```

Each device receives a UCAN from the master identity, granting it capabilities. Devices can further delegate capabilities to apps or services.

## Performance

- **Peer DID creation**: ~30 µs (target: < 50ms) ✅
- **UCAN delegation verification**: ~29 µs (target: < 10ms) ✅
- **Key rotation**: Preserves existing sync relationships ✅
- **Revocation propagation**: Within 1 sync cycle ✅

## Usage

### Creating a Master Identity

```rust
use vudo_identity::MasterIdentity;

let master = MasterIdentity::generate("Alice").await?;
println!("Master DID: {}", master.did);
```

### Linking a Device

```rust
use vudo_identity::{MasterIdentity, DeviceIdentity};

let mut master = MasterIdentity::generate("Alice").await?;
let device = DeviceIdentity::generate("Alice's Phone").await?;

let master_key = master.signing_key();
let link = master.link_device(
    "Alice's Phone".to_string(),
    device.did().clone(),
    &master_key,
).await?;
```

### Creating UCANs

```rust
use vudo_identity::{Did, Ucan, Capability};
use chrono::Utc;

let ucan = Ucan::new(
    issuer_did,
    audience_did,
    vec![Capability::new("vudo://myapp/*", "read")],
    Utc::now().timestamp() as u64 + 3600, // 1 hour
    None,
    None,
    vec![],
).sign(&issuer_key)?;

// Verify UCAN
ucan.verify()?;
```

### Delegating Capabilities

```rust
// Create delegation chain: Alice → Bob → Carol
let bob_to_carol = alice_to_bob.delegate(
    carol_did,
    vec![Capability::new("vudo://myapp/data", "read")],
    Utc::now().timestamp() as u64 + 1800,
    &bob_key,
)?;

// Verify entire chain
bob_to_carol.verify()?;
```

### Key Rotation

```rust
use ed25519_dalek::SigningKey;
use x25519_dalek::StaticSecret;
use rand::rngs::OsRng;

let new_key = SigningKey::generate(&mut OsRng);
let new_encryption = StaticSecret::random_from_rng(&mut OsRng);

let rotation = master.rotate_key(new_key, new_encryption).await?;
assert!(rotation.in_grace_period());
```

### DID Resolution

```rust
use vudo_identity::DidResolver;

let resolver = DidResolver::new();
let doc = resolver.resolve(&did).await?;
println!("Resolved DID: {}", doc.id);
```

## Examples

See the `examples/` directory for complete working examples:

- **create_identity.rs**: Creating master and device identities
- **device_linking.rs**: Managing multiple devices with revocation
- **ucan_delegation.rs**: Building UCAN delegation chains
- **key_rotation.rs**: Rotating keys with grace periods

Run examples:

```bash
cargo run --example create_identity
cargo run --example device_linking
cargo run --example ucan_delegation
cargo run --example key_rotation
```

## Testing

Run all tests:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

## Security Considerations

- Master keys should be kept offline in cold storage
- Device keys are used for daily operations
- UCANs have expiration times to limit exposure
- Revocation lists are synced via P2P gossip
- Key rotation includes grace periods to prevent service disruption

## References

- [did:peer spec](https://identity.foundation/peer-did-method-spec/)
- [UCAN spec](https://ucan.xyz/)
- [DID Core](https://www.w3.org/TR/did-core/)

## License

MIT OR Apache-2.0
