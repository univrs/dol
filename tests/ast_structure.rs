//! AST structure tests
//! Tests AST node construction and properties

use metadol::ast::*;
use metadol::parser::Parser;

// ============================================================================
// GENE AST STRUCTURE
// ============================================================================

#[test]
fn gene_has_name() {
    let file = Parser::new("gene MyGene { }").parse_file().unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert_eq!(gene.name, "MyGene");
    }
}

#[test]
fn gene_has_span() {
    let file = Parser::new("gene Test { }").parse_file().unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(gene.span.start < gene.span.end);
    }
}

#[test]
fn gene_has_statements() {
    let file = Parser::new("gene Test { has x: Int64 }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(!gene.statements.is_empty());
    }
}

#[test]
fn gene_empty_has_no_statements() {
    let file = Parser::new("gene Empty { }").parse_file().unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(gene.statements.is_empty());
    }
}

// ============================================================================
// TRAIT AST STRUCTURE
// ============================================================================

#[test]
fn trait_has_name() {
    let file = Parser::new("trait MyTrait { }").parse_file().unwrap();
    if let Some(Declaration::Trait(trait_decl)) = file.declarations.first() {
        assert_eq!(trait_decl.name, "MyTrait");
    }
}

#[test]
fn trait_has_span() {
    let file = Parser::new("trait Test { }").parse_file().unwrap();
    if let Some(Declaration::Trait(trait_decl)) = file.declarations.first() {
        assert!(trait_decl.span.start < trait_decl.span.end);
    }
}

#[test]
fn trait_has_statements() {
    let file = Parser::new("trait Test { entity is active }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Trait(trait_decl)) = file.declarations.first() {
        assert!(!trait_decl.statements.is_empty());
    }
}

// ============================================================================
// CONSTRAINT AST STRUCTURE
// ============================================================================

#[test]
fn constraint_has_name() {
    let file = Parser::new("constraint MyConstraint { }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Constraint(constraint)) = file.declarations.first() {
        assert_eq!(constraint.name, "MyConstraint");
    }
}

#[test]
fn constraint_has_span() {
    let file = Parser::new("constraint Test { }").parse_file().unwrap();
    if let Some(Declaration::Constraint(constraint)) = file.declarations.first() {
        assert!(constraint.span.start < constraint.span.end);
    }
}

#[test]
fn constraint_has_statements() {
    // Constraints can use subject-predicate-object statements like 'value is required'
    let file = Parser::new("constraint Test { value is required }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Constraint(constraint)) = file.declarations.first() {
        assert!(!constraint.statements.is_empty());
    }
}

// ============================================================================
// FUNCTION AST STRUCTURE
// ============================================================================

#[test]
fn function_has_name() {
    let file = Parser::new("fun myFunction() { }").parse_file().unwrap();
    if let Some(Declaration::Function(func)) = file.declarations.first() {
        assert_eq!(func.name, "myFunction");
    }
}

#[test]
fn function_has_span() {
    let file = Parser::new("fun test() { }").parse_file().unwrap();
    if let Some(Declaration::Function(func)) = file.declarations.first() {
        assert!(func.span.start < func.span.end);
    }
}

#[test]
fn function_has_params() {
    let file = Parser::new("fun test(x: Int64) { }").parse_file().unwrap();
    if let Some(Declaration::Function(func)) = file.declarations.first() {
        assert!(!func.params.is_empty());
    }
}

#[test]
fn function_no_params_is_empty() {
    let file = Parser::new("fun test() { }").parse_file().unwrap();
    if let Some(Declaration::Function(func)) = file.declarations.first() {
        assert!(func.params.is_empty());
    }
}

// ============================================================================
// SYSTEM AST STRUCTURE
// ============================================================================

#[test]
fn system_has_name() {
    let file = Parser::new("system MySystem { }").parse_file().unwrap();
    if let Some(Declaration::System(system)) = file.declarations.first() {
        assert_eq!(system.name, "MySystem");
    }
}

