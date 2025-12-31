# STRESS TESTER Agent Report: Level 3-5 Test Cases

## Executive Summary

Created **6 test files** across three complexity levels to stress test the DOL to WASM pipeline:

- **Level 3 (Types)**: 2 files - Gene definitions with typed fields and constraints
- **Level 4 (Control Flow)**: 2 files - If/else and match expressions
- **Level 5 (Advanced)**: 2 files - Trait definitions and system implementations

---

## Test Files Created

### Level 3: Types (`/test-cases/level3-types/`)

#### 1. `simple_gene.dol`

```dol
module types @ 0.1.0

gene Point {
    has x: Float64
    has y: Float64
}
```

**Tests:**
- Module declaration with version
- Gene definition with typed fields
- Float64 primitive type usage

---

#### 2. `gene_with_constraint.dol`

```dol
module constrained @ 0.1.0

gene Credits {
    has amount: UInt64

    constraint non_negative {
        this.amount >= 0
    }
}
```

**Tests:**
- Gene with embedded constraint
- `this` keyword for self-reference
- Comparison operator in constraint expression
- UInt64 primitive type

---

### Level 4: Control Flow (`/test-cases/level4-control/`)

#### 3. `if_else.dol`

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

**Tests:**
- Function definition with typed parameters
- Return type annotation
- If/else control flow
- Comparison operators
- Return statements in branches

---

#### 4. `match_expr.dol`

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

**Tests:**
- Match expression as function body
- Integer literal patterns
- Wildcard pattern (`_`)
- String literals as match arms
- Implicit return from match

---

### Level 5: Advanced (`/test-cases/level5-advanced/`)

#### 5. `trait_def.dol`

```dol
module traits @ 0.1.0

trait Calculator {
    is add(a: Int32, b: Int32) -> Int32
    is multiply(a: Int32, b: Int32) -> Int32
}
```

**Tests:**
- Trait definition syntax
- Multiple method signatures in trait
- `is` keyword for method declarations
- Typed parameters and return types

---

#### 6. `system_impl.dol`

```dol
module systems @ 0.1.0

trait Greeter {
    is greet(name: String) -> String
}

system SimpleGreeter {
    impl Greeter {
        is greet(name: String) -> String {
            return "Hello, " + name
        }
    }
}
```

**Tests:**
- Trait definition
- System definition
- Trait implementation in system
- Method body implementation
- String concatenation operator (`+`)
- String parameter and return types

---

## Syntax Notes

### Observations from Existing Codebase

Based on analysis of existing DOL files in the repository:

1. **Module Declaration**: Uses `module name @ version` format (e.g., `module types @ 0.1.0`)

2. **Gene Syntax**:
   - Uses `has field: Type` for field declarations
   - Constraints can be embedded within genes using `constraint name { expr }`

3. **Function Syntax**:
   - `fun name(params) -> ReturnType { body }`
   - Consistent with examples in `/tests/codegen/golden/input/function.dol`

4. **Trait Syntax**:
   - Uses `is method(params) -> Type` for method signatures
   - Matches existing patterns in `/examples/traits/`

5. **Match Syntax**:
   - Uses `match expr { pattern => result }` with `=>` arrow syntax
   - Consistent with `/home/ardeshir/repos/univrs-dol/dol/types.dol`

6. **Control Flow**:
   - Standard `if condition { } else { }` blocks
   - Found in existing codebase functions

### Type Naming Convention

From `/home/ardeshir/repos/univrs-dol/dol/types.dol`:
- Signed integers: `Int8`, `Int16`, `Int32`, `Int64`
- Unsigned integers: `UInt8`, `UInt16`, `UInt32`, `UInt64`
- Floats: `Float32`, `Float64`
- Other: `Bool`, `String`, `Void`

---

## Expected WASM Compilation Results

Based on the Scout report findings:

| Test File | Expected Result | Reason |
|-----------|----------------|--------|
| `simple_gene.dol` | FAIL | WasmCompiler only supports `Declaration::Function` |
| `gene_with_constraint.dol` | FAIL | Genes not supported in WASM compiler |
| `if_else.dol` | FAIL | Control flow (if/else) not implemented |
| `match_expr.dol` | FAIL | Match expressions not implemented |
| `trait_def.dol` | FAIL | Traits not supported in WASM compiler |
| `system_impl.dol` | FAIL | Systems not supported in WASM compiler |

These test cases are designed to expose the current limitations documented in `scout-entry-points.md`:
- Only `Declaration::Function` types compile to WASM
- No control flow (if, match, loops)
- No gene/trait/system support

---

## Test Execution Commands

```bash
# Parse test (should work if parser supports syntax)
cargo run --bin dol-parse -- test-cases/level3-types/simple_gene.dol

# WASM compilation attempt (requires wasm feature)
# NOTE: No CLI currently exists - must use library API

# Using Spirit compiler (returns placeholder WASM)
# metadol::compiler::spirit::compile_file(path)

# Using WasmCompiler directly (limited support)
# metadol::wasm::WasmCompiler::new().compile(&ast)
```

---

## File Locations

| Level | File | Path |
|-------|------|------|
| 3 | simple_gene.dol | `/home/ardeshir/repos/univrs-dol/test-cases/level3-types/simple_gene.dol` |
| 3 | gene_with_constraint.dol | `/home/ardeshir/repos/univrs-dol/test-cases/level3-types/gene_with_constraint.dol` |
| 4 | if_else.dol | `/home/ardeshir/repos/univrs-dol/test-cases/level4-control/if_else.dol` |
| 4 | match_expr.dol | `/home/ardeshir/repos/univrs-dol/test-cases/level4-control/match_expr.dol` |
| 5 | trait_def.dol | `/home/ardeshir/repos/univrs-dol/test-cases/level5-advanced/trait_def.dol` |
| 5 | system_impl.dol | `/home/ardeshir/repos/univrs-dol/test-cases/level5-advanced/system_impl.dol` |

---

*Generated by STRESS TESTER Agent - 2025-12-30*
