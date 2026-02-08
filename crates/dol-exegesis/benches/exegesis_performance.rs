//! Performance benchmarks for dol-exegesis.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dol_exegesis::ExegesisManager;
use std::sync::Arc;
use vudo_state::StateEngine;

fn bench_create_exegesis(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("create_exegesis", |b| {
        b.to_async(&runtime).iter(|| async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());
            let manager = ExegesisManager::new(state_engine).await.unwrap();

            manager
                .create_exegesis(
                    black_box("benchmark.gene"),
                    black_box("1.0.0"),
                    black_box("Benchmark exegesis content"),
                )
                .await
                .unwrap();
        });
    });
}

fn bench_edit_exegesis(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("edit_exegesis", |b| {
        b.to_async(&runtime).iter(|| async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());
            let manager = ExegesisManager::new(state_engine).await.unwrap();

            manager
                .create_exegesis("benchmark.gene", "1.0.0", "Initial")
                .await
                .unwrap();

            manager
                .edit_exegesis(
                    black_box("benchmark.gene"),
                    black_box("1.0.0"),
                    black_box("did:peer:bench"),
                    |content| {
                        content.push_str(" - Edited");
                    },
                )
                .await
                .unwrap();
        });
    });
}

fn bench_get_exegesis(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("get_exegesis", |b| {
        b.to_async(&runtime).iter(|| async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());
            let manager = ExegesisManager::new(state_engine).await.unwrap();

            manager
                .create_exegesis("benchmark.gene", "1.0.0", "Content")
                .await
                .unwrap();

            manager
                .get_exegesis(black_box("benchmark.gene"), black_box("1.0.0"))
                .await
                .unwrap();
        });
    });
}

fn bench_concurrent_edits(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent_edits_3_users", |b| {
        b.to_async(&runtime).iter(|| async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());
            let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

            manager
                .create_exegesis("benchmark.gene", "1.0.0", "Base")
                .await
                .unwrap();

            let handles: Vec<_> = vec![
                ("did:peer:alice", "Alice"),
                ("did:peer:bob", "Bob"),
                ("did:peer:charlie", "Charlie"),
            ]
            .into_iter()
            .map(|(did, name)| {
                let mgr = Arc::clone(&manager);
                tokio::spawn(async move {
                    mgr.edit_exegesis("benchmark.gene", "1.0.0", did, |content| {
                        content.push_str(&format!(" {}", name));
                    })
                    .await
                })
            })
            .collect();

            for handle in handles {
                handle.await.unwrap().unwrap();
            }
        });
    });
}

criterion_group!(
    benches,
    bench_create_exegesis,
    bench_edit_exegesis,
    bench_get_exegesis,
    bench_concurrent_edits
);
criterion_main!(benches);
