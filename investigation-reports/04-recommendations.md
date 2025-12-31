# DOL → WASM Recommendations Report

**Date:** 2025-12-30
**Investigation:** Claude Flow Swarm - DOL WASM Pipeline Analysis
**Status:** COMPLETE

---

## Executive Summary

Based on comprehensive analysis of the DOL codebase, this report provides prioritized, actionable recommendations for achieving production-ready WASM output. The recommendations are organized by priority (P0-P3) with estimated complexity and dependencies.

---

## Strategic Decision: Which Path Forward?

### Option A: Enhance Direct WASM Compiler
**Recommended: YES**

- Pros: Simpler, fewer dependencies, already working
- Cons: Manual WASM emission, harder to optimize
- Effort: Medium
- Timeline: Iterative improvements possible

### Option B: Complete MLIR → WASM Pipeline
**Recommended: LONG TERM**

- Pros: Access to MLIR optimization passes, industry standard
- Cons: Complex, requires LLVM toolchain, significant work
- Effort: High
- Timeline: Significant investment

### Option C: Hybrid Approach
**Recommended: YES (Parallel tracks)**

- Direct WASM for immediate functionality
- MLIR for future optimization

---

## P0: Critical Recommendations

### R1: Add Local Variables to Direct WASM Compiler

**File:** `src/wasm/compiler.rs`

**Current State:** No support for `let` bindings

**Required Changes:**
```rust
// Add to WasmCompiler struct
local_vars: HashMap<String, (u32, ValType)>,  // name -> (index, type)

// Add method
fn emit_let_binding(&self, function: &mut Function, binding: &LetStmt) -> Result<(), WasmError>

// Update emit_statement to handle Statement::Let
```

**Complexity:** Medium
**Dependencies:** None
**Impact:** Unlocks most useful programs

---

### R2: Add Control Flow to Direct WASM Compiler

**File:** `src/wasm/compiler.rs`

**Current State:** No if/else, loops, or match

**Required Changes:**

```rust
// If/Else using WASM block structure
fn emit_if_expr(&self, function: &mut Function, if_expr: &IfExpr) -> Result<(), WasmError> {
    // Emit condition
    self.emit_expression(function, &if_expr.condition)?;

    // Use block/br_if pattern or WASM if instruction
    function.instruction(&Instruction::If(BlockType::Result(result_type)));
    self.emit_block(function, &if_expr.then_branch)?;
    function.instruction(&Instruction::Else);
    self.emit_block(function, &if_expr.else_branch)?;
    function.instruction(&Instruction::End);
    Ok(())
}
```

**Complexity:** Medium
**Dependencies:** R1 (local vars helpful but not required)
**Impact:** Unlocks Level 4 tests (control flow)

---

### R3: Add CLI for WASM Compilation

**Current State:** No CLI binary for WASM output

**Required Changes:**

Create `src/bin/dol-wasm.rs`:
```rust
use clap::Parser;
use metadol::wasm::WasmCompiler;

#[derive(Parser)]
struct Args {
    /// Input DOL file
    input: PathBuf,

    /// Output WASM file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable optimization
    #[arg(long)]
    optimize: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let source = std::fs::read_to_string(&args.input)?;
    let module = metadol::parse_file(&source)?;

    let compiler = WasmCompiler::new().with_optimization(args.optimize);
    let wasm = compiler.compile(&module)?;

    let output = args.output.unwrap_or_else(|| {
        args.input.with_extension("wasm")
    });
    std::fs::write(output, wasm)?;
    Ok(())
}
```

**Complexity:** Low
**Dependencies:** None
**Impact:** User-facing feature, enables CI integration

---

## P1: High Priority Recommendations

### R4: Add Gene Compilation to WASM

**File:** `src/wasm/compiler.rs`

**Current State:** `Declaration::Gene` returns error

**Required Changes:**

1. Map Gene fields to WASM memory layout
2. Generate constructor function
3. Generate getter/setter functions
4. Add memory section to WASM module

```rust
fn compile_gene(&self, gene: &GeneDecl) -> Result<Vec<u8>, WasmError> {
    // Calculate memory layout
    let layout = self.calculate_gene_layout(gene)?;

    // Generate $new_GeneName function
    let constructor = self.generate_gene_constructor(gene, &layout)?;

    // Generate field accessors
    let accessors = gene.fields.iter()
        .map(|f| self.generate_field_accessor(gene, f, &layout))
        .collect::<Result<Vec<_>, _>>()?;

    // Combine into module
    self.emit_gene_module(gene, constructor, accessors)
}
```

**Complexity:** High
**Dependencies:** Memory management design
**Impact:** Unlocks DOL's core abstraction

---

### R5: Add Match Expression Support

**File:** `src/wasm/compiler.rs`

**Current State:** Match returns error

**Required Changes:**

```rust
fn emit_match_expr(&self, function: &mut Function, match_expr: &MatchExpr) -> Result<(), WasmError> {
    // Emit scrutinee
    self.emit_expression(function, &match_expr.scrutinee)?;

    // For each arm, emit comparison and branch
    for arm in &match_expr.arms {
        // Duplicate scrutinee value
        function.instruction(&Instruction::LocalTee(temp_local));

        // Emit pattern comparison
        self.emit_pattern_match(function, &arm.pattern)?;

        // Conditional branch to arm body
        function.instruction(&Instruction::BrIf(arm_block));
    }
}
```

**Complexity:** High
**Dependencies:** R2 (control flow)
**Impact:** Unlocks pattern matching (core DOL feature)

---

### R6: Implement String Support

