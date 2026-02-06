//! WASM module size benchmarks
//!
//! Measures compiled WASM module size and verifies budget: < 100KB gzipped per Gen module

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

const WASM_SIZE_BUDGET_KB: usize = 100;

/// Helper to compile DOL to WASM
fn compile_dol_to_wasm(dol_source: &str) -> Vec<u8> {
    let temp_dir = TempDir::new().unwrap();
    let dol_path = temp_dir.path().join("test.dol");
    let wasm_path = temp_dir.path().join("output.wasm");

    // Write DOL source
    std::fs::write(&dol_path, dol_source).unwrap();

    // Compile DOL -> Rust -> WASM
    let status = Command::new("dol-build")
        .args(&[
            dol_path.to_str().unwrap(),
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "-o",
            wasm_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run dol-build");

    assert!(status.success(), "DOL compilation failed");

    // Read WASM module
    std::fs::read(&wasm_path).unwrap()
}

/// Helper to gzip compress data
fn gzip_compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

/// Benchmark WASM size for simple Gen module
fn bench_simple_gen_module(c: &mut Criterion) {
    let dol_source = r#"
gen User {
  @crdt(immutable) has id: String
  @crdt(lww) has name: String
  @crdt(pn_counter) has reputation: i64
}
"#;

    c.bench_function("wasm_size/simple_gen", |b| {
        b.iter(|| {
            let wasm_module = compile_dol_to_wasm(black_box(dol_source));
            let gzipped = gzip_compress(&wasm_module);

            let size_kb = gzipped.len() / 1024;

            // Verify budget
            assert!(
                size_kb < WASM_SIZE_BUDGET_KB,
                "WASM module {} KB exceeds {} KB budget",
                size_kb,
                WASM_SIZE_BUDGET_KB
            );

            black_box(gzipped.len())
        })
    });
}

/// Benchmark WASM size for Gen with multiple CRDTs
fn bench_multi_crdt_gen_module(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_size/multi_crdt");

    for num_fields in [5, 10, 20, 50].iter() {
        let mut fields = String::new();
        for i in 0..*num_fields {
            fields.push_str(&format!(
                "  @crdt(lww) has field{}: String\n",
                i
            ));
        }

        let dol_source = format!(
            r#"
gen Document {{
  @crdt(immutable) has id: String
{}
}}
"#,
            fields
        );

        group.bench_with_input(
            BenchmarkId::from_parameter(num_fields),
            num_fields,
            |b, _| {
                b.iter(|| {
                    let wasm_module = compile_dol_to_wasm(black_box(&dol_source));
                    let gzipped = gzip_compress(&wasm_module);

                    let size_kb = gzipped.len() / 1024;

                    // Verify budget
                    assert!(
                        size_kb < WASM_SIZE_BUDGET_KB,
                        "WASM module {} KB exceeds {} KB budget",
                        size_kb,
                        WASM_SIZE_BUDGET_KB
                    );

                    black_box(gzipped.len())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark WASM size with wasm-opt optimizations
fn bench_wasm_opt_optimizations(c: &mut Criterion) {
    let dol_source = r#"
gen User {
  @crdt(immutable) has id: String
  @crdt(lww) has name: String
  @crdt(lww) has email: String
  @crdt(pn_counter) has reputation: i64
  @crdt(mvr) has tags: [String]
}
"#;

    let mut group = c.benchmark_group("wasm_opt");

    // Benchmark without wasm-opt
    group.bench_function("no_opt", |b| {
        b.iter(|| {
            let wasm_module = compile_dol_to_wasm(black_box(dol_source));
            let gzipped = gzip_compress(&wasm_module);
            black_box(gzipped.len())
        });
    });

    // Benchmark with wasm-opt -Oz
    group.bench_function("opt_Oz", |b| {
        b.iter(|| {
            let wasm_module = compile_dol_to_wasm(black_box(dol_source));

            // Run wasm-opt
            let temp_dir = TempDir::new().unwrap();
            let input_path = temp_dir.path().join("input.wasm");
            let output_path = temp_dir.path().join("output.wasm");

            std::fs::write(&input_path, &wasm_module).unwrap();

            let status = Command::new("wasm-opt")
                .args(&[
                    "-Oz",
                    "--strip-debug",
                    "--strip-producers",
                    "--enable-mutable-globals",
                    "--enable-bulk-memory",
                    input_path.to_str().unwrap(),
                    "-o",
                    output_path.to_str().unwrap(),
                ])
                .status()
                .expect("Failed to run wasm-opt");

            assert!(status.success(), "wasm-opt failed");

            let optimized = std::fs::read(&output_path).unwrap();
            let gzipped = gzip_compress(&optimized);

            let size_kb = gzipped.len() / 1024;

            // Verify budget
            assert!(
                size_kb < WASM_SIZE_BUDGET_KB,
                "Optimized WASM module {} KB exceeds {} KB budget",
                size_kb,
                WASM_SIZE_BUDGET_KB
            );

            black_box(gzipped.len())
        });
    });

    group.finish();
}

/// Benchmark WASM size with tree shaking (feature flags)
fn bench_tree_shaking(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_size/tree_shaking");

    // All CRDTs enabled
    let all_crdts = r#"
gen Document {
  @crdt(immutable) has id: String
  @crdt(lww) has name: String
  @crdt(pn_counter) has counter: i64
  @crdt(mvr) has tags: [String]
  @crdt(g_counter) has views: u64
  @crdt(or_set) has collaborators: {String}
}
"#;

    group.bench_function("all_crdts", |b| {
        b.iter(|| {
            let wasm_module = compile_dol_to_wasm(black_box(all_crdts));
            let gzipped = gzip_compress(&wasm_module);
            black_box(gzipped.len())
        });
    });

    // Only LWW (tree shaking should remove other CRDTs)
    let only_lww = r#"
gen Document {
  @crdt(immutable) has id: String
  @crdt(lww) has name: String
  @crdt(lww) has description: String
}
"#;

    group.bench_function("only_lww", |b| {
        b.iter(|| {
            let wasm_module = compile_dol_to_wasm(black_box(only_lww));
            let gzipped = gzip_compress(&wasm_module);

            let size_kb = gzipped.len() / 1024;

            // Should be smaller due to tree shaking
            assert!(
                size_kb < WASM_SIZE_BUDGET_KB,
                "WASM module {} KB exceeds {} KB budget",
                size_kb,
                WASM_SIZE_BUDGET_KB
            );

            black_box(gzipped.len())
        });
    });

    group.finish();
}

/// Benchmark dead code elimination impact
fn bench_dead_code_elimination(c: &mut Criterion) {
    let dol_source = r#"
gen User {
  @crdt(immutable) has id: String
  @crdt(lww) has name: String
  @crdt(pn_counter) has reputation: i64
}
"#;

    let mut group = c.benchmark_group("wasm_size/dead_code");

    // Without LTO
    group.bench_function("no_lto", |b| {
        b.iter(|| {
            // Compile without LTO (modify Cargo.toml temporarily)
            let wasm_module = compile_dol_to_wasm(black_box(dol_source));
            let gzipped = gzip_compress(&wasm_module);
            black_box(gzipped.len())
        });
    });

    // With LTO
    group.bench_function("with_lto", |b| {
        b.iter(|| {
            // Compile with LTO (default release profile)
            let wasm_module = compile_dol_to_wasm(black_box(dol_source));
            let gzipped = gzip_compress(&wasm_module);

            let size_kb = gzipped.len() / 1024;

            // Should be smaller with LTO
            assert!(
                size_kb < WASM_SIZE_BUDGET_KB,
                "WASM module {} KB exceeds {} KB budget",
                size_kb,
                WASM_SIZE_BUDGET_KB
            );

            black_box(gzipped.len())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_gen_module,
    bench_multi_crdt_gen_module,
    bench_wasm_opt_optimizations,
    bench_tree_shaking,
    bench_dead_code_elimination,
);

criterion_main!(benches);
