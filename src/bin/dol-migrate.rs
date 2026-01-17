//! DOL Syntax Migration Tool
//!
//! Migrates DOL files between versions with different migration paths.
//!
//! # Usage
//!
//! ```bash
//! # Migrate from v0.7.x to v0.8.0
//! dol-migrate 0.7-to-0.8 src/container.dol
//!
//! # Migrate a directory
//! dol-migrate 0.7-to-0.8 src/
//!
//! # Dry run (preview changes without applying)
//! dol-migrate 0.7-to-0.8 --dry-run src/
//!
//! # Show diff of changes
//! dol-migrate 0.7-to-0.8 --diff src/
//!
//! # Modernize return statements
//! dol-migrate 0.7-to-0.8 --modernize src/
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// DOL migration CLI
#[derive(Parser)]
#[command(name = "dol-migrate")]
#[command(about = "Migrate DOL files between versions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate from v0.7.x to v0.8.0
    #[command(name = "0.7-to-0.8")]
    V07ToV08 {
        /// Path to file or directory to migrate
        path: PathBuf,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,

        /// Show diff of changes
        #[arg(long)]
        diff: bool,

        /// Modernize return statements (remove 'return' before final expression)
        #[arg(long)]
        modernize: bool,
    },

    /// Migrate from v0.2 to v0.3 (legacy)
    #[command(name = "0.2-to-0.3")]
    V02ToV03 {
        /// Path to file or directory to migrate
        path: PathBuf,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,

        /// Show diff of changes
        #[arg(long)]
        diff: bool,
    },
}

/// Migration rules from v0.2 to v0.3
const MIGRATIONS_V02_TO_V03: &[(&str, &str)] = &[
    // Bindings: let → val, let mut → var
    (r"\blet\s+mut\s+(\w+)", "var $1"),
    (r"\blet\s+(\w+)", "val $1"),
    // Quantifiers: each/all → forall
    (r"\beach\s+(\w+)\s+in\b", "forall $1 in"),
    (r"\ball\s+(\w+)\s+in\b", "forall $1 in"),
    // Module: module → mod
    (r"\bmodule\s+", "mod "),
    // Negation: never → not
    (r"\bnever\s+", "not "),
    // Inheritance: derives from → extends
    (r"\bderives\s+from\s+", "extends "),
    // Equality: matches → ==
    (r"\bmatches\s+", "== "),
    // Test syntax: given → val (in test context)
    (r"\bgiven\s+(\w+)\s*=", "val $1 ="),
    // then → assert (in test context)
    (r"\bthen\s+", "assert "),
];

/// Migration rules from v0.7 to v0.8
fn get_migrations_v07_to_v08() -> Vec<(Regex, &'static str, String)> {
    vec![
        // Keywords (word boundaries to avoid partial matches)
        (
            Regex::new(r"\bgene\b").unwrap(),
            "gene → gen",
            "gen".to_string(),
        ),
        (
            Regex::new(r"\bconstraint\b").unwrap(),
            "constraint → rule",
            "rule".to_string(),
        ),
        (
            Regex::new(r"\bevolves\b").unwrap(),
            "evolves → evo",
            "evo".to_string(),
        ),
        (
            Regex::new(r"\bexegesis\b").unwrap(),
            "exegesis → docs",
            "docs".to_string(),
        ),
        // Signed integer types
        (
            Regex::new(r"\bInt8\b").unwrap(),
            "Int8 → i8",
            "i8".to_string(),
        ),
        (
            Regex::new(r"\bInt16\b").unwrap(),
            "Int16 → i16",
            "i16".to_string(),
        ),
        (
            Regex::new(r"\bInt32\b").unwrap(),
            "Int32 → i32",
            "i32".to_string(),
        ),
        (
            Regex::new(r"\bInt64\b").unwrap(),
            "Int64 → i64",
            "i64".to_string(),
        ),
        // Unsigned integer types
        (
            Regex::new(r"\bUInt8\b").unwrap(),
            "UInt8 → u8",
            "u8".to_string(),
        ),
        (
            Regex::new(r"\bUInt16\b").unwrap(),
            "UInt16 → u16",
            "u16".to_string(),
        ),
        (
            Regex::new(r"\bUInt32\b").unwrap(),
            "UInt32 → u32",
            "u32".to_string(),
        ),
        (
            Regex::new(r"\bUInt64\b").unwrap(),
            "UInt64 → u64",
            "u64".to_string(),
        ),
        // Float types
        (
            Regex::new(r"\bFloat32\b").unwrap(),
            "Float32 → f32",
            "f32".to_string(),
        ),
        (
            Regex::new(r"\bFloat64\b").unwrap(),
            "Float64 → f64",
            "f64".to_string(),
        ),
        // Other primitive types
        (
            Regex::new(r"\bBool\b").unwrap(),
            "Bool → bool",
            "bool".to_string(),
        ),
        (
            Regex::new(r"\bString\b").unwrap(),
            "String → string",
            "string".to_string(),
        ),
        (
            Regex::new(r"\bVoid\b").unwrap(),
            "Void → ()",
            "()".to_string(),
        ),
        // Generic types
        (
            Regex::new(r"\bList<([^>]+)>").unwrap(),
            "List<T> → Vec<T>",
            "Vec<$1>".to_string(),
        ),
        (
            Regex::new(r"\bOptional<([^>]+)>").unwrap(),
            "Optional<T> → Option<T>",
            "Option<$1>".to_string(),
        ),
    ]
}

