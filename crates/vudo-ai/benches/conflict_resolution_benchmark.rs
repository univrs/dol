//! Benchmarks for conflict resolution operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use vudo_ai::{
    conflict_resolution::{Conflict, ConflictResolver, ConflictValue},
    embedding::EmbeddingService,
    inference::InferenceEngine,
    model_manager::{ModelId, ModelManager, ModelMetadata, ModelType},
};

fn setup_conflict_resolver() -> ConflictResolver {
    let manager = Arc::new(ModelManager::new());

    let metadata = ModelMetadata {
        id: ModelId::new("bench-embedding"),
        name: "Bench Embedding".to_string(),
        description: "Test".to_string(),
        version: "1.0.0".to_string(),
        input_dims: vec![1, 512],
        output_dims: vec![1, 384],
        size_bytes: 1000,
        model_type: ModelType::Embedding,
        wasm_compatible: true,
    };

    manager.register(metadata).unwrap();
    manager
        .load(&ModelId::new("bench-embedding"), vec![0u8; 1000])
        .unwrap();

    let embedding_service = Arc::new(EmbeddingService::new(
        Arc::clone(&manager),
        ModelId::new("bench-embedding"),
    ));

    let inference_engine = Arc::new(InferenceEngine::new(manager));

    ConflictResolver::new(embedding_service, inference_engine)
}

fn create_conflict(local: &str, remote: &str) -> Conflict {
    Conflict {
        document_id: "test-doc".to_string(),
        object_id: "root".to_string(),
        key: "content".to_string(),
        local_value: ConflictValue::String(local.to_string()),
        remote_value: ConflictValue::String(remote.to_string()),
        context: vec![],
    }
}

fn bench_single_conflict_resolution(c: &mut Criterion) {
    let resolver = setup_conflict_resolver();
    let conflict = create_conflict("local value", "remote value");

    c.bench_function("resolve_single_conflict", |b| {
        b.iter(|| resolver.suggest_resolution(black_box(&conflict)).unwrap())
    });
}

fn bench_identical_values(c: &mut Criterion) {
    let resolver = setup_conflict_resolver();
    let conflict = create_conflict("same value", "same value");

    c.bench_function("resolve_identical_values", |b| {
        b.iter(|| resolver.suggest_resolution(black_box(&conflict)).unwrap())
    });
}

fn bench_append_detection(c: &mut Criterion) {
    let resolver = setup_conflict_resolver();
    let conflict = create_conflict("Hello", "Hello World");

    c.bench_function("resolve_append_operation", |b| {
        b.iter(|| resolver.suggest_resolution(black_box(&conflict)).unwrap())
    });
}

fn bench_batch_conflict_resolution(c: &mut Criterion) {
    let resolver = setup_conflict_resolver();

    let mut group = c.benchmark_group("batch_conflict_resolution");

    for count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let conflicts: Vec<Conflict> = (0..count)
                .map(|i| create_conflict(&format!("local{}", i), &format!("remote{}", i)))
                .collect();

            b.iter(|| resolver.batch_suggest(black_box(conflicts.clone())).unwrap())
        });
    }

    group.finish();
}

fn bench_semantic_similarity_check(c: &mut Criterion) {
    let resolver = setup_conflict_resolver();

    // Conflicts with varying semantic similarity
    let conflicts = vec![
        create_conflict("machine learning", "deep learning"),
        create_conflict("cooking recipes", "food preparation"),
        create_conflict("hello world", "goodbye universe"),
    ];

    c.bench_function("semantic_similarity_analysis", |b| {
        b.iter(|| {
            for conflict in &conflicts {
                resolver.suggest_resolution(black_box(conflict)).unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_single_conflict_resolution,
    bench_identical_values,
    bench_append_detection,
    bench_batch_conflict_resolution,
    bench_semantic_similarity_check,
);
criterion_main!(benches);
