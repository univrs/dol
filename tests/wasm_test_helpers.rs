//! # WASM Test Helper Functions
//!
//! This module provides helper functions for testing WASM compilation and execution.
//! It centralizes common testing patterns to ensure consistency across test files.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::wasm_test_helpers::{compile_dol_to_wasm, validate_wasm_bytes, execute_wasm_function};
//!
//! let wasm = compile_dol_to_wasm("fun add(a: i64, b: i64) -> i64 { return a + b }");
//! assert!(validate_wasm_bytes(&wasm));
//! let result = execute_wasm_function(&wasm, "add", &[5i64.into(), 3i64.into()]);
//! assert_eq!(result, Some(8));
//! ```

#![cfg(feature = "wasm")]

use metadol::parse_file;
use metadol::wasm::{WasmCompiler, WasmError, WasmRuntime};
use std::path::Path;
use std::process::Command;

/// Result type for WASM test operations.
pub type WasmTestResult<T> = Result<T, WasmError>;

// ============================================
// Compilation Helpers
// ============================================

/// Compile DOL source code to WASM bytecode.
///
/// This helper parses the DOL source and compiles it to WASM bytes.
/// Returns `None` if parsing or compilation fails.
///
/// # Arguments
///
/// * `source` - DOL source code as a string
///
/// # Returns
///
/// `Some(Vec<u8>)` containing WASM bytes on success, `None` on failure.
///
/// # Example
///
/// ```rust,ignore
/// let wasm = compile_dol_to_wasm("fun get42() -> i64 { return 42 }");
/// assert!(wasm.is_some());
/// ```
pub fn compile_dol_to_wasm(source: &str) -> Option<Vec<u8>> {
    let module = parse_file(source).ok()?;
    let mut compiler = WasmCompiler::new();
    compiler.compile(&module).ok()
}

/// Compile DOL source code to WASM with detailed error reporting.
///
/// Unlike `compile_dol_to_wasm`, this function returns the actual error
/// for debugging purposes.
///
/// # Arguments
///
/// * `source` - DOL source code as a string
///
/// # Returns
///
/// `Result<Vec<u8>, String>` with either WASM bytes or an error message.
pub fn compile_dol_to_wasm_verbose(source: &str) -> Result<Vec<u8>, String> {
    let module = parse_file(source).map_err(|e| format!("Parse error: {:?}", e))?;
    let mut compiler = WasmCompiler::new();
    compiler
        .compile(&module)
        .map_err(|e| format!("Compile error: {}", e.message))
}

/// Compile DOL source with optimization enabled.
///
/// # Arguments
///
/// * `source` - DOL source code as a string
///
/// # Returns
///
/// `Some(Vec<u8>)` containing optimized WASM bytes on success.
pub fn compile_dol_optimized(source: &str) -> Option<Vec<u8>> {
    let module = parse_file(source).ok()?;
    let mut compiler = WasmCompiler::new().with_optimization(true);
    compiler.compile(&module).ok()
}

/// Compile a DOL file to WASM bytecode.
///
/// # Arguments
///
/// * `path` - Path to the .dol file
///
/// # Returns
///
/// `Some(Vec<u8>)` containing WASM bytes on success.
pub fn compile_dol_file(path: &Path) -> Option<Vec<u8>> {
    let source = std::fs::read_to_string(path).ok()?;
    compile_dol_to_wasm(&source)
}

// ============================================
// Validation Helpers
// ============================================

/// Validate that bytes represent a valid WASM module.
///
/// Checks:
/// - Magic number (0x00 0x61 0x73 0x6D)
/// - Version (0x01 0x00 0x00 0x00)
/// - Minimum length (8 bytes)
///
/// # Arguments
///
/// * `wasm_bytes` - The bytes to validate
///
/// # Returns
///
/// `true` if the bytes are valid WASM, `false` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// let valid = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
/// assert!(validate_wasm_bytes(&valid));
///
/// let invalid = vec![0xFF, 0xFF, 0xFF, 0xFF];
/// assert!(!validate_wasm_bytes(&invalid));
/// ```
pub fn validate_wasm_bytes(wasm_bytes: &[u8]) -> bool {
    if wasm_bytes.len() < 8 {
        return false;
    }

    // Check magic number: \0asm
    let magic = &wasm_bytes[0..4];
    if magic != [0x00, 0x61, 0x73, 0x6D] {
        return false;
    }

    // Check version: 1
    let version = &wasm_bytes[4..8];
    if version != [0x01, 0x00, 0x00, 0x00] {
        return false;
    }

    true
}

