//! Integration tests for VUDO AI

use vudo_ai::{
    conflict_resolution::{Conflict, ConflictResolver, ConflictValue, ResolutionStrategy},
    embedding::{Embedding, EmbeddingService},
    inference::{InferenceEngine, InferenceTensor},
    model_manager::{ModelId, ModelManager, ModelMetadata, ModelType},
    AIService,
};
use std::sync::Arc;

/// Setup a test AI service with embedding model.
fn setup_test_service() -> AIService {
    let service = AIService::new();

    let metadata = ModelMetadata {
        id: ModelId::new("test-embedding"),
        name: "Test Embedding Model".to_string(),
        description: "Test embedding model for integration tests".to_string(),
        version: "1.0.0".to_string(),
        input_dims: vec![1, 512],
        output_dims: vec![1, 384],
        size_bytes: 1000,
        model_type: ModelType::Embedding,
        wasm_compatible: true,
    };

    service.model_manager.register(metadata).unwrap();
    service
        .model_manager
        .load(&ModelId::new("test-embedding"), vec![0u8; 1000])
        .unwrap();

    service.with_embedding_service(ModelId::new("test-embedding"))
}

#[test]
fn test_end_to_end_embedding_workflow() {
    let service = setup_test_service();
    let embedding_service = service.embedding_service.as_ref().unwrap();

    // Index documents
    embedding_service
        .index_document("doc1".to_string(), "Artificial intelligence and machine learning")
        .unwrap();
    embedding_service
        .index_document("doc2".to_string(), "Cooking recipes and food preparation")
        .unwrap();
    embedding_service
        .index_document("doc3".to_string(), "Deep learning neural networks")
        .unwrap();

    // Search for AI-related content
    let results = embedding_service
        .search("AI and neural networks", 3)
        .unwrap();

    assert_eq!(results.len(), 3);
    // Results should be sorted by relevance
    assert!(results[0].score >= results[1].score);
    assert!(results[1].score >= results[2].score);
}

#[test]
fn test_end_to_end_conflict_resolution() {
    let service = setup_test_service()
        .with_conflict_resolver()
        .unwrap();

    let resolver = service.conflict_resolver.as_ref().unwrap();

    // Test identical values
    let conflict1 = Conflict {
        document_id: "doc1".to_string(),
        object_id: "root".to_string(),
        key: "title".to_string(),
        local_value: ConflictValue::String("Same Title".to_string()),
        remote_value: ConflictValue::String("Same Title".to_string()),
        context: vec![],
    };

    let suggestion1 = resolver.suggest_resolution(&conflict1).unwrap();
    assert!(matches!(suggestion1.strategy, ResolutionStrategy::UseLocal));
    assert_eq!(suggestion1.confidence, 1.0);

    // Test append-like operation
    let conflict2 = Conflict {
        document_id: "doc2".to_string(),
        object_id: "root".to_string(),
        key: "content".to_string(),
        local_value: ConflictValue::String("Hello".to_string()),
        remote_value: ConflictValue::String("Hello World".to_string()),
        context: vec![],
    };

    let suggestion2 = resolver.suggest_resolution(&conflict2).unwrap();
    assert!(matches!(suggestion2.strategy, ResolutionStrategy::Merge(_)));
}

#[test]
fn test_batch_conflict_resolution() {
    let service = setup_test_service()
        .with_conflict_resolver()
        .unwrap();

    let resolver = service.conflict_resolver.as_ref().unwrap();

    let conflicts = vec![
        Conflict {
            document_id: "doc1".to_string(),
            object_id: "root".to_string(),
            key: "field1".to_string(),
            local_value: ConflictValue::String("a".to_string()),
            remote_value: ConflictValue::String("a".to_string()),
            context: vec![],
        },
        Conflict {
            document_id: "doc2".to_string(),
            object_id: "root".to_string(),
            key: "field2".to_string(),
            local_value: ConflictValue::String("short".to_string()),
            remote_value: ConflictValue::String("much longer value".to_string()),
            context: vec![],
        },
        Conflict {
            document_id: "doc3".to_string(),
            object_id: "root".to_string(),
            key: "field3".to_string(),
            local_value: ConflictValue::Int(42).to_string(),
            remote_value: ConflictValue::Int(100).to_string(),
            context: vec![],
        },
    ];

    let suggestions = resolver.batch_suggest(conflicts).unwrap();
    assert_eq!(suggestions.len(), 3);

    // First should be identical values
    assert!(matches!(suggestions[0].strategy, ResolutionStrategy::UseLocal));

    // All should have explanations
    for suggestion in suggestions {
        assert!(!suggestion.explanation.is_empty());
    }
}

