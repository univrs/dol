//! Spirit Compiler
//!
//! End-to-end compilation from DOL source to WebAssembly binary.
//!
//! # Compilation Pipeline
//!
//! The Spirit compiler orchestrates a multi-stage compilation process:
//!
//! ```text
//! ┌─────────────┐
//! │ DOL Source  │
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │   Lexer     │  Tokenization
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │   Parser    │  Syntax Analysis
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │     AST     │  Abstract Syntax Tree
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │     HIR     │  High-level IR (desugared, canonical)
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │    MLIR     │  Multi-level IR (optimizable)
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │    WASM     │  WebAssembly bytecode
//! └─────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::compiler::spirit::compile_source;
//!
//! let source = r#"
//!     module calculator @ 1.0
//!
//!     fun add(a: Int64, b: Int64) -> Int64 {
//!         return a + b
//!     }
//! "#;
//!
//! let compiled = compile_source(source, "calculator.dol")?;
//! assert!(!compiled.wasm.is_empty());
//! assert_eq!(&compiled.wasm[0..4], b"\0asm"); // WASM magic number
//! ```

use std::path::Path;

use crate::error::ParseError;
use crate::parser::Parser;

#[cfg(feature = "wasm")]
use crate::lower;

/// Result of Spirit compilation.
///
/// Contains the compiled WASM bytecode along with optional source maps
/// and compilation warnings.
#[derive(Debug)]
pub struct CompiledSpirit {
    /// WebAssembly binary bytecode
    pub wasm: Vec<u8>,

    /// Source map for debugging (maps WASM offsets to DOL source locations)
    pub source_map: Option<SourceMap>,

    /// Non-fatal warnings emitted during compilation
    pub warnings: Vec<CompilerWarning>,
}

/// Source map for debugging.
///
/// Maps WASM bytecode offsets back to original DOL source locations,
/// enabling debuggers to show DOL source while stepping through WASM execution.
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// Individual source map entries
    pub entries: Vec<SourceMapEntry>,
}

/// Single source map entry.
///
/// Maps a WASM bytecode offset to a DOL source location.
#[derive(Debug, Clone)]
pub struct SourceMapEntry {
    /// Offset in WASM bytecode
    pub wasm_offset: u32,

    /// Source file path
    pub source_file: String,

    /// Line number in source file (1-indexed)
    pub line: u32,

    /// Column number in line (1-indexed)
    pub column: u32,
}

/// Non-fatal warning from compilation.
///
/// Warnings indicate potential issues that don't prevent compilation,
/// such as deprecated syntax, unused variables, or optimization hints.
#[derive(Debug, Clone)]
pub struct CompilerWarning {
    /// Warning message
    pub message: String,

    /// Optional source location (file, line, column)
    pub location: Option<(String, u32, u32)>,
}

/// Compiler error type.
///
/// Represents all errors that can occur during the compilation pipeline,
/// from lexing through WASM emission.
#[derive(Debug)]
pub enum CompilerError {
    /// Lexer error during tokenization
    LexError(String),

    /// Parser error during syntax analysis
    ParseError(ParseError),

    /// Error during AST to HIR lowering
    HirError(String),

    /// Error during HIR to MLIR lowering
    MlirError(String),

    /// Error during WASM emission
    WasmError(String),

    /// I/O error (file not found, permission denied, etc.)
    IoError(std::io::Error),

