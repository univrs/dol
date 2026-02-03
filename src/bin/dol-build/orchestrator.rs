//! Build orchestration for DOL Spirit packages.
//!
//! This module coordinates the multi-stage build pipeline:
//! 1. Resolve modules from Spirit manifest
//! 2. Generate Rust code from DOL sources
//! 3. Generate Cargo.toml with dependencies
//! 4. Compile to WASM with cargo
//! 5. Run wasm-bindgen for JS bindings
//! 6. Package the output

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use metadol::ast::{DolFile, Version};
use metadol::codegen::RustCodegen;
use metadol::manifest::{BuildConfig, Dependency, SpiritManifest};
use metadol::{parse_dol_file, ParseError};

/// Represents a resolved DOL module with its source and dependencies.
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name (e.g., "lib", "types")
    pub name: String,
    /// Full module path (e.g., "container.lib")
    pub full_path: String,
    /// File path to the .dol source
    pub file_path: PathBuf,
    /// Parsed DOL file content
    pub content: DolFile,
    /// Other modules this one depends on
    pub dependencies: Vec<String>,
}

/// A packaged Spirit ready for distribution.
#[derive(Debug, Clone)]
pub struct SpiritPackage {
    /// Spirit name
    pub name: String,
    /// Spirit version
    pub version: Version,
    /// Path to the compiled WASM binary
    pub wasm_path: PathBuf,
    /// Path to generated JS bindings (if wasm-bindgen was run)
    pub js_bindings: Option<PathBuf>,
    /// Path to TypeScript definitions (if wasm-bindgen was run)
    pub ts_definitions: Option<PathBuf>,
    /// Build artifacts directory
    pub artifacts_dir: PathBuf,
}

/// Multi-stage build orchestrator for DOL Spirit packages.
pub struct BuildOrchestrator {
    /// Parsed Spirit manifest
    manifest: SpiritManifest,
    /// Build configuration
    config: BuildConfig,
    /// Output directory for build artifacts
    output_dir: PathBuf,
    /// Source directory containing DOL files
    source_dir: PathBuf,
}

impl BuildOrchestrator {
    /// Create a new build orchestrator.
    ///
    /// # Arguments
    ///
    /// * `manifest` - Parsed Spirit.dol manifest
    /// * `config` - Build configuration
    /// * `output_dir` - Directory for build artifacts
    /// * `source_dir` - Directory containing DOL source files
    pub fn new(
        manifest: SpiritManifest,
        config: BuildConfig,
        output_dir: PathBuf,
        source_dir: PathBuf,
    ) -> Self {
        Self {
            manifest,
            config,
            output_dir,
            source_dir,
        }
    }

    /// Create from a Spirit.dol manifest file.
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - Path to Spirit.dol
    /// * `output_dir` - Directory for build artifacts
    pub fn from_manifest_file(
        manifest_path: &Path,
        output_dir: PathBuf,
    ) -> Result<Self, BuildError> {
        let source = fs::read_to_string(manifest_path)
            .map_err(|e| BuildError::IoError(format!("Failed to read manifest: {}", e)))?;

        let manifest = metadol::manifest::ManifestParser::new(&source)
            .parse()
            .map_err(|e| BuildError::ParseError(format!("Failed to parse manifest: {}", e)))?;

        let config = manifest.get_build_config();

        let source_dir = manifest_path
            .parent()
            .ok_or_else(|| BuildError::InvalidPath("Manifest has no parent directory".into()))?
            .to_path_buf();

        Ok(Self::new(manifest, config, output_dir, source_dir))
    }

    /// Resolve all modules referenced in the manifest.
    ///
    /// This discovers all DOL files, parses them, and builds a dependency graph.
    pub fn resolve_modules(&self) -> Result<Vec<Module>, BuildError> {
        let mut modules = Vec::new();
        let mut discovered = HashSet::new();

        // Start with the entry point
        let entry_file = self.manifest.entry_file();
        self.resolve_module_recursive(entry_file, &mut modules, &mut discovered)?;

        // Also include explicitly declared modules
        for module_export in &self.manifest.modules {
            let module_file = format!("{}.dol", module_export.name);
            if !discovered.contains(&module_file) {
                self.resolve_module_recursive(&module_file, &mut modules, &mut discovered)?;
            }
        }

        // Sort by dependencies (topological sort)
        self.sort_modules_by_dependencies(&mut modules)?;

        Ok(modules)
    }

