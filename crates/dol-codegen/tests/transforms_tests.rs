//! AST Transformation Tests

use dol::ast::{
    CrdtAnnotation, CrdtStrategy, Declaration, DolFile, Gen, HasField, Span, Statement, TypeExpr,
    Visibility,
};
use dol_codegen::{
    transforms::{CrdtExpansionVisitor, TypeInferenceVisitor, Visitor},
    CodegenContext, Target,
};

fn create_test_gen_with_crdt() -> Gen {
    Gen {
        visibility: Visibility::Public,
        name: "chat.message".to_string(),
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
                name: "content".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: Some(CrdtAnnotation {
                    strategy: CrdtStrategy::Peritext,
                    options: vec![],
                    span: Span::default(),
                }),
                personal: false,
                span: Span::default(),
            })),
        ],
        exegesis: "A chat message with CRDT support".to_string(),
        span: Span::default(),
    }
}

#[test]
fn test_type_inference_visitor() {
    let mut visitor = TypeInferenceVisitor::new();

    let type_expr = TypeExpr::Named("i32".to_string());
    let result = visitor.infer_type(&type_expr);
    assert!(result.is_ok());

    let inferred = result.unwrap();
    assert_eq!(inferred, dol::ast::Type::I32);
}

#[test]
fn test_type_inference_generic() {
    let mut visitor = TypeInferenceVisitor::new();

    let vec_type = TypeExpr::Generic {
        name: "Vec".to_string(),
        args: vec![TypeExpr::Named("String".to_string())],
    };

    let result = visitor.infer_type(&vec_type);
    assert!(result.is_ok());

    let inferred = result.unwrap();
    assert!(matches!(inferred, dol::ast::Type::Vec(_)));
}

#[test]
fn test_crdt_expansion_visitor() {
    let mut gen = create_test_gen_with_crdt();
    let mut visitor = CrdtExpansionVisitor::new(true);

    let result = visitor.visit_gen(&mut gen);
    assert!(result.is_ok());
}

#[test]
fn test_transform_pipeline() {
    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(create_test_gen_with_crdt())],
    };

    let context = CodegenContext::new(Target::Rust);
    let result = dol_codegen::transforms::transform(&file, &context);

    assert!(result.is_ok());
}

#[test]
fn test_crdt_strategy_serialization() {
    assert_eq!(CrdtStrategy::Immutable.as_str(), "immutable");
    assert_eq!(CrdtStrategy::Lww.as_str(), "lww");
    assert_eq!(CrdtStrategy::OrSet.as_str(), "or_set");
    assert_eq!(CrdtStrategy::PnCounter.as_str(), "pn_counter");
    assert_eq!(CrdtStrategy::Peritext.as_str(), "peritext");
    assert_eq!(CrdtStrategy::Rga.as_str(), "rga");
    assert_eq!(CrdtStrategy::MvRegister.as_str(), "mv_register");
}

#[test]
fn test_crdt_strategy_parsing() {
    assert_eq!(
        CrdtStrategy::from_str("immutable"),
        Some(CrdtStrategy::Immutable)
    );
    assert_eq!(CrdtStrategy::from_str("lww"), Some(CrdtStrategy::Lww));
    assert_eq!(
        CrdtStrategy::from_str("or_set"),
        Some(CrdtStrategy::OrSet)
    );
    assert_eq!(CrdtStrategy::from_str("invalid"), None);
}

#[test]
fn test_type_inference_option() {
    let mut visitor = TypeInferenceVisitor::new();

    let option_type = TypeExpr::Generic {
        name: "Option".to_string(),
        args: vec![TypeExpr::Named("i32".to_string())],
    };

    let result = visitor.infer_type(&option_type);
    assert!(result.is_ok());

    let inferred = result.unwrap();
    assert!(matches!(inferred, dol::ast::Type::Option(_)));
}

#[test]
fn test_type_inference_result() {
    let mut visitor = TypeInferenceVisitor::new();

    let result_type = TypeExpr::Generic {
        name: "Result".to_string(),
        args: vec![
            TypeExpr::Named("String".to_string()),
            TypeExpr::Named("i32".to_string()),
        ],
    };

    let result = visitor.infer_type(&result_type);
    assert!(result.is_ok());

    let inferred = result.unwrap();
    assert!(matches!(inferred, dol::ast::Type::Result(_, _)));
}
