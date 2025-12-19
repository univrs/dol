//! Parser unit tests for Metal DOL AST generation
//!
//! These tests verify that the parser correctly transforms token streams
//! into valid Abstract Syntax Trees.

use metadol::parser::Parser;
use metadol::ast::{Declaration, Statement, Gene, Trait, Constraint, System, Evolution};

mod gene_tests {
    use super::*;

    #[test]
    fn test_parse_simple_gene() {
        let input = r#"
gene container.exists {
  container has identity
  container has state
  container has boundaries
}

exegesis {
  A container is the fundamental unit of workload isolation.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        let decl = result.unwrap();
        match decl {
            Declaration::Gene(gene) => {
                assert_eq!(gene.name, "container.exists");
                assert_eq!(gene.statements.len(), 3);
                assert!(gene.exegesis.contains("fundamental unit"));
            }
            _ => panic!("Expected Gene declaration"),
        }
    }

    #[test]
    fn test_gene_with_derives() {
        let input = r#"
gene identity.cryptographic {
  identity derives from ed25519 keypair
  identity is self-sovereign
  identity requires no authority
}

exegesis {
  Cryptographic identity is the foundation of trust.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok());
        
        if let Declaration::Gene(gene) = result.unwrap() {
            assert_eq!(gene.name, "identity.cryptographic");
            
            // Verify derives statement
            let derives_stmt = &gene.statements[0];
            match derives_stmt {
                Statement::DerivesFrom { subject, origin, .. } => {
                    assert_eq!(subject, "identity");
                    assert!(origin.contains("ed25519"));
                }
                _ => panic!("Expected DerivesFrom statement"),
            }
        }
    }

    #[test]
    fn test_gene_missing_exegesis() {
        let input = r#"
gene container.exists {
  container has identity
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        // Should fail - exegesis is mandatory
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("exegesis"), "Error should mention exegesis");
    }
}

mod trait_tests {
    use super::*;

    #[test]
    fn test_parse_trait_with_uses() {
        let input = r#"
trait container.lifecycle {
  uses container.exists
  uses identity.cryptographic
  uses state.finite

  container is created
  container is started
  container is stopped
  container is destroyed

  each transition emits event
}

exegesis {
  The container lifecycle defines the state machine.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        if let Declaration::Trait(trait_decl) = result.unwrap() {
            assert_eq!(trait_decl.name, "container.lifecycle");
            
            // Count uses statements
            let uses_count = trait_decl.statements.iter()
                .filter(|s| matches!(s, Statement::Uses { .. }))
                .count();
            assert_eq!(uses_count, 3);
            
            // Count is statements
            let is_count = trait_decl.statements.iter()
                .filter(|s| matches!(s, Statement::Is { .. }))
                .count();
            assert_eq!(is_count, 4);
        } else {
            panic!("Expected Trait declaration");
        }
    }

    #[test]
    fn test_trait_with_emits() {
        let input = r#"
trait container.events {
  uses container.lifecycle

  state change emits notification
  transition emits audit event
}

exegesis {
  Event emission for observability.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok());
        
        if let Declaration::Trait(trait_decl) = result.unwrap() {
            let emits_stmts: Vec<_> = trait_decl.statements.iter()
                .filter(|s| matches!(s, Statement::Emits { .. }))
                .collect();
            assert_eq!(emits_stmts.len(), 2);
        }
    }
}

mod constraint_tests {
    use super::*;

    #[test]
    fn test_parse_constraint() {
        let input = r#"
constraint container.integrity {
  container state matches declared state
  container identity never changes
  container boundaries are enforced
}

exegesis {
  Container integrity ensures runtime matches declared ontology.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        if let Declaration::Constraint(constraint) = result.unwrap() {
            assert_eq!(constraint.name, "container.integrity");
            
            // Should have matches and never statements
            let has_matches = constraint.statements.iter()
                .any(|s| matches!(s, Statement::Matches { .. }));
            let has_never = constraint.statements.iter()
                .any(|s| matches!(s, Statement::Never { .. }));
            
            assert!(has_matches, "Should have matches statement");
            assert!(has_never, "Should have never statement");
        } else {
            panic!("Expected Constraint declaration");
        }
    }
}

mod system_tests {
    use super::*;

    #[test]
    fn test_parse_system_with_requirements() {
        let input = r#"
system univrs.orchestrator @ 0.1.0 {
  requires container.lifecycle >= 0.0.2
  requires node.discovery >= 0.0.1
  requires cluster.consistency >= 0.0.1

  nodes discover peers via gossip
  containers schedule across nodes
  consensus validates state transitions
  
  all operations are authenticated
  all state is replicated
}

exegesis {
  The Univrs orchestrator is the primary system composition.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        if let Declaration::System(system) = result.unwrap() {
            assert_eq!(system.name, "univrs.orchestrator");
            assert_eq!(system.version, "0.1.0");
            assert_eq!(system.requirements.len(), 3);
            
            // Verify first requirement
            let first_req = &system.requirements[0];
            assert_eq!(first_req.name, "container.lifecycle");
            assert_eq!(first_req.constraint, ">=");
            assert_eq!(first_req.version, "0.0.2");
        } else {
            panic!("Expected System declaration");
        }
    }
}

mod evolution_tests {
    use super::*;

