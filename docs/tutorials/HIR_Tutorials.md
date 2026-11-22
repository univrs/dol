# HIR Tutorial (dol 0.8.2)

> HIR is DOL's High-level Intermediate Representation: a smaller, desugared form of the AST.
> It was introduced in crate v0.3.0. This page describes HIR as it exists in `dol` **0.8.2**
> (`src/hir/`, `src/lower/`, `src/codegen/hir_rust.rs`).

## Where HIR sits

```
DOL source ──parse──► AST ──lower──► HIR ──► Rust source   (HirRustCodegen)
                       │
                       └──────────────► WASM               (direct AST backend)
```

- `metadol::lower::lower_file` parses a whole file and lowers **every** declaration to HIR.
  It uses `Parser::parse_file`, not the first-declaration-only `parse_file`.
- The HIR-based Rust generator is `metadol::codegen::HirRustCodegen`.
- **The WASM path does not use HIR.** `src/compiler/spirit.rs` lowers to HIR "for
  diagnostics only" and compiles WASM directly from the AST.

## Node types

From `src/hir/types.rs`:

| Category | Count | Variants |
|----------|-------|----------|
| Declarations (`HirDecl`) | 4 | `Type`, `Trait`, `Function`, `Module` |
| Expressions (`HirExpr`) | 12 | `Literal`, `Var`, `Binary`, `Unary`, `Call`, `MethodCall`, `Field`, `Index`, `Block`, `If`, `Match`, `Lambda` |
| Statements (`HirStmt`) | 6 | `Val`, `Var`, `Assign`, `Expr`, `Return`, `Break` |
| Types (`HirType`) | 8 | `Named`, `Tuple`, `Array`, `Function`, `Ref`, `Optional`, `Var`, `Error` |
| Patterns (`HirPat`) | 6 | `Wildcard`, `Var`, `Literal`, `Constructor`, `Tuple`, `Or` |

## How AST declarations lower

From `src/lower/decl.rs`:

| AST declaration | HIR | What is kept |
|-----------------|-----|--------------|
| `gen` / `type` | `HirDecl::Type` | Name, fields and statements |
| `trait` | `HirDecl::Trait` | |
| `fun`, `sex fun` | `HirDecl::Function` | |
| `rule` | `HirDecl::Trait` | **Name only**: the items list is empty |
| `system` | `HirDecl::Module` | **Name only**: no contents |
| `evo` | `HirDecl::Module` | **Name only** |
| `const`, `sex var` | `HirDecl::Module` | **Name only** |

Rules, systems, evolutions, constants and `sex var` declarations therefore lose their bodies
when lowered.

## Desugaring

From `src/lower/expr.rs`:

| Surface syntax | HIR |
|----------------|-----|
| `let x = e`, `val x = e`, `var x = e` | `Val { pat: x, init: e }`. The parser maps all three to the same AST node, so mutability is not tracked |
| `x = e` | `Assign` |
| `x \|> f` | `Call(f, [x])` |
| `f >> g` | `Lambda(x => g(f(x)))` |
| `for x in xs { body }` | A block holding `match next(...) { Some(x) => body, None => break }`. The comments in `src/lower/expr.rs` describe this as simplified |
| `while c { body }` | A block holding `if c { body } else { break }` |
| `continue` | A call to a `continue` marker |

`let mut`, `+=`, `each x in xs` and `forall x in xs { }` used as a loop do **not** parse in
0.8.2. Older versions of this page described them as desugaring rules.

## Example

This file parses and lowers:

```dol
gen point.2d {
    has x: f64
    has y: f64
}

docs {
    A 2D point.
}

fun norm2(x: f64, y: f64) -> f64 {
    val sq = x * x + y * y
    return sq
}

fun clamp(v: i64) -> i64 {
    return match v {
        n if n < 0 => 0,
        _ => v
    }
}
```

Lowering and Rust generation from Rust code:

```rust
use metadol::codegen::{compile_to_rust_via_hir, compile_with_diagnostics, HirRustCodegen};
use metadol::lower::lower_file;

fn main() -> Result<(), metadol::error::ParseError> {
    let source = "gen point.2d {\n    has x: f64\n    has y: f64\n}\n";

    // Lower to HIR and inspect it
    let (hir, ctx) = lower_file(source)?;
    println!("{} top-level HIR decls", hir.decls.len());
    println!("module name: {:?}", ctx.symbols.resolve(hir.name));

    // Generate Rust from HIR (low-level)
    let mut codegen = HirRustCodegen::with_symbols(ctx.symbols);
    let rust = codegen.generate(&hir);
    println!("{rust}");

    // Convenience wrappers
    let _rust = compile_to_rust_via_hir(source)?;
    let (_rust, diagnostics) = compile_with_diagnostics(source)?;
    println!("{} lowering diagnostics", diagnostics.len());
    Ok(())
}
```

This program compiles and runs against 0.8.2. The Rust that `HirRustCodegen` produces is
rough, though. For the gen above it emits `pub struct point.2d { pub x: i32, pub y: i32 }`,
which is not valid Rust: the struct name contains a dot, and the `f64` fields became `i32`.
How good the generated Rust is depends on the input. According to the
[status audit](../DOL-STATUS-2026-11.md), only 14 of 41 example files produce Rust that
compiles standalone with `rustc`. The audit measured this with `dol-codegen`.

## DOL-in-DOL (bootstrap) status

**DOL is not self-hosting.** As of 0.8.2:

- The bootstrap sources `dol/*.dol` (lexer, parser, AST, type checker, codegen, ...) exist and
  all **parse**.
- The Rust crate generated from them **fails to compile with about 850 errors**.
- `stage1/` contains only a `Cargo.toml`, and `stage2/` does not exist.

Treat the `dol/` sources as a large parser test corpus and as research, not as a working
compiler.

## Migration tooling

`dol-migrate` (build with `--features cli`) has two subcommands:

```bash
cargo run --features cli --bin dol-migrate -- 0.7-to-0.8 --dry-run src/
cargo run --features cli --bin dol-migrate -- 0.7-to-0.8 --diff src/
cargo run --features cli --bin dol-migrate -- 0.2-to-0.3 src/
```

There is no `dol migrate` command. No `dol` binary exists.

The current deprecations are the v0.8.0 keyword and type renames
(`gene`→`gen`, `constraint`→`rule`, `evolves`→`evo`, `exegesis`→`docs`, `Int64`→`i64`, ...).
They parse with warnings ("deprecated in v0.8.0") and are planned to become errors in 0.9. See
the [syntax reference](./syntax-reference.md#legacy-deprecated-syntax).
