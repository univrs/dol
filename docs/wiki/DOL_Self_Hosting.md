# DOL Self-Hosting: Status

> **DOL is not self-hosting.** The compiler is the Rust crate in `src/`. A DOL-in-DOL
> compiler is a research goal: its sources exist, but the code generated from them does not compile.
> Verified at dol 0.8.2. Source: [DOL-STATUS-2026-11](../DOL-STATUS-2026-11.md) §5.8.

## What exists

| Item | Status |
|------|--------|
| Bootstrap sources `dol/*.dol` | 10 files, ≈9.9k lines: `ast`, `bootstrap`, `codegen`, `lexer`, `main`, `mod`, `parser`, `token`, `typechecker`, `types`. **All 10 parse** with the Rust parser (whole-file parse via `metadol::parse_file_all`). |
| Rust crate generated from those sources | **Fails to compile, with 850 errors.** |
| CHANGELOG 0.2.2 milestone ("ast.dol, token.dol and bootstrap.dol … compiles successfully, 0 errors") | **No longer holds.** The same subset now gives 19 errors. |
| `stage1/` | Contains only a `Cargo.toml` (package `dol-stage1`), with no sources. |
| `stage2/` | Does not exist. |
| `--features self-hosting` | Does not exist. `Cargo.toml` defines no such feature. |

## The intended bootstrap (not achieved)

```text
Stage 0: Rust implementation (src/)            exists and is the real compiler
Stage 1: dol/*.dol compiled to Rust by Stage 0 generated crate does not compile
Stage 2: Stage 1 compiles dol/*.dol            not started
Stage 3: Stage 2 output == Stage 1 output      not started
```

## Trying it yourself

The bootstrap sources can be parsed and fed to the Rust code generator:

```bash
# Parse-check the bootstrap sources (note: dol-parse only reads the first
# declaration in each file; see DOL-STATUS-2026-11 §5.1)
cargo run --features cli --bin dol-parse -- dol/

# Generate a Rust crate from them (default output dir: stage2/).
# The generated crate does not compile today.
cargo run --features cli --bin dol-build-crate -- dol/*.dol --output /tmp/dol-stage1
```

## What would make it real

From the status audit (§9, P3): either restore the `stage1/` sources and add an honest bootstrap test to CI, or relabel self-hosting as research. The roadmap's earlier target date for self-hosting (March 2026) was missed.
