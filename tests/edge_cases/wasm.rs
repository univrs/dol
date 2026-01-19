//! WASM Runtime Edge Case Tests for DOL
//!
//! Tests WASM compilation and runtime edge cases:
//! - Stack overflow from deep recursion
//! - Memory exhaustion
//! - Very large data structures
//! - Concurrent access patterns
//! - Hot-reload during execution
//! - Invalid WASM recovery
//!
//! These tests help discover bugs in the WASM compiler and runtime.

#![cfg(feature = "wasm-runtime")]

use metadol::parse_file;

#[cfg(feature = "wasm-compile")]
use metadol::wasm::WasmCompiler;

#[cfg(feature = "wasm-runtime")]
use metadol::wasm::{WasmModule, WasmRuntime};

// ============================================================================
// STACK OVERFLOW TESTS
// ============================================================================

#[cfg(all(feature = "wasm-compile", feature = "wasm-runtime"))]
mod stack_overflow {
    use super::*;

    #[test]
    fn deep_recursion_factorial() {
        // Factorial with very deep recursion
        let source = r#"
fun factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}
"#;

        let decl = parse_file(source).expect("Should parse");
        let mut compiler = WasmCompiler::new();
        let wasm_bytes = compiler.compile(&decl);

        match wasm_bytes {
            Ok(bytes) => {
                let runtime = WasmRuntime::new();
                let module = runtime.load_module(&bytes);

                match module {
                    Ok(mut m) => {
                        // Test with small value first (should work)
                        let result_10 = m.call_function("factorial", &[10i64.into()]);
                        println!("factorial(10) = {:?}", result_10);

                        // Test with moderate value
                        let result_20 = m.call_function("factorial", &[20i64.into()]);
                        match result_20 {
                            Ok(v) => println!("factorial(20) = {:?}", v),
                            Err(e) => println!("NOTE: factorial(20) overflow: {:?}", e),
                        }

                        // Test with large value that will cause stack overflow
                        let result_10000 = m.call_function("factorial", &[10000i64.into()]);
                        match result_10000 {
                            Ok(_) => {
                                println!(
                                    "NOTE: factorial(10000) succeeded (tail call optimization?)"
                                );
                            }
                            Err(e) => {
                                println!("NOTE: factorial(10000) failed as expected: {:?}", e);
                                // Verify it's a stack overflow or recursion limit error
                            }
                        }
                    }
                    Err(e) => println!("NOTE: Module load failed: {:?}", e),
                }
            }
            Err(e) => println!("NOTE: Compilation failed: {:?}", e),
        }
    }

    #[test]
    fn mutual_recursion_stack_overflow() {
        // Mutual recursion between two functions
        let source = r#"
fun is_even(n: i64) -> bool {
    if n == 0 { return true }
    return is_odd(n - 1)
}

fun is_odd(n: i64) -> bool {
    if n == 0 { return false }
    return is_even(n - 1)
}
"#;

        let decl = parse_file(source);
        match decl {
            Ok(d) => {
                let mut compiler = WasmCompiler::new();
                match compiler.compile(&d) {
                    Ok(bytes) => {
                        let runtime = WasmRuntime::new();
                        match runtime.load_module(&bytes) {
                            Ok(mut m) => {
                                // Test with large value
                                let result = m.call_function("is_even", &[100000i64.into()]);
                                match result {
                                    Ok(v) => println!("NOTE: is_even(100000) = {:?}", v),
                                    Err(e) => {
                                        println!("NOTE: Mutual recursion limit hit: {:?}", e)
                                    }
                                }
                            }
                            Err(e) => println!("NOTE: Module load failed: {:?}", e),
                        }
                    }
                    Err(e) => println!("NOTE: Compilation failed: {:?}", e),
                }
            }
            Err(e) => println!("NOTE: Parse failed (mutual recursion): {:?}", e),
        }
    }

    #[test]
    fn fibonacci_recursive_deep() {
        // Exponential recursion - tests both stack and time limits
        let source = r#"
fun fib(n: i64) -> i64 {
    if n <= 1 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}
"#;

        let decl = parse_file(source).expect("Should parse");
        let mut compiler = WasmCompiler::new();
        let wasm_bytes = compiler.compile(&decl);

        match wasm_bytes {
            Ok(bytes) => {
                let runtime = WasmRuntime::new();
                match runtime.load_module(&bytes) {
                    Ok(mut m) => {
                        // fib(30) = 832040, should work
                        let result_30 = m.call_function("fib", &[30i64.into()]);
                        match result_30 {
                            Ok(v) => println!("fib(30) = {:?}", v),
                            Err(e) => println!("NOTE: fib(30) failed: {:?}", e),
                        }

                        // fib(40) might be too slow/deep
                        // Don't actually run this in normal tests - just document
                        println!("NOTE: fib(40+) would likely timeout or overflow");
                    }
                    Err(e) => println!("NOTE: Module load failed: {:?}", e),
                }
            }
            Err(e) => println!("NOTE: Compilation failed: {:?}", e),
        }
    }
}

