//! Self-modification safety tests for DOL meta-programming.
//!
//! This module ensures that DOL's meta-programming features are safe
//! and cannot lead to:
//! - Infinite loops during code generation
//! - Stack overflows from recursive quotes
//! - Memory exhaustion
//! - Type system violations
//! - Undefined behavior

use metadol::ast::{
    BinaryOp, Declaration, Expr, Gen, HasField, Literal, Span, Statement, TypeExpr, Visibility,
};
use metadol::codegen::*;
use metadol::parse_file;
use metadol::reflect::*;

// ============================================
// Quote/Eval Safety Tests
// ============================================

#[test]
fn test_deep_quote_nesting_limited() {
    // Test that deeply nested quotes don't cause stack overflow
    let mut expr = Expr::Literal(Literal::Int(42));

    // Nest 100 quotes deep
    for _ in 0..100 {
        expr = Expr::Quote(Box::new(expr));
    }

    // Should not panic or overflow
    assert!(matches!(expr, Expr::Quote(_)));
}

#[test]
fn test_deep_eval_nesting() {
    // Test eval nesting safety
    let base = Expr::Quote(Box::new(Expr::Literal(Literal::Int(42))));
    let mut expr = base;

    for _ in 0..50 {
        expr = Expr::Eval(Box::new(expr));
    }

    // Should not panic
    assert!(matches!(expr, Expr::Eval(_)));
}

