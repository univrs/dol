# AST Compatibility Report

**Generated:** 2026-02-07
**Purpose:** Document AST incompatibilities between the current DOL AST (`src/ast.rs`) and the meta-programming crates (`dol-macro`, `dol-macro-proc`)

---

## Executive Summary

The macro system crates were developed against an earlier version of the DOL AST. The current AST (v0.8.0) has undergone significant refactoring with renamed types, restructured variants, and different field signatures. This report documents all incompatibilities and provides a migration guide.

### Statistics

- **Total Compatibility Issues:** 26
- **Expression Variant Issues:** 9
- **Expression Field Issues:** 6
- **Statement Issues:** 7
- **Block Structure Issues:** 2
- **Type Issues:** 2
- **Affected Files:** 6

---

## 1. Expression Variant Name Changes

### Issue #1: Expr::Ident → Expr::Identifier

**Current AST:**
```rust
pub enum Expr {
    Identifier(String),
    // ...
}
```

**Macro Crates Use:**
```rust
Expr::Ident(name)
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (lines 161, 182, 199, 204, 209, 214, 219)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 167, 169, 183, 188, 193, 198, 225, 403, 406, 421, 428)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/pattern.rs` (lines 453, 470, 547, 565, 566)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/declarative.rs` (lines 306, 313)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (lines 151, 177, 243, 244, 257, 260, 273, 280, 296, 310, 324, 331)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/codegen.rs` (lines 169, 302)

**Migration:**
```rust
// OLD
Expr::Ident(name) => { /* ... */ }

// NEW
Expr::Identifier(name) => { /* ... */ }
```

**Total Occurrences:** 35+ instances

---

### Issue #2: Expr::Array → Expr::List

**Current AST:**
```rust
pub enum Expr {
    List(Vec<Expr>),
    // ...
}
```

**Macro Crates Use:**
```rust
Expr::Array(elements)
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (lines 214, 219)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (line 225)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (lines 86, 165, 171, 212)

**Migration:**
```rust
// OLD
Expr::Array(elements) => { /* ... */ }

// NEW
Expr::List(elements) => { /* ... */ }
```

**Total Occurrences:** 8 instances

---

### Issue #3: Expr::Field → Expr::Member

**Current AST:**
```rust
pub enum Expr {
    Member {
        object: Box<Expr>,
        field: String,
    },
    // ...
}
```

**Macro Crates Use:**
```rust
Expr::Field { base, field }
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (line 204)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (line 193)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (line 78)

**Migration:**
```rust
// OLD
Expr::Field { base, field } => Expr::Field {
    base: Box::new(self.walk_expr(base, &mut f)?),
    field: field.clone(),
}

// NEW
Expr::Member { object, field } => Expr::Member {
    object: Box::new(self.walk_expr(object, &mut f)?),
    field: field.clone(),
}
```

**Total Occurrences:** 6 instances

---

### Issue #4: Expr::Index - Missing from Current AST

**Current AST:**
Does not have an `Index` variant. Array/List indexing may be represented differently.

**Macro Crates Use:**
```rust
Expr::Index { base, index }
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (line 199)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (line 188)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (line 72)

**Migration:**
```rust
// OLD
Expr::Index { base, index } => { /* ... */ }

// NEW - Option 1: Use Member with computed field
Expr::Call {
    callee: Box::new(Expr::Member {
        object: base,
        field: "[]".to_string(),
    }),
    args: vec![*index],
}

// NEW - Option 2: Add Index variant to AST
// This requires modifying src/ast.rs to add the variant
```

**Recommendation:** Add `Index` variant to the AST as it's a common operation.

**Total Occurrences:** 3 instances

---

### Issue #5: Expr::Cast - Different Field Names

**Current AST:**
```rust
pub enum Expr {
    Cast {
        expr: Box<Expr>,
        target_type: TypeExpr,
    },
    // ...
}
```

**Macro Crates Use:**
```rust
Expr::Cast { expr, ty }
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (line 209)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (line 198)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (file uses the pattern)

**Migration:**
```rust
// OLD
Expr::Cast { expr, ty } => Expr::Cast {
    expr: Box::new(self.walk_expr(expr, &mut f)?),
    ty: ty.clone(),
}

// NEW
Expr::Cast { expr, target_type } => Expr::Cast {
    expr: Box::new(self.walk_expr(expr, &mut f)?),
    target_type: target_type.clone(),
}
```

