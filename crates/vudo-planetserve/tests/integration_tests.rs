//! Integration tests for PlanetServe privacy-preserving sync

use std::sync::Arc;
use std::time::{Duration, Instant};
use vudo_identity::MasterIdentity;
use vudo_p2p::{P2PConfig, VudoP2P};
use vudo_planetserve::config::{PrivacyConfig, PrivacyLevel};
use vudo_planetserve::{PlanetServeAdapter, RelayNode};
use vudo_state::StateEngine;
use x25519_dalek::{PublicKey, StaticSecret};

async fn create_test_adapter(config: PrivacyConfig) -> PlanetServeAdapter {
    let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let p2p = Arc::new(VudoP2P::new(state_engine, P2PConfig::default()).await.unwrap());

    PlanetServeAdapter::new(identity, p2p, config)
        .await
        .unwrap()
}

fn create_relay(did: &str) -> RelayNode {
    let secret = StaticSecret::random_from_rng(&mut rand::thread_rng());
    let public_key = PublicKey::from(&secret);
    RelayNode::new(did.to_string(), public_key)
}

#[tokio::test]
async fn test_end_to_end_sync_no_privacy() {
    let adapter = create_test_adapter(PrivacyConfig::fast_open()).await;

    let data = b"Test message without privacy".to_vec();
    let result = adapter.sync_private("test_ns", "doc1", data).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_end_to_end_sync_basic_privacy() {
    let adapter = create_test_adapter(PrivacyConfig::basic()).await;

    let data = b"Test message with basic privacy".to_vec();
    let result = adapter.sync_private("test_ns", "doc1", data).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_end_to_end_sync_standard_privacy() {
    let adapter = create_test_adapter(PrivacyConfig::default()).await;

    let data = b"Test message with standard privacy".to_vec();
    let start = Instant::now();
    let result = adapter.sync_private("test_ns", "doc1", data).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // Timing jitter should add some delay (but not too much)
    assert!(elapsed < Duration::from_millis(200));
}

#[tokio::test]
async fn test_end_to_end_sync_maximum_privacy() {
    let adapter = create_test_adapter(PrivacyConfig::privacy_max()).await;

    // Add some test relays
    for i in 0..5 {
        adapter.add_relay(create_relay(&format!("relay{}", i)));
    }

    let data = b"Test message with maximum privacy".to_vec();
    let result = adapter.sync_private("test_ns", "doc1", data).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_privacy_level_escalation() {
    // Start with no privacy
    let adapter = create_test_adapter(PrivacyConfig::fast_open()).await;
    assert_eq!(adapter.config().level, PrivacyLevel::None);

    // Create new adapter with higher privacy
    let adapter = create_test_adapter(PrivacyConfig::basic()).await;
    assert_eq!(adapter.config().level, PrivacyLevel::Basic);

    let adapter = create_test_adapter(PrivacyConfig::default()).await;
    assert_eq!(adapter.config().level, PrivacyLevel::Standard);

    let adapter = create_test_adapter(PrivacyConfig::privacy_max()).await;
    assert_eq!(adapter.config().level, PrivacyLevel::Maximum);
}

#[tokio::test]
async fn test_sida_fragmentation_and_reconstruction() {
    use vudo_planetserve::config::SidaConfig;
    use vudo_planetserve::sida::SidaFragmenter;

    let config = SidaConfig { k: 3, n: 5 };
    let fragmenter = SidaFragmenter::new(config).unwrap();

    let original = b"This is a test message that will be fragmented".to_vec();

    // Fragment
    let fragments = fragmenter.fragment(&original).unwrap();
    assert_eq!(fragments.len(), 5);

    // Verify no single fragment reveals the message
    for fragment in &fragments {
        assert_ne!(fragment.data, original);
    }

    // Reconstruct from minimum fragments (3)
    let subset: Vec<_> = fragments.iter().take(3).cloned().collect();
    let reconstructed = fragmenter.reconstruct(subset).unwrap();
    assert_eq!(reconstructed, original);

    // Reconstruct from different subset
    let subset2: Vec<_> = vec![
        fragments[0].clone(),
        fragments[2].clone(),
        fragments[4].clone(),
    ];
    let reconstructed2 = fragmenter.reconstruct(subset2).unwrap();
    assert_eq!(reconstructed2, original);
}

#[tokio::test]
async fn test_onion_routing_latency() {
    let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let p2p = Arc::new(VudoP2P::new(state_engine, P2PConfig::default()).await.unwrap());

    let router = vudo_planetserve::onion::OnionRouter::new(Arc::clone(&identity), p2p);

    // Add relays
    for i in 0..5 {
        router.add_relay(create_relay(&format!("relay{}", i)));
    }

    // Build circuit
    let start = Instant::now();
    let circuit = router.build_circuit("did:peer:destination", 3).await;
    let elapsed = start.elapsed();

    assert!(circuit.is_ok());
    let circuit = circuit.unwrap();
    assert_eq!(circuit.hops(), 3);

    // Circuit build should be fast (< 500ms)
    assert!(elapsed < Duration::from_millis(500));
}

#[tokio::test]
async fn test_metadata_padding_size_distribution() {
    use vudo_planetserve::obfuscator::MetadataObfuscator;

    let config = PrivacyConfig {
        level: PrivacyLevel::Basic,
        padding_size: 1024,
        ..Default::default()
    };

    let obfuscator = MetadataObfuscator::new(config);

    // Test different message sizes
    let sizes = vec![1, 100, 500, 1000, 1500, 2000, 5000];

    for size in sizes {
        let message = vec![0u8; size];
        let padded = obfuscator.pad_message(&message);

        // All padded messages should be multiples of 1024
        assert_eq!(padded.len() % 1024, 0);

        // Padded size should be at least original size
        assert!(padded.len() >= message.len());
    }
}

#[tokio::test]
async fn test_timing_jitter_randomness() {
    use vudo_planetserve::obfuscator::MetadataObfuscator;

    let config = PrivacyConfig {
        level: PrivacyLevel::Standard,
        timing_jitter: 100,
        ..Default::default()
    };

    let obfuscator = MetadataObfuscator::new(config);

    // Collect timing samples
    let mut timings = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        obfuscator.apply_timing_jitter().await;
        let elapsed = start.elapsed();
        timings.push(elapsed);
    }

    // Check that timings vary (randomness)
    let all_same = timings.windows(2).all(|w| w[0] == w[1]);
    assert!(!all_same, "Timing jitter should be random");

    // Check that all timings are within bounds
    for timing in timings {
        assert!(timing < Duration::from_millis(150));
    }
}

#[tokio::test]
async fn test_cover_traffic_generation() {
    use vudo_planetserve::obfuscator::MetadataObfuscator;

    let config = PrivacyConfig {
        level: PrivacyLevel::Maximum,
        cover_traffic_rate: 60.0, // 60 messages/min = 1/sec
        padding_size: 1024,
        ..Default::default()
    };

    let obfuscator = MetadataObfuscator::new(config);

    // Start cover traffic
    let handle = obfuscator.start_cover_traffic();
    assert!(handle.is_some());

    let handle = handle.unwrap();

    // Wait for a bit to let cover traffic run
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stop cover traffic
    handle.stop();
}

#[tokio::test]
async fn test_no_single_peer_observes_full_message() {
    use vudo_planetserve::config::SidaConfig;
    use vudo_planetserve::sida::SidaFragmenter;

    let config = SidaConfig { k: 3, n: 5 };
    let fragmenter = SidaFragmenter::new(config).unwrap();

    let secret_message = b"This is a secret that no single peer should see";
    let fragments = fragmenter.fragment(secret_message).unwrap();

    // Simulate peers receiving fragments
    for (_i, fragment) in fragments.iter().enumerate() {
        // Each peer only sees their fragment
        let peer_data = &fragment.data;

        // Verify peer cannot reconstruct from single fragment
        // (We can't easily test this without implementing a search,
        // but we can verify the data doesn't match the original)
        assert_ne!(peer_data, secret_message);

        // Verify fragment is different from original
        assert!(peer_data.len() < secret_message.len() || peer_data != secret_message);
    }

    // Only when k fragments are combined can message be reconstructed
    let subset: Vec<_> = fragments.iter().take(3).cloned().collect();
    let reconstructed = fragmenter.reconstruct(subset).unwrap();
    assert_eq!(reconstructed, secret_message);
}

#[tokio::test]
async fn test_relay_selection_strategies() {
    use vudo_planetserve::config::{OnionConfig, RelaySelectionStrategy};
    use vudo_planetserve::onion::OnionRouter;

    let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let p2p = Arc::new(VudoP2P::new(state_engine, P2PConfig::default()).await.unwrap());

    let strategies = vec![
        RelaySelectionStrategy::Random,
        RelaySelectionStrategy::LowLatency,
        RelaySelectionStrategy::HighReliability,
        RelaySelectionStrategy::Balanced,
    ];

    for strategy in strategies {
        let config = OnionConfig {
            hops: 2,
            relay_selection_strategy: strategy,
        };

        let router = OnionRouter::with_config(Arc::clone(&identity), Arc::clone(&p2p), config);

        // Add relays with varying characteristics
        for i in 0..5 {
            let mut relay = create_relay(&format!("relay{}", i));
            relay.latency = Duration::from_millis(50 + i as u64 * 10);
            relay.reliability = 1.0 - (i as f64 * 0.1);
            router.add_relay(relay);
        }

        let circuit = router.build_circuit("did:peer:dest", 2).await;
        assert!(circuit.is_ok());
    }
}

#[tokio::test]
async fn test_adapter_start_stop_lifecycle() {
    let adapter = create_test_adapter(PrivacyConfig::privacy_max()).await;

    // Add relays (needed for privacy_max mode)
    for i in 0..5 {
        adapter.add_relay(create_relay(&format!("relay{}", i)));
    }

    // Start
    let result = adapter.start().await;
    assert!(result.is_ok());

    // Sync
    let data = b"test".to_vec();
    let result = adapter.sync_private("test", "doc1", data).await;
    assert!(result.is_ok());

    // Stop
    let result = adapter.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_privacy_overhead_comparison() {
    let configs = vec![
        ("None", PrivacyConfig::fast_open()),
        ("Basic", PrivacyConfig::basic()),
        ("Standard", PrivacyConfig::default()),
    ];

    for (name, config) in configs {
        let adapter = create_test_adapter(config).await;

        let data = b"test message".to_vec();
        let start = Instant::now();
        adapter.sync_private("test", "doc1", data).await.unwrap();
        let elapsed = start.elapsed();

        println!("{} privacy: {:?}", name, elapsed);

        // All should complete within reasonable time
        assert!(elapsed < Duration::from_secs(1));
    }
}

#[tokio::test]
async fn test_large_message_fragmentation() {
    use vudo_planetserve::config::SidaConfig;
    use vudo_planetserve::sida::SidaFragmenter;

    let config = SidaConfig { k: 3, n: 5 };
    let fragmenter = SidaFragmenter::new(config).unwrap();

    // Large message (1MB)
    let large_message: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

    let start = Instant::now();
    let fragments = fragmenter.fragment(&large_message).unwrap();
    let fragment_time = start.elapsed();

    assert_eq!(fragments.len(), 5);

    // Reconstruction
    let subset: Vec<_> = fragments.iter().take(3).cloned().collect();
    let start = Instant::now();
    let reconstructed = fragmenter.reconstruct(subset).unwrap();
    let reconstruct_time = start.elapsed();

    assert_eq!(reconstructed, large_message);

    println!(
        "Large message (1MB): fragment={:?}, reconstruct={:?}",
        fragment_time, reconstruct_time
    );

    // Should be reasonably fast (< 100ms for both operations)
    assert!(fragment_time < Duration::from_millis(100));
    assert!(reconstruct_time < Duration::from_millis(100));
}

#[tokio::test]
async fn test_relay_failure_handling() {
    let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
    let state_engine = Arc::new(StateEngine::new().await.unwrap());
    let p2p = Arc::new(VudoP2P::new(state_engine, P2PConfig::default()).await.unwrap());

    let router = vudo_planetserve::onion::OnionRouter::new(identity, p2p);

    // Add relays
    for i in 0..5 {
        let mut relay = create_relay(&format!("relay{}", i));
        // Make some relays unreliable
        if i < 2 {
            relay.reliability = 0.3;
        }
        router.add_relay(relay);
    }

    // Should still be able to build circuit despite some unreliable relays
    let circuit = router.build_circuit("did:peer:dest", 3).await;
    assert!(circuit.is_ok());
}
