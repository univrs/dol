# DOL 2.0 Roadmap Checkpoint

> **Document Version:** 1.0.0  
> **Last Updated:** December 22, 2025  
> **Status:** Active Development  
> **Repository:** [github.com/univrs/metadol](https://github.com/univrs/metadol)

---

## Executive Summary

Design Ontology Language (DOL) 2.0 represents a major evolution from the specification-only DOL 1.x to a **Turing-complete programming language** with full type inference, functional composition, and multi-target code generation. This checkpoint documents our progress from concept to working compiler.

### Key Milestones Achieved

| Milestone | Status | Commit | Tests |
|-----------|--------|--------|-------|
| DOL 2.0 Specification | ✅ Complete | — | 130 .dol files validated |
| Phase 1: Lexer + Parser | ✅ Complete | `91f4b4d` | 272 passing |
| Phase 2: Type Checker | ✅ Complete | `ae0e688` | 87 passing |
| Phase 3.2: Rust Codegen | ✅ Complete | `7c61b46` | 8 passing |
| **Total Tests** | | | **367 passing** |

---

## Table of Contents

1. [DOL 1.x → DOL 2.0: What Changed](#dol-1x--dol-20-what-changed)
2. [Language Features](#language-features)
3. [Compiler Architecture](#compiler-architecture)
4. [Phase 1: Lexer & Parser](#phase-1-lexer--parser)
5. [Phase 2: Type System](#phase-2-type-system)
6. [Phase 3: Code Generation](#phase-3-code-generation)
7. [CLI Tools](#cli-tools)
8. [Vocabulary System](#vocabulary-system)
9. [Roadmap: What's Next](#roadmap-whats-next)
10. [Migration Guide: DOL 1.x → 2.0](#migration-guide-dol-1x--20)
11. [Contributing](#contributing)

---

## DOL 1.x → DOL 2.0: What Changed

### DOL 1.x (Specification Language)

DOL 1.x was a **declarative specification language** for defining ontological contracts:

```dol
// DOL 1.x - Declaration only
gene ProcessId {
  type: UInt64
  constraint positive { this.value > 0 }
  exegesis { A process knows its identity. }
}
```

**Limitations:**
- No control flow
- No functions with bodies
- No expressions beyond constraints
- Required external runtime for execution

### DOL 2.0 (Programming Language)

DOL 2.0 is a **Turing-complete functional language** that compiles to multiple targets:

```dol
// DOL 2.0 - Full programming language
gene Calculator {
  type: Module
  
  fun fibonacci(n: Int64) -> Int64 {
    match n {
      0 { return 0 }
      1 { return 1 }
      _ { return fibonacci(n - 1) + fibonacci(n - 2) }
    }
  }
  
  fun pipeline(x: Int64) -> Int64 {
    return x |> double >> increment |> square
  }
  
  constraint valid_input {
    this.n >= 0
  }
  
  exegesis {
    Mathematical operations with full type safety.
  }
}
```

### Comparison Table

| Feature | DOL 1.x | DOL 2.0 |
|---------|---------|---------|
| Gene definitions | ✅ | ✅ |
| Trait definitions | ✅ | ✅ |
| Constraints | ✅ | ✅ Enhanced |
| Exegesis | ✅ | ✅ |
| Control flow | ❌ | ✅ `if/else/match/for/while/loop` |
| Functions with bodies | ❌ | ✅ |
| Lambda expressions | ❌ | ✅ `(x) -> x * 2` |
| Pipe operators | ❌ | ✅ `\|>` `>>` `<\|` |
| Pattern matching | ❌ | ✅ With guards |
| Type inference | ❌ | ✅ Bidirectional |
| Meta-programming | ❌ | ✅ `'` `!` `#` `?` |
| Compiles to Rust | ❌ | ✅ |
| Compiles to WASM | ❌ | 🚧 In progress |
| Self-hosting | ❌ | 🎯 Year 1 Q4 goal |

---

## Language Features

### Operators by Category

#### Composition Operators

| Operator | Name | Description | Example |
|----------|------|-------------|---------|
| `\|>` | Pipe | Forward function application | `x \|> f` → `f(x)` |
| `>>` | Compose | Function composition (left-to-right) | `f >> g` → `(x) -> g(f(x))` |
| `<\|` | Back-pipe | Reverse function application | `f <\| x` → `f(x)` |
| `@` | Apply | Applicative apply | `f @ x` |
| `:=` | Bind | Monadic bind | `m := f` |

#### Meta-Programming Operators

| Operator | Name | Description | Example |
|----------|------|-------------|---------|
| `'` | Quote | Defer evaluation | `'expr` |
| `!` | Eval | Force evaluation | `!quoted` |
| `#` | Macro | Macro expansion | `#macro_name` |
| `?` | Reflect | Runtime reflection | `?type` |
| `[\|` | Idiom Open | Begin idiom bracket | `[\| f x y \|]` |
| `\|]` | Idiom Close | End idiom bracket | |

#### Control Flow Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `if` | Conditional branch | `if x > 0 { ... }` |
| `else` | Alternative branch | `else { ... }` |
| `match` | Pattern matching | `match x { 0 { } _ { } }` |
| `for` | Iteration | `for item in items { }` |
| `while` | Conditional loop | `while x > 0 { }` |
| `loop` | Infinite loop | `loop { break }` |
| `break` | Exit loop | `break` |
| `continue` | Next iteration | `continue` |
| `return` | Return value | `return x` |
| `where` | Pattern guard | `n where n > 0` |

#### Type & Lambda Operators

| Operator | Name | Description | Example |
|----------|------|-------------|---------|
| `->` | Arrow | Function type / lambda | `Int32 -> Int32` |
| `=>` | Fat Arrow | Constraint implication | `A => B` |
| `\|` | Bar | Union types / match arms | `A \| B` |
| `_` | Wildcard | Match anything | `match x { _ { } }` |

### Type System

#### Primitive Types

| Type | Rust Equivalent | Description |
|------|-----------------|-------------|
| `Void` | `()` | No value |
| `Bool` | `bool` | Boolean |
| `Int8` | `i8` | 8-bit signed |
| `Int16` | `i16` | 16-bit signed |
| `Int32` | `i32` | 32-bit signed |
| `Int64` | `i64` | 64-bit signed (default) |
| `UInt8` | `u8` | 8-bit unsigned |
| `UInt16` | `u16` | 16-bit unsigned |
| `UInt32` | `u32` | 32-bit unsigned |
| `UInt64` | `u64` | 64-bit unsigned |
| `Float32` | `f32` | 32-bit float |
| `Float64` | `f64` | 64-bit float (default) |
| `String` | `String` | UTF-8 string |

#### Compound Types

| Type | Rust Equivalent | Example |
|------|-----------------|---------|
| `List<T>` | `Vec<T>` | `List<Int32>` |
| `Map<K, V>` | `HashMap<K, V>` | `Map<String, Int64>` |
| `Option<T>` | `Option<T>` | `Option<String>` |
| `Result<T, E>` | `Result<T, E>` | `Result<Int32, String>` |
| `Function<A, B>` | `fn(A) -> B` | `Function<Int32, Bool>` |
| `Tuple<...>` | `(A, B, ...)` | `Tuple<Int32, String>` |

#### Type Inference

The type checker performs **bidirectional type inference**:

```dol
// Literals infer their types
x = 42          // x: Int64 (inferred)
y = 3.14        // y: Float64 (inferred)
z = "hello"     // z: String (inferred)

// Functions infer return types
fun double(n: Int32) -> Int32 {
  return n * 2  // Verified: Int32 * Int32 → Int32
}

// Pipes propagate types
result = 10 |> double >> increment
// 10: Int64 → double: Int32 → increment: Int32
```

---

## Compiler Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        DOL 2.0 Compiler                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────┐    ┌─────────┐    ┌───────────┐    ┌───────────┐  │
│  │  Source │ -> │  Lexer  │ -> │   Parser  │ -> │    AST    │  │
│  │  (.dol) │    │         │    │  (Pratt)  │    │           │  │
│  └─────────┘    └─────────┘    └───────────┘    └─────┬─────┘  │
│                                                       │        │
│                                                       ▼        │
│                                               ┌───────────┐    │
│                                               │   Type    │    │
│                                               │  Checker  │    │
│                                               └─────┬─────┘    │
│                                                     │          │
│                     ┌───────────────────────────────┼────┐     │
│                     │                               │    │     │
│                     ▼                               ▼    ▼     │
│              ┌───────────┐                   ┌─────────────┐   │
│              │   Rust    │                   │    MLIR     │   │
│              │  Codegen  │                   │   Codegen   │   │
│              └─────┬─────┘                   └──────┬──────┘   │
│                    │                                │          │
│                    ▼                                ▼          │
│              ┌───────────┐                   ┌───────────┐     │
│              │   .rs     │                   │   WASM    │     │
│              │  files    │                   │  binary   │     │
│              └───────────┘                   └───────────┘     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Crate Structure

```
metadol/
├── src/
│   ├── lib.rs              # Library root, re-exports
│   ├── lexer.rs            # Tokenization (logos)
│   ├── ast.rs              # Abstract Syntax Tree nodes
│   ├── parser.rs           # Recursive descent parser
│   ├── pratt.rs            # Pratt parser for expressions
│   ├── typechecker.rs      # Type inference & checking
│   ├── codegen/
│   │   ├── mod.rs          # Codegen traits
│   │   └── rust.rs         # Rust code generator
│   └── bin/
│       ├── dol-parse.rs    # Parse CLI
│       ├── dol-check.rs    # Type check CLI
│       ├── dol-test.rs     # Test runner CLI
│       └── dol-codegen.rs  # Code generation CLI
├── examples/
│   ├── genes/              # Gene definitions
│   ├── traits/             # Trait definitions
│   └── stdlib/             # Standard library (10 files)
└── tests/
    ├── lexer_tests.rs
    ├── parser_tests.rs
    ├── typechecker_tests.rs
    └── codegen_tests.rs
```

---

## Phase 1: Lexer & Parser

**Commit:** `91f4b4d`  
**Tests:** 272 passing  
**Status:** ✅ Complete

### Lexer (src/lexer.rs)

Built with [logos](https://github.com/maciejhirsz/logos) for high-performance tokenization.

#### Token Categories

| Category | Tokens |
|----------|--------|
| Keywords | `gene`, `trait`, `system`, `constraint`, `evolves`, `exegesis`, `fun`, `return`, `match`, `if`, `else`, `for`, `while`, `loop`, `break`, `continue`, `where`, `in`, `requires`, `law`, `type`, `has`, `is` |
| Types | `Int8`, `Int16`, `Int32`, `Int64`, `UInt8`, `UInt16`, `UInt32`, `UInt64`, `Float32`, `Float64`, `Bool`, `String`, `Void`, `Self` |
| Operators | `\|>`, `>>`, `<\|`, `@`, `:=`, `->`, `=>`, `==`, `!=`, `>=`, `<=`, `>`, `<`, `+`, `-`, `*`, `/`, `%`, `&&`, `\|\|`, `!`, `'`, `#`, `?`, `[\|`, `\|]` |
| Delimiters | `{`, `}`, `(`, `)`, `[`, `]`, `:`, `,`, `.`, `_` |
| Literals | Integers, Floats, Strings, Booleans |
| Identifiers | `[a-zA-Z_][a-zA-Z0-9_]*` |

### Parser (src/parser.rs + src/pratt.rs)

Hybrid parser combining:
- **Recursive descent** for declarations (gene, trait, system)
- **Pratt parsing** for expressions with operator precedence

#### Operator Precedence Table

| Level | Operators | Associativity | Description |
|-------|-----------|---------------|-------------|
| 1 | `\|\|` | Left | Logical OR |
| 2 | `&&` | Left | Logical AND |
| 3 | `==` `!=` | Left | Equality |
| 4 | `<` `>` `<=` `>=` | Left | Comparison |
| 5 | `\|>` `<\|` | Left | Pipe |
| 6 | `>>` | Left | Compose |
| 7 | `:=` | Right | Bind |
| 8 | `@` | Left | Apply |
| 9 | `+` `-` | Left | Additive |
| 10 | `*` `/` `%` | Left | Multiplicative |
| 11 | `**` | Right | Power |
| 12 | `!` `-` `'` `?` | Prefix | Unary |
| 13 | `.` `()` `[]` | Left | Postfix/Call |

### AST Nodes (src/ast.rs)

```rust
// Top-level declarations
enum Declaration {
    Gene { name, body },
    Trait { name, type_params, body },
    System { name, body },
    Constraint { name, body },
    Evolution { from, to, body },
    Function { name, params, return_type, body },
}

// Expressions
enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary { op, left, right },
    Unary { op, operand },
    Call { callee, args },
    Lambda { params, body },
    If { condition, then_branch, else_branch },
    Match { scrutinee, arms },
    Block { statements, expr },
    // ...
}

// Statements
enum Stmt {
    Let { name, type_ann, value },
    Expr(Expr),
    Return(Option<Expr>),
    For { var, iter, body },
    While { condition, body },
    Loop { body },
    Break,
    Continue,
}
```

---

## Phase 2: Type System

**Commit:** `ae0e688`  
**File:** `src/typechecker.rs` (1,193 lines)  
**Tests:** 87 passing  
**Status:** ✅ Complete

### Type Representation

```rust
enum Type {
    // Primitives
    Void, Bool,
    Int8, Int16, Int32, Int64,
    UInt8, UInt16, UInt32, UInt64,
    Float32, Float64,
    String,
    
    // Compound
    Function { params: Vec<Type>, ret: Box<Type> },
    Tuple(Vec<Type>),
    
    // Parametric
    Generic { name: String, params: Vec<Type> },
    
    // Inference
    Var(usize),      // Type variable for unification
    Unknown,         // Not yet inferred
    Any,             // Accepts anything
    Error,           // Type error occurred
}
```

### Type Environment

```rust
struct TypeEnv {
    bindings: HashMap<String, Type>,
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    fn child(&self) -> TypeEnv;       // Create nested scope
    fn bind(&mut self, name, ty);     // Add binding
    fn lookup(&self, name) -> Option<Type>;  // Find in scope chain
}
```

### Inference Rules

| Expression | Inferred Type |
|------------|---------------|
| `42` | `Int64` |
| `3.14` | `Float64` |
| `true` / `false` | `Bool` |
| `"hello"` | `String` |
| `a + b` | Unify numeric types |
| `a && b` | `Bool` (requires Bool operands) |
| `a == b` | `Bool` (requires same type operands) |
| `f(x)` | Return type of `f` |
| `x \|> f` | Return type of `f` |
| `f >> g` | `Function<A, C>` where `f: A→B`, `g: B→C` |
| `(x: T) -> e` | `Function<T, typeof(e)>` |
| `if c { a } else { b }` | Unify types of `a` and `b` |
| `match x { ... }` | Unify all arm types |

### Type Errors

```rust
enum TypeError {
    Mismatch { expected: Type, found: Type, span: Span },
    UndefinedVariable { name: String, span: Span },
    NotCallable { ty: Type, span: Span },
    ArityMismatch { expected: usize, found: usize, span: Span },
    NotNumeric { ty: Type, span: Span },
    IncompatibleBranches { then_ty: Type, else_ty: Type, span: Span },
}
```

---

## Phase 3: Code Generation

### 3.2: Rust Code Generator

**Commit:** `7c61b46`  
**File:** `src/codegen/rust.rs` (617 lines)  
**Tests:** 8 passing  
**Status:** ✅ Complete

#### DOL → Rust Mapping

| DOL Declaration | Generated Rust |
|-----------------|----------------|
| `gene Name { has field: Type }` | `struct Name { field: Type }` |
| `trait Name { is method() -> T }` | `trait Name { fn method(&self) -> T; }` |
| `constraint name { expr }` | `fn validate_name(value: &T) -> bool { expr }` |
| `system Name { requires ... }` | `mod name { /* docs */ }` |
| `evolution From -> To { }` | `/* Changelog comment */` |

#### Type Mapping

| DOL Type | Rust Type |
|----------|-----------|
| `Int8` | `i8` |
| `Int16` | `i16` |
| `Int32` | `i32` |
| `Int64` | `i64` |
| `UInt8` | `u8` |
| `UInt16` | `u16` |
| `UInt32` | `u32` |
| `UInt64` | `u64` |
| `Float32` | `f32` |
| `Float64` | `f64` |
| `Bool` | `bool` |
| `String` | `String` |
| `Void` | `()` |
| `List<T>` | `Vec<T>` |
| `Map<K, V>` | `std::collections::HashMap<K, V>` |
| `Option<T>` | `Option<T>` |
| `Result<T, E>` | `Result<T, E>` |

#### Example: DOL → Rust

**Input (container.dol):**
```dol
gene Container {
  has id: UInt64
  has name: String
  has running: Bool
  
  constraint valid_id {
    this.id > 0
  }
  
  exegesis {
    A container is an isolated execution environment.
  }
}
```

**Output (container.rs):**
```rust
/// A container is an isolated execution environment.
#[derive(Debug, Clone)]
pub struct Container {
    pub id: u64,
    pub name: String,
    pub running: bool,
}

impl Container {
    /// Validates the valid_id constraint
    pub fn validate_valid_id(&self) -> bool {
        self.id > 0
    }
    
    /// Validates all constraints
    pub fn validate(&self) -> bool {
        self.validate_valid_id()
    }
}
```

---

## CLI Tools

### dol-parse

Parse DOL files and output AST:

```bash
# Parse single file
dol-parse examples/genes/container.dol

# Output JSON AST
dol-parse --format json examples/genes/container.dol

# Verbose with spans
dol-parse -v examples/genes/container.dol
```

### dol-check

Type check DOL files:

```bash
# Check single file
dol-check examples/genes/container.dol

# Check directory
dol-check examples/

# Strict mode (warnings as errors)
dol-check --strict examples/
```

### dol-test

Run DOL test suites:

```bash
# Run all tests
dol-test

# Run specific test file
dol-test tests/parser_tests.rs

# Verbose output
dol-test -v
```

### dol-codegen

Generate code from DOL:

```bash
# Generate Rust (default)
dol-codegen examples/genes/container.dol

# Output to file
dol-codegen -o generated.rs examples/genes/container.dol

# Recursive directory processing
dol-codegen --recursive examples/

# Future: other targets
dol-codegen --target wasm examples/genes/container.dol
dol-codegen --target typescript examples/genes/container.dol
```

---

## Vocabulary System

DOL 2.0 maintains the **dual vocabulary** system for developer and creator audiences:

### Developer Vocabulary (Technical)

| Term | Description |
|------|-------------|
| Gene | Fundamental unit of specification |
| Trait | Interface contract with laws |
| System | Composed module of genes |
| Constraint | Validation rule |
| Evolution | Version migration path |
| Exegesis | Self-documentation |

### Creator Vocabulary (Mystical)

| Technical | Creator | Description |
|-----------|---------|-------------|
| Module | Spirit | Shareable .dol package |
| Function | Spell | Executable transformation |
| Session | Séance | Collaborative editing session |
| Service | Loa | Autonomous background service |
| Network | Mycelium | P2P communication fabric |
| Hub | Mitan | Central coordination point |
| Token | Gris-Gris | Authentication credential |
| Admin | Mambo | System administrator |
| Creator | Bondye | Original author |

---

## Roadmap: What's Next

### Year 1: Genesis — "The language that writes itself"

| Quarter | Milestone | Status |
|---------|-----------|--------|
| Q1 | DOL Turing Extensions | ✅ Complete |
| | - Lexer + Parser | ✅ `91f4b4d` |
| | - Type Checker | ✅ `ae0e688` |
| | - Rust Codegen | ✅ `7c61b46` |
| Q2 | Meta-Programming | 🚧 Next |
| | - Quote/Eval (`'`, `!`) | ⏳ |
| | - Macro System (`#`) | ⏳ |
| | - Reflection (`?`) | ⏳ |
| Q3 | LLVM MCP Server | ⏳ Planned |
| | - MLIR Codegen | ⏳ |
| | - WASM Backend | ⏳ |
| | - MCP Integration | ⏳ |
| Q4 | Self-Hosting | ⏳ Planned |
| | - DOL compiles DOL | ⏳ |
| | - Bootstrap compiler | ⏳ |

### Year 2: Manifestation — "The machine that runs Spirits"

| Quarter | Milestone |
|---------|-----------|
| Q1 | VUDO VM (WASM runtime) |
| Q2 | VUDO OS (Spirit orchestration) |
| Q3 | Tauri IDE (Visual editor) |
| Q4 | Mycelium Network (P2P) |

### Year 3: Emergence — "The garden that grows itself"

| Quarter | Milestone |
|---------|-----------|
| Q1 | Mycelial Credits (Economics) |
| Q2 | Spirit Marketplace |
| Q3 | Browser IDE |
| Q4 | Imaginarium Launch |

---

## Migration Guide: DOL 1.x → 2.0

### Backward Compatibility

DOL 2.0 is **fully backward compatible** with DOL 1.x. All existing `.dol` files will parse and type-check correctly.

**Verified:** 10/10 stdlib files pass with DOL 2.0 compiler.

### New Features Available

When migrating, you can optionally adopt:

#### 1. Function Bodies

```dol
// DOL 1.x
gene Math {
  is add(a: Int32, b: Int32) -> Int32
}

// DOL 2.0 - Add implementation
gene Math {
  fun add(a: Int32, b: Int32) -> Int32 {
    return a + b
  }
}
```

#### 2. Pattern Matching

```dol
// DOL 2.0
fun describe(n: Int32) -> String {
  match n {
    0 { return "zero" }
    n where n > 0 { return "positive" }
    _ { return "negative" }
  }
}
```

#### 3. Pipe Operators

```dol
// DOL 2.0
fun process(x: Int32) -> Int32 {
  return x |> validate >> transform |> finalize
}
```

#### 4. Lambda Expressions

```dol
// DOL 2.0
items.map((x) -> x * 2)
items.filter((x) -> x > 0)
```

### Breaking Changes

**None.** DOL 2.0 is a strict superset of DOL 1.x.

---

## Contributing

### Development Setup

```bash
# Clone repository
git clone https://github.com/univrs/metadol.git
cd metadol

# Build
cargo build

# Run tests
cargo test

# Run all tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Lint
cargo clippy
```

### Test Coverage

| Module | Tests | Status |
|--------|-------|--------|
| Lexer | 80+ | ✅ |
| Parser | 150+ | ✅ |
| Type Checker | 87 | ✅ |
| Rust Codegen | 8 | ✅ |
| **Total** | **367** | ✅ |

### Adding New Features

1. Write failing test in `tests/`
2. Implement feature in `src/`
3. Run `cargo test`
4. Run `cargo fmt && cargo clippy`
5. Submit PR

### Documentation

- **API Docs:** `cargo doc --open`
- **Book:** [book.univrs.io](https://book.univrs.io)
- **Learn:** [learn.univrs.io/dol](https://learn.univrs.io/dol)

---

## References

- [DOL 2.0 Specification](https://github.com/univrs/metadol/tree/main/spec)
- [VUDO Landing](https://vudo.univrs.io)
- [Univrs Documentation](https://learn.univrs.io)
- [GitHub Repository](https://github.com/univrs/metadol)

---

*"Systems designed to evolve and adapt to change."*

— The VUDO Team
