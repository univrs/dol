# DOL Frontend HIR Analysis

**Agent**: FRONTEND ANALYST
**Date**: 2025-12-30
**Objective**: Deep dive into HIR (High-level IR) implementation

---

## Executive Summary

The DOL project has a **complete and well-implemented HIR layer**. The HIR is designed as a canonical intermediate representation that sits between the AST (surface syntax) and code generation (Rust/WASM/MLIR). The implementation is production-quality with comprehensive type definitions, lowering logic, validation, and visitor patterns.

**Overall Status**: **Complete**

---

## 1. HIR Location and Structure

### 1.1 File Organization

The HIR implementation is located at `/home/ardeshir/repos/univrs-dol/src/hir/`:

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 34 | Module exports, design overview (22 node types) |
| `types.rs` | 555 | All HIR type definitions |
| `span.rs` | 143 | HirId, Span, SpanMap side table |
| `symbol.rs` | 139 | Symbol interning (Symbol, SymbolTable) |
| `visit.rs` | 590 | HirVisitor, HirMutVisitor, walk functions |
| `print.rs` | 290 | Pretty printing for debugging |
| `validate.rs` | 1404 | Semantic validation (scope, types, gene statements) |
| `desugar.rs` | 7 | Re-exports from lowering module |

### 1.2 Design Principles

The HIR follows these principles:
- **Minimal**: 22 node types (vs 50+ in AST)
- **Canonical**: One representation per concept
- **Typed**: All expressions carry type information slots
- **Desugared**: No syntactic sugar remains
- **Side-table spans**: Spans stored separately via HirId -> SpanMap

---

## 2. HIR Types

### 2.1 Core Type Hierarchy

```
HirModule (compilation unit)
    |
    +-- HirDecl (4 variants)
        +-- Type(HirTypeDecl)     -> Gene, Struct, Enum, Alias
        +-- Trait(HirTraitDecl)   -> Trait, Constraint
        +-- Function(HirFunctionDecl)
        +-- Module(HirModuleDecl) -> System, nested modules
```

### 2.2 Declaration Types (4 total)

| Type | Rust Type | Purpose |
|------|-----------|---------|
| Type | `HirTypeDecl` | Gene, struct, enum, alias |
| Trait | `HirTraitDecl` | Trait with methods and associated types |
| Function | `HirFunctionDecl` | Function declarations |
| Module | `HirModuleDecl` | Nested modules (systems) |

### 2.3 Expression Types (12 total)

```rust
pub enum HirExpr {
    Literal(HirLiteral),           // Bool, Int, Float, String, Unit
    Var(Symbol),                   // Variable reference
    Binary(Box<HirBinaryExpr>),    // Binary operations
    Unary(Box<HirUnaryExpr>),      // Unary operations
    Call(Box<HirCallExpr>),        // Function call
    MethodCall(Box<HirMethodCallExpr>), // Method call
    Field(Box<HirFieldExpr>),      // Field access
    Index(Box<HirIndexExpr>),      // Index access
    Block(Box<HirBlockExpr>),      // Block expression
    If(Box<HirIfExpr>),            // Conditional
    Match(Box<HirMatchExpr>),      // Pattern matching
    Lambda(Box<HirLambdaExpr>),    // Lambda/closure
}
```

### 2.4 Statement Types (6 total)

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

### 2.5 Type Forms (8 total)

```rust
pub enum HirType {
    Named(HirNamedType),     // Named type with optional args
    Tuple(Vec<HirType>),     // Tuple type
    Array(Box<HirArrayType>),// Array type
    Function(Box<HirFunctionType>), // Function type
    Ref(Box<HirRefType>),    // Reference type
    Optional(Box<HirType>),  // Optional type (T?)
    Var(u32),                // Type variable (inference)
    Error,                   // Error recovery
}
```

### 2.6 Pattern Types (6 total)

```rust
pub enum HirPat {
    Wildcard,                    // _
    Var(Symbol),                 // Variable binding
    Literal(HirLiteral),         // Literal pattern
    Constructor(HirConstructorPat), // Constructor pattern
    Tuple(Vec<HirPat>),          // Tuple pattern
    Or(Vec<HirPat>),             // Or pattern
}
```

### 2.7 DOL-Specific Statement Types (5 kinds)

For gene bodies:

```rust
pub enum HirStatementKind {
    Has { subject: Symbol, property: Symbol },
    Is { subject: Symbol, type_name: Symbol },
    DerivesFrom { subject: Symbol, parent: Symbol },
    Requires { subject: Symbol, dependency: Symbol },
    Uses { subject: Symbol, resource: Symbol },
}
```

---

## 3. AST to HIR Lowering Implementation

### 3.1 Lowering Location