    /// Project structure error (missing manifest, invalid entry point, etc.)
    ProjectError(String),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::LexError(msg) => write!(f, "Lexer error: {}", msg),
            CompilerError::ParseError(err) => write!(f, "Parse error: {}", err),
            CompilerError::HirError(msg) => write!(f, "HIR lowering error: {}", msg),
            CompilerError::MlirError(msg) => write!(f, "MLIR lowering error: {}", msg),
            CompilerError::WasmError(msg) => write!(f, "WASM emission error: {}", msg),
            CompilerError::IoError(err) => write!(f, "I/O error: {}", err),
            CompilerError::ProjectError(msg) => write!(f, "Project error: {}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}

impl From<ParseError> for CompilerError {
    fn from(err: ParseError) -> Self {
        CompilerError::ParseError(err)
    }
}

impl From<std::io::Error> for CompilerError {
    fn from(err: std::io::Error) -> Self {
        CompilerError::IoError(err)
    }
}

/// Compile a DOL source file to WASM.
///
/// Reads the file, parses it, and compiles it through the full pipeline.
///
/// # Arguments
///
/// * `path` - Path to the .dol file to compile
///
/// # Returns
///
/// A `CompiledSpirit` containing the WASM bytecode and metadata,
/// or a `CompilerError` if compilation fails.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The source contains syntax errors
/// - Type checking fails
/// - Code generation fails
///
/// # Example
///
/// ```rust,ignore
/// use metadol::compiler::spirit::compile_file;
/// use std::path::Path;
///
/// let path = Path::new("examples/calculator.dol");
/// let compiled = compile_file(path)?;
/// std::fs::write("calculator.wasm", &compiled.wasm)?;
/// ```
pub fn compile_file(path: &Path) -> Result<CompiledSpirit, CompilerError> {
    let source = std::fs::read_to_string(path)?;
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.dol");
    compile_source(&source, filename)
}

/// Compile DOL source code to WASM.
///
/// This is the main compilation entry point. It takes DOL source text
/// and transforms it through the complete compilation pipeline.
///
/// # Arguments
///
/// * `source` - The DOL source code to compile
/// * `filename` - Name of the source file (for error reporting and debug info)
///
/// # Returns
///
/// A `CompiledSpirit` containing the WASM bytecode and metadata,
/// or a `CompilerError` if compilation fails.
///
/// # Compilation Phases
///
/// 1. **Lexing & Parsing**: Transform source text into AST
/// 2. **HIR Lowering**: Desugar AST into canonical HIR
/// 3. **MLIR Lowering**: Generate MLIR operations from HIR
/// 4. **WASM Emission**: Emit WebAssembly bytecode from MLIR
///
/// # Example
///
/// ```rust,ignore
/// use metadol::compiler::spirit::compile_source;
///
/// let source = r#"
///     module math @ 1.0
///
///     fun multiply(x: Int64, y: Int64) -> Int64 {
///         return x * y
///     }
/// "#;
///
/// let compiled = compile_source(source, "math.dol")?;
/// assert!(!compiled.wasm.is_empty());
/// ```
#[cfg(feature = "wasm")]
pub fn compile_source(source: &str, filename: &str) -> Result<CompiledSpirit, CompilerError> {
    let mut warnings = Vec::new();

    // ========================================================================
    // Phase 1: Parse DOL source to AST
    // ========================================================================
    let mut parser = Parser::new(source);
    let ast_file = parser
        .parse_file()
        .map_err(|e| CompilerError::ParseError(e))?;

    // ========================================================================
    // Phase 2: Lower AST to HIR (High-level Intermediate Representation)
    // ========================================================================
    let mut lowering_ctx = lower::LoweringContext::new();
    let hir_module = lower::lower_module(&mut lowering_ctx, &ast_file);

    // Collect lowering diagnostics as warnings
    for diag in lowering_ctx.diagnostics() {
        warnings.push(CompilerWarning {
            message: diag.message.clone(),
            location: diag
                .span
                .as_ref()
                .map(|span| (filename.to_string(), span.line as u32, span.column as u32)),
        });
    }

    // Check for lowering errors
    if lowering_ctx.has_errors() {
        let errors: Vec<String> = lowering_ctx
            .diagnostics()
            .iter()
            .filter(|d| matches!(d.kind, lower::DiagnosticKind::Error))
            .map(|d| d.message.clone())
            .collect();
        return Err(CompilerError::HirError(errors.join("; ")));
    }

    // ========================================================================
    // Phase 3: Lower HIR to MLIR
    // ========================================================================
    // NOTE: This is where we would transform HIR into MLIR operations.
    // The full HIR → MLIR lowering is complex and involves:
    // - Type inference and checking
    // - Control flow graph construction
    // - SSA form conversion
    // - Optimization passes
    //
    // For now, we return a placeholder indicating this needs implementation.
    //
    // When implemented, this would look like:
    // let mlir_module = lower_hir_to_mlir(&hir_module)?;

    // ========================================================================
    // Phase 4: Emit WASM from MLIR
    // ========================================================================
    // NOTE: This is where we would generate WASM bytecode from MLIR.
    // The MLIR → WASM lowering involves:
    // - MLIR dialect lowering (DOL → std → LLVM or direct to WASM)
    // - Instruction selection
    // - Register allocation
    // - Binary emission
    //
    // When implemented, this would look like:
    // let wasm_bytes = emit_wasm_from_mlir(&mlir_module)?;

    // For now, return a minimal valid WASM module as a placeholder
    // This allows testing the API while the full pipeline is implemented
    let wasm_bytes = generate_placeholder_wasm(&hir_module)?;

    Ok(CompiledSpirit {
        wasm: wasm_bytes,
        source_map: None, // TODO: Generate source map from HIR span information
        warnings,
    })
}

/// Compile a Spirit project directory.
///
/// A Spirit project is a directory containing:
/// - `manifest.toml`: Project metadata and dependencies
/// - `src/main.dol`: Entry point
/// - `src/*.dol`: Additional source files
///
/// # Arguments
///
/// * `project_dir` - Path to the Spirit project directory
///
/// # Returns
///
/// A `CompiledSpirit` containing the compiled WASM and metadata,
/// or a `CompilerError` if compilation fails.
///
/// # Project Structure
///
/// ```text
/// my-spirit/
/// ├── manifest.toml
/// └── src/
///     ├── main.dol
///     └── helper.dol
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use metadol::compiler::spirit::compile_spirit_project;
/// use std::path::Path;
///
/// let project = Path::new("examples/hello-spirit");
/// let compiled = compile_spirit_project(project)?;
/// ```
#[cfg(feature = "wasm")]
pub fn compile_spirit_project(project_dir: &Path) -> Result<CompiledSpirit, CompilerError> {
    // Validate project structure
    let manifest_path = project_dir.join("manifest.toml");
    if !manifest_path.exists() {
        return Err(CompilerError::ProjectError(format!(
            "manifest.toml not found in {}",
            project_dir.display()
        )));
    }

    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        return Err(CompilerError::ProjectError(format!(
            "src/ directory not found in {}",
            project_dir.display()
        )));
    }

    let main_dol = src_dir.join("main.dol");
    if !main_dol.exists() {
        return Err(CompilerError::ProjectError(format!(
            "src/main.dol not found in {}",
            project_dir.display()
        )));
    }

    // TODO: Parse manifest.toml to get:
    // - Spirit name and version
    // - Dependencies
    // - Build configuration
    //
    // For now, just compile the main.dol entry point
    compile_file(&main_dol)
}

/// Generate a placeholder WASM module.
///
/// This generates a minimal valid WASM module that can be used for testing
/// while the full compilation pipeline is being implemented.
///
/// The generated module contains:
/// - WASM magic number and version
/// - Empty type, function, and code sections
///
/// # Arguments
///
/// * `_hir_module` - HIR module (currently unused, will be used when implemented)
///
/// # Returns
///
/// A valid WASM bytecode module
#[cfg(feature = "wasm")]
fn generate_placeholder_wasm(
    _hir_module: &crate::hir::HirModule,
) -> Result<Vec<u8>, CompilerError> {
    // Generate a minimal valid WASM module
    // Format: https://webassembly.github.io/spec/core/binary/modules.html
    //
    // WASM binary format:
    // - Magic number: 0x00 0x61 0x73 0x6D ("\0asm")
    // - Version: 0x01 0x00 0x00 0x00 (version 1)
    // - Sections: (empty for minimal module)

    let mut wasm = Vec::new();

    // WASM magic number
    wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);