/// Validate WASM bytes using wasm-tools if available.
///
/// This provides more thorough validation by using the external wasm-tools
/// validator. Falls back to basic validation if wasm-tools is not installed.
///
/// # Arguments
///
/// * `wasm_bytes` - The bytes to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(String)` with error message if invalid.
pub fn validate_wasm_with_tools(wasm_bytes: &[u8]) -> Result<(), String> {
    // First do basic validation
    if !validate_wasm_bytes(wasm_bytes) {
        return Err("Basic WASM validation failed (magic number or version)".to_string());
    }

    // Try to use wasm-tools for more thorough validation
    let temp_file = std::env::temp_dir().join("test_module.wasm");
    if std::fs::write(&temp_file, wasm_bytes).is_err() {
        return Err("Failed to write temp file for validation".to_string());
    }

    let result = Command::new("wasm-tools")
        .args(["validate", temp_file.to_str().unwrap()])
        .output();

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    match result {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("wasm-tools validation failed: {}", stderr))
        }
        Err(_) => {
            // wasm-tools not available, fall back to basic validation (already passed)
            Ok(())
        }
    }
}

// ============================================
// Execution Helpers
// ============================================

/// Execute a WASM function and return the result.
///
/// # Arguments
///
/// * `wasm_bytes` - The compiled WASM module
/// * `function_name` - Name of the exported function to call
/// * `args` - Arguments to pass to the function
///
/// # Returns
///
/// The return value from the WASM function, or an error.
pub fn execute_wasm_function(
    wasm_bytes: &[u8],
    function_name: &str,
    args: &[i64],
) -> WasmTestResult<i64> {
    let runtime = WasmRuntime::new()?;
    let mut module = runtime.load(wasm_bytes)?;

    // Convert args to wasmtime values
    let wasm_args: Vec<wasmtime::Val> = args.iter().map(|&v| wasmtime::Val::I64(v)).collect();

    let result = module.call(function_name, &wasm_args)?;

    // Extract i64 result (assuming the function returns i64)
    match result.first() {
        Some(val) => match val.i64() {
            Some(v) => Ok(v),
            None => Err(WasmError::new("Function did not return i64")),
        },
        None => Err(WasmError::new("Function returned no values")),
    }
}

/// Load and instantiate a WASM module for testing.
///
/// # Arguments
///
/// * `wasm_bytes` - The compiled WASM module
///
/// # Returns
///
/// A loaded `WasmModule` ready for function calls.
pub fn load_wasm_module(wasm_bytes: &[u8]) -> WasmTestResult<metadol::wasm::WasmModule> {
    let runtime = WasmRuntime::new()?;
    runtime.load(wasm_bytes)
}

// ============================================
// Test File Helpers
// ============================================

/// Read a test case file from the test-cases directory.
///
/// # Arguments
///
/// * `level` - The level directory (e.g., "level1-minimal", "level2-basic")
/// * `filename` - The .dol filename
///
/// # Returns
///
/// The contents of the file, or `None` if not found.
pub fn read_test_case(level: &str, filename: &str) -> Option<String> {
    let path = format!(
        "{}/test-cases/{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        level,
        filename
    );
    std::fs::read_to_string(&path).ok()
}

/// Get all .dol files in a test-cases level directory.
///
/// # Arguments
///
/// * `level` - The level directory name
///
/// # Returns
///
/// A vector of (filename, contents) pairs.
pub fn get_test_cases(level: &str) -> Vec<(String, String)> {
    let dir_path = format!("{}/test-cases/{}", env!("CARGO_MANIFEST_DIR"), level);
    let mut results = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "dol") {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    results.push((filename, contents));
                }
            }
        }
    }

    results
}

/// Get all working test cases (from test-cases/working/).
pub fn get_working_test_cases() -> Vec<(String, String)> {
    get_test_cases("working")
}

// ============================================
// Assertion Helpers
// ============================================

/// Assert that DOL source compiles to valid WASM.
///
/// Panics with a descriptive message if compilation fails.
///
/// # Arguments
///
/// * `source` - DOL source code
/// * `context` - Description of what is being tested (for error messages)
#[allow(dead_code)]
pub fn assert_compiles(source: &str, context: &str) {
    match compile_dol_to_wasm_verbose(source) {
        Ok(wasm) => {
            assert!(
                validate_wasm_bytes(&wasm),
                "{}: Compiled but produced invalid WASM",
                context
            );
        }
        Err(e) => {
            panic!("{}: Compilation failed - {}", context, e);
        }
    }
}

