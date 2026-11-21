# Inspecting WASM produced by DOL

> Applies to `dol` **0.8.2**. The WASM backend (`metadol::wasm::WasmCompiler`, feature `wasm` or
> `wasm-compile`) compiles directly from the AST. It does not go through MLIR (`src/mlir` is a
> feature-gated stub and is not in CI). The backend covers only a **subset** of the language. In
> particular, it does not support:
> - lambdas, tuples, lists, `sex { }` blocks, `?`, quote/meta, idiom brackets, `this` or casts
> - `for` over anything except a range
> - `u64` and `string` fields in gene layouts
> - `println`
>
> Declarative-only files compile to an empty module.

## 1. Produce a module

The repository ships an example that compiles a small `Counter` gene plus a free function:

```bash
cargo run --example compile_counter --features wasm
# ...
# Generated 295 bytes of WASM
# Wrote: vertical-slice-results/counter.wasm
# WASM validated successfully!
# Exports:
#   - Counter.increment
#   - Counter.get_value
#   - Counter.add
#   - add_numbers
#   - new_Counter
#   - memory
```

(The example's source still uses the deprecated `gene` keyword and `Int64` type, so it prints
deprecation warnings.)

## 2. Inspect and run it

These tools are not part of this repo. Install them separately: `cargo install wasm-tools`, and
wasmtime from <https://wasmtime.dev>.

```bash
wasm-tools validate vertical-slice-results/counter.wasm && echo valid
wasm-tools print vertical-slice-results/counter.wasm | head -20
wasmtime run --invoke add_numbers vertical-slice-results/counter.wasm 3 4
# 7   (wasmtime warns that --invoke with arguments is experimental)
```

## Host functions

Since 0.8.2, a bodiless `sex fun host_*` declaration imports from module `vudo`, using the VUDO VM
linker's signatures (`src/wasm/host_abi.rs`). Running such a module therefore needs the VUDO VM
(univrs/vudo). Plain `wasmtime` cannot provide those imports.
