# DOL Codegen Fix Requirements

> **Status:** Action Items for src/codegen/rust.rs

## Overview

This document catalogs fixes currently in `bootstrap-fix.sh` that should be implemented in the Rust codegen.

## Fix Categories

### 1. Cross-Module Import Generation

**Problem:** Generated files have no `use` statements.

**Current Fix:**
```bash
sed -i '4a use crate::token::Span;' ast.rs
```

**Permanent Fix:** Track type references and emit imports.

### 2. String Method Translation

**Problem:** DOL uses `char_at()` and `substring()` - not in Rust.

**Permanent Fix:**
```rust
fn translate_method_call(method: &str, ...) -> String {
    match method {
        "char_at" => format!("{}.chars().nth({} as usize).unwrap()", ...),
        "substring" => format!("{}[{} as usize..{} as usize].to_string()", ...),
        _ => ...
    }
}
```

### 3. Expression Semicolon Handling

**Problem:** Expression-bodied functions get trailing `;`, returning `()`.

```rust
// WRONG
pub fn is_digit(c: char) -> bool {
    ((c >= '0') && (c <= '9'));  // Returns ()
}
```

**Permanent Fix:** Detect final expression in block, omit semicolon.

### 4. Null Coalescing Operator (??)

**Problem:** `??` has no Rust equivalent.

**Permanent Fix:** `a ?? b` → `a.unwrap_or(b)`

### 5. Constructor Generation

**Problem:** `Type.new(args)` generates wrong arity calls.

**Permanent Fix:** Generate struct literals or proper `new()` with defaults.

### 6. Match Arm Type Consistency

**Problem:** Match arms return different types.

**Permanent Fix:** Analyze arms, ensure type consistency.

## Priority

| Fix | Priority | Impact |
|-----|----------|--------|
| Expression semicolons | P0 | 13 errors |
| Cross-module imports | P0 | 150 errors |
| String methods | P1 | 4 errors |
| ?? operator | P1 | 1 error |
| Match arms | P1 | 2 errors |
| Constructors | P2 | 2 errors |

## Testing

After each codegen fix:
```bash
make regen
cargo check 2>&1 | grep "^error\[" | wc -l  # Should decrease
```
