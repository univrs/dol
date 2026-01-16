//! # WASM Runtime
//!
//! Provides a runtime environment for executing compiled Metal DOL WASM modules.
//!
//! The runtime uses Wasmtime as the underlying WASM execution engine, providing
//! a sandboxed environment for running DOL validation logic and ontology checks.
//!
//! ## Example
//!
//! ```rust,ignore
//! use metadol::wasm::{WasmRuntime, WasmCompiler};
//!
//! // Compile DOL to WASM
//! let compiler = WasmCompiler::new();
//! let wasm_bytes = compiler.compile(&module)?;
//!
//! // Create runtime and load module
//! let runtime = WasmRuntime::new()?;
//! let wasm_module = runtime.load(&wasm_bytes)?;
//!
//! // Call exported functions
//! let result = wasm_module.call("validate", &[])?;
//! ```

#[cfg(feature = "wasm-runtime")]
use crate::wasm::WasmError;
#[cfg(feature = "wasm-runtime")]
use std::path::Path;
#[cfg(feature = "wasm-runtime")]
use wasmtime::{Engine, Instance, Linker, Module, Store, Val};

/// WASM runtime for executing Metal DOL modules.
///
/// The `WasmRuntime` wraps a Wasmtime engine and provides methods for
/// loading and executing WASM modules compiled from DOL declarations.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::WasmRuntime;
///
/// let runtime = WasmRuntime::new()?;
/// ```
#[cfg(feature = "wasm-runtime")]
pub struct WasmRuntime {
    engine: Engine,
}

#[cfg(feature = "wasm-runtime")]
impl WasmRuntime {
    /// Create a new WASM runtime.
    ///
    /// Initializes a Wasmtime engine with default configuration suitable
    /// for running DOL-compiled WASM modules.
    ///
    /// # Returns
    ///
    /// A new `WasmRuntime` instance on success, or a `WasmError` if
    /// engine initialization fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmRuntime;
    ///
    /// let runtime = WasmRuntime::new()?;
    /// ```
    pub fn new() -> Result<Self, WasmError> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// Load WASM bytecode into a module.
    ///
    /// Takes compiled WASM bytecode and instantiates it in the runtime,
    /// creating a `WasmModule` that can be used to call exported functions.
    ///
    /// # Arguments
    ///
    /// * `wasm_bytes` - The WASM bytecode to load
    ///
    /// # Returns
    ///
    /// A `WasmModule` instance on success, or a `WasmError` if loading
    /// or instantiation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmRuntime;
    ///
    /// let runtime = WasmRuntime::new()?;
    /// let wasm_module = runtime.load(&wasm_bytes)?;
    /// ```
    pub fn load(&self, wasm_bytes: &[u8]) -> Result<WasmModule, WasmError> {
        let module = Module::from_binary(&self.engine, wasm_bytes)?;
        let linker = Linker::new(&self.engine);
        let mut store = Store::new(&self.engine, ());

        // Instantiate the module with an empty linker
        // (no host imports for now)
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(WasmModule { instance, store })
    }

    /// Load WASM bytecode from a file.
    ///
    /// Convenience method that reads WASM bytecode from a file and loads it.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the WASM file
    ///
    /// # Returns
    ///
    /// A `WasmModule` instance on success, or a `WasmError` if reading,
    /// loading, or instantiation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmRuntime;
    ///
    /// let runtime = WasmRuntime::new()?;
    /// let wasm_module = runtime.load_file("module.wasm")?;
    /// ```
    pub fn load_file(&self, path: impl AsRef<Path>) -> Result<WasmModule, WasmError> {
        let wasm_bytes = std::fs::read(path)?;
        self.load(&wasm_bytes)
    }
}

#[cfg(feature = "wasm-runtime")]
impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default WasmRuntime")
    }
}

/// An instantiated WASM module.
///
/// Represents a loaded and instantiated WASM module that can be executed.
/// Provides methods for calling exported functions.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::WasmRuntime;
///
/// let runtime = WasmRuntime::new()?;
/// let wasm_module = runtime.load(&wasm_bytes)?;
/// let result = wasm_module.call("validate", &[])?;
/// ```
#[cfg(feature = "wasm-runtime")]
pub struct WasmModule {
    instance: Instance,
    store: Store<()>,
}

#[cfg(feature = "wasm-runtime")]
impl WasmModule {
    /// Call an exported function in the WASM module.
    ///
    /// Invokes a function exported by the WASM module with the given arguments.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the exported function to call
    /// * `args` - Slice of arguments to pass to the function
    ///
    /// # Returns
    ///
    /// A vector of return values on success, or a `WasmError` if the function
    /// doesn't exist or execution fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmRuntime;
    /// use wasmtime::Val;
    ///
    /// let runtime = WasmRuntime::new()?;
    /// let wasm_module = runtime.load(&wasm_bytes)?;
    ///
    /// // Call function with no arguments
    /// let result = wasm_module.call("validate", &[])?;
    ///
    /// // Call function with arguments
    /// let args = vec![Val::I32(42)];
    /// let result = wasm_module.call("check_value", &args)?;
    /// ```
    pub fn call(&mut self, name: &str, args: &[Val]) -> Result<Vec<Val>, WasmError> {
        let func = self
            .instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| WasmError::new(format!("Function '{}' not found", name)))?;

        let mut results = vec![Val::I32(0); func.ty(&self.store).results().len()];
        func.call(&mut self.store, args, &mut results)?;

        Ok(results)
    }

    /// Get the underlying Wasmtime instance.
    ///
    /// Provides access to the raw Wasmtime instance for advanced use cases.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmRuntime;
    ///
    /// let runtime = WasmRuntime::new()?;
    /// let wasm_module = runtime.load(&wasm_bytes)?;
    /// let instance = wasm_module.instance();
    /// ```
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Get a mutable reference to the store.
    ///
    /// Provides access to the Wasmtime store for advanced operations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmRuntime;
    ///
    /// let runtime = WasmRuntime::new()?;
    /// let mut wasm_module = runtime.load(&wasm_bytes)?;
    /// let store = wasm_module.store_mut();
    /// ```
    pub fn store_mut(&mut self) -> &mut Store<()> {
        &mut self.store
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-runtime")]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_new() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_runtime_default() {
        let runtime = WasmRuntime::default();
        // Should not panic
        drop(runtime);
    }

    #[test]
    fn test_load_invalid_wasm() {
        let runtime = WasmRuntime::new().unwrap();
        let result = runtime.load(&[0x00, 0x01, 0x02, 0x03]);
        assert!(result.is_err());
    }

    // Note: More comprehensive tests would require valid WASM bytecode,
    // which will be available once the compiler is fully implemented.
}
