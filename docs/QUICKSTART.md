# DOL Quick Start (dol 0.8.2)

This guide walks you through building the DOL tools from a checkout, writing a small `.dol` file, checking it, and
generating code from it. All commands were run against `dol` 0.8.2.
For what works and what doesn't, see [DOL-STATUS-2026-11.md](DOL-STATUS-2026-11.md).

## 1. Build the tools

Every CLI binary needs the `cli` feature:

```bash
git clone https://github.com/univrs/dol.git
cd dol
cargo build --features cli
# or install the binaries from the checkout:
cargo install --path . --features cli
```

This builds `dol-parse`, `dol-check`, `dol-test`, `dol-codegen`, `dol-migrate`, `dol-mcp`, `dol-build` and
`dol-build-crate`. There is no single `dol` binary.

## 2. Write a DOL file

Save this as `counter.dol`:

```dol
gen counter.state {
    has value: i64 = 0
    has label: string
}

docs {
    A counter holds a signed integer value and a human-readable label.
}

rule counter.bounded {
    value never overflows
}

docs {
    The counter value must stay within the range of i64.
}

fun increment(n: i64) -> i64 {
    return n + 1
}
```

The syntax is v0.8: `gen`, `rule`, `evo` and `docs`, with lowercase types. The older `gene`, `constraint`, `evolves`,
`exegesis` and `Int64`/`String` forms still parse, but print deprecation warnings.

## 3. Parse and check it

```bash
dol-parse counter.dol
dol-check --require-exegesis counter.dol
```

Output of `dol-parse`:

```text
✓ counter.dol (counter.state)
    counter.state gene with 2 statements

Summary
  Total:    1
  Success:  1
```

> **Limitation (0.8.2):** `dol-parse` and `dol-check` only parse the **first declaration** in each file. The rule
> and the function above are not checked, and anything after the first declaration is silently ignored, including
> syntax errors. To parse a whole file from Rust, call `metadol::parse_file_all(&source)`, which parses every
> declaration and fails on any error.

## 4. Generate code

```bash
dol-codegen --target rust counter.dol
dol-codegen --target typescript counter.dol
dol-codegen --target jsonschema counter.dol
```

Code generation quality varies. For example, the TypeScript output for this file currently produces an empty
`CounterState` interface. Generated Rust does not always compile.

## 5. Compile to WebAssembly (library API)

The direct WASM backend (beta) is available as a library function behind the `wasm-compile` feature:

```toml
[dependencies]
metadol = { package = "dol", version = "0.8.2", features = ["wasm-compile"] }
```

```rust
use metadol::compiler::spirit::compile_source;

fn main() {
    let source = r#"
fun add(a: i64, b: i64) -> i64 {
    return a + b
}
"#;
    let compiled = compile_source(source, "add.dol").expect("compile");
    std::fs::write("add.wasm", &compiled.wasm).unwrap();
}
```

The WASM backend covers only a subset of the language. Lambdas, tuples, lists, `sex { }` blocks, `?`, and
`for` loops over anything other than a range are not supported yet. Bodiless `sex fun host_*` declarations import
VUDO VM host functions from module `vudo`.

`dol-build` (Spirit project → Rust → wasm32) exists, but in 0.8.2 it fails on all of the example Spirits. Treat it as
experimental.

## Next steps

- [Tutorials](tutorials/)
- [Migration guide (v0.7 → v0.8 syntax)](migration-guide.md)
- [Grammar](grammar.ebnf)
- [Examples](../examples/)
