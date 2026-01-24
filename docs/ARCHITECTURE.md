# DOL v0.8.0 Architecture

> Domain Ontology Language - Comprehensive Technical Reference

## Executive Summary

DOL (Domain Ontology Language) v0.8.0 is a declarative domain-specific language for ontology-first software development. It provides a complete compiler toolchain that transforms natural-language-like specifications into executable code for multiple targets.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DOL v0.8.0 COMPILER PIPELINE                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────┐    ┌────────┐    ┌─────┐    ┌─────┐    ┌──────────────────┐  │
│   │  .dol   │───▶│ Lexer  │───▶│ AST │───▶│ HIR │───▶│ Code Generation  │  │
│   │ Source  │    │(logos) │    │     │    │     │    │                  │  │
│   └─────────┘    └────────┘    └─────┘    └─────┘    │  ┌────────────┐  │  │
│                       │            │          │       │  │    Rust    │  │  │
│                       ▼            ▼          ▼       │  ├────────────┤  │  │
│                  ┌─────────┐  ┌─────────┐ ┌───────┐   │  │ TypeScript │  │  │
│                  │ Tokens  │  │Validator│ │ Type  │   │  ├────────────┤  │  │
│                  └─────────┘  └─────────┘ │Checker│   │  │ JSON Schema│  │  │
│                                           └───────┘   │  ├────────────┤  │  │
│                                                       │  │    WASM    │  │  │
│                                                       │  └────────────┘  │  │
│                                                       └──────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key Metrics:**
- **Version**: 0.8.0 "Clarity"
- **Rust Edition**: 2021 (MSRV 1.81)
- **Library Name**: `metadol` (Metal DOL)
- **Total Tests**: 631 passing
- **Codebase**: ~1M lines

---

## Table of Contents

