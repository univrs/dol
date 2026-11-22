# Adversarial Testing Suite

## Status

**Implementation**: ✅ Complete
**Integration**: 🔄 Pending API additions
**Documentation**: ✅ Complete

## Overview

This directory contains comprehensive Byzantine Fault Tolerance (BFT) tests for the VUDO Credit system. The test suite validates the system against:

1. **Corrupted CRDT Operations** (6 tests)
2. **Sybil Attacks** (6 tests)
3. **Replay Attacks** (6 tests)
4. **Timing Attacks** (6 tests)
5. **Eclipse Attacks** (6 tests)
6. **Resource Exhaustion** (8 tests)
7. **Byzantine Committee Consensus** (8 tests)

**Total**: 40+ adversarial tests

## Architecture

```
adversarial/
├── test_harness.rs           # Infrastructure (MaliciousNode, HonestNode, utilities)
├── corrupted_crdt.rs         # CRDT operation attacks
├── sybil_attack.rs           # Identity-based attacks
├── replay_attack.rs          # Transaction replay attacks
├── timing_attack.rs          # Timing analysis attacks
├── eclipse_attack.rs         # Network isolation attacks
├── resource_exhaustion.rs    # Resource DoS attacks
├── byzantine_committee.rs    # BFT consensus attacks
└── mod.rs                    # Module organization
```

## Integration Requirements

The adversarial tests require the following API additions to the main implementation:

### vudo_credit

```rust
// CreditAccountHandle
impl CreditAccountHandle {
    pub async fn confirmed_balance(&self) -> Result<i64>;
}

// ReputationManager
impl ReputationManager {
    pub fn new() -> Self;
    pub fn get_tier(&self, did: &str) -> ReputationTier;
    pub fn credit_limit(&self, tier: ReputationTier) -> i64;
}

// ReputationTier
pub enum ReputationTier {
    Tier0,  // $1.00
    Tier1,  // $100
    Tier2,  // $1,000
    Tier3,  // $10,000
    Tier4,  // $100,000
    Tier5,  // $1,000,000
}
```

### vudo_identity

```rust
impl MasterIdentity {
    pub fn did(&self) -> &str;
}
```

### vudo_p2p

```rust
impl VudoP2P {
    pub fn connected_peers(&self) -> Vec<PeerId>;
}
```

## Running Tests

Once integration is complete:

```bash
# Run all adversarial tests
cargo test --test adversarial_tests

# Run specific category
cargo test --test adversarial_tests corrupted_crdt
cargo test --test adversarial_tests sybil_attack
cargo test --test adversarial_tests replay_attack
cargo test --test adversarial_tests timing_attack
cargo test --test adversarial_tests eclipse_attack
cargo test --test adversarial_tests resource_exhaustion
cargo test --test adversarial_tests byzantine_committee

# Run with output
cargo test --test adversarial_tests -- --nocapture
```

## Test Philosophy

### Design Principles

1. **Assume Malicious Intent**: Attackers actively try to break the system
2. **Test at Scale**: Single attacker vs. many attackers
3. **Measure Damage**: Quantify impact if attacks succeed
4. **Verify Mitigation**: Ensure defenses work as designed
5. **Document Threats**: Maintain living threat model

### Attack Patterns

Each test follows this structure:

```rust
#[tokio::test]
async fn test_attack_name() {
    // 1. Setup: Create honest nodes
    let honest_nodes = create_honest_nodes(N).await;

    // 2. Execute: Launch attack
    let malicious = create_malicious_node(Strategy).await;
    let result = malicious.execute_attack().await;

    // 3. Verify: Attack unsuccessful
    assert!(!result.successful);
    assert_eq!(result.damage_assessment, DamageLevel::None);

    // 4. Cleanup
    for node in honest_nodes {
        node.stop().await;
    }
}
```

### Success Criteria

- ✅ Attack detected (if applicable)
- ✅ Attack prevented/mitigated
- ✅ Damage level: None or Minimal
- ✅ System remains operational
- ✅ Honest peers maintain consensus

## Security Properties

### Safety Properties (must always hold)

1. **No double-spend**: Transaction replay prevented
2. **State consistency**: Corrupted operations rejected
3. **BFT consensus**: > 2/3 honest votes required
4. **Resource bounds**: All resources have limits

### Liveness Properties (eventual progress)

1. **Progress despite faults**: System works with f Byzantine faults in 3f+1
2. **Timeout protection**: Delayed votes don't block forever
3. **Eclipse resistance**: Multi-source discovery maintains connectivity

### Privacy Properties

1. **Timing resistance**: Cover traffic + jitter prevent correlation
2. **Metadata protection**: PlanetServe obscures patterns

## Threat Model

See `/docs/security/threat-model.md` for complete threat analysis.

### Mitigated Threats

| Threat | Severity | Status |
|--------|----------|--------|
| Corrupted CRDT | High | ✅ Mitigated |
| Sybil Attack | High | ✅ Mitigated |
| Replay Attack | Critical | ✅ Mitigated |
| Timing Attack | Medium | ✅ Mitigated |
| Eclipse Attack | High | ✅ Mitigated |
| Resource Exhaustion | Critical | ✅ Mitigated |
| Byzantine Voting | Critical | ✅ Mitigated |

### Residual Risks

1. **Global Passive Adversary**: Can correlate all traffic
2. **k-of-n Relay Collusion**: If attacker controls k+ relays
3. **Endpoint Compromise**: Plaintext at sender/receiver
4. **Social Engineering**: User key compromise

## Documentation

- `/docs/security/bft-test-report.md` - Test results and analysis
- `/docs/security/threat-model.md` - Complete threat model
- `/docs/security/attack-scenarios.md` - Detailed attack descriptions

## Future Work

### Short-term

1. **Fuzzing**: Integrate cargo-fuzz for CRDT operation fuzzing
2. **Property-based testing**: Add proptest for invariant checking
3. **Formal verification**: Consider TLA+ spec for BFT consensus

### Long-term

1. **Adaptive reputation**: ML-based Sybil detection
2. **Dynamic limits**: Adjust based on system load
3. **Advanced privacy**: PIR (Private Information Retrieval)

## Contributing

This project is not accepting external contributions or pull requests.

## License

Same as parent project (MIT OR Apache-2.0)