/// Return statement modernization pattern
fn get_return_modernization_regex() -> Regex {
    // Match: return <expr> at end of block (before closing brace)
    // This is a simplified pattern - in production, you'd want a proper AST-based approach
    Regex::new(r"(?m)^\s*return\s+([^;\n]+)\s*\n\s*}").unwrap()
}

/// Migration result
#[derive(Debug)]
pub struct MigrationResult {
    pub path: PathBuf,
    pub original: String,
    pub migrated: String,
    pub changes: Vec<String>,
}

impl MigrationResult {
    pub fn has_changes(&self) -> bool {
        self.original != self.migrated
    }

    pub fn generate_diff(&self) -> String {
        if !self.has_changes() {
            return String::new();
        }

        let mut diff = String::new();
        diff.push_str(&format!("--- {}\n", self.path.display()));
        diff.push_str(&format!("+++ {}\n", self.path.display()));

        let original_lines: Vec<&str> = self.original.lines().collect();
        let migrated_lines: Vec<&str> = self.migrated.lines().collect();

        for (i, (orig, migr)) in original_lines.iter().zip(migrated_lines.iter()).enumerate() {
            if orig != migr {
                diff.push_str(&format!("{:4} - {}\n", i + 1, orig));
                diff.push_str(&format!("{:4} + {}\n", i + 1, migr));
            }
        }

        diff
    }
}

/// Apply v0.7 to v0.8 migration rules
fn migrate_v07_to_v08(source: &str, modernize: bool) -> (String, Vec<String>) {
    let mut result = source.to_string();
    let mut changes = Vec::new();

    // Apply keyword and type migrations
    for (regex, description, replacement) in get_migrations_v07_to_v08() {
        if regex.is_match(&result) {
            changes.push(description.to_string());
            result = regex.replace_all(&result, replacement.as_str()).to_string();
        }
    }

    // Apply return statement modernization if requested
    if modernize {
        let return_regex = get_return_modernization_regex();
        if return_regex.is_match(&result) {
            changes.push("return <expr> → <expr> (final expression)".to_string());
            result = return_regex.replace_all(&result, "$1\n}").to_string();
        }
    }

    (result, changes)
}

/// Apply v0.2 to v0.3 migration rules
fn migrate_v02_to_v03(source: &str) -> (String, Vec<String>) {
    let mut result = source.to_string();
    let mut changes = Vec::new();

    for (pattern, replacement) in MIGRATIONS_V02_TO_V03 {
        let re = Regex::new(pattern).expect("Invalid regex pattern");
        if re.is_match(&result) {
            changes.push(format!("{} -> {}", pattern, replacement));
            result = re.replace_all(&result, *replacement).to_string();
        }
    }

    (result, changes)
}