// ============================================================================
// MEMORY LIMIT TESTS
// ============================================================================

#[cfg(all(feature = "wasm-compile", feature = "wasm-runtime"))]
mod memory_limits {
    use super::*;

    #[test]
    fn large_array_allocation() {
        // Try to allocate a very large array
        let source = r#"
fun create_large_array() -> i64 {
    let arr: Vec<i64> = []
    let i = 0
    while i < 1000000 {
        arr = push(arr, i)
        i = i + 1
    }
    return len(arr)
}
"#;

        let decl = parse_file(source);
        match decl {
            Ok(d) => {
                let mut compiler = WasmCompiler::new();
                match compiler.compile(&d) {
                    Ok(bytes) => {
                        let runtime = WasmRuntime::new();
                        match runtime.load_module(&bytes) {
                            Ok(mut m) => {
                                let result = m.call_function("create_large_array", &[]);
                                match result {
                                    Ok(v) => {
                                        println!("NOTE: Created array with {} elements", v)
                                    }
                                    Err(e) => {
                                        println!("NOTE: Large array allocation failed: {:?}", e)
                                    }
                                }
                            }
                            Err(e) => println!("NOTE: Module load failed: {:?}", e),
                        }
                    }
                    Err(e) => println!("NOTE: Compilation failed: {:?}", e),
                }
            }
            Err(e) => println!("NOTE: Parse failed: {:?}", e),
        }
    }

    #[test]
    fn nested_struct_allocation() {
        // Create deeply nested data structures
        let source = r#"
gen Node {
    has value: i64
    has left: Option<Node>
    has right: Option<Node>
}

fun create_tree(depth: i64) -> Node {
    if depth <= 0 {
        return Node { value: 0, left: None, right: None }
    }
    let left_child = create_tree(depth - 1)
    let right_child = create_tree(depth - 1)
    return Node { value: depth, left: Some(left_child), right: Some(right_child) }
}
"#;

        let decl = parse_file(source);
        match decl {
            Ok(d) => {
                let mut compiler = WasmCompiler::new();
                match compiler.compile(&d) {
                    Ok(bytes) => {
                        let runtime = WasmRuntime::new();
                        match runtime.load_module(&bytes) {
                            Ok(mut m) => {
                                // Depth 10 = 2^10 = 1024 nodes
                                let result_10 = m.call_function("create_tree", &[10i64.into()]);
                                match result_10 {
                                    Ok(_) => println!("NOTE: Created tree with depth 10"),
                                    Err(e) => println!("NOTE: Tree depth 10 failed: {:?}", e),
                                }

                                // Depth 20 = 2^20 = 1M nodes
                                let result_20 = m.call_function("create_tree", &[20i64.into()]);
                                match result_20 {
                                    Ok(_) => println!("NOTE: Created tree with depth 20"),
                                    Err(e) => println!("NOTE: Tree depth 20 failed: {:?}", e),
                                }
                            }
                            Err(e) => println!("NOTE: Module load failed: {:?}", e),
                        }
                    }
                    Err(e) => println!("NOTE: Compilation failed: {:?}", e),
                }
            }
            Err(e) => println!("NOTE: Parse failed: {:?}", e),
        }
    }

