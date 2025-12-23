//! Comprehensive parser tests for Metal DOL.
//!
//! These tests verify correct parsing of all DOL language constructs.

use metadol::ast::{Declaration, Quantifier, Statement};
use metadol::error::ParseError;
use metadol::parser::Parser;

/// Helper to parse a string and return the declaration
fn parse(input: &str) -> Result<Declaration, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

// ============================================
// 1. Gene Declaration Tests
// ============================================

#[test]
fn test_parse_simple_gene() {
    let input = r#"
gene container.exists {
  container has identity
}

exegesis {
  A container is fundamental.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.name, "container.exists");
        assert_eq!(gene.statements.len(), 1);
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_parse_gene_multiple_statements() {
    let input = r#"
gene container.exists {
  container has identity
  container has state
  container has boundaries
  container has resources
}

exegesis {
  Container properties.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.statements.len(), 4);
    } else {
        panic!("Expected Gene");
    }
}

#[test]
fn test_parse_gene_is_statement() {
    let input = r#"
gene container.states {
  container is immutable
}

exegesis {
  Container state property.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Gene(gene) = result.unwrap() {
        match &gene.statements[0] {
            Statement::Is { subject, state, .. } => {
                assert_eq!(subject, "container");
                assert_eq!(state, "immutable");
            }
            _ => panic!("Expected Is statement"),
        }
    } else {
        panic!("Expected Gene");
    }
}

#[test]
fn test_parse_gene_derives_from() {
    let input = r#"
gene identity.cryptographic {
  identity derives from ed25519 keypair
}

exegesis {
  Cryptographic identity derivation.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Gene(gene) = result.unwrap() {
        match &gene.statements[0] {
            Statement::DerivesFrom {
                subject, origin, ..
            } => {
                assert_eq!(subject, "identity");
                assert_eq!(origin, "ed25519 keypair");
            }
            _ => panic!("Expected DerivesFrom statement"),
        }
    } else {
        panic!("Expected Gene");
    }
}

#[test]
fn test_parse_gene_requires() {
    let input = r#"
gene identity.authority {
  identity requires no external authority
}

exegesis {
  Self-sovereign identity.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        match &gene.statements[0] {
            Statement::Requires {
                subject,
                requirement,
                ..
            } => {
                assert_eq!(subject, "identity");
                assert!(requirement.contains("external"));
            }
            _ => panic!("Expected Requires statement"),
        }
    } else {
        panic!("Expected Gene");
    }
}

// ============================================
// 2. Trait Declaration Tests
// ============================================

#[test]
fn test_parse_simple_trait() {
    let input = r#"
trait container.lifecycle {
  uses container.exists
  container is created
}

exegesis {
  Lifecycle management.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Trait(trait_decl) = result.unwrap() {
        assert_eq!(trait_decl.name, "container.lifecycle");
        assert_eq!(trait_decl.statements.len(), 2);
    } else {
        panic!("Expected Trait");
    }
}

#[test]
fn test_parse_trait_multiple_uses() {
    let input = r#"
trait container.networking {
  uses container.exists
  uses network.core
  uses identity.cryptographic
}

exegesis {
  Container networking composition.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Trait(trait_decl) = result.unwrap() {
        let uses_count = trait_decl
            .statements
            .iter()
            .filter(|s| matches!(s, Statement::Uses { .. }))
            .count();
        assert_eq!(uses_count, 3);
    } else {
        panic!("Expected Trait");
    }
}

#[test]
fn test_parse_trait_with_quantified() {
    let input = r#"
trait container.lifecycle {
  uses container.exists
  container is started
  each transition emits event
}

exegesis {
  Lifecycle with events.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Trait(trait_decl) = result.unwrap() {
        let has_quantified = trait_decl
            .statements
            .iter()
            .any(|s| matches!(s, Statement::Quantified { .. }));
        assert!(has_quantified);
    } else {
        panic!("Expected Trait");
    }
}

#[test]
fn test_parse_trait_emits() {
    let input = r#"
trait container.events {
  uses container.exists
  transition emits event
}

exegesis {
  Event emission.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Trait(trait_decl) = result.unwrap() {
        let has_emits = trait_decl
            .statements
            .iter()
            .any(|s| matches!(s, Statement::Emits { .. }));
        assert!(has_emits);
    } else {
        panic!("Expected Trait");
    }
}

// ============================================
// 3. Constraint Declaration Tests
// ============================================

#[test]
fn test_parse_simple_constraint() {
    let input = r#"
constraint container.integrity {
  state matches declared
}

exegesis {
  Container integrity rules.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Constraint(constraint) = result.unwrap() {
        assert_eq!(constraint.name, "container.integrity");
    } else {
        panic!("Expected Constraint");
    }
}

#[test]
fn test_parse_constraint_never() {
    let input = r#"
constraint identity.immutable {
  identity never changes
}

exegesis {
  Identity immutability constraint.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Constraint(constraint) = result.unwrap() {
        match &constraint.statements[0] {
            Statement::Never {
                subject, action, ..
            } => {
                assert_eq!(subject, "identity");
                assert_eq!(action, "changes");
            }
            _ => panic!("Expected Never statement"),
        }
    } else {
        panic!("Expected Constraint");
    }
}

#[test]
fn test_parse_constraint_matches() {
    let input = r#"
constraint state.consistency {
  runtime matches declared state
}

exegesis {
  State consistency constraint.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Constraint(constraint) = result.unwrap() {
        match &constraint.statements[0] {
            Statement::Matches {
                subject, target, ..
            } => {
                assert_eq!(subject, "runtime");
                assert!(target.contains("declared"));
            }
            _ => panic!("Expected Matches statement"),
        }
    } else {
        panic!("Expected Constraint");
    }
}

// ============================================
// 4. System Declaration Tests
// ============================================

#[test]
fn test_parse_simple_system() {
    let input = r#"
system univrs.orchestrator @ 0.1.0 {
  requires container.lifecycle >= 0.0.2
}

exegesis {
  The main orchestrator system.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::System(system) = result.unwrap() {
        assert_eq!(system.name, "univrs.orchestrator");
        assert_eq!(system.version, "0.1.0");
        assert_eq!(system.requirements.len(), 1);
    } else {
        panic!("Expected System");
    }
}

#[test]
fn test_parse_system_multiple_requirements() {
    let input = r#"
system univrs.scheduler @ 0.2.0 {
  requires container.lifecycle >= 0.0.2
  requires node.discovery >= 0.0.1
  requires cluster.membership >= 0.1.0
}

exegesis {
  Scheduler with multiple dependencies.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::System(system) = result.unwrap() {
        assert_eq!(system.requirements.len(), 3);
        assert_eq!(system.requirements[0].name, "container.lifecycle");
        assert_eq!(system.requirements[0].constraint, ">=");
        assert_eq!(system.requirements[0].version, "0.0.2");
    } else {
        panic!("Expected System");
    }
}

#[test]
fn test_parse_system_with_statements() {
    let input = r#"
system univrs.api @ 1.0.0 {
  requires container.lifecycle >= 0.0.2
  all operations is authenticated
}

exegesis {
  API system with authentication.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::System(system) = result.unwrap() {
        assert!(!system.statements.is_empty());
    } else {
        panic!("Expected System");
    }
}

// ============================================
// 5. Evolution Declaration Tests
// ============================================

#[test]
fn test_parse_simple_evolution() {
    let input = r#"
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
}

exegesis {
  Adding pause state.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Evolution(evolution) = result.unwrap() {
        assert_eq!(evolution.name, "container.lifecycle");
        assert_eq!(evolution.version, "0.0.2");
        assert_eq!(evolution.parent_version, "0.0.1");
        assert_eq!(evolution.additions.len(), 1);
    } else {
        panic!("Expected Evolution");
    }
}

#[test]
fn test_parse_evolution_with_deprecates() {
    let input = r#"
evolves api.endpoints @ 2.0.0 > 1.0.0 {
  adds response has pagination
  deprecates response is unlimited
}

exegesis {
  API pagination update.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Evolution(evolution) = result.unwrap() {
        assert_eq!(evolution.additions.len(), 1);
        assert_eq!(evolution.deprecations.len(), 1);
    } else {
        panic!("Expected Evolution");
    }
}

#[test]
fn test_parse_evolution_with_removes() {
    let input = r#"
evolves api.legacy @ 3.0.0 > 2.0.0 {
  removes old.endpoint
}

exegesis {
  Removing legacy endpoint.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Evolution(evolution) = result.unwrap() {
        assert_eq!(evolution.removals.len(), 1);
        assert_eq!(evolution.removals[0], "old.endpoint");
    } else {
        panic!("Expected Evolution");
    }
}

#[test]
fn test_parse_evolution_with_because() {
    let input = r#"
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  because "migration requires state preservation"
}

exegesis {
  Pause for migration.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Evolution(evolution) = result.unwrap() {
        assert!(evolution.rationale.is_some());
        assert!(evolution.rationale.unwrap().contains("migration"));
    } else {
        panic!("Expected Evolution");
    }
}

// ============================================
// 6. Statement Type Tests
// ============================================

#[test]
fn test_has_statement_parsing() {
    let input = r#"
gene test.has {
  subject has property
}

exegesis {
  Has statement test.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_is_statement_parsing() {
    let input = r#"
gene test.is {
  subject is state
}

exegesis {
  Is statement test.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_quantified_each() {
    let input = r#"
trait test.each {
  uses container.exists
  each item emits event
}

exegesis {
  Each quantifier test.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Trait(trait_decl) = result.unwrap() {
        let quantified = trait_decl.statements.iter().find(|s| {
            matches!(
                s,
                Statement::Quantified {
                    quantifier: Quantifier::Each,
                    ..
                }
            )
        });
        assert!(quantified.is_some());
    }
}

#[test]
fn test_quantified_all() {
    let input = r#"
trait test.all {
  uses container.exists
  all operations is authenticated
}

exegesis {
  All quantifier test.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());
}

// ============================================
// 7. Error Case Tests
// ============================================

#[test]
fn test_error_missing_exegesis() {
    let input = r#"
gene container.exists {
  container has identity
}
"#;
    let result = parse(input);
    assert!(result.is_err());

    match result {
        Err(ParseError::MissingExegesis { .. }) => {}
        Err(e) => panic!("Expected MissingExegesis, got: {:?}", e),
        Ok(_) => panic!("Expected error"),
    }
}

#[test]
fn test_error_invalid_declaration() {
    let input = "invalid declaration";
    let result = parse(input);
    assert!(result.is_err());
}

#[test]
fn test_error_missing_brace() {
    let input = r#"
gene container.exists
  container has identity
}

exegesis {
  Missing opening brace.
}
"#;
    let result = parse(input);
    assert!(result.is_err());
}

#[test]
fn test_error_missing_name() {
    let input = r#"
gene {
  container has identity
}

exegesis {
  Missing name.
}
"#;
    let result = parse(input);
    assert!(result.is_err());
}

// ============================================
// 8. Edge Case Tests
// ============================================

#[test]
fn test_parse_empty_body() {
    let input = r#"
gene empty.body {
}

exegesis {
  Empty body is valid.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Gene(gene) = result.unwrap() {
        assert!(gene.statements.is_empty());
    }
}

#[test]
fn test_parse_with_comments() {
    let input = r#"
// This is a comment
gene container.exists {
  // Comment inside
  container has identity
}

exegesis {
  With comments.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiline_exegesis() {
    let input = r#"
gene container.exists {
  container has identity
}

exegesis {
  Line one.
  Line two.
  Line three with more details
  about the container.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_deeply_qualified_name() {
    let input = r#"
gene domain.subdomain.component.property {
  subject has property
}

exegesis {
  Deeply qualified identifier.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.name, "domain.subdomain.component.property");
    }
}

// ============================================
// 9. Version Constraint Tests
// ============================================

#[test]
fn test_parse_version_greater_equal() {
    let input = r#"
system test.system @ 1.0.0 {
  requires dep.one >= 0.1.0
}

exegesis {
  Greater equal constraint.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::System(system) = result.unwrap() {
        assert_eq!(system.requirements[0].constraint, ">=");
    }
}

#[test]
fn test_parse_version_greater() {
    let input = r#"
system test.system @ 1.0.0 {
  requires dep.one > 0.1.0
}

exegesis {
  Greater constraint.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::System(system) = result.unwrap() {
        assert_eq!(system.requirements[0].constraint, ">");
    }
}

#[test]
fn test_parse_version_equal() {
    let input = r#"
system test.system @ 1.0.0 {
  requires dep.one = 0.1.0
}

exegesis {
  Exact version constraint.
}
"#;
    let result = parse(input);
    assert!(result.is_ok());

    if let Declaration::System(system) = result.unwrap() {
        assert_eq!(system.requirements[0].constraint, "=");
    }
}

// ============================================
// 10. Declaration Name Tests
// ============================================

#[test]
fn test_declaration_name_method() {
    let input = r#"
gene test.name {
  subject has property
}

exegesis {
  Name method test.
}
"#;
    let result = parse(input).unwrap();
    assert_eq!(result.name(), "test.name");
}

#[test]
fn test_declaration_exegesis_method() {
    let input = r#"
gene test.exegesis {
  subject has property
}

exegesis {
  This is the exegesis text.
}
"#;
    let result = parse(input).unwrap();
    assert!(result.exegesis().contains("exegesis text"));
}

#[test]
fn test_collect_dependencies() {
    let input = r#"
trait test.deps {
  uses dep.one
  uses dep.two
  subject is state
}

exegesis {
  Dependency collection test.
}
"#;
    let result = parse(input).unwrap();
    let deps = result.collect_dependencies();
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&"dep.one".to_string()));
    assert!(deps.contains(&"dep.two".to_string()));
}

// ============================================
// DOL 2.0 Expression Parsing Tests
// ============================================

#[test]
fn test_parse_pipe_expression() {
    use metadol::ast::{BinaryOp, Expr};

    let mut parser = Parser::new("data |> transform |> validate");
    let expr = parser.parse_expr(0).unwrap();

    // Verify it's a pipe expression
    match expr {
        Expr::Binary {
            op: BinaryOp::Pipe, ..
        } => {}
        _ => panic!("Expected pipe expression"),
    }
}

#[test]
fn test_parse_compose_expression() {
    use metadol::ast::{BinaryOp, Expr};

    let mut parser = Parser::new("double >> increment >> square");
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Binary {
            op: BinaryOp::Compose,
            ..
        } => {}
        _ => panic!("Expected compose expression"),
    }
}

#[test]
fn test_parse_if_expression() {
    use metadol::ast::Expr;

    let input = "if x { result }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::If {
            condition,
            then_branch: _,
            else_branch,
        } => {
            match *condition {
                Expr::Identifier(ref name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier condition"),
            }
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected if expression"),
    }
}

#[test]
fn test_parse_if_else_expression() {
    use metadol::ast::Expr;

    let input = "if condition { positive } else { negative }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::If {
            condition,
            then_branch: _,
            else_branch,
        } => {
            match *condition {
                Expr::Identifier(ref name) => assert_eq!(name, "condition"),
                _ => panic!("Expected identifier condition"),
            }
            assert!(else_branch.is_some());
        }
        _ => panic!("Expected if-else expression"),
    }
}

#[test]
fn test_parse_if_else_if_chain() {
    use metadol::ast::Expr;

    let input = "if x { a } else if y { b } else { c }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::If { else_branch, .. } => {
            // Else branch should be another if expression
            match else_branch {
                Some(else_expr) => match *else_expr {
                    Expr::If { .. } => {}
                    _ => panic!("Expected nested if in else branch"),
                },
                None => panic!("Expected else branch"),
            }
        }
        _ => panic!("Expected if expression"),
    }
}

#[test]
fn test_parse_match_expression() {
    use metadol::ast::Expr;

    let input = "match value { x => result, _ => default }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Match { scrutinee, arms } => {
            match *scrutinee {
                Expr::Identifier(ref name) => assert_eq!(name, "value"),
                _ => panic!("Expected identifier scrutinee"),
            }
            assert_eq!(arms.len(), 2);
        }
        _ => panic!("Expected match expression"),
    }
}

#[test]
fn test_parse_match_with_guards() {
    use metadol::ast::Expr;

    let input = "match x { value if condition => result }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            assert!(arms[0].guard.is_some());
        }
        _ => panic!("Expected match expression"),
    }
}

#[test]
fn test_parse_lambda() {
    use metadol::ast::Expr;

    let input = "|x| x";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Lambda {
            params,
            return_type,
            body,
        } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].0, "x");
            assert!(return_type.is_none());
            match *body {
                Expr::Identifier(ref name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier body"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_parse_lambda_with_type() {
    use metadol::ast::{Expr, TypeExpr};

    let input = "|x: Int32| -> Int32 { x }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Lambda {
            params,
            return_type,
            ..
        } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].0, "x");
            match &params[0].1 {
                Some(TypeExpr::Named(name)) => assert_eq!(name, "Int32"),
                _ => panic!("Expected Int32 type annotation"),
            }
            match return_type {
                Some(TypeExpr::Named(name)) => assert_eq!(name, "Int32"),
                _ => panic!("Expected Int32 return type"),
            }
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_parse_lambda_multiple_params() {
    use metadol::ast::Expr;

    let input = "|x, y, z| x";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Lambda { params, .. } => {
            assert_eq!(params.len(), 3);
            assert_eq!(params[0].0, "x");
            assert_eq!(params[1].0, "y");
            assert_eq!(params[2].0, "z");
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_parse_for_loop() {
    use metadol::ast::Stmt;

    let input = "for item in items { break; }";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::For {
            binding,
            iterable: _,
            body,
        } => {
            assert_eq!(binding, "item");
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected for statement"),
    }
}

#[test]
fn test_parse_while_loop() {
    use metadol::ast::Stmt;

    let input = "while condition { continue; }";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::While { condition: _, body } => {
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_parse_loop() {
    use metadol::ast::Stmt;

    let input = "loop { break; }";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Loop { body } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::Break => {}
                _ => panic!("Expected break statement"),
            }
        }
        _ => panic!("Expected loop statement"),
    }
}

#[test]
fn test_parse_let_statement() {
    use metadol::ast::Stmt;

    let input = "let x = value;";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Let {
            name,
            type_ann,
            value: _,
        } => {
            assert_eq!(name, "x");
            assert!(type_ann.is_none());
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_let_with_type() {
    use metadol::ast::{Stmt, TypeExpr};

    let input = "let x: Int32 = value;";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Let {
            name,
            type_ann,
            value: _,
        } => {
            assert_eq!(name, "x");
            match type_ann {
                Some(TypeExpr::Named(t)) => assert_eq!(t, "Int32"),
                _ => panic!("Expected Int32 type annotation"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_return_statement() {
    use metadol::ast::Stmt;

    let input = "return value;";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Return(Some(_)) => {}
        _ => panic!("Expected return statement with value"),
    }
}

#[test]
fn test_parse_return_void() {
    use metadol::ast::Stmt;

    let input = "return;";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    match stmt {
        Stmt::Return(None) => {}
        _ => panic!("Expected return statement without value"),
    }
}

#[test]
fn test_parse_break_continue() {
    use metadol::ast::Stmt;

    let mut parser = Parser::new("break;");
    match parser.parse_stmt().unwrap() {
        Stmt::Break => {}
        _ => panic!("Expected break"),
    }

    let mut parser = Parser::new("continue;");
    match parser.parse_stmt().unwrap() {
        Stmt::Continue => {}
        _ => panic!("Expected continue"),
    }
}

#[test]
fn test_parse_block_expression() {
    use metadol::ast::Expr;

    let input = "{ let x = 1; x }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Block {
            statements,
            final_expr,
        } => {
            assert_eq!(statements.len(), 1);
            assert!(final_expr.is_some());
        }
        _ => panic!("Expected block expression"),
    }
}

#[test]
fn test_parse_operator_precedence() {
    use metadol::ast::{BinaryOp, Expr};

    // Test that |> has lower precedence than >>
    let input = "a |> f >> g";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Binary {
            left: _,
            op: BinaryOp::Pipe,
            right,
        } => {
            // Right side should be the compose expression
            match *right {
                Expr::Binary {
                    op: BinaryOp::Compose,
                    ..
                } => {}
                _ => panic!("Expected compose on right side of pipe"),
            }
        }
        _ => panic!("Expected pipe as top-level operator"),
    }
}

#[test]
fn test_parse_quote_expression() {
    use metadol::ast::Expr;

    let input = "'expr";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Quote(operand) => match *operand {
            Expr::Identifier(ref name) => assert_eq!(name, "expr"),
            _ => panic!("Expected identifier"),
        },
        _ => panic!("Expected quote expression"),
    }
}

#[test]
fn test_parse_eval_expression() {
    use metadol::ast::Expr;

    let input = "!{ code }";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Eval(_) => {}
        _ => panic!("Expected eval expression"),
    }
}

#[test]
fn test_parse_logical_not() {
    use metadol::ast::{Expr, UnaryOp};

    let input = "!value";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Unary {
            op: UnaryOp::Not,
            operand,
        } => match *operand {
            Expr::Identifier(ref name) => assert_eq!(name, "value"),
            _ => panic!("Expected identifier"),
        },
        _ => panic!("Expected not expression"),
    }
}

#[test]
fn test_parse_reflect_expression() {
    use metadol::ast::{Expr, UnaryOp};

    let input = "?TypeName";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Reflect is currently parsed as a unary operator
    match expr {
        Expr::Unary {
            op: UnaryOp::Reflect,
            ..
        } => {}
        _ => panic!("Expected reflect expression"),
    }
}

#[test]
fn test_parse_function_call() {
    use metadol::ast::Expr;

    let input = "func(a, b, c)";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::Call { callee, args } => {
            match *callee {
                Expr::Identifier(ref name) => assert_eq!(name, "func"),
                _ => panic!("Expected identifier callee"),
            }
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_member_access() {
    let input = "obj.field";
    let mut parser = Parser::new(input);
    let result = parser.parse_expr(0);

    // Member access with dot - just verify it parses
    assert!(result.is_ok(), "Failed to parse member access");
}

#[test]
fn test_parse_arithmetic_operators() {
    use metadol::ast::{BinaryOp, Expr};

    let operators = vec![
        ("+", BinaryOp::Add),
        ("-", BinaryOp::Sub),
        ("*", BinaryOp::Mul),
        ("/", BinaryOp::Div),
        ("%", BinaryOp::Mod),
        ("^", BinaryOp::Pow),
    ];

    for (op_str, expected_op) in operators {
        let input = format!("a {} b", op_str);
        let mut parser = Parser::new(&input);
        let expr = parser.parse_expr(0).unwrap();

        match expr {
            Expr::Binary { op, .. } => assert_eq!(op, expected_op),
            _ => panic!("Expected binary expression for {}", op_str),
        }
    }
}

#[test]
fn test_parse_comparison_operators() {
    use metadol::ast::{BinaryOp, Expr};

    let operators = vec![
        ("==", BinaryOp::Eq),
        ("!=", BinaryOp::Ne),
        ("<", BinaryOp::Lt),
        ("<=", BinaryOp::Le),
        (">", BinaryOp::Gt),
        (">=", BinaryOp::Ge),
    ];

    for (op_str, expected_op) in operators {
        let input = format!("a {} b", op_str);
        let mut parser = Parser::new(&input);
        let expr = parser.parse_expr(0).unwrap();

        match expr {
            Expr::Binary { op, .. } => assert_eq!(op, expected_op),
            _ => panic!("Expected binary expression for {}", op_str),
        }
    }
}

#[test]
fn test_parse_logical_operators() {
    use metadol::ast::{BinaryOp, Expr};

    let input = "a && b";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();
    match expr {
        Expr::Binary {
            op: BinaryOp::And, ..
        } => {}
        _ => panic!("Expected and expression"),
    }

    let input = "a || b";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();
    match expr {
        Expr::Binary {
            op: BinaryOp::Or, ..
        } => {}
        _ => panic!("Expected or expression"),
    }
}

#[test]
fn test_parse_pattern_wildcard() {
    use metadol::ast::Pattern;

    let mut parser = Parser::new("_");
    let pattern = parser.parse_pattern().unwrap();

    match pattern {
        Pattern::Wildcard => {}
        _ => panic!("Expected wildcard pattern"),
    }
}

#[test]
fn test_parse_pattern_identifier() {
    use metadol::ast::Pattern;

    let mut parser = Parser::new("value");
    let pattern = parser.parse_pattern().unwrap();

    match pattern {
        Pattern::Identifier(name) => assert_eq!(name, "value"),
        _ => panic!("Expected identifier pattern"),
    }
}

#[test]
fn test_parse_pattern_constructor() {
    use metadol::ast::Pattern;

    let mut parser = Parser::new("Some(x, y)");
    let pattern = parser.parse_pattern().unwrap();

    match pattern {
        Pattern::Constructor { name, fields } => {
            assert_eq!(name, "Some");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected constructor pattern"),
    }
}

#[test]
fn test_parse_pattern_tuple() {
    use metadol::ast::Pattern;

    let mut parser = Parser::new("(x, y, z)");
    let pattern = parser.parse_pattern().unwrap();

    match pattern {
        Pattern::Tuple(patterns) => {
            assert_eq!(patterns.len(), 3);
        }
        _ => panic!("Expected tuple pattern"),
    }
}

#[test]
fn test_parse_type_named() {
    use metadol::ast::TypeExpr;

    let mut parser = Parser::new("MyType");
    let type_expr = parser.parse_type().unwrap();

    match type_expr {
        TypeExpr::Named(name) => assert_eq!(name, "MyType"),
        _ => panic!("Expected named type"),
    }
}

#[test]
fn test_parse_type_builtin() {
    use metadol::ast::TypeExpr;

    let types = vec![
        "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64", "Float32",
        "Float64", "Bool", "String", "Void",
    ];

    for type_name in types {
        let mut parser = Parser::new(type_name);
        let type_expr = parser.parse_type().unwrap();

        match type_expr {
            TypeExpr::Named(name) => assert_eq!(name, type_name),
            _ => panic!("Expected named type for {}", type_name),
        }
    }
}

#[test]
fn test_parse_type_generic() {
    use metadol::ast::TypeExpr;

    let mut parser = Parser::new("List<Int32>");
    let type_expr = parser.parse_type().unwrap();

    match type_expr {
        TypeExpr::Generic { name, args } => {
            assert_eq!(name, "List");
            assert_eq!(args.len(), 1);
            match &args[0] {
                TypeExpr::Named(n) => assert_eq!(n, "Int32"),
                _ => panic!("Expected Int32 type argument"),
            }
        }
        _ => panic!("Expected generic type"),
    }
}

#[test]
fn test_parse_type_tuple() {
    use metadol::ast::TypeExpr;

    let mut parser = Parser::new("(Int32, String)");
    let type_expr = parser.parse_type().unwrap();

    match type_expr {
        TypeExpr::Tuple(types) => {
            assert_eq!(types.len(), 2);
        }
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_parse_type_function() {
    use metadol::ast::TypeExpr;

    let mut parser = Parser::new("(Int32, String) -> Bool");
    let type_expr = parser.parse_type().unwrap();

    match type_expr {
        TypeExpr::Function {
            params,
            return_type,
        } => {
            assert_eq!(params.len(), 2);
            match *return_type {
                TypeExpr::Named(ref name) => assert_eq!(name, "Bool"),
                _ => panic!("Expected Bool return type"),
            }
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_parse_complex_nested_expression() {
    use metadol::ast::Expr;

    // Complex expression combining multiple DOL 2.0 features
    let input = "data |> (|x| x) >> validate";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    // Just verify it parses without error
    match expr {
        Expr::Binary { .. } => {}
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_parse_idiom_bracket() {
    use metadol::ast::Expr;

    // Idiom brackets: [| f a b |]
    let input = "[| add x y |]";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::IdiomBracket { func, args } => {
            // The function should be the identifier "add"
            match *func {
                Expr::Identifier(ref name) => assert_eq!(name, "add"),
                _ => panic!("Expected identifier for function"),
            }
            // Should have 2 arguments
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected IdiomBracket expression"),
    }
}

#[test]
fn test_parse_idiom_bracket_single_arg() {
    use metadol::ast::Expr;

    // Idiom brackets with single argument: [| f a |]
    let input = "[| transform value |]";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::IdiomBracket { func, args } => {
            match *func {
                Expr::Identifier(ref name) => assert_eq!(name, "transform"),
                _ => panic!("Expected identifier for function"),
            }
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected IdiomBracket expression"),
    }
}

#[test]
fn test_parse_idiom_bracket_nested() {
    use metadol::ast::Expr;

    // Idiom brackets with expressions as arguments
    // Note: add (1 + 2) is parsed as a function call, so we get:
    // func = add(1+2), args = [3]
    let input = "[| add (1 + 2) 3 |]";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::IdiomBracket { func, args } => {
            // The first expression is add(1+2) as a call
            match *func {
                Expr::Call { callee, args: call_args } => {
                    match *callee {
                        Expr::Identifier(ref name) => assert_eq!(name, "add"),
                        _ => panic!("Expected identifier for callee"),
                    }
                    assert_eq!(call_args.len(), 1);
                }
                _ => panic!("Expected Call expression for function"),
            }
            // The remaining argument is 3
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected IdiomBracket expression"),
    }
}

#[test]
fn test_parse_idiom_bracket_simple_literals() {
    use metadol::ast::Expr;

    // Simple idiom brackets with literals
    let input = "[| f 1 2 |]";
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr(0).unwrap();

    match expr {
        Expr::IdiomBracket { func, args } => {
            match *func {
                Expr::Identifier(ref name) => assert_eq!(name, "f"),
                _ => panic!("Expected identifier for function"),
            }
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected IdiomBracket expression"),
    }
}