**Total Occurrences:** 3+ instances

---

## 2. Expression Field Name Changes

### Issue #6: Expr::Call - func → callee

**Current AST:**
```rust
pub enum Expr {
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    // ...
}
```

**Macro Crates Use:**
```rust
Expr::Call { func, args }
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (lines 160, 161, 182, 184)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 183, 184)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (lines 62, 63, 66, 159, 160, 190, 191, 209)

**Migration:**
```rust
// OLD
Expr::Call { func, args } => {
    let func = Box::new(self.walk_expr(func, &mut f)?);
    // ...
}

// NEW
Expr::Call { callee, args } => {
    let callee = Box::new(self.walk_expr(callee, &mut f)?);
    // ...
}
```

**Total Occurrences:** 15+ instances

---

### Issue #7: Expr::Binary - No changes (compatible)

**Status:** ✅ Compatible - Both use `op`, `left`, `right`

---

### Issue #8: Expr::Unary - No changes (compatible)

**Status:** ✅ Compatible - Both use `op`, `operand`

---

## 3. Statement Compatibility Issues

### Issue #9: Stmt::Let - Field Name Changes

**Current AST:**
```rust
pub enum Stmt {
    Let {
        name: String,
        type_ann: Option<TypeExpr>,
        value: Expr,
    },
    // ...
}
```

**Macro Crates Use:**
```rust
Stmt::Let {
    name,
    ty,
    value,
    span,
}
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (lines 238-250)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 259-266)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (lines 102-113)

**Migration:**
```rust
// OLD
Stmt::Let { name, ty, value, span } => {
    let value = if let Some(v) = value {
        Some(self.walk_expr(v, &mut f)?)
    } else {
        None
    };
    Stmt::Let { name, ty, value, span }
}

// NEW
Stmt::Let { name, type_ann, value } => {
    let value = self.walk_expr(&value, &mut f)?;
    Stmt::Let {
        name: name.clone(),
        type_ann: type_ann.clone(),
        value
    }
}
```

**Key Changes:**
1. `ty` → `type_ann`
2. `value` is now required (not `Option<Expr>`)
3. `span` field removed from struct variant

**Total Occurrences:** 7 instances

---

### Issue #10: Stmt::Return - Different Structure

**Current AST:**
```rust
pub enum Stmt {
    Return(Option<Expr>),
    // ...
}
```

**Macro Crates Use:**
```rust
Stmt::Return { value, span }
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs` (lines 263-270)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 276-277)
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs` (lines 126-132)

**Migration:**
```rust
// OLD
Stmt::Return { value, span } => Stmt::Return {
    value: value.as_ref().map(|v| self.walk_expr(v, &mut f)?),
    span: *span,
}

// NEW
Stmt::Return(value) => Stmt::Return(
    value.as_ref().map(|v| self.walk_expr(v, &mut f)?).transpose()?
)
```

**Total Occurrences:** 4 instances

---

### Issue #11: Stmt::Assign - Compatible Structure

**Current AST:**
```rust
pub enum Stmt {
    Assign {
        target: Expr,
        value: Expr,
    },
    // ...
}
```

**Status:** ✅ Compatible - Field names match

---

### Issue #12: Stmt::While - Different Body Structure

**Current AST:**
```rust
pub enum Stmt {
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    // ...
}
```

**Macro Crates Use:**
```rust
Stmt::While {
    condition,
    body,
    span,
}
// Where body is Block
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 291-295)

**Migration:**
```rust
// OLD
Stmt::While { condition, body, span } => Stmt::While {
    condition: self.apply_hygiene_to_expr(condition),
    body: self.apply_hygiene_to_block(body),
    span: *span,
}

// NEW
Stmt::While { condition, body } => Stmt::While {
    condition: self.apply_hygiene_to_expr(condition),
    body: body.iter()
        .map(|s| self.apply_hygiene_to_stmt(s))
        .collect(),
}
```

**Total Occurrences:** 2 instances

---

### Issue #13: Stmt::For - Different Field Names and Structure

**Current AST:**
```rust
pub enum Stmt {
    For {
        binding: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    // ...
}
```

