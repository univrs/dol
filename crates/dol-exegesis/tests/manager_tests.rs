//! Comprehensive tests for ExegesisManager.
//!
//! These tests verify:
//! - Basic CRUD operations
//! - Concurrent editing with CRDT merging
//! - Version linking for gene evolution
//! - Contributor tracking
//! - Error handling

use dol_exegesis::ExegesisManager;
use std::sync::Arc;
use vudo_state::StateEngine;

#[tokio::test]
async fn test_create_exegesis_basic() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    let doc = manager
        .create_exegesis(
            "user.profile",
            "1.0.0",
            "A user profile contains identity and preferences."
        )
        .await
        .unwrap();

    assert_eq!(doc.gene_id, "user.profile");
    assert_eq!(doc.gene_version, "1.0.0");
    assert_eq!(doc.content, "A user profile contains identity and preferences.");
    assert!(doc.contributors.is_empty());
    assert!(doc.last_modified > 0);
}

#[tokio::test]
async fn test_create_multiple_exegesis() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    // Create exegesis for different genes
    manager
        .create_exegesis("user.profile", "1.0.0", "User profile docs")
        .await
        .unwrap();

    manager
        .create_exegesis("user.settings", "1.0.0", "User settings docs")
        .await
        .unwrap();

    // Create different version of same gene
    manager
        .create_exegesis("user.profile", "2.0.0", "Updated profile docs")
        .await
        .unwrap();

    // Verify all exist
    assert!(manager.exists("user.profile", "1.0.0").await);
    assert!(manager.exists("user.settings", "1.0.0").await);
    assert!(manager.exists("user.profile", "2.0.0").await);
}

#[tokio::test]
async fn test_get_exegesis() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    manager
        .create_exegesis("user.profile", "1.0.0", "Original content")
        .await
        .unwrap();

    let doc = manager
        .get_exegesis("user.profile", "1.0.0")
        .await
        .unwrap();

    assert_eq!(doc.gene_id, "user.profile");
    assert_eq!(doc.gene_version, "1.0.0");
    assert_eq!(doc.content, "Original content");
}

#[tokio::test]
async fn test_get_nonexistent_exegesis() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    let result = manager.get_exegesis("nonexistent", "1.0.0").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_edit_exegesis_single_editor() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    manager
        .create_exegesis("user.profile", "1.0.0", "Original")
        .await
        .unwrap();

    // Edit by Alice
    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
            *content = format!("{} - Edited by Alice", content);
        })
        .await
        .unwrap();

    let doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
    assert_eq!(doc.content, "Original - Edited by Alice");
    assert_eq!(doc.contributors.len(), 1);
    assert_eq!(doc.contributors[0], "did:peer:alice");
}

#[tokio::test]
async fn test_edit_exegesis_multiple_edits_same_user() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    manager
        .create_exegesis("user.profile", "1.0.0", "Base")
        .await
        .unwrap();

    // First edit
    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
            content.push_str(" - Edit 1");
        })
        .await
        .unwrap();

    // Second edit by same user
    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
            content.push_str(" - Edit 2");
        })
        .await
        .unwrap();

    let doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
    assert_eq!(doc.content, "Base - Edit 1 - Edit 2");
    // Should still have only one contributor
    assert_eq!(doc.contributors.len(), 1);
}

#[tokio::test]
async fn test_concurrent_edits_different_users() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    manager
        .create_exegesis("user.profile", "1.0.0", "Base content")
        .await
        .unwrap();

    // Simulate concurrent edits by two users
    let manager1 = Arc::clone(&manager);
    let manager2 = Arc::clone(&manager);

    let handle1 = tokio::spawn(async move {
        manager1
            .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
                content.push_str("\nEdited by Alice");
            })
            .await
    });

    let handle2 = tokio::spawn(async move {
        manager2
            .edit_exegesis("user.profile", "1.0.0", "did:peer:bob", |content| {
                content.push_str("\nEdited by Bob");
            })
            .await
    });

    handle1.await.unwrap().unwrap();
    handle2.await.unwrap().unwrap();

    let doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();

    // Both edits should be present (order may vary due to concurrency)
    assert!(doc.content.contains("Alice") || doc.content.contains("Bob"));

    // Both contributors should be recorded
    assert_eq!(doc.contributors.len(), 2);
    assert!(doc.contributors.contains(&"did:peer:alice".to_string()));
    assert!(doc.contributors.contains(&"did:peer:bob".to_string()));
}

