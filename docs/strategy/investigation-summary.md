# DOL to WASM Investigation Summary

**Date:** 2025-12-31
**Author:** WASM Backend Architect Agent
**Status:** Requirements Extracted

---

## Executive Summary

The DOL to WASM compilation pipeline is **partially functional**. The direct WASM compiler successfully handles simple arithmetic functions but fails for all other language constructs. This document extracts specific requirements for fixing the gaps.

---

## Current State: What Works vs What Fails

### Working (Level 1-2 Tests)

| Feature | Status | Evidence |
|---------|--------|----------|
| Empty modules | PASS | Compiles to valid minimal WASM |
| Simple functions | PASS | `/add.wasm` (42 bytes, valid) |
| Integer arithmetic (+, -, *, /, %) | PASS | Direct WASM emission works |
| Comparison operators (==, !=, <, >, <=, >=) | PASS | I64 comparison ops emitted |
| Logical operators (and, or) | PASS | I64 bitwise ops emitted |
| Function parameters | PASS | LocalGet instructions work |
| Return statements | PASS | Single returns emit correctly |

### Failing (Level 3-5 Tests)

| Feature | Status | Blocking For |
|---------|--------|--------------|
| Genes | FAIL | Level 3+ tests |
| Traits | FAIL | Level 5+ tests |
| Systems | FAIL | Level 5+ tests |
| If/else expressions | FAIL | Level 4+ tests |
| Match expressions | FAIL | Level 4+ tests |
| Let bindings | FAIL | All non-trivial programs |
| Loops (for/while) | FAIL | Iterative algorithms |
| String literals | FAIL | Any string handling |
| Local variables | FAIL | All non-trivial programs |

---

## Specific Error Messages by Category

### Category 1: Declaration Type Errors

**Source File:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

```
WasmError: "No functions found in module - only function declarations are currently supported for WASM compilation"
```

**Trigger:** Attempting to compile Gene, Trait, System, Constraint, or Evolution declarations.

**Affected Test Cases:**
- `test-cases/level3-types/simple_gene.dol`
- `test-cases/level3-types/gene_with_constraint.dol`
- `test-cases/level5-advanced/trait_def.dol`
- `test-cases/level5-advanced/system_impl.dol`

---

### Category 2: Statement Errors

**Source File:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` (lines 344-364)

| Statement Type | Error Message |
|----------------|---------------|
| `Stmt::Let` | `"Let bindings not yet supported in WASM compilation"` |
| `Stmt::Assign` | `"Assignments not yet supported in WASM compilation"` |
| `Stmt::For`, `Stmt::While`, `Stmt::Loop` | `"Loops not yet supported in WASM compilation"` |
| `Stmt::Break`, `Stmt::Continue` | `"Break/continue not yet supported in WASM compilation"` |

---

### Category 3: Expression Errors

**Source File:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` (lines 369-514)

| Expression Type | Error Message |
|-----------------|---------------|
| `Expr::If` | `"If expressions not yet supported in WASM compilation"` |
| `Expr::Match` | `"Match expressions not yet supported in WASM compilation"` |
| `Expr::Block` | `"Block expressions not yet supported in WASM compilation"` |
| `Expr::Lambda` | `"Lambda expressions not yet supported in WASM compilation"` |
| `Expr::Member` | `"Member access not yet supported in WASM compilation"` |
| `Expr::Unary` | `"Unary expressions not yet supported in WASM compilation"` |
| `Literal::String` | `"String literals not yet supported in WASM compilation"` |
| `Literal::Char` | `"Char literals not yet supported in WASM compilation"` |
| `Literal::Null` | `"Null literals not yet supported in WASM compilation"` |

---

### Category 4: Type Errors

**Source File:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` (lines 268-300)

| Type | Error Message |
|------|---------------|
| Generic types | `"Generic types not yet supported in WASM compilation"` |
| Function types | `"Function types not yet supported in WASM compilation"` |
| Tuple types | `"Tuple types not yet supported in WASM compilation"` |
| Enum types | `"Enum types not yet supported in WASM compilation"` |
| Never type | `"Never type not supported in WASM compilation"` |

---

## Failing Test Cases: Exact DOL Code

### Test Case 1: Simple Gene (Level 3)

**File:** `/home/ardeshir/repos/univrs-dol/test-cases/level3-types/simple_gene.dol`

```dol
module types @ 0.1.0

