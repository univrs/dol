//! AI-assisted conflict resolution for CRDT merge conflicts.
//!
//! This module provides intelligent suggestions for resolving CRDT conflicts
//! using local AI models that analyze the conflict context and suggest resolutions.

use crate::embedding::EmbeddingService;
use crate::error::{AIError, Result};
use crate::inference::InferenceEngine;
use crate::model_manager::ModelId;
use automerge::{AutoCommit, ObjId};
use automerge::transaction::Transactable;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// A conflict detected in CRDT merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Document ID.
    pub document_id: String,
    /// Object ID where conflict occurred.
    pub object_id: String,
    /// Property key.
    pub key: String,
    /// Local value.
    pub local_value: ConflictValue,
    /// Remote value.
    pub remote_value: ConflictValue,
    /// Context around the conflict (e.g., other fields in the object).
    pub context: Vec<(String, String)>,
}

/// A value involved in a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictValue {
    /// String value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// Null value.
    Null,
    /// Complex value (JSON representation).
    Complex(serde_json::Value),
}

impl ConflictValue {
    /// Get a string representation of the value.
    pub fn to_string_repr(&self) -> String {
        match self {
            ConflictValue::String(s) => s.clone(),
            ConflictValue::Int(i) => i.to_string(),
            ConflictValue::Float(f) => f.to_string(),
            ConflictValue::Bool(b) => b.to_string(),
            ConflictValue::Null => "null".to_string(),
            ConflictValue::Complex(v) => serde_json::to_string(v).unwrap_or_default(),
        }
    }
}

/// A suggested resolution for a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionSuggestion {
    /// The conflict being resolved.
    pub conflict: Conflict,
    /// Suggested resolution strategy.
    pub strategy: ResolutionStrategy,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f32,
    /// Explanation for the suggestion.
    pub explanation: String,
}

/// Strategy for resolving a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Use the local value.
    UseLocal,
    /// Use the remote value.
    UseRemote,
    /// Merge both values (e.g., concatenate strings).
    Merge(String),
    /// Use a custom value.
    Custom(ConflictValue),
    /// Keep both values (create a list).
    KeepBoth,
    /// Needs manual resolution.
    ManualRequired,
}

/// AI-powered conflict resolution engine.
pub struct ConflictResolver {
    /// Embedding service for semantic analysis.
    embedding_service: Arc<EmbeddingService>,
    /// Inference engine for ML-based decisions.
    inference_engine: Arc<InferenceEngine>,
    /// Model ID for conflict resolution.
    resolution_model_id: Option<ModelId>,
}

impl ConflictResolver {
    /// Create a new conflict resolver.
    pub fn new(
        embedding_service: Arc<EmbeddingService>,
        inference_engine: Arc<InferenceEngine>,
    ) -> Self {
        Self {
            embedding_service,
            inference_engine,
            resolution_model_id: None,
        }
    }

    /// Create a conflict resolver with a specific model.
    pub fn with_model(
        embedding_service: Arc<EmbeddingService>,
        inference_engine: Arc<InferenceEngine>,
        model_id: ModelId,
    ) -> Self {
        Self {
            embedding_service,
            inference_engine,
            resolution_model_id: Some(model_id),
        }
    }

    /// Suggest a resolution for a conflict.
    pub fn suggest_resolution(&self, conflict: &Conflict) -> Result<ResolutionSuggestion> {
        info!("Analyzing conflict for {}.{}", conflict.document_id, conflict.key);

        // Use rule-based heuristics for now
        // In production, this would use ML models for more sophisticated analysis
        let suggestion = self.analyze_conflict_heuristic(conflict)?;

        debug!(
            "Suggested resolution: {:?} with confidence {}",
            suggestion.strategy, suggestion.confidence
        );

        Ok(suggestion)
    }

