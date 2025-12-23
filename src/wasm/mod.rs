//! # WASM Backend for Metal DOL
//!
//! This module provides WebAssembly compilation and runtime support for Metal DOL.
//! It enables compiling DOL ontology declarations to WASM bytecode and executing
//! them in a sandboxed WASM runtime.
//!
//! ## Architecture
//!
//! The WASM backend consists of two main components:
//!
//! - **Compiler**: Transforms DOL AST → MLIR → LLVM IR → WASM bytecode
//! - **Runtime**: Executes WASM modules using the Wasmtime runtime
//!
//! ## Usage
//!
//! The WASM backend is gated behind the `wasm` feature flag. Enable it in your
//! `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! metadol = { version = "0.0.1", features = ["wasm"] }
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use metadol::wasm::{WasmCompiler, WasmRuntime};
//!
//! // Compile DOL to WASM
//! let compiler = WasmCompiler::new()
//!     .with_optimization(true)
//!     .with_debug_info(false);
//!
//! let wasm_bytes = compiler.compile(module)?;
//!
//! // Execute WASM
//! let runtime = WasmRuntime::new()?;
//! let wasm_module = runtime.load(&wasm_bytes)?;
//! let result = wasm_module.call("validate", &[])?;
//! ```
//!
//! ## Compilation Pipeline
//!
//! 1. **DOL AST**: Parse DOL source into abstract syntax tree
//! 2. **MLIR**: Lower DOL AST to MLIR dialect (func, arith, cf)
//! 3. **LLVM IR**: Convert MLIR to LLVM intermediate representation
//! 4. **WASM**: Compile LLVM IR to WebAssembly bytecode
//!
//! ## Current Status
//!
//! This is a skeleton implementation for Q3 Phase 2. The full MLIR → LLVM → WASM
//! lowering pipeline is complex and will be implemented in future phases.
//!
//! ## Feature Flags
//!
//! - `wasm`: Enables WASM compilation and runtime (requires `mlir`)
//!
//! ## See Also
//!
//! - [`WasmCompiler`]: Compiles DOL modules to WASM bytecode
//! - [`WasmRuntime`]: Executes WASM modules
//! - [`WasmError`]: Error type for WASM operations

use std::error::Error;
use std::fmt;

pub mod compiler;
pub mod runtime;

// Re-export public types when wasm feature is enabled
#[cfg(feature = "wasm")]
pub use compiler::WasmCompiler;
#[cfg(feature = "wasm")]
pub use runtime::{WasmModule, WasmRuntime};

/// Error type for WASM backend operations.
///
/// Represents errors that can occur during WASM compilation, loading,
/// or execution. This includes compilation failures, runtime errors,
/// and I/O errors.
///
/// # Example
///
/// ```rust
/// use metadol::wasm::WasmError;
///
/// fn example() -> Result<(), WasmError> {
///     Err(WasmError {
///         message: "WASM compilation failed".to_string(),
///     })
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WasmError {
    /// Human-readable error message
    pub message: String,
}

impl WasmError {
    /// Create a new WASM error with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - Error message describing what went wrong
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::wasm::WasmError;
    ///
    /// let error = WasmError::new("Invalid WASM module");
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WASM error: {}", self.message)
    }
}

impl Error for WasmError {}

impl From<std::io::Error> for WasmError {
    fn from(err: std::io::Error) -> Self {
        WasmError::new(format!("I/O error: {}", err))
    }
}

#[cfg(feature = "wasm")]
impl From<wasmtime::Error> for WasmError {
    fn from(err: wasmtime::Error) -> Self {
        WasmError::new(format!("Wasmtime error: {}", err))
    }
}
