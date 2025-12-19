//! dol-check - Validate DOL files and check coverage
//!
//! A CI-friendly validation tool that ensures DOL files meet quality
//! standards and coverage requirements.
//!
//! # Usage
//!
//! ```bash
//! # Check all DOL files in current directory
//! dol-check .
//!
//! # Require exegesis on all declarations
//! dol-check --require-exegesis examples/
//!
//! # Check coverage against source files
//! dol-check --coverage-source src/ examples/
//!
//! # CI mode with strict checks
//! dol-check --strict --ci examples/
//! ```

use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::path::PathBuf;
use std::process::ExitCode;

use metadol::{parse_file, validate, Declaration};

/// Validate DOL files and check coverage
#[derive(Parser, Debug)]
#[command(name = "dol-check")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directories or files to check
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Require exegesis blocks to be present and non-empty
    #[arg(long)]
    require_exegesis: bool,

    /// Minimum exegesis length (characters)
    #[arg(long, default_value = "20")]
    min_exegesis_length: usize,

    /// Source directory to check coverage against
    #[arg(long)]
    coverage_source: Option<PathBuf>,

    /// Strict mode: treat warnings as errors
    #[arg(long)]
    strict: bool,

    /// CI mode: exit code only, minimal output
    #[arg(long)]
    ci: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    format: OutputFormat,

    /// Quiet mode
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Pretty,
    Json,
    Compact,
}

#[derive(Debug, Default)]
struct CheckResults {
    files_checked: usize,
    files_passed: usize,
    files_failed: usize,
    total_errors: usize,
    total_warnings: usize,
    declarations: Vec<DeclarationInfo>,
    errors: Vec<CheckError>,
    warnings: Vec<CheckWarning>,
    coverage: Option<CoverageReport>,
}

#[derive(Debug)]
struct DeclarationInfo {
    path: PathBuf,
    name: String,
    decl_type: String,
    exegesis_length: usize,
}

#[derive(Debug)]
struct CheckError {
    path: PathBuf,
    message: String,
    line: Option<usize>,
}

#[derive(Debug)]
struct CheckWarning {
    path: PathBuf,
    message: String,
    line: Option<usize>,
}

#[derive(Debug)]
struct CoverageReport {
    source_items: usize,
    covered_items: usize,
    coverage_percent: f64,
    uncovered: Vec<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let results = run_checks(&args);

    // Output results
    match args.format {
        OutputFormat::Pretty => output_pretty(&results, &args),
        OutputFormat::Json => output_json(&results),
        OutputFormat::Compact => output_compact(&results),
    }

    // Determine exit code
    let has_errors = results.files_failed > 0 || results.total_errors > 0;
    let has_warnings = results.total_warnings > 0;

