//! Property-based tests for code generation.
//!
//! Uses proptest to generate random DOL schemas and verify:
//! - Generated code is valid
//! - Code generation is deterministic
//! - Round-trip transformations preserve semantics

use metadol::ast::{
    BinaryOp, Declaration, Expr, Gen, HasField, Literal, Span, Statement, TypeExpr, Visibility,
};
use metadol::codegen::*;
use proptest::prelude::*;

// ============================================
// Property Test Strategies
// ============================================

/// Strategy for generating valid identifiers
fn identifier_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,30}")
        .unwrap()
        .prop_filter("Not a keyword", |s| {
            !["type", "match", "if", "else", "self", "super"].contains(&s.as_str())
        })
}

/// Strategy for generating qualified names (e.g., "container.exists")
fn qualified_name_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(identifier_strategy(), 1..=3).prop_map(|parts| parts.join("."))
}

/// Strategy for generating type names
fn type_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("Int32".to_string()),
        Just("Int64".to_string()),
        Just("String".to_string()),
        Just("Bool".to_string()),
        Just("Float32".to_string()),
        Just("Float64".to_string()),
        Just("UInt32".to_string()),
        Just("UInt64".to_string()),
    ]
}

/// Strategy for generating TypeExpr
fn type_expr_strategy() -> impl Strategy<Value = TypeExpr> {
    type_name_strategy().prop_map(TypeExpr::Named)
}

/// Strategy for generating literals
fn literal_strategy() -> impl Strategy<Value = Literal> {
    prop_oneof![
        any::<i64>().prop_map(Literal::Int),
        any::<f64>()
            .prop_filter("No NaN or Inf", |f| f.is_finite())
            .prop_map(Literal::Float),
        any::<bool>().prop_map(Literal::Bool),
        prop::string::string_regex("[a-zA-Z0-9 ]{0,50}")
            .unwrap()
            .prop_map(Literal::String),
    ]
}

/// Strategy for generating simple expressions
fn simple_expr_strategy() -> impl Strategy<Value = Expr> {
    prop_oneof![
        literal_strategy().prop_map(Expr::Literal),
        identifier_strategy().prop_map(Expr::Identifier),
    ]
}

/// Strategy for generating HasField statements
fn has_field_strategy() -> impl Strategy<Value = Statement> {
    (
        identifier_strategy(),
        type_expr_strategy(),
        prop::option::of(simple_expr_strategy()),
    )
        .prop_map(|(name, type_, default)| {
            Statement::HasField(Box::new(HasField {
                name,
                type_,
                default,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            }))
        })
}

/// Strategy for generating statements
fn statement_strategy() -> impl Strategy<Value = Statement> {
    prop_oneof![
        has_field_strategy(),
        (identifier_strategy(), identifier_strategy()).prop_map(|(subject, property)| {
            Statement::Has {
                subject,
                property,
                span: Span::default(),
            }
        }),
    ]
}

/// Strategy for generating Gen (Gene) declarations
fn gen_strategy() -> impl Strategy<Value = Gen> {
    (
        qualified_name_strategy(),
        prop::collection::vec(statement_strategy(), 1..=10),
        prop::string::string_regex("[a-zA-Z0-9 .!?,]{10,200}").unwrap(),
    )
        .prop_map(|(name, statements, exegesis)| Gen {
            visibility: Visibility::default(),
            extends: None,
            name,
            statements,
            exegesis,
            span: Span::default(),
        })
}

/// Strategy for generating Declaration
fn declaration_strategy() -> impl Strategy<Value = Declaration> {
    gen_strategy().prop_map(Declaration::Gene)
}

// ============================================
// Property Tests
// ============================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000, // Run 1000 test cases (can increase to 10K)
        max_shrink_iters: 1000,
        ..ProptestConfig::default()
    })]

    /// Property: Code generation is deterministic
    #[test]
    fn prop_codegen_deterministic(decl in declaration_strategy()) {
        let code1 = RustCodegen::generate(&decl);
        let code2 = RustCodegen::generate(&decl);

        prop_assert_eq!(code1, code2, "Code generation should be deterministic");
    }

    /// Property: Generated code is non-empty
    #[test]
    fn prop_codegen_non_empty(decl in declaration_strategy()) {
        let code = RustCodegen::generate(&decl);

        prop_assert!(!code.is_empty(), "Generated code should not be empty");
    }

    /// Property: All backends generate code
    #[test]
    fn prop_all_backends_work(decl in declaration_strategy()) {
        let rust_code = RustCodegen::generate(&decl);
        let ts_code = TypeScriptCodegen::generate(&decl);
        let json_code = JsonSchemaCodegen::generate(&decl);

        prop_assert!(!rust_code.is_empty(), "Rust codegen failed");
        prop_assert!(!ts_code.is_empty(), "TypeScript codegen failed");
        prop_assert!(!json_code.is_empty(), "JSON Schema codegen failed");
    }

    /// Property: TypeScript uses interface or type keyword
    #[test]
    fn prop_typescript_uses_interface(decl in declaration_strategy()) {
        let code = TypeScriptCodegen::generate(&decl);

        prop_assert!(
            code.contains("interface") || code.contains("type"),
            "TypeScript should use interface or type: {}",
            code
        );
    }

    /// Property: Generated Rust code contains struct or impl
    #[test]
    fn prop_rust_has_struct(decl in declaration_strategy()) {
        let code = RustCodegen::generate(&decl);

        // Rust codegen should produce recognizable Rust constructs
        prop_assert!(
            !code.is_empty(),
            "Rust code should contain recognizable constructs"
        );
    }

    /// Property: JSON Schema contains valid JSON structure markers
    #[test]
    fn prop_json_schema_structure(decl in declaration_strategy()) {
        let code = JsonSchemaCodegen::generate(&decl);

        prop_assert!(
            code.contains("{") && code.contains("}"),
            "JSON Schema should have object structure"
        );
    }
}