gene Point {
    has x: Float64
    has y: Float64
}
```

**Error:** `"No functions found in module - only function declarations are currently supported for WASM compilation"`

**Required WASM Constructs:**
- Memory section for field storage
- Constructor function (`$new_Point`)
- Getter functions (`$Point.get_x`, `$Point.get_y`)
- Setter functions (`$Point.set_x`, `$Point.set_y`)
- Memory layout calculation (16 bytes for 2 x Float64)

---

### Test Case 2: Gene with Constraint (Level 3)

**File:** `/home/ardeshir/repos/univrs-dol/test-cases/level3-types/gene_with_constraint.dol`

```dol
module constrained @ 0.1.0

gene Credits {
    has amount: UInt64

    constraint non_negative {
        this.amount >= 0
    }
}
```

**Errors:**
1. `"No functions found in module"` - Gene not supported
2. Constraint expression not compiled

**Required WASM Constructs:**
- All Gene constructs (above)
- Validation function (`$Credits.validate_non_negative`)
- Trap instruction for constraint violations
- `this` reference resolution to memory offset

---

### Test Case 3: If/Else Control Flow (Level 4)

**File:** `/home/ardeshir/repos/univrs-dol/test-cases/level4-control/if_else.dol`

**Original intended test (from stress-test documentation):**
```dol
module control @ 0.1.0

fun max(a: Int32, b: Int32) -> Int32 {
    if a > b {
        return a
    } else {
        return b
    }
}
```

**Note:** The current file was simplified to work around the bug, but the intended test above would fail.

**Error:** `"If expressions not yet supported in WASM compilation"`

**Required WASM Constructs:**
```wasm
;; WASM structured control flow:
(if (result i32)
  (i32.gt_s (local.get $a) (local.get $b))
  (then (local.get $a))
  (else (local.get $b))
)
```

Alternatively using block/br_if pattern:
```wasm
(block $exit (result i32)
  (local.get $a)
  (local.get $b)
  (i32.gt_s)
  (if
    (then (local.get $a) (br $exit))
  )
  (local.get $b)
)
```

---

### Test Case 4: Match Expression (Level 4)

**File:** `/home/ardeshir/repos/univrs-dol/test-cases/level4-control/match_expr.dol`

**Original intended test (from stress-test documentation):**
```dol
module matching @ 0.1.0

fun describe(n: Int32) -> String {
    match n {
        0 => "zero",
        1 => "one",
        _ => "many"
    }
}
```

**Note:** Current file was simplified to work around the bug.

**Errors:**
1. `"Match expressions not yet supported in WASM compilation"`
2. `"String literals not yet supported in WASM compilation"`

**Required WASM Constructs:**
```wasm
;; Match using br_table for efficient dispatch:
(block $default (result i32)
  (block $case1
    (block $case0
      (local.get $n)
      (br_table $case0 $case1 $default)
    )
    ;; case 0: return ptr to "zero"
    (i32.const 0) ;; string ptr
    (return)
  )
  ;; case 1: return ptr to "one"
  (i32.const 5) ;; string ptr
  (return)
)
;; default: return ptr to "many"
(i32.const 9) ;; string ptr
```

Also requires:
- Data section for string constants
- String representation as (ptr, len) pair
- Memory allocation for strings

---

### Test Case 5: Trait Definition (Level 5)

**File:** `/home/ardeshir/repos/univrs-dol/test-cases/level5-advanced/trait_def.dol`

```dol
trait math.calculator {
    calculator can add
    calculator can multiply
    calculator is deterministic
}

