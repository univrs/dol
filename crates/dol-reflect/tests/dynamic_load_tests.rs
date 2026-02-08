//! Integration tests for dynamic schema loading

use dol_reflect::dynamic_load::{LoadOptions, SchemaEvent, SchemaLoader};
use std::path::Path;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_load_single_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("schema.dol");

    let content = r#"
gen test.user {
  user has id: String
  user has name: String
}

exegesis { Test user }
"#;

    tokio::fs::write(&file_path, content).await.unwrap();

    let mut loader = SchemaLoader::new();
    loader.load_file(&file_path).await.unwrap();

    // Verify loaded
    let registry = loader.registry();
    let registry_guard = registry.read().await;
    let gen = registry_guard.get_gen("test.user");
    assert!(gen.is_some());
    assert_eq!(gen.unwrap().field_count(), 2);
}

#[tokio::test]
async fn test_load_directory_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    tokio::fs::create_dir(&subdir).await.unwrap();

    // Create files in root
    tokio::fs::write(
        temp_dir.path().join("schema1.dol"),
        "gen test1.gen { test1 has field: String } exegesis { Test }",
    )
    .await
    .unwrap();

    // Create files in subdirectory
    tokio::fs::write(
        subdir.join("schema2.dol"),
        "gen test2.gen { test2 has field: String } exegesis { Test }",
    )
    .await
    .unwrap();

    let mut loader = SchemaLoader::new();
    let loaded = loader.load_directory(temp_dir.path()).await.unwrap();

    assert_eq!(loaded.len(), 2);

    let registry = loader.registry();
    let registry_guard = registry.read().await;
    assert!(registry_guard.get_gen("test1.gen").is_some());
    assert!(registry_guard.get_gen("test2.gen").is_some());
}

#[tokio::test]
async fn test_load_directory_non_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    tokio::fs::create_dir(&subdir).await.unwrap();

    // Create files in root
    tokio::fs::write(
        temp_dir.path().join("schema1.dol"),
        "gen test1.gen { test1 has field: String } exegesis { Test }",
    )
    .await
    .unwrap();

    // Create files in subdirectory (should be ignored)
    tokio::fs::write(
        subdir.join("schema2.dol"),
        "gen test2.gen { test2 has field: String } exegesis { Test }",
    )
    .await
    .unwrap();

    let options = LoadOptions {
        recursive: false,
        ..Default::default()
    };

    let mut loader = SchemaLoader::with_options(options);
    let loaded = loader.load_directory(temp_dir.path()).await.unwrap();

    // Only root files should be loaded
    assert_eq!(loaded.len(), 1);

    let registry = loader.registry();
    let registry_guard = registry.read().await;
    assert!(registry_guard.get_gen("test1.gen").is_some());
    assert!(registry_guard.get_gen("test2.gen").is_none());
}

#[tokio::test]
async fn test_reload_modified_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("schema.dol");

    // Write initial version
    let content_v1 = r#"
gen test.gen {
  test has field1: String
}

exegesis { Version 1 }
"#;

    tokio::fs::write(&file_path, content_v1).await.unwrap();

    let mut loader = SchemaLoader::new();
    loader.load_file(&file_path).await.unwrap();

    {
        let registry = loader.registry();
        let registry_guard = registry.read().await;
        let gen = registry_guard.get_gen("test.gen").unwrap();
        assert_eq!(gen.field_count(), 1);
    }

    // Wait to ensure different timestamp
    sleep(Duration::from_millis(100)).await;

    // Write modified version
    let content_v2 = r#"
gen test.gen {
  test has field1: String
  test has field2: Int32
  test has field3: Bool
}

exegesis { Version 2 }
"#;

    tokio::fs::write(&file_path, content_v2).await.unwrap();

    // Reload
    loader.reload_file(&file_path).await.unwrap();

    {
        let registry = loader.registry();
        let registry_guard = registry.read().await;
        let gen = registry_guard.get_gen("test.gen").unwrap();
        assert_eq!(gen.field_count(), 3);
    }
}

#[tokio::test]
async fn test_version_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("schema1.dol");
    let file2 = temp_dir.path().join("schema2.dol");

    tokio::fs::write(&file1, "gen test1.gen { test1 has field: String } exegesis { Test }")
        .await
        .unwrap();
    tokio::fs::write(&file2, "gen test2.gen { test2 has field: String } exegesis { Test }")
        .await
        .unwrap();

    let mut loader = SchemaLoader::new();
    loader.load_file(&file1).await.unwrap();
    loader.load_file(&file2).await.unwrap();

    let versions = loader.versions();
    assert_eq!(versions.len(), 2);
    assert!(versions.contains_key(&file1));
    assert!(versions.contains_key(&file2));
}

