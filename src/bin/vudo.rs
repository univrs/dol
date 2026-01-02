//! vudo - VUDO Spirit runtime and toolchain
//!
//! A unified CLI for compiling, running, and checking DOL spirits.
//!
//! # Usage
//!
//! ```bash
//! # Run a WASM spirit
//! vudo run counter.wasm
//! vudo run counter.wasm -f add_numbers -a '[3, 4]'
//!
//! # Compile DOL to WASM
//! vudo compile counter.dol -o counter.wasm
//!
//! # Type-check DOL files
//! vudo check counter.dol
//! ```

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::process::ExitCode;

/// VUDO Spirit runtime and toolchain
#[derive(Parser, Debug)]
#[command(name = "vudo")]
#[command(author, version, about = "VUDO Spirit runtime and toolchain")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a WASM spirit
    Run(RunArgs),

    /// Compile DOL to WASM
    Compile(CompileArgs),

    /// Type-check DOL files
    Check(CheckArgs),
}

/// Arguments for the run command
#[derive(Parser, Debug)]
struct RunArgs {
    /// Path to WASM file
    #[arg(required = true)]
    file: PathBuf,

    /// Function to call (default: main or first exported function)
    #[arg(short, long)]
    function: Option<String>,

    /// Arguments as JSON array (e.g., '[3, 4]' or '[1234]')
    #[arg(short, long)]
    args: Option<String>,

    /// Initial memory pages (default: 16 = 1MB)
    #[arg(long, default_value = "16")]
    memory: u32,

    /// Enable execution tracing
    #[arg(long)]
    trace: bool,
}

/// Arguments for the compile command
#[derive(Parser, Debug)]
struct CompileArgs {
    /// Path to DOL file
    #[arg(required = true)]
    file: PathBuf,

    /// Output file (default: <input>.wasm)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable optimization
    #[arg(long)]
    optimize: bool,

    /// Include debug info in output
    #[arg(long)]
    debug: bool,
}

/// Arguments for the check command
#[derive(Parser, Debug)]
struct CheckArgs {
    /// Files or directories to check
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Treat warnings as errors
    #[arg(long)]
    strict: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Recursively check directories
    #[arg(short, long)]
    recursive: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run(args) => cmd_run(args, cli.verbose, cli.quiet),
        Commands::Compile(args) => cmd_compile(args, cli.verbose, cli.quiet),
        Commands::Check(args) => cmd_check(args, cli.verbose, cli.quiet),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", "error".red(), e);
            ExitCode::FAILURE
        }
    }
}

// =============================================================================
// Run Command
// =============================================================================

#[cfg(feature = "wasm")]
fn cmd_run(args: RunArgs, verbose: bool, quiet: bool) -> Result<(), String> {
    use wasmtime::{Engine, Instance, Module, Store, Val};

    if !args.file.exists() {
        return Err(format!("File not found: {}", args.file.display()));
    }

    if verbose {
        eprintln!("{} {}", "Loading".cyan(), args.file.display());
    }

    // Read WASM bytes
    let wasm_bytes =
        std::fs::read(&args.file).map_err(|e| format!("Failed to read WASM file: {}", e))?;

    if verbose {
        eprintln!("  {} bytes loaded", wasm_bytes.len());
    }

    // Create wasmtime engine and module
    let engine = Engine::default();
    let module =
        Module::new(&engine, &wasm_bytes).map_err(|e| format!("Failed to compile WASM: {}", e))?;

    // Create store with default state
    let mut store = Store::new(&engine, ());

    // Create instance
    let instance = Instance::new(&mut store, &module, &[])
        .map_err(|e| format!("Failed to instantiate WASM: {}", e))?;

    // Find function to call
    let func_name = args.function.as_deref().unwrap_or_else(|| {
        // Try to find main, or use first exported function
        if instance.get_func(&mut store, "main").is_some() {
            "main"
        } else {
            // Get first exported function
            module
                .exports()
                .find(|e| e.ty().func().is_some())
                .map(|e| e.name())
                .unwrap_or("main")
        }
    });

    let func = instance
        .get_func(&mut store, func_name)
        .ok_or_else(|| format!("Function '{}' not found", func_name))?;

    if verbose {
        eprintln!("{} {}()", "Calling".cyan(), func_name);
    }

    // Parse arguments
    let call_args = parse_json_args(&args.args, &func, &store)?;

    // Prepare result storage
    let func_ty = func.ty(&store);
    let mut results: Vec<Val> = func_ty.results().map(|_| Val::I64(0)).collect();

    // Call function
    func.call(&mut store, &call_args, &mut results)
        .map_err(|e| format!("Execution error: {}", e))?;

    // Print results
    if !quiet {
        if results.is_empty() {
            println!("(no return value)");
        } else if results.len() == 1 {
            println!("{}", format_val(&results[0]));
        } else {
            let formatted: Vec<String> = results.iter().map(format_val).collect();
            println!("({})", formatted.join(", "));
        }
    }

    Ok(())
}

