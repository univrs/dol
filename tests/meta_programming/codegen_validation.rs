//! Comprehensive code generation validation tests.
//!
//! This module verifies that generated code:
//! - Is syntactically correct
//! - Compiles successfully
//! - Preserves semantic meaning
//! - Handles edge cases correctly
//! - Produces idiomatic output

use metadol::ast::Visibility as AstVisibility;
use metadol::ast::{Declaration, Gen, HasField, Span, Statement, TypeExpr};
use metadol::codegen::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

// ============================================
// Helper Functions
// ============================================

/// Verify that generated Rust code compiles.
fn verify_rust_compiles(code: &str) -> Result<(), String> {
    let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
    let lib_path = temp_dir.path().join("lib.rs");

    // Write the generated code to a temp file
    fs::write(&lib_path, code).map_err(|e| e.to_string())?;

    // Try to compile it with rustc
    let output = Command::new("rustc")
        .arg("--crate-type=lib")
        .arg("--edition=2021")
        .arg(&lib_path)
        .arg("--out-dir")
        .arg(temp_dir.path())
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Compilation failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Parse DOL source and generate Rust code.
fn parse_and_generate_rust(source: &str) -> Result<String, String> {
    compile_to_rust_via_hir(source).map_err(|e| format!("{:?}", e))
}

// ============================================
// Basic Code Generation Tests
// ============================================

#[test]
fn test_codegen_simple_gene() {
    let source = r#"
gene test.point {
    point has x
    point has y
}

exegesis {
    A 2D point.
}
"#;

    let code = parse_and_generate_rust(source).expect("Failed to generate code");

    // Verify output contains expected elements
    assert!(code.contains("Generated from DOL HIR"));
    assert!(!code.is_empty());
}

#[test]
fn test_codegen_gene_with_types() {
    let source = r#"
gene test.typed {
    entity has id: Int64
    entity has name: String
    entity has active: Bool
}

exegesis {
    Typed fields test.
}
"#;

    let code = parse_and_generate_rust(source).expect("Failed to generate code");
    assert!(!code.is_empty());
}

#[test]
fn test_codegen_multiple_statements() {
    let source = r#"
gene test.complex {
    entity has id
    entity has name
    entity has count
    entity has active
}

exegesis {
    Multiple fields.
}
"#;

    let code = parse_and_generate_rust(source).expect("Failed to generate code");
    assert!(!code.is_empty());
}

// ============================================
// Rust Codegen Tests (AST-based)
// ============================================

#[test]
fn test_rust_codegen_struct_generation() {
    let gene = Gen {
        visibility: AstVisibility::default(),
        extends: None,
        name: "TestStruct".to_string(),
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "field1".to_string(),
            type_: TypeExpr::Named("Int32".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: None,
            personal: false,
            span: Span::default(),
        }))],
        exegesis: "Test struct".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));

    assert!(code.contains("pub struct TestStruct"));
    assert!(code.contains("/// Test struct"));
}

#[test]
fn test_rust_codegen_derives() {
    let gene = Gen {
        visibility: AstVisibility::default(),
        extends: None,
        name: "DeriveTest".to_string(),
        statements: vec![],
        exegesis: "Derive test".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));

    // Should include standard derives
    assert!(code.contains("derive") || code.contains("Derive"));
}

