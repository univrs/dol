//! dol-parse - Parse and validate Metal DOL files
//!
//! A command-line tool for parsing DOL files and outputting AST
//! in various formats. Designed for both interactive use and CI integration.
//!
//! # Usage
//!
//! ```bash
//! # Parse a single file
//! dol-parse examples/genes/container.exists.dol
//!
//! # Parse with JSON output
//! dol-parse --format json examples/genes/container.exists.dol
//!
//! # Parse all files in directory
//! dol-parse --recursive examples/
//!
//! # Validate only (no output)
//! dol-parse --validate examples/
//!
//! # CI mode (exit code only)
//! dol-parse --ci --recursive .
//! ```

use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::path::PathBuf;
use std::process::ExitCode;

use metadol::{parse_file, validate, Declaration, ValidationResult};

/// Parse and validate Metal DOL files
#[derive(Parser, Debug)]
#[command(name = "dol-parse")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files or directories to parse
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    format: OutputFormat,

    /// Recursively process directories
    #[arg(short, long)]
    recursive: bool,

    /// Validate only, don't output AST
    #[arg(long)]
    validate: bool,

    /// CI mode: minimal output, exit code indicates success/failure
    #[arg(long)]
    ci: bool,

    /// Show warnings in addition to errors
    #[arg(short, long)]
    warnings: bool,

    /// Quiet mode: only show errors
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Human-readable formatted output
    Pretty,
    /// JSON output for tooling integration
    Json,
    /// Compact single-line output
    Compact,
    /// Debug format (Rust debug output)
    Debug,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let mut total_files = 0;
    let mut successful = 0;
    let mut failed = 0;
    let mut warnings_count = 0;

    // Collect all DOL files
    let files = collect_dol_files(&args.paths, args.recursive);

    if files.is_empty() {
        if !args.quiet {
            eprintln!("{}: No .dol files found", "warning".yellow());
        }
        return ExitCode::SUCCESS;
    }

    let mut results: Vec<ParseResult> = Vec::new();

    for path in &files {
        total_files += 1;

        match process_file(path) {
            Ok((decl, validation)) => {
                successful += 1;
                if validation.has_warnings() {
                    warnings_count += validation.warnings.len();
                }
                results.push(ParseResult {
                    path: path.clone(),
                    success: true,
                    declaration: Some(decl),
                    validation: Some(validation),
                    error: None,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(ParseResult {
                    path: path.clone(),
                    success: false,
                    declaration: None,
                    validation: None,
                    error: Some(e),
                });
            }
        }
    }

    // Output results based on format
    if !args.ci || failed > 0 {
        output_results(&results, &args);
    }

    // Print summary unless quiet
    if !args.quiet && !args.ci {
        println!();
        println!("{}", "Summary".bold());
        println!("  Total:    {}", total_files);
        println!("  Success:  {}", successful.to_string().green());
        if failed > 0 {
            println!("  Failed:   {}", failed.to_string().red());
        }
        if args.warnings && warnings_count > 0 {
            println!("  Warnings: {}", warnings_count.to_string().yellow());
        }
    }

    // CI mode summary
    if args.ci && failed > 0 {
        eprintln!(
            "{}: {}/{} files failed to parse",
            "error".red(),
            failed,
            total_files
        );
    }

    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

struct ParseResult {
    path: PathBuf,
    success: bool,
    declaration: Option<Declaration>,
    validation: Option<ValidationResult>,
    error: Option<String>,
}

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
                // Only immediate children
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

fn process_file(path: &PathBuf) -> Result<(Declaration, ValidationResult), String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let decl = parse_file(&source).map_err(|e| format!("Parse error: {}", e))?;

    let validation = validate(&decl);

    if !validation.is_valid() {
        let errors: Vec<String> = validation.errors.iter().map(|e| e.to_string()).collect();
        return Err(format!("Validation errors:\n  {}", errors.join("\n  ")));
    }

    Ok((decl, validation))
}

fn output_results(results: &[ParseResult], args: &Args) {
    match args.format {
        OutputFormat::Json => output_json(results, args),
        OutputFormat::Pretty => output_pretty(results, args),
        OutputFormat::Compact => output_compact(results, args),
        OutputFormat::Debug => output_debug(results, args),
    }
}

fn output_json(results: &[ParseResult], _args: &Args) {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        files: Vec<JsonFileResult>,
        summary: JsonSummary,
    }

    #[derive(serde::Serialize)]
    struct JsonFileResult {
        path: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        declaration_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct JsonSummary {
        total: usize,
        successful: usize,
        failed: usize,
    }

    let files: Vec<JsonFileResult> = results
        .iter()
        .map(|r| JsonFileResult {
            path: r.path.display().to_string(),
            success: r.success,
            declaration_type: r.declaration.as_ref().map(|d| match d {
                Declaration::Gene(_) => "gene".to_string(),
                Declaration::Trait(_) => "trait".to_string(),
                Declaration::Constraint(_) => "constraint".to_string(),
                Declaration::System(_) => "system".to_string(),
                Declaration::Evolution(_) => "evolution".to_string(),
                Declaration::Function(_) => "function".to_string(),
                Declaration::Const(_) => "const".to_string(),
            }),
            name: r.declaration.as_ref().map(|d| d.name().to_string()),
            error: r.error.clone(),
        })
        .collect();

    let successful = results.iter().filter(|r| r.success).count();

    let output = JsonOutput {
        summary: JsonSummary {
            total: results.len(),
            successful,
            failed: results.len() - successful,
        },
        files,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn output_pretty(results: &[ParseResult], args: &Args) {
    for result in results {
        if result.success {
            if !args.validate {
                let decl = result.declaration.as_ref().unwrap();
                println!(
                    "{} {} {}",
                    "✓".green(),
                    result.path.display(),
                    format!("({})", decl.name()).dimmed()
                );

                if !args.quiet {
                    print_declaration_summary(decl);
                }
            }

            // Show warnings if enabled
            if args.warnings {
                if let Some(validation) = &result.validation {
                    for warning in &validation.warnings {
                        println!("  {}: {}", "warning".yellow(), warning);
                    }
                }
            }
        } else {
            println!("{} {}", "✗".red(), result.path.display());
            if let Some(error) = &result.error {
                for line in error.lines() {
                    println!("  {}", line.red());
                }
            }
        }
    }
}

fn output_compact(results: &[ParseResult], _args: &Args) {
    for result in results {
        if result.success {
            let decl = result.declaration.as_ref().unwrap();
            let decl_type = match decl {
                Declaration::Gene(_) => "gene",
                Declaration::Trait(_) => "trait",
                Declaration::Constraint(_) => "constraint",
                Declaration::System(_) => "system",
                Declaration::Evolution(_) => "evolution",
                Declaration::Function(_) => "function",
                Declaration::Const(_) => "const",
            };
            println!(
                "OK\t{}\t{}\t{}",
                result.path.display(),
                decl_type,
                decl.name()
            );
        } else {
            println!(
                "ERR\t{}\t{}",
                result.path.display(),
                result.error.as_deref().unwrap_or("unknown")
            );
        }
    }
}

fn output_debug(results: &[ParseResult], _args: &Args) {
    for result in results {
        println!("=== {} ===", result.path.display());
        if let Some(decl) = &result.declaration {
            println!("{:#?}", decl);
        }
        if let Some(error) = &result.error {
            println!("Error: {}", error);
        }
        println!();
    }
}

fn print_declaration_summary(decl: &Declaration) {
    match decl {
        Declaration::Gene(g) => {
            println!(
                "    {} gene with {} statements",
                g.name.dimmed(),
                g.statements.len()
            );
        }
        Declaration::Trait(t) => {
            let uses_count = t
                .statements
                .iter()
                .filter(|s| matches!(s, metadol::Statement::Uses { .. }))
                .count();
            println!(
                "    {} trait using {} dependencies, {} behaviors",
                t.name.dimmed(),
                uses_count,
                t.statements.len() - uses_count
            );
        }
        Declaration::Constraint(c) => {
            println!(
                "    {} constraint with {} rules",
                c.name.dimmed(),
                c.statements.len()
            );
        }
        Declaration::System(s) => {
            println!(
                "    {} system @ {} with {} requirements",
                s.name.dimmed(),
                s.version,
                s.requirements.len()
            );
        }
        Declaration::Evolution(e) => {
            println!(
                "    {} evolution {} > {} ({} additions)",
                e.name.dimmed(),
                e.version,
                e.parent_version,
                e.additions.len()
            );
        }
        Declaration::Function(f) => {
            println!(
                "    {} function with {} params",
                f.name.dimmed(),
                f.params.len()
            );
        }
        Declaration::Const(v) => {
            println!("    {} constant", v.name.dimmed());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_dol_files_empty() {
        let files = collect_dol_files(&[], false);
        assert!(files.is_empty());
    }

    #[test]
    fn test_output_format_variants() {
        assert_eq!(OutputFormat::Pretty, OutputFormat::Pretty);
        assert_ne!(OutputFormat::Pretty, OutputFormat::Json);
    }
}
