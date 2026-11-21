# VUDO/DOL: Project Summary

> **Crate:** `dol` 0.8.2 (library `metadol`), released 2026-11-20
> **Status:** pre-1.0. The parser and toolchain work, and the WASM backend is beta. 1.0 criteria are not yet written down.
> **Authoritative status:** [DOL-STATUS-2026-11.md](DOL-STATUS-2026-11.md)

This page replaces an earlier summary that described the project as "Version 1.0.0, Production-Ready" with "1,450+ tests".
Those claims were not accurate. For full detail and evidence, see the status audit linked above.

## What DOL is

DOL (Design Ontology Language) is an ontology-first DSL. You declare what a system *is*: genes (`gen`), traits, rules
(`rule`), systems and evolutions (`evo`), each with a `docs` block. You can then compile it to other targets.
The language also has functions (`fun`), control flow, pattern matching, lambdas, pipes and meta-programming syntax.

**"DOL 2.0".** This is the historical name for the expression-capable generation of the language, introduced around crate
v0.1.0. It is not a version number and has no separate specification. The language is versioned by the `dol` crate.
The current syntax era is v0.8 "Clarity": `gen`/`rule`/`evo`/`docs` and lowercase types.

## What works (verified at v0.8.2)

| Component | Status |
|---|---|
| Lexer / parser / AST | Works. 2,183 tests pass with default features and 2,405 with `cli,serde,wasm` (root crate only). |
| Validator | Runs, but type errors are **not** enforced. |
| HIR + lowering | Works. Not used by the WASM path. |
| Direct WASM backend + VUDO host ABI | Beta. Supports a subset of the language. The 15 `vudo.host_*` imports match the VUDO VM linker. |
| Rust / TypeScript / JSON Schema codegen | Works, but quality varies. 14/41 generated Rust files compile standalone. |
| `dol-migrate`, `dol-mcp` | Work. |
| LLVM native backend (`llvm-backend/`) | Beta. Needs a manual LLVM 18 setup and is not in CI. |
| VUDO VM end-to-end | Works in `univrs/vudo` CI: DOL source → WASM → `vudo run`. |

## Known limitations

- `dol-parse` and `dol-check` only parse the **first declaration** in each file.
- Self-hosting is not working. The bootstrap sources in `dol/*.dol` parse, but the generated crate does not compile.
- The local-first crates (`crates/vudo-*`) are not built in CI. P2P discovery, BFT voting and registry sync are simulated.
- MLIR support is stubbed and feature-gated.

## Repository map

| Path | Contents |
|---|---|
| `src/` | Core crate: lexer, parser, AST, validator, HIR, codegen, WASM backend, CLIs (`src/bin/`) |
| `dol-abi/` | Host ABI definitions (`vm_host_functions()`) |
| `crates/dol-*` | Codegen, macro, reflection and exegesis crates |
| `crates/vudo-*` | Local-first runtime libraries (state, storage, p2p, identity, privacy, credit, ai, planetserve) |
| `llvm-backend/` | LLVM native backend (beta) |
| `dol/` | Self-hosting bootstrap sources (experimental) |

## Quick start

```bash
git clone https://github.com/univrs/dol.git && cd dol
cargo test
cargo run --features cli --bin dol-parse -- examples/genes/counter.dol
```

See [QUICKSTART.md](QUICKSTART.md) and [tutorials/](tutorials/).