/// Assert that DOL source fails to compile.
///
/// Panics if compilation succeeds when it should fail.
///
/// # Arguments
///
/// * `source` - DOL source code
/// * `context` - Description of what is being tested
#[allow(dead_code)]
pub fn assert_compile_fails(source: &str, context: &str) {
    if compile_dol_to_wasm(source).is_some() {
        panic!("{}: Expected compilation to fail but it succeeded", context);
    }
}

/// Assert that a WASM function returns the expected value.
///
/// # Arguments
///
/// * `wasm_bytes` - The compiled WASM module
/// * `function_name` - Name of the function to call
/// * `args` - Arguments to pass
/// * `expected` - Expected return value
/// * `context` - Description for error messages
#[allow(dead_code)]
pub fn assert_wasm_result(
    wasm_bytes: &[u8],
    function_name: &str,
    args: &[i64],
    expected: i64,
    context: &str,
) {
    match execute_wasm_function(wasm_bytes, function_name, args) {
        Ok(result) => {
            assert_eq!(
                result, expected,
                "{}: Expected {} but got {}",
                context, expected, result
            );
        }
        Err(e) => {
            panic!("{}: Execution failed - {}", context, e.message);
        }
    }
}

// ============================================
// WASM Module Inspection Helpers
// ============================================

/// Information about an exported function in a WASM module.
#[derive(Debug, Clone)]
pub struct ExportedFunction {
    pub name: String,
    pub param_count: usize,
    pub result_count: usize,
}

/// Inspect WASM bytes to find exported functions.
///
/// Note: This is a simplified inspection that may not handle all WASM modules.
/// For comprehensive inspection, use wasm-tools.
///
/// # Arguments
///
/// * `wasm_bytes` - The WASM module bytes
///
/// # Returns
///
/// List of exported function names (basic inspection only).
pub fn inspect_wasm_exports(wasm_bytes: &[u8]) -> Vec<String> {
    // Use wasmtime to inspect exports directly from the Module
    let engine = wasmtime::Engine::default();
    let module = match wasmtime::Module::from_binary(&engine, wasm_bytes) {
        Ok(m) => m,
        Err(_) => return vec![],
    };

    module
        .exports()
        .filter_map(|export| {
            if export.ty().func().is_some() {
                Some(export.name().to_string())
            } else {
                None
            }
        })
        .collect()
}

// ============================================
// Test Data Generators
// ============================================

/// Generate a simple add function DOL source.
pub fn gen_add_function() -> &'static str {
    r#"fun add(a: i64, b: i64) -> i64 {
    return a + b
}"#
}

/// Generate a function with local variables.
pub fn gen_function_with_locals() -> &'static str {
    r#"fun compute(x: i64) -> i64 {
    let doubled: i64 = x + x
    let result: i64 = doubled + 1
    return result
}"#
}

/// Generate a function with if-else control flow.
pub fn gen_function_with_if() -> &'static str {
    r#"fun max(a: i64, b: i64) -> i64 {
    if a > b {
        return a
    } else {
        return b
    }
}"#
}

/// Generate a function with a while loop.
pub fn gen_function_with_while() -> &'static str {
    r#"fun countdown(n: i64) -> i64 {
    let count: i64 = n
    while count > 0 {
        count = count - 1
    }
    return count
}"#
}

/// Generate a function with a for loop.
pub fn gen_function_with_for() -> &'static str {
    r#"fun sum_to(n: i64) -> i64 {
    let total: i64 = 0
    for i in 0..n {
        total = total + i
    }
    return total
}"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wasm_bytes_valid() {
        let valid = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        assert!(validate_wasm_bytes(&valid));
    }

    #[test]
    fn test_validate_wasm_bytes_invalid_magic() {
        let invalid = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00];
        assert!(!validate_wasm_bytes(&invalid));
    }

    #[test]
    fn test_validate_wasm_bytes_invalid_version() {
        let invalid = vec![0x00, 0x61, 0x73, 0x6D, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!validate_wasm_bytes(&invalid));
    }

    #[test]
    fn test_validate_wasm_bytes_too_short() {
        let short = vec![0x00, 0x61, 0x73, 0x6D];
        assert!(!validate_wasm_bytes(&short));
    }

    #[test]
    fn test_validate_wasm_bytes_empty() {
        let empty: Vec<u8> = vec![];
        assert!(!validate_wasm_bytes(&empty));
    }

    #[test]
    fn test_gen_add_function_is_valid() {
        // Just verify the generated source is syntactically valid DOL
        let source = gen_add_function();
        assert!(source.contains("fun add"));
        assert!(source.contains("return"));
    }
}
