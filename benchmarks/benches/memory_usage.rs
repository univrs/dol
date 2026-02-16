//! Memory usage benchmarks
//!
//! Measures memory consumption and verifies budget: < 50MB for 100K-record dataset

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;
use vudo_state::{DocumentId, StateEngine};

const MEMORY_BUDGET_MB: usize = 50;

/// Helper to get current memory usage (platform-specific)
#[cfg(target_os = "linux")]
fn get_memory_usage() -> usize {
    use std::fs;

    let status = fs::read_to_string("/proc/self/status").unwrap();
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse::<usize>().unwrap() * 1024; // Convert KB to bytes
            }
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
fn get_memory_usage() -> usize {
    // Fallback for non-Linux platforms
    // This is a rough estimate
    0
}

/// Benchmark memory usage for 100K records - target: < 50MB
fn bench_100k_records(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memory/100k_records", |b| {
        b.to_async(&rt).iter(|| async {
            let baseline_memory = get_memory_usage();

            let state_engine = Arc::new(StateEngine::new().await.unwrap());

            // Create 100K records
            let mut handles = Vec::new();
            for i in 0..100_000 {
                let doc_id = DocumentId::new("records", &format!("record{}", i));
                let handle = state_engine.create_document(doc_id).await.unwrap();

                // Add some data to each record
                handle
                    .update(|doc| {
                        doc.put(automerge::ROOT, "id", i as i64)?;
                        doc.put(automerge::ROOT, "name", format!("Record {}", i))?;
                        doc.put(automerge::ROOT, "value", i as i64 * 2)?;
                        Ok(())
                    })
                    .unwrap();

                handles.push(handle);
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);
            let delta_mb = delta_bytes / (1024 * 1024);

            // Verify budget
            #[cfg(target_os = "linux")]
            assert!(
                delta_mb < MEMORY_BUDGET_MB,
                "Memory usage {} MB exceeds {} MB budget",
                delta_mb,
                MEMORY_BUDGET_MB
            );

            black_box((delta_mb, handles))
        });
    });
}

