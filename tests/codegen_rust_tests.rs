//! Comprehensive integration tests for DOL → Rust code generation.
//!
//! These tests verify that the Rust code generator correctly transforms
//! DOL declarations into valid Rust code.

use metadol::ast::*;
use metadol::codegen::{RustCodegen, TypeMapper};

// ============================================
// 1. Gene → Struct Generation Tests
// ============================================

#[test]
fn test_codegen_simple_gene() {
    let gene = Gene {
        name: "Point".to_string(),
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "x".to_string(),
                type_: TypeExpr::Named("Float64".to_string()),
                default: Some(Expr::Literal(Literal::Float(0.0))),
                constraint: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "y".to_string(),
                type_: TypeExpr::Named("Float64".to_string()),
                default: Some(Expr::Literal(Literal::Float(0.0))),
                constraint: None,
                span: Span::default(),
            })),
        ],
        exegesis: "A 2D point in Cartesian coordinates".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));

    // Verify struct declaration
    assert!(code.contains("pub struct Point"), "Should generate struct");
    assert!(code.contains("pub x: f64"), "Should generate x field");
    assert!(code.contains("pub y: f64"), "Should generate y field");
    assert!(
        code.contains("/// A 2D point in Cartesian coordinates"),
        "Should include doc comment"
    );
    assert!(
        code.contains("#[derive(Debug, Clone"),
        "Should have derive macro"
    );
}

#[test]
fn test_codegen_gene_with_constraint() {
    let gene = Gene {
        name: "PositiveNumber".to_string(),
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "value".to_string(),
            type_: TypeExpr::Named("Int64".to_string()),
            default: None,
            constraint: Some(Expr::Binary {
                left: Box::new(Expr::Identifier("value".to_string())),
                op: BinaryOp::Gt,
                right: Box::new(Expr::Literal(Literal::Int(0))),
            }),
            span: Span::default(),
        }))],
        exegesis: "A number that must be positive".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));

    assert!(
        code.contains("pub struct PositiveNumber"),
        "Should generate struct"
    );
    assert!(
        code.contains("pub value: i64"),
        "Should generate value field"
    );
    assert!(
        code.contains("fn validate_value(&self) -> bool"),
        "Should generate validator"
    );
    assert!(
        code.contains("fn validate_all(&self) -> bool"),
        "Should generate validate_all"
    );
}

#[test]
fn test_codegen_gene_with_multiple_types() {
    let gene = Gene {
        name: "User".to_string(),
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "id".to_string(),
                type_: TypeExpr::Named("UInt64".to_string()),
                default: None,
                constraint: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "name".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "email".to_string(),
                type_: TypeExpr::Generic {
                    name: "Option".to_string(),
                    args: vec![TypeExpr::Named("String".to_string())],
                },
                default: Some(Expr::Literal(Literal::Null)),
                constraint: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "tags".to_string(),
                type_: TypeExpr::Generic {
                    name: "List".to_string(),
                    args: vec![TypeExpr::Named("String".to_string())],
                },
                default: None,
                constraint: None,
                span: Span::default(),
            })),
        ],
        exegesis: "A user entity with various field types".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));

    assert!(code.contains("pub id: isize"), "Should map UInt64 to isize");
    assert!(
        code.contains("pub name: String"),
        "Should map String to String"
    );
    assert!(
        code.contains("pub email: Option<String>"),
        "Should map Option correctly"
    );
    assert!(
        code.contains("pub tags: Vec<String>"),
        "Should map List to Vec"
    );
}

#[test]
fn test_codegen_gene_with_legacy_has_statement() {
    let gene = Gene {
        name: "Legacy".to_string(),
        statements: vec![
            Statement::Has {
                subject: "legacy".to_string(),
                property: "prop1".to_string(),
                span: Span::default(),
            },
            Statement::Has {
                subject: "legacy".to_string(),
                property: "prop2".to_string(),
                span: Span::default(),
            },
        ],
        exegesis: "Legacy gene using old syntax".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));

    // Legacy has statements default to String type
    assert!(
        code.contains("pub prop1: String"),
        "Should generate prop1 with String type"
    );
    assert!(
        code.contains("pub prop2: String"),
        "Should generate prop2 with String type"
    );
}

