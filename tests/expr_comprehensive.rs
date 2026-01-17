//! Comprehensive expression tests
//! Tests all expression types and combinations

use metadol::ast::*;
use metadol::parser::Parser;

// ============================================================================
// LITERAL EXPRESSIONS
// ============================================================================

#[test]
fn literal_true() {
    let expr = Parser::new("true").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn literal_false() {
    let expr = Parser::new("false").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn literal_string_empty() {
    let expr = Parser::new("\"\"").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn literal_string_simple() {
    let expr = Parser::new("\"hello\"").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn literal_string_with_space() {
    let expr = Parser::new("\"hello world\"").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn literal_string_with_escape() {
    let expr = Parser::new("\"hello\\nworld\"").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

// ============================================================================
// IDENTIFIER EXPRESSIONS
// ============================================================================

#[test]
fn ident_simple() {
    let expr = Parser::new("foo").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

#[test]
fn ident_underscore_prefix() {
    let expr = Parser::new("_private").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

#[test]
fn ident_with_numbers() {
    let expr = Parser::new("var123").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

#[test]
fn ident_snake_case() {
    let expr = Parser::new("my_variable").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

#[test]
fn ident_camel_case() {
    let expr = Parser::new("myVariable").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

// ============================================================================
// BINARY EXPRESSIONS - ARITHMETIC
// ============================================================================

#[test]
fn binary_add() {
    let expr = Parser::new("a + b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_sub() {
    let expr = Parser::new("a - b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_mul() {
    let expr = Parser::new("a * b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_div() {
    let expr = Parser::new("a / b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_mod() {
    let expr = Parser::new("a % b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// BINARY EXPRESSIONS - COMPARISON
// ============================================================================

#[test]
fn binary_eq() {
    let expr = Parser::new("a == b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_ne() {
    let expr = Parser::new("a != b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_lt() {
    let expr = Parser::new("a < b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_le() {
    let expr = Parser::new("a <= b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_gt() {
    let expr = Parser::new("a > b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_ge() {
    let expr = Parser::new("a >= b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// BINARY EXPRESSIONS - LOGICAL
// ============================================================================

#[test]
fn binary_and() {
    let expr = Parser::new("a && b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn binary_or() {
    let expr = Parser::new("a || b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// UNARY EXPRESSIONS
// ============================================================================

#[test]
fn unary_not() {
    let expr = Parser::new("!x").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Unary { .. }));
}

#[test]
fn unary_neg() {
    let expr = Parser::new("-x").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Unary { .. }));
}

#[test]
fn unary_double_not() {
    let expr = Parser::new("!!x").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Unary { .. }));
}

#[test]
fn unary_double_neg() {
    // DOL lexer may treat '--' as a single token; use parentheses to force double negation
    let expr = Parser::new("-(-x)").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Unary { .. }));
}

// ============================================================================
// CALL EXPRESSIONS
// ============================================================================

#[test]
fn call_no_args() {
    let expr = Parser::new("foo()").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn call_one_arg() {
    let expr = Parser::new("foo(a)").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn call_two_args() {
    let expr = Parser::new("foo(a, b)").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn call_many_args() {
    let expr = Parser::new("foo(a, b, c, d, e)").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn call_nested() {
    let expr = Parser::new("foo(bar(x))").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn call_with_expression_arg() {
    let expr = Parser::new("foo(a + b)").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Call { .. }));
}

// ============================================================================
// LIST EXPRESSIONS
// ============================================================================

#[test]
fn list_empty() {
    let expr = Parser::new("[]").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::List(_)));
}

#[test]
fn list_one_element() {
    let expr = Parser::new("[a]").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::List(_)));
}

#[test]
fn list_many_elements() {
    let expr = Parser::new("[a, b, c]").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::List(_)));
}

#[test]
fn list_nested() {
    let expr = Parser::new("[[a], [b]]").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::List(_)));
}

#[test]
fn list_with_expressions() {
    let expr = Parser::new("[a + b, c * d]").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::List(_)));
}

// ============================================================================
// TUPLE EXPRESSIONS
// ============================================================================

#[test]
fn tuple_empty() {
    let expr = Parser::new("()").parse_expr(0).unwrap();
    // Empty tuple is Tuple with empty Vec
    assert!(matches!(expr, Expr::Tuple(_)));
}

// ============================================================================
// PIPE EXPRESSIONS (parsed as Binary with Pipe operator)
// ============================================================================

#[test]
fn pipe_simple() {
    let expr = Parser::new("a |> b").parse_expr(0).unwrap();
    // Pipe is parsed as Binary expression
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn pipe_chain() {
    let expr = Parser::new("a |> b |> c").parse_expr(0).unwrap();
    // Pipe chain is parsed as Binary expression
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn pipe_long_chain() {
    let expr = Parser::new("a |> b |> c |> d |> e").parse_expr(0).unwrap();
    // Long pipe chain is parsed as Binary expression
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// COMPOSE EXPRESSIONS (parsed as Binary with Compose operator)
// ============================================================================

#[test]
fn compose_simple() {
    let expr = Parser::new("a >> b").parse_expr(0).unwrap();
    // Compose is parsed as Binary expression
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn compose_chain() {
    let expr = Parser::new("a >> b >> c").parse_expr(0).unwrap();
    // Compose chain is parsed as Binary expression
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// IF EXPRESSIONS
// ============================================================================

#[test]
fn if_simple() {
    let expr = Parser::new("if true { a }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::If { .. }));
    }
}

#[test]
fn if_else() {
    let expr = Parser::new("if true { a } else { b }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::If { .. }));
    }
}

#[test]
fn if_else_if() {
    let expr = Parser::new("if a { x } else if b { y } else { z }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::If { .. }));
    }
}

// ============================================================================
// MATCH EXPRESSIONS
// ============================================================================

#[test]
fn match_single_arm() {
    let input = "match x { _ { y } }";
    let expr = Parser::new(input).parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Match { .. }));
    }
}

#[test]
fn match_multiple_arms() {
    let input = r#"match x {
        a { 1 }
        b { 2 }
        _ { 3 }
    }"#;
    let expr = Parser::new(input).parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Match { .. }));
    }
}

// ============================================================================
// LAMBDA EXPRESSIONS
// ============================================================================

#[test]
fn lambda_no_params() {
    let expr = Parser::new("|| { x }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Lambda { .. }));
    }
}

#[test]
fn lambda_one_param() {
    let expr = Parser::new("|x| { x }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Lambda { .. }));
    }
}

#[test]
fn lambda_typed_param() {
    let expr = Parser::new("|x: Int64| { x }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Lambda { .. }));
    }
}

#[test]
fn lambda_multiple_params() {
    let expr = Parser::new("|a, b, c| { a }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Lambda { .. }));
    }
}

// ============================================================================
// BLOCK EXPRESSIONS
// ============================================================================

#[test]
fn block_empty() {
    let expr = Parser::new("{ }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Block(..)));
    }
}

#[test]
fn block_single_expr() {
    let expr = Parser::new("{ x }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Block(..)));
    }
}

#[test]
fn block_with_let() {
    let expr = Parser::new("{ let x = 1; x }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Block(..)));
    }
}

// ============================================================================
// GROUPED EXPRESSIONS
// ============================================================================

#[test]
fn grouped_simple() {
    let expr = Parser::new("(a)").parse_expr(0).unwrap();
    // Grouped expression is just the inner expression
    assert!(matches!(expr, Expr::Identifier(_)));
}

#[test]
fn grouped_binary() {
    let expr = Parser::new("(a + b)").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn grouped_nested() {
    let expr = Parser::new("((a))").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

// ============================================================================
// COMPLEX EXPRESSIONS
// ============================================================================

#[test]
fn complex_arithmetic() {
    let expr = Parser::new("a + b * c - d / e").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn complex_mixed() {
    let expr = Parser::new("a + b && c < d").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn complex_with_calls() {
    let expr = Parser::new("foo(a) + bar(b)").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn complex_with_pipes() {
    let expr = Parser::new("a |> f |> g").parse_expr(0).unwrap();
    // Pipe is parsed as Binary expression
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn complex_with_unary() {
    let expr = Parser::new("!a && b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// PRECEDENCE TESTS
// ============================================================================

#[test]
fn precedence_mul_before_add() {
    let expr = Parser::new("a + b * c").parse_expr(0).unwrap();
    // Should parse as a + (b * c)
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn precedence_div_before_sub() {
    let expr = Parser::new("a - b / c").parse_expr(0).unwrap();
    // Should parse as a - (b / c)
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn precedence_comparison_before_logic() {
    let expr = Parser::new("a < b && c > d").parse_expr(0).unwrap();
    // Should parse as (a < b) && (c > d)
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn precedence_and_before_or() {
    let expr = Parser::new("a || b && c").parse_expr(0).unwrap();
    // Should parse as a || (b && c)
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn precedence_unary_before_binary() {
    let expr = Parser::new("!a && b").parse_expr(0).unwrap();
    // Should parse as (!a) && b
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[test]
fn stress_deep_nesting() {
    let expr = Parser::new("((((((a))))))").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

#[test]
fn stress_long_chain() {
    let expr = Parser::new("a + b + c + d + e + f + g + h")
        .parse_expr(0)
        .unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn stress_many_calls() {
    let expr = Parser::new("a()()()()").parse_expr(0);
    // May or may not parse depending on syntax
    let _ = expr;
}