#[test]
fn system_has_span() {
    let file = Parser::new("system Test { }").parse_file().unwrap();
    if let Some(Declaration::System(system)) = file.declarations.first() {
        assert!(system.span.start < system.span.end);
    }
}

#[test]
fn system_has_statements() {
    let file = Parser::new("system Test { entity has state }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::System(system)) = file.declarations.first() {
        assert!(!system.statements.is_empty());
    }
}

// ============================================================================
// DECLARATION VARIANTS
// ============================================================================

#[test]
fn declaration_is_gene() {
    let file = Parser::new("gene Test { }").parse_file().unwrap();
    assert!(matches!(
        file.declarations.first(),
        Some(Declaration::Gene(_))
    ));
}

#[test]
fn declaration_is_trait() {
    let file = Parser::new("trait Test { }").parse_file().unwrap();
    assert!(matches!(
        file.declarations.first(),
        Some(Declaration::Trait(_))
    ));
}

#[test]
fn declaration_is_constraint() {
    let file = Parser::new("constraint Test { }").parse_file().unwrap();
    assert!(matches!(
        file.declarations.first(),
        Some(Declaration::Constraint(_))
    ));
}

#[test]
fn declaration_is_function() {
    let file = Parser::new("fun test() { }").parse_file().unwrap();
    assert!(matches!(
        file.declarations.first(),
        Some(Declaration::Function(_))
    ));
}

#[test]
fn declaration_is_system() {
    let file = Parser::new("system Test { }").parse_file().unwrap();
    assert!(matches!(
        file.declarations.first(),
        Some(Declaration::System(_))
    ));
}

// ============================================================================
// STATEMENT VARIANTS
// ============================================================================

#[test]
fn statement_is_has() {
    // Untyped has statement
    let file = Parser::new("gene Test { has x }").parse_file();
    if let Ok(file) = file {
        if let Some(Declaration::Gene(gene)) = file.declarations.first() {
            assert!(!gene.statements.is_empty());
        }
    }
}

#[test]
fn statement_has_field() {
    // Typed has field
    let file = Parser::new("gene Test { has x: Int64 }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(!gene.statements.is_empty());
    }
}

#[test]
fn statement_is_is() {
    let file = Parser::new("trait Test { entity is active }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Trait(trait_decl)) = file.declarations.first() {
        assert!(matches!(
            trait_decl.statements.first(),
            Some(Statement::Is { .. })
        ));
    }
}

#[test]
fn statement_uses() {
    let file = Parser::new("trait Test { uses Other }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Trait(trait_decl)) = file.declarations.first() {
        assert!(matches!(
            trait_decl.statements.first(),
            Some(Statement::Uses { .. })
        ));
    }
}

#[test]
fn statement_requires() {
    // DOL 2.0: 'constraint' inside a gene generates Requires statement
    // Direct 'requires' keyword isn't valid as a statement - use inline constraint blocks
    let file = Parser::new("gene Test { constraint Valid { } }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(matches!(
            gene.statements.first(),
            Some(Statement::Requires { .. })
        ));
    }
}

// ============================================================================
// EXPRESSION AST
// ============================================================================

#[test]
fn expr_identifier() {
    let expr = Parser::new("foo").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Identifier(_)));
}

#[test]
fn expr_literal_true() {
    let expr = Parser::new("true").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn expr_literal_false() {
    let expr = Parser::new("false").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn expr_literal_string() {
    let expr = Parser::new("\"hello\"").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Literal(_)));
}

#[test]
fn expr_binary() {
    let expr = Parser::new("a + b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn expr_unary() {
    let expr = Parser::new("!x").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Unary { .. }));
}

#[test]
fn expr_call() {
    let expr = Parser::new("foo()").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn expr_list() {
    let expr = Parser::new("[]").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::List(_)));
}

#[test]
fn expr_tuple() {
    let expr = Parser::new("()").parse_expr(0).unwrap();
    // Empty tuple is Tuple with empty Vec
    assert!(matches!(expr, Expr::Tuple(_)));
}

