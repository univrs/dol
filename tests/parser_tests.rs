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
