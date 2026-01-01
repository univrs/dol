//! End-to-end compiler tests for the DOL compilation pipeline.
//!
//! These tests verify the complete pipeline:
//! DOL Source → Lexer → Parser → AST → HIR → MLIR → WASM
//!
//! Tests are organized by:
//! 1. Basic parsing tests (DOL → AST)
//! 2. HIR lowering tests (AST → HIR)
//! 3. Full compilation pipeline tests (when wasm feature enabled)
//! 4. Error handling tests

use metadol::{parse_dol_file, parse_file, parse_file_all};

// ============================================
// 1. Basic Parsing Tests
// ============================================

#[test]
fn test_parse_simple_gene() {
    let source = r#"
gene Counter {
    has value: Int64
}
exegesis { A counter. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse simple gene: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Counter");
}

#[test]
fn test_parse_gene_with_multiple_fields() {
    let source = r#"
gene Person {
    has name: String
    has age: Int32
    has email: String
}
exegesis { A person with basic attributes. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse gene with multiple fields: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Person");
}

#[test]
fn test_parse_function() {
    let source = r#"
fun add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
exegesis { Adds two integers. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse function: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_gene_with_function() {
    let source = r#"
gene Calculator {
    has result: Int64

    fun add(a: Int64, b: Int64) -> Int64 {
        return a + b
    }

    fun multiply(a: Int64, b: Int64) -> Int64 {
        return a * b
    }
}
exegesis { A calculator gene with arithmetic methods. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse gene with function: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Calculator");
}

#[test]
fn test_parse_trait_declaration() {
    let source = r#"
trait Lifecycle {
    uses Container.exists
    entity is created
    entity is started
    entity is stopped
}
exegesis { Basic lifecycle trait. }
"#;
    let result = parse_file(source);
    assert!(result.is_ok(), "Failed to parse trait: {:?}", result.err());

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Lifecycle");
}

#[test]
fn test_parse_constraint_declaration() {
    let source = r#"
constraint Integrity {
    state matches declared_state
    identity never changes
}
exegesis { Integrity constraint. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse constraint: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Integrity");
}

#[test]
fn test_parse_system_declaration() {
    let source = r#"
system Orchestrator @ 0.1.0 {
    requires container.lifecycle >= 0.0.2
    requires node.discovery >= 0.0.1
}
exegesis { Orchestrator system. }
"#;
    let result = parse_file(source);
    assert!(result.is_ok(), "Failed to parse system: {:?}", result.err());

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Orchestrator");
}

#[test]
fn test_parse_multiple_declarations() {
    let source = r#"
gene Counter {
    has value: Int64
}

fun increment(c: Counter) -> Counter {
    return c
}
"#;
    let result = parse_file_all(source);
    assert!(
        result.is_ok(),
        "Failed to parse multiple declarations: {:?}",
        result.err()
    );

    let decls = result.unwrap();
    assert_eq!(decls.len(), 2, "Expected 2 declarations");
    assert_eq!(decls[0].name(), "Counter");
    assert_eq!(decls[1].name(), "increment");
}

#[test]
fn test_parse_with_module_declaration() {
    let source = r#"
module my.test.module @ 1.0.0

gene TestGene {
    has property: String
}
exegesis { A test gene in a module. }
"#;
    let result = parse_dol_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse with module declaration: {:?}",
        result.err()
    );

    let file = result.unwrap();
    assert!(file.module.is_some());
    assert_eq!(file.declarations.len(), 1);
}

#[test]
fn test_parse_with_use_declarations() {
    let source = r#"
module my.module

use std.io
use std.collections.{HashMap, HashSet}

gene Data {
    has values: HashMap
}
exegesis { Data with collections. }
"#;
    let result = parse_dol_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse with use declarations: {:?}",
        result.err()
    );

    let file = result.unwrap();
    assert_eq!(file.uses.len(), 2, "Expected 2 use declarations");
}

// ============================================
// 2. HIR Lowering Tests
// ============================================

#[test]
fn test_lower_simple_gene_to_hir() {
    let source = r#"
gene Counter {
    has value: Int64
}
exegesis { A counter. }
"#;
    let result = metadol::lower::lower_file(source);
    assert!(
        result.is_ok(),
        "Failed to lower simple gene to HIR: {:?}",
        result.err()
    );

    let (_hir, ctx) = result.unwrap();
    assert!(
        !ctx.has_errors(),
        "Lowering produced errors: {:?}",
        ctx.diagnostics()
    );
    assert_eq!(_hir.decls.len(), 1, "Expected 1 HIR declaration");
}

#[test]
fn test_lower_trait_to_hir() {
    let source = r#"
trait Lifecycle {
    uses Container.exists
    entity is created
}
exegesis { Lifecycle trait. }
"#;
    let result = metadol::lower::lower_file(source);
    assert!(
        result.is_ok(),
        "Failed to lower trait to HIR: {:?}",
        result.err()
    );

    let (_hir, ctx) = result.unwrap();
    assert!(
        !ctx.has_errors(),
        "Lowering produced errors: {:?}",
        ctx.diagnostics()
    );
}

#[test]
fn test_lower_function_to_hir() {
    let source = r#"
fun add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
exegesis { Adds two integers. }
"#;
    let result = metadol::lower::lower_file(source);
    assert!(
        result.is_ok(),
        "Failed to lower function to HIR: {:?}",
        result.err()
    );

    let (_hir, ctx) = result.unwrap();
    assert!(
        !ctx.has_errors(),
        "Lowering produced errors: {:?}",
        ctx.diagnostics()
    );
}

#[test]
fn test_lower_complex_gene_to_hir() {
    let source = r#"
gene Calculator {
    has result: Int64

    fun add(a: Int64, b: Int64) -> Int64 {
        return a + b
    }
}
exegesis { Calculator with methods. }
"#;
    let result = metadol::lower::lower_file(source);
    assert!(
        result.is_ok(),
        "Failed to lower complex gene to HIR: {:?}",
        result.err()
    );

    let (_hir, ctx) = result.unwrap();
    assert!(
        !ctx.has_errors(),
        "Lowering produced errors: {:?}",
        ctx.diagnostics()
    );
}

#[test]
fn test_lower_with_module() {
    let source = r#"
module my.test.module

gene TestGene {
    entity has property
}
exegesis { Test gene. }
"#;
    let result = metadol::lower::lower_file(source);
    assert!(
        result.is_ok(),
        "Failed to lower with module: {:?}",
        result.err()
    );

    let (hir, ctx) = result.unwrap();
    assert!(
        !ctx.has_errors(),
        "Lowering produced errors: {:?}",
        ctx.diagnostics()
    );

    // Module name should be from module declaration
    let module_name = ctx.symbols.resolve(hir.name);
    assert_eq!(module_name, Some("my.test.module"));
}

// ============================================
// 3. Full Compilation Pipeline Tests (WASM)
// ============================================

#[cfg(feature = "wasm")]
#[test]
fn test_compile_simple_gene_to_wasm() {
    use metadol::wasm::WasmCompiler;

    let source = r#"
gene Counter {
    has value: Int64
}
exegesis { A counter. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let compiler = WasmCompiler::new();
    let result = compiler.compile(&module);

    // Currently returns error when compiling genes (not functions) - this is expected
    // When fully implemented, this should succeed or handle genes properly
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(
        err.message.contains("No functions found") || err.message.contains("not fully implemented"),
        "Expected function-related error, got: {}",
        err.message
    );
}

#[cfg(feature = "wasm")]
#[test]
fn test_compile_function_to_wasm() {
    use metadol::wasm::WasmCompiler;

    let source = r#"
fun add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
exegesis { Adds two integers. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let compiler = WasmCompiler::new()
        .with_optimization(true)
        .with_debug_info(false);

    let result = compiler.compile(&module);

    // Compilation should succeed now that Int32 is supported
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

    // Verify the output is valid WASM
    let wasm_bytes = result.unwrap();
    assert!(wasm_bytes.len() >= 8, "WASM output too short");
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Invalid WASM magic number");
}

#[cfg(feature = "wasm")]
#[test]
fn test_compiler_options() {
    use metadol::wasm::WasmCompiler;

    // Test that compiler can be created with various options
    let _compiler = WasmCompiler::new();

    // Test with optimization
    let _compiler = WasmCompiler::new().with_optimization(true);

    // Test with debug info disabled
    let _compiler = WasmCompiler::new().with_debug_info(false);

    // Test chaining
    let _compiler = WasmCompiler::new()
        .with_optimization(true)
        .with_debug_info(false);

    // All configurations should be accepted without errors
}

// ============================================
// 4. Error Handling Tests
// ============================================

#[test]
fn test_parse_invalid_syntax() {
    let source = r#"
gene InvalidGene {
    this is not valid syntax
}
"#;
    let result = parse_file(source);
    assert!(result.is_err(), "Should fail to parse invalid syntax");
}

#[test]
fn test_parse_missing_exegesis() {
    // DOL 2.0 is tolerant of missing exegesis - it defaults to empty string
    let source = r#"
gene Counter {
    has value: Int64
}
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Should parse gene without exegesis (DOL 2.0 tolerant)"
    );

    let decl = result.unwrap();
    assert_eq!(
        decl.exegesis(),
        "",
        "Missing exegesis should default to empty string"
    );
}

#[test]
fn test_parse_incomplete_declaration() {
    let source = r#"
gene Counter {
"#;
    let result = parse_file(source);
    assert!(
        result.is_err(),
        "Should fail to parse incomplete declaration"
    );
}

#[test]
fn test_parse_invalid_type_annotation() {
    let source = r#"
gene Counter {
    has value: InvalidType
}
exegesis { Invalid type. }
"#;
    let result = parse_file(source);
    // Parser should accept any identifier as a type name
    // Type validation happens in later stages
    assert!(
        result.is_ok(),
        "Parser should accept any identifier as type name"
    );
}

#[test]
fn test_parse_empty_source() {
    let source = "";
    let result = parse_file(source);
    assert!(result.is_err(), "Should fail to parse empty source");
}

#[test]
fn test_parse_whitespace_only() {
    let source = "   \n  \t  \n  ";
    let result = parse_file(source);
    assert!(
        result.is_err(),
        "Should fail to parse whitespace-only source"
    );
}

// ============================================
// 5. Complex Integration Tests
// ============================================

#[test]
fn test_parse_and_validate_gene() {
    use metadol::validate;

    let source = r#"
gene container.exists {
    container has identity
    container has status
}
exegesis { A container is the fundamental unit of workload isolation. }
"#;
    let result = parse_file(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let decl = result.unwrap();
    let validation = validate(&decl);
    assert!(
        validation.is_valid(),
        "Validation failed: {:?}",
        validation.errors
    );
}

#[test]
fn test_parse_gene_with_expressions() {
    let source = r#"
gene Calculator {
    has x: Int64
    has y: Int64

    fun compute() -> Int64 {
        let sum = x + y
        let product = x * y
        return sum + product
    }
}
exegesis { Calculator with expression-based computation. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse gene with expressions: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_gene_with_control_flow() {
    let source = r#"
gene Validator {
    has value: Int32

    fun validate() -> Bool {
        if value > 0 {
            return true
        }
        return false
    }
}
exegesis { Validator with control flow. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse gene with control flow: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_gene_with_pattern_matching() {
    let source = r#"
gene Matcher {
    fun classify(x: Int32) -> String {
        match x {
            0 => "zero",
            1 => "one",
            _ => "many",
        }
    }
}
exegesis { Matcher with pattern matching. }
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse gene with pattern matching: {:?}",
        result.err()
    );
}

#[test]
fn test_lower_multiple_declarations() {
    let source = r#"
module test.module

gene Counter {
    has value: Int64
}

fun increment(c: Counter) -> Counter {
    return c
}
"#;
    let result = metadol::lower::lower_file(source);
    assert!(
        result.is_ok(),
        "Failed to lower multiple declarations: {:?}",
        result.err()
    );

    let (_hir, ctx) = result.unwrap();
    assert!(
        !ctx.has_errors(),
        "Lowering produced errors: {:?}",
        ctx.diagnostics()
    );
    assert_eq!(_hir.decls.len(), 2, "Expected 2 HIR declarations");
}

// ============================================
// 6. Real-World Examples
// ============================================

#[test]
fn test_parse_container_example() {
    let source = r#"
gene container.exists {
    container has identity
    container has state
    container has boundaries
    container has resources
}

exegesis {
    A container is the fundamental unit of workload isolation.
    It has a unique identity, maintains state, enforces boundaries,
    and manages resources.
}
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse container example: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "container.exists");
}

#[test]
fn test_parse_lifecycle_example() {
    let source = r#"
trait container.lifecycle {
    uses container.exists

    container is created
    container is started
    container is running
    container is stopped
    container is destroyed

    each transition emits event
}

exegesis {
    The container lifecycle defines the state machine for container
    management. Each state transition is observable through events.
}
"#;
    let result = parse_file(source);
    assert!(
        result.is_ok(),
        "Failed to parse lifecycle example: {:?}",
        result.err()
    );
}

#[test]
fn test_end_to_end_pipeline() {
    let source = r#"
gene SimpleCounter {
    has count: Int64

    fun increment() -> Int64 {
        return count + 1
    }
}
exegesis { A simple counter implementation. }
"#;

    // Step 1: Parse to AST
    let ast = parse_file(source).expect("Parse failed");
    assert_eq!(ast.name(), "SimpleCounter");

    // Step 2: Lower to HIR
    let (hir, ctx) = metadol::lower::lower_file(source).expect("Lowering failed");
    assert!(!ctx.has_errors(), "HIR lowering produced errors");
    assert_eq!(hir.decls.len(), 1);

    // Step 3: Validate
    let validation = metadol::validate(&ast);
    assert!(validation.is_valid(), "Validation failed");

    // Step 4: WASM compilation (when feature is enabled)
    #[cfg(feature = "wasm")]
    {
        use metadol::wasm::WasmCompiler;
        let compiler = WasmCompiler::new();
        let _wasm_result = compiler.compile(&ast);
        // Currently returns NotImplemented - this is expected
    }
}
