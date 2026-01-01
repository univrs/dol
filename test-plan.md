# DOL WASM Compilation Test Plan

This document outlines the test strategy and test cases for the DOL to WASM compilation feature implementation.

## Overview

The DOL WASM backend compiles DOL (Design Ontology Language) source code to WebAssembly (WASM) bytecode. This test plan covers:

1. **Current working features** (already passing)
2. **Features under development** (advanced genes, traits)
3. **Future features** (complex patterns, optimizations)

**Last Updated:** 2026-01-01
**Test Status:** 31 WASM execution tests passing

## Test Infrastructure

### Available Tools

| Tool | Version | Purpose |
|------|---------|---------|
| wasmtime | 40.0.0 | WASM execution runtime |
| wasm-tools | 1.243.0 | WASM validation and inspection |
| cargo test | - | Rust test framework |

### Test Helper Module

Location: `tests/wasm_test_helpers.rs`

Key functions:
- `compile_dol_to_wasm(source)` - Compile DOL source to WASM bytes
- `validate_wasm_bytes(bytes)` - Validate WASM magic number and version
- `validate_wasm_with_tools(bytes)` - Full validation using wasm-tools
- `execute_wasm_function(bytes, name, args)` - Execute WASM function
- `assert_compiles(source, context)` - Assert compilation succeeds
- `assert_wasm_result(bytes, name, args, expected, context)` - Assert execution result

### Running Tests

```bash
# Run all WASM tests
cargo test --features wasm

# Run specific test file
cargo test --features wasm wasm_execution

# List available tests
cargo test --features wasm --list
```

## Test Directory Structure

```
test-cases/
├── level1-minimal/          # Minimal viable tests
│   ├── empty_module.dol     # Empty module declaration
│   ├── exegesis_only.dol    # Just exegesis, no code
│   └── single_const.dol     # Single function
│
├── level2-basic/            # Basic features
│   ├── add_function.dol     # Simple addition
│   ├── arithmetic.dol       # Arithmetic operations
│   └── locals_test.dol      # Local variable tests (NEW)
│
├── level3-types/            # Type-related tests
│   ├── simple_gene.dol      # Basic gene declaration
│   ├── gene_with_constraint.dol
│   └── gene_methods_test.dol # Gene with methods (NEW)
│
├── level4-control/          # Control flow tests
│   ├── if_else.dol          # Basic if-else
│   ├── if_else_test.dol     # Comprehensive if-else (NEW)
│   ├── match_expr.dol       # Match expressions
│   └── loop_test.dol        # Loop tests (NEW)
│
├── level5-advanced/         # Advanced features
│   ├── trait_def.dol        # Trait definitions
│   ├── trait_impl_test.dol  # Trait implementations (NEW)
│   └── system_impl.dol      # System declarations
│
├── working/                 # Known passing tests (symlinks/copies)
│   ├── single_const.dol
│   ├── add_function.dol
│   ├── arithmetic.dol
│   └── ...
│
└── failing/                 # Known failing tests (for tracking)
    └── (empty - add failures here)
```

## Current Test Coverage

### Working Features (Level 1-4)

