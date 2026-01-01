//! WASM execution and runtime tests.
//!
//! These tests verify WASM module validation and execution using the
//! Wasmtime runtime. All tests are feature-gated with `#[cfg(feature = "wasm")]`.
//!
//! Tests are organized by:
//! 1. WASM module validation (magic number, version)
//! 2. WASM runtime initialization
//! 3. WASM execution (when wasmtime is available)
//! 4. WASM error handling

#![cfg(feature = "wasm")]

use metadol::parse_file;
use metadol::wasm::{WasmCompiler, WasmError, WasmRuntime};

// ============================================
// 1. WASM Module Validation Tests
// ============================================

#[test]
fn test_wasm_magic_number() {
    // Valid WASM binary must start with magic number: 0x00 0x61 0x73 0x6D (\0asm)
    let valid_wasm = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // The magic number should be "\0asm"
    assert_eq!(&valid_wasm[0..4], b"\0asm");
}

#[test]
fn test_wasm_version() {
    // Valid WASM binary version is 1 (0x01 0x00 0x00 0x00 in little-endian)
    let valid_wasm = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // Bytes 4-7 should encode version 1
    let version = u32::from_le_bytes([valid_wasm[4], valid_wasm[5], valid_wasm[6], valid_wasm[7]]);
    assert_eq!(version, 1);
}

#[test]
fn test_wasm_invalid_magic_number() {
    // Invalid WASM binary with wrong magic number
    let invalid_wasm = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let result = runtime.load(&invalid_wasm);

    // Should fail with invalid magic number
    assert!(result.is_err());
}

#[test]
fn test_wasm_invalid_version() {
    // Invalid WASM binary with wrong version
    let invalid_wasm = vec![0x00, 0x61, 0x73, 0x6D, 0xFF, 0xFF, 0xFF, 0xFF];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let result = runtime.load(&invalid_wasm);

    // Should fail with invalid version
    assert!(result.is_err());
}

#[test]
fn test_wasm_truncated_module() {
    // Truncated WASM binary (only magic number, no version)
    let truncated_wasm = vec![0x00, 0x61, 0x73, 0x6D];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let result = runtime.load(&truncated_wasm);

    // Should fail with truncated module
    assert!(result.is_err());
}

#[test]
fn test_wasm_empty_module() {
    // Empty WASM binary
    let empty_wasm = vec![];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let result = runtime.load(&empty_wasm);

    // Should fail with empty module
    assert!(result.is_err());
}

// ============================================
// 2. WASM Runtime Initialization Tests
// ============================================

#[test]
fn test_wasm_runtime_new() {
    let result = WasmRuntime::new();
    assert!(
        result.is_ok(),
        "Failed to create WASM runtime: {:?}",
        result.err()
    );
}

#[test]
fn test_wasm_runtime_multiple_instances() {
    // Should be able to create multiple runtime instances
    let runtime1 = WasmRuntime::new();
    let runtime2 = WasmRuntime::new();

    assert!(runtime1.is_ok());
    assert!(runtime2.is_ok());
}

// ============================================
// 3. WASM Execution Tests
// ============================================

#[test]
fn test_wasm_load_minimal_module() {
    // Minimal valid WASM module with just magic number and version
    let minimal_wasm = vec![
        0x00, 0x61, 0x73, 0x6D, // Magic number: \0asm
        0x01, 0x00, 0x00, 0x00, // Version: 1
    ];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let result = runtime.load(&minimal_wasm);

    // Loading minimal module should succeed
    assert!(
        result.is_ok(),
        "Failed to load minimal WASM module: {:?}",
        result.err()
    );
}

#[test]
fn test_wasm_module_with_function() {
    // WASM module with a simple function that returns 42
    let wasm_with_func = vec![
        0x00, 0x61, 0x73, 0x6D, // Magic number
        0x01, 0x00, 0x00, 0x00, // Version
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7F, // Type section: () -> i32
        0x03, 0x02, 0x01, 0x00, // Function section: 1 function with type 0
        0x07, 0x09, 0x01, 0x05, 0x67, 0x65, 0x74, 0x34, 0x32, 0x00,
        0x00, // Export section: "get42"
        0x0A, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2A, 0x0B, // Code section: return 42
    ];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let result = runtime.load(&wasm_with_func);

    assert!(
        result.is_ok(),
        "Failed to load WASM module with function: {:?}",
        result.err()
    );

    let mut module = result.unwrap();

    // Try to call the exported function
    let call_result = module.call("get42", &[]);

    assert!(
        call_result.is_ok(),
        "Failed to call WASM function: {:?}",
        call_result.err()
    );
}

#[test]
fn test_wasm_call_nonexistent_function() {
    // Minimal WASM module without any exported functions
    let minimal_wasm = vec![
        0x00, 0x61, 0x73, 0x6D, // Magic number
        0x01, 0x00, 0x00, 0x00, // Version
    ];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut module = runtime.load(&minimal_wasm).expect("Failed to load module");

    // Try to call a function that doesn't exist
    let result = module.call("nonexistent", &[]);

    assert!(result.is_err(), "Should fail to call nonexistent function");
}

// ============================================
// 4. WASM Compiler Integration Tests
// ============================================

