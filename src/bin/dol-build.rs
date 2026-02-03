//! dol-build - Build DOL Spirit projects to WASM
//!
//! This tool orchestrates the complete build pipeline from Spirit.dol manifest
//! to a fully packaged WASM binary with JavaScript bindings.
//!
//! # Build Pipeline
//!
//! ```text
//! ┌─────────────────┐
//! │  Spirit.dol     │  Parse manifest, resolve config
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  lib.dol + deps │  Resolve module dependencies
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  Rust Codegen   │  Generate Rust code from DOL
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  Cargo.toml     │  Generate build manifest
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  cargo build    │  Compile to WASM
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  wasm-bindgen   │  Generate JS bindings
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  Package output │  Bundle WASM + manifest.json
//! └─────────────────┘
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Build a Spirit project in current directory
//! dol-build
//!
//! # Build a specific Spirit project
//! dol-build ./my-spirit
//!
//! # Build with custom output directory
//! dol-build -o ./dist
//!
//! # Build with optimizations
//! dol-build --release
//!
//! # Build without wasm-bindgen step
//! dol-build --no-bindgen
//! ```

use clap::Parser;
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use metadol::codegen::HirRustCodegen;
use metadol::lower::lower_file;
use metadol::manifest::{parse_spirit_manifest, BuildConfig, SpiritManifest};

/// Build a DOL Spirit project to WASM
#[derive(Parser, Debug)]
#[command(name = "dol-build")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Spirit project directory (defaults to current directory)
    #[arg(default_value = ".")]
    project_dir: PathBuf,

    /// Output directory for build artifacts
    #[arg(short, long, default_value = "target/spirit")]
    output: PathBuf,

    /// Build in release mode with optimizations
    #[arg(short, long)]
    release: bool,

    /// Skip wasm-bindgen step
    #[arg(long)]
    no_bindgen: bool,

    /// Clean build (remove existing artifacts first)
    #[arg(long)]
    clean: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Quiet mode: only show errors
    #[arg(short, long)]
    quiet: bool,
}

/// Build orchestrator that manages the multi-stage build pipeline
struct BuildOrchestrator {
    project_dir: PathBuf,
    output_dir: PathBuf,
    manifest: SpiritManifest,
    build_config: BuildConfig,
    verbose: bool,
    quiet: bool,
}

impl BuildOrchestrator {
    /// Create a new build orchestrator
    fn new(
        project_dir: PathBuf,
        output_dir: PathBuf,
        manifest: SpiritManifest,
        verbose: bool,
        quiet: bool,
    ) -> Self {
        let build_config = manifest.build_config.clone().unwrap_or_default();

        Self {
            project_dir,
            output_dir,
            manifest,
            build_config,
            verbose,
            quiet,
        }
    }

    /// Execute the complete build pipeline
    fn build(&self, release: bool, skip_bindgen: bool) -> Result<(), String> {
        self.log_info(&format!(
            "Building Spirit: {}",
            self.manifest.qualified_name()
        ));

        // Stage 1: Resolve modules
        let modules = self.stage1_resolve_modules()?;
        self.log_success(&format!("Resolved {} modules", modules.len()));

        // Stage 2: Generate Rust code
        let rust_files = self.stage2_codegen_rust(&modules)?;
        self.log_success(&format!("Generated {} Rust files", rust_files.len()));

        // Stage 3: Generate Cargo.toml
        self.stage3_generate_cargo_toml()?;
        self.log_success("Generated Cargo.toml");

        // Stage 4: Run cargo build
        let wasm_path = self.stage4_cargo_build(release)?;
        self.log_success(&format!("Built WASM: {}", wasm_path.display()));

        // Stage 5: Run wasm-bindgen (optional)
        if !skip_bindgen {
            let bindgen_output = self.stage5_wasm_bindgen(&wasm_path)?;
            self.log_success(&format!(
                "Generated JS bindings: {}",
                bindgen_output.display()
            ));
        }

        // Stage 6: Package output
        self.stage6_package_output(&wasm_path)?;
        self.log_success("Packaged output");

        Ok(())
    }

