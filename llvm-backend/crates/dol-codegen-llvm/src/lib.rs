//! DOL LLVM Code Generation Backend
//!
//! This crate provides LLVM IR generation for DOL (Design Ontology Language),
//! enabling compilation to native machine code for ARM64, RISC-V, and x86-64.
//!
//! # Architecture
//!
//! ```text
//! DOL HIR → LLVM IR → Native Object → Linked Binary
//! ```
//!
//! # Supported Targets
//!
//! - `aarch64-unknown-linux-gnu` (ARM64 Linux)
//! - `aarch64-apple-darwin` (ARM64 macOS)
//! - `riscv64gc-unknown-linux-gnu` (RISC-V 64-bit)
//! - `x86_64-unknown-linux-gnu` (x86-64 Linux)
//! - `x86_64-pc-windows-msvc` (x86-64 Windows)

pub mod abi;
pub mod functions;
pub mod hir_lowering;
pub mod structs;
pub mod targets;
pub mod types;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{Target, TargetMachine, TargetTriple};
use thiserror::Error;

/// Errors that can occur during LLVM code generation
#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("Unsupported target: {0}")]
    UnsupportedTarget(String),

    #[error("LLVM error: {0}")]
    LlvmError(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Link error: {0}")]
    LinkError(String),
}

/// Result type for codegen operations
pub type Result<T> = std::result::Result<T, CodegenError>;

/// LLVM code generator for DOL
pub struct LlvmCodegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    target_machine: TargetMachine,
}

impl<'ctx> LlvmCodegen<'ctx> {
    /// Create a new LLVM code generator for the specified target
    pub fn new(context: &'ctx Context, module_name: &str, target_triple: &str) -> Result<Self> {
        // Initialize LLVM targets
        Target::initialize_all(&inkwell::targets::InitializationConfig::default());

        let triple = TargetTriple::create(target_triple);
        let target = Target::from_triple(&triple)
            .map_err(|e| CodegenError::UnsupportedTarget(e.to_string()))?;

        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Default,
                inkwell::targets::RelocMode::PIC,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| CodegenError::UnsupportedTarget(target_triple.to_string()))?;

        let module = context.create_module(module_name);
        module.set_triple(&triple);

        Ok(Self {
            context,
            module,
            target_machine,
        })
    }

    /// Get the LLVM context
    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    /// Get the LLVM module
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Emit object code to a file
    pub fn emit_object(&self, path: &std::path::Path) -> Result<()> {
        self.target_machine
            .write_to_file(&self.module, inkwell::targets::FileType::Object, path)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Emit LLVM IR to a string (for debugging)
    pub fn emit_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_codegen() {
        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test", "x86_64-unknown-linux-gnu");
        assert!(codegen.is_ok());
    }
}
