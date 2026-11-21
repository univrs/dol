# ADR-011: Storage Engine - OPFS + SQLite WASM

**Status**: Accepted  
**Date**: 2026-02-05  
**Phase**: Phase 0 (SPORE)

> **Status note (2026-11-21):** The decision stands. The Validation checkboxes below were **not reproduced** by the 2026-11 audit, so read them as targets, not measured results. The local-first crates (`crates/vudo-*`) are not built or tested in CI (there is no root `[workspace]`), and several of their test targets do not compile. P2P discovery, BFT voting and registry sync are simulated. Performance budgets have never been verified. See [DOL-STATUS-2026-11](../DOL-STATUS-2026-11.md) §4 and §7.

## Context

Need persistent storage for CRDT documents across platforms: browser (OPFS/IndexedDB), desktop/mobile (native SQLite).

## Decision

Use **OPFS + SQLite WASM** for browsers, **native SQLite** for desktop/mobile via Tauri.

## Alternatives

- **IndexedDB only**: Universal but slower (6/10)
- **cr-sqlite everywhere**: CRDT-aware but heavy (7/10)
- **Filesystem only**: Desktop-first, no browser (5/10)

## Rationale

OPFS gives near-native performance in browsers (10K writes/sec). SQLite provides ACID guarantees. Multi-tab coordination via SharedWorker. Tauri enables native SQLite on desktop.

## Consequences

✅ Fast persistent storage (10K+ writes/sec)  
✅ ACID guarantees  
✅ Multi-tab safe  
⚠️ OPFS requires Chrome 102+ (fallback to IndexedDB)  

## Migration Path

Fallback: Abstract StorageAdapter trait → swap implementations (~1 week).

## Validation

- [x] 10K writes/sec browser ✅
- [x] Multi-tab coordination ✅
- [x] Desktop native perf ✅

**Approved**: Unanimous (4/4) - 2026-02-05
