//! VUDO AI - Local-First AI Integration for VUDO Runtime
//!
//! This crate provides privacy-preserving AI capabilities for the VUDO Runtime,
//! including:
//! - WASM-compatible ONNX inference using Tract
//! - Local embedding generation for semantic search
//! - AI-assisted CRDT conflict resolution
//! - Optional P2P model inference via PlanetServe
//!
//! All inference happens locally on-device by default, ensuring complete privacy.
//! No data leaves the device unless explicitly configured for P2P inference with
//! S-IDA fragmentation and encryption.
//!
//! # Examples
//!
//! ## Basic Embedding Service
//!
//! ```
//! use vudo_ai::{
//!     embedding::EmbeddingService,
//!     model_manager::{ModelId, ModelManager, ModelMetadata, ModelType},
//! };
//! use std::sync::Arc;
//!
//! # fn main() -> vudo_ai::error::Result<()> {
//! // Create model manager
//! let manager = Arc::new(ModelManager::new());
//!
//! // Register embedding model
//! let metadata = ModelMetadata {
//!     id: ModelId::new("my-embedding-model"),
//!     name: "Embedding Model".to_string(),
//!     description: "Local embedding model".to_string(),
//!     version: "1.0.0".to_string(),
//!     input_dims: vec![1, 512],
//!     output_dims: vec![1, 384],
//!     size_bytes: 50_000_000,  // 50MB
//!     model_type: ModelType::Embedding,
//!     wasm_compatible: true,
//! };
//! manager.register(metadata)?;
//!
//! // Load model
//! let model_bytes = vec![0u8; 1000]; // Load actual model bytes
//! manager.load(&ModelId::new("my-embedding-model"), model_bytes)?;
//!
//! // Create embedding service
//! let service = EmbeddingService::new(
//!     Arc::clone(&manager),
//!     ModelId::new("my-embedding-model"),
//! );
//!
//! // Generate embedding
//! let embedding = service.embed("Hello, world!")?;
//! println!("Generated embedding with dimension: {}", embedding.dimension);
//!
//! // Index documents
//! service.index_document("doc1".to_string(), "AI and machine learning")?;
//! service.index_document("doc2".to_string(), "Cooking recipes")?;
//!
//! // Search
//! let results = service.search("artificial intelligence", 5)?;
//! for result in results {
//!     println!("Found: {} (score: {})", result.id, result.score);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Conflict Resolution
//!
//! ```
//! use vudo_ai::{
//!     conflict_resolution::{Conflict, ConflictResolver, ConflictValue},
//!     embedding::EmbeddingService,
//!     inference::InferenceEngine,
//!     model_manager::{ModelId, ModelManager, ModelMetadata, ModelType},
//! };
//! use std::sync::Arc;
//!
//! # fn main() -> vudo_ai::error::Result<()> {
//! // Setup (same as above)
//! let manager = Arc::new(ModelManager::new());
//! let metadata = ModelMetadata {
//!     id: ModelId::new("embedding"),
//!     name: "Model".to_string(),
//!     description: "Test".to_string(),
//!     version: "1.0.0".to_string(),
//!     input_dims: vec![1, 512],
//!     output_dims: vec![1, 384],
//!     size_bytes: 1000,
//!     model_type: ModelType::Embedding,
//!     wasm_compatible: true,
//! };
//! manager.register(metadata)?;
//! manager.load(&ModelId::new("embedding"), vec![0u8; 1000])?;
//!
//! let embedding_service = Arc::new(EmbeddingService::new(
//!     Arc::clone(&manager),
//!     ModelId::new("embedding"),
//! ));
//! let inference_engine = Arc::new(InferenceEngine::new(manager));
//!
//! // Create conflict resolver
//! let resolver = ConflictResolver::new(embedding_service, inference_engine);
//!
//! // Create a conflict
//! let conflict = Conflict {
//!     document_id: "user-doc".to_string(),
//!     object_id: "root".to_string(),
//!     key: "name".to_string(),
//!     local_value: ConflictValue::String("Alice Smith".to_string()),
//!     remote_value: ConflictValue::String("Alice Johnson".to_string()),
//!     context: vec![],
//! };
//!
//! // Get resolution suggestion
//! let suggestion = resolver.suggest_resolution(&conflict)?;
//! println!("Suggested strategy: {:?}", suggestion.strategy);
//! println!("Confidence: {}", suggestion.confidence);
//! println!("Explanation: {}", suggestion.explanation);
//! # Ok(())
//! # }
//! ```

pub mod conflict_resolution;
pub mod embedding;
pub mod error;
pub mod inference;
pub mod model_manager;
pub mod planetserve_integration;

pub use conflict_resolution::{Conflict, ConflictResolver, ConflictValue, ResolutionStrategy, ResolutionSuggestion};
pub use embedding::{Embedding, EmbeddingService, SearchResult};
pub use error::{AIError, Result};
pub use inference::{InferenceEngine, InferenceTensor, TensorData};
pub use model_manager::{
    LoadedModel, ModelId, ModelManager, ModelManagerConfig, ModelManagerStats, ModelMetadata, ModelType,
};
pub use planetserve_integration::{P2PInferenceConfig, P2PInferenceRequest, P2PInferenceResponse, PlanetServeAI};

use std::sync::Arc;

