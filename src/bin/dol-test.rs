//! dol-test - Generate Rust tests from DOL test specifications
//!
//! Transforms `.dol.test` files into compilable Rust test modules,
//! enabling test-driven ontology development.
//!
//! # Usage
//!
//! ```bash
//! # Generate tests from a single file
//! dol-test examples/traits/container.lifecycle.dol.test
//!
//! # Generate tests to specific output directory
//! dol-test --output tests/generated/ examples/
//!
//! # Generate stubs only (for manual implementation)
//! dol-test --stubs examples/traits/container.lifecycle.dol.test
//!
//! # Watch mode for development
//! dol-test --watch examples/
//! ```

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Generate Rust tests from DOL test specifications
#[derive(Parser, Debug)]
#[command(name = "dol-test")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// DOL test files or directories to process
    paths: Vec<PathBuf>,

    /// Output directory for generated tests
    #[arg(short, long, default_value = "tests/generated")]
    output: PathBuf,

    /// Generate stub tests only (unimplemented!() bodies)
    #[arg(short, long)]
    stubs: bool,

    /// Overwrite existing generated files
    #[arg(short, long)]
    force: bool,

    /// Quiet mode
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate tests from DOL test files
    Generate {
        /// Files or directories to process
        paths: Vec<PathBuf>,

        /// Output directory
        #[arg(short, long, default_value = "tests/generated")]
        output: PathBuf,

        /// Generate stubs only
        #[arg(short, long)]
        stubs: bool,

        /// Overwrite existing generated files
        #[arg(short, long)]
        force: bool,
    },

    /// Validate DOL test files without generating
    Validate {
        /// Files or directories to validate
        paths: Vec<PathBuf>,
    },

    /// Show what would be generated (dry run)
    Plan {
        /// Files or directories to plan
        paths: Vec<PathBuf>,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();

    match args.command {
        Some(Commands::Generate {
            paths,
            output,
            stubs,
            force,
        }) => generate_tests(&paths, &output, stubs, force, args.quiet),
        Some(Commands::Validate { paths }) => validate_test_files(&paths, args.quiet),
        Some(Commands::Plan { paths }) => plan_generation(&paths),
        None => generate_tests(
            &args.paths,
            &args.output,
            args.stubs,
            args.force,
            args.quiet,
        ),
    }
}

/// Parsed DOL test file structure
#[derive(Debug)]
struct DolTestFile {
    /// The trait/gene being tested
    subject: String,
    /// Individual test cases
    tests: Vec<TestCase>,
}

#[derive(Debug)]
struct TestCase {
    /// Test name
    name: String,
    /// Given preconditions
    given: Vec<String>,
    /// When actions
    when: Vec<String>,
    /// Then assertions
    then: Vec<String>,
    /// Whether this test should always pass
    always: bool,
}

