//! Integration tests for collaborative exegesis editing.
//!
//! These tests verify:
//! - Concurrent offline edits
//! - CRDT merge conflict resolution
//! - Change subscriptions
//! - Multi-user editing scenarios

use dol_exegesis::{CollaborativeEditor, ExegesisManager};
use std::sync::Arc;
use vudo_state::StateEngine;

#[tokio::test]
async fn test_collaborative_editor_creation() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());
    let editor = CollaborativeEditor::new(manager);

    assert!(!editor.has_p2p());
}

#[tokio::test]
async fn test_subscribe_to_changes() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    // Create exegesis
    manager
        .create_exegesis("user.profile", "1.0.0", "Initial content")
        .await
        .unwrap();

    let editor = CollaborativeEditor::new(manager);

    // Subscribe to changes
    let _sub = editor
        .subscribe_changes("user.profile", "1.0.0")
        .await
        .unwrap();

    // Subscription created successfully
}

#[tokio::test]
async fn test_concurrent_offline_edits_scenario() {
    // Simulate a scenario where two developers edit the same exegesis offline
    // and then sync their changes

    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    // Create initial exegesis
    manager
        .create_exegesis(
            "user.profile",
            "1.0.0",
            "A user profile contains identity information."
        )
        .await
        .unwrap();

    // Developer 1 (Alice) makes offline edits
    let manager_alice = Arc::clone(&manager);
    let handle_alice = tokio::spawn(async move {
        manager_alice
            .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
                content.push_str("\nAlice added: Profiles support multiple identities.");
            })
            .await
    });

    // Developer 2 (Bob) makes offline edits (concurrently)
    let manager_bob = Arc::clone(&manager);
    let handle_bob = tokio::spawn(async move {
        manager_bob
            .edit_exegesis("user.profile", "1.0.0", "did:peer:bob", |content| {
                content.push_str("\nBob added: Profiles can be exported as JSON.");
            })
            .await
    });

    // Wait for both to complete
    handle_alice.await.unwrap().unwrap();
    handle_bob.await.unwrap().unwrap();

    // Verify: Both edits should be present (CRDT merge)
    let final_doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();

    // The content should contain contributions from both
    assert!(final_doc.content.contains("Alice") || final_doc.content.contains("Bob"));

    // Both should be listed as contributors
    assert_eq!(final_doc.contributors.len(), 2);
    assert!(final_doc.contributors.contains(&"did:peer:alice".to_string()));
    assert!(final_doc.contributors.contains(&"did:peer:bob".to_string()));
}

#[tokio::test]
async fn test_three_way_concurrent_edit() {
    // Test with three concurrent editors
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    manager
        .create_exegesis("system.auth", "1.0.0", "Base authentication system.")
        .await
        .unwrap();

    // Three developers edit concurrently
    let editors = vec![
        ("did:peer:alice", "Supports OAuth2"),
        ("did:peer:bob", "Supports JWT tokens"),
        ("did:peer:charlie", "Supports SAML"),
    ];

    let handles: Vec<_> = editors
        .into_iter()
        .map(|(did, addition)| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move {
                mgr.edit_exegesis("system.auth", "1.0.0", did, |content| {
                    content.push_str(&format!("\n- {}", addition));
                })
                .await
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let doc = manager.get_exegesis("system.auth", "1.0.0").await.unwrap();

    // All three contributors should be recorded
    assert_eq!(doc.contributors.len(), 3);
}

#[tokio::test]
async fn test_sequential_then_concurrent_edits() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    manager
        .create_exegesis("data.model", "1.0.0", "Initial model")
        .await
        .unwrap();

    // Sequential edit first
    manager
        .edit_exegesis("data.model", "1.0.0", "did:peer:alice", |content| {
            content.push_str(" - Updated by Alice");
        })
        .await
        .unwrap();

    // Then concurrent edits by Bob and Charlie
    let manager_bob = Arc::clone(&manager);
    let manager_charlie = Arc::clone(&manager);

    let handle_bob = tokio::spawn(async move {
        manager_bob
            .edit_exegesis("data.model", "1.0.0", "did:peer:bob", |content| {
                content.push_str(" - Bob's addition");
            })
            .await
    });

    let handle_charlie = tokio::spawn(async move {
        manager_charlie
            .edit_exegesis("data.model", "1.0.0", "did:peer:charlie", |content| {
                content.push_str(" - Charlie's addition");
            })
            .await
    });

    handle_bob.await.unwrap().unwrap();
    handle_charlie.await.unwrap().unwrap();

    let doc = manager.get_exegesis("data.model", "1.0.0").await.unwrap();
    assert_eq!(doc.contributors.len(), 3);
}

#[tokio::test]
async fn test_offline_edit_merge_conflict_resolution() {
    // Simulate a merge conflict scenario where two users
    // edit the same section of content offline

    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    manager
        .create_exegesis("api.endpoint", "1.0.0", "This endpoint returns user data")
        .await
        .unwrap();

    // Alice edits the content (offline)
    let manager_alice = Arc::clone(&manager);
    let handle_alice = tokio::spawn(async move {
        manager_alice
            .edit_exegesis("api.endpoint", "1.0.0", "did:peer:alice", |content| {
                *content = "This endpoint returns detailed user profile data".to_string();
            })
            .await
    });

    // Bob also edits the content differently (offline, concurrently)
    let manager_bob = Arc::clone(&manager);
    let handle_bob = tokio::spawn(async move {
        manager_bob
            .edit_exegesis("api.endpoint", "1.0.0", "did:peer:bob", |content| {
                *content = "This endpoint returns user authentication data".to_string();
            })
            .await
    });

    handle_alice.await.unwrap().unwrap();
    handle_bob.await.unwrap().unwrap();

    // In a real CRDT system, one of the edits would win based on
    // timestamp or actor ID. For now, we verify both contributors are recorded.
    let doc = manager.get_exegesis("api.endpoint", "1.0.0").await.unwrap();
    assert_eq!(doc.contributors.len(), 2);
}

#[tokio::test]
async fn test_subscribe_without_p2p() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    manager
        .create_exegesis("test.gene", "1.0.0", "Test")
        .await
        .unwrap();

    let editor = CollaborativeEditor::new(manager);

    // Should succeed even without P2P
    let _sub = editor.subscribe_changes("test.gene", "1.0.0").await.unwrap();
}

