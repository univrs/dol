//! Criterion benchmarks for memory footprint comparison

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use eg_walker_dol::{EgWalkerText, AutomergeText, TextCrdt};

fn bench_memory_per_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_per_operation");

    for ops in [100, 1000, 5000].iter() {
        // Eg-walker memory
        group.bench_with_input(BenchmarkId::new("egwalker", ops), ops, |b, &ops| {
            b.iter(|| {
                let mut doc = EgWalkerText::new("bench".to_string());
                for i in 0..ops {
                    doc.insert(doc.len(), "x").unwrap();
                }
                let memory = doc.memory_size();
                black_box(memory)
            });
        });

        // Automerge memory
        group.bench_with_input(BenchmarkId::new("automerge", ops), ops, |b, &ops| {
            b.iter(|| {
                let mut doc = AutomergeText::new("bench".to_string());
                for i in 0..ops {
                    doc.insert(doc.len(), "x").unwrap();
                }
                let memory = doc.memory_size();
                black_box(memory)
            });
        });
    }

    group.finish();
}

fn bench_serialization_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_size");

    for ops in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", ops), ops, |b, &ops| {
            b.iter_batched(
                || {
                    let mut doc = EgWalkerText::new("bench".to_string());
                    for i in 0..ops {
                        doc.insert(doc.len(), &format!("Op {} ", i)).unwrap();
                    }
                    doc
                },
                |doc| {
                    let bytes = doc.to_bytes().unwrap();
                    black_box(bytes.len())
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("automerge", ops), ops, |b, &ops| {
            b.iter_batched(
                || {
                    let mut doc = AutomergeText::new("bench".to_string());
                    for i in 0..ops {
                        doc.insert(doc.len(), &format!("Op {} ", i)).unwrap();
                    }
                    doc
                },
                |doc| {
                    let bytes = doc.to_bytes().unwrap();
                    black_box(bytes.len())
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_load_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_time");

    for ops in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", ops), ops, |b, &ops| {
            // Prepare serialized document
            let mut doc = EgWalkerText::new("bench".to_string());
            for i in 0..ops {
                doc.insert(doc.len(), &format!("Op {} ", i)).unwrap();
            }
            let bytes = doc.to_bytes().unwrap();

            b.iter(|| {
                let loaded = EgWalkerText::from_bytes(&bytes).unwrap();
                black_box(loaded)
            });
        });

        group.bench_with_input(BenchmarkId::new("automerge", ops), ops, |b, &ops| {
            // Prepare serialized document
            let mut doc = AutomergeText::new("bench".to_string());
            for i in 0..ops {
                doc.insert(doc.len(), &format!("Op {} ", i)).unwrap();
            }
            let bytes = doc.to_bytes().unwrap();

            b.iter(|| {
                let loaded = AutomergeText::from_bytes(&bytes).unwrap();
                black_box(loaded)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_memory_per_operation,
    bench_serialization_size,
    bench_load_time
);
criterion_main!(benches);
