# DOL WASM Backend Implementation Plan

**Version**: 1.0
**Date**: 2025-12-31
**Author**: WASM Backend Architect Agent

---

## Table of Contents

1. [Current Architecture Summary](#1-current-architecture-summary)
2. [Extension Points Identified](#2-extension-points-identified)
3. [Proposed Changes](#3-proposed-changes)
   - [Phase 1: Local Variables](#phase-1-local-variables)
   - [Phase 2: Control Flow](#phase-2-control-flow)
   - [Phase 3: Genes (Structs)](#phase-3-genes-structs)
   - [Phase 4: Advanced Features](#phase-4-advanced-features)
4. [Implementation Order](#4-implementation-order)
5. [Testing Strategy](#5-testing-strategy)
6. [Appendix: WASM Instruction Reference](#appendix-wasm-instruction-reference)

---

## 1. Current Architecture Summary

### Files Structure

```
src/wasm/
├── mod.rs           # Module exports, WasmError type
├── compiler.rs      # DOL AST → WASM bytecode emission
└── runtime.rs       # Wasmtime-based execution engine
```

### Current Capabilities

The `WasmCompiler` in `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` currently supports:

| Feature | Status | Implementation |
|---------|--------|----------------|
| Function declarations | Working | `compile()`, lines 176-241 |
| Parameter access | Working | `LocalGet` in `emit_expression()`, lines 406-413 |
| Integer literals (i64) | Working | `I64Const`, line 382 |
| Float literals (f64) | Working | `F64Const`, line 385 |
| Boolean literals | Working | `I32Const`, lines 388-389 |
| Binary operations | Working | `emit_binary_op()`, lines 520-592 |
| Return statements | Working | `emit_statement()`, lines 333-338 |
| Function calls | Partial | Only direct calls, hardcoded index 0, lines 423-437 |

### Current Limitations

All these constructs return `WasmError`:

| Feature | Error Location | Required For |
|---------|---------------|--------------|
| Let bindings | Line 344-348 | Local variables |
| Assignments | Line 349-353 | Mutable state |
| For/While/Loop | Lines 354-358 | Iteration |
| Break/Continue | Lines 359-363 | Loop control |
| If expressions | Lines 439-443 | Conditionals |
| Block expressions | Lines 444-448 | Scoping |
| Match expressions | Lines 449-453 | Pattern matching |
| Lambda expressions | Lines 454-458 | Higher-order functions |
| Member access | Lines 459-463 | Gene field access |
| Unary operators | Lines 464-468 | Negation, boolean not |
| List/Tuple literals | Lines 469-473 | Compound data |
| String/Char literals | Lines 391-404 | Text handling |
| Gene declarations | Line 254-258 | Struct compilation |

### Module Layout Generated

Current WASM module structure:
```
[Type Section]     → Function signatures (params → results)
[Function Section] → Type indices for each function
[Export Section]   → All functions exported by name
[Code Section]     → Function bodies with locals=[]
```

**Key observation**: The Code Section currently emits `Function::new(vec![])` with an empty locals vector (line 234). This is the primary extension point for local variables.

---

## 2. Extension Points Identified

### 2.1 Compiler State (New Structure Needed)

**Location**: `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

Currently, compilation is stateless per-function. We need:

```rust
// NEW: Add after line 66
/// Compilation context for a single function
struct FunctionContext<'a> {
    /// The function being compiled
    func_decl: &'a FunctionDecl,

    /// Maps variable names to local indices
    locals: LocalsTable,

    /// Type of each local variable for instruction selection
    local_types: Vec<ValType>,

    /// Current control flow block depth (for break/continue)
    block_depth: u32,

    /// Label stack for break targets
    break_labels: Vec<u32>,

    /// Label stack for continue targets
    continue_labels: Vec<u32>,
}

/// Tracks local variable allocation
struct LocalsTable {
    /// Parameter count (parameters are locals 0..n-1)
    param_count: u32,

    /// Maps variable names to local indices
    name_to_index: HashMap<String, u32>,

    /// Next available local index
    next_index: u32,
}
```

### 2.2 Memory Management (New Structure Needed)

For genes/structs, we need linear memory:

```rust
// NEW: Add to WasmCompiler or separate module
struct MemoryLayout {
    /// Base pointer for heap allocations
    heap_base: u32,

    /// Current allocation pointer
    heap_ptr: u32,

    /// Gene type layouts: name → (size, field_offsets)
    gene_layouts: HashMap<String, GeneLayout>,
}

struct GeneLayout {
    /// Total size in bytes (aligned)
    size: u32,

    /// Field name → (offset, type)
    fields: HashMap<String, (u32, ValType)>,
}
```

### 2.3 Symbol Table (New Structure Needed)

For function calls:

```rust
// NEW: Module-level context
struct ModuleContext {
    /// Function name → function index
    function_indices: HashMap<String, u32>,

    /// Gene name → type index (for gene allocation)
    gene_type_indices: HashMap<String, u32>,
}
```

### 2.4 Extension Points in Existing Methods

| Method | Line | Extension Point |
|--------|------|-----------------|
| `emit_function_body()` | 304-320 | Add `FunctionContext` setup |
| `emit_statement()` | 323-367 | Add cases for Let, Assign, loops |
| `emit_expression()` | 369-517 | Add cases for If, Block, Match |
| `compile()` | 176-241 | Add Memory section, Gene handling |
| `extract_functions()` | 246-260 | Extract functions from Genes/Traits |

---

## 3. Proposed Changes

### Phase 1: Local Variables

**Goal**: Support `let` bindings and variable assignment within functions.

#### 3.1.1 Add LocalsTable

**File**: `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

```rust
// Add after line 44 (imports)
use std::collections::HashMap;

// Add after line 66 (after WasmCompiler struct)
/// Tracks local variable indices during compilation
#[derive(Debug, Default)]
struct LocalsTable {
    /// Number of function parameters (locals 0..param_count-1)
    param_count: u32,
    /// Maps variable names to local indices
    name_to_index: HashMap<String, u32>,
    /// Types of declared locals (not including params)
    local_types: Vec<wasm_encoder::ValType>,
}

impl LocalsTable {
    fn new(params: &[FunctionParam]) -> Result<Self, WasmError> {
        let mut table = Self {
            param_count: params.len() as u32,
            name_to_index: HashMap::new(),
            local_types: Vec::new(),
        };

        // Register parameters as locals 0..n-1
        for (i, param) in params.iter().enumerate() {
            table.name_to_index.insert(param.name.clone(), i as u32);
        }

        Ok(table)
    }

    /// Declare a new local variable, returns its index
    fn declare(&mut self, name: &str, val_type: wasm_encoder::ValType) -> u32 {
        let index = self.param_count + self.local_types.len() as u32;
        self.name_to_index.insert(name.to_string(), index);
        self.local_types.push(val_type);
        index
    }

    /// Look up a variable by name
    fn lookup(&self, name: &str) -> Option<u32> {
        self.name_to_index.get(name).copied()
    }

    /// Get the declared locals for WASM code section
    fn get_locals(&self) -> Vec<(u32, wasm_encoder::ValType)> {
        // Group consecutive locals of the same type
        let mut result = Vec::new();
        for ty in &self.local_types {
            if let Some((count, last_ty)) = result.last_mut() {
                if last_ty == ty {
                    *count += 1;
                    continue;
                }
            }
            result.push((1, *ty));
        }
        result
    }
}
```

#### 3.1.2 Modify emit_function_body()

**Current** (lines 304-320):
```rust
fn emit_function_body(
    &self,
    function: &mut wasm_encoder::Function,
    func_decl: &crate::ast::FunctionDecl,
) -> Result<(), WasmError> {
```

**Proposed change**:
```rust
fn emit_function_body(
    &self,
    func_decl: &crate::ast::FunctionDecl,
) -> Result<(Vec<(u32, wasm_encoder::ValType)>, Vec<wasm_encoder::Instruction<'static>>), WasmError> {
    use wasm_encoder::Instruction;

    // Create locals table from parameters
    let mut locals = LocalsTable::new(&func_decl.params)?;
    let mut instructions = Vec::new();

    // First pass: collect all let bindings to declare locals
    self.collect_locals(&func_decl.body, &mut locals)?;

    // Second pass: emit instructions
    for stmt in &func_decl.body {
        self.emit_statement(&mut instructions, stmt, &mut locals)?;
    }

    instructions.push(Instruction::End);

    Ok((locals.get_locals(), instructions))
}
```

#### 3.1.3 Add collect_locals method

```rust
/// Collect all let bindings to pre-declare locals
fn collect_locals(
    &self,
    stmts: &[crate::ast::Stmt],
    locals: &mut LocalsTable,
) -> Result<(), WasmError> {
    use crate::ast::Stmt;

    for stmt in stmts {
        match stmt {
            Stmt::Let { name, type_ann, .. } => {
                let val_type = if let Some(ty) = type_ann {
                    self.dol_type_to_wasm(ty)?
                } else {
                    // Default to i64 for untyped locals
                    wasm_encoder::ValType::I64
                };
                locals.declare(name, val_type);
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } | Stmt::Loop { body, .. } => {
                self.collect_locals(body, locals)?;
            }
            _ => {}
        }
    }
    Ok(())
}
```

#### 3.1.4 Implement Let statement

**Current** (lines 344-348):
```rust
Stmt::Let { .. } => {
    return Err(WasmError::new(
        "Let bindings not yet supported in WASM compilation",
    ))
}
```

**Proposed**:
```rust
Stmt::Let { name, value, .. } => {
    // Emit the value expression
    self.emit_expression(instructions, value, locals)?;

    // Store in local variable
    let local_idx = locals.lookup(name)
        .ok_or_else(|| WasmError::new(format!("Undeclared variable: {}", name)))?;
    instructions.push(Instruction::LocalSet(local_idx));
}
```

#### 3.1.5 Implement Assignment

**Current** (lines 349-353):
```rust
Stmt::Assign { .. } => {
    return Err(WasmError::new(
        "Assignments not yet supported in WASM compilation",
    ))
}
```

**Proposed**:
```rust
Stmt::Assign { target, value } => {
    match target {
        Expr::Identifier(name) => {
            // Emit the value
            self.emit_expression(instructions, value, locals)?;

            // Store in local
            let local_idx = locals.lookup(name)
                .ok_or_else(|| WasmError::new(format!("Undeclared variable: {}", name)))?;
            instructions.push(Instruction::LocalSet(local_idx));
        }
        Expr::Member { object, field } => {
            // Gene field assignment - requires memory (Phase 3)
            return Err(WasmError::new(
                "Field assignment requires gene support (coming in Phase 3)",
            ));
        }
        _ => {
            return Err(WasmError::new(
                "Unsupported assignment target",
            ));
        }
    }
}
```

#### 3.1.6 Update Identifier lookup

**Current** (lines 406-413):
```rust
Expr::Identifier(name) => {
    let param_idx = func_decl
        .params
        .iter()
        .position(|p| p.name == *name)
        .ok_or_else(|| WasmError::new(format!("Unknown identifier: {}", name)))?;
    function.instruction(&Instruction::LocalGet(param_idx as u32));
}
```

**Proposed**:
```rust
Expr::Identifier(name) => {
    let local_idx = locals.lookup(name)
        .ok_or_else(|| WasmError::new(format!("Unknown identifier: {}", name)))?;
    instructions.push(Instruction::LocalGet(local_idx));
}
```

---

### Phase 2: Control Flow

**Goal**: Support if/else expressions, match expressions, and loops.

#### 3.2.1 If Expressions

WASM structured control flow:
```wasm
(if (result i64)
  (then ... produces value ...)
  (else ... produces value ...))
```

**Implementation** (replace lines 439-443):

```rust
Expr::If { condition, then_branch, else_branch } => {
    // Emit condition
    self.emit_expression(instructions, condition, locals)?;

    // Determine result type (if both branches return a value)
    let block_type = if else_branch.is_some() {
        // For now, assume i64 result type
        wasm_encoder::BlockType::Result(wasm_encoder::ValType::I64)
    } else {
        wasm_encoder::BlockType::Empty
    };

    instructions.push(Instruction::If(block_type));

    // Then branch
    self.emit_expression(instructions, then_branch, locals)?;

    // Else branch
    if let Some(else_expr) = else_branch {
        instructions.push(Instruction::Else);
        self.emit_expression(instructions, else_expr, locals)?;
    }

    instructions.push(Instruction::End);
}
```

#### 3.2.2 Block Expressions

```rust
Expr::Block { statements, final_expr } => {
    // Create a block for scoping
    let block_type = if final_expr.is_some() {
        wasm_encoder::BlockType::Result(wasm_encoder::ValType::I64)
    } else {
        wasm_encoder::BlockType::Empty
    };

    instructions.push(Instruction::Block(block_type));

    // Emit statements
    for stmt in statements {
        self.emit_statement(instructions, stmt, locals)?;
    }

    // Emit final expression if present
    if let Some(expr) = final_expr {
        self.emit_expression(instructions, expr, locals)?;
    }

    instructions.push(Instruction::End);
}
```

#### 3.2.3 Match Expressions

Match expressions compile to a series of if-else chains or br_table for dense integer patterns.

```rust
Expr::Match { scrutinee, arms } => {
    // Emit scrutinee and store in temporary
    self.emit_expression(instructions, scrutinee, locals)?;
    let scrutinee_local = locals.declare("__match_scrutinee", wasm_encoder::ValType::I64);
    instructions.push(Instruction::LocalSet(scrutinee_local));

    // Outer block for match result
    instructions.push(Instruction::Block(wasm_encoder::BlockType::Result(wasm_encoder::ValType::I64)));

    // Compile each arm as an if-else chain
    for (i, arm) in arms.iter().enumerate() {
        let is_last = i == arms.len() - 1;

        match &arm.pattern {
            Pattern::Wildcard => {
                // Default case - just emit body
                self.emit_expression(instructions, &arm.body, locals)?;
            }
            Pattern::Literal(Literal::Int(n)) => {
                // Compare with literal
                instructions.push(Instruction::LocalGet(scrutinee_local));
                instructions.push(Instruction::I64Const(*n));
                instructions.push(Instruction::I64Eq);

                instructions.push(Instruction::If(wasm_encoder::BlockType::Result(wasm_encoder::ValType::I64)));
                self.emit_expression(instructions, &arm.body, locals)?;

                if !is_last {
                    instructions.push(Instruction::Else);
                    // Next arm continues here
                }
            }
            Pattern::Identifier(name) => {
                // Bind scrutinee to variable
                let binding_local = locals.declare(name, wasm_encoder::ValType::I64);
                instructions.push(Instruction::LocalGet(scrutinee_local));
                instructions.push(Instruction::LocalSet(binding_local));
                self.emit_expression(instructions, &arm.body, locals)?;
            }
            _ => {
                return Err(WasmError::new(format!(
                    "Unsupported pattern in match: {:?}", arm.pattern
                )));
            }
        }
    }

    // Close all if blocks
    for _ in 0..arms.len().saturating_sub(1) {
        instructions.push(Instruction::End);
    }

    instructions.push(Instruction::End); // Close outer block
}
```

#### 3.2.4 While Loop

```rust
Stmt::While { condition, body } => {
    // Outer block for break target
    instructions.push(Instruction::Block(wasm_encoder::BlockType::Empty));

    // Inner loop for continue target
    instructions.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

    // Evaluate condition
    self.emit_expression(instructions, condition, locals)?;

    // Branch out if condition is false (break out of block)
    instructions.push(Instruction::I32Eqz);
    instructions.push(Instruction::BrIf(1)); // Break to outer block

    // Loop body
    for stmt in body {
        self.emit_statement(instructions, stmt, locals)?;
    }

    // Continue (branch back to loop start)
    instructions.push(Instruction::Br(0));

    instructions.push(Instruction::End); // End loop
    instructions.push(Instruction::End); // End block
}
```

#### 3.2.5 For Loop

For loops require iterator protocol. Simplify to while loop pattern:

```rust
Stmt::For { binding, iterable, body } => {
    // For now, only support range iteration: for i in 0..n
    match iterable {
        Expr::Binary { left, op: BinaryOp::Range, right } => {
            // Declare loop variable
            let loop_var = locals.declare(binding, wasm_encoder::ValType::I64);

            // Initialize loop variable with start value
            self.emit_expression(instructions, left, locals)?;
            instructions.push(Instruction::LocalSet(loop_var));

            // Emit end value and store
            let end_var = locals.declare("__for_end", wasm_encoder::ValType::I64);
            self.emit_expression(instructions, right, locals)?;
            instructions.push(Instruction::LocalSet(end_var));

            // Outer block for break
            instructions.push(Instruction::Block(wasm_encoder::BlockType::Empty));

            // Loop
            instructions.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

            // Check condition: loop_var < end_var
            instructions.push(Instruction::LocalGet(loop_var));
            instructions.push(Instruction::LocalGet(end_var));
            instructions.push(Instruction::I64LtS);
            instructions.push(Instruction::I32Eqz);
            instructions.push(Instruction::BrIf(1)); // Break if not less

            // Body
            for stmt in body {
                self.emit_statement(instructions, stmt, locals)?;
            }

            // Increment loop variable
            instructions.push(Instruction::LocalGet(loop_var));
            instructions.push(Instruction::I64Const(1));
            instructions.push(Instruction::I64Add);
            instructions.push(Instruction::LocalSet(loop_var));

            // Continue
            instructions.push(Instruction::Br(0));

            instructions.push(Instruction::End); // End loop
            instructions.push(Instruction::End); // End block
        }
        _ => {
            return Err(WasmError::new(
                "For loops currently only support range iteration (0..n)",
            ));
        }
    }
}
```

#### 3.2.6 Break/Continue with Block Tracking

Add to `FunctionContext`:

```rust
struct LoopContext {
    /// Block depth for break (outer block)
    break_depth: u32,
    /// Block depth for continue (loop block)
    continue_depth: u32,
}
```

```rust
Stmt::Break => {
    if let Some(loop_ctx) = &current_loop {
        instructions.push(Instruction::Br(loop_ctx.break_depth));
    } else {
        return Err(WasmError::new("Break outside of loop"));
    }
}

Stmt::Continue => {
    if let Some(loop_ctx) = &current_loop {
        instructions.push(Instruction::Br(loop_ctx.continue_depth));
    } else {
        return Err(WasmError::new("Continue outside of loop"));
    }
}
```

---

### Phase 3: Genes (Structs)

**Goal**: Compile gene declarations to WASM linear memory layouts.

#### 3.3.1 Memory Section

Add to `compile()` method after function section:

```rust
// Memory section: 1 page (64KB) minimum
let mut memory_section = MemorySection::new();
memory_section.memory(MemoryType {
    minimum: 1,
    maximum: None,
    memory64: false,
    shared: false,
});
wasm_module.section(&memory_section);

// Export memory
exports.export("memory", ExportKind::Memory, 0);
```

#### 3.3.2 Gene Layout Calculation

```rust
struct GeneLayout {
    /// Total size in bytes (8-byte aligned)
    size: u32,
    /// Field name → (offset, WASM type, byte size)
    fields: Vec<(String, u32, wasm_encoder::ValType, u32)>,
}

impl GeneLayout {
    fn from_gene(gene: &Gene, compiler: &WasmCompiler) -> Result<Self, WasmError> {
        let mut offset = 0u32;
        let mut fields = Vec::new();

        for stmt in &gene.statements {
            if let Statement::HasField(field) = stmt {
                let val_type = compiler.dol_type_to_wasm(&field.type_)?;
                let size = match val_type {
                    ValType::I32 => 4,
                    ValType::I64 => 8,
                    ValType::F32 => 4,
                    ValType::F64 => 8,
                    _ => return Err(WasmError::new("Unsupported field type")),
                };

                // Align offset
                let align = size;
                offset = (offset + align - 1) & !(align - 1);

                fields.push((field.name.clone(), offset, val_type, size));
                offset += size;
            }
        }

        // Final alignment to 8 bytes
        let size = (offset + 7) & !7;

        Ok(Self { size, fields })
    }

    fn get_field(&self, name: &str) -> Option<(u32, wasm_encoder::ValType)> {
        self.fields.iter()
            .find(|(n, _, _, _)| n == name)
            .map(|(_, offset, ty, _)| (*offset, *ty))
    }
}
```

#### 3.3.3 Gene Allocation Function

Generate a `__alloc_GeneName` function for each gene:

```rust
fn emit_gene_allocator(
    &self,
    gene: &Gene,
    layout: &GeneLayout,
) -> Result<(FunctionType, Vec<Instruction<'static>>), WasmError> {
    // Type: () -> i32 (returns pointer)
    let func_type = FunctionType::new(vec![], vec![ValType::I32]);

    let mut instructions = Vec::new();

    // Get current heap pointer from global
    instructions.push(Instruction::GlobalGet(0)); // __heap_ptr global

    // Save pointer for return
    instructions.push(Instruction::LocalTee(0));

    // Bump heap pointer by size
    instructions.push(Instruction::GlobalGet(0));
    instructions.push(Instruction::I32Const(layout.size as i32));
    instructions.push(Instruction::I32Add);
    instructions.push(Instruction::GlobalSet(0));

    // Initialize fields to default values (0)
    for (_, offset, ty, size) in &layout.fields {
        instructions.push(Instruction::LocalGet(0)); // Base pointer
        match ty {
            ValType::I32 => {
                instructions.push(Instruction::I32Const(0));
                instructions.push(Instruction::I32Store(MemArg {
                    offset: *offset as u64,
                    align: 2,
                    memory_index: 0,
                }));
            }
            ValType::I64 => {
                instructions.push(Instruction::I64Const(0));
                instructions.push(Instruction::I64Store(MemArg {
                    offset: *offset as u64,
                    align: 3,
                    memory_index: 0,
                }));
            }
            ValType::F32 => {
                instructions.push(Instruction::F32Const(0.0));
                instructions.push(Instruction::F32Store(MemArg {
                    offset: *offset as u64,
                    align: 2,
                    memory_index: 0,
                }));
            }
            ValType::F64 => {
                instructions.push(Instruction::F64Const(0.0));
                instructions.push(Instruction::F64Store(MemArg {
                    offset: *offset as u64,
                    align: 3,
                    memory_index: 0,
                }));
            }
            _ => {}
        }
    }

    // Return pointer
    instructions.push(Instruction::LocalGet(0));
    instructions.push(Instruction::End);

    Ok((func_type, instructions))
}
```

#### 3.3.4 Member Access (Field Read)

```rust
Expr::Member { object, field } => {
    // Emit object expression (should produce pointer)
    self.emit_expression(instructions, object, locals)?;

    // Look up gene type from context
    let gene_layout = self.get_gene_layout_for_expr(object)?;

    let (offset, val_type) = gene_layout.get_field(field)
        .ok_or_else(|| WasmError::new(format!("Unknown field: {}", field)))?;

    // Load from memory
    match val_type {
        ValType::I32 => {
            instructions.push(Instruction::I32Load(MemArg {
                offset: offset as u64,
                align: 2,
                memory_index: 0,
            }));
        }
        ValType::I64 => {
            instructions.push(Instruction::I64Load(MemArg {
                offset: offset as u64,
                align: 3,
                memory_index: 0,
            }));
        }
        ValType::F32 => {
            instructions.push(Instruction::F32Load(MemArg {
                offset: offset as u64,
                align: 2,
                memory_index: 0,
            }));
        }
        ValType::F64 => {
            instructions.push(Instruction::F64Load(MemArg {
                offset: offset as u64,
                align: 3,
                memory_index: 0,
            }));
        }
        _ => return Err(WasmError::new("Unsupported field type")),
    }
}
```

#### 3.3.5 Field Assignment (Field Write)

In `emit_statement` for `Stmt::Assign` with `Expr::Member` target:

```rust
Expr::Member { object, field } => {
    // Emit object expression (pointer)
    self.emit_expression(instructions, object, locals)?;

    // Emit value
    self.emit_expression(instructions, value, locals)?;

    // Look up field
    let gene_layout = self.get_gene_layout_for_expr(object)?;
    let (offset, val_type) = gene_layout.get_field(field)
        .ok_or_else(|| WasmError::new(format!("Unknown field: {}", field)))?;

    // Store to memory
    match val_type {
        ValType::I32 => {
            instructions.push(Instruction::I32Store(MemArg {
                offset: offset as u64,
                align: 2,
                memory_index: 0,
            }));
        }
        ValType::I64 => {
            instructions.push(Instruction::I64Store(MemArg {
                offset: offset as u64,
                align: 3,
                memory_index: 0,
            }));
        }
        // ... similar for F32, F64
    }
}
```

#### 3.3.6 Globals Section

For heap management:

```rust
// Global section: heap pointer
let mut globals = GlobalSection::new();
globals.global(
    GlobalType {
        val_type: ValType::I32,
        mutable: true,
    },
    &Instruction::I32Const(1024), // Start heap at 1KB offset
);
wasm_module.section(&globals);
```

---

### Phase 4: Advanced Features

#### 3.4.1 Unary Operators

```rust
Expr::Unary { op, operand } => {
    self.emit_expression(instructions, operand, locals)?;

    match op {
        UnaryOp::Neg => {
            // 0 - value
            instructions.push(Instruction::I64Const(0));
            self.emit_expression(instructions, operand, locals)?;
            instructions.push(Instruction::I64Sub);
        }
        UnaryOp::Not => {
            instructions.push(Instruction::I32Eqz);
        }
        _ => return Err(WasmError::new(format!("Unsupported unary op: {:?}", op))),
    }
}
```

#### 3.4.2 Function Call Improvements

Add `ModuleContext` to track function indices:

```rust
fn build_function_index_map(
    declarations: &[Declaration],
) -> HashMap<String, u32> {
    let mut index = 0u32;
    let mut map = HashMap::new();

    for decl in declarations {
        match decl {
            Declaration::Function(f) => {
                map.insert(f.name.clone(), index);
                index += 1;
            }
            Declaration::Gene(g) => {
                // Allocator function
                map.insert(format!("__alloc_{}", g.name), index);
                index += 1;

                // Gene methods
                for stmt in &g.statements {
                    if let Statement::Function(f) = stmt {
                        map.insert(format!("{}.{}", g.name, f.name), index);
                        index += 1;
                    }
                }
            }
            _ => {}
        }
    }

    map
}
```

Update call emission:

```rust
Expr::Call { callee, args } => {
    // Emit arguments
    for arg in args {
        self.emit_expression(instructions, arg, locals)?;
    }

    // Get function index
    if let Expr::Identifier(name) = callee.as_ref() {
        let func_idx = self.module_context.function_indices.get(name)
            .ok_or_else(|| WasmError::new(format!("Unknown function: {}", name)))?;
        instructions.push(Instruction::Call(*func_idx));
    } else if let Expr::Member { object, field } = callee.as_ref() {
        // Method call: obj.method(args)
        // First arg is self (object pointer)
        self.emit_expression(instructions, object, locals)?;

        // Get gene type and method index
        let gene_name = self.get_gene_type_for_expr(object)?;
        let method_name = format!("{}.{}", gene_name, field);
        let func_idx = self.module_context.function_indices.get(&method_name)
            .ok_or_else(|| WasmError::new(format!("Unknown method: {}", method_name)))?;

        instructions.push(Instruction::Call(*func_idx));
    } else {
        return Err(WasmError::new("Unsupported callee expression"));
    }
}
```

#### 3.4.3 Type Inference for Operations

Add type tracking to emit correct WASM instructions:

```rust
enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

fn emit_binary_op_typed(
    &self,
    instructions: &mut Vec<Instruction<'static>>,
    op: BinaryOp,
    ty: WasmType,
) -> Result<(), WasmError> {
    match (op, ty) {
        (BinaryOp::Add, WasmType::I32) => instructions.push(Instruction::I32Add),
        (BinaryOp::Add, WasmType::I64) => instructions.push(Instruction::I64Add),
        (BinaryOp::Add, WasmType::F32) => instructions.push(Instruction::F32Add),
        (BinaryOp::Add, WasmType::F64) => instructions.push(Instruction::F64Add),
        // ... etc
    }
    Ok(())
}
```

---

## 4. Implementation Order

### Recommended Sequence

```
Week 1: Phase 1 - Local Variables
├── Day 1-2: LocalsTable implementation
├── Day 3-4: Let bindings
├── Day 5: Assignment statements
└── Day 6-7: Testing and debugging

Week 2: Phase 2a - Basic Control Flow
├── Day 1-2: If/else expressions
├── Day 3-4: Block expressions
└── Day 5-7: Testing

Week 3: Phase 2b - Loops
├── Day 1-2: While loops
├── Day 3-4: For loops (range only)
├── Day 5: Break/continue
└── Day 6-7: Testing

Week 4: Phase 2c - Match
├── Day 1-3: Match expressions (simple patterns)
├── Day 4-5: Pattern guards
└── Day 6-7: Testing

Week 5-6: Phase 3 - Genes
├── Days 1-3: Memory layout and allocation
├── Days 4-6: Field access (read/write)
├── Days 7-9: Gene methods
└── Days 10-14: Testing and integration

Week 7: Phase 4 - Polish
├── Day 1-2: Unary operators
├── Day 3-4: Function call improvements
├── Day 5-7: Type inference, error messages, docs
```

### Dependencies

```
LocalsTable ──────────────────────────────────────┐
     │                                            │
     ▼                                            │
Let Bindings ────────────────────────────────────┼───► Tests
     │                                            │
     ▼                                            │
If/Else ─────────► Match (depends on conditionals)│
     │                                            │
     ▼                                            │
Block Expressions                                 │
     │                                            │
     ▼                                            │
While Loop ──────► For Loop (uses same primitives)│
     │                                            │
     ▼                                            │
Break/Continue                                    │
     │                                            │
     ▼                                            │
Memory Section ──► Gene Allocation ──► Field Access
                        │
                        ▼
                   Gene Methods
```

---

## 5. Testing Strategy

### Unit Tests for Each Phase

**Phase 1 Tests** (`tests/wasm_locals_tests.rs`):

```rust
#[test]
fn test_let_binding_simple() {
    let source = r#"
    fun test() -> i64 {
        let x = 42
        return x
    }
    exegesis { Test. }
    "#;
    // Compile and verify execution returns 42
}

#[test]
fn test_let_binding_with_expression() {
    let source = r#"
    fun test(a: i64) -> i64 {
        let x = a + 10
        return x
    }
    exegesis { Test. }
    "#;
    // Compile and verify test(5) returns 15
}

#[test]
fn test_variable_reassignment() {
    let source = r#"
    fun test() -> i64 {
        let x = 1
        x = 2
        return x
    }
    exegesis { Test. }
    "#;
    // Compile and verify returns 2
}
```

**Phase 2 Tests** (`tests/wasm_control_flow_tests.rs`):

```rust
#[test]
fn test_if_else() {
    let source = r#"
    fun max(a: i64, b: i64) -> i64 {
        if a > b { a } else { b }
    }
    exegesis { Test. }
    "#;
}

#[test]
fn test_while_loop() {
    let source = r#"
    fun sum_to(n: i64) -> i64 {
        let sum = 0
        let i = 0
        while i <= n {
            sum = sum + i
            i = i + 1
        }
        return sum
    }
    exegesis { Test. }
    "#;
}

#[test]
fn test_match_literal() {
    let source = r#"
    fun classify(x: i64) -> i64 {
        match x {
            0 => 100,
            1 => 200,
            _ => 300,
        }
    }
    exegesis { Test. }
    "#;
}
```

**Phase 3 Tests** (`tests/wasm_gene_tests.rs`):

```rust
#[test]
fn test_gene_allocation() {
    let source = r#"
    gene Point {
        has x: i64
        has y: i64
    }
    exegesis { A 2D point. }
    "#;
    // Verify allocator function exists
}

#[test]
fn test_gene_field_access() {
    let source = r#"
    gene Counter {
        has value: i64

        fun get() -> i64 {
            return self.value
        }

        fun increment() {
            self.value = self.value + 1
        }
    }
    exegesis { A counter. }
    "#;
}
```

### Integration Tests

Update `/home/ardeshir/repos/univrs-dol/tests/wasm_execution.rs` to enable the currently-ignored tests as features are implemented.

---

## Appendix: WASM Instruction Reference

### Local Variables

| Instruction | Description |
|-------------|-------------|
| `local.get $idx` | Push local variable onto stack |
| `local.set $idx` | Pop stack and store in local |
| `local.tee $idx` | Store in local without popping |

### Control Flow

| Instruction | Description |
|-------------|-------------|
| `block $label` | Start structured block |
| `loop $label` | Start loop block |
| `if / else / end` | Conditional |
| `br $depth` | Branch (break/continue) |
| `br_if $depth` | Conditional branch |
| `br_table` | Multi-way branch |

### Memory Operations

| Instruction | Description |
|-------------|-------------|
| `i32.load offset align` | Load 4 bytes as i32 |
| `i64.load offset align` | Load 8 bytes as i64 |
| `i32.store offset align` | Store i32 (4 bytes) |
| `i64.store offset align` | Store i64 (8 bytes) |
| `memory.grow` | Grow memory by pages |

### Type Conversion

| Instruction | Description |
|-------------|-------------|
| `i32.wrap_i64` | i64 → i32 |
| `i64.extend_i32_s` | i32 → i64 (signed) |
| `f64.convert_i64_s` | i64 → f64 |
| `i64.trunc_f64_s` | f64 → i64 |

---

## Summary

This implementation plan provides a structured approach to extending the DOL WASM compiler from basic function compilation to full language support. The phased approach ensures:

1. **Incremental progress**: Each phase builds on the previous
2. **Testability**: Clear test cases for each feature
3. **Maintainability**: Well-defined extension points
4. **Compatibility**: Works with existing `wasm-encoder` and `wasmtime` infrastructure

Key files to modify:
- `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` - All compilation changes
- `/home/ardeshir/repos/univrs-dol/src/wasm/mod.rs` - New types if needed
- `/home/ardeshir/repos/univrs-dol/tests/wasm_execution.rs` - Enable tests as features land

Start with Phase 1 (Local Variables) as it unblocks all other phases.
