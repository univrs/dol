//! Benchmarks for SQLite adapter performance.

use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vudo_storage::StorageAdapter;
use vudo_storage_native::SqliteAdapter;

async fn setup_adapter() -> SqliteAdapter {
    let adapter = SqliteAdapter::in_memory().await.unwrap();
    adapter.init().await.unwrap();
    adapter
}

fn bench_save(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("save_single_document", |b| {
        let adapter = runtime.block_on(setup_adapter());
        let data = Bytes::from("test document data");

        b.to_async(&runtime).iter(|| async {
            adapter
                .save("users", "alice", black_box(data.clone()))
                .await
                .unwrap();
        });
    });
}

fn bench_bulk_save(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    for size in [100, 1000, 10000].iter() {
        c.bench_with_input(BenchmarkId::new("bulk_save", size), size, |b, &size| {
            let adapter = runtime.block_on(setup_adapter());
            let data = Bytes::from("test document data");

            b.to_async(&runtime).iter(|| async {
                for i in 0..size {
                    adapter
                        .save("users", &format!("user{}", i), data.clone())
                        .await
                        .unwrap();
                }
            });
        });
    }
}

fn bench_load(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("load_document", |b| {
        let adapter = runtime.block_on(async {
            let adapter = setup_adapter().await;
            adapter
                .save("users", "alice", Bytes::from("test data"))
                .await
                .unwrap();
            adapter
        });

        b.to_async(&runtime).iter(|| async {
            black_box(adapter.load("users", "alice").await.unwrap());
        });
    });
}

fn bench_query(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("query_all", |b| {
        let adapter = runtime.block_on(async {
            let adapter = setup_adapter().await;
            for i in 0..100 {
                adapter
                    .save("users", &format!("user{}", i), Bytes::from("test data"))
                    .await
                    .unwrap();
            }
            adapter
        });

        b.to_async(&runtime).iter(|| async {
            black_box(
                adapter
                    .query("users", vudo_storage::QueryFilter::All)
                    .await
                    .unwrap(),
            );
        });
    });
}

criterion_group!(benches, bench_save, bench_bulk_save, bench_load, bench_query);
criterion_main!(benches);