    /// Stage 1: Scan filesystem for all .dol modules
    fn stage1_resolve_modules(&self) -> Result<HashMap<String, String>, String> {
        self.log_stage("Stage 1: Resolving modules");

        let mut modules = HashMap::new();

        // Scan for all .dol files in src/ directory
        let src_dir = self.project_dir.join("src");
        if !src_dir.exists() {
            return Err("Source directory not found: src/".to_string());
        }

        // Walk the src directory and find all .dol files
        self.scan_dol_files(&src_dir, &src_dir, &mut modules)?;

        if modules.is_empty() {
            return Err("No .dol files found in src/".to_string());
        }

        Ok(modules)
    }

    /// Recursively scan for .dol files and load them
    fn scan_dol_files(
        &self,
        dir: &Path,
        base_dir: &Path,
        modules: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_dol_files(&path, base_dir, modules)?;
            } else if path.extension().is_some_and(|ext| ext == "dol") {
                // Found a .dol file
                let relative_path = path
                    .strip_prefix(base_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;

                // Convert path to module name: src/genes/cell.dol -> genes_cell
                let module_name = relative_path
                    .with_extension("")
                    .to_string_lossy()
                    .replace(['/', '-'], "_");

                // Skip mod.dol files (just directory markers)
                if module_name.ends_with("_mod") {
                    continue;
                }

                self.log_verbose(&format!(
                    "Found module: {} ({})",
                    module_name,
                    path.display()
                ));

                // Read source
                let source = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

                modules.insert(module_name, source);
            }
        }

