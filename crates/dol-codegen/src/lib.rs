//! DOL Template-Based Code Generation Framework
//!
//! This crate provides a flexible template-based code generation framework for
//! transforming DOL declarations into multiple target languages and formats.
//!
//! # Features
//!
//! - **Template Engine**: Handlebars and Tera template support
//! - **AST Transformations**: Visitor pattern for custom AST transformations
//! - **Multi-Target Generation**: Rust, TypeScript, WIT, Python, JSON Schema
//! - **Type Inference**: Automatic type inference and elaboration
//! - **CRDT Expansion**: Automatic expansion of CRDT annotations
//!
//! # Example
//!
//! ```rust,ignore
//! use dol_codegen::{generate, CodegenContext, Target};
//! use dol::ast::DolFile;
//!
//! // Parse or construct a DOL file
//! let file = DolFile {
//!     module: None,
//!     uses: vec![],
//!     declarations: vec![],
//! };
//!
//! // Create a codegen context
//! let context = CodegenContext::new(Target::Rust)
//!     .with_docs(true);
//!
//! // Generate code
//! let code = generate(&file, &context).unwrap();
//! ```

pub mod template_engine;
pub mod transforms;
pub mod targets;

use dol::ast::DolFile;
use thiserror::Error;

/// Code generation errors
#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Template error: {0}")]
    Template(String),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("Type inference error: {0}")]
    TypeInference(String),

    #[error("Unsupported target: {0}")]
    UnsupportedTarget(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Handlebars error: {0}")]
    Handlebars(String),

    #[error("Tera error: {0}")]
    Tera(String),
}

/// Code generation result type
pub type Result<T> = std::result::Result<T, CodegenError>;

/// Code generation target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Rust code generation
    Rust,
    /// TypeScript code generation
    TypeScript,
    /// WebAssembly Interface Types
    Wit,
    /// Python code generation
    Python,
    /// JSON Schema
    JsonSchema,
}

impl Target {
    /// Returns the file extension for the target
    pub fn extension(&self) -> &'static str {
        match self {
            Target::Rust => "rs",
            Target::TypeScript => "ts",
            Target::Wit => "wit",
            Target::Python => "py",
            Target::JsonSchema => "json",
        }
    }

    /// Returns the language name for the target
    pub fn language(&self) -> &'static str {
        match self {
            Target::Rust => "Rust",
            Target::TypeScript => "TypeScript",
            Target::Wit => "WIT",
            Target::Python => "Python",
            Target::JsonSchema => "JSON Schema",
        }
    }
}

/// Code generation context
#[derive(Debug, Clone)]
pub struct CodegenContext {
    /// Target platform
    pub target: Target,

    /// Include documentation comments
    pub include_docs: bool,

    /// Generate builder pattern methods
    pub generate_builders: bool,

    /// Generate serde support (Rust)
    pub derive_serde: bool,

    /// Custom module name
    pub module_name: Option<String>,

    /// Template directory path
    pub template_dir: Option<String>,

    /// Enable type inference
    pub enable_type_inference: bool,

    /// Enable CRDT expansion
    pub enable_crdt_expansion: bool,
}

impl Default for CodegenContext {
    fn default() -> Self {
        Self {
            target: Target::Rust,
            include_docs: true,
            generate_builders: false,
            derive_serde: true,
            module_name: None,
            template_dir: None,
            enable_type_inference: true,
            enable_crdt_expansion: true,
        }
    }
}

impl CodegenContext {
    /// Create a new codegen context for the given target
    pub fn new(target: Target) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    /// Set the module name
    pub fn with_module_name(mut self, name: impl Into<String>) -> Self {
        self.module_name = Some(name.into());
        self
    }

    /// Set whether to include documentation
    pub fn with_docs(mut self, include: bool) -> Self {
        self.include_docs = include;
        self
    }

    /// Set whether to generate builders
    pub fn with_builders(mut self, generate: bool) -> Self {
        self.generate_builders = generate;
        self
    }

    /// Set the template directory
    pub fn with_template_dir(mut self, dir: impl Into<String>) -> Self {
        self.template_dir = Some(dir.into());
        self
    }
}

/// Main code generation function
pub fn generate(file: &DolFile, context: &CodegenContext) -> Result<String> {
    // Apply AST transformations
    let transformed = transforms::transform(file, context)?;

    // Generate code using the template engine
    template_engine::generate(&transformed, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_extension() {
        assert_eq!(Target::Rust.extension(), "rs");
        assert_eq!(Target::TypeScript.extension(), "ts");
        assert_eq!(Target::Wit.extension(), "wit");
        assert_eq!(Target::Python.extension(), "py");
        assert_eq!(Target::JsonSchema.extension(), "json");
    }

    #[test]
    fn test_context_builder() {
        let ctx = CodegenContext::new(Target::TypeScript)
            .with_module_name("test_module")
            .with_docs(false)
            .with_builders(true);

        assert_eq!(ctx.target, Target::TypeScript);
        assert_eq!(ctx.module_name, Some("test_module".to_string()));
        assert!(!ctx.include_docs);
        assert!(ctx.generate_builders);
    }
}