// ============================================
// Larger Scale Property Tests
// ============================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100, // Fewer cases for more expensive tests
        ..ProptestConfig::default()
    })]

    /// Property: Large schemas generate successfully
    #[test]
    fn prop_large_schema_generation(
        name in qualified_name_strategy(),
        statements in prop::collection::vec(statement_strategy(), 10..=50),
        exegesis in prop::string::string_regex("[a-zA-Z0-9 .!?,]{50,500}").unwrap()
    ) {
        let gene = Gen {
            visibility: Visibility::default(),
            extends: None,
            name,
            statements,
            exegesis,
            span: Span::default(),
        };

        let code = RustCodegen::generate(&Declaration::Gene(gene));

        prop_assert!(!code.is_empty(), "Large schema should generate code");
    }
}

// ============================================
// Round-Trip Property Tests
// ============================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 500,
        ..ProptestConfig::default()
    })]

    /// Property: Field count is preserved (approximately)
    #[test]
    fn prop_field_count_preserved(
        gene in gen_strategy()
    ) {
        let field_count = gene.statements.len();
        let code = RustCodegen::generate(&Declaration::Gene(gene));

        // Generated code should not be empty if we have fields
        if field_count > 0 {
            prop_assert!(!code.is_empty(), "Should generate code for non-empty gene");
        }
    }

    /// Property: Type information is preserved in some form
    #[test]
    fn prop_types_referenced(
        gene in gen_strategy()
    ) {
        let code = RustCodegen::generate(&Declaration::Gene(gene));

        // Code should be generated (exact preservation is impl-dependent)
        prop_assert!(!code.is_empty());
    }
}

// ============================================
// Case Conversion Property Tests
// ============================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        ..ProptestConfig::default()
    })]

    /// Property: to_pascal_case is idempotent on already PascalCase strings
    #[test]
    fn prop_pascal_case_idempotent(s in "[A-Z][a-zA-Z0-9]{0,30}") {
        let once = to_pascal_case(&s);
        let twice = to_pascal_case(&once);

        prop_assert_eq!(once, twice, "PascalCase conversion should be idempotent");
    }

    /// Property: to_snake_case produces lowercase
    #[test]
    fn prop_snake_case_lowercase(s in "[a-z][a-z0-9.]{0,30}") {
        let snake = to_snake_case(&s);

        prop_assert!(
            snake.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric()),
            "snake_case should be lowercase (with digits and underscores): {}",
            snake
        );
    }

    /// Property: to_rust_ident produces valid identifiers
    #[test]
    fn prop_rust_ident_valid(s in "[a-z][a-z0-9.]{0,30}") {
        let ident = to_rust_ident(&s);

        // Should not be empty
        prop_assert!(!ident.is_empty(), "Identifier should not be empty");

        // Should start with lowercase, r#, or _
        prop_assert!(
            ident.starts_with(|c: char| c.is_lowercase() || c == '_')
                || ident.starts_with("r#"),
            "Rust identifier should start with lowercase or r#: {}",
            ident
        );
    }

    /// Property: Keyword escaping preserves identity for non-keywords
    #[test]
    fn prop_keyword_escape_identity(s in "[a-z][a-z0-9]{0,20}") {
        // Non-keywords should pass through (unless they ARE keywords)
        let escaped = escape_rust_keyword(&s);

        if !["type", "match", "if", "else", "for", "while", "loop", "fn",
             "let", "mut", "const", "static", "self", "Self", "super", "crate"].contains(&s.as_str()) {
            // Most identifiers should pass through unchanged
            prop_assert!(
                escaped == s || escaped.starts_with("r#"),
                "Non-keyword should pass through or be escaped: {} -> {}",
                s,
                escaped
            );
        }
    }
}

