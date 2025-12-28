// ═══════════════════════════════════════════════════════════════════════════════
// DOL HIR TEST HARNESS
// ═══════════════════════════════════════════════════════════════════════════════
//
// tests/hir/mod.rs
//
// Comprehensive test suite for HIR development.
// Run with: cargo test hir::
//
// ═══════════════════════════════════════════════════════════════════════════════

mod types_tests;
mod desugar_tests;
mod validate_tests;

use crate::hir::{HirNode, HirType, HirExpr, HirStmt};
use crate::parser::parse;
use crate::hir::desugar::desugar;
use crate::hir::validate::validate;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse DOL source and desugar to HIR, panicking on failure
pub fn parse_to_hir(source: &str) -> HirNode {
    let ast = parse(source).expect("parse failed");
    desugar(&ast).expect("desugar failed")
}

/// Parse, desugar, and validate, panicking on failure
pub fn parse_validate(source: &str) -> HirNode {
    let hir = parse_to_hir(source);
    validate(&hir).expect("validation failed");
    hir
}

/// Assert that parsing fails with expected error
pub fn assert_parse_error(source: &str, expected_error: &str) {
    let result = parse(source);
    assert!(result.is_err(), "expected parse error, got success");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains(expected_error),
        "expected error containing '{}', got '{}'",
        expected_error,
        err
    );
}

/// Assert that desugaring fails with expected error
pub fn assert_desugar_error(source: &str, expected_error: &str) {
    let ast = parse(source).expect("parse should succeed");
    let result = desugar(&ast);
    assert!(result.is_err(), "expected desugar error, got success");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains(expected_error),
        "expected error containing '{}', got '{}'",
        expected_error,
        err
    );
}

