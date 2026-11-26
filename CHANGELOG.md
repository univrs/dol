# Changelog

All notable changes to DOL (Design Ontology Language) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Releases 0.2.3, 0.3.1, 0.4.0, 0.4.1, 0.6.0, 0.7.0, 0.7.1 and 0.7.2 were tagged without changelog
entries; see `git log` between the tags for their changes. Dates below are git tag dates
(0.8.x: crates.io publish dates).

## [Unreleased]

### Changed
- Packaging: the published crate now contains only `src/`, `Cargo.toml`, `README.md`, `CHANGELOG.md` and the licence files. dol 0.8.2 shipped 734 files, including internal notes, prototypes and sibling apps.
- The internal `wasm-stress-test` binary now requires `--features cli,wasm`, so `cargo install dol` without `--features cli` no longer installs it.
- README rewritten for v0.8.2.
- `dol-parse` and `dol-check` parse the whole file and report every declaration. Previously they saw only the first declaration and silently ignored the rest of the file, including garbage. Unparseable trailing content is now an error. `dol-parse --format json` lists a `declarations` array per file, replacing the single `declaration_type`/`name` pair.
- `metadol::parse_file` and `parse_and_validate` still return only the first declaration; their docs now say so and point to `parse_dol_file` / `parse_file_all`.
- `ast::Trait` has a new `laws: Vec<LawDecl>` field (serde default: empty). Code that builds `Trait` literals must set it.
- `dol-migrate`: `--diff` is now preview-only, like `--dry-run`. Add `--write` to show the diff and apply it. Plain invocation still rewrites files in place.

### Added
- `a implies b` binary operator: right-associative, binds looser than `||` (Lean-alignment L0.1).
- `exists x: T. p` in expression position (L0.2).
- `law name { ... }` without a parameter list (L0.3). Quantifiers accept several binders: `forall a: A, b: B. p` is `forall a: A. forall b: B. p`.
- Trait laws are kept in the AST, and `dol-parse` shows them (L0.4).
- Type enforcement under `dol-check --typecheck` / `ValidationOptions { typecheck: true }`. Before this, the validator's type pass was an empty placeholder and type-incorrect programs passed. Checking is gradual: names the validator cannot resolve (builtins, host functions, imports) are typed as unknown and accepted. Now enforced:
  - `has field: T = default` against `T`; `where` constraints must be `bool`.
  - `const` / `var` values against their annotation.
  - `fun` bodies (top-level and inside `gen`/`trait`/`rule`/`system`): `let x: T = e`, `return e` against the declared return type, assignment, `if`/`while` conditions, operator operands, and arity and argument types of calls to functions declared in the same file. Fields and sibling functions are visible in method bodies.
  - Type names in field, parameter, return and `const` types that are declared nowhere. `dol-check` resolves names across all files it checks. Files with `use` imports are skipped. Single capital letters are taken to be type variables, because `gen` generic parameters are not kept in the AST.
  - The type checker now understands the lowercase type names the parser emits (`i32`, `u64`, `f64`, `bool`, `string`, `()`). Before this it mapped them to unknown generic types.
- `validator::validate_declarations` and `validator::declared_type_names`. `TypeChecker::gradual`, `TypeChecker::bind`, `TypeChecker::check_function`, `Type::primitive_or_unknown`.

### Not yet enforced (type checking)
Each item below has an `#[ignore]`d test in `tests/type_enforcement.rs`, except where noted.
- Only primitive types (numeric, `bool`, `string`, `()`) are compared. User-defined, generic, tuple and function types are treated as unknown, so `Vec<i64>` vs `Vec<string>` is not caught.
- Numeric widths and signedness: all numeric types are mutually compatible (`has n: u32 = -1` passes).
- Field access (`a.b`, `this.x`) is untyped.
- Not covered by a test: implicit (trailing-expression) returns, effect/purity checking, and names brought in by `use`.

