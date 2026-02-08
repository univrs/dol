//! Local embedding system for semantic search.
//!
//! This module provides functionality for generating text embeddings locally
//! using small ONNX models that run entirely on-device.

use crate::error::{AIError, Result};
use crate::model_manager::{ModelId, ModelManager};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Vector embedding (normalized).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// The embedding vector.
    pub vector: Vec<f32>,
    /// Vector dimension.
    pub dimension: usize,
}

impl Embedding {
    /// Create a new embedding from a vector.
    pub fn new(vector: Vec<f32>) -> Self {
        let dimension = vector.len();
        Self { vector, dimension }
    }

    /// Create a normalized embedding.
    pub fn normalized(mut vector: Vec<f32>) -> Self {
        let magnitude = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for x in &mut vector {
                *x /= magnitude;
            }
        }
        Self::new(vector)
    }

    /// Compute cosine similarity with another embedding.
    pub fn cosine_similarity(&self, other: &Embedding) -> Result<f32> {
        if self.dimension != other.dimension {
            return Err(AIError::Embedding(format!(
                "Dimension mismatch: {} vs {}",
                self.dimension, other.dimension
            )));
        }

        let dot_product: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();

        Ok(dot_product)
    }

    /// Compute Euclidean distance to another embedding.
    pub fn euclidean_distance(&self, other: &Embedding) -> Result<f32> {
        if self.dimension != other.dimension {
            return Err(AIError::Embedding(format!(
                "Dimension mismatch: {} vs {}",
                self.dimension, other.dimension
            )));
        }

        let distance: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt();

        Ok(distance)
    }
}

/// Result of a similarity search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document identifier.
    pub id: String,
    /// Similarity score (0.0 to 1.0, higher is more similar).
    pub score: f32,
    /// The embedding vector.
    pub embedding: Embedding,
}

/// Embedding service for generating and searching embeddings.
pub struct EmbeddingService {
    /// Model manager.
    model_manager: Arc<ModelManager>,
    /// Embedding index (document_id -> embedding).
    index: DashMap<String, Embedding>,
    /// Default model ID for embeddings.
    default_model_id: ModelId,
}

impl EmbeddingService {
    /// Create a new embedding service.
    pub fn new(model_manager: Arc<ModelManager>, default_model_id: ModelId) -> Self {
        Self {
            model_manager,
            index: DashMap::new(),
            default_model_id,
        }
    }

    /// Generate an embedding for text using the default model.
    pub fn embed(&self, text: &str) -> Result<Embedding> {
        self.embed_with_model(text, &self.default_model_id)
    }

    /// Generate an embedding for text using a specific model.
    pub fn embed_with_model(&self, text: &str, model_id: &ModelId) -> Result<Embedding> {
        debug!("Generating embedding for text with model {}", model_id);

        // Get model from cache
        let model = self
            .model_manager
            .get(model_id)
            .ok_or_else(|| AIError::ModelNotFound(model_id.to_string()))?;

        // For now, use a simple deterministic hash-based embedding
        // In production, this would call the ONNX model
        let embedding_vec = self.simple_hash_embedding(text, model.metadata.output_dims[1]);

        Ok(Embedding::normalized(embedding_vec))
    }

    /// Simple hash-based embedding (placeholder for ONNX inference).
    ///
    /// This generates a deterministic embedding from text using a simple
    /// hashing approach. In production, this would be replaced with actual
    /// ONNX model inference.
    fn simple_hash_embedding(&self, text: &str, dimension: usize) -> Vec<f32> {
        let hash = blake3::hash(text.as_bytes());
        let hash_bytes = hash.as_bytes();

        let mut vec = Vec::with_capacity(dimension);
        for i in 0..dimension {
            // Use hash bytes to generate deterministic values
            let byte_index = i % hash_bytes.len();
            let value = (hash_bytes[byte_index] as f32) / 255.0 - 0.5; // Normalize to [-0.5, 0.5]
            vec.push(value);
        }

        vec
    }

    /// Index a document with its text content.
    pub fn index_document(&self, id: String, text: &str) -> Result<()> {
        info!("Indexing document: {}", id);
        let embedding = self.embed(text)?;
        self.index.insert(id, embedding);
        Ok(())
    }

    /// Remove a document from the index.
    pub fn remove_document(&self, id: &str) -> Result<()> {
        info!("Removing document from index: {}", id);
        self.index.remove(id);
        Ok(())
    }

    /// Search for similar documents using text query.
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        debug!("Searching for similar documents: query='{}', top_k={}", query, top_k);

