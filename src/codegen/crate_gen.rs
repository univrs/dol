//! Crate-level code generation for DOL.
//!
//! Generates a complete Rust crate from multiple DOL files,
//! with one .rs file per DOL module.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::ast::{DolFile, UseDecl, UseItems};
use crate::codegen::rust::RustCodegen;

/// Configuration for crate generation.
#[derive(Debug, Clone)]
pub struct CrateConfig {
    /// Name of the generated crate
    pub crate_name: String,
    /// Version of the generated crate
    pub crate_version: String,
    /// Output directory for the crate
    pub output_dir: String,
}

impl Default for CrateConfig {
    fn default() -> Self {
        Self {
            crate_name: "dol_generated".to_string(),
            crate_version: "0.1.0".to_string(),
            output_dir: "stage2".to_string(),
        }
    }
}

/// Crate code generator.
///
/// Generates a complete Rust crate from multiple DOL files.
pub struct CrateCodegen {
    config: CrateConfig,
}

/// Information about a DOL module for code generation.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Module name (last component of path, e.g., "lexer" from "dol.lexer")
    pub name: String,
    /// Full module path (e.g., "dol.lexer")
    pub full_path: String,
    /// Source file path
    pub source_path: String,
    /// Dependencies (other modules this one imports)
    pub dependencies: Vec<String>,
}

impl CrateCodegen {
    /// Create a new crate generator with default configuration.
    pub fn new(output_dir: &str) -> Self {
        Self {
            config: CrateConfig {
                output_dir: output_dir.to_string(),
                ..Default::default()
            },
        }
    }

    /// Create a new crate generator with custom configuration.
    pub fn with_config(config: CrateConfig) -> Self {
        Self { config }
    }

    /// Generate a complete Rust crate from DOL files.
    ///
    /// # Arguments
    ///
    /// * `files` - Parsed DOL files to generate from
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error message on failure.
    pub fn generate(&self, files: &[(String, DolFile)]) -> Result<(), String> {
        // Create output directory structure
        let src_dir = format!("{}/src", self.config.output_dir);
        fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {}", e))?;

        // Analyze modules to get dependency order
        let modules: Vec<ModuleInfo> = files
            .iter()
            .map(|(path, file)| self.analyze_module(path, file))
            .collect();

        // Generate each module file
        for ((_source_path, file), module) in files.iter().zip(modules.iter()) {
            self.gen_module_file(file, module, &modules)?;
        }

        // Generate lib.rs
        self.gen_lib_rs(&modules)?;

        // Generate prelude.rs
        self.gen_prelude_rs(&modules)?;

        // Generate Cargo.toml
        self.gen_cargo_toml()?;

        Ok(())
    }