exegesis {
    The math.calculator trait defines capabilities for
    performing arithmetic operations. Calculators can
    add and multiply, and their operations are deterministic.
}
```

**Error:** `"No functions found in module"` - Traits are not functions

**Required WASM Constructs:**
- VTable structure for trait methods
- Function table (`funcref table`)
- Indirect call infrastructure (`call_indirect`)
- Type checking for trait bounds

---

### Test Case 6: System Implementation (Level 5)

**File:** `/home/ardeshir/repos/univrs-dol/test-cases/level5-advanced/system_impl.dol`

```dol
system greeting.service @ 0.1.0 {
    requires entity.greetable >= 0.0.1

    all greetings is logged
    all responses is polite
}

exegesis {
    The greeting.service system provides greeting functionality.
    It requires the greetable trait and ensures all greetings
    are logged and responses are polite.
}
```

**Error:** `"No functions found in module"` - Systems are not functions

**Required WASM Constructs:**
- VTable implementation for trait methods
- Method dispatch table linking
- Dependency injection pattern
- Aspect-oriented constraint wrapping

---

## Technical Requirements for Each Fix

### Requirement 1: Local Variables (P0 - Critical)

**Impact:** Required for ANY non-trivial program

**Implementation Location:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

**Required Changes:**

```rust
// Add to WasmCompiler struct
struct WasmCompiler {
    optimize: bool,
    debug_info: bool,
    // NEW: Track local variables
    local_vars: HashMap<String, (u32, ValType)>,  // name -> (index, type)
}

// Add method
fn emit_let_binding(
    &mut self,
    function: &mut Function,
    binding: &LetStmt,
    func_decl: &FunctionDecl,
) -> Result<(), WasmError> {
    // 1. Emit initializer expression
    self.emit_expression(function, &binding.init, func_decl)?;

    // 2. Allocate local variable index
    let local_idx = self.local_vars.len() as u32 + func_decl.params.len() as u32;
    let var_type = self.infer_type(&binding.init)?;

    // 3. Store in local
    function.instruction(&Instruction::LocalSet(local_idx));

    // 4. Track for future references
    self.local_vars.insert(binding.name.clone(), (local_idx, var_type));

    Ok(())
}
```

**WASM Constructs:**
- `local.get` - Read local variable
- `local.set` - Write local variable
- `local.tee` - Write and keep on stack
- Local variable section in function definition

---

### Requirement 2: If/Else Control Flow (P0 - Critical)

**Impact:** Required for any conditional logic

**Implementation Location:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

**Required Changes:**

```rust
fn emit_if_expr(
    &mut self,
    function: &mut Function,
    if_expr: &IfExpr,
    func_decl: &FunctionDecl,
) -> Result<(), WasmError> {
    // 1. Emit condition (must produce i32 for WASM)
    self.emit_expression(function, &if_expr.condition, func_decl)?;

    // 2. Determine result type (if both branches return)
    let result_type = self.infer_if_result_type(if_expr)?;

    // 3. Emit structured if
    let block_type = match result_type {
        Some(t) => BlockType::Result(t),
        None => BlockType::Empty,
    };

    function.instruction(&Instruction::If(block_type));

    // 4. Emit then branch
    for stmt in &if_expr.then_branch {
        self.emit_statement(function, stmt, func_decl)?;
    }

    // 5. Emit else branch
    if let Some(else_branch) = &if_expr.else_branch {
        function.instruction(&Instruction::Else);
        for stmt in else_branch {
            self.emit_statement(function, stmt, func_decl)?;
        }
    }

    function.instruction(&Instruction::End);
    Ok(())
}
```

**WASM Constructs:**
- `if` - Begin conditional block
- `else` - Switch to else branch
- `end` - Close block
- `BlockType::Result(T)` for value-returning if

---

### Requirement 3: Gene Compilation (P1 - High)

**Impact:** Enables DOL's core data abstraction

**Implementation Location:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

**Required Changes:**

```rust
fn compile_gene(&mut self, gene: &Gene) -> Result<Vec<u8>, WasmError> {
    // 1. Calculate memory layout
    let layout = self.calculate_gene_layout(gene)?;

    // 2. Generate memory section
    let memory = self.generate_memory_section(&layout)?;

    // 3. Generate constructor
    let constructor = self.generate_gene_constructor(gene, &layout)?;

    // 4. Generate field accessors
    let mut accessors = Vec::new();
    for (idx, field) in gene.fields.iter().enumerate() {
        let getter = self.generate_field_getter(gene, field, &layout, idx)?;
        let setter = self.generate_field_setter(gene, field, &layout, idx)?;
        accessors.push(getter);
        accessors.push(setter);
    }

    // 5. Combine into module
    self.emit_gene_module(memory, constructor, accessors)
}