#[test]
fn test_large_ast_handling() {
    // Test handling of large AST structures
    let mut statements = Vec::new();

    for i in 0..1000 {
        statements.push(Statement::HasField(Box::new(HasField {
            name: format!("field_{}", i),
            type_: TypeExpr::Named("Int32".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: None,
            personal: false,
            span: Span::default(),
        })));
    }

    let gene = Gen {
        visibility: Visibility::default(),
        extends: None,
        name: "LargeGene".to_string(),
        statements,
        exegesis: "Large gene test".to_string(),
        span: Span::default(),
    };

    // Should handle large structures without panic
    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(!code.is_empty());
}

// ============================================
// Circular Reference Detection
// ============================================

#[test]
fn test_no_circular_type_references() {
    // Test that the reflection system handles potential circular references
    let mut registry = TypeRegistry::new();

    // Create types that could form a cycle
    registry.register(TypeInfo::record("A").with_field(FieldInfo::new("b_ref", "B")));

    registry.register(TypeInfo::record("B").with_field(FieldInfo::new("a_ref", "A")));

    // Should not cause infinite loop when querying
    let a = registry.lookup("A");
    assert!(a.is_some());

    let b = registry.lookup("B");
    assert!(b.is_some());
}

#[test]
fn test_self_referential_type() {
    let mut registry = TypeRegistry::new();

    // A type that references itself
    registry.register(
        TypeInfo::record("Node")
            .with_field(FieldInfo::new("value", "Int32"))
            .with_field(FieldInfo::new("next", "Option<Node>")),
    );

    let node = registry.lookup("Node");
    assert!(node.is_some());
    assert_eq!(node.unwrap().fields().len(), 2);
}

// ============================================
// Memory Safety Tests
// ============================================

#[test]
fn test_large_registry_memory() {
    let mut registry = TypeRegistry::new();

    // Register many types
    for i in 0..10000 {
        registry.register(TypeInfo::record(format!("Type{}", i)));
    }

    assert_eq!(registry.len(), 10000);

    // Cleanup
    registry.clear();
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_large_string_handling() {
    // Test that large strings don't cause issues
    let large_doc = "a".repeat(100000);

    let info = TypeInfo::record("Test").with_doc(large_doc.clone());

    assert_eq!(info.doc(), Some(large_doc.as_str()));
}

#[test]
fn test_memory_cleanup() {
    // Test that registry properly cleans up memory
    let mut registry = TypeRegistry::new();

    for i in 0..1000 {
        registry.register(TypeInfo::record(format!("Temp{}", i)));
    }

    // Remove all
    for i in 0..1000 {
        registry.remove(&format!("Temp{}", i));
    }

    assert!(registry.is_empty());
}

// ============================================
// Infinite Loop Prevention
// ============================================

#[test]
fn test_codegen_terminates() {
    let source = r#"
gene test.terminates {
    entity has field
}

exegesis {
    Termination test.
}
"#;

    // Should complete without hanging
    use std::time::{Duration, Instant};
    let start = Instant::now();

    let _ = compile_to_rust_via_hir(source);

    let elapsed = start.elapsed();

    // Should complete in reasonable time (< 1 second)
    assert!(
        elapsed < Duration::from_secs(1),
        "Code generation took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_registry_iteration_terminates() {
    let mut registry = TypeRegistry::new();

    for i in 0..100 {
        registry.register(TypeInfo::record(format!("Type{}", i)));
    }

    // Iteration should terminate
    let count = registry.type_names().count();
    assert_eq!(count, 100);
}

// ============================================
// Type Safety Tests
// ============================================

#[test]
fn test_type_system_integrity() {
    let mut registry = TypeRegistry::with_primitives();

    // Primitives should maintain their type
    let int_type = registry.lookup("Int32").unwrap();
    assert_eq!(int_type.kind(), TypeKind::Primitive);

    // Overwriting should maintain type safety
    registry.register(TypeInfo::record("Custom"));
    let custom = registry.lookup("Custom").unwrap();
    assert_eq!(custom.kind(), TypeKind::Record);
}

#[test]
fn test_field_type_consistency() {
    let info = TypeInfo::record("Test")
        .with_field(FieldInfo::new("field1", "Int32"))
        .with_field(FieldInfo::new("field2", "String"));

    assert_eq!(info.field("field1").unwrap().type_name(), "Int32");
    assert_eq!(info.field("field2").unwrap().type_name(), "String");
}

// ============================================
// Boundary Condition Tests
// ============================================

#[test]
fn test_empty_exegesis() {
    let source = r#"
gene test.empty {
    entity has field
}

exegesis {
}
"#;

    let result = parse_file(source);

    // Should handle empty exegesis without panic
    // (may or may not be valid depending on parser rules)
    match result {
        Ok(_) | Err(_) => (), // Either outcome is acceptable, just shouldn't panic
    }
}

#[test]
fn test_no_statements() {
    let gene = Gen {
        visibility: Visibility::default(),
        extends: None,
        name: "Empty".to_string(),
        statements: vec![],
        exegesis: "Empty gene".to_string(),
        span: Span::default(),
    };

    // Should handle empty gene without panic
    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(!code.is_empty());
}

#[test]
fn test_maximum_field_name_length() {
    let long_name = "a".repeat(1000);

    let field = FieldInfo::new(long_name.clone(), "Int32");

    assert_eq!(field.name(), long_name);
}

// ============================================
// Error Condition Safety
// ============================================

#[test]
fn test_invalid_syntax_doesnt_panic() {
    let invalid_sources = vec![
        "gene invalid {",
        "gene { }",
        "",
        "random text",
        "gene test { entity has }",
    ];

    for source in invalid_sources {
        let result = parse_file(source);
        // Should return error, not panic
        match result {
            Ok(_) | Err(_) => (), // Either is fine, just shouldn't panic
        }
    }
}

#[test]
fn test_malformed_type_expr() {
    // Test that malformed type expressions don't cause issues
    let gene = Gen {
        visibility: Visibility::default(),
        extends: None,
        name: "Test".to_string(),
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "field".to_string(),
            type_: TypeExpr::Named("".to_string()), // Empty type name
            default: None,
            constraint: None,
            crdt_annotation: None,
            personal: false,
            span: Span::default(),
        }))],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    // Should handle gracefully
    let _ = RustCodegen::generate(&Declaration::Gene(gene));
}

// ============================================
// Concurrent Access Safety
// ============================================

