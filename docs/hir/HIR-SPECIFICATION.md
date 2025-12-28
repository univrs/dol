# DOL HIR Specification

**Version**: 0.4.0
**Status**: Implementation Reference
**Last Updated**: 2024

---

## Table of Contents

1. [Purpose and Goals](#1-purpose-and-goals)
2. [Design Principles](#2-design-principles)
3. [Type Hierarchy](#3-type-hierarchy)
4. [Node Type Reference](#4-node-type-reference)
5. [Desugaring Rules (AST to HIR)](#5-desugaring-rules-ast-to-hir)
6. [Codegen Mapping (HIR to Rust)](#6-codegen-mapping-hir-to-rust)
7. [Example Transformations](#7-example-transformations)
8. [Span Tracking and Error Reporting](#8-span-tracking-and-error-reporting)
9. [Symbol Interning](#9-symbol-interning)
10. [Visitor Pattern](#10-visitor-pattern)

---

## 1. Purpose and Goals

The High-level Intermediate Representation (HIR) serves as the canonical representation for DOL programs after parsing. It acts as a bridge between the surface syntax (AST) and code generation (Rust/WASM).

### Primary Goals

1. **Canonicalization**: Multiple surface syntaxes map to the same HIR form
2. **Desugaring**: All syntactic sugar is removed before HIR
3. **Type Preparation**: HIR is structured to facilitate type inference and checking
4. **Minimal Node Types**: Reduce 50+ AST variants to 22 canonical HIR forms
5. **Efficient Traversal**: Support visitor pattern for analysis and transformation

### HIR in the Compilation Pipeline

```
DOL Source (.dol)
       |
       v
   [Lexer]  --> Token Stream
       |
       v
   [Parser] --> AST (Abstract Syntax Tree)
       |
       v
   [Lowering] --> HIR (High-level IR)
       |
       v
   [Type Checking] --> Typed HIR
       |
       v
   [Codegen] --> Rust/WASM/MLIR
```

---

## 2. Design Principles

### 2.1 Minimal Representation

HIR uses exactly **22 core node types** organized into 6 categories:

| Category | Count | Purpose |
|----------|-------|---------|
| Module | 1 | Top-level compilation unit |
| Declarations | 4 | Type, Trait, Function, Module |
| Expressions | 12 | All computational forms |
| Statements | 6 | Control flow and bindings |
| Types | 8 | Type annotations |
| Patterns | 6 | Pattern matching |

### 2.2 Canonical Forms

Every concept has exactly one HIR representation:

- **Gene** -> `HirDecl::Type` with `HirTypeDef::Gene`
- **Trait** -> `HirDecl::Trait`
- **Constraint** -> `HirDecl::Trait` (constraints are traits with invariants)
- **System** -> `HirDecl::Module` containing functions and state

### 2.3 Side Table Architecture

Unlike AST which embeds spans directly in nodes, HIR uses a **side table** design:

```
HirNode ----[HirId]----> SpanMap -----> Span
```

Benefits:
- Smaller node sizes
- Cache-friendly traversal
- Optional span information
- Easy serialization

---

## 3. Type Hierarchy

### 3.1 ASCII Type Diagram

```
                            HirModule
                                |
                                v
                 +-----------------------------+
                 |         HirDecl (4)         |
                 +-----------------------------+
                 |  Type  | Trait | Function | Module
                 +----+---+---+---+-----+-----+---+---+
                      |       |         |         |
              HirTypeDecl  HirTraitDecl |    HirModuleDecl
                   |           |        |         |
                   v           v        v         v
              HirTypeDef   HirTraitItem  ...    (nested decls)
                   |           |
       +-----------+-----------+
       |     |     |     |
     Alias Struct Enum Gene
                         |
                         v
                    HirStatement
                         |
          +--------------+--------------+
          |      |       |      |       |
         Has    Is   DerivesFrom Requires Uses


                            HirExpr (12)
                                |
    +-------+-------+-------+---+---+-------+-------+
    |       |       |       |       |       |       |
  Literal  Var   Binary  Unary   Call  MethodCall  ...
                    |       |       |
               HirBinaryExpr |  HirCallExpr
                             |
                       HirUnaryExpr


                            HirStmt (6)
                                |
          +--------+--------+---+---+--------+--------+
          |        |        |       |        |        |
         Val      Var    Assign   Expr    Return   Break


                            HirType (8)
                                |
    +-------+-------+-------+---+---+-------+-------+---+
    |       |       |       |       |       |       |   |
  Named   Tuple  Array  Function  Ref  Optional  Var Error


                            HirPat (6)
                                |
          +--------+--------+---+---+--------+--------+
          |        |        |       |        |        |
       Wildcard   Var   Literal Constructor Tuple    Or
```

### 3.2 Infrastructure Types

```
HirId (u32)          -- Unique node identifier
Symbol (u32)         -- Interned string reference
Span { start, end }  -- Byte offsets in source
SpanMap              -- HirId -> Span mapping
SymbolTable          -- String interning table
```

---

## 4. Node Type Reference

### 4.1 Module Level

#### `HirModule`
Top-level compilation unit containing all declarations from a DOL file.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `HirId` | Unique identifier |
| `name` | `Symbol` | Module name (from `module` decl or filename) |
| `decls` | `Vec<HirDecl>` | Top-level declarations |

### 4.2 Declarations (4 types)

#### `HirDecl`
Enumeration of all declaration forms:

```rust
pub enum HirDecl {
    Type(HirTypeDecl),      // gene, struct, enum
    Trait(HirTraitDecl),    // trait, constraint
    Function(HirFunctionDecl),
    Module(HirModuleDecl),  // system, nested module
}
```

#### `HirTypeDecl`
Type/gene declaration.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `HirId` | Unique identifier |
| `name` | `Symbol` | Type name |
| `type_params` | `Vec<HirTypeParam>` | Generic parameters |
| `body` | `HirTypeDef` | Definition body |

#### `HirTypeDef`
Body of a type declaration:

```rust
pub enum HirTypeDef {
    Alias(HirType),           // type Foo = Bar
    Struct(Vec<HirField>),    // struct with fields
    Enum(Vec<HirVariant>),    // enum with variants
    Gene(Vec<HirStatement>),  // DOL gene body
}
```

#### `HirTraitDecl`
Trait or constraint declaration.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `HirId` | Unique identifier |
| `name` | `Symbol` | Trait name |
| `type_params` | `Vec<HirTypeParam>` | Generic parameters |
| `bounds` | `Vec<HirType>` | Super traits |
| `items` | `Vec<HirTraitItem>` | Methods and associated types |

#### `HirTraitItem`
Item within a trait:

```rust
pub enum HirTraitItem {
    Method(HirFunctionDecl),
    AssocType(HirAssocType),
}
```

#### `HirFunctionDecl`
Function declaration.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `HirId` | Unique identifier |
| `name` | `Symbol` | Function name |
| `type_params` | `Vec<HirTypeParam>` | Generic parameters |
| `params` | `Vec<HirParam>` | Function parameters |
| `return_type` | `HirType` | Return type |
| `body` | `Option<HirExpr>` | Body (None for abstract) |

#### `HirModuleDecl`
Nested module (or system) declaration.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `HirId` | Unique identifier |
| `name` | `Symbol` | Module name |
| `decls` | `Vec<HirDecl>` | Contained declarations |

### 4.3 DOL Statements (5 kinds)

These statements appear in gene bodies and represent ontological predicates.

#### `HirStatement`
Wrapper with ID for span tracking:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `HirId` | Unique identifier |
| `kind` | `HirStatementKind` | Statement variant |

#### `HirStatementKind`
DOL-specific predicate statements:

```rust
pub enum HirStatementKind {
    Has { subject: Symbol, property: Symbol },
    Is { subject: Symbol, type_name: Symbol },
    DerivesFrom { subject: Symbol, parent: Symbol },
    Requires { subject: Symbol, dependency: Symbol },
    Uses { subject: Symbol, resource: Symbol },
}
```

| Kind | DOL Syntax | Example |
|------|------------|---------|
| `Has` | `subject has property` | `container has identity` |
| `Is` | `subject is state` | `container is created` |
| `DerivesFrom` | `subject derives from parent` | `id derives from keypair` |
| `Requires` | `subject requires dependency` | `auth requires certificate` |
| `Uses` | `uses reference` | `uses container.exists` |

### 4.4 Expressions (12 types)

#### `HirExpr`
All expression forms:

```rust
pub enum HirExpr {
    Literal(HirLiteral),
    Var(Symbol),
    Binary(Box<HirBinaryExpr>),
    Unary(Box<HirUnaryExpr>),
    Call(Box<HirCallExpr>),
    MethodCall(Box<HirMethodCallExpr>),
    Field(Box<HirFieldExpr>),
    Index(Box<HirIndexExpr>),
    Block(Box<HirBlockExpr>),
    If(Box<HirIfExpr>),
    Match(Box<HirMatchExpr>),
    Lambda(Box<HirLambdaExpr>),
}
```

#### Expression Sub-types

| Type | Fields | Description |
|------|--------|-------------|
| `HirLiteral` | `Bool`, `Int`, `Float`, `String`, `Unit` | Literal values |
| `HirBinaryExpr` | `left`, `op`, `right` | Binary operations |
| `HirUnaryExpr` | `op`, `operand` | Unary operations |
| `HirCallExpr` | `func`, `args` | Function calls |
| `HirMethodCallExpr` | `receiver`, `method`, `args` | Method calls |
| `HirFieldExpr` | `base`, `field` | Field access |
| `HirIndexExpr` | `base`, `index` | Array/map indexing |
| `HirBlockExpr` | `stmts`, `expr` | Block with optional result |
| `HirIfExpr` | `cond`, `then_branch`, `else_branch` | Conditional |
| `HirMatchExpr` | `scrutinee`, `arms` | Pattern matching |
| `HirLambdaExpr` | `params`, `return_type`, `body` | Closures |

#### `HirBinaryOp` (13 operators)

| Operator | Symbol | Category |
|----------|--------|----------|
| `Add` | `+` | Arithmetic |
| `Sub` | `-` | Arithmetic |
| `Mul` | `*` | Arithmetic |
| `Div` | `/` | Arithmetic |
| `Mod` | `%` | Arithmetic |
| `Eq` | `==` | Comparison |
| `Ne` | `!=` | Comparison |
| `Lt` | `<` | Comparison |
| `Le` | `<=` | Comparison |
| `Gt` | `>` | Comparison |
| `Ge` | `>=` | Comparison |
| `And` | `&&` | Logical |
| `Or` | `\|\|` | Logical |

#### `HirUnaryOp` (2 operators)

| Operator | Symbol | Description |
|----------|--------|-------------|
| `Neg` | `-` | Numeric negation |
| `Not` | `!` | Logical negation |

### 4.5 Statements (6 types)

Imperative statements within function/block bodies:

```rust
pub enum HirStmt {
    Val(HirValStmt),      // Immutable binding
    Var(HirVarStmt),      // Mutable binding
    Assign(HirAssignStmt),// Assignment
    Expr(HirExpr),        // Expression statement
    Return(Option<HirExpr>),
    Break(Option<HirExpr>),
}
```

| Type | Fields | Description |
|------|--------|-------------|
| `HirValStmt` | `pat`, `ty?`, `init` | `let x = ...` |
| `HirVarStmt` | `pat`, `ty?`, `init` | `var x = ...` |
| `HirAssignStmt` | `lhs`, `rhs` | `x = ...` |

### 4.6 Types (8 forms)

```rust
pub enum HirType {
    Named(HirNamedType),     // Named type with args
    Tuple(Vec<HirType>),     // (T, U, V)
    Array(Box<HirArrayType>),// [T; N] or [T]
    Function(Box<HirFunctionType>), // (T) -> U
    Ref(Box<HirRefType>),    // &T or &mut T
    Optional(Box<HirType>),  // T?
    Var(u32),                // Type variable
    Error,                   // Error recovery
}
```

| Type | Example | Description |
|------|---------|-------------|
| `Named` | `List<Int32>` | Named type with optional args |
| `Tuple` | `(Int32, String)` | Tuple type |
| `Array` | `[Int32; 10]` | Fixed or dynamic array |
| `Function` | `(Int32) -> Bool` | Function type |
| `Ref` | `&mut String` | Reference type |
| `Optional` | `Int32?` | Optional/nullable |
| `Var` | `?0` | Inference variable |
| `Error` | - | Error placeholder |

### 4.7 Patterns (6 forms)

```rust
pub enum HirPat {
    Wildcard,                    // _
    Var(Symbol),                 // x
    Literal(HirLiteral),         // 42, "hello"
    Constructor(HirConstructorPat), // Some(x)
    Tuple(Vec<HirPat>),          // (a, b, c)
    Or(Vec<HirPat>),             // a | b | c
}
```

| Pattern | Example | Description |
|---------|---------|-------------|
| `Wildcard` | `_` | Match anything, no binding |
| `Var` | `x` | Bind to variable |
| `Literal` | `42` | Match exact value |
| `Constructor` | `Some(x)` | Destructure variant |
| `Tuple` | `(a, b)` | Destructure tuple |
| `Or` | `0 \| 1` | Alternative patterns |

---

## 5. Desugaring Rules (AST to HIR)

### 5.1 Declaration Mappings

| AST Node | HIR Form | Notes |
|----------|----------|-------|
| `ast::Gene` | `HirDecl::Type(HirTypeDef::Gene)` | Statements preserved |
| `ast::Trait` | `HirDecl::Trait` | Methods in `items` |
| `ast::Constraint` | `HirDecl::Trait` | Invariants as method bodies |
| `ast::System` | `HirDecl::Module` | Functions + state |
| `ast::Evolution` | *Metadata only* | Not in HIR |
| `ast::FunctionDecl` | `HirDecl::Function` | Direct mapping |

### 5.2 Statement Mappings (Gene Body)

| AST Statement | HIR StatementKind |
|---------------|-------------------|
| `Statement::Has { subject, property }` | `HirStatementKind::Has { subject, property }` |
| `Statement::Is { subject, state }` | `HirStatementKind::Is { subject, type_name }` |
| `Statement::DerivesFrom { subject, origin }` | `HirStatementKind::DerivesFrom { subject, parent }` |
| `Statement::Requires { subject, requirement }` | `HirStatementKind::Requires { subject, dependency }` |
| `Statement::Uses { reference }` | `HirStatementKind::Uses { subject: self, resource }` |
| `Statement::HasField(field)` | *Lowered to struct field* |
| `Statement::Function(func)` | *Moved to type impl block* |

### 5.3 Expression Mappings

| AST Expression | HIR Expression | Desugaring Notes |
|----------------|----------------|------------------|
| `Expr::Literal` | `HirExpr::Literal` | Direct |
| `Expr::Identifier` | `HirExpr::Var` | Symbol interned |
| `Expr::Binary` | `HirExpr::Binary` | Operator normalized |
| `Expr::Unary` | `HirExpr::Unary` | Direct |
| `Expr::Call` | `HirExpr::Call` | Direct |
| `Expr::Member { object, field }` | `HirExpr::Field` or `HirExpr::MethodCall` | Context-dependent |
| `Expr::If` | `HirExpr::If` | Direct |
| `Expr::Match` | `HirExpr::Match` | Direct |
| `Expr::Block` | `HirExpr::Block` | Statements converted |
| `Expr::Lambda` | `HirExpr::Lambda` | Direct |
| `Expr::Pipe (a \|> f)` | `HirExpr::Call(f, [a])` | Desugared |
| `Expr::Compose (f >> g)` | `HirExpr::Lambda(x => g(f(x)))` | Desugared |
| `Expr::IdiomBracket` | Desugared `map`/`apply` chain | Applicative |
| `Expr::Forall` / `Expr::Exists` | *Logical predicates* | Constraint checking |
| `Expr::Quote` / `Expr::Unquote` | *Macro expansion* | Pre-HIR phase |

### 5.4 Type Mappings

| AST Type | HIR Type |
|----------|----------|
| `TypeExpr::Named` | `HirType::Named` |
| `TypeExpr::Generic` | `HirType::Named` (with args) |
| `TypeExpr::Function` | `HirType::Function` |
| `TypeExpr::Tuple` | `HirType::Tuple` |
| `TypeExpr::Never` | Special handling |

### 5.5 Special Desugaring Rules

#### Pipe Operator
```
// DOL: a |> f |> g
// Desugars to: g(f(a))
HirExpr::Call {
    func: g,
    args: [HirExpr::Call { func: f, args: [a] }]
}
```

#### Idiom Brackets
```
// DOL: [| f a b |]
// Desugars to: a.map(f).apply(b)
HirExpr::MethodCall {
    receiver: HirExpr::MethodCall {
        receiver: a,
        method: "map",
        args: [f]
    },
    method: "apply",
    args: [b]
}
```

#### SEX Blocks
```
// DOL: sex { mutable_code }
// Desugars to block with mutable context flag
HirExpr::Block { stmts, expr, is_sex: true }
```

---

## 6. Codegen Mapping (HIR to Rust)

### 6.1 Type Generation

| HIR Type | Rust Type |
|----------|-----------|
| `Named("Int8")` | `i8` |
| `Named("Int16")` | `i16` |
| `Named("Int32")` | `i32` |
| `Named("Int64")` | `i64` |
| `Named("UInt8")` | `u8` |
| `Named("UInt16")` | `u16` |
| `Named("UInt32")` | `u32` |
| `Named("UInt64")` | `u64` |
| `Named("Float32")` | `f32` |
| `Named("Float64")` | `f64` |
| `Named("Bool")` | `bool` |
| `Named("String")` | `String` |
| `Named("Char")` | `char` |
| `Named("List", [T])` | `Vec<T>` |
| `Named("Map", [K, V])` | `HashMap<K, V>` |
| `Named("Set", [T])` | `HashSet<T>` |
| `Named("Option", [T])` | `Option<T>` |
| `Named("Result", [T, E])` | `Result<T, E>` |
| `Tuple([...])` | `(T1, T2, ...)` |
| `Array(T, Some(n))` | `[T; n]` |
| `Array(T, None)` | `Vec<T>` |
| `Function([P...], R)` | `fn(P...) -> R` |
| `Ref(T, false)` | `&T` |
| `Ref(T, true)` | `&mut T` |
| `Optional(T)` | `Option<T>` |

### 6.2 Declaration Generation

#### Gene to Struct

```rust
// HIR: HirDecl::Type { name: "container.exists", body: Gene([...]) }
// Rust:
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerExists {
    pub identity: Identity,
    pub state: ContainerState,
    // ... fields from HasField statements
}

impl ContainerExists {
    // ... methods from Function statements
}
```

#### Trait Declaration

```rust
// HIR: HirDecl::Trait { name: "lifecycle", bounds: [...], items: [...] }
// Rust:
pub trait Lifecycle: Clone + Debug {
    fn start(&self) -> Result<(), Error>;
    fn stop(&mut self);
    // ... from items
}
```

#### System to Module

```rust
// HIR: HirDecl::Module { name: "orchestrator", decls: [...] }
// Rust:
#[derive(Debug, Clone, PartialEq)]
pub struct Orchestrator {
    // state fields
}

impl Orchestrator {
    pub fn new() -> Self { ... }
    // methods
}
```

### 6.3 Statement Generation

| HIR Statement | Rust Code |
|---------------|-----------|
| `Val { pat, init }` | `let pat = init;` |
| `Var { pat, init }` | `let mut pat = init;` |
| `Assign { lhs, rhs }` | `lhs = rhs;` |
| `Expr(e)` | `e;` |
| `Return(Some(e))` | `return e;` |
| `Return(None)` | `return;` |
| `Break(Some(e))` | `break e;` |
| `Break(None)` | `break;` |

### 6.4 Expression Generation

| HIR Expression | Rust Code |
|----------------|-----------|
| `Literal(Int(42))` | `42` |
| `Literal(Float(3.14))` | `3.14` |
| `Literal(String(s))` | `"s".to_string()` |
| `Literal(Bool(true))` | `true` |
| `Literal(Unit)` | `()` |
| `Var(x)` | `x` |
| `Binary { left, Add, right }` | `(left + right)` |
| `Call { func, args }` | `func(args...)` |
| `MethodCall { recv, method, args }` | `recv.method(args...)` |
| `Field { base, field }` | `base.field` |
| `Index { base, index }` | `base[index]` |
| `If { cond, then, Some(else) }` | `if cond { then } else { else }` |
| `Match { scrutinee, arms }` | `match scrutinee { arms... }` |
| `Lambda { params, body }` | `\|params\| body` |
| `Block { stmts, Some(expr) }` | `{ stmts; expr }` |

---

## 7. Example Transformations

### 7.1 Complete Gene Transformation

**DOL Source:**
```dol
gene container.exists {
    container has identity
    container has state
    container has name: String = "unnamed"

    fun get_name() -> String {
        return self.name
    }
}

exegesis {
    A container is the fundamental unit of isolation.
}
```

**AST:**
```rust
Declaration::Gene(Gene {
    name: "container.exists",
    extends: None,
    statements: vec![
        Statement::Has { subject: "container", property: "identity", .. },
        Statement::Has { subject: "container", property: "state", .. },
        Statement::HasField(HasField {
            name: "name",
            type_: TypeExpr::Named("String"),
            default: Some(Expr::Literal(Literal::String("unnamed"))),
            ..
        }),
        Statement::Function(FunctionDecl { name: "get_name", .. }),
    ],
    exegesis: "A container is the fundamental unit...",
    ..
})
```

**HIR:**
```rust
HirDecl::Type(HirTypeDecl {
    id: HirId(1),
    name: Symbol("container.exists"),
    type_params: vec![],
    body: HirTypeDef::Gene(vec![
        HirStatement {
            id: HirId(2),
            kind: HirStatementKind::Has {
                subject: Symbol("container"),
                property: Symbol("identity"),
            },
        },
        HirStatement {
            id: HirId(3),
            kind: HirStatementKind::Has {
                subject: Symbol("container"),
                property: Symbol("state"),
            },
        },
    ]),
})
// Note: HasField and Function are moved to separate struct definition
```

**Generated Rust:**
```rust
/// A container is the fundamental unit of isolation.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerExists {
    pub identity: Identity,
    pub state: ContainerState,
    pub name: String,
}

impl ContainerExists {
    pub fn new() -> Self {
        Self {
            identity: Default::default(),
            state: Default::default(),
            name: "unnamed".to_string(),
        }
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }
}
```

### 7.2 Trait Transformation

**DOL Source:**
```dol
trait container.lifecycle {
    uses container.exists

    container is created
    container is started
    container is stopped

    fun start() -> Result<(), Error>
    fun stop() -> ()
}
```

**HIR:**
```rust
HirDecl::Trait(HirTraitDecl {
    id: HirId(10),
    name: Symbol("container.lifecycle"),
    type_params: vec![],
    bounds: vec![],  // No super traits
    items: vec![
        HirTraitItem::Method(HirFunctionDecl {
            name: Symbol("start"),
            params: vec![],
            return_type: HirType::Named(HirNamedType {
                name: Symbol("Result"),
                args: vec![/* () */, /* Error */],
            }),
            body: None,  // Abstract method
            ..
        }),
        HirTraitItem::Method(HirFunctionDecl {
            name: Symbol("stop"),
            params: vec![],
            return_type: HirType::Tuple(vec![]),  // ()
            body: None,
            ..
        }),
    ],
})
```

**Generated Rust:**
```rust
/// Container lifecycle management
pub trait ContainerLifecycle {
    fn start(&self) -> Result<(), Error>;
    fn stop(&mut self) -> ();
}
```

### 7.3 Expression Transformation

**DOL Source:**
```dol
fun process(data: List<Int32>) -> Int32 {
    data
        |> filter(|x| x > 0)
        |> map(|x| x * 2)
        |> sum()
}
```

**HIR (after pipe desugaring):**
```rust
HirExpr::Call {
    func: HirExpr::Var(Symbol("sum")),
    args: vec![
        HirExpr::Call {
            func: HirExpr::Var(Symbol("map")),
            args: vec![
                HirExpr::Call {
                    func: HirExpr::Var(Symbol("filter")),
                    args: vec![
                        HirExpr::Var(Symbol("data")),
                        HirExpr::Lambda {
                            params: vec![HirParam { pat: HirPat::Var(Symbol("x")), .. }],
                            body: HirExpr::Binary { /* x > 0 */ },
                            ..
                        },
                    ],
                },
                HirExpr::Lambda {
                    params: vec![HirParam { pat: HirPat::Var(Symbol("x")), .. }],
                    body: HirExpr::Binary { /* x * 2 */ },
                    ..
                },
            ],
        },
    ],
}
```

**Generated Rust:**
```rust
pub fn process(data: Vec<i32>) -> i32 {
    sum(map(filter(data, |x| x > 0), |x| x * 2))
}
// Or with method chaining:
pub fn process(data: Vec<i32>) -> i32 {
    data.iter()
        .filter(|x| *x > 0)
        .map(|x| x * 2)
        .sum()
}
```

---

## 8. Span Tracking and Error Reporting

### 8.1 HirId System

Every HIR node has a unique `HirId`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirId(pub(crate) u32);

impl HirId {
    pub fn new() -> Self {
        // Atomically increment global counter
        let id = NEXT_HIR_ID.fetch_add(1, Ordering::SeqCst);
        Self(id)
    }
}
```

### 8.2 SpanMap Side Table

Spans are stored separately from nodes:

```rust
pub struct SpanMap {
    spans: HashMap<HirId, Span>,
}

impl SpanMap {
    pub fn insert(&mut self, id: HirId, span: Span);
    pub fn get(&self, id: HirId) -> Option<Span>;
    pub fn get_or_dummy(&self, id: HirId) -> Span;
}
```

### 8.3 Error Reporting Flow

```
1. Error detected at HirId(42)
2. Look up span: span_map.get(HirId(42)) -> Span { start: 100, end: 150 }
3. Map byte offsets to line/column (via source map)
4. Generate error message with location
```

Example error:
```
error[E0308]: type mismatch
  --> src/main.dol:15:10
   |
15 |     let x: Int32 = "hello"
   |            ^^^^^ expected `Int32`, found `String`
```

---

## 9. Symbol Interning

### 9.1 Symbol Table

Strings are interned for efficient comparison and storage:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Symbol(u32);

pub struct SymbolTable {
    strings_to_ids: HashMap<String, u32>,
    ids_to_strings: Vec<String>,
}

impl SymbolTable {
    pub fn intern(&mut self, s: &str) -> Symbol;
    pub fn resolve(&self, sym: Symbol) -> Option<&str>;
}
```

### 9.2 Benefits

1. **O(1) comparison**: Compare u32 instead of strings
2. **Memory efficiency**: Each string stored once
3. **Cache friendly**: Small identifiers fit in registers
4. **Serialization**: Easy to serialize symbol IDs

---

## 10. Visitor Pattern

### 10.1 Immutable Visitor

```rust
pub trait HirVisitor: Sized {
    fn visit_module(&mut self, module: &HirModule) { walk_module(self, module); }
    fn visit_decl(&mut self, decl: &HirDecl) { walk_decl(self, decl); }
    fn visit_type_decl(&mut self, decl: &HirTypeDecl) { walk_type_decl(self, decl); }
    fn visit_trait_decl(&mut self, decl: &HirTraitDecl) { walk_trait_decl(self, decl); }
    fn visit_function_decl(&mut self, decl: &HirFunctionDecl);
    fn visit_module_decl(&mut self, decl: &HirModuleDecl);
    fn visit_expr(&mut self, expr: &HirExpr) { walk_expr(self, expr); }
    fn visit_stmt(&mut self, stmt: &HirStmt) { walk_stmt(self, stmt); }
    fn visit_type(&mut self, ty: &HirType) { walk_type(self, ty); }
    fn visit_pat(&mut self, pat: &HirPat) { walk_pat(self, pat); }
}
```

### 10.2 Mutable Visitor

```rust
pub trait HirMutVisitor: Sized {
    fn visit_module_mut(&mut self, module: &mut HirModule);
    fn visit_decl_mut(&mut self, decl: &mut HirDecl);
    fn visit_expr_mut(&mut self, expr: &mut HirExpr);
    fn visit_stmt_mut(&mut self, stmt: &mut HirStmt);
    fn visit_type_mut(&mut self, ty: &mut HirType);
    fn visit_pat_mut(&mut self, pat: &mut HirPat);
}
```

### 10.3 Walk Functions

Each visitor method has a corresponding `walk_*` function that traverses children:

```rust
pub fn walk_module<V: HirVisitor>(visitor: &mut V, module: &HirModule) {
    for decl in &module.decls {
        visitor.visit_decl(decl);
    }
}

pub fn walk_expr<V: HirVisitor>(visitor: &mut V, expr: &HirExpr) {
    match expr {
        HirExpr::Literal(_) | HirExpr::Var(_) => {}
        HirExpr::Binary(bin) => {
            visitor.visit_expr(&bin.left);
            visitor.visit_expr(&bin.right);
        }
        // ... other cases
    }
}
```

### 10.4 Example: Declaration Counter

```rust
struct DeclCounter {
    type_count: usize,
    trait_count: usize,
    function_count: usize,
}

impl HirVisitor for DeclCounter {
    fn visit_decl(&mut self, decl: &HirDecl) {
        match decl {
            HirDecl::Type(_) => self.type_count += 1,
            HirDecl::Trait(_) => self.trait_count += 1,
            HirDecl::Function(_) => self.function_count += 1,
            HirDecl::Module(m) => {
                for d in &m.decls {
                    self.visit_decl(d);
                }
            }
        }
    }
}
```

---

## Appendix A: Node Count Summary

| Category | Types | Fields/Variants |
|----------|-------|-----------------|
| **Module** | 1 | 3 fields |
| **Declarations** | 4 | ~15 fields total |
| **Declaration Helpers** | 8 | ~25 fields |
| **DOL Statements** | 2 | 5 variants |
| **Expressions** | 12 | ~40 fields |
| **Expression Helpers** | 10 | ~30 fields |
| **Statements** | 6 | ~10 fields |
| **Types** | 8 | ~15 fields |
| **Type Helpers** | 4 | ~10 fields |
| **Patterns** | 6 | ~8 fields |
| **Pattern Helpers** | 1 | ~3 fields |
| **Infrastructure** | 3 | (HirId, Symbol, Span) |
| **Total** | ~36 | ~160 fields |

---

## Appendix B: File Organization

```
src/hir/
    mod.rs       -- Module exports, design overview
    types.rs     -- All 36+ HIR types (555 lines)
    span.rs      -- HirId, Span, SpanMap (143 lines)
    symbol.rs    -- Symbol, SymbolTable (138 lines)
    visit.rs     -- HirVisitor, HirMutVisitor (590 lines)
    print.rs     -- HirPrinter, pretty printing (290 lines)
```

---

## Appendix C: Version History

| Version | Changes |
|---------|---------|
| 0.1.0 | Initial HIR design with basic types |
| 0.2.0 | Added DOL-specific statements |
| 0.3.0 | Symbol interning, span side tables |
| 0.4.0 | Complete visitor pattern, documentation |

---

*This specification is the authoritative reference for DOL HIR implementation.*
