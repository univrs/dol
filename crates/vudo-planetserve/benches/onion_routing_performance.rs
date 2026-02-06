//! Benchmarks for onion routing performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use vudo_identity::MasterIdentity;
use vudo_p2p::{P2PConfig, VudoP2P};
use vudo_planetserve::config::OnionConfig;
use vudo_planetserve::onion::{OnionRouter, RelayNode};
use vudo_state::StateEngine;
use x25519_dalek::{PublicKey, StaticSecret};

fn create_relay(did: &str) -> RelayNode {
    let secret = StaticSecret::random_from_rng(&mut rand::thread_rng());
    let public_key = PublicKey::from(&secret);
    RelayNode::new(did.to_string(), public_key)
}

fn bench_circuit_building(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("onion_circuit_building");

    let hops = vec![1, 2, 3, 5];

    for hop_count in hops {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_hops", hop_count)),
            &hop_count,
            |b, &hops| {
                b.to_async(&rt).iter(|| async move {
                    let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
                    let state_engine = Arc::new(StateEngine::new().await.unwrap());
                    let p2p = Arc::new(
                        VudoP2P::new(state_engine, P2PConfig::default())
                            .await
                            .unwrap(),
                    );

                    let config = OnionConfig {
                        hops,
                        relay_selection_strategy: vudo_planetserve::config::RelaySelectionStrategy::Random,
                    };

                    let router = OnionRouter::with_config(identity, p2p, config);

                    // Add relays
                    for i in 0..10 {
                        router.add_relay(create_relay(&format!("relay{}", i)));
                    }

                    router
                        .build_circuit(black_box("did:peer:destination"), hops)
                        .await
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_onion_encryption(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("onion_encryption");

    let message_sizes = vec![
        256,    // 256B
        1024,   // 1KB
        10240,  // 10KB
    ];

    for size in message_sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}B", size)),
            &size,
            |b, &msg_size| {
                b.to_async(&rt).iter(|| async move {
                    let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
                    let state_engine = Arc::new(StateEngine::new().await.unwrap());
                    let p2p = Arc::new(
                        VudoP2P::new(state_engine, P2PConfig::default())
                            .await
                            .unwrap(),
                    );

                    let router = OnionRouter::new(identity, p2p);

                    // Add relays
                    for i in 0..5 {
                        router.add_relay(create_relay(&format!("relay{}", i)));
                    }

                    let circuit = router
                        .build_circuit("did:peer:destination", 3)
                        .await
                        .unwrap();

                    let message = vec![0u8; msg_size];

                    router
                        .send_onion(black_box(&circuit), black_box(&message))
                        .await
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_circuit_building, bench_onion_encryption);
criterion_main!(benches);
