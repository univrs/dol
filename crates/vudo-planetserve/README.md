# VUDO PlanetServe - Privacy-Preserving Sync

Privacy-preserving synchronization infrastructure for VUDO Runtime, inspired by distributed systems research from PlanetLab, Tor, and Vuvuzela.

## Features

- **S-IDA Fragmentation**: Fragment messages using Reed-Solomon erasure coding (k-of-n threshold)
- **Onion Routing**: Multi-hop routing with layered encryption to hide sender-receiver relationships
- **Metadata Obfuscation**: Message padding, timing jitter, and cover traffic to resist traffic analysis
- **BFT Verification**: Byzantine fault-tolerant committees with privacy preservation
- **Configurable Privacy Levels**: Trade off privacy for performance

## Privacy Guarantees

### S-IDA (Secure Information Dispersal Algorithm)

- Message fragmented into n shards
- Any k shards can reconstruct the original
- Having < k shards reveals **NO information**
- No single peer observes the full message

### Onion Routing

- Entry relay knows sender, **not receiver**
- Exit relay knows receiver, **not sender**
- Middle relays know **neither**
- No single relay can correlate sender and receiver

### Metadata Obfuscation

- **Message padding**: Hides actual content size
- **Timing jitter**: Makes timing unpredictable
- **Cover traffic**: Hides sync frequency

## Usage

### Fast-Open Mode (No Privacy)

```rust
use vudo_planetserve::{PlanetServeAdapter, config::PrivacyConfig};
use std::sync::Arc;

let adapter = PlanetServeAdapter::new(
    identity,
    p2p,
    PrivacyConfig::fast_open(), // No privacy, maximum speed
).await?;

adapter.sync_private("namespace", "doc_id", data).await?;
```

### Privacy-Max Mode (Full Privacy)

```rust
let adapter = PlanetServeAdapter::new(
    identity,
    p2p,
    PrivacyConfig::privacy_max(), // Maximum privacy
).await?;

adapter.start().await?; // Start cover traffic
adapter.sync_private("namespace", "doc_id", data).await?;
adapter.stop().await?;
```

### BFT Private Voting

```rust
use vudo_planetserve::bft::{BftPrivateCommittee, Proposal};

let committee = BftPrivateCommittee::new(
    vec![
        "did:peer:member1".to_string(),
        "did:peer:member2".to_string(),
        "did:peer:member3".to_string(),
        "did:peer:member4".to_string(),
        "did:peer:member5".to_string(),
    ],
    adapter,
);

let proposal = Proposal::new("credit_reconciliation", data);
let result = committee.private_vote(&proposal).await?;
```

## Privacy Levels

| Level    | S-IDA | Onion | Padding | Jitter | Cover | Latency Overhead |
|----------|-------|-------|---------|--------|-------|------------------|
| None     | No    | No    | No      | No     | No    | 0ms              |
| Basic    | No    | No    | Yes     | No     | No    | <5ms             |
| Standard | No    | No    | Yes     | Yes    | No    | ~100ms           |
| Maximum  | Yes   | Yes   | Yes     | Yes    | Yes   | ~500ms           |

## Performance Tuning

### S-IDA Parameters

- `k=2, n=3`: Fast, basic redundancy (1 failure tolerated)
- `k=3, n=5`: Balanced (2 failures tolerated, **DEFAULT**)
- `k=5, n=7`: High redundancy (2 failures tolerated, slower)

### Onion Routing

- `hops=1`: No anonymity, fast
- `hops=2`: Basic anonymity (**DEFAULT**)
- `hops=3`: Strong anonymity (Tor-level)

### Padding

- `1024 bytes`: Fast, moderate privacy
- `4096 bytes`: Balanced (**DEFAULT**)
- `16384 bytes`: Strong privacy, slower

## Examples

Run the examples to see PlanetServe in action:

```bash
# S-IDA fragmentation demo
cargo run --example sida_fragmentation

# Onion routing demo
cargo run --example onion_routing

# Privacy levels comparison
cargo run --example privacy_levels

# BFT private voting demo
cargo run --example bft_private_vote
```

## Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run benchmarks
cargo bench
```

## Security Considerations

### Threat Model

PlanetServe provides defense against:
- **Passive observers**: Cannot see message contents or correlation
- **Compromised relays**: Single relay compromise reveals nothing
- **Traffic analysis**: Padding, jitter, and cover traffic resist timing attacks
- **Partial collusion**: Need k-of-n fragments to reconstruct

PlanetServe does NOT protect against:
- **Global passive adversary**: Can correlate all network traffic
- **k-of-n relay collusion**: Can reconstruct if controlling k+ relays
- **Endpoint compromise**: Plaintext visible at sender/receiver

### Best Practices

- Use **Maximum** privacy for sensitive operations (credit transfers)
- Use **Standard** privacy for normal operations
- Use **Basic** or **None** for public data
- Rotate relay pools regularly
- Monitor relay reliability and latency
- Set appropriate k and n values for S-IDA

## References

- [S-IDA Paper](https://dl.acm.org/doi/10.1145/1315245.1315318) - Information Dispersal Algorithm
- [Tor Onion Routing](https://www.torproject.org/) - Onion routing protocol
- [Vuvuzela](https://vuvuzela.io/) - Metadata-private messaging
- [PlanetLab](https://www.planet-lab.org/) - Distributed systems research

## License

MIT OR Apache-2.0
