# DOL Language Status — Ground Truth (2026-11-20)

> Audit of `univrs/dol` at **origin/main `117e236` = tag v0.8.2 = crates.io `dol` 0.8.2**, plus consumer repos
> (univrs-vudo, univrs-network, vudo.univrs.io, nexus, orchestration, enr), learn.univrs.io, book.univrs.io and the
> published crate. "Verified" means a command was run or code was read at that commit; "Inferred" is marked.
>
> Note: the local checkout `~/repos/univrs-dol` `main` was 9 commits behind origin (missing the whole v0.8.2 / H0 work)
> at audit time. Pull before relying on it.

---

## 1. TL;DR

| Question | Answer |
|---|---|
| Latest DOL release | **0.8.2** (crates.io, 2026-11-20). "Spirit Hosting H0 — toolchain truth": DOL-compiled Spirits link and run in the VUDO VM. |
| What is "DOL 2.0"? | **A language-generation label, not a version.** It was coined at crate v0.1.0 (CHANGELOG) for "DOL + functional programming, pattern matching, control flow, meta-programming" (`docs/tutorials/05-dol2-features.md:5`). DOL 1.x was the purely declarative ontology language (gene/trait/constraint/system/evolves). No "DOL 2.0 spec" document exists; ADR-003 links to `docs/02-DOL-2.0-SPECIFICATION.md`, but that file does not exist. |
| What is v0.8.2? | **The crate/toolchain semver**, and the only versioning that is enforced. Syntax era = **v0.8 "Clarity"**: `gen`/`rule`/`evo`/`docs` keywords and lowercase Rust-style types. The old forms still parse with warnings and become errors at v0.9. |
| Is there a formal spec in production? | **No.** `docs/grammar.ebnf` says "Version 0.0.1". It only describes DOL 1.x declarations and says DOL has "no expression operators". **The de facto spec is `src/lexer.rs` + `src/parser.rs` at v0.8.2 plus CHANGELOG.md.** |
| What actually runs in production? | **Only `univrs/vudo` (VUDO VM + `vudo` CLI) uses DOL as a compiler**: it pins `dol = "0.8.2"` (`wasm-compile`) from crates.io, and CI compiles a DOL `sex fun host_log` program to WASM and runs it in the VM (`e2e_spirit_lifecycle::test_h0_*`, green 2026-11-20). |
| vudo.univrs.io | Ships an **in-browser dol 0.8.0** parser/compiler (`dol_wasm_bg.wasm`, built 2026-01-17). It cannot run 0.8.2 host-calling Spirits because there is no `vudo` import module in the worker. The UI says "DOL v0.8.0", "Built with DOL v0.8.1" and "DOL 2.0" in different places. |
| nexus.univrs.io | **No DOL dependency at all.** The site returns 401 behind Cloudflare. The nexus repo uses OWL/TTL ontologies; archived docs mention DOL v0.4/v0.5. |
| DOLOS | It is a naming/architecture layer (L6, renamed from "VUDO OS" in PR #33). The `univrs-network` Spirit host runs `vudo_vm` but **does not depend on the DOL compiler**; its tests use hand-written WAT. **DOLOS does not consume DOL yet.** |
| Is the crates.io README accurate? | **No, it is the least accurate public source.** It is the never-updated "DOL 2.0" README with v0.7-era syntax, a nonexistent `dol build` command, nonexistent `--format rust/typescript` flags, wrong MCP tool names, wrong `use dol::` paths, stale test counts, broken links, and no mention of the 0.8.2 changes. Details in §6. |

---

## 2. Versioning: three numbering systems that got conflated

| Label | Meaning | Where used |
|---|---|---|
| **DOL 1.x** (implicit) | Declarative ontology only: `gene`, `trait`, `constraint`, `system`, `evolves`, `exegesis` | `docs/grammar.ebnf` (v0.0.1), most `ontology/*.dol` across repos |
| **DOL 2.0** | Language generation with expressions, `fun`, `match`, lambdas, pipes, quote/eval, `sex` effects | README title, tutorials, vudo.univrs.io status bar. No spec, no version gate. |
| **Crate semver 0.x.y** | The real, shipped toolchain. 0.8.x = "Clarity" keyword set. | crates.io, CHANGELOG, git tags |
| "DOL v0.9.0" | Some Spirit examples claim this version | `examples/spirits/*`. **Unreleased; mislabeled.** |

CHANGELOG defects:
- 0.8.0 is dated **2025**-01-17, but crates.io shows **2026**-01-17.
- There is **no 0.8.1 entry**, though crates.io has 0.8.1 (2026-01-26).
- Deprecation warnings say "deprecated in v0.8.0", but ADR-003 says v0.8.1.

**Recommendation:** retire "DOL 2.0" as a public label (or define it formally as "the language implemented by crate ≥0.8"), and version the *language spec* separately from the crate once a spec exists.

---

## 3. What's REAL (verified at v0.8.2)

### 3.1 The language as the parser actually accepts it
**Top-level declarations** (`src/parser.rs:384-501`): `gen`/`gene`, `type`, `trait`, `rule`/`constraint`, `system`, `evo`/`evolves`, `sex`, `fun`, `const`, `docs`/`exegesis`, `use`, `mod`.

**Verified to parse:**
- **Ontology:**
  - `gen` with `has`/`is`/`derives from`/`requires`/`uses`/`emits`
  - typed fields (`has x: u64 = …`, `Vec<T>`, `Option<T>`)
  - `trait`, `rule`, `system`, `evo X @ v > v`
  - `gen X extends Y`, `has k: enum {…}`, `type X {…}`
  - `law name(params) {…}`
- **Functions:**
  - `fun` with `if/else`, `match` (with guards), `for i in 0..10`, `while`, `loop/break`, `return`
  - lambdas `|x: i64| x+1`, the `|>` and `>>` operators, `?` try
  - method calls, struct literals, tuples, function types `(i64) -> bool`, `::` paths
- **Effects:** `sex fun`, `sex var`, `sex { }` blocks, `const`, `val`
- **Modules:** `module x @ ver`, `use a.b`, `use a::{X,Y}`
- **Meta:** `'quote`, `!eval`, `?Type` reflect, `[| |]` idiom brackets, `#macro(...)`, `#[attr]`
- **CRDT:** `@crdt(lww|or_set|…)` field annotations, which produce a real `CrdtAnnotation` in the AST
- **Visibility:** `pub`, `pub(spirit)`, `pub(crate)`. These are *accepted* but dropped (see §5).

**Types:** `i8…i128`, `u8…u128`, `f32`, `f64`, `bool`, `string`, `()`, `Vec<T>`, `Option<T>`. Legacy `Int32`/`String`/`Void`/… still accepted with warnings.

### 3.2 Toolchain components

| Component | Status | Evidence |
|---|---|---|
| Lexer / parser / AST | REAL | 2,512 tests pass with `--features cli,serde,wasm`, including doctests (0 failures). |
| Validator | REAL but shallow | Runs, but **type errors are not enforced** (§5). |
| HIR + lowering | REAL | Lowers everything that parses. **Not used by the WASM path**: `src/compiler/spirit.rs:303` keeps it "for diagnostics only". |
| Direct WASM backend + VUDO host ABI | **REAL (beta)**, the 0.8.2 headline | `src/wasm/host_abi.rs`: 15 `vudo.host_*` imports, **name and signature parity verified** against `univrs-vudo/vudo/vudo_vm/src/linker.rs`. 114 + 163 WASM tests. Covers a subset of the language (§5). |
| Rust / TypeScript / JSON-Schema codegen (`dol-codegen`) | REAL, quality varies | TS and JSON Schema: 22/23 files generate. Rust output: **only 14/41 compile standalone** with `rustc`. |
| `dol-migrate` | REAL | `0.7-to-0.8`, `0.2-to-0.3` |
| `dol-mcp` (MCP server) | REAL | JSON-RPC stdio. Tools: `parse`, `typecheck`, `compile_rust`, `compile_wasm`, `list_macros`, `expand_macro`, `recommend_crdt`, … |
| Macros / reflection | REAL in core | `src/macros` (36 tests), `src/reflect.rs` (17 tests) |
| LLVM native (`llvm-backend/`, `dol-native`) | REAL (beta), out of CI | 15/15 tests pass; 12/12 `examples/native` build, link and run. Needs a manual LLVM-18 setup and is not in CI. |
| VUDO VM end-to-end | **REAL** | `univrs-vudo` CI: DOL source → WASM → `vudo run` → `[VUDO:INFO] hello from DOL`. H1a tests run DOL-built loops under fuel limits. |

---

## 4. What's IN-FLIGHT

| Item | State |
|---|---|
| **Spirit Hosting plan** | H0 shipped in dol 0.8.2. H1a/H1c work is landing in `univrs-vudo` / `univrs-network` (fuel limits, Spirit host). **H1+ is not defined anywhere in the dol repo.** |
| **v0.9 deprecation cut** | Planned: old keywords and PascalCase types become errors; `dol-abi::standard_host_functions()` (22 `vudo_*` names) is deprecated in favour of `vm_host_functions()` (15). **Blocker:** 127+ ontology `.dol` files across repos (orchestration 108, vudo 8, enr 6, network 4) are still pre-0.8 syntax and would stop parsing. |
| **DOLOS integration** | Renamed in docs only (PR #33). Network Spirit host runs signed `.spirit` WASM via `vudo_vm`; no DOL compile step in network CI. The rename is not on the live vudo.univrs.io site. |
| **learn.univrs.io v0.8.2 update** | Committed (`7f574e4`), **not deployed**: the live bundle still says v0.8.1. `/dol` returns HTTP 404 status (the SPA shell still renders). |
| **CRDT → Automerge codegen** | Lives in `crates/dol-codegen-rust` (29 tests, string-`contains` assertions). Not wired into any CLI, and its output is never compiled. |
| **Local-first stack** (`vudo-state`, `-storage`, `-p2p`, `-credit`, `-privacy`, `-identity`, `planetserve`, `vudo-ai`) | Libraries mostly pass their unit tests, but **none are in CI** (no root `[workspace]`). Several test targets don't compile. P2P discovery, BFT voting and registry sync are simulated. |
| **MLIR** | `src/mlir` ≈2.9k lines, 48 stub markers. Feature-gated, doesn't build without MLIR headers, not in CI. |

---

## 5. What's broken or misleading in the shipped toolchain (verified)

1. **`dol-parse` and `dol-check` only see the first declaration in a file.**
   - The call path is `parse_file` (`src/lib.rs:250`) → `Parser::parse()`, which returns after the first declaration (`src/parser.rs:83-91`: "For now, we just return the first declaration").
   - Everything after it is silently ignored, including garbage. This inflates every CLI pass rate. Real whole-file parse rates: spirits **12/53** (CLI reports 39/53), stdlib 1/8, v0.8 examples 1/4, tests/corpus 1/5.
   - `parse_file_all` exists but the CLIs don't use it.
2. **No type enforcement.**
   - `fun f() -> i64 { return "hello" + true }` passes validate, `dol-check --typecheck`, HIR lowering and WASM compilation.
   - `validate_statement_types` is an empty placeholder (`src/validator.rs:741-772`).
3. **Parsed but discarded:**
   - `pub` (the AST always records `Private`)
   - generic params on gen/trait (`type_params` is always `None`)
   - a `rule` nested in a `gen` (its body is dropped)
   - `mod x { … }` contents (skipped)
4. **Documented but not parseable:**
   - `let mut`, `+=`, `&T`, `[T]`, `fun id<T>(…)`
   - `impl`, top-level `spirit`, `extern`, `provides`
   - `evo A > B` with `added`, `type X is enum`
   - `with IO` effects, backtick/`~` quasi-quote
   - `gene name @1.0.0 {` (versioned header used throughout tutorial 02)
   - float exponents `1e9`
5. **WASM backend gaps:**
   - Unsupported: lambdas, tuples, lists, `sex {}` blocks, `?`, quote/meta, idioms, `this`, casts, and `for` over anything other than a range.
   - Gene layouts reject `u64` and `string`.
   - `println` is unknown, so `examples/native` compiles 0/12 to WASM.
   - Declarative-only files compile to an empty 17-byte module.
6. **`dol-build`** (Spirit → Rust → wasm32) fails on **all 9** example Spirits.
7. **`vudo` bin doesn't compile** (non-exhaustive match at `src/bin/vudo.rs:553`).
8. **Self-hosting is not real:**
   - The bootstrap sources exist: `dol/*.dol`, 10 files, ≈9.9k lines, all of which parse.
   - The crate generated from them fails with **850 compile errors**.
   - The CHANGELOG 0.2.2 "0 errors" claim now gives 19 errors.
   - `stage1/` contains only a `Cargo.toml`, and `stage2/` doesn't exist.
9. **CI blind spot:**
   - There is no root `[workspace]`, so the 16 `crates/*`, `apps/*`, `llvm-backend` and `benchmarks` are not built in CI.
   - Silently broken there: `dol-codegen-wit`, `apps/gen-registry` (manifest error), `benchmarks`, several crate test targets.
   - The Performance CI workflow has failed 10 runs in a row.
10. **Published crate hygiene:**
    - `dol-0.8.2.crate` ships 734 files (1.8 MB) including `CLAUDE.md`, `TASK_*.md`, `prototypes/`, `apps/`, `add.wasm`.
    - All bins need `--features cli`, so `cargo install dol` likely installs nothing (inferred from Cargo.toml).

---

## 6. Public documentation accuracy

| Claim | crates.io README | learn.univrs.io (live) | book.univrs.io | Truth (v0.8.2 code) |
|---|---|---|---|---|
| Version | "DOL 2.0", v0.8.0/0.8.1 refs | v0.8.1 | changelog stops at 0.8.0; intro says v0.2.3 | 0.8.2 |
| Keywords | `gene`/`constraint`/`exegesis`/`evolves` only | `gen`/`rule`/`docs`/`evo` ✓ | `gen`/`rule`/`docs`/`evo` ✓ | gen/rule/evo/docs (old = deprecated) |
| Types | `UInt64`, `String`, `Void` | lowercase ✓ | lowercase ✓ | lowercase |
| CLI | `dol build --target rust` (**no `dol` binary exists**); `--format rust/typescript` (**errors**) | mostly right | mostly right, documents `--tree-shake` | dol-parse, dol-check, dol-test, dol-codegen, dol-migrate, dol-mcp, dol-build, dol-build-crate |
| Install | clone + build | `cargo install dol` (needs `--features cli`) | `cargo install univrs` (wrong) | `cargo install dol --features cli` (inferred) |
| Quick-start output | "2 declarations, 3 fields…" | — | — | Reports 1 declaration; the trait is silently dropped (§5.1) |
| Library path | `use dol::transform`, `DesugarIdiom` | — | — | `metadol::…`, `IdiomDesugar` |
| MCP tool names | `dol/parse`, `dol/compile/rust`… | — | — | `parse`, `compile_rust`, … |
| Self-hosting | "Planned Q4" | — | "Full self-hosting… Stage2 compiles!" | Not real (§5.8) |
| Test count | 1914 | 1,705+ / 1,800+ | 1532 | **2,512** with `cli,serde,wasm`, including doctests (root crate only) |
| VUDO host ABI (0.8.2 headline) | absent | absent | absent | shipped |
| Links | badges → `univrs/metadol`; `docs/specification.md` (missing); `learn.univrs.io/dol` (404) | — | — | — |

**Ranking for syntax accuracy:** book `dol.md` / `releases/v0.8.0.md` ≈ learn > crates.io README.
**The only fully accurate sources** are `CHANGELOG.md` (0.8.2 entry) and the code.

**Overclaiming in-repo docs** (should be corrected or moved to `archive/`):
- `docs/VUDO-DOL-PROJECT-SUMMARY.md` ("Version 1.0.0, Production-Ready")
- `COMPILER-TARGETS.md` (Automerge codegen "Production")
- `TASK_T4_1_PERFORMANCE_COMPLETE.md` (benchmarks don't compile)
- `TASK_T4_2_WORKSPACE_COMPLETE.md` (cited schemas don't exist)
- `apps/gen-registry/IMPLEMENTATION_SUMMARY.md` (crate doesn't build)
- `AI_POWERED_TOOLS_IMPLEMENTATION.md` (NL→DOL is keyword heuristics; LSP is a completion library, not a server)
- `docs/wiki/DOL_Self_Hosting.md` and `docs/tutorials/HIR_Tutorials.md` ("DOL is self-hosting")
- README roadmap (MLIR ✅)
- Tutorials: `02-writing-genes.md` (0/18 blocks parse), `01-getting-started.md` (1/6), `syntax-reference.md` (old types plus unparseable syntax), ADR-003 examples (0/2)

---

## 7. ASPIRATIONAL (docs only / no working code)

- A formal DOL 2.0 language specification (missing file; EBNF frozen at 0.0.1)
- Self-hosting compiler (target "Mar 2026", missed)
- Effect annotations on functions (`with IO`), generics on functions, `impl` blocks, top-level `spirit` declarations, quasi-quote
- `stdlib/` (only `.md` specs: functor/monad/…; no `.dol` or `.rs`)
- Package manager / Mycelium Spirit registry (README "Q5"); `gen-registry` doesn't build
- IDE / LSP server (README "Q6"); only a completion-provider library exists
- Workspace reference app (no Cargo.toml; schemas missing)
- Performance budgets (7 ms merge, 1,500 ops/s): never verified
- Real P2P / BFT / distributed AI inference (simulated)
- DOLOS as a DOL-native OS: currently a VM host with no DOL compilation in its pipeline
- 1.0 criteria: **not written down anywhere**

---

## 8. The goal post (as stated across sources)

**Vision:** DOL is an ontology-first DSL. You declare what a system *is* (genes, traits, rules, evolutions with mandatory `docs`), then compile it to multiple targets (Rust, TS, JSON Schema, WASM, native). DOL-compiled **Spirits** run sandboxed in the **VUDO VM** with capability-gated host functions, as the application layer of **DOLOS**, the local-first, privacy-preserving, P2P "DOL Operating System". CRDT annotations give convergent shared state for free. Long term: self-hosting.

**Milestones as documented:**
1. ✅ H0 "toolchain truth": DOL → WASM → VUDO VM link parity (0.8.2)
2. ⏳ H1+: Spirit lifecycle and fuel in the VM / network host (in `univrs-vudo` and `univrs-network`, undocumented in dol)
3. ⏳ v0.9: remove legacy syntax
4. ⏳ Q4 self-hosting, Q5 package registry, Q6 LSP (README; undated in practice)
5. ❓ 1.0: undefined

---

## 9. Gaps, prioritised

**P0: honesty and correctness of what's shipped**
1. Make `dol-parse`/`dol-check` use `parse_file_all`, and fail on any unparsed trailing content. Every pass rate today is overstated.
2. Rewrite the README for v0.8.2: current keywords and types, the real binary list, `cargo install dol --features cli`, the H0 host ABI, the `metadol` paths, the real MCP tool names, current test counts, and working links. Add `include = [...]` to Cargo.toml to stop shipping internal files.
3. Fix CHANGELOG dates and add the missing 0.8.1 entry.
4. Wire `validate_statement_types` to the typechecker so type errors fail.

**P1: one spec, one truth**
5. Write a real grammar (EBNF) and spec for the v0.8 language as implemented. Version it (e.g. "DOL Language 0.8"), and make tutorials and examples CI-tested against it (doc blocks parsed as whole files).
6. Decide what "DOL 2.0" means, or drop the label from README, tutorials and the vudo.univrs.io status bar.
7. Add a root `[workspace]` (or CI matrix) covering `crates/*`, `apps/*`, `llvm-backend` and `benchmarks`.
8. Deploy learn.univrs.io v0.8.2; update the book changelog and intro; fix the `/dol` 404.

**P2: close the language gaps the VM/DOLOS path needs**
9. Parser: `let mut`, `+=`, generics on `fun`, `impl`, top-level `spirit`, effect rows (`with …`), float exponents, versioned gen headers. Stop silently dropping `pub`, generics, nested rules and `mod` bodies.
10. WASM backend: strings and `u64` in gene layouts, lambdas, tuples, lists, `sex {}` blocks, `?`, `for` over collections. Route through HIR instead of the AST.
11. Rebuild vudo.univrs.io's `dol-wasm` against crates.io `dol = "0.8.2"` (not a sibling path dep), and add a `vudo` import module to the browser worker so H0 Spirits run in-browser.
12. Have DOLOS (`univrs-network`) compile at least one DOL Spirit in CI instead of only hand-written WAT.
13. Migrate the 127+ legacy ontology `.dol` files across repos (`dol-migrate 0.7-to-0.8`) **before** v0.9 makes them errors.

**P3: aspirations**
14. Fix the `vudo` bin, `dol-build` on example Spirits, Rust codegen compile rate (14/41), and the `dol-codegen-wit` / `gen-registry` / `benchmarks` builds.
15. Either restore `stage1/` sources and an honest bootstrap test, or relabel self-hosting as research.
16. Write H1–Hn and **1.0 exit criteria** into this repo (`docs/roadmap.md`).
17. Archive or correct the overclaiming status docs listed in §6.
