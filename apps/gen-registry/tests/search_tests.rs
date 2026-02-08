//! Search engine tests

use gen_registry::{models::SearchIndex, search::{SearchEngine, SearchQuery}, GenModule};
use tempfile::TempDir;

async fn create_test_search_engine() -> (SearchEngine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let engine = SearchEngine::new(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();
    (engine, temp_dir)
}

#[tokio::test]
async fn test_create_search_engine() {
    let (_engine, _temp) = create_test_search_engine().await;
}

#[tokio::test]
async fn test_search_query_new() {
    let query = SearchQuery::new("test");
    assert_eq!(query.text, "test");
    assert_eq!(query.limit, 20);
}

#[tokio::test]
async fn test_search_query_with_limit() {
    let query = SearchQuery::new("test").with_limit(50);
    assert_eq!(query.limit, 50);
}

#[tokio::test]
async fn test_index_module() {
    let (engine, _temp) = create_test_search_engine().await;

    let module = GenModule::new(
        "io.univrs.test",
        "Test Module",
        "A test module for search",
        "did:key:alice",
        "MIT",
    );

    let index = SearchIndex::new(&module);
    assert!(engine.index_module(&index).await.is_ok());
}

#[tokio::test]
async fn test_search_empty() {
    let (engine, _temp) = create_test_search_engine().await;

    let query = SearchQuery::new("nonexistent");
    let results = engine.search(&query).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_search_indexed_module() {
    let (engine, _temp) = create_test_search_engine().await;

    let mut module = GenModule::new(
        "io.univrs.auth",
        "Authentication",
        "User authentication module",
        "did:key:alice",
        "MIT",
    );
    module.add_tag("authentication");
    module.add_tag("security");

    let index = SearchIndex::new(&module);
    engine.index_module(&index).await.unwrap();

    let query = SearchQuery::new("authentication");
    let results = engine.search(&query).await.unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_search_index_keywords() {
    let module = GenModule::new(
        "io.univrs.database",
        "Database Access",
        "PostgreSQL database connector",
        "did:key:alice",
        "MIT",
    );

    let index = SearchIndex::new(&module);
    assert!(index.keywords.contains("database"));
    assert!(index.keywords.contains("access"));
    assert!(index.keywords.contains("postgresql"));
}

#[tokio::test]
async fn test_search_index_with_tags() {
    let mut module = GenModule::new(
        "io.univrs.crypto",
        "Cryptography",
        "Encryption utilities",
        "did:key:alice",
        "MIT",
    );
    module.add_tag("encryption");
    module.add_tag("security");

    let index = SearchIndex::new(&module);
    assert!(index.keywords.contains("encryption"));
    assert!(index.keywords.contains("security"));
}

#[tokio::test]
async fn test_search_limit() {
    let (engine, _temp) = create_test_search_engine().await;

    // Index multiple modules
    for i in 0..30 {
        let module = GenModule::new(
            format!("io.univrs.test{}", i),
            format!("Test {}", i),
            "Test module",
            "did:key:alice",
            "MIT",
        );
        let index = SearchIndex::new(&module);
        engine.index_module(&index).await.unwrap();
    }

    let query = SearchQuery::new("test").with_limit(10);
    let results = engine.search(&query).await.unwrap();
    assert!(results.len() <= 10);
}
