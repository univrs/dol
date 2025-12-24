# DOL 2.0 Implementation Checklist

> **For:** Claude-flow task orchestration  
> **Priority:** Q2 Meta-Programming  
> **Format:** Executable task list with dependencies

---

## Quick Reference: What's Done vs What's Next

```
✅ COMPLETE                    🚧 IN PROGRESS              ⏳ FUTURE
─────────────                  ──────────────              ────────
Lexer (logos)                  Quote operator              MLIR backend
Parser (Pratt)                 Eval operator               WASM codegen
Type Checker                   Macro system                Self-hosting
Rust Codegen                   Reflect operator            VUDO VM
367 tests passing              Biological primitives       Séance sessions
```

---

## Sprint 1: Quote Operator (Week 1-2)

### Tasks

- [ ] **LEXER-Q1**: Add `Quote` token to lexer
  ```rust
  // src/lexer.rs - Add to Token enum
  #[token("'")]
  Quote,
  ```
  - File: `src/lexer.rs`
  - Est: 30 min
  - Tests: `tests/lexer_tests.rs` - Add 5 quote token tests

- [ ] **AST-Q1**: Add Quoted expression variant
  ```rust
  // src/ast.rs - Add to Expr enum
  Quoted {
      inner: Box<Expr>,
      span: Span,
  },
  ```
  - File: `src/ast.rs`
  - Est: 30 min
  - Depends: None

- [ ] **PARSE-Q1**: Parse quote as prefix operator
  ```rust
  // src/pratt.rs - Add to prefix parsing
  Token::Quote => {
      let inner = self.parse_expr()?;
      Ok(Expr::Quoted { inner: Box::new(inner), span })
  }
  ```
  - File: `src/pratt.rs`
  - Est: 1 hour
  - Depends: LEXER-Q1, AST-Q1

- [ ] **TYPE-Q1**: Add Quoted<T> type wrapper
  ```rust
  // src/typechecker.rs
  Type::Quoted(inner_type) => {
      // Quoted expressions have type Quoted<T>
  }
  ```
  - File: `src/typechecker.rs`
  - Est: 2 hours
  - Depends: AST-Q1

- [ ] **CODEGEN-Q1**: Generate AST structs in Rust output
  - File: `src/codegen/rust.rs`
  - Est: 3 hours
  - Depends: TYPE-Q1

- [ ] **TEST-Q1**: Quote operator tests
  - File: `tests/quote_tests.rs` (new)
  - Count: 20 tests minimum
  - Categories: Simple, Nested, Types, Errors

---

## Sprint 2: Eval Operator (Week 3-4)

### Tasks

- [ ] **LEXER-E1**: Add `Eval` token (reuse `!` with context)
  ```rust
  // src/lexer.rs - Note: ! already exists for Not
  // Need context-aware parsing in parser
  ```
  - File: `src/lexer.rs`
  - Est: 1 hour (handle ambiguity with Not)

- [ ] **AST-E1**: Add Eval expression variant
  ```rust
  // src/ast.rs
  Eval {
      quoted: Box<Expr>,
      bindings: Option<HashMap<String, Expr>>,
      span: Span,
  },
  ```
  - File: `src/ast.rs`
  - Est: 30 min

- [ ] **PARSE-E1**: Parse eval with optional `where` clause
  ```rust
  // Syntax: !expr or !expr where { a = 1, b = 2 }
  ```
  - File: `src/parser.rs`
  - Est: 2 hours
  - Depends: AST-E1

- [ ] **INTERP-E1**: Create AST interpreter module
  ```rust
  // src/interpreter/mod.rs (NEW MODULE)
  pub fn eval(expr: &Expr, env: &Environment) -> Result<Value, EvalError>
  ```
  - Files: `src/interpreter/mod.rs`, `src/interpreter/env.rs`
  - Est: 8 hours
  - Depends: Full AST support

- [ ] **TYPE-E1**: Unwrap Quoted<T> → T for eval
  ```rust
  // If e : Quoted<T>, then !e : T
  ```
  - File: `src/typechecker.rs`
  - Est: 2 hours
  - Depends: TYPE-Q1