/// Assert that validation fails with expected error
pub fn assert_validation_error(source: &str, expected_error: &str) {
    let hir = parse_to_hir(source);
    let result = validate(&hir);
    assert!(result.is_err(), "expected validation error, got success");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains(expected_error),
        "expected error containing '{}', got '{}'",
        expected_error,
        err
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// FIXTURE MACROS
// ═══════════════════════════════════════════════════════════════════════════════

/// Load fixture file from tests/fixtures/
macro_rules! fixture {
    ($path:expr) => {
        include_str!(concat!("fixtures/", $path))
    };
}

/// Test that a fixture parses and desugars successfully
macro_rules! test_fixture_ok {
    ($name:ident, $path:expr) => {
        #[test]
        fn $name() {
            let source = fixture!($path);
            let _hir = parse_validate(source);
        }
    };
}

/// Test that a fixture fails with expected error
macro_rules! test_fixture_err {
    ($name:ident, $path:expr, $error:expr) => {
        #[test]
        fn $name() {
            let source = fixture!($path);
            assert_validation_error(source, $error);
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE TYPE TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod core_types {
    use super::*;

    #[test]
    fn gene_desugars_to_struct() {
        let hir = parse_to_hir("gene Foo { has x: Int64 }");
        assert!(matches!(hir, HirNode::Type(HirType::Struct { .. })));
    }

    #[test]
    fn trait_desugars_to_interface() {
        let hir = parse_to_hir("trait Bar { is do_thing() -> Int64 }");
        assert!(matches!(hir, HirNode::Type(HirType::Interface { .. })));
    }

    #[test]
    fn let_desugars_to_immutable_binding() {
        let hir = parse_to_hir("let x = 42");
        match hir {
            HirNode::Stmt(HirStmt::Binding { mutable, .. }) => {
                assert!(!mutable, "let should be immutable");
            }
            _ => panic!("expected binding"),
        }
    }

    #[test]
    fn var_desugars_to_mutable_binding() {
        let hir = parse_to_hir("var x = 42");
        match hir {
            HirNode::Stmt(HirStmt::Binding { mutable, .. }) => {
                assert!(mutable, "var should be mutable");
            }
            _ => panic!("expected binding"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXPRESSION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod expressions {
    use super::*;

    #[test]
    fn literal_int() {
        let hir = parse_to_hir("42");
        assert!(matches!(hir, HirNode::Expr(HirExpr::Literal { .. })));
    }

    #[test]
    fn literal_string() {
        let hir = parse_to_hir(r#""hello""#);
        assert!(matches!(hir, HirNode::Expr(HirExpr::Literal { .. })));
    }

    #[test]
    fn binary_add() {
        let hir = parse_to_hir("1 + 2");
        match hir {
            HirNode::Expr(HirExpr::Binary { op, .. }) => {
                assert_eq!(op.to_string(), "+");
            }
            _ => panic!("expected binary"),
        }
    }

    #[test]
    fn pipe_desugars_to_call() {
        let hir = parse_to_hir("x |> f");
        // x |> f becomes f(x)
        match hir {
            HirNode::Expr(HirExpr::Call { callee, args, .. }) => {
                assert_eq!(args.len(), 1);
            }
            _ => panic!("expected call"),
        }
    }

    #[test]
    fn compose_desugars_to_lambda() {
        let hir = parse_to_hir("f >> g");
        // f >> g becomes |x| g(f(x))
        assert!(matches!(hir, HirNode::Expr(HirExpr::Lambda { .. })));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTROL FLOW TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod control_flow {
    use super::*;

    #[test]
    fn if_expression() {
        let hir = parse_to_hir("if true { 1 } else { 2 }");
        assert!(matches!(hir, HirNode::Expr(HirExpr::If { .. })));
    }

    #[test]
    fn match_expression() {
        let hir = parse_to_hir("match x { 0 { zero } _ { other } }");
        assert!(matches!(hir, HirNode::Expr(HirExpr::Match { .. })));
    }

    #[test]
    fn for_loop_desugars() {
        let hir = parse_to_hir("for x in items { process(x) }");
        assert!(matches!(hir, HirNode::Stmt(HirStmt::Loop { .. })));
    }

    #[test]
    fn while_loop_desugars() {
        let hir = parse_to_hir("while running { tick() }");
        assert!(matches!(hir, HirNode::Stmt(HirStmt::Loop { .. })));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VALIDATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod validation {
    use super::*;

    #[test]
    fn valid_function() {
        parse_validate("fun add(a: Int64, b: Int64) -> Int64 { return a + b }");
    }

    #[test]
    fn undefined_variable_error() {
        assert_validation_error(
            "fun f() -> Int64 { return undefined_var }",
            "undefined"
        );
    }

    #[test]
    fn type_mismatch_error() {
        assert_validation_error(
            r#"fun f() -> Int64 { return "string" }"#,
            "type mismatch"
        );
    }

    #[test]
    fn missing_return_error() {
        assert_validation_error(
            "fun f() -> Int64 { let x = 1 }",
            "missing return"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FIXTURE TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod fixtures {
    use super::*;

    // Valid fixtures
    test_fixture_ok!(valid_gene_basic, "valid/gene_basic.dol");
    test_fixture_ok!(valid_gene_methods, "valid/gene_with_methods.dol");
    test_fixture_ok!(valid_control_flow, "valid/function_control_flow.dol");
    test_fixture_ok!(valid_pattern_match, "valid/pattern_matching.dol");
    test_fixture_ok!(valid_pipes, "valid/pipe_operators.dol");

    // Invalid fixtures
    test_fixture_err!(invalid_syntax, "invalid/syntax_error.dol", "expected");
    test_fixture_err!(invalid_type, "invalid/type_error.dol", "type mismatch");
    test_fixture_err!(invalid_undefined, "invalid/undefined_variable.dol", "undefined");
}

// ═══════════════════════════════════════════════════════════════════════════════
// BOOTSTRAP TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod bootstrap {
    use super::*;
    use std::fs;

    #[test]
    fn dol_types_parses() {
        let source = fs::read_to_string("dol/types.dol").expect("read types.dol");
        let _hir = parse_to_hir(&source);
    }

    #[test]
    fn dol_token_parses() {
        let source = fs::read_to_string("dol/token.dol").expect("read token.dol");
        let _hir = parse_to_hir(&source);
    }

    #[test]
    fn dol_ast_parses() {
        let source = fs::read_to_string("dol/ast.dol").expect("read ast.dol");
        let _hir = parse_to_hir(&source);
    }

    #[test]
    fn dol_lexer_parses() {
        let source = fs::read_to_string("dol/lexer.dol").expect("read lexer.dol");
        let _hir = parse_to_hir(&source);
    }
}
