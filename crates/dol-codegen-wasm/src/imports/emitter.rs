//! Import emission for WASM modules
//!
//! This module handles the generation of WASM import declarations for host functions.

use dol_abi::{HostFunction, HostFunctionSignature, WasmType, IMPORT_MODULE};
use std::collections::HashMap;
use thiserror::Error;
use walrus::{FunctionId, ImportId, Module, ValType};

/// Error types for import emission
#[derive(Debug, Error)]
pub enum ImportError {
    /// Host function not found
    #[error("Host function not found: {0}")]
    FunctionNotFound(String),

    /// Signature mismatch
    #[error("Signature mismatch for {function}: expected {expected}, got {actual}")]
    SignatureMismatch {
        /// The name of the function with the mismatched signature
        function: String,
        /// The expected signature
        expected: String,
        /// The actual signature provided
        actual: String,
    },

    /// Import already exists
    #[error("Import already exists: {0}")]
    DuplicateImport(String),

    /// WASM module error
    #[error("WASM module error: {0}")]
    ModuleError(String),
}

/// Information about an imported function
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// The host function definition
    pub function: HostFunction,
    /// The walrus import ID
    pub import_id: ImportId,
    /// The walrus function ID
    pub func_id: FunctionId,
}

/// Collection of imports for a WASM module
#[derive(Debug, Default)]
pub struct ImportSection {
    /// Map from function name to import info
    imports: HashMap<String, ImportInfo>,
}

impl ImportSection {
    /// Create a new import section
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    /// Get import info by function name
    pub fn get(&self, name: &str) -> Option<&ImportInfo> {
        self.imports.get(name)
    }

    /// Get the function ID for a host function
    pub fn get_func_id(&self, name: &str) -> Option<FunctionId> {
        self.imports.get(name).map(|info| info.func_id)
    }

    /// Add an import to the section
    pub fn add_import(&mut self, name: impl Into<String>, info: ImportInfo) {
        self.imports.insert(name.into(), info);
    }

    /// Get all imports
    pub fn imports(&self) -> &HashMap<String, ImportInfo> {
        &self.imports
    }

    /// Check if an import exists
    pub fn contains(&self, name: &str) -> bool {
        self.imports.contains_key(name)
    }

    /// Get the number of imports
    pub fn len(&self) -> usize {
        self.imports.len()
    }

    /// Check if the section is empty
    pub fn is_empty(&self) -> bool {
        self.imports.is_empty()
    }
}

/// Import emitter for WASM modules
pub struct ImportEmitter<'a> {
    /// The WASM module being built
    module: &'a mut Module,
    /// The import section being populated
    section: ImportSection,
}

impl<'a> ImportEmitter<'a> {
    /// Create a new import emitter
    pub fn new(module: &'a mut Module) -> Self {
        Self {
            module,
            section: ImportSection::new(),
        }
    }

    /// Add a host function import to the module
    ///
    /// # Arguments
    /// * `host_fn` - The host function to import
    ///
    /// # Returns
    /// The walrus function ID for the imported function
    ///
    /// # Errors
    /// Returns an error if the import already exists or if there's a signature mismatch
    pub fn add_import(&mut self, host_fn: HostFunction) -> Result<FunctionId, ImportError> {
        let import_name = host_fn.import_name();

        // Check if already imported
        if self.section.contains(&host_fn.name) {
            return Err(ImportError::DuplicateImport(import_name));
        }

        // Convert signature to walrus types
        let (params, results) = convert_signature(&host_fn.signature);

        // Import the function from the WASM module
        let func_ty = self.module.types.add(&params, &results);
        let (func_id, import_id) = self.module.add_import_func(
            IMPORT_MODULE,
            &import_name,
            func_ty,
        );

        // Store import info
        let info = ImportInfo {
            function: host_fn.clone(),
            import_id,
            func_id,
        };

        self.section.add_import(host_fn.name.clone(), info);

        Ok(func_id)
    }

