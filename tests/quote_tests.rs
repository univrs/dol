//! Tests for the quote and eval operators in Metal DOL.
//!
//! These tests verify the metaprogramming capabilities of DOL 2.0,
//! including quote (') for capturing expressions as AST data and
//! eval (!) for evaluating quoted expressions.

use metadol::ast::{BinaryOp, Expr, Literal, MatchArm, Pattern, UnaryOp};
use metadol::typechecker::{Type, TypeChecker};

// ============================================
// Helper Functions
// ============================================

fn int_lit(n: i64) -> Expr {
    Expr::Literal(Literal::Int(n))
}

fn float_lit(n: f64) -> Expr {
    Expr::Literal(Literal::Float(n))
}

fn bool_lit(b: bool) -> Expr {
    Expr::Literal(Literal::Bool(b))
}

fn string_lit(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.to_string()))
}

fn ident(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

// ============================================
// AST Construction Tests
// ============================================

#[test]
fn test_quote_literal() {
    // Test that Quote(42) can be constructed
    let expr = Expr::Quote(Box::new(int_lit(42)));

    match expr {
        Expr::Quote(inner) => {
            assert!(matches!(*inner, Expr::Literal(Literal::Int(42))));
        }
        _ => panic!("Expected Quote expression"),
    }
}

#[test]
fn test_quote_binary_expr() {
    // '(1 + 2) should be Quote(Binary(Add, 1, 2))
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(int_lit(2)),
    }));

    match expr {
        Expr::Quote(inner) => match *inner {
            Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Add),
            _ => panic!("Expected Binary expression inside Quote"),
        },
        _ => panic!("Expected Quote expression"),
    }
}

#[test]
fn test_nested_quote() {
    // ''42 should be Quote(Quote(42))
    let expr = Expr::Quote(Box::new(Expr::Quote(Box::new(int_lit(42)))));

    match expr {
        Expr::Quote(inner) => {
            assert!(matches!(*inner, Expr::Quote(_)));
        }
        _ => panic!("Expected nested Quote"),
    }
}

#[test]
fn test_quote_expr_construction() {
    // Test that Quote expressions can be constructed and inspected
    let expr = Expr::Quote(Box::new(bool_lit(true)));

    if let Expr::Quote(inner) = expr {
        if let Expr::Literal(Literal::Bool(b)) = *inner {
            assert!(b);
        } else {
            panic!("Expected Bool literal");
        }
    } else {
        panic!("Expected Quote");
    }
}

#[test]
fn test_eval_expr_construction() {
    // Test Eval expression construction
    let quoted = Expr::Quote(Box::new(string_lit("hello")));
    let expr = Expr::Eval(Box::new(quoted));

    if let Expr::Eval(inner) = expr {
        if let Expr::Quote(_) = *inner {
            // Good
        } else {
            panic!("Expected Quote inside Eval");
        }
    } else {
        panic!("Expected Eval");
    }
}

// ============================================
// Type Checking Tests
// ============================================

#[test]
fn test_quote_type_is_quoted() {
    let mut checker = TypeChecker::new();

    // '42 should have type Quoted<Int64>
    let expr = Expr::Quote(Box::new(int_lit(42)));
    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Type::Int64);
        }
        _ => panic!("Expected Quoted type, got {:?}", ty),
    }
}

#[test]
fn test_quote_preserves_inner_type() {
    let mut checker = TypeChecker::new();

    // '(1.5) should have type Quoted<Float64>
    let expr = Expr::Quote(Box::new(float_lit(1.5)));
    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Type::Float64);
        }
        _ => panic!("Expected Quoted type"),
    }
}

#[test]
fn test_quote_bool_type() {
    let mut checker = TypeChecker::new();

    // 'true should have type Quoted<Bool>
    let expr = Expr::Quote(Box::new(bool_lit(true)));
    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Type::Bool);
        }
        _ => panic!("Expected Quoted type"),
    }
}

#[test]
fn test_quote_string_type() {
    let mut checker = TypeChecker::new();

    // '"hello" should have type Quoted<String>
    let expr = Expr::Quote(Box::new(string_lit("hello")));
    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Type::String);
        }
        _ => panic!("Expected Quoted type"),
    }
}

#[test]
fn test_eval_type_unwraps_quoted() {
    let mut checker = TypeChecker::new();

    // !('42) should have type Int64 (unwrapped from Quoted<Int64>)
    let expr = Expr::Eval(Box::new(Expr::Quote(Box::new(int_lit(42)))));
    let ty = checker.infer(&expr).unwrap();

    assert_eq!(ty, Type::Int64);
}