    #[test]
    fn string_concatenation_memory() {
        // Exponential string growth
        let source = r#"
fun grow_string(iterations: i64) -> string {
    let s = "a"
    let i = 0
    while i < iterations {
        s = s + s
        i = i + 1
    }
    return s
}
"#;

        let decl = parse_file(source);
        match decl {
            Ok(d) => {
                let mut compiler = WasmCompiler::new();
                match compiler.compile(&d) {
                    Ok(bytes) => {
                        let runtime = WasmRuntime::new();
                        match runtime.load_module(&bytes) {
                            Ok(mut m) => {
                                // 10 iterations = 2^10 = 1024 chars
                                let result_10 = m.call_function("grow_string", &[10i64.into()]);
                                match result_10 {
                                    Ok(_) => println!("NOTE: Created string 2^10 chars"),
                                    Err(e) => println!("NOTE: String 2^10 failed: {:?}", e),
                                }

                                // 20 iterations = 2^20 = 1M chars
                                let result_20 = m.call_function("grow_string", &[20i64.into()]);
                                match result_20 {
                                    Ok(_) => println!("NOTE: Created string 2^20 chars"),
                                    Err(e) => println!("NOTE: String 2^20 failed: {:?}", e),
                                }

                                // 30 iterations = 2^30 = 1B chars
                                let result_30 = m.call_function("grow_string", &[30i64.into()]);
                                match result_30 {
                                    Ok(_) => println!("NOTE: Created string 2^30 chars"),
                                    Err(e) => {
                                        println!("NOTE: String 2^30 failed (expected): {:?}", e)
                                    }
                                }
                            }
                            Err(e) => println!("NOTE: Module load failed: {:?}", e),
                        }
                    }
                    Err(e) => println!("NOTE: Compilation failed: {:?}", e),
                }
            }
            Err(e) => println!("NOTE: Parse failed: {:?}", e),
        }
    }
}

// ============================================================================
// HOT RELOAD TESTS
// ============================================================================

#[cfg(all(feature = "wasm-compile", feature = "wasm-runtime"))]
mod hot_reload {
    use super::*;

    #[test]
    fn reload_function_definition() {
        // Test hot-reloading a modified function
        let source_v1 = r#"
fun get_value() -> i64 {
    return 42
}
"#;

        let source_v2 = r#"
fun get_value() -> i64 {
    return 100
}
"#;

        let decl_v1 = parse_file(source_v1).expect("Should parse v1");
        let decl_v2 = parse_file(source_v2).expect("Should parse v2");

        let mut compiler = WasmCompiler::new();
        let bytes_v1 = compiler.compile(&decl_v1);
        let bytes_v2 = compiler.compile(&decl_v2);

        match (bytes_v1, bytes_v2) {
            (Ok(b1), Ok(b2)) => {
                let runtime = WasmRuntime::new();

                // Load v1
                match runtime.load_module(&b1) {
                    Ok(mut m1) => {
                        let result_v1 = m1.call_function("get_value", &[]);
                        println!("V1 result: {:?}", result_v1);
                    }
                    Err(e) => println!("NOTE: V1 module load failed: {:?}", e),
                }

                // Load v2 (simulating hot reload)
                match runtime.load_module(&b2) {
                    Ok(mut m2) => {
                        let result_v2 = m2.call_function("get_value", &[]);
                        println!("V2 result: {:?}", result_v2);
                    }
                    Err(e) => println!("NOTE: V2 module load failed: {:?}", e),
                }
            }
            (Err(e1), _) => println!("NOTE: V1 compilation failed: {:?}", e1),
            (_, Err(e2)) => println!("NOTE: V2 compilation failed: {:?}", e2),
        }
    }

    #[test]
    fn reload_with_state_change() {
        // Test that state is handled correctly during reload
        let source_with_state = r#"
let counter: i64 = 0

fun increment() -> i64 {
    counter = counter + 1
    return counter
}

fun get_counter() -> i64 {
    return counter
}
"#;

        let decl = parse_file(source_with_state);
        match decl {
            Ok(d) => {
                let mut compiler = WasmCompiler::new();
                match compiler.compile(&d) {
                    Ok(bytes) => {
                        let runtime = WasmRuntime::new();
                        match runtime.load_module(&bytes) {
                            Ok(mut m) => {
                                // Increment a few times
                                for i in 0..5 {
                                    let r = m.call_function("increment", &[]);
                                    println!("Increment {}: {:?}", i, r);
                                }

                                // Note: On reload, state would be lost
                                println!(
                                    "NOTE: Hot reload would reset counter state to initial value"
                                );
                            }
                            Err(e) => println!("NOTE: Module load failed: {:?}", e),
                        }
                    }
                    Err(e) => println!("NOTE: Compilation failed: {:?}", e),
                }
            }
            Err(e) => println!("NOTE: Parse failed: {:?}", e),
        }
    }