// ============================================
// 2. Trait → Trait Generation Tests
// ============================================

#[test]
fn test_codegen_simple_trait() {
    let trait_decl = Trait {
        name: "Lifecycle".to_string(),
        statements: vec![
            Statement::Is {
                subject: "entity".to_string(),
                state: "created".to_string(),
                span: Span::default(),
            },
            Statement::Is {
                subject: "entity".to_string(),
                state: "active".to_string(),
                span: Span::default(),
            },
            Statement::Is {
                subject: "entity".to_string(),
                state: "terminated".to_string(),
                span: Span::default(),
            },
        ],
        exegesis: "Lifecycle state machine".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Trait(trait_decl));

    assert!(
        code.contains("pub trait Lifecycle"),
        "Should generate trait"
    );
    assert!(
        code.contains("fn is_created(&self) -> bool"),
        "Should generate is_created method"
    );
    assert!(
        code.contains("fn is_active(&self) -> bool"),
        "Should generate is_active method"
    );
    assert!(
        code.contains("fn is_terminated(&self) -> bool"),
        "Should generate is_terminated method"
    );
    assert!(
        code.contains("/// Lifecycle state machine"),
        "Should include doc comment"
    );
}

#[test]
fn test_codegen_trait_with_supertraits() {
    let trait_decl = Trait {
        name: "Advanced".to_string(),
        statements: vec![
            Statement::Uses {
                reference: "Basic".to_string(),
                span: Span::default(),
            },
            Statement::Uses {
                reference: "Extended".to_string(),
                span: Span::default(),
            },
            Statement::Is {
                subject: "entity".to_string(),
                state: "ready".to_string(),
                span: Span::default(),
            },
        ],
        exegesis: "Advanced trait with multiple supertraits".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Trait(trait_decl));

    assert!(
        code.contains("pub trait Advanced: Basic + Extended"),
        "Should include supertraits"
    );
    assert!(
        code.contains("fn is_ready(&self) -> bool"),
        "Should generate method"
    );
}

#[test]
fn test_codegen_trait_no_supertraits() {
    let trait_decl = Trait {
        name: "Simple".to_string(),
        statements: vec![Statement::Is {
            subject: "entity".to_string(),
            state: "active".to_string(),
            span: Span::default(),
        }],
        exegesis: "Simple trait".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Trait(trait_decl));

    // Should not have supertraits
    assert!(
        code.contains("pub trait Simple {"),
        "Should generate trait without supertraits"
    );
}

// ============================================
// 3. System → Module Generation Tests
// ============================================

#[test]
fn test_codegen_system_module() {
    let system = System {
        name: "orchestrator.core".to_string(),
        version: "1.0.0".to_string(),
        requirements: vec![
            Requirement {
                name: "lifecycle".to_string(),
                constraint: ">=".to_string(),
                version: "0.5.0".to_string(),
                span: Span::default(),
            },
            Requirement {
                name: "networking".to_string(),
                constraint: "^".to_string(),
                version: "2.0.0".to_string(),
                span: Span::default(),
            },
        ],
        statements: vec![],
        exegesis: "Core orchestration system".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::System(system));

    assert!(
        code.contains("pub mod orchestrator_core"),
        "Should generate module"
    );
    assert!(
        code.contains("/// System version: 1.0.0"),
        "Should include version"
    );
    assert!(
        code.contains("//! # Requirements"),
        "Should include requirements section"
    );
    assert!(
        code.contains("`lifecycle` >= 0.5.0"),
        "Should list first requirement"
    );
    assert!(
        code.contains("`networking` ^ 2.0.0"),
        "Should list second requirement"
    );
    assert!(
        code.contains("/// Core orchestration system"),
        "Should include doc comment"
    );
}

