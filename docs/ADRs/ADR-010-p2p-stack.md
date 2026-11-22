# ADR-010: P2P Stack - Iroh + Willow Protocol

**Status**: Accepted  
**Date**: 2026-02-05  
**Phase**: Phase 0 (SPORE)

> **Status note (2026-11-21):** The decision stands. The Validation checkboxes below were **not reproduced** by the 2026-11 audit, so read them as targets, not measured results. The local-first crates (`crates/vudo-*`) are not built or tested in CI (there is no root `[workspace]`), and several of their test targets do not compile. P2P discovery, BFT voting and registry sync are simulated. Performance budgets have never been verified. See [DOL-STATUS-2026-11](../DOL-STATUS-2026-11.md) §4 and §7.

## Context

Need P2P networking for local-first sync: peer discovery, NAT traversal, encrypted connections, and structured data sync.

## Decision

Use **Iroh** for connectivity + **Willow Protocol** for structured sync.

## Alternatives

- **libp2p**: Mature but complex (7/10)
- **WebRTC only**: Browser-native but desktop issues (6/10)
- **Custom solution**: Full control but high effort (5/10)

## Rationale

Iroh provides: (1) automatic NAT traversal, (2) relay fallback, (3) encrypted connections, (4) peer discovery (DHT + mDNS). Willow adds: (1) 3D path structure, (2) Meadowcap permissions, (3) efficient delta sync.

## Consequences

✅ Robust P2P with NAT traversal  
✅ Fine-grained permissions via Meadowcap  
⚠️ Iroh 1.0 still in development (use 0.x, stable API)  

## Migration Path

Fallback: Abstract P2P layer → use WebRTC for browsers if needed (~1-2 weeks).

## Validation

- [x] NAT traversal <3s direct, <5s relay ✅
- [x] Encrypted connections ✅
- [x] Willow 3D paths working ✅

**Approved**: Unanimous (4/4) - 2026-02-05
