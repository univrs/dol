# DOL v0.3.0 Release - HIR and Language Cleanup

> **Release Date**: December 2025 (CHANGELOG: 2025-12-27)
> **Status**: Released. **Historical document.**

> **Read this first (2026-11 correction).** This page is the v0.3.0 announcement. Much of the syntax it
> announced was never implemented as described, and the current language is the **v0.8 "Clarity"** syntax
> (`gen`/`rule`/`evo`/`docs`, lowercase types) shipped in dol 0.8.x. As of dol 0.8.2:
> - `let` is **not** deprecated and emits no warning; `let`, `val` and `var` all parse. `let mut` does not parse.
> - `each`/`all` quantified statements, `module`, and `derives from` still parse **without** errors.
> - `extends` works only in a declaration header (`gen B extends A { … }`), not as a statement (`container extends image`).
> - `forall x: T. expr` and `forall x in xs { … }` parse as expressions. `forall items satisfy condition` does not.
> - `type X { … }` is still accepted as an alias for `gen`; since 0.8.0 the preferred keyword is `gen`.
> - `test "…" { given … when … then … }` blocks are not top-level declarations.
> - There is no `dol` binary: `dol migrate` and `dol compile` do not exist. The migration tool is `dol-migrate 0.2-to-0.3`, and all binaries need `--features cli`.
> - The library crate is imported as `metadol`, not `dol`.
>
> Code blocks that do not parse with dol 0.8.2 are fenced as `text` below. For current syntax, see the tutorials and the
> [Examples Gallery](./DOL_Examples_Gallery.md).

