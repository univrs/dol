//! Debug test for WASM generation
#![cfg(feature = "wasm")]

use metadol::parse_file;
use metadol::wasm::layout::{FieldLayout, GeneLayout, WasmFieldType};
use metadol::wasm::{WasmCompiler, WasmRuntime};

#[test]
fn debug_control_flow_wasm() {
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

    // Print the AST to understand the structure
    println!("Parsed AST:\n{:#?}", module);

    // Compile to WASM
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    println!("\nWASM size: {} bytes", wasm_bytes.len());
    println!("WASM hex dump:");
    for (i, byte) in wasm_bytes.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();

    // Try to load with WasmRuntime to get specific error
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    match runtime.load(&wasm_bytes) {
        Ok(_) => println!("WasmRuntime load: SUCCESS"),
        Err(e) => println!("WasmRuntime load FAILED: {:?}", e),
    }
}

#[test]
fn debug_match_wasm() {
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

    println!("Match AST:\n{:#?}", module);

    // Compile to WASM
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    println!("\nMatch WASM size: {} bytes", wasm_bytes.len());
    println!("WASM hex dump:");
    for (i, byte) in wasm_bytes.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();

    // Try to load with WasmRuntime to get specific error
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    match runtime.load(&wasm_bytes) {
        Ok(_) => println!("WasmRuntime load: SUCCESS"),
        Err(e) => println!("WasmRuntime load FAILED: {:?}", e),
    }
}

#[test]
fn test_member_access() {
    // Test member access on struct literals
    let source = r#"
fun get_x() -> i64 {
    let p = Point { x: 42, y: 10 }
    return p.x
}
exegesis { Gets the x coordinate. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    // Print AST for debugging
    println!("Parsed AST:\n{:#?}", module);

    // Create a Point layout
    let point_layout = GeneLayout {
        name: "Point".to_string(),
        fields: vec![
            FieldLayout::primitive("x", 0, WasmFieldType::I64),
            FieldLayout::primitive("y", 8, WasmFieldType::I64),
        ],
        total_size: 16,
        alignment: 8,
    };

    // Create compiler and register the layout
    let mut compiler = WasmCompiler::new();
    compiler.register_gene_layout(point_layout);

    // Compile to WASM
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    println!("Member access WASM size: {} bytes", wasm_bytes.len());

    // Validate by loading into runtime
    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Call the function
    let result = wasm_module.call("get_x", &[]).expect("Call failed");
    println!("get_x() returned: {:?}", result);

    // Verify result
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));
}

#[test]
fn test_function_call() {
    // Test calling one function from another
    // Note: The compiler currently only supports single function modules,
    // but this test is here for when multi-function modules are supported.
    // For now, we test that the function index lookup at least doesn't fail.
    let source = r#"
fun double(x: i64) -> i64 {
    return x * 2
}
exegesis { Doubles a number. }
"#;
    let module = parse_file(source).expect("Failed to parse");

    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module).expect("Compilation failed");

    println!("Function call WASM size: {} bytes", wasm_bytes.len());

    let runtime = WasmRuntime::new().expect("Failed to create runtime");
    let mut wasm_module = runtime.load(&wasm_bytes).expect("Failed to load module");

    // Test the function
    let result = wasm_module
        .call("double", &[21i64.into()])
        .expect("Call failed");
    println!("double(21) returned: {:?}", result);
    assert_eq!(result.first().and_then(|v| v.i64()), Some(42));
}
