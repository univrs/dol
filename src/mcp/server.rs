//! MCP Server implementation for Metal DOL.
//!
//! This module implements the Model Context Protocol server that exposes
//! Metal DOL's capabilities as callable tools.

use super::DolTool;
use crate::{
    ast::Expr,
    codegen::{RustCodegen, TypeScriptCodegen},
    eval::Interpreter,
    macros::BuiltinMacros,
    parse_file,
    reflect::TypeRegistry,
    typechecker::TypeChecker,
};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// MCP Server for Metal DOL.
///
/// Provides a Model Context Protocol interface to DOL's parsing,
/// type checking, code generation, and evaluation capabilities.
pub struct McpServer {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

impl McpServer {
    /// Creates a new MCP server.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::mcp::McpServer;
    ///
    /// let server = McpServer::new();
    /// assert_eq!(server.name, "metadol-mcp");
    /// ```
    pub fn new() -> Self {
        Self {
            name: "metadol-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Handles a tool invocation.
    ///
    /// Dispatches to the appropriate tool handler based on the tool type.
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to invoke
    /// * `args` - Tool arguments as a HashMap
    ///
    /// # Returns
    ///
    /// A `ToolResult` containing the tool's output or an error message.
    pub fn handle_tool(&self, tool: DolTool, args: ToolArgs) -> Result<ToolResult, String> {
        match tool {
            DolTool::Parse => self.tool_parse(args),
            DolTool::TypeCheck => self.tool_typecheck(args),
            DolTool::CompileRust => self.tool_compile_rust(args),
            DolTool::CompileTypeScript => self.tool_compile_typescript(args),
            DolTool::CompileWasm => self.tool_compile_wasm(args),
            DolTool::Eval => self.tool_eval(args),
            DolTool::Reflect => self.tool_reflect(args),
            DolTool::Format => self.tool_format(args),
            DolTool::ListMacros => self.tool_list_macros(args),
            DolTool::ExpandMacro => self.tool_expand_macro(args),
        }
    }

    fn tool_parse(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let source = args.get_string("source")?;

        match parse_file(&source) {
            Ok(decl) => {
                #[cfg(feature = "serde")]
                {
                    let json = serde_json::to_string_pretty(&decl)
                        .map_err(|e| format!("Failed to serialize AST: {}", e))?;
                    Ok(ToolResult::json(json))
                }
                #[cfg(not(feature = "serde"))]
                {
                    Ok(ToolResult::text(format!("{:#?}", decl)))
                }
            }
            Err(e) => Err(format!("Parse error: {}", e)),
        }
    }

    fn tool_typecheck(&self, args: ToolArgs) -> Result<ToolResult, String> {
        #[cfg(feature = "serde")]
        {
            let expr_json = args.get_string("expr")?;
            let expr: Expr = serde_json::from_str(&expr_json)
                .map_err(|e| format!("Failed to parse expression JSON: {}", e))?;

            let mut checker = TypeChecker::new();
            match checker.infer(&expr) {
                Ok(ty) => Ok(ToolResult::text(format!("{:?}", ty))),
                Err(e) => Err(format!("Type error: {}", e)),
            }
        }
        #[cfg(not(feature = "serde"))]
        {
            Err("Type checking requires the 'serde' feature".to_string())
        }
    }

    fn tool_compile_rust(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let source = args.get_string("source")?;

        match parse_file(&source) {
            Ok(decl) => {
                let rust_code = RustCodegen::generate(&decl);
                Ok(ToolResult::text(rust_code))
            }
            Err(e) => Err(format!("Parse error: {}", e)),
        }
    }

    fn tool_compile_typescript(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let source = args.get_string("source")?;

        match parse_file(&source) {
            Ok(decl) => {
                let ts_code = TypeScriptCodegen::generate(&decl);
                Ok(ToolResult::text(ts_code))
            }
            Err(e) => Err(format!("Parse error: {}", e)),
        }
    }

    fn tool_compile_wasm(&self, _args: ToolArgs) -> Result<ToolResult, String> {
        Err("WebAssembly compilation is not yet implemented".to_string())
    }

    fn tool_eval(&self, args: ToolArgs) -> Result<ToolResult, String> {
        #[cfg(feature = "serde")]
        {
            let expr_json = args.get_string("expr")?;
            let expr: Expr = serde_json::from_str(&expr_json)
                .map_err(|e| format!("Failed to parse expression JSON: {}", e))?;

            let mut interpreter = Interpreter::new();
            match interpreter.eval(&expr) {
                Ok(value) => Ok(ToolResult::text(format!("{:?}", value))),
                Err(e) => Err(format!("Evaluation error: {}", e)),
            }
        }
        #[cfg(not(feature = "serde"))]
        {
            Err("Evaluation requires the 'serde' feature".to_string())
        }
    }

    fn tool_reflect(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let type_name = args.get_string("type_name")?;

        let registry = TypeRegistry::new();
        match registry.lookup(&type_name) {
            Some(type_info) => Ok(ToolResult::text(format!("{:#?}", type_info))),
            None => Err(format!("Type '{}' not found in registry", type_name)),
        }
    }

    fn tool_format(&self, _args: ToolArgs) -> Result<ToolResult, String> {
        Err("DOL source formatting is not yet implemented".to_string())
    }

    fn tool_list_macros(&self, _args: ToolArgs) -> Result<ToolResult, String> {
        let builtins = BuiltinMacros::new();
        let macro_names: Vec<&str> = builtins.names().collect();

        let mut output = String::from("Available macros:\n");
        for name in macro_names {
            output.push_str(&format!("  - #{}\n", name));
        }

        Ok(ToolResult::text(output))
    }

    fn tool_expand_macro(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let macro_name = args.get_string("macro_name")?;

        // For now, just return information about the macro
        let builtins = BuiltinMacros::new();
        if builtins.get(&macro_name).is_some() {
            Ok(ToolResult::text(format!(
                "Macro #{} is available. Full expansion requires macro context.",
                macro_name
            )))
        } else {
            Err(format!("Macro '{}' not found", macro_name))
        }
    }

    /// Returns the server manifest describing available tools.
    ///
    /// The manifest includes metadata about each tool, including
    /// its name, description, and parameter schema.
    pub fn manifest(&self) -> ServerManifest {
        ServerManifest {
            name: self.name.clone(),
            version: self.version.clone(),
            tools: vec![
                ToolDef {
                    name: "parse".to_string(),
                    description: "Parse DOL source code into an AST".to_string(),
                    parameters: vec![ParamDef {
                        name: "source".to_string(),
                        description: "DOL source code to parse".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "typecheck".to_string(),
                    description: "Type check a DOL expression".to_string(),
                    parameters: vec![ParamDef {
                        name: "expr".to_string(),
                        description: "DOL expression to type check (JSON)".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "compile_rust".to_string(),
                    description: "Generate Rust code from DOL declarations".to_string(),
                    parameters: vec![ParamDef {
                        name: "source".to_string(),
                        description: "DOL source code to compile".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "compile_typescript".to_string(),
                    description: "Generate TypeScript code from DOL declarations".to_string(),
                    parameters: vec![ParamDef {
                        name: "source".to_string(),
                        description: "DOL source code to compile".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "compile_wasm".to_string(),
                    description: "Compile DOL to WebAssembly (future feature)".to_string(),
                    parameters: vec![ParamDef {
                        name: "source".to_string(),
                        description: "DOL source code to compile".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "eval".to_string(),
                    description: "Evaluate a DOL expression".to_string(),
                    parameters: vec![ParamDef {
                        name: "expr".to_string(),
                        description: "DOL expression to evaluate (JSON)".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "reflect".to_string(),
                    description: "Get runtime type information for a DOL type".to_string(),
                    parameters: vec![ParamDef {
                        name: "type_name".to_string(),
                        description: "Name of the type to reflect on".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "format".to_string(),
                    description: "Format DOL source code (future feature)".to_string(),
                    parameters: vec![ParamDef {
                        name: "source".to_string(),
                        description: "DOL source code to format".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "list_macros".to_string(),
                    description: "List all available macros".to_string(),
                    parameters: vec![],
                },
                ToolDef {
                    name: "expand_macro".to_string(),
                    description: "Expand a macro invocation".to_string(),
                    parameters: vec![
                        ParamDef {
                            name: "macro_name".to_string(),
                            description: "Name of the macro to expand".to_string(),
                            required: true,
                        },
                        ParamDef {
                            name: "args".to_string(),
                            description: "Macro arguments (JSON)".to_string(),
                            required: false,
                        },
                    ],
                },
            ],
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool arguments wrapper.
///
/// Wraps a HashMap of arguments and provides typed access methods.
pub struct ToolArgs {
    args: HashMap<String, serde_json::Value>,
}

impl ToolArgs {
    /// Creates a new ToolArgs from a HashMap.
    pub fn new(args: HashMap<String, serde_json::Value>) -> Self {
        Self { args }
    }

    /// Gets a string argument by name.
    ///
    /// Returns an error if the argument is missing or not a string.
    pub fn get_string(&self, key: &str) -> Result<String, String> {
        self.args
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Missing or invalid argument: {}", key))
    }

    /// Gets an optional string argument by name.
    pub fn get_optional_string(&self, key: &str) -> Option<String> {
        self.args
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Gets the raw JSON value for an argument.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.args.get(key)
    }
}

/// Tool execution result.
///
/// Contains the output of a tool invocation, including
/// content type and the actual content.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ToolResult {
    /// Content type (e.g., "text/plain", "application/json")
    pub content_type: String,
    /// Content string
    pub content: String,
}

impl ToolResult {
    /// Creates a plain text result.
    pub fn text(content: String) -> Self {
        Self {
            content_type: "text/plain".to_string(),
            content,
        }
    }

    /// Creates a JSON result.
    pub fn json(content: String) -> Self {
        Self {
            content_type: "application/json".to_string(),
            content,
        }
    }
}

/// Server manifest describing available tools.
///
/// The manifest is returned by the server to inform clients
/// about available capabilities.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ServerManifest {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Available tools
    pub tools: Vec<ToolDef>,
}

/// Tool definition in the manifest.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ToolDef {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool parameters
    pub parameters: Vec<ParamDef>,
}

/// Parameter definition for a tool.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParamDef {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Whether the parameter is required
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = McpServer::new();
        assert_eq!(server.name, "metadol-mcp");
        assert!(!server.version.is_empty());
    }

    #[test]
    fn test_manifest() {
        let server = McpServer::new();
        let manifest = server.manifest();

        assert_eq!(manifest.name, "metadol-mcp");
        assert!(!manifest.tools.is_empty());

        // Check that parse tool exists
        assert!(manifest.tools.iter().any(|t| t.name == "parse"));
    }

    #[test]
    fn test_list_macros() {
        let server = McpServer::new();
        let args = ToolArgs::new(HashMap::new());

        let result = server.tool_list_macros(args);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.content.contains("Available macros"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_parse_tool() {
        let server = McpServer::new();
        let mut args_map = HashMap::new();
        args_map.insert(
            "source".to_string(),
            serde_json::Value::String(
                r#"gene test.example {
  test has id
}

exegesis {
  Test gene.
}"#
                .to_string(),
            ),
        );
        let args = ToolArgs::new(args_map);

        let result = server.tool_parse(args);
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Parse should succeed");
    }
}