### Fixed
- Numeric literals `1e9`, `3.0e8`, `1_000`, `0x10`, `0b1010` and `0o17` were split into a number followed by an identifier.
- A `{` or `}` inside a `docs { }` / `exegesis { }` block ended the block early. An unterminated docs block is now an error.
- A `use` line before an `evo` declaration, or mid-file before a `const`, swallowed that declaration. `use` lines are now parsed rather than skipped token by token, and junk after a use path is an error. `parse_dol_file` keeps mid-file `use` declarations.
- `mod x { }` as the first item of a file was a parse error. Block-module bodies are still skipped, not kept in the AST.
- `docs { }` inside a `gen`, `type` or `rule` body was dropped (empty exegesis) and replaced by a placeholder `_skip is _noop` statement.
- Whole-file parsing no longer returns placeholder declarations (`_module_doc`, `_module_decl`, `_use_only`) for file-level docs blocks, trailing `mod` blocks or `use` lines.
- `vudo` binary compiles again (`EvalResult::SpiritLoaded` was not handled in the REPL loop). Smoke tests added.
- `examples/wasm_basic.rs` compiles again (missing `attributes` field), so plain `cargo test --features cli,serde,wasm` builds.
- `dol-migrate` no longer rewrites prose. Line comments, string literals and `docs`/`exegesis` bodies are left untouched, e.g. "gene" stays "gene" in documentation text.
- `dol-codegen -t typescript` and `-t jsonschema` now emit typed `has x: T` fields; they were silently dropped. JSON Schema fields with a default are not marked required.
- Stale "DOL v0.9.0" module docs in `manifest` and `system`.
- `apps/gen-registry`: removed `[[bench]]` and `[[bin]]` entries whose sources do not exist, and fixed API drift so the library and CLI compile. Its integration tests and examples still do not compile.
- `crates/dol-codegen-wit` compiles again (missing `HasField::personal`). Trivial API drift fixed in examples, tests and benches of `dol-reflect`, `vudo-ai`, `vudo-planetserve` and `vudo-state`.
- Dead links to the removed `docs/book/` CRDT guide.
- `docs/security/IMPLEMENTATION-SUMMARY.md` and `bft-test-report.md` no longer claim passing tests or production readiness: the adversarial suite does not compile and BFT voting is simulated. `docs/reports/DOL_Bootstrap_Completion.md` is flagged as a task brief, because self-hosting does not work.

## [0.8.2] - 2026-11-20

Spirit Hosting Plan H0 ("toolchain truth"): DOL-compiled Spirits now link and run in the VUDO VM.