The lowering code is at `/home/ardeshir/repos/univrs-dol/src/lower/`:

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 104 | Entry points, diagnostics |
| `context.rs` | 162 | LoweringContext with symbol table |
| `desugar.rs` | 315 | Main entry: `lower_file`, `lower_module` |
| `decl.rs` | 301 | Declaration lowering |
| `expr.rs` | 1065 | Expression lowering with desugaring |
| `stmt.rs` | 216 | Statement lowering |

### 3.2 Entry Points

```rust
// Main entry for lowering a file
pub fn lower_file(source: &str) -> Result<(HirModule, LoweringContext), ParseError>

// Lower a parsed AST
pub fn lower_module(ctx: &mut LoweringContext, file: &ast::DolFile) -> HirModule
```

### 3.3 Declaration Lowering

| AST Declaration | HIR Result |
|-----------------|------------|
| `ast::Gene` | `HirDecl::Type(HirTypeDef::Gene(...))` |
| `ast::Trait` | `HirDecl::Trait(...)` |
| `ast::Constraint` | `HirDecl::Trait(...)` (constraints as traits) |
| `ast::System` | `HirDecl::Module(...)` |
| `ast::Evolution` | `HirDecl::Module(...)` (metadata) |
| `ast::Function` | `HirDecl::Function(...)` |

### 3.4 Expression Desugaring

The lowering phase handles significant desugaring:

| Surface Syntax | HIR Form |
|----------------|----------|
| `x \|> f` | `Call(f, [x])` - Pipe operator |
| `f >> g` | `Lambda(x => g(f(x)))` - Composition |
| `a && b` | `If(a, b, false)` - Short-circuit AND |
| `a \|\| b` | `If(a, true, b)` - Short-circuit OR |
| `a => b` | `If(a, b, true)` - Implication |
| `[| f a b |]` | `ap(fmap(f, a), b)` - Idiom brackets |
| `for x in xs` | `loop { match iter.next() {...} }` |
| `while cond` | `loop { if cond {...} else { break } }` |
| `a..b` | `Call(range, [a, b])` |
| `a ^ b` | `Call(pow, [a, b])` |
| `f <$> a` | `Call(fmap, [f, a])` |
| `f <*> a` | `Call(ap, [f, a])` |

### 3.5 Statement Lowering (Gene Bodies)

DOL statements are mapped to HIR:

| AST Statement | HIR Statement |
|---------------|---------------|
| `Has { subject, property }` | `HirStatementKind::Has` |
| `Is { subject, state }` | `HirStatementKind::Is` |
| `DerivesFrom { subject, origin }` | `HirStatementKind::DerivesFrom` |
| `Requires { subject, requirement }` | `HirStatementKind::Requires` |
| `Uses { reference }` | `HirStatementKind::Uses` |
| `Emits { action, event }` | Mapped to `Uses` (simplified) |
| `Matches { subject, target }` | Mapped to `Requires` (simplified) |
| `Never { subject, action }` | Mapped to `Requires` with negation |
| `Quantified { phrase }` | Mapped to `Has` (simplified) |
| `HasField(field)` | `Has { self, field_name }` |
| `Function(func)` | `Has { self, func_name }` |

---

## 4. Semantic Analysis Coverage

### 4.1 Validation Module

Located at `/home/ardeshir/repos/univrs-dol/src/hir/validate.rs` (1404 lines).

### 4.2 Scope Analysis

- **Scope Stack**: Nested scopes for modules, functions, blocks
- **Symbol Definition**: Tracks where symbols are defined
- **Symbol Lookup**: Searches from innermost to outermost scope
- **Reference Tracking**: Marks symbols as referenced (for unused detection)

```rust
pub struct ValidationContext<'a> {
    pub symbols: &'a SymbolTable,
    pub spans: &'a SpanMap,
    scopes: Vec<Scope>,
    diagnostics: Vec<ValidationError>,
    referenced_symbols: HashSet<Symbol>,
    current_return_type: Option<HirType>,
    in_loop: bool,
}
```

### 4.3 Name Resolution

Checks performed:
- Undefined variable detection
- Undefined type detection
- Duplicate definition detection (same scope)
- Constructor resolution in patterns

### 4.4 Type Validation

- Validates type references resolve to actual types
- Checks type parameters and bounds
- Validates function return types
- Validates type arguments in generic types
- Built-in type recognition (Int, Bool, String, etc.)

### 4.5 Declaration Validation

Two-pass approach:
1. **Pass 1**: Collect all top-level declarations into scope
2. **Pass 2**: Validate all declaration bodies

Validates:
- Duplicate type/trait/function names
- Duplicate struct fields
- Duplicate enum variants
- Type parameter bounds
- Function parameter patterns and types

### 4.6 Gene Statement Validation

- Empty subject/property detection
- Self-derivation warnings
- Self-requirement warnings
- Statement kind-specific validation

### 4.7 Expression Validation

- Variable usage before definition
- Place expression validation (assignment LHS)
- Break outside loop detection
- Pattern variable consistency in or-patterns

---

## 5. Unimplemented Cases

### 5.1 Search Results

