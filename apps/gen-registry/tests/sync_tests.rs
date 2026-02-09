//! P2P synchronization tests

use gen_registry::{sync::P2PSync, RegistryConfig};
use std::sync::Arc;
use vudo_state::StateEngine;

async fn create_test_sync() -> P2PSync {
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let mut config = RegistryConfig::default();
    config.owner_did = "did:key:test".to_string();

    P2PSync::new(state_engine, config).await.unwrap()
}

#[tokio::test]
async fn test_create_p2p_sync() {
    let _sync = create_test_sync().await;
}

#[tokio::test]
async fn test_discover_peers_empty() {
    let sync = create_test_sync().await;
    let peers = sync.discover_peers().await.unwrap();
    assert_eq!(peers.len(), 0);
}

#[tokio::test]
async fn test_get_progress_initial() {
    let sync = create_test_sync().await;
    let progress = sync.get_progress();

    assert_eq!(progress.peers_connected, 0);
    assert_eq!(progress.synced_modules, 0);
}

#[tokio::test]
async fn test_update_sync_state() {
    let sync = create_test_sync().await;

    sync.update_sync_state("peer1", 100, 200);

    let progress = sync.get_progress();
    assert_eq!(progress.peers_connected, 1);
}

#[tokio::test]
async fn test_update_sync_state_multiple() {
    let sync = create_test_sync().await;

    sync.update_sync_state("peer1", 100, 200);
    sync.update_sync_state("peer2", 150, 250);
    sync.update_sync_state("peer3", 200, 300);

    let progress = sync.get_progress();
    assert_eq!(progress.peers_connected, 3);
}

#[tokio::test]
async fn test_sync_nonexistent_module() {
    let sync = create_test_sync().await;
    let result = sync.sync_module("io.nonexistent.module").await;
    // Should not panic, may succeed or fail depending on implementation
}

#[tokio::test]
async fn test_fetch_nonexistent_module() {
    let sync = create_test_sync().await;
    let result = sync.fetch_module("io.nonexistent.module").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sync_ratings() {
    let sync = create_test_sync().await;
    let result = sync.sync_ratings("io.univrs.test").await;
    assert!(result.is_ok());
}