#[cfg(feature = "wasm")]
fn parse_json_args(
    args_json: &Option<String>,
    func: &wasmtime::Func,
    store: &wasmtime::Store<()>,
) -> Result<Vec<wasmtime::Val>, String> {
    use wasmtime::{Val, ValType};

    let Some(json_str) = args_json else {
        return Ok(vec![]);
    };

    // Parse JSON array
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON arguments: {}", e))?;

    let arr = parsed
        .as_array()
        .ok_or_else(|| "Arguments must be a JSON array".to_string())?;

    let func_ty = func.ty(store);
    let param_types: Vec<_> = func_ty.params().collect();

    if arr.len() != param_types.len() {
        return Err(format!(
            "Expected {} arguments, got {}",
            param_types.len(),
            arr.len()
        ));
    }

    let mut vals = Vec::with_capacity(arr.len());

    for (i, (val, ty)) in arr.iter().zip(param_types.iter()).enumerate() {
        let wasm_val = match ty {
            ValType::I32 => {
                let n = val
                    .as_i64()
                    .ok_or_else(|| format!("Argument {} must be an integer", i))?;
                Val::I32(n as i32)
            }
            ValType::I64 => {
                let n = val
                    .as_i64()
                    .ok_or_else(|| format!("Argument {} must be an integer", i))?;
                Val::I64(n)
            }
            ValType::F32 => {
                let n = val
                    .as_f64()
                    .ok_or_else(|| format!("Argument {} must be a number", i))?;
                Val::F32((n as f32).to_bits())
            }
            ValType::F64 => {
                let n = val
                    .as_f64()
                    .ok_or_else(|| format!("Argument {} must be a number", i))?;
                Val::F64(n.to_bits())
            }
            _ => return Err(format!("Unsupported parameter type at position {}", i)),
        };
        vals.push(wasm_val);
    }

    Ok(vals)
}

#[cfg(feature = "wasm")]
fn format_val(val: &wasmtime::Val) -> String {
    match val {
        wasmtime::Val::I32(n) => n.to_string(),
        wasmtime::Val::I64(n) => n.to_string(),
        wasmtime::Val::F32(bits) => f32::from_bits(*bits).to_string(),
        wasmtime::Val::F64(bits) => f64::from_bits(*bits).to_string(),
        _ => format!("{:?}", val),
    }
}

#[cfg(not(feature = "wasm"))]
fn cmd_run(_args: RunArgs, _verbose: bool, _quiet: bool) -> Result<(), String> {
    Err("WASM feature not enabled. Rebuild with --features wasm".to_string())
}

// =============================================================================
// Compile Command
// =============================================================================