    /// Analyze conflict using heuristic rules.
    fn analyze_conflict_heuristic(&self, conflict: &Conflict) -> Result<ResolutionSuggestion> {
        // Compute semantic similarity between values
        let local_str = conflict.local_value.to_string_repr();
        let remote_str = conflict.remote_value.to_string_repr();

        let (strategy, confidence, explanation) = if local_str == remote_str {
            // Values are identical, use either
            (
                ResolutionStrategy::UseLocal,
                1.0,
                "Values are identical".to_string(),
            )
        } else if self.are_semantically_similar(&local_str, &remote_str)? {
            // Values are semantically similar, merge them
            let merged = format!("{} | {}", local_str, remote_str);
            (
                ResolutionStrategy::Merge(merged),
                0.75,
                "Values are semantically similar, suggested merge".to_string(),
            )
        } else if self.is_append_operation(&local_str, &remote_str) {
            // Looks like an append operation
            let merged = format!("{}{}", local_str, remote_str);
            (
                ResolutionStrategy::Merge(merged),
                0.8,
                "Detected append operation".to_string(),
            )
        } else if local_str.len() > remote_str.len() {
            // Favor more detailed value
            (
                ResolutionStrategy::UseLocal,
                0.6,
                "Local value is more detailed".to_string(),
            )
        } else if remote_str.len() > local_str.len() {
            (
                ResolutionStrategy::UseRemote,
                0.6,
                "Remote value is more detailed".to_string(),
            )
        } else {
            // Cannot determine, manual resolution needed
            (
                ResolutionStrategy::ManualRequired,
                0.0,
                "Cannot automatically resolve, manual review required".to_string(),
            )
        };

        Ok(ResolutionSuggestion {
            conflict: conflict.clone(),
            strategy,
            confidence,
            explanation,
        })
    }

    /// Check if two strings are semantically similar using embeddings.
    fn are_semantically_similar(&self, text1: &str, text2: &str) -> Result<bool> {
        if text1.is_empty() || text2.is_empty() {
            return Ok(false);
        }

        let emb1 = self.embedding_service.embed(text1)?;
        let emb2 = self.embedding_service.embed(text2)?;

        let similarity = emb1.cosine_similarity(&emb2)?;

        // Threshold for semantic similarity
        Ok(similarity > 0.85)
    }

    /// Check if the operation looks like an append.
    fn is_append_operation(&self, text1: &str, text2: &str) -> bool {
        // Simple heuristic: check if one starts with the other
        text1.starts_with(text2) || text2.starts_with(text1)
    }

    /// Batch process multiple conflicts.
    pub fn batch_suggest(&self, conflicts: Vec<Conflict>) -> Result<Vec<ResolutionSuggestion>> {
        info!("Processing {} conflicts in batch", conflicts.len());

        let mut suggestions = Vec::new();
        for conflict in conflicts {
            match self.suggest_resolution(&conflict) {
                Ok(suggestion) => suggestions.push(suggestion),
                Err(e) => {
                    warn!("Failed to resolve conflict: {}", e);
                    suggestions.push(ResolutionSuggestion {
                        conflict,
                        strategy: ResolutionStrategy::ManualRequired,
                        confidence: 0.0,
                        explanation: format!("Error during resolution: {}", e),
                    });
                }
            }
        }

        Ok(suggestions)
    }

    /// Apply a resolution suggestion to an Automerge document.
    pub fn apply_resolution(
        &self,
        doc: &mut AutoCommit,
        obj_id: &ObjId,
        suggestion: &ResolutionSuggestion,
    ) -> Result<()> {
        let key = &suggestion.conflict.key;

        match &suggestion.strategy {
            ResolutionStrategy::UseLocal => {
                debug!("Applying resolution: UseLocal for {}", key);
                // Local value is already in place, no action needed
                Ok(())
            }
            ResolutionStrategy::UseRemote => {
                debug!("Applying resolution: UseRemote for {}", key);
                // Apply remote value
                match &suggestion.conflict.remote_value {
                    ConflictValue::String(s) => {
                        doc.put(obj_id, key, s.as_str())
                            .map_err(|e| AIError::ConflictResolution(e.to_string()))?;
                    }
                    ConflictValue::Int(i) => {
                        doc.put(obj_id, key, *i)
                            .map_err(|e| AIError::ConflictResolution(e.to_string()))?;
                    }
                    ConflictValue::Bool(b) => {
                        doc.put(obj_id, key, *b)
                            .map_err(|e| AIError::ConflictResolution(e.to_string()))?;
                    }
                    ConflictValue::Float(f) => {
                        doc.put(obj_id, key, *f)
                            .map_err(|e| AIError::ConflictResolution(e.to_string()))?;
                    }
                    ConflictValue::Null => {
                        doc.delete(obj_id, key)
                            .map_err(|e| AIError::ConflictResolution(e.to_string()))?;
                    }
                    ConflictValue::Complex(_) => {
                        return Err(AIError::ConflictResolution(
                            "Complex values not yet supported".to_string(),
                        ));
                    }
                }
                Ok(())
            }
            ResolutionStrategy::Merge(merged) => {
                debug!("Applying resolution: Merge for {}", key);
                doc.put(obj_id, key, merged.as_str())
                    .map_err(|e| AIError::ConflictResolution(e.to_string()))?;
                Ok(())
            }
            ResolutionStrategy::Custom(value) => {
                debug!("Applying resolution: Custom for {}", key);
                match value {
                    ConflictValue::String(s) => {
                        doc.put(obj_id, key, s.as_str())
                            .map_err(|e| AIError::ConflictResolution(e.to_string()))?;
                    }
                    _ => {
                        return Err(AIError::ConflictResolution(
                            "Only string custom values supported".to_string(),
                        ));
                    }
                }
                Ok(())
            }
            ResolutionStrategy::KeepBoth => {
                Err(AIError::ConflictResolution(
                    "KeepBoth strategy not yet implemented".to_string(),
                ))
            }
            ResolutionStrategy::ManualRequired => {
                Err(AIError::ConflictResolution(
                    "Manual resolution required, cannot auto-apply".to_string(),
                ))
            }
        }
    }

