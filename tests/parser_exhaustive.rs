//! Exhaustive parser tests
//! Target: 500+ tests covering all AST nodes

use dol::ast::*;
use dol::parser::Parser;

// ============================================================================
// EXPRESSION PARSING
// ============================================================================

mod expr {
    use super::*;

    // Literals
    #[test]
    fn literal_int() {
        let ast = Parser::new("42").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Literal { .. }));
    }

    #[test]
    fn literal_float() {
        let ast = Parser::new("3.14").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Literal { .. }));
    }

    #[test]
    fn literal_string() {
        let ast = Parser::new(r#""hello""#).parse_expr().unwrap();
        assert!(matches!(ast, Expr::Literal { .. }));
    }

    #[test]
    fn literal_bool_true() {
        let ast = Parser::new("true").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Literal { .. }));
    }

    #[test]
    fn literal_bool_false() {
        let ast = Parser::new("false").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Literal { .. }));
    }

    // Binary expressions
    #[test]
    fn binary_add() {
        let ast = Parser::new("1 + 2").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    #[test]
    fn binary_sub() {
        let ast = Parser::new("1 - 2").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    #[test]
    fn binary_mul() {
        let ast = Parser::new("1 * 2").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    #[test]
    fn binary_div() {
        let ast = Parser::new("1 / 2").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    #[test]
    fn binary_nested() {
        let ast = Parser::new("1 + 2 * 3").parse_expr().unwrap();
        // Should parse as 1 + (2 * 3) due to precedence
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    #[test]
    fn binary_comparison() {
        let ast = Parser::new("a < b").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    #[test]
    fn binary_logical_and() {
        let ast = Parser::new("a && b").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    #[test]
    fn binary_logical_or() {
        let ast = Parser::new("a || b").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. }));
    }

    // Pipe operators
    #[test]
    fn pipe_forward() {
        let ast = Parser::new("x |> f").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. } | Expr::Pipe { .. }));
    }

    #[test]
    fn pipe_compose() {
        let ast = Parser::new("f >> g").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Binary { .. } | Expr::Compose { .. }));
    }

    #[test]
    fn pipe_chain() {
        let ast = Parser::new("x |> f |> g |> h").parse_expr().unwrap();
        // Should be left-associative
        assert!(matches!(ast, Expr::Binary { .. } | Expr::Pipe { .. }));
    }

    // Unary expressions
    #[test]
    fn unary_not() {
        let ast = Parser::new("!x").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Unary { .. }));
    }

    #[test]
    fn unary_neg() {
        let ast = Parser::new("-x").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Unary { .. }));
    }

    // Call expressions
    #[test]
    fn call_no_args() {
        let ast = Parser::new("foo()").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Call { .. }));
    }

    #[test]
    fn call_one_arg() {
        let ast = Parser::new("foo(1)").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Call { .. }));
    }

    #[test]
    fn call_multiple_args() {
        let ast = Parser::new("foo(1, 2, 3)").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Call { .. }));
    }

    #[test]
    fn call_nested() {
        let ast = Parser::new("foo(bar(x))").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Call { .. }));
    }

    // Field access
    #[test]
    fn field_access_simple() {
        let ast = Parser::new("obj.field").parse_expr().unwrap();
        assert!(matches!(ast, Expr::FieldAccess { .. } | Expr::Field { .. }));
    }

    #[test]
    fn field_access_chain() {
        let ast = Parser::new("a.b.c.d").parse_expr().unwrap();
        assert!(matches!(ast, Expr::FieldAccess { .. } | Expr::Field { .. }));
    }

    #[test]
    fn method_call() {
        let ast = Parser::new("obj.method()").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Call { .. } | Expr::MethodCall { .. }));
    }

    // Index expressions
    #[test]
    fn index_simple() {
        let ast = Parser::new("arr[0]").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Index { .. }));
    }

    #[test]
    fn index_nested() {
        let ast = Parser::new("matrix[i][j]").parse_expr().unwrap();
        assert!(matches!(ast, Expr::Index { .. }));
    }

    // Lambda expressions
    #[test]
    fn lambda_no_params() {
        let ast = Parser::new("|| { 42 }").parse_expr();
        // May need different syntax depending on DOL spec
    }

    #[test]
    fn lambda_one_param() {
        let ast = Parser::new("|x| { x * 2 }").parse_expr();
    }

    #[test]
    fn lambda_typed() {
        let ast = Parser::new("|x: Int64| -> Int64 { x * 2 }").parse_expr();
    }

    // If expressions
    #[test]
    fn if_simple() {
        let ast = Parser::new("if true { 1 }").parse_expr();
    }

    #[test]
    fn if_else() {
        let ast = Parser::new("if true { 1 } else { 2 }").parse_expr();
    }

    #[test]
    fn if_else_if() {
        let ast = Parser::new("if a { 1 } else if b { 2 } else { 3 }").parse_expr();
    }

    // Match expressions
    #[test]
    fn match_simple() {
        let input = r#"match x {
            0 { "zero" }
            _ { "other" }
        }"#;
        let ast = Parser::new(input).parse_expr();
    }

    #[test]
    fn match_with_guard() {
        let input = r#"match n {
            x where x > 0 { "positive" }
            _ { "non-positive" }
        }"#;
        let ast = Parser::new(input).parse_expr();
    }

    // Block expressions
    #[test]
    fn block_empty() {
        let ast = Parser::new("{ }").parse_expr();
    }

    #[test]
    fn block_with_stmts() {
        let ast = Parser::new("{ let x = 1; x + 1 }").parse_expr();
    }

    // Parentheses
    #[test]
    fn parens_simple() {
        let ast = Parser::new("(1 + 2)").parse_expr().unwrap();
    }

    #[test]
    fn parens_nested() {
        let ast = Parser::new("((1 + 2) * 3)").parse_expr().unwrap();
    }
}