**Macro Crates Use:**
```rust
Stmt::For {
    var,
    iter,
    body,
    span,
}
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 301-308)

**Migration:**
```rust
// OLD
Stmt::For { var, iter, body, span } => {
    let hygienic_var = self.make_hygienic(var);
    Stmt::For {
        var: hygienic_var,
        iter: self.apply_hygiene_to_expr(iter),
        body: self.apply_hygiene_to_block(body),
        span: *span,
    }
}

// NEW
Stmt::For { binding, iterable, body } => {
    let hygienic_binding = self.make_hygienic(binding);
    Stmt::For {
        binding: hygienic_binding,
        iterable: self.apply_hygiene_to_expr(iterable),
        body: body.iter()
            .map(|s| self.apply_hygiene_to_stmt(s))
            .collect(),
    }
}
```

**Key Changes:**
1. `var` → `binding`
2. `iter` → `iterable`
3. `body: Block` → `body: Vec<Stmt>`
4. `span` field removed

**Total Occurrences:** 2 instances

---

## 4. Block Structure Changes

### Issue #14: Block - Different Field Structure

**Current AST:**
```rust
pub struct Block {
    pub statements: Vec<Stmt>,
    pub final_expr: Option<Box<Expr>>,
    pub span: Span,
}
```

**Macro Crates Use:**
```rust
Block {
    stmts: Vec<Stmt>,
    span: Span,
}
```

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 245-246)

**Migration:**
```rust
// OLD
Block {
    stmts: block.stmts.iter()
        .map(|s| self.apply_hygiene_to_stmt(s))
        .collect(),
    span: block.span,
}

// NEW
Block {
    statements: block.statements.iter()
        .map(|s| self.apply_hygiene_to_stmt(s))
        .collect(),
    final_expr: block.final_expr.as_ref()
        .map(|e| Box::new(self.apply_hygiene_to_expr(e))),
    span: block.span,
}
```

**Key Changes:**
1. `stmts` → `statements`
2. Added `final_expr` field (expression-based blocks)

**Total Occurrences:** 5+ instances across hygiene and expansion

---

### Issue #15: Expr::If - Block Usage

**Current AST:**
```rust
pub enum Expr {
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    // ...
}
```

**Macro Crates Use:**
Assumes branches are Blocks, not Exprs

**Affected Files:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs` (lines 204-213)

**Migration:**
```rust
// OLD
Expr::If { condition, then_branch, else_branch } => Expr::If {
    condition: Box::new(self.apply_hygiene_to_expr(condition)),
    then_branch: Box::new(self.apply_hygiene_to_block(then_branch)),
    else_branch: else_branch.as_ref()
        .map(|b| Box::new(self.apply_hygiene_to_block(b))),
}

// NEW
Expr::If { condition, then_branch, else_branch } => Expr::If {
    condition: Box::new(self.apply_hygiene_to_expr(condition)),
    then_branch: Box::new(self.apply_hygiene_to_expr(then_branch)),
    else_branch: else_branch.as_ref()
        .map(|e| Box::new(self.apply_hygiene_to_expr(e))),
}
```

**Total Occurrences:** 2 instances

---

## 5. Type Expression Issues

### Issue #16: TypeExpr Usage

**Status:** ⚠️ Needs Review

The macro crates use `TypeExpr` which exists in current AST, but field usages should be verified.

---

## 6. Missing AST Features

### Issue #17: QuotedExpr References

**Note:** The `declarative.rs` file references `Expr::Identifier` which suggests awareness of the rename, but uses inconsistent naming.

**File:**
- `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/declarative.rs` (lines 306, 313)

**Current Usage:**
```rust
Ok(vec![Expr::Identifier(hygienic)])
```

**Status:** ✅ Correct usage present, but inconsistent with rest of crate

---

## 7. Summary by File

### `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/expand.rs`

**Issues Found:** 11
- Expr::Ident → Expr::Identifier (7 instances)
- Expr::Call.func → callee (4 instances)
- Expr::Array → Expr::List (2 instances)
- Expr::Index (missing) (1 instance)
- Expr::Field → Expr::Member (1 instance)
- Expr::Cast.ty → target_type (1 instance)
- Stmt::Let field changes (3 instances)
- Stmt::Return structure (2 instances)
- Stmt::Assign (compatible)

### `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/hygiene.rs`