#[test]
fn test_codegen_system_no_requirements() {
    let system = System {
        name: "simple".to_string(),
        version: "1.0.0".to_string(),
        requirements: vec![],
        statements: vec![],
        exegesis: "Simple system".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::System(system));

    assert!(code.contains("pub mod simple"), "Should generate module");
    assert!(
        !code.contains("//! # Requirements"),
        "Should not have requirements section"
    );
}

// ============================================
// 4. Constraint → Validation Generation Tests
// ============================================

#[test]
fn test_codegen_constraint() {
    let constraint = Constraint {
        name: "data.integrity".to_string(),
        statements: vec![
            Statement::Matches {
                subject: "checksum".to_string(),
                target: "computed_checksum".to_string(),
                span: Span::default(),
            },
            Statement::Never {
                subject: "data".to_string(),
                action: "corrupted".to_string(),
                span: Span::default(),
            },
        ],
        exegesis: "Ensures data integrity through checksums".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Constraint(constraint));

    assert!(
        code.contains("fn validate_data_integrity"),
        "Should generate validation function"
    );
    assert!(
        code.contains("// checksum matches computed_checksum"),
        "Should include matches comment"
    );
    assert!(
        code.contains("// data never corrupted"),
        "Should include never comment"
    );
    assert!(
        code.contains("/// Ensures data integrity"),
        "Should include doc comment"
    );
}

#[test]
fn test_codegen_constraint_empty() {
    let constraint = Constraint {
        name: "empty".to_string(),
        statements: vec![],
        exegesis: "Empty constraint".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Constraint(constraint));

    assert!(
        code.contains("fn validate_empty"),
        "Should generate validation function"
    );
    assert!(code.contains("true"), "Should return true by default");
}

// ============================================
// 5. Evolution → Documentation Generation Tests
// ============================================

#[test]
fn test_codegen_evolution() {
    let evolution = Evolution {
        name: "Container".to_string(),
        version: "2.0.0".to_string(),
        parent_version: "1.0.0".to_string(),
        additions: vec![Statement::Has {
            subject: "container".to_string(),
            property: "gpu_support".to_string(),
            span: Span::default(),
        }],
        deprecations: vec![Statement::Has {
            subject: "container".to_string(),
            property: "legacy_api".to_string(),
            span: Span::default(),
        }],
        removals: vec!["old_field".to_string()],
        rationale: Some("GPU support is now standard".to_string()),
        exegesis: "Version 2.0 adds GPU capabilities".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Evolution(evolution));

    assert!(
        code.contains("// Evolution: Container @ 2.0.0 (from 1.0.0)"),
        "Should include version info"
    );
    assert!(
        code.contains("// Rationale: GPU support is now standard"),
        "Should include rationale"
    );
    assert!(code.contains("// Additions:"), "Should list additions");
    assert!(
        code.contains("// Deprecations:"),
        "Should list deprecations"
    );
    assert!(code.contains("// Removals:"), "Should list removals");
    assert!(
        code.contains("//   - old_field"),
        "Should list removal items"
    );
}

#[test]
fn test_codegen_evolution_no_rationale() {
    let evolution = Evolution {
        name: "Simple".to_string(),
        version: "1.1.0".to_string(),
        parent_version: "1.0.0".to_string(),
        additions: vec![],
        deprecations: vec![],
        removals: vec![],
        rationale: None,
        exegesis: "Minor update".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Evolution(evolution));

    assert!(
        code.contains("// Evolution: Simple @ 1.1.0 (from 1.0.0)"),
        "Should include version info"
    );
    assert!(!code.contains("// Rationale:"), "Should not have rationale");
}

// ============================================
// 6. SEX (Side Effect eXecution) Tests
// ============================================