/// Migrate a single file
fn migrate_file(
    path: &Path,
    migration_fn: impl Fn(&str) -> (String, Vec<String>),
) -> Result<MigrationResult> {
    let original = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let (migrated, changes) = migration_fn(&original);

    Ok(MigrationResult {
        path: path.to_path_buf(),
        original,
        migrated,
        changes,
    })
}

/// Migrate all .dol files in a directory
fn migrate_directory(
    dir: &Path,
    migration_fn: &impl Fn(&str) -> (String, Vec<String>),
) -> Result<Vec<MigrationResult>> {
    let mut results = Vec::new();

    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            results.extend(migrate_directory(&path, migration_fn)?);
        } else if path.extension().is_some_and(|e| e == "dol") {
            results.push(migrate_file(&path, migration_fn)?);
        }
    }

    Ok(results)
}

/// Process migration results
fn process_results(results: Vec<MigrationResult>, dry_run: bool, show_diff: bool) -> Result<()> {
    let mut total_files = 0;
    let mut changed_files = 0;

    for result in results {
        total_files += 1;

        if result.has_changes() {
            changed_files += 1;

            if show_diff {
                println!("{}", result.generate_diff());
            } else {
                println!("{} {}", "✓".green().bold(), result.path.display());
                for change in &result.changes {
                    println!("  {} {}", "→".blue(), change);
                }
            }

            if !dry_run {
                fs::write(&result.path, &result.migrated)
                    .with_context(|| format!("Failed to write file: {}", result.path.display()))?;
            }
        } else if !show_diff {
            println!("{} {} (no changes)", "·".dimmed(), result.path.display());
        }
    }

    println!();
    if dry_run {
        println!(
            "{} {} files scanned, {} would be changed",
            "DRY RUN:".yellow().bold(),
            total_files,
            changed_files
        );
        if changed_files > 0 {
            println!("Run without {} to apply changes.", "--dry-run".cyan());
        }
    } else {
        println!(
            "{} {} files processed, {} files updated",
            "DONE:".green().bold(),
            total_files,
            changed_files
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::V07ToV08 {
            path,
            dry_run,
            diff,
            modernize,
        } => {
            println!(
                "{} Migrating from {} to {}",
                "→".blue().bold(),
                "v0.7.x".cyan(),
                "v0.8.0".cyan()
            );

            if dry_run {
                println!("{}", "(dry run - no files will be modified)".yellow());
            }
            if modernize {
                println!("{}", "(modernizing return statements)".yellow());
            }
            println!();

            let migration_fn = move |s: &str| migrate_v07_to_v08(s, modernize);

            let results = if path.is_dir() {
                migrate_directory(&path, &migration_fn)?
            } else {
                vec![migrate_file(&path, migration_fn)?]
            };

            process_results(results, dry_run, diff)?;
        }

        Commands::V02ToV03 {
            path,
            dry_run,
            diff,
        } => {
            println!(
                "{} Migrating from {} to {} (legacy)",
                "→".blue().bold(),
                "v0.2".cyan(),
                "v0.3".cyan()
            );

            if dry_run {
                println!("{}", "(dry run - no files will be modified)".yellow());
            }
            println!();

            let migration_fn = |s: &str| migrate_v02_to_v03(s);

            let results = if path.is_dir() {
                migrate_directory(&path, &migration_fn)?
            } else {
                vec![migrate_file(&path, migration_fn)?]
            };

            process_results(results, dry_run, diff)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // v0.2 to v0.3 tests
    #[test]
    fn test_v02_migrate_let_to_val() {
        let (result, changes) = migrate_v02_to_v03("let x = 42");
        assert_eq!(result, "val x = 42");
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_v02_migrate_let_mut_to_var() {
        let (result, _) = migrate_v02_to_v03("let mut counter = 0");
        assert_eq!(result, "var counter = 0");
    }

    #[test]
    fn test_v02_migrate_module_to_mod() {
        let (result, _) = migrate_v02_to_v03("module dol.parser @ 0.3.0");
        assert_eq!(result, "mod dol.parser @ 0.3.0");
    }

    // v0.7 to v0.8 tests
    #[test]
    fn test_v08_migrate_gene_to_gen() {
        let (result, changes) = migrate_v07_to_v08("gene container.exists {", false);
        assert_eq!(result, "gen container.exists {");
        assert!(changes.contains(&"gene → gen".to_string()));
    }

    #[test]
    fn test_v08_migrate_constraint_to_rule() {
        let (result, _) = migrate_v07_to_v08("constraint must_exist {", false);
        assert_eq!(result, "rule must_exist {");
    }

    #[test]
    fn test_v08_migrate_exegesis_to_docs() {
        let (result, _) = migrate_v07_to_v08("exegesis {", false);
        assert_eq!(result, "docs {");
    }

    #[test]
    fn test_v08_migrate_evolves_to_evo() {
        let (result, _) = migrate_v07_to_v08("evolves 0.1.0 -> 0.2.0", false);
        assert_eq!(result, "evo 0.1.0 -> 0.2.0");
    }

    #[test]
    fn test_v08_migrate_int_types() {
        let (result, _) = migrate_v07_to_v08("val x: Int32 = 42", false);
        assert_eq!(result, "val x: i32 = 42");

        let (result, _) = migrate_v07_to_v08("val y: UInt64 = 100", false);
        assert_eq!(result, "val y: u64 = 100");
    }

    #[test]
    fn test_v08_migrate_float_types() {
        let (result, _) = migrate_v07_to_v08("val x: Float32 = 3.14", false);
        assert_eq!(result, "val x: f32 = 3.14");

        let (result, _) = migrate_v07_to_v08("val y: Float64 = 2.718", false);
        assert_eq!(result, "val y: f64 = 2.718");
    }

    #[test]
    fn test_v08_migrate_bool_and_string() {
        let (result, _) = migrate_v07_to_v08("val flag: Bool = true", false);
        assert_eq!(result, "val flag: bool = true");

        let (result, _) = migrate_v07_to_v08("val name: String = \"test\"", false);
        assert_eq!(result, "val name: string = \"test\"");
    }

    #[test]
    fn test_v08_migrate_void_type() {
        let (result, _) = migrate_v07_to_v08("fun action() -> Void {", false);
        assert_eq!(result, "fun action() -> () {");
    }

    #[test]
    fn test_v08_migrate_list_type() {
        let (result, _) = migrate_v07_to_v08("val items: List<Int32>", false);
        assert_eq!(result, "val items: Vec<i32>");
    }

    #[test]
    fn test_v08_migrate_optional_type() {
        let (result, _) = migrate_v07_to_v08("val maybe: Optional<String>", false);
        assert_eq!(result, "val maybe: Option<string>");
    }

    #[test]
    fn test_v08_migrate_nested_generics() {
        let (result, _) = migrate_v07_to_v08("val data: List<Optional<Int32>>", false);
        assert_eq!(result, "val data: Vec<Option<i32>>");
    }

    #[test]
    fn test_v08_migrate_return_modernization() {
        let source = r#"fun calc() -> Int32 {
    return 42
}"#;
        let (result, changes) = migrate_v07_to_v08(source, true);
        assert!(result.contains("42\n}"));
        assert!(!result.contains("return 42"));
        assert!(changes.iter().any(|c| c.contains("return")));
    }

    #[test]
    fn test_v08_full_migration() {
        let source = r#"gene container.exists {
    container has identity: String
    container has status: Int32
}

exegesis {
    A container is the fundamental unit.
}

constraint must_have_id {
    container.identity never empty
}

fun get_status() -> Int32 {
    return status
}
"#;
        let (result, _) = migrate_v07_to_v08(source, true);

        assert!(result.contains("gen container.exists"));
        assert!(result.contains("docs {"));
        assert!(result.contains("rule must_have_id"));
        assert!(result.contains("identity: string"));
        assert!(result.contains("status: i32"));
        assert!(result.contains("-> i32"));
        assert!(!result.contains("return status"));
    }

    #[test]
    fn test_v08_no_partial_matches() {
        // Ensure we don't match partial words
        let (result, _) = migrate_v07_to_v08("val gene_name = \"test\"", false);
        // Should not change gene_name to gen_name
        assert!(result.contains("gene_name"));
    }
}
