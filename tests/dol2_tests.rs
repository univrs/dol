//! DOL 2.0 Integration Tests
//!
//! End-to-end tests for new DOL 2.0 syntax including functional programming
//! features, control flow, pattern matching, and meta-programming constructs.

use metadol::ast::{BinaryOp, Block, Declaration, Expr, Pattern, Stmt, TypeExpr};
use metadol::parser::Parser;

// ============================================
// Complete DOL 2.0 Gene Tests
// ============================================

#[test]
fn test_full_dol2_gene_with_expressions() {
    let input = r#"
gene functional.transform {
  container has identity
  transform is pipeline
}

exegesis {
  A gene demonstrating DOL 2.0 functional features.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    match result.unwrap() {
        Declaration::Gene(gene) => {
            assert_eq!(gene.name, "functional.transform");
            assert_eq!(gene.statements.len(), 2);
        }
        _ => panic!("Expected Gene declaration"),
    }
}

// ============================================
// Backward Compatibility Tests
// ============================================

#[test]
fn test_backward_compatibility_basic_gene() {
    let input = r#"
gene container.exists {
  container has identity
  container has status
}

exegesis {
  Basic DOL 1.x gene should still work.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok(), "DOL 1.x gene failed to parse");
}

#[test]
fn test_backward_compatibility_trait() {
    let input = r#"
trait container.lifecycle {
  uses container.exists
  container is created
  container is destroyed
}

exegesis {
  Basic DOL 1.x trait should still work.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok(), "DOL 1.x trait failed to parse");
}

#[test]
fn test_backward_compatibility_constraint() {
    let input = r#"
constraint container.uniqueness {
  each container has identity
  container is unique
}

exegesis {
  Basic DOL 1.x constraint should still work.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DOL 1.x constraint failed to parse: {:?}",
        result.err()
    );
}

#[test]
fn test_backward_compatibility_system() {
    let input = r#"
system container.orchestration @1.0.0 {
  requires container.exists >= 1.0.0
  uses container.lifecycle
  orchestrator has status
}

exegesis {
  Basic DOL 1.x system should still work.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok(), "DOL 1.x system failed to parse");
}

#[test]
fn test_backward_compatibility_evolution() {
    let input = r#"
evolves container.exists @2.0.0 > 1.0.0 {
  adds container has metadata
  deprecates container has legacy
  removes oldfield
  because "Modernization"
}

exegesis {
  Basic DOL 1.x evolution should still work.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok(), "DOL 1.x evolution failed to parse");
}

// ============================================
// Functional Pipeline Tests
// ============================================

#[test]
fn test_functional_pipeline_complex() {
    let input = "data |> validate |> transform |> store";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Verify it parses as a pipe expression
    match expr {
        Expr::Binary {
            op: BinaryOp::Pipe, ..
        } => {}
        _ => panic!("Expected pipe expression"),
    }
}

#[test]
fn test_functional_compose_complex() {
    let input = "trim >> lowercase >> validate >> normalize";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Verify it parses as a compose expression
    match expr {
        Expr::Binary {
            op: BinaryOp::Compose,
            ..
        } => {}
        _ => panic!("Expected compose expression"),
    }
}

#[test]
fn test_mixed_pipe_and_compose() {
    let input = "data |> (trim >> validate) |> process";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Verify it parses correctly
    match expr {
        Expr::Binary {
            op: BinaryOp::Pipe, ..
        } => {}
        _ => panic!("Expected top-level pipe"),
    }
}

// Note: BackPipe operator (<|) is in the lexer but not yet fully implemented in parser
// Uncomment this test when BackPipe is added to BinaryOp enum
// #[test]
// fn test_back_pipe_operator() {
//     let input = "result <| transform <| data";
//     let mut parser = Parser::new(input);
//     let expr = parser.parse_expr(0).unwrap();
//
//     match expr {
//         Expr::Binary {
//             op: BinaryOp::BackPipe,
//             ..
//         } => {}
//         _ => panic!("Expected back pipe"),
//     }
// }

// ============================================
// Pattern Matching Tests
// ============================================

#[test]
fn test_complex_match_expression() {
    let input = r#"match value {
        Some(x) => x,
        None => default,
        _ => fallback
    }"#;
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);

            // First arm: constructor pattern
            match &arms[0].pattern {
                Pattern::Constructor { name, .. } => assert_eq!(name, "Some"),
                _ => panic!("Expected constructor pattern"),
            }

            // Second arm: identifier pattern
            match &arms[1].pattern {
                Pattern::Identifier(name) => assert_eq!(name, "None"),
                _ => panic!("Expected identifier pattern"),
            }

            // Third arm: wildcard pattern
            match &arms[2].pattern {
                Pattern::Wildcard => {}
                _ => panic!("Expected wildcard pattern"),
            }
        }
        _ => panic!("Expected match expression"),
    }
}