#[test]
fn test_codegen_sex_function() {
    let gen = RustCodegen::new();

    let func = FunctionDecl {
        visibility: Visibility::Public,
        purity: Purity::Sex,
        name: "print_message".to_string(),
        type_params: None,
        params: vec![FunctionParam {
            name: "msg".to_string(),
            type_ann: TypeExpr::Named("String".to_string()),
        }],
        return_type: None,
        body: vec![Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Identifier("println".to_string())),
            args: vec![Expr::Identifier("msg".to_string())],
        })],
        exegesis: String::new(),
        span: Span::default(),
    };

    let output = gen.gen_sex_function(Visibility::Public, &func);
    assert!(output.contains("/// Side-effectful function"));
    assert!(output.contains("pub fn print_message(msg: String)"));
    assert!(output.contains("println!(\"{}\", msg)"));
}

#[test]
fn test_codegen_sex_function_with_return() {
    let gen = RustCodegen::new();

    let func = FunctionDecl {
        visibility: Visibility::Private,
        purity: Purity::Sex,
        name: "increment".to_string(),
        type_params: None,
        params: vec![FunctionParam {
            name: "x".to_string(),
            type_ann: TypeExpr::Named("Int32".to_string()),
        }],
        return_type: Some(TypeExpr::Named("Int32".to_string())),
        body: vec![Stmt::Return(Some(Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(1))),
        }))],
        exegesis: String::new(),
        span: Span::default(),
    };

    let output = gen.gen_sex_function(Visibility::Private, &func);
    assert!(output.contains("fn increment(x: i32) -> i32"));
    assert!(output.contains("return (x + 1_i64);"));
}

#[test]
fn test_codegen_global_var() {
    let gen = RustCodegen::new();

    let var = VarDecl {
        mutability: Mutability::Mutable,
        name: "counter".to_string(),
        type_ann: Some(TypeExpr::Named("Int64".to_string())),
        value: Some(Expr::Literal(Literal::Int(0))),
        span: Span::default(),
    };

    let output = gen.gen_global_var(&var);
    assert!(output.contains("static mut COUNTER: i64 = 0_i64;"));
}

#[test]
fn test_codegen_constant() {
    let gen = RustCodegen::new();

    let var = VarDecl {
        mutability: Mutability::Immutable,
        name: "MAX_SIZE".to_string(),
        type_ann: Some(TypeExpr::Named("Int64".to_string())),
        value: Some(Expr::Literal(Literal::Int(1024))),
        span: Span::default(),
    };

    let output = gen.gen_constant(&var);
    assert!(output.contains("const MAX_SIZE: i64 = 1024_i64;"));
}

#[test]
fn test_codegen_extern_function() {
    let gen = RustCodegen::new();

    let decl = ExternDecl {
        abi: Some("C".to_string()),
        name: "add".to_string(),
        params: vec![
            FunctionParam {
                name: "a".to_string(),
                type_ann: TypeExpr::Named("Int32".to_string()),
            },
            FunctionParam {
                name: "b".to_string(),
                type_ann: TypeExpr::Named("Int32".to_string()),
            },
        ],
        return_type: Some(TypeExpr::Named("Int32".to_string())),
        span: Span::default(),
    };

    let output = gen.gen_extern(&decl);
    assert!(output.contains("extern \"C\" {"));
    assert!(output.contains("fn add(a: i32, b: i32) -> i32;"));
}

#[test]
fn test_codegen_global_access() {
    let gen = RustCodegen::new();
    assert_eq!(gen.gen_global_access("counter"), "unsafe { COUNTER }");
}

#[test]
fn test_codegen_global_mutation() {
    let gen = RustCodegen::new();
    assert_eq!(
        gen.gen_global_mutation("counter", "42"),
        "unsafe { COUNTER = 42; }"
    );
}

// ============================================
// 7. Type Mapping Tests
// ============================================

#[test]
fn test_codegen_type_mapping_primitives() {
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Int8".to_string())),
        "i8"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Int16".to_string())),
        "i16"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Int32".to_string())),
        "i32"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Int64".to_string())),
        "i64"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("UInt8".to_string())),
        "u8"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("UInt16".to_string())),
        "u16"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("UInt32".to_string())),
        "u32"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("UInt64".to_string())),
        "isize"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Float32".to_string())),
        "f32"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Float64".to_string())),
        "f64"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("String".to_string())),
        "String"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Bool".to_string())),
        "bool"
    );
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Named("Void".to_string())),
        "()"
    );
}

