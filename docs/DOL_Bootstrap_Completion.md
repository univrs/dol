# DOL Bootstrap Completion - Pragmatic Hybrid Approach

## Mission

Get the DOL self-hosting compiler to compile successfully. Perfection is NOT the goal - a working bootstrap is. Manual patches and pragmatic fixes are acceptable.

## Current State

- **Error count**: 700
- **Phase**: Bootstrap verification (Stage 2)
- **Files**: `dol/*.dol` → `target/stage1/*.rs`

## Success Criteria

```bash
cd ~/repos/univrs-dol/target/stage1
cargo build 2>&1 | grep -c "^error"
# Target: 0 errors (or <10 with known workarounds)
```

## Error Priority Queue

Work through these in order. Each category has specific fix strategies.

---

### PRIORITY 1: Missing Types (E0433) - 68 errors
**Strategy**: Add missing type definitions

These types are referenced but not defined:
- `UseStmt`
- `GeneMember`  
- `FieldDecl`
- (identify others from error output)

**Fix approach**:
```rust
// Add to generated prelude.rs or relevant module:

#[derive(Debug, Clone, PartialEq)]
pub struct UseStmt {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

// OR if they should be type aliases:
pub type UseStmt = UseDecl;
pub type GeneMember = Statement;
```

**Action**: 
1. List all E0433 undefined types: `cargo build 2>&1 | grep E0433 | grep -oP "cannot find type \`\K[^']+"`
2. For each type, either:
   - Add struct/enum definition
   - Add type alias to existing type
   - Add import if it exists elsewhere

---

### PRIORITY 2: Intermediate Struct Wrappers (E0533) - 20 errors
**Strategy**: Generate wrapper structs for complex enum variants

DOL uses patterns like `Stmt.Let(LetStmt { name, value })` but codegen produces inline variants.

**Fix approach A** (preferred): Generate intermediate structs
```rust
// Generate these alongside the enum:
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: String,
    pub type_ann: Option<TypeExpr>,
    pub value: Expr,
}

pub enum Stmt {
    Let(LetStmt),  // Wrap in struct
    // ...
}
```

**Fix approach B** (fallback): Manual patch file
```rust
// patches/stmt_wrappers.rs
pub type LetStmt = ...;  // Compatibility shim
```

**Action**:
1. List affected variants from E0533 errors
2. Modify `gen_enum_from_gene()` in codegen.dol to generate wrapper structs for struct variants
3. If too complex, create manual patch file

---

### PRIORITY 3: Option Mismatches (E0308) - 222 errors  
**Strategy**: Fix Option wrapping inconsistencies

Two sub-categories:

**A) Field should be Option but isn't (codegen fix)**
```rust
// If DOL code does: if field.x != None { ... }
// Then field.x should be Option<T>

// In gen_field(), detect optional fields and wrap:
pub constraint: Option<Expr>,  // Not just Expr
```

**B) Comparison with None on non-Option (source pattern)**
```dol
// DOL code:
if value != None { use(value) }

// Should generate:
if let Some(v) = value { use(v) }
// OR if value is not Option:
{ use(value) }  // Just remove the check
```

**Fix approach**: 
1. Identify which fields DOL treats as optional (search for `!= None` and `== None` patterns)
2. Update `gen_field()` to wrap those fields in `Option<T>`
3. For remaining cases, add a codegen pass that transforms `x != None` to `x.is_some()` when x is Option

**Pragmatic fallback**: Create a sed/awk script to post-process generated Rust:
```bash
# patches/fix_option.sh
sed -i 's/(\([a-z_]*\) != None)/\1.is_some()/g' target/stage1/*.rs
sed -i 's/(\([a-z_]*\) == None)/\1.is_none()/g' target/stage1/*.rs
```

---

### PRIORITY 4: Missing Methods/Variants (E0599) - 160 errors
**Strategy**: Add impl blocks and missing variants

Sub-categories:

**A) Methods called on wrong type**
```
error: no method named `unwrap` found for struct `Vec<...>`
```
Fix: The code expects Option, got Vec. Related to E0308.

**B) Missing enum variants**
```
error: no variant named `Foo` on enum `Bar`
```
Fix: Add missing variant to enum definition in codegen.

**C) Missing impl methods**
```
error: no method named `length` found for struct `Vec`
```
Fix: Map DOL method names to Rust equivalents:
- `length()` → `len()`
- `push()` → `push()` ✓
- `is_empty()` → `is_empty()` ✓

**Action**:
1. Collect all E0599 errors
2. Categorize into: wrong-type vs missing-variant vs method-name-mapping
3. Add method name mappings to `gen_expr()` Call handling
4. Add missing variants to enum generation

---

### PRIORITY 5: Field Access Errors (E0609) - 43 errors
**Strategy**: Fix struct field shapes

```
error: no field `foo` on type `Bar`
```

**Causes**:
- Field name mapping wrong (DOL `type` → Rust `type_`)
- Field exists on different type
- Field missing from struct generation

**Action**:
1. For each E0609, check if it's a naming issue or structural issue
2. Update `gen_field()` name mappings
3. Add missing fields to struct generation

---

## Pragmatic Patches Directory

Create `target/stage1/patches/` for manual fixes that are too complex to automate:

```
target/stage1/
├── patches/
│   ├── mod.rs           # pub mod declarations
│   ├── type_aliases.rs  # type UseStmt = UseDecl; etc.
│   ├── compat.rs        # Compatibility shims
│   └── apply.sh         # Post-processing script
├── src/
│   └── lib.rs           # Add: mod patches;
```

## Workflow Loop

```
REPEAT:
    1. cargo build 2>&1 > errors.txt
    2. COUNT=$(grep -c "^error" errors.txt)
    3. echo "Errors remaining: $COUNT"
    
    4. IF $COUNT == 0:
           DONE - Bootstrap successful!
           
    5. IF $COUNT < 20:
           Review errors manually
           Apply targeted fixes or patches
           
    6. ELSE:
           Categorize errors by code (E0433, E0599, etc.)
           Pick highest-impact category
           Apply systematic fix
           
    7. git commit -m "bootstrap: reduce errors to $COUNT"
    
UNTIL $COUNT == 0 OR stuck for 3 iterations
```

## Stuck Protocol

If error count plateaus for 3+ iterations:

1. **Dump full error list**: `cargo build 2>&1 | head -200 > stuck_errors.txt`
2. **Identify blocking pattern**: Look for one root cause creating many errors
3. **Consider manual patch**: Some patterns are faster to patch than fix in codegen
4. **Report to human**: Share stuck_errors.txt and analysis

## Commands Reference

```bash
# Full build check
cd ~/repos/univrs-dol/target/stage1 && cargo build 2>&1

# Error count only
cargo build 2>&1 | grep -c "^error"

# Errors by category
cargo build 2>&1 | grep "^error\[" | sed 's/].*/]/' | sort | uniq -c | sort -rn

# Specific error type
cargo build 2>&1 | grep "E0433" | head -20

# Find undefined types
cargo build 2>&1 | grep E0433 | grep -oP "cannot find (type|struct|enum) \`\K[^\`]+" | sort -u

# Find missing methods
cargo build 2>&1 | grep E0599 | grep -oP "no method named \`\K[^\`]+" | sort -u

# Regenerate from DOL (after codegen.dol changes)
cd ~/repos/univrs-dol
cargo run --bin dol-compile -- dol/*.dol -o target/stage1/src/
```

## Exit Conditions

**Success**: 
- `cargo build` completes with 0 errors
- OR completes with <10 errors that have documented workarounds

**Escalate to human**:
- Stuck at same error count for 3+ iterations
- Error requires architectural decision (not just codegen fix)
- Unsure whether to patch DOL source vs fix codegen

---

## START

Begin with Priority 1 (E0433 - Missing Types). Report error count after each fix round.
