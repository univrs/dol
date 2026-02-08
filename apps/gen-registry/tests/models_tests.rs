//! Data models tests

use gen_registry::{Capability, Dependency, GenModule, InstalledModule, ModuleVersion, Rating};

#[test]
fn test_gen_module_new() {
    let module = GenModule::new(
        "io.univrs.test",
        "Test Module",
        "A test module",
        "did:key:alice",
        "MIT",
    );

    assert_eq!(module.id, "io.univrs.test");
    assert_eq!(module.name, "Test Module");
    assert_eq!(module.description, "A test module");
    assert_eq!(module.author_did, "did:key:alice");
    assert_eq!(module.license, "MIT");
    assert_eq!(module.download_count, 0);
}

#[test]
fn test_module_version_new() {
    let version = ModuleVersion::new(
        "1.0.0",
        "abc123",
        1024,
        "Initial release",
        "signature",
    );

    assert_eq!(version.version, "1.0.0");
    assert_eq!(version.wasm_hash, "abc123");
    assert_eq!(version.wasm_size, 1024);
    assert_eq!(version.changelog, "Initial release");
    assert!(!version.deprecated);
    assert!(!version.yanked);
}

#[test]
fn test_capability_function() {
    let cap = Capability::function("authenticate", "fn(String, String) -> Result<Token>");

    assert_eq!(cap.name, "authenticate");
    assert_eq!(cap.signature, "fn(String, String) -> Result<Token>");
}

#[test]
fn test_capability_type() {
    let cap = Capability::type_def("User", "struct User { id: String }");

    assert_eq!(cap.name, "User");
}

#[test]
fn test_dependency_new() {
    let dep = Dependency::new("io.univrs.crypto", "^1.0.0");

    assert_eq!(dep.module_id, "io.univrs.crypto");
    assert_eq!(dep.version_requirement, "^1.0.0");
    assert!(!dep.optional);
}

#[test]
fn test_dependency_optional() {
    let dep = Dependency::new("io.univrs.logging", "^0.5.0").optional();

    assert!(dep.optional);
}

#[test]
fn test_rating_new() {
    let rating = Rating::new("io.univrs.test", "did:key:alice", 5);

    assert_eq!(rating.module_id, "io.univrs.test");
    assert_eq!(rating.user_did, "did:key:alice");
    assert_eq!(rating.stars, 5);
    assert!(rating.review.is_empty());
}

#[test]
fn test_rating_with_review() {
    let rating = Rating::new("io.univrs.test", "did:key:alice", 4)
        .with_review("Great module!");

    assert_eq!(rating.review, "Great module!");
}

#[test]
fn test_rating_clamp_high() {
    let rating = Rating::new("io.univrs.test", "did:key:alice", 10);
    assert_eq!(rating.stars, 5);
}

#[test]
fn test_rating_clamp_low() {
    let rating = Rating::new("io.univrs.test", "did:key:alice", 0);
    assert_eq!(rating.stars, 1);
}

#[test]
fn test_installed_module_new() {
    let installed = InstalledModule::new("io.univrs.test", "1.0.0");

    assert_eq!(installed.module_id, "io.univrs.test");
    assert_eq!(installed.version, "1.0.0");
    assert!(!installed.auto_update);
    assert_eq!(installed.update_history.len(), 0);
}

#[test]
fn test_installed_module_record_update() {
    let mut installed = InstalledModule::new("io.univrs.test", "1.0.0");

    installed.record_update("1.0.0".to_string(), "1.1.0".to_string(), true);

    assert_eq!(installed.version, "1.1.0");
    assert_eq!(installed.update_history.len(), 1);
    assert!(installed.update_history[0].success);
}

#[test]
fn test_module_add_version() {
    let mut module = GenModule::new(
        "io.univrs.versioning",
        "Version Test",
        "Testing versions",
        "did:key:alice",
        "MIT",
    );

    let version = ModuleVersion::new("1.0.0", "hash1", 100, "Initial", "sig1");
    module.add_version(version);

    assert_eq!(module.versions.len(), 1);
    assert_eq!(module.latest_version, "1.0.0");
}

#[test]
fn test_module_add_multiple_versions() {
    let mut module = GenModule::new(
        "io.univrs.multi",
        "Multi Version",
        "Testing",
        "did:key:alice",
        "MIT",
    );

    module.add_version(ModuleVersion::new("1.0.0", "h1", 100, "V1", "s1"));
    module.add_version(ModuleVersion::new("1.1.0", "h2", 100, "V2", "s2"));
    module.add_version(ModuleVersion::new("2.0.0", "h3", 100, "V3", "s3"));

    assert_eq!(module.versions.len(), 3);
    assert_eq!(module.latest_version, "2.0.0");
}

#[test]
fn test_module_validate_valid_ids() {
    let valid_ids = vec![
        "io.univrs.user",
        "com.example.auth",
        "org.company.database.postgres",
    ];

    for id in valid_ids {
        let module = GenModule::new(id, "Test", "Test", "did:key:test", "MIT");
        assert!(module.validate_id(), "ID {} should be valid", id);
    }
}

#[test]
fn test_module_validate_invalid_ids() {
    let invalid_ids = vec!["invalid", "io", "io.", ".io.test", "io..test"];

    for id in invalid_ids {
        let module = GenModule::new(id, "Test", "Test", "did:key:test", "MIT");
        assert!(!module.validate_id(), "ID {} should be invalid", id);
    }
}
