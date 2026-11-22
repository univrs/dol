# DOL → WASM Compilation Guide (dol 0.8.2)

> **Status: beta.** This page covers only what works in `dol` 0.8.2. The facts are taken from
> [`docs/DOL-STATUS-2026-11.md`](../../DOL-STATUS-2026-11.md) and the code.

## Two pipelines, one of which works

| Pipeline | Entry point | Status in 0.8.2 |
|----------|-------------|-----------------|
| **Direct WASM backend**: DOL AST → WASM | `metadol::wasm::WasmCompiler`, `metadol::compiler::spirit::compile_source` (feature `wasm-compile`) | **Works for a subset of the language.** CI runs its output in the VUDO VM |
| Spirit → Rust → `wasm32` | `dol-build` (needs a `Spirit.dol` manifest, `cargo` and `wasm-bindgen`) | **Fails on all 9 example Spirits** |

The `vudo` binary in this repository (`src/bin/vudo.rs`) **does not compile** in 0.8.2 (a
non-exhaustive match). The VUDO VM and its `vudo run` CLI live in the separate
`univrs/vudo` repository. That repository pins `dol = "0.8.2"` and runs DOL-compiled WASM
in CI.

## Compiling with the direct backend

Add the crate with the `wasm-compile` feature:

```toml
[dependencies]
dol = { version = "0.8.2", features = ["wasm-compile"] }
```

The library name is `metadol`. This program was compiled and run against 0.8.2:

```rust
use metadol::compiler::spirit::compile_source;
use metadol::parse_dol_file;
use metadol::wasm::WasmCompiler;

const SOURCE: &str = r#"
sex fun host_log(level: i64, msg: string) -> i64

sex fun hello() -> i64 {
    return host_log(2, "hello from DOL")
}

fun add(a: i64, b: i64) -> i64 {
    return a + b
}

fun sum_to(n: i64) -> i64 {
    let total = 0
    for i in 0..n {
        total = total + i
    }
    return total
}
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Low-level: AST -> WASM bytes
    let file = parse_dol_file(SOURCE)?;
    let wasm = WasmCompiler::new().compile_file(&file)?;
    std::fs::write("hello.wasm", &wasm)?;

    // Pipeline wrapper (parse, lower to HIR for diagnostics, emit WASM)
    let compiled = compile_source(SOURCE, "hello.dol")?;
    println!("{} bytes, {} warnings", compiled.wasm.len(), compiled.warnings.len());
    Ok(())
}
```

Here is the same DOL source on its own:

```dol
sex fun host_log(level: i64, msg: string) -> i64

sex fun hello() -> i64 {
    return host_log(2, "hello from DOL")
}

fun add(a: i64, b: i64) -> i64 {
    return a + b
}
```

## Host functions (the VUDO host ABI)

A `sex fun` with **no body** whose name matches a VUDO VM host function is compiled as an
import from module `vudo`. Its signature is the VM linker's exact signature
(`src/wasm/host_abi.rs`, tested in `tests/wasm_host_abi.rs`). There are 15 host functions:

`host_time_now`, `host_random_bytes`, `host_log`, `host_storage_read`, `host_storage_write`,
`host_storage_delete`, `host_network_connect`, `host_network_listen`, `host_network_broadcast`,
`host_credit_balance`, `host_credit_transfer`, `host_credit_reserve`, `host_credit_release`,
`host_credit_consume`, `host_credit_available`.

- A `string` argument (a literal, a `string` parameter or a typed `string` local) is passed as
  `(ptr, len)`. A module that imports a pointer-taking host function also exports `memory`.
- These are compile errors: a bodiless `sex fun` that the VM does not provide (an unknown name,
  the wrong arity, or a float where the host expects an integer).

## What the backend does not support (0.8.2)

From the status audit, §5.5:

- Lambdas, tuples, lists, `sex { }` blocks, the `?` operator, quote/eval/reflect, idiom
  brackets, `this`, and casts.
- `for` loops over anything other than a range (`a..b`).
- Gene layouts with `u64` or `string` fields.
- `println`: it is not a known function, so `examples/native` compiles 0 of 12 files to WASM.
- Declarative-only files (just `gen`/`trait`/`rule`) compile to an empty 17-byte module.
- **No type checking**: ill-typed programs are not rejected before code generation.

## Inspecting output

Any standard WASM tool works on the emitted bytes, for example:

```bash
wasm-tools print hello.wasm          # text format
wasm-tools validate hello.wasm
```

## See also

- [Syntax reference](../syntax-reference.md)
- [Expressions and functions](../05-dol2-features.md)
- [Game of Life benchmark](./game-of-life-benchmark.md): uses the Rust-based pipeline
