//! WASM Pipeline Stress Test Binary
//!
//! Tests Parse -> Validate -> WASM for all test cases
//! Run with: cargo run --bin wasm-stress-test --features "cli wasm"

use std::fs;
use std::path::Path;

#[cfg(feature = "wasm")]
use metadol::WasmCompiler;
use metadol::{parse_file_all, validate};

#[derive(Debug)]
struct TestResult {
    name: String,
    level: String,
    parse: Result<usize, String>, // Ok(count) or Err(message)
    validate: Result<(), String>,
    wasm: Result<(), String>,
}

impl TestResult {
    fn parse_ok(&self) -> bool {
        self.parse.is_ok()
    }
    fn validate_ok(&self) -> bool {
        self.validate.is_ok()
    }
    fn wasm_ok(&self) -> bool {
        self.wasm.is_ok()
    }
    fn error_msg(&self) -> String {
        if let Err(e) = &self.parse {
            return e.clone();
        }
        if let Err(e) = &self.validate {
            return e.clone();
        }
        if let Err(e) = &self.wasm {
            return e.clone();
        }
        String::new()
    }
}

fn test_file(path: &Path) -> TestResult {
    let name = path.file_name().unwrap().to_string_lossy().to_string();
    let level = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Read file
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult {
                name,
                level,
                parse: Err(format!("Read: {}", e)),
                validate: Err("N/A".into()),
                wasm: Err("N/A".into()),
            }
        }
    };

    // Parse
    let decls = match parse_file_all(&source) {
        Ok(d) if d.is_empty() => {
            return TestResult {
                name,
                level,
                parse: Err("No declarations".into()),
                validate: Err("N/A".into()),
                wasm: Err("N/A".into()),
            }
        }
        Ok(d) => d,
        Err(e) => {
            return TestResult {
                name,
                level,
                parse: Err(format!("{}", e)),
                validate: Err("N/A".into()),
                wasm: Err("N/A".into()),
            }
        }
    };

    let parse_count = decls.len();

    // Validate
    let mut validation_errors = Vec::new();
    for decl in &decls {
        let result = validate(decl);
        for err in &result.errors {
            validation_errors.push(format!("{:?}", err));
        }
    }
    let validate_result = if validation_errors.is_empty() {
        Ok(())
    } else {
        Err(validation_errors.join("; "))
    };

    // WASM Compilation
    #[cfg(feature = "wasm")]
    let wasm_result = {
        let compiler = WasmCompiler::new();
        let mut wasm_success = false;
        let mut wasm_error = String::from("No functions");

        for decl in &decls {
            match compiler.compile(decl) {
                Ok(bytes) => {
                    wasm_success = true;
                    wasm_error = format!("{} bytes", bytes.len());
                    break;
                }
                Err(e) => {
                    wasm_error = e.message;
                }
            }
        }

        if wasm_success {
            Ok(())
        } else {
            Err(wasm_error)
        }
    };

    #[cfg(not(feature = "wasm"))]
    let wasm_result = Err("WASM feature disabled".into());

    TestResult {
        name,
        level,
        parse: Ok(parse_count),
        validate: validate_result,
        wasm: wasm_result,
    }
}

fn main() {
    let test_dirs = [
        "test-cases/level1-minimal",
        "test-cases/level2-basic",
        "test-cases/level3-types",
        "test-cases/level4-control",
        "test-cases/level5-advanced",
    ];

    println!("========================================");
    println!("  DOL -> WASM Pipeline Stress Test");
    println!("========================================\n");

    let mut results = Vec::new();

    for dir in &test_dirs {
        let path = Path::new(dir);
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "dol").unwrap_or(false) {
                    results.push(test_file(&path));
                }
            }
        }
    }

    // Sort by level
    results.sort_by(|a, b| a.level.cmp(&b.level).then_with(|| a.name.cmp(&b.name)));

    // Print header
    println!(
        "{:<30} | {:^7} | {:^8} | {:^6} | Error",
        "Test File", "Parse", "Validate", "WASM"
    );
    println!(
        "{:-<30}-+-{:-^7}-+-{:-^8}-+-{:-^6}-+-{:-<50}",
        "", "", "", "", ""
    );

    // Print results
    let mut stats = (0, 0, 0, 0, 0, 0); // parse_pass, parse_fail, val_pass, val_fail, wasm_pass, wasm_na

    for r in &results {
        let parse_str = if r.parse_ok() { "PASS" } else { "FAIL" };
        let validate_str = if r.validate_ok() {
            "PASS"
        } else if r.parse_ok() {
            "FAIL"
        } else {
            "-"
        };
        let wasm_str = if r.wasm_ok() {
            "PASS"
        } else if r.parse_ok() {
            "N/A"
        } else {
            "-"
        };

        let err = r.error_msg();
        let err_display = if err.len() > 50 {
            format!("{}...", &err[..47])
        } else {
            err
        };

        println!(
            "{:<30} | {:^7} | {:^8} | {:^6} | {}",
            r.name, parse_str, validate_str, wasm_str, err_display
        );

        if r.parse_ok() {
            stats.0 += 1;
        } else {
            stats.1 += 1;
        }
        if r.validate_ok() {
            stats.2 += 1;
        } else if r.parse_ok() {
            stats.3 += 1;
        }
        if r.wasm_ok() {
            stats.4 += 1;
        } else if r.parse_ok() {
            stats.5 += 1;
        }
    }

    println!("\n========================================");
    println!("  Summary");
    println!("========================================");
    println!("Total tests:      {}", results.len());
    println!("Parse:            {} passed, {} failed", stats.0, stats.1);
    println!("Validate:         {} passed, {} failed", stats.2, stats.3);
    println!(
        "WASM Compile:     {} passed, {} N/A (non-function)",
        stats.4, stats.5
    );
    println!();

    // Organize into working/failing
    let working_dir = Path::new("test-cases/working");
    let failing_dir = Path::new("test-cases/failing");

    fs::create_dir_all(working_dir).ok();
    fs::create_dir_all(failing_dir).ok();

    for r in &results {
        let src_path = Path::new("test-cases").join(&r.level).join(&r.name);

        if r.parse_ok() && r.validate_ok() {
            let dst = working_dir.join(&r.name);
            fs::copy(&src_path, &dst).ok();
        } else {
            let dst = failing_dir.join(&r.name);
            fs::copy(&src_path, &dst).ok();
        }
    }

    println!("Tests organized into test-cases/working/ and test-cases/failing/");
}