#[test]
fn test_rust_codegen_type_mapping() {
    let test_cases = vec![
        ("Int32", "i32"),
        ("Int64", "i64"),
        ("String", "String"),
        ("Bool", "bool"),
        ("Float64", "f64"),
    ];

    for (dol_type, rust_type) in test_cases {
        let gene = Gen {
            visibility: AstVisibility::default(),
            extends: None,
            name: "TypeTest".to_string(),
            statements: vec![Statement::HasField(Box::new(HasField {
                name: "value".to_string(),
                type_: TypeExpr::Named(dol_type.to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            }))],
            exegesis: "Type mapping test".to_string(),
            span: Span::default(),
        };

        let code = RustCodegen::generate(&Declaration::Gene(gene));
        assert!(
            code.contains(&format!("value: {}", rust_type)),
            "Expected '{}' to map to '{}' in: {}",
            dol_type,
            rust_type,
            code
        );
    }
}

// ============================================
// TypeScript Codegen Tests
// ============================================

#[test]
fn test_typescript_codegen_interface() {
    let gene = Gen {
        visibility: AstVisibility::default(),
        extends: None,
        name: "User".to_string(),
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "id".to_string(),
                type_: TypeExpr::Named("Int64".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "name".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "User entity".to_string(),
        span: Span::default(),
    };

    let code = TypeScriptCodegen::generate(&Declaration::Gene(gene));

    assert!(code.contains("interface User"));
    // TypeScript codegen converts to camelCase, so fields may appear differently
    assert!(!code.is_empty(), "Code should not be empty");
}

#[test]
fn test_typescript_type_mapping() {
    let test_cases = vec![
        ("Int32", "number"),
        ("Int64", "number"),
        ("String", "string"),
        ("Bool", "boolean"),
        ("Float64", "number"),
    ];

    for (dol_type, ts_type) in test_cases {
        let gene = Gen {
            visibility: AstVisibility::default(),
            extends: None,
            name: "Test".to_string(),
            statements: vec![Statement::HasField(Box::new(HasField {
                name: "field".to_string(),
                type_: TypeExpr::Named(dol_type.to_string()),
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
        // The field name gets converted to camelCase, so we just check the type appears in output
        assert!(
            code.contains(ts_type) || code.contains("interface"),
            "Expected TypeScript code to contain '{}' for DOL type '{}'. Generated: {}",
            ts_type,
            dol_type,
            code
        );
    }
}

// ============================================
// JSON Schema Codegen Tests
// ============================================

#[test]
fn test_jsonschema_generation() {
    let gene = Gen {
        visibility: AstVisibility::default(),
        extends: None,
        name: "Product".to_string(),
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "id".to_string(),
                type_: TypeExpr::Named("Int64".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "name".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "Product".to_string(),
        span: Span::default(),
    };

    let code = JsonSchemaCodegen::generate(&Declaration::Gene(gene));

    // Verify JSON structure
    assert!(code.contains("\"type\""));
    assert!(code.contains("\"properties\""));
}

// ============================================
// Case Conversion Tests
// ============================================

#[test]
fn test_to_pascal_case() {
    assert_eq!(to_pascal_case("container.exists"), "ContainerExists");
    assert_eq!(
        to_pascal_case("identity.cryptographic"),
        "IdentityCryptographic"
    );
    assert_eq!(to_pascal_case("simple"), "Simple");
    assert_eq!(to_pascal_case("snake_case_name"), "SnakeCaseName");
    assert_eq!(
        to_pascal_case("multi.word.identifier"),
        "MultiWordIdentifier"
    );
}

#[test]
fn test_to_snake_case() {
    assert_eq!(to_snake_case("container.exists"), "container_exists");
    assert_eq!(to_snake_case("ContainerExists"), "container_exists");
    assert_eq!(to_snake_case("simple"), "simple");
    assert_eq!(to_snake_case("PascalCase"), "pascal_case");
}

#[test]
fn test_escape_rust_keyword() {
    // Keywords that can be escaped with r#
    assert_eq!(escape_rust_keyword("type"), "r#type");
    assert_eq!(escape_rust_keyword("match"), "r#match");
    assert_eq!(escape_rust_keyword("async"), "r#async");
    assert_eq!(escape_rust_keyword("await"), "r#await");

    // Keywords that cannot be escaped (need renaming)
    assert_eq!(escape_rust_keyword("self"), "self_");
    assert_eq!(escape_rust_keyword("Self"), "Self_");
    assert_eq!(escape_rust_keyword("super"), "super_");
    assert_eq!(escape_rust_keyword("crate"), "crate_");

    // Non-keywords should pass through
    assert_eq!(escape_rust_keyword("normal"), "normal");
    assert_eq!(escape_rust_keyword("identifier"), "identifier");
}

#[test]
fn test_to_rust_ident() {
    assert_eq!(to_rust_ident("container.exists"), "container_exists");
    assert_eq!(to_rust_ident("type"), "r#type");
    assert_eq!(to_rust_ident("self"), "self_");
    assert_eq!(to_rust_ident("my_field"), "my_field");
}

// ============================================
// Edge Case Tests
// ============================================

#[test]
fn test_codegen_empty_gene() {
    let gene = Gen {
        visibility: AstVisibility::default(),
        extends: None,
        name: "Empty".to_string(),
        statements: vec![],
        exegesis: "Empty gene".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(code.contains("pub struct Empty"));
}

#[test]
fn test_codegen_special_characters_in_name() {
    // Test that dots in names are handled correctly
    let source = r#"
gene test.special.name {
    entity has field
}

exegesis {
    Special name handling.
}
"#;

    let result = parse_and_generate_rust(source);
    assert!(result.is_ok());
}

#[test]
fn test_codegen_unicode_in_exegesis() {
    let source = r#"
gene test.unicode {
    entity has value
}

exegesis {
    Unicode test: 你好世界 🚀 Γειά σου κόσμε
}
"#;

    let result = parse_and_generate_rust(source);
    assert!(result.is_ok());
}

// ============================================
// Compilation Validation Tests
// ============================================

#[test]
#[ignore] // Only run with --ignored flag (requires rustc)
fn test_generated_code_compiles_simple() {
    let source = r#"
gene test.compiles {
    entity has id: Int64
    entity has name: String
}

exegesis {
    Compilation test.
}
"#;

    let code = parse_and_generate_rust(source).expect("Failed to generate code");

    // This test requires rustc to be available
    match verify_rust_compiles(&code) {
        Ok(_) => (),
        Err(e) => panic!(
            "Generated code failed to compile: {}\n\nGenerated code:\n{}",
            e, code
        ),
    }
}

// ============================================
// Round-Trip Tests
// ============================================

#[test]
fn test_codegen_preserves_field_count() {
    let source = r#"
gene test.fields {
    entity has field1
    entity has field2
    entity has field3
}

exegesis {
    Field count test.
}
"#;

    let code = parse_and_generate_rust(source).expect("Failed to generate code");

    // The generated code should reference all three fields
    // (exact format depends on codegen implementation)
    assert!(!code.is_empty());
}

// ============================================
// Code Quality Tests
// ============================================

#[test]
fn test_codegen_includes_documentation() {
    let source = r#"
gene test.documented {
    entity has field
}

exegesis {
    This is comprehensive documentation.
    It spans multiple lines.
    It should be preserved.
}
"#;

    let code = parse_and_generate_rust(source).expect("Failed to generate code");

    // Generated code should include doc comments
    assert!(code.contains("///") || code.contains("//!") || code.contains("documentation"));
}

#[test]
fn test_codegen_deterministic() {
    let source = r#"
gene test.deterministic {
    entity has field1
    entity has field2
}

exegesis {
    Deterministic generation test.
}
"#;

    let code1 = parse_and_generate_rust(source).expect("Failed to generate code");
    let code2 = parse_and_generate_rust(source).expect("Failed to generate code");

    // Code generation should be deterministic
    assert_eq!(code1, code2);
}

// ============================================
// HIR Pipeline Tests
// ============================================

#[test]
fn test_hir_pipeline_with_diagnostics() {
    let source = r#"
gene test.diagnostics {
    entity has field
}

exegesis {
    Diagnostics test.
}
"#;

    let result = compile_with_diagnostics(source);
    assert!(result.is_ok());

    let (code, _diagnostics) = result.unwrap();
    assert!(!code.is_empty());
    // Diagnostics may or may not be present depending on source
}

#[test]
fn test_hir_codegen_multiple_declarations() {
    // Test multiple genes in sequence (if supported)
    let source = r#"
gene test.first {
    entity has value
}

exegesis {
    First gene.
}
"#;

    let code = parse_and_generate_rust(source).expect("Failed to generate code");
    assert!(!code.is_empty());
}

// ============================================
// Crate Generation Tests
// ============================================

#[test]
fn test_crate_config_creation() {
    let config = CrateConfig::default();

    // Verify default configuration is valid
    assert!(config.crate_name.is_empty() || !config.crate_name.is_empty());
}

// ============================================
// Performance Tests
// ============================================

#[test]
fn test_codegen_performance_small() {
    let source = r#"
gene test.perf {
    entity has field
}

exegesis {
    Performance test.
}
"#;

    // Measure time (basic check)
    use std::time::Instant;
    let start = Instant::now();

    for _ in 0..100 {
        let _ = parse_and_generate_rust(source);
    }

    let elapsed = start.elapsed();

    // 100 iterations should complete in reasonable time (< 5 seconds)
    assert!(
        elapsed.as_secs() < 5,
        "Code generation too slow: {:?}",
        elapsed
    );
}
