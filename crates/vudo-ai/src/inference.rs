//! ONNX inference engine for local AI models.
//!
//! This module provides a WASM-compatible inference engine using the Tract
//! ONNX runtime, enabling local AI inference without sending data externally.

use crate::error::{AIError, Result};
use crate::model_manager::{LoadedModel, ModelId, ModelManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tract_core::prelude::*;
use tract_onnx::prelude::*;
use tracing::{debug, info};

/// Inference engine for running ONNX models locally.
pub struct InferenceEngine {
    /// Model manager for loading models.
    model_manager: Arc<ModelManager>,
}

impl InferenceEngine {
    /// Create a new inference engine.
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }

    /// Run inference on a model with the given input tensors.
    pub fn infer(
        &self,
        model_id: &ModelId,
        inputs: Vec<InferenceTensor>,
    ) -> Result<Vec<InferenceTensor>> {
        info!("Running inference with model: {}", model_id);

        // Get model from manager
        let loaded_model = self
            .model_manager
            .get(model_id)
            .ok_or_else(|| AIError::ModelNotFound(model_id.to_string()))?;

        // Load ONNX model using Tract
        let model = self.load_tract_model(&loaded_model)?;

        // Convert inputs to Tract tensors
        let tract_inputs = self.convert_inputs_to_tract(inputs)?;

        // Run inference
        debug!("Executing model inference");
        let tract_outputs = model
            .run(tract_inputs)
            .map_err(|e| AIError::Inference(e.to_string()))?;

        // Convert outputs back to InferenceTensor
        let outputs = self.convert_tract_outputs(tract_outputs)?;

        info!("Inference completed successfully, {} outputs", outputs.len());
        Ok(outputs)
    }

    /// Load a Tract ONNX model from bytes.
    fn load_tract_model(&self, loaded_model: &LoadedModel) -> Result<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>> {
        debug!("Loading Tract ONNX model");

        // Parse ONNX model
        let model = tract_onnx::onnx()
            .model_for_read(&mut &loaded_model.model_bytes[..])
            .map_err(|e| AIError::OnnxError(format!("Failed to parse ONNX model: {}", e)))?;

        // Optimize model
        let model = model
            .into_optimized()
            .map_err(|e| AIError::OnnxError(format!("Failed to optimize model: {}", e)))?;

        // Create runnable model
        let model = model
            .into_runnable()
            .map_err(|e| AIError::OnnxError(format!("Failed to create runnable model: {}", e)))?;

        Ok(model)
    }

    /// Convert InferenceTensor inputs to Tract tensors.
    fn convert_inputs_to_tract(&self, inputs: Vec<InferenceTensor>) -> Result<TVec<TValue>> {
        let mut tract_inputs = tvec![];

        for input in inputs {
            let tensor = match input.data {
                TensorData::Float32(data) => {
                    let shape: Vec<usize> = input.shape.clone();
                    // Create tensor using tract's API
                    tract_ndarray::Array::from_shape_vec(
                        shape.as_slice(),
                        data
                    )
                    .map_err(|e| AIError::Inference(format!("Invalid tensor shape: {}", e)))?
                    .into_tensor()
                }
                TensorData::Int64(data) => {
                    let shape: Vec<usize> = input.shape.clone();
                    // Create tensor using tract's API
                    tract_ndarray::Array::from_shape_vec(
                        shape.as_slice(),
                        data
                    )
                    .map_err(|e| AIError::Inference(format!("Invalid tensor shape: {}", e)))?
                    .into_tensor()
                }
            };

            tract_inputs.push(tensor.into());
        }

        Ok(tract_inputs)
    }

    /// Convert Tract outputs to InferenceTensor.
    fn convert_tract_outputs(&self, outputs: TVec<TValue>) -> Result<Vec<InferenceTensor>> {
        let mut result = Vec::new();

        for output in outputs {
            let tensor = output.into_tensor();

            // Try to convert to f32 tensor
            if let Ok(tensor_f32) = tensor.to_array_view::<f32>() {
                let shape = tensor_f32.shape().to_vec();
                let data = tensor_f32.iter().copied().collect();

                result.push(InferenceTensor {
                    name: None,
                    shape,
                    data: TensorData::Float32(data),
                });
            }
            // Try to convert to i64 tensor
            else if let Ok(tensor_i64) = tensor.to_array_view::<i64>() {
                let shape = tensor_i64.shape().to_vec();
                let data = tensor_i64.iter().copied().collect();

                result.push(InferenceTensor {
                    name: None,
                    shape,
                    data: TensorData::Int64(data),
                });
            } else {
                return Err(AIError::Inference(
                    "Unsupported output tensor type".to_string(),
                ));
            }
        }

        Ok(result)
    }

    /// Get model metadata.
    pub fn get_model_metadata(&self, model_id: &ModelId) -> Result<crate::model_manager::ModelMetadata> {
        self.model_manager
            .get_metadata(model_id)
            .ok_or_else(|| AIError::ModelNotFound(model_id.to_string()))
    }
}

/// Tensor for inference input/output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceTensor {
    /// Optional tensor name.
    pub name: Option<String>,
    /// Tensor shape.
    pub shape: Vec<usize>,
    /// Tensor data.
    pub data: TensorData,
}

