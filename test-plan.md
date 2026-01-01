# DOL WASM Compilation Test Plan

This document outlines the test strategy and test cases for the DOL to WASM compilation feature implementation.

## Overview

The DOL WASM backend compiles DOL (Design Ontology Language) source code to WebAssembly (WASM) bytecode. This test plan covers:

1. **Current working features** (already passing)
2. **Features under development** (local variables, control flow)
3. **Future features** (genes, traits, advanced patterns)

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

### Working Features (Level 1-2)

| Feature | Test File | Status |
|---------|-----------|--------|
| Module declaration | level1-minimal/*.dol | PASS |
| Function declaration | level2-basic/add_function.dol | PASS |
| i64 parameters | level2-basic/arithmetic.dol | PASS |
| i64 return type | level2-basic/*.dol | PASS |
| Binary operators (+, -, *, /) | level2-basic/arithmetic.dol | PASS |
| Comparison operators (>, <, ==) | - | PASS |
| Return statements | level2-basic/*.dol | PASS |
| Integer literals | - | PASS |
| Float literals | - | PASS |

### Under Development (Level 2-4)

| Feature | Test File | Status | Priority |
|---------|-----------|--------|----------|
| Local variables (let) | level2-basic/locals_test.dol | TODO | HIGH |
| Type inference for locals | level2-basic/locals_test.dol | TODO | HIGH |
| If statements | level4-control/if_else_test.dol | TODO | HIGH |
| If-else | level4-control/if_else_test.dol | TODO | HIGH |
| While loops | level4-control/loop_test.dol | TODO | MEDIUM |
| For loops | level4-control/loop_test.dol | TODO | MEDIUM |
| Break/continue | level4-control/loop_test.dol | TODO | MEDIUM |
| Match expressions | level4-control/match_expr.dol | TODO | LOW |

### Future Features (Level 3-5)

| Feature | Test File | Status | Priority |
|---------|-----------|--------|----------|
| Gene fields | level3-types/gene_methods_test.dol | TODO | MEDIUM |
| Gene methods | level3-types/gene_methods_test.dol | TODO | MEDIUM |
| Gene inheritance | level3-types/gene_methods_test.dol | TODO | LOW |
| Trait definitions | level5-advanced/trait_impl_test.dol | TODO | LOW |
| Trait implementations | level5-advanced/trait_impl_test.dol | TODO | LOW |
| System declarations | level5-advanced/system_impl.dol | TODO | LOW |

## Test Cases by Feature

### 1. Local Variables (HIGH PRIORITY)

**File:** `test-cases/level2-basic/locals_test.dol`

| Test | Description | Expected Result |
|------|-------------|-----------------|
| test_simple_let | let binding with type annotation | Compiles, executes correctly |
| test_let_inference | let binding with type inference | Compiles, executes correctly |
| test_multiple_locals | Multiple let bindings | Compiles, executes correctly |
| test_chained_locals | Locals used in sequence | Compiles, executes correctly |
| test_shadowing | Variable shadowing | Compiles, handles shadowing |
| test_float_local | Float type local | Compiles with f64 |
| test_bool_local | Boolean type local | Compiles with i32 |

**WASM Implementation Requirements:**
- Pre-scan function body to collect local declarations
- Allocate locals after function parameters
- Map local names to indices
- Emit `local.set` for initialization
- Emit `local.get` for access

### 2. Control Flow - If/Else (HIGH PRIORITY)

**File:** `test-cases/level4-control/if_else_test.dol`

| Test | Description | Expected Result |
|------|-------------|-----------------|
| test_simple_if | Single if, fall-through | Block structure correct |
| test_if_else | If with else branch | Both branches work |
| test_if_else_chain | Multiple else-if | All conditions checked |
| test_max | Max of two values | Returns larger value |
| test_min | Min of two values | Returns smaller value |
| test_abs | Absolute value | Handles negative correctly |
| test_clamp | Value clamping | Bounds checking works |
| test_nested_if | Nested conditionals | Proper nesting |
| test_complex_condition | && in condition | Short-circuit eval |
| test_or_condition | \|\| in condition | Short-circuit eval |

**WASM Implementation Requirements:**
- Use WASM `if` instruction for conditionals
- Use WASM `block`/`br` for structured control flow
- Handle else branches with `else` instruction
- Emit comparison results as i32 (0/1)

### 3. Control Flow - Loops (MEDIUM PRIORITY)

**File:** `test-cases/level4-control/loop_test.dol`

| Test | Description | Expected Result |
|------|-------------|-----------------|
| test_while_countdown | While with decrement | Terminates at 0 |
| test_while_sum | While with accumulator | Correct sum |
| test_for_sum | For loop summation | Correct sum |
| test_for_factorial | For loop product | Correct factorial |
| test_loop_break | Infinite loop with break | Exits at target |
| test_loop_continue | Loop with continue | Skips correctly |
| test_nested_while | Nested while loops | Correct iterations |
| test_nested_for | Nested for loops | Correct iterations |
| test_gcd | GCD algorithm | Correct result |

**WASM Implementation Requirements:**
- Use WASM `loop` instruction for while loops
- Use WASM `block` + `br_if` for loop conditions
- Implement `br` for break (branch out of block)
- Implement `br` + label for continue (branch to loop head)
- For loops need range iteration (desugar to while)

### 4. Genes (MEDIUM PRIORITY)

**File:** `test-cases/level3-types/gene_methods_test.dol`

| Test | Description | Expected Result |
|------|-------------|-----------------|
| Point gene | Gene with x, y fields | Compiles to memory layout |
| Counter gene | Gene with method | Method callable |
| Calculator gene | Multiple methods | All methods work |
| Rectangle gene | Area/perimeter | Computed from fields |
| Vector2D gene | Float fields | f64 operations work |
| Dog extends Animal | Inheritance | Fields inherited |

**WASM Implementation Requirements:**
- Memory layout for gene fields
- Method compilation as functions
- Field access via memory instructions
- Self/this parameter handling
- Inheritance via field embedding

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

### Compilation Errors (as of current state)

The WASM compiler has type mismatches in `emit_statement`:
- `emit_expression` expects `&FunctionDecl` but receives `&LocalsTable`
- Need to fix signature or pass both

### Missing Features

1. **Local Variables**: `LocalsTable` exists but not fully integrated
2. **Control Flow**: Stubs exist, return "not implemented" errors
3. **Genes**: Extract functions from genes not implemented
4. **Traits**: Not parsed for WASM compilation

## Success Criteria

### Phase 1: Local Variables
- [ ] All tests in `locals_test.dol` compile
- [ ] WASM validates with wasm-tools
- [ ] Execution produces correct results

### Phase 2: Control Flow
- [ ] All tests in `if_else_test.dol` compile
- [ ] All tests in `loop_test.dol` compile
- [ ] Break/continue work correctly
- [ ] Nested control flow works

### Phase 3: Genes
- [ ] Simple gene with fields compiles
- [ ] Gene methods are callable
- [ ] Field access works

### Phase 4: Integration
- [ ] All `working/` tests pass
- [ ] CI runs WASM tests
- [ ] Test coverage > 80% for wasm module

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
