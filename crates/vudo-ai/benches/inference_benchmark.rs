//! Benchmarks for inference operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use vudo_ai::{
    inference::{InferenceEngine, InferenceTensor},
    model_manager::{ModelId, ModelManager, ModelMetadata, ModelType},
};

fn setup_inference_engine() -> (Arc<ModelManager>, InferenceEngine) {
    let manager = Arc::new(ModelManager::new());

    let metadata = ModelMetadata {
        id: ModelId::new("bench-model"),
        name: "Benchmark Model".to_string(),
        description: "Model for benchmarking".to_string(),
        version: "1.0.0".to_string(),
        input_dims: vec![1, 512],
        output_dims: vec![1, 384],
        size_bytes: 1000,
        model_type: ModelType::Custom,
        wasm_compatible: true,
    };

    manager.register(metadata).unwrap();
    manager
        .load(&ModelId::new("bench-model"), vec![0u8; 1000])
        .unwrap();

    let engine = InferenceEngine::new(Arc::clone(&manager));
    (manager, engine)
}

fn bench_tensor_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_creation");

    for size in [128, 512, 1024, 4096].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("float32", size), size, |b, &size| {
            let data = vec![0.5f32; size];
            b.iter(|| InferenceTensor::float32(vec![1, size], black_box(data.clone())).unwrap())
        });
    }

    group.finish();
}

fn bench_tensor_access(c: &mut Criterion) {
    let tensor = InferenceTensor::float32(vec![1, 512], vec![0.5; 512]).unwrap();

    c.bench_function("tensor_as_f32", |b| {
        b.iter(|| black_box(&tensor).as_f32().unwrap())
    });
}

fn bench_model_loading(c: &mut Criterion) {
    let manager = Arc::new(ModelManager::new());

    let metadata = ModelMetadata {
        id: ModelId::new("load-test"),
        name: "Load Test".to_string(),
        description: "Test".to_string(),
        version: "1.0.0".to_string(),
        input_dims: vec![1, 512],
        output_dims: vec![1, 384],
        size_bytes: 1000,
        model_type: ModelType::Custom,
        wasm_compatible: true,
    };

    manager.register(metadata).unwrap();

    c.bench_function("model_load", |b| {
        b.iter(|| {
            let model_bytes = vec![0u8; 1000];
            manager
                .load(&ModelId::new("load-test"), black_box(model_bytes))
                .unwrap()
        })
    });
}

fn bench_model_cache_access(c: &mut Criterion) {
    let (manager, _) = setup_inference_engine();

    c.bench_function("model_cache_get", |b| {
        b.iter(|| manager.get(black_box(&ModelId::new("bench-model"))))
    });
}

criterion_group!(
    benches,
    bench_tensor_creation,
    bench_tensor_access,
    bench_model_loading,
    bench_model_cache_access,
);
criterion_main!(benches);
