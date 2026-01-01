# Control Flow Design for DOL WASM Compiler

This document describes the design for implementing control flow constructs in the DOL-to-WASM compiler. The current compiler at `src/wasm/compiler.rs` handles basic expressions and function calls but lacks support for control flow.

## Table of Contents

1. [Overview](#overview)
2. [WASM Control Flow Primer](#wasm-control-flow-primer)
3. [Label Management Strategy](#label-management-strategy)
4. [If/Else Emission](#ifelse-emission)
5. [Match Expression Emission](#match-expression-emission)
6. [Loop Emission](#loop-emission)
7. [Block Expression Emission](#block-expression-emission)
8. [Break and Continue](#break-and-continue)
9. [Edge Cases and Error Handling](#edge-cases-and-error-handling)
10. [Implementation Plan](#implementation-plan)

---

## Overview

### Current State

The compiler in `src/wasm/compiler.rs` currently supports:
- Function declarations with typed parameters
- Integer (i64) and float (f64) literals
- Binary operations (add, sub, mul, div, comparisons, logical)
- Function calls and return statements
- Variable references (function parameters)

### Target State

After implementing this design, the compiler will support:
- If/else expressions with proper type handling
- Match expressions with pattern matching
- For loops over ranges
- While loops
- Infinite loops (loop)
- Break and continue statements
- Block expressions with final values

---

## WASM Control Flow Primer

WASM uses structured control flow, not goto-style jumps. Key instructions:

### Basic Instructions

```wasm
;; Block - creates a labeled block, br jumps to END of block
(block $label (result i32)
  ...
  (br $label)           ;; jumps to after the block
  ...
)

;; Loop - creates a labeled loop, br jumps to START of loop
(loop $label (result i32)
  ...
  (br $label)           ;; jumps back to loop start (continue)
  ...
)

;; If/else - conditional branching
(if (result i32)
  (condition)           ;; i32 on stack (0 = false, non-0 = true)
  (then
    (i32.const 1))
  (else
    (i32.const 0)))

;; Branch instructions
(br $label)             ;; unconditional branch
(br_if $label)          ;; conditional branch (pops i32 condition)
(br_table $l0 $l1 $l2 $default)  ;; computed branch (pops i32 index)
```

### Label Depth

WASM uses relative label depths, not absolute labels:
- `br 0` branches to the innermost block/loop
- `br 1` branches to the second innermost
- etc.

---

## Label Management Strategy

### LabelStack Structure

```rust
/// Tracks nested control flow blocks for label resolution.
#[derive(Debug, Clone)]
pub struct LabelStack {
    /// Stack of active labels with their types
    labels: Vec<LabelInfo>,
}

#[derive(Debug, Clone)]
pub struct LabelInfo {
    /// Optional user-provided label name (for named loops)
    name: Option<String>,
    /// Whether this is a loop (for break/continue semantics)
    is_loop: bool,
    /// Whether break should target this label
    is_breakable: bool,
    /// Result type of the block (None = void)
    result_type: Option<wasm_encoder::ValType>,
}

impl LabelStack {
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    /// Push a new label onto the stack
    pub fn push(&mut self, info: LabelInfo) {
        self.labels.push(info);
    }

    /// Pop a label from the stack
    pub fn pop(&mut self) -> Option<LabelInfo> {
        self.labels.pop()
    }

    /// Get the relative depth for a break (targets end of breakable block)
    pub fn break_depth(&self, name: Option<&str>) -> Option<u32> {
        for (i, label) in self.labels.iter().rev().enumerate() {
            if label.is_breakable {
                if name.is_none() || label.name.as_deref() == name {
                    return Some(i as u32);
                }
            }
        }
        None
    }

    /// Get the relative depth for a continue (targets start of loop)
    pub fn continue_depth(&self, name: Option<&str>) -> Option<u32> {
        for (i, label) in self.labels.iter().rev().enumerate() {
            if label.is_loop {
                if name.is_none() || label.name.as_deref() == name {
                    return Some(i as u32);
                }
            }
        }
        None
    }

    /// Current nesting depth
    pub fn depth(&self) -> usize {
        self.labels.len()
    }
}
```

### Integration with WasmCompiler

```rust
#[cfg(feature = "wasm")]
#[derive(Debug, Clone)]
pub struct WasmCompiler {
    optimize: bool,
    debug_info: bool,
    /// Label stack for control flow tracking
    label_stack: LabelStack,
    /// Local variable indices (name -> index)
    locals: HashMap<String, u32>,
    /// Number of function parameters (locals start after these)
    param_count: u32,
}
```

---

## If/Else Emission

### DOL AST

```rust
Expr::If {
    condition: Box<Expr>,
    then_branch: Box<Expr>,
    else_branch: Option<Box<Expr>>,
}
```

### Design Considerations

1. **Result Type Inference**: WASM `if` can have a result type. We need to determine it.
2. **No Else Case**: If no else branch, the if produces no value (unit type).
3. **Type Consistency**: Both branches must produce the same type.

### WASM Patterns

**If with else (has result):**
```wasm
(if (result i64)
  (local.get $cond)       ;; condition expression
  (then
    (i64.const 1))        ;; then branch
  (else
    (i64.const 0)))       ;; else branch
```

**If without else (no result):**
```wasm
(if
  (local.get $cond)       ;; condition expression
  (then
    (call $side_effect))) ;; then branch only
```

### Implementation

```rust
/// Emit an if/else expression.
///
/// # Arguments
/// * `function` - The WASM function being built
/// * `condition` - The condition expression
/// * `then_branch` - Expression to evaluate if true
/// * `else_branch` - Optional expression to evaluate if false
/// * `context` - Compilation context (func_decl, label_stack, etc.)
fn emit_if_expr(
    &mut self,
    function: &mut wasm_encoder::Function,
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
    context: &mut EmitContext,
) -> Result<Option<ValType>, WasmError> {
    use wasm_encoder::Instruction;

    // 1. Emit the condition (must produce i32)
    self.emit_expression(function, condition, context)?;

    // 2. Determine result type
    let result_type = match else_branch {
        Some(else_expr) => {
            // Both branches exist - infer type from then branch
            // (proper type checking should ensure they match)
            self.infer_expr_type(then_branch, context)?
        }
        None => {
            // No else branch - no result
            None
        }
    };

    // 3. Emit if instruction with appropriate block type
    let block_type = match result_type {
        Some(ValType::I32) => wasm_encoder::BlockType::Result(ValType::I32),
        Some(ValType::I64) => wasm_encoder::BlockType::Result(ValType::I64),
        Some(ValType::F32) => wasm_encoder::BlockType::Result(ValType::F32),
        Some(ValType::F64) => wasm_encoder::BlockType::Result(ValType::F64),
        None => wasm_encoder::BlockType::Empty,
        _ => return Err(WasmError::new("Unsupported result type for if expression")),
    };

    // 4. Push label for potential break
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: false,
        is_breakable: true,
        result_type,
    });

    // 5. Emit the if with branches
    function.instruction(&Instruction::If(block_type));

    // 6. Emit then branch
    self.emit_expression(function, then_branch, context)?;

    // 7. Emit else branch if present
    if let Some(else_expr) = else_branch {
        function.instruction(&Instruction::Else);
        self.emit_expression(function, else_expr, context)?;
    }

    // 8. End the if block
    function.instruction(&Instruction::End);

    // 9. Pop label
    self.label_stack.pop();

    Ok(result_type)
}
```

### Edge Cases

1. **Nested if/else chains**: Each nested if gets its own label depth
2. **If as statement**: When used as `Stmt::Expr(Expr::If {...})`, drop the result
3. **Type mismatch**: Error if then and else branches have different types

---

## Match Expression Emission

### DOL AST

```rust
Expr::Match {
    scrutinee: Box<Expr>,
    arms: Vec<MatchArm>,
}

struct MatchArm {
    pattern: Pattern,
    guard: Option<Box<Expr>>,
    body: Box<Expr>,
}

enum Pattern {
    Wildcard,
    Identifier(String),
    Literal(Literal),
    Constructor { name: String, fields: Vec<Pattern> },
    Tuple(Vec<Pattern>),
    Or(Vec<Pattern>),
}
```

### Design Strategies

1. **Simple Integer Match**: Cascading if/else for small number of arms
2. **Dense Integer Match**: Use `br_table` for efficiency
3. **Pattern Binding**: Store matched value in local, bind identifiers

### Strategy 1: Cascading If/Else (Default)

For small number of arms or non-integer patterns:

```rust
/// Emit match as cascading if/else.
fn emit_match_cascading(
    &mut self,
    function: &mut wasm_encoder::Function,
    scrutinee: &Expr,
    arms: &[MatchArm],
    context: &mut EmitContext,
) -> Result<Option<ValType>, WasmError> {
    use wasm_encoder::Instruction;

    // 1. Evaluate scrutinee and store in local
    self.emit_expression(function, scrutinee, context)?;
    let scrutinee_local = self.allocate_local(ValType::I64); // Assume i64 for now
    function.instruction(&Instruction::LocalSet(scrutinee_local));

    // 2. Determine result type from first arm body
    let result_type = self.infer_expr_type(&arms[0].body, context)?;
    let block_type = self.val_type_to_block_type(result_type);

    // 3. Create outer block for the entire match
    function.instruction(&Instruction::Block(block_type));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: false,
        is_breakable: true,
        result_type,
    });

    // 4. For each arm, generate: check pattern, emit body, br to end
    for (i, arm) in arms.iter().enumerate() {
        let is_last = i == arms.len() - 1;

        // Check if pattern matches
        match &arm.pattern {
            Pattern::Wildcard => {
                // Always matches - just emit body
                self.emit_expression(function, &arm.body, context)?;
                if !is_last {
                    // Branch out of match
                    function.instruction(&Instruction::Br(0));
                }
            }
            Pattern::Literal(lit) => {
                // Compare scrutinee with literal
                function.instruction(&Instruction::LocalGet(scrutinee_local));
                self.emit_literal(function, lit)?;
                function.instruction(&Instruction::I64Eq);

                // Create block for this arm
                function.instruction(&Instruction::If(block_type));
                self.emit_expression(function, &arm.body, context)?;
                function.instruction(&Instruction::Br(1)); // Branch past outer block
                function.instruction(&Instruction::End);
            }
            Pattern::Identifier(name) => {
                // Bind the value to a new local
                let binding_local = self.allocate_local(ValType::I64);
                function.instruction(&Instruction::LocalGet(scrutinee_local));
                function.instruction(&Instruction::LocalSet(binding_local));

                // Add binding to scope
                context.add_local(name.clone(), binding_local);

                self.emit_expression(function, &arm.body, context)?;
                if !is_last {
                    function.instruction(&Instruction::Br(0));
                }

                context.remove_local(name);
            }
            // Handle other patterns...
            _ => return Err(WasmError::new("Unsupported pattern type")),
        }

        // Handle guard if present
        if let Some(guard) = &arm.guard {
            // Wrap in additional check
            // Pattern check AND guard must both pass
        }
    }

    function.instruction(&Instruction::End);
    self.label_stack.pop();

    Ok(result_type)
}
```

### Strategy 2: br_table for Dense Integer Matching

When matching many consecutive integer values:

```rust
/// Emit match using br_table for dense integer patterns.
fn emit_match_br_table(
    &mut self,
    function: &mut wasm_encoder::Function,
    scrutinee: &Expr,
    arms: &[MatchArm],
    context: &mut EmitContext,
) -> Result<Option<ValType>, WasmError> {
    use wasm_encoder::Instruction;

    // Analyze arms to find min/max values and build jump table
    let (min_val, max_val, table_entries) = self.analyze_integer_match(arms)?;

    let result_type = self.infer_expr_type(&arms[0].body, context)?;
    let block_type = self.val_type_to_block_type(result_type);

    // Create nested blocks for each arm + default
    // Structure:
    //   block $exit
    //     block $default
    //       block $case2
    //         block $case1
    //           block $case0
    //             br_table $case0 $case1 $case2 $default (scrutinee - min)
    //           end ;; case0
    //           <arm0 body>
    //           br $exit
    //         end ;; case1
    //         <arm1 body>
    //         br $exit
    //       end ;; case2
    //       <arm2 body>
    //       br $exit
    //     end ;; default
    //     <default body>
    //   end ;; exit

    let num_cases = (max_val - min_val + 1) as usize;

    // Outer exit block
    function.instruction(&Instruction::Block(block_type));

    // Create nested blocks for each case
    for _ in 0..=num_cases {
        function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));
    }

    // Emit scrutinee - min_val (adjust to 0-based index)
    self.emit_expression(function, scrutinee, context)?;
    if min_val != 0 {
        function.instruction(&Instruction::I64Const(min_val));
        function.instruction(&Instruction::I64Sub);
    }
    function.instruction(&Instruction::I32WrapI64); // br_table needs i32

    // Build the label table
    let labels: Vec<u32> = table_entries
        .iter()
        .map(|&case_idx| (num_cases - case_idx) as u32)
        .collect();
    let default_label = num_cases as u32;

    // Emit br_table
    function.instruction(&Instruction::BrTable(
        labels.into(),
        default_label,
    ));

    // Emit each arm body
    for (i, arm) in arms.iter().enumerate() {
        function.instruction(&Instruction::End);
        self.emit_expression(function, &arm.body, context)?;
        function.instruction(&Instruction::Br((num_cases - i) as u32));
    }

    // Emit default (last end)
    function.instruction(&Instruction::End);
    // Default case if not covered by explicit arms
    function.instruction(&Instruction::Unreachable);

    function.instruction(&Instruction::End); // exit block

    Ok(result_type)
}

/// Analyze match arms to determine if br_table is appropriate.
fn analyze_integer_match(&self, arms: &[MatchArm]) -> Result<(i64, i64, Vec<usize>), WasmError> {
    let mut values: Vec<(i64, usize)> = Vec::new();
    let mut has_default = false;

    for (i, arm) in arms.iter().enumerate() {
        match &arm.pattern {
            Pattern::Literal(Literal::Int(n)) => {
                values.push((*n, i));
            }
            Pattern::Wildcard | Pattern::Identifier(_) => {
                has_default = true;
            }
            _ => return Err(WasmError::new("br_table requires integer literals")),
        }
    }

    if values.is_empty() {
        return Err(WasmError::new("No integer patterns for br_table"));
    }

    values.sort_by_key(|(v, _)| *v);
    let min_val = values[0].0;
    let max_val = values[values.len() - 1].0;

    // Check if dense enough (at least 50% coverage)
    let range = max_val - min_val + 1;
    if values.len() < (range as usize / 2) {
        return Err(WasmError::new("Pattern too sparse for br_table"));
    }

    // Build table entries
    let mut table = vec![arms.len() - 1; range as usize]; // default to last arm
    for (val, arm_idx) in values {
        table[(val - min_val) as usize] = arm_idx;
    }

    Ok((min_val, max_val, table))
}
```

---

## Loop Emission

### DOL AST

```rust
// For loop
Stmt::For {
    binding: String,
    iterable: Expr,
    body: Vec<Stmt>,
}

// While loop
Stmt::While {
    condition: Expr,
    body: Vec<Stmt>,
}

// Infinite loop
Stmt::Loop {
    body: Vec<Stmt>,
}
```

### For Loop (Range-based)

DOL `for i in 0..10` translates to WASM:

```wasm
(local $i i64)
(local.set $i (i64.const 0))        ;; i = start
(block $break
  (loop $continue
    ;; Check: i < end
    (br_if $break
      (i64.ge_s (local.get $i) (i64.const 10)))

    ;; Body
    ...

    ;; Increment
    (local.set $i
      (i64.add (local.get $i) (i64.const 1)))

    ;; Continue loop
    (br $continue)
  )
)
```

### Implementation

```rust
/// Emit a for loop over a range.
fn emit_for_stmt(
    &mut self,
    function: &mut wasm_encoder::Function,
    binding: &str,
    iterable: &Expr,
    body: &[Stmt],
    context: &mut EmitContext,
) -> Result<(), WasmError> {
    use wasm_encoder::Instruction;

    // 1. Parse the range expression (assumes Expr::Binary with Range op)
    let (start, end) = match iterable {
        Expr::Binary { left, op: BinaryOp::Range, right } => {
            (left.as_ref(), right.as_ref())
        }
        _ => return Err(WasmError::new("For loops currently only support range iteration")),
    };

    // 2. Allocate loop variable
    let loop_var = self.allocate_local(ValType::I64);
    context.add_local(binding.to_string(), loop_var);

    // 3. Allocate end value local
    let end_var = self.allocate_local(ValType::I64);

    // 4. Initialize: loop_var = start
    self.emit_expression(function, start, context)?;
    function.instruction(&Instruction::LocalSet(loop_var));

    // 5. Store end value
    self.emit_expression(function, end, context)?;
    function.instruction(&Instruction::LocalSet(end_var));

    // 6. Outer block for break
    function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: false,
        is_breakable: true,
        result_type: None,
    });

    // 7. Inner loop for continue
    function.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: true,
        is_breakable: false,
        result_type: None,
    });

    // 8. Check condition: i < end
    function.instruction(&Instruction::LocalGet(loop_var));
    function.instruction(&Instruction::LocalGet(end_var));
    function.instruction(&Instruction::I64GeS); // i >= end
    function.instruction(&Instruction::BrIf(1)); // break if true

    // 9. Emit body
    for stmt in body {
        self.emit_statement(function, stmt, context)?;
    }

    // 10. Increment loop variable
    function.instruction(&Instruction::LocalGet(loop_var));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(loop_var));

    // 11. Continue loop
    function.instruction(&Instruction::Br(0));

    // 12. End loop and block
    function.instruction(&Instruction::End); // loop
    self.label_stack.pop();
    function.instruction(&Instruction::End); // block
    self.label_stack.pop();

    // 13. Clean up scope
    context.remove_local(binding);

    Ok(())
}
```

### While Loop

```rust
/// Emit a while loop.
fn emit_while_stmt(
    &mut self,
    function: &mut wasm_encoder::Function,
    condition: &Expr,
    body: &[Stmt],
    context: &mut EmitContext,
) -> Result<(), WasmError> {
    use wasm_encoder::Instruction;

    // Structure:
    // block $break
    //   loop $continue
    //     br_if $break (not condition)
    //     <body>
    //     br $continue
    //   end
    // end

    // 1. Outer block for break
    function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: false,
        is_breakable: true,
        result_type: None,
    });

    // 2. Inner loop for continue
    function.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: true,
        is_breakable: false,
        result_type: None,
    });

    // 3. Check condition (negate for early exit)
    self.emit_expression(function, condition, context)?;
    function.instruction(&Instruction::I32Eqz); // NOT condition
    function.instruction(&Instruction::BrIf(1)); // break if condition is false

    // 4. Emit body
    for stmt in body {
        self.emit_statement(function, stmt, context)?;
    }

    // 5. Continue loop
    function.instruction(&Instruction::Br(0));

    // 6. End loop and block
    function.instruction(&Instruction::End); // loop
    self.label_stack.pop();
    function.instruction(&Instruction::End); // block
    self.label_stack.pop();

    Ok(())
}
```

### Infinite Loop

```rust
/// Emit an infinite loop.
fn emit_loop_stmt(
    &mut self,
    function: &mut wasm_encoder::Function,
    body: &[Stmt],
    context: &mut EmitContext,
) -> Result<(), WasmError> {
    use wasm_encoder::Instruction;

    // Structure:
    // block $break
    //   loop $continue
    //     <body>
    //     br $continue
    //   end
    // end

    // 1. Outer block for break
    function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: false,
        is_breakable: true,
        result_type: None,
    });

    // 2. Loop
    function.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: true,
        is_breakable: false,
        result_type: None,
    });

    // 3. Emit body
    for stmt in body {
        self.emit_statement(function, stmt, context)?;
    }

    // 4. Continue loop unconditionally
    function.instruction(&Instruction::Br(0));

    // 5. End loop and block
    function.instruction(&Instruction::End); // loop
    self.label_stack.pop();
    function.instruction(&Instruction::End); // block
    self.label_stack.pop();

    Ok(())
}
```

---

## Block Expression Emission

### DOL AST

```rust
Expr::Block {
    statements: Vec<Stmt>,
    final_expr: Option<Box<Expr>>,
}
```

### Implementation

```rust
/// Emit a block expression.
fn emit_block_expr(
    &mut self,
    function: &mut wasm_encoder::Function,
    statements: &[Stmt],
    final_expr: Option<&Expr>,
    context: &mut EmitContext,
) -> Result<Option<ValType>, WasmError> {
    use wasm_encoder::Instruction;

    // 1. Determine result type
    let result_type = match final_expr {
        Some(expr) => self.infer_expr_type(expr, context)?,
        None => None,
    };
    let block_type = self.val_type_to_block_type(result_type);

    // 2. Create WASM block
    function.instruction(&Instruction::Block(block_type));
    self.label_stack.push(LabelInfo {
        name: None,
        is_loop: false,
        is_breakable: true,
        result_type,
    });

    // 3. Emit statements
    for stmt in statements {
        self.emit_statement(function, stmt, context)?;
    }

    // 4. Emit final expression if present
    if let Some(expr) = final_expr {
        self.emit_expression(function, expr, context)?;
    }

    // 5. End block
    function.instruction(&Instruction::End);
    self.label_stack.pop();

    Ok(result_type)
}
```

---

## Break and Continue

### Implementation

```rust
/// Emit a break statement.
fn emit_break(
    &mut self,
    function: &mut wasm_encoder::Function,
    _context: &mut EmitContext,
) -> Result<(), WasmError> {
    use wasm_encoder::Instruction;

    // Find the depth to the nearest breakable block
    match self.label_stack.break_depth(None) {
        Some(depth) => {
            function.instruction(&Instruction::Br(depth));
            Ok(())
        }
        None => Err(WasmError::new("Break statement outside of loop or block")),
    }
}

/// Emit a continue statement.
fn emit_continue(
    &mut self,
    function: &mut wasm_encoder::Function,
    _context: &mut EmitContext,
) -> Result<(), WasmError> {
    use wasm_encoder::Instruction;

    // Find the depth to the nearest loop
    match self.label_stack.continue_depth(None) {
        Some(depth) => {
            function.instruction(&Instruction::Br(depth));
            Ok(())
        }
        None => Err(WasmError::new("Continue statement outside of loop")),
    }
}
```

---

## Edge Cases and Error Handling

### Type Mismatches

```rust
/// Verify that if/else branches have matching types.
fn check_branch_types(
    &self,
    then_type: Option<ValType>,
    else_type: Option<ValType>,
) -> Result<(), WasmError> {
    match (then_type, else_type) {
        (None, None) => Ok(()),
        (Some(t1), Some(t2)) if t1 == t2 => Ok(()),
        (Some(t1), Some(t2)) => Err(WasmError::new(format!(
            "If branches have mismatched types: {:?} vs {:?}",
            t1, t2
        ))),
        (Some(_), None) | (None, Some(_)) => Err(WasmError::new(
            "If with else must have matching branch types",
        )),
    }
}
```

### Unreachable Code

Handle `break` and `continue` making subsequent code unreachable:

```rust
/// Check if a statement unconditionally exits.
fn is_terminator(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) => true,
        Stmt::Break => true,
        Stmt::Continue => true,
        _ => false,
    }
}
```

### Nested Control Flow

Example of deeply nested control flow:

```dol
fun complex() -> i64 {
    for i in 0..10 {
        if i == 5 {
            while true {
                if some_condition() {
                    break  // breaks inner while
                }
            }
            continue  // continues for loop
        }
    }
    return 0
}
```

The label stack correctly handles this:
- `break` in the `if` finds depth=1 (skipping the `if` block to reach `while`'s break target)
- `continue` finds depth=3 (skipping `if`, `while`'s loop, `while`'s block, to reach `for`'s loop)

### Pattern Exhaustiveness

Match expressions should be exhaustive:

```rust
/// Check if match patterns cover all cases.
fn check_exhaustiveness(&self, scrutinee_type: &TypeExpr, arms: &[MatchArm]) -> Result<(), WasmError> {
    let has_wildcard = arms.iter().any(|arm| {
        matches!(arm.pattern, Pattern::Wildcard | Pattern::Identifier(_))
    });

    if !has_wildcard {
        // For now, require a wildcard or identifier catch-all
        // Future: actual exhaustiveness checking
        return Err(WasmError::new(
            "Match expression must have a wildcard (_) or identifier catch-all pattern",
        ));
    }

    Ok(())
}
```

---

## Implementation Plan

### Phase 1: Foundation (Week 1)

1. **Add LabelStack to WasmCompiler**
   - Implement `LabelStack` struct
   - Add to compiler state
   - Add helper methods for depth calculation

2. **Add Local Variable Management**
   - Implement `locals: HashMap<String, u32>`
   - Add `allocate_local()` method
   - Track locals count for function locals section

3. **Implement EmitContext**
   - Scope management for local variables
   - Pass through all emit methods

### Phase 2: Basic Control Flow (Week 2)

4. **Implement Block Expressions**
   - `emit_block_expr()`
   - Handle statements and final expression

5. **Implement If/Else**
   - `emit_if_expr()`
   - Type inference for result type
   - Handle if-without-else

6. **Implement Break/Continue**
   - `emit_break()`
   - `emit_continue()`
   - Label depth resolution

### Phase 3: Loops (Week 3)

7. **Implement While Loop**
   - `emit_while_stmt()`
   - Condition inversion pattern

8. **Implement Infinite Loop**
   - `emit_loop_stmt()`

9. **Implement For Loop**
   - `emit_for_stmt()`
   - Range parsing
   - Counter management

### Phase 4: Pattern Matching (Week 4)

10. **Implement Basic Match**
    - `emit_match_cascading()`
    - Literal patterns
    - Wildcard pattern
    - Identifier binding

11. **Implement br_table Optimization**
    - `emit_match_br_table()`
    - Pattern analysis
    - Jump table generation

12. **Add Guard Support**
    - Guards on match arms
    - Combined pattern + guard checking

### Phase 5: Testing and Polish (Week 5)

13. **Unit Tests**
    - Test each control flow construct
    - Test nested control flow
    - Test edge cases

14. **Integration Tests**
    - Compile and execute test functions
    - Verify correct behavior in wasmtime

15. **Documentation**
    - Update module documentation
    - Add examples

---

## Appendix: Complete Updated emit_statement

```rust
fn emit_statement(
    &mut self,
    function: &mut wasm_encoder::Function,
    stmt: &Stmt,
    context: &mut EmitContext,
) -> Result<(), WasmError> {
    use wasm_encoder::Instruction;

    match stmt {
        Stmt::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                self.emit_expression(function, expr, context)?;
            }
            function.instruction(&Instruction::Return);
        }
        Stmt::Expr(expr) => {
            let result_type = self.emit_expression(function, expr, context)?;
            // Drop the result if the expression produces a value
            if result_type.is_some() {
                function.instruction(&Instruction::Drop);
            }
        }
        Stmt::Let { name, type_ann, value } => {
            // Allocate local
            let val_type = self.type_expr_to_val_type(type_ann.as_ref())?;
            let local_idx = self.allocate_local(val_type);
            context.add_local(name.clone(), local_idx);

            // Emit value and store
            self.emit_expression(function, value, context)?;
            function.instruction(&Instruction::LocalSet(local_idx));
        }
        Stmt::Assign { target, value } => {
            // Handle simple variable assignment
            if let Expr::Identifier(name) = target {
                let local_idx = context.get_local(name)
                    .ok_or_else(|| WasmError::new(format!("Unknown variable: {}", name)))?;
                self.emit_expression(function, value, context)?;
                function.instruction(&Instruction::LocalSet(local_idx));
            } else {
                return Err(WasmError::new("Complex assignment targets not yet supported"));
            }
        }
        Stmt::For { binding, iterable, body } => {
            self.emit_for_stmt(function, binding, iterable, body, context)?;
        }
        Stmt::While { condition, body } => {
            self.emit_while_stmt(function, condition, body, context)?;
        }
        Stmt::Loop { body } => {
            self.emit_loop_stmt(function, body, context)?;
        }
        Stmt::Break => {
            self.emit_break(function, context)?;
        }
        Stmt::Continue => {
            self.emit_continue(function, context)?;
        }
    }

    Ok(())
}
```

---

## References

- [WebAssembly Specification - Control Instructions](https://webassembly.github.io/spec/core/syntax/instructions.html#control-instructions)
- [wasm-encoder crate documentation](https://docs.rs/wasm-encoder)
- [DOL AST definitions](src/ast.rs)
- [Current WASM compiler](src/wasm/compiler.rs)
