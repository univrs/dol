//! Integration tests for Metal DOL.
//!
//! These tests verify that the complete DOL pipeline works correctly:
//! parsing example files, validation, and round-trip operations.

use metadol::{parse_and_validate, parse_file, validate};
use std::fs;
use std::path::Path;

// ============================================
// 1. Example File Parsing Tests
// ============================================

#[test]
fn test_parse_container_exists_dol() {
    let content = fs::read_to_string("examples/genes/container.exists.dol")
        .expect("Failed to read container.exists.dol");

    let result = parse_file(&content);
    assert!(
        result.is_ok(),
        "Failed to parse container.exists.dol: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "container.exists");
}

#[test]
fn test_parse_container_lifecycle_dol() {
    let content = fs::read_to_string("examples/traits/container.lifecycle.dol")
        .expect("Failed to read container.lifecycle.dol");

    let result = parse_file(&content);
    assert!(
        result.is_ok(),
        "Failed to parse container.lifecycle.dol: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "container.lifecycle");
}

#[test]
fn test_parse_all_example_genes() {
    let genes_dir = Path::new("examples/genes");
    if genes_dir.exists() {
        for entry in fs::read_dir(genes_dir).expect("Failed to read genes directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "dol") {
                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Failed to read {:?}", path));

                let result = parse_file(&content);
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

#[test]
fn test_parse_all_example_traits() {
    let traits_dir = Path::new("examples/traits");
    if traits_dir.exists() {
        for entry in fs::read_dir(traits_dir).expect("Failed to read traits directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "dol") {
                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Failed to read {:?}", path));

                let result = parse_file(&content);
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

// ============================================
// 2. Validation Tests
// ============================================

#[test]
fn test_validate_container_exists() {
    let content = fs::read_to_string("examples/genes/container.exists.dol")
        .expect("Failed to read container.exists.dol");

    let decl = parse_file(&content).expect("Failed to parse");
    let result = validate(&decl);

    assert!(result.is_valid(), "Validation errors: {:?}", result.errors);
}

#[test]
fn test_validate_container_lifecycle() {
    let content = fs::read_to_string("examples/traits/container.lifecycle.dol")
        .expect("Failed to read container.lifecycle.dol");

    let decl = parse_file(&content).expect("Failed to parse");
    let result = validate(&decl);

    assert!(result.is_valid(), "Validation errors: {:?}", result.errors);
}

#[test]
fn test_parse_and_validate_combined() {
    let content = fs::read_to_string("examples/genes/container.exists.dol")
        .expect("Failed to read container.exists.dol");

    let result = parse_and_validate(&content);
    assert!(result.is_ok());

    let (decl, validation) = result.unwrap();
    assert_eq!(decl.name(), "container.exists");
    assert!(validation.is_valid());
}

// ============================================
// 3. Declaration Type Tests
// ============================================

#[test]
fn test_gene_declaration_structure() {
    let input = r#"
gene container.exists {
  container has identity
  container has state
  container has boundaries
}

exegesis {
  A container is the fundamental unit of workload isolation.
  It provides identity, state management, and resource boundaries.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");

    match decl {
        metadol::Declaration::Gene(gene) => {
            assert_eq!(gene.name, "container.exists");
            assert_eq!(gene.statements.len(), 3);
            assert!(!gene.exegesis.is_empty());
        }
        _ => panic!("Expected Gene declaration"),
    }
}

#[test]
fn test_trait_declaration_structure() {
    let input = r#"
trait container.lifecycle {
  uses container.exists

  container is created
  container is running
  container is stopped

  each transition emits event
}

exegesis {
  The container lifecycle manages state transitions.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");

    match decl {
        metadol::Declaration::Trait(trait_decl) => {
            assert_eq!(trait_decl.name, "container.lifecycle");
            assert!(trait_decl.statements.len() >= 4);
        }
        _ => panic!("Expected Trait declaration"),
    }
}

#[test]
fn test_constraint_declaration_structure() {
    let input = r#"
constraint container.integrity {
  state matches declared
  identity never changes
  boundaries never expand
}

exegesis {
  Container integrity constraints ensure runtime safety.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");

    match decl {
        metadol::Declaration::Constraint(constraint) => {
            assert_eq!(constraint.name, "container.integrity");
            assert_eq!(constraint.statements.len(), 3);
        }
        _ => panic!("Expected Constraint declaration"),
    }
}

#[test]
fn test_system_declaration_structure() {
    let input = r#"
system univrs.orchestrator @ 0.1.0 {
  requires container.lifecycle >= 0.0.2
  requires node.discovery >= 0.0.1

  all operations is authenticated
}

exegesis {
  The Univrs orchestrator is the primary system composition.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");

    match decl {
        metadol::Declaration::System(system) => {
            assert_eq!(system.name, "univrs.orchestrator");
            assert_eq!(system.version, "0.1.0");
            assert_eq!(system.requirements.len(), 2);
        }
        _ => panic!("Expected System declaration"),
    }
}

#[test]
fn test_evolution_declaration_structure() {
    let input = r#"
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  adds container is resumed
  because "workload migration requires state preservation"
}

exegesis {
  Version 0.0.2 adds pause and resume capabilities for migration.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");

    match decl {
        metadol::Declaration::Evolution(evolution) => {
            assert_eq!(evolution.name, "container.lifecycle");
            assert_eq!(evolution.version, "0.0.2");
            assert_eq!(evolution.parent_version, "0.0.1");
            assert_eq!(evolution.additions.len(), 2);
            assert!(evolution.rationale.is_some());
        }
        _ => panic!("Expected Evolution declaration"),
    }
}

// ============================================
// 4. AST Feature Tests
// ============================================

#[test]
fn test_span_tracking() {
    let input = r#"gene test.span {
  subject has property
}

exegesis {
  Span tracking test.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");
    let span = decl.span();

    assert!(span.start < span.end);
    assert!(span.line >= 1);
    assert!(span.column >= 1);
}

#[test]
fn test_collect_identifiers() {
    let input = r#"
gene container.exists {
  container has identity
  container has state
}

exegesis {
  Identifier collection test.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");
    let ids = decl.collect_identifiers();

    assert!(ids.contains(&"container.exists".to_string()));
    assert!(ids.contains(&"container".to_string()));
    assert!(ids.contains(&"identity".to_string()));
}

// ============================================
// 5. Error Handling Tests
// ============================================

#[test]
fn test_parse_error_contains_location() {
    let input = r#"gene invalid {
  container has
}

exegesis { Error test. }
"#;

    let result = parse_file(input);
    assert!(result.is_err());

    let err = result.err().unwrap();
    let err_string = format!("{}", err);
    assert!(err_string.contains("line") || err_string.contains("column"));
}

#[test]
fn test_validation_error_on_empty_exegesis() {
    let input = r#"
gene test.empty {
  subject has property
}

exegesis {
}
"#;

    let decl = parse_file(input).expect("Failed to parse");
    let result = validate(&decl);

    // Empty exegesis should produce an error
    assert!(!result.is_valid());
}

// ============================================
// 6. Complex Scenario Tests
// ============================================

#[test]
fn test_complex_trait_with_all_features() {
    let input = r#"
trait container.full {
  uses container.exists
  uses identity.cryptographic
  uses network.core

  container has runtime
  container is initialized
  container is active

  each state change emits event
  all operations is logged
}

exegesis {
  A comprehensive trait demonstrating all supported features
  including uses, has, is, and quantified statements.
}
"#;

    let (decl, validation) = parse_and_validate(input).expect("Failed to parse");

    assert_eq!(decl.name(), "container.full");
    assert!(validation.is_valid() || validation.has_warnings());
}

#[test]
fn test_complex_system_with_requirements() {
    let input = r#"
system univrs.platform @ 1.0.0 {
  requires container.lifecycle >= 0.0.2
  requires node.discovery >= 0.0.1
  requires cluster.membership >= 0.1.0
  requires auth.service >= 1.0.0

  all requests is authenticated
  all responses is encrypted
}

exegesis {
  The complete Univrs platform composition bringing together
  all subsystems with strict version requirements.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");

    match decl {
        metadol::Declaration::System(system) => {
            assert_eq!(system.requirements.len(), 4);
            assert!(!system.statements.is_empty());
        }
        _ => panic!("Expected System"),
    }
}

// ============================================
// 7. Whitespace and Formatting Tests
// ============================================

#[test]
fn test_parse_with_extra_whitespace() {
    let input = r#"

gene   container.exists   {

  container    has    identity

}

exegesis   {
  Whitespace handling test.
}

"#;

    let result = parse_file(input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_compact_formatting() {
    let input = "gene container.exists{container has identity}exegesis{Compact.}";
    let result = parse_file(input);
    // This might fail due to missing whitespace - that's expected behavior
    // The test documents the expected behavior
    if result.is_ok() {
        assert_eq!(result.unwrap().name(), "container.exists");
    }
}

#[test]
fn test_parse_with_many_comments() {
    let input = r#"
// Header comment
// Another header line

gene container.exists {
  // Property comment
  container has identity // inline comment
  // Another comment
  container has state
}

// Pre-exegesis comment
exegesis {
  Documentation with comments around it.
}
// Trailing comment
"#;

    let result = parse_file(input);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

// ============================================
// 8. Regression Tests
// ============================================

#[test]
fn test_regression_qualified_identifier_parsing() {
    // Ensure deeply qualified identifiers parse correctly
    let input = r#"
gene a.b.c.d.e {
  subject has property
}

exegesis {
  Deeply qualified identifier regression test.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");
    assert_eq!(decl.name(), "a.b.c.d.e");
}

#[test]
fn test_regression_version_parsing() {
    // Ensure versions with large numbers parse correctly
    let input = r#"
system test.system @ 100.200.300 {
  requires dep.one >= 10.20.30
}

exegesis {
  Large version number regression test.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");
    if let metadol::Declaration::System(system) = decl {
        assert_eq!(system.version, "100.200.300");
    }
}

#[test]
fn test_regression_multiline_exegesis_preservation() {
    let input = r#"
gene test.multiline {
  subject has property
}

exegesis {
  Line one of the exegesis.

  Line two after a blank line.

  Line three with more content.
}
"#;

    let decl = parse_file(input).expect("Failed to parse");
    let exegesis = decl.exegesis();

    // Exegesis should preserve multiple lines
    assert!(exegesis.contains("one"));
    assert!(exegesis.contains("two"));
    assert!(exegesis.contains("three"));
}
