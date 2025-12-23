# Meta DOL 2.0

[![Build Status](https://github.com/univrs/metadol/workflows/CI/badge.svg)](https://github.com/univrs/metadol/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.81%2B-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-516%20passing-brightgreen.svg)](https://github.com/univrs/metadol/actions)

**A declarative DSL for ontology-first development with meta-programming and multi-target compilation**

---

## Overview

Meta DOL (Design Ontology Language) is a production-ready DSL toolchain that enables ontology-first development. Instead of writing code and hoping it aligns with your domain model, Meta DOL lets you declare your domain's fundamental structure, behaviors, and constraints explicitly. The toolchain then validates, type-checks, and compiles to multiple targets including Rust, TypeScript, WebAssembly, and JSON Schema.

### What Makes DOL 2.0 Different?

| Traditional Approach | DOL 2.0 Approach |
|---------------------|------------------|
| Code → Tests → Documentation | **Design Ontology → Tests → Code** |
| Runtime errors from type mismatches | **Compile-time guarantees** |
| Scattered domain knowledge | **Single source of truth** |
| Manual code generation | **Multi-target compilation** |
| Limited metaprogramming | **Quote, Eval, Macros, Reflection** |

DOL 2.0 treats ontology as a first-class concern. You define **genes** (atomic types), **traits** (composable behaviors), **constraints** (invariants), **systems** (compositions), and **evolutions** (version tracking). Each declaration includes mandatory **exegesis**—human-readable documentation that bridges formal specification and domain understanding.

---

## Features

### Core Language

- **Genes** — Atomic types with fields and constraints
- **Traits** — Interface contracts with methods and events
- **Constraints** — Validation rules and invariants
- **Systems** — Module compositions with versioned dependencies
- **Evolutions** — Semantic versioning and migration tracking

### DOL 2.0 Expressions

```dol
// Lambdas and higher-order functions
transform = |x: Int64| -> Int64 { x * 2 }

// Pattern matching with guards
match container.state {
    Running if container.healthy { continue() }
    Stopped { restart() }
    _ { log("unknown state") }
}

// Pipelines and composition
result = data |> validate >> transform |> persist

// Control flow
for item in collection {
    if item.active { process(item) }
}
```

### Meta-Programming (Q2)

```dol
// Quote — capture expressions as AST data
expr = '(1 + 2 * 3)           // Quoted<Int64>

// Eval — execute quoted expressions
result = !expr                 // Int64 = 7

// Macros — compile-time code transformation
#derive(Debug, Clone)
gene Container { has id: UInt64 }

message = #format("Hello, {}!", name)
#assert(count > 0, "count must be positive")

// Reflect — runtime type introspection
info = ?Container
// info.name == "Container"
// info.fields == [{ name: "id", type: "UInt64" }]

// Idiom Brackets — applicative functor style
result = [| add mx my |]      // Desugars to: add <$> mx <*> my
```

### 20 Built-in Macros

| Macro | Description |
|-------|-------------|
| `#stringify(expr)` | Convert expression to string |
| `#concat(a, b, ...)` | Concatenate string literals |
| `#env("VAR")` | Read environment variable at compile time |
| `#cfg(condition)` | Conditional compilation |
| `#derive(Trait, ...)` | Generate trait implementations |
| `#assert(cond)` | Runtime assertion with auto-generated message |
| `#assert_eq(a, b)` | Assert equality |
| `#assert_ne(a, b)` | Assert inequality |
| `#format(fmt, ...)` | String formatting with `{}` placeholders |
| `#dbg(expr)` | Debug print (returns value) |
| `#todo(msg)` | Mark unimplemented code |
| `#unreachable()` | Mark unreachable code paths |
| `#compile_error(msg)` | Emit compile-time error |
| `#vec(a, b, c)` | Create vector from elements |
| `#file()` | Current file name |
| `#line()` | Current line number |
| `#column()` | Current column number |
| `#module_path()` | Current module path |
| `#option_env("VAR")` | Optional environment variable |
| `#include(path)` | Include file contents |

### Multi-Target Compilation

```bash
# Compile to Rust
dol build --target rust src/domain.dol -o generated/

# Compile to TypeScript
dol build --target typescript src/domain.dol -o generated/

# Generate JSON Schema for validation
dol build --target jsonschema src/domain.dol -o schemas/

# Compile to WebAssembly (requires LLVM 18)
dol build --target wasm src/domain.dol -o app.wasm
```

### MCP Server (AI Integration)

DOL 2.0 includes a Model Context Protocol (MCP) server for AI-driven development:

```bash
# Start MCP server
dol-mcp serve

# Available tools
dol/parse           # Parse DOL source → AST
dol/typecheck       # Type check source
dol/compile/rust    # Compile to Rust
dol/compile/typescript  # Compile to TypeScript
dol/eval            # Evaluate expression
dol/reflect         # Get type information
dol/format          # Format source code
dol/macros/list     # List available macros
dol/macros/expand   # Expand a macro
```

---

## Quick Start

### Prerequisites

- Rust toolchain 1.81 or later ([install from rustup.rs](https://rustup.rs))

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/univrs/metadol.git
cd metadol

# Build the project
cargo build --release

# Run tests (516 tests)
cargo test
```

### Create Your First .dol File

Create a file named `example.dol`:

```dol
gene user.Account {
    has id: UInt64
    has email: String
    has created_at: Timestamp
    
    constraint valid_email {
        this.email.contains("@")
    }
}

exegesis {
    A user account represents an authenticated entity in the system.
    Every account has a unique identifier, validated email address,
    and creation timestamp.
}

trait user.Authenticatable {
    uses user.Account
    
    fun authenticate(password: String) -> Bool
    fun reset_password(new_password: String) -> Result<Void, Error>
    
    each authentication emits AuthEvent
}

exegesis {
    Authenticatable provides password-based authentication
    with secure password reset capabilities.
}
```

### Parse and Validate

```bash
# Parse the file
cargo run --features cli --bin dol-parse -- example.dol

# Validate with type checking
cargo run --features cli --bin dol-check -- example.dol

# Generate Rust code
cargo run --features cli --bin dol-parse -- --format rust example.dol
```

### Expected Output

```
✓ example.dol
    user.Account gene with 3 fields, 1 constraint
    user.Authenticatable trait with 2 methods

Summary
  Total:    2 declarations
  Success:  2
  Errors:   0
```

---

## Installation

### From Source

```bash
# Clone and build
git clone https://github.com/univrs/metadol.git
cd metadol
cargo build --release

# Install CLI tools
cargo install --path . --features cli

# Verify installation
dol-parse --version
dol-check --version
dol-test --version
```

### Optional Features

```bash
# With CLI tools
cargo build --features cli

# With serialization support
cargo build --features serde

# With MLIR/WASM (requires LLVM 18)
cargo build --features mlir,wasm
```

---

## CLI Tools

### `dol-parse`

Parse DOL files and output AST in various formats.

```bash
dol-parse <file.dol>                    # Human-readable output
dol-parse --format json <file.dol>      # JSON AST
dol-parse --format rust <file.dol>      # Generate Rust code
dol-parse --format typescript <file.dol> # Generate TypeScript
```

### `dol-check`

Validate DOL files with full type checking.

```bash
dol-check examples/                     # Check directory
dol-check --strict src/**/*.dol         # Strict mode
dol-check --require-exegesis *.dol      # Require documentation
```

### `dol-test`

Run tests defined in `.dol.test` files.

```bash
dol-test examples/tests/                # Run test suite
dol-test --output tests/ *.dol.test     # Generate Rust tests
```

### `dol-mcp`

Model Context Protocol server for AI integration.

```bash
dol-mcp serve                           # Start MCP server (stdio)
dol-mcp manifest                        # Print server manifest
dol-mcp tool dol/parse source="..."     # Execute tool directly
```

---

## Language Reference

### Genes

Atomic types with fields, constraints, and optional inheritance.

```dol
gene container.Container {
    has id: UInt64
    has name: String
    has state: ContainerState
    has resources: ResourceLimits
    
    constraint valid_name {
        this.name.length > 0 && this.name.length <= 255
    }
    
    constraint resources_bounded {
        this.resources.memory <= MAX_MEMORY
    }
}

exegesis {
    A Container is the fundamental unit of workload isolation.
    Each container has immutable identity, mutable state,
    and enforced resource boundaries.
}
```

### Traits

Interface contracts with methods, events, and quantified statements.

```dol
trait container.Lifecycle {
    uses container.Container
    
    fun start() -> Result<Void, Error>
    fun stop(force: Bool) -> Result<Void, Error>
    fun restart() -> Result<Void, Error>
    
    each state_change emits LifecycleEvent
    all containers is monitored
}

exegesis {
    Lifecycle defines the state machine for container
    management with event emission on transitions.
}
```

### Constraints

Validation rules and domain invariants.

```dol
constraint container.Integrity {
    identity matches original_identity
    state never undefined
    boundaries never exceeded
}

exegesis {
    Container integrity ensures immutable identity,
    defined state, and enforced resource limits.
}
```

### Systems

Top-level compositions with versioned dependencies.

```dol
system univrs.Orchestrator @ 0.3.0 {
    requires container.Container >= 0.1.0
    requires container.Lifecycle >= 0.1.0
    requires scheduler.Scheduler >= 0.2.0
    
    each container has supervisor
    all containers is health_checked
}

exegesis {
    The Orchestrator manages container lifecycles,
    scheduling, and health monitoring across nodes.
}
```

### Evolutions

Version migration with semantic tracking.

```dol
evolves container.Lifecycle @ 0.2.0 > 0.1.0 {
    adds fun pause() -> Result<Void, Error>
    adds fun resume() -> Result<Void, Error>
    deprecates fun restart()
    because "Live migration requires pause/resume semantics"
}

exegesis {
    Version 0.2.0 introduces pause/resume for live migration
    support, deprecating the atomic restart operation.
}
```

---

## Type System

### Primitive Types

| Type | Description |
|------|-------------|
| `Bool` | Boolean value |
| `Int8`, `Int16`, `Int32`, `Int64` | Signed integers |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Unsigned integers |
| `Float32`, `Float64` | Floating-point numbers |
| `String` | UTF-8 string |
| `Bytes` | Byte array |
| `Timestamp` | Date/time |
| `Duration` | Time span |
| `Void` | No value |

### Generic Types

```dol
// Collections
List<T>
Map<K, V>
Set<T>

// Results
Option<T>
Result<T, E>

// Quoted expressions (meta-programming)
Quoted<T>
```

### Function Types

```dol
// Simple function
Fun<Int64, Int64>

// Multiple parameters
Fun<(Int64, String), Bool>

// Higher-order
Fun<Fun<Int64, Int64>, Int64>
```

---

## AST Transformation

DOL 2.0 includes a powerful AST transformation framework:

```rust
use metadol::transform::{Pass, PassPipeline, Visitor, Fold};

// Built-in passes
let pipeline = PassPipeline::new()
    .add(ConstantFolding)
    .add(DeadCodeElimination)
    .add(DesugarIdiom)
    .add(Simplify);

let optimized = pipeline.run(ast)?;
```

### Available Passes

| Pass | Description |
|------|-------------|
| `ConstantFolding` | Evaluate constant expressions at compile time |
| `DeadCodeElimination` | Remove unreachable code |
| `DesugarIdiom` | Transform `[| f x y |]` to `f <$> x <*> y` |
| `Simplify` | Simplify expressions (double negation, identity ops) |

---

## Project Structure

```
metadol/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── lexer.rs            # Tokenization
│   ├── parser.rs           # AST construction
│   ├── pratt.rs            # Expression parsing (Pratt parser)
│   ├── ast.rs              # AST node definitions
│   ├── typechecker.rs      # Type inference and checking
│   ├── validator.rs        # Semantic validation
│   ├── error.rs            # Error types
│   ├── codegen/            # Code generation
│   │   ├── rust.rs         # Rust target
│   │   ├── typescript.rs   # TypeScript target
│   │   └── jsonschema.rs   # JSON Schema target
│   ├── eval/               # Expression evaluation
│   │   ├── interpreter.rs  # Quote/Eval execution
│   │   ├── value.rs        # Runtime values
│   │   └── builtins.rs     # Built-in functions
│   ├── macros/             # Macro system
│   │   ├── builtin.rs      # 20 built-in macros
│   │   ├── expand.rs       # Macro expansion
│   │   └── registry.rs     # Macro registry
│   ├── reflect.rs          # Runtime type introspection
│   ├── transform/          # AST transformations
│   │   ├── visitor.rs      # AST traversal
│   │   ├── fold.rs         # Functional transformation
│   │   ├── passes.rs       # Optimization passes
│   │   └── desugar_idiom.rs # Idiom bracket desugaring
│   ├── mlir/               # MLIR code generation (optional)
│   ├── wasm/               # WASM backend (optional)
│   ├── mcp/                # MCP server
│   └── bin/                # CLI binaries
├── tests/                  # Test suites (516 tests)
├── examples/               # Example DOL files
└── docs/                   # Documentation
```

---

## Testing

```bash
# Run all tests (516 tests)
cargo test

# Run specific test suite
cargo test --test parser_tests
cargo test --test macro_tests
cargo test --test idiom_tests

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Test Coverage

| Suite | Tests |
|-------|-------|
| Unit Tests | 187 |
| DOL2 Tests | 41 |
| Idiom Tests | 27 |
| Integration Tests | 24 |
| Lexer Tests | 77 |
| Parser Tests | 80 |
| Quote Tests | 33 |
| Reflect Tests | 17 |
| Doc Tests | 30 |
| **Total** | **516** |

---

## Roadmap

### Completed

| Phase | Milestone | Status |
|-------|-----------|--------|
| Q1 | Foundation — Lexer, Parser, TypeChecker, Codegen | ✅ |
| Q2 | Meta-Programming — Quote, Eval, Macro, Reflect, Idiom | ✅ |
| Q3 | Infrastructure — MLIR, WASM, MCP Server | ✅ |

### Planned

| Phase | Milestone | Description |
|-------|-----------|-------------|
| Q4 | Self-Hosting | DOL compiler written in DOL |
| Q5 | Package Manager | Spirit packages, Mycelium registry |
| Q6 | IDE Support | LSP server, syntax highlighting |

---

## Examples

The `examples/` directory contains comprehensive examples:

```
examples/
├── genes/          # Atomic type definitions
├── traits/         # Interface contracts
├── constraints/    # Validation rules
├── systems/        # System compositions
├── evolutions/     # Version migrations
└── tests/          # Test specifications
```

---

## Documentation

- **[Language Specification](docs/specification.md)** — Formal language spec
- **[Grammar (EBNF)](docs/grammar.ebnf)** — Formal grammar
- **[Tutorials](docs/tutorials/)** — Step-by-step guides
- **[API Docs](https://docs.rs/metadol)** — Rust API reference

Generate local documentation:

```bash
cargo doc --open
```

---

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Run `cargo fmt` before committing
2. Ensure `cargo clippy -- -D warnings` passes
3. Add tests for new functionality
4. Update documentation for API changes

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## Acknowledgments

Meta DOL is part of the [Univrs](https://github.com/univrs) ecosystem, building the foundation for VUDO OS — a distributed, AI-native operating system where systems describe their ontological nature before their functionality.

---

**Built with Rust. Powered by Ontology. Driven by Clarity.** 🍄
