# DOL command-line tools

> Applies to `dol` **0.8.2**. Every binary needs a Cargo feature: `cli` for the `dol-*` tools and
> `vudo` for `vudo`.

## The `dol-*` tools (feature `cli`)

| Binary | Purpose |
|---|---|
| `dol-parse` | Parse and validate `.dol` files. Flags: `-f pretty\|json\|compact\|debug`, `-r`, `--validate`, `--ci`, `-w`, `-q`, `--tree-shake`, `--roots`, `--shake-analyze` |
| `dol-check` | CI gate. Flags: `--require-exegesis`, `--min-exegesis-length`, `--coverage-source`, `--strict`, `--typecheck`, `--ci`, `-f`, `-q` |
| `dol-test` | Generate Rust tests from `.dol.test` files |
| `dol-codegen` | Generate Rust, TypeScript or JSON Schema from DOL |
| `dol-migrate` | Migrate syntax (`0.7-to-0.8`, `0.2-to-0.3`) |
| `dol-mcp` | MCP server (JSON-RPC over stdio) |
| `dol-build` | Spirit project → Rust → `wasm32` build pipeline |
| `dol-build-crate` | Generate a Rust crate from DOL files |

```bash
cargo build --release --features cli
cargo run --features cli --bin dol-parse -- --help
```

Known limitations in 0.8.2:
- `dol-parse` and `dol-check` parse **only the first declaration** of each file. The rest of the file,
  including syntax errors, is ignored. The library function `metadol::parse_file_all` parses the whole
  file.
- Type errors are **not enforced**. The validator's statement-level type check is a placeholder, so
  `dol-check --typecheck` passes ill-typed functions.
- `dol-build` fails on all 9 example Spirits in `examples/spirits/`.

## The `vudo` binary (feature `vudo`)

> **Status: does not compile in 0.8.2.** `cargo build --features vudo --bin vudo` fails with a
> non-exhaustive `match` at `src/bin/vudo.rs:553` (`EvalResult::SpiritLoaded` is not covered).
> The working VUDO VM and `vudo` CLI that run DOL-compiled Spirits live in the separate
> [univrs/vudo](https://github.com/univrs/vudo) repository, which depends on `dol = "0.8.2"`.

For reference, `src/bin/vudo.rs` defines this interface. None of it can be run until the build error
is fixed:

| Command | Arguments and flags |
|---|---|
| `vudo run <file.wasm>` | `-f/--function <name>`, `-a/--args <json array>`, `--memory <pages>` (default 16), `--trace` |
| `vudo compile <file.dol>` | `-o/--output <file>`, `--optimize`, `--debug` |
| `vudo check <paths...>` | `-r/--recursive`, `--strict`, `--json` |
| `vudo repl` | `-l/--load <file>`, `--tree-shake`, `--optimize`, `--session <name>` |
| global | `-v/--verbose`, `-q/--quiet`, `-h/--help`, `-V/--version` |

`vudo check` runs parsing and validation only. Like `dol-check`, it does not enforce types.