    /// Get statistics about resolution suggestions.
    pub fn stats(&self) -> ConflictResolverStats {
        ConflictResolverStats {
            has_model: self.resolution_model_id.is_some(),
            model_id: self.resolution_model_id.clone(),
        }
    }
}

/// Statistics about the conflict resolver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolverStats {
    /// Whether a resolution model is configured.
    pub has_model: bool,
    /// Model ID if configured.
    pub model_id: Option<ModelId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::EmbeddingService;
    use crate::inference::InferenceEngine;
    use crate::model_manager::{ModelManager, ModelMetadata, ModelType};
    use automerge::ReadDoc;

    fn setup_test_resolver() -> ConflictResolver {
        let manager = Arc::new(ModelManager::new());

        let metadata = ModelMetadata {
            id: ModelId::new("test-embedding"),
            name: "Test Embedding".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            input_dims: vec![1, 512],
            output_dims: vec![1, 384],
            size_bytes: 1000,
            model_type: ModelType::Embedding,
            wasm_compatible: true,
        };

        manager.register(metadata).unwrap();
        manager.load(&ModelId::new("test-embedding"), vec![0u8; 1000]).unwrap();

        let embedding_service = Arc::new(EmbeddingService::new(
            Arc::clone(&manager),
            ModelId::new("test-embedding"),
        ));

        let inference_engine = Arc::new(InferenceEngine::new(manager));

        ConflictResolver::new(embedding_service, inference_engine)
    }

    fn create_test_conflict(local: &str, remote: &str) -> Conflict {
        Conflict {
            document_id: "test-doc".to_string(),
            object_id: "root".to_string(),
            key: "content".to_string(),
            local_value: ConflictValue::String(local.to_string()),
            remote_value: ConflictValue::String(remote.to_string()),
            context: vec![],
        }
    }

    #[test]
    fn test_conflict_resolver_new() {
        let _resolver = setup_test_resolver();
        // Just verify construction
    }

    #[test]
    fn test_suggest_resolution_identical() {
        let resolver = setup_test_resolver();
        let conflict = create_test_conflict("same text", "same text");

        let suggestion = resolver.suggest_resolution(&conflict).unwrap();
        assert!(matches!(suggestion.strategy, ResolutionStrategy::UseLocal));
        assert_eq!(suggestion.confidence, 1.0);
    }

    #[test]
    fn test_suggest_resolution_append() {
        let resolver = setup_test_resolver();
        let conflict = create_test_conflict("Hello", "Hello World");

        let suggestion = resolver.suggest_resolution(&conflict).unwrap();
        assert!(matches!(suggestion.strategy, ResolutionStrategy::Merge(_)));
    }

    #[test]
    fn test_suggest_resolution_favor_detailed() {
        let resolver = setup_test_resolver();
        let conflict = create_test_conflict("short", "much longer text");

        let suggestion = resolver.suggest_resolution(&conflict).unwrap();
        assert!(matches!(suggestion.strategy, ResolutionStrategy::UseRemote));
        assert!(suggestion.confidence > 0.0);
    }

    #[test]
    fn test_batch_suggest() {
        let resolver = setup_test_resolver();

        let conflicts = vec![
            create_test_conflict("a", "a"),
            create_test_conflict("b", "c"),
            create_test_conflict("Hello", "Hello World"),
        ];

        let suggestions = resolver.batch_suggest(conflicts).unwrap();
        assert_eq!(suggestions.len(), 3);
    }

    #[test]
    fn test_conflict_value_to_string() {
        assert_eq!(ConflictValue::String("test".to_string()).to_string_repr(), "test");
        assert_eq!(ConflictValue::Int(42).to_string_repr(), "42");
        assert_eq!(ConflictValue::Bool(true).to_string_repr(), "true");
        assert_eq!(ConflictValue::Null.to_string_repr(), "null");
    }

    #[test]
    fn test_apply_resolution_use_local() {
        let resolver = setup_test_resolver();
        let mut doc = AutoCommit::new();
        let obj_id = automerge::ROOT;

        doc.put(&obj_id, "key", "local").unwrap();

        let suggestion = ResolutionSuggestion {
            conflict: create_test_conflict("local", "remote"),
            strategy: ResolutionStrategy::UseLocal,
            confidence: 1.0,
            explanation: "Test".to_string(),
        };

        resolver.apply_resolution(&mut doc, &obj_id, &suggestion).unwrap();

        // Verify local value is preserved
        let value = doc.get(&obj_id, "key").unwrap();
        assert!(value.is_some());
    }

    #[test]
    fn test_apply_resolution_use_remote() {
        let resolver = setup_test_resolver();
        let mut doc = AutoCommit::new();
        let obj_id = automerge::ROOT;

        doc.put(&obj_id, "key", "local").unwrap();

        let suggestion = ResolutionSuggestion {
            conflict: create_test_conflict("local", "remote"),
            strategy: ResolutionStrategy::UseRemote,
            confidence: 0.8,
            explanation: "Test".to_string(),
        };

        resolver.apply_resolution(&mut doc, &obj_id, &suggestion).unwrap();

        // Value should be updated to remote
        let value = doc.get(&obj_id, "key").unwrap();
        assert!(value.is_some());
    }

    #[test]
    fn test_apply_resolution_merge() {
        let resolver = setup_test_resolver();
        let mut doc = AutoCommit::new();
        let obj_id = automerge::ROOT;

        let merged = "merged value".to_string();
        let suggestion = ResolutionSuggestion {
            conflict: create_test_conflict("a", "b"),
            strategy: ResolutionStrategy::Merge(merged),
            confidence: 0.75,
            explanation: "Test".to_string(),
        };

        resolver.apply_resolution(&mut doc, &obj_id, &suggestion).unwrap();
    }

    #[test]
    fn test_apply_resolution_manual_required() {
        let resolver = setup_test_resolver();
        let mut doc = AutoCommit::new();
        let obj_id = automerge::ROOT;

        let suggestion = ResolutionSuggestion {
            conflict: create_test_conflict("a", "b"),
            strategy: ResolutionStrategy::ManualRequired,
            confidence: 0.0,
            explanation: "Manual needed".to_string(),
        };

        let result = resolver.apply_resolution(&mut doc, &obj_id, &suggestion);
        assert!(result.is_err());
    }

    #[test]
    fn test_conflict_resolver_stats() {
        let resolver = setup_test_resolver();
        let stats = resolver.stats();
        assert!(!stats.has_model);
    }

    #[test]
    fn test_conflict_resolver_with_model() {
        let manager = Arc::new(ModelManager::new());
        let embedding_service = Arc::new(EmbeddingService::new(
            Arc::clone(&manager),
            ModelId::new("test"),
        ));
        let inference_engine = Arc::new(InferenceEngine::new(manager));

        let resolver = ConflictResolver::with_model(
            embedding_service,
            inference_engine,
            ModelId::new("test-model"),
        );

        let stats = resolver.stats();
        assert!(stats.has_model);
        assert_eq!(stats.model_id, Some(ModelId::new("test-model")));
    }
}