    /// Recursively resolve a module and its dependencies.
    fn resolve_module_recursive(
        &self,
        file_name: &str,
        modules: &mut Vec<Module>,
        discovered: &mut HashSet<String>,
    ) -> Result<(), BuildError> {
        if discovered.contains(file_name) {
            return Ok(());
        }

        discovered.insert(file_name.to_string());

        let file_path = self.source_dir.join(file_name);
        if !file_path.exists() {
            return Err(BuildError::ModuleNotFound(format!(
                "Module file not found: {}",
                file_path.display()
            )));
        }

        let source = fs::read_to_string(&file_path)
            .map_err(|e| BuildError::IoError(format!("Failed to read {}: {}", file_name, e)))?;

        let content = parse_dol_file(&source).map_err(|e| {
            BuildError::ParseError(format!("Failed to parse {}: {}", file_name, e))
        })?;

        // Extract module name
        let module_name = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let full_path = content
            .module
            .as_ref()
            .map(|m| m.path.join("."))
            .unwrap_or_else(|| module_name.clone());

        // Extract dependencies from use declarations
        let mut dependencies = Vec::new();
        for use_decl in &content.uses {
            if let Some(first) = use_decl.path.first() {
                // Local module references (not std or external)
                if first != "std" && !first.starts_with('@') {
                    if let Some(dep_module) = use_decl.path.get(1) {
                        dependencies.push(dep_module.clone());

                        // Recursively resolve this dependency
                        let dep_file = format!("{}.dol", dep_module);
                        self.resolve_module_recursive(&dep_file, modules, discovered)?;
                    }
                }
            }
        }

        modules.push(Module {
            name: module_name,
            full_path,
            file_path,
            content,
            dependencies,
        });

        Ok(())
    }

    /// Sort modules in dependency order (topological sort).
    fn sort_modules_by_dependencies(&self, modules: &mut Vec<Module>) -> Result<(), BuildError> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        let module_map: HashMap<String, &Module> = modules
            .iter()
            .map(|m| (m.name.clone(), m))
            .collect();

        for module in modules.iter() {
            self.visit_module(
                &module.name,
                &module_map,
                &mut visited,
                &mut visiting,
                &mut sorted,
            )?;
        }

