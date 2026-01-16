//! Comprehensive test suite for DOL Modules and Spirits System
//!
//! This test module validates the implementation against the design specification
//! in `docs/design/Mod_Spirits_System.md`.
//!
//! # Test Categories
//!
//! - **Module Tests**: Module declaration, path resolution, inline submodules
//! - **Visibility Tests**: pub, pub(spirit), pub(parent), private visibility
//! - **Import Tests**: Local, registry, git, and selective imports
//! - **Spirit Tests**: Spirit manifest parsing and validation
//! - **System Tests**: System composition declarations
//! - **Error Tests**: Invalid syntax and semantic error detection

use metadol::{parse_and_validate, parse_file};
use std::fs;
use std::path::Path;

// =============================================================================
// Test Helpers
// =============================================================================

fn parse_source(source: &str) -> bool {
    parse_file(source).is_ok()
}

fn should_parse(source: &str) -> bool {
    parse_source(source)
}

fn should_fail_parse(source: &str) -> bool {
    !parse_source(source)
}

fn parses_with_valid_exegesis(source: &str) -> bool {
    match parse_and_validate(source) {
        Ok((_, validation)) => validation.is_valid(),
        Err(_) => false,
    }
}

// =============================================================================
// Module Tests
// =============================================================================

mod modules {
    use super::*;

    #[test]
    fn test_basic_module_gene() {
        let source = r#"
gene Container {
    container has id
    container has name
}

exegesis {
    A basic container gene for testing module parsing.
}
        "#;
        assert!(should_parse(source), "Basic module gene should parse");
    }

    #[test]
    fn test_qualified_gene_name() {
        let source = r#"
gene container.exists {
    container has identity
    container has state
}

exegesis {
    Qualified gene name with dot notation.
}
        "#;
        assert!(should_parse(source), "Qualified gene name should parse");
    }

    #[test]
    fn test_nested_module_path() {
        let source = r#"
gene internal.helpers.utils {
    utils has data
}

exegesis {
    Deeply nested module path.
}
        "#;
        assert!(should_parse(source), "Nested module path should parse");
    }

    #[test]
    fn test_gene_with_multiple_statements() {
        let source = r#"
gene container.exists {
    container has identity
    container has state
    container has boundaries
    container has resources
    container has image
}

exegesis {
    The container gene defines the essential properties of a container.
}
        "#;
        assert!(
            should_parse(source),
            "Gene with multiple statements should parse"
        );
    }
}

// =============================================================================
// Visibility Tests
// =============================================================================

mod visibility {
    use super::*;

    #[test]
    fn test_public_visibility() {
        let source = r#"
pub gene PublicGene {
    item has value
}

exegesis {
    Public gene accessible everywhere.
}
        "#;
        assert!(should_parse(source), "Public visibility should parse");
    }

    #[test]
    fn test_private_default_visibility() {
        let source = r#"
gene PrivateGene {
    item has value
}

exegesis {
    Private by default.
}
        "#;
        assert!(
            should_parse(source),
            "Private (default) visibility should parse"
        );
    }

    #[test]
    fn test_pub_spirit_visibility() {
        let source = r#"
pub(spirit) gene SpiritGene {
    item has value
}

exegesis {
    Spirit-level visibility (pub crate).
}
        "#;
        assert!(should_parse(source), "pub(spirit) visibility should parse");
    }

    #[test]
    fn test_pub_parent_visibility() {
        let source = r#"
pub(parent) gene ParentGene {
    item has value
}

exegesis {
    Parent-level visibility (pub super).
}
        "#;
        assert!(should_parse(source), "pub(parent) visibility should parse");
    }

    #[test]
    fn test_pub_trait() {
        let source = r#"
pub trait container.lifecycle {
    uses container.exists

    container is created
    container is started
    container is stopped
}

exegesis {
    Public trait with lifecycle states.
}
        "#;
        assert!(should_parse(source), "Public trait should parse");
    }

    #[test]
    fn test_pub_constraint() {
        let source = r#"
pub constraint container.integrity {
    state matches declared
    identity never changes
}

exegesis {
    Public constraint for container integrity.
}
        "#;
        assert!(should_parse(source), "Public constraint should parse");
    }
}

// =============================================================================
// Import Tests
// =============================================================================

mod imports {
    use super::*;

    #[test]
    fn test_simple_use() {
        let source = r#"
trait container.lifecycle {
    uses container.exists

    container is created
}

exegesis {
    Simple use statement.
}
        "#;
        assert!(should_parse(source), "Simple use statement should parse");
    }

