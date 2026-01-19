//! Module system tests for DOL.
//!
//! Tests module declarations, use statements, import sources,
//! and visibility rules.

use metadol::ast::{ImportSource, UseItems, Visibility};
use metadol::parser::Parser;

// ============================================================================
// Module Declaration Tests
// ============================================================================

#[test]
fn module_simple() {
    let input = r#"
module container

gen Point {
    point has x: i64
}
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    let module = file.module.expect("should have module");
    assert_eq!(module.path, vec!["container"]);
    assert!(module.version.is_none());
}

#[test]
fn module_qualified_path() {
    let input = r#"
module univrs.container.state

gen State {
    state has value
}
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    let module = file.module.expect("should have module");
    assert_eq!(module.path, vec!["univrs", "container", "state"]);
}

#[test]
fn module_with_version() {
    let input = r#"
module container @ 1.2.3

gen Container {
    container has id
}
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    let module = file.module.expect("should have module");
    assert_eq!(module.path, vec!["container"]);
    let version = module.version.expect("should have version");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
}

// ============================================================================
// Local Use Declaration Tests
// ============================================================================

#[test]
fn use_local_simple() {
    let input = r#"
use container

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    assert!(matches!(use_decl.source, ImportSource::Local));
    assert_eq!(use_decl.path, vec!["container"]);
    assert!(matches!(use_decl.items, UseItems::Single));
    assert!(matches!(use_decl.visibility, Visibility::Private));
}

#[test]
fn use_local_qualified() {
    let input = r#"
use container.state.Running

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    assert!(matches!(use_decl.source, ImportSource::Local));
    assert_eq!(use_decl.path, vec!["container", "state", "Running"]);
}

#[test]
fn use_local_glob() {
    let input = r#"
use container.*

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    assert!(matches!(use_decl.items, UseItems::All));
}

#[test]
fn use_local_named_items() {
    let input = r#"
use container.{Container, State, Config}

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    if let UseItems::Named(items) = &use_decl.items {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].name, "Container");
        assert_eq!(items[1].name, "State");
        assert_eq!(items[2].name, "Config");
    } else {
        panic!("expected named items");
    }
}

#[test]
fn use_local_with_alias() {
    let input = r#"
use container.{Container as C, State as S}

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    if let UseItems::Named(items) = &use_decl.items {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Container");
        assert_eq!(items[0].alias, Some("C".to_string()));
        assert_eq!(items[1].name, "State");
        assert_eq!(items[1].alias, Some("S".to_string()));
    } else {
        panic!("expected named items");
    }
}

#[test]
fn use_module_alias() {
    let input = r#"
use container.state as st

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    assert_eq!(use_decl.alias, Some("st".to_string()));
}

// ============================================================================
// Public Use (Re-export) Tests
// ============================================================================

#[test]
fn pub_use_simple() {
    let input = r#"
pub use container.Container

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    assert!(matches!(use_decl.visibility, Visibility::Public));
}

#[test]
fn pub_use_named() {
    let input = r#"
pub use container.{Container, State}

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    assert!(matches!(use_decl.visibility, Visibility::Public));
    if let UseItems::Named(items) = &use_decl.items {
        assert_eq!(items.len(), 2);
    } else {
        panic!("expected named items");
    }
}

// ============================================================================
// Registry Import Tests
// ============================================================================

#[test]
fn use_registry_simple() {
    let input = r#"
use @univrs/std

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    if let ImportSource::Registry {
        org,
        package,
        version,
    } = &use_decl.source
    {
        assert_eq!(org, "univrs");
        assert_eq!(package, "std");
        assert!(version.is_none());
    } else {
        panic!("expected registry import");
    }
}

#[test]
fn use_registry_with_subpath() {
    let input = r#"
use @univrs/std.io.println

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    if let ImportSource::Registry { org, package, .. } = &use_decl.source {
        assert_eq!(org, "univrs");
        assert_eq!(package, "std");
    } else {
        panic!("expected registry import");
    }
    // The subpath (io.println) should be in the path
    assert_eq!(use_decl.path, vec!["io", "println"]);
}

#[test]
fn use_registry_with_named_items() {
    let input = r#"
use @univrs/http.{Request, Response}

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    if let ImportSource::Registry { org, package, .. } = &use_decl.source {
        assert_eq!(org, "univrs");
        assert_eq!(package, "http");
    } else {
        panic!("expected registry import");
    }
    if let UseItems::Named(items) = &use_decl.items {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Request");
        assert_eq!(items[1].name, "Response");
    } else {
        panic!("expected named items");
    }
}

// ============================================================================
// Multiple Use Declarations Tests
// ============================================================================

#[test]
fn multiple_use_declarations() {
    let input = r#"
use container
use lifecycle
pub use @univrs/std.io

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 3);

    // First: local container
    assert!(matches!(file.uses[0].source, ImportSource::Local));
    assert_eq!(file.uses[0].path, vec!["container"]);
    assert!(matches!(file.uses[0].visibility, Visibility::Private));

    // Second: local lifecycle
    assert!(matches!(file.uses[1].source, ImportSource::Local));
    assert_eq!(file.uses[1].path, vec!["lifecycle"]);

    // Third: pub use registry
    assert!(matches!(file.uses[2].visibility, Visibility::Public));
    if let ImportSource::Registry { org, package, .. } = &file.uses[2].source {
        assert_eq!(org, "univrs");
        assert_eq!(package, "std");
    } else {
        panic!("expected registry import");
    }
    // The subpath "io" should be in the path
    assert_eq!(file.uses[2].path, vec!["io"]);
}

// ============================================================================
// Complete File Structure Tests
// ============================================================================

#[test]
fn complete_file_with_module_and_uses() {
    let input = r#"
module container.lib @ 1.0.0

use internal.helpers
pub use state.{ContainerState, Running, Stopped}

gen Container {
    container has id: u64
    container has name: string
}

fun create(name: string) -> Container {
    Container { id: 0, name: name }
}
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    // Check module
    let module = file.module.expect("should have module");
    assert_eq!(module.path, vec!["container", "lib"]);
    assert!(module.version.is_some());

    // Check uses
    assert_eq!(file.uses.len(), 2);

    // Private import
    assert!(matches!(file.uses[0].visibility, Visibility::Private));

    // Public re-export
    assert!(matches!(file.uses[1].visibility, Visibility::Public));
    if let UseItems::Named(items) = &file.uses[1].items {
        assert_eq!(items.len(), 3);
    }

    // Check declarations
    assert_eq!(file.declarations.len(), 2); // gene + function
}

// ============================================================================
// Path Separator Variants Tests
// ============================================================================

#[test]
fn use_with_double_colon_separator() {
    let input = r#"
use container::state::Running

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    assert_eq!(use_decl.path, vec!["container", "state", "Running"]);
}

#[test]
fn use_mixed_separators() {
    let input = r#"
use container.state::config

gen Point { point has x }
"#;
    let mut parser = Parser::new(input);
    let file = parser.parse_file().expect("should parse");

    assert_eq!(file.uses.len(), 1);
    let use_decl = &file.uses[0];
    // Both . and :: should work as path separators
    assert!(use_decl.path.len() >= 2);
}