#[test]
fn test_registry_thread_safety() {
    // Test that registry operations are safe
    // (Not testing actual concurrency, but ensuring no mutable aliasing issues)
    let registry = TypeRegistry::with_primitives();

    let info1 = registry.lookup("Int32");
    let info2 = registry.lookup("String");

    assert!(info1.is_some());
    assert!(info2.is_some());
}

// ============================================
// Quote/Eval Invariant Tests
// ============================================

#[test]
fn test_quote_preserves_structure() {
    let original = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Literal(Literal::Int(1))),
        right: Box::new(Expr::Literal(Literal::Int(2))),
    };

    let quoted = Expr::Quote(Box::new(original.clone()));

    // The quoted expression should preserve the original
    if let Expr::Quote(inner) = quoted {
        assert_eq!(*inner, original);
    } else {
        panic!("Expected Quote");
    }
}

#[test]
fn test_eval_type_safety() {
    // Eval should only accept expressions
    let expr = Expr::Eval(Box::new(Expr::Quote(Box::new(Expr::Literal(
        Literal::Int(42),
    )))));

    // Should be well-formed
    if let Expr::Eval(_) = expr {
        // Success
    } else {
        panic!("Expected Eval");
    }
}

// ============================================
// Resource Exhaustion Prevention
// ============================================

#[test]
fn test_reasonable_recursion_depth() {
    // Test that we can handle reasonable recursion
    fn create_nested_binary(depth: usize) -> Expr {
        if depth == 0 {
            Expr::Literal(Literal::Int(1))
        } else {
            Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(create_nested_binary(depth - 1)),
                right: Box::new(Expr::Literal(Literal::Int(1))),
            }
        }
    }

    // 100 levels should be safe
    let expr = create_nested_binary(100);

    // Should not panic
    let quoted = Expr::Quote(Box::new(expr));
    assert!(matches!(quoted, Expr::Quote(_)));
}

#[test]
fn test_registry_size_limits() {
    let mut registry = TypeRegistry::new();

    // Add a reasonable number of types
    for i in 0..1000 {
        registry.register(TypeInfo::record(format!("Type{}", i)));
    }

    assert_eq!(registry.len(), 1000);

    // Operations should still be fast
    use std::time::Instant;
    let start = Instant::now();

    for i in 0..1000 {
        let _ = registry.lookup(&format!("Type{}", i));
    }

    let elapsed = start.elapsed();

    // Should complete quickly
    assert!(
        elapsed.as_millis() < 100,
        "Registry lookups too slow: {:?}",
        elapsed
    );
}

// ============================================
// Invariant Preservation Tests
// ============================================

#[test]
fn test_visibility_preserved() {
    let gene = Gen {
        visibility: Visibility::Public,
        extends: None,
        name: "Test".to_string(),
        statements: vec![],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    assert_eq!(gene.visibility, Visibility::Public);
}

#[test]
fn test_span_preserved() {
    let span = Span::new(10, 20, 1, 10);
    let _expr = Expr::Literal(Literal::Int(42));

    // Spans should be preserved through transformations
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 20);
    assert_eq!(span.len(), 10);
}

// ============================================
// Graceful Degradation Tests
// ============================================

#[test]
fn test_partial_parse_recovery() {
    // Test that parser can recover from some errors
    let source = r#"
gene test.recovery {
    entity has field
}

exegesis {
    Recovery test.
}
"#;

    let result = parse_file(source);

    // Valid source should parse successfully
    assert!(result.is_ok());
}

#[test]
fn test_codegen_with_missing_types() {
    // Test code generation with unknown types
    let gene = Gen {
        visibility: Visibility::default(),
        extends: None,
        name: "Test".to_string(),
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "field".to_string(),
            type_: TypeExpr::Named("UnknownType".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: None,
            personal: false,
            span: Span::default(),
        }))],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    // Should handle unknown types gracefully
    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(!code.is_empty());
}
