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

// ============================================
// F64 Field Type Coverage Tests
// ============================================

/// Test member access with F64 field type
#[test]
fn test_member_access_f64_field() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let source = r#"
fun get_temperature() -> f64 {
    let sensor = Sensor { temperature: 98.6, humidity: 0.65 }
    return sensor.temperature
}
exegesis { Gets the temperature from a sensor. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Create Sensor layout with F64 fields
    let sensor_layout = GeneLayout {
        name: "Sensor".to_string(),
        fields: vec![
            FieldLayout::primitive("temperature", 0, WasmFieldType::F64),
            FieldLayout::primitive("humidity", 8, WasmFieldType::F64),
        ],
        total_size: 16,
        alignment: 8,
    };

    // Create compiler and register the layout
    let mut compiler = WasmCompiler::new();
    compiler.register_gene_layout(sensor_layout);

    // Compile to WASM
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Validate by loading into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Call the function
    let result = wasm_module
        .call("get_temperature", &[])
        .expect("Call failed");

    // Verify F64 result - should be 98.6
    let temp = result
        .first()
        .and_then(|v| v.f64())
        .expect("Expected f64 result");
    assert!((temp - 98.6).abs() < 0.001, "Expected 98.6, got {}", temp);
}

/// Test struct literal with F64 fields and member access
#[test]
fn test_struct_literal_f64_field() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let source = r#"
fun get_humidity() -> f64 {
    let sensor = Sensor { temperature: 72.5, humidity: 0.45 }
    return sensor.humidity
}
exegesis { Gets humidity from a sensor. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Create Sensor layout with F64 fields
    let sensor_layout = GeneLayout {
        name: "Sensor".to_string(),
        fields: vec![
            FieldLayout::primitive("temperature", 0, WasmFieldType::F64),
            FieldLayout::primitive("humidity", 8, WasmFieldType::F64),
        ],
        total_size: 16,
        alignment: 8,
    };

    let mut compiler = WasmCompiler::new();
    compiler.register_gene_layout(sensor_layout);

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module.call("get_humidity", &[]).expect("Call failed");

    // Verify F64 result - should be 0.45
    let humidity = result
        .first()
        .and_then(|v| v.f64())
        .expect("Expected f64 result");
    assert!(
        (humidity - 0.45).abs() < 0.001,
        "Expected 0.45, got {}",
        humidity
    );
}

/// Test struct literal with mixed I64 and F64 fields
#[test]
fn test_struct_literal_mixed_i64_f64() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let source = r#"
fun get_average_temp() -> f64 {
    let reading = WeatherReading { timestamp: 1704312000, temperature: 72.5, pressure: 1013.25 }
    return reading.pressure
}
exegesis { Gets pressure from a weather reading with mixed field types. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Create WeatherReading layout with mixed I64 and F64 fields
    let reading_layout = GeneLayout {
        name: "WeatherReading".to_string(),
        fields: vec![
            FieldLayout::primitive("timestamp", 0, WasmFieldType::I64),
            FieldLayout::primitive("temperature", 8, WasmFieldType::F64),
            FieldLayout::primitive("pressure", 16, WasmFieldType::F64),
        ],
        total_size: 24,
        alignment: 8,
    };

    let mut compiler = WasmCompiler::new();
    compiler.register_gene_layout(reading_layout);

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("get_average_temp", &[])
        .expect("Call failed");

    // Verify F64 result - should be 1013.25
    let pressure = result
        .first()
        .and_then(|v| v.f64())
        .expect("Expected f64 result");
    assert!(
        (pressure - 1013.25).abs() < 0.001,
        "Expected 1013.25, got {}",
        pressure
    );
}

// ============================================
// Unary Operator Coverage Tests
// ============================================

