//! Comprehensive tests for SEX (Side Effect eXecution) system
//!
//! Tests cover:
//! - File context detection
//! - Effect tracking
//! - Linting
//! - Type checker effect context
//! - Rust code generation

use metadol::ast::{
    Declaration, Expr, ExternDecl, FunctionDecl, FunctionParam, Gene, Literal, Mutability, Purity,
    Span, Statement, Stmt, TypeExpr, VarDecl, Visibility,
};
use metadol::codegen::RustCodegen;
use metadol::sex::context::SexContext;
use metadol::sex::tracking::{Effect, EffectKind, EffectTracker};
use metadol::sex::{file_sex_context, is_sex_file, LintResult, SexLinter};
use metadol::typechecker::{EffectContext, TypeChecker};
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════
// File Context Detection Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_sex_file_extension() {
    assert!(is_sex_file(Path::new("io.sex.dol")));
    assert!(is_sex_file(Path::new("network.sex.dol")));
    assert!(is_sex_file(Path::new("src/ffi.sex.dol")));
    assert!(!is_sex_file(Path::new("container.dol")));
    assert!(!is_sex_file(Path::new("pure.dol")));
}

#[test]
fn test_sex_directory() {
    assert!(is_sex_file(Path::new("sex/globals.dol")));
    assert!(is_sex_file(Path::new("src/sex/io.dol")));
    assert!(is_sex_file(Path::new("lib/sex/ffi.dol")));
    assert!(!is_sex_file(Path::new("src/genes/process.dol")));
    assert!(!is_sex_file(Path::new("traits/lifecycle.dol")));
}

#[test]
fn test_file_sex_context_pure() {
    let ctx = file_sex_context(Path::new("container.dol"));
    assert_eq!(ctx, SexContext::Pure);
}

#[test]
fn test_file_sex_context_sex() {
    let ctx = file_sex_context(Path::new("io.sex.dol"));
    assert_eq!(ctx, SexContext::Sex);

    let ctx2 = file_sex_context(Path::new("sex/globals.dol"));
    assert_eq!(ctx2, SexContext::Sex);
}

#[test]
fn test_sex_context_is_pure() {
    assert!(SexContext::Pure.is_pure());
    assert!(!SexContext::Sex.is_pure());
}

#[test]
fn test_sex_context_is_sex() {
    assert!(SexContext::Sex.is_sex());
    assert!(!SexContext::Pure.is_sex());
}

#[test]
fn test_sex_context_display() {
    assert_eq!(SexContext::Pure.to_string(), "pure");
    assert_eq!(SexContext::Sex.to_string(), "sex");
}

// ═══════════════════════════════════════════════════════════════════
// Effect Tracking Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_effect_kind_display() {
    assert_eq!(EffectKind::Io.to_string(), "I/O operation");
    assert_eq!(EffectKind::Ffi.to_string(), "FFI call");
    assert_eq!(
        EffectKind::MutableGlobal.to_string(),
        "mutable global state"
    );
    assert_eq!(
        EffectKind::NonDeterministic.to_string(),
        "non-deterministic operation"
    );
    assert_eq!(EffectKind::Unsafe.to_string(), "unsafe operation");
    assert_eq!(EffectKind::General.to_string(), "side effect");
}

#[test]
fn test_effect_kind_description() {
    assert_eq!(EffectKind::Io.description(), "I/O operation");
    assert_eq!(EffectKind::Ffi.description(), "FFI call");
    assert_eq!(
        EffectKind::MutableGlobal.description(),
        "mutable global state"
    );
    assert_eq!(
        EffectKind::NonDeterministic.description(),
        "non-deterministic operation"
    );
    assert_eq!(EffectKind::Unsafe.description(), "unsafe operation");
    assert_eq!(EffectKind::General.description(), "side effect");
}

#[test]
fn test_effect_creation() {
    let effect = Effect::new(EffectKind::Io, Span::default());
    assert_eq!(effect.kind, EffectKind::Io);
    assert!(effect.context.is_none());
}

#[test]
fn test_effect_with_context() {
    let effect = Effect::with_context(
        EffectKind::Ffi,
        Span::default(),
        "calling malloc".to_string(),
    );
    assert_eq!(effect.kind, EffectKind::Ffi);
    assert_eq!(effect.context, Some("calling malloc".to_string()));
}

#[test]
fn test_effect_tracker_creation() {
    let tracker = EffectTracker::new();
    assert!(!tracker.has_effects("nonexistent"));
}

