//! Criterion benchmarks for text operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use eg_walker_dol::{EgWalkerText, AutomergeText, TextCrdt};

fn bench_insert_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_operations");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", size), size, |b, &size| {
            b.iter(|| {
                let mut doc = EgWalkerText::new("bench".to_string());
                for i in 0..size {
                    doc.insert(doc.len(), black_box("x")).unwrap();
                }
                doc
            });
        });

        group.bench_with_input(BenchmarkId::new("automerge", size), size, |b, &size| {
            b.iter(|| {
                let mut doc = AutomergeText::new("bench".to_string());
                for i in 0..size {
                    doc.insert(doc.len(), black_box("x")).unwrap();
                }
                doc
            });
        });
    }

    group.finish();
}

fn bench_delete_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete_operations");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut doc = EgWalkerText::new("bench".to_string());
                    for _ in 0..size {
                        doc.insert(doc.len(), "x").unwrap();
                    }
                    doc
                },
                |mut doc| {
                    for _ in 0..size {
                        if doc.len() > 0 {
                            doc.delete(0, 1).unwrap();
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("automerge", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut doc = AutomergeText::new("bench".to_string());
                    for _ in 0..size {
                        doc.insert(doc.len(), "x").unwrap();
                    }
                    doc
                },
                |mut doc| {
                    for _ in 0..size {
                        if doc.len() > 0 {
                            doc.delete(0, 1).unwrap();
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_random_edits(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_edits");

    for size in [100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", size), size, |b, &size| {
            b.iter(|| {
                let mut doc = EgWalkerText::new("bench".to_string());
                for i in 0..size {
                    // Mix of inserts and deletes
                    if i % 3 == 0 && doc.len() > 0 {
                        let pos = doc.len() / 2;
                        doc.delete(pos, 1).unwrap();
                    } else {
                        let pos = if doc.len() > 0 { doc.len() / 2 } else { 0 };
                        doc.insert(pos, black_box("x")).unwrap();
                    }
                }
                doc
            });
        });

        group.bench_with_input(BenchmarkId::new("automerge", size), size, |b, &size| {
            b.iter(|| {
                let mut doc = AutomergeText::new("bench".to_string());
                for i in 0..size {
                    if i % 3 == 0 && doc.len() > 0 {
                        let pos = doc.len() / 2;
                        doc.delete(pos, 1).unwrap();
                    } else {
                        let pos = if doc.len() > 0 { doc.len() / 2 } else { 0 };
                        doc.insert(pos, black_box("x")).unwrap();
                    }
                }
                doc
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_insert_operations,
    bench_delete_operations,
    bench_random_edits
);
criterion_main!(benches);
