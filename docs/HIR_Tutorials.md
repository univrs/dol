# DOL v0.3.0 HIR Tutorials

> **High-level Intermediate Representation for Ontology-First Development**

[![GitHub Release](https://img.shields.io/github/v/release/univrs/dol)](https://github.com/univrs/dol/releases/tag/v0.3.0)
[![Crates.io](https://img.shields.io/crates/v/dol)](https://crates.io/crates/dol/0.3.0)
[![docs.rs](https://docs.rs/dol/badge.svg)](https://docs.rs/dol)

**Get DOL v0.3.0:**
- **GitHub**: [github.com/univrs/dol/releases/tag/v0.3.0](https://github.com/univrs/dol/releases/tag/v0.3.0)
- **Crates.io**: [crates.io/crates/dol/0.3.0](https://crates.io/crates/dol/0.3.0)

---

## Table of Contents

1. [Introduction to HIR](#introduction-to-hir)
2. [The Compilation Pipeline](#the-compilation-pipeline)
3. [HIR Node Types](#hir-node-types)
4. [Desugaring Rules](#desugaring-rules)
5. [DOL-in-DOL Development](#dol-in-dol-development)
6. [Code Generation](#code-generation)
7. [Migration Guide](#migration-guide)
8. [Examples](#examples)

---

## Introduction to HIR

HIR (High-level Intermediate Representation) is the canonical representation for DOL programs. It serves as the bridge between the surface syntax (what developers write) and code generation (what gets compiled).

### Why HIR?

| Aspect | Before (AST) | After (HIR) |
|--------|--------------|-------------|
| Node Types | 50+ | 22 |
| Keywords | 93 | ~55 |
| Representations per concept | Multiple | One |
| Codegen complexity | High | Low |

### Design Principles

1. **Minimal**: 22 node types cover all language constructs
2. **Canonical**: One representation per concept
3. **Typed**: All expressions carry type information
4. **Desugared**: No syntactic sugar remains

---

## The Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                    DOL v0.3.0 Compilation Pipeline                  │
└─────────────────────────────────────────────────────────────────────┘

    ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
    │   DOL    │     │   AST    │     │   HIR    │     │   Rust   │
    │  Source  │ ──► │  (Parse) │ ──► │ (Lower)  │ ──► │ (Codegen)│
    └──────────┘     └──────────┘     └──────────┘     └──────────┘
         │                │                │                │
         │                │                │                │
         ▼                ▼                ▼                ▼
    .dol files       50+ nodes        22 nodes        .rs files
    val/var          let/mut          Val/Var         let/let mut
    forall           each/all         Loop+Match      for/while
    extends          derives from     Extends         : Parent
```

### Pipeline Stages

#### Stage 1: Parsing (Source → AST)

The parser accepts both old and new syntax:

```dol
// Both of these parse to the same AST:
let x = 42          // v0.2.x syntax (deprecated)
val x = 42          // v0.3.0 syntax (preferred)
```

#### Stage 2: Lowering (AST → HIR)

Lowering transforms the AST into canonical HIR form:

```rust
use dol::lower::lower_file;

let source = r#"
gene point.2d {
    point has x
    point has y
}

exegesis {
    A 2D point with x and y coordinates.
}
"#;

let (hir, ctx) = lower_file(source)?;

// ctx contains:
// - Symbol table (interned strings)
// - Span map (source locations)
// - Diagnostics (deprecation warnings)
```

#### Stage 3: Code Generation (HIR → Rust)

The HIR codegen produces clean Rust code:

```rust
use dol::codegen::compile_to_rust_via_hir;

let rust_code = compile_to_rust_via_hir(source)?;
// Produces:
// pub struct Point2d {
//     pub x: String,
//     pub y: String,
// }
```

---

## HIR Node Types

### Overview (22 Total)

| Category | Count | Types |
|----------|-------|-------|
| Declarations | 4 | Type, Trait, Function, Module |
| Expressions | 12 | Literal, Var, Binary, Unary, Call, MethodCall, Field, Index, Block, If, Match, Lambda |
| Statements | 6 | Val, Var, Assign, Expr, Return, Break |
| Types | 8 | Named, Tuple, Array, Function, Ref, Optional, Var, Error |
| Patterns | 6 | Wildcard, Var, Literal, Constructor, Tuple, Or |

### Declaration Forms

```dol
// All declarations desugar to one of 4 forms:

// 1. Type Declaration
pub type Container {
    id: UInt64
    name: String
    status: ContainerStatus
}

// 2. Trait Declaration
trait Lifecycle {
    start: fun() -> Void
    stop: fun() -> Void
}

// 3. Function Declaration
fun process(val input: String) -> Result {
    // ...
}

// 4. Module Declaration
mod container.runtime @ 0.3.0 {
    // nested declarations
}
```

### Expression Forms

```dol
// 12 expression forms in HIR:

// Atoms
val x = 42                      // Literal
val y = x                       // Var

// Compound
val sum = a + b                 // Binary
val neg = -x                    // Unary
val result = process(data)      // Call
val len = list.length()         // MethodCall
val name = user.name            // Field
val first = items[0]            // Index

// Control
val value = { stmt1; stmt2; expr }  // Block
val max = if a > b { a } else { b } // If
val msg = match status {            // Match
    Ok(v) => v,
    Err(e) => "error"
}

// Functions
val double = |x| x * 2          // Lambda
```

### Statement Forms

```dol
// 6 statement forms (note val/var, not let/mut!):

val x = 42              // Immutable binding
var counter = 0         // Mutable binding
counter = counter + 1   // Assignment
process(data)           // Expression statement
return result           // Return
break                   // Break (with optional value)
```

---

## Desugaring Rules

### Binding Syntax

| Surface Syntax | HIR Form | Status |
|----------------|----------|--------|
| `let x = 1` | `Val { name: x, ... }` | Deprecated |
| `val x = 1` | `Val { name: x, ... }` | Preferred |
| `let mut x = 1` | `Var { name: x, ... }` | Deprecated |
| `var x = 1` | `Var { name: x, ... }` | Preferred |

### Control Flow

| Surface Syntax | HIR Form |
|----------------|----------|
| `for x in xs { body }` | `Loop { Match(iter.next(), Some(x) => body, None => Break) }` |
| `while cond { body }` | `Loop { If(cond, body, Break) }` |
| `each x in xs { body }` | Same as `for` (deprecated) |

### Operators

| Surface Syntax | HIR Form |
|----------------|----------|
| `x \|> f \|> g` | `Call(g, Call(f, x))` |
| `f >> g` | `Lambda { Call(g, Call(f, param)) }` |
| `a && b` | `If(a, b, false)` |
| `a \|\| b` | `If(a, true, b)` |
| `x += 1` | `Assign(x, Binary(x, Add, 1))` |

### Keywords

| Before | After | HIR |
|--------|-------|-----|
| `each` | `forall` | Loop+Match |
| `all` | `forall` | Loop+Match |
| `module` | `mod` | Module |
| `never` | `not` | Unary(Not) |
| `derives from` | `extends` | Extends field |
| `matches` | `==` | Binary(Eq) |

---

## DOL-in-DOL Development

DOL is self-hosting - the compiler is written in DOL itself. Here's how the HIR types are defined in DOL:

### HIR Types in DOL Syntax

```dol
mod dol.hir @ 0.1.0

// Symbol interning for fast comparison
pub type Symbol {
    id: UInt32  // Index into symbol table
}

// HIR expression with attached type
pub type HirExpr {
    ty: HirType           // Every expression has a type
    kind: HirExprKind
}

// Expression kinds (12 forms)
pub type HirExprKind {
    kind: enum {
        // Atoms
        Lit { value: HirLit },
        Var { name: Symbol },

        // Compound
        Binary { left: Box<HirExpr>, op: BinOp, right: Box<HirExpr> },
        Unary { op: UnOp, operand: Box<HirExpr> },
        Call { func: Box<HirExpr>, args: List<HirExpr> },
        Field { expr: Box<HirExpr>, field: Symbol },
        Index { expr: Box<HirExpr>, index: Box<HirExpr> },

        // Control
        If { cond: Box<HirExpr>, then_: Box<HirExpr>, else_: Box<HirExpr> },
        Match { scrutinee: Box<HirExpr>, arms: List<HirArm> },
        Loop { body: Box<HirExpr>, label: Option<Symbol> },

        // Functions
        Lambda { params: List<HirParam>, body: Box<HirExpr> }
    }
}

// Statements (note val/var naming!)
pub type HirStmt {
    kind: enum {
        Val { name: Symbol, ty: HirType, value: HirExpr },  // Immutable
        Var { name: Symbol, ty: HirType, value: HirExpr },  // Mutable
        Assign { target: HirExpr, value: HirExpr },
        Expr { expr: HirExpr },
        Return { value: Option<HirExpr> },
        Break { value: Option<HirExpr>, label: Option<Symbol> }
    }
}
```

### Self-Hosted Compiler Example

From `dol/ast.dol` - the AST definitions in DOL:

```dol
mod dol.ast @ 0.3.0

/// DOL declaration - top-level construct
pub type Declaration {
    kind: enum {
        Gene {
            name: QualifiedName
            statements: List<Statement>
            exegesis: Option<Exegesis>
            extends: Option<QualifiedName>  // NEW in v0.3.0
        },
        Trait {
            name: QualifiedName
            statements: List<Statement>
            exegesis: Option<Exegesis>
        },
        Constraint {
            name: QualifiedName
            statements: List<Statement>
            exegesis: Option<Exegesis>
        },
        // ... more variants
    }
    span: Span
}

/// Statement within a declaration
pub type Statement {
    kind: enum {
        Has { subject: Identifier, property: Identifier, type_: Option<TypeExpr> },
        Is { subject: Identifier, type_name: Identifier },
        DerivesFrom { subject: Identifier, parent: Identifier },
        Requires { subject: Identifier, dependency: Identifier },
        Uses { subject: Identifier, resource: Identifier },
        Can { subject: Identifier, capability: Identifier }
    }
    span: Span
}
```

---

## Code Generation

### Using the HIR Codegen

```rust
use dol::codegen::{compile_to_rust_via_hir, compile_with_diagnostics};

// Simple compilation
let rust_code = compile_to_rust_via_hir(dol_source)?;

// With diagnostics (for deprecation warnings)
let (rust_code, diagnostics) = compile_with_diagnostics(dol_source)?;
for diag in diagnostics {
    eprintln!("Warning: {}", diag);
}
```

### Generated Code Example

**Input (DOL):**
```dol
gene container.runtime {
    container has id
    container has name
    container has status
    container extends image
}

exegesis {
    A runtime container instance.
}
```

**Output (Rust):**
```rust
// Generated from DOL HIR
// Source: container.runtime

/// A runtime container instance.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerRuntime {
    pub id: String,
    pub name: String,
    pub status: String,
}
```

---

## Migration Guide

### Automatic Migration

Use the `dol-migrate` tool:

```bash
# Migrate all files in a directory
dol migrate --from 0.2 --to 0.3 src/

# Preview changes without applying
dol migrate --from 0.2 --to 0.3 --dry-run src/

# Migrate a single file
dol migrate --from 0.2 --to 0.3 path/to/file.dol
```

### Manual Migration Checklist

| Before (v0.2.x) | After (v0.3.0) |
|-----------------|----------------|
| `let x = 1` | `val x = 1` |
| `let mut x = 1` | `var x = 1` |
| `each x in xs` | `forall x in xs` |
| `all xs satisfy p` | `forall xs satisfy p` |
| `module foo` | `mod foo` |
| `never empty` | `not empty` |
| `derives from parent` | `extends parent` |
| `matches expected` | `== expected` |
| `given x = ...` | `val x = ...` |
| `then assert` | `assert` |

### Deprecation Timeline

| Syntax | v0.3.0 | v0.4.0 | v1.0.0 |
|--------|--------|--------|--------|
| `let` | Warning | Warning | Error |
| `mut` | Warning | Warning | Error |
| `gene` | Works | Soft warn | Warning |
| `each` | Warning | Error | Removed |
| `module` | Warning | Error | Removed |

---

## Examples

These examples are verified from the DOL compiler test suite.

### Basic Gene Declaration
**Source**: `examples/genes/hello.world.dol`

```dol
gene hello.world {
  message has content
  message has sender
  message has timestamp
}

exegesis {
  The hello.world gene is the simplest possible DOL example.
}
```

### Gene with Typed Properties
**Source**: `tests/corpus/traits/trait_relationships.dol`

```dol
module tests.trait_relationships @ 1.0.0

pub gene SimpleValue {
    has value: String
    has count: Int64 = 0

    fun get_value() -> String {
        return this.value
    }
}
```

### Generic Types
**Source**: `tests/corpus/genes/nested_generics.dol`

```dol
module tests.nested_generics @ 1.0.0

// Simple generic
pub gene Container<T> {
    has item: T
}

// Nested generic with constraints
pub gene Bounded<T: Comparable> {
    has items: List<T>

    fun max() -> Option<T> {
        if this.items.is_empty() {
            return None
        }
        return Some(this.items.reduce(|a, b| if a > b { a } else { b }))
    }
}
```

### Traits with Laws
**Source**: `tests/corpus/genes/complex_constraints.dol`

```dol
pub trait Ordered {
    is compare(other: Self) -> Int64

    law reflexive {
        forall x: Self. x.compare(x) == 0
    }

    law antisymmetric {
        forall x: Self. forall y: Self.
            x.compare(y) <= 0 && y.compare(x) <= 0 implies
                x.compare(y) == 0
    }
}
```

### SEX Functions (Side Effects)
**Source**: `tests/corpus/sex/nested_sex.dol`

```dol
// Global mutable state
sex var COUNTER: Int64 = 0
sex var LOG: List<String> = []

// SEX function with side effects
sex fun increment() -> Int64 {
    COUNTER += 1
    return COUNTER
}

// Pure function with contained sex block
fun compute_with_logging(x: Int64) -> Int64 {
    result = x * 2 + 1

    sex {
        LOG.push("computed: " + result.to_string())
    }

    return result
}
```

### Evolution Chain
**Source**: `tests/corpus/genes/evolution_chain.dol`

```dol
// Base type
pub gene EntityV1 {
    has id: UInt32
    has name: String
}

// First evolution - add fields
evolves EntityV1 > EntityV2 @ 2.0.0 {
    added created_at: Int64 = 0
    added updated_at: Int64 = 0

    migrate from EntityV1 {
        return EntityV2 {
            ...old,
            created_at: 0,
            updated_at: 0
        }
    }
}
```

### Constraints with Quantifiers
**Source**: `tests/corpus/genes/complex_constraints.dol`

```dol
pub gene OrderedList {
    has items: List<Int64>

    // Simple constraint
    constraint non_empty {
        this.items.length() > 0
    }

    // Forall constraint
    constraint sorted {
        forall i: UInt64.
            i < this.items.length() - 1 implies
                this.items[i] <= this.items[i + 1]
    }

    // Exists constraint
    constraint has_positive {
        exists x: Int64. x in this.items && x > 0
    }
}
```

### System Declaration
**Source**: `examples/systems/greeting.service.dol`

```dol
system greeting.service @0.1.0 {
  requires entity.greetable >= 0.0.1
  requires greeting.protocol >= 0.0.1

  uses hello.world
  service has greeting.templates
  service has response.timeout
}

exegesis {
  The greeting.service system composes genes, traits, and constraints
  into a complete, versioned component.
}
```

---

## Quick Reference

### HIR Node Count: 22

```
Declarations:  4  (Type, Trait, Function, Module)
Expressions:  12  (Lit, Var, Binary, Unary, Call, MethodCall,
                   Field, Index, Block, If, Match, Lambda)
Statements:    6  (Val, Var, Assign, Expr, Return, Break)
```

### Keyword Changes Summary

```
let      → val     (immutable value)
let mut  → var     (mutable variable)
gene     → type    (gradual migration)
each/all → forall  (unified quantifier)
module   → mod     (shorter)
never    → not     (consistent)
derives from → extends (standard term)
matches  → ==      (standard operator)
```

### Compilation API

```rust
// Simple
let code = compile_to_rust_via_hir(source)?;

// With diagnostics
let (code, warnings) = compile_with_diagnostics(source)?;

// Low-level access
let (hir, ctx) = lower_file(source)?;
let mut codegen = HirRustCodegen::with_symbols(ctx.symbols);
let code = codegen.generate(&hir);
```

---

*"Simplicity is the ultimate sophistication."* — Leonardo da Vinci
