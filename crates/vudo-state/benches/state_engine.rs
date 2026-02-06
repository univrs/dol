//! Benchmarks for the state engine.

use automerge::ROOT;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vudo_state::*;

fn benchmark_document_create(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("document_create", |b| {
        b.to_async(&runtime).iter(|| async {
            let engine = StateEngine::new().await.unwrap();
            let doc_id = DocumentId::new("users", "alice");
            black_box(engine.create_document(doc_id).await.unwrap());
        });
    });
}

fn benchmark_document_read(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("document_read", |b| {
        let engine = runtime.block_on(async {
            let engine = StateEngine::new().await.unwrap();
            let doc_id = DocumentId::new("users", "alice");
            let handle = engine.create_document(doc_id).await.unwrap();
            handle
                .update(|doc| {
                    doc.put(ROOT, "name", "Alice")?;
                    doc.put(ROOT, "age", 30i64)?;
                    Ok(())
                })
                .unwrap();
            engine
        });

        b.iter(|| {
            let doc_id = DocumentId::new("users", "alice");
            let handle = runtime.block_on(engine.get_document(&doc_id)).unwrap();
            handle
                .read(|doc| {
                    let _name: String = doc.get(ROOT, "name")?.unwrap().0.into();
                    let _age: i64 = doc.get(ROOT, "age")?.unwrap().0.into();
                    Ok(())
                })
                .unwrap();
        });
    });
}

fn benchmark_document_write(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("document_write", |b| {
        let (engine, handle) = runtime.block_on(async {
            let engine = StateEngine::new().await.unwrap();
            let doc_id = DocumentId::new("users", "alice");
            let handle = engine.create_document(doc_id).await.unwrap();
            (engine, handle)
        });

        let mut counter = 0i64;
        b.iter(|| {
            counter += 1;
            handle
                .update(|doc| {
                    doc.put(ROOT, "counter", black_box(counter))?;
                    Ok(())
                })
                .unwrap();
        });
    });
}

fn benchmark_subscription_notify(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("subscription_notify", |b| {
        let (engine, handle) = runtime.block_on(async {
            let engine = StateEngine::new().await.unwrap();
            let doc_id = DocumentId::new("users", "alice");
            let handle = engine.create_document(doc_id.clone()).await.unwrap();

            // Create subscriptions
            let filter = SubscriptionFilter::Document(doc_id);
            for _ in 0..10 {
                engine.subscribe(filter.clone()).await;
            }

            (engine, handle)
        });

        b.iter(|| {
            handle
                .update_reactive(&engine.observable, |doc| {
                    doc.put(ROOT, "value", black_box(42i64))?;
                    Ok(())
                })
                .unwrap();
            engine.observable.flush_batch();
        });
    });
}

fn benchmark_transaction_commit(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("transaction_commit", |b| {
        let engine = runtime.block_on(async {
            let engine = StateEngine::new().await.unwrap();
            for i in 0..10 {
                let doc_id = DocumentId::new("docs", &format!("doc{}", i));
                engine.create_document(doc_id).await.unwrap();
            }
            engine
        });

        b.iter(|| {
            let tx = engine.begin_transaction();
            for i in 0..10 {
                let doc_id = DocumentId::new("docs", &format!("doc{}", i));
                tx.update(&doc_id, |doc| {
                    doc.put(ROOT, "value", black_box(i as i64))?;
                    Ok(())
                })
                .unwrap();
            }
            engine.commit_transaction(tx).unwrap();
        });
    });
}

fn benchmark_snapshot_create(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("snapshot_create", |b| {
        let (engine, handle) = runtime.block_on(async {
            let engine = StateEngine::new().await.unwrap();
            let doc_id = DocumentId::new("users", "alice");
            let handle = engine.create_document(doc_id).await.unwrap();

            // Add some data
            for i in 0..100 {
                handle
                    .update(|doc| {
                        doc.put(ROOT, format!("key{}", i), i as i64)?;
                        Ok(())
                    })
                    .unwrap();
            }

            (engine, handle)
        });

        b.to_async(&runtime).iter(|| async {
            black_box(engine.snapshot(&handle).await.unwrap());
        });
    });
}

fn benchmark_operation_queue_enqueue(c: &mut Criterion) {
    c.bench_function("operation_queue_enqueue", |b| {
        let queue = OperationQueue::new();
        let mut counter = 0;

        b.iter(|| {
            counter += 1;
            let doc_id = DocumentId::new("users", &format!("user{}", counter));
            let op = Operation::new(OperationType::Create { document_id: doc_id });
            black_box(queue.enqueue(op).unwrap());
        });
    });
}

fn benchmark_throughput(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("throughput");

    for num_ops in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                b.to_async(&runtime).iter(|| async move {
                    let engine = StateEngine::new().await.unwrap();
                    let doc_id = DocumentId::new("users", "alice");
                    let handle = engine.create_document(doc_id).await.unwrap();

                    for i in 0..num_ops {
                        handle
                            .update(|doc| {
                                doc.put(ROOT, format!("key{}", i), i as i64)?;
                                Ok(())
                            })
                            .unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_document_create,
    benchmark_document_read,
    benchmark_document_write,
    benchmark_subscription_notify,
    benchmark_transaction_commit,
    benchmark_snapshot_create,
    benchmark_operation_queue_enqueue,
    benchmark_throughput,
);

criterion_main!(benches);