#[test]
fn test_eval_type_unwraps_quoted_float() {
    let mut checker = TypeChecker::new();

    // !('1.5) should have type Float64
    let expr = Expr::Eval(Box::new(Expr::Quote(Box::new(float_lit(1.5)))));
    let ty = checker.infer(&expr).unwrap();

    assert_eq!(ty, Type::Float64);
}

#[test]
fn test_eval_type_unwraps_quoted_bool() {
    let mut checker = TypeChecker::new();

    // !('true) should have type Bool
    let expr = Expr::Eval(Box::new(Expr::Quote(Box::new(bool_lit(true)))));
    let ty = checker.infer(&expr).unwrap();

    assert_eq!(ty, Type::Bool);
}

// Note: The following test requires TypeChecker.env to be public or
// a public method to bind variables. Currently env is private.
// This test demonstrates the intended behavior but may not compile
// until the API is adjusted.
#[test]
#[ignore = "Requires public access to TypeChecker environment"]
fn test_quote_identifier() {
    let mut checker = TypeChecker::new();

    // Quote of identifier - type depends on identifier's type
    // First bind x to Int32
    // Note: This requires making env public or adding a bind method
    // checker.env.bind("x", Type::Int32);

    let expr = Expr::Quote(Box::new(ident("x")));
    let _ty = checker.infer(&expr);

    // Would check: ty is Quoted<Int32>
}

#[test]
fn test_quote_lambda() {
    let mut checker = TypeChecker::new();

    // '(|x| x + 1) - quote a lambda
    let lambda = Expr::Lambda {
        params: vec![("x".to_string(), None)],
        body: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(ident("x")),
            right: Box::new(int_lit(1)),
        }),
        return_type: None,
    };
    let expr = Expr::Quote(Box::new(lambda));
    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            // Inner type should be a function type
            match &args[0] {
                Type::Function { .. } => {}
                _ => panic!("Expected function type inside Quoted"),
            }
        }
        _ => panic!("Expected Quoted type"),
    }
}

#[test]
fn test_nested_quote_type() {
    let mut checker = TypeChecker::new();

    // ''42 should have type Quoted<Quoted<Int64>>
    let expr = Expr::Quote(Box::new(Expr::Quote(Box::new(int_lit(42)))));
    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic {
            name: outer_name,
            args: outer_args,
        } => {
            assert_eq!(outer_name, "Quoted");
            assert_eq!(outer_args.len(), 1);
            match &outer_args[0] {
                Type::Generic {
                    name: inner_name,
                    args: inner_args,
                } => {
                    assert_eq!(inner_name, "Quoted");
                    assert_eq!(inner_args.len(), 1);
                    assert_eq!(inner_args[0], Type::Int64);
                }
                _ => panic!("Expected nested Quoted type"),
            }
        }
        _ => panic!("Expected Quoted type"),
    }
}

// ============================================
// Round-trip Tests
// ============================================

#[test]
fn test_quote_eval_roundtrip_type() {
    let mut checker = TypeChecker::new();

    // !('(1 + 2)) should have same type as (1 + 2)
    let inner = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(int_lit(2)),
    };

    let original_type = checker.infer(&inner).unwrap();

    let quoted_then_evaled = Expr::Eval(Box::new(Expr::Quote(Box::new(inner.clone()))));
    let roundtrip_type = checker.infer(&quoted_then_evaled).unwrap();

    // Types should be the same
    assert_eq!(original_type, roundtrip_type);
}

#[test]
fn test_quote_eval_roundtrip_literal() {
    let mut checker = TypeChecker::new();

    // !('42) should have same type as 42
    let literal = int_lit(42);
    let original_type = checker.infer(&literal).unwrap();

    let quoted_then_evaled = Expr::Eval(Box::new(Expr::Quote(Box::new(literal.clone()))));
    let roundtrip_type = checker.infer(&quoted_then_evaled).unwrap();

    assert_eq!(original_type, roundtrip_type);
}

#[test]
fn test_double_quote_double_eval_roundtrip() {
    let mut checker = TypeChecker::new();

    // !(!'(''42)) should have same type as 42
    let literal = int_lit(42);
    let original_type = checker.infer(&literal).unwrap();

    let double_quoted = Expr::Quote(Box::new(Expr::Quote(Box::new(literal.clone()))));
    let double_evaled = Expr::Eval(Box::new(Expr::Eval(Box::new(double_quoted))));
    let roundtrip_type = checker.infer(&double_evaled).unwrap();

    assert_eq!(original_type, roundtrip_type);
}

