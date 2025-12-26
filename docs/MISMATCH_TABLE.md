# DOL vs Rust AST Mismatch Table

> **Status:** AUDIT IN PROGRESS  
> **Last Updated:** December 2025  
> **Purpose:** Track every difference between DOL source expectations and Rust AST implementation

---

## Audit Method

```bash
# What DOL sources expect (enum variants, field names)
grep -hE "Expr\.[A-Z][a-z]+|Stmt\.[A-Z][a-z]+|Decl\.[A-Z][a-z]+" dol/*.dol | sort -u

# What Rust provides
grep -E "^\s+[A-Z][a-zA-Z]+(\s*{|\s*\()" src/ast.rs | sort -u
```

---

## 1. Enum Variant Name Mismatches

| DOL Expects | Rust Has | File(s) Using | Action | Status |
|-------------|----------|---------------|--------|--------|
| `BinaryOp` | `BinOp` | codegen.dol, parser.dol | Rename | ⬜ TODO |
| `UnaryOp` | `UnOp` | codegen.dol | Rename | ⬜ TODO |
| `Statement` | `Stmt` | codegen.dol, parser.dol | Rename | ⬜ TODO |
| `Literal` | `Lit` | codegen.dol | Rename | ⬜ TODO |
| `Identifier` | `Ident` | parser.dol | Rename | ⬜ TODO |
| `FieldAccess` | `Field` | codegen.dol | Rename | ⬜ TODO |
| `MethodCall` | `Call` (combined) | codegen.dol | Add variant | ⬜ TODO |

### Commands to Find More
```bash
# Extract DOL variant references
grep -oh "Expr\.[A-Z][a-zA-Z]*" dol/*.dol | sort | uniq -c | sort -rn

# Extract Rust variants
grep -E "^\s+[A-Z][a-zA-Z]+\s*[({]" src/ast.rs
```

---

## 2. Struct vs Tuple Variant Mismatches

DOL sources use struct syntax: `Expr.Binary { left, op, right }`
Rust uses tuple syntax: `Expr::Binary(left, op, right)`

| Variant | DOL Fields | Rust Structure | Action | Status |
|---------|------------|----------------|--------|--------|
| `Expr::Binary` | `{ left, op, right, span }` | `(Box<Expr>, BinOp, Box<Expr>)` | Convert to struct | ⬜ TODO |
| `Expr::Unary` | `{ op, operand, span }` | `(UnOp, Box<Expr>)` | Convert to struct | ⬜ TODO |
| `Expr::Call` | `{ callee, args, span }` | `(Box<Expr>, Vec<Expr>)` | Convert to struct | ⬜ TODO |
| `Expr::If` | `{ condition, then_branch, else_branch, span }` | `(Box<Expr>, Box<Expr>, Option<Box<Expr>>)` | Convert to struct | ⬜ TODO |
| `Expr::Match` | `{ scrutinee, arms, span }` | `(Box<Expr>, Vec<MatchArm>)` | Convert to struct | ⬜ TODO |
| `Expr::Lambda` | `{ params, body, span }` | `(Vec<Param>, Box<Expr>)` | Convert to struct | ⬜ TODO |
| `Expr::Block` | `{ statements, expr, span }` | `(Vec<Stmt>, Option<Box<Expr>>)` | Convert to struct | ⬜ TODO |
| `Expr::Index` | `{ object, index, span }` | `(Box<Expr>, Box<Expr>)` | Convert to struct | ⬜ TODO |
| `Expr::Field` | `{ object, field, span }` | `(Box<Expr>, String)` | Convert to struct | ⬜ TODO |

### Template for Conversion

```rust
// BEFORE (tuple variant)
pub enum Expr {
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

// AFTER (struct variant)
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
}

// BEFORE (match usage)
Expr::Binary(left, op, right) => { ... }

// AFTER (match usage)
Expr::Binary { left, op, right, .. } => { ... }
```

---

## 3. Missing Types

Types referenced in DOL sources but not defined in Rust AST:

