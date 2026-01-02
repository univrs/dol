//! Compile Counter.dol to counter.wasm for vertical slice testing
//! Run with: cargo run --example compile_counter --features wasm

use metadol::parse_dol_file;
use metadol::wasm::WasmCompiler;
use std::fs;

fn main() {
    let source = r#"
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
}

/// Standalone function for simpler test
fun add_numbers(a: Int64, b: Int64) -> Int64 {
    return a + b
}
"#;

    println!("=== Compiling Counter.dol to WASM ===\n");

    // Parse
    println!("--- Parsing ---");
    let file = match parse_dol_file(source) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };
    println!("Parsed {} declarations", file.declarations.len());

    // Compile to WASM
    println!("\n--- Compiling to WASM ---");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = match compiler.compile_file(&file) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Compile error: {}", e.message);
            std::process::exit(1);
        }
    };
    println!("Generated {} bytes of WASM", wasm_bytes.len());

    // Write to file
    fs::create_dir_all("vertical-slice-results").unwrap();
    let output_path = "vertical-slice-results/counter.wasm";
    fs::write(output_path, &wasm_bytes).expect("Failed to write WASM file");
    println!("\nWrote: {}", output_path);

    // Validate with wasmtime
    println!("\n--- Validating WASM ---");
    let engine = wasmtime::Engine::default();
    match wasmtime::Module::new(&engine, &wasm_bytes) {
        Ok(wasm_module) => {
            println!("WASM validated successfully!");
            println!("\nExports:");
            for export in wasm_module.exports() {
                println!("  - {}", export.name());
            }
        }
        Err(e) => {
            eprintln!("Validation error: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== Compilation Complete ===");
}