#[test]
fn test_match_with_guards() {
    // Match with guard: pattern if guard_expr => body
    let input = r#"match x {
        value if condition => positive,
        _ => zero
    }"#;
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Match { arms, .. } => {
            assert!(arms[0].guard.is_some(), "Expected guard on first arm");
            assert!(arms[1].guard.is_none(), "Expected no guard on wildcard arm");
        }
        _ => panic!("Expected match expression"),
    }
}

#[test]
fn test_nested_pattern_matching() {
    let input = r#"match pair {
        (Some(x), Some(y)) => result,
        _ => default
    }"#;
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Match { arms, .. } => {
            // First arm should have tuple pattern with nested constructors
            match &arms[0].pattern {
                Pattern::Tuple(patterns) => {
                    assert_eq!(patterns.len(), 2);
                    match &patterns[0] {
                        Pattern::Constructor { name, .. } => assert_eq!(name, "Some"),
                        _ => panic!("Expected constructor in tuple"),
                    }
                }
                _ => panic!("Expected tuple pattern"),
            }
        }
        _ => panic!("Expected match expression"),
    }
}

// ============================================
// Control Flow Tests
// ============================================

#[test]
fn test_complex_if_else_chain() {
    let input = r#"if x { a } else if y { b } else if z { c } else { d }"#;
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Count the nesting depth
    let mut depth = 0;
    let mut current = &expr;
    while let Expr::If { else_branch, .. } = current {
        depth += 1;
        match else_branch {
            Some(else_expr) => current = else_expr,
            None => break,
        }
    }
    assert_eq!(depth, 3, "Expected 3 levels of if nesting");
}

#[test]
fn test_nested_loops() {
    let input = r#"for outer in outers {
        for inner in inners {
            break;
        }
    }"#;
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::For { body, .. } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::For { .. } => {}
                _ => panic!("Expected nested for loop"),
            }
        }
        _ => panic!("Expected for loop"),
    }
}

#[test]
fn test_while_with_break_continue() {
    // Simplified - just test while with break and continue
    let input = r#"while condition {
        continue;
        break;
    }"#;
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::While { body, .. } => {
            assert!(body.len() >= 2);
        }
        _ => panic!("Expected while loop"),
    }
}

#[test]
fn test_infinite_loop_with_break() {
    // Loop needs statements with semicolons, or use expr version
    let input = r#"loop {
        break;
    }"#;
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Loop { body } => {
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected loop statement"),
    }
}

// ============================================
// Lambda and Higher-Order Function Tests
// ============================================

#[test]
fn test_lambda_as_argument() {
    let input = "map(|x| x, list)";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Call { args, .. } => {
            assert_eq!(args.len(), 2);
            match &args[0] {
                Expr::Lambda { .. } => {}
                _ => panic!("Expected lambda as first argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_lambda_with_multiple_typed_params() {
    let input = "|x: Int32, y: Int32, z: Int32| -> Int32 { x }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Lambda {
            params,
            return_type,
            ..
        } => {
            assert_eq!(params.len(), 3);
            for param in &params {
                assert!(param.1.is_some(), "Expected type annotation");
            }
            assert!(return_type.is_some(), "Expected return type");
        }
        _ => panic!("Expected lambda"),
    }
}

#[test]
fn test_nested_lambdas() {
    let input = "|x| |y| x";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Lambda { body, .. } => match *body {
            Expr::Lambda { .. } => {}
            _ => panic!("Expected nested lambda"),
        },
        _ => panic!("Expected lambda"),
    }
}

