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

use metadol::transform::{TreeShaking, TreeShakingStats};
use metadol::{parse_dol_file, validate, Declaration, ValidationResult};

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

    /// Enable tree shaking to eliminate unused declarations
    #[arg(long)]
    tree_shake: bool,

    /// Additional root declarations for tree shaking (comma-separated)
    /// Use with --tree-shake to preserve specific declarations
    #[arg(long, value_delimiter = ',')]
    roots: Vec<String>,

    /// Analyze tree shaking without modifying output (dry-run)
    #[arg(long)]
    shake_analyze: bool,
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
            Ok((declarations, validations)) => {
                successful += 1;
                warnings_count += validations.iter().map(|v| v.warnings.len()).sum::<usize>();
                let eliminated = vec![false; declarations.len()];
                results.push(ParseResult {
                    path: path.clone(),
                    success: true,
                    declarations,
                    validations,
                    error: None,
                    eliminated,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(ParseResult {
                    path: path.clone(),
                    success: false,
                    declarations: Vec::new(),
                    validations: Vec::new(),
                    error: Some(e),
                    eliminated: Vec::new(),
                });
            }
        }
    }

    // Apply tree shaking if enabled
    let shake_stats = if args.tree_shake || args.shake_analyze {
        Some(apply_tree_shaking(&mut results, &args))
    } else {
        None
    };

    // Output results based on format
    if !args.ci || failed > 0 {
        output_results(&results, &args, shake_stats.as_ref());
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

        // Tree shaking summary
        if let Some(ref stats) = shake_stats {
            println!();
            println!("{}", "Tree Shaking".bold());
            println!("  Roots:      {}", stats.root_count);
            println!(
                "  Retained:   {}",
                stats.retained_declarations.to_string().green()
            );
            println!(
                "  Eliminated: {}",
                stats.eliminated_declarations.to_string().yellow()
            );
            if args.shake_analyze && !stats.eliminated_names.is_empty() {
                println!("  Would eliminate:");
                for name in &stats.eliminated_names {
                    println!("    - {}", name.dimmed());
                }
            }
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

/// Apply tree shaking to parsed declarations
fn apply_tree_shaking(results: &mut [ParseResult], args: &Args) -> TreeShakingStats {
    // Collect all successful declarations
    let declarations: Vec<Declaration> = results
        .iter()
        .flat_map(|r| r.declarations.iter().cloned())
        .collect();

    // Create tree shaker with extra roots
    let mut shaker = TreeShaking::new();
    for root in &args.roots {
        shaker.add_root(root);
    }

    // Analyze or shake
    if args.shake_analyze {
        // Dry-run: just analyze without modifying
        shaker.analyze(&declarations)
    } else {
        // Actually perform tree shaking
        let stats = shaker.analyze(&declarations);

        // Mark eliminated declarations in results
        let reachable = shaker.reachable_names();
        for result in results.iter_mut() {
            for (decl, eliminated) in result.declarations.iter().zip(result.eliminated.iter_mut()) {
                if !reachable.contains(decl.name()) {
                    *eliminated = true;
                }
            }
        }

        stats
    }
}

struct ParseResult {
    path: PathBuf,
    success: bool,
    /// Every declaration in the file, in source order
    declarations: Vec<Declaration>,
    /// Validation result for each declaration (same order)
    validations: Vec<ValidationResult>,
    error: Option<String>,
    /// Whether each declaration was eliminated by tree shaking (same order)
    eliminated: Vec<bool>,
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

/// Parses the whole file (every declaration; trailing content is an error)
/// and validates each declaration.
fn process_file(path: &PathBuf) -> Result<(Vec<Declaration>, Vec<ValidationResult>), String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let file = parse_dol_file(&source).map_err(|e| format!("Parse error: {}", e))?;

    let validations: Vec<ValidationResult> = file.declarations.iter().map(validate).collect();

    let errors: Vec<String> = file
        .declarations
        .iter()
        .zip(&validations)
        .flat_map(|(decl, v)| {
            v.errors
                .iter()
                .map(move |e| format!("{}: {}", decl.name(), e))
        })
        .collect();
    if !errors.is_empty() {
        return Err(format!("Validation errors:\n  {}", errors.join("\n  ")));
    }

    Ok((file.declarations, validations))
}

/// Short type label for a declaration.
fn decl_type(decl: &Declaration) -> &'static str {
    match decl {
        Declaration::Gene(_) => "gene",
        Declaration::Trait(_) => "trait",
        Declaration::Constraint(_) => "constraint",
        Declaration::System(_) => "system",
        Declaration::Evolution(_) => "evolution",
        Declaration::Function(_) => "function",
        Declaration::Const(_) => "const",
        Declaration::SexVar(_) => "sex_var",
    }
}

fn output_results(results: &[ParseResult], args: &Args, shake_stats: Option<&TreeShakingStats>) {
    match args.format {
        OutputFormat::Json => output_json(results, args, shake_stats),
        OutputFormat::Pretty => output_pretty(results, args),
        OutputFormat::Compact => output_compact(results, args),
        OutputFormat::Debug => output_debug(results, args),
    }
}

fn output_json(results: &[ParseResult], _args: &Args, shake_stats: Option<&TreeShakingStats>) {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        files: Vec<JsonFileResult>,
        summary: JsonSummary,
        #[serde(skip_serializing_if = "Option::is_none")]
        tree_shaking: Option<JsonTreeShaking>,
    }

    #[derive(serde::Serialize)]
    struct JsonFileResult {
        path: String,
        success: bool,
        declarations: Vec<JsonDeclaration>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct JsonDeclaration {
        #[serde(rename = "type")]
        decl_type: String,
        name: String,
        /// Law names (traits only)
        #[serde(skip_serializing_if = "Vec::is_empty")]
        laws: Vec<String>,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        eliminated: bool,
    }

    #[derive(serde::Serialize)]
    struct JsonSummary {
        total: usize,
        successful: usize,
        failed: usize,
    }

    #[derive(serde::Serialize)]
    struct JsonTreeShaking {
        roots: usize,
        retained: usize,
        eliminated: usize,
        eliminated_names: Vec<String>,
    }

    let files: Vec<JsonFileResult> = results
        .iter()
        .map(|r| JsonFileResult {
            path: r.path.display().to_string(),
            success: r.success,
            declarations: r
                .declarations
                .iter()
                .zip(&r.eliminated)
                .map(|(d, &eliminated)| JsonDeclaration {
                    decl_type: decl_type(d).to_string(),
                    name: d.name().to_string(),
                    laws: match d {
                        Declaration::Trait(t) => t.laws.iter().map(|l| l.name.clone()).collect(),
                        _ => Vec::new(),
                    },
                    eliminated,
                })
                .collect(),
            error: r.error.clone(),
        })
        .collect();

    let successful = results.iter().filter(|r| r.success).count();

    let tree_shaking = shake_stats.map(|stats| JsonTreeShaking {
        roots: stats.root_count,
        retained: stats.retained_declarations,
        eliminated: stats.eliminated_declarations,
        eliminated_names: stats.eliminated_names.clone(),
    });

    let output = JsonOutput {
        summary: JsonSummary {
            total: results.len(),
            successful,
            failed: results.len() - successful,
        },
        files,
        tree_shaking,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn output_pretty(results: &[ParseResult], args: &Args) {
    for result in results {
        if result.success {
            if !args.validate {
                let all_eliminated =
                    !result.eliminated.is_empty() && result.eliminated.iter().all(|&e| e);
                let status = if all_eliminated {
                    "~".yellow()
                } else {
                    "✓".green()
                };
                let suffix = format!("({} declarations)", result.declarations.len()).dimmed();
                println!("{} {} {}", status, result.path.display(), suffix);

                if !args.quiet {
                    for (decl, &eliminated) in result.declarations.iter().zip(&result.eliminated) {
                        if eliminated {
                            println!("    {} {}", decl.name().dimmed(), "[eliminated]".yellow());
                        } else {
                            print_declaration_summary(decl);
                        }
                    }
                }
            }

            // Show warnings if enabled
            if args.warnings {
                for validation in &result.validations {
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
            for (decl, &eliminated) in result.declarations.iter().zip(&result.eliminated) {
                let status = if eliminated { "ELIM" } else { "OK" };
                println!(
                    "{}\t{}\t{}\t{}",
                    status,
                    result.path.display(),
                    decl_type(decl),
                    decl.name()
                );
            }
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
        for (decl, &eliminated) in result.declarations.iter().zip(&result.eliminated) {
            if eliminated {
                println!("[ELIMINATED]");
            }
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
                "    {} trait using {} dependencies, {} behaviors, {} laws",
                t.name.dimmed(),
                uses_count,
                t.statements.len() - uses_count,
                t.laws.len()
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
        Declaration::Const(c) => {
            println!("    {} constant", c.name.dimmed());
        }
        Declaration::SexVar(v) => {
            println!("    {} sex var (mutable global)", v.name.dimmed());
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