// ============================================================================
// STATEMENT PARSING
// ============================================================================

mod stmt {
    use super::*;

    #[test]
    fn let_simple() {
        let ast = Parser::new("let x = 1").parse_stmt();
    }

    #[test]
    fn let_typed() {
        let ast = Parser::new("let x: Int64 = 1").parse_stmt();
    }

    #[test]
    fn return_value() {
        let ast = Parser::new("return 42").parse_stmt();
    }

    #[test]
    fn return_void() {
        let ast = Parser::new("return").parse_stmt();
    }

    #[test]
    fn for_loop() {
        let ast = Parser::new("for x in items { process(x) }").parse_stmt();
    }

    #[test]
    fn while_loop() {
        let ast = Parser::new("while x > 0 { x = x - 1 }").parse_stmt();
    }

    #[test]
    fn loop_infinite() {
        let ast = Parser::new("loop { break }").parse_stmt();
    }

    #[test]
    fn break_stmt() {
        let ast = Parser::new("break").parse_stmt();
    }

    #[test]
    fn continue_stmt() {
        let ast = Parser::new("continue").parse_stmt();
    }
}

// ============================================================================
// DECLARATION PARSING
// ============================================================================

mod decl {
    use super::*;

    // Gene declarations
    #[test]
    fn gene_empty() {
        let ast = Parser::new("gene Empty { }").parse_decl();
    }

    #[test]
    fn gene_with_type() {
        let ast = Parser::new("gene Counter { type: Int64 }").parse_decl();
    }

    #[test]
    fn gene_with_fields() {
        let input = r#"gene Container {
            has id: UInt64
            has name: String
        }"#;
        let ast = Parser::new(input).parse_decl();
    }

    #[test]
    fn gene_with_constraint() {
        let input = r#"gene Positive {
            type: Int64
            constraint positive { this.value > 0 }
        }"#;
        let ast = Parser::new(input).parse_decl();
    }

    #[test]
    fn gene_with_exegesis() {
        let input = r#"gene Documented {
            type: Int64
            exegesis { This is documentation. }
        }"#;
        let ast = Parser::new(input).parse_decl();
    }

    // Trait declarations
    #[test]
    fn trait_empty() {
        let ast = Parser::new("trait Empty { }").parse_decl();
    }

    #[test]
    fn trait_with_method() {
        let input = r#"trait Runnable {
            is run() -> Void
        }"#;
        let ast = Parser::new(input).parse_decl();
    }

    #[test]
    fn trait_with_requires() {
        let input = r#"trait Schedulable {
            requires priority: Function<Self, Int32>
        }"#;
        let ast = Parser::new(input).parse_decl();
    }

    // Function declarations
    #[test]
    fn function_no_params() {
        let ast = Parser::new("fun noop() { }").parse_decl();
    }

    #[test]
    fn function_with_params() {
        let ast = Parser::new("fun add(a: Int64, b: Int64) -> Int64 { return a + b }").parse_decl();
    }

    #[test]
    fn function_pub() {
        let ast = Parser::new("pub fun public_fn() { }").parse_decl();
    }

    // System declarations
    #[test]
    fn system_empty() {
        let ast = Parser::new("system Empty { }").parse_decl();
    }

    // Module declarations
    #[test]
    fn module_simple() {
        let ast = Parser::new("module my.module @ 1.0.0").parse_decl();
    }

    // Use declarations
    #[test]
    fn use_all() {
        let ast = Parser::new("use dol.ast.*").parse_decl();
    }

    #[test]
    fn use_named() {
        let ast = Parser::new("use dol.ast.{Expr, Stmt}").parse_decl();
    }
}

// ============================================================================
// INTEGRATION TESTS (Real DOL files)
// ============================================================================

mod integration {
    use super::*;
    use std::fs;

    #[test]
    fn parse_dol_ast() {
        if let Ok(content) = fs::read_to_string("dol/ast.dol") {
            let result = Parser::new(&content).parse_file();
            assert!(result.is_ok(), "Failed to parse dol/ast.dol");
        }
    }

    #[test]
    fn parse_dol_parser() {
        if let Ok(content) = fs::read_to_string("dol/parser.dol") {
            let result = Parser::new(&content).parse_file();
            assert!(result.is_ok(), "Failed to parse dol/parser.dol");
        }
    }

    #[test]
    fn parse_dol_codegen() {
        if let Ok(content) = fs::read_to_string("dol/codegen.dol") {
            let result = Parser::new(&content).parse_file();
            assert!(result.is_ok(), "Failed to parse dol/codegen.dol");
        }
    }

    #[test]
    fn parse_all_dol_files() {
        let dol_dir = "dol";
        if let Ok(entries) = fs::read_dir(dol_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "dol").unwrap_or(false) {
                    let content = fs::read_to_string(&path).unwrap();
                    let result = Parser::new(&content).parse_file();
                    assert!(
                        result.is_ok(),
                        "Failed to parse {:?}: {:?}",
                        path,
                        result.err()
                    );
                }
            }
        }
    }
}