- [ ] **TEST-E1**: Eval operator tests
  - File: `tests/eval_tests.rs` (new)
  - Count: 25 tests minimum
  - Categories: Simple, Bindings, Errors, Quote+Eval

---

## Sprint 3: Macro Infrastructure (Week 5-6)

### Tasks

- [ ] **LEXER-M1**: Add `MacroInvoke` token
  ```rust
  #[token("#")]
  MacroInvoke,
  ```
  - File: `src/lexer.rs`
  - Est: 30 min

- [ ] **AST-M1**: Add Macro declaration and call variants
  ```rust
  // Declarations
  Decl::Macro {
      name: String,
      params: Vec<MacroParam>,
      body: Box<Expr>,
  }
  
  // Expressions  
  Expr::MacroCall {
      name: String,
      args: Vec<Expr>,
  }
  ```
  - File: `src/ast.rs`
  - Est: 1 hour

- [ ] **PARSE-M1**: Parse macro definitions
  ```rust
  // macro name(param: Type, ...) -> Type { body }
  ```
  - File: `src/parser.rs`
  - Est: 3 hours

- [ ] **PARSE-M2**: Parse macro invocations
  ```rust
  // #macro_name(arg1, arg2, ...)
  // #macro_name { block }
  ```
  - File: `src/parser.rs`
  - Est: 2 hours
  - Depends: PARSE-M1

- [ ] **MACRO-M1**: Create macro expansion engine
  - Files: `src/macros/mod.rs`, `src/macros/expand.rs`
  - Est: 8 hours
  - Depends: Quote+Eval working

- [ ] **MACRO-M2**: Implement hygiene system
  - File: `src/macros/hygiene.rs`
  - Est: 4 hours
  - Depends: MACRO-M1

---

## Sprint 4: Built-in Macros Set 1 (Week 7-8)

### Tasks (5 tests each)

- [ ] **BUILTIN-01**: `#stringify(expr)` → String
- [ ] **BUILTIN-02**: `#concat(a, b, ...)` → String
- [ ] **BUILTIN-03**: `#format(fmt, ...)` → String
- [ ] **BUILTIN-04**: `#assert(cond)` → Void
- [ ] **BUILTIN-05**: `#assert_eq(a, b)` → Void
- [ ] **BUILTIN-06**: `#dbg(expr)` → T (prints and returns)
- [ ] **BUILTIN-07**: `#file()` → String
- [ ] **BUILTIN-08**: `#line()` → UInt32
- [ ] **BUILTIN-09**: `#column()` → UInt32
- [ ] **BUILTIN-10**: `#module_path()` → String

**All in:** `src/macros/builtin.rs`  
**Tests in:** `tests/macro_builtin_tests.rs`

---

## Sprint 5: Built-in Macros Set 2 (Week 9-10)

### Tasks (5 tests each)

- [ ] **BUILTIN-11**: `#cfg(condition)` → Conditional compilation
- [ ] **BUILTIN-12**: `#derive(Trait, ...)` → Trait implementation
- [ ] **BUILTIN-13**: `#env("VAR")` → String (compile-time)
- [ ] **BUILTIN-14**: `#option_env("VAR")` → Option<String>
- [ ] **BUILTIN-15**: `#todo(msg)` → panic placeholder
- [ ] **BUILTIN-16**: `#unreachable()` → never type
- [ ] **BUILTIN-17**: `#compile_error(msg)` → compile-time error
- [ ] **BUILTIN-18**: `#vec(a, b, c)` → Vec<T>
- [ ] **BUILTIN-19**: `#include(path)` → String (file contents)
- [ ] **BUILTIN-20**: `#assert_ne(a, b)` → Void

---

## Sprint 6: Reflect Operator (Week 11-12)

### Tasks

- [ ] **AST-R1**: Add Reflect expression
  ```rust
  Expr::Reflect {
      target: Box<Expr>,  // ?T or ?value
      span: Span,
  }
  ```