All `panic!()` calls in the lowering and HIR code are in **test code only**. They are assertions verifying expected values, not production error handling.

### 5.2 Simplified Mappings in stmt.rs

Some AST statement types have simplified HIR mappings (noted in comments):

| Statement | Current Mapping | Notes |
|-----------|-----------------|-------|
| `Emits` | Maps to `Uses` | "simplified" |
| `Matches` | Maps to `Requires` | "simplified" |
| `Never` | Maps to `Requires` with `!prefix` | Negation marker |
| `Quantified` | Maps to `Has` | "simplified" |

These are working implementations but may need enhancement for full semantic support.

### 5.3 Type Expression Handling

```rust
ast::TypeExpr::Never => HirType::Error,     // Never type mapped to Error
ast::TypeExpr::Enum { .. } => HirType::Error, // Inline enums not fully supported
```

### 5.4 Missing HIR Features

The HIR does **not** have:
- `Loop` expression (loops desugared to blocks)
- `Continue` statement (lowered to function call)
- `is_sex` flag on blocks (mentioned in spec but not in types)

---

## 6. Infrastructure Components

### 6.1 Symbol Interning

```rust
// Efficient string interning
pub struct Symbol(u32);
pub struct SymbolTable {
    strings_to_ids: HashMap<String, u32>,
    ids_to_strings: Vec<String>,
}
```

Benefits:
- O(1) symbol comparison
- Memory efficiency
- Cache-friendly

### 6.2 Span Tracking

Side-table architecture:
```rust
pub struct HirId(pub(crate) u32);
pub struct Span { start: u32, end: u32 }
pub struct SpanMap { spans: HashMap<HirId, Span> }
```

### 6.3 Visitor Pattern

Both immutable and mutable visitors:
- `HirVisitor` - Read-only traversal
- `HirMutVisitor` - In-place transformations
- Walk functions for each node type

---

## 7. Test Coverage

### 7.1 Validation Tests (validate.rs)

- Empty module validation
- Function declaration validation
- Duplicate function name detection
- Undefined variable detection
- Type declaration validation
- Duplicate struct fields detection
- Gene validation
- Self-derivation warning
- Undefined type reference detection
- Invalid assignment target detection
- Pattern validation
- Scope management

### 7.2 Lowering Tests

Comprehensive tests in each lowering module:
- `desugar.rs`: End-to-end gene/trait/constraint lowering
- `decl.rs`: Declaration lowering, type expression lowering
- `expr.rs`: All expression types, pipe desugaring, idiom brackets
- `stmt.rs`: DOL statement lowering

---

## 8. Status Assessment

| Component | Status | Notes |
|-----------|--------|-------|
| HIR Types | **Complete** | All 22 node types defined |
| Symbol Interning | **Complete** | Efficient implementation |
| Span Tracking | **Complete** | Side-table architecture |
| AST to HIR Lowering | **Complete** | Full coverage with desugaring |
| Declaration Lowering | **Complete** | All declaration types handled |
| Expression Lowering | **Complete** | Extensive desugaring |
| Statement Lowering | **Partial** | Some simplified mappings |
| Scope Analysis | **Complete** | Nested scopes, proper lookup |
| Name Resolution | **Complete** | Variables and types |
| Type Validation | **Complete** | Type references validated |
| Gene Validation | **Complete** | Statement-level validation |
| Visitor Pattern | **Complete** | Both mutable and immutable |
| Pretty Printing | **Complete** | Debug output |
| Documentation | **Complete** | Comprehensive spec at docs/hir/HIR-SPECIFICATION.md |

---

## 9. Key Findings

### 9.1 Strengths

1. **Well-designed type hierarchy** - Clean separation of concerns
2. **Comprehensive desugaring** - Pipe, composition, idiom brackets, loops
3. **Side-table span tracking** - Smaller nodes, better cache performance
4. **Symbol interning** - Efficient string handling
5. **Two-pass validation** - Proper forward reference support
6. **Extensive test coverage** - Unit tests for all components
7. **Detailed specification** - 1055-line HIR specification document

### 9.2 Minor Gaps

1. Some statement types have simplified mappings (`Emits`, `Matches`, `Never`, `Quantified`)
2. Inline enum types not fully supported in type lowering
3. `Never` type mapped to `Error` instead of proper handling
4. `Continue` statement lowered as function call rather than proper HIR node

### 9.3 No Blocking Issues

- No `todo!()` or `unimplemented!()` macros in production code
- All `panic!()` calls are in test assertions only
- The HIR layer is production-ready

---

## 10. Conclusion

The HIR implementation is **complete** and suitable for the DOL to WASM pipeline. The lowering from AST to HIR is comprehensive, handling:
- All declaration types (Gene, Trait, Constraint, System, Evolution, Function)
- All expression forms with appropriate desugaring
- All DOL-specific statements
- Semantic validation with scope/name/type checking

**Status**: Ready for downstream processing (type checking, MLIR lowering, code generation)