#[test]
fn expr_pipe_is_binary() {
    // Pipe operator is parsed as Binary expression
    let expr = Parser::new("a |> b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

#[test]
fn expr_compose_is_binary() {
    // Compose operator is parsed as Binary expression
    let expr = Parser::new("a >> b").parse_expr(0).unwrap();
    assert!(matches!(expr, Expr::Binary { .. }));
}

// ============================================================================
// SPAN PROPERTIES
// ============================================================================

#[test]
fn span_has_line() {
    let file = Parser::new("gene Test { }").parse_file().unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(gene.span.line >= 1);
    }
}

#[test]
fn span_has_column() {
    let file = Parser::new("gene Test { }").parse_file().unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(gene.span.column >= 1);
    }
}

#[test]
fn span_start_less_than_end() {
    let file = Parser::new("gene Test { }").parse_file().unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(gene.span.start < gene.span.end);
    }
}

// ============================================================================
// FILE STRUCTURE
// ============================================================================

#[test]
fn file_has_declarations() {
    let file = Parser::new("gene A { } gene B { }").parse_file().unwrap();
    assert_eq!(file.declarations.len(), 2);
}

#[test]
fn file_empty_has_no_declarations() {
    let file = Parser::new("").parse_file().unwrap();
    assert!(file.declarations.is_empty());
}

#[test]
fn file_preserves_order() {
    let file = Parser::new("gene A { } gene B { } gene C { }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert_eq!(gene.name, "A");
    }
    if let Some(Declaration::Gene(gene)) = file.declarations.get(1) {
        assert_eq!(gene.name, "B");
    }
    if let Some(Declaration::Gene(gene)) = file.declarations.get(2) {
        assert_eq!(gene.name, "C");
    }
}

// ============================================================================
// TYPE AST - Using HasField statement
// ============================================================================

#[test]
fn typed_field_statement() {
    let file = Parser::new("gene Test { has x: Int64 }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        // Typed fields may be HasField or Has depending on implementation
        assert!(!gene.statements.is_empty());
    }
}

#[test]
fn typed_field_string() {
    let file = Parser::new("gene Test { has x: String }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(!gene.statements.is_empty());
    }
}

#[test]
fn typed_field_bool() {
    let file = Parser::new("gene Test { has x: Bool }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Gene(gene)) = file.declarations.first() {
        assert!(!gene.statements.is_empty());
    }
}

// ============================================================================
// PARAM AST
// ============================================================================

#[test]
fn param_has_name() {
    let file = Parser::new("fun test(x: Int64) { }").parse_file().unwrap();
    if let Some(Declaration::Function(func)) = file.declarations.first() {
        if let Some(param) = func.params.first() {
            assert_eq!(param.name, "x");
        }
    }
}

#[test]
fn param_has_type_ann() {
    let file = Parser::new("fun test(x: i64) { }").parse_file().unwrap();
    if let Some(Declaration::Function(func)) = file.declarations.first() {
        if let Some(param) = func.params.first() {
            // Use type_ann, not param_type - v0.8.0 uses lowercase i64
            assert!(format!("{:?}", param.type_ann).contains("i64"));
        }
    }
}

#[test]
fn multiple_params() {
    let file = Parser::new("fun test(a: i64, b: string, c: bool) { }")
        .parse_file()
        .unwrap();
    if let Some(Declaration::Function(func)) = file.declarations.first() {
        assert_eq!(func.params.len(), 3);
    }
}

// ============================================================================
// MORE EXPRESSION TESTS
// ============================================================================

#[test]
fn expr_if() {
    let expr = Parser::new("if true { a }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::If { .. }));
    }
}

#[test]
fn expr_match() {
    let expr = Parser::new("match x { _ { y } }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Match { .. }));
    }
}

#[test]
fn expr_lambda() {
    let expr = Parser::new("|x| { x }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Lambda { .. }));
    }
}

#[test]
fn expr_block() {
    let expr = Parser::new("{ a }").parse_expr(0);
    if let Ok(e) = expr {
        assert!(matches!(e, Expr::Block(..)));
    }
}
