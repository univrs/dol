//! Basic example of WASM compilation
//!
//! This example demonstrates how to compile a simple DOL function to WASM bytecode.
//!
//! Run with:
//! ```bash
//! cargo run --example wasm_basic --features wasm
//! ```

#[cfg(feature = "wasm")]
fn main() {
    use metadol::ast::{
        BinaryOp, Declaration, Expr, FunctionDecl, FunctionParam, Purity, Span, Stmt, TypeExpr,
        Visibility,
    };
    use metadol::wasm::WasmCompiler;

    // Create a simple addition function:
    // fun add(a: i64, b: i64) -> i64 { return a + b }
    let func = FunctionDecl {
        visibility: Visibility::Public,
        purity: Purity::Pure,
        name: "add".to_string(),
        type_params: None,
        params: vec![
            FunctionParam {
                name: "a".to_string(),
                type_ann: TypeExpr::Named("i64".to_string()),
            },
            FunctionParam {
                name: "b".to_string(),
                type_ann: TypeExpr::Named("i64".to_string()),
            },
        ],
        return_type: Some(TypeExpr::Named("i64".to_string())),
        body: vec![Stmt::Return(Some(Expr::Binary {
            left: Box::new(Expr::Identifier("a".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expr::Identifier("b".to_string())),
        }))],
        exegesis: "Adds two numbers".to_string(),
        span: Span::default(),
    };

    let decl = Declaration::Function(Box::new(func));

    // Compile to WASM
    let mut compiler = WasmCompiler::new();
    match compiler.compile(&decl) {
        Ok(wasm_bytes) => {
            println!("✓ Successfully compiled to WASM!");
            println!("  Module size: {} bytes", wasm_bytes.len());
            println!(
                "  Magic number: {:02x} {:02x} {:02x} {:02x}",
                wasm_bytes[0], wasm_bytes[1], wasm_bytes[2], wasm_bytes[3]
            );
            println!(
                "  Version: {:02x} {:02x} {:02x} {:02x}",
                wasm_bytes[4], wasm_bytes[5], wasm_bytes[6], wasm_bytes[7]
            );

            // Write to file
            std::fs::write("add.wasm", &wasm_bytes).expect("Failed to write WASM file");
            println!("  Written to: add.wasm");
        }
        Err(e) => {
            eprintln!("✗ Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("This example requires the 'wasm' feature to be enabled.");
    eprintln!("Run with: cargo run --example wasm_basic --features wasm");
}
