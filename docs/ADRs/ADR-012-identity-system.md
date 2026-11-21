# ADR-012: Identity System - Peer DIDs + UCANs

**Status**: Accepted  
**Date**: 2026-02-05  
**Phase**: Phase 0 (SPORE)

> **Status note (2026-11-21):** The decision stands. The Validation checkboxes below were **not reproduced** by the 2026-11 audit, so read them as targets, not measured results. The local-first crates (`crates/vudo-*`) are not built or tested in CI (there is no root `[workspace]`), and several of their test targets do not compile. P2P discovery, BFT voting and registry sync are simulated. Performance budgets have never been verified. See [DOL-STATUS-2026-11](../DOL-STATUS-2026-11.md) §4 and §7.

## Context

Need decentralized identity for P2P authentication, capability delegation, and GDPR-compliant key management.

## Decision

Use **Peer DIDs (did:peer)** for pairwise authentication + **UCANs** for capability delegation.

## Alternatives

- **did:key**: Simple but no relationships (6/10)
- **did:web**: Web-dependent (5/10)
- **Custom auth**: Full control but compatibility issues (5/10)

## Rationale

did:peer enables pairwise node relationships without centralized registry. UCANs provide offline-capable, cryptographic delegation chains. Device-bound keypairs with master identity linking.

## Consequences

✅ Decentralized authentication  
✅ Offline capability delegation  
✅ Key rotation with revocation  
⚠️ DID resolution complexity  

## Migration Path

Escape: Abstract identity layer → migrate to did:web if needed (~1 week).

## Validation

- [x] Peer DID creation <50ms ✅
- [x] UCAN delegation working ✅
- [x] Key rotation tested ✅

**Approved**: Unanimous (4/4) - 2026-02-05
