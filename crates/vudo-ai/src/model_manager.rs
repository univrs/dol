//! Model management for local AI inference.
//!
//! This module provides functionality for loading, caching, and managing AI models
//! that run entirely locally on-device with WASM compatibility.

use crate::error::{AIError, Result};
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Unique identifier for a model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    /// Create a new model ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model identifier.
    pub id: ModelId,
    /// Human-readable name.
    pub name: String,
    /// Model description.
    pub description: String,
    /// Model version.
    pub version: String,
    /// Input dimensions (e.g., [1, 512] for embeddings).
    pub input_dims: Vec<usize>,
    /// Output dimensions.
    pub output_dims: Vec<usize>,
    /// Model size in bytes.
    pub size_bytes: usize,
    /// Model type.
    pub model_type: ModelType,
    /// Whether this model is WASM-compatible.
    pub wasm_compatible: bool,
}

/// Type of AI model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Embedding model for semantic search.
    Embedding,
    /// Classification model.
    Classification,
    /// Sequence-to-sequence model.
    Seq2Seq,
    /// Custom model type.
    Custom,
}

/// A loaded model ready for inference.
pub struct LoadedModel {
    /// Model metadata.
    pub metadata: ModelMetadata,
    /// ONNX model bytes.
    pub model_bytes: Vec<u8>,
    /// Timestamp when loaded.
    pub loaded_at: std::time::Instant,
}

/// Model manager for loading and caching models.
pub struct ModelManager {
    /// Cache of loaded models (LRU eviction).
    cache: Arc<RwLock<LruCache<ModelId, Arc<LoadedModel>>>>,
    /// Model registry (metadata only).
    registry: DashMap<ModelId, ModelMetadata>,
    /// Maximum cache size in number of models.
    max_cache_size: usize,
    /// Maximum memory usage in bytes.
    max_memory_bytes: usize,
    /// Current memory usage in bytes.
    current_memory_bytes: Arc<RwLock<usize>>,
}

impl ModelManager {
    /// Create a new model manager with default settings.
    pub fn new() -> Self {
        Self::with_config(ModelManagerConfig::default())
    }