| Type | Used In | Purpose | Definition Needed | Status |
|------|---------|---------|-------------------|--------|
| `UseDecl` | parser.dol, ast.dol | Import declaration | Yes | ⬜ TODO |
| `UseItems` | parser.dol | Import specifier | Yes | ⬜ TODO |
| `HasField` | parser.dol, codegen.dol | Gene field | Yes | ⬜ TODO |
| `IsMethod` | parser.dol | Trait method | Yes | ⬜ TODO |
| `LawDecl` | parser.dol | Law declaration | Yes | ⬜ TODO |
| `ExegesisBlock` | parser.dol | Documentation | Yes | ⬜ TODO |
| `MatchArm` | parser.dol, codegen.dol | Match case | Check if exists | ⬜ TODO |
| `CallArg` | codegen.dol | Function arg | Check if exists | ⬜ TODO |
| `ModuleDecl` | parser.dol | Module header | Verify complete | ⬜ TODO |

### Template for New Type

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub items: UseItems,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseItems {
    All,                    // use foo.*
    Single,                 // use foo
    Named(Vec<String>),     // use foo.{A, B}
}
```

---

## 4. Field Name Mismatches

| Type | DOL Field | Rust Field | Action | Status |
|------|-----------|------------|--------|--------|
| `GeneDecl` | `body` | `fields` | Rename | ⬜ TODO |
| `GeneDecl` | `constraints` | (missing?) | Add | ⬜ TODO |
| `FunctionDecl` | `body` | `block` | Rename | ⬜ TODO |
| `TraitDecl` | `methods` | `items` | Rename | ⬜ TODO |

---

## 5. Type Mapping Differences

| DOL Type | Current Rust Mapping | Correct Rust Mapping | Status |
|----------|---------------------|---------------------|--------|
| `Int64` | `i64` | `i64` | ✅ OK |
| `UInt64` | `u64` | `u64` | ✅ OK |
| `String` | `String` | `String` | ✅ OK |
| `Bool` | `bool` | `bool` | ✅ OK |
| `List<T>` | `Vec<T>` | `Vec<T>` | ✅ OK |
| `Map<K,V>` | `HashMap<K,V>` | `HashMap<K,V>` | ✅ OK |
| `Option<T>` | `Option<T>` | `Option<T>` | ✅ OK |
| `Self` | `Self` | `Self` | ✅ OK |

---

## 6. Pattern Syntax Differences

| DOL Pattern | Rust Equivalent | Used In | Status |
|-------------|-----------------|---------|--------|
| `Type.Variant` | `Type::Variant` | codegen.dol | ✅ Handled |
| `Type.Variant { field }` | `Type::Variant { field, .. }` | codegen.dol | ⬜ TODO |
| `_` (wildcard) | `_` | parser.dol | ✅ OK |

---

## 7. Method/Function Style Differences

| DOL Style | Rust Equivalent | Example |
|-----------|-----------------|---------|
| `obj.method(args)` | `obj.method(args)` or `Type::method(&obj, args)` | Field access |
| `Type.new()` | `Type::new()` | Constructor |
| `this.field` | `self.field` | Self reference |

---

## Resolution Priority

1. **High Priority** (Blocks stage2 compilation)
   - Missing types (UseDecl, HasField, etc.)
   - Struct vs tuple variants
   - Enum variant names

2. **Medium Priority** (Causes warnings/partial failures)
   - Field name mismatches
   - Missing span fields

3. **Low Priority** (Cleanup)
   - Consistent naming conventions
   - Documentation alignment

---

## Progress Tracking

| Phase | Items | Completed | Remaining |
|-------|-------|-----------|-----------|
| Name Mismatches | ~7 | 0 | 7 |
| Struct Variants | ~10 | 0 | 10 |
| Missing Types | ~8 | 0 | 8 |
| Field Names | ~4 | 0 | 4 |
| **Total** | **~29** | **0** | **29** |

---

## Notes

- Each change requires updating: `src/ast.rs`, `src/parser.rs`, `src/codegen/*.rs`, `tests/*.rs`
- Run `cargo test` after each batch of changes
- Commit working states frequently
- Document any DOL source changes needed (should be minimal)
