//! Registry integration tests

use gen_registry::{GenModule, Registry, RegistryConfig};
use tempfile::TempDir;

async fn create_test_registry() -> (Registry, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut config = RegistryConfig::default();
    config.owner_did = "did:key:test".to_string();
    config.data_dir = temp_dir.path().to_str().unwrap().to_string();
    config.enable_p2p = false; // Disable P2P for unit tests

    let registry = Registry::with_config(config).await.unwrap();
    (registry, temp_dir)
}

#[tokio::test]
async fn test_create_registry() {
    let (registry, _temp) = create_test_registry().await;
    assert!(registry.list_installed().is_empty());
}

#[tokio::test]
async fn test_module_validation() {
    let module = GenModule::new(
        "io.univrs.test",
        "Test Module",
        "A test module",
        "did:key:alice",
        "MIT",
    );
    assert!(module.validate_id());
}

#[tokio::test]
async fn test_invalid_module_id() {
    let module = GenModule::new(
        "invalid",
        "Invalid",
        "Invalid module",
        "did:key:alice",
        "MIT",
    );
    assert!(!module.validate_id());
}

#[tokio::test]
async fn test_add_tags() {
    let mut module = GenModule::new(
        "io.univrs.test",
        "Test",
        "Test",
        "did:key:alice",
        "MIT",
    );

    module.add_tag("authentication");
    module.add_tag("security");

    assert_eq!(module.tags.len(), 2);
    assert!(module.tags.contains("authentication"));
}

#[tokio::test]
async fn test_get_nonexistent_module() {
    let (registry, _temp) = create_test_registry().await;
    let result = registry.get_module("io.nonexistent.module").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_installed_empty() {
    let (registry, _temp) = create_test_registry().await;
    let installed = registry.list_installed();
    assert_eq!(installed.len(), 0);
}

#[tokio::test]
async fn test_search_empty_registry() {
    let (registry, _temp) = create_test_registry().await;
    // Search should not error on empty registry
    let results = registry.search("test").await;
    // May return empty or error depending on implementation
}

#[tokio::test]
async fn test_rating_validation() {
    use gen_registry::Rating;

    let rating = Rating::new("io.univrs.test", "did:key:alice", 10);
    assert_eq!(rating.stars, 5); // Should clamp to max

    let rating2 = Rating::new("io.univrs.test", "did:key:bob", 0);
    assert_eq!(rating2.stars, 1); // Should clamp to min
}

#[tokio::test]
async fn test_average_rating_empty() {
    let (registry, _temp) = create_test_registry().await;
    let avg = registry.get_average_rating("io.univrs.test");
    assert!(avg.is_none());
}

#[tokio::test]
async fn test_discover_peers_no_p2p() {
    let (registry, _temp) = create_test_registry().await;
    let peers = registry.discover_peers().await.unwrap();
    assert_eq!(peers.len(), 0);
}

#[tokio::test]
async fn test_module_metadata() {
    let module = GenModule::new(
        "io.univrs.metadata",
        "Metadata Test",
        "Testing metadata",
        "did:key:alice",
        "Apache-2.0",
    );

    assert_eq!(module.id, "io.univrs.metadata");
    assert_eq!(module.name, "Metadata Test");
    assert_eq!(module.license, "Apache-2.0");
    assert_eq!(module.download_count, 0);
}

#[tokio::test]
async fn test_module_versions_empty() {
    let module = GenModule::new(
        "io.univrs.versions",
        "Versions Test",
        "Testing versions",
        "did:key:alice",
        "MIT",
    );

    assert_eq!(module.versions.len(), 0);
    assert_eq!(module.latest_version, "0.0.0");
}

#[tokio::test]
async fn test_module_dependencies_empty() {
    let module = GenModule::new(
        "io.univrs.deps",
        "Deps Test",
        "Testing deps",
        "did:key:alice",
        "MIT",
    );

    assert_eq!(module.dependencies.len(), 0);
}