/// Tensor data types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TensorData {
    /// 32-bit floating point data.
    Float32(Vec<f32>),
    /// 64-bit integer data.
    Int64(Vec<i64>),
}

impl InferenceTensor {
    /// Create a new float32 tensor.
    pub fn float32(shape: Vec<usize>, data: Vec<f32>) -> Result<Self> {
        let expected_size: usize = shape.iter().product();
        if data.len() != expected_size {
            return Err(AIError::InvalidInputDimensions {
                expected: format!("{:?} (size: {})", shape, expected_size),
                actual: format!("data length: {}", data.len()),
            });
        }

        Ok(Self {
            name: None,
            shape,
            data: TensorData::Float32(data),
        })
    }

    /// Create a new int64 tensor.
    pub fn int64(shape: Vec<usize>, data: Vec<i64>) -> Result<Self> {
        let expected_size: usize = shape.iter().product();
        if data.len() != expected_size {
            return Err(AIError::InvalidInputDimensions {
                expected: format!("{:?} (size: {})", shape, expected_size),
                actual: format!("data length: {}", data.len()),
            });
        }

        Ok(Self {
            name: None,
            shape,
            data: TensorData::Int64(data),
        })
    }

    /// Set the tensor name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Get the tensor as f32 slice.
    pub fn as_f32(&self) -> Option<&[f32]> {
        match &self.data {
            TensorData::Float32(data) => Some(data),
            _ => None,
        }
    }

    /// Get the tensor as i64 slice.
    pub fn as_i64(&self) -> Option<&[i64]> {
        match &self.data {
            TensorData::Int64(data) => Some(data),
            _ => None,
        }
    }

    /// Get the total number of elements.
    pub fn size(&self) -> usize {
        self.shape.iter().product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_manager::{ModelManager, ModelMetadata, ModelType};

    fn setup_test_engine() -> (Arc<ModelManager>, InferenceEngine) {
        let manager = Arc::new(ModelManager::new());
        let engine = InferenceEngine::new(Arc::clone(&manager));
        (manager, engine)
    }

    #[test]
    fn test_inference_engine_new() {
        let (_, _engine) = setup_test_engine();
        // Just verify it constructs successfully
    }

    #[test]
    fn test_inference_tensor_float32() {
        let tensor = InferenceTensor::float32(vec![2, 3], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        assert_eq!(tensor.shape, vec![2, 3]);
        assert_eq!(tensor.size(), 6);
        assert_eq!(tensor.as_f32().unwrap().len(), 6);
    }

    #[test]
    fn test_inference_tensor_int64() {
        let tensor = InferenceTensor::int64(vec![2, 2], vec![1, 2, 3, 4]).unwrap();
        assert_eq!(tensor.shape, vec![2, 2]);
        assert_eq!(tensor.size(), 4);
        assert_eq!(tensor.as_i64().unwrap().len(), 4);
    }

    #[test]
    fn test_inference_tensor_invalid_size() {
        let result = InferenceTensor::float32(vec![2, 3], vec![1.0, 2.0, 3.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_inference_tensor_with_name() {
        let tensor = InferenceTensor::float32(vec![1, 2], vec![1.0, 2.0])
            .unwrap()
            .with_name("input");
        assert_eq!(tensor.name, Some("input".to_string()));
    }

    #[test]
    fn test_tensor_as_f32() {
        let tensor = InferenceTensor::float32(vec![2], vec![1.0, 2.0]).unwrap();
        assert_eq!(tensor.as_f32().unwrap(), &[1.0, 2.0]);
        assert!(tensor.as_i64().is_none());
    }

    #[test]
    fn test_tensor_as_i64() {
        let tensor = InferenceTensor::int64(vec![2], vec![1, 2]).unwrap();
        assert_eq!(tensor.as_i64().unwrap(), &[1, 2]);
        assert!(tensor.as_f32().is_none());
    }

    #[test]
    fn test_tensor_size() {
        let tensor = InferenceTensor::float32(vec![2, 3, 4], vec![0.0; 24]).unwrap();
        assert_eq!(tensor.size(), 24);
    }

    #[test]
    fn test_get_model_metadata_not_found() {
        let (_, engine) = setup_test_engine();
        let result = engine.get_model_metadata(&ModelId::new("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_model_metadata_success() {
        let (manager, engine) = setup_test_engine();

        let metadata = ModelMetadata {
            id: ModelId::new("test-model"),
            name: "Test Model".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            input_dims: vec![1, 512],
            output_dims: vec![1, 384],
            size_bytes: 1000,
            model_type: ModelType::Custom,
            wasm_compatible: true,
        };

        manager.register(metadata.clone()).unwrap();

        let retrieved = engine.get_model_metadata(&ModelId::new("test-model")).unwrap();
        assert_eq!(retrieved.id, metadata.id);
    }

    #[test]
    fn test_tensor_data_serialization() {
        let tensor = InferenceTensor::float32(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let json = serde_json::to_string(&tensor).unwrap();
        let deserialized: InferenceTensor = serde_json::from_str(&json).unwrap();
        assert_eq!(tensor.shape, deserialized.shape);
    }
}