        // Replace modules with sorted order
        *modules = sorted;
        Ok(())
    }

    fn visit_module(
        &self,
        name: &str,
        module_map: &HashMap<String, &Module>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        sorted: &mut Vec<Module>,
    ) -> Result<(), BuildError> {
        if visited.contains(name) {
            return Ok(());
        }

        if visiting.contains(name) {
            return Err(BuildError::CircularDependency(format!(
                "Circular dependency detected involving module: {}",
                name
            )));
        }

        visiting.insert(name.to_string());

        if let Some(module) = module_map.get(name) {
            // Visit dependencies first
            for dep in &module.dependencies {
                self.visit_module(dep, module_map, visited, visiting, sorted)?;
            }

            sorted.push((*module).clone());
            visited.insert(name.to_string());
        }

        visiting.remove(name);
        Ok(())
    }

    /// Generate Rust code from resolved modules.
    ///
    /// Creates a Rust crate structure in the output directory.
    pub fn codegen_rust(&self, modules: &[Module]) -> Result<(), BuildError> {
        let src_dir = self.output_dir.join("src");
        fs::create_dir_all(&src_dir)
            .map_err(|e| BuildError::IoError(format!("Failed to create src dir: {}", e)))?;

        let codegen = RustCodegen::new();

        // Generate each module
        for module in modules {
            let mut content = String::new();

            // Module documentation
            content.push_str(&format!("//! Module: {}\n", module.full_path));
            content.push_str("//! Generated from DOL source - do not edit manually\n\n");

            // Generate use statements
            content.push_str("#[allow(unused_imports)]\n");
            for dep in &module.dependencies {
                content.push_str(&format!("use crate::{}::*;\n", dep));
            }
            content.push('\n');

            // Generate declarations
            let declarations_code = codegen.gen_file(&module.content.declarations);
            content.push_str(&declarations_code);

            // Write module file
            let module_file = src_dir.join(format!("{}.rs", module.name));
            fs::write(&module_file, content).map_err(|e| {
                BuildError::IoError(format!("Failed to write {}: {}", module.name, e))
            })?;
        }

        // Generate lib.rs
        self.generate_lib_rs(modules)?;

        Ok(())
    }

    /// Generate lib.rs with module declarations.
    fn generate_lib_rs(&self, modules: &[Module]) -> Result<(), BuildError> {
        let mut content = String::new();

        content.push_str(&format!(
            "//! {} - Generated DOL Spirit\n",
            self.manifest.name
        ));
        content.push_str("//! Do not edit manually - regenerate from DOL source\n\n");

        // Standard library imports
        content.push_str("use std::collections::HashMap;\n\n");

        // Module declarations
        for module in modules {
            content.push_str(&format!("pub mod {};\n", module.name));
        }

        // Write lib.rs
        let lib_path = self.output_dir.join("src/lib.rs");
        fs::write(&lib_path, content)
            .map_err(|e| BuildError::IoError(format!("Failed to write lib.rs: {}", e)))?;

        Ok(())
    }

    /// Generate Cargo.toml with dependencies.
    pub fn generate_cargo_toml(&self) -> Result<(), BuildError> {
        let mut content = String::new();

        // Package section
        content.push_str("[package]\n");
        content.push_str(&format!("name = \"{}\"\n", self.manifest.name));
        content.push_str(&format!("version = \"{}\"\n", self.manifest.version));
        content.push_str(&format!("edition = \"{}\"\n", self.config.rust_edition));
        content.push_str("\n");

        // Lib section for WASM
        content.push_str("[lib]\n");
        content.push_str("crate-type = [\"cdylib\", \"rlib\"]\n\n");

        // Dependencies
        content.push_str("[dependencies]\n");

        // Add wasm-bindgen for WASM targets
        content.push_str("wasm-bindgen = \"0.2\"\n");

        // Add dependencies from manifest
        for dep in &self.manifest.dependencies {
            let dep_name = self.dependency_to_cargo_name(dep);
            if let Some(version) = &dep.version_constraint {
                content.push_str(&format!("{} = \"{}\"\n", dep_name, version));
            } else {
                content.push_str(&format!("{} = \"*\"\n", dep_name));
            }
        }

        content.push_str("\n");

        // Profile for release builds
        content.push_str("[profile.release]\n");
        content.push_str("opt-level = \"z\"\n");
        content.push_str("lto = true\n");
        content.push_str("codegen-units = 1\n");

        // Write Cargo.toml
        let cargo_path = self.output_dir.join("Cargo.toml");
        fs::write(&cargo_path, content)
            .map_err(|e| BuildError::IoError(format!("Failed to write Cargo.toml: {}", e)))?;

        Ok(())
    }

    /// Convert a DOL dependency to a Cargo package name.
    fn dependency_to_cargo_name(&self, dep: &Dependency) -> String {
        // Convert @scope/package to scope-package
        dep.path.join("-").replace('@', "")
    }

    /// Compile the Rust crate to WASM using cargo.
    pub fn cargo_build(&self) -> Result<PathBuf, BuildError> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--release")
            .arg("--target")
            .arg(&self.config.wasm_target)
            .current_dir(&self.output_dir);

        let output = cmd.output().map_err(|e| {
            BuildError::CompileError(format!("Failed to run cargo build: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::CompileError(format!(
                "Cargo build failed:\n{}",
                stderr
            )));
        }

        // Locate the WASM binary
        let wasm_name = format!("{}.wasm", self.manifest.name.replace('-', "_"));
        let wasm_path = self
            .output_dir
            .join("target")
            .join(&self.config.wasm_target)
            .join("release")
            .join(&wasm_name);

        if !wasm_path.exists() {
            return Err(BuildError::CompileError(format!(
                "WASM binary not found at: {}",
                wasm_path.display()
            )));
        }

        Ok(wasm_path)
    }

    /// Run wasm-bindgen to generate JS bindings.
    pub fn wasm_bindgen(&self, wasm_path: &Path) -> Result<(), BuildError> {
        let bindgen_out = self.output_dir.join("pkg");
        fs::create_dir_all(&bindgen_out).map_err(|e| {
            BuildError::IoError(format!("Failed to create bindgen output dir: {}", e))
        })?;

        let mut cmd = Command::new("wasm-bindgen");
        cmd.arg(wasm_path)
            .arg("--out-dir")
            .arg(&bindgen_out)
            .arg("--target")
            .arg("web");

        let output = cmd.output().map_err(|e| {
            BuildError::CompileError(format!("Failed to run wasm-bindgen: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::CompileError(format!(
                "wasm-bindgen failed:\n{}",
                stderr
            )));
        }

        Ok(())
    }

    /// Package the build output into a SpiritPackage.
    pub fn package(&self) -> Result<SpiritPackage, BuildError> {
        // Locate the WASM binary
        let wasm_name = format!("{}.wasm", self.manifest.name.replace('-', "_"));
        let wasm_path = self
            .output_dir
            .join("target")
            .join(&self.config.wasm_target)
            .join("release")
            .join(&wasm_name);

        if !wasm_path.exists() {
            return Err(BuildError::CompileError(format!(
                "WASM binary not found: {}",
                wasm_path.display()
            )));
        }

        // Check for wasm-bindgen outputs
        let pkg_dir = self.output_dir.join("pkg");
        let js_bindings = pkg_dir.join(format!("{}.js", self.manifest.name.replace('-', "_")));
        let ts_definitions = pkg_dir.join(format!("{}.d.ts", self.manifest.name.replace('-', "_")));

        Ok(SpiritPackage {
            name: self.manifest.name.clone(),
            version: self.manifest.version.clone(),
            wasm_path,
            js_bindings: if js_bindings.exists() {
                Some(js_bindings)
            } else {
                None
            },
            ts_definitions: if ts_definitions.exists() {
                Some(ts_definitions)
            } else {
                None
            },
            artifacts_dir: self.output_dir.clone(),
        })
    }

    /// Run the complete build pipeline.
    ///
    /// This is a convenience method that executes all stages in sequence:
    /// 1. Resolve modules
    /// 2. Generate Rust code
    /// 3. Generate Cargo.toml
    /// 4. Compile to WASM
    /// 5. Run wasm-bindgen
    /// 6. Package
    pub fn build(&self) -> Result<SpiritPackage, BuildError> {
        // Stage 1: Resolve modules
        let modules = self.resolve_modules()?;

        // Stage 2: Generate Rust code
        self.codegen_rust(&modules)?;

        // Stage 3: Generate Cargo.toml
        self.generate_cargo_toml()?;

        // Stage 4: Compile to WASM
        let wasm_path = self.cargo_build()?;

        // Stage 5: Run wasm-bindgen
        self.wasm_bindgen(&wasm_path)?;

        // Stage 6: Package
        self.package()
    }
}

