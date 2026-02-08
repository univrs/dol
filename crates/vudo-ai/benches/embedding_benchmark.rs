//! Benchmarks for embedding operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use vudo_ai::{
    embedding::EmbeddingService,
    model_manager::{ModelId, ModelManager, ModelMetadata, ModelType},
};

fn setup_embedding_service() -> EmbeddingService {
    let manager = Arc::new(ModelManager::new());

    let metadata = ModelMetadata {
        id: ModelId::new("bench-embedding"),
        name: "Benchmark Embedding Model".to_string(),
        description: "Model for benchmarking".to_string(),
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

    EmbeddingService::new(manager, ModelId::new("bench-embedding"))
}

fn bench_single_embedding(c: &mut Criterion) {
    let service = setup_embedding_service();

    c.bench_function("embed_single_text", |b| {
        b.iter(|| {
            service.embed(black_box("This is a test sentence for embedding generation")).unwrap()
        })
    });
}

fn bench_batch_embedding(c: &mut Criterion) {
    let service = setup_embedding_service();
    let texts = vec![
        "First test sentence",
        "Second test sentence",
        "Third test sentence with more words",
        "Fourth test sentence",
        "Fifth and final test sentence for batching",
    ];

    c.bench_function("embed_batch_5", |b| {
        b.iter(|| {
            for text in &texts {
                service.embed(black_box(text)).unwrap();
            }
        })
    });
}

fn bench_document_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_indexing");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let service = setup_embedding_service();
                for i in 0..size {
                    service
                        .index_document(
                            format!("doc{}", i),
                            black_box("This is a document that needs to be indexed"),
                        )
                        .unwrap();
                }
            })
        });
    }

    group.finish();
}

fn bench_similarity_search(c: &mut Criterion) {
    let service = setup_embedding_service();

    // Pre-index documents
    for i in 0..100 {
        service
            .index_document(format!("doc{}", i), &format!("Document content number {}", i))
            .unwrap();
    }

    let mut group = c.benchmark_group("similarity_search");

    for k in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(k), k, |b, &k| {
            b.iter(|| service.search(black_box("test query"), k).unwrap())
        });
    }

    group.finish();
}

fn bench_cosine_similarity(c: &mut Criterion) {
    let service = setup_embedding_service();

    let emb1 = service.embed("first text").unwrap();
    let emb2 = service.embed("second text").unwrap();

    c.bench_function("cosine_similarity", |b| {
        b.iter(|| emb1.cosine_similarity(black_box(&emb2)).unwrap())
    });
}

fn bench_embedding_normalization(c: &mut Criterion) {
    use vudo_ai::embedding::Embedding;

    let vector = vec![0.5; 384];

    c.bench_function("embedding_normalization", |b| {
        b.iter(|| Embedding::normalized(black_box(vector.clone())))
    });
}

criterion_group!(
    benches,
    bench_single_embedding,
    bench_batch_embedding,
    bench_document_indexing,
    bench_similarity_search,
    bench_cosine_similarity,
    bench_embedding_normalization,
);
criterion_main!(benches);