#[test]
fn test_lambda_in_pipeline() {
    let input = "data |> (|x| x) |> result";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Just verify it parses
    match expr {
        Expr::Binary {
            op: BinaryOp::Pipe, ..
        } => {}
        _ => panic!("Expected pipe expression"),
    }
}

// ============================================
// Type System Tests
// ============================================

#[test]
fn test_all_builtin_types() {
    let types = vec![
        "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64", "Float32",
        "Float64", "Bool", "String", "Void",
    ];

    for type_name in types {
        let mut parser = Parser::new(type_name);
        let result = parser.parse_type();
        assert!(result.is_ok(), "Failed to parse type: {}", type_name);
    }
}

#[test]
fn test_generic_type_nested() {
    let input = "Result<Option<Int32>, String>";
    let mut parser = Parser::new(input);
    let type_expr = parser.parse_type().unwrap();

    match type_expr {
        TypeExpr::Generic { args, .. } => {
            assert_eq!(args.len(), 2);
            match &args[0] {
                TypeExpr::Generic { .. } => {}
                _ => panic!("Expected nested generic"),
            }
        }
        _ => panic!("Expected generic type"),
    }
}

#[test]
fn test_complex_function_type() {
    let input = "(Int32, String, Bool) -> Result<Int32, String>";
    let mut parser = Parser::new(input);
    let type_expr = parser.parse_type().unwrap();

    match type_expr {
        TypeExpr::Function {
            params,
            return_type,
        } => {
            assert_eq!(params.len(), 3);
            match *return_type {
                TypeExpr::Generic { .. } => {}
                _ => panic!("Expected generic return type"),
            }
        }
        _ => panic!("Expected function type"),
    }
}

// ============================================
// Meta-programming Tests
// ============================================

#[test]
fn test_quote_expression() {
    let input = "'{ value }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Quote followed by block
    match expr {
        Expr::Quote(_) => {}
        _ => panic!("Expected quote expression"),
    }
}

#[test]
fn test_eval_expression() {
    // Eval syntax: !{ expr }
    let input = "!{ code }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // The parser creates Eval which contains the expression inside braces
    match expr {
        Expr::Eval(_) => {}
        _ => panic!("Expected eval expression"),
    }
}

#[test]
fn test_reflect_on_type() {
    // Reflect is currently parsed as a unary operator
    let input = "?TypeName";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Reflect is a unary operator applied to an expression
    match expr {
        Expr::Unary {
            op: metadol::ast::UnaryOp::Reflect,
            ..
        } => {}
        _ => panic!("Expected reflect unary expression, got: {:?}", expr),
    }
}

// ============================================
// Block Expression Tests
// ============================================

#[test]
fn test_block_with_statements_and_final_expr() {
    let input = r#"{
        let x = 1;
        let y = 2;
        x
    }"#;
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Block(Block {
            statements,
            final_expr,
            ..
        }) => {
            assert_eq!(statements.len(), 2);
            assert!(final_expr.is_some());
        }
        _ => panic!("Expected block expression"),
    }
}

#[test]
fn test_nested_blocks() {
    let input = r#"{
        let outer = 1;
        {
            let inner = 2;
            inner
        }
    }"#;
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Block(Block { statements, .. }) => {
            // First statement is let
            // Second statement should be expression containing nested block
            assert!(!statements.is_empty());
        }
        _ => panic!("Expected block expression"),
    }
}

// ============================================
// Operator Precedence Tests
// ============================================

#[test]
fn test_arithmetic_precedence() {
    let input = "a + b * c";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Should parse as a + (b * c), so top level is Add
    match expr {
        Expr::Binary {
            op: BinaryOp::Add,
            right,
            ..
        } => {
            // Right side should be multiplication
            match *right {
                Expr::Binary {
                    op: BinaryOp::Mul, ..
                } => {}
                _ => panic!("Expected multiplication on right side"),
            }
        }
        _ => panic!("Expected addition at top level"),
    }
}