    /// Create a new model manager with custom configuration.
    pub fn with_config(config: ModelManagerConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.max_cache_size).unwrap();
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            registry: DashMap::new(),
            max_cache_size: config.max_cache_size,
            max_memory_bytes: config.max_memory_bytes,
            current_memory_bytes: Arc::new(RwLock::new(0)),
        }
    }

    /// Register a model in the registry.
    pub fn register(&self, metadata: ModelMetadata) -> Result<()> {
        info!("Registering model: {}", metadata.id);
        self.registry.insert(metadata.id.clone(), metadata);
        Ok(())
    }

    /// Unregister a model from the registry.
    pub fn unregister(&self, id: &ModelId) -> Result<()> {
        info!("Unregistering model: {}", id);
        self.registry.remove(id);
        self.evict(id)?;
        Ok(())
    }

    /// Load a model into the cache.
    pub fn load(&self, id: &ModelId, model_bytes: Vec<u8>) -> Result<Arc<LoadedModel>> {
        let metadata = self
            .registry
            .get(id)
            .ok_or_else(|| AIError::ModelNotFound(id.to_string()))?
            .clone();

        // Check memory constraints
        let model_size = model_bytes.len();
        let mut current_memory = self.current_memory_bytes.write();
        if *current_memory + model_size > self.max_memory_bytes {
            // Try to evict LRU models to make space
            self.evict_lru_until_space(model_size)?;
            *current_memory = *self.current_memory_bytes.read();
        }

        let loaded = Arc::new(LoadedModel {
            metadata: metadata.clone(),
            model_bytes,
            loaded_at: std::time::Instant::now(),
        });

        debug!("Loading model {} into cache", id);
        let mut cache = self.cache.write();

        // If inserting would evict, track memory
        if cache.len() >= self.max_cache_size {
            if let Some((_, evicted)) = cache.pop_lru() {
                let evicted_size = evicted.model_bytes.len();
                *self.current_memory_bytes.write() -= evicted_size;
                debug!("Evicted model from cache, freed {} bytes", evicted_size);
            }
        }

        cache.put(id.clone(), Arc::clone(&loaded));
        *self.current_memory_bytes.write() += model_size;

        info!(
            "Loaded model {} ({} bytes) - cache: {}/{}",
            id,
            model_size,
            cache.len(),
            self.max_cache_size
        );

        Ok(loaded)
    }

    /// Get a model from the cache.
    pub fn get(&self, id: &ModelId) -> Option<Arc<LoadedModel>> {
        let mut cache = self.cache.write();
        cache.get(id).map(Arc::clone)
    }

    /// Evict a specific model from the cache.
    pub fn evict(&self, id: &ModelId) -> Result<()> {
        let mut cache = self.cache.write();
        if let Some(model) = cache.pop(id) {
            let size = model.model_bytes.len();
            *self.current_memory_bytes.write() -= size;
            debug!("Evicted model {} ({} bytes)", id, size);
        }
        Ok(())
    }

    /// Evict LRU models until we have enough space.
    fn evict_lru_until_space(&self, needed: usize) -> Result<()> {
        let mut cache = self.cache.write();
        let mut current_memory = self.current_memory_bytes.write();

        while *current_memory + needed > self.max_memory_bytes && !cache.is_empty() {
            if let Some((id, model)) = cache.pop_lru() {
                let size = model.model_bytes.len();
                *current_memory -= size;
                warn!("Evicted LRU model {} to make space ({} bytes)", id, size);
            } else {
                break;
            }
        }

        if *current_memory + needed > self.max_memory_bytes {
            return Err(AIError::ResourceExhaustion(
                "Cannot free enough memory for model".to_string(),
            ));
        }

        Ok(())
    }

    /// Clear all models from the cache.
    pub fn clear(&self) -> Result<()> {
        info!("Clearing model cache");
        let mut cache = self.cache.write();
        cache.clear();
        *self.current_memory_bytes.write() = 0;
        Ok(())
    }

    /// Get statistics about the model manager.
    pub fn stats(&self) -> ModelManagerStats {
        let cache = self.cache.read();
        ModelManagerStats {
            registered_models: self.registry.len(),
            cached_models: cache.len(),
            memory_used_bytes: *self.current_memory_bytes.read(),
            max_memory_bytes: self.max_memory_bytes,
            max_cache_size: self.max_cache_size,
        }
    }

    /// List all registered models.
    pub fn list_models(&self) -> Vec<ModelMetadata> {
        self.registry
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get metadata for a specific model.
    pub fn get_metadata(&self, id: &ModelId) -> Option<ModelMetadata> {
        self.registry.get(id).map(|entry| entry.value().clone())
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the model manager.
#[derive(Debug, Clone)]
pub struct ModelManagerConfig {
    /// Maximum number of models to cache.
    pub max_cache_size: usize,
    /// Maximum memory usage in bytes (default: 200MB).
    pub max_memory_bytes: usize,
}

impl Default for ModelManagerConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 5,
            max_memory_bytes: 200 * 1024 * 1024, // 200MB
        }
    }
}

