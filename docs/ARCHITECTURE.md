# DOL Architecture (dol 0.8.2)

> Describes crate `dol` 0.8.2 (library `metadol`), syntax era v0.8 "Clarity", Rust edition 2021, MSRV 1.84.
> For verified status, test counts and known gaps, see [DOL-STATUS-2026-11.md](DOL-STATUS-2026-11.md).
> This document replaces an earlier v0.8.0 architecture reference. That version listed files that do not exist,
> claimed a `logos`-based lexer, gave stale test counts, and described the WASM path as going through HIR.

## Terminology: "DOL 2.0"

"DOL 2.0" is the historical name for the **expression-capable generation** of the language: functions, control flow,
pattern matching, lambdas, pipes and meta-programming, introduced around crate v0.1.0. It is not a version
number, and no separate "DOL 2.0 specification" exists. The language is versioned by the `dol` crate (currently
0.8.2). The de facto specification is `src/lexer.rs` + `src/parser.rs`, summarised in [grammar.ebnf](grammar.ebnf).

## Pipeline

```
 .dol source
     │
     ▼
 Lexer (src/lexer.rs, hand-written)
     │
     ▼
 Parser (src/parser.rs, recursive descent + Pratt expressions in src/pratt.rs)
     │
     ▼
 AST (src/ast.rs) ──┬──► Validator (src/validator.rs): structural checks; type errors are NOT enforced
                    │
                    ├──► AST codegen (src/codegen/): Rust, TypeScript, JSON Schema   [dol-codegen]
                    │
                    ├──► Direct WASM backend (src/wasm/)                            [wasm-compile feature]
                    │
                    └──► HIR lowering (src/lower/ → src/hir/)
                              ├──► HIR Rust codegen (src/codegen/hir_rust.rs)        [dol-build]
                              └──► LLVM native backend (llvm-backend/, separate workspace, beta)
```

Two points that are easy to get wrong:

- **WASM does not go through HIR.** `metadol::compiler::spirit::compile_source` lowers to HIR **for diagnostics
  only**, then emits WASM directly from the AST via `crate::wasm::WasmCompiler` (`src/compiler/spirit.rs`).
- **Single-declaration CLIs.** `dol-parse` and `dol-check` call `parse_file`, which returns only the
  **first** declaration and ignores the rest of the file. `dol-codegen` and `wasm-stress-test` use `parse_file_all`,
  and `dol-build-crate` uses `parse_dol_file`. Both of those parse the whole file.

## Source layout

| Path | Contents |
|---|---|
| `src/lexer.rs` | Hand-written lexer. Keyword table in `keyword_kind`. Identifiers may contain dots (`container.exists` is one token). `//` and `--` start line comments. (`logos` is listed in Cargo.toml but not used by the lexer.) |
| `src/parser.rs` | Recursive-descent parser. Top-level dispatch is in `parse_declaration`. `parse` (first declaration only), `parse_all` and `parse_file` (whole file). |
| `src/pratt.rs` | Infix and prefix binding powers for expressions |
| `src/ast.rs` | AST: `Declaration` (Gene, Trait, Constraint, System, Evolution, Function, Const, SexVar), `Statement`, `Expr`, `Stmt`, `TypeExpr`, `Pattern` |
| `src/validator.rs` | Semantic validation. `validate_statement_types` is an empty placeholder, so type errors pass. |
| `src/typechecker.rs`, `src/types.rs` | Type checker. Not wired into validation. |
| `src/hir/`, `src/lower/` | HIR definitions (`HirDecl`, `HirExpr`, `HirStmt`, `HirType`, `HirPat`, …) and AST → HIR lowering (`metadol::lower::lower_file`) |
| `src/codegen/` | `rust.rs`, `typescript.rs`, `jsonschema.rs` (AST-based), `hir_rust.rs` (HIR-based), `crate_gen.rs` (`dol-build-crate`) |
| `src/wasm/` | Direct WASM backend: `compiler.rs`, `layout.rs`, `alloc.rs`, `runtime.rs` (wasmtime, `wasm-runtime` feature) and `host_abi.rs`, which defines 15 `vudo.host_*` imports matching the VUDO VM linker |
| `src/compiler/` | Spirit compile pipeline (`compile_source`, `compile_file`), behind the `wasm-compile` feature |
| `src/macros/`, `src/reflect.rs` | Built-in macros and expansion; type reflection |
| `src/transform/` | AST passes: visitor, fold, idiom desugaring (`IdiomDesugar`), tree shaking |
| `src/sex/` | Side-effect (`sex`) context, tracking and purity lint |
| `src/eval/`, `src/repl/` | Expression interpreter and REPL support |
| `src/mcp/` | MCP server (behind the `serde` feature) |
| `src/lsp/` | Completion-provider library (`completion.rs`). This is **not** an LSP server. |
| `src/mlir/` | MLIR backend. Stubbed, behind the `mlir` feature, needs MLIR headers, not in CI. |
| `src/manifest.rs`, `src/system.rs` | Spirit / system manifest parsing |
| `src/network/`, `src/swarm/` | Hyphal-network modelling modules |
| `src/bin/` | CLIs (see below) |
| `dol-abi/` | Host ABI crate: `vm_host_functions()` (15) and the deprecated `standard_host_functions()` (22 `vudo_*` names) |
| `crates/` | `dol-codegen*`, `dol-macro*`, `dol-reflect`, `dol-exegesis`, `dol-test`, and the local-first `vudo-*` crates. **Not built in CI**, because there is no root `[workspace]`. |
| `llvm-backend/` | LLVM native backend and `dol-native` CLI. Beta, separate workspace, not in CI. See [native-compilation.md](native-compilation.md). |
| `dol/` | Self-hosting bootstrap sources (10 `.dol` files). They parse, but the generated crate does not compile. Self-hosting is **not** working. |

