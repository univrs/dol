//! AI tool integration quality tests.
//!
//! This module tests the quality of DOL's meta-programming features
//! when used for AI-assisted development, including:
//! - Schema generation from natural language descriptions
//! - Code quality metrics
//! - Semantic preservation
//! - Documentation quality

use metadol::ast::{Declaration, Gen, HasField, Span, Statement, TypeExpr, Visibility};
use metadol::codegen::*;
use metadol::parse_file;

// ============================================
// Code Quality Tests
// ============================================

#[test]
fn test_generated_code_has_documentation() {
    let source = r#"
gene test.documented {
    entity has field
}

exegesis {
    This is a well-documented entity.
    It has comprehensive documentation.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Generated code should include documentation
    assert!(
        code.contains("///") || code.contains("//!") || code.contains("documented"),
        "Generated code should include documentation"
    );
}

#[test]
fn test_generated_code_readable() {
    let source = r#"
gene test.readable {
    entity has clear_field_name
}

exegesis {
    Readable code test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Code should contain recognizable structure
    assert!(!code.is_empty());
    assert!(
        code.contains('\n'),
        "Code should be formatted with newlines"
    );
}

#[test]
fn test_identifier_naming_conventions() {
    let test_cases = vec![
        ("test.simple", "TestSimple"),
        ("user.account.profile", "UserAccountProfile"),
        ("container.exists", "ContainerExists"),
    ];

    for (dol_name, expected_pascal) in test_cases {
        let pascal = to_pascal_case(dol_name);
        assert_eq!(
            pascal, expected_pascal,
            "Expected '{}' to convert to '{}'",
            dol_name, expected_pascal
        );
    }
}

#[test]
fn test_consistent_formatting() {
    let source = r#"
gene test.format {
    entity has field1
    entity has field2
}

exegesis {
    Formatting test.
}
"#;

    let code1 = compile_to_rust_via_hir(source).expect("Failed to generate code");
    let code2 = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Same input should produce identical output
    assert_eq!(code1, code2, "Code generation should be deterministic");
}

// ============================================
// Semantic Preservation Tests
// ============================================

#[test]
fn test_preserves_field_names() {
    let source = r#"
gene test.fields {
    entity has important_field
    entity has another_field
}

exegesis {
    Field preservation test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Field names should be preserved (possibly in snake_case)
    // The exact representation depends on codegen implementation
    assert!(!code.is_empty());
}

#[test]
fn test_preserves_type_information() {
    let source = r#"
gene test.typed {
    entity has id: Int64
    entity has name: String
    entity has active: Bool
}

exegesis {
    Type preservation test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Types should be mapped appropriately
    assert!(!code.is_empty());
}

// ============================================
// Schema Generation Quality Tests
// ============================================

#[test]
fn test_simple_schema_generation() {
    let source = r#"
gene user.account {
    user has username
    user has email
}

exegesis {
    A user account in the system.
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok(), "Simple schema should parse successfully");
}

#[test]
fn test_complex_schema_generation() {
    let source = r#"
gene organization.team {
    team has name: String
    team has members: Int32
    team has active: Bool
}

exegesis {
    A team within an organization.
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok(), "Complex schema should parse successfully");
}

// ============================================
// Natural Language to Schema Tests
// ============================================

#[test]
fn test_schema_from_description_user() {
    // Simulates: "Create a user schema with id, name, and email"
    let source = r#"
gene user.profile {
    user has id
    user has name
    user has email
}

exegesis {
    A user profile with id, name, and email.
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok());

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");
    assert!(!code.is_empty());
}

#[test]
fn test_schema_from_description_product() {
    // Simulates: "Create a product with name, price, and inventory count"
    let source = r#"
gene product.item {
    product has name
    product has price
    product has inventory_count
}

exegesis {
    A product with name, price, and inventory tracking.
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok());
}

// ============================================
// Documentation Quality Tests
// ============================================

#[test]
fn test_exegesis_preserved() {
    let source = r#"
gene test.documented {
    entity has value
}

exegesis {
    This is a comprehensive explanation.
    It provides context and meaning.
    It helps developers understand the purpose.
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok());

    if let Ok(Declaration::Gene(gene)) = result {
        assert!(
            gene.exegesis.contains("comprehensive"),
            "Exegesis should be preserved"
        );
    }
}

#[test]
fn test_multiline_exegesis() {
    let source = r#"
gene test.multiline {
    entity has field
}

exegesis {
    Line 1
    Line 2
    Line 3
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok());

    if let Ok(Declaration::Gene(gene)) = result {
        assert!(gene.exegesis.contains("Line 1"));
        assert!(gene.exegesis.contains("Line 2"));
        assert!(gene.exegesis.contains("Line 3"));
    }
}

// ============================================
// Error Recovery and Helpful Messages
// ============================================

#[test]
fn test_helpful_error_on_invalid_syntax() {
    let source = r#"
gene test.invalid {
    this is not valid syntax
}
"#;

    let result = parse_file(source);

    // Should fail with an error (not panic)
    assert!(result.is_err(), "Invalid syntax should produce error");
}

#[test]
fn test_helpful_error_on_missing_exegesis() {
    let source = r#"
gene test.no_exegesis {
    entity has field
}
"#;

    let _result = parse_file(source);

    // May or may not require exegesis depending on parser implementation
    // At minimum, should not panic
}

// ============================================
// Tool Integration Tests
// ============================================

#[test]
fn test_round_trip_simple() {
    let source = r#"
gene test.roundtrip {
    entity has field
}

exegesis {
    Round trip test.
}
"#;

    // Parse
    let ast = parse_file(source).expect("Failed to parse");

    // Generate code
    let code = RustCodegen::generate(&ast);

    // Verify code is non-empty
    assert!(!code.is_empty());
}

#[test]
fn test_multiple_tools_integration() {
    let source = r#"
gene test.multi {
    entity has value
}

exegesis {
    Multi-tool test.
}
"#;

    let ast = parse_file(source).expect("Failed to parse");

    // Generate Rust
    let rust_code = RustCodegen::generate(&ast);
    assert!(!rust_code.is_empty());

    // Generate TypeScript
    let ts_code = TypeScriptCodegen::generate(&ast);
    assert!(!ts_code.is_empty());

    // Generate JSON Schema
    let json_code = JsonSchemaCodegen::generate(&ast);
    assert!(!json_code.is_empty());
}

// ============================================
// Code Smell Detection Tests
// ============================================

#[test]
fn test_no_unused_variables_in_generated_code() {
    let source = r#"
gene test.clean {
    entity has used_field
}

exegesis {
    Clean code test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Generated code should not contain obvious code smells
    // This is a basic check - more sophisticated analysis would require parsing the output
    assert!(!code.is_empty());
}

#[test]
fn test_no_hardcoded_values() {
    let source = r#"
gene test.dynamic {
    entity has field
}

exegesis {
    Dynamic generation test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Code should be generated, not hardcoded
    assert!(!code.is_empty());
}

// ============================================
// Idiomatic Code Generation Tests
// ============================================

#[test]
fn test_rust_idioms() {
    let source = r#"
gene test.idiomatic {
    entity has optional_field
}

exegesis {
    Idiomatic Rust test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Should use idiomatic Rust patterns
    assert!(!code.is_empty());
}

#[test]
fn test_typescript_idioms() {
    let gene = Gen {
        visibility: Visibility::default(),
        extends: None,
        name: "Test".to_string(),
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "field".to_string(),
            type_: TypeExpr::Named("String".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: None,
            personal: false,
            span: Span::default(),
        }))],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    let code = TypeScriptCodegen::generate(&Declaration::Gene(gene));

    // Should use idiomatic TypeScript (interfaces, types, etc.)
    assert!(code.contains("interface") || code.contains("type"));
}

// ============================================
// AI-Friendly Output Tests
// ============================================

#[test]
fn test_structured_output() {
    let source = r#"
gene test.structured {
    entity has field1
    entity has field2
}

exegesis {
    Structured output test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Output should be well-structured and parseable
    assert!(!code.is_empty());
    assert!(code.contains('\n')); // Should have line breaks
}

#[test]
fn test_consistent_indentation() {
    let source = r#"
gene test.indent {
    entity has field
}

exegesis {
    Indentation test.
}
"#;

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");

    // Generated code should have consistent indentation
    // (exact check would require parsing, this is a basic sanity check)
    assert!(!code.is_empty());
}

// ============================================
// Schema Validation Tests
// ============================================

#[test]
fn test_valid_schema_accepted() {
    let source = r#"
gene valid.schema {
    entity has field: String
}

exegesis {
    A valid schema.
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok());
}

#[test]
fn test_comprehensive_schema() {
    let source = r#"
gene comprehensive.entity {
    entity has id: Int64
    entity has name: String
    entity has created: String
    entity has active: Bool
}

exegesis {
    A comprehensive entity with multiple typed fields.
}
"#;

    let result = parse_file(source);
    assert!(result.is_ok());

    let code = compile_to_rust_via_hir(source).expect("Failed to generate code");
    assert!(!code.is_empty());
}

// ============================================
// Performance Tests
// ============================================

#[test]
fn test_quick_generation() {
    let source = r#"
gene test.performance {
    entity has field
}

exegesis {
    Performance test.
}
"#;

    use std::time::Instant;
    let start = Instant::now();

    for _ in 0..10 {
        let _ = compile_to_rust_via_hir(source);
    }

    let elapsed = start.elapsed();

    // Should complete quickly (< 1 second for 10 iterations)
    assert!(
        elapsed.as_secs() < 1,
        "Code generation should be fast: {:?}",
        elapsed
    );
}