    #[test]
    fn reload_with_signature_change() {
        // Test reloading when function signature changes
        let source_v1 = r#"
fun process(x: i64) -> i64 {
    return x * 2
}
"#;

        let source_v2 = r#"
fun process(x: i64, y: i64) -> i64 {
    return x + y
}
"#;

        let decl_v1 = parse_file(source_v1).expect("Should parse v1");
        let decl_v2 = parse_file(source_v2).expect("Should parse v2");

        let mut compiler = WasmCompiler::new();
        let bytes_v1 = compiler.compile(&decl_v1);
        let bytes_v2 = compiler.compile(&decl_v2);

        match (bytes_v1, bytes_v2) {
            (Ok(b1), Ok(b2)) => {
                let runtime = WasmRuntime::new();

                // Load v1 - takes 1 arg
                match runtime.load_module(&b1) {
                    Ok(mut m1) => {
                        let result = m1.call_function("process", &[5i64.into()]);
                        println!("V1 process(5) = {:?}", result);
                    }
                    Err(e) => println!("NOTE: V1 load failed: {:?}", e),
                }

                // Load v2 - takes 2 args
                match runtime.load_module(&b2) {
                    Ok(mut m2) => {
                        let result = m2.call_function("process", &[5i64.into(), 3i64.into()]);
                        println!("V2 process(5, 3) = {:?}", result);

                        // Try calling with old signature (should fail or use default)
                        let bad_result = m2.call_function("process", &[5i64.into()]);
                        match bad_result {
                            Ok(v) => println!("NOTE: V2 with 1 arg (old signature) = {:?}", v),
                            Err(e) => {
                                println!(
                                    "NOTE: V2 with 1 arg correctly rejects old signature: {:?}",
                                    e
                                )
                            }
                        }
                    }
                    Err(e) => println!("NOTE: V2 load failed: {:?}", e),
                }
            }
            _ => println!("NOTE: Compilation failed"),
        }
    }
}

// ============================================================================
// INVALID WASM RECOVERY TESTS
// ============================================================================

#[cfg(feature = "wasm-runtime")]
mod invalid_wasm {
    use super::*;

    #[test]
    fn load_invalid_bytes() {
        let runtime = WasmRuntime::new();

        // Try to load garbage bytes
        let garbage = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let result = runtime.load_module(&garbage);

        assert!(result.is_err(), "Loading garbage should fail");
        if let Err(e) = result {
            println!("NOTE: Invalid WASM bytes error: {:?}", e);
        }
    }

    #[test]
    fn load_truncated_wasm() {
        // Valid WASM header but truncated
        let truncated = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // Version 1
                  // Missing rest of module
        ];

        let runtime = WasmRuntime::new();
        let result = runtime.load_module(&truncated);