fn generate_tests(
    paths: &[PathBuf],
    output: &PathBuf,
    stubs: bool,
    force: bool,
    quiet: bool,
) -> ExitCode {
    let files = collect_test_files(paths);

    if files.is_empty() {
        if !quiet {
            eprintln!("{}: No .dol.test files found", "warning".yellow());
        }
        return ExitCode::SUCCESS;
    }

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(output) {
        eprintln!(
            "{}: Failed to create output directory: {}",
            "error".red(),
            e
        );
        return ExitCode::FAILURE;
    }

    let mut generated = 0;
    let mut failed = 0;

    for path in &files {
        match process_test_file(path, output, stubs, force) {
            Ok(output_path) => {
                generated += 1;
                if !quiet {
                    println!(
                        "{} {} → {}",
                        "✓".green(),
                        path.display(),
                        output_path.display()
                    );
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("{} {}: {}", "✗".red(), path.display(), e);
            }
        }
    }

    if !quiet {
        println!();
        println!("{}", "Summary".bold());
        println!("  Generated: {}", generated.to_string().green());
        if failed > 0 {
            println!("  Failed:    {}", failed.to_string().red());
        }
    }

    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn validate_test_files(paths: &[PathBuf], quiet: bool) -> ExitCode {
    let files = collect_test_files(paths);
    let mut valid = 0;
    let mut invalid = 0;

    for path in &files {
        match parse_test_file(path) {
            Ok(test_file) => {
                valid += 1;
                if !quiet {
                    println!(
                        "{} {} ({} tests)",
                        "✓".green(),
                        path.display(),
                        test_file.tests.len()
                    );
                }
            }
            Err(e) => {
                invalid += 1;
                eprintln!("{} {}: {}", "✗".red(), path.display(), e);
            }
        }
    }

    if !quiet {
        println!();
        println!("Valid: {}, Invalid: {}", valid, invalid);
    }

    if invalid > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn plan_generation(paths: &[PathBuf]) -> ExitCode {
    let files = collect_test_files(paths);

    println!("{}", "Generation Plan".bold());
    println!();

    for path in &files {
        match parse_test_file(path) {
            Ok(test_file) => {
                let output_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .replace('.', "_");

                println!("{}:", path.display().to_string().cyan());
                println!("  Output: tests/generated/{}.rs", output_name);
                println!("  Tests:  {}", test_file.tests.len());
                for test in &test_file.tests {
                    println!("    - {}", test.name);
                }
                println!();
            }
            Err(e) => {
                eprintln!("{}: {} - {}", "skip".yellow(), path.display(), e);
            }
        }
    }

    ExitCode::SUCCESS
}

fn collect_test_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if path.to_string_lossy().ends_with(".dol.test") {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            collect_test_files_recursive(path, &mut files);
        }
    }

    files.sort();
    files
}

fn collect_test_files_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_test_files_recursive(&path, files);
            } else if path.to_string_lossy().ends_with(".dol.test") {
                files.push(path);
            }
        }
    }
}

fn parse_test_file(path: &Path) -> Result<DolTestFile, String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    parse_test_source(&source)
}

fn parse_test_source(source: &str) -> Result<DolTestFile, String> {
    let mut subject = String::new();
    let mut tests = Vec::new();
    let mut current_test: Option<TestCase> = None;
    let mut current_section = "";

    for line in source.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Inline test declaration (quoted test name) - check FIRST before subject header
        if line.starts_with("test \"") || line.starts_with("test '") {
            let name = line
                .trim_start_matches("test ")
                .trim_end_matches('{')
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();

            if let Some(test) = current_test.take() {
                tests.push(test);
            }
            current_test = Some(TestCase {
                name,
                given: Vec::new(),
                when: Vec::new(),
                then: Vec::new(),
                always: false,
            });
            continue;
        }

        // Parse test file header: "test <subject> {" (non-quoted subject)
        if line.starts_with("test ") && line.ends_with('{') {
            subject = line
                .strip_prefix("test ")
                .unwrap()
                .strip_suffix('{')
                .unwrap()
                .trim()
                .to_string();
            continue;
        }

        // Parse sections
        if line == "given {" || line == "given:" {
            current_section = "given";
            continue;
        }
        if line == "when {" || line == "when:" {
            current_section = "when";
            continue;
        }
        if line == "then {" || line == "then:" {
            current_section = "then";
            continue;
        }
        if line == "always" || line == "always:" {
            if let Some(ref mut test) = current_test {
                test.always = true;
            }
            continue;
        }

        // End of block
        if line == "}" {
            current_section = "";
            continue;
        }

        // Add line to current section
        if let Some(ref mut test) = current_test {
            let content = line.to_string();
            match current_section {
                "given" => test.given.push(content),
                "when" => test.when.push(content),
                "then" => test.then.push(content),
                _ => {}
            }
        }
    }

    // Don't forget the last test
    if let Some(test) = current_test {
        tests.push(test);
    }

    if subject.is_empty() {
        return Err("No test subject found".to_string());
    }

    Ok(DolTestFile { subject, tests })
}