    if has_errors || (args.strict && has_warnings) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run_checks(args: &Args) -> CheckResults {
    let mut results = CheckResults::default();

    // Collect DOL files
    let files = collect_dol_files(&args.paths);

    for path in files {
        results.files_checked += 1;

        match check_file(&path, args) {
            Ok((decl_info, warnings)) => {
                results.files_passed += 1;
                results.declarations.push(decl_info);
                for warning in warnings {
                    results.warnings.push(warning);
                    results.total_warnings += 1;
                }
            }
            Err(errors) => {
                results.files_failed += 1;
                for error in errors {
                    results.errors.push(error);
                    results.total_errors += 1;
                }
            }
        }
    }

    // Run coverage check if requested
    if let Some(source_dir) = &args.coverage_source {
        results.coverage = Some(check_coverage(source_dir, &results.declarations));
    }

    results
}

fn collect_dol_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "dol") {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            collect_dol_files_recursive(path, &mut files);
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

fn check_file(
    path: &PathBuf,
    args: &Args,
) -> Result<(DeclarationInfo, Vec<CheckWarning>), Vec<CheckError>> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Read file
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            errors.push(CheckError {
                path: path.clone(),
                message: format!("Failed to read file: {}", e),
                line: None,
            });
            return Err(errors);
        }
    };

    // Parse file
    let decl = match parse_file(&source) {
        Ok(d) => d,
        Err(e) => {
            errors.push(CheckError {
                path: path.clone(),
                message: format!("Parse error: {}", e),
                line: Some(e.span().line),
            });
            return Err(errors);
        }
    };

    // Validate
    let validation = validate(&decl);

    // Check validation errors
    for error in &validation.errors {
        errors.push(CheckError {
            path: path.clone(),
            message: error.to_string(),
            line: None,
        });
    }

    // Check validation warnings
    for warning in &validation.warnings {
        warnings.push(CheckWarning {
            path: path.clone(),
            message: warning.to_string(),
            line: None,
        });
    }

    // Check exegesis requirements
    let exegesis = decl.exegesis();
    let exegesis_length = exegesis.trim().len();

    if args.require_exegesis {
        if exegesis.trim().is_empty() {
            errors.push(CheckError {
                path: path.clone(),
                message: "Missing or empty exegesis block".to_string(),
                line: None,
            });
        } else if exegesis_length < args.min_exegesis_length {
            warnings.push(CheckWarning {
                path: path.clone(),
                message: format!(
                    "Exegesis too short ({} chars, minimum {})",
                    exegesis_length, args.min_exegesis_length
                ),
                line: None,
            });
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let decl_type = match &decl {
        Declaration::Gene(_) => "gene",
        Declaration::Trait(_) => "trait",
        Declaration::Constraint(_) => "constraint",
        Declaration::System(_) => "system",
        Declaration::Evolution(_) => "evolution",
    };

    Ok((
        DeclarationInfo {
            path: path.clone(),
            name: decl.name().to_string(),
            decl_type: decl_type.to_string(),
            exegesis_length,
        },
        warnings,
    ))
}

fn check_coverage(source_dir: &PathBuf, declarations: &[DeclarationInfo]) -> CoverageReport {
    // Collect source items (simplified: look for struct/enum/fn definitions)
    let source_items = collect_source_items(source_dir);

    // Map declarations to coverage
    let declared_names: Vec<&str> = declarations.iter().map(|d| d.name.as_str()).collect();

    let mut covered = 0;
    let mut uncovered = Vec::new();

    for item in &source_items {
        // Simple heuristic: check if any declaration name relates to this item
        let is_covered = declared_names.iter().any(|name| {
            let parts: Vec<&str> = name.split('.').collect();
            parts
                .iter()
                .any(|part| item.to_lowercase().contains(&part.to_lowercase()))
        });

        if is_covered {
            covered += 1;
        } else {
            uncovered.push(item.clone());
        }
    }

    let coverage_percent = if source_items.is_empty() {
        100.0
    } else {
        (covered as f64 / source_items.len() as f64) * 100.0
    };

    CoverageReport {
        source_items: source_items.len(),
        covered_items: covered,
        coverage_percent,
        uncovered,
    }
}

fn collect_source_items(dir: &PathBuf) -> Vec<String> {
    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                items.extend(collect_source_items(&path));
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(source) = std::fs::read_to_string(&path) {
                    // Extract struct/enum/fn names (simplified)
                    for line in source.lines() {
                        let line = line.trim();
                        if line.starts_with("pub struct ")
                            || line.starts_with("pub enum ")
                            || line.starts_with("pub fn ")
                            || line.starts_with("struct ")
                            || line.starts_with("enum ")
                            || line.starts_with("fn ")
                        {
                            if let Some(name) = extract_item_name(line) {
                                items.push(name);
                            }
                        }
                    }
                }
            }
        }
    }

    items
}