    #[test]
    fn test_parse_evolution() {
        let input = r#"
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  adds container is resumed
  
  because "workload migration requires state preservation"
}

exegesis {
  Version 0.0.2 extends the lifecycle to support pause and resume.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        if let Declaration::Evolution(evolution) = result.unwrap() {
            assert_eq!(evolution.name, "container.lifecycle");
            assert_eq!(evolution.version, "0.0.2");
            assert_eq!(evolution.parent_version, "0.0.1");
            
            // Should have 2 adds and 1 because
            assert_eq!(evolution.additions.len(), 2);
            assert!(evolution.rationale.is_some());
            assert!(evolution.rationale.unwrap().contains("migration"));
        } else {
            panic!("Expected Evolution declaration");
        }
    }

    #[test]
    fn test_evolution_with_deprecations() {
        let input = r#"
evolves container.lifecycle @ 0.1.0 > 0.0.3 {
  adds container is archived
  deprecates container is destroyed
  
  because "soft deletion preferred over hard deletion"
}

exegesis {
  Version 0.1.0 introduces archival as the preferred termination method.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok());
        
        if let Declaration::Evolution(evolution) = result.unwrap() {
            assert_eq!(evolution.additions.len(), 1);
            assert_eq!(evolution.deprecations.len(), 1);
        }
    }
}

mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_missing_brace() {
        let input = r#"
gene container.exists
  container has identity
}

exegesis {
  Missing opening brace.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("expected") || err.message.contains("{"));
    }

    #[test]
    fn test_invalid_statement() {
        let input = r#"
gene container.exists {
  container foo bar baz
}

exegesis {
  Invalid statement predicate.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error should indicate unexpected token or invalid predicate
        assert!(err.span.line > 0, "Error should have valid span");
    }

    #[test]
    fn test_error_span_accuracy() {
        let input = r#"
gene container.exists {
  container has identity
  container BADTOKEN state
}

exegesis {
  Error on line 4.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.span.line, 4, "Error should be on line 4");
    }
}

mod exegesis_tests {
    use super::*;

    #[test]
    fn test_multiline_exegesis() {
        let input = r#"
gene container.exists {
  container has identity
}

exegesis {
  This is the first paragraph of the exegesis.
  
  This is the second paragraph with more detail about
  the container abstraction and its role in the system.
  
  Key points:
  - Identity is cryptographic
  - State is immutable
  - Boundaries are enforced
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok());
        
        if let Declaration::Gene(gene) = result.unwrap() {
            assert!(gene.exegesis.contains("first paragraph"));
            assert!(gene.exegesis.contains("second paragraph"));
            assert!(gene.exegesis.contains("Key points"));
        }
    }

    #[test]
    fn test_exegesis_preserves_formatting() {
        let input = r#"
gene test.formatting {
  test has property
}

exegesis {
  Line 1
  Line 2
  
  Line 4 after blank
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        if let Declaration::Gene(gene) = result.unwrap() {
            // Exegesis should preserve line structure
            assert!(gene.exegesis.contains('\n'), "Should preserve newlines");
        }
    }
}

mod integration_tests {
    use super::*;
    use std::fs;

    #[test]
    #[ignore] // Enable when example files exist
    fn test_parse_all_examples() {
        let example_dirs = ["examples/genes", "examples/traits", "examples/constraints"];
        
        for dir in example_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "dol") {
                        let content = fs::read_to_string(&path)
                            .expect(&format!("Failed to read {:?}", path));
                        
                        let mut parser = Parser::new(&content);
                        let result = parser.parse();
                        
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
}

mod ast_visitor_tests {
    use super::*;

    #[test]
    fn test_collect_all_identifiers() {
        let input = r#"
gene container.exists {
  container has identity
  container has state
}

exegesis {
  Test gene.
}
"#;
        let mut parser = Parser::new(input);
        let decl = parser.parse().unwrap();
        
        let identifiers = decl.collect_identifiers();
        
        assert!(identifiers.contains(&"container.exists".to_string()));
        assert!(identifiers.contains(&"identity".to_string()));
        assert!(identifiers.contains(&"state".to_string()));
    }

    #[test]
    fn test_collect_dependencies() {
        let input = r#"
trait container.lifecycle {
  uses container.exists
  uses identity.cryptographic
  
  container is created
}

exegesis {
  Test trait.
}
"#;
        let mut parser = Parser::new(input);
        let decl = parser.parse().unwrap();
        
        let deps = decl.collect_dependencies();
        
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"container.exists".to_string()));
        assert!(deps.contains(&"identity.cryptographic".to_string()));
    }
}