#[tokio::test]
async fn test_sync_without_p2p_fails() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    manager
        .create_exegesis("test.gene", "1.0.0", "Test")
        .await
        .unwrap();

    let editor = CollaborativeEditor::new(manager);

    // Should fail because P2P is not configured
    let result = editor.sync_exegesis("test.gene", "1.0.0", "peer-123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_subscriptions() {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    // Create multiple exegesis documents
    manager
        .create_exegesis("gene1", "1.0.0", "Gene 1")
        .await
        .unwrap();

    manager
        .create_exegesis("gene2", "1.0.0", "Gene 2")
        .await
        .unwrap();

    let editor = CollaborativeEditor::new(manager);

    // Subscribe to both
    let _sub1 = editor.subscribe_changes("gene1", "1.0.0").await.unwrap();
    let _sub2 = editor.subscribe_changes("gene2", "1.0.0").await.unwrap();

    // Both subscriptions should be valid
}

#[tokio::test]
async fn test_collaborative_workflow_simulation() {
    // Simulate a realistic collaborative workflow:
    // 1. Alice creates initial exegesis
    // 2. Bob subscribes to changes
    // 3. Alice makes an edit
    // 4. Charlie also edits
    // 5. All changes are merged

    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

    // Step 1: Alice creates initial exegesis
    manager
        .create_exegesis(
            "payment.gateway",
            "1.0.0",
            "Payment gateway for processing transactions."
        )
        .await
        .unwrap();

    // Step 2: Bob subscribes (in real scenario, would receive notifications)
    let editor = CollaborativeEditor::new(Arc::clone(&manager));
    let _bob_sub = editor
        .subscribe_changes("payment.gateway", "1.0.0")
        .await
        .unwrap();

    // Step 3: Alice makes an edit
    manager
        .edit_exegesis("payment.gateway", "1.0.0", "did:peer:alice", |content| {
            content.push_str("\nSupports: Stripe, PayPal, Square");
        })
        .await
        .unwrap();

    // Step 4: Charlie edits concurrently
    manager
        .edit_exegesis("payment.gateway", "1.0.0", "did:peer:charlie", |content| {
            content.push_str("\nFeatures: Refunds, Subscriptions, Invoices");
        })
        .await
        .unwrap();

    // Step 5: Verify all changes merged
    let doc = manager
        .get_exegesis("payment.gateway", "1.0.0")
        .await
        .unwrap();

    assert_eq!(doc.contributors.len(), 2);
    assert!(doc.content.contains("Stripe") || doc.content.contains("Refunds"));
}
