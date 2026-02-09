//! Integration Tests for DOL Code Generation

use dol::ast::{
    CrdtAnnotation, CrdtStrategy, Declaration, DolFile, Gen, HasField, Span, Statement, TypeExpr,
    Visibility,
};
use dol_codegen::{generate, CodegenContext, Target};

#[test]
fn test_end_to_end_rust_generation() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "user.profile".to_string(),
        extends: None,
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "id".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: Some(CrdtAnnotation {
                    strategy: CrdtStrategy::Immutable,
                    options: vec![],
                    span: Span::default(),
                }),
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "name".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: Some(CrdtAnnotation {
                    strategy: CrdtStrategy::Lww,
                    options: vec![],
                    span: Span::default(),
                }),
                personal: true,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "age".to_string(),
                type_: TypeExpr::Named("i32".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "A user profile with CRDT support".to_string(),
        span: Span::default(),
    };

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen)],
    };

    let context = CodegenContext::new(Target::Rust)
        .with_docs(true)
        .with_module_name("user");

    let result = generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("pub struct UserProfile"));
    assert!(code.contains("pub id: String"));
    assert!(code.contains("pub name: String"));
    assert!(code.contains("pub age: i32"));
    assert!(code.contains("/// A user profile with CRDT support"));
}

#[test]
fn test_end_to_end_typescript_generation() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "todo.item".to_string(),
        extends: None,
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "id".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "title".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "completed".to_string(),
                type_: TypeExpr::Named("bool".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "A todo item".to_string(),
        span: Span::default(),
    };

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen)],
    };

    let context = CodegenContext::new(Target::TypeScript)
        .with_docs(true)
        .with_module_name("todo");

    let result = generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("export interface TodoItem"));
    assert!(code.contains("id: string"));
    assert!(code.contains("title: string"));
    assert!(code.contains("completed: boolean"));
}

#[test]
fn test_end_to_end_all_targets() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "Message".to_string(),
        extends: None,
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "content".to_string(),
            type_: TypeExpr::Named("String".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: None,
            personal: false,
            span: Span::default(),
        }))],
        exegesis: "A simple message".to_string(),
        span: Span::default(),
    };

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen)],
    };

    for target in [
        Target::Rust,
        Target::TypeScript,
        Target::Wit,
        Target::Python,
        Target::JsonSchema,
    ] {
        let context = CodegenContext::new(target).with_docs(true);
        let result = generate(&file, &context);

        assert!(result.is_ok(), "Failed for target {:?}", target);
        let code = result.unwrap();
        assert!(!code.is_empty(), "Empty code for target {:?}", target);
    }
}

#[test]
fn test_multiple_declarations() {
    let gen1 = Gen {
        visibility: Visibility::Public,
        name: "Point".to_string(),
        extends: None,
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "x".to_string(),
                type_: TypeExpr::Named("f64".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "y".to_string(),
                type_: TypeExpr::Named("f64".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "A 2D point".to_string(),
        span: Span::default(),
    };

    let gen2 = Gen {
        visibility: Visibility::Public,
        name: "Line".to_string(),
        extends: None,
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "start".to_string(),
                type_: TypeExpr::Named("Point".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "end".to_string(),
                type_: TypeExpr::Named("Point".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "A line segment".to_string(),
        span: Span::default(),
    };

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen1), Declaration::Gene(gen2)],
    };

    let context = CodegenContext::new(Target::Rust);
    let result = generate(&file, &context);

    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("pub struct Point"));
    assert!(code.contains("pub struct Line"));
}

#[test]
fn test_context_configuration() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "Test".to_string(),
        extends: None,
        statements: vec![],
        exegesis: "A test".to_string(),
        span: Span::default(),
    };

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen)],
    };

    // Test with docs
    let context = CodegenContext::new(Target::Rust).with_docs(true);
    let result = generate(&file, &context);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("/// A test"));

    // Test without docs
    let context = CodegenContext::new(Target::Rust).with_docs(false);
    let result = generate(&file, &context);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(!code.contains("/// A test"));
}