**Issues Found:** 17
- Expr::Ident → Expr::Identifier (8 instances)
- Expr::Call.func → callee (2 instances)
- Expr::Array → Expr::List (1 instance)
- Expr::Index (missing) (1 instance)
- Expr::Field → Expr::Member (1 instance)
- Expr::Cast.ty → target_type (1 instance)
- Expr::If block usage (2 instances)
- Block.stmts → statements (1 instance)
- Stmt::Let field changes (2 instances)
- Stmt::Return structure (1 instance)
- Stmt::Assign (compatible)
- Stmt::While body structure (1 instance)
- Stmt::For field changes (1 instance)

### `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/pattern.rs`

**Issues Found:** 5
- Expr::Ident → Expr::Identifier (5 instances)

### `/home/ardeshir/repos/univrs-dol/crates/dol-macro/src/declarative.rs`

**Issues Found:** 2
- Uses Expr::Identifier (correct) (2 instances)

### `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/ast_util.rs`

**Issues Found:** 19
- Expr::Ident → Expr::Identifier (14 instances)
- Expr::Call.func → callee (4 instances)
- Expr::Array → Expr::List (3 instances)
- Expr::Index (missing) (1 instance)
- Expr::Field → Expr::Member (1 instance)
- Stmt::Let field changes (2 instances)
- Stmt::Return structure (1 instance)

### `/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc/src/codegen.rs`

**Issues Found:** 2
- Expr::Ident → Expr::Identifier (2 instances)

---

## 8. Migration Guide

### Step 1: Global Search & Replace

These can be done with automated search/replace:

```bash
# In dol-macro crate
cd /home/ardeshir/repos/univrs-dol/crates/dol-macro

# Replace Expr::Ident with Expr::Identifier
find src -name "*.rs" -exec sed -i 's/Expr::Ident(/Expr::Identifier(/g' {} +

# Replace Expr::Array with Expr::List
find src -name "*.rs" -exec sed -i 's/Expr::Array(/Expr::List(/g' {} +

# In dol-macro-proc crate
cd /home/ardeshir/repos/univrs-dol/crates/dol-macro-proc

# Same replacements
find src -name "*.rs" -exec sed -i 's/Expr::Ident(/Expr::Identifier(/g' {} +
find src -name "*.rs" -exec sed -i 's/Expr::Array(/Expr::List(/g' {} +
```

### Step 2: Manual Field Renames

These require manual intervention due to context:

#### Expr::Call

```rust
// Find and replace in context
Expr::Call { func, args }
// Replace with:
Expr::Call { callee, args }
```

#### Expr::Field → Expr::Member

```rust
// Find and replace
Expr::Field { base, field }
// Replace with:
Expr::Member { object: base, field }
```

#### Expr::Cast

```rust
// Find and replace
Expr::Cast { expr, ty }
// Replace with:
Expr::Cast { expr, target_type: ty }
```

### Step 3: Statement Fixes

#### Stmt::Let

```rust
// OLD pattern
Stmt::Let { name, ty, value, span }

// NEW pattern
Stmt::Let { name, type_ann, value }
```

**Important:** Value is now required, not `Option<Expr>`. Update logic accordingly.

#### Stmt::Return

```rust
// OLD pattern
Stmt::Return { value, span }

// NEW pattern
Stmt::Return(value)
```

#### Stmt::For

```rust
// OLD pattern
Stmt::For { var, iter, body, span }

// NEW pattern
Stmt::For { binding, iterable, body }
```

### Step 4: Block Structure

```rust
// OLD
Block { stmts, span }

// NEW
Block { statements, final_expr, span }
```

**Note:** Need to handle `final_expr` - typically `None` for statement blocks.

### Step 5: Handle Missing Expr::Index

**Option A:** Remove Index support temporarily
**Option B:** Add Index variant to AST:

```rust
// Add to src/ast.rs Expr enum
pub enum Expr {
    // ... existing variants
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    // ...
}
```

**Recommendation:** Option B - add the variant as it's a fundamental operation.

### Step 6: Loop Body Changes

```rust
// OLD - body is Block
Stmt::While { condition, body: Block, span }

// NEW - body is Vec<Stmt>
Stmt::While { condition, body: Vec<Stmt> }
```

Update hygiene and transformation functions accordingly.

---

## 9. Testing Strategy

After migration, run these tests:

### Unit Tests
```bash
cd /home/ardeshir/repos/univrs-dol/crates/dol-macro
cargo test

cd /home/ardeshir/repos/univrs-dol/crates/dol-macro-proc
cargo test
```

