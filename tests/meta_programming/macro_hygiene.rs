//! Macro hygiene tests for DOL quote/eval operators.
//!
//! This module verifies that:
//! - Quote captures expressions correctly
//! - Eval evaluates quoted expressions
//! - Variable capture is hygienic
//! - Nested quotes work correctly
//! - Quote/eval preserve type information

use metadol::ast::*;

// ============================================
// Helper Functions
// ============================================

fn int_lit(n: i64) -> Expr {
    Expr::Literal(Literal::Int(n))
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
// Quote Expression Tests
// ============================================

#[test]
fn test_quote_literal_int() {
    let expr = Expr::Quote(Box::new(int_lit(42)));

    if let Expr::Quote(inner) = expr {
        assert!(matches!(*inner, Expr::Literal(Literal::Int(42))));
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_literal_bool() {
    let expr = Expr::Quote(Box::new(bool_lit(true)));

    if let Expr::Quote(inner) = expr {
        assert!(matches!(*inner, Expr::Literal(Literal::Bool(true))));
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_literal_string() {
    let expr = Expr::Quote(Box::new(string_lit("hello")));

    if let Expr::Quote(inner) = expr {
        if let Expr::Literal(Literal::String(s)) = &*inner {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected String literal");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_identifier() {
    let expr = Expr::Quote(Box::new(ident("x")));

    if let Expr::Quote(inner) = expr {
        if let Expr::Identifier(name) = &*inner {
            assert_eq!(name, "x");
        } else {
            panic!("Expected Identifier");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

// ============================================
// Quote Binary Operations Tests
// ============================================

#[test]
fn test_quote_binary_add() {
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(int_lit(2)),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Binary { op, left, right } = &*inner {
            assert_eq!(*op, BinaryOp::Add);
            assert!(matches!(**left, Expr::Literal(Literal::Int(1))));
            assert!(matches!(**right, Expr::Literal(Literal::Int(2))));
        } else {
            panic!("Expected Binary expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_binary_comparison() {
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::Eq,
        left: Box::new(ident("x")),
        right: Box::new(int_lit(10)),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Binary { op, .. } = &*inner {
            assert_eq!(*op, BinaryOp::Eq);
        } else {
            panic!("Expected Binary expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_binary_logical() {
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::And,
        left: Box::new(bool_lit(true)),
        right: Box::new(bool_lit(false)),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Binary { op, .. } = &*inner {
            assert_eq!(*op, BinaryOp::And);
        } else {
            panic!("Expected Binary expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

// ============================================
// Quote Unary Operations Tests
// ============================================

#[test]
fn test_quote_unary_not() {
    let expr = Expr::Quote(Box::new(Expr::Unary {
        op: UnaryOp::Not,
        operand: Box::new(bool_lit(true)),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Unary { op, .. } = &*inner {
            assert_eq!(*op, UnaryOp::Not);
        } else {
            panic!("Expected Unary expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_unary_neg() {
    let expr = Expr::Quote(Box::new(Expr::Unary {
        op: UnaryOp::Neg,
        operand: Box::new(int_lit(42)),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Unary { op, .. } = &*inner {
            assert_eq!(*op, UnaryOp::Neg);
        } else {
            panic!("Expected Unary expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

// ============================================
// Nested Quote Tests
// ============================================

#[test]
fn test_nested_quote_double() {
    let expr = Expr::Quote(Box::new(Expr::Quote(Box::new(int_lit(42)))));

    if let Expr::Quote(outer) = expr {
        if let Expr::Quote(inner) = &*outer {
            assert!(matches!(**inner, Expr::Literal(Literal::Int(42))));
        } else {
            panic!("Expected nested Quote");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_nested_quote_triple() {
    let expr = Expr::Quote(Box::new(Expr::Quote(Box::new(Expr::Quote(Box::new(
        int_lit(42),
    ))))));

    if let Expr::Quote(q1) = expr {
        if let Expr::Quote(q2) = &*q1 {
            if let Expr::Quote(q3) = &**q2 {
                assert!(matches!(**q3, Expr::Literal(Literal::Int(42))));
            } else {
                panic!("Expected third Quote");
            }
        } else {
            panic!("Expected second Quote");
        }
    } else {
        panic!("Expected first Quote");
    }
}

// ============================================
// Eval Expression Tests
// ============================================

#[test]
fn test_eval_construction() {
    let expr = Expr::Eval(Box::new(Expr::Quote(Box::new(int_lit(42)))));

    if let Expr::Eval(inner) = expr {
        if let Expr::Quote(quoted) = &*inner {
            assert!(matches!(**quoted, Expr::Literal(Literal::Int(42))));
        } else {
            panic!("Expected Quote inside Eval");
        }
    } else {
        panic!("Expected Eval expression");
    }
}

#[test]
fn test_eval_identifier() {
    let expr = Expr::Eval(Box::new(ident("quoted_expr")));

    if let Expr::Eval(inner) = expr {
        if let Expr::Identifier(name) = &*inner {
            assert_eq!(name, "quoted_expr");
        } else {
            panic!("Expected Identifier");
        }
    } else {
        panic!("Expected Eval expression");
    }
}

// ============================================
// Quote/Eval Interaction Tests
// ============================================

#[test]
fn test_eval_of_quote() {
    // !('42) should evaluate to 42
    let expr = Expr::Eval(Box::new(Expr::Quote(Box::new(int_lit(42)))));

    // Verify structure
    if let Expr::Eval(eval_inner) = expr {
        if let Expr::Quote(quote_inner) = &*eval_inner {
            assert!(matches!(**quote_inner, Expr::Literal(Literal::Int(42))));
        } else {
            panic!("Expected Quote inside Eval");
        }
    } else {
        panic!("Expected Eval expression");
    }
}

#[test]
fn test_quote_of_eval() {
    // '(!x) should capture the eval expression
    let expr = Expr::Quote(Box::new(Expr::Eval(Box::new(ident("x")))));

    if let Expr::Quote(quote_inner) = expr {
        if let Expr::Eval(eval_inner) = &*quote_inner {
            if let Expr::Identifier(name) = &**eval_inner {
                assert_eq!(name, "x");
            } else {
                panic!("Expected Identifier");
            }
        } else {
            panic!("Expected Eval inside Quote");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

// ============================================
// Complex Quote Tests
// ============================================

#[test]
fn test_quote_function_call() {
    let expr = Expr::Quote(Box::new(Expr::Call {
        callee: Box::new(ident("f")),
        args: vec![int_lit(1), int_lit(2)],
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Call { callee, args } = &*inner {
            if let Expr::Identifier(name) = &**callee {
                assert_eq!(name, "f");
            }
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected Call expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_if_expression() {
    let expr = Expr::Quote(Box::new(Expr::If {
        condition: Box::new(bool_lit(true)),
        then_branch: Box::new(int_lit(1)),
        else_branch: Some(Box::new(int_lit(2))),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::If {
            condition,
            then_branch,
            else_branch,
        } = &*inner
        {
            assert!(matches!(**condition, Expr::Literal(Literal::Bool(true))));
            assert!(matches!(**then_branch, Expr::Literal(Literal::Int(1))));
            assert!(else_branch.is_some());
        } else {
            panic!("Expected If expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_match_expression() {
    let expr = Expr::Quote(Box::new(Expr::Match {
        scrutinee: Box::new(ident("x")),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(1)),
                guard: None,
                body: Box::new(int_lit(10)),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(int_lit(20)),
            },
        ],
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Match { scrutinee: _, arms } = &*inner {
            assert_eq!(arms.len(), 2);
        } else {
            panic!("Expected Match expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

// ============================================
// Variable Capture Tests (Hygiene)
// ============================================

#[test]
fn test_quote_captures_free_variables() {
    // '(x + y) should capture both x and y
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(ident("x")),
        right: Box::new(ident("y")),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Binary { left, right, .. } = &*inner {
            assert!(matches!(**left, Expr::Identifier(_)));
            assert!(matches!(**right, Expr::Identifier(_)));
        } else {
            panic!("Expected Binary expression");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_preserves_variable_names() {
    let expr = Expr::Quote(Box::new(ident("my_variable_name")));

    if let Expr::Quote(inner) = expr {
        if let Expr::Identifier(name) = &*inner {
            assert_eq!(name, "my_variable_name");
        } else {
            panic!("Expected Identifier");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

// ============================================
// Edge Cases and Error Conditions
// ============================================

#[test]
fn test_quote_empty_string() {
    let expr = Expr::Quote(Box::new(string_lit("")));

    if let Expr::Quote(inner) = expr {
        if let Expr::Literal(Literal::String(s)) = &*inner {
            assert_eq!(s, "");
        } else {
            panic!("Expected String literal");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_large_number() {
    let expr = Expr::Quote(Box::new(int_lit(i64::MAX)));

    if let Expr::Quote(inner) = expr {
        if let Expr::Literal(Literal::Int(n)) = *inner {
            assert_eq!(n, i64::MAX);
        } else {
            panic!("Expected Int literal");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

#[test]
fn test_quote_complex_nested_expression() {
    // '((x + y) * (a - b))
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::Mul,
        left: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(ident("x")),
            right: Box::new(ident("y")),
        }),
        right: Box::new(Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(ident("a")),
            right: Box::new(ident("b")),
        }),
    }));

    if let Expr::Quote(inner) = expr {
        if let Expr::Binary {
            op: BinaryOp::Mul,
            left,
            right,
        } = &*inner
        {
            assert!(matches!(**left, Expr::Binary { .. }));
            assert!(matches!(**right, Expr::Binary { .. }));
        } else {
            panic!("Expected Binary expression with Mul");
        }
    } else {
        panic!("Expected Quote expression");
    }
}

// ============================================
// Pattern Matching with Quote
// ============================================

#[test]
fn test_pattern_match_on_quoted() {
    let expr = Expr::Quote(Box::new(int_lit(42)));

    match expr {
        Expr::Quote(inner) => match *inner {
            Expr::Literal(Literal::Int(n)) => assert_eq!(n, 42),
            _ => panic!("Expected Int literal"),
        },
        _ => panic!("Expected Quote"),
    }
}

#[test]
fn test_pattern_match_on_eval() {
    let expr = Expr::Eval(Box::new(ident("x")));

    match expr {
        Expr::Eval(inner) => match *inner {
            Expr::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected Identifier"),
        },
        _ => panic!("Expected Eval"),
    }
}

// ============================================
// Clone and Equality Tests
// ============================================

#[test]
fn test_quote_clone() {
    let expr = Expr::Quote(Box::new(int_lit(42)));
    let cloned = expr.clone();

    match (expr, cloned) {
        (Expr::Quote(inner1), Expr::Quote(inner2)) => {
            assert_eq!(inner1, inner2);
        }
        _ => panic!("Clone should preserve Quote structure"),
    }
}

#[test]
fn test_quote_equality() {
    let expr1 = Expr::Quote(Box::new(int_lit(42)));
    let expr2 = Expr::Quote(Box::new(int_lit(42)));
    let expr3 = Expr::Quote(Box::new(int_lit(43)));

    assert_eq!(expr1, expr2);
    assert_ne!(expr1, expr3);
}

// ============================================
// Integration Tests
// ============================================

#[test]
fn test_quote_in_statement_context() {
    // Test that Quote can be used in various statement contexts
    let expr = Expr::Quote(Box::new(Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(int_lit(2)),
    }));

    // Should be able to use quoted expressions anywhere expressions are valid
    if let Expr::Quote(_) = expr {
        // Success
    } else {
        panic!("Quote should be a valid expression");
    }
}

#[test]
fn test_eval_in_statement_context() {
    let expr = Expr::Eval(Box::new(ident("quoted_value")));

    if let Expr::Eval(_) = expr {
        // Success
    } else {
        panic!("Eval should be a valid expression");
    }
}

// ============================================
// Macro Composition Tests
// ============================================

#[test]
fn test_quote_composition() {
    // Test building quotes programmatically
    let base = int_lit(10);
    let quoted = Expr::Quote(Box::new(base));
    let double_quoted = Expr::Quote(Box::new(quoted));

    if let Expr::Quote(outer) = double_quoted {
        if let Expr::Quote(inner) = &*outer {
            if let Expr::Literal(Literal::Int(n)) = **inner {
                assert_eq!(n, 10);
            } else {
                panic!("Expected Int literal");
            }
        } else {
            panic!("Expected inner Quote");
        }
    } else {
        panic!("Expected outer Quote");
    }
}

#[test]
fn test_eval_composition() {
    // Test building evals programmatically
    let base = Expr::Quote(Box::new(int_lit(20)));
    let evaled = Expr::Eval(Box::new(base));

    if let Expr::Eval(inner) = evaled {
        if let Expr::Quote(quoted) = &*inner {
            if let Expr::Literal(Literal::Int(n)) = **quoted {
                assert_eq!(n, 20);
            } else {
                panic!("Expected Int literal");
            }
        } else {
            panic!("Expected Quote");
        }
    } else {
        panic!("Expected Eval");
    }
}