// ============================================
// Complex Expression Tests
// ============================================

#[test]
fn test_quote_complex_expression() {
    // '(if x then 1 else 2)
    let expr = Expr::Quote(Box::new(Expr::If {
        condition: Box::new(ident("x")),
        then_branch: Box::new(int_lit(1)),
        else_branch: Some(Box::new(int_lit(2))),
    }));

    match expr {
        Expr::Quote(inner) => {
            assert!(matches!(*inner, Expr::If { .. }));
        }
        _ => panic!("Expected Quote"),
    }
}

#[test]
fn test_quote_match_expression() {
    // Quote a match expression
    let expr = Expr::Quote(Box::new(Expr::Match {
        scrutinee: Box::new(ident("x")),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(1)),
                guard: None,
                body: Box::new(string_lit("one")),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(string_lit("other")),
            },
        ],
    }));

    match expr {
        Expr::Quote(inner) => {
            assert!(matches!(*inner, Expr::Match { .. }));
        }
        _ => panic!("Expected Quote"),
    }
}

#[test]
fn test_quote_pipeline() {
    // '(a |> b |> c)
    let pipeline = Expr::Binary {
        op: BinaryOp::Pipe,
        left: Box::new(Expr::Binary {
            op: BinaryOp::Pipe,
            left: Box::new(ident("a")),
            right: Box::new(ident("b")),
        }),
        right: Box::new(ident("c")),
    };

    let expr = Expr::Quote(Box::new(pipeline));

    match expr {
        Expr::Quote(inner) => match *inner {
            Expr::Binary {
                op: BinaryOp::Pipe, ..
            } => {}
            _ => panic!("Expected pipe expression inside quote"),
        },
        _ => panic!("Expected Quote"),
    }
}

#[test]
fn test_quote_composition() {
    // '(f >> g >> h)
    let composition = Expr::Binary {
        op: BinaryOp::Compose,
        left: Box::new(Expr::Binary {
            op: BinaryOp::Compose,
            left: Box::new(ident("f")),
            right: Box::new(ident("g")),
        }),
        right: Box::new(ident("h")),
    };

    let expr = Expr::Quote(Box::new(composition));

    match expr {
        Expr::Quote(inner) => match *inner {
            Expr::Binary {
                op: BinaryOp::Compose,
                ..
            } => {}
            _ => panic!("Expected compose expression inside quote"),
        },
        _ => panic!("Expected Quote"),
    }
}

#[test]
fn test_quote_nested_lambdas() {
    // '(|x| |y| x + y)
    let nested_lambda = Expr::Lambda {
        params: vec![("x".to_string(), None)],
        body: Box::new(Expr::Lambda {
            params: vec![("y".to_string(), None)],
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(ident("x")),
                right: Box::new(ident("y")),
            }),
            return_type: None,
        }),
        return_type: None,
    };

    let expr = Expr::Quote(Box::new(nested_lambda));

    match expr {
        Expr::Quote(inner) => match *inner {
            Expr::Lambda { .. } => {}
            _ => panic!("Expected lambda inside quote"),
        },
        _ => panic!("Expected Quote"),
    }
}

#[test]
fn test_quote_call_expression() {
    // '(f(x, y, z))
    let call = Expr::Call {
        callee: Box::new(ident("f")),
        args: vec![ident("x"), ident("y"), ident("z")],
    };

    let expr = Expr::Quote(Box::new(call));

    match expr {
        Expr::Quote(inner) => match *inner {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected call expression inside quote"),
        },
        _ => panic!("Expected Quote"),
    }
}

// ============================================
// Type Checking Error Tests
// ============================================

#[test]
fn test_eval_non_quoted_fails() {
    let mut checker = TypeChecker::new();

    // !(42) should fail - cannot eval a non-quoted expression
    let expr = Expr::Eval(Box::new(int_lit(42)));
    let result = checker.infer(&expr);

    // Should return Error type due to type error
    match result {
        Ok(Type::Error) => {}
        Ok(other) => panic!("Expected Error type, got {:?}", other),
        Err(e) => panic!("Expected Ok(Error), got Err: {:?}", e),
    }

    // Should have collected an error
    assert!(!checker.is_ok(), "Expected type checker to have errors");
}