#[test]
fn test_effect_tracker_purity_context() {
    let mut tracker = EffectTracker::new();

    // Default is none
    assert_eq!(tracker.get_purity("test"), None);

    // Set purity
    tracker.set_purity("test".to_string(), Purity::Sex);
    assert_eq!(tracker.get_purity("test"), Some(Purity::Sex));

    tracker.set_purity("pure_fn".to_string(), Purity::Pure);
    assert_eq!(tracker.get_purity("pure_fn"), Some(Purity::Pure));
}

#[test]
fn test_effect_tracker_gene_with_io() {
    let mut tracker = EffectTracker::new();

    let gene = Gene {
        name: "io.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "io".to_string(),
            property: "file_read".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    tracker.track_declaration(&Declaration::Gene(gene));
    assert!(tracker.has_effects("io.gene"));

    let effects = tracker.get_effects("io.gene");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Io);
}

#[test]
fn test_effect_tracker_gene_with_ffi() {
    let mut tracker = EffectTracker::new();

    let gene = Gene {
        name: "ffi.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "ffi".to_string(),
            property: "extern_call".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    tracker.track_declaration(&Declaration::Gene(gene));
    assert!(tracker.has_effects("ffi.gene"));

    let effects = tracker.get_effects("ffi.gene");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::Ffi);
}

#[test]
fn test_effect_tracker_gene_with_global() {
    let mut tracker = EffectTracker::new();

    let gene = Gene {
        name: "global.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "state".to_string(),
            property: "global_counter".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    tracker.track_declaration(&Declaration::Gene(gene));
    assert!(tracker.has_effects("global.gene"));

    let effects = tracker.get_effects("global.gene");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::MutableGlobal);
}

#[test]
fn test_effect_tracker_pure_gene() {
    let mut tracker = EffectTracker::new();

    let gene = Gene {
        name: "pure.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "thing".to_string(),
            property: "property".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test".to_string(),
        span: Span::default(),
    };

    tracker.track_declaration(&Declaration::Gene(gene));
    assert!(!tracker.has_effects("pure.gene"));
}

// ═══════════════════════════════════════════════════════════════════
// Linting Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_lint_result_creation() {
    let result = LintResult::new();
    assert!(result.is_clean());
    assert!(!result.has_errors());
    assert!(!result.has_warnings());
}

#[test]
fn test_lint_result_with_error() {
    let mut result = LintResult::new();

    result.add_error(metadol::sex::SexLintError::IoOutsideSex {
        operation: "println".to_string(),
        span: Span::default(),
    });

    assert!(result.has_errors());
    assert!(!result.is_clean());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn test_lint_result_with_warning() {
    let mut result = LintResult::new();

    result.add_warning(metadol::sex::SexLintWarning::LargeSexBlock {
        size: 100,
        max_size: 50,
        span: Span::default(),
    });

    assert!(result.has_warnings());
    assert!(!result.is_clean());
    assert_eq!(result.warnings.len(), 1);
}

#[test]
fn test_linter_pure_gene_no_effects() {
    let linter = SexLinter::new(SexContext::Pure);

    let gene = Gene {
        name: "test.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "test".to_string(),
            property: "property".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test gene with sufficient documentation for linting".to_string(),
        span: Span::default(),
    };

    let result = linter.lint_declaration(&Declaration::Gene(gene));
    assert!(result.is_clean());
}

#[test]
fn test_linter_io_in_pure_context() {
    let linter = SexLinter::new(SexContext::Pure);

    let gene = Gene {
        name: "io.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "io".to_string(),
            property: "file_read".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test gene with I/O operations in pure context".to_string(),
        span: Span::default(),
    };

    let result = linter.lint_declaration(&Declaration::Gene(gene));
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].code(), "E004");
}

#[test]
fn test_linter_ffi_in_pure_context() {
    let linter = SexLinter::new(SexContext::Pure);

    let gene = Gene {
        name: "ffi.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "ffi".to_string(),
            property: "extern_func".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test gene with FFI in pure context".to_string(),
        span: Span::default(),
    };

    let result = linter.lint_declaration(&Declaration::Gene(gene));
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].code(), "E003");
}