#[test]
fn test_embedding_similarity() {
    let service = setup_test_service();
    let embedding_service = service.embedding_service.as_ref().unwrap();

    let emb1 = embedding_service.embed("machine learning").unwrap();
    let emb2 = embedding_service.embed("machine learning").unwrap();
    let emb3 = embedding_service.embed("cooking recipes").unwrap();

    // Same text should produce same embedding
    assert_eq!(emb1.vector, emb2.vector);

    // Different text should produce different embeddings
    assert_ne!(emb1.vector, emb3.vector);

    // Cosine similarity should be 1.0 for identical embeddings
    let similarity = emb1.cosine_similarity(&emb2).unwrap();
    assert!((similarity - 1.0).abs() < 0.0001);
}

#[test]
fn test_model_cache_eviction() {
    let config = vudo_ai::model_manager::ModelManagerConfig {
        max_cache_size: 2,
        max_memory_bytes: 10_000,
    };

    let service = AIService::with_config(config);

    // Register 3 models
    for i in 1..=3 {
        let metadata = ModelMetadata {
            id: ModelId::new(&format!("model-{}", i)),
            name: format!("Model {}", i),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            input_dims: vec![1, 512],
            output_dims: vec![1, 384],
            size_bytes: 1000,
            model_type: ModelType::Embedding,
            wasm_compatible: true,
        };
        service.model_manager.register(metadata).unwrap();
    }

    // Load all 3 models
    for i in 1..=3 {
        service
            .model_manager
            .load(&ModelId::new(&format!("model-{}", i)), vec![0u8; 1000])
            .unwrap();
    }

    // Only 2 should be cached due to LRU eviction
    let stats = service.model_manager.stats();
    assert_eq!(stats.cached_models, 2);
    assert_eq!(stats.registered_models, 3);
}

#[test]
fn test_privacy_preserving_search() {
    let service = setup_test_service();
    let embedding_service = service.embedding_service.as_ref().unwrap();

    // Index sensitive documents
    embedding_service
        .index_document("private1".to_string(), "confidential information")
        .unwrap();
    embedding_service
        .index_document("private2".to_string(), "secret data")
        .unwrap();

    // Search happens entirely locally
    let results = embedding_service
        .search("confidential secrets", 10)
        .unwrap();

    // Verify search worked (locally)
    assert_eq!(results.len(), 2);

    // All operations are local - no network calls
    // This is ensured by the architecture: no network dependencies in embedding module
}

#[test]
fn test_tensor_operations() {
    // Test creating tensors with different types
    let float_tensor = InferenceTensor::float32(vec![2, 3], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
    assert_eq!(float_tensor.size(), 6);
    assert_eq!(float_tensor.shape, vec![2, 3]);

    let int_tensor = InferenceTensor::int64(vec![3], vec![1, 2, 3]).unwrap();
    assert_eq!(int_tensor.size(), 3);

    // Test invalid dimensions
    let result = InferenceTensor::float32(vec![2, 3], vec![1.0, 2.0]);
    assert!(result.is_err());
}

#[test]
fn test_embedding_index_operations() {
    let service = setup_test_service();
    let embedding_service = service.embedding_service.as_ref().unwrap();

    // Index multiple documents
    for i in 0..10 {
        embedding_service
            .index_document(format!("doc{}", i), &format!("document content {}", i))
            .unwrap();
    }

    assert_eq!(embedding_service.index_size(), 10);

    // Remove some documents
    embedding_service.remove_document("doc5").unwrap();
    embedding_service.remove_document("doc7").unwrap();

    assert_eq!(embedding_service.index_size(), 8);

    // Clear index
    embedding_service.clear_index().unwrap();
    assert_eq!(embedding_service.index_size(), 0);
}

#[test]
fn test_service_stats() {
    let service = setup_test_service();
    let stats = service.stats();

    assert_eq!(stats.model_manager.registered_models, 1);
    assert_eq!(stats.model_manager.cached_models, 1);
    assert!(stats.embedding_service.is_some());
    assert!(stats.conflict_resolver.is_none());

    let embedding_stats = stats.embedding_service.unwrap();
    assert_eq!(embedding_stats.indexed_documents, 0);
}

#[test]
fn test_concurrent_embedding_operations() {
    use std::thread;

    let service = Arc::new(setup_test_service());
    let embedding_service = service.embedding_service.as_ref().unwrap().clone();

    let mut handles = vec![];

    // Spawn multiple threads doing embedding operations
    for i in 0..5 {
        let es = Arc::clone(&embedding_service);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let doc_id = format!("doc-{}-{}", i, j);
                let content = format!("content from thread {} doc {}", i, j);
                es.index_document(doc_id, &content).unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Should have 50 documents indexed
    assert_eq!(embedding_service.index_size(), 50);
}

#[test]
fn test_wasm_compatibility_flags() {
    let service = setup_test_service();

    let metadata = service
        .model_manager
        .get_metadata(&ModelId::new("test-embedding"))
        .unwrap();

    assert!(metadata.wasm_compatible);
    assert_eq!(metadata.model_type, ModelType::Embedding);
}

#[test]
fn test_model_not_found_error() {
    let service = AIService::new();
    let result = service
        .inference_engine
        .get_model_metadata(&ModelId::new("nonexistent"));

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
