//! Multi-Target Code Generation Tests

use dol::ast::{Declaration, DolFile, Gen, HasField, Span, Statement, TypeExpr, Visibility};
use dol_codegen::{targets, CodegenContext, Target};

fn create_simple_gen() -> Gen {
    Gen {
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
    }
}

fn create_test_file() -> DolFile {
    DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(create_simple_gen())],
    }
}

#[test]
fn test_rust_generation() {
    let file = create_test_file();
    let context = CodegenContext::new(Target::Rust).with_docs(true);

    let result = targets::rust::generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("pub struct Point"));
    assert!(code.contains("pub x: f64"));
    assert!(code.contains("pub y: f64"));
    assert!(code.contains("/// A 2D point"));
}

#[test]
fn test_rust_builder_generation() {
    let file = create_test_file();
    let context = CodegenContext::new(Target::Rust)
        .with_docs(true)
        .with_builders(true);

    let result = targets::rust::generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("PointBuilder"));
    assert!(code.contains("pub fn build"));
}

#[test]
fn test_typescript_generation() {
    let file = create_test_file();
    let context = CodegenContext::new(Target::TypeScript).with_docs(true);

    let result = targets::typescript::generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("export interface Point"));
    assert!(code.contains("export class Point"));
    assert!(code.contains("x: number"));
    assert!(code.contains("y: number"));
    assert!(code.contains("* A 2D point"));
}

#[test]
fn test_wit_generation() {
    let file = create_test_file();
    let context = CodegenContext::new(Target::Wit).with_docs(true);

    let result = targets::wit::generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("record point"));
    assert!(code.contains("x: f64"));
    assert!(code.contains("y: f64"));
    assert!(code.contains("/// A 2D point"));
}

#[test]
fn test_python_generation() {
    let file = create_test_file();
    let context = CodegenContext::new(Target::Python).with_docs(true);

    let result = targets::python::generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("@dataclass"));
    assert!(code.contains("class Point:"));
    assert!(code.contains("x: float"));
    assert!(code.contains("y: float"));
    assert!(code.contains("A 2D point"));
}

#[test]
fn test_json_schema_generation() {
    let file = create_test_file();
    let context = CodegenContext::new(Target::JsonSchema).with_docs(true);

    let result = targets::json_schema::generate(&file, &context);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("json-schema.org"));
    assert!(code.contains("Point"));
    assert!(code.contains("A 2D point"));
}

#[test]
fn test_all_targets_generate() {
    let file = create_test_file();

    for target in [
        Target::Rust,
        Target::TypeScript,
        Target::Wit,
        Target::Python,
        Target::JsonSchema,
    ] {
        let context = CodegenContext::new(target);
        let result = targets::generate(&file, &context);
        assert!(result.is_ok(), "Failed to generate {:?}", target);

        let code = result.unwrap();
        assert!(!code.is_empty(), "Empty output for {:?}", target);
    }
}

#[test]
fn test_complex_type_generation() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "ComplexType".to_string(),
        extends: None,
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "items".to_string(),
                type_: TypeExpr::Generic {
                    name: "Vec".to_string(),
                    args: vec![TypeExpr::Named("String".to_string())],
                },
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "count".to_string(),
                type_: TypeExpr::Generic {
                    name: "Option".to_string(),
                    args: vec![TypeExpr::Named("i32".to_string())],
                },
                default: None,
                constraint: None,
                crdt_annotation: None,
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "A complex type with generic fields".to_string(),
        span: Span::default(),
    };

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen)],
    };

    // Test Rust
    let context = CodegenContext::new(Target::Rust);
    let result = targets::rust::generate(&file, &context);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("Vec<String>"));
    assert!(code.contains("Option<i32>"));

    // Test TypeScript
    let context = CodegenContext::new(Target::TypeScript);
    let result = targets::typescript::generate(&file, &context);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("string[]"));
    assert!(code.contains("number | null"));

    // Test Python
    let context = CodegenContext::new(Target::Python);
    let result = targets::python::generate(&file, &context);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("List[str]"));
    assert!(code.contains("Optional[int]"));
}

#[test]
fn test_generator_names() {
    assert_eq!(targets::rust::GENERATOR_NAME, "Rust");
    assert_eq!(targets::typescript::GENERATOR_NAME, "TypeScript");
    assert_eq!(targets::wit::GENERATOR_NAME, "WIT");
    assert_eq!(targets::python::GENERATOR_NAME, "Python");
    assert_eq!(targets::json_schema::GENERATOR_NAME, "JSON Schema");
}
