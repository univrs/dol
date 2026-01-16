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
//! - **Compiler**: Transforms DOL AST → WASM bytecode (direct emission)
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
//! The current implementation uses direct WASM emission via wasm-encoder:
//!
//! 1. **DOL AST**: Parse DOL source into abstract syntax tree
//! 2. **WASM Bytecode**: Directly emit WASM instructions and sections
//!
//! This approach is simpler and more self-contained than the full MLIR pipeline.
//! For advanced optimization and LLVM integration, enable the `wasm-mlir` feature
//! (requires LLVM 18 installed).
//!
//! ## Supported Features
//!
//! - Function declarations with typed parameters and return values
//! - Integer (i64) and float (f64) literals
//! - Binary operations (add, sub, mul, div, mod, comparisons, logical)
//! - Function calls and return statements
//! - Variable references (function parameters)
//!
//! ## Limitations
//!
//! - No complex types (structs, enums, tuples)
//! - No local variables (let bindings)
//! - No control flow (if, loops, match)
//! - No closures or higher-order functions
//!
//! ## Feature Flags
//!
//! - `wasm`: Enables WASM compilation and runtime (direct emission)
//! - `wasm-mlir`: Enables WASM compilation via MLIR pipeline (requires LLVM 18)
//!
//! ## See Also
//!
//! - [`WasmCompiler`]: Compiles DOL modules to WASM bytecode
//! - [`WasmRuntime`]: Executes WASM modules
//! - [`WasmError`]: Error type for WASM operations

use std::error::Error;
use std::fmt;

pub mod alloc;
pub mod compiler;
pub mod layout;
pub mod runtime;

// Re-export compiler when wasm-compile feature is enabled (browser-compatible)
#[cfg(feature = "wasm-compile")]
pub use compiler::WasmCompiler;

// Re-export runtime when wasm-runtime feature is enabled (native only)
#[cfg(feature = "wasm-runtime")]
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

#[cfg(feature = "wasm-runtime")]
impl From<wasmtime::Error> for WasmError {
    fn from(err: wasmtime::Error) -> Self {
        WasmError::new(format!("Wasmtime error: {}", err))
    }
}