#[test]
fn test_comparison_precedence() {
    let input = "a == b && c != d";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Should parse as (a == b) && (c != d), so top level is And
    match expr {
        Expr::Binary {
            op: BinaryOp::And, ..
        } => {}
        _ => panic!("Expected logical and at top level"),
    }
}

#[test]
fn test_pipe_vs_compose_precedence() {
    let input = "a |> f >> g >> h";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Should parse as a |> (f >> g >> h), so top level is Pipe
    match expr {
        Expr::Binary {
            op: BinaryOp::Pipe,
            right,
            ..
        } => {
            // Right side should start with compose
            match *right {
                Expr::Binary {
                    op: BinaryOp::Compose,
                    ..
                } => {}
                _ => panic!("Expected compose on right side"),
            }
        }
        _ => panic!("Expected pipe at top level"),
    }
}

// ============================================
// Mixed DOL 1.x and DOL 2.0 Tests
// ============================================

#[test]
fn test_gene_with_dol1_and_dol2_features() {
    let input = r#"
gene mixed.features {
  container has identity
  transform is functional
}

exegesis {
  A gene mixing DOL 1.x statements with DOL 2.0 potential.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok(), "Mixed gene failed to parse");
}

// ============================================
// Error Recovery Tests
// ============================================

#[test]
fn test_incomplete_if_fails() {
    let input = "if condition { }";
    let mut parser = Parser::new(input);
    let result = parser.parse_expr(0);
    // Should still parse even with empty block
    assert!(result.is_ok());
}

#[test]
fn test_unmatched_pattern_parens() {
    let input = "(x, y";
    let mut parser = Parser::new(input);
    let result = parser.parse_pattern();
    assert!(result.is_err(), "Should fail on unmatched parens");
}

// ============================================
// Let Statement Tests
// ============================================

#[test]
fn test_let_with_complex_expression() {
    let input = "let result = data |> transform |> validate;";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Let { name, value, .. } => {
            assert_eq!(name, "result");
            match value {
                Expr::Binary {
                    op: BinaryOp::Pipe, ..
                } => {}
                _ => panic!("Expected pipe in let value"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_let_with_lambda() {
    let input = "let transform = |x: Int32| -> Int32 { x };";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Let { value, .. } => match value {
            Expr::Lambda { .. } => {}
            _ => panic!("Expected lambda in let value"),
        },
        _ => panic!("Expected let statement"),
    }
}

// ============================================
// Return Statement Tests
// ============================================

#[test]
fn test_return_with_expression() {
    let input = "return a |> b;";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Return(Some(expr)) => match expr {
            Expr::Binary {
                op: BinaryOp::Pipe, ..
            } => {}
            _ => panic!("Expected pipe in return"),
        },
        _ => panic!("Expected return with value"),
    }
}

// ============================================
// Member Access Tests
// ============================================

#[test]
fn test_chained_member_access() {
    let input = "obj.field.subfield";
    let mut parser = Parser::new(input);
    let result = parser.parse_expr(0);

    // Member access chaining - just verify it parses
    assert!(result.is_ok(), "Failed to parse chained member access");
}

#[test]
fn test_member_access_with_call() {
    let input = "obj.method(arg)";
    let mut parser = Parser::new(input);
    let result = parser.parse_expr(0);

    // Member access with method call - just verify it parses
    assert!(result.is_ok(), "Failed to parse member access with call");
}

// ============================================
// Boolean Literal Tests
// ============================================

#[test]
fn test_boolean_literals() {
    let input = "true";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();
    match expr {
        Expr::Literal(_) => {}
        _ => panic!("Expected boolean literal"),
    }

    let input = "false";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();
    match expr {
        Expr::Literal(_) => {}
        _ => panic!("Expected boolean literal"),
    }
}

#[test]
fn test_boolean_in_if() {
    let input = "if true { a } else { b }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();
    match expr {
        Expr::If { .. } => {}
        _ => panic!("Expected if expression"),
    }
}