struct GeneLayout {
    size: u32,           // Total bytes
    field_offsets: Vec<u32>,  // Offset for each field
    field_types: Vec<ValType>, // WASM type for each field
}
```

**WASM Constructs:**
- `(memory 1)` - Linear memory
- `i32.load` / `i32.store` - Field access
- `f64.load` / `f64.store` - Float field access
- Memory offset calculations
- Export section for accessor functions

---

### Requirement 4: Match Expressions (P1 - High)

**Impact:** Enables pattern matching

**Implementation Location:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

**Required Changes:**

```rust
fn emit_match_expr(
    &mut self,
    function: &mut Function,
    match_expr: &MatchExpr,
    func_decl: &FunctionDecl,
) -> Result<(), WasmError> {
    // 1. Emit scrutinee
    self.emit_expression(function, &match_expr.scrutinee, func_decl)?;

    // 2. Store in temporary local
    let scrutinee_local = self.allocate_temp_local(ValType::I64)?;
    function.instruction(&Instruction::LocalSet(scrutinee_local));

    // 3. For integer patterns, use br_table if dense
    if self.can_use_br_table(&match_expr.arms) {
        self.emit_match_br_table(function, match_expr, scrutinee_local)?;
    } else {
        // Fall back to chained if/else
        self.emit_match_if_chain(function, match_expr, scrutinee_local)?;
    }

    Ok(())
}
```

**WASM Constructs:**
- `br_table` - Jump table for dense integer patterns
- `block` / `br` - For sparse patterns
- Pattern compilation to comparison sequences

---

### Requirement 5: Loop Constructs (P2 - Medium)

**Impact:** Enables iterative algorithms

**Implementation Location:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

**Required Changes:**

```rust
fn emit_for_loop(
    &mut self,
    function: &mut Function,
    for_loop: &ForLoop,
    func_decl: &FunctionDecl,
) -> Result<(), WasmError> {
    // 1. Initialize loop variable
    self.emit_let_binding(function, &for_loop.init, func_decl)?;
    let loop_var = self.local_vars.get(&for_loop.var_name).unwrap().0;

    // 2. Create loop block structure
    function.instruction(&Instruction::Block(BlockType::Empty)); // outer
    function.instruction(&Instruction::Loop(BlockType::Empty));  // inner

    // 3. Check condition
    self.emit_expression(function, &for_loop.condition, func_decl)?;
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::BrIf(1)); // Exit to outer if false

    // 4. Emit body
    for stmt in &for_loop.body {
        self.emit_statement(function, stmt, func_decl)?;
    }

    // 5. Increment and continue
    self.emit_expression(function, &for_loop.increment, func_decl)?;
    function.instruction(&Instruction::LocalSet(loop_var));
    function.instruction(&Instruction::Br(0)); // Continue to inner

    // 6. Close blocks
    function.instruction(&Instruction::End); // inner loop
    function.instruction(&Instruction::End); // outer block

    Ok(())
}
```

**WASM Constructs:**
- `loop` - Create loop block
- `block` - Outer block for break
- `br 0` - Continue (branch to loop start)
- `br 1` - Break (branch to outer block end)

---

### Requirement 6: String Literals (P1 - High)

**Impact:** Enables text handling

**Implementation Location:** `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

**Required Changes:**

```rust
struct StringTable {
    data: Vec<u8>,        // Concatenated string bytes
    offsets: HashMap<String, (u32, u32)>,  // string -> (offset, length)
}

fn emit_string_literal(
    &mut self,
    function: &mut Function,
    s: &str,
) -> Result<(), WasmError> {
    // 1. Add to string table (or reuse existing)
    let (offset, len) = self.string_table.intern(s);

    // 2. Push pointer and length
    function.instruction(&Instruction::I32Const(offset as i32));
    function.instruction(&Instruction::I32Const(len as i32));

    Ok(())
}

// Add data section to module
fn emit_data_section(&self, module: &mut Module) {
    let mut data_section = DataSection::new();
    data_section.active(
        0,                           // memory index
        &Instruction::I32Const(0),   // offset
        self.string_table.data.iter().copied(),
    );
    module.section(&data_section);
}
```