        match result {
            Ok(_) => println!("NOTE: Truncated WASM was accepted (empty module?)"),
            Err(e) => println!("NOTE: Truncated WASM correctly rejected: {:?}", e),
        }
    }

    #[test]
    fn load_empty_bytes() {
        let runtime = WasmRuntime::new();
        let empty: Vec<u8> = vec![];
        let result = runtime.load_module(&empty);

        assert!(result.is_err(), "Loading empty bytes should fail");
    }

    #[test]
    fn call_nonexistent_function() {
        let source = r#"
fun existing_function() -> i64 {
    return 42
}
"#;

        let decl = parse_file(source);
        if let Ok(d) = decl {
            #[cfg(feature = "wasm-compile")]
            {
                let mut compiler = WasmCompiler::new();
                if let Ok(bytes) = compiler.compile(&d) {
                    let runtime = WasmRuntime::new();
                    if let Ok(mut m) = runtime.load_module(&bytes) {
                        // Try to call function that doesn't exist
                        let result = m.call_function("nonexistent_function", &[]);
                        match result {
                            Ok(v) => {
                                println!("BUG: Calling nonexistent function returned: {:?}", v)
                            }
                            Err(e) => {
                                println!("NOTE: Correctly rejected nonexistent function: {:?}", e)
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn call_with_wrong_argument_count() {
        let source = r#"
fun add(a: i64, b: i64) -> i64 {
    return a + b
}
"#;

        let decl = parse_file(source);
        if let Ok(d) = decl {
            #[cfg(feature = "wasm-compile")]
            {
                let mut compiler = WasmCompiler::new();
                if let Ok(bytes) = compiler.compile(&d) {
                    let runtime = WasmRuntime::new();
                    if let Ok(mut m) = runtime.load_module(&bytes) {
                        // Too few arguments
                        let result_1 = m.call_function("add", &[1i64.into()]);
                        match result_1 {
                            Ok(v) => println!("NOTE: add(1) with missing arg = {:?}", v),
                            Err(e) => {
                                println!(
                                    "NOTE: Correctly rejected 1 arg for 2-arg function: {:?}",
                                    e
                                )
                            }
                        }

                        // Too many arguments
                        let result_3 =
                            m.call_function("add", &[1i64.into(), 2i64.into(), 3i64.into()]);
                        match result_3 {
                            Ok(v) => println!("NOTE: add(1,2,3) with extra arg = {:?}", v),
                            Err(e) => {
                                println!(
                                    "NOTE: Correctly rejected 3 args for 2-arg function: {:?}",
                                    e
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn call_with_wrong_argument_types() {
        let source = r#"
fun double(x: i64) -> i64 {
    return x * 2
}
"#;

        let decl = parse_file(source);
        if let Ok(d) = decl {
            #[cfg(feature = "wasm-compile")]
            {
                let mut compiler = WasmCompiler::new();
                if let Ok(bytes) = compiler.compile(&d) {
                    let runtime = WasmRuntime::new();
                    if let Ok(mut m) = runtime.load_module(&bytes) {
                        // Try passing f64 instead of i64
                        // Note: This depends on how the runtime handles type coercion
                        let result = m.call_function("double", &[3.14f64.into()]);
                        match result {
                            Ok(v) => println!("NOTE: double(3.14) with type coercion = {:?}", v),
                            Err(e) => println!("NOTE: Type mismatch rejected: {:?}", e),
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// CONCURRENT ACCESS TESTS
// ============================================================================

#[cfg(all(feature = "wasm-compile", feature = "wasm-runtime"))]
mod concurrent_access {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn multiple_module_instances() {
        let source = r#"
fun compute(x: i64) -> i64 {
    return x * x
}
"#;

        let decl = parse_file(source);
        if let Ok(d) = decl {
            let mut compiler = WasmCompiler::new();
            if let Ok(bytes) = compiler.compile(&d) {
                let bytes = Arc::new(bytes);
                let runtime = Arc::new(WasmRuntime::new());

                let handles: Vec<_> = (0..4)
                    .map(|i| {
                        let bytes = Arc::clone(&bytes);
                        let runtime = Arc::clone(&runtime);
                        thread::spawn(move || {
                            // Each thread creates its own module instance
                            match runtime.load_module(&bytes) {
                                Ok(mut m) => {
                                    let result = m.call_function("compute", &[(i * 10).into()]);
                                    println!("Thread {} compute({}) = {:?}", i, i * 10, result);
                                }
                                Err(e) => println!("Thread {} load failed: {:?}", i, e),
                            }
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.join().expect("Thread should complete");
                }
            }
        }
    }

    #[test]
    fn shared_runtime_concurrent_loads() {
        let sources = vec![
            "fun f1() -> i64 { return 1 }",
            "fun f2() -> i64 { return 2 }",
            "fun f3() -> i64 { return 3 }",
            "fun f4() -> i64 { return 4 }",
        ];

        let runtime = Arc::new(WasmRuntime::new());

        let handles: Vec<_> = sources
            .into_iter()
            .enumerate()
            .map(|(i, source)| {
                let runtime = Arc::clone(&runtime);
                let source = source.to_string();
                thread::spawn(move || {
                    let decl = parse_file(&source);
                    if let Ok(d) = decl {
                        let mut compiler = WasmCompiler::new();
                        if let Ok(bytes) = compiler.compile(&d) {
                            match runtime.load_module(&bytes) {
                                Ok(mut m) => {
                                    let fname = format!("f{}", i + 1);
                                    let result = m.call_function(&fname, &[]);
                                    println!("Concurrent load {}: {} = {:?}", i, fname, result);
                                }
                                Err(e) => println!("Concurrent load {} failed: {:?}", i, e),
                            }
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread should complete");
        }
    }
}

// ============================================================================
// PERFORMANCE EDGE CASES
// ============================================================================

#[cfg(all(feature = "wasm-compile", feature = "wasm-runtime"))]
mod performance {
    use super::*;
    use std::time::Instant;

    #[test]
    fn tight_loop_performance() {
        let source = r#"
fun sum_to(n: i64) -> i64 {
    let total = 0
    let i = 0
    while i < n {
        total = total + i
        i = i + 1
    }
    return total
}
"#;

        let decl = parse_file(source);
        if let Ok(d) = decl {
            let mut compiler = WasmCompiler::new();
            if let Ok(bytes) = compiler.compile(&d) {
                let runtime = WasmRuntime::new();
                if let Ok(mut m) = runtime.load_module(&bytes) {
                    // Time various sizes
                    for exp in [4, 5, 6, 7] {
                        let n = 10i64.pow(exp);
                        let start = Instant::now();
                        let result = m.call_function("sum_to", &[n.into()]);
                        let elapsed = start.elapsed();
                        println!("sum_to(10^{}) = {:?} in {:?}", exp, result, elapsed);
                    }
                }
            }
        }
    }

    #[test]
    fn compilation_time_large_function() {
        // Generate a large function with many statements
        let mut source = String::from("fun large_function() -> i64 {\n    let x = 0\n");
        for i in 0..1000 {
            source.push_str(&format!("    x = x + {}\n", i));
        }
        source.push_str("    return x\n}\n");

        let start = Instant::now();
        let decl = parse_file(&source);
        let parse_time = start.elapsed();
        println!("Parse time for 1000-statement function: {:?}", parse_time);

        if let Ok(d) = decl {
            let start = Instant::now();
            let mut compiler = WasmCompiler::new();
            let result = compiler.compile(&d);
            let compile_time = start.elapsed();
            println!(
                "Compile time for 1000-statement function: {:?}",
                compile_time
            );

            if let Ok(bytes) = result {
                println!("WASM output size: {} bytes", bytes.len());
            }
        }
    }
}

// ============================================================================
// PHYSICS FORMULA WASM TESTS
// ============================================================================

#[cfg(all(feature = "wasm-compile", feature = "wasm-runtime"))]
mod physics_wasm {
    use super::*;

    #[test]
    fn gravitational_force_wasm() {
        let source = r#"
const G: f64 = 6.67430e-11

fun gravitational_force(m1: f64, m2: f64, r: f64) -> f64 {
    if r == 0.0 {
        return 1.0e308
    }
    return G * m1 * m2 / (r * r)
}
"#;

        let decl = parse_file(source);
        if let Ok(d) = decl {
            let mut compiler = WasmCompiler::new();
            if let Ok(bytes) = compiler.compile(&d) {
                let runtime = WasmRuntime::new();
                if let Ok(mut m) = runtime.load_module(&bytes) {
                    // Earth-Moon distance
                    let m_earth = 5.972e24f64;
                    let m_moon = 7.342e22f64;
                    let r = 3.844e8f64;

                    let result = m.call_function(
                        "gravitational_force",
                        &[m_earth.into(), m_moon.into(), r.into()],
                    );
                    println!("Earth-Moon gravitational force: {:?} N", result);

                    // Edge case: r = 0
                    let edge_result = m.call_function(
                        "gravitational_force",
                        &[1.0f64.into(), 1.0f64.into(), 0.0f64.into()],
                    );
                    println!("Force at r=0: {:?}", edge_result);
                }
            }
        }
    }

    #[test]
    fn ideal_gas_law_wasm() {
        let source = r#"
const R: f64 = 8.314

fun calculate_pressure(moles: f64, temperature: f64, volume: f64) -> f64 {
    if volume == 0.0 {
        return 1.0e308
    }
    return (moles * R * temperature) / volume
}
"#;

        let decl = parse_file(source);
        if let Ok(d) = decl {
            let mut compiler = WasmCompiler::new();
            if let Ok(bytes) = compiler.compile(&d) {
                let runtime = WasmRuntime::new();
                if let Ok(mut m) = runtime.load_module(&bytes) {
                    // STP: 1 mol at 273.15 K in 22.4 L
                    let n = 1.0f64;
                    let t = 273.15f64;
                    let v = 0.0224f64; // m³

                    let result =
                        m.call_function("calculate_pressure", &[n.into(), t.into(), v.into()]);
                    println!("Pressure at STP: {:?} Pa (expected ~101325)", result);

                    // Edge case: v = 0
                    let edge_result = m.call_function(
                        "calculate_pressure",
                        &[1.0f64.into(), 300.0f64.into(), 0.0f64.into()],
                    );
                    println!("Pressure at V=0: {:?}", edge_result);
                }
            }
        }
    }
}
