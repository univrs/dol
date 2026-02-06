//! Comprehensive sync performance benchmarks for vudo-p2p
//!
//! Tests sync throughput, latency, and merge performance against budget targets:
//! - Sync throughput: > 1000 ops/sec sustained
//! - CRDT merge latency: < 10ms for 10K operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use vudo_p2p::{P2PNode, SyncMessage};
use vudo_state::{DocumentId, StateEngine};

/// Benchmark sync throughput - target: > 1000 ops/sec
fn bench_sync_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("sync_throughput");

    for num_ops in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*num_ops as u64));

        let (node_a, node_b) = rt.block_on(async {
            let state_a = Arc::new(StateEngine::new().await.unwrap());
            let state_b = Arc::new(StateEngine::new().await.unwrap());

            let node_a = P2PNode::new(state_a).await.unwrap();
            let node_b = P2PNode::new(state_b).await.unwrap();

            (node_a, node_b)
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                b.to_async(&rt).iter(|| async {
                    // Create document with operations
                    let doc_id = DocumentId::new("docs", "test");
                    let handle_a = node_a.state_engine.create_document(doc_id.clone()).await.unwrap();

                    // Generate operations
                    let start = Instant::now();
                    for i in 0..num_ops {
                        handle_a.update(|doc| {
                            doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                            Ok(())
                        }).unwrap();
                    }

                    // Sync to peer
                    node_a.sync_document(&doc_id, &node_b).await.unwrap();
                    let elapsed = start.elapsed();

                    // Verify throughput budget (> 1000 ops/sec)
                    let throughput = num_ops as f64 / elapsed.as_secs_f64();
                    if num_ops >= 1000 {
                        assert!(
                            throughput > 1000.0,
                            "Sync throughput {} ops/sec below 1000 ops/sec budget for {} operations",
                            throughput,
                            num_ops
                        );
                    }

                    black_box(elapsed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark CRDT merge latency - target: < 10ms for 10K operations
fn bench_merge_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("merge_latency");

    for num_ops in [1000, 5000, 10000, 20000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                b.to_async(&rt).iter(|| async {
                    let state_a = Arc::new(StateEngine::new().await.unwrap());
                    let state_b = Arc::new(StateEngine::new().await.unwrap());

                    // Create two diverged documents
                    let doc_id = DocumentId::new("docs", "test");
                    let handle_a = state_a.create_document(doc_id.clone()).await.unwrap();
                    let handle_b = state_b.create_document(doc_id.clone()).await.unwrap();

                    // Add operations to both (simulating concurrent edits)
                    for i in 0..num_ops/2 {
                        handle_a.update(|doc| {
                            doc.put(automerge::ROOT, format!("key_a_{}", i), i as i64)?;
                            Ok(())
                        }).unwrap();

                        handle_b.update(|doc| {
                            doc.put(automerge::ROOT, format!("key_b_{}", i), i as i64)?;
                            Ok(())
                        }).unwrap();
                    }

                    // Measure merge time
                    let start = Instant::now();

                    // Get changes from both documents
                    let changes_a = handle_a.read(|doc| Ok(doc.get_changes(&[]))).unwrap();
                    let changes_b = handle_b.read(|doc| Ok(doc.get_changes(&[]))).unwrap();

                    // Apply changes from B to A (merge)
                    handle_a.update(|doc| {
                        doc.apply_changes(changes_b)?;
                        Ok(())
                    }).unwrap();

                    let elapsed = start.elapsed();

                    // Verify merge latency budget (< 10ms for 10K ops)
                    if num_ops >= 10000 {
                        assert!(
                            elapsed < Duration::from_millis(10),
                            "Merge took {:?}, exceeds 10ms budget for {} operations",
                            elapsed,
                            num_ops
                        );
                    }

                    black_box(elapsed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark delta sync (only send changed operations)
fn bench_delta_sync(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let (node_a, node_b) = rt.block_on(async {
        let state_a = Arc::new(StateEngine::new().await.unwrap());
        let state_b = Arc::new(StateEngine::new().await.unwrap());

        let node_a = P2PNode::new(state_a).await.unwrap();
        let node_b = P2PNode::new(state_b).await.unwrap();

        // Initial sync
        let doc_id = DocumentId::new("docs", "test");
        let handle_a = node_a.state_engine.create_document(doc_id.clone()).await.unwrap();

        for i in 0..1000 {
            handle_a.update(|doc| {
                doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                Ok(())
            }).unwrap();
        }

        node_a.sync_document(&doc_id, &node_b).await.unwrap();

        (node_a, node_b)
    });

    c.bench_function("delta_sync", |b| {
        b.to_async(&rt).iter(|| async {
            let doc_id = DocumentId::new("docs", "test");
            let handle_a = node_a.state_engine.get_document(&doc_id).await.unwrap();

            // Make small change
            handle_a.update(|doc| {
                doc.put(automerge::ROOT, "new_key", 42i64)?;
                Ok(())
            }).unwrap();

            // Sync only delta
            black_box(node_a.sync_document(&doc_id, &node_b).await.unwrap());
        });
    });
}

/// Benchmark parallel sync (multiple documents concurrently)
fn bench_parallel_sync(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("parallel_sync");

    for num_docs in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_docs),
            num_docs,
            |b, &num_docs| {
                b.to_async(&rt).iter(|| async {
                    let state_a = Arc::new(StateEngine::new().await.unwrap());
                    let state_b = Arc::new(StateEngine::new().await.unwrap());

                    let node_a = P2PNode::new(Arc::clone(&state_a)).await.unwrap();
                    let node_b = P2PNode::new(Arc::clone(&state_b)).await.unwrap();

                    // Create multiple documents
                    let mut doc_ids = Vec::new();
                    for i in 0..num_docs {
                        let doc_id = DocumentId::new("docs", &format!("doc{}", i));
                        let handle = state_a.create_document(doc_id.clone()).await.unwrap();

                        // Add some data
                        handle.update(|doc| {
                            doc.put(automerge::ROOT, "value", i as i64)?;
                            Ok(())
                        }).unwrap();

                        doc_ids.push(doc_id);
                    }

                    // Sync all documents in parallel
                    let mut handles = Vec::new();
                    for doc_id in doc_ids {
                        let node_a = node_a.clone();
                        let node_b = node_b.clone();
                        handles.push(tokio::spawn(async move {
                            node_a.sync_document(&doc_id, &node_b).await
                        }));
                    }

                    black_box(futures::future::join_all(handles).await);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark sync message compression
fn bench_message_compression(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("message_compression", |b| {
        b.to_async(&rt).iter(|| async {
            let state = Arc::new(StateEngine::new().await.unwrap());
            let doc_id = DocumentId::new("docs", "test");
            let handle = state.create_document(doc_id.clone()).await.unwrap();

            // Create large document
            for i in 0..1000 {
                handle.update(|doc| {
                    doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                }).unwrap();
            }

            // Get changes and compress
            let changes = handle.read(|doc| Ok(doc.get_changes(&[]))).unwrap();
            let message = SyncMessage::new(doc_id.clone(), changes);

            black_box(message.compress().unwrap())
        });
    });
}

/// Benchmark operation deduplication (using bloom filter)
fn bench_operation_deduplication(c: &mut Criterion) {
    use vudo_p2p::BloomFilter;

    let mut group = c.benchmark_group("operation_deduplication");

    for num_ops in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                b.iter(|| {
                    let bloom = BloomFilter::new(num_ops, 0.01);

                    for i in 0..num_ops {
                        let op_id = format!("op_{}", i);
                        if !bloom.contains(&op_id) {
                            bloom.insert(&op_id);
                        }
                    }

                    black_box(bloom)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark incremental sync (track last sync vector clock)
fn bench_incremental_sync(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("incremental_sync", |b| {
        b.to_async(&rt).iter(|| async {
            let state_a = Arc::new(StateEngine::new().await.unwrap());
            let state_b = Arc::new(StateEngine::new().await.unwrap());

            let node_a = P2PNode::new(Arc::clone(&state_a)).await.unwrap();
            let node_b = P2PNode::new(Arc::clone(&state_b)).await.unwrap();

            let doc_id = DocumentId::new("docs", "test");
            let handle_a = state_a.create_document(doc_id.clone()).await.unwrap();

            // Initial sync
            for i in 0..100 {
                handle_a.update(|doc| {
                    doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                }).unwrap();
            }
            node_a.sync_document(&doc_id, &node_b).await.unwrap();

            // Add more operations
            for i in 100..200 {
                handle_a.update(|doc| {
                    doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                }).unwrap();
            }

            // Only sync new operations (incremental)
            black_box(node_a.sync_document_incremental(&doc_id, &node_b).await.unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_sync_throughput,
    bench_merge_latency,
    bench_delta_sync,
    bench_parallel_sync,
    bench_message_compression,
    bench_operation_deduplication,
    bench_incremental_sync,
);

criterion_main!(benches);
