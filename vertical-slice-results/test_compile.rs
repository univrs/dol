#[cfg(feature = "wasm")]
mod vertical_slice {
    use metadol::parse_file;
    use metadol::wasm::WasmCompiler;
    use std::fs;

    #[test]
    fn compile_counter_to_wasm() {
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
        
        let module = parse_file(source).expect("Parse failed");
        let mut compiler = WasmCompiler::new();
        let wasm_bytes = compiler.compile(&module).expect("Compile failed");
        
        fs::write("vertical-slice-results/counter.wasm", &wasm_bytes).expect("Write failed");
        println!("Wrote {} bytes to counter.wasm", wasm_bytes.len());
        
        // Validate
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, &wasm_bytes).expect("Invalid WASM");
        for export in module.exports() {
            println!("Export: {} ({:?})", export.name(), export.ty());
        }
    }
}