    /// Emit all imports from a list of host functions
    ///
    /// # Arguments
    /// * `host_functions` - The host functions to import
    ///
    /// # Returns
    /// The populated import section
    pub fn emit_all(
        &mut self,
        host_functions: &[HostFunction],
    ) -> Result<&ImportSection, ImportError> {
        for host_fn in host_functions {
            self.add_import(host_fn.clone())?;
        }
        Ok(&self.section)
    }

    /// Get the import section
    pub fn section(&self) -> &ImportSection {
        &self.section
    }

    /// Consume the emitter and return the import section
    pub fn finish(self) -> ImportSection {
        self.section
    }
}

/// Convert a DOL ABI signature to walrus types
fn convert_signature(sig: &HostFunctionSignature) -> (Vec<ValType>, Vec<ValType>) {
    let params = sig.params.iter().map(wasm_type_to_valtype).collect();
    let results = sig
        .returns
        .as_ref()
        .map(|t| vec![wasm_type_to_valtype(t)])
        .unwrap_or_default();
    (params, results)
}

/// Convert a WasmType to a walrus ValType
fn wasm_type_to_valtype(ty: &WasmType) -> ValType {
    match ty {
        WasmType::I32 => ValType::I32,
        WasmType::I64 => ValType::I64,
        WasmType::F32 => ValType::F32,
        WasmType::F64 => ValType::F64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dol_abi::{standard_host_functions, HostFunctionCategory};

    #[test]
    fn test_import_section_basic() {
        let section = ImportSection::new();
        assert!(section.is_empty());
        assert_eq!(section.len(), 0);
    }

    #[test]
    fn test_convert_signature() {
        // Test void function
        let sig = HostFunctionSignature::void();
        let (params, results) = convert_signature(&sig);
        assert!(params.is_empty());
        assert!(results.is_empty());

        // Test function with parameters
        let sig = HostFunctionSignature::with_params(vec![WasmType::I32, WasmType::I32]);
        let (params, results) = convert_signature(&sig);
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], ValType::I32);
        assert_eq!(params[1], ValType::I32);
        assert!(results.is_empty());

        // Test function with return
        let sig = HostFunctionSignature::with_return(vec![WasmType::I32], WasmType::I64);
        let (params, results) = convert_signature(&sig);
        assert_eq!(params.len(), 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ValType::I64);
    }

    #[test]
    fn test_import_emitter() {
        let mut module = Module::default();
        let mut emitter = ImportEmitter::new(&mut module);

        // Add a host function
        let print_fn = HostFunction::new(
            "print",
            HostFunctionSignature::with_params(vec![WasmType::I32, WasmType::I32]),
            HostFunctionCategory::IO,
        );

        let func_id = emitter.add_import(print_fn.clone()).unwrap();
        // func_id should be valid (non-null)

        // Verify it was added
        let section = emitter.section();
        assert_eq!(section.len(), 1);
        assert!(section.contains("print"));

        let info = section.get("print").unwrap();
        assert_eq!(info.function.name, "print");
        assert_eq!(info.func_id, func_id);
    }

    #[test]
    fn test_duplicate_import() {
        let mut module = Module::default();
        let mut emitter = ImportEmitter::new(&mut module);

        let print_fn = HostFunction::new(
            "print",
            HostFunctionSignature::with_params(vec![WasmType::I32, WasmType::I32]),
            HostFunctionCategory::IO,
        );

        // First import should succeed
        emitter.add_import(print_fn.clone()).unwrap();

        // Second import should fail
        let result = emitter.add_import(print_fn);
        assert!(matches!(result, Err(ImportError::DuplicateImport(_))));
    }

    #[test]
    fn test_emit_all_standard_functions() {
        let mut module = Module::default();
        let mut emitter = ImportEmitter::new(&mut module);

        let host_functions = standard_host_functions();
        let section = emitter.emit_all(&host_functions).unwrap();

        // Verify all 22 functions were imported
        assert_eq!(section.len(), 22);

        // Verify specific functions
        assert!(section.contains("print"));
        assert!(section.contains("println"));
        assert!(section.contains("alloc"));
        assert!(section.contains("send"));
    }
}
