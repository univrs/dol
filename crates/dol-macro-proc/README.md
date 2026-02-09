# DOL Procedural Macro System

This crate provides procedural macros for Metal DOL, enabling Rust-style derive macros, attribute macros, and function-like macros.

## Features

- **Derive Macros**: Automatically generate trait implementations
- **Attribute Macros**: Transform declarations with attributes
- **Function-like Macros**: Custom syntax extensions
- **AST Manipulation**: Powerful API for transforming DOL AST
- **Code Generation**: Generate Rust, WIT, TypeScript, and documentation

## Modules

- `derive.rs`: Derive macro implementations (Debug, Clone, PartialEq, Gen)
- `attribute.rs`: Attribute macro implementations (cached, async, memoize, test, etc.)
- `function.rs`: Function-like macro implementations (sql, format, json, regex, etc.)
- `ast_util.rs`: AST traversal and manipulation utilities
- `codegen.rs`: Code generation for multiple target formats
- `error.rs`: Error types for procedural macros

## Derive Macros

Supported traits:
- `Debug`: Formatted debug output
- `Clone`: Cloning capability
- `PartialEq`: Equality comparison
- `Eq`: Total equality
- `Hash`: Hashing
- `Default`: Default values
- `Gen`: DOL-specific Gen trait

Usage in DOL:
```dol
#[derive(Debug, Clone, PartialEq)]
gene container.exists {
  container has identity
}
```

## Attribute Macros

Available attributes:
- `#[cached]`: Add caching behavior
- `#[async]`: Mark as asynchronous
- `#[memoize]`: Add memoization
- `#[deprecated]`: Mark as deprecated
- `#[test]`: Mark as test function
- `#[inline]`: Suggest inlining
- `#[cfg]`: Conditional compilation

Usage:
```dol
#[cached]
spell compute(x: Int) -> Int {
  x * x
}
```

## Function-like Macros

Available macros:
- `sql!`: SQL syntax checking and query building
- `format!`: String formatting
- `json!`: JSON literal construction
- `regex!`: Regex pattern validation
- `include_str!`: Include file as string
- `concat!`: Concatenate string literals

Usage:
```dol
let query = sql!("SELECT * FROM users WHERE id = ?", user_id);
let message = format!("Hello, {}!", name);
```

## Code Generation

Generate code for multiple targets:
- **Rust**: Struct definitions and trait implementations
- **WIT**: WebAssembly Interface Types
- **TypeScript**: Type definitions
- **Documentation**: Markdown documentation

Example:
```rust
use dol_macro_proc::codegen::{generate_rust_code, generate_wit_code};

let rust_code = generate_rust_code(&decl)?;
let wit_interface = generate_wit_code(&decl)?;
```

## AST Utilities

The `AstManipulator` provides:
- Recursive expression walking
- Identifier finding and replacement
- Node counting
- Pattern checking

Example:
```rust
use dol_macro_proc::ast_util::AstManipulator;

let manipulator = AstManipulator::new();
let idents = manipulator.find_identifiers(&expr);
let replaced = manipulator.replace_identifier(&expr, "old", "new");
```

## Status

**Current State**: Implementation complete with comprehensive tests. Minor AST compatibility issues need to be resolved to match the current DOL AST structure.

**Completed**:
- ✅ Derive macro system
- ✅ Attribute macro registry
- ✅ Function-like macro system
- ✅ AST manipulation utilities
- ✅ Multi-target code generation
- ✅ Comprehensive test suite

**Pending**:
- AST compatibility updates for current DOL AST
- Integration with proc-macro system
- Additional derive implementations