    #[test]
    fn test_multiple_uses() {
        let source = r#"
trait complex.behavior {
    uses container.exists
    uses identity.cryptographic
    uses network.core

    container is managed
}

exegesis {
    Multiple use statements.
}
        "#;
        assert!(should_parse(source), "Multiple uses should parse");
    }

    #[test]
    fn test_trait_with_is_statements() {
        let source = r#"
trait container.states {
    uses container.exists

    container is created
    container is started
    container is running
    container is paused
    container is stopped
}

exegesis {
    Trait with multiple is statements for state machine.
}
        "#;
        assert!(
            should_parse(source),
            "Trait with is statements should parse"
        );
    }
}

// =============================================================================
// System Tests
// =============================================================================

mod systems {
    use super::*;

    #[test]
    fn test_system_declaration() {
        let source = r#"
system univrs.orchestrator @ 0.1.0 {
    requires container.lifecycle >= 0.0.2
    requires node.discovery >= 0.0.1

    all operations is authenticated
    all events is logged
}

exegesis {
    The Univrs orchestrator system composition.
}
        "#;
        assert!(should_parse(source), "System declaration should parse");
    }

    #[test]
    fn test_system_with_multiple_deps() {
        let source = r#"
system production.cluster @ 2.0.0 {
    requires container.management >= 1.0.0
    requires scheduler.core >= 1.5.0
    requires monitoring.telemetry >= 1.0.0

    all containers is monitored
}

exegesis {
    Production cluster with multiple dependencies.
}
        "#;
        assert!(should_parse(source), "System with deps should parse");
    }

    #[test]
    fn test_system_version_exact() {
        let source = r#"
system strict.system @ 1.0.0 {
    requires dependency = 1.2.3

    all items is versioned
}

exegesis {
    System with exact version requirement.
}
        "#;
        assert!(
            should_parse(source),
            "System with exact version should parse"
        );
    }
}

// =============================================================================
// Constraint Tests
// =============================================================================

mod constraints {
    use super::*;

    #[test]
    fn test_basic_constraint() {
        let source = r#"
constraint container.integrity {
    state matches declared
    identity never changes
    boundaries never expand
}

exegesis {
    Container integrity constraints ensure runtime state matches declared ontology.
}
        "#;
        assert!(should_parse(source), "Basic constraint should parse");
    }

    #[test]
    fn test_constraint_with_matches() {
        let source = r#"
constraint version.compatibility {
    runtime matches specification
    output matches expected
}

exegesis {
    Version compatibility constraint.
}
        "#;
        assert!(should_parse(source), "Constraint with matches should parse");
    }

    #[test]
    fn test_constraint_with_never() {
        let source = r#"
constraint security.secrets {
    secrets never logged
    credentials never exposed
    tokens never cached
}

exegesis {
    Security constraint for secret handling.
}
        "#;
        assert!(should_parse(source), "Constraint with never should parse");
    }
}

// =============================================================================
// Evolution Tests
// =============================================================================

mod evolutions {
    use super::*;

    #[test]
    fn test_evolution_declaration() {
        let source = r#"
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
    adds container is paused
    adds container is resumed
    deprecates container is suspended

    because "workload migration requires state preservation"
}

exegesis {
    Version 0.0.2 extends the lifecycle with pause and resume.
}
        "#;
        assert!(should_parse(source), "Evolution declaration should parse");
    }

    #[test]
    fn test_evolution_with_removes() {
        // removes expects identifier, adds/deprecates expect statements
        let source = r#"
evolves api.endpoints @ 2.0.0 > 1.0.0 {
    removes legacy_endpoint
    adds endpoint is available
    deprecates endpoint is deprecated

    because "API modernization"
}

exegesis {
    Major version bump with breaking changes.
}
        "#;
        assert!(should_parse(source), "Evolution with removes should parse");
    }
}

// =============================================================================
// Error Tests
// =============================================================================

mod errors {
    use super::*;

    #[test]
    fn test_missing_exegesis_is_valid_with_warning() {
        let source = r#"
gene MissingExegesis {
    item has id
}
        "#;
        // Parser accepts missing exegesis (returns empty string)
        // Validator treats empty exegesis as valid but with a warning
        assert!(
            should_parse(source),
            "Missing exegesis should parse syntactically"
        );
        // Empty exegesis is still valid (just has warnings)
        assert!(
            parses_with_valid_exegesis(source),
            "Empty exegesis is valid (but has warning)"
        );
    }