#[cfg(feature = "wasm")]
fn cmd_compile(args: CompileArgs, verbose: bool, quiet: bool) -> Result<(), String> {
    use metadol::parse_dol_file;
    use metadol::wasm::WasmCompiler;

    if !args.file.exists() {
        return Err(format!("File not found: {}", args.file.display()));
    }

    if verbose {
        eprintln!("{} {}", "Compiling".cyan(), args.file.display());
    }

    // Read DOL source
    let source =
        std::fs::read_to_string(&args.file).map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse
    let file = parse_dol_file(&source).map_err(|e| format!("Parse error: {:?}", e))?;

    if verbose {
        eprintln!("  Parsed {} declarations", file.declarations.len());
    }

    // Compile to WASM
    let mut compiler = WasmCompiler::new();
    if args.optimize {
        compiler = compiler.with_optimization(true);
    }

    let wasm_bytes = compiler
        .compile_file(&file)
        .map_err(|e| format!("Compile error: {}", e.message))?;

    if verbose {
        eprintln!("  Generated {} bytes of WASM", wasm_bytes.len());
    }

    // Determine output path
    let output_path = args
        .output
        .unwrap_or_else(|| args.file.with_extension("wasm"));

    // Write output
    std::fs::write(&output_path, &wasm_bytes)
        .map_err(|e| format!("Failed to write output: {}", e))?;

    if !quiet {
        eprintln!(
            "{} {} ({} bytes)",
            "Wrote".green(),
            output_path.display(),
            wasm_bytes.len()
        );
    }

    Ok(())
}

#[cfg(not(feature = "wasm"))]
fn cmd_compile(_args: CompileArgs, _verbose: bool, _quiet: bool) -> Result<(), String> {
    Err("WASM feature not enabled. Rebuild with --features wasm".to_string())
}

// =============================================================================
// Check Command
// =============================================================================

fn cmd_check(args: CheckArgs, verbose: bool, quiet: bool) -> Result<(), String> {
    let files = collect_dol_files(&args.paths, args.recursive);

    if files.is_empty() {
        if !quiet {
            eprintln!("{}: No .dol files found", "warning".yellow());
        }
        return Ok(());
    }

    if verbose {
        eprintln!("Checking {} file(s)...", files.len());
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut errors: Vec<(PathBuf, String)> = Vec::new();

    for path in &files {
        match check_file(path) {
            Ok(()) => {
                passed += 1;
                if verbose {
                    eprintln!("{} {}", "  OK".green(), path.display());
                }
            }
            Err(e) => {
                failed += 1;
                if !args.json {
                    eprintln!("{} {}: {}", "FAIL".red(), path.display(), e);
                }
                errors.push((path.clone(), e));
            }
        }
    }

    if args.json {
        let result = serde_json::json!({
            "passed": passed,
            "failed": failed,
            "errors": errors.iter().map(|(p, e)| {
                serde_json::json!({
                    "file": p.display().to_string(),
                    "error": e
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else if !quiet {
        eprintln!(
            "\n{} passed, {} failed",
            passed.to_string().green(),
            if failed > 0 {
                failed.to_string().red()
            } else {
                failed.to_string().normal()
            }
        );
    }

    if failed > 0 && args.strict {
        Err(format!("{} file(s) failed type check", failed))
    } else if failed > 0 {
        // Non-strict mode: report but don't fail
        Ok(())
    } else {
        Ok(())
    }
}

fn check_file(path: &PathBuf) -> Result<(), String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse the file
    metadol::parse_file(&source).map_err(|e| format!("Parse error: {}", e))?;

    // TODO: Add type checking when typechecker is integrated

    Ok(())
}

// =============================================================================
// Utilities
// =============================================================================

fn collect_dol_files(paths: &[PathBuf], recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "dol") {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            if recursive {
                collect_dol_files_recursive(path, &mut files);
            } else {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() && p.extension().is_some_and(|ext| ext == "dol") {
                            files.push(p);
                        }
                    }
                }
            }
        }
    }

    files.sort();
    files
}

fn collect_dol_files_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_dol_files_recursive(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "dol") {
                files.push(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_empty() {
        let files = collect_dol_files(&[], false);
        assert!(files.is_empty());
    }
}
