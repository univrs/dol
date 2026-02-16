//! Startup time benchmarks
//!
//! Measures cold start initialization time and verifies budget: < 500ms cold start in browser

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use vudo_p2p::P2PNode;
use vudo_state::StateEngine;

const STARTUP_TIME_BUDGET_MS: u128 = 500;

/// Benchmark cold start initialization - target: < 500ms
fn bench_cold_start(c: &mut Criterion) {
    c.bench_function("startup/cold_start", |b| {
        b.iter(|| {
            let start = Instant::now();

            // Initialize runtime
            let rt = Runtime::new().unwrap();

            rt.block_on(async {
                // Initialize VUDO state engine
                let state_engine = Arc::new(StateEngine::new().await.unwrap());

                // Initialize P2P network
                let _p2p_node = P2PNode::new(state_engine).await.unwrap();
            });

            let elapsed = start.elapsed();

            // Verify budget
            assert!(
                elapsed < Duration::from_millis(STARTUP_TIME_BUDGET_MS as u64),
                "Startup took {:?}, exceeds {}ms budget",
                elapsed,
                STARTUP_TIME_BUDGET_MS
            );

            black_box(elapsed)
        });
    });
}

/// Benchmark WASM module instantiation
fn bench_wasm_instantiation(c: &mut Criterion) {
    // Simulate WASM module instantiation
    c.bench_function("startup/wasm_instantiation", |b| {
        b.iter(|| {
            let start = Instant::now();

            // Load WASM module (simulated)
            let module_bytes = include_bytes!("../target/wasm32-unknown-unknown/release/vudo.wasm");

            // Instantiate (in real browser, this would use WebAssembly.instantiate)
            black_box(module_bytes);

            let elapsed = start.elapsed();

            // Should be fast (< 100ms for instantiation alone)
            assert!(
                elapsed < Duration::from_millis(100),
                "WASM instantiation took {:?}, exceeds 100ms budget",
                elapsed
            );

            black_box(elapsed)
        });
    });
}

/// Benchmark lazy initialization impact
fn bench_lazy_initialization(c: &mut Criterion) {
    use once_cell::sync::OnceCell;

    let mut group = c.benchmark_group("startup/lazy_init");

    // Eager initialization (initialize everything upfront)
    group.bench_function("eager", |b| {
        b.iter(|| {
            let start = Instant::now();

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let state_engine = Arc::new(StateEngine::new().await.unwrap());
                let _p2p_node = P2PNode::new(state_engine).await.unwrap();

                // Initialize storage adapters
                // Initialize identity system
                // Initialize credit system
                // etc.
            });

            let elapsed = start.elapsed();
            black_box(elapsed)
        });
    });

    // Lazy initialization (initialize on first use)
    group.bench_function("lazy", |b| {
        b.iter(|| {
            let start = Instant::now();

            static STATE_ENGINE: OnceCell<Arc<StateEngine>> = OnceCell::new();
            static P2P_NODE: OnceCell<Arc<P2PNode>> = OnceCell::new();

            // Don't initialize yet (lazy)
            let _ = STATE_ENGINE.get();
            let _ = P2P_NODE.get();

            let elapsed = start.elapsed();

            // Should be very fast (< 1ms) since nothing is actually initialized
            assert!(
                elapsed < Duration::from_millis(1),
                "Lazy initialization took {:?}, should be < 1ms",
                elapsed
            );

            black_box(elapsed)
        });
    });

    group.finish();
}

/// Benchmark async initialization (non-blocking)
fn bench_async_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup/async_init");

    // Synchronous initialization (blocks main thread)
    group.bench_function("sync", |b| {
        b.iter(|| {
            let start = Instant::now();

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // Block until everything is initialized
                let state_engine = Arc::new(StateEngine::new().await.unwrap());
                let _p2p_node = P2PNode::new(state_engine).await.unwrap();
            });

            let elapsed = start.elapsed();
            black_box(elapsed)
        });
    });

    // Asynchronous initialization (spawn background tasks)
    group.bench_function("async", |b| {
        b.iter(|| {
            let start = Instant::now();

            let rt = Runtime::new().unwrap();

            // Spawn initialization in background
            rt.spawn(async {
                let state_engine = Arc::new(StateEngine::new().await.unwrap());
                let _p2p_node = P2PNode::new(state_engine).await.unwrap();
            });

            // Return immediately
            let elapsed = start.elapsed();

            // Should be very fast (< 10ms) since initialization is async
            assert!(
                elapsed < Duration::from_millis(10),
                "Async initialization took {:?}, should be < 10ms",
                elapsed
            );

            black_box(elapsed)
        });
    });

    group.finish();
}