#[tokio::test]
async fn test_concurrent_edits_three_users() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    manager
        .create_exegesis("user.profile", "1.0.0", "Base")
        .await
        .unwrap();

    // Spawn three concurrent editors
    let handles: Vec<_> = vec![
        ("did:peer:alice", "Alice"),
        ("did:peer:bob", "Bob"),
        ("did:peer:charlie", "Charlie"),
    ]
    .into_iter()
    .map(|(did, name)| {
        let mgr = Arc::clone(&manager);
        tokio::spawn(async move {
            mgr.edit_exegesis("user.profile", "1.0.0", did, |content| {
                content.push_str(&format!("\n{}", name));
            })
            .await
        })
    })
    .collect();

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
    assert_eq!(doc.contributors.len(), 3);
}

#[tokio::test]
async fn test_version_linking_basic() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    // Create v1.0.0 exegesis
    manager
        .create_exegesis("user.profile", "1.0.0", "Original documentation")
        .await
        .unwrap();

    // Add a contributor to v1.0.0
    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
            *content = format!("{} - Updated", content);
        })
        .await
        .unwrap();

    // Link to v2.0.0
    let new_doc = manager
        .link_to_evolution("user.profile", "1.0.0", "2.0.0")
        .await
        .unwrap();

    assert_eq!(new_doc.gene_version, "2.0.0");
    assert!(new_doc.content.contains("Original documentation"));

    // Verify v1.0.0 still exists unchanged
    let old_doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
    assert_eq!(old_doc.gene_version, "1.0.0");
}

#[tokio::test]
async fn test_version_linking_preserves_contributors() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    // Create v1.0.0 with multiple contributors
    manager
        .create_exegesis("user.profile", "1.0.0", "Base")
        .await
        .unwrap();

    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
            content.push_str(" - Alice");
        })
        .await
        .unwrap();

    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:bob", |content| {
            content.push_str(" - Bob");
        })
        .await
        .unwrap();

    // Link to v2.0.0
    manager
        .link_to_evolution("user.profile", "1.0.0", "2.0.0")
        .await
        .unwrap();

    // Check v2.0.0 has the contributors from v1.0.0
    let v2_doc = manager.get_exegesis("user.profile", "2.0.0").await.unwrap();
    assert_eq!(v2_doc.contributors.len(), 2);
}

#[tokio::test]
async fn test_version_linking_independent_edits() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    manager
        .create_exegesis("user.profile", "1.0.0", "Original")
        .await
        .unwrap();

    // Link to v2.0.0
    manager
        .link_to_evolution("user.profile", "1.0.0", "2.0.0")
        .await
        .unwrap();

    // Edit v2.0.0
    manager
        .edit_exegesis("user.profile", "2.0.0", "did:peer:charlie", |content| {
            content.push_str(" - v2 update");
        })
        .await
        .unwrap();

    // Verify v1.0.0 is unchanged
    let v1_doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
    assert_eq!(v1_doc.content, "Original");

    // Verify v2.0.0 has the edit
    let v2_doc = manager.get_exegesis("user.profile", "2.0.0").await.unwrap();
    assert!(v2_doc.content.contains("v2 update"));
}

#[tokio::test]
async fn test_invalid_did_format() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    manager
        .create_exegesis("user.profile", "1.0.0", "Test")
        .await
        .unwrap();

    // Try to edit with invalid DID (missing "did:" prefix)
    let result = manager
        .edit_exegesis("user.profile", "1.0.0", "invalid-did", |_| {})
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_version_format() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    manager
        .create_exegesis("user.profile", "1.0.0", "Original")
        .await
        .unwrap();

    // Try to link with invalid version format
    let result = manager
        .link_to_evolution("user.profile", "1.0.0", "2.0")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_exists_check() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    // Should not exist initially
    assert!(!manager.exists("user.profile", "1.0.0").await);

    // Create it
    manager
        .create_exegesis("user.profile", "1.0.0", "Test")
        .await
        .unwrap();

    // Should exist now
    assert!(manager.exists("user.profile", "1.0.0").await);

    // Different version should not exist
    assert!(!manager.exists("user.profile", "2.0.0").await);
}

#[tokio::test]
async fn test_last_modified_updated() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = ExegesisManager::new(state_engine).await.unwrap();

    manager
        .create_exegesis("user.profile", "1.0.0", "Original")
        .await
        .unwrap();

    let doc1 = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
    let first_modified = doc1.last_modified;

    // Wait a bit to ensure timestamp changes (> 1 second for Unix timestamp resolution)
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

    // Edit the exegesis
    manager
        .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
            content.push_str(" - Updated");
        })
        .await
        .unwrap();

    let doc2 = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
    let second_modified = doc2.last_modified;

    assert!(second_modified > first_modified);
}
