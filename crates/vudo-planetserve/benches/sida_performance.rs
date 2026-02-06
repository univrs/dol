//! Benchmarks for S-IDA fragmentation performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vudo_planetserve::sida::{SidaConfig, SidaFragmenter};

fn bench_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sida_fragmentation");

    // Test different k-of-n configurations
    let configs = vec![
        (2, 3, "2-of-3"),
        (3, 5, "3-of-5"),
        (5, 7, "5-of-7"),
    ];

    // Test different message sizes
    let sizes = vec![
        1_024,      // 1KB
        10_240,     // 10KB
        102_400,    // 100KB
        1_048_576,  // 1MB
    ];

    for (k, n, label) in configs {
        let config = SidaConfig { k, n };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        for size in &sizes {
            let message = vec![0u8; *size];

            group.throughput(Throughput::Bytes(*size as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("fragment_{}", label), size),
                &message,
                |b, msg| {
                    b.iter(|| {
                        fragmenter.fragment(black_box(msg)).unwrap()
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_reconstruction(c: &mut Criterion) {
    let mut group = c.benchmark_group("sida_reconstruction");

    let configs = vec![
        (2, 3, "2-of-3"),
        (3, 5, "3-of-5"),
        (5, 7, "5-of-7"),
    ];

    let sizes = vec![
        1_024,      // 1KB
        10_240,     // 10KB
        102_400,    // 100KB
        1_048_576,  // 1MB
    ];

    for (k, n, label) in configs {
        let config = SidaConfig { k, n };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        for size in &sizes {
            let message = vec![0u8; *size];
            let fragments = fragmenter.fragment(&message).unwrap();

            // Take k fragments for reconstruction
            let subset: Vec<_> = fragments.iter().take(k).cloned().collect();

            group.throughput(Throughput::Bytes(*size as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("reconstruct_{}", label), size),
                &subset,
                |b, frags| {
                    b.iter(|| {
                        fragmenter.reconstruct(black_box(frags.clone())).unwrap()
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_fragmentation, bench_reconstruction);
criterion_main!(benches);