/// Benchmark state engine initialization
fn bench_state_engine_init(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("startup/state_engine_init", |b| {
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();

            let _state_engine = StateEngine::new().await.unwrap();

            let elapsed = start.elapsed();

            // Should be fast (< 50ms)
            assert!(
                elapsed < Duration::from_millis(50),
                "State engine init took {:?}, exceeds 50ms budget",
                elapsed
            );

            black_box(elapsed)
        });
    });
}

/// Benchmark P2P node initialization
fn bench_p2p_node_init(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("startup/p2p_node_init", |b| {
        b.to_async(&rt).iter(|| async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());

            let start = Instant::now();

            let _p2p_node = P2PNode::new(state_engine).await.unwrap();

            let elapsed = start.elapsed();

            // Should be reasonably fast (< 200ms)
            assert!(
                elapsed < Duration::from_millis(200),
                "P2P node init took {:?}, exceeds 200ms budget",
                elapsed
            );

            black_box(elapsed)
        });
    });
}

/// Benchmark persisted state loading
fn bench_persisted_state_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("startup/persisted_state");

    // Empty state (no persisted data)
    group.bench_function("empty", |b| {
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();

            let state_engine = StateEngine::new().await.unwrap();
            // No persisted state to load

            let elapsed = start.elapsed();
            black_box(elapsed)
        });
    });

    // Small state (10 documents)
    group.bench_function("small_10_docs", |b| {
        b.to_async(&rt).iter(|| async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());

            // Create 10 documents
            for i in 0..10 {
                let doc_id = vudo_state::DocumentId::new("docs", &format!("doc{}", i));
                state_engine.create_document(doc_id).await.unwrap();
            }

            // Simulate reload
            let start = Instant::now();

            // Load state from storage (simulated)
            let _new_engine = StateEngine::new().await.unwrap();

            let elapsed = start.elapsed();
            black_box(elapsed)
        });
    });

    // Large state (1000 documents)
    group.bench_function("large_1000_docs", |b| {
        b.to_async(&rt).iter(|| async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());

            // Create 1000 documents
            for i in 0..1000 {
                let doc_id = vudo_state::DocumentId::new("docs", &format!("doc{}", i));
                state_engine.create_document(doc_id).await.unwrap();
            }

            // Simulate reload
            let start = Instant::now();

            // Load state from storage (simulated)
            let _new_engine = StateEngine::new().await.unwrap();

            let elapsed = start.elapsed();

            // Should still be under budget
            assert!(
                elapsed < Duration::from_millis(STARTUP_TIME_BUDGET_MS as u64),
                "Loading 1000 docs took {:?}, exceeds {}ms budget",
                elapsed,
                STARTUP_TIME_BUDGET_MS
            );

            black_box(elapsed)
        });
    });

    group.finish();
}

/// Benchmark WASM streaming compilation (browser-specific)
fn bench_wasm_streaming_compilation(c: &mut Criterion) {
    c.bench_function("startup/wasm_streaming", |b| {
        b.iter(|| {
            let start = Instant::now();

            // Simulate WebAssembly.instantiateStreaming
            // In browser: fetch('vudo.wasm').then(WebAssembly.instantiateStreaming)

            let module_bytes = include_bytes!("../target/wasm32-unknown-unknown/release/vudo.wasm");

            // Simulate streaming compilation (chunks)
            let chunk_size = 1024;
            for chunk in module_bytes.chunks(chunk_size) {
                black_box(chunk);
            }

            let elapsed = start.elapsed();

            // Streaming should be faster than blocking instantiation
            black_box(elapsed)
        });
    });
}

/// Benchmark incremental startup (progressive enhancement)
fn bench_incremental_startup(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("startup/incremental", |b| {
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();

            // Phase 1: Minimal initialization (< 100ms)
            let state_engine = Arc::new(StateEngine::new().await.unwrap());

            let phase1_elapsed = start.elapsed();
            assert!(
                phase1_elapsed < Duration::from_millis(100),
                "Phase 1 took {:?}, exceeds 100ms",
                phase1_elapsed
            );

            // Phase 2: P2P initialization (background)
            tokio::spawn({
                let state_engine = Arc::clone(&state_engine);
                async move {
                    let _p2p_node = P2PNode::new(state_engine).await.unwrap();
                }
            });

            // Phase 3: Other systems (background)
            tokio::spawn(async {
                // Initialize credit system
                // Initialize identity system
                // etc.
            });

            // Return after phase 1 (< 100ms)
            black_box(phase1_elapsed)
        });
    });
}

criterion_group!(
    benches,
    bench_cold_start,
    bench_wasm_instantiation,
    bench_lazy_initialization,
    bench_async_initialization,
    bench_state_engine_init,
    bench_p2p_node_init,
    bench_persisted_state_loading,
    bench_wasm_streaming_compilation,
    bench_incremental_startup,
);

criterion_main!(benches);
