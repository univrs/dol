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

use metadol::parse_dol_file;
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

// ============================================
// 8. Loop and Variable Reassignment Tests
// ============================================

#[test]
fn test_compile_and_execute_while_loop_sum() {
    // Test while loop with variable reassignment: sum of 0+1+2+3+4 = 10
    let source = r#"
fun test_while_sum(n: i64) -> i64 {
    let total: i64 = 0
    let i: i64 = 0
    while i < n {
        total = total + i
        i = i + 1
    }
    return total
}
exegesis { Sums integers from 0 to n-1 using a while loop. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // test_while_sum(5) should return 0+1+2+3+4 = 10
    let result = wasm_module
        .call("test_while_sum", &[5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(10));

    // test_while_sum(1) should return 0
    let result = wasm_module
        .call("test_while_sum", &[1i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));

    // test_while_sum(0) should return 0 (never enters loop)
    let result = wasm_module
        .call("test_while_sum", &[0i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

#[test]
fn test_compile_and_execute_for_loop_sum() {
    // Test for loop: sum of 0..n
    let source = r#"
fun test_for_sum(n: i64) -> i64 {
    let total: i64 = 0
    for i in 0..n {
        total = total + i
    }
    return total
}
exegesis { Sums integers from 0 to n-1 using a for loop. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // test_for_sum(5) should return 0+1+2+3+4 = 10
    let result = wasm_module
        .call("test_for_sum", &[5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(10));
}

#[test]
fn test_compile_and_execute_loop_with_break() {
    // Test infinite loop with break
    let source = r#"
fun test_loop_break(target: i64) -> i64 {
    let counter: i64 = 0
    loop {
        counter = counter + 1
        if counter >= target {
            break
        }
    }
    return counter
}
exegesis { Counts up until reaching target using loop with break. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // test_loop_break(5) should return 5
    let result = wasm_module
        .call("test_loop_break", &[5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(5));
}

#[test]
fn test_compile_and_execute_variable_reassignment() {
    // Test basic variable reassignment
    let source = r#"
fun test_reassign(x: i64) -> i64 {
    let a: i64 = x
    a = a + 10
    a = a * 2
    return a
}
exegesis { Tests variable reassignment. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // test_reassign(5) should return (5 + 10) * 2 = 30
    let result = wasm_module
        .call("test_reassign", &[5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(30));
}

#[test]
fn test_compile_and_execute_factorial() {
    // Test for loop computing factorial
    let source = r#"
fun factorial(n: i64) -> i64 {
    let result: i64 = 1
    for i in 1..n {
        result = result * i
    }
    return result
}
exegesis { Computes factorial of n-1 using for loop. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // factorial(5) = 1*2*3*4 = 24 (range is exclusive: 1..5 means 1,2,3,4)
    let result = wasm_module
        .call("factorial", &[5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(24));

    // factorial(1) = 1 (empty range)
    let result = wasm_module
        .call("factorial", &[1i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));
}

// ============================================
// 8. Gene Inheritance Tests
// ============================================

#[test]
fn test_compile_gene_inheritance_layout() {
    use metadol::Parser;

    // Parse a file with parent and child genes
    let source = r#"
module inheritance_test @ 0.1.0

gene Animal {
    has age: i64

    fun get_age() -> i64 {
        return age
    }
}

gene Dog extends Animal {
    has breed_id: i64

    fun bark_count() -> i64 {
        return age * 2
    }
}
"#;

    let mut parser = Parser::new(source);
    let dol_file = parser.parse_file().expect("Failed to parse");

    // Compile the file - should handle inheritance ordering
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler
        .compile_file(&dol_file)
        .expect("Compilation failed");

    // Verify WASM is valid
    assert!(wasm_bytes.len() > 8, "WASM should have content");
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Should have WASM magic");

    // Load and verify the module
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load WASM");

    // Test parent method - pass 0 as self pointer (memory starts at 0, initialized to 0)
    // age (at offset 0) will be 0, so get_age() returns 0
    let result = wasm_module
        .call("Animal.get_age", &[0i32.into()])
        .expect("Call Animal.get_age failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

#[test]
fn test_compile_gene_inheritance_child_access_parent_field() {
    use metadol::Parser;

    let source = r#"
module inheritance_test @ 0.1.0

gene Animal {
    has age: i64
}

gene Dog extends Animal {
    has breed_id: i64

    fun bark_count() -> i64 {
        return age * 2
    }
}
"#;

    let mut parser = Parser::new(source);
    let dol_file = parser.parse_file().expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler
        .compile_file(&dol_file)
        .expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load WASM");

    // Dog layout: age at offset 0 (inherited), breed_id at offset 8
    // Memory starts at 0, so age = 0
    // bark_count() returns age * 2 = 0 * 2 = 0
    let result = wasm_module
        .call("Dog.bark_count", &[0i32.into()])
        .expect("Call Dog.bark_count failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

#[test]
fn test_compile_gene_inheritance_reverse_order() {
    use metadol::Parser;

    // Child is declared BEFORE parent - should still work due to topological sort
    let source = r#"
module inheritance_test @ 0.1.0

gene Dog extends Animal {
    has breed_id: i64

    fun get_breed() -> i64 {
        return breed_id
    }
}

gene Animal {
    has age: i64

    fun get_age() -> i64 {
        return age
    }
}
"#;

    let mut parser = Parser::new(source);
    let dol_file = parser.parse_file().expect("Failed to parse");

    // This should still compile - the compiler should sort genes by dependency
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler
        .compile_file(&dol_file)
        .expect("Compilation failed with reverse order");

    // Verify module is loadable
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let _wasm_module = runtime.load(&wasm_bytes).expect("Failed to load WASM");
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

// ============================================
// 10. Enum Type Tests
// ============================================

#[test]
fn test_enum_variant_access() {
    // Test accessing enum variants which should compile to i32 constants
    // Note: Functions return the enum type (which maps to i32 in WASM)
    let source = r#"
fun get_node() -> AccountType {
    return AccountType.Node
}
exegesis { Function returning enum variant discriminant. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Create compiler and register the enum
    let mut compiler = WasmCompiler::new();
    compiler.register_enum(
        "AccountType",
        vec![
            "Node".to_string(),
            "RevivalPool".to_string(),
            "Treasury".to_string(),
        ],
    );

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test enum variant - should return discriminant index 0
    let result = wasm_module.call("get_node", &[]).expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i32()),
        Some(0),
        "Node should be index 0"
    );
}

#[test]
fn test_enum_variant_revival_pool() {
    // Test second enum variant
    let source = r#"
fun get_revival_pool() -> AccountType {
    return AccountType.RevivalPool
}
exegesis { Function returning RevivalPool variant. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    compiler.register_enum(
        "AccountType",
        vec![
            "Node".to_string(),
            "RevivalPool".to_string(),
            "Treasury".to_string(),
        ],
    );

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("get_revival_pool", &[])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i32()),
        Some(1),
        "RevivalPool should be index 1"
    );
}

#[test]
fn test_enum_variant_treasury() {
    // Test third enum variant
    let source = r#"
fun get_treasury() -> AccountType {
    return AccountType.Treasury
}
exegesis { Function returning Treasury variant. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    compiler.register_enum(
        "AccountType",
        vec![
            "Node".to_string(),
            "RevivalPool".to_string(),
            "Treasury".to_string(),
        ],
    );

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module.call("get_treasury", &[]).expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i32()),
        Some(2),
        "Treasury should be index 2"
    );
}

#[test]
fn test_enum_type_mapping() {
    // Test that enum types are correctly mapped to i32 in function signatures
    let source = r#"
fun compare_account_type(a: AccountType, b: AccountType) -> i32 {
    if a == b {
        return 1
    }
    return 0
}
exegesis { Compare two account types. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Create compiler and register the enum
    let mut compiler = WasmCompiler::new();
    compiler.register_enum(
        "AccountType",
        vec![
            "Node".to_string(),
            "RevivalPool".to_string(),
            "Treasury".to_string(),
        ],
    );

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test comparing same enum values
    // Note: DOL `i32` return type maps to WASM i64 for uniformity
    let result = wasm_module
        .call("compare_account_type", &[0i32.into(), 0i32.into()])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(1),
        "Same values should be equal"
    );

    // Test comparing different enum values
    let result = wasm_module
        .call("compare_account_type", &[0i32.into(), 1i32.into()])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(0),
        "Different values should not be equal"
    );
}

#[test]
fn test_multiple_enums() {
    // Test with multiple enum types registered
    // Note: Functions return enum types (which map to i32 in WASM)
    let source = r#"
fun get_cpu_resource() -> ResourceType {
    return ResourceType.Cpu
}
exegesis { Gets CPU resource type. }

fun get_storage_resource() -> ResourceType {
    return ResourceType.Storage
}
exegesis { Gets storage resource type. }

fun get_admin_role() -> Role {
    return Role.Admin
}
exegesis { Gets admin role. }
"#;
    let file = parse_dol_file(source).expect("Failed to parse");

    // Create compiler and register multiple enums
    let mut compiler = WasmCompiler::new();
    compiler.register_enum(
        "ResourceType",
        vec![
            "Cpu".to_string(),
            "Memory".to_string(),
            "Gpu".to_string(),
            "Storage".to_string(),
            "Bandwidth".to_string(),
        ],
    );
    compiler.register_enum(
        "Role",
        vec!["Admin".to_string(), "User".to_string(), "Guest".to_string()],
    );

    let wasm_bytes = compiler.compile_file(&file).expect("Compilation failed");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test ResourceType variants
    let result = wasm_module
        .call("get_cpu_resource", &[])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i32()),
        Some(0),
        "Cpu should be index 0"
    );

    let result = wasm_module
        .call("get_storage_resource", &[])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i32()),
        Some(3),
        "Storage should be index 3"
    );

    // Test Role variants
    let result = wasm_module
        .call("get_admin_role", &[])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i32()),
        Some(0),
        "Admin should be index 0"
    );
}

#[test]
fn test_enum_registry_api() {
    // Test the enum registry API on the compiler
    let mut compiler = WasmCompiler::new();

    // Should not have any enums initially
    assert!(!compiler.has_enum("AccountType"));

    // Register an enum
    compiler.register_enum(
        "AccountType",
        vec![
            "Node".to_string(),
            "RevivalPool".to_string(),
            "Treasury".to_string(),
        ],
    );

    // Should now have the enum
    assert!(compiler.has_enum("AccountType"));
    assert!(!compiler.has_enum("UnknownType"));

    // Should be able to get variant indices
    assert_eq!(
        compiler.get_enum_variant_index("AccountType", "Node"),
        Some(0)
    );
    assert_eq!(
        compiler.get_enum_variant_index("AccountType", "RevivalPool"),
        Some(1)
    );
    assert_eq!(
        compiler.get_enum_variant_index("AccountType", "Treasury"),
        Some(2)
    );

    // Unknown variants should return None
    assert_eq!(
        compiler.get_enum_variant_index("AccountType", "Unknown"),
        None
    );
    assert_eq!(compiler.get_enum_variant_index("UnknownType", "Node"), None);
}

// ============================================
// 12. String Type Tests
// ============================================

#[test]
fn test_compile_string_literal_returns_pointer() {
    // Test that string literals compile to return an i32 pointer
    let source = r#"
fun get_greeting() -> String {
    return "Hello, World!"
}
exegesis { Returns a greeting string. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let result = compiler.compile(&module);

    assert!(
        result.is_ok(),
        "String literal compilation failed: {:?}",
        result.err()
    );

    let wasm_bytes = result.unwrap();

    // Verify the output is valid WASM
    assert!(wasm_bytes.len() >= 8, "WASM output too short");
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Invalid WASM magic number");

    // Load and execute
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // The function returns a pointer (i32) to the string in memory
    let result = wasm_module.call("get_greeting", &[]).expect("Call failed");

    // Verify we get an i32 pointer back (the address where the string is stored)
    let ptr = result
        .first()
        .and_then(|v| v.i32())
        .expect("Expected i32 result");
    // The string should be at offset 0 in the data section
    assert_eq!(ptr, 0, "Expected string pointer at offset 0");
}

#[test]
#[ignore] // Requires get_memory() implementation
fn test_string_literal_in_data_section() {
    // Test that string data is properly stored in the WASM data section
    let source = r#"
fun get_message() -> String {
    return "Test"
}
exegesis { Returns a test string. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Call the function to get the string pointer
    let result = wasm_module.call("get_message", &[]).expect("Call failed");

    let ptr = result
        .first()
        .and_then(|v| v.i32())
        .expect("Expected i32 result");
    assert_eq!(ptr, 0, "Expected string at offset 0");

    // TODO: Implement get_memory() on WasmModule to verify string data
    // For now, just verify the pointer was returned
}

#[test]
fn test_multiple_string_literals() {
    // Test that multiple string literals are properly stored and deduplicated
    let source = r#"
fun get_hello() -> String {
    return "hello"
}
exegesis { Returns hello. }

fun get_world() -> String {
    return "world"
}
exegesis { Returns world. }

fun get_hello_again() -> String {
    return "hello"
}
exegesis { Returns hello again. }
"#;
    let file = parse_dol_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile_file(&file).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Get pointers for all three strings
    let hello_ptr = wasm_module
        .call("get_hello", &[])
        .expect("Call failed")
        .first()
        .and_then(|v| v.i32())
        .expect("Expected i32");

    let world_ptr = wasm_module
        .call("get_world", &[])
        .expect("Call failed")
        .first()
        .and_then(|v| v.i32())
        .expect("Expected i32");

    let hello_again_ptr = wasm_module
        .call("get_hello_again", &[])
        .expect("Call failed")
        .first()
        .and_then(|v| v.i32())
        .expect("Expected i32");

    // "hello" should be deduplicated - both functions return the same pointer
    assert_eq!(
        hello_ptr, hello_again_ptr,
        "Duplicate strings should have same pointer"
    );

    // "world" should be at a different offset than "hello"
    assert_ne!(
        hello_ptr, world_ptr,
        "Different strings should have different pointers"
    );
}

#[test]
fn test_string_type_in_function_parameter() {
    // Test that String type is accepted as a function parameter
    let source = r#"
fun identity(s: String) -> String {
    return s
}
exegesis { Returns the input string. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let result = compiler.compile(&module);

    assert!(
        result.is_ok(),
        "String parameter compilation failed: {:?}",
        result.err()
    );
}

#[test]
#[ignore] // Requires get_memory() implementation
fn test_empty_string_literal() {
    // Test that empty string literals work correctly
    let source = r#"
fun get_empty() -> String {
    return ""
}
exegesis { Returns an empty string. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Call the function
    let result = wasm_module.call("get_empty", &[]).expect("Call failed");

    // Should return a pointer
    let ptr = result
        .first()
        .and_then(|v| v.i32())
        .expect("Expected i32 result");
    assert_eq!(ptr, 0, "Expected empty string at offset 0");

    // TODO: Implement get_memory() on WasmModule to verify empty string length
}

// ============================================
// 13. Hello World Spirit End-to-End Tests
// ============================================

/// This test demonstrates the complete DOL → WASM → Execution pipeline
/// for a "Hello World" Spirit - the minimal viable Spirit in the Univrs ecosystem.
#[test]
fn test_hello_world_spirit_e2e() {
    // A minimal Spirit with state and behavior
    // Uses parse_file for single declaration (currently supported)
    let source = r#"
fun spirit_main(input: i64) -> i64 {
    return input * 2
}
exegesis { Spirit main entry point - doubles input. }
"#;

    let module = parse_file(source).expect("Failed to parse Hello Spirit");

    // Compile to WASM
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler
        .compile(&module)
        .expect("Failed to compile Hello Spirit to WASM");

    // Verify valid WASM module
    assert!(wasm_bytes.len() > 8, "WASM should have content");
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Should have WASM magic number");

    // Load into runtime
    let runtime = WasmRuntime::new().expect("Failed to create WASM runtime");
    let mut wasm_module = runtime
        .load(&wasm_bytes)
        .expect("Failed to load Spirit WASM");

    // Execute spirit_main - the entry point
    let result = wasm_module
        .call("spirit_main", &[21i64.into()])
        .expect("Call spirit_main failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(42),
        "Spirit should double input: 21 * 2 = 42"
    );
}

/// Test a Spirit with fibonacci computation
#[test]
fn test_spirit_with_computation() {
    let source = r#"
fun fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n
    }
    let a: i64 = 0
    let b: i64 = 1
    let i: i64 = 2
    while i <= n {
        let temp: i64 = a + b
        a = b
        b = temp
        i = i + 1
    }
    return b
}
exegesis { Computes nth Fibonacci number iteratively. }
"#;

    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test Fibonacci: fib(10) = 55
    let result = wasm_module
        .call("fibonacci", &[10i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(55));

    // Test Fibonacci: fib(7) = 13
    let result = wasm_module
        .call("fibonacci", &[7i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(13));
}

/// Test Spirit with ENR-like entropy calculation
#[test]
fn test_spirit_enr_entropy() {
    let source = r#"
fun calculate_entropy_cost(hops: i64, base_cost: i64) -> i64 {
    return hops * base_cost
}
exegesis { Calculate entropy cost based on hop count. }
"#;

    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test entropy calculation: 5 hops * 10 cost = 50
    let result = wasm_module
        .call("calculate_entropy_cost", &[5i64.into(), 10i64.into()])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(50),
        "5 hops * 10 cost = 50"
    );
}

/// Test Spirit with complex control flow
#[test]
fn test_spirit_control_flow() {
    let source = r#"
fun process_request(request_type: i64, value: i64) -> i64 {
    match request_type {
        0 => {
            // Echo request
            return value
        },
        1 => {
            // Double request
            return value * 2
        },
        2 => {
            // Square request
            return value * value
        },
        _ => {
            // Unknown request
            return 0
        },
    }
}
exegesis { Process different request types. }
"#;

    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test echo: type 0, value 42 -> 42
    let result = wasm_module
        .call("process_request", &[0i64.into(), 42i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));

    // Test double: type 1, value 21 -> 42
    let result = wasm_module
        .call("process_request", &[1i64.into(), 21i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));

    // Test square: type 2, value 7 -> 49
    let result = wasm_module
        .call("process_request", &[2i64.into(), 7i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(49));

    // Test unknown: type 99, value 100 -> 0
    let result = wasm_module
        .call("process_request", &[99i64.into(), 100i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}
