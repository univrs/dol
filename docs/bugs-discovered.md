# DOL Bugs Discovered Through Edge Case Testing

This document tracks bugs and edge cases discovered during Phase 5 (Spirits Jam) edge case testing. Tests are located in `tests/edge_cases/`.

## Bug Discovery Summary

| ID | Category | Severity | Status | Description |
|----|----------|----------|--------|-------------|
| BUG-001 | Numerical | Medium | Open | Integer division by zero behavior undefined |
| BUG-002 | Numerical | Low | Open | NaN comparison behavior not IEEE 754 compliant |
| BUG-003 | Parser | Medium | Open | Deep nesting may cause stack overflow |
| BUG-004 | Parser | Low | Open | Unicode identifier support unclear |
| BUG-005 | WASM | High | Open | Deep recursion causes unrecoverable crash |
| BUG-006 | WASM | Medium | Open | Memory limit exceeded gives unhelpful error |
| BUG-007 | Modules | Medium | Open | Circular dependencies not detected until runtime |
| BUG-008 | Modules | Low | Open | Shadowing warnings not implemented |

---

## Detailed Bug Reports

### BUG-001: Integer Division by Zero

**Category:** Numerical
**Severity:** Medium
**File:** `tests/edge_cases/numerical.rs` - `division_by_zero::integer_division_by_zero`

**Description:**
Integer division by zero behavior is undefined. The evaluator either:
- Returns an error (acceptable)
- Panics (unacceptable for robustness)
- Returns a special value (needs documentation)

**Reproduction:**
```dol
fun test() -> i64 {
    return 100 / 0
}
```

**Expected Behavior:**
Division by zero should return a well-defined error with clear error message.

**Suggested Fix:**
Add explicit check in evaluator binary operations:
```rust
BinaryOp::Div => {
    if right == 0 {
        return Err(EvalError::division_by_zero(span));
    }
    left / right
}
```

---

### BUG-002: NaN Comparison Not IEEE 754 Compliant

**Category:** Numerical
**Severity:** Low
**File:** `tests/edge_cases/numerical.rs` - `nan_infinity::nan_comparison_always_false`

**Description:**
Per IEEE 754, `NaN == NaN` should return `false`. Current implementation may not handle this correctly if using Rust's `PartialEq` directly.

**Reproduction:**
```dol
fun test() -> bool {
    let nan = 0.0 / 0.0
    return nan == nan  // Should be false
}
```

**Expected Behavior:**
`NaN == NaN` should return `false` per IEEE 754 specification.

**Suggested Fix:**
Ensure float comparisons use proper IEEE 754 semantics:
```rust
(Value::Float(a), Value::Float(b)) => {
    // Note: This already handles NaN correctly if using standard Rust comparison
    Value::Bool(a == b)
}
```

---

### BUG-003: Deep Expression Nesting May Stack Overflow

**Category:** Parser
**Severity:** Medium
**File:** `tests/edge_cases/parser.rs` - `deeply_nested::nested_parentheses_100_levels`

**Description:**
The recursive descent parser uses call stack for parsing nested expressions. With 100+ levels of nesting, this may cause stack overflow on systems with limited stack size.

**Reproduction:**
```dol
// Generate with: "(" * 100 + "42" + ")" * 100
((((((((((((((((((((((((((((((((((((((((((((((((((42))))))))))))))))))))))))))))))))))))))))))))))))))
```

**Expected Behavior:**
Either:
1. Parse successfully with any reasonable nesting depth
2. Return a clear error "maximum nesting depth exceeded"

**Suggested Fix:**
Add depth counter to parser:
```rust
const MAX_NESTING_DEPTH: usize = 256;

fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
    self.depth += 1;
    if self.depth > MAX_NESTING_DEPTH {
        return Err(ParseError::max_nesting_exceeded(self.current_span()));
    }
    // ... parsing logic
    self.depth -= 1;
    Ok(expr)
}
```

---

### BUG-004: Unicode Identifier Support Unclear

**Category:** Parser
**Severity:** Low
**File:** `tests/edge_cases/parser.rs` - `unicode::unicode_identifier_basic`

**Description:**
It's unclear whether DOL supports Unicode identifiers (e.g., `données`, `変数`, `переменная`). Current behavior is inconsistent.

**Reproduction:**
```dol
gen Données {
    données has valeur: i64
}
```

**Expected Behavior:**
Either:
1. Support Unicode identifiers (like Rust)
2. Reject with clear error "identifiers must be ASCII"

**Suggested Fix:**
Document the decision in the language spec. If supporting Unicode:
```rust
fn is_identifier_char(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c.is_numeric()
}
```

---

### BUG-005: Deep Recursion Causes Unrecoverable WASM Crash

**Category:** WASM
**Severity:** High
**File:** `tests/edge_cases/wasm.rs` - `stack_overflow::deep_recursion_factorial`

**Description:**
Calling a recursive function with a large argument (e.g., `factorial(10000)`) causes the WASM runtime to crash without a recoverable error.

**Reproduction:**
```dol
fun factorial(n: i64) -> i64 {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}

// Call: factorial(10000)
```

**Expected Behavior:**
- Return error "stack overflow" or "maximum recursion depth exceeded"
- Runtime should recover and continue processing other calls

**Suggested Fix:**
1. Implement tail call optimization for eligible functions
2. Add recursion depth limit at WASM level
3. Use trampolining for deep recursion

