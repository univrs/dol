//! Criterion benchmarks for merge performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use eg_walker_dol::{EgWalkerText, AutomergeText, TextCrdt};

fn bench_two_way_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("two_way_merge");

    for ops in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", ops), ops, |b, &ops| {
            b.iter_batched(
                || {
                    let mut alice = EgWalkerText::new("alice".to_string());
                    let mut bob = alice.fork();

                    for i in 0..ops {
                        alice.insert(alice.len(), &format!("A{} ", i)).unwrap();
                        bob.insert(bob.len(), &format!("B{} ", i)).unwrap();
                    }

                    (alice, bob)
                },
                |(mut alice, bob)| {
                    alice.merge(&bob).unwrap();
                    alice
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("automerge", ops), ops, |b, &ops| {
            b.iter_batched(
                || {
                    let mut alice = AutomergeText::new("alice".to_string());
                    let mut bob = alice.fork();

                    for i in 0..ops {
                        alice.insert(alice.len(), &format!("A{} ", i)).unwrap();
                        bob.insert(bob.len(), &format!("B{} ", i)).unwrap();
                    }

                    (alice, bob)
                },
                |(mut alice, bob)| {
                    alice.merge(&bob).unwrap();
                    alice
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_multi_way_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_way_merge");

    for replicas in [3, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", replicas), replicas, |b, &replicas| {
            b.iter_batched(
                || {
                    let base = EgWalkerText::new("base".to_string());
                    let mut docs: Vec<_> = (0..replicas)
                        .map(|i| {
                            let mut doc = base.fork();
                            for j in 0..100 {
                                doc.insert(doc.len(), &format!("R{}-{} ", i, j)).unwrap();
                            }
                            doc
                        })
                        .collect();
                    docs
                },
                |mut docs| {
                    let mut result = docs[0].clone();
                    for doc in &docs[1..] {
                        result.merge(doc).unwrap();
                    }
                    result
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("automerge", replicas), replicas, |b, &replicas| {
            b.iter_batched(
                || {
                    let base = AutomergeText::new("base".to_string());
                    let mut docs: Vec<_> = (0..replicas)
                        .map(|i| {
                            let mut doc = base.fork();
                            for j in 0..100 {
                                doc.insert(doc.len(), &format!("R{}-{} ", i, j)).unwrap();
                            }
                            doc
                        })
                        .collect();
                    docs
                },
                |mut docs| {
                    let mut result = docs[0].clone();
                    for doc in &docs[1..] {
                        result.merge(doc).unwrap();
                    }
                    result
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_diverged_branches(c: &mut Criterion) {
    let mut group = c.benchmark_group("diverged_branches");

    // Simulate long-running offline branches
    for divergence in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("egwalker", divergence), divergence, |b, &divergence| {
            b.iter_batched(
                || {
                    let mut alice = EgWalkerText::new("alice".to_string());
                    alice.insert(0, "Initial content").unwrap();

                    let mut bob = alice.fork();

                    // Alice and Bob diverge significantly
                    for i in 0..divergence {
                        alice.insert(alice.len(), &format!("A{} ", i)).unwrap();
                        bob.insert(0, &format!("B{} ", i)).unwrap(); // Insert at beginning
                    }

                    (alice, bob)
                },
                |(mut alice, bob)| {
                    alice.merge(&bob).unwrap();
                    alice
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("automerge", divergence), divergence, |b, &divergence| {
            b.iter_batched(
                || {
                    let mut alice = AutomergeText::new("alice".to_string());
                    alice.insert(0, "Initial content").unwrap();

                    let mut bob = alice.fork();

                    for i in 0..divergence {
                        alice.insert(alice.len(), &format!("A{} ", i)).unwrap();
                        bob.insert(0, &format!("B{} ", i)).unwrap();
                    }

                    (alice, bob)
                },
                |(mut alice, bob)| {
                    alice.merge(&bob).unwrap();
                    alice
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_two_way_merge,
    bench_multi_way_merge,
    bench_diverged_branches
);
criterion_main!(benches);
