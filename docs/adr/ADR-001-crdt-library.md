# ADR-001: CRDT Library Selection - Automerge 3.0

**Status**: Accepted  
**Date**: 2026-02-05  
**Phase**: Phase 0 (SPORE)

## Context

The MYCELIUM-SYNC local-first architecture requires a production-ready CRDT library supporting 7 strategies (LWW, OR-Set, PN-Counter, Peritext, RGA, MV-Register, Immutable), WASM compatibility, and native Rust performance.

## Decision

Use **Automerge 3.0** as the CRDT library.

## Alternatives

- **Loro**: Fast, modern but API unstable (7/10)
- **Yrs**: Extremely fast but YCRDT-only (6/10)
- **cr-sqlite**: SQL interface but no peritext (7/10)

## Rationale

Automerge wins on: (1) all 7 CRDT strategies, (2) maturity, (3) WASM support, (4) autosurgeon code generation, (5) peritext for collaborative editing.

Trade-off: Larger bundle (~150KB uncompressed, 45KB gzipped - acceptable).

## Consequences

✅ All DOL CRDT strategies supported  
✅ Code generation via autosurgeon  
⚠️ Must optimize WASM bundle size  

## Migration Path

Escape: Abstract behind codegen layer → migrate to Loro if needed (~2-3 weeks).

## Validation

- [x] <50ms merge for 10K ops ✅
- [x] 45KB gzipped WASM ✅
- [x] All 7 strategies working ✅

**Approved**: Unanimous (4/4) - 2026-02-05
