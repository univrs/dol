//! # WASM Compiler
//!
//! Compiles Metal DOL modules to WebAssembly bytecode.
//!
//! The compiler transforms DOL declarations through several intermediate
//! representations before producing WASM:
//!
//! ```text
//! DOL AST → MLIR → LLVM IR → WASM
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use metadol::wasm::WasmCompiler;
//! use metadol::parse_file;
//!
//! let source = r#"
//! gene container.exists {
//!   container has identity
//! }
//!
//! exegesis {
//!   A container exists.
//! }
//! "#;
//!
//! let module = parse_file(source)?;
//! let compiler = WasmCompiler::new()
//!     .with_optimization(true);
//!
//! let wasm_bytes = compiler.compile(&module)?;
//! ```

#[cfg(feature = "wasm")]
use crate::ast::Declaration;
#[cfg(feature = "wasm")]
use crate::wasm::WasmError;
#[cfg(feature = "wasm")]
use std::path::Path;

/// WASM compiler for Metal DOL modules.
///
/// The `WasmCompiler` transforms DOL declarations into WebAssembly bytecode.
/// It provides control over optimization levels and debug information.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::WasmCompiler;
///
/// let compiler = WasmCompiler::new()
///     .with_optimization(true)
///     .with_debug_info(false);
/// ```
#[cfg(feature = "wasm")]
#[derive(Debug, Clone)]
pub struct WasmCompiler {
    /// Enable LLVM optimizations
    optimize: bool,
    /// Include debug information in WASM
    debug_info: bool,
}

#[cfg(feature = "wasm")]
impl WasmCompiler {
    /// Create a new WASM compiler with default settings.
    ///
    /// Default settings:
    /// - Optimization: disabled
    /// - Debug info: enabled
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let compiler = WasmCompiler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            optimize: false,
            debug_info: true,
        }
    }

    /// Enable or disable optimizations.
    ///
    /// When enabled, LLVM will run optimization passes on the IR before
    /// generating WASM bytecode. This produces smaller and faster WASM
    /// modules but increases compilation time.
    ///
    /// # Arguments
    ///
    /// * `optimize` - `true` to enable optimizations, `false` to disable
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let compiler = WasmCompiler::new().with_optimization(true);
    /// ```
    pub fn with_optimization(mut self, optimize: bool) -> Self {
        self.optimize = optimize;
        self
    }

    /// Enable or disable debug information.
    ///
    /// When enabled, the WASM module will include source location information
    /// for debugging. This increases module size but improves debuggability.
    ///
    /// # Arguments
    ///
    /// * `debug_info` - `true` to include debug info, `false` to omit
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let compiler = WasmCompiler::new().with_debug_info(false);
    /// ```
    pub fn with_debug_info(mut self, debug_info: bool) -> Self {
        self.debug_info = debug_info;
        self
    }

    /// Compile a DOL module to WASM bytecode.
    ///
    /// Takes a DOL declaration AST and transforms it through MLIR and LLVM IR
    /// to produce WebAssembly bytecode.
    ///
    /// # Arguments
    ///
    /// * `module` - The DOL module to compile
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the WASM bytecode on success, or a `WasmError`
    /// if compilation fails.
    ///
    /// # Current Status
    ///
    /// This is a skeleton implementation. The full MLIR → LLVM → WASM lowering
    /// pipeline is complex and will be implemented in future phases.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    /// use metadol::parse_file;
    ///
    /// let source = r#"
    /// gene container.exists {
    ///   container has identity
    /// }
    ///
    /// exegesis {
    ///   A container exists.
    /// }
    /// "#;
    ///
    /// let module = parse_file(source)?;
    /// let compiler = WasmCompiler::new();
    /// let wasm_bytes = compiler.compile(&module)?;
    /// ```
    pub fn compile(&self, _module: &Declaration) -> Result<Vec<u8>, WasmError> {
        // TODO: Implement full compilation pipeline:
        // 1. Generate MLIR from DOL AST
        // 2. Lower MLIR to LLVM IR
        // 3. Use LLVM to compile to WASM
        //
        // This requires:
        // - MLIR func/arith/cf dialect code generation
        // - LLVM IR lowering passes
        // - WASM target configuration
        //
        // For now, return a placeholder error indicating the feature
        // is not yet fully implemented.

        Err(WasmError::new(
            "WASM backend not fully implemented: MLIR to LLVM to WASM lowering pipeline is complex and will be completed in a future phase",
        ))
    }

    /// Compile a DOL module to WASM and write to a file.
    ///
    /// Convenience method that calls [`compile`](WasmCompiler::compile) and
    /// writes the resulting bytecode to a file.
    ///
    /// # Arguments
    ///
    /// * `module` - The DOL module to compile
    /// * `output_path` - Path to write the WASM file
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `WasmError` if compilation or writing fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    /// use metadol::parse_file;
    ///
    /// let module = parse_file(source)?;
    /// let compiler = WasmCompiler::new();
    /// compiler.compile_to_file(&module, "output.wasm")?;
    /// ```
    pub fn compile_to_file(
        &self,
        module: &Declaration,
        output_path: impl AsRef<Path>,
    ) -> Result<(), WasmError> {
        let wasm_bytes = self.compile(module)?;
        std::fs::write(output_path, wasm_bytes)?;
        Ok(())
    }
}

#[cfg(feature = "wasm")]
impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(feature = "wasm")]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_new() {
        let compiler = WasmCompiler::new();
        assert!(!compiler.optimize);
        assert!(compiler.debug_info);
    }

    #[test]
    fn test_compiler_with_optimization() {
        let compiler = WasmCompiler::new().with_optimization(true);
        assert!(compiler.optimize);
    }

    #[test]
    fn test_compiler_with_debug_info() {
        let compiler = WasmCompiler::new().with_debug_info(false);
        assert!(!compiler.debug_info);
    }

    #[test]
    fn test_compiler_chaining() {
        let compiler = WasmCompiler::new()
            .with_optimization(true)
            .with_debug_info(false);
        assert!(compiler.optimize);
        assert!(!compiler.debug_info);
    }

    #[test]
    fn test_compiler_default() {
        let compiler = WasmCompiler::default();
        assert!(!compiler.optimize);
        assert!(compiler.debug_info);
    }
}