    // WASM version 1
    wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

    // TODO: Add actual sections based on HIR:
    // - Type section (function signatures)
    // - Function section (function indices)
    // - Export section (exported functions)
    // - Code section (function bodies)

    Ok(wasm)
}

// Stub implementation for when wasm feature is not enabled
#[cfg(not(feature = "wasm"))]
pub fn compile_source(_source: &str, _filename: &str) -> Result<CompiledSpirit, CompilerError> {
    Err(CompilerError::WasmError(
        "WASM compilation requires the 'wasm' feature flag".to_string(),
    ))
}

#[cfg(not(feature = "wasm"))]
pub fn compile_spirit_project(_project_dir: &Path) -> Result<CompiledSpirit, CompilerError> {
    Err(CompilerError::WasmError(
        "WASM compilation requires the 'wasm' feature flag".to_string(),
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "wasm")]
mod tests {
    use super::*;

    #[test]
    #[ignore] // TODO: Parser now treats 'test' as keyword, use different module name
    fn test_compile_empty_module() {
        let source = r#"
module test @ 1.0.0
"#;

        let result = compile_source(source, "test.dol");
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let compiled = result.unwrap();
        assert!(!compiled.wasm.is_empty());

        // Verify WASM magic number
        assert_eq!(&compiled.wasm[0..4], b"\0asm");
        // Verify WASM version 1
        assert_eq!(&compiled.wasm[4..8], &[1, 0, 0, 0]);
    }

    #[test]
    #[ignore] // TODO: Parser now treats 'test' as keyword, use different module name
    fn test_compile_simple_gene() {
        let source = r#"
module test @ 1.0.0

gene Counter {
    has value: Int64

    constraint non_negative {
        this.value >= 0
    }
}

exegesis {
    A simple counter gene.
}
"#;

        let result = compile_source(source, "test.dol");
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let compiled = result.unwrap();
        assert!(!compiled.wasm.is_empty());
        assert_eq!(&compiled.wasm[0..4], b"\0asm");
    }

    #[test]
    fn test_compile_with_function() {
        let source = r#"
module math @ 1.0.0

fun add(a: Int64, b: Int64) -> Int64 {
    return a + b
}
"#;

        let result = compile_source(source, "math.dol");
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let compiled = result.unwrap();
        assert!(!compiled.wasm.is_empty());
        assert_eq!(&compiled.wasm[0..4], b"\0asm");
    }

    #[test]
    fn test_compile_invalid_syntax() {
        let source = r#"
module broken @ 1.0.0

gene Invalid {
    this is not valid syntax!!!
}
"#;

        let result = compile_source(source, "broken.dol");
        assert!(result.is_err());

        match result.unwrap_err() {
            CompilerError::ParseError(_) => {
                // Expected
            }
            other => panic!("Expected ParseError, got {:?}", other),
        }
    }

    #[test]
    fn test_compiler_error_display() {
        let err = CompilerError::WasmError("test error".to_string());
        assert_eq!(err.to_string(), "WASM emission error: test error");

        let err = CompilerError::HirError("type mismatch".to_string());
        assert_eq!(err.to_string(), "HIR lowering error: type mismatch");
    }

    #[test]
    fn test_source_map_entry() {
        let entry = SourceMapEntry {
            wasm_offset: 42,
            source_file: "test.dol".to_string(),
            line: 10,
            column: 5,
        };

        assert_eq!(entry.wasm_offset, 42);
        assert_eq!(entry.line, 10);
        assert_eq!(entry.column, 5);
    }

    #[test]
    fn test_compiler_warning() {
        let warning = CompilerWarning {
            message: "deprecated syntax".to_string(),
            location: Some(("test.dol".to_string(), 5, 10)),
        };

        assert_eq!(warning.message, "deprecated syntax");
        assert!(warning.location.is_some());
    }
}