/// Benchmark memory usage with varying dataset sizes
fn bench_memory_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory/scaling");

    for num_records in [1000, 10_000, 50_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_records),
            num_records,
            |b, &num_records| {
                b.to_async(&rt).iter(|| async {
                    let baseline_memory = get_memory_usage();

                    let state_engine = Arc::new(StateEngine::new().await.unwrap());

                    // Create records
                    for i in 0..num_records {
                        let doc_id = DocumentId::new("records", &format!("record{}", i));
                        let handle = state_engine.create_document(doc_id).await.unwrap();

                        handle
                            .update(|doc| {
                                doc.put(automerge::ROOT, "id", i as i64)?;
                                doc.put(automerge::ROOT, "data", format!("Data {}", i))?;
                                Ok(())
                            })
                            .unwrap();
                    }

                    let peak_memory = get_memory_usage();
                    let delta_bytes = peak_memory.saturating_sub(baseline_memory);
                    let delta_mb = delta_bytes / (1024 * 1024);

                    // Verify budget for 100K records
                    #[cfg(target_os = "linux")]
                    if num_records >= 100_000 {
                        assert!(
                            delta_mb < MEMORY_BUDGET_MB,
                            "Memory usage {} MB exceeds {} MB budget for {} records",
                            delta_mb,
                            MEMORY_BUDGET_MB,
                            num_records
                        );
                    }

                    black_box(delta_mb)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage with string interning
fn bench_string_interning(c: &mut Criterion) {
    use string_cache::DefaultAtom;

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory/string_interning");

    // Without interning
    group.bench_function("no_interning", |b| {
        b.iter(|| {
            let baseline_memory = get_memory_usage();

            // Create 10K strings with many duplicates
            let mut strings = Vec::new();
            for i in 0..10_000 {
                let s = format!("string_{}", i % 100); // Only 100 unique strings
                strings.push(s);
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box((delta_bytes, strings))
        });
    });

    // With interning
    group.bench_function("with_interning", |b| {
        b.iter(|| {
            let baseline_memory = get_memory_usage();

            // Create 10K atoms with many duplicates
            let mut atoms = Vec::new();
            for i in 0..10_000 {
                let s = format!("string_{}", i % 100); // Only 100 unique strings
                atoms.push(DefaultAtom::from(s));
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box((delta_bytes, atoms))
        });
    });

    group.finish();
}

/// Benchmark memory usage with compact data structures
fn bench_compact_structures(c: &mut Criterion) {
    use smallvec::SmallVec;

    let mut group = c.benchmark_group("memory/compact_structures");

    // Regular Vec
    group.bench_function("vec", |b| {
        b.iter(|| {
            let baseline_memory = get_memory_usage();

            let mut vecs = Vec::new();
            for _ in 0..10_000 {
                let v: Vec<i32> = vec![1, 2, 3]; // Small vec
                vecs.push(v);
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box((delta_bytes, vecs))
        });
    });

    // SmallVec (stack-allocated for small sizes)
    group.bench_function("smallvec", |b| {
        b.iter(|| {
            let baseline_memory = get_memory_usage();

            let mut vecs = Vec::new();
            for _ in 0..10_000 {
                let mut v: SmallVec<[i32; 4]> = SmallVec::new();
                v.push(1);
                v.push(2);
                v.push(3);
                vecs.push(v);
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box((delta_bytes, vecs))
        });
    });

    group.finish();
}

/// Benchmark memory pooling impact
fn bench_memory_pooling(c: &mut Criterion) {
    use typed_arena::Arena;

    let mut group = c.benchmark_group("memory/pooling");

    // Without pooling (allocate/free repeatedly)
    group.bench_function("no_pooling", |b| {
        b.iter(|| {
            let baseline_memory = get_memory_usage();

            let mut allocations = Vec::new();
            for i in 0..1000 {
                let boxed = Box::new(i);
                allocations.push(boxed);
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box((delta_bytes, allocations))
        });
    });

    // With pooling (arena allocation)
    group.bench_function("with_pooling", |b| {
        b.iter(|| {
            let baseline_memory = get_memory_usage();

            let arena = Arena::new();
            let mut refs = Vec::new();
            for i in 0..1000 {
                let r = arena.alloc(i);
                refs.push(r);
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box((delta_bytes, refs))
        });
    });

    group.finish();
}

/// Benchmark lazy materialization impact
fn bench_lazy_materialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory/lazy_materialization");

    // Eager materialization (compute document state immediately)
    group.bench_function("eager", |b| {
        b.to_async(&rt).iter(|| async {
            let baseline_memory = get_memory_usage();

            let state_engine = Arc::new(StateEngine::new().await.unwrap());
            let doc_id = DocumentId::new("docs", "test");
            let handle = state_engine.create_document(doc_id).await.unwrap();

            // Add operations
            for i in 0..1000 {
                handle
                    .update(|doc| {
                        doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                        Ok(())
                    })
                    .unwrap();

                // Eagerly read state after each update
                handle
                    .read(|doc| {
                        let _: Option<(automerge::Value, automerge::ObjId)> =
                            doc.get(automerge::ROOT, format!("key{}", i))?;
                        Ok(())
                    })
                    .unwrap();
            }

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box(delta_bytes)
        });
    });

    // Lazy materialization (only compute when needed)
    group.bench_function("lazy", |b| {
        b.to_async(&rt).iter(|| async {
            let baseline_memory = get_memory_usage();

            let state_engine = Arc::new(StateEngine::new().await.unwrap());
            let doc_id = DocumentId::new("docs", "test");
            let handle = state_engine.create_document(doc_id).await.unwrap();

            // Add operations (don't read state)
            for i in 0..1000 {
                handle
                    .update(|doc| {
                        doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                        Ok(())
                    })
                    .unwrap();
            }

            // Only materialize once at the end
            handle
                .read(|doc| {
                    for i in 0..1000 {
                        let _: Option<(automerge::Value, automerge::ObjId)> =
                            doc.get(automerge::ROOT, format!("key{}", i))?;
                    }
                    Ok(())
                })
                .unwrap();

            let peak_memory = get_memory_usage();
            let delta_bytes = peak_memory.saturating_sub(baseline_memory);

            black_box(delta_bytes)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_100k_records,
    bench_memory_scaling,
    bench_string_interning,
    bench_compact_structures,
    bench_memory_pooling,
    bench_lazy_materialization,
);

criterion_main!(benches);