[![GitHub Release](https://img.shields.io/github/v/release/univrs/dol)](https://github.com/univrs/dol/releases/tag/v0.3.0)
[![Crates.io](https://img.shields.io/crates/v/dol)](https://crates.io/crates/dol/0.3.0)
[![docs.rs](https://docs.rs/dol/badge.svg)](https://docs.rs/dol)

**Download:**
- **GitHub**: [github.com/univrs/dol/releases/tag/v0.3.0](https://github.com/univrs/dol/releases/tag/v0.3.0)
- **Crates.io**: [crates.io/crates/dol/0.3.0](https://crates.io/crates/dol/0.3.0)

---

## What's New in v0.3.0

DOL v0.3.0 introduces the **HIR (High-level Intermediate Representation)**, a major architectural improvement that simplifies the language while making it more consistent and easier to learn.

### Key Highlights

- **Reduced complexity**: 22 canonical node types (down from 50+)
- **Cleaner syntax**: `val`/`var` bindings replace `let`/`let mut`
- **Unified keywords**: `forall` replaces `each`, `all`, and `forall`
- **Standard terms**: `extends` replaces `derives from`
- **Better tooling**: New migration tool for automatic code updates

---

## New Syntax Overview

### Bindings: `val` and `var`

The new binding syntax provides clear symmetry between immutable values and mutable variables:

```text
// Immutable value (cannot be changed)
val name = "DOL"
val count = 42

// Mutable variable (can be reassigned)
var counter = 0
counter = counter + 1
```

**Why the change?**
- `val` = value (immutable)
- `var` = variable (mutable)
- Clear, symmetric naming

### Type Declarations

While `gene` is still supported for backward compatibility, `type` is now the preferred keyword:

```dol
// Preferred (v0.3.0+)
pub type Container {
    id: UInt64
    name: String
    status: Status
}

// Still works (for compatibility)
pub gene Container {
    has id: UInt64
    has name: String
    has status: Status
}
```

### Inheritance: `extends`

The verbose `derives from` is replaced with the standard `extends`:

```text
// Before
container derives from image

// After
container extends image

// In type declarations
pub type WebServer extends Server {
    port: UInt16
}
```

### Unified Quantifier: `forall`

Three keywords are unified into one:

```text
// Before (three ways to say the same thing)
each item in items { validate(item) }
all items satisfy condition
forall x: T. predicate(x)

// After (one way)
forall item in items { validate(item) }
forall items satisfy condition
forall x: T. predicate(x)
```

---

## The HIR Architecture

### What is HIR?

HIR (High-level Intermediate Representation) is an internal representation that sits between parsing and code generation:

```
DOL Source → Parser → AST → Lowering → HIR → Codegen → Rust
```

### Benefits

1. **Simpler Codegen**: Only 22 node types to handle
2. **Canonical Forms**: One representation per concept
3. **Better Errors**: Cleaner error messages
4. **Easier Extensions**: Add new surface syntax without changing codegen

### Node Type Summary

| Category | Count | Examples |
|----------|-------|----------|
| Declarations | 4 | Type, Trait, Function, Module |
| Expressions | 12 | Literal, Call, If, Match, Lambda |
| Statements | 6 | Val, Var, Assign, Return, Break |

---

## Migration Guide

### Automatic Migration

Use the `dol-migrate` binary:

```bash
# Migrate a directory
cargo run --features cli --bin dol-migrate -- 0.2-to-0.3 src/

# Preview changes first
cargo run --features cli --bin dol-migrate -- 0.2-to-0.3 --dry-run src/
```

### Migration Rules

| v0.2.x | v0.3.0 |
|--------|--------|
| `let x = 1` | `val x = 1` |
| `let mut x = 1` | `var x = 1` |
| `each x in xs` | `forall x in xs` |
| `module foo` | `mod foo` |
| `derives from x` | `extends x` |
| `never valid` | `not valid` |

### Deprecation Warnings

The original announcement promised warnings like the one below. **dol 0.8.2 emits no `let` deprecation warning.** The warnings that do exist are the v0.8.0 keyword/type deprecations (e.g. `'gene' keyword is deprecated in v0.8.0, use 'gen' instead`).

```
warning: `let` is deprecated, use `val` instead
  --> src/main.dol:5:1
   |
 5 | let x = 42
   | ^^^ deprecated
   |
   = help: replace with `val x = 42`
```

---

## API Changes

### New Compilation Functions

```rust
use metadol::codegen::{
    compile_to_rust_via_hir,
    compile_with_diagnostics,
};

// Simple compilation
let rust_code = compile_to_rust_via_hir(dol_source)?;

// Get diagnostics too
let (rust_code, warnings) = compile_with_diagnostics(dol_source)?;
```

### Low-Level HIR Access

```rust
use metadol::lower::lower_file;
use metadol::codegen::HirRustCodegen;

// Parse and lower to HIR
let (hir, ctx) = lower_file(source)?;

// Access symbols
let name = ctx.symbols.resolve(hir.name);

// Generate code
let mut codegen = HirRustCodegen::with_symbols(ctx.symbols);
let code = codegen.generate(&hir);
```

---

## Complete Example

### Before (v0.2.x)

```text
module greeting.service @ 0.2.0

gene greeting.entity {
    entity has identity
    entity has name: String
    entity derives from template

    each greeting in greetings {
        greeting never empty
    }
}

trait entity.greetable {
    uses entity.identity
    greetable can greet
}

test "greeting works" {
    given g = Greeting.new("Hello")
    when g.send()
    then g.delivered == true
}
```

### After (v0.3.0)

```text
mod greeting.service @ 0.3.0

type greeting.entity {
    identity: Identity
    name: String
    extends template

    forall greeting in greetings {
        not greeting.empty
    }
}

trait entity.greetable {
    use entity.identity
    greet: fun() -> Void
}

test "greeting works" {
    val g = Greeting.new("Hello")
    g.send()
    assert g.delivered == true
}
```

---

## Getting Started

### Installation

```bash
# Install the CLI binaries from crates.io (they all require the `cli` feature)
cargo install dol --features cli

# Or add as a library dependency (imported as `metadol`)
cargo add dol
```

**Direct links:**
- [GitHub Releases](https://github.com/univrs/dol/releases/tag/v0.3.0)
- [Crates.io](https://crates.io/crates/dol/0.3.0)

### Quick Start

1. **Migrate existing code**:
   ```bash
   cargo run --features cli --bin dol-migrate -- 0.2-to-0.3 src/
   ```

2. **Write new code with v0.3.0 syntax**:
   ```text
   mod myapp @ 0.1.0

   pub type User {
       id: UInt64
       name: String
       email: String
   }

   fun create_user(val name: String, val email: String) -> User {
       User { id: generate_id(), name, email }
   }
   ```

3. **Compile to Rust**:
   ```bash
   cargo run --features cli --bin dol-codegen -- --target rust src/myapp.dol -o src/generated/myapp.rs
   ```

---

## Resources

### Official Links
- [GitHub Repository](https://github.com/univrs/dol) - Source code and issues
- [Crates.io](https://crates.io/crates/dol) - Rust package registry
- [API Documentation](https://docs.rs/dol) - Rust API docs

### Documentation
- [HIR Tutorials](../tutorials/HIR_Tutorials.md) - Detailed HIR documentation
- [DOL Self-Hosting](./DOL_Self_Hosting.md) - Status of the DOL-in-DOL effort (not self-hosting)
- [Examples Gallery](./DOL_Examples_Gallery.md) - Comprehensive examples

---

## Backward Compatibility

The schedule originally announced here (`let`, `each`/`all`, `module` and `derives from` becoming errors in v0.4.0) **did not happen**. As of dol 0.8.2:

| Feature | Status in 0.8.2 |
|---------|-----------------|
| `let` | Accepted, no warning |
| `let mut` | Does not parse |
| `each`/`all` statements | Accepted, no warning |
| `module` | Accepted, no warning |
| `derives from` | Accepted, no warning |
| `gene`, `exegesis`, `constraint`, `evolves`, PascalCase types | Accepted with a "deprecated in v0.8.0" warning; planned to become errors in v0.9 (unreleased) |

---

*Questions? Visit [github.com/univrs/dol](https://github.com/univrs/dol) or join our Discord.*
