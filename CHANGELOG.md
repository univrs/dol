# Changelog

All notable changes to DOL (Design Ontology Language) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2024-12-25 - "Community"

### Added
- **CHANGELOG.md** - Keep a Changelog format with full version history
- **GitHub Issue Templates** (`.github/ISSUE_TEMPLATE/`)
  - Bug report template with DOL code blocks
  - Feature request template with syntax proposals
  - Documentation issue template
  - Template chooser config with helpful links
- **Release Workflow** (`.github/workflows/release.yml`)
  - Automated builds for Linux, macOS (x86_64 + ARM), Windows
  - GitHub Release creation with changelog excerpt
  - crates.io publishing support

### Fixed
- Improved codegen operators with Box for HasField variant

## [0.2.0] - 2024-12-25 - "Meta-Programming"

### Added
- **Quote/Eval System** - AST manipulation at runtime
  - `'expr` captures expression as `Quoted<T>` AST data
  - `!quoted` evaluates quoted AST back to value
  - Quasi-quote (`` `template ``) with unquote (`~splice`)
- **Reflection System** (`src/reflect.rs`)
  - `?Type` returns `TypeInfo` at runtime
  - `TypeInfo`, `FieldInfo`, `MethodInfo` structs
  - `TypeRegistry` for type introspection
- **Macro System** (`src/macros/`)
  - `#derive(Trait, ...)` - Generate trait implementations
  - `#stringify(expr)` - Convert expression to string
  - `#concat(a, b, ...)` - Concatenate string literals
  - `#env("VAR")` - Read environment variable at compile-time
  - `#cfg(condition)` - Conditional compilation
  - `#assert(cond)` - Runtime assertion
  - `#assert_eq(a, b)` / `#assert_ne(a, b)` - Equality assertions
  - `#format(fmt, ...)` - String formatting
  - `#dbg(expr)` - Debug print (returns value)
  - `#todo(msg)` / `#unreachable()` - Development markers
  - `#file()`, `#line()`, `#column()`, `#module_path()` - Source location
  - `#vec(a, b, c)` - Vector literal
  - `#compile_error(msg)` - Compile-time error
- **Idiom Brackets** - Applicative functor syntax
  - `[| f x y |]` desugars to `f <$> x <*> y`
  - Desugaring pass in `src/transform/desugar_idiom.rs`
- **AST Transform Framework** (`src/transform/`)
  - Visitor pattern for AST traversal
  - Fold pattern for AST transformation
  - Multiple optimization passes
- **SEX System Documentation** (`docs/sex-system.md`)
- **Syntax Reference** (`docs/syntax-reference.md`)
- **Rust Codegen Tests** (`tests/codegen_rust_tests.rs`, `tests/codegen_operators_test.rs`)

### Changed
- Bumped version to 0.2.0
- Test count: 274 passing (reorganized test structure)
- Pratt parser extended with meta-operators (Quote: 135, Eval: 130, Reflect: 135 precedence)

### Fixed
- Reserved keyword collisions (`exists`, `state` renamed in examples)
- Ontology files updated for DOL 2.0 syntax compliance
- Binary expression parsing improvements

## [0.1.0] - 2024-12-24 - "Genesis"

### Added
- **DOL 2.0 Parser** - Full support for modern DOL syntax
  - `module` declarations with versioning (`module name @ 1.0.0`)
  - `pub` visibility modifiers
  - `use` imports with destructuring (`use path.{ items }`)
  - `fun` function declarations with bodies
  - Inline `constraint` blocks
  - `state` declarations in systems
  - `law` declarations in traits
  - Generic type parameters (`<T, U>`)
- **SEX System** - Side Effect eXecution for explicit effect tracking
  - `sex fun` - Functions with side effects
  - `sex var` - Mutable global variables
  - `sex { }` - Inline effect blocks
  - `sex extern` - FFI declarations
  - Effect tracking in typechecker
  - Lint rules (E001-E004, W001-W002)
- **Biology Module** - Biomimicry modeling examples
  - Mycelium network simulation
  - Ecosystem dynamics (Lotka-Volterra)
  - Evolution and speciation
  - Hyphal growth patterns
  - Nutrient transport systems
- **Code Generation**
  - Rust code generator (`src/codegen/rust.rs`)
  - TypeScript code generator (`src/codegen/typescript.rs`)
  - JSON Schema generator (`src/codegen/jsonschema.rs`)
- **CLI Tools**
  - `dol-parse` - Parse DOL files, output JSON AST
  - `dol-test` - Generate tests from `.dol.test` files
  - `dol-check` - Validation gate for CI
  - `dol-codegen` - Code generation CLI
  - `dol-mcp` - MCP server for IDE integration
- **Documentation**
  - EBNF Grammar (`docs/grammar.ebnf`)
  - Language Specification (`docs/specification.md`)
  - 5 Tutorials (`docs/tutorials/01-05`)
  - Publishing Guide (`docs/publish_crate.md`)
- **CI/CD** - GitHub Actions workflow (`.github/workflows/ci.yml`)

### New Keywords
`module`, `use`, `pub`, `fun`, `const`, `var`, `extern`, `sex`, `state`, `law`, `forall`, `migrate`

## [0.0.1] - 2024-12-19 - "Prototype"

### Added
- Initial DOL parser implementation
- Lexer using `logos` crate
- Recursive descent parser
- Basic AST definitions
- Gene, Trait, System, Constraint, Evolution declarations
- Span tracking for error reporting
- Basic validation

---

[Unreleased]: https://github.com/univrs/dol/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/univrs/dol/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/univrs/dol/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/univrs/dol/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/univrs/dol/releases/tag/v0.0.1
