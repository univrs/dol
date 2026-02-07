# DOL Macro System

This crate provides the declarative macro system for Metal DOL, enabling compile-time metaprogramming through pattern-based code generation and transformation.

## Features

- **Declarative Macros**: macro_rules!-style pattern matching
- **Hygienic Expansion**: Automatic name resolution and scoping
- **Compile-time Code Generation**: Transform AST at compile time
- **Standard Library**: Common macros for everyday use

## Modules

- `declarative.rs`: Core declarative macro implementation with MacroRule and MacroTemplate
- `pattern.rs`: Pattern matching system with fragment specifiers and repetition
- `hygiene.rs`: Hygienic macro expansion to prevent name collisions
- `expand.rs`: Macro expansion engine with recursion tracking
- `registry.rs`: Macro registry for storing and looking up macros
- `stdlib.rs`: Standard library of common macros (const, add, identity, vec, etc.)
- `error.rs`: Error types for macro operations

## Usage

```rust
use dol_macro::prelude::*;

// Create a macro registry
let mut registry = MacroRegistry::new();

// Register standard library macros
dol_macro::stdlib::register_stdlib_macros(&mut registry);

// Create an expander
let mut expander = MacroExpander::with_registry(registry);

// Expand macros
let invocation = MacroInvocation::simple("identity", Span::default());
let result = expander.expand(&invocation);
```

## Pattern Matching

Patterns support:
- Empty patterns: `()`
- Metavariables: `$name:fragment`
- Token literals
- Sequences: multiple patterns in order
- Repetitions: `$(...)* `, `$(...)+`, `$(...)?`

Fragment specifiers include:
- `ident`: Identifier
- `expr`: Expression
- `stmt`: Statement
- `type`: Type
- `block`: Block
- `literal`: Literal value
- `tt`: Token tree

## Hygiene

The hygiene system ensures:
- Variables introduced by macros don't shadow variables at the call site
- Macros can't accidentally capture variables from the call site
- Each macro expansion has its own lexical scope

## Standard Library Macros

- `const!`: Constant value
- `add!`: Addition
- `identity!`: Identity function
- `vec!`: Vector literal
- `assert_eq!`: Equality assertion
- `min!`: Minimum of two values
- `max!`: Maximum of two values

## Status

**Current State**: Implementation complete with comprehensive tests. Minor AST compatibility issues need to be resolved to match the current DOL AST structure (Expr::Identifier vs Expr::Ident, etc.).

**Completed**:
- ✅ Core declarative macro system
- ✅ Pattern matching with fragment specifiers
- ✅ Hygienic macro expansion
- ✅ Macro registry and expansion engine
- ✅ Standard library macros
- ✅ Comprehensive test suite

**Pending**:
- AST compatibility updates for current DOL AST
- Integration with main DOL parser
- Additional standard library macros
