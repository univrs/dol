# DOL — Design Ontology Language

[![CI](https://github.com/univrs/dol/actions/workflows/ci.yml/badge.svg)](https://github.com/univrs/dol/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/dol.svg)](https://crates.io/crates/dol)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![Rust Version](https://img.shields.io/badge/rust-1.84%2B-orange.svg)](https://www.rust-lang.org)

> [!CAUTION]
> DOL is an experimental research project. It changes often, has no stability guarantees, and is not production software. No support is provided (see [SUPPORT.md](SUPPORT.md)). Read [Known limitations](#known-limitations) before relying on it.

DOL is a declarative DSL for ontology-first development. You declare what a system *is* — genes (data shapes), traits (behaviours), rules (invariants), systems (compositions) and evolutions (version lineage), each with a `docs` block — and the toolchain parses, validates and generates code from it. The language also has functions, pattern matching, effects (`sex`) and meta-programming.

This README describes **crate `dol` v0.8.2** (library name `metadol`). The syntax era is v0.8 "Clarity". For a verified, line-by-line account of what works, see [docs/DOL-STATUS-2026-11.md](docs/DOL-STATUS-2026-11.md).

> **About "DOL 2.0".** Older docs call the language "DOL 2.0". That is a historical name for the generation of the language that added expressions (functions, `match`, lambdas, pipes, quote/eval, `sex` effects) on top of the original declarative DOL 1.x. It is not a version number and there is no separate DOL 2.0 specification. The only enforced version is the crate version.

---

## What's new in 0.8.2 — VUDO host ABI

DOL-compiled Spirits now link and run in the VUDO VM. A bodiless `sex fun host_*` declaration compiles to a WebAssembly import from module `vudo`, with the exact signature the VM linker registers (`src/wasm/host_abi.rs`). There are 15 host functions:

| Area | Functions |
|------|-----------|
| Time / random | `host_time_now`, `host_random_bytes` |
| Logging | `host_log` |
| Storage | `host_storage_read`, `host_storage_write`, `host_storage_delete` |
| Network | `host_network_connect`, `host_network_listen`, `host_network_broadcast` |
| Credit | `host_credit_balance`, `host_credit_transfer`, `host_credit_reserve`, `host_credit_release`, `host_credit_consume`, `host_credit_available` |

```dol
sex fun host_log(level: i64, msg: string) -> i64

sex fun hello() -> i64 {
    return host_log(2, "hello from DOL")
}
```

A `string` argument lowers to a `(ptr, len)` pair, and modules that import pointer-taking host functions export `memory`. Declaring a `host_*` function the VM does not provide, or with the wrong arity or parameter types, is a compile error. See [CHANGELOG.md](CHANGELOG.md#082---2026-11-20) for details.

---

## Installation

All command-line tools are behind the `cli` feature:

```bash
cargo install dol --features cli
```

This installs `dol-parse`, `dol-check`, `dol-test`, `dol-codegen`, `dol-migrate`, `dol-mcp`, `dol-build` and `dol-build-crate`. There is no single `dol` binary. Without `--features cli`, no DOL tools are installed.

From source:

```bash
git clone https://github.com/univrs/dol.git
cd dol
cargo build --features cli --bins
```

Requires Rust 1.84 or later.

To use DOL as a library:

```toml
[dependencies]
dol = "0.8.2"                                          # parser, validator, codegen, transforms
# dol = { version = "0.8.2", features = ["wasm-compile"] }  # + direct WASM compiler
```

The library is imported as `metadol`, not `dol`.

### Cargo features

| Feature | Enables |
|---------|---------|
| *(default)* | Lexer, parser, AST, validator, typechecker, HIR lowering, Rust/TypeScript/JSON Schema codegen, macros, reflection, transforms |
| `cli` | The command-line tools (implies `serde`) |
| `serde` | Serialization of the AST; required by the MCP server |
| `wasm-compile` | Direct DOL → WASM compiler (`metadol::compile_source`, `WasmCompiler`); browser-compatible |
| `wasm-runtime` | WASM execution via `wasmtime` (native only) |
| `wasm` | `wasm-compile` + `wasm-runtime` |
| `mlir`, `wasm-mlir` | Experimental MLIR backend. Needs MLIR/LLVM development headers; not built in CI |
| `vudo` | The `vudo` binary. **It does not compile in 0.8.2** (non-exhaustive match in `src/bin/vudo.rs`) |

---

## Quick start

Create `example.dol`:

```dol
gen user.account {
    has id: u64
    has email: string
    has active: bool
}

docs {
    A user account is an authenticated entity with a unique id,
    an email address and an active flag.
}

trait user.authenticatable {
    uses user.account
    each login emits auth_event
}

docs {
    An authenticatable account emits an event on every login.
}

rule user.identity {
    id never changes
    email matches verified_address
}

docs {
    An account keeps its id for life, and its email must match a
    verified address.
}
```

Generate code from it:

```bash
dol-codegen --target rust example.dol         # Rust structs/traits
dol-codegen --target typescript example.dol   # TypeScript
dol-codegen --target jsonschema example.dol   # JSON Schema
```

`dol-codegen` reads every declaration in the file. **`dol-parse` and `dol-check` do not**: in 0.8.2 they parse only the first declaration of each file and silently ignore the rest. On the file above, `dol-parse example.dol` reports a single `user.account` gene. See [Known limitations](#known-limitations).

### As a library

```rust
use metadol::{parse_file_all, validate};

let source = std::fs::read_to_string("example.dol")?;
let decls = parse_file_all(&source)?; // every declaration in the file

for decl in &decls {
    let result = validate(decl);
    println!("{}: valid = {}", decl.name(), result.is_valid());
}
```

Use `parse_file_all` (or `metadol::parse_dol_file`, which also returns `module`/`use` headers). `metadol::parse_file` returns only the first declaration.

---

## The language (v0.8)

### Keywords

| Current | Legacy (deprecated) |
|---------|---------------------|
| `gen` | `gene` |
| `rule` | `constraint` |
| `evo` | `evolves` |
| `docs` | `exegesis` |

Legacy keywords still parse but print a deprecation warning. They are planned to become errors in v0.9. `dol-migrate 0.7-to-0.8` rewrites them.

Other top-level declarations: `type`, `trait`, `system`, `fun`, `sex`, `const`, `use`, `mod`/`module`.

### Types

Types follow Rust naming: `i8`…`i128`, `u8`…`u128`, `f32`, `f64`, `bool`, `string`, `()`, `Vec<T>`, `Option<T>`. Legacy names (`Int64`, `UInt64`, `String`, `Bool`, `Void`, `List<T>`, …) still parse with a deprecation warning.

### Ontology declarations

Write each `docs { }` block **after** the closing brace of its declaration. Missing or short docs are a validator warning; `dol-check --require-exegesis` makes them an error.

```dol
system user.directory @ 0.1.0 {
    requires user.account >= 0.1.0
    requires user.authenticatable >= 0.1.0
}

docs {
    The directory composes accounts and authentication.
}

evo user.account @ 0.2.0 > 0.1.0 {
    adds has display_name: string
    because "users asked for a visible name"
}

docs {
    Version 0.2.0 adds a display name to accounts.
}
```

### Functions and control flow

```dol
fun classify(n: i64) -> string {
    match n {
        0 => "zero",
        x if x < 0 => "negative",
        _ => "positive"
    }
}

fun sum_to(limit: i64) -> i64 {
    var total: i64 = 0
    for i in 0..limit {
        total = total + i
    }
    return total
}

fun pipeline(x: i64) -> i64 {
    val double = |v: i64| v * 2
    return x |> double
}
```

The parser also accepts `if`/`else`, `while`, `loop`/`break`, `?`, method calls, struct literals, tuples, function types such as `(i64) -> bool`, `::` paths, and the `>>` composition operator.

### CRDT annotations

```dol
gen chat.message {
    @crdt(lww)
    has text: string

    @crdt(or_set)
    has reactions: Set<string>

    @crdt(pn_counter)
    has likes: i64
}

docs {
    A chat message whose text is last-writer-wins, whose reactions
    are an add-wins set and whose like count is a PN-counter.
}
```

The validator checks that each strategy fits its field type. The MCP tool `recommend_crdt` suggests strategies.

### Meta-programming

The core library implements quote (`'expr`), eval (`!expr`), reflection (`?Type`), idiom brackets (`[| f x y |]`) and macros (`#name(...)`):

```dol
fun meta() -> i64 {
    val expr = '(1 + 2 * 3)
    val result = !expr
    return result
}
```

There are 20 built-in macros: `#derive`, `#stringify`, `#concat`, `#env`, `#option_env`, `#cfg`, `#file`, `#line`, `#column`, `#include_str`, `#assert`, `#assert_eq`, `#assert_ne`, `#debug_assert`, `#format`, `#dbg`, `#todo`, `#unreachable`, `#compile_error`, `#vec`.

Meta-programming is evaluated by the in-library interpreter. The WASM backend does not support it yet.

---

## Command-line tools

All need `--features cli`.

| Tool | What it does |
|------|--------------|
| `dol-parse` | Parse files and print a summary (`--format pretty\|json\|compact\|debug`), with `--tree-shake`. **Only the first declaration per file is parsed.** |
| `dol-check` | Validation gate for CI: `--require-exegesis`, `--strict`, `--typecheck`, `--ci`. **Only the first declaration per file is checked.** |
| `dol-test` | Generate Rust tests from `.dol.test` files (`generate`, `validate`, `plan`). |
| `dol-codegen` | Generate Rust, TypeScript or JSON Schema (`--target rust\|typescript\|jsonschema`). |
| `dol-migrate` | Rewrite syntax: `0.7-to-0.8` and `0.2-to-0.3`. `--dry-run` / `--diff` preview only (add `--write` to apply with a diff); with neither flag files are rewritten in place. Comments, strings and `docs` bodies are never changed. |
| `dol-mcp` | Model Context Protocol server over stdio (`dol-mcp serve`), plus `dol-mcp manifest` and `dol-mcp tool <name> key=value`. |
| `dol-build` | Spirit project → Rust → `wasm32` + JS bindings. **Experimental:** it fails on all 9 example Spirits in `examples/spirits/`. |
| `dol-build-crate` | Generate a Rust crate from DOL files (used by the self-hosting bootstrap). **Experimental.** |

### MCP tools

`dol-mcp` exposes these tools by name: `parse`, `typecheck`, `compile_rust`, `compile_typescript`, `compile_wasm`, `eval`, `reflect`, `format`, `list_macros`, `expand_macro`, `validate_schema`, `recommend_crdt`, `explain_strategy`, `generate_example`, `generate_schema_from_description`, `validate_and_suggest`, `get_suggestions`.

`compile_wasm` and `format` currently return "not yet implemented". The server handles the JSON-RPC methods `initialize` (which returns the tool manifest) and `tools/call`; it does not implement `tools/list`.

```bash
dol-mcp tool parse 'source=gen point { has x: i64 }'
```

---

## Compiling to WebAssembly

With the `wasm-compile` feature, `metadol::compile_source` compiles DOL source directly to a WASM module:

```rust
let source = r#"
sex fun host_log(level: i64, msg: string) -> i64

sex fun hello() -> i64 {
    return host_log(2, "hello from DOL")
}
"#;
let spirit = metadol::compile_source(source, "hello.dol")?;
std::fs::write("hello.wasm", &spirit.wasm)?;
```

The resulting module imports `vudo.host_log` and runs in the VUDO VM (`univrs/vudo`). The WASM backend covers a subset of the language; see below.

---

## AST transforms

```rust
use metadol::transform::{ConstantFolding, DeadCodeElimination, IdiomDesugar, PassPipeline};

let mut pipeline = PassPipeline::new();
pipeline
    .add(ConstantFolding::new())
    .add(DeadCodeElimination::new())
    .add(IdiomDesugar::new());
let optimized = pipeline.run_all(decls)?;
```

| Pass | Path | Description |
|------|------|-------------|
| `ConstantFolding` | `metadol::transform` | Evaluate constant expressions |
| `DeadCodeElimination` | `metadol::transform` | Remove unreachable code |
| `IdiomDesugar` | `metadol::transform` | Rewrite `[\| f x y \|]` to `f <$> x <*> y` |
| `Simplify` | `metadol::transform::passes` | Simplify expressions (double negation, identity operations) |
| `TreeShaking` | `metadol::transform` | Drop declarations unreachable from the roots (used by `dol-parse --tree-shake`) |

---

## Component status (v0.8.2)

| Component | Status | Notes |
|-----------|--------|-------|
| Lexer, parser, AST | Real | Some syntax is parsed then discarded (see below) |
| Validator | Real, shallow | Checks naming, docs and CRDT strategies; does **not** enforce types |
| Typechecker | Partial | Runs with `dol-check --typecheck`, but type errors do not fail validation |
| HIR lowering | Real | Not used by the WASM backend |
| Direct WASM backend + VUDO host ABI | Beta | Subset of the language |
| Rust codegen | Real, quality varies | Output does not always compile standalone |
| TypeScript / JSON Schema codegen | Real, quality varies | Typed fields (`has x: T`) are currently omitted from the output |
| `dol-migrate`, `dol-test` | Real | |
| `dol-mcp` | Real | `compile_wasm` and `format` are stubs |
| Macros, reflection, quote/eval | Real in the library | Not supported by the WASM backend |
| `dol-build` (Spirit → Rust → WASM) | Experimental | Fails on the example Spirits |
| LLVM native backend (`llvm-backend/`) | Beta | Separate crate, not published, not in CI; needs a manual LLVM 18 setup |
| MLIR backend (`mlir` feature) | Experimental | Largely stubs; not in CI |
| `vudo` binary | Broken | Does not compile in 0.8.2 |
| Automerge codegen (`crates/dol-codegen-rust`) | Experimental | Separate unpublished crate, not wired into any CLI |
| Self-hosting (`dol/*.dol`) | Not working | The sources parse, but the generated Rust crate does not compile |

---

## Known limitations

- **`dol-parse` and `dol-check` only parse the first declaration in a file.** Everything after it is ignored, including invalid content, so their pass rates are overstated. Use `metadol::parse_file_all` or `dol-codegen` to process whole files.
- **Types are not enforced.** For example, `fun f() -> i64 { return "hello" + true }` passes validation, `dol-check --typecheck`, HIR lowering and WASM compilation.
- **Parsed but discarded:** `pub` visibility, generic parameters on `gen`/`trait`, a `rule` nested inside a `gen`, the contents of `mod x { … }`, and a `docs` block placed inside a `gen` body that has fields (put `docs` after the closing brace instead).
- **Not parseable** (even though some older docs show it): `let mut`, `+=`, `&T`, `[T]`, generic functions `fun id<T>(…)`, `impl`, top-level `spirit`, `extern`, `with IO` effect annotations, quasi-quote, versioned headers like `gen name @1.0.0 {`, and float exponents such as `1e9`.
- **WASM is a subset.** Not supported: lambdas, tuples, lists, `sex { }` blocks, `?`, quote/eval and other meta-programming, idiom brackets, `this`, casts, and `for` loops over anything except a range. Gene layouts reject `u64` and `string` fields. `println` is unknown. Purely declarative files compile to an empty module.
- **Self-hosting does not work.** The bootstrap sources in `dol/` parse, but the crate generated from them fails to compile.
- **MLIR is experimental** and does not build without MLIR headers.
- `docs/grammar.ebnf` is out of date: it describes only the original declarative language. The de facto specification is `src/lexer.rs`, `src/parser.rs` and [CHANGELOG.md](CHANGELOG.md).

---

## Testing

```bash
cargo test                                                   # default features
cargo test --features cli,serde,wasm --lib --bins --tests    # with CLI and WASM
cargo test --features cli,serde,wasm --doc
```

On 2026-11-21, with `--features cli,serde,wasm`, the two commands above ran 2,512 tests (including doctests) with 0 failures. These counts cover the root crate only. The crates under `crates/`, `apps/`, `llvm-backend/` and `benchmarks/` are not part of a Cargo workspace and are not built in CI.

---

## Roadmap (planned, not implemented)

None of the following exists yet:

- v0.9: legacy keywords and type names become errors.
- `dol-parse` / `dol-check` parsing whole files, and type errors failing validation.
- A specification and grammar for the language as implemented.
- Wider WASM coverage (strings and `u64` in gene layouts, lambdas, collections).
- A self-hosting compiler.
- A package registry for Spirits.
- A language server (an LSP completion library exists in `src/lsp`, but no server).

---

## Documentation

- [docs/DOL-STATUS-2026-11.md](docs/DOL-STATUS-2026-11.md) — verified status audit of v0.8.2
- [CHANGELOG.md](CHANGELOG.md) — release history
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — architecture reference (written for v0.8.0)
- [docs/tutorials/](docs/tutorials/) — tutorials (some examples predate v0.8 syntax)
- [API docs on docs.rs](https://docs.rs/dol)
- [learn.univrs.io](https://learn.univrs.io) — learning site for the Univrs ecosystem

---

## Contributing

This project is not accepting external contributions or pull requests.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

DOL is part of the [Univrs](https://github.com/univrs) ecosystem. DOL-compiled Spirits run in the VUDO VM, the application layer of DOLOS.