#[test]
fn test_wasm_compiler_error_message() {
    let source = r#"
gene Counter {
    has value: Int64
}
exegesis { A counter. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let result = compiler.compile(&module);

    // Should return error when compiling genes (only functions are supported)
    assert!(result.is_err());

    let err = result.unwrap_err();
    // The error message changed - now it's about requiring functions
    assert!(
        err.message.contains("No functions found") || err.message.contains("not fully implemented"),
        "Expected function-related error, got: {}",
        err.message
    );
}

#[test]
fn test_wasm_compiler_with_optimization() {
    let source = r#"
fun add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
exegesis { Adds two integers. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new().with_optimization(true);
    let result = compiler.compile(&module);

    // Compilation should succeed now that Int32 is supported
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

    // Verify the output is valid WASM
    let wasm_bytes = result.unwrap();
    assert!(wasm_bytes.len() >= 8, "WASM output too short");
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Invalid WASM magic number");
}

#[test]
fn test_wasm_compiler_default() {
    let _compiler = WasmCompiler::default();

    // Default compiler should be created successfully
}

// ============================================
// 5. Error Type Tests
// ============================================

#[test]
fn test_wasm_error_new() {
    let error = WasmError::new("Test error");
    assert_eq!(error.message, "Test error");
}

#[test]
fn test_wasm_error_display() {
    let error = WasmError::new("Test error");
    let display = format!("{}", error);

    assert!(display.contains("WASM error"));
    assert!(display.contains("Test error"));
}

#[test]
fn test_wasm_error_from_io() {
    use std::io;

    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let wasm_error: WasmError = io_error.into();

    assert!(wasm_error.message.contains("I/O error"));
    assert!(wasm_error.message.contains("File not found"));
}

// ============================================
// 6. Future: Full Pipeline Tests
// ============================================

// These tests are placeholders for when the full compilation pipeline is implemented

#[test]
fn test_compile_and_execute_simple_function() {
    let source = r#"
fun add(a: i64, b: i64) -> i64 {
    return a + b
}
exegesis { Adds two integers. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Compile to WASM
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Call the function
    let result = wasm_module
        .call("add", &[5i64.into(), 3i64.into()])
        .expect("Call failed");

    // Verify result - WASM returns i64
    assert_eq!(result.first().and_then(|v| v.i64()), Some(8));
}

#[test]
fn test_compile_and_execute_gene_method_with_field_access() {
    let source = r#"
gene Counter {
    has value: Int64

    fun increment() -> Int64 {
        return value + 1
    }
}
exegesis { A counter with increment method. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Compile to WASM
    let mut compiler = WasmCompiler::new().with_optimization(true);
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // The method now takes an implicit 'self' parameter (i32 pointer to Counter instance)
    // For this test, we pass 0 as the self pointer (memory address of a Counter)
    // The value at that address will be read (likely 0 from uninitialized memory)
    // So value + 1 should return 1
    let result = wasm_module
        .call("Counter.increment", &[0i32.into()])
        .expect("Call failed");

    // The result should be value (0) + 1 = 1
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));
}

#[test]
fn test_compile_and_execute_gene_method_simple() {
    // Test gene method that doesn't require field access
    let source = r#"
gene Math {
    fun add(a: i64, b: i64) -> i64 {
        return a + b
    }

    fun multiply(x: i64, y: i64) -> i64 {
        return x * y
    }
}
exegesis { Simple math operations. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Compile to WASM
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Call Math.add
    let result = wasm_module
        .call("Math.add", &[5i64.into(), 3i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(8));

    // Call Math.multiply
    let result = wasm_module
        .call("Math.multiply", &[6i64.into(), 7i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));
}

#[test]
fn test_compile_with_control_flow() {
    let source = r#"
fun max(a: i64, b: i64) -> i64 {
    if a > b {
        return a
    }
    return b
}
exegesis { Returns the maximum of two integers. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test with a > b
    let result = wasm_module
        .call("max", &[10i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(10));

    // Test with a < b
    let result = wasm_module
        .call("max", &[3i64.into(), 7i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(7));
}

#[test]
fn test_compile_with_pattern_matching() {
    let source = r#"
fun classify(x: i64) -> i64 {
    match x {
        0 => 100,
        1 => 200,
        _ => 300,
    }
}
exegesis { Classifies an integer. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test each case
    let result = wasm_module
        .call("classify", &[0i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(100));

    let result = wasm_module
        .call("classify", &[1i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(200));

    let result = wasm_module
        .call("classify", &[42i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(300));
}

// ============================================
// 7. Performance and Stress Tests
// ============================================

#[test]
fn test_wasm_runtime_many_modules() {
    let minimal_wasm = vec![
        0x00, 0x61, 0x73, 0x6D, // Magic number
        0x01, 0x00, 0x00, 0x00, // Version
    ];

    let runtime = WasmRuntime::new().expect("Failed to create runtime");

    // Load the same module multiple times
    for _ in 0..10 {
        let result = runtime.load(&minimal_wasm);
        assert!(result.is_ok(), "Failed to load module");
    }
}

#[test]
#[ignore] // Remove this when performance testing is needed
fn test_wasm_compilation_performance() {
    use std::time::Instant;

    let source = r#"
gene LargeGene {
    has field1: Int64
    has field2: Int64
    has field3: Int64
    has field4: Int64
    has field5: Int64

    fun method1() -> Int64 { return field1 }
    fun method2() -> Int64 { return field2 }
    fun method3() -> Int64 { return field3 }
    fun method4() -> Int64 { return field4 }
    fun method5() -> Int64 { return field5 }
}
exegesis { A large gene for performance testing. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new().with_optimization(true);

    let start = Instant::now();
    let _result = compiler.compile(&module);
    let duration = start.elapsed();

    // Compilation should complete in reasonable time (< 5 seconds)
    assert!(
        duration.as_secs() < 5,
        "Compilation took too long: {:?}",
        duration
    );
}