#[test]
fn test_linter_mutable_global_in_pure_context() {
    let linter = SexLinter::new(SexContext::Pure);

    let gene = Gene {
        name: "global.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "state".to_string(),
            property: "global_var".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test gene with global state in pure context".to_string(),
        span: Span::default(),
    };

    let result = linter.lint_declaration(&Declaration::Gene(gene));
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].code(), "E002");
}

#[test]
fn test_linter_large_block_warning() {
    let linter = SexLinter::new(SexContext::Pure).with_max_block_size(5);

    let statements: Vec<Statement> = (0..10)
        .map(|i| Statement::Has {
            subject: "test".to_string(),
            property: format!("prop{}", i),
            span: Span::default(),
        })
        .collect();

    let gene = Gene {
        name: "test.gene".to_string(),
        statements,
        exegesis: "Test gene with many statements".to_string(),
        span: Span::default(),
    };

    let result = linter.lint_declaration(&Declaration::Gene(gene));
    assert!(result.has_warnings());
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0].code(), "W001");
}

#[test]
fn test_linter_sex_without_documentation() {
    let linter = SexLinter::new(SexContext::Sex);

    let gene = Gene {
        name: "test.gene".to_string(),
        statements: vec![],
        exegesis: "Short".to_string(), // Too short
        span: Span::default(),
    };

    let result = linter.lint_declaration(&Declaration::Gene(gene));
    assert!(result.has_warnings());
    assert_eq!(result.warnings[0].code(), "W002");
}

#[test]
fn test_linter_sex_context_allows_effects() {
    let linter = SexLinter::new(SexContext::Sex);

    let gene = Gene {
        name: "io.gene".to_string(),
        statements: vec![Statement::Has {
            subject: "io".to_string(),
            property: "file_read".to_string(),
            span: Span::default(),
        }],
        exegesis: "Test gene with I/O operations in sex context - this is allowed".to_string(),
        span: Span::default(),
    };

    let result = linter.lint_declaration(&Declaration::Gene(gene));
    // Should not have errors since we're in sex context
    assert!(!result.has_errors());
}

// ═══════════════════════════════════════════════════════════════════
// Type Checker Effect Context Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_typechecker_default_context() {
    let checker = TypeChecker::new();
    assert_eq!(checker.current_effect_context(), EffectContext::Pure);
    assert!(!checker.in_sex_context());
}

#[test]
fn test_typechecker_enter_exit_context() {
    let mut checker = TypeChecker::new();

    // Enter sex context
    checker.enter_sex_context();
    assert!(checker.in_sex_context());
    assert_eq!(checker.current_effect_context(), EffectContext::Sex);

    // Exit sex context
    checker.exit_sex_context();
    assert!(!checker.in_sex_context());
    assert_eq!(checker.current_effect_context(), EffectContext::Pure);
}

#[test]
fn test_typechecker_nested_contexts() {
    let mut checker = TypeChecker::new();

    // Pure -> Sex -> Sex -> Pure
    checker.enter_sex_context();
    assert!(checker.in_sex_context());

    checker.enter_sex_context();
    assert!(checker.in_sex_context());

    checker.exit_sex_context();
    assert!(checker.in_sex_context());

    checker.exit_sex_context();
    assert!(!checker.in_sex_context());
}

#[test]
fn test_typechecker_sex_block_inference() {
    let mut checker = TypeChecker::new();

    // Create a sex block with an integer literal
    let sex_block = Expr::SexBlock {
        statements: vec![],
        final_expr: Some(Box::new(Expr::Literal(Literal::Int(42)))),
    };

    let ty = checker.infer(&sex_block).unwrap();
    assert_eq!(ty, metadol::typechecker::Type::Int64);

    // Should be back in pure context after inference
    assert!(!checker.in_sex_context());
}

#[test]
fn test_typechecker_sex_block_with_statements() {
    let mut checker = TypeChecker::new();

    // Create a sex block with statements and final expression
    let sex_block = Expr::SexBlock {
        statements: vec![Stmt::Let {
            name: "x".to_string(),
            type_ann: None,
            value: Expr::Literal(Literal::Int(10)),
        }],
        final_expr: Some(Box::new(Expr::Literal(Literal::Bool(true)))),
    };

    let ty = checker.infer(&sex_block).unwrap();
    assert_eq!(ty, metadol::typechecker::Type::Bool);

    // Should be back in pure context after inference
    assert!(!checker.in_sex_context());
}