    /// Analyze a DOL file to extract module information.
    fn analyze_module(&self, source_path: &str, file: &DolFile) -> ModuleInfo {
        let name = file
            .module
            .as_ref()
            .and_then(|m| m.path.last().cloned())
            .unwrap_or_else(|| {
                // Extract from filename
                Path::new(source_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

        let full_path = file
            .module
            .as_ref()
            .map(|m| m.path.join("."))
            .unwrap_or_else(|| name.clone());

        // Extract dependencies from use declarations
        let dependencies: Vec<String> = file
            .uses
            .iter()
            .filter_map(|u| {
                // Get the first part of the path if it starts with "dol"
                if u.path.first().map(|s| s.as_str()) == Some("dol") {
                    u.path.get(1).cloned()
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        ModuleInfo {
            name,
            full_path,
            source_path: source_path.to_string(),
            dependencies,
        }
    }

    /// Generate a single module file.
    fn gen_module_file(
        &self,
        file: &DolFile,
        module: &ModuleInfo,
        all_modules: &[ModuleInfo],
    ) -> Result<(), String> {
        let mut content = String::new();

        // Module doc comment
        content.push_str(&format!("//! Module: {}\n", module.full_path));
        content.push_str("//! Generated from DOL source - do not edit\n\n");

        // Generate imports based on use declarations
        content.push_str(&self.gen_imports(file));

        // Add imports from other DOL modules to resolve cross-module references
        // This handles cases where types are used but not explicitly imported in DOL
        content.push_str(&self.gen_sibling_imports(module, all_modules));
        content.push('\n');

        // Generate declarations using existing RustCodegen
        let rust_gen = RustCodegen::new();
        content.push_str(&rust_gen.gen_file(&file.declarations));

        // Write file
        let file_path = format!("{}/src/{}.rs", self.config.output_dir, module.name);
        fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;

        Ok(())
    }

    /// Generate import statements from DOL use declarations.
    fn gen_imports(&self, file: &DolFile) -> String {
        let mut output = String::new();
        let mut seen = HashSet::new();

        for use_decl in &file.uses {
            let rust_import = self.resolve_import(use_decl);
            if !rust_import.is_empty() && !seen.contains(&rust_import) {
                output.push_str(&rust_import);
                output.push('\n');
                seen.insert(rust_import);
            }
        }

        output
    }

    /// Convert a DOL use declaration to a Rust use statement.
    fn resolve_import(&self, use_decl: &UseDecl) -> String {
        let path = use_decl.path.join("::");

        // dol.X -> crate::X
        if path.starts_with("dol::") {
            let internal = path.replacen("dol::", "crate::", 1);
            match &use_decl.items {
                UseItems::All => format!("use {}::*;", internal),
                UseItems::Single => format!("use {};", internal),
                UseItems::Named(items) => {
                    let names: Vec<&str> = items.iter().map(|i| i.name.as_str()).collect();
                    format!("use {}::{{ {} }};", internal, names.join(", "))
                }
            }
        }
        // std.X -> std::X
        else if path.starts_with("std::") {
            format!("use {};", path)
        }
        // Other external crates
        else {
            format!("use {};", path)
        }
    }

    /// Generate imports from sibling modules to handle cross-module type references.
    /// This adds `use crate::sibling::*;` for each other module.
    fn gen_sibling_imports(&self, current: &ModuleInfo, all_modules: &[ModuleInfo]) -> String {
        let mut output = String::new();
        output.push_str("#[allow(unused_imports)]\n");

        // Always import compatibility types first
        output.push_str("use crate::compat::*;\n");

        for module in all_modules {
            // Skip self, main, and mod (root)
            if module.name == current.name
                || module.name == "main"
                || module.name == "mod"
                || module.name == "dol"
            {
                continue;
            }
            output.push_str(&format!("use crate::{}::*;\n", module.name));
        }

        output
    }

    /// Generate lib.rs with mod declarations.
    fn gen_lib_rs(&self, modules: &[ModuleInfo]) -> Result<(), String> {
        let mut content = String::from("//! DOL Generated Crate\n");
        content.push_str("//! Do not edit manually - regenerate from DOL source\n\n");

        // Add standard library imports
        content.push_str("use std::collections::HashMap;\n\n");

        // Compatibility module must come first
        content.push_str("pub mod compat;\n\n");

        // Module declarations
        for module in modules {
            if module.name == "main" {
                continue; // main.rs is separate, not a module
            }
            if module.name == "mod" {
                continue; // skip mod.dol as it's the root
            }
            content.push_str(&format!("pub mod {};\n", module.name));
        }

        content.push_str("\npub mod prelude;\n");

        let path = format!("{}/src/lib.rs", self.config.output_dir);
        fs::write(&path, content).map_err(|e| format!("Failed to write lib.rs: {}", e))?;

        // Generate compat.rs
        self.gen_compat_rs()?;

        Ok(())
    }

    /// Generate compat.rs with compatibility types for DOL sources.
    fn gen_compat_rs(&self) -> Result<(), String> {
        let content = r#"//! Compatibility layer for types expected by DOL sources
//! These are stub/alias types that bridge the gap between DOL source expectations
//! and the generated AST structure.

// Type aliases for renamed types
pub type BinaryOp = super::ast::BinOp;
pub type Statement = super::ast::Stmt;
pub type TypeParams = Vec<super::ast::TypeParam>;

// Placeholder - Literal is used as a parameter type but doesn't exist as separate enum
pub type Literal = super::ast::Expr;

// Stub types for missing definitions
#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub items: UseItems,
    pub alias: Option<String>,
    pub span: super::token::Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseItems {
    All,
    Single,
    Named(Vec<UseItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseItem {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HasField {
    pub name: String,
    pub ty: super::ast::TypeExpr,
    pub default: Option<super::ast::Expr>,
    pub span: super::token::Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvolutionDecl {
    pub name: String,
    pub span: super::token::Span,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Purity {
    Pure,
    Impure,
    Sex,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    Has(String, String, super::ast::TypeExpr, super::token::Span),
    Is(String, String, super::token::Span),
    Requires(super::ast::Expr, super::token::Span),
    Ensures(super::ast::Expr, super::token::Span),
    Invariant(super::ast::Expr, super::token::Span),
    Law(String, super::ast::Expr, super::token::Span),
}
"#;
        let path = format!("{}/src/compat.rs", self.config.output_dir);
        fs::write(&path, content).map_err(|e| format!("Failed to write compat.rs: {}", e))?;

        Ok(())
    }

    /// Generate prelude.rs with re-exports.
    fn gen_prelude_rs(&self, modules: &[ModuleInfo]) -> Result<(), String> {
        let mut content = String::from("//! Prelude - common re-exports\n\n");

        // Re-export compat types
        content.push_str("pub use crate::compat::*;\n\n");

        for module in modules {
            if module.name == "main" || module.name == "mod" {
                continue;
            }
            content.push_str(&format!("pub use crate::{}::*;\n", module.name));
        }

        let path = format!("{}/src/prelude.rs", self.config.output_dir);
        fs::write(&path, content).map_err(|e| format!("Failed to write prelude.rs: {}", e))?;

        Ok(())
    }

    /// Generate Cargo.toml.
    fn gen_cargo_toml(&self) -> Result<(), String> {
        let content = format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "2021"

[dependencies]
"#,
            self.config.crate_name, self.config.crate_version
        );

        let path = format!("{}/Cargo.toml", self.config.output_dir);
        fs::write(&path, content).map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CrateConfig::default();
        assert_eq!(config.crate_name, "dol_generated");
        assert_eq!(config.crate_version, "0.1.0");
    }

    #[test]
    fn test_dol_file_default() {
        let file = DolFile {
            module: None,
            uses: Vec::new(),
            declarations: Vec::new(),
        };
        assert!(file.module.is_none());
        assert!(file.uses.is_empty());
        assert!(file.declarations.is_empty());
    }

    #[test]
    fn test_crate_codegen_new() {
        let codegen = CrateCodegen::new("test_output");
        assert_eq!(codegen.config.output_dir, "test_output");
        assert_eq!(codegen.config.crate_name, "dol_generated");
    }
}