**Current State:** String literals fail compilation

**Required Changes:**

1. Add data section to WASM for string constants
2. Implement string type as (ptr, len) pair
3. Add memory allocation for dynamic strings

```rust
fn emit_string_literal(&self, module: &mut Module, s: &str) -> (u32, u32) {
    // Add to data section
    let offset = self.data_section_offset;
    let bytes = s.as_bytes();
    self.data_section.extend_from_slice(bytes);
    self.data_section_offset += bytes.len() as u32;

    (offset, bytes.len() as u32)
}
```

**Complexity:** Medium
**Dependencies:** Memory section setup
**Impact:** Required for many real programs

---

## P2: Medium Priority Recommendations

### R7: Add Loop Constructs

**File:** `src/wasm/compiler.rs`

**Required Changes:**

```rust
fn emit_for_loop(&self, function: &mut Function, for_loop: &ForLoop) -> Result<(), WasmError> {
    // Initialize iterator
    self.emit_expression(function, &for_loop.iterator)?;

    // Loop block
    function.instruction(&Instruction::Loop(BlockType::Empty));

    // Check condition
    self.emit_loop_condition(function, &for_loop)?;
    function.instruction(&Instruction::BrIf(1));  // Exit if false

    // Body
    self.emit_block(function, &for_loop.body)?;

    // Increment and continue
    function.instruction(&Instruction::Br(0));
    function.instruction(&Instruction::End);
    Ok(())
}
```

**Complexity:** Medium
**Dependencies:** R1 (local vars), R2 (control flow)
**Impact:** Unlocks iterative algorithms

---

### R8: Add Trait Compilation

**File:** `src/wasm/compiler.rs`

**Required Changes:**

1. Generate vtable structure for trait methods
2. Generate trait object representation
3. Add indirect call support

**Complexity:** High
**Dependencies:** R4 (gene compilation)
**Impact:** Unlocks polymorphism

---

### R9: Enable Ignored Tests

**Files:** `tests/wasm_execution.rs`, `tests/compiler_e2e.rs`

**Current State:** Many `#[ignore]` tests

**Required Changes:**

As features are implemented, remove `#[ignore]` from:
- `test_compile_and_execute_simple_function`
- `test_compile_with_control_flow`
- `test_compile_with_pattern_matching`
- `test_compile_gene_method`

**Complexity:** Low (per feature)
**Dependencies:** Feature implementation
**Impact:** CI verification of WASM output

---

## P3: Low Priority Recommendations

### R10: Add WASM Optimization Passes

**Current State:** `optimize: bool` flag exists but does nothing

**Required Changes:**

1. Implement basic peephole optimizations
2. Consider binaryen integration for advanced optimization

```rust
fn optimize_wasm(&self, wasm: Vec<u8>) -> Vec<u8> {
    if self.optimize {
        // Run wasm-opt or equivalent
        binaryen::optimize(&wasm, OptimizationLevel::O2)
    } else {
        wasm
    }
}
```

**Complexity:** Medium
**Dependencies:** None
**Impact:** Smaller, faster WASM output

---

### R11: Add Source Maps

**Current State:** No debug info in WASM

**Required Changes:**

1. Track source locations during compilation
2. Generate DWARF debug info or sourcemaps
3. Add name section to WASM

**Complexity:** Medium
**Dependencies:** None
**Impact:** Debugging support

---

### R12: Complete MLIR Pipeline (Long-term)

**Current State:** MLIR → WASM not connected

**Required Changes:**

1. Add MLIR pass manager
2. Lower to LLVM dialect
3. Configure WASM target
4. Emit via LLVM backend

**Complexity:** Very High
**Dependencies:** LLVM toolchain
**Impact:** Access to industrial-strength optimization

---

## Implementation Roadmap

### Phase 1: Core WASM (2-4 weeks effort)
- R1: Local variables
- R2: Control flow
- R3: CLI binary
- R9: Enable tests

### Phase 2: Language Features (4-6 weeks effort)
- R4: Gene compilation
- R5: Match expressions
- R6: String support
- R7: Loops

### Phase 3: Advanced Features (6+ weeks effort)
- R8: Trait compilation
- R10: Optimization
- R11: Source maps

### Phase 4: MLIR Path (Long-term)
- R12: Complete MLIR → WASM

---

## Quick Wins

These can be implemented in 1-2 days each:

| Item | File | Change |
|------|------|--------|
| CLI | `src/bin/dol-wasm.rs` | New file |
| Unary ops | `src/wasm/compiler.rs` | Add negation, not |
| Better errors | `src/wasm/mod.rs` | Improve error messages |
| CI check | `.github/workflows/` | Verify add.wasm |

---

## Dependencies Graph

```
R1 (Local Vars) ──┬──▶ R2 (Control Flow) ──┬──▶ R5 (Match)
                  │                         │
                  │                         └──▶ R7 (Loops)
                  │
                  └──▶ R4 (Genes) ──────────────▶ R8 (Traits)

R3 (CLI) ─────────────▶ R9 (Tests) ─────────────▶ CI Integration

R6 (Strings) ─────────▶ Memory Management ──────▶ R4 (Genes)
```

---

## Success Metrics

After implementing P0 + P1 recommendations:

| Metric | Current | Target |
|--------|---------|--------|
| Test cases passing | 27% | 80% |
| Declaration types supported | 1/7 | 4/7 |
| Expression types supported | 40% | 80% |
| Control flow support | 0% | 100% |
| CLI usability | None | Full |

---

*Generated by Claude Flow Swarm - Synthesizer Agent*