#[test]
fn test_eval_unknown_type() {
    let mut checker = TypeChecker::new();

    // Eval of an identifier with unknown type
    let expr = Expr::Eval(Box::new(ident("unknown")));
    let result = checker.infer(&expr);

    // Should fail because identifier is undefined
    assert!(result.is_err());
}

// ============================================
// Unary Quote Operator Tests
// ============================================

#[test]
fn test_unary_quote_operator() {
    let mut checker = TypeChecker::new();

    // Using the Unary operator form with Quote
    let expr = Expr::Unary {
        op: UnaryOp::Quote,
        operand: Box::new(int_lit(42)),
    };

    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Type::Int64);
        }
        _ => panic!("Expected Quoted type"),
    }
}

#[test]
fn test_unary_quote_complex_expr() {
    let mut checker = TypeChecker::new();

    // Quote using unary operator on a complex expression
    let expr = Expr::Unary {
        op: UnaryOp::Quote,
        operand: Box::new(Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(int_lit(5)),
            right: Box::new(int_lit(10)),
        }),
    };

    let ty = checker.infer(&expr).unwrap();

    match ty {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            assert!(args[0].is_integer());
        }
        _ => panic!("Expected Quoted type"),
    }
}

// ============================================
// QuotedExpr Conversion Tests
// ============================================

#[test]
fn test_quoted_expr_from_expr() {
    use metadol::ast::QuotedExpr;

    // Test conversion from Expr to QuotedExpr
    let expr = int_lit(42);
    let quoted = QuotedExpr::from_expr(&expr);

    match quoted {
        QuotedExpr::Literal(Literal::Int(42)) => {}
        _ => panic!("Expected literal in QuotedExpr"),
    }
}

#[test]
fn test_quoted_expr_to_expr() {
    use metadol::ast::QuotedExpr;

    // Test conversion from QuotedExpr back to Expr
    let quoted = QuotedExpr::Literal(Literal::Int(42));
    let expr = quoted.to_expr();

    match expr {
        Expr::Literal(Literal::Int(42)) => {}
        _ => panic!("Expected literal in Expr"),
    }
}

#[test]
fn test_quoted_expr_roundtrip() {
    use metadol::ast::QuotedExpr;

    // Test roundtrip conversion
    let original = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(int_lit(2)),
    };

    let quoted = QuotedExpr::from_expr(&original);
    let converted_back = quoted.to_expr();

    // Verify structure is preserved
    match converted_back {
        Expr::Binary {
            op: BinaryOp::Add, ..
        } => {}
        _ => panic!("Expected binary addition after roundtrip"),
    }
}

#[test]
fn test_quoted_expr_nested_quote() {
    use metadol::ast::QuotedExpr;

    // Test QuotedExpr with nested quotes
    let expr = Expr::Quote(Box::new(int_lit(42)));
    let quoted = QuotedExpr::from_expr(&expr);

    match quoted {
        QuotedExpr::Quote(_) => {}
        _ => panic!("Expected Quote in QuotedExpr"),
    }
}

// ============================================
// Integration Tests
// ============================================

#[test]
fn test_quote_preserves_all_literal_types() {
    let mut checker = TypeChecker::new();

    let test_cases = vec![
        (int_lit(42), Type::Int64),
        (float_lit(1.5), Type::Float64),
        (bool_lit(true), Type::Bool),
        (string_lit("test"), Type::String),
    ];

    for (expr, expected_inner) in test_cases {
        let quoted = Expr::Quote(Box::new(expr));
        let ty = checker.infer(&quoted).unwrap();

        match ty {
            Type::Generic { name, args } => {
                assert_eq!(name, "Quoted");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], expected_inner);
            }
            _ => panic!("Expected Quoted type for literal"),
        }
    }
}

#[test]
fn test_arithmetic_in_quoted_context() {
    let mut checker = TypeChecker::new();

    // '(1 + 2 * 3) - quote preserves operator precedence
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(int_lit(2)),
            right: Box::new(int_lit(3)),
        }),
    }));

    // Just verify it type checks without error
    let result = checker.infer(&expr);
    assert!(result.is_ok());

    // Should be Quoted<Int64>
    match result.unwrap() {
        Type::Generic { name, args } => {
            assert_eq!(name, "Quoted");
            assert_eq!(args.len(), 1);
            assert!(args[0].is_integer());
        }
        _ => panic!("Expected Quoted type"),
    }
}
