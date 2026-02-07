//! Template Engine Integration Tests

use dol::ast::{Declaration, DolFile, Gen, HasField, Span, Statement, TypeExpr, Visibility};
use dol_codegen::{template_engine, CodegenContext, Target};

fn create_test_gen() -> Gen {
    Gen {
        visibility: Visibility::Public,
        name: "test.point".to_string(),
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
        exegesis: "A 2D point in space".to_string(),
        span: Span::default(),
    }
}

fn create_test_file() -> DolFile {
    DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(create_test_gen())],
    }
}

#[test]
fn test_template_data_creation() {
    let file = create_test_file();
    let context = CodegenContext::new(Target::Rust);

    let engine = template_engine::TemplateEngine::new(Target::Rust);
    let result = engine.generate(&file, &context);

    assert!(result.is_ok());
}

#[test]
fn test_rust_type_mapping() {
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("String".to_string()), Target::Rust),
        "String"
    );
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("i32".to_string()), Target::Rust),
        "i32"
    );
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("f64".to_string()), Target::Rust),
        "f64"
    );

    let vec_type = TypeExpr::Generic {
        name: "Vec".to_string(),
        args: vec![TypeExpr::Named("String".to_string())],
    };
    assert_eq!(
        template_engine::map_type_expr(&vec_type, Target::Rust),
        "Vec<String>"
    );
}

#[test]
fn test_typescript_type_mapping() {
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("String".to_string()), Target::TypeScript),
        "string"
    );
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("i32".to_string()), Target::TypeScript),
        "number"
    );

    let option_type = TypeExpr::Generic {
        name: "Option".to_string(),
        args: vec![TypeExpr::Named("String".to_string())],
    };
    assert_eq!(
        template_engine::map_type_expr(&option_type, Target::TypeScript),
        "string | null"
    );
}

#[test]
fn test_wit_type_mapping() {
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("String".to_string()), Target::Wit),
        "string"
    );
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("i32".to_string()), Target::Wit),
        "s32"
    );

    let list_type = TypeExpr::Generic {
        name: "Vec".to_string(),
        args: vec![TypeExpr::Named("i32".to_string())],
    };
    assert_eq!(
        template_engine::map_type_expr(&list_type, Target::Wit),
        "list<s32>"
    );
}

#[test]
fn test_python_type_mapping() {
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("String".to_string()), Target::Python),
        "str"
    );
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("i32".to_string()), Target::Python),
        "int"
    );

    let list_type = TypeExpr::Generic {
        name: "Vec".to_string(),
        args: vec![TypeExpr::Named("String".to_string())],
    };
    assert_eq!(
        template_engine::map_type_expr(&list_type, Target::Python),
        "List[str]"
    );
}

#[test]
fn test_json_schema_type_mapping() {
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("String".to_string()), Target::JsonSchema),
        "string"
    );
    assert_eq!(
        template_engine::map_type_expr(&TypeExpr::Named("i32".to_string()), Target::JsonSchema),
        "integer"
    );
}

#[test]
fn test_generate_all_targets() {
    let file = create_test_file();

    for target in [
        Target::Rust,
        Target::TypeScript,
        Target::Wit,
        Target::Python,
        Target::JsonSchema,
    ] {
        let context = CodegenContext::new(target);
        let result = template_engine::generate(&file, &context);
        assert!(result.is_ok(), "Failed to generate {:?}", target);

        let code = result.unwrap();
        assert!(!code.is_empty(), "Empty output for {:?}", target);
    }
}
