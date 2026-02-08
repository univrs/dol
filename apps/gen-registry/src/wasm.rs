//! WASM module handling and validation

use crate::{
    error::{Error, Result},
    models::Capability,
};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

/// WASM module
pub struct WasmModule {
    bytes: Vec<u8>,
    hash: String,
}

impl WasmModule {
    /// Load WASM from file
    pub async fn from_file(path: &Path) -> Result<Self> {
        info!("Loading WASM from {:?}", path);

        let bytes = fs::read(path).await?;
        let hash = Self::compute_hash(&bytes);

        Ok(Self { bytes, hash })
    }

    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let hash = Self::compute_hash(&bytes);
        Self { bytes, hash }
    }

    /// Get SHA-256 hash
    pub fn hash(&self) -> String {
        self.hash.clone()
    }

    /// Get size in bytes
    pub fn size(&self) -> u64 {
        self.bytes.len() as u64
    }

    /// Get raw bytes
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Extract capabilities from WASM exports
    pub fn extract_capabilities(&self) -> Result<Vec<Capability>> {
        debug!("Extracting capabilities from WASM");

        // In real implementation:
        // 1. Parse WASM binary using wasmparser
        // 2. Extract export section
        // 3. Parse function signatures
        // 4. Return as Capability list

        // Placeholder
        Ok(Vec::new())
    }

    /// Compute SHA-256 hash
    fn compute_hash(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}

/// WASM validator
pub struct WasmValidator {
    max_size: u64,
    allowed_imports: Vec<String>,
}

impl WasmValidator {
    pub fn new() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10 MB
            allowed_imports: vec![
                "env.memory".to_string(),
                "wasi_snapshot_preview1.*".to_string(),
            ],
        }
    }

    /// Validate WASM module
    pub fn validate(&self, module: &WasmModule) -> Result<()> {
        debug!("Validating WASM module");

        // Check size
        if module.size() > self.max_size {
            return Err(Error::WasmValidationFailed(format!(
                "Module size {} exceeds maximum {}",
                module.size(),
                self.max_size
            )));
        }

        // Validate WASM binary
        self.validate_binary(module.bytes())?;

        // Check imports
        self.validate_imports(module.bytes())?;

        info!("WASM validation passed");
        Ok(())
    }

    /// Validate WASM binary format
    fn validate_binary(&self, bytes: &[u8]) -> Result<()> {
        // Check magic number
        if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
            return Err(Error::WasmValidationFailed(
                "Invalid WASM magic number".to_string(),
            ));
        }

        // Check version (1)
        if bytes.len() < 8 || bytes[4] != 1 || bytes[5] != 0 || bytes[6] != 0 || bytes[7] != 0 {
            return Err(Error::WasmValidationFailed("Invalid WASM version".to_string()));
        }

        // In real implementation:
        // Use wasmparser to fully validate module

        Ok(())
    }

    /// Validate imports
    fn validate_imports(&self, _bytes: &[u8]) -> Result<()> {
        // In real implementation:
        // 1. Parse import section
        // 2. Check against allowed_imports
        // 3. Reject suspicious imports

        Ok(())
    }

    /// Verify hash matches content
    pub fn verify_hash(&self, module: &WasmModule, expected_hash: &str) -> Result<()> {
        if module.hash() != expected_hash {
            return Err(Error::HashMismatch {
                expected: expected_hash.to_string(),
                actual: module.hash(),
            });
        }
        Ok(())
    }
}

impl Default for WasmValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_module() {
        // Minimal valid WASM module
        let bytes = vec![
            0x00, 0x61, 0x73, 0x6d, // Magic number: \0asm
            0x01, 0x00, 0x00, 0x00, // Version: 1
        ];

        let module = WasmModule::from_bytes(bytes);
        assert_eq!(module.size(), 8);
        assert!(!module.hash().is_empty());
    }

    #[test]
    fn test_wasm_validator() {
        let bytes = vec![
            0x00, 0x61, 0x73, 0x6d, // Magic
            0x01, 0x00, 0x00, 0x00, // Version
        ];

        let module = WasmModule::from_bytes(bytes);
        let validator = WasmValidator::new();

        assert!(validator.validate(&module).is_ok());
    }

    #[test]
    fn test_invalid_magic() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];

        let module = WasmModule::from_bytes(bytes);
        let validator = WasmValidator::new();

        assert!(validator.validate(&module).is_err());
    }

    #[test]
    fn test_hash_verification() {
        let bytes = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
        ];

        let module = WasmModule::from_bytes(bytes);
        let validator = WasmValidator::new();
        let hash = module.hash();

        assert!(validator.verify_hash(&module, &hash).is_ok());
        assert!(validator.verify_hash(&module, "invalid").is_err());
    }
}