/// Statistics about the model manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManagerStats {
    /// Number of registered models.
    pub registered_models: usize,
    /// Number of cached models.
    pub cached_models: usize,
    /// Memory used by cached models in bytes.
    pub memory_used_bytes: usize,
    /// Maximum memory allowed in bytes.
    pub max_memory_bytes: usize,
    /// Maximum cache size.
    pub max_cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata(id: &str, size: usize) -> ModelMetadata {
        ModelMetadata {
            id: ModelId::new(id),
            name: format!("Test Model {}", id),
            description: "Test model".to_string(),
            version: "1.0.0".to_string(),
            input_dims: vec![1, 512],
            output_dims: vec![1, 384],
            size_bytes: size,
            model_type: ModelType::Embedding,
            wasm_compatible: true,
        }
    }

    #[test]
    fn test_model_manager_new() {
        let manager = ModelManager::new();
        let stats = manager.stats();
        assert_eq!(stats.registered_models, 0);
        assert_eq!(stats.cached_models, 0);
    }

    #[test]
    fn test_register_model() {
        let manager = ModelManager::new();
        let metadata = create_test_metadata("test-1", 1000);
        manager.register(metadata.clone()).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.registered_models, 1);

        let retrieved = manager.get_metadata(&ModelId::new("test-1")).unwrap();
        assert_eq!(retrieved.id, metadata.id);
    }

    #[test]
    fn test_load_model() {
        let manager = ModelManager::new();
        let metadata = create_test_metadata("test-1", 1000);
        manager.register(metadata).unwrap();

        let model_bytes = vec![0u8; 1000];
        let loaded = manager.load(&ModelId::new("test-1"), model_bytes).unwrap();

        assert_eq!(loaded.metadata.id, ModelId::new("test-1"));
        assert_eq!(loaded.model_bytes.len(), 1000);

        let stats = manager.stats();
        assert_eq!(stats.cached_models, 1);
        assert_eq!(stats.memory_used_bytes, 1000);
    }

    #[test]
    fn test_get_model() {
        let manager = ModelManager::new();
        let metadata = create_test_metadata("test-1", 1000);
        manager.register(metadata).unwrap();

        let model_bytes = vec![0u8; 1000];
        manager.load(&ModelId::new("test-1"), model_bytes).unwrap();

        let retrieved = manager.get(&ModelId::new("test-1")).unwrap();
        assert_eq!(retrieved.metadata.id, ModelId::new("test-1"));
    }

    #[test]
    fn test_evict_model() {
        let manager = ModelManager::new();
        let metadata = create_test_metadata("test-1", 1000);
        manager.register(metadata).unwrap();

        let model_bytes = vec![0u8; 1000];
        manager.load(&ModelId::new("test-1"), model_bytes).unwrap();

        manager.evict(&ModelId::new("test-1")).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.cached_models, 0);
        assert_eq!(stats.memory_used_bytes, 0);
    }

    #[test]
    fn test_lru_eviction() {
        let config = ModelManagerConfig {
            max_cache_size: 2,
            max_memory_bytes: 10_000,
        };
        let manager = ModelManager::with_config(config);

        // Register and load 3 models
        for i in 1..=3 {
            let id = format!("test-{}", i);
            let metadata = create_test_metadata(&id, 1000);
            manager.register(metadata).unwrap();
            manager.load(&ModelId::new(&id), vec![0u8; 1000]).unwrap();
        }

        // Should only have 2 models cached (LRU eviction)
        let stats = manager.stats();
        assert_eq!(stats.cached_models, 2);
        assert_eq!(stats.memory_used_bytes, 2000);

        // First model should be evicted
        assert!(manager.get(&ModelId::new("test-1")).is_none());
        assert!(manager.get(&ModelId::new("test-2")).is_some());
        assert!(manager.get(&ModelId::new("test-3")).is_some());
    }

    #[test]
    fn test_memory_limit() {
        let config = ModelManagerConfig {
            max_cache_size: 10,
            max_memory_bytes: 2500,
        };
        let manager = ModelManager::with_config(config);

        // Register models
        for i in 1..=3 {
            let id = format!("test-{}", i);
            let metadata = create_test_metadata(&id, 1000);
            manager.register(metadata).unwrap();
        }

        // Load models until memory limit
        manager.load(&ModelId::new("test-1"), vec![0u8; 1000]).unwrap();
        manager.load(&ModelId::new("test-2"), vec![0u8; 1000]).unwrap();

        // Should trigger eviction of test-1 to make space
        manager.load(&ModelId::new("test-3"), vec![0u8; 1000]).unwrap();

        let stats = manager.stats();
        assert!(stats.memory_used_bytes <= 2500);
    }

    #[test]
    fn test_clear_cache() {
        let manager = ModelManager::new();

        for i in 1..=3 {
            let id = format!("test-{}", i);
            let metadata = create_test_metadata(&id, 1000);
            manager.register(metadata).unwrap();
            manager.load(&ModelId::new(&id), vec![0u8; 1000]).unwrap();
        }

        manager.clear().unwrap();

        let stats = manager.stats();
        assert_eq!(stats.cached_models, 0);
        assert_eq!(stats.memory_used_bytes, 0);
        assert_eq!(stats.registered_models, 3); // Registered models remain
    }

    #[test]
    fn test_unregister_model() {
        let manager = ModelManager::new();
        let metadata = create_test_metadata("test-1", 1000);
        manager.register(metadata).unwrap();
        manager.load(&ModelId::new("test-1"), vec![0u8; 1000]).unwrap();

        manager.unregister(&ModelId::new("test-1")).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.registered_models, 0);
        assert_eq!(stats.cached_models, 0);
    }

    #[test]
    fn test_list_models() {
        let manager = ModelManager::new();

        for i in 1..=3 {
            let metadata = create_test_metadata(&format!("test-{}", i), 1000);
            manager.register(metadata).unwrap();
        }

        let models = manager.list_models();
        assert_eq!(models.len(), 3);
    }

    #[test]
    fn test_model_id_display() {
        let id = ModelId::new("test-model");
        assert_eq!(id.to_string(), "test-model");
    }

    #[test]
    fn test_model_type_serialization() {
        let model_type = ModelType::Embedding;
        let json = serde_json::to_string(&model_type).unwrap();
        let deserialized: ModelType = serde_json::from_str(&json).unwrap();
        assert_eq!(model_type, deserialized);
    }
}
