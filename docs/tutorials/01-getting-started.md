# Tutorial 1: Getting Started with DOL

> Applies to the `dol` crate **0.8.2** (library name `metadol`). Syntax is the v0.8 "Clarity" keyword set:
> `gen`, `rule`, `evo`, `docs` and lowercase types (`string`, `i64`, `bool`, ...). The older keywords
> (`gene`, `constraint`, `evolves`, `exegesis`) and PascalCase types (`String`, `Int64`, ...) still parse
> but print deprecation warnings ("deprecated in v0.8.0"); they are planned to become errors in v0.9.

## Welcome to DOL

DOL (Design Ontology Language) is a domain-specific language for **ontology-first development**. You
describe what your domain *is* (genes, traits, rules, systems and their evolution), each with a
`docs` block that explains it, and then use the toolchain to parse, validate and generate code from it.

### The DOL-first idea

Code-first development usually goes:
1. Write code
2. Add comments and docs
3. Hope the implementation matches the design

DOL-first inverts this:
1. **Define your ontology** in DOL (genes, traits, rules, systems)
2. **Validate** it with the DOL tools
3. **Generate or write code** that implements it

### What DOL models well

- **Domain entities** (users, products, orders): `gen`
- **Behaviours** (lifecycle states, events): `trait`
- **Invariants and business rules**: `rule`
- **Top-level compositions with versioned dependencies**: `system`
- **Versioned change**: `evo`

DOL also has an expression language (functions, `if`/`match`, loops, lambdas). That is covered in
[05-dol2-features.md](05-dol2-features.md) and [syntax-reference.md](syntax-reference.md).

## Installation

### Prerequisites

You need a Rust toolchain (the crate's minimum supported Rust version is 1.84):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building the toolchain

The command-line tools are behind the `cli` feature:

```bash
git clone https://github.com/univrs/dol.git
cd dol
cargo build --release --features cli
```

This builds `dol-parse`, `dol-check`, `dol-test`, `dol-codegen`, `dol-migrate`, `dol-mcp`, `dol-build`
and `dol-build-crate` into `target/release/`.

Installing from crates.io should work the same way, as long as you enable the feature
(`cargo install dol --features cli`); without `--features cli` no binaries are built.

### Verify the build

```bash
cargo run --features cli --bin dol-parse -- --version
# dol-parse 0.8.2
```

## Your first DOL file

Create `my-first.dol`:

```dol
gen container.exists {
    has identifier: string
    container is entity
    container is persistent
}

docs {
    A container exists in the system and has a unique identifier.
    This is the most basic property of any container.
}
```

Let's break it down.

### Declaration header

`gen container.exists {` declares a gene:

- `gen` declares an atomic ontological unit (a "gene").
- `container.exists` is a qualified identifier. Dots are part of the name (`domain.property`).
- Genes have **no version in their header**. `gen name @1.0.0 {` does not parse. Versions live on
  `module x @ 1.0.0`, `system X @ 1.0.0 { }` and `evo X @ 1.1.0 > 1.0.0 { }`.

### The `docs` block

The `docs { ... }` block after the closing brace documents the declaration. (It can also go inside the
braces as the last item.)

DOL's convention is that every declaration has docs. Here is what the 0.8.2 tools actually enforce:
- The parser accepts a declaration without docs.
- The validator (and `dol-parse`) warns when the docs are shorter than 20 characters.
- `dol-check --require-exegesis` fails a file whose declaration has no docs.

Avoid `{` and `}` inside docs text. The parser does not handle nested braces there.

### Statements

These are the statements inside the body (a fragment, not a complete file):

```text
has identifier: string
container is entity
container is persistent
```

- `has name: type` declares a typed property.
- `subject is state` declares a category or state. The subject is required. A bare `is entity`
  does not parse.
- Other predicates, covered in [Tutorial 2](02-writing-genes.md), are `subject has property`,
  `subject derives from origin`, and `subject requires dependency`.

## Running dol-parse

```bash
cargo run --features cli --bin dol-parse -- my-first.dol
```

Output:

```
✓ my-first.dol (container.exists)
    container.exists gene with 3 statements

Summary
  Total:    1
  Success:  1
```

**Important caveat:** in 0.8.2, `dol-parse` and `dol-check` parse **only the first declaration** of
each file. Everything after it is ignored, including syntax errors. To check a whole file, call the
library function `metadol::parse_file_all` from Rust.

### Common errors

**Version on a gene header** (not supported):

```text
gen container.exists @1.0.0 {
    has identifier: string
}
```

```
✗ bad1.dol
  Parse error: expected {, found '@' at line 1, column 22
```

**No docs block:** the file still parses, but you get a warning:

```
✓ nodocs.dol (container.exists)
    container.exists gene with 1 statements
  warning: exegesis is unusually short (0 chars) at line 1, column 1
```

With `dol-check --require-exegesis`, the same file fails with `Missing or empty exegesis block`.

## Output formats

`dol-parse` supports `-f pretty|json|compact|debug`:

```bash
cargo run --features cli --bin dol-parse -- -f json my-first.dol
```

```json
{
  "files": [
    {
      "path": "my-first.dol",
      "success": true,
      "declaration_type": "gene",
      "name": "container.exists"
    }
  ],
  "summary": {
    "total": 1,
    "successful": 1,
    "failed": 0
  }
}
```

The JSON output is a per-file summary, not the AST. To work with the AST, use the library
(`metadol::parse_file_all`, `metadol::parse_dol_file`).

## Next steps

1. [Tutorial 2: Writing Genes](02-writing-genes.md) covers gene statements and fields in depth.
2. [Tutorial 3: Composing Traits](03-composing-traits.md) covers combining genes into behaviours.
3. [Tutorial 4: System Design](04-system-design.md) covers systems, version requirements and `evo`.

### Explore the examples

```bash
ls examples/genes/ examples/traits/ examples/systems/ examples/v0.8/
cargo run --features cli --bin dol-parse -- -r examples/genes
```

Remember that `dol-parse` only checks the first declaration of each file.

### Reference

- [syntax-reference.md](syntax-reference.md) is the language reference for the 0.8.2 parser.
- [../grammar.ebnf](../grammar.ebnf) is the grammar.
- The de facto specification is `src/lexer.rs` and `src/parser.rs`.

## Not yet supported in 0.8.2

These forms appear in older docs but **do not parse** today:
- versioned gene headers (`gen name @1.0.0 {`)
- `let mut` and compound assignment (`+=`, `-=`)
- reference and slice types (`&T`, `[T]`)
- generic functions (`fun id<T>(...)`)
- `impl` blocks, top-level `spirit` declarations, `extern fun` without `sex`
- `type X is enum { ... }` (use `has kind: enum { A, B }` inside a `gen`)
- effect annotations (`fun f() with IO`)
- backtick or `~` quasi-quotes

Float exponents (`1e9`) do not cause an error, but they are mis-lexed as `1` followed by an identifier
`e9`.

## Key takeaways

1. **Ontology first.** You describe the domain before writing the implementation.
2. **Docs are a convention.** `dol-check --require-exegesis` enforces them.
3. **Qualified identifiers** use dot notation (`domain.property`).
4. **Versions** go on modules, systems and `evo` declarations, not on gene headers.
5. **The CLI checks only the first declaration per file** in 0.8.2.