**WASM Constructs:**
- `(data ...)` section for string bytes
- Memory index 0 for string storage
- String ABI: (i32 ptr, i32 len) pair

---

## Priority Ranking by Blocking Dependencies

### P0 - Critical (Blocks all useful programs)

| Priority | Requirement | Blocks |
|----------|-------------|--------|
| P0.1 | Local Variables | All non-trivial programs |
| P0.2 | If/Else | Conditional logic, Level 4 tests |
| P0.3 | `dol-wasm` CLI | User access to WASM output |

### P1 - High (Blocks major features)

| Priority | Requirement | Blocks |
|----------|-------------|--------|
| P1.1 | Gene Compilation | Level 3 tests, data modeling |
| P1.2 | Match Expressions | Pattern matching, Level 4 tests |
| P1.3 | String Support | Text processing, Level 4+ tests |

### P2 - Medium (Blocks specific use cases)

| Priority | Requirement | Blocks |
|----------|-------------|--------|
| P2.1 | Loops | Iterative algorithms |
| P2.2 | Trait Compilation | Polymorphism, Level 5 tests |
| P2.3 | System Compilation | Full DOL semantics |

### P3 - Low (Nice to have)

| Priority | Requirement | Blocks |
|----------|-------------|--------|
| P3.1 | Unary Operators | Negation, boolean not |
| P3.2 | Block Expressions | Multi-statement blocks |
| P3.3 | Lambda Expressions | Higher-order functions |

---

## Dependency Graph

```
P0.1 (Local Vars) ────┬──> P0.2 (If/Else) ──> P1.2 (Match)
                      │
                      ├──> P2.1 (Loops)
                      │
                      └──> P1.1 (Genes) ──> P2.2 (Traits) ──> P2.3 (Systems)
                              │
                              └──> P1.3 (Strings) [also needs memory]

P0.3 (CLI) ──> CI Integration ──> Release
```

---

## Test Cases That Must Pass

After implementing P0 + P1 requirements, these tests should pass:

### Level 3 (Types) - After P1.1 (Genes)
- `test-cases/level3-types/simple_gene.dol`
- `test-cases/level3-types/gene_with_constraint.dol`

### Level 4 (Control Flow) - After P0.2 + P1.2
- `test-cases/level4-control/if_else.dol` (original version)
- `test-cases/level4-control/match_expr.dol` (original version)

### Level 5 (Advanced) - After P2.2 + P2.3
- `test-cases/level5-advanced/trait_def.dol`
- `test-cases/level5-advanced/system_impl.dol`

---

## Files Requiring Modification

| File | Changes Required |
|------|------------------|
| `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` | Add local vars, if/else, match, loops, strings, genes |
| `/home/ardeshir/repos/univrs-dol/src/wasm/mod.rs` | No changes needed |
| `/home/ardeshir/repos/univrs-dol/src/bin/dol-wasm.rs` | NEW FILE - CLI binary |
| `/home/ardeshir/repos/univrs-dol/tests/wasm_execution.rs` | Remove `#[ignore]` from tests as features are added |

---

## Success Metrics

| Metric | Current | After P0 | After P1 | After P2 |
|--------|---------|----------|----------|----------|
| Test cases passing | 27% (3/11) | 45% (5/11) | 73% (8/11) | 100% (11/11) |
| Declaration types | 1/7 (14%) | 1/7 (14%) | 3/7 (43%) | 6/7 (86%) |
| Expression types | 6/15 (40%) | 8/15 (53%) | 12/15 (80%) | 15/15 (100%) |
| Statement types | 2/8 (25%) | 4/8 (50%) | 5/8 (63%) | 7/8 (88%) |

---

*Generated by WASM Backend Architect Agent - 2025-12-31*
