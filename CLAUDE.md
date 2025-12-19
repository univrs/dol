# Metal DOL Remediation Project

## Project Overview

This project transforms Metal DOL from an early-stage prototype into a production-ready DSL toolchain for ontology-first development. The remediation addresses 10 critical areas identified in the project analysis.

## Repository Context

- **Project**: univrs/metadol
- **Language**: Rust (100%)
- **Current State**: Prototype with lexer, parser, AST - lacking tests, docs, CI
- **Target State**: Production-ready with full test coverage, CI/CD, formal grammar

## Architecture

```
metadol/
├── src/
│   ├── lib.rs           # Library entry point, re-exports
│   ├── lexer.rs         # Tokenizer using logos
│   ├── parser.rs        # Recursive descent parser
│   ├── ast.rs           # AST definitions with Span tracking
│   ├── validator.rs     # Semantic validation
│   ├── codegen.rs       # Test code generation
│   ├── error.rs         # Error types with thiserror
│   └── bin/
│       ├── dol-parse.rs # Parser CLI
│       ├── dol-test.rs  # Test generator CLI
│       └── dol-check.rs # Validation CLI
├── tests/
│   ├── lexer_tests.rs   # 20+ lexer tests
│   ├── parser_tests.rs  # 20+ parser tests
│   └── integration_tests.rs
├── examples/
│   ├── genes/           # Gene examples
│   ├── traits/          # Trait examples
│   ├── constraints/     # Constraint examples
│   └── systems/         # System examples
├── docs/
│   ├── grammar.ebnf     # Formal EBNF grammar
│   ├── specification.md # Language spec
│   └── tutorials/       # Step-by-step guides
└── .github/
    └── workflows/
        └── ci.yml       # GitHub Actions CI
```

## Key Technical Decisions

### Lexer Implementation
- Use `logos` crate for fast, declarative tokenization
- Token types: keywords, identifiers (qualified with dots), versions, operators
- Span tracking for error reporting

### Parser Implementation  
- Recursive descent parser (hand-written, not generated)
- Produces strongly-typed AST
- Error recovery with helpful messages including source location

### AST Design
- `Declaration` enum with variants: Gene, Trait, Constraint, System, Evolution
- `Statement` enum for predicates: Has, Is, DerivesFrom, Requires, Uses, etc.
- All nodes include `Span` for source location

### CLI Tools
- `dol-parse`: Parse and validate DOL files, output JSON AST
- `dol-test`: Generate Rust tests from .dol.test files  
- `dol-check`: CI validation gate, check exegesis coverage

## Development Guidelines

### Code Style
- Run `cargo fmt` before all commits
- Zero `clippy` warnings allowed
- All public items must have `///` doc comments
- Use `Result<T, E>` for fallible operations

### Testing Requirements
- Unit tests for lexer: all token types, edge cases
- Unit tests for parser: all declaration types, error cases
- Integration tests: parse all example files
- Target: >80% code coverage

### Documentation
- Every public struct, enum, function documented
- Code examples in doc comments
- Grammar specification in EBNF
- Tutorials for common workflows

## Phase Definitions

See `docs/roadmap.md` for detailed phase breakdown:
- **Phase 1**: Foundation (tests, docs, Cargo.toml)
- **Phase 2**: Tooling (CLI implementations)
- **Phase 3**: Documentation (tutorials, examples)
- **Phase 4**: Community (release, contributing)

## Agent Coordination

### Specialist Agents

1. **lexer-agent**: Implements and tests lexer
2. **parser-agent**: Implements and tests parser
3. **cli-agent**: Implements CLI tools
4. **docs-agent**: Documentation and examples
5. **ci-agent**: CI/CD and release automation

### Task Dependencies

```
lexer-agent completes → parser-agent can start
parser-agent completes → cli-agent can start
lexer+parser complete → docs-agent examples
cli-agent completes → ci-agent integration
```

## Success Criteria

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Formatting clean (`cargo fmt --check`)
- [ ] Docs build (`cargo doc`)
- [ ] All example .dol files parse successfully
- [ ] CI pipeline green
- [ ] First release tagged (v0.0.1)

## Commands Reference

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test -- --nocapture
cargo test lexer_tests
cargo test parser_tests

# Lint
cargo fmt --check
cargo clippy -- -D warnings

# Docs
cargo doc --open
cargo doc --document-private-items

# Run CLI tools (after implementation)
cargo run --bin dol-parse -- examples/genes/container.exists.dol
cargo run --bin dol-test -- examples/traits/container.lifecycle.dol.test
cargo run --bin dol-check -- examples/ --require-exegesis
```

## Notes

- DOL files use `.dol` extension
- Test files use `.dol.test` extension
- Exegesis is MANDATORY for all declarations
- Version numbers follow semver (X.Y.Z)
- Qualified identifiers use dot notation (domain.property)
