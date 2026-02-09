//! Version resolution tests

use gen_registry::{version::VersionResolver, Dependency, GenModule};

#[test]
fn test_version_requirement_parse() {
    use gen_registry::version::VersionRequirement;

    let req = VersionRequirement::parse("io.univrs.test", "^1.0.0").unwrap();
    assert_eq!(req.module_id, "io.univrs.test");
}

#[test]
fn test_version_requirement_matches() {
    use gen_registry::version::VersionRequirement;

    let req = VersionRequirement::parse("io.univrs.test", "^1.0.0").unwrap();
    assert!(req.matches("1.2.3").unwrap());
    assert!(req.matches("1.9.9").unwrap());
    assert!(!req.matches("2.0.0").unwrap());
}

#[test]
fn test_version_requirement_exact() {
    use gen_registry::version::VersionRequirement;

    let req = VersionRequirement::parse("io.univrs.test", "=1.2.3").unwrap();
    assert!(req.matches("1.2.3").unwrap());
    assert!(!req.matches("1.2.4").unwrap());
}

#[test]
fn test_version_requirement_range() {
    use gen_registry::version::VersionRequirement;

    let req = VersionRequirement::parse("io.univrs.test", ">=1.0.0, <2.0.0").unwrap();
    assert!(req.matches("1.5.0").unwrap());
    assert!(!req.matches("2.0.0").unwrap());
}

#[test]
fn test_version_resolver_new() {
    let resolver = VersionResolver::new();
    assert!(resolver.topological_sort(&[]).unwrap().is_empty());
}

#[test]
fn test_cycle_detection() {
    let mut resolver = VersionResolver::new();
    resolver.add_dependency("A", "B");
    resolver.add_dependency("B", "C");
    resolver.add_dependency("C", "A");

    assert!(resolver.has_cycle("A", "B").unwrap());
}

#[test]
fn test_no_cycle() {
    let mut resolver = VersionResolver::new();
    resolver.add_dependency("A", "B");
    resolver.add_dependency("B", "C");

    assert!(!resolver.has_cycle("A", "B").unwrap());
    assert!(!resolver.has_cycle("B", "C").unwrap());
}

#[test]
fn test_topological_sort_linear() {
    let mut resolver = VersionResolver::new();
    resolver.add_dependency("A", "B");
    resolver.add_dependency("B", "C");

    let sorted = resolver
        .topological_sort(&["A".to_string(), "B".to_string(), "C".to_string()])
        .unwrap();

    // C should come before B, B before A
    let c_pos = sorted.iter().position(|s| s == "C").unwrap();
    let b_pos = sorted.iter().position(|s| s == "B").unwrap();
    let a_pos = sorted.iter().position(|s| s == "A").unwrap();

    assert!(c_pos < b_pos);
    assert!(b_pos < a_pos);
}

#[test]
fn test_topological_sort_diamond() {
    let mut resolver = VersionResolver::new();
    resolver.add_dependency("A", "B");
    resolver.add_dependency("A", "C");
    resolver.add_dependency("B", "D");
    resolver.add_dependency("C", "D");

    let sorted = resolver
        .topological_sort(&[
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ])
        .unwrap();

    // D should come before B and C, B and C before A
    let d_pos = sorted.iter().position(|s| s == "D").unwrap();
    let b_pos = sorted.iter().position(|s| s == "B").unwrap();
    let c_pos = sorted.iter().position(|s| s == "C").unwrap();
    let a_pos = sorted.iter().position(|s| s == "A").unwrap();

    assert!(d_pos < b_pos);
    assert!(d_pos < c_pos);
    assert!(b_pos < a_pos);
    assert!(c_pos < a_pos);
}

#[test]
fn test_cycle_detection_direct() {
    let mut resolver = VersionResolver::new();
    resolver.add_dependency("A", "A");

    assert!(resolver.has_cycle("A", "A").unwrap());
}

#[test]
fn test_dependency_creation() {
    let dep = Dependency::new("io.univrs.test", "^1.0.0");
    assert_eq!(dep.module_id, "io.univrs.test");
    assert_eq!(dep.version_requirement, "^1.0.0");
    assert!(!dep.optional);
}

#[test]
fn test_optional_dependency() {
    let dep = Dependency::new("io.univrs.test", "^1.0.0").optional();
    assert!(dep.optional);
}

#[test]
fn test_multiple_dependencies() {
    let mut module = GenModule::new(
        "io.univrs.multi",
        "Multi Deps",
        "Testing",
        "did:key:alice",
        "MIT",
    );

    module.add_dependency(Dependency::new("io.univrs.a", "^1.0.0"));
    module.add_dependency(Dependency::new("io.univrs.b", "^2.0.0"));
    module.add_dependency(Dependency::new("io.univrs.c", "^3.0.0").optional());

    assert_eq!(module.dependencies.len(), 3);
}