---

### BUG-006: Memory Limit Error Messages Unhelpful

**Category:** WASM
**Severity:** Medium
**File:** `tests/edge_cases/wasm.rs` - `memory_limits::large_array_allocation`

**Description:**
When WASM memory limits are exceeded, the error message doesn't indicate what caused the issue or how much memory was attempted.

**Reproduction:**
```dol
fun create_large() -> Vec<i64> {
    let arr = []
    let i = 0
    while i < 100000000 {
        arr = push(arr, i)
        i = i + 1
    }
    return arr
}
```

**Expected Behavior:**
Error message like: "Memory limit exceeded: attempted to allocate 800MB, limit is 256MB"

**Suggested Fix:**
Track allocation sizes and provide detailed error:
```rust
impl WasmRuntime {
    fn allocate(&mut self, size: usize) -> Result<*mut u8, WasmError> {
        if self.used + size > self.limit {
            return Err(WasmError::memory_exceeded(size, self.limit, self.used));
        }
        // ... allocation
    }
}
```

---

### BUG-007: Circular Dependencies Not Detected Until Runtime

**Category:** Modules
**Severity:** Medium
**File:** `tests/edge_cases/modules.rs` - `circular_deps::direct_circular_reference`

**Description:**
Circular module dependencies (A → B → A) are not detected at parse or compile time. They only manifest as errors at runtime or during linking.

**Reproduction:**
```dol
// module_a.dol
module a @ 1.0.0
use b.Thing
gen AThing { has b: Thing }

// module_b.dol
module b @ 1.0.0
use a.AThing
gen Thing { has a: AThing }
```

**Expected Behavior:**
Compiler should detect circular dependencies during analysis phase with clear error:
"Circular dependency detected: a → b → a"

**Suggested Fix:**
Add dependency graph analysis during compilation:
```rust
impl ModuleResolver {
    fn check_circular_deps(&self) -> Result<(), CompileError> {
        let graph = self.build_dependency_graph();
        if let Some(cycle) = graph.find_cycle() {
            return Err(CompileError::circular_dependency(cycle));
        }
        Ok(())
    }
}
```

---

### BUG-008: Name Shadowing Warnings Not Implemented

**Category:** Modules
**Severity:** Low
**File:** `tests/edge_cases/modules.rs` - `shadowing::local_shadows_import`

**Description:**
When a local declaration shadows an imported name, no warning is issued. This can lead to subtle bugs where the wrong type is used.

**Reproduction:**
```dol
module test @ 1.0.0

use other.Thing  // Imports Thing from 'other'

gen Thing {      // Shadows the import!
    has value: i64
}
```

**Expected Behavior:**
Warning: "Local declaration 'Thing' shadows imported 'other.Thing'"

**Suggested Fix:**
Add shadowing detection during name resolution:
```rust
impl NameResolver {
    fn bind(&mut self, name: &str, decl: Declaration) {
        if let Some(existing) = self.lookup(name) {
            self.warnings.push(Warning::shadowing(name, existing, decl));
        }
        self.bindings.insert(name.to_string(), decl);
    }
}
```

---

## Potential Bugs Requiring Further Investigation

### INVESTIGATE-001: Float Precision in Physics Calculations

**File:** `tests/edge_cases/numerical.rs` - `precision::carnot_efficiency_precision`

When temperatures are very close (e.g., `T_hot = 1000.0`, `T_cold = 999.9999999`), the Carnot efficiency calculation may lose precision. Need to verify that physics Spirit formulas maintain adequate precision for scientific computing use cases.

### INVESTIGATE-002: Hot Reload State Handling

**File:** `tests/edge_cases/wasm.rs` - `hot_reload::reload_with_state_change`

During hot reload, global state is reset. Need to document this behavior and potentially provide state preservation mechanisms for REPL workflow.

### INVESTIGATE-003: Diamond Dependency Type Identity

**File:** `tests/edge_cases/modules.rs` - `diamond_deps::basic_diamond_pattern`

When the same type is imported through multiple paths (diamond pattern), verify that type identity is preserved. `b.Shared` and `c.Shared` should be the same type as `d.Shared`.

### INVESTIGATE-004: Subnormal Number Performance

**File:** `tests/edge_cases/numerical.rs` - `subnormal::gradual_underflow_in_physics`

Operations on subnormal numbers can be significantly slower on some hardware. Physics calculations near absolute zero may trigger this. May need option to flush subnormals to zero for performance.

---

## Fixed Bugs

_No bugs have been fixed yet. This section will be updated as bugs are resolved._

---

## Running Edge Case Tests

```bash
# Run all edge case tests
cargo test edge_cases

# Run specific category
cargo test edge_cases::numerical
cargo test edge_cases::parser
cargo test edge_cases::wasm
cargo test edge_cases::modules

# Run with output to see notes
cargo test edge_cases -- --nocapture
```

---

## Contributing Bug Reports

When adding new bug reports, include:

1. **Unique ID**: BUG-XXX format
2. **Category**: Numerical, Parser, WASM, Modules, Codegen, etc.
3. **Severity**: Critical, High, Medium, Low
4. **Reproduction**: Minimal DOL code that triggers the bug
5. **Expected vs Actual**: What should happen vs what does happen
6. **Suggested Fix**: If known, include code snippet

File bugs against the GitHub repository: https://github.com/univrs/dol/issues
