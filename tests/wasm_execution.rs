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
// 9. Vertical Slice Output Test
// ============================================

#[test]
fn test_compile_counter_to_file() {
    use metadol::Parser;
    use std::fs;

    let source = r#"
module counter_test @ 0.1.0

/// Counter gene with increment and add methods
gene Counter {
    has value: Int64

    fun increment() -> Int64 {
        return value + 1
    }

    fun get_value() -> Int64 {
        return value
    }

    fun add(n: Int64) -> Int64 {
        return value + n
    }

    // Test field mutation
    sex fun set_value(new_val: Int64) -> Int64 {
        self.value = new_val
        return value
    }

    // Increment with mutation
    sex fun increment_mut() -> Int64 {
        self.value = value + 1
        return value
    }
}

/// Standalone function for simpler test
fun add_numbers(a: Int64, b: Int64) -> Int64 {
    return a + b
}
"#;

    let mut parser = Parser::new(source);
    let dol_file = parser.parse_file().expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler
        .compile_file(&dol_file)
        .expect("Compilation failed");

    // Write to file for TypeScript integration test
    fs::write("vertical-slice-results/counter.wasm", &wasm_bytes).expect("Failed to write file");
    println!("Wrote {} bytes to counter.wasm", wasm_bytes.len());

    // Verify module loads
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test add_numbers
    let result = wasm_module
        .call("add_numbers", &[3i64.into(), 4i64.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(7));

    // Test Counter.increment with self pointer at 0
    let result = wasm_module
        .call("Counter.increment", &[0i32.into()])
        .expect("Call failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    // Test Counter.set_value (field mutation)
    // Allocate at address 1024 (past static data)
    let result = wasm_module
        .call("Counter.set_value", &[1024i32.into(), 42i64.into()])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(42),
        "set_value should return new value"
    );

    // Verify value was actually stored
    let result = wasm_module
        .call("Counter.get_value", &[1024i32.into()])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(42),
        "get_value should read the stored value"
    );

    // Test increment_mut (field mutation)
    let result = wasm_module
        .call("Counter.increment_mut", &[1024i32.into()])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(43),
        "increment_mut should update and return new value"
    );

    // Verify increment persisted
    let result = wasm_module
        .call("Counter.get_value", &[1024i32.into()])
        .expect("Call failed");
    assert_eq!(
        result.first().and_then(|v| v.i64()),
        Some(43),
        "get_value should see updated value after increment_mut"
    );
}

// ============================================
// 10. SEX Var Global Variable Tests
// ============================================

#[test]
fn test_compile_and_execute_sex_var_global() {
    use metadol::Parser;

    // Test sex var: global mutable variable
    let source = r#"
module sex_var_test @ 0.1.0

sex var counter: i64 = 0

sex fun increment() -> i64 {
    counter = counter + 1
    return counter
}

sex fun get_counter() -> i64 {
    return counter
}

sex fun reset() -> i64 {
    counter = 0
    return counter
}
"#;

    let mut parser = Parser::new(source);
    let dol_file = parser.parse_file().expect("Failed to parse");

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

    // Initial value should be 0
    let result = wasm_module
        .call("get_counter", &[])
        .expect("Call get_counter failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));

    // First increment should return 1
    let result = wasm_module
        .call("increment", &[])
        .expect("Call increment failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(1));

    // Second increment should return 2
    let result = wasm_module
        .call("increment", &[])
        .expect("Call increment failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(2));

    // Get should confirm value is 2
    let result = wasm_module
        .call("get_counter", &[])
        .expect("Call get_counter failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(2));

    // Reset should return 0
    let result = wasm_module.call("reset", &[]).expect("Call reset failed");
    assert_eq!(result.first().and_then(|v| v.i64()), Some(0));
}

// ============================================
// 11. ENR Entropy Core Compilation Test
// ============================================

#[test]
fn test_compile_entropy_core_from_file() {
    use metadol::Parser;
    use std::fs;

    let source = fs::read_to_string("test-cases/enr-subset/entropy_core.dol")
        .expect("Failed to read entropy_core.dol");

    let mut parser = Parser::new(&source);
    let dol_file = parser
        .parse_file()
        .expect("Failed to parse entropy_core.dol");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler
        .compile_file(&dol_file)
        .expect("Failed to compile entropy_core.dol");

    // Verify valid WASM
    assert!(wasm_bytes.len() >= 8, "WASM too short");
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Invalid WASM magic number");

    // Write to file for inspection
    fs::write("test-cases/enr-subset/entropy_core.wasm", &wasm_bytes)
        .expect("Failed to write entropy_core.wasm");
    println!("Wrote {} bytes to entropy_core.wasm", wasm_bytes.len());

    // Load and verify module
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test add_entropy(3.0, 4.0) = 7.0
    let result = wasm_module
        .call(
            "add_entropy",
            &[
                wasmtime::Val::F64(3.0f64.to_bits()),
                wasmtime::Val::F64(4.0f64.to_bits()),
            ],
        )
        .expect("Call add_entropy failed");
    let value = match result.first() {
        Some(wasmtime::Val::F64(bits)) => f64::from_bits(*bits),
        _ => panic!("Expected f64 return"),
    };
    assert!(
        (value - 7.0).abs() < 0.001,
        "add_entropy(3.0, 4.0) should be 7.0, got {}",
        value
    );

    // Test scale_entropy(5.0, 2.0) = 10.0
    let result = wasm_module
        .call(
            "scale_entropy",
            &[
                wasmtime::Val::F64(5.0f64.to_bits()),
                wasmtime::Val::F64(2.0f64.to_bits()),
            ],
        )
        .expect("Call scale_entropy failed");
    let value = match result.first() {
        Some(wasmtime::Val::F64(bits)) => f64::from_bits(*bits),
        _ => panic!("Expected f64 return"),
    };
    assert!(
        (value - 10.0).abs() < 0.001,
        "scale_entropy(5.0, 2.0) should be 10.0, got {}",
        value
    );

    // Test clamp_entropy(15.0, 10.0) = 10.0 (clamped)
    let result = wasm_module
        .call(
            "clamp_entropy",
            &[
                wasmtime::Val::F64(15.0f64.to_bits()),
                wasmtime::Val::F64(10.0f64.to_bits()),
            ],
        )
        .expect("Call clamp_entropy failed");
    let value = match result.first() {
        Some(wasmtime::Val::F64(bits)) => f64::from_bits(*bits),
        _ => panic!("Expected f64 return"),
    };
    assert!(
        (value - 10.0).abs() < 0.001,
        "clamp_entropy(15.0, 10.0) should be 10.0, got {}",
        value
    );

    // Test clamp_entropy(5.0, 10.0) = 5.0 (not clamped)
    let result = wasm_module
        .call(
            "clamp_entropy",
            &[
                wasmtime::Val::F64(5.0f64.to_bits()),
                wasmtime::Val::F64(10.0f64.to_bits()),
            ],
        )
        .expect("Call clamp_entropy failed");
    let value = match result.first() {
        Some(wasmtime::Val::F64(bits)) => f64::from_bits(*bits),
        _ => panic!("Expected f64 return"),
    };
    assert!(
        (value - 5.0).abs() < 0.001,
        "clamp_entropy(5.0, 10.0) should be 5.0, got {}",
        value
    );

    println!("All entropy_core.dol functions executed successfully!");
}
