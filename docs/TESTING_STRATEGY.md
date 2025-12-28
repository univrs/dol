# DOL Testing Strategy

> **Complete Testing at Every Step**
> *If it's not tested, it's not done.*

---

## Testing Philosophy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│    "Self-hosting requires absolute confidence in every transformation"      │
│                                                                             │
│    DOL Source → Parse → AST → Desugar → HIR → Validate → Emit → Target    │
│         ↑         ↑       ↑       ↑        ↑       ↑        ↑       ↑       │
│        [T]       [T]     [T]     [T]      [T]     [T]      [T]     [T]      │
│                                                                             │
│    Every arrow is a transformation. Every transformation has tests.         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Test Categories

### 1. Unit Tests

Test individual functions in isolation.

```rust
// tests/hir/types_test.rs

#[test]
fn hir_type_from_gene() {
    let gene = parse_gene("gene Foo { has x: Int64 }");
    let hir = desugar_gene(&gene);
    
    assert!(matches!(hir, HirType::Struct { .. }));
    assert_eq!(hir.name(), "Foo");
    assert_eq!(hir.fields().len(), 1);
}

#[test]
fn hir_binding_from_let() {
    let stmt = parse_stmt("let x = 42");
    let hir = desugar_stmt(&stmt);
    
    assert!(matches!(hir, HirStmt::Binding { .. }));
    assert_eq!(hir.mutable(), false);
}
```

### 2. Property Tests

Test invariants that must always hold.

```rust
// tests/hir/properties_test.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn desugar_preserves_semantics(source in valid_dol_source()) {
        let ast = parse(&source).unwrap();
        let hir = desugar(&ast).unwrap();
        
        // Semantic equivalence: AST and HIR produce same output
        let ast_result = interpret_ast(&ast);
        let hir_result = interpret_hir(&hir);
        
        prop_assert_eq!(ast_result, hir_result);
    }
    
    #[test]
    fn roundtrip_parse_format(source in valid_dol_source()) {
        let ast1 = parse(&source).unwrap();
        let formatted = format_ast(&ast1);
        let ast2 = parse(&formatted).unwrap();
        
        prop_assert_eq!(ast1, ast2);
    }
}
```

### 3. Snapshot Tests

Test that output matches expected results.

```rust
// tests/hir/snapshots_test.rs

#[test]
fn snapshot_gene_desugar() {
    let source = include_str!("fixtures/gene_basic.dol");
    let hir = parse_and_desugar(source).unwrap();
    
    insta::assert_snapshot!(format_hir(&hir));
}

#[test]
fn snapshot_function_desugar() {
    let source = include_str!("fixtures/function_basic.dol");
    let hir = parse_and_desugar(source).unwrap();
    
    insta::assert_snapshot!(format_hir(&hir));
}
```

### 4. Integration Tests

Test multiple components together.

```rust
// tests/integration/pipeline_test.rs

#[test]
fn full_pipeline_simple_gene() {
    let source = r#"
        gene Counter {
            has value: Int64
            
            fun increment() -> Int64 {
                return self.value + 1
            }
        }
    "#;
    
    // Parse
    let ast = parse(source).expect("parse failed");
    
    // Desugar
    let hir = desugar(&ast).expect("desugar failed");
    
    // Validate
    validate(&hir).expect("validation failed");
    
    // Emit Rust
    let rust = emit_rust(&hir).expect("emit failed");
    
    // Compile Rust
    let compiled = compile_rust(&rust).expect("compile failed");
    
    assert!(compiled.success());
}
```

### 5. End-to-End Tests

Test the complete system.

```rust
// tests/e2e/bootstrap_test.rs

#[test]
fn bootstrap_types_module() {
    let result = Command::new("cargo")
        .args(["run", "--bin", "dol-codegen", "--", 
               "--target", "rust", "dol/types.dol"])
        .output()
        .expect("failed to run codegen");
    
    assert!(result.status.success());
    
    let rust_code = String::from_utf8(result.stdout).unwrap();
    assert!(rust_code.contains("pub struct"));
}

#[test]
fn bootstrap_compiles_without_fixes() {
    // This test tracks our progress toward eliminating the fix script
    let result = Command::new("make")
        .args(["regen"])
        .output()
        .expect("failed to regenerate");
    
    let check = Command::new("cargo")
        .args(["check"])
        .current_dir("target/bootstrap")
        .output()
        .expect("failed to check");
    
    let errors = count_errors(&check.stderr);
    
    // Track error count - should decrease over time
    println!("Bootstrap errors (raw): {}", errors);
    
    // Eventually this should pass:
    // assert_eq!(errors, 0, "Bootstrap should compile without fixes");
}
```

---

## Test Organization

