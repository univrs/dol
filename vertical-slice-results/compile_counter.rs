//! Compile Counter.dol to counter.wasm for vertical slice testing

use metadol::parse_file;
use metadol::wasm::WasmCompiler;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read Counter.dol source
    let source = fs::read_to_string("vertical-slice-results/counter.dol")?;
    println!("=== Compiling Counter.dol to WASM ===\n");
    println!("Source:\n{}", source);

    // Parse
    println!("\n--- Parsing ---");
    let module = parse_file(&source)?;
    println!("Parsed {} declarations", module.declarations.len());

    // Compile to WASM
    println!("\n--- Compiling to WASM ---");
    let mut compiler = WasmCompiler::new();
    let wasm_bytes = compiler.compile(&module)?;
    println!("Generated {} bytes of WASM", wasm_bytes.len());

    // Write to file
    let output_path = Path::new("vertical-slice-results/counter.wasm");
    fs::write(output_path, &wasm_bytes)?;
    println!("\nWrote: {}", output_path.display());

    // Validate with wasmtime
    println!("\n--- Validating WASM ---");
    let engine = wasmtime::Engine::default();
    match wasmtime::Module::new(&engine, &wasm_bytes) {
        Ok(module) => {
            println!("WASM validated successfully!");
            println!("Exports:");
            for export in module.exports() {
                println!("  - {}: {:?}", export.name(), export.ty());
            }
        }
        Err(e) => {
            eprintln!("Validation error: {}", e);
            return Err(e.into());
        }
    }

    println!("\n=== Compilation Complete ===");
    Ok(())
}