// ═══════════════════════════════════════════════════════════════════
// Rust Code Generation Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_codegen_visibility() {
    let gen = RustCodegen::new();

    assert_eq!(gen.gen_visibility(Visibility::Public), "pub ");
    assert_eq!(gen.gen_visibility(Visibility::PubSpirit), "pub(crate) ");
    assert_eq!(gen.gen_visibility(Visibility::PubParent), "pub(super) ");
    assert_eq!(gen.gen_visibility(Visibility::Private), "");
}

#[test]
fn test_codegen_primitive_types() {
    let gen = RustCodegen::new();

    assert_eq!(gen.gen_type(&TypeExpr::Named("Int8".to_string())), "i8");
    assert_eq!(gen.gen_type(&TypeExpr::Named("Int16".to_string())), "i16");
    assert_eq!(gen.gen_type(&TypeExpr::Named("Int32".to_string())), "i32");
    assert_eq!(gen.gen_type(&TypeExpr::Named("Int64".to_string())), "i64");
    assert_eq!(gen.gen_type(&TypeExpr::Named("UInt8".to_string())), "u8");
    assert_eq!(gen.gen_type(&TypeExpr::Named("UInt16".to_string())), "u16");
    assert_eq!(gen.gen_type(&TypeExpr::Named("UInt32".to_string())), "u32");
    assert_eq!(gen.gen_type(&TypeExpr::Named("UInt64".to_string())), "u64");
    assert_eq!(gen.gen_type(&TypeExpr::Named("Float32".to_string())), "f32");
    assert_eq!(gen.gen_type(&TypeExpr::Named("Float64".to_string())), "f64");
    assert_eq!(gen.gen_type(&TypeExpr::Named("Bool".to_string())), "bool");
    assert_eq!(
        gen.gen_type(&TypeExpr::Named("String".to_string())),
        "String"
    );
    assert_eq!(gen.gen_type(&TypeExpr::Named("Void".to_string())), "()");
}

#[test]
fn test_codegen_generic_types() {
    let gen = RustCodegen::new();

    let vec_type = TypeExpr::Generic {
        name: "Vec".to_string(),
        args: vec![TypeExpr::Named("Int32".to_string())],
    };
    assert_eq!(gen.gen_type(&vec_type), "Vec<i32>");

    let option_type = TypeExpr::Generic {
        name: "Option".to_string(),
        args: vec![TypeExpr::Named("String".to_string())],
    };
    assert_eq!(gen.gen_type(&option_type), "Option<String>");
}

#[test]
fn test_codegen_constant() {
    let gen = RustCodegen::new();

    let var = VarDecl {
        mutability: Mutability::Immutable,
        name: "MAX_SIZE".to_string(),
        type_ann: Some(TypeExpr::Named("Int64".to_string())),
        value: Some(Expr::Literal(Literal::Int(100))),
        span: Span::default(),
    };

    let output = gen.gen_constant(&var);
    assert!(output.contains("const MAX_SIZE: i64 = 100;"));
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
    assert!(output.contains("static mut COUNTER: i64 = 0;"));
}

#[test]
fn test_codegen_extern_function() {
    let gen = RustCodegen::new();

    let decl = ExternDecl {
        abi: Some("C".to_string()),
        name: "malloc".to_string(),
        params: vec![FunctionParam {
            name: "size".to_string(),
            type_ann: TypeExpr::Named("UInt64".to_string()),
        }],
        return_type: Some(TypeExpr::Generic {
            name: "Ptr".to_string(),
            args: vec![TypeExpr::Named("Void".to_string())],
        }),
        span: Span::default(),
    };

    let output = gen.gen_extern(&decl);
    assert!(output.contains("extern \"C\" {"));
    assert!(output.contains("fn malloc(size: u64)"));
}

#[test]
fn test_codegen_extern_block() {
    let gen = RustCodegen::new();

    let functions = vec![
        ExternDecl {
            abi: None,
            name: "getpid".to_string(),
            params: vec![],
            return_type: Some(TypeExpr::Named("Int32".to_string())),
            span: Span::default(),
        },
        ExternDecl {
            abi: None,
            name: "fork".to_string(),
            params: vec![],
            return_type: Some(TypeExpr::Named("Int32".to_string())),
            span: Span::default(),
        },
    ];

    let output = gen.gen_extern_block(Some("C"), &functions);
    assert!(output.contains("extern \"C\" {"));
    assert!(output.contains("fn getpid() -> i32;"));
    assert!(output.contains("fn fork() -> i32;"));
}

