//! MCP (Model Context Protocol) Server for Metal DOL.
//!
//! This module provides an MCP server that exposes Metal DOL's capabilities
//! as tools that can be invoked by AI assistants and other MCP clients.
//!
//! # Overview
//!
//! The MCP server enables Metal DOL to be used as a language service through
//! the Model Context Protocol. It provides tools for parsing, type checking,
//! code generation, evaluation, and reflection on DOL source code.
//!
//! # Available Tools
//!
//! - **parse**: Parse DOL source code into an AST
//! - **typecheck**: Type check DOL expressions and validate types
//! - **compile_rust**: Generate Rust code from DOL declarations
//! - **compile_typescript**: Generate TypeScript code from DOL declarations
//! - **compile_wasm**: Compile DOL to WebAssembly (future)
//! - **eval**: Evaluate DOL expressions at runtime
//! - **reflect**: Get runtime type information for DOL types
//! - **format**: Format DOL source code (future)
//! - **list_macros**: List all available macros
//! - **expand_macro**: Expand a specific macro invocation
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::mcp::{McpServer, DolTool};
//!
//! let server = McpServer::new();
//! let manifest = server.manifest();
//!
//! // Handle a tool call
//! let result = server.handle_tool(DolTool::Parse, args)?;
//! ```
//!
//! # MCP Integration
//!
//! The server implements the Model Context Protocol specification,
//! allowing DOL to be used as a tool by AI assistants like Claude.

pub mod server;
pub mod tools;

pub use server::{McpServer, ParamDef, ServerManifest, ToolArgs, ToolDef, ToolResult};

/// DOL tools available through the MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DolTool {
    /// Parse DOL source code into an AST
    Parse,
    /// Type check DOL expressions
    TypeCheck,
    /// Compile to Rust
    CompileRust,
    /// Compile to TypeScript
    CompileTypeScript,
    /// Compile to WebAssembly
    CompileWasm,
    /// Evaluate DOL expressions
    Eval,
    /// Get runtime type information
    Reflect,
    /// Format DOL source code
    Format,
    /// List available macros
    ListMacros,
    /// Expand a macro
    ExpandMacro,
}

impl DolTool {
    /// Get the tool name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            DolTool::Parse => "parse",
            DolTool::TypeCheck => "typecheck",
            DolTool::CompileRust => "compile_rust",
            DolTool::CompileTypeScript => "compile_typescript",
            DolTool::CompileWasm => "compile_wasm",
            DolTool::Eval => "eval",
            DolTool::Reflect => "reflect",
            DolTool::Format => "format",
            DolTool::ListMacros => "list_macros",
            DolTool::ExpandMacro => "expand_macro",
        }
    }

    /// Parse a tool name from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "parse" => Some(DolTool::Parse),
            "typecheck" => Some(DolTool::TypeCheck),
            "compile_rust" => Some(DolTool::CompileRust),
            "compile_typescript" => Some(DolTool::CompileTypeScript),
            "compile_wasm" => Some(DolTool::CompileWasm),
            "eval" => Some(DolTool::Eval),
            "reflect" => Some(DolTool::Reflect),
            "format" => Some(DolTool::Format),
            "list_macros" => Some(DolTool::ListMacros),
            "expand_macro" => Some(DolTool::ExpandMacro),
            _ => None,
        }
    }
}