| Feature | Test File | Status |
|---------|-----------|--------|
| Module declaration | level1-minimal/*.dol | PASS |
| Function declaration | level2-basic/add_function.dol | PASS |
| i64 parameters | level2-basic/arithmetic.dol | PASS |
| i64 return type | level2-basic/*.dol | PASS |
| Binary operators (+, -, *, /) | level2-basic/arithmetic.dol | PASS |
| Comparison operators (>, <, ==, !=) | - | PASS |
| Return statements | level2-basic/*.dol | PASS |
| Integer literals | - | PASS |
| Float literals | - | PASS |
| Local variables (let) | level2-basic/locals_test.dol | PASS |
| Variable reassignment | level4-control/loop_test.dol | PASS |
| If statements | level4-control/if_else_test.dol | PASS |
| If-else | level4-control/if_else_test.dol | PASS |
| While loops | level4-control/loop_test.dol | PASS |
| For loops | level4-control/loop_test.dol | PASS |
| Break/continue | level4-control/loop_test.dol | PASS |
| Nested control flow | level4-control/loop_test.dol | PASS |
| Gene methods (simple) | level3-types/gene_methods_test.dol | PASS |
| Gene field access | level3-types/gene_methods_test.dol | PASS |
| Implicit self parameter | level3-types/gene_methods_test.dol | PASS |
| Gene inheritance | tests/wasm_execution.rs | PASS |
| Pattern matching (basic) | - | PASS |

### Under Development (Level 3-5)

| Feature | Test File | Status | Priority |
|---------|-----------|--------|----------|
| Complex gene layouts | level3-types/gene_methods_test.dol | TODO | MEDIUM |
| Match expressions (complex) | level4-control/match_expr.dol | TODO | LOW |
| Trait definitions | level5-advanced/trait_impl_test.dol | TODO | LOW |
| Trait implementations | level5-advanced/trait_impl_test.dol | TODO | LOW |
| System declarations | level5-advanced/system_impl.dol | TODO | LOW |

## Test Cases by Feature

### 1. Local Variables ✅ COMPLETE

**File:** `test-cases/level2-basic/locals_test.dol`

| Test | Description | Status |
|------|-------------|--------|
| test_simple_let | let binding with type annotation | PASS |
| test_let_inference | let binding with type inference | PASS |
| test_multiple_locals | Multiple let bindings | PASS |
| test_chained_locals | Locals used in sequence | PASS |
| test_shadowing | Variable shadowing | PASS |
| test_float_local | Float type local | PASS |
| test_bool_local | Boolean type local | PASS |

**WASM Implementation (Completed):**
- Pre-scan function body to collect local declarations
- Allocate locals after function parameters
- Map local names to indices via `LocalsTable`
- Emit `local.set` for initialization
- Emit `local.get` for access

### 2. Control Flow - If/Else ✅ COMPLETE

**File:** `test-cases/level4-control/if_else_test.dol`

| Test | Description | Status |
|------|-------------|--------|
| test_simple_if | Single if, fall-through | PASS |
| test_if_else | If with else branch | PASS |
| test_if_else_chain | Multiple else-if | PASS |
| test_max | Max of two values | PASS |
| test_min | Min of two values | PASS |
| test_abs | Absolute value | PASS |
| test_clamp | Value clamping | PASS |
| test_nested_if | Nested conditionals | PASS |
| test_complex_condition | && in condition | PASS |
| test_or_condition | \|\| in condition | PASS |

**WASM Implementation (Completed):**
- Use WASM `if` instruction for conditionals
- Use WASM `block`/`br` for structured control flow
- Handle else branches with `else` instruction
- Emit comparison results as i32 (0/1)

### 3. Control Flow - Loops ✅ COMPLETE

**File:** `test-cases/level4-control/loop_test.dol`

| Test | Description | Status |
|------|-------------|--------|
| test_while_countdown | While with decrement | PASS |
| test_while_sum | While with accumulator | PASS |
| test_for_sum | For loop summation | PASS |
| test_for_factorial | For loop product | PASS |
| test_loop_break | Infinite loop with break | PASS |
| test_loop_continue | Loop with continue | PASS |
| test_nested_while | Nested while loops | PASS |
| test_nested_for | Nested for loops | PASS |
| test_gcd | GCD algorithm | PASS |

**WASM Implementation (Completed):**
- Use WASM `loop` instruction for while loops
- Use WASM `block` + `br_if` for loop conditions
- Implement `br` for break (branch out of block)
- Implement `br` + label for continue (branch to loop head)
- For loops desugared to while loops
- **Block depth tracking via `LoopContext`**: Tracks relative break/continue depths when nested inside if/match blocks

### 4. Genes ✅ MOSTLY COMPLETE

**File:** `test-cases/level3-types/gene_methods_test.dol`

| Test | Description | Status |
|------|-------------|--------|
| Point gene | Gene with x, y fields | PASS |
| Counter gene | Gene with method | PASS |
| Calculator gene | Multiple methods | PASS |
| Rectangle gene | Area/perimeter | PASS |
| Vector2D gene | Float fields | PASS |
| Dog extends Animal | Inheritance | PASS |
| test_compile_gene_inheritance_layout | Parent/child compilation | PASS |
| test_compile_gene_inheritance_child_access_parent_field | Inherited field access | PASS |
| test_compile_gene_inheritance_reverse_order | Topological ordering | PASS |

**WASM Implementation (Completed):**
- [x] Memory layout for gene fields via `GeneLayout`
- [x] Method compilation as functions with `gene_` prefix
- [x] Field access via memory instructions (i64.load/i64.store)
- [x] Implicit self parameter via `GeneContext`
- [x] Inheritance via field embedding in `compute_gene_layout`
- [x] Topological ordering via `register_gene_layouts_ordered`
- [x] Parent field access in child methods via extended `GeneContext`

### 5. Traits (LOW PRIORITY)

**File:** `test-cases/level5-advanced/trait_impl_test.dol`

| Test | Description | Expected Result |
|------|-------------|-----------------|
| Comparable trait | Trait with capabilities | Parses correctly |
| Numeric trait | Arithmetic capabilities | Parses correctly |
| IntWrapper gene | Gene with trait methods | Methods compile |

**WASM Implementation Requirements:**
- Trait method tables (vtables)
- Dynamic dispatch (if needed)
- Trait constraint checking at compile time

## Validation Steps

### For Each Test Case:

1. **Parse Test**
   ```bash
   cargo run --bin dol-parse -- test-cases/<level>/<file>.dol
   ```

2. **Compile Test**
   ```rust
   let wasm = compile_dol_to_wasm(&source);
   assert!(wasm.is_some(), "Compilation should succeed");
   ```

3. **Validate WASM**
   ```bash
   wasm-tools validate <output>.wasm
   ```
   Or in code:
   ```rust
   assert!(validate_wasm_with_tools(&wasm_bytes).is_ok());
   ```

4. **Execute Test**
   ```rust
   let result = execute_wasm_function(&wasm, "function_name", &[args]);
   assert_eq!(result, Ok(expected_value));
   ```

5. **Runtime Validation**
   ```bash
   wasmtime <output>.wasm --invoke function_name arg1 arg2
   ```

## Known Issues

### Resolved Issues

The following issues have been fixed:

1. ~~`emit_expression` signature mismatch~~ - Fixed by passing both `FunctionDecl` and `LocalsTable`
2. ~~Local variables not integrated~~ - `LocalsTable` now fully functional
3. ~~Control flow stubs~~ - If/else, while, for, break/continue all implemented
4. ~~Break inside if blocks~~ - Fixed with `LoopContext` block depth tracking
5. ~~Gene methods not extracted~~ - Gene method extraction now works with implicit self
6. ~~Gene inheritance not implemented~~ - Fixed with field embedding in `compute_gene_layout` + topological ordering

### Remaining Limitations

1. **Complex Gene Layouts**: Nested genes not fully supported
2. **Traits**: Trait method dispatch not implemented
3. **Systems**: System declarations parsed but not compiled to WASM

## Success Criteria

### Phase 1: Local Variables ✅ COMPLETE
- [x] All tests in `locals_test.dol` compile
- [x] WASM validates with wasm-tools
- [x] Execution produces correct results

### Phase 2: Control Flow ✅ COMPLETE
- [x] All tests in `if_else_test.dol` compile
- [x] All tests in `loop_test.dol` compile
- [x] Break/continue work correctly
- [x] Nested control flow works (with block depth tracking)

### Phase 3: Genes ✅ MOSTLY COMPLETE
- [x] Simple gene with fields compiles
- [x] Gene methods are callable
- [x] Field access works
- [x] Implicit self parameter works
- [x] Gene inheritance (field embedding + topological ordering)
- [ ] Complex nested genes

### Phase 4: Integration (In Progress)
- [x] All `working/` tests pass
- [ ] CI runs WASM tests
- [ ] Test coverage > 80% for wasm module

### Phase 5: Traits & Systems (Future)
- [ ] Trait definitions compile
- [ ] Trait implementations work
- [ ] System declarations compile

## How to Add New Tests

1. Create `.dol` file in appropriate `level*` directory
2. Add entry to this document
3. Run parse test to ensure syntax is valid
4. If expected to pass, copy to `working/`
5. If expected to fail, add to `failing/` with comment explaining why

## References

- WASM specification: https://webassembly.github.io/spec/
- wasmtime docs: https://docs.wasmtime.dev/
- wasm-encoder crate: https://docs.rs/wasm-encoder/
- DOL language spec: `docs/specification.md`
