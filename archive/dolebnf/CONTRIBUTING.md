# Contributing to Metal DOL

Thank you for your interest in contributing to Metal DOL! This document provides guidelines and best practices for contributing to the project.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Community Guidelines](#community-guidelines)

## Getting Started

### Prerequisites

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **Git**: For version control
- **A text editor**: VS Code with rust-analyzer recommended

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/metadol.git
   cd metadol
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/univrs/metadol.git
   ```

## Development Setup

```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Build documentation
cargo doc --open
```

### IDE Setup (VS Code)

Recommended extensions:
- rust-analyzer
- Even Better TOML
- CodeLLDB (for debugging)

## Making Changes

### Branch Naming

Use descriptive branch names with prefixes:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes

Example: `feature/add-test-generator`

### Commit Messages

Follow conventional commit format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Example:
```
feat(parser): add support for evolution declarations

Implements parsing of 'evolves' blocks including:
- Version lineage (@ version > parent)
- adds/deprecates/removes statements
- because rationale strings

Closes #42
```

## Code Style

### Rust Guidelines

1. **Run `cargo fmt`** before committing
2. **Address all `clippy` warnings**
3. **Document public items** with `///` doc comments
4. **Use meaningful variable names**
5. **Keep functions focused** - one function, one purpose

### Documentation Comments

All public items must have documentation:

```rust
/// Parses a gene declaration from the token stream.
///
/// A gene is the atomic unit of DOL, declaring fundamental truths
/// that cannot be decomposed further.
///
/// # Arguments
///
/// * `tokens` - The token stream to parse from
///
/// # Returns
///
/// A `Gene` AST node on success, or a `ParseError` on failure.
///
/// # Example
///
/// ```rust
/// let mut parser = Parser::new("gene container.exists { ... }");
/// let gene = parser.parse_gene()?;
/// assert_eq!(gene.name, "container.exists");
/// ```
pub fn parse_gene(&mut self) -> Result<Gene, ParseError> {
    // implementation
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Provide helpful error messages with context
- Include source location in parser errors
- Use `thiserror` for error definitions

## Testing

### Test Requirements

- All new features must include tests
- All bug fixes should include regression tests
- Maintain >80% code coverage for core modules

### Test Organization

```
tests/
├── lexer_tests.rs      # Lexer unit tests
├── parser_tests.rs     # Parser unit tests
├── integration_tests.rs # End-to-end tests
└── fixtures/           # Test DOL files
    ├── valid/          # Valid DOL files
    └── invalid/        # Invalid DOL files for error testing
```

### Writing Tests

```rust
#[test]
fn test_descriptive_name() {
    // Arrange
    let input = "gene test { }";
    
    // Act
    let result = parse(input);
    
    // Assert
    assert!(result.is_ok());
}
```

### Running Specific Tests

```bash
# Run tests matching pattern
cargo test parse_gene

# Run a specific test module
cargo test lexer_tests

# Run tests with stdout
cargo test -- --nocapture
```

## Documentation

### DOL Language Documentation

When adding language features:
1. Update `docs/specification.md`
2. Update `docs/grammar.ebnf`
3. Add examples in `examples/`
4. Create or update tutorials if needed

### API Documentation

- All public functions, structs, and enums must be documented
- Include code examples where helpful
- Document panics and errors
- Use `#[doc(hidden)]` for internal-only items

## Pull Request Process

### Before Submitting

1. **Sync with upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks**:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   cargo doc
   ```

3. **Update documentation** for any changed functionality

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe testing done

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] All CI checks pass
```

### Review Process

1. Create PR with descriptive title and body
2. Request review from maintainers
3. Address feedback constructively
4. Squash commits if requested
5. Maintainer merges when approved

## Community Guidelines

### Code of Conduct

We follow the [Contributor Covenant](https://www.contributor-covenant.org/). Please be respectful and inclusive.

### Getting Help

- **Questions**: Open a [Discussion](https://github.com/univrs/metadol/discussions)
- **Bugs**: Open an [Issue](https://github.com/univrs/metadol/issues) with reproduction steps
- **Feature Requests**: Open an Issue with use case description

### Communication

- Be respectful and constructive
- Assume good intentions
- Provide context in discussions
- Acknowledge and credit others' work

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes for significant contributions
- Project documentation where appropriate

---

Thank you for contributing to Metal DOL! Your efforts help make ontology-first development a reality.