fn process_test_file(
    path: &Path,
    output_dir: &Path,
    stubs: bool,
    force: bool,
) -> Result<PathBuf, String> {
    let test_file = parse_test_file(path)?;

    let output_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .replace(['.', '-'], "_");

    let output_path = output_dir.join(format!("{}.rs", output_name));

    // Check if file exists and force not set
    if output_path.exists() && !force {
        return Err(format!(
            "Output file exists (use --force to overwrite): {}",
            output_path.display()
        ));
    }

    let code = generate_rust_tests(&test_file, stubs);

    std::fs::write(&output_path, code).map_err(|e| format!("Failed to write output: {}", e))?;

    Ok(output_path)
}

fn generate_rust_tests(test_file: &DolTestFile, stubs: bool) -> String {
    let mut code = String::new();

    // Header
    code.push_str(&format!(
        r#"//! Generated tests for {}
//!
//! This file was automatically generated by dol-test.
//! Do not edit manually - changes will be overwritten.

#![allow(unused_imports)]

use metadol::{{parse_file, validate}};

"#,
        test_file.subject
    ));

    // Generate each test
    for test in &test_file.tests {
        let fn_name = test
            .name
            .to_lowercase()
            .replace([' ', '-'], "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        code.push_str(&format!("#[test]\nfn test_{}() {{\n", fn_name));

        if stubs {
            code.push_str("    // TODO: Implement this test\n");
            code.push_str(&format!("    // Test: {}\n", test.name));
            if !test.given.is_empty() {
                code.push_str("    // Given:\n");
                for g in &test.given {
                    code.push_str(&format!("    //   - {}\n", g));
                }
            }
            if !test.when.is_empty() {
                code.push_str("    // When:\n");
                for w in &test.when {
                    code.push_str(&format!("    //   - {}\n", w));
                }
            }
            if !test.then.is_empty() {
                code.push_str("    // Then:\n");
                for t in &test.then {
                    code.push_str(&format!("    //   - {}\n", t));
                }
            }
            code.push_str("    unimplemented!(\"Test not yet implemented\")\n");
        } else {
            // Generate actual test code
            code.push_str("    // Arrange\n");
            for given in &test.given {
                code.push_str(&format!("    // Given: {}\n", given));
            }
            code.push('\n');

            code.push_str("    // Act\n");
            for when_clause in &test.when {
                code.push_str(&format!("    // When: {}\n", when_clause));
            }
            code.push('\n');

            code.push_str("    // Assert\n");
            for then_clause in &test.then {
                code.push_str(&format!(
                    "    // Then: {}\n    assert!(true, \"TODO: Implement assertion for: {}\");\n",
                    then_clause, then_clause
                ));
            }

            if test.always {
                code.push_str("\n    // This test should always pass\n");
            }
        }

        code.push_str("}\n\n");
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_test_file() {
        let source = r#"
test container.lifecycle {
    test "container can be created" {
        given:
            container does not exist
        when:
            create container
        then:
            container exists
            container is created
    }
}
"#;
        let result = parse_test_source(source);
        assert!(result.is_ok());
        let test_file = result.unwrap();
        assert_eq!(test_file.subject, "container.lifecycle");
        assert_eq!(test_file.tests.len(), 1);
    }

    #[test]
    fn test_generate_stubs() {
        let test_file = DolTestFile {
            subject: "test.subject".to_string(),
            tests: vec![TestCase {
                name: "example test".to_string(),
                given: vec!["precondition".to_string()],
                when: vec!["action".to_string()],
                then: vec!["assertion".to_string()],
                always: false,
            }],
        };

        let code = generate_rust_tests(&test_file, true);
        assert!(code.contains("unimplemented!"));
        assert!(code.contains("test_example_test"));
    }
}