1. [Project Structure](#1-project-structure)
2. [v0.8.0 Language Changes](#2-v080-language-changes)
3. [Lexer Architecture](#3-lexer-architecture)
4. [Parser Architecture](#4-parser-architecture)
5. [Abstract Syntax Tree](#5-abstract-syntax-tree)
6. [High-Level IR (HIR)](#6-high-level-ir-hir)
7. [Type System](#7-type-system)
8. [Code Generation](#8-code-generation)
9. [WASM Backend](#9-wasm-backend)
10. [Macro System](#10-macro-system)
11. [SEX (Side Effect eXecution)](#11-sex-side-effect-execution)
12. [MCP Integration](#12-mcp-integration)
13. [CLI Tools](#13-cli-tools)
14. [Transformation Passes](#14-transformation-passes)
15. [Operator Precedence](#15-operator-precedence)
16. [What DOL Can Build](#16-what-dol-can-build)

---

## 1. Project Structure

```
univrs-dol/
├── Cargo.toml                    # Workspace manifest
├── src/
│   ├── lib.rs                    # Library entry point
│   │
│   ├── lexer.rs                  # Tokenization (logos-based)
│   ├── parser.rs                 # Recursive descent parser
│   ├── ast.rs                    # AST node definitions
│   ├── pratt.rs                  # Operator precedence table
│   │
│   ├── hir/                      # High-Level IR
│   │   ├── mod.rs                # HIR node types
│   │   ├── types.rs              # HIR type system
│   │   ├── symbol.rs             # Symbol table
│   │   └── validate.rs           # HIR validation
│   │
│   ├── lower/                    # AST → HIR lowering
│   │   ├── mod.rs                # Lowering pipeline
│   │   ├── desugar.rs            # Desugaring rules
│   │   └── expr.rs               # Expression lowering
│   │
│   ├── typechecker.rs            # Type inference & checking
│   ├── validator.rs              # Semantic validation
│   │
│   ├── codegen/                  # Code generators
│   │   ├── rust.rs               # Rust codegen
│   │   ├── typescript.rs         # TypeScript codegen
│   │   ├── jsonschema.rs         # JSON Schema codegen
│   │   └── hir_rust.rs           # HIR-based Rust codegen
│   │
│   ├── wasm/                     # WebAssembly backend
│   │   ├── compiler.rs           # WASM code emission
│   │   └── layout.rs             # Memory layout
│   │
│   ├── macros/                   # Macro system
│   │   ├── builtin.rs            # Built-in macros
│   │   └── expand.rs             # Macro expansion
│   │
│   ├── sex/                      # Side effect tracking
│   │   ├── context.rs            # SEX context
│   │   └── lint.rs               # Purity linting
│   │
│   ├── mcp/                      # MCP server
│   │   └── server.rs             # Tool implementations
│   │
│   ├── bin/                      # CLI tools
│   │   ├── dol-parse.rs
│   │   ├── dol-check.rs
│   │   ├── dol-codegen.rs
│   │   ├── dol-migrate.rs
│   │   ├── dol-mcp.rs
│   │   └── dol-test.rs
│   │
│   └── transform/                # AST transformation passes
│       ├── passes.rs             # Optimization passes
│       └── visitor.rs            # Visitor pattern
│
├── examples/                     # Example DOL files
│   ├── genes/
│   ├── traits/
│   ├── constraints/
│   └── evolutions/
│
├── stdlib/                       # Standard library
│
└── tests/                        # Test suite
    ├── codegen/
    ├── e2e/
    └── corpus/
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| `cli` | Enable all CLI binaries |
| `serde` | JSON serialization support |
| `mlir` | MLIR code generation (requires LLVM 18) |
| `wasm` | Full WASM compilation support |
| `wasm-runtime` | Native WASM execution (Wasmtime) |
| `vudo` | Container orchestration tools |

---

## 2. v0.8.0 Language Changes

### Renamed Keywords

| v0.7.x | v0.8.0+ | Description |
|--------|---------|-------------|
| `gene` | `gen` | Atomic type declaration |
| `constraint` | `rule` | Invariant declaration |
| `evolves` | `evo` | Evolution declaration |
| `exegesis` | `docs` | Documentation block |

Old keywords still work but emit deprecation warnings.

### Modernized Type Names

| v0.7.x | v0.8.0+ | Notes |
|--------|---------|-------|
| `Int8`, `Int16`, `Int32`, `Int64` | `i8`, `i16`, `i32`, `i64` | Rust-aligned |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | `u8`, `u16`, `u32`, `u64` | Rust-aligned |
| `Float32`, `Float64` | `f32`, `f64` | Rust-aligned |
| `BoolType` | `bool` | Rust-aligned |
| `StringType` | `string` | Rust-aligned |
| `Void` | `()` | Unit type |
| `List<T>` | `Vec<T>` | Vector type |

### Migration Tool

```bash
# Automatically migrate v0.7 files to v0.8 syntax
dol-migrate 0.7-to-0.8 src/

# Preview changes first
dol-migrate 0.7-to-0.8 --diff src/
```

---

## 3. Lexer Architecture

The lexer uses the `logos` crate for fast, zero-copy tokenization.

```
┌─────────────────────────────────────────────────────────┐
│                    LEXER PIPELINE                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│   Source Text                                           │
│        │                                                │
│        ▼                                                │
│   ┌─────────────────────────────────────────────────┐   │
│   │              logos Tokenizer                     │   │
│   │  ┌─────────┐ ┌─────────┐ ┌─────────┐           │   │
│   │  │Keywords │ │Operators│ │Literals │           │   │
│   │  └─────────┘ └─────────┘ └─────────┘           │   │
│   └─────────────────────────────────────────────────┘   │
│        │                                                │
│        ▼                                                │
│   Token Stream with Spans                               │
│        │                                                │
│        ▼                                                │
│   Parser                                                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Token Categories

**Declaration Keywords:**
```
gen, trait, rule, system, evo, docs
gene, constraint, evolves, exegesis  (deprecated aliases)
```

**Predicate Keywords:**
```
has, is, derives, from, requires, uses, emits, matches, never, can
```

**Evolution Keywords:**
```
adds, deprecates, removes, because
```

**Quantifiers:**
```
each, all, forall, exists
```

**Control Flow (DOL 2.0):**
```
if, else, match, for, while, loop, break, continue, return
let, val, var, fun, in, where, pub, module, use
```

**Special Operators (DOL 2.0):**

| Token | Symbol | Description |
|-------|--------|-------------|
| Pipe | `\|>` | Pipeline operator |
| Compose | `>>` | Function composition |
| BackPipe | `<\|` | Reverse pipeline |
| Quote | `'` | AST capture |
| Bang | `!` | Eval quoted expr |
| Reflect | `?` | Type introspection |
| Macro | `#` | Macro invocation |
| IdiomOpen | `[\|` | Applicative start |
| IdiomClose | `\|]` | Applicative end |

### Token Structure

```rust
pub struct Token {
    pub kind: TokenKind,      // Token category
    pub lexeme: String,       // Original text
    pub span: Span,           // Source location
}

pub struct Span {
    pub start: usize,         // Byte offset start
    pub end: usize,           // Byte offset end
    pub line: usize,          // Line number (1-indexed)
    pub column: usize,        // Column (1-indexed)
}
```

---

## 4. Parser Architecture

Recursive descent parser with two-token lookahead for ambiguity resolution.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PARSER ARCHITECTURE                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Token Stream                                                          │
│        │                                                                │
│        ▼                                                                │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                    Recursive Descent Parser                      │   │
│   │                                                                  │   │
│   │   ┌───────────────┐    ┌───────────────┐    ┌───────────────┐   │   │
│   │   │ parse_file()  │───▶│parse_decl()   │───▶│parse_stmt()   │   │   │
│   │   └───────────────┘    └───────────────┘    └───────────────┘   │   │
│   │          │                     │                    │            │   │
│   │          ▼                     ▼                    ▼            │   │
│   │   ┌───────────────┐    ┌───────────────┐    ┌───────────────┐   │   │
│   │   │ ModuleDecl    │    │ Gen/Trait/... │    │ Has/Is/Uses   │   │   │
│   │   └───────────────┘    └───────────────┘    └───────────────┘   │   │
│   │                                                                  │   │
│   │   ┌─────────────────────────────────────────────────────────┐   │   │
│   │   │              Pratt Parser (Expressions)                  │   │   │
│   │   │   parse_expression(precedence) → Expr                    │   │   │
│   │   └─────────────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│        │                                                                │
│        ▼                                                                │
│   Abstract Syntax Tree (AST)                                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Grammar Summary

```
file       := [module] [use*] declaration*
module     := 'module' path '@' version

declaration := gen | trait | rule | system | evo | fun | const

gen        := 'gen' NAME '{' statement* '}' docs
trait      := 'trait' NAME '{' statement* '}' docs
rule       := 'rule' NAME '{' statement* '}' docs
system     := 'system' NAME '@' VERSION '{' requirement* statement* '}' docs
evo        := 'evo' NAME '@' VERSION '>' PARENT '{' change* '}' docs

statement  := has_stmt | is_stmt | uses_stmt | derives_stmt
            | matches_stmt | never_stmt | emits_stmt | quantified

has_stmt   := SUBJECT 'has' PROPERTY [':' type] ['=' default]
is_stmt    := SUBJECT 'is' STATE
uses_stmt  := 'uses' REFERENCE
derives_stmt := SUBJECT 'derives' 'from' ORIGIN
matches_stmt := SUBJECT 'matches' TARGET
never_stmt := SUBJECT 'never' ACTION
emits_stmt := ACTION 'emits' EVENT
quantified := ('each' | 'all') phrase predicate

docs       := 'docs' '{' text '}'
```

### Expression Parsing (Pratt Parser)

For DOL 2.0 expressions, a Pratt parser handles operator precedence:

```rust
fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
    let mut lhs = self.parse_prefix()?;

    while let Some((l_bp, r_bp)) = self.infix_binding_power() {
        if l_bp < min_bp { break; }
        self.advance();
        let rhs = self.parse_expression(r_bp)?;
        lhs = self.make_binary(lhs, op, rhs);
    }

    Ok(lhs)
}
```

---

## 5. Abstract Syntax Tree

### Node Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           AST NODE TYPES                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   DolFile                                                               │
│   ├── module: Option<ModuleDecl>                                        │
│   ├── uses: Vec<UseDecl>                                                │
│   └── declarations: Vec<Declaration>                                    │
│                                                                         │
│   Declaration (enum)                                                    │
│   ├── Gen { name, statements, docs, span }                              │
│   ├── Trait { name, statements, docs, span }                            │
│   ├── Rule { name, statements, docs, span }                             │
│   ├── System { name, version, requirements, statements, docs, span }    │
│   ├── Evo { name, version, parent, adds, deprecates, removes, span }    │
│   ├── Function { name, params, return_type, body, span }                │
│   └── Const { name, type, value, span }                                 │
│                                                                         │
│   Statement (enum)                                                      │
│   ├── Has { subject, property, span }                                   │
│   ├── HasField { subject, name, type, default, constraint, span }       │
│   ├── Is { subject, state, span }                                       │
│   ├── DerivesFrom { subject, origin, span }                             │
│   ├── Uses { reference, span }                                          │
│   ├── Emits { action, event, span }                                     │
│   ├── Matches { subject, target, span }                                 │
│   ├── Never { subject, action, span }                                   │
│   └── Quantified { quantifier, phrase, predicate, span }                │
│                                                                         │
│   Expr (enum) - DOL 2.0                                                 │
│   ├── Literal(Literal)                                                  │
│   ├── Identifier(String)                                                │
│   ├── Binary { left, op, right }                                        │
│   ├── Unary { op, operand }                                             │
│   ├── Call { callee, args }                                             │
│   ├── Member { object, field }                                          │
│   ├── Lambda { params, body, return_type }                              │
│   ├── If { condition, then_branch, else_branch }                        │
│   ├── Match { scrutinee, arms }                                         │
│   ├── Block { statements, expr }                                        │
│   ├── Quote(Expr)           // 'expr - capture AST                      │
│   ├── Eval(Expr)            // !expr - evaluate quoted                  │
│   └── Reflect(TypeExpr)     // ?Type - introspection                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Core Types

```rust
pub struct Gen {
    pub visibility: Visibility,
    pub name: String,              // e.g., "container.exists"
    pub extends: Option<String>,   // Parent type
    pub statements: Vec<Statement>,
    pub exegesis: String,          // Mandatory docs
    pub span: Span,
}

pub struct System {
    pub name: String,              // e.g., "univrs.orchestrator"
    pub version: String,           // Semver
    pub requirements: Vec<Requirement>,
    pub statements: Vec<Statement>,
    pub exegesis: String,
    pub span: Span,
}

pub struct Evo {
    pub name: String,
    pub version: String,           // New version
    pub parent_version: String,    // Base version
    pub additions: Vec<Statement>,
    pub deprecations: Vec<Statement>,
    pub removals: Vec<String>,
    pub rationale: Option<String>, // 'because' clause
    pub exegesis: String,
    pub span: Span,
}
```

---

## 6. High-Level IR (HIR)

### Design Goals

| Goal | Description |
|------|-------------|
| Minimal | 22 node types (vs 50+ in AST) |
| Canonical | One representation per concept |
| Typed | All expressions carry type info |
| Desugared | No syntactic sugar remains |

### HIR Node Types

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         HIR NODE TYPES (22)                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   HirDecl (4)                   HirExpr (12)           HirStmt (6)      │
│   ├── Type                      ├── Literal            ├── Val          │
│   ├── Trait                     ├── Var                ├── Var          │
│   ├── Function                  ├── App                ├── Assign       │
│   └── Module                    ├── Lam                ├── Expr         │
│                                 ├── Let                ├── Return       │
│                                 ├── If                 └── Break        │
│                                 ├── Match                               │
│                                 ├── Proj                                │
│                                 ├── Call                                │
│                                 ├── BinOp                               │
│                                 ├── Record                              │
│                                 └── Tuple                               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Desugaring Rules

| Source Construct | Desugars To |
|------------------|-------------|
| `let x = e` | `Val { name: x, value: e }` |
| `var x = e` | `Var { name: x, value: e }` |
| `for x in xs { body }` | `loop { match iter.next() { ... } }` |
| `while c { body }` | `loop { if c { body } else { break } }` |
| `x \|> f \|> g` | `g(f(x))` |
| `a && b` | `if a { b } else { false }` |
| `a \|\| b` | `if a { true } else { b }` |
| `[\| f a b \|]` | `f <$> a <*> b` |

### Lowering Pipeline

```
┌─────────┐    ┌───────────────┐    ┌─────────────┐    ┌─────┐
│   AST   │───▶│   Desugar     │───▶│   Symbol    │───▶│ HIR │
│         │    │   Pass        │    │   Resolution│    │     │
└─────────┘    └───────────────┘    └─────────────┘    └─────┘
                     │
                     ▼
              Deprecation Warnings
              (old keywords detected)
```

---

## 7. Type System

### Type Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          DOL TYPE SYSTEM                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Primitives                                                            │
│   ├── Numeric: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128        │
│   ├── Floating: f32, f64                                                │
│   ├── Boolean: bool                                                     │
│   ├── String: string                                                    │
│   └── Unit: ()                                                          │
│                                                                         │
│   Compound                                                              │
│   ├── Vec<T>                    // Dynamic array                        │
│   ├── Option<T>                 // Optional value                       │
│   ├── Result<T, E>              // Success or error                     │
│   ├── Map<K, V>                 // Key-value mapping                    │
│   ├── (T1, T2, ...)             // Tuple                                │
│   └── [T; N]                    // Fixed array                          │
│                                                                         │
│   Function Types                                                        │
│   └── (T1, T2) -> R             // Function signature                   │
│                                                                         │
│   User-Defined                                                          │
│   ├── Named(String)             // Reference by name                    │
│   └── Generic<T1, T2>           // Parameterized type                   │
│                                                                         │
│   Type System                                                           │
│   ├── Var(n)                    // Type variable (inference)            │
│   ├── Any                       // Gradual typing                       │
│   ├── Never                     // Bottom type                          │
│   └── Unknown                   // Unresolved                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Type Inference

The type checker uses bidirectional type checking:

1. **Synthesis**: Derive type from expression
2. **Checking**: Verify expression matches expected type

```
Literal(42)           ⇒ i64
Literal(3.14)         ⇒ f64
Literal(true)         ⇒ bool
Literal("text")       ⇒ string

Binary(a + b)         ⇒ Numeric (unified from a and b)
Lambda(|x: i64| x+1)  ⇒ (i64) -> i64
If(c, t, e)           ⇒ T (where t ≈ e)
```

---

## 8. Code Generation

### Multi-Target Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      CODE GENERATION TARGETS                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                              AST / HIR                                  │
│                                  │                                      │
│            ┌─────────────────────┼─────────────────────┐                │
│            │                     │                     │                │
│            ▼                     ▼                     ▼                │
│   ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐      │
│   │   RustCodegen   │   │TypeScriptCodegen│   │JsonSchemaCodegen│      │
│   │                 │   │                 │   │                 │      │
│   │  ┌───────────┐  │   │  ┌───────────┐  │   │  ┌───────────┐  │      │
│   │  │  Structs  │  │   │  │Interfaces │  │   │  │  Schemas  │  │      │
│   │  │  Traits   │  │   │  │  Types    │  │   │  │Properties │  │      │
│   │  │  Impls    │  │   │  │  Enums    │  │   │  │Constraints│  │      │
│   │  └───────────┘  │   │  └───────────┘  │   │  └───────────┘  │      │
│   └─────────────────┘   └─────────────────┘   └─────────────────┘      │
│            │                     │                     │                │
│            ▼                     ▼                     ▼                │
│        .rs files            .ts files            .json files            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Type Mappings

| DOL Type | Rust | TypeScript | JSON Schema |
|----------|------|------------|-------------|
| `i8` | `i8` | `number` | `{ "type": "integer", "minimum": -128 }` |
| `i32` | `i32` | `number` | `{ "type": "integer" }` |
| `i64` | `i64` | `bigint` | `{ "type": "integer" }` |
| `f64` | `f64` | `number` | `{ "type": "number" }` |
| `bool` | `bool` | `boolean` | `{ "type": "boolean" }` |
| `string` | `String` | `string` | `{ "type": "string" }` |
| `Vec<T>` | `Vec<T>` | `T[]` | `{ "type": "array", "items": {...} }` |
| `Option<T>` | `Option<T>` | `T \| null` | `{ "oneOf": [...] }` |

### Generated Code Example

**DOL Input:**
```dol
gen container.exists {
  container has id
  container has name
  container has state
  container derives from creation
}

docs {
  A container represents an isolated execution environment.
}
```

**Rust Output:**
```rust
/// A container represents an isolated execution environment.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerExists {
    pub id: String,
    pub name: String,
    pub state: String,
}
```

---

## 9. WASM Backend

### Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      WASM COMPILATION PIPELINE                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   DOL Source                                                            │
│        │                                                                │
│        ▼                                                                │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                    Spirit Compiler                               │   │
│   │                                                                  │   │
│   │   Parse → Lower → TypeCheck → Optimize → WasmEmit               │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│        │                                                                │
│        ▼                                                                │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                    WASM Module                                   │   │
│   │   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐               │   │
│   │   │ Types   │ │Functions│ │ Memory  │ │ Exports │               │   │
│   │   └─────────┘ └─────────┘ └─────────┘ └─────────┘               │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│        │                                                                │
│        ▼                                                                │
│   .wasm Binary + Source Map                                             │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Memory Layout

The WASM compiler handles:
- Stack allocation for locals
- Heap allocation for dynamic data
- Field offset calculation with alignment
- Reference counting for memory safety

---

## 10. Macro System

### Built-in Macros (20+)

| Macro | Signature | Description |
|-------|-----------|-------------|
| `#stringify(e)` | `Expr → string` | Convert expression to string |
| `#concat(a,b,...)` | `string... → string` | Concatenate strings |
| `#env("VAR")` | `string → string` | Read environment variable |
| `#cfg(cond)` | `bool → ()` | Conditional compilation |
| `#derive(T,...)` | `Decl → Decl` | Generate trait implementations |
| `#assert(c)` | `bool → ()` | Runtime assertion |
| `#assert_eq(a,b)` | `T, T → ()` | Assert equality |
| `#format(fmt,...)` | `string,... → string` | String formatting |
| `#dbg(e)` | `T → T` | Debug print (returns value) |
| `#todo(msg)` | `string → Never` | Mark unimplemented |
| `#unreachable()` | `→ Never` | Mark unreachable code |
| `#compile_error(m)` | `string → Never` | Compile-time error |
| `#vec(a,b,c)` | `T... → Vec<T>` | Vector literal |
| `#file()` | `→ string` | Current filename |
| `#line()` | `→ u32` | Current line number |
| `#column()` | `→ u32` | Current column |
| `#module_path()` | `→ string` | Current module path |

### Macro Expansion

```
Source with Macros
        │
        ▼
┌───────────────────┐
│  MacroExpander    │
│  ┌─────────────┐  │
│  │BuiltinMacros│  │
│  └─────────────┘  │
│  ┌─────────────┐  │
│  │CustomMacros │  │
│  └─────────────┘  │
└───────────────────┘
        │
        ▼
Expanded AST
```

---

## 11. SEX (Side Effect eXecution)

DOL enforces purity by default. Side effects require explicit marking.

### Purity Zones

```
src/
├── pure/                    # Pure code (default)
│   ├── container.dol        # No I/O, no mutations
│   └── identity.dol
│
└── sex/                     # Side-effecting code
    ├── io.dol               # File I/O
    └── network.dol          # Network calls
```

### SEX Markers

```dol
// Pure function (default)
fun add(a: i64, b: i64) -> i64 {
  a + b
}

// Side-effecting function
sex fun write_log(msg: string) -> () {
  // I/O operation
}

// Side-effecting block
sex {
  // All code here can have effects
}
```

### Effect Tracking

| Context | Pure Calls | SEX Calls | I/O |
|---------|------------|-----------|-----|
| Pure | ✓ | ✗ | ✗ |
| SEX | ✓ | ✓ | ✓ |

---

## 12. MCP Integration

DOL provides an MCP server for AI assistant integration.

### Available Tools

| Tool | Description |
|------|-------------|
| `dol/parse` | Parse DOL source to AST |
| `dol/typecheck` | Type check DOL code |
| `dol/compile-rust` | Generate Rust code |
| `dol/compile-typescript` | Generate TypeScript |
| `dol/compile-wasm` | Compile to WebAssembly |
| `dol/eval` | Evaluate DOL expression |
| `dol/reflect` | Introspect DOL types |
| `dol/format` | Format DOL source |
| `dol/list-macros` | List available macros |
| `dol/expand-macro` | Expand macro invocation |

### Server Usage

```bash
# Start MCP server
dol-mcp serve --port 8000

# Or via stdio for subprocess mode
dol-mcp stdio
```

---

## 13. CLI Tools

### Tool Overview

| Binary | Purpose | Usage |
|--------|---------|-------|
| `dol-parse` | Parse to JSON AST | `dol-parse file.dol -o ast.json` |
| `dol-check` | Validate syntax/types | `dol-check file.dol --strict` |
| `dol-codegen` | Generate code | `dol-codegen file.dol --target rust` |
| `dol-test` | Run tests | `dol-test tests/` |
| `dol-mcp` | MCP server | `dol-mcp serve` |
| `dol-migrate` | Version migration | `dol-migrate 0.7-to-0.8 src/` |
| `dol-build-crate` | Generate Rust crate | `dol-build-crate file.dol` |
| `vudo` | Container orchestration | `vudo run spirit.dol` |

### Common Workflows

```bash
# Validate and type-check
dol-check src/*.dol --strict

# Generate Rust library
dol-codegen src/*.dol --target rust -o generated/

# Run test suite
dol-test tests/*.dol.test

# Compile to WASM
dol-codegen spirit.dol --target wasm -o app.wasm

# Migrate codebase to v0.8.0
dol-migrate 0.7-to-0.8 --diff src/
dol-migrate 0.7-to-0.8 src/
```

---

## 14. Transformation Passes

### Pass Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      TRANSFORMATION PIPELINE                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Input AST                                                             │
│       │                                                                 │
│       ▼                                                                 │
│   ┌─────────────────┐                                                   │
│   │ConstantFolding  │  Evaluate constant expressions at compile time   │
│   └─────────────────┘                                                   │
│       │                                                                 │
│       ▼                                                                 │
│   ┌─────────────────┐                                                   │
│   │DeadCodeElim     │  Remove unreachable code                          │
│   └─────────────────┘                                                   │
│       │                                                                 │
│       ▼                                                                 │
│   ┌─────────────────┐                                                   │
│   │TreeShaking      │  Remove unused declarations                       │
│   └─────────────────┘                                                   │
│       │                                                                 │
│       ▼                                                                 │
│   Optimized AST                                                         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Pass Interface

```rust
pub trait Pass {
    fn name(&self) -> &str;
    fn should_run(&self, decl: &Declaration) -> bool;
    fn run(&mut self, decl: Declaration) -> PassResult<Declaration>;
}
```

### Visitor Pattern

```rust
pub trait Visitor {
    fn visit_declaration(&mut self, decl: &Declaration);
    fn visit_expression(&mut self, expr: &Expr);
    fn visit_statement(&mut self, stmt: &Statement);
}
```

---

## 15. Operator Precedence

### Binding Power Table

| Operator | Left BP | Right BP | Associativity | Description |
|----------|---------|----------|---------------|-------------|
| `:=` | 10 | 9 | Right | Bind |
| `\|>` | 21 | 20 | Left | Pipe |
| `@` | 31 | 30 | Left | Apply |
| `>>` | 40 | 41 | Right | Compose |
| `->` | 50 | 51 | Right | Arrow |
| `.` | 55 | 55 | — | Member access |
| `\|\|` | 61 | 60 | Left | Logical OR |
| `&&` | 71 | 70 | Left | Logical AND |
| `==`, `!=` | 80 | 80 | — | Equality |
| `<`, `>`, `<=`, `>=` | 90 | 90 | — | Comparison |
| `+`, `-` | 101 | 100 | Left | Addition |
| `*`, `/`, `%` | 111 | 110 | Left | Multiplication |
| `^` | 120 | 121 | Right | Power |
| `as` | 131 | 130 | Left | Type cast |
| `'`, `!`, `?` | — | 135 | — | Quote/Eval/Reflect |

---

## 16. What DOL Can Build

### Ontology Specifications

DOL excels at defining domain models with:
- Type definitions (`gen`)
- Behavior interfaces (`trait`)
- Invariants (`rule`)
- System composition (`system`)
- Version evolution (`evo`)

### Code Generation Targets

| Target | Use Case |
|--------|----------|
| **Rust** | Backend services, CLI tools, libraries |
| **TypeScript** | Frontend types, API clients |
| **JSON Schema** | API validation, documentation |
| **WASM** | Browser apps, edge computing, plugins |

### System Capabilities

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     DOL CAPABILITY MATRIX                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Domain Modeling                                                       │
│   ├── Ontology-first design                                             │
│   ├── Natural language predicates                                       │
│   ├── Mandatory documentation                                           │
│   └── Evolution tracking                                                │
│                                                                         │
│   Type Safety                                                           │
│   ├── Full type inference                                               │
│   ├── Generic types                                                     │
│   ├── Algebraic data types                                              │
│   └── Effect tracking (SEX)                                             │
│                                                                         │
│   Meta-Programming                                                      │
│   ├── 20+ built-in macros                                               │
│   ├── Quote/eval for AST manipulation                                   │
│   ├── Reflection for introspection                                      │
│   └── Idiom brackets (applicative)                                      │
│                                                                         │
│   Multi-Target Compilation                                              │
│   ├── Rust (structs, traits, impls)                                     │
│   ├── TypeScript (interfaces, types)                                    │
│   ├── JSON Schema (validation)                                          │
│   └── WebAssembly (portable execution)                                  │
│                                                                         │
│   Tooling                                                               │
│   ├── Parser with error recovery                                        │
│   ├── Type checker                                                      │
│   ├── MCP server (AI integration)                                       │
│   ├── Migration tools                                                   │
│   └── REPL for exploration                                              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Example Use Cases

1. **Univrs.io Ecosystem Ontology**
   - Define network, orchestration, economics types
   - Generate Rust implementations
   - Track specification evolution

2. **API Contract Definition**
   - Define data models in DOL
   - Generate TypeScript types for frontend
   - Generate JSON Schema for validation
   - Generate Rust types for backend

3. **WASM Plugins**
   - Write portable business logic
   - Compile to WASM
   - Run anywhere (browser, edge, server)

4. **Domain-Driven Design**
   - Model ubiquitous language in DOL
   - Mandatory docs enforce shared understanding
   - Evolution tracks domain changes

---

## Appendix A: Quick Reference

### Declaration Syntax

```dol
// Type definition
gen domain.entity {
  entity has property
  entity is state
  entity derives from source
}
docs { Documentation here. }

// Behavior interface
trait domain.behavior {
  uses domain.entity
  entity can action
  each event emits notification
}
docs { Documentation here. }

// Invariant
rule domain.invariant {
  property never exceeds limit
  state matches expected
}
docs { Documentation here. }

// System composition
system domain.application @ 1.0.0 {
  requires dependency >= 0.1.0
  all operations is authenticated
}
docs { Documentation here. }

// Version evolution
evo domain.entity @ 1.1.0 > 1.0.0 {
  adds entity has new_property
  deprecates entity has old_property
  because "Improved API design"
}
docs { Documentation here. }
```

### Expression Syntax (DOL 2.0)

```dol
// Literals
42, 3.14, true, "string"

// Operators
a + b, x |> f, data >> transform

// Functions
fun add(a: i64, b: i64) -> i64 { a + b }

// Lambdas
|x| x * 2
|x: i64, y: i64| -> i64 { x + y }

// Control flow
if condition { then } else { else }
match value { pattern => result, _ => default }
for item in list { process(item) }

// Meta-programming
'expr          // Quote
!quoted        // Eval
?Type          // Reflect
#macro(args)   // Macro call
```

---

## Appendix B: File Extensions

| Extension | Purpose |
|-----------|---------|
| `.dol` | DOL source file |
| `.sex.dol` | Side-effecting DOL |
| `.dol.test` | DOL test file |
| `spirit.dol` | WASM package manifest |
| `system.dol` | Multi-spirit system manifest |

---

## Appendix C: Error Codes

| Code | Category | Description |
|------|----------|-------------|
| E001 | Syntax | Unexpected token |
| E002 | Syntax | Missing documentation |
| E003 | Semantic | Undefined reference |
| E004 | Semantic | Duplicate definition |
| E005 | Type | Type mismatch |
| E006 | Type | Cannot infer type |
| E007 | Effect | Pure context calling SEX |
| W001 | Deprecation | Using deprecated keyword |
| W002 | Style | Naming convention violation |