        Ok(())
    }

    /// Stage 2: Generate Rust code from DOL modules
    fn stage2_codegen_rust(
        &self,
        modules: &HashMap<String, String>,
    ) -> Result<Vec<PathBuf>, String> {
        self.log_stage("Stage 2: Generating Rust code");

        // Create src/generated/ directory structure (matches build.sh)
        let generated_dir = self.output_dir.join("src").join("generated");
        std::fs::create_dir_all(&generated_dir)
            .map_err(|e| format!("Failed to create src/generated/: {}", e))?;

        let mut rust_files = Vec::new();

        for (module_name, source) in modules {
            self.log_verbose(&format!("Compiling {} ...", module_name));

            // Lower DOL to HIR and generate Rust
            let (hir, ctx) = lower_file(source).map_err(|e| format!("Parse error: {}", e))?;

            // Check for errors
            if ctx.has_errors() {
                let errors: Vec<String> = ctx
                    .diagnostics()
                    .iter()
                    .filter(|d| matches!(d.kind, metadol::lower::DiagnosticKind::Error))
                    .map(|d| d.message.clone())
                    .collect();
                return Err(format!(
                    "Compilation errors in {}: {}",
                    module_name,
                    errors.join("; ")
                ));
            }

            // Generate Rust code
            let mut codegen = HirRustCodegen::with_symbols(ctx.symbols);
            let rust_code = codegen.generate(&hir);

            // Write to generated/ directory
            let rust_file = generated_dir.join(format!("{}.rs", module_name));
            std::fs::write(&rust_file, rust_code)
                .map_err(|e| format!("Failed to write {}: {}", rust_file.display(), e))?;

            rust_files.push(rust_file);
        }

        // Generate src/lib.rs that imports the generated modules
        let lib_rs = self.output_dir.join("src").join("lib.rs");
        let mut lib_content = String::from("//! Generated Spirit library\n\n");

        // Create generated module
        lib_content.push_str("pub mod generated {\n");

        // Add module declarations for each generated file
        let mut module_names: Vec<_> = modules.keys().collect();
        module_names.sort(); // Alphabetical order

        for module_name in module_names {
            lib_content.push_str(&format!("    pub mod {};\n", module_name));
        }

        lib_content.push_str("}\n\n");

        // Re-export all generated modules at top level
        lib_content.push_str("// Re-export generated modules\n");
        for module_name in modules.keys() {
            lib_content.push_str(&format!("pub use generated::{};\n", module_name));
        }

        std::fs::write(&lib_rs, lib_content)
            .map_err(|e| format!("Failed to write lib.rs: {}", e))?;

        rust_files.push(lib_rs);

        Ok(rust_files)
    }

    /// Stage 3: Generate Cargo.toml for the Rust crate
    fn stage3_generate_cargo_toml(&self) -> Result<(), String> {
        self.log_stage("Stage 3: Generating Cargo.toml");

        let cargo_toml_path = self.output_dir.join("Cargo.toml");

        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "{}"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1

[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-Oz", "--enable-mutable-globals"]
"#,
            self.manifest.name, self.manifest.version, self.build_config.rust_edition
        );

        std::fs::write(&cargo_toml_path, cargo_toml)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

        Ok(())
    }

    /// Stage 4: Run cargo build to compile to WASM
    fn stage4_cargo_build(&self, release: bool) -> Result<PathBuf, String> {
        self.log_stage("Stage 4: Compiling to WASM");

        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--target")
            .arg(&self.build_config.wasm_target)
            .current_dir(&self.output_dir);

        if release || self.build_config.optimize {
            cmd.arg("--release");
        }

        self.log_verbose(&format!("Running: {:?}", cmd));

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("cargo build failed:\n{}", stderr));
        }

        // Determine WASM output path
        let profile = if release || self.build_config.optimize {
            "release"
        } else {
            "debug"
        };

        let wasm_path = self
            .output_dir
            .join("target")
            .join(&self.build_config.wasm_target)
            .join(profile)
            .join(format!("{}.wasm", self.manifest.name.replace('-', "_")));

        if !wasm_path.exists() {
            return Err(format!("WASM output not found at: {}", wasm_path.display()));
        }

        Ok(wasm_path)
    }

    /// Stage 5: Run wasm-bindgen to generate JS bindings
    fn stage5_wasm_bindgen(&self, wasm_path: &Path) -> Result<PathBuf, String> {
        self.log_stage("Stage 5: Generating JS bindings");

        let bindgen_output = self.output_dir.join("pkg");
        std::fs::create_dir_all(&bindgen_output)
            .map_err(|e| format!("Failed to create pkg/: {}", e))?;

        let mut cmd = Command::new("wasm-bindgen");
        cmd.arg(wasm_path)
            .arg("--out-dir")
            .arg(&bindgen_output)
            .arg("--target")
            .arg("web");

        self.log_verbose(&format!("Running: {:?}", cmd));

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run wasm-bindgen: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("wasm-bindgen failed:\n{}", stderr));
        }

        Ok(bindgen_output)
    }

    /// Stage 6: Package output with manifest and metadata
    fn stage6_package_output(&self, wasm_path: &Path) -> Result<(), String> {
        self.log_stage("Stage 6: Packaging output");

        // Create manifest.json with Spirit metadata
        let manifest_json = serde_json::json!({
            "name": self.manifest.name,
            "version": self.manifest.version.to_string(),
            "docs": self.manifest.docs,
            "entry": self.manifest.entry_file(),
            "target": self.build_config.wasm_target,
            "modules": self.manifest.modules.iter().map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "visibility": format!("{:?}", m.visibility),
                })
            }).collect::<Vec<_>>(),
        });

        let manifest_path = self.output_dir.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest_json)
                .map_err(|e| format!("Failed to serialize manifest: {}", e))?,
        )
        .map_err(|e| format!("Failed to write manifest.json: {}", e))?;

        // Copy WASM to output root
        let final_wasm = self.output_dir.join(format!("{}.wasm", self.manifest.name));
        std::fs::copy(wasm_path, &final_wasm).map_err(|e| format!("Failed to copy WASM: {}", e))?;

        Ok(())
    }

    // Logging helpers
    fn log_stage(&self, message: &str) {
        if !self.quiet {
            eprintln!("\n{}", message.cyan().bold());
        }
    }

    fn log_info(&self, message: &str) {
        if !self.quiet {
            eprintln!("{} {}", "info:".blue().bold(), message);
        }
    }

    fn log_success(&self, message: &str) {
        if !self.quiet {
            eprintln!("{} {}", "✓".green().bold(), message);
        }
    }

    fn log_verbose(&self, message: &str) {
        if self.verbose && !self.quiet {
            eprintln!("{} {}", "debug:".dimmed(), message.dimmed());
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();

    // Validate project directory
    if !args.project_dir.exists() {
        eprintln!(
            "{}: Project directory not found: {}",
            "error".red(),
            args.project_dir.display()
        );
        return ExitCode::FAILURE;
    }

    // Find Spirit.dol manifest
    let spirit_path = args.project_dir.join("Spirit.dol");
    if !spirit_path.exists() {
        eprintln!(
            "{}: Spirit.dol not found in {}",
            "error".red(),
            args.project_dir.display()
        );
        eprintln!("A Spirit project must have a Spirit.dol manifest file.");
        return ExitCode::FAILURE;
    }

    // Parse manifest
    let manifest_source = match std::fs::read_to_string(&spirit_path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("{}: Failed to read Spirit.dol: {}", "error".red(), e);
            return ExitCode::FAILURE;
        }
    };

    let manifest = match parse_spirit_manifest(&manifest_source) {
        Ok(manifest) => manifest,
        Err(e) => {
            eprintln!("{}: Failed to parse Spirit.dol: {}", "error".red(), e);
            return ExitCode::FAILURE;
        }
    };

    // Clean output directory if requested
    if args.clean && args.output.exists() {
        if !args.quiet {
            eprintln!("{} {}", "Cleaning".yellow(), args.output.display());
        }
        if let Err(e) = std::fs::remove_dir_all(&args.output) {
            eprintln!("{}: Failed to clean output directory: {}", "error".red(), e);
            return ExitCode::FAILURE;
        }
    }

    // Create output directory
    if let Err(e) = std::fs::create_dir_all(&args.output) {
        eprintln!(
            "{}: Failed to create output directory: {}",
            "error".red(),
            e
        );
        return ExitCode::FAILURE;
    }

    // Create orchestrator and run build
    let orchestrator = BuildOrchestrator::new(
        args.project_dir.clone(),
        args.output.clone(),
        manifest,
        args.verbose,
        args.quiet,
    );

    match orchestrator.build(args.release, args.no_bindgen) {
        Ok(()) => {
            if !args.quiet {
                eprintln!(
                    "\n{} Spirit build complete: {}",
                    "✓".green().bold(),
                    args.output.display()
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("\n{}: {}", "error".red().bold(), e);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_orchestrator_creation() {
        use metadol::ast::{Span, Version};
        use metadol::manifest::SpiritConfig;

        let manifest = SpiritManifest {
            name: "test".to_string(),
            version: Version {
                major: 0,
                minor: 1,
                patch: 0,
                suffix: None,
            },
            docs: None,
            dependencies: vec![],
            requirements: vec![],
            config: SpiritConfig {
                entry: "lib.dol".to_string(),
                target: "wasm32".to_string(),
                features: vec![],
                span: Span::default(),
            },
            targets: None,
            build_config: None,
            modules: vec![],
            span: Span::default(),
        };

        let orchestrator = BuildOrchestrator::new(
            PathBuf::from("."),
            PathBuf::from("target/spirit"),
            manifest,
            false,
            false,
        );

        assert_eq!(orchestrator.manifest.name, "test");
        assert_eq!(orchestrator.build_config.rust_edition, "2021");
    }
}
