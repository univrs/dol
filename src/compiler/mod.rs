//! Spirit Compiler Module
//!
//! Provides end-to-end compilation from DOL source to WebAssembly.
//!
//! # Overview
//!
//! The Spirit compiler orchestrates the complete compilation pipeline:
//!
//! ```text
//! DOL Source → Lexer → Parser → AST → HIR → MLIR → WASM
//! ```
//!
//! # Architecture
//!
//! The compilation process is divided into distinct phases:
//!
//! 1. **Parsing**: Transform DOL source text into an Abstract Syntax Tree (AST)
//! 2. **Lowering**: Convert AST to High-level Intermediate Representation (HIR)
//! 3. **MLIR Generation**: Lower HIR to MLIR dialect operations
//! 4. **WASM Emission**: Generate WebAssembly bytecode from MLIR
//!
//! # Feature Flags
//!
//! This module requires the `wasm` feature flag:
//!
//! ```toml
//! [dependencies]
//! metadol = { version = "0.5.0", features = ["wasm"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::compiler::spirit::{compile_source, compile_file};
//! use std::path::Path;
//!
//! // Compile from source string
//! let source = r#"
//!     module example @ 1.0
//!
//!     fun add(a: Int64, b: Int64) -> Int64 {
//!         return a + b
//!     }
//! "#;
//!
//! let result = compile_source(source, "example.dol")?;
//! println!("Compiled {} bytes of WASM", result.wasm.len());
//!
//! // Compile from file
//! let spirit = compile_file(Path::new("src/main.dol"))?;
//! std::fs::write("output.wasm", &spirit.wasm)?;
//! ```
//!
//! # Spirit Projects
//!
//! A Spirit project is a directory containing:
//! - `manifest.toml`: Project metadata and dependencies
//! - `src/main.dol`: Entry point for the Spirit
//! - `src/*.dol`: Additional source files
//!
//! Use `compile_spirit_project()` to compile an entire project:
//!
//! ```rust,ignore
//! use metadol::compiler::spirit::compile_spirit_project;
//! use std::path::Path;
//!
//! let project_dir = Path::new("my-spirit");
//! let compiled = compile_spirit_project(project_dir)?;
//! ```

// Spirit compiler requires both wasm-compile (for bytecode emission) and lower/hir modules
#[cfg(feature = "wasm-compile")]
pub mod spirit;

#[cfg(feature = "wasm-compile")]
pub use spirit::{
    compile_file, compile_source, compile_spirit_project, CompiledSpirit, CompilerError,
    CompilerWarning, SourceMap, SourceMapEntry,
};