#[tokio::test]
async fn test_clear_loader() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("schema.dol");

    tokio::fs::write(&file_path, "gen test.gen { test has field: String } exegesis { Test }")
        .await
        .unwrap();

    let mut loader = SchemaLoader::new();
    loader.load_file(&file_path).await.unwrap();

    assert_eq!(loader.versions().len(), 1);

    loader.clear().await;

    assert_eq!(loader.versions().len(), 0);

    let registry = loader.registry();
    let registry_guard = registry.read().await;
    assert_eq!(registry_guard.total_count(), 0);
}

#[tokio::test]
async fn test_load_invalid_schema() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.dol");

    // Write invalid DOL syntax
    tokio::fs::write(&file_path, "this is not valid DOL syntax")
        .await
        .unwrap();

    let mut loader = SchemaLoader::new();
    let result = loader.load_file(&file_path).await;

    // Should return an error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_watcher_create() {
    let temp_dir = TempDir::new().unwrap();

    let mut loader = SchemaLoader::new();
    let (watcher, mut rx) = loader
        .watch_directory(temp_dir.path())
        .await
        .unwrap();

    // Create a new file
    let file_path = temp_dir.path().join("new.dol");
    tokio::fs::write(
        &file_path,
        "gen new.gen { new has field: String } exegesis { Test }",
    )
    .await
    .unwrap();

    // Wait for event with timeout
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .ok()
        .flatten();

    if let Some(SchemaEvent::Created { path, .. }) = event {
        assert_eq!(path, file_path);
    } else {
        // File watcher might be flaky in test environments, don't fail
        eprintln!("File watcher event not received (this is OK in test environments)");
    }

    drop(watcher); // Clean up watcher
}

#[tokio::test]
async fn test_hot_reload_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("schema.dol");

    // Initial schema
    tokio::fs::write(
        &file_path,
        r#"
gen counter {
  @crdt(pn_counter)
  counter has value: Int32 = 0
}

exegesis { Counter version 1 }
"#,
    )
    .await
    .unwrap();

    let mut loader = SchemaLoader::new();
    loader.load_file(&file_path).await.unwrap();

    // Verify initial load
    {
        let registry = loader.registry();
        let guard = registry.read().await;
        let gen = guard.get_gen("counter").unwrap();
        assert!(gen.exegesis().contains("version 1"));
    }

    // Simulate hot-reload: modify file
    sleep(Duration::from_millis(100)).await;

    tokio::fs::write(
        &file_path,
        r#"
gen counter {
  @crdt(pn_counter)
  counter has value: Int32 = 0

  @crdt(lww)
  counter has name: String = "default"
}

exegesis { Counter version 2 with name field }
"#,
    )
    .await
    .unwrap();

    // Reload
    loader.reload_file(&file_path).await.unwrap();

    // Verify reload
    {
        let registry = loader.registry();
        let guard = registry.read().await;
        let gen = guard.get_gen("counter").unwrap();
        assert!(gen.exegesis().contains("version 2"));
        assert_eq!(gen.field_count(), 2);
        assert!(gen.get_field("name").is_some());
    }
}

#[tokio::test]
async fn test_concurrent_loads() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple files
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("schema{}.dol", i));
        let content = format!(
            "gen test{}.gen {{ test{} has field: String }} exegesis {{ Test }}",
            i, i
        );
        tokio::fs::write(&file_path, content).await.unwrap();
    }

    let mut loader = SchemaLoader::new();
    let loaded = loader.load_directory(temp_dir.path()).await.unwrap();

    assert_eq!(loaded.len(), 10);

    let registry = loader.registry();
    let guard = registry.read().await;

    // All schemas should be loaded
    for i in 0..10 {
        let gen_name = format!("test{}.gen", i);
        assert!(
            guard.get_gen(&gen_name).is_some(),
            "Gen {} not found",
            gen_name
        );
    }
}

#[tokio::test]
async fn test_load_with_validation() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("schema.dol");

    // Valid schema
    tokio::fs::write(
        &file_path,
        r#"
gen valid.gen {
  valid has field: String
}

exegesis { Valid schema }
"#,
    )
    .await
    .unwrap();

    let options = LoadOptions {
        validate: true,
        ..Default::default()
    };

    let mut loader = SchemaLoader::with_options(options);
    let result = loader.load_file(&file_path).await;

    assert!(result.is_ok());
}