#[test]
fn test_codegen_type_mapping_generics() {
    // Option<T>
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Generic {
            name: "Option".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        }),
        "Option<String>"
    );

    // List<T> -> Vec<T>
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![TypeExpr::Named("Int32".to_string())],
        }),
        "Vec<i32>"
    );

    // Map<K, V> -> HashMap<K, V>
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Generic {
            name: "Map".to_string(),
            args: vec![
                TypeExpr::Named("String".to_string()),
                TypeExpr::Named("Int64".to_string())
            ],
        }),
        "std::collections::HashMap<String, i64>"
    );

    // Result<T, E>
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Generic {
            name: "Result".to_string(),
            args: vec![
                TypeExpr::Named("Int32".to_string()),
                TypeExpr::Named("String".to_string())
            ],
        }),
        "Result<i32, String>"
    );
}

#[test]
fn test_codegen_type_mapping_tuples() {
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Tuple(vec![
            TypeExpr::Named("Int32".to_string()),
            TypeExpr::Named("String".to_string()),
        ])),
        "(i32, String)"
    );

    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Tuple(vec![
            TypeExpr::Named("Bool".to_string()),
            TypeExpr::Named("Float64".to_string()),
            TypeExpr::Named("String".to_string()),
        ])),
        "(bool, f64, String)"
    );
}

#[test]
fn test_codegen_type_mapping_functions() {
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Function {
            params: vec![
                TypeExpr::Named("Int32".to_string()),
                TypeExpr::Named("String".to_string()),
            ],
            return_type: Box::new(TypeExpr::Named("Bool".to_string())),
        }),
        "fn(i32, String) -> bool"
    );
}

#[test]
fn test_codegen_type_mapping_nested_generics() {
    // Option<List<String>>
    assert_eq!(
        RustCodegen::map_type_expr(&TypeExpr::Generic {
            name: "Option".to_string(),
            args: vec![TypeExpr::Generic {
                name: "List".to_string(),
                args: vec![TypeExpr::Named("String".to_string())],
            }],
        }),
        "Option<Vec<String>>"
    );
}

// ============================================
// 8. Multiple Declaration Tests
// ============================================

#[test]
fn test_codegen_generate_all() {
    let gene = Gene {
        name: "Point".to_string(),
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "x".to_string(),
            type_: TypeExpr::Named("Int32".to_string()),
            default: None,
            constraint: None,
            span: Span::default(),
        }))],
        exegesis: "A point".to_string(),
        span: Span::default(),
    };

    let trait_decl = Trait {
        name: "Drawable".to_string(),
        statements: vec![Statement::Is {
            subject: "entity".to_string(),
            state: "visible".to_string(),
            span: Span::default(),
        }],
        exegesis: "Can be drawn".to_string(),
        span: Span::default(),
    };

    let decls = vec![Declaration::Gene(gene), Declaration::Trait(trait_decl)];

    let code = RustCodegen::generate_all(&decls);

    assert!(code.contains("pub struct Point"));
    assert!(code.contains("pub trait Drawable"));
}

// ============================================
// 9. Visibility Tests
// ============================================

#[test]
fn test_codegen_gen_visibility() {
    let gen = RustCodegen::new();
    assert_eq!(gen.gen_visibility(Visibility::Public), "pub ");
    assert_eq!(gen.gen_visibility(Visibility::PubSpirit), "pub(crate) ");
    assert_eq!(gen.gen_visibility(Visibility::PubParent), "pub(super) ");
    assert_eq!(gen.gen_visibility(Visibility::Private), "");
}

// ============================================
// 10. Type Info Generation Tests
// ============================================

