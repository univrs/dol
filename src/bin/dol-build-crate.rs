//! dol-build-crate - Generate a complete Rust crate from DOL files
//!
//! This tool generates a modular Rust crate structure from multiple DOL files,
//! with one .rs file per DOL module.
//!
//! # Usage
//!
//! ```bash
//! # Generate a crate from all DOL files in a directory
//! dol-build-crate dol/*.dol -o stage2/
//!
//! # Generate with custom crate name
//! dol-build-crate dol/*.dol -o stage2/ --name my_crate
//!
//! # Generate with custom version
//! dol-build-crate dol/*.dol -o stage2/ --crate-version 1.0.0
//! ```

use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use std::process::ExitCode;

use metadol::ast::DolFile;
use metadol::codegen::{CrateCodegen, CrateConfig};
use metadol::parse_dol_file;

/// Generate a complete Rust crate from Metal DOL files
#[derive(Parser, Debug)]
#[command(name = "dol-build-crate")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// DOL files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Output directory for the generated crate
    #[arg(short, long, default_value = "stage2")]
    output: PathBuf,

    /// Name of the generated crate
    #[arg(long, default_value = "dol_generated")]
    name: String,

    /// Version of the generated crate
    #[arg(long = "crate-version", default_value = "0.1.0")]
    crate_version: String,

    /// Quiet mode: only show errors
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    // Collect all DOL files
    let files = collect_dol_files(&args.files);

    if files.is_empty() {
        eprintln!("{}: No .dol files found", "error".red());
        return ExitCode::FAILURE;
    }

    if !args.quiet {
        eprintln!(
            "{} {} DOL files to {}",
            "Processing".cyan(),
            files.len(),
            args.output.display()
        );
    }

    // Parse all files
    let mut parsed_files = Vec::new();
    let mut failed = 0;

    for path in &files {
        match parse_dol_file_from_path(path) {
            Ok(dol_file) => {
                if !args.quiet {
                    eprintln!("  {} {}", "Parsed".green(), path.display());
                }
                parsed_files.push((path.display().to_string(), dol_file));
            }
            Err(e) => {
                failed += 1;
                eprintln!("  {} {}: {}", "Error".red(), path.display(), e);
            }
        }
    }

    if failed > 0 {
        eprintln!("\n{}: {} file(s) failed to parse", "error".red(), failed);
        return ExitCode::FAILURE;
    }

    // Generate the crate
    let config = CrateConfig {
        crate_name: args.name.clone(),
        crate_version: args.crate_version.clone(),
        output_dir: args.output.display().to_string(),
    };

    let generator = CrateCodegen::with_config(config);

    match generator.generate(&parsed_files) {
        Ok(()) => {
            if !args.quiet {
                eprintln!(
                    "\n{} crate at {}",
                    "Generated".green().bold(),
                    args.output.display()
                );
                eprintln!("  {} src/{}.rs files", files.len(), "");
                eprintln!("  {} lib.rs with mod declarations", "1");
                eprintln!("  {} prelude.rs with re-exports", "1");
                eprintln!("  {} Cargo.toml", "1");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}: {}", "error".red(), e);
            ExitCode::FAILURE
        }
    }
}

fn collect_dol_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "dol") {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            // Recursively collect .dol files from directory
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

    files.sort();
    files
}

fn parse_dol_file_from_path(path: &PathBuf) -> Result<DolFile, String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    parse_dol_file(&source).map_err(|e| format!("Parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_dol_files_empty() {
        let files = collect_dol_files(&[]);
        assert!(files.is_empty());
    }
}