```
tests/
├── unit/
│   ├── lexer/
│   │   ├── tokens_test.rs
│   │   ├── spans_test.rs
│   │   └── keywords_test.rs
│   ├── parser/
│   │   ├── expressions_test.rs
│   │   ├── statements_test.rs
│   │   └── declarations_test.rs
│   ├── hir/
│   │   ├── types_test.rs
│   │   ├── desugar_test.rs
│   │   └── validate_test.rs
│   └── codegen/
│       ├── rust_test.rs
│       └── wasm_test.rs
├── integration/
│   ├── pipeline_test.rs
│   ├── error_handling_test.rs
│   └── diagnostics_test.rs
├── e2e/
│   ├── bootstrap_test.rs
│   ├── self_host_test.rs
│   └── cli_test.rs
├── fixtures/
│   ├── valid/
│   │   ├── gene_basic.dol
│   │   ├── gene_complex.dol
│   │   ├── trait_basic.dol
│   │   └── ...
│   ├── invalid/
│   │   ├── syntax_error.dol
│   │   ├── type_error.dol
│   │   └── ...
│   └── snapshots/
│       └── ...
└── properties/
    ├── desugar_properties.rs
    └── roundtrip_properties.rs
```

---

## Test Fixtures for HIR

### Valid DOL Fixtures

```dol
// fixtures/valid/gene_basic.dol
gene Counter {
    has value: Int64
}

// fixtures/valid/gene_with_methods.dol
gene Counter {
    has value: Int64
    
    fun increment() -> Int64 {
        return self.value + 1
    }
    
    fun reset() {
        self.value = 0
    }
}

// fixtures/valid/function_control_flow.dol
fun fibonacci(n: Int64) -> Int64 {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

// fixtures/valid/pattern_matching.dol
fun describe(opt: Option<Int64>) -> String {
    match opt {
        Some(x) { return "has value: " + x.to_string() }
        None { return "empty" }
    }
}

// fixtures/valid/pipe_operators.dol
fun process(x: Int64) -> Int64 {
    return x |> double >> increment |> square
}
```

### Invalid DOL Fixtures (Error Cases)

```dol
// fixtures/invalid/syntax_error.dol
gene Broken {
    has value Int64  // Missing colon
}

// fixtures/invalid/type_error.dol
fun bad() -> Int64 {
    return "string"  // Type mismatch
}

// fixtures/invalid/undefined_variable.dol
fun bad() -> Int64 {
    return undefined_var  // Undefined
}
```

---

## HIR-Specific Test Cases

### Desugaring Tests

| DOL Input | Expected HIR |
|-----------|--------------|
| `gene Foo { has x: Int64 }` | `HirStruct { name: "Foo", fields: [...] }` |
| `let x = 42` | `HirBinding { name: "x", mutable: false, value: HirLiteral(42) }` |
| `var y = 0` | `HirBinding { name: "y", mutable: true, value: HirLiteral(0) }` |
| `fun f() { }` | `HirBinding { name: "f", value: HirLambda { ... } }` |
| `for x in xs { }` | `HirLoop { kind: ForIn, ... }` |
| `while cond { }` | `HirLoop { kind: While, ... }` |
| `a \|> f` | `HirCall { callee: "f", args: [a] }` |
| `f >> g` | `HirLambda { body: HirCall { g, [HirCall { f, [x] }] } }` |

### Validation Tests

```rust
#[test]
fn validate_rejects_undefined_variable() {
    let hir = desugar(parse("fun f() { return x }").unwrap()).unwrap();
    let result = validate(&hir);
    
    assert!(matches!(result, Err(ValidationError::UndefinedVariable { name: "x", .. })));
}

#[test]
fn validate_rejects_type_mismatch() {
    let hir = desugar(parse(r#"fun f() -> Int64 { return "str" }"#).unwrap()).unwrap();
    let result = validate(&hir);
    
    assert!(matches!(result, Err(ValidationError::TypeMismatch { .. })));
}

#[test]
fn validate_accepts_valid_program() {
    let hir = desugar(parse("fun f(x: Int64) -> Int64 { return x + 1 }").unwrap()).unwrap();
    let result = validate(&hir);
    
    assert!(result.is_ok());
}
```

---

## Coverage Requirements

### Per-File Coverage

| File | Min Coverage | Notes |
|------|--------------|-------|
| `src/hir/types.rs` | 95% | Core types must be fully tested |
| `src/hir/desugar.rs` | 95% | Every rule tested |
| `src/hir/validate.rs` | 90% | All error paths tested |
| `src/codegen/rust.rs` | 85% | Main paths tested |

### Running Coverage

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage/

# Check coverage in CI
cargo tarpaulin --fail-under 80
```

---

## Test Commands

```bash
# Run all tests
cargo test

# Run specific test module
cargo test hir::

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_gene_desugar

# Run property tests
cargo test properties::

# Update snapshots
cargo insta review

# Run with coverage
cargo tarpaulin

# Run E2E tests only
cargo test --test e2e

# Run fast tests only (exclude slow integration)
cargo test --lib
```

---

## CI Test Matrix

```yaml
test:
  strategy:
    matrix:
      rust: [stable, beta, nightly]
      os: [ubuntu-latest, macos-latest]
  steps:
    - cargo test --lib          # Unit tests
    - cargo test --test integration  # Integration
    - cargo test --test e2e     # E2E (ubuntu only)
    - cargo tarpaulin           # Coverage (ubuntu only)
```

---

## Test-Driven Development Flow

```
1. Write failing test for new HIR feature
2. Implement minimal code to pass
3. Refactor while keeping tests green
4. Add property tests for invariants
5. Add snapshot tests for outputs
6. Update coverage requirements
7. Document in test file
```

---

*"If it's not tested, it's not done."*