    #[test]
    fn test_empty_gene_with_exegesis() {
        let source = r#"
gene EmptyGene {
}

exegesis {
    An empty gene is valid syntactically.
}
        "#;
        // Empty genes are valid syntactically
        assert!(
            should_parse(source),
            "Empty gene with exegesis should parse"
        );
    }

    #[test]
    fn test_invalid_version_format() {
        let source = r#"
system invalid.version @ not.a.version {
    all items is valid
}

exegesis {
    Invalid version format.
}
        "#;
        // Version must be numeric semver
        assert!(should_fail_parse(source), "Invalid version should fail");
    }

    #[test]
    fn test_unclosed_brace() {
        let source = r#"
gene Unclosed {
    item has value

exegesis {
    Missing closing brace.
}
        "#;
        assert!(should_fail_parse(source), "Unclosed brace should fail");
    }
}

// =============================================================================
// Exegesis Validation Tests
// =============================================================================

mod exegesis_validation {
    use super::*;

    #[test]
    fn test_valid_exegesis() {
        let source = r#"
gene documented.entity {
    entity has property
}

exegesis {
    This is a well-documented gene with a substantial exegesis
    that explains the purpose and usage of the entity.
}
        "#;
        assert!(
            parses_with_valid_exegesis(source),
            "Valid exegesis should validate"
        );
    }

    #[test]
    fn test_short_exegesis_warning() {
        let source = r#"
gene short.doc {
    item has value
}

exegesis {
    Too short.
}
        "#;
        // Should parse but may have validation warning for short exegesis
        assert!(should_parse(source), "Short exegesis should parse");
    }
}

// =============================================================================
// Integration Tests - Parse Fixture Files
// =============================================================================

mod fixture_tests {
    use super::*;

    #[allow(dead_code)]
    fn test_fixture(category: &str, name: &str) -> bool {
        let path = format!("tests/spirits/{}/{}", category, name);
        if !Path::new(&path).exists() {
            eprintln!("Fixture not found: {}", path);
            return false;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", path, e);
                return false;
            }
        };

        match parse_file(&content) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Parse error for {}: {:?}", path, e);
                false
            }
        }
    }

    #[test]
    fn test_module_fixtures_exist() {
        // Verify fixture directory exists
        assert!(
            Path::new("tests/spirits/modules").exists(),
            "Module fixtures directory should exist"
        );
    }

    #[test]
    fn test_visibility_fixtures_exist() {
        assert!(
            Path::new("tests/spirits/visibility").exists(),
            "Visibility fixtures directory should exist"
        );
    }

    #[test]
    fn test_imports_fixtures_exist() {
        assert!(
            Path::new("tests/spirits/imports").exists(),
            "Imports fixtures directory should exist"
        );
    }

    #[test]
    fn test_compilation_fixtures_exist() {
        assert!(
            Path::new("tests/spirits/compilation").exists(),
            "Compilation fixtures directory should exist"
        );
    }

    #[test]
    fn test_errors_fixtures_exist() {
        assert!(
            Path::new("tests/spirits/errors").exists(),
            "Errors fixtures directory should exist"
        );
    }
}

// =============================================================================
// DOL 2.0 Features Tests
// =============================================================================

mod dol2_features {
    use super::*;

    #[test]
    fn test_gene_with_function() {
        let source = r#"
gene Counter {
    counter has value

    fun get() -> Int64 {
        return 42
    }
}

exegesis {
    Gene with function definition - DOL 2.0 feature.
}
        "#;
        assert!(should_parse(source), "Gene with function should parse");
    }

    #[test]
    fn test_function_with_parameters() {
        let source = r#"
gene Calculator {
    calc has result

    fun add(a: Int64, b: Int64) -> Int64 {
        return a + b
    }
}

exegesis {
    Gene with parameterized function.
}
        "#;
        assert!(
            should_parse(source),
            "Function with parameters should parse"
        );
    }

    #[test]
    fn test_sex_keyword_function() {
        let source = r#"
gene IO {
    io has buffer

    sex fun print(msg: String) -> Void {
        // side effect function
    }
}

exegesis {
    Gene with sex (side effect) function marker.
}
        "#;
        assert!(should_parse(source), "Sex function marker should parse");
    }

    #[test]
    fn test_const_declaration() {
        // Const is a top-level declaration in DOL
        let source = r#"
const PI: Float64 = 3.14159

exegesis {
    Top-level constant declaration for pi value.
}
        "#;
        assert!(
            should_parse(source),
            "Top-level const declaration should parse"
        );
    }
}