fn extract_item_name(line: &str) -> Option<String> {
    let line = line
        .trim_start_matches("pub ")
        .trim_start_matches("struct ")
        .trim_start_matches("enum ")
        .trim_start_matches("fn ");

    let name: String = line
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn output_pretty(results: &CheckResults, args: &Args) {
    if args.quiet && results.files_failed == 0 && results.total_errors == 0 {
        return;
    }

    // Print errors
    for error in &results.errors {
        print!("{} ", "✗".red());
        print!("{}", error.path.display());
        if let Some(line) = error.line {
            print!(":{}", line);
        }
        println!();
        println!("  {}", error.message.red());
    }

    // Print warnings (unless quiet)
    if !args.quiet {
        for warning in &results.warnings {
            print!("{} ", "⚠".yellow());
            print!("{}", warning.path.display());
            if let Some(line) = warning.line {
                print!(":{}", line);
            }
            println!();
            println!("  {}", warning.message.yellow());
        }
    }

    // Print coverage if available
    if let Some(coverage) = &results.coverage {
        println!();
        println!("{}", "Coverage Report".bold());
        println!(
            "  Coverage: {:.1}% ({}/{})",
            coverage.coverage_percent, coverage.covered_items, coverage.source_items
        );

        if !coverage.uncovered.is_empty() && !args.quiet {
            println!("  Uncovered items:");
            for item in coverage.uncovered.iter().take(10) {
                println!("    - {}", item);
            }
            if coverage.uncovered.len() > 10 {
                println!("    ... and {} more", coverage.uncovered.len() - 10);
            }
        }
    }

    // Print summary
    if !args.ci {
        println!();
        println!("{}", "Summary".bold());
        println!("  Files:    {}", results.files_checked);
        println!("  Passed:   {}", results.files_passed.to_string().green());
        if results.files_failed > 0 {
            println!("  Failed:   {}", results.files_failed.to_string().red());
        }
        if results.total_warnings > 0 {
            println!(
                "  Warnings: {}",
                results.total_warnings.to_string().yellow()
            );
        }
    }
}

fn output_json(results: &CheckResults) {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        files_checked: usize,
        files_passed: usize,
        files_failed: usize,
        total_errors: usize,
        total_warnings: usize,
        errors: Vec<JsonError>,
        warnings: Vec<JsonWarning>,
        declarations: Vec<JsonDeclaration>,
        coverage: Option<JsonCoverage>,
    }

    #[derive(serde::Serialize)]
    struct JsonError {
        path: String,
        message: String,
        line: Option<usize>,
    }

    #[derive(serde::Serialize)]
    struct JsonWarning {
        path: String,
        message: String,
        line: Option<usize>,
    }

    #[derive(serde::Serialize)]
    struct JsonDeclaration {
        path: String,
        name: String,
        #[serde(rename = "type")]
        decl_type: String,
        exegesis_length: usize,
    }

    #[derive(serde::Serialize)]
    struct JsonCoverage {
        source_items: usize,
        covered_items: usize,
        coverage_percent: f64,
        uncovered: Vec<String>,
    }

    let output = JsonOutput {
        files_checked: results.files_checked,
        files_passed: results.files_passed,
        files_failed: results.files_failed,
        total_errors: results.total_errors,
        total_warnings: results.total_warnings,
        errors: results
            .errors
            .iter()
            .map(|e| JsonError {
                path: e.path.display().to_string(),
                message: e.message.clone(),
                line: e.line,
            })
            .collect(),
        warnings: results
            .warnings
            .iter()
            .map(|w| JsonWarning {
                path: w.path.display().to_string(),
                message: w.message.clone(),
                line: w.line,
            })
            .collect(),
        declarations: results
            .declarations
            .iter()
            .map(|d| JsonDeclaration {
                path: d.path.display().to_string(),
                name: d.name.clone(),
                decl_type: d.decl_type.clone(),
                exegesis_length: d.exegesis_length,
            })
            .collect(),
        coverage: results.coverage.as_ref().map(|c| JsonCoverage {
            source_items: c.source_items,
            covered_items: c.covered_items,
            coverage_percent: c.coverage_percent,
            uncovered: c.uncovered.clone(),
        }),
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn output_compact(results: &CheckResults) {
    println!(
        "CHECKED:{} PASSED:{} FAILED:{} ERRORS:{} WARNINGS:{}",
        results.files_checked,
        results.files_passed,
        results.files_failed,
        results.total_errors,
        results.total_warnings
    );

    if let Some(coverage) = &results.coverage {
        println!("COVERAGE:{:.1}%", coverage.coverage_percent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_item_name() {
        assert_eq!(extract_item_name("struct Foo {"), Some("Foo".to_string()));
        assert_eq!(extract_item_name("pub fn bar()"), Some("bar".to_string()));
        assert_eq!(extract_item_name("enum Baz"), Some("Baz".to_string()));
    }

    #[test]
    fn test_collect_empty() {
        let files = collect_dol_files(&[]);
        assert!(files.is_empty());
    }
}