- [ ] **TYPE-R1**: Define TypeInfo gene in DOL stdlib
  ```dol
  // stdlib/reflect.dol
  pub gene TypeInfo {
      has name: String
      has module_path: String
      has fields: List<FieldInfo>
      has methods: List<MethodInfo>
      has constraints: List<String>
      has traits: List<String>
      has exegesis: String
  }
  ```

- [ ] **COMPILE-R1**: Generate TypeInfo at compile time
  - File: `src/typechecker.rs`
  - Insert TypeInfo generation during type checking

- [ ] **CODEGEN-R1**: Emit TypeInfo structs in Rust
  - File: `src/codegen/rust.rs`
  - Generate `impl TypeInfo for T { ... }`

- [ ] **TEST-R1**: Reflect operator tests
  - File: `tests/reflect_tests.rs`
  - Count: 30 tests minimum

---

## Sprint 7: Biological Primitives (Week 13-14)

### Tasks

- [ ] **BIO-01**: Define Hyphal trait in stdlib
  ```dol
  // stdlib/biology/hyphal.dol
  trait Hyphal { ... }
  ```

- [ ] **BIO-02**: Define Nutrient gene
  ```dol
  // stdlib/biology/nutrient.dol
  gene Nutrient { ... }
  ```

- [ ] **BIO-03**: Define Transport trait
  ```dol
  // stdlib/biology/transport.dol
  trait Transport<T> { ... }
  ```

- [ ] **BIO-04**: Define Ecosystem system
  ```dol
  // stdlib/biology/ecosystem.dol
  system Ecosystem { ... }
  ```

- [ ] **BIO-05**: Add evolves visualization
  - Generate Mermaid diagrams from evolves chains

---

## Sprint 8: Integration & Documentation (Week 15-16)

### Tasks

- [ ] **INT-01**: Meta-programming integration tests
  - Quote + Eval + Macro + Reflect working together
  - 50+ integration test cases

- [ ] **INT-02**: Update README with Q2 features

- [ ] **INT-03**: Write meta-programming guide

- [ ] **INT-04**: Update CHANGELOG

- [ ] **INT-05**: Performance benchmarks
  - Macro expansion time
  - Reflection overhead
  - Compilation time with meta-programming

---

## Verification Commands

```bash
# Run all tests
cargo test

# Run specific test module
cargo test quote
cargo test eval
cargo test macro
cargo test reflect

# Check coverage
cargo tarpaulin --out Html

# Benchmark
cargo bench

# Full validation
cargo test && cargo clippy && cargo fmt --check
```

---

## Dependencies Graph

```
LEXER-Q1 ──┬─► PARSE-Q1 ──► TYPE-Q1 ──► CODEGEN-Q1
           │
AST-Q1 ────┘
           
LEXER-E1 ──┬─► PARSE-E1 ──► INTERP-E1 ──► TYPE-E1
           │
AST-E1 ────┘

LEXER-M1 ──┬─► PARSE-M1 ──► PARSE-M2 ──► MACRO-M1 ──► MACRO-M2
           │
AST-M1 ────┘

TYPE-Q1 + INTERP-E1 + MACRO-M1 ──► BUILTIN-* 

AST-R1 ──► TYPE-R1 ──► COMPILE-R1 ──► CODEGEN-R1

ALL ──► BIO-* ──► INT-*
```

---

## Daily Standup Template

```markdown
## Date: YYYY-MM-DD

### Completed
- [ Task ID ]: Description

### In Progress  
- [ Task ID ]: Description (X% complete)

### Blocked
- [ Task ID ]: Blocker description

### Next
- [ Task ID ]: Next priority task
```

---

## Definition of Done

Each task is complete when:

1. ✅ Code compiles without warnings
2. ✅ All new tests pass
3. ✅ No regressions (existing tests still pass)
4. ✅ Code formatted (`cargo fmt`)
5. ✅ Clippy clean (`cargo clippy`)
6. ✅ Documentation updated (if user-facing)

---

*Ready for Claude-flow orchestration*