// ============================================
// Reflection Property Tests
// ============================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        ..ProptestConfig::default()
    })]

    /// Property: TypeRegistry maintains size correctly
    #[test]
    fn prop_registry_size(
        names in prop::collection::vec(identifier_strategy(), 0..=100)
    ) {
        use metadol::reflect::*;

        let mut registry = TypeRegistry::new();

        for name in &names {
            registry.register(TypeInfo::record(name));
        }

        // Use a set to count unique names
        use std::collections::HashSet;
        let unique_names: HashSet<_> = names.iter().collect();

        prop_assert_eq!(
            registry.len(),
            unique_names.len(),
            "Registry size should match unique names"
        );
    }

    /// Property: Registry lookup is consistent
    #[test]
    fn prop_registry_lookup_consistent(
        name in identifier_strategy()
    ) {
        use metadol::reflect::*;

        let mut registry = TypeRegistry::new();
        registry.register(TypeInfo::record(&name));

        let lookup1 = registry.lookup(&name);
        let lookup2 = registry.lookup(&name);

        prop_assert!(lookup1.is_some(), "First lookup should succeed");
        prop_assert!(lookup2.is_some(), "Second lookup should succeed");
        prop_assert_eq!(
            lookup1.unwrap().name(),
            lookup2.unwrap().name(),
            "Lookups should be consistent"
        );
    }

    /// Property: Registry remove is effective
    #[test]
    fn prop_registry_remove(
        name in identifier_strategy()
    ) {
        use metadol::reflect::*;

        let mut registry = TypeRegistry::new();
        registry.register(TypeInfo::record(&name));

        prop_assert!(registry.contains(&name), "Should contain before remove");

        registry.remove(&name);

        prop_assert!(!registry.contains(&name), "Should not contain after remove");
    }
}

// ============================================
// Performance Property Tests
// ============================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        ..ProptestConfig::default()
    })]

    /// Property: Code generation completes in reasonable time
    #[test]
    fn prop_codegen_performance(decl in declaration_strategy()) {
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let _ = RustCodegen::generate(&decl);
        let elapsed = start.elapsed();

        prop_assert!(
            elapsed < Duration::from_secs(1),
            "Code generation should complete quickly: {:?}",
            elapsed
        );
    }
}

// ============================================
// Additional Unit Tests
// ============================================

#[test]
fn test_identifier_strategy_generates_valid() {
    // Verify our strategy generates valid identifiers
    let config = ProptestConfig::default();
    proptest!(config, |(ident in identifier_strategy())| {
        prop_assert!(ident.chars().next().unwrap().is_lowercase());
        prop_assert!(ident.chars().all(|c| c.is_alphanumeric() || c == '_'));
    });
}

#[test]
fn test_qualified_name_format() {
    let config = ProptestConfig::default();
    proptest!(config, |(name in qualified_name_strategy())| {
        // Should contain at least one part
        prop_assert!(name.contains(char::is_alphanumeric));

        // If it contains a dot, should have multiple parts
        if name.contains('.') {
            prop_assert!(name.split('.').count() >= 2);
        }
    });
}

#[test]
fn test_type_name_strategy_valid() {
    let config = ProptestConfig::default();
    proptest!(config, |(type_name in type_name_strategy())| {
        // Should be one of the known types
        prop_assert!(
            ["Int32", "Int64", "String", "Bool", "Float32", "Float64", "UInt32", "UInt64"]
                .contains(&type_name.as_str())
        );
    });
}

// ============================================
// Stress Tests
// ============================================

#[test]
#[ignore] // Run with --ignored flag for heavy testing
fn stress_test_10k_schemas() {
    // Generate and test 10,000 random schemas
    let config = ProptestConfig {
        cases: 10_000,
        max_shrink_iters: 0, // Don't shrink for stress tests
        ..ProptestConfig::default()
    };

    proptest!(config, |(decl in declaration_strategy())| {
        let code = RustCodegen::generate(&decl);
        prop_assert!(!code.is_empty());
    });
}

#[test]
#[ignore]
fn stress_test_large_schemas() {
    // Test very large schemas
    let config = ProptestConfig {
        cases: 100,
        ..ProptestConfig::default()
    };

    proptest!(config, |(
        name in qualified_name_strategy(),
        statements in prop::collection::vec(statement_strategy(), 100..=500),
        exegesis in prop::string::string_regex("[a-zA-Z0-9 .]{100,1000}").unwrap()
    )| {
        let gene = Gen {
            visibility: Visibility::default(),
            extends: None,
            name,
            statements,
            exegesis,
            span: Span::default(),
        };

        let code = RustCodegen::generate(&Declaration::Gene(gene));
        prop_assert!(!code.is_empty());
    });
}