#[test]
fn test_codegen_type_info() {
    let gen = RustCodegen::new();
    let gene = Gene {
        name: "User".to_string(),
        statements: vec![
            Statement::Has {
                subject: "user".to_string(),
                property: "name".to_string(),
                span: Span::default(),
            },
            Statement::Has {
                subject: "user".to_string(),
                property: "email".to_string(),
                span: Span::default(),
            },
        ],
        exegesis: "A user entity".to_string(),
        span: Span::default(),
    };

    let output = gen.gen_type_info(&gene);
    assert!(output.contains("TypeInfo::record(\"User\")"));
    assert!(output.contains(".with_field(FieldInfo::new(\"name\", \"String\"))"));
    assert!(output.contains(".with_field(FieldInfo::new(\"email\", \"String\"))"));
    assert!(output.contains(".with_doc(\"A user entity\")"));
}

#[test]
fn test_codegen_type_registry() {
    let gen = RustCodegen::new();
    let gene1 = Gene {
        name: "Point".to_string(),
        statements: vec![],
        exegesis: "A point".to_string(),
        span: Span::default(),
    };

    let gene2 = Gene {
        name: "Line".to_string(),
        statements: vec![],
        exegesis: "A line".to_string(),
        span: Span::default(),
    };

    let decls = vec![Declaration::Gene(gene1), Declaration::Gene(gene2)];

    let output = gen.gen_type_registry(&decls);
    assert!(output.contains("pub fn register_types(registry: &mut TypeRegistry)"));
    assert!(output.contains("registry.register("));
    assert!(output.contains("TypeInfo::record(\"Point\")"));
    assert!(output.contains("TypeInfo::record(\"Line\")"));
}

// ============================================
// 11. Parameter Generation Tests
// ============================================

#[test]
fn test_codegen_gen_param() {
    let gen = RustCodegen::new();

    let param = FunctionParam {
        name: "count".to_string(),
        type_ann: TypeExpr::Named("Int64".to_string()),
    };

    let output = gen.gen_param(&param);
    assert_eq!(output, "count: i64");
}

#[test]
fn test_codegen_gen_param_complex_type() {
    let gen = RustCodegen::new();

    let param = FunctionParam {
        name: "items".to_string(),
        type_ann: TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        },
    };

    let output = gen.gen_param(&param);
    assert_eq!(output, "items: Vec<String>");
}

// ============================================
// 12. Naming Convention Tests
// ============================================

#[test]
fn test_codegen_naming_snake_to_pascal() {
    // Test that gene names are converted to PascalCase
    let gene = Gene {
        name: "my.special.gene".to_string(),
        statements: vec![],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(
        code.contains("pub struct MySpecialGene"),
        "Should convert to PascalCase"
    );
}

#[test]
fn test_codegen_naming_field_snake_case() {
    // Test that field names are converted to snake_case
    let gene = Gene {
        name: "Test".to_string(),
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "MyField".to_string(),
            type_: TypeExpr::Named("Int32".to_string()),
            default: None,
            constraint: None,
            span: Span::default(),
        }))],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(
        code.contains("my_field: i32"),
        "Should convert to snake_case"
    );
}

// ============================================
// 13. Edge Cases
// ============================================

#[test]
fn test_codegen_empty_gene() {
    let gene = Gene {
        name: "Empty".to_string(),
        statements: vec![],
        exegesis: "Empty gene".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(code.contains("pub struct Empty"));
    assert!(code.contains("/// Empty gene"));
}

#[test]
fn test_codegen_empty_trait() {
    let trait_decl = Trait {
        name: "Empty".to_string(),
        statements: vec![],
        exegesis: "Empty trait".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Trait(trait_decl));
    assert!(code.contains("pub trait Empty"));
}

#[test]
fn test_codegen_multiline_exegesis() {
    let gene = Gene {
        name: "Test".to_string(),
        statements: vec![],
        exegesis: "Line 1\nLine 2\nLine 3".to_string(),
        span: Span::default(),
    };

    let code = RustCodegen::generate(&Declaration::Gene(gene));
    assert!(code.contains("/// Line 1"));
    assert!(code.contains("/// Line 2"));
    assert!(code.contains("/// Line 3"));
}
