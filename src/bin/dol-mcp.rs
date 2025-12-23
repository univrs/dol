//! DOL MCP Server
//!
//! Run the Model Context Protocol server for AI integration with Metal DOL.
//!
//! # Usage
//!
//! ```bash
//! # Start MCP server (listens on stdio)
//! dol-mcp serve
//!
//! # Print server manifest
//! dol-mcp manifest
//!
//! # Execute a specific tool directly
//! dol-mcp tool parse source="gene x { x has y }"
//! echo 'gene x { x has y }' | dol-mcp tool parse
//! ```

use metadol::mcp::{DolTool, McpServer, ToolArgs};
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let server = McpServer::new();

    match args[1].as_str() {
        "serve" => run_server(server),
        "manifest" => print_manifest(server),
        "tool" if args.len() >= 3 => run_tool(server, &args[2], &args[3..]),
        _ => print_usage(),
    }
}

fn print_usage() {
    eprintln!("DOL MCP Server - Model Context Protocol for Metal DOL");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  dol-mcp serve              Start MCP server (stdio)");
    eprintln!("  dol-mcp manifest           Print server manifest");
    eprintln!("  dol-mcp tool <name> [args] Execute a tool");
    eprintln!();
    eprintln!("Available Tools:");
    eprintln!("  parse              Parse DOL source code");
    eprintln!("  typecheck          Type check DOL expression");
    eprintln!("  compile_rust       Compile to Rust");
    eprintln!("  compile_typescript Compile to TypeScript");
    eprintln!("  compile_jsonschema Compile to JSON Schema");
    eprintln!("  compile_wasm       Compile to WebAssembly");
    eprintln!("  eval               Evaluate DOL expression");
    eprintln!("  reflect            Get type information");
    eprintln!("  format             Format DOL source");
    eprintln!("  list_macros        List available macros");
    eprintln!("  expand_macro       Expand a macro");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  dol-mcp tool parse source=\"gene x {{ x has y }}\"");
    eprintln!("  echo 'gene x {{ x has y }}' | dol-mcp tool parse");
    eprintln!("  dol-mcp tool list_macros");
}

fn run_server(server: McpServer) {
    eprintln!("DOL MCP Server v{}", server.version);
    eprintln!("Listening on stdio for JSON-RPC requests...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                // Parse JSON-RPC request
                if let Ok(request) = serde_json::from_str::<serde_json::Value>(&input) {
                    let response = handle_request(&server, request);
                    let output = serde_json::to_string(&response).unwrap();
                    writeln!(stdout, "{}", output).unwrap();
                    stdout.flush().unwrap();
                } else {
                    eprintln!("Invalid JSON-RPC request: {}", input);
                }
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

fn handle_request(server: &McpServer, request: serde_json::Value) -> serde_json::Value {
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let result = match method {
        "initialize" => {
            let manifest = server.manifest();
            serde_json::to_value(manifest).unwrap()
        }
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or_default();
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            // Parse tool name into DolTool enum
            if let Some(tool) = DolTool::from_str(tool_name) {
                let args_map: HashMap<String, serde_json::Value> =
                    serde_json::from_value(args).unwrap_or_default();
                let tool_args = ToolArgs::new(args_map);

                match server.handle_tool(tool, tool_args) {
                    Ok(result) => serde_json::json!({
                        "content_type": result.content_type,
                        "content": result.content
                    }),
                    Err(e) => serde_json::json!({ "error": e }),
                }
            } else {
                serde_json::json!({ "error": format!("Unknown tool: {}", tool_name) })
            }
        }
        _ => serde_json::json!({ "error": format!("Unknown method: {}", method) }),
    };

    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn print_manifest(server: McpServer) {
    let manifest = server.manifest();
    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
}

fn run_tool(server: McpServer, tool_name: &str, args: &[String]) {
    // Parse tool name into DolTool enum
    let tool = match DolTool::from_str(tool_name) {
        Some(t) => t,
        None => {
            eprintln!("Error: Unknown tool '{}'", tool_name);
            eprintln!("Run 'dol-mcp' to see available tools");
            std::process::exit(1);
        }
    };

    let mut values = HashMap::new();

    // Parse key=value args
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            values.insert(
                key.to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }

    // Read from stdin if no args provided and tool needs input
    let needs_source = matches!(
        tool,
        DolTool::Parse
            | DolTool::CompileRust
            | DolTool::CompileTypeScript
            | DolTool::CompileWasm
            | DolTool::Format
    );

    let needs_expr = matches!(tool, DolTool::TypeCheck | DolTool::Eval);

    if !values.contains_key("source") && needs_source {
        if atty::isnt(atty::Stream::Stdin) {
            // Read from stdin if it's piped
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .expect("Failed to read from stdin");
            values.insert("source".to_string(), serde_json::Value::String(source));
        } else {
            eprintln!("Error: 'source' parameter is required");
            eprintln!("Provide it as: source=\"...\" or pipe input via stdin");
            std::process::exit(1);
        }
    }

    if !values.contains_key("expr") && needs_expr && atty::isnt(atty::Stream::Stdin) {
        let mut expr = String::new();
        io::stdin()
            .read_to_string(&mut expr)
            .expect("Failed to read from stdin");
        values.insert("expr".to_string(), serde_json::Value::String(expr));
    }

    let tool_args = ToolArgs::new(values);

    match server.handle_tool(tool, tool_args) {
        Ok(result) => {
            if result.content_type == "application/json" {
                // Pretty print JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&result.content) {
                    println!("{}", serde_json::to_string_pretty(&json).unwrap());
                } else {
                    println!("{}", result.content);
                }
            } else {
                println!("{}", result.content);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