#[test]
fn test_codegen_global_access_wrapper() {
    let gen = RustCodegen::new();
    assert_eq!(gen.gen_global_access("counter"), "unsafe { COUNTER }");
}

#[test]
fn test_codegen_global_mutation_wrapper() {
    let gen = RustCodegen::new();
    assert_eq!(
        gen.gen_global_mutation("counter", "42"),
        "unsafe { COUNTER = 42; }"
    );
}

#[test]
fn test_codegen_sex_function() {
    let gen = RustCodegen::new();

    let func = FunctionDecl {
        visibility: Visibility::Public,
        purity: Purity::Sex,
        name: "log".to_string(),
        type_params: None,
        params: vec![FunctionParam {
            name: "msg".to_string(),
            type_ann: TypeExpr::Named("String".to_string()),
        }],
        return_type: Some(TypeExpr::Named("Void".to_string())),
        body: vec![],
        span: Span::default(),
    };

    let output = gen.gen_sex_function(Visibility::Public, &func);
    assert!(output.contains("/// Side-effectful function"));
    assert!(output.contains("pub fn log(msg: String) -> ()"));
}

#[test]
fn test_codegen_sex_block() {
    let gen = RustCodegen::new();

    let statements = vec![Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Identifier("println".to_string())),
        args: vec![Expr::Literal(Literal::String("hello".to_string()))],
    })];

    let output = gen.gen_sex_block(&statements, None);
    assert!(output.contains("/* sex block */"));
    assert!(output.contains("println(\"hello\")"));
}

// ═══════════════════════════════════════════════════════════════════
// Integration Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_full_sex_workflow() {
    // 1. Check file context
    let path = Path::new("io.sex.dol");
    assert!(is_sex_file(path));
    assert_eq!(file_sex_context(path), SexContext::Sex);

    // 2. Track effects
    let mut tracker = EffectTracker::new();
    tracker.set_purity("log".to_string(), Purity::Sex);
    assert_eq!(tracker.get_purity("log"), Some(Purity::Sex));

    // 3. Type check with effect context
    let mut checker = TypeChecker::new();
    checker.enter_sex_context();
    let expr = Expr::Literal(Literal::Int(42));
    let _ = checker.infer(&expr);
    checker.exit_sex_context();

    // 4. Generate code
    let gen = RustCodegen::new();
    let var = VarDecl {
        mutability: Mutability::Mutable,
        name: "counter".to_string(),
        type_ann: Some(TypeExpr::Named("Int64".to_string())),
        value: Some(Expr::Literal(Literal::Int(0))),
        span: Span::default(),
    };
    let output = gen.gen_global_var(&var);
    assert!(output.contains("static mut COUNTER"));
}

#[test]
fn test_purity_annotations() {
    let mut tracker = EffectTracker::new();

    // Test setting and getting different purity levels
    tracker.set_purity("pure_func".to_string(), Purity::Pure);
    tracker.set_purity("io_func".to_string(), Purity::Sex);

    assert_eq!(tracker.get_purity("pure_func"), Some(Purity::Pure));
    assert_eq!(tracker.get_purity("io_func"), Some(Purity::Sex));
    assert_eq!(tracker.get_purity("unknown_func"), None);
}

#[test]
fn test_error_codes() {
    use metadol::sex::{SexLintError, SexLintWarning};

    let err1 = SexLintError::SexInPureContext {
        effect_kind: EffectKind::Io,
        span: Span::default(),
        message: "test".to_string(),
    };
    assert_eq!(err1.code(), "E001");

    let err2 = SexLintError::MutableGlobalOutsideSex {
        name: "test".to_string(),
        span: Span::default(),
    };
    assert_eq!(err2.code(), "E002");

    let err3 = SexLintError::FfiOutsideSex {
        name: "test".to_string(),
        span: Span::default(),
    };
    assert_eq!(err3.code(), "E003");

    let err4 = SexLintError::IoOutsideSex {
        operation: "test".to_string(),
        span: Span::default(),
    };
    assert_eq!(err4.code(), "E004");

    let warn1 = SexLintWarning::LargeSexBlock {
        size: 100,
        max_size: 50,
        span: Span::default(),
    };
    assert_eq!(warn1.code(), "W001");

    let warn2 = SexLintWarning::SexFunctionWithoutDocumentation {
        name: "test".to_string(),
        span: Span::default(),
    };
    assert_eq!(warn2.code(), "W002");
}