/// Test unary negation operator
#[test]
fn test_unary_negation() {
    let source = r#"
fun negate(x: i64) -> i64 {
    return -x
}
exegesis { Negates an integer. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test negating positive number
    let result = wasm_module
        .call("negate", &[42i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(-42));

    // Test negating negative number (double negation)
    let result = wasm_module
        .call("negate", &[(-100i64).into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(100));

    // Test negating zero
    let result = wasm_module
        .call("negate", &[0i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test unary negation in expressions
#[test]
fn test_unary_negation_in_expression() {
    let source = r#"
fun subtract_via_negation(a: i64, b: i64) -> i64 {
    return a + -b
}
exegesis { Subtracts b from a using negation. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // 10 + (-3) = 7
    let result = wasm_module
        .call("subtract_via_negation", &[10i64.into(), 3i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(7));
}

/// Test unary not operator (boolean negation)
#[test]
fn test_unary_not() {
    let source = r#"
fun invert(x: i64) -> i64 {
    if !x {
        return 1
    }
    return 0
}
exegesis { Returns 1 if x is falsy (0), 0 otherwise. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // !0 is true, so return 1
    let result = wasm_module
        .call("invert", &[0i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    // !42 is false (42 != 0), so return 0
    let result = wasm_module
        .call("invert", &[42i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));

    // !1 is false, so return 0
    let result = wasm_module
        .call("invert", &[1i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

// ============================================
// Additional Expression Coverage Tests
// ============================================

/// Test nested struct field access
#[test]
fn test_nested_field_access() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let source = r#"
fun sum_coordinates() -> i64 {
    let p1 = Point { x: 10, y: 20 }
    let p2 = Point { x: 5, y: 15 }
    return p1.x + p2.y
}
exegesis { Sums x from one point and y from another. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let point_layout = GeneLayout {
        name: "Point".to_string(),
        fields: vec![
            FieldLayout::primitive("x", 0, WasmFieldType::I64),
            FieldLayout::primitive("y", 8, WasmFieldType::I64),
        ],
        total_size: 16,
        alignment: 8,
    };

    let mut compiler = WasmCompiler::new();
    compiler.register_gene_layout(point_layout);

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("sum_coordinates", &[])
        .expect("Call failed");

    // 10 + 15 = 25
    assert_eq!(result.first().and_then(|v| v.i64()), Some(25));
}

/// Test multiple struct literals in same function
#[test]
fn test_multiple_struct_literals() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let source = r#"
fun distance_squared() -> i64 {
    let start = Point { x: 0, y: 0 }
    let end = Point { x: 3, y: 4 }
    let dx = end.x - start.x
    let dy = end.y - start.y
    return dx * dx + dy * dy
}
exegesis { Calculates squared distance between two points. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let point_layout = GeneLayout {
        name: "Point".to_string(),
        fields: vec![
            FieldLayout::primitive("x", 0, WasmFieldType::I64),
            FieldLayout::primitive("y", 8, WasmFieldType::I64),
        ],
        total_size: 16,
        alignment: 8,
    };

    let mut compiler = WasmCompiler::new();
    compiler.register_gene_layout(point_layout);

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("distance_squared", &[])
        .expect("Call failed");

    // 3^2 + 4^2 = 9 + 16 = 25
    assert_eq!(result.first().and_then(|v| v.i64()), Some(25));
}

/// Test struct field in conditional
#[test]
fn test_struct_field_in_conditional() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let source = r#"
fun is_origin() -> i64 {
    let p = Point { x: 0, y: 0 }
    if p.x == 0 {
        if p.y == 0 {
            return 1
        }
    }
    return 0
}
exegesis { Checks if point is at origin. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let point_layout = GeneLayout {
        name: "Point".to_string(),
        fields: vec![
            FieldLayout::primitive("x", 0, WasmFieldType::I64),
            FieldLayout::primitive("y", 8, WasmFieldType::I64),
        ],
        total_size: 16,
        alignment: 8,
    };

    let mut compiler = WasmCompiler::new();
    compiler.register_gene_layout(point_layout);

    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module.call("is_origin", &[]).expect("Call failed");

    // Point is at origin, should return 1
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));
}

/// Test double negation
#[test]
fn test_double_negation() {
    let source = r#"
fun double_negate(x: i64) -> i64 {
    return -(-x)
}
exegesis { Double negates a number (returns original). }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Double negation should return original value
    let result = wasm_module
        .call("double_negate", &[42i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));

    let result = wasm_module
        .call("double_negate", &[(-17i64).into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(-17));
}

// ============================================
// 10. Layout Module Unit Tests
// ============================================

/// Test WasmFieldType::size for all variants
#[test]
fn test_wasm_field_type_size() {
    use metadol::wasm::layout::WasmFieldType;

    assert_eq!(WasmFieldType::I32.size(), 4);
    assert_eq!(WasmFieldType::I64.size(), 8);
    assert_eq!(WasmFieldType::F32.size(), 4);
    assert_eq!(WasmFieldType::F64.size(), 8);
    assert_eq!(WasmFieldType::Ptr.size(), 4);
}

/// Test WasmFieldType::alignment for all variants
#[test]
fn test_wasm_field_type_alignment() {
    use metadol::wasm::layout::WasmFieldType;

    assert_eq!(WasmFieldType::I32.alignment(), 4);
    assert_eq!(WasmFieldType::I64.alignment(), 8);
    assert_eq!(WasmFieldType::F32.alignment(), 4);
    assert_eq!(WasmFieldType::F64.alignment(), 8);
    assert_eq!(WasmFieldType::Ptr.alignment(), 4);
}

/// Test WasmFieldType::alignment_log2 for all variants
#[test]
fn test_wasm_field_type_alignment_log2() {
    use metadol::wasm::layout::WasmFieldType;

    // log2(4) = 2, log2(8) = 3
    assert_eq!(WasmFieldType::I32.alignment_log2(), 2);
    assert_eq!(WasmFieldType::I64.alignment_log2(), 3);
    assert_eq!(WasmFieldType::F32.alignment_log2(), 2);
    assert_eq!(WasmFieldType::F64.alignment_log2(), 3);
    assert_eq!(WasmFieldType::Ptr.alignment_log2(), 2);
}

/// Test WasmFieldType Display trait
#[test]
fn test_wasm_field_type_display() {
    use metadol::wasm::layout::WasmFieldType;

    assert_eq!(format!("{}", WasmFieldType::I32), "i32");
    assert_eq!(format!("{}", WasmFieldType::I64), "i64");
    assert_eq!(format!("{}", WasmFieldType::F32), "f32");
    assert_eq!(format!("{}", WasmFieldType::F64), "f64");
    assert_eq!(format!("{}", WasmFieldType::Ptr), "ptr");
}

/// Test WasmFieldType::to_val_type for WASM encoding
#[test]
fn test_wasm_field_type_to_val_type() {
    use metadol::wasm::layout::WasmFieldType;

    // I32 and Ptr both map to ValType::I32
    assert_eq!(WasmFieldType::I32.to_val_type(), wasm_encoder::ValType::I32);
    assert_eq!(WasmFieldType::Ptr.to_val_type(), wasm_encoder::ValType::I32);
    assert_eq!(WasmFieldType::I64.to_val_type(), wasm_encoder::ValType::I64);
    assert_eq!(WasmFieldType::F32.to_val_type(), wasm_encoder::ValType::F32);
    assert_eq!(WasmFieldType::F64.to_val_type(), wasm_encoder::ValType::F64);
}

/// Test FieldLayout::primitive constructor
#[test]
fn test_field_layout_primitive() {
    use metadol::wasm::layout::{FieldLayout, WasmFieldType};

    let field = FieldLayout::primitive("x", 0, WasmFieldType::F64);

    assert_eq!(field.name, "x");
    assert_eq!(field.offset, 0);
    assert_eq!(field.size, 8);
    assert_eq!(field.alignment, 8);
    assert_eq!(field.wasm_type, WasmFieldType::F64);
    assert!(!field.is_reference);
    assert!(field.nested_layout.is_none());
}

/// Test FieldLayout::reference constructor
#[test]
fn test_field_layout_reference() {
    use metadol::wasm::layout::{FieldLayout, WasmFieldType};

    let field = FieldLayout::reference("other", 16);

    assert_eq!(field.name, "other");
    assert_eq!(field.offset, 16);
    assert_eq!(field.size, 4);
    assert_eq!(field.alignment, 4);
    assert_eq!(field.wasm_type, WasmFieldType::Ptr);
    assert!(field.is_reference);
    assert!(field.nested_layout.is_none());
}

/// Test FieldLayout::inline constructor
#[test]
fn test_field_layout_inline() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let inner_layout = GeneLayout {
        name: "Inner".to_string(),
        fields: vec![FieldLayout::primitive("val", 0, WasmFieldType::I32)],
        total_size: 4,
        alignment: 4,
    };

    let field = FieldLayout::inline("nested", 8, inner_layout);

    assert_eq!(field.name, "nested");
    assert_eq!(field.offset, 8);
    assert_eq!(field.size, 4);
    assert_eq!(field.alignment, 4);
    assert!(!field.is_reference);
    assert!(field.nested_layout.is_some());
    assert_eq!(field.nested_layout.as_ref().unwrap().name, "Inner");
}

/// Test GeneLayout::new constructor
#[test]
fn test_gene_layout_new() {
    use metadol::wasm::layout::GeneLayout;

    let layout = GeneLayout::new("TestGene");

    assert_eq!(layout.name, "TestGene");
    assert!(layout.fields.is_empty());
    assert_eq!(layout.total_size, 0);
    assert_eq!(layout.alignment, 1);
}

/// Test GeneLayout::default
#[test]
fn test_gene_layout_default() {
    use metadol::wasm::layout::GeneLayout;

    let layout = GeneLayout::default();

    assert_eq!(layout.name, "");
    assert!(layout.fields.is_empty());
    assert_eq!(layout.total_size, 0);
    assert_eq!(layout.alignment, 1);
}

/// Test GeneLayout::get_field and get_field_offset
#[test]
fn test_gene_layout_get_field() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let layout = GeneLayout {
        name: "Point".to_string(),
        fields: vec![
            FieldLayout::primitive("x", 0, WasmFieldType::F64),
            FieldLayout::primitive("y", 8, WasmFieldType::F64),
        ],
        total_size: 16,
        alignment: 8,
    };

    // Test get_field
    let x_field = layout.get_field("x");
    assert!(x_field.is_some());
    assert_eq!(x_field.unwrap().offset, 0);

    let y_field = layout.get_field("y");
    assert!(y_field.is_some());
    assert_eq!(y_field.unwrap().offset, 8);

    let z_field = layout.get_field("z");
    assert!(z_field.is_none());

    // Test get_field_offset
    assert_eq!(layout.get_field_offset("x"), Some(0));
    assert_eq!(layout.get_field_offset("y"), Some(8));
    assert_eq!(layout.get_field_offset("z"), None);
}

/// Test GeneLayout::type_id
#[test]
fn test_gene_layout_type_id() {
    use metadol::wasm::layout::GeneLayout;

    let layout1 = GeneLayout::new("Point");
    let layout2 = GeneLayout::new("Point");
    let layout3 = GeneLayout::new("Vector");

    // Same name should produce same type_id
    assert_eq!(layout1.type_id(), layout2.type_id());

    // Different name should produce different type_id
    assert_ne!(layout1.type_id(), layout3.type_id());

    // Type IDs should be non-zero for non-empty names
    assert_ne!(layout1.type_id(), 0);
}

/// Test GeneLayout::has_references and pointer_offsets
#[test]
fn test_gene_layout_references() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    // Layout without references
    let no_refs = GeneLayout {
        name: "NoRefs".to_string(),
        fields: vec![
            FieldLayout::primitive("a", 0, WasmFieldType::I64),
            FieldLayout::primitive("b", 8, WasmFieldType::I64),
        ],
        total_size: 16,
        alignment: 8,
    };
    assert!(!no_refs.has_references());
    assert!(no_refs.pointer_offsets().is_empty());

    // Layout with references
    let with_refs = GeneLayout {
        name: "WithRefs".to_string(),
        fields: vec![
            FieldLayout::primitive("val", 0, WasmFieldType::I64),
            FieldLayout::reference("next", 8),
            FieldLayout::reference("prev", 12),
        ],
        total_size: 16,
        alignment: 8,
    };
    assert!(with_refs.has_references());
    let offsets = with_refs.pointer_offsets();
    assert_eq!(offsets.len(), 2);
    assert!(offsets.contains(&8));
    assert!(offsets.contains(&12));
}

/// Test GeneLayout::field_count and is_empty
#[test]
fn test_gene_layout_field_count() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};

    let empty = GeneLayout::new("Empty");
    assert!(empty.is_empty());
    assert_eq!(empty.field_count(), 0);

    let with_fields = GeneLayout {
        name: "Point".to_string(),
        fields: vec![
            FieldLayout::primitive("x", 0, WasmFieldType::F64),
            FieldLayout::primitive("y", 8, WasmFieldType::F64),
        ],
        total_size: 16,
        alignment: 8,
    };
    assert!(!with_fields.is_empty());
    assert_eq!(with_fields.field_count(), 2);
}

/// Test GeneLayoutRegistry operations
#[test]
fn test_gene_layout_registry() {
    use metadol::wasm::layout::{FieldLayout, GeneLayout, GeneLayoutRegistry, WasmFieldType};

    let mut registry = GeneLayoutRegistry::new();

    // Initially empty
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(!registry.contains("Point"));
    assert!(registry.get("Point").is_none());

    // Register a layout
    let point = GeneLayout {
        name: "Point".to_string(),
        fields: vec![
            FieldLayout::primitive("x", 0, WasmFieldType::F64),
            FieldLayout::primitive("y", 8, WasmFieldType::F64),
        ],
        total_size: 16,
        alignment: 8,
    };
    registry.register(point);

    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
    assert!(registry.contains("Point"));

    let retrieved = registry.get("Point");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().total_size, 16);

    // Register another layout
    let vector = GeneLayout {
        name: "Vector".to_string(),
        fields: vec![
            FieldLayout::primitive("dx", 0, WasmFieldType::F64),
            FieldLayout::primitive("dy", 8, WasmFieldType::F64),
        ],
        total_size: 16,
        alignment: 8,
    };
    registry.register(vector);

    assert_eq!(registry.len(), 2);

    // Iterate
    let names: Vec<_> = registry.iter().map(|(n, _)| n.clone()).collect();
    assert!(names.contains(&"Point".to_string()));
    assert!(names.contains(&"Vector".to_string()));

    // Remove
    let removed = registry.remove("Point");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "Point");
    assert!(!registry.contains("Point"));
    assert_eq!(registry.len(), 1);

    // Clear
    registry.clear();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

// ============================================
// 11. Alloc Module Unit Tests
// ============================================

/// Test BumpAllocator::new and accessor methods
#[test]
fn test_bump_allocator_new() {
    use metadol::wasm::alloc::BumpAllocator;

    let allocator = BumpAllocator::new(0, 1);

    assert_eq!(allocator.heap_base_global(), 0);
    assert_eq!(allocator.heap_end_global(), 1);

    // Test with different indices
    let allocator2 = BumpAllocator::new(5, 10);
    assert_eq!(allocator2.heap_base_global(), 5);
    assert_eq!(allocator2.heap_end_global(), 10);
}

/// Test alloc module constants
#[test]
fn test_alloc_constants() {
    use metadol::wasm::alloc::{
        DEFAULT_HEAP_START, DEFAULT_MEMORY_PAGES, MAX_MEMORY_PAGES, PAGE_SIZE,
    };

    assert_eq!(DEFAULT_MEMORY_PAGES, 1);
    assert_eq!(MAX_MEMORY_PAGES, 256);
    assert_eq!(PAGE_SIZE, 65536);
    assert_eq!(DEFAULT_HEAP_START, 1024);

    // Validate relationships
    assert!(DEFAULT_HEAP_START < PAGE_SIZE);
    assert!(DEFAULT_MEMORY_PAGES <= MAX_MEMORY_PAGES);
}

/// Test BumpAllocator::emit_globals generates valid WASM
#[test]
fn test_bump_allocator_emit_globals() {
    use metadol::wasm::alloc::BumpAllocator;
    use wasm_encoder::Module;

    let mut module = Module::new();
    BumpAllocator::emit_globals(&mut module, 1024);

    // Module should encode without panic
    let bytes = module.finish();
    assert!(!bytes.is_empty());

    // Should start with WASM magic number
    assert_eq!(&bytes[0..4], b"\0asm");
}

/// Test BumpAllocator::emit_globals_with_end
#[test]
fn test_bump_allocator_emit_globals_with_end() {
    use metadol::wasm::alloc::BumpAllocator;
    use wasm_encoder::Module;

    let mut module = Module::new();
    // 4 pages = 256KB
    BumpAllocator::emit_globals_with_end(&mut module, 2048, 4 * 65536);

    let bytes = module.finish();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"\0asm");
}

/// Test BumpAllocator::emit_memory_section
#[test]
fn test_bump_allocator_emit_memory_section() {
    use metadol::wasm::alloc::BumpAllocator;
    use wasm_encoder::Module;

    let mut module = Module::new();
    BumpAllocator::emit_memory_section(&mut module, 2); // 2 pages = 128KB

    let bytes = module.finish();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"\0asm");
}

/// Test BumpAllocator::emit_alloc_function generates instructions
#[test]
fn test_bump_allocator_emit_alloc_function() {
    use metadol::wasm::alloc::BumpAllocator;

    let instructions = BumpAllocator::emit_alloc_function();

    // Should have a non-trivial number of instructions
    assert!(instructions.len() > 10);

    // First instruction should be GlobalGet(0) to load heap base
    assert!(matches!(
        instructions[0],
        wasm_encoder::Instruction::GlobalGet(0)
    ));

    // Last instruction should be End
    assert!(matches!(
        instructions.last(),
        Some(wasm_encoder::Instruction::End)
    ));
}

/// Test BumpAllocator::build_alloc_function creates a complete function
#[test]
fn test_bump_allocator_build_alloc_function() {
    use metadol::wasm::alloc::BumpAllocator;

    let function = BumpAllocator::build_alloc_function();

    // Function should encode without error
    let mut func_section = wasm_encoder::CodeSection::new();
    func_section.function(&function);
    // If it encodes without panic, the function is valid
}

/// Test BumpAllocator::alloc_type_signature
#[test]
fn test_bump_allocator_alloc_type_signature() {
    use metadol::wasm::alloc::BumpAllocator;

    let (params, results) = BumpAllocator::alloc_type_signature();

    // alloc(size: i32, align: i32) -> i32
    assert_eq!(params.len(), 2);
    assert_eq!(params[0], wasm_encoder::ValType::I32);
    assert_eq!(params[1], wasm_encoder::ValType::I32);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], wasm_encoder::ValType::I32);
}

// ============================================
// 12. Runtime Module Additional Tests
// ============================================

/// Test WasmRuntime::default implementation
#[test]
fn test_wasm_runtime_default() {
    // Default implementation should not panic
    let runtime = WasmRuntime::default();
    drop(runtime);
}

/// Test WasmModule::instance accessor
#[test]
fn test_wasm_module_instance() {
    let source = r#"
fun get_value() -> i64 {
    return 42
}
exegesis { Returns 42. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test instance accessor - get instance and verify we can access it
    let instance = wasm_module.instance();

    // We can't easily test exports without borrowing issues,
    // but we can verify the instance exists
    let _ = instance;

    // Verify module still works
    let result = wasm_module.call("get_value", &[]).expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));
}

/// Test WasmModule::store_mut accessor
#[test]
fn test_wasm_module_store_mut() {
    let source = r#"
fun add(a: i64, b: i64) -> i64 {
    return a + b
}
exegesis { Adds two numbers. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Access store_mut - it should work (the store data is () which is unit type)
    {
        let store = wasm_module.store_mut();
        // Store should be usable - accessing data doesn't panic
        let _data = store.data();
    }

    // Call function to verify module still works after accessing store
    let result = wasm_module
        .call("add", &[10i64.into(), 32i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));
}

/// Test WasmRuntime::load_file with non-existent file
#[test]
fn test_wasm_runtime_load_file_error() {
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let result = runtime.load_file("/nonexistent/path/module.wasm");

    // Should fail with file not found error
    assert!(result.is_err());
}

/// Test WasmRuntime::load_file with valid WASM file
#[test]
fn test_wasm_runtime_load_file() {
    use std::io::Write;

    // Create a valid WASM module
    let source = r#"
fun double(x: i64) -> i64 {
    return x * 2
}
exegesis { Doubles a number. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    // Write to temp file
    let temp_path = "/tmp/test_load_file.wasm";
    let mut file = std::fs::File::create(temp_path).expect("Failed to create temp file");
    file.write_all(&wasm_bytes)
        .expect("Failed to write WASM bytes");
    drop(file);

    // Load from file
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime
        .load_file(temp_path)
        .expect("Failed to load from file");

    // Verify it works
    let result = wasm_module
        .call("double", &[21i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));

    // Cleanup
    std::fs::remove_file(temp_path).ok();
}

// ============================================
// 13. Compiler Coverage Tests
// ============================================

/// Test boolean literal compilation
#[test]
fn test_bool_literal_compilation() {
    let source = r#"
fun get_true() -> i64 {
    if true {
        return 1
    }
    return 0
}
exegesis { Returns 1. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module.call("get_true", &[]).expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));
}

/// Test boolean false literal
#[test]
fn test_bool_false_literal() {
    let source = r#"
fun get_false() -> i64 {
    if false {
        return 1
    }
    return 0
}
exegesis { Returns 0. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module.call("get_false", &[]).expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test match expression with wildcards
#[test]
fn test_match_with_wildcard() {
    let source = r#"
fun classify(x: i64) -> i64 {
    match x {
        0 => 100,
        1 => 200,
        _ => 999,
    }
}
exegesis { Classifies value. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test specific cases
    let result = wasm_module
        .call("classify", &[0i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(100));

    let result = wasm_module
        .call("classify", &[1i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(200));

    // Wildcard case
    let result = wasm_module
        .call("classify", &[42i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(999));
}

/// Test if-else with returns
#[test]
fn test_if_else_with_returns() {
    let source = r#"
fun sign(x: i64) -> i64 {
    if x > 0 {
        return 1
    }
    if x < 0 {
        return -1
    }
    return 0
}
exegesis { Returns sign of number. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("sign", &[5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("sign", &[(-7i64).into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(-1));

    let result = wasm_module
        .call("sign", &[0i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test while loop
#[test]
fn test_while_loop() {
    let source = r#"
fun sum_to(n: i64) -> i64 {
    let result = 0
    let i = 0
    while i <= n {
        result = result + i
        i = i + 1
    }
    return result
}
exegesis { Sum of 0 to n. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // sum(0..5) = 0+1+2+3+4+5 = 15
    let result = wasm_module
        .call("sum_to", &[5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(15));
}

/// Test for loop with range literals
#[test]
fn test_for_loop_range_literal() {
    let source = r#"
fun sum_five() -> i64 {
    let total = 0
    for i in 0..5 {
        total = total + i
    }
    return total
}
exegesis { Sum 0 to 4. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // sum(0..5) = 0+1+2+3+4 = 10
    let result = wasm_module.call("sum_five", &[]).expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(10));
}

/// Test modulo operator
#[test]
fn test_modulo_operator() {
    let source = r#"
fun is_even(x: i64) -> i64 {
    if x % 2 == 0 {
        return 1
    }
    return 0
}
exegesis { Returns 1 if even. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("is_even", &[4i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("is_even", &[7i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test division operator
#[test]
fn test_division_operator() {
    let source = r#"
fun divide(a: i64, b: i64) -> i64 {
    return a / b
}
exegesis { Divides a by b. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("divide", &[42i64.into(), 6i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(7));
}

/// Test logical AND operator
#[test]
fn test_logical_and() {
    let source = r#"
fun both_positive(a: i64, b: i64) -> i64 {
    if a > 0 && b > 0 {
        return 1
    }
    return 0
}
exegesis { Returns 1 if both positive. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("both_positive", &[5i64.into(), 3i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("both_positive", &[5i64.into(), (-3i64).into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test logical OR operator
#[test]
fn test_logical_or() {
    let source = r#"
fun either_positive(a: i64, b: i64) -> i64 {
    if a > 0 || b > 0 {
        return 1
    }
    return 0
}
exegesis { Returns 1 if either positive. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("either_positive", &[(-5i64).into(), 3i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("either_positive", &[(-5i64).into(), (-3i64).into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test less than or equal operator
#[test]
fn test_lte_operator() {
    let source = r#"
fun is_lte(a: i64, b: i64) -> i64 {
    if a <= b {
        return 1
    }
    return 0
}
exegesis { Returns 1 if a <= b. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("is_lte", &[5i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("is_lte", &[4i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("is_lte", &[6i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test greater than or equal operator
#[test]
fn test_gte_operator() {
    let source = r#"
fun is_gte(a: i64, b: i64) -> i64 {
    if a >= b {
        return 1
    }
    return 0
}
exegesis { Returns 1 if a >= b. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("is_gte", &[5i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("is_gte", &[6i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    let result = wasm_module
        .call("is_gte", &[4i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

/// Test not equal operator
#[test]
fn test_not_equal_operator() {
    let source = r#"
fun is_ne(a: i64, b: i64) -> i64 {
    if a != b {
        return 1
    }
    return 0
}
exegesis { Returns 1 if a != b. }
"#;
    let module = parse_file(source).expect("Failed to parse");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    let result = wasm_module
        .call("is_ne", &[5i64.into(), 5i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));

    let result = wasm_module
        .call("is_ne", &[5i64.into(), 6i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));
}

// ============================================
// Gene Constructor Tests
// ============================================

/// Test that gene constructors are exported and callable
#[test]
fn test_gene_constructor_export() {
    let source = r#"
        /// A 2D point gene
        gene Point {
            has x: Int64
            has y: Int64
        }

        /// Function that uses a Point
        pub fun get_point_sum() -> Int64 {
            let p = Point { x: 10, y: 20 }
            p.x + p.y
        }
    "#;

    let file = parse_dol_file(source).expect("Parse failed");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile_file(&file).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load WASM");

    // Test the regular function that uses struct literals
    let result = wasm_module.call("get_point_sum", &[]).expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(30));

    // The constructor new_Point should be exported
    // It takes two i64 parameters (x, y) and returns i32 (pointer)
    let result = wasm_module
        .call("new_Point", &[100i64.into(), 200i64.into()])
        .expect("Call new_Point failed");

    // Should return a valid pointer (non-null)
    let ptr = result
        .first()
        .and_then(|v| v.i32())
        .expect("Expected i32 pointer result");
    assert!(ptr >= 0, "Pointer should be non-negative");
}

/// Test gene constructor with different field types
#[test]
fn test_gene_constructor_with_floats() {
    let source = r#"
        /// A 2D vector with float components
        gene Vector2D {
            has dx: Float64
            has dy: Float64
        }

        /// Create and use a vector
        pub fun test_vector() -> Float64 {
            let v = Vector2D { dx: 3.0, dy: 4.0 }
            v.dx + v.dy
        }
    "#;

    let file = parse_dol_file(source).expect("Parse failed");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile_file(&file).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load WASM");

    // Test the function
    let result = wasm_module.call("test_vector", &[]).expect("Call failed");
    let sum = result.first().and_then(|v| v.f64()).expect("Expected f64");
    assert!((sum - 7.0).abs() < 0.0001, "Expected 7.0, got {}", sum);

    // Call constructor directly with f64 parameters
    // Note: wasmtime uses f64 bits for F64 args
    let result = wasm_module.call(
        "new_Vector2D",
        &[
            wasmtime::Val::F64(1.5f64.to_bits()),
            wasmtime::Val::F64(2.5f64.to_bits()),
        ],
    );

    // Constructor should work
    assert!(
        result.is_ok(),
        "new_Vector2D constructor should be callable: {:?}",
        result.err()
    );
}

/// Test multiple genes with constructors
#[test]
fn test_multiple_gene_constructors() {
    let source = r#"
        /// A 2D point
        gene Point {
            has x: Int64
            has y: Int64
        }

        /// A 3D point
        gene Point3D {
            has x: Int64
            has y: Int64
            has z: Int64
        }

        /// Sum coordinates of both points
        pub fun test_both() -> Int64 {
            let p2 = Point { x: 1, y: 2 }
            let p3 = Point3D { x: 10, y: 20, z: 30 }
            p2.x + p2.y + p3.x + p3.y + p3.z
        }
    "#;

    let file = parse_dol_file(source).expect("Parse failed");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile_file(&file).expect("Compilation failed");

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load WASM");

    // Test the combined function
    let result = wasm_module.call("test_both", &[]).expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(63),
        "1+2+10+20+30 = 63"
    );

    // Both constructors should be exported
    let result = wasm_module
        .call("new_Point", &[5i64.into(), 6i64.into()])
        .expect("new_Point should be exported");
    assert!(result.first().and_then(|v| v.i32()).is_some());

    let result = wasm_module
        .call("new_Point3D", &[1i64.into(), 2i64.into(), 3i64.into()])
        .expect("new_Point3D should be exported");
    assert!(result.first().and_then(|v| v.i32()).is_some());
}