### Changed
- **Host ABI unified with the VUDO VM** (univrs/dol#34). Bodiless `sex fun host_*` declarations import from module `vudo` with the VM linker's exact signatures (`src/wasm/host_abi.rs`). Previously they imported from `loa` and could not link.
- A `sex fun` the VM does not provide (unknown name, wrong arity, float where the host expects an integer) is now a compile error.
- `compile_source` and the Spirit pipeline emit real WASM instead of a placeholder module (univrs/dol#31).

### Added
- String literals, `string` parameters and typed string locals passed to `host_*` functions lower to `(ptr, len)`. Modules that import pointer-taking host functions export `memory`.
- `dol-abi`: `vm_host_functions()` (the 15 VM host functions).
- `dol-build` CLI (DOL → WASM), wasm-bindgen annotations, `dol-abi` crate, LLVM native backend (`dol-native`, beta), meta-programming system, CRDT/Automerge codegen.
  - Note: `dol-abi/`, the LLVM backend (`llvm-backend/`) and the Automerge codegen (`crates/dol-codegen-rust`) are separate crates in this repository. They are not part of the published `dol` crate. Only `dol-abi` is built in CI. The Automerge codegen is not wired into any CLI.

### Known issues
- `dol-parse` and `dol-check` only parse the first declaration in each file.
- The `vudo` binary (`--features vudo`) does not compile.
- Without `--features cli`, `cargo install dol` installs only the internal `wasm-stress-test` binary.

### Deprecated
- `dol-abi::standard_host_functions()` (the 22 `vudo_*` names). Use `vm_host_functions()`.

### Fixed
- Clippy lints that kept the CI `Check` job red.
- MCP doc tests; `.dol` examples updated to v0.8 lowercase type syntax.

### Notes
- Minimum supported Rust version is 1.84.


## [0.8.1] - 2026-01-26

Reconstructed from `git log v0.8.0..v0.8.1`; no changelog entry was written at release time.

### Added
- Module system in the parser: `module path @ version`, `use` with visibility (`pub`, `pub(spirit)`, `pub(parent)`), import sources (local, `@org/package` registry, `@git:`, `@https:`), glob and named imports with aliases (#19).
- File-level validation (`validate_file`, `validate_file_with_options`) for module and `use` declarations (#19).
- Spirit REPL (`src/repl/`) with tree shaking; tree shaking integrated into `WasmCompiler::compile_file` (#19).
- Rust codegen for wasm-bindgen: `#[wasm_export]` → `#[wasm_bindgen]`, `thread_local!` + `RefCell` for `sex var` globals.
- Example Spirits under `examples/spirits/` (scientific modelling, art, Game of Life). Several of these files label themselves "DOL v0.9.0"; no v0.9.0 has been released.
- `docs/ARCHITECTURE.md` (architecture reference written for v0.8.0).

### Changed
- Bootstrap compiler sources in `dol/` updated to v0.8.0 syntax; `List<T>` replaced with `Vec<T>`.

### Fixed
- Parser: `gen` and `docs` keywords are handled when skipping `use` statements.
- `InvalidDeclaration` error message lists the v0.8.0 keywords.
- WASM backend updated for the v0.8.0 `Block`/`SexBlock` AST changes.
- Tests use non-deprecated type names.

## [0.8.0] - 2026-01-17 - "Clarity"

### Language Changes

#### New Preferred Keywords
- `gen` replaces `gene` for genome declarations
- `rule` replaces `constraint` for constraint declarations
- `evo` replaces `evolves` for evolution declarations
- `docs` replaces `exegesis` for documentation blocks

#### Modernized Type Names (Rust-aligned)
- `string` replaces `String`
- `bool` replaces `Bool`
- `i8`, `i16`, `i32`, `i64` replace `Int8`, `Int16`, `Int32`, `Int64`
- `u8`, `u16`, `u32`, `u64` replace `UInt8`, `UInt16`, `UInt32`, `UInt64`
- `f32`, `f64` replace `Float32`, `Float64`
- `()` replaces `Void` for unit type
- `Vec<T>` replaces `List<T>` for vectors

#### Deprecated Syntax (warnings in 0.8, errors in 0.9)
The old keywords and types are deprecated but still supported with compiler warnings.

### New Features

#### Tree Shaking
- Added `TreeShaking` pass for dead code elimination
- Analyzes dependency graphs to identify unreachable declarations
- Public declarations and their transitive dependencies are preserved
- Use `--tree-shake` flag in `dol-parse` CLI

#### CLI Improvements
- `dol-parse --tree-shake` - Enable tree shaking
- `dol-parse --roots <names>` - Specify additional root declarations
- `dol-parse --shake-analyze` - Dry-run to preview eliminations
- Output formats (pretty, JSON, compact, debug) show eliminated declarations

#### Migration Tool
- `dol-migrate 0.7-to-0.8 <path>` - Automatic syntax migration
- `--dry-run` flag to preview changes
- `--diff` flag to show before/after
- `--modernize` flag for return statement updates

### Internal Changes
- Lexer updated with new keywords and type tokens
- Parser supports both old and new syntax
- Deprecation warnings emitted at parse time with source locations
- Type name normalization in codegen

### Migration

```bash
# Auto-migrate your DOL files to v0.8.0 syntax
cargo run --bin dol-migrate --features cli -- 0.7-to-0.8 src/

# Preview changes first
cargo run --bin dol-migrate --features cli -- 0.7-to-0.8 --diff src/
```

## [0.3.0] - 2025-12-27 - "HIR"

### Language Changes

#### New Syntax
- `val` for immutable bindings (replaces `let`)
- `var` for mutable bindings (replaces `let mut`)
- `type` as preferred type declaration keyword
- `extends` for inheritance (replaces `derives from`)
- `forall` as unified quantifier (replaces `each`/`all`)

#### Deprecated Syntax (warnings in 0.3, errors in 0.4)
- `let` → use `val`
- `let mut` → use `var`
- `each` → use `forall`
- `all` (quantifier) → use `forall`
- `module` → use `mod`
- `never` → use `not`
- `derives from` → use `extends`
- `matches` → use `==`

### Internal Changes
- Added HIR (High-level Intermediate Representation)
- Reduced AST complexity from 50+ to 22 node types
- Simplified codegen via canonical HIR forms
- Added migration tool: `dol-migrate 0.2-to-0.3`

### New Modules
- `hir/` - HIR type definitions and utilities
- `lower/` - AST to HIR lowering with desugaring
- `codegen/hir_rust.rs` - HIR-based Rust code generation

### Migration

```bash
# Auto-migrate your DOL files
cargo run --bin dol-migrate --features cli -- 0.2-to-0.3 src/
```

## [0.2.2] - 2025-12-26 - "Bootstrap"

### Added
- **bootstrap.dol** - Wrapper types for DOL parser compatibility (LetStmt, VarStmt, BinaryExpr, CallExpr, etc.)
- Tuple variant with span construction in codegen
- Named wildcard pattern support (`args: _` syntax)
- Missing TokenKind variants (Law, Sex, Quote, Var)

### Fixed
- Recursive type handling with `Box<Block>` indirection
- String scrutinee matching with `.as_str()` conversion
- Type mismatches (CallArg vs Arg, proper clone for ownership)
- Pattern qualification for ambiguous variant names

### Changed
- Removed `Eq` from default derives (f64 doesn't implement Eq)

### Milestone
- **DOL self-hosting bootstrap generates valid Rust** - Generated code from ast.dol, token.dol, and bootstrap.dol compiles successfully (2544 lines, 0 errors)
  - Note (2026-11): this no longer holds. Regenerating the same files with the v0.8.2 toolchain produces code with 19 compile errors, and the crate generated from all of `dol/*.dol` fails with 850 errors. DOL is not self-hosting. See `docs/DOL-STATUS-2026-11.md`.

## [0.2.1] - 2025-12-25 - "Community"

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

## [0.2.0] - 2025-12-25 - "Meta-Programming"

### Added
- **Quote/Eval System** - AST manipulation at runtime
  - `'expr` captures expression as `Quoted<T>` AST data
  - `!quoted` evaluates quoted AST back to value
  - Quasi-quote (`` `template ``) with unquote (`~splice`) (note: the v0.8.2 parser does not accept this syntax)
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

## [0.1.0] - 2025-12-24 - "Genesis"

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

## [0.0.1] - 2025-12-19 - "Prototype"

### Added
- Initial DOL parser implementation
- Lexer using `logos` crate
- Recursive descent parser
- Basic AST definitions
- Gene, Trait, System, Constraint, Evolution declarations
- Span tracking for error reporting
- Basic validation

---

[Unreleased]: https://github.com/univrs/dol/compare/v0.8.2...HEAD
[0.8.2]: https://github.com/univrs/dol/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/univrs/dol/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/univrs/dol/compare/v0.7.2...v0.8.0
[0.3.0]: https://github.com/univrs/dol/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/univrs/dol/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/univrs/dol/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/univrs/dol/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/univrs/dol/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/univrs/dol/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/univrs/dol/releases/tag/v0.0.1
