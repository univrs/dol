# VUDO Runtime Threat Model

**Version**: 1.0
**Date**: 2026-02-05
**Status**: Active

---

## Table of Contents

1. [Introduction](#introduction)
2. [System Overview](#system-overview)
3. [Threat Actors](#threat-actors)
4. [Attack Surface](#attack-surface)
5. [Threat Scenarios](#threat-scenarios)
6. [Security Controls](#security-controls)
7. [Residual Risks](#residual-risks)
8. [Assumptions](#assumptions)

---

## Introduction

This document presents a comprehensive threat model for the VUDO Runtime, a local-first, peer-to-peer distributed system with mutual credit and privacy-preserving sync.

### Scope

This threat model covers:
- VUDO Credit (mutual credit system)
- VUDO State (CRDT-based state management)
- VUDO P2P (Iroh-based networking)
- VUDO PlanetServe (privacy-preserving sync)
- VUDO Identity (decentralized identity)

### Methodology

We use the STRIDE methodology:
- **S**poofing
- **T**ampering
- **R**epudiation
- **I**nformation Disclosure
- **D**enial of Service
- **E**levation of Privilege

---

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                       VUDO Runtime                          │
├──────────────┬──────────────┬──────────────┬───────────────┤
│  Identity    │   Credit     │    State     │      P2P      │
│              │              │              │               │
│  DID-based   │  Mutual      │  Automerge   │  Iroh +       │
│  UCAN auth   │  Credit      │  CRDTs       │  Willow       │
└──────────────┴──────────────┴──────────────┴───────────────┘
                              │
                        ┌─────┴─────┐
                        │           │
                 ┌──────▼─────┐ ┌──▼───────┐
                 │ Honest     │ │ Malicious│
                 │ Peers      │ │ Peers    │
                 └────────────┘ └──────────┘
```

### Trust Boundaries

1. **Device boundary**: Code running on user's device (trusted)
2. **Network boundary**: P2P network traffic (untrusted)
3. **Peer boundary**: Remote peer nodes (untrusted)
4. **Storage boundary**: Local storage (trusted, encrypted)

---

## Threat Actors

### 1. Passive Observer (Low-Medium Capability)

**Motivation**: Surveillance, data collection
**Capabilities**:
- Observe network traffic
- Correlate timing patterns
- Analyze message sizes

**Goals**:
- Identify who is syncing with whom
- Determine what is being synced
- Build social graph

**Defenses**:
- Cover traffic (PlanetServe)
- Timing jitter
- Message padding
- Onion routing

### 2. Active Attacker (Medium Capability)

**Motivation**: Disruption, theft, fraud
**Capabilities**:
- Send malicious messages
- Create fake identities (Sybil attack)
- Replay old transactions
- Flood network with traffic

**Goals**:
- Double-spend credit
- Corrupt shared documents
- Exhaust system resources
- Disrupt consensus

**Defenses**:
- Transaction ID tracking
- CRDT validation
- Reputation tiers
- Resource limits
- BFT consensus

### 3. Byzantine Peer (Medium-High Capability)

**Motivation**: Disruption, fraud
**Capabilities**:
- Participate in consensus with malicious votes
- Send conflicting votes to different peers
- Delay or drop messages
- Collude with other Byzantine peers

**Goals**:
- Disrupt credit reconciliation
- Approve fraudulent transactions
- Break consensus

**Defenses**:
- 3f+1 BFT committee
- > 2/3 quorum
- Cryptographic signatures
- Timeout protection

### 4. Eclipse Attacker (High Capability)

**Motivation**: Isolation, control
**Capabilities**:
- Control many peer nodes
- Flood target with connections
- Block connections to honest peers

**Goals**:
- Isolate target node from honest network
- Feed false information to target
- Prevent target from syncing

**Defenses**:
- Multi-source discovery (mDNS + DHT + relay)
- Connection limits
- Peer reputation
- Periodic peer refresh

### 5. Global Passive Adversary (Very High Capability)

**Motivation**: Mass surveillance
**Capabilities**:
- Observe all network traffic
- Correlate timing across the entire network
- Control multiple relays

**Goals**:
- Deanonymize all users
- Build complete social graph
- Decrypt message contents

**Defenses**:
- Onion routing (PlanetServe)
- S-IDA fragmentation
- End-to-end encryption
- **Note**: Complete defense against GPA is not possible

---

## Attack Surface

### 1. Network Layer

**Attack Vectors**:
- Man-in-the-middle (MITM)
- Traffic analysis
- DoS/DDoS
- Eclipse attack

**Security Controls**:
- QUIC encryption (TLS 1.3)
- Multi-source discovery
- Rate limiting
- Connection limits

**Risk Level**: Medium (mitigated by encryption and discovery)

### 2. CRDT Operations

**Attack Vectors**:
- Corrupted operations
- Invalid actor IDs
- Out-of-order sequences
- Malformed patches

**Security Controls**:
- Automerge validation
- Actor ID verification
- Sequence number checking
- Patch validation

**Risk Level**: Low (strong validation in place)

### 3. Mutual Credit

**Attack Vectors**:
- Double-spend
- Replay attack
- Sybil attack (credit inflation)
- Overdraft exploitation

**Security Controls**:
- Transaction ID tracking
- Nonce mechanism
- Reputation tiers
- BFT reconciliation

**Risk Level**: Low (multiple layers of defense)

### 4. Identity System

**Attack Vectors**:
- Key compromise
- Impersonation
- UCAN forgery
- Delegation abuse

**Security Controls**:
- Ed25519 signatures
- Key rotation
- UCAN expiration
- Delegation limits

**Risk Level**: Medium (depends on key management)

### 5. P2P Discovery

**Attack Vectors**:
- Eclipse attack
- Sybil nodes
- Fake peers
- Poisoned DHT

**Security Controls**:
- Multi-source discovery
- Peer reputation
- Connection limits
- Periodic refresh

**Risk Level**: Medium (mitigated but requires vigilance)

### 6. Resource Management

**Attack Vectors**:
- Memory exhaustion
- CPU exhaustion
- Bandwidth exhaustion
- Storage exhaustion
- Connection exhaustion

**Security Controls**:
- Document size limits (10MB)
- Operation limits (10k/sync)
- Memory limits (50MB/doc)
- Connection limits (100)
- Rate limiting

**Risk Level**: Low (comprehensive limits)

---

## Threat Scenarios

### Scenario 1: Replay Attack on Mutual Credit

**Attacker**: Active Attacker
**STRIDE**: Tampering, Elevation of Privilege

**Attack Sequence**:
1. Alice pays Bob $10.00 (transaction TX-001)
2. Attacker intercepts transaction TX-001
3. Attacker replays TX-001 to spend Alice's credit twice

**Impact**: Alice loses $10.00, Bob gains $10.00 twice (double-spend)

**Mitigation**:
- Transaction ID uniqueness (UUID)
- Transaction log deduplication
- Nonce mechanism (monotonic counter)

**Status**: ✅ **MITIGATED** (replay detected in < 10ms)

---

### Scenario 2: Sybil Attack on Credit System

**Attacker**: Active Attacker
**STRIDE**: Elevation of Privilege

**Attack Sequence**:
1. Attacker creates 1000 fake identities (Sybil_0 to Sybil_999)
2. Sybils cross-credit each other
3. Attacker attempts to inflate credit balance

**Impact**: Attacker gains $X in fraudulent credit

**Mitigation**:
- Reputation tier system
- New identities start at Tier 0 ($1.00 limit)
- Reputation requires time + transaction history

**Status**: ✅ **MITIGATED** (1000 Sybils = $1000 max, negligible impact)

---

### Scenario 3: Corrupted CRDT Operations

**Attacker**: Byzantine Peer
**STRIDE**: Tampering

**Attack Sequence**:
1. Malicious peer joins network
2. Peer sends corrupted Automerge operations
3. Honest peers receive corrupted operations

**Impact**: Shared documents become corrupted

**Mitigation**:
- Automerge operation validation
- Actor ID verification
- Sequence number checking
- Peer flagging on repeated violations

**Status**: ✅ **MITIGATED** (corrupted operations rejected)

---

### Scenario 4: Eclipse Attack on Target Node

**Attacker**: Eclipse Attacker
**STRIDE**: Denial of Service

**Attack Sequence**:
1. Attacker controls 50 malicious nodes
2. Malicious nodes flood target with connection requests
3. Target's connection slots fill with malicious peers
4. Target isolated from honest network

**Impact**: Target cannot sync with honest peers

**Mitigation**:
- Multi-source discovery (mDNS, DHT, relay)
- Connection limits per peer
- Peer reputation prioritization
- Periodic peer refresh (every 30s)

**Status**: ✅ **MITIGATED** (target maintains 3+ honest connections)

---

### Scenario 5: Timing Attack on Privacy Sync

**Attacker**: Passive Observer
**STRIDE**: Information Disclosure

**Attack Sequence**:
1. Alice and Bob sync 100 messages
2. Attacker observes network traffic
3. Attacker analyzes timing patterns
4. Attacker infers Alice-Bob sync correlation

**Impact**: Privacy violation (who syncs with whom revealed)

**Mitigation**:
- Cover traffic (10 msgs/min in privacy-max)
- Timing jitter (0-500ms random delay)
- Message padding (hide content size)
- Onion routing (hide sender-receiver)

**Status**: ✅ **MITIGATED** (correlation infeasible)

---

### Scenario 6: Resource Exhaustion

**Attacker**: Active Attacker
**STRIDE**: Denial of Service

**Attack Sequence**:
1. Attacker sends 100MB document (exceeds limit)
2. Honest node attempts to process
3. Node runs out of memory

**Impact**: Node crashes or becomes unresponsive

**Mitigation**:
- Document size limit (10MB)
- Memory limit per document (50MB)
- Pre-allocation checks
- Early rejection

**Status**: ✅ **MITIGATED** (oversized documents rejected in < 10ms)

---

### Scenario 7: Byzantine Voting in BFT Committee

**Attacker**: Byzantine Peer
**STRIDE**: Tampering, Elevation of Privilege

**Attack Sequence**:
1. Credit reconciliation committee of 7 nodes (3f+1 where f=2)
2. 2 Byzantine nodes vote for incorrect balance
3. 5 honest nodes vote for correct balance

**Impact**: If > f Byzantine faults, consensus could fail

**Mitigation**:
- 3f+1 committee size
- > 2/3 quorum (5/7 votes)
- Cryptographic signature verification
- Timeout after 10s

**Status**: ✅ **MITIGATED** (consensus achieved with ≤ f faults)

---

## Security Controls

### Layer 1: Network Security

| Control | Type | Effectiveness |
|---------|------|---------------|
| QUIC/TLS encryption | Cryptographic | High |
| Multi-source discovery | Architectural | High |
| Connection limits | Resource | Medium |
| Rate limiting | Resource | Medium |

### Layer 2: CRDT Security

| Control | Type | Effectiveness |
|---------|------|---------------|
| Operation validation | Validation | High |
| Actor ID verification | Validation | High |
| Sequence checking | Validation | High |
| Peer flagging | Behavioral | Medium |

### Layer 3: Credit Security

| Control | Type | Effectiveness |
|---------|------|---------------|
| Transaction ID tracking | Architectural | High |
| Nonce mechanism | Cryptographic | High |
| Reputation tiers | Behavioral | High |
| BFT reconciliation | Consensus | High |

### Layer 4: Privacy Security

| Control | Type | Effectiveness |
|---------|------|---------------|
| Cover traffic | Obfuscation | Medium-High |
| Timing jitter | Obfuscation | Medium |
| Onion routing | Cryptographic | High |
| S-IDA fragmentation | Cryptographic | High |

### Layer 5: Resource Security

| Control | Type | Effectiveness |
|---------|------|---------------|
| Size limits | Resource | High |
| Memory limits | Resource | High |
| Rate limits | Resource | High |
| Deduplication | Optimization | High |

---

## Residual Risks

### 1. Global Passive Adversary (GPA)

**Risk Level**: Medium
**Impact**: Privacy violation (mass surveillance)
**Likelihood**: Low (requires nation-state resources)

**Mitigation Status**: Partial
- Onion routing helps but doesn't eliminate GPA threat
- S-IDA fragmentation provides some defense
- End-to-end encryption protects content

**Recommendation**: Use privacy-max mode for sensitive operations

---

### 2. k-of-n Relay Collusion

**Risk Level**: Low
**Impact**: Message reconstruction (S-IDA bypass)
**Likelihood**: Low (requires controlling k+ relays)

**Mitigation Status**: Partial
- n=7, k=5 for critical data requires controlling 5/7 relays
- Relay selection algorithm chooses diverse relays
- Regular relay rotation

**Recommendation**: Use n=7, k=5 for high-security scenarios

---

### 3. Endpoint Compromise

**Risk Level**: High
**Impact**: Complete device compromise
**Likelihood**: Low-Medium (depends on user practices)

**Mitigation Status**: Out of scope
- Protocol-layer security cannot prevent endpoint compromise
- Device-level security required (full disk encryption, secure boot)
- Hardware security modules (HSMs) recommended for high-value keys

**Recommendation**: Follow device security best practices

---

### 4. Social Engineering

**Risk Level**: Medium
**Impact**: Key compromise, phishing
**Likelihood**: Medium

**Mitigation Status**: Out of scope
- User education required
- UCAN delegation limits reduce impact
- Key rotation reduces window of compromise

**Recommendation**: User training and awareness programs

---

## Assumptions

This threat model makes the following assumptions:

### Trusted Components

1. **Local device**: User's device is not compromised
2. **Cryptographic primitives**: Ed25519, ChaCha20-Poly1305, Blake3 are secure
3. **Automerge library**: Automerge CRDT implementation is correct
4. **Rust compiler**: Rust toolchain is not compromised

### Honest Majority

1. **BFT committees**: Honest majority (> 2/3) in credit reconciliation
2. **Network peers**: Majority of peers are honest
3. **Relay nodes**: Majority of relays are honest

### Operational

1. **Key management**: Users protect their private keys
2. **Software updates**: Users install security updates
3. **Network connectivity**: Basic internet connectivity available

---

## Conclusion

The VUDO Runtime threat model identifies and mitigates major threats through a defense-in-depth strategy:

1. **Network layer**: Encryption, discovery, limits
2. **CRDT layer**: Validation, verification
3. **Credit layer**: Tracking, reputation, consensus
4. **Privacy layer**: Obfuscation, fragmentation, routing
5. **Resource layer**: Limits, deduplication

**Residual risks** (GPA, collusion, endpoint compromise) require additional operational controls and user practices.

**Overall Risk Level**: **LOW** for honest users following best practices.

---

**Document Owners**: VUDO Security Team
**Last Review**: 2026-02-05
**Next Review**: 2026-05-05 (Quarterly)
