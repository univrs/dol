//! Error types for VUDO AI

use thiserror::Error;

/// Result type alias for VUDO AI operations.
pub type Result<T> = std::result::Result<T, AIError>;

/// Errors that can occur in the VUDO AI system.
#[derive(Error, Debug)]
pub enum AIError {
    /// Model loading error.
    #[error("Model loading error: {0}")]
    ModelLoading(String),

    /// Model not found error.
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Inference error.
    #[error("Inference error: {0}")]
    Inference(String),

    /// Invalid input dimensions.
    #[error("Invalid input dimensions: expected {expected}, got {actual}")]
    InvalidInputDimensions { expected: String, actual: String },

    /// Invalid output dimensions.
    #[error("Invalid output dimensions: expected {expected}, got {actual}")]
    InvalidOutputDimensions { expected: String, actual: String },

    /// Embedding error.
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Conflict resolution error.
    #[error("Conflict resolution error: {0}")]
    ConflictResolution(String),

    /// PlanetServe integration error.
    #[error("PlanetServe integration error: {0}")]
    PlanetServe(String),

    /// State engine error.
    #[error("State engine error: {0}")]
    StateEngine(#[from] vudo_state::error::StateError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ONNX model error.
    #[error("ONNX model error: {0}")]
    OnnxError(String),

    /// Model cache full.
    #[error("Model cache full")]
    CacheFull,

    /// Invalid model format.
    #[error("Invalid model format: {0}")]
    InvalidModelFormat(String),

    /// Privacy violation: attempted to send data externally.
    #[error("Privacy violation: {0}")]
    PrivacyViolation(String),

    /// Resource exhaustion (memory, compute).
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),

    /// WASM-specific error.
    #[error("WASM error: {0}")]
    WasmError(String),

    /// Tokio join error.
    #[error("Tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),

    /// Generic internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<tract_onnx::prelude::TractError> for AIError {
    fn from(err: tract_onnx::prelude::TractError) -> Self {
        AIError::OnnxError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AIError::ModelNotFound("test-model".to_string());
        assert_eq!(err.to_string(), "Model not found: test-model");
    }

    #[test]
    fn test_error_from_state_error() {
        let doc_id = vudo_state::document_store::DocumentId::new("test", "doc");
        let state_err = vudo_state::error::StateError::DocumentNotFound(doc_id.to_string());
        let ai_err: AIError = state_err.into();
        assert!(ai_err.to_string().contains("Document not found"));
    }

    #[test]
    fn test_invalid_input_dimensions() {
        let err = AIError::InvalidInputDimensions {
            expected: "[1, 512]".to_string(),
            actual: "[1, 256]".to_string(),
        };
        assert!(err.to_string().contains("expected"));
        assert!(err.to_string().contains("[1, 512]"));
    }

    #[test]
    fn test_privacy_violation() {
        let err = AIError::PrivacyViolation("Attempted to send data to external server".to_string());
        assert!(err.to_string().contains("Privacy violation"));
    }
}
