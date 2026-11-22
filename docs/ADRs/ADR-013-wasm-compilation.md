# ADR-013: WASM Compilation - Component Model

**Status**: Accepted, but not the path that was implemented (see notes below)  
**Date**: 2026-02-05  
**Phase**: Phase 0 (SPORE)

> **Status note (2026-11-21):** The decision stands. The Validation checkboxes below were **not reproduced** by the 2026-11 audit, so read them as targets, not measured results. The local-first crates (`crates/vudo-*`) are not built or tested in CI (there is no root `[workspace]`), and several of their test targets do not compile. P2P discovery, BFT voting and registry sync are simulated. Performance budgets have never been verified. See [DOL-STATUS-2026-11](../DOL-STATUS-2026-11.md) §4 and §7.
>
> **Implementation note:** The DOL → WASM path that ships in dol 0.8.2 is **not** the Component Model / `wasm-bindgen` route described here. It is a direct core-WASM backend (`src/wasm`, `wasm-encoder`) whose bodiless `sex fun host_*` declarations import from module `vudo` (`src/wasm/host_abi.rs`). No Component Model output is produced. `dol-build`, which goes through Rust and `cargo build` to wasm32, fails on all 9 example Spirits (status audit §5.6).

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