## Feature flags

| Feature | Enables |
|---|---|
| (default) | Library only: lexer, parser, AST, validator, HIR, codegen, macros |
| `cli` | All CLI binaries (implies `serde`) |
| `serde` | Serialization and the MCP module |
| `wasm-compile` | Direct WASM backend + `compiler` module (wasm-encoder) |
| `wasm-runtime` | wasmtime-based execution |
| `wasm` | `wasm-compile` + `wasm-runtime` |
| `mlir`, `wasm-mlir` | MLIR backend (stubbed) |
| `vudo` | `cli` + `wasm`; the `vudo` binary. This binary does not compile in 0.8.2. |

## Language surface (as parsed by 0.8.2)

Top-level declarations (`parse_declaration`): `gen`/`gene`, `type` (same shape as `gen`), `trait`, `rule`/`constraint`,
`system`, `evo`/`evolves`, `sex` (`sex fun`, `sex var`, `sex extern fun`), `fun`, `const`, `docs`/`exegesis`, `use`
and `mod`/`module`. Optional `pub` / `pub(...)` and `#[attr]` prefixes are allowed.

- The keywords `gene`, `constraint`, `evolves` and `exegesis`, and the PascalCase types (`Int64`, `String`, `Void`,
  `List<T>`, …), were deprecated in **v0.8.0**. They still parse with warnings, and are planned to become errors in
  0.9, which has not been released.
- Accepted but discarded:
  - `pub` visibility (the AST always records `Private`)
  - generic parameters on `gen`/`trait`/`rule`/`system`
  - bodies of `rule` blocks nested inside a `gen`
  - the contents of `mod x { … }`
- Not parseable in 0.8.2:
  - `let mut` and `+=`
  - `&T` and `[T]` types
  - generic functions `fun f<T>`
  - `impl`, top-level `spirit`, `extern` without `sex`
  - `evo A > B`, the `added` keyword, `type X is enum`
  - `with IO` effect rows, and backtick or `~` quasi-quote
  - versioned gen headers `gen x @1.0.0 {`

The full grammar is in [grammar.ebnf](grammar.ebnf).

### Operator precedence (`src/pratt.rs`)

| Operator | Binding power (left, right) | Associativity |
|---|---|---|
| `implies` | (3, 2) | right. The lexer produces this token, but `make_binary_expr` rejects it, so it does not parse. |
| `:=` | (10, 9) | right |
| `\|>` | (21, 20) | left |
| `@` (apply) | (31, 30) | left |
| `>>` (compose) | (40, 41) | right |
| `->` | (50, 51) | right |
| `..` (range) | (55, 55) | non-associative |
| `\|\|` | (61, 60) | left |
| `&&` | (71, 70) | left |
| `==` `!=` | (80, 80) | non-associative |
| `<` `<=` `>` `>=` | (90, 90) | non-associative |
| `+` `-` | (101, 100) | left |
| `*` `/` `%` | (111, 110) | left |
| `^` | (120, 121) | right |
| `as` | (131, 130) | left |
| `.` | (141, 140) | left |

Prefix: `-`, `!`, `*` (130); `'` quote, `?` reflect (135). The parser also handles `!{ … }` (eval), `[| … |]` (idiom
brackets), `#name(…)` (macro call), lambdas `|x: T| expr`, and the postfix `?` (try) operator.

## CLI tools (`src/bin/`, all require `--features cli`)

| Binary | Purpose | Notes |
|---|---|---|
| `dol-parse` | Parse + validate, print the AST (`--format pretty/json/compact/debug`), tree shaking (`--tree-shake`) | First declaration only |
| `dol-check` | CI gate (`--require-exegesis`, `--typecheck`, `--strict`) | First declaration only. `--typecheck` does not reject type errors. |
| `dol-test` | Generate Rust tests from `.dol.test` files | |
| `dol-codegen` | `--target rust/typescript/jsonschema` | Whole file. Output quality varies; generated Rust does not always compile. |
| `dol-migrate` | `0.7-to-0.8`, `0.2-to-0.3` syntax migration | See [migration-guide.md](migration-guide.md) |
| `dol-mcp` | MCP server over stdio (`serve`, `manifest`, `tool <name>`) | Tools: `parse`, `typecheck`, `compile_rust`, `compile_typescript`, `compile_wasm`, `eval`, `reflect`, `format`, `list_macros`, `expand_macro`, `validate_schema`, `recommend_crdt`, `explain_strategy`, `generate_example`, … |
| `dol-build` | Spirit project → Rust → wasm32 | Fails on all example Spirits in 0.8.2 |
| `dol-build-crate` | Generate a Rust crate from DOL files | Used by the bootstrap experiment |
| `vudo` | VUDO CLI (`vudo` feature) | Does not compile in 0.8.2 (non-exhaustive match) |

## Known gaps

The full list is in [DOL-STATUS-2026-11.md §5](DOL-STATUS-2026-11.md). In summary:

- No type enforcement.
- Single-declaration CLIs.
- The WASM backend covers only a subset of the language:
  - Not supported: lambdas, tuples, lists, `sex {}` blocks, `?`, quote/meta, and `for` over anything other than
    a range.
  - Gene layouts reject `u64` and `string`.
- No CI coverage for `crates/*`, `apps/*`, `llvm-backend` or `benchmarks`.
- Self-hosting is not working.