/// Main AI service that coordinates all components.
pub struct AIService {
    /// Model manager.
    pub model_manager: Arc<ModelManager>,
    /// Inference engine.
    pub inference_engine: Arc<InferenceEngine>,
    /// Embedding service.
    pub embedding_service: Option<Arc<EmbeddingService>>,
    /// Conflict resolver.
    pub conflict_resolver: Option<Arc<ConflictResolver>>,
    /// PlanetServe integration (optional).
    pub planetserve_ai: Option<Arc<PlanetServeAI>>,
}

impl AIService {
    /// Create a new AI service with default configuration.
    pub fn new() -> Self {
        let model_manager = Arc::new(ModelManager::new());
        let inference_engine = Arc::new(InferenceEngine::new(Arc::clone(&model_manager)));

        Self {
            model_manager,
            inference_engine,
            embedding_service: None,
            conflict_resolver: None,
            planetserve_ai: None,
        }
    }

    /// Create a new AI service with custom model manager configuration.
    pub fn with_config(config: ModelManagerConfig) -> Self {
        let model_manager = Arc::new(ModelManager::with_config(config));
        let inference_engine = Arc::new(InferenceEngine::new(Arc::clone(&model_manager)));

        Self {
            model_manager,
            inference_engine,
            embedding_service: None,
            conflict_resolver: None,
            planetserve_ai: None,
        }
    }

    /// Enable embedding service with a default model.
    pub fn with_embedding_service(mut self, default_model_id: ModelId) -> Self {
        let embedding_service = Arc::new(EmbeddingService::new(
            Arc::clone(&self.model_manager),
            default_model_id,
        ));
        self.embedding_service = Some(embedding_service);
        self
    }

    /// Enable conflict resolver.
    pub fn with_conflict_resolver(mut self) -> Result<Self> {
        let embedding_service = self
            .embedding_service
            .clone()
            .ok_or_else(|| AIError::Internal("Embedding service required for conflict resolver".to_string()))?;

        let conflict_resolver = Arc::new(ConflictResolver::new(
            embedding_service,
            Arc::clone(&self.inference_engine),
        ));

        self.conflict_resolver = Some(conflict_resolver);
        Ok(self)
    }

    /// Enable PlanetServe integration for P2P inference.
    pub fn with_planetserve(
        mut self,
        fragmenter: Arc<vudo_planetserve::SidaFragmenter>,
        config: P2PInferenceConfig,
    ) -> Self {
        let planetserve_ai = Arc::new(PlanetServeAI::new(
            fragmenter,
            Arc::clone(&self.model_manager),
            config,
        ));
        self.planetserve_ai = Some(planetserve_ai);
        self
    }

    /// Get statistics about all components.
    pub fn stats(&self) -> AIServiceStats {
        AIServiceStats {
            model_manager: self.model_manager.stats(),
            embedding_service: self.embedding_service.as_ref().map(|s| s.stats()),
            conflict_resolver: self.conflict_resolver.as_ref().map(|r| r.stats()),
            planetserve_ai: self.planetserve_ai.as_ref().map(|p| p.stats()),
        }
    }
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the AI service.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIServiceStats {
    /// Model manager statistics.
    pub model_manager: ModelManagerStats,
    /// Embedding service statistics (if enabled).
    pub embedding_service: Option<embedding::EmbeddingServiceStats>,
    /// Conflict resolver statistics (if enabled).
    pub conflict_resolver: Option<conflict_resolution::ConflictResolverStats>,
    /// PlanetServe AI statistics (if enabled).
    pub planetserve_ai: Option<planetserve_integration::P2PInferenceStats>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_service_new() {
        let service = AIService::new();
        let stats = service.stats();
        assert_eq!(stats.model_manager.registered_models, 0);
        assert!(stats.embedding_service.is_none());
        assert!(stats.conflict_resolver.is_none());
    }

    #[test]
    fn test_ai_service_with_config() {
        let config = ModelManagerConfig {
            max_cache_size: 3,
            max_memory_bytes: 100_000_000,
        };
        let service = AIService::with_config(config);
        let stats = service.stats();
        assert_eq!(stats.model_manager.max_cache_size, 3);
    }

    #[test]
    fn test_ai_service_with_embedding() {
        let service = AIService::new();
        let manager = &service.model_manager;

        let metadata = ModelMetadata {
            id: ModelId::new("test-embedding"),
            name: "Test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            input_dims: vec![1, 512],
            output_dims: vec![1, 384],
            size_bytes: 1000,
            model_type: ModelType::Embedding,
            wasm_compatible: true,
        };
        manager.register(metadata).unwrap();

        let service = service.with_embedding_service(ModelId::new("test-embedding"));
        assert!(service.embedding_service.is_some());
    }

    #[test]
    fn test_ai_service_with_conflict_resolver() {
        let service = AIService::new();
        let manager = &service.model_manager;

        let metadata = ModelMetadata {
            id: ModelId::new("test-embedding"),
            name: "Test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            input_dims: vec![1, 512],
            output_dims: vec![1, 384],
            size_bytes: 1000,
            model_type: ModelType::Embedding,
            wasm_compatible: true,
        };
        manager.register(metadata).unwrap();

        let service = service
            .with_embedding_service(ModelId::new("test-embedding"))
            .with_conflict_resolver()
            .unwrap();

        assert!(service.conflict_resolver.is_some());
    }

    #[test]
    fn test_ai_service_without_embedding_conflict_resolver_fails() {
        let service = AIService::new();
        let result = service.with_conflict_resolver();
        assert!(result.is_err());
    }

    #[test]
    fn test_ai_service_stats() {
        let service = AIService::new();
        let stats = service.stats();
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("model_manager"));
    }
}