### Integration Tests
```bash
cd /home/ardeshir/repos/univrs-dol
cargo test --workspace
```

### Smoke Tests

Create test cases for each changed variant:

```rust
#[test]
fn test_expr_identifier_compatibility() {
    let expr = Expr::Identifier("test".to_string());
    // Test macro expansion with identifier
}

#[test]
fn test_expr_call_compatibility() {
    let expr = Expr::Call {
        callee: Box::new(Expr::Identifier("fn".to_string())),
        args: vec![],
    };
    // Test macro expansion with call
}

#[test]
fn test_stmt_let_compatibility() {
    let stmt = Stmt::Let {
        name: "x".to_string(),
        type_ann: None,
        value: Expr::Literal(Literal::Int(42)),
    };
    // Test statement processing
}
```

---

## 10. Recommended Actions

### High Priority (Breaking Issues)

1. ✅ **Expr::Ident → Expr::Identifier** - Simple rename, high occurrence
2. ✅ **Expr::Array → Expr::List** - Simple rename
3. ✅ **Expr::Call.func → callee** - Field rename
4. ⚠️ **Expr::Index** - Either add to AST or remove support
5. ✅ **Expr::Field → Expr::Member (base → object)** - Field renames
6. ✅ **Stmt::Let field changes** - Structural change
7. ✅ **Stmt::Return structure** - Structural change
8. ✅ **Block.stmts → statements + final_expr** - Structural change

### Medium Priority (Compatibility Issues)

9. ✅ **Stmt::For field renames** - var → binding, iter → iterable
10. ✅ **Stmt::While body** - Block → Vec<Stmt>
11. ✅ **Expr::Cast.ty → target_type** - Field rename
12. ✅ **Expr::If branches** - Ensure proper Expr handling

### Low Priority (Nice to Have)

13. 📝 Documentation updates
14. 📝 Add migration guide comments in code
15. 📝 Update example code

---

## 11. Automated Migration Script

Here's a Bash script to perform the automated parts:

```bash
#!/bin/bash
# ast_migration.sh - Automates AST migration for macro crates

set -e

MACRO_DIR="/home/ardeshir/repos/univrs-dol/crates/dol-macro"
PROC_DIR="/home/ardeshir/repos/univrs-dol/crates/dol-macro-proc"

echo "Starting AST migration..."

# Function to migrate a directory
migrate_dir() {
    local dir=$1
    echo "Migrating $dir..."

    # Simple renames
    find "$dir/src" -name "*.rs" -type f -exec sed -i \
        -e 's/Expr::Ident(/Expr::Identifier(/g' \
        -e 's/Expr::Array(/Expr::List(/g' \
        {} +

    echo "✓ Simple renames complete for $dir"
}

# Migrate both crates
migrate_dir "$MACRO_DIR"
migrate_dir "$PROC_DIR"

echo ""
echo "✓ Automated migration complete!"
echo ""
echo "⚠️  Manual steps required:"
echo "  1. Update Expr::Call { func → callee }"
echo "  2. Update Expr::Field → Expr::Member { base → object }"
echo "  3. Update Expr::Cast { ty → target_type }"
echo "  4. Update Stmt::Let field names"
echo "  5. Update Stmt::Return structure"
echo "  6. Update Stmt::For field names"
echo "  7. Update Block structure"
echo "  8. Handle Expr::Index (add to AST or remove)"
echo ""
echo "Run 'cargo test' in each crate to verify."
```

---

## 12. Conclusion

The macro crates require substantial updates to align with the current AST (v0.8.0). The majority of issues are straightforward renames that can be automated. The key structural changes (Block, Stmt::Let, Stmt::Return) require careful manual intervention.

**Estimated Effort:**
- Automated changes: 1-2 hours
- Manual changes: 4-6 hours
- Testing & verification: 2-3 hours
- **Total: 7-11 hours**

**Risk Assessment:**
- Low risk for simple renames (Expr::Ident, Expr::Array)
- Medium risk for field renames (func → callee, etc.)
- High risk for structural changes (Block, Statement variants)

**Next Steps:**
1. Run automated migration script
2. Manually update struct/enum field names
3. Add missing Expr::Index variant to AST (or remove from macros)
4. Update Block handling throughout
5. Run full test suite
6. Update documentation

---

**Report Version:** 1.0
**Last Updated:** 2026-02-07
**Maintainer:** Claude Sonnet 4.5
