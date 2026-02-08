# ADR-005: WASM Compilation - Component Model

**Status**: Accepted  
**Date**: 2026-02-05  
**Phase**: Phase 0 (SPORE)

## Context

Need DOL → WASM compilation for browser and WASM runtime execution. Must support modular composition and small bundle sizes.

## Decision

Use **WASM Component Model** with Rust → WASM compilation via `wasm-pack` and `wasm-bindgen`.

## Alternatives

- **Core WASM only**: Smaller but no composition (6/10)
- **WASI-only**: Runtime focus, limited browser (5/10)
- **AssemblyScript**: Simpler but less mature (6/10)

## Rationale

Component Model enables modular DOL Gene composition. Rust → WASM gives native performance. `wasm-opt` achieves <100KB gzipped. WIT interfaces map directly to DOL Traits.

## Consequences

✅ Modular Gene composition  
✅ <100KB gzipped per module  
✅ Native Rust performance  
⚠️ Component Model still W3C Phase 2 (use jco transpiler for browsers)  

## Migration Path

Fallback: Target core WASM for browsers, Component Model for native (~1 week).

## Validation

- [x] <100KB gzipped WASM ✅
- [x] Browser + native working ✅
- [x] WIT → TypeScript types ✅

**Approved**: Unanimous (4/4) - 2026-02-05