/// Errors that can occur during the build process.
#[derive(Debug)]
pub enum BuildError {
    /// I/O error
    IoError(String),
    /// Parse error
    ParseError(String),
    /// Module not found
    ModuleNotFound(String),
    /// Circular dependency
    CircularDependency(String),
    /// Invalid path
    InvalidPath(String),
    /// Compilation error
    CompileError(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::IoError(msg) => write!(f, "I/O error: {}", msg),
            BuildError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BuildError::ModuleNotFound(msg) => write!(f, "Module not found: {}", msg),
            BuildError::CircularDependency(msg) => write!(f, "Circular dependency: {}", msg),
            BuildError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            BuildError::CompileError(msg) => write!(f, "Compilation error: {}", msg),
        }
    }
}

impl std::error::Error for BuildError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let module = Module {
            name: "test".to_string(),
            full_path: "test.module".to_string(),
            file_path: PathBuf::from("test.dol"),
            content: DolFile {
                module: None,
                uses: Vec::new(),
                declarations: Vec::new(),
            },
            dependencies: Vec::new(),
        };

        assert_eq!(module.name, "test");
        assert_eq!(module.dependencies.len(), 0);
    }

    #[test]
    fn test_spirit_package_creation() {
        let package = SpiritPackage {
            name: "test-spirit".to_string(),
            version: Version {
                major: 1,
                minor: 0,
                patch: 0,
                suffix: None,
            },
            wasm_path: PathBuf::from("test.wasm"),
            js_bindings: None,
            ts_definitions: None,
            artifacts_dir: PathBuf::from("./artifacts"),
        };

        assert_eq!(package.name, "test-spirit");
        assert!(package.js_bindings.is_none());
    }
}