        let query_embedding = self.embed(query)?;
        self.search_by_embedding(&query_embedding, top_k)
    }

    /// Search for similar documents using an embedding.
    pub fn search_by_embedding(&self, query_embedding: &Embedding, top_k: usize) -> Result<Vec<SearchResult>> {
        let mut results: Vec<SearchResult> = self
            .index
            .iter()
            .map(|entry| {
                let id = entry.key().clone();
                let embedding = entry.value().clone();
                let score = query_embedding.cosine_similarity(&embedding).unwrap_or(0.0);

                SearchResult {
                    id,
                    score,
                    embedding,
                }
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Return top K
        results.truncate(top_k);

        Ok(results)
    }

    /// Get the number of indexed documents.
    pub fn index_size(&self) -> usize {
        self.index.len()
    }

    /// Clear the index.
    pub fn clear_index(&self) -> Result<()> {
        info!("Clearing embedding index");
        self.index.clear();
        Ok(())
    }

    /// Get statistics about the embedding service.
    pub fn stats(&self) -> EmbeddingServiceStats {
        EmbeddingServiceStats {
            indexed_documents: self.index.len(),
            model_id: self.default_model_id.clone(),
        }
    }
}

/// Statistics about the embedding service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingServiceStats {
    /// Number of indexed documents.
    pub indexed_documents: usize,
    /// Default model ID.
    pub model_id: ModelId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_manager::{ModelMetadata, ModelType};

    fn setup_test_service() -> (Arc<ModelManager>, EmbeddingService) {
        let manager = Arc::new(ModelManager::new());

        let metadata = ModelMetadata {
            id: ModelId::new("test-embedding-model"),
            name: "Test Embedding Model".to_string(),
            description: "Test model for embeddings".to_string(),
            version: "1.0.0".to_string(),
            input_dims: vec![1, 512],
            output_dims: vec![1, 384],
            size_bytes: 1000,
            model_type: ModelType::Embedding,
            wasm_compatible: true,
        };

        manager.register(metadata).unwrap();
        let model_bytes = vec![0u8; 1000];
        manager.load(&ModelId::new("test-embedding-model"), model_bytes).unwrap();

        let service = EmbeddingService::new(
            Arc::clone(&manager),
            ModelId::new("test-embedding-model"),
        );

        (manager, service)
    }

    #[test]
    fn test_embedding_new() {
        let vec = vec![1.0, 2.0, 3.0];
        let emb = Embedding::new(vec.clone());
        assert_eq!(emb.vector, vec);
        assert_eq!(emb.dimension, 3);
    }

    #[test]
    fn test_embedding_normalized() {
        let vec = vec![3.0, 4.0]; // Magnitude = 5.0
        let emb = Embedding::normalized(vec);
        assert!((emb.vector[0] - 0.6).abs() < 0.0001);
        assert!((emb.vector[1] - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity() {
        let emb1 = Embedding::new(vec![1.0, 0.0]);
        let emb2 = Embedding::new(vec![1.0, 0.0]);
        let similarity = emb1.cosine_similarity(&emb2).unwrap();
        assert!((similarity - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let emb1 = Embedding::normalized(vec![1.0, 0.0]);
        let emb2 = Embedding::normalized(vec![0.0, 1.0]);
        let similarity = emb1.cosine_similarity(&emb2).unwrap();
        assert!(similarity.abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_dimension_mismatch() {
        let emb1 = Embedding::new(vec![1.0, 0.0]);
        let emb2 = Embedding::new(vec![1.0, 0.0, 0.0]);
        assert!(emb1.cosine_similarity(&emb2).is_err());
    }

    #[test]
    fn test_euclidean_distance() {
        let emb1 = Embedding::new(vec![0.0, 0.0]);
        let emb2 = Embedding::new(vec![3.0, 4.0]);
        let distance = emb1.euclidean_distance(&emb2).unwrap();
        assert!((distance - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_embedding_service_new() {
        let (_, service) = setup_test_service();
        let stats = service.stats();
        assert_eq!(stats.indexed_documents, 0);
    }

    #[test]
    fn test_embed() {
        let (_, service) = setup_test_service();
        let embedding = service.embed("hello world").unwrap();
        assert_eq!(embedding.dimension, 384);
    }

    #[test]
    fn test_embed_deterministic() {
        let (_, service) = setup_test_service();
        let emb1 = service.embed("test text").unwrap();
        let emb2 = service.embed("test text").unwrap();
        assert_eq!(emb1.vector, emb2.vector);
    }

    #[test]
    fn test_index_document() {
        let (_, service) = setup_test_service();
        service.index_document("doc1".to_string(), "hello world").unwrap();
        assert_eq!(service.index_size(), 1);
    }

    #[test]
    fn test_remove_document() {
        let (_, service) = setup_test_service();
        service.index_document("doc1".to_string(), "hello world").unwrap();
        service.remove_document("doc1").unwrap();
        assert_eq!(service.index_size(), 0);
    }

    #[test]
    fn test_search() {
        let (_, service) = setup_test_service();

        service.index_document("doc1".to_string(), "artificial intelligence").unwrap();
        service.index_document("doc2".to_string(), "machine learning").unwrap();
        service.index_document("doc3".to_string(), "cooking recipes").unwrap();

        let results = service.search("AI and ML", 2).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score);
    }

    #[test]
    fn test_search_empty_index() {
        let (_, service) = setup_test_service();
        let results = service.search("test query", 5).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_top_k_limit() {
        let (_, service) = setup_test_service();

        for i in 0..10 {
            service.index_document(format!("doc{}", i), &format!("document {}", i)).unwrap();
        }

        let results = service.search("document", 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_clear_index() {
        let (_, service) = setup_test_service();

        service.index_document("doc1".to_string(), "test1").unwrap();
        service.index_document("doc2".to_string(), "test2").unwrap();

        service.clear_index().unwrap();
        assert_eq!(service.index_size(), 0);
    }

    #[test]
    fn test_search_by_embedding() {
        let (_, service) = setup_test_service();

        service.index_document("doc1".to_string(), "test document").unwrap();

        let query_emb = service.embed("test query").unwrap();
        let results = service.search_by_embedding(&query_emb, 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");
    }

    #[test]
    fn test_embedding_service_stats() {
        let (_, service) = setup_test_service();

        service.index_document("doc1".to_string(), "test").unwrap();

        let stats = service.stats();
        assert_eq!(stats.indexed_documents, 1);
        assert_eq!(stats.model_id.to_string(), "test-embedding-model");
    }
}
