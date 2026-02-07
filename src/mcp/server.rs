//! MCP Server implementation for Metal DOL.
//!
//! This module implements the Model Context Protocol server that exposes
//! Metal DOL's capabilities as callable tools.

use super::{
    diagnostics::SchemaDiagnostics,
    nl_to_dol::{NlRequirement, NlToDolConverter},
    recommendations::{ConsistencyLevel, CrdtRecommender, UsagePattern},
    schema_validator::{SchemaValidator, ValidationContext},
    suggestions::{SuggestionContext, SuggestionEngine},
    DolTool,
};
use crate::{
    codegen::{RustCodegen, TypeScriptCodegen},
    macros::BuiltinMacros,
    parse_file,
    reflect::TypeRegistry,
    validator::validate,
};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use crate::{ast::Expr, eval::Interpreter, typechecker::TypeChecker};

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
            // General tools
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

            // CRDT-specific tools
            DolTool::ValidateSchema => self.tool_validate_schema(args),
            DolTool::RecommendCrdt => self.tool_recommend_crdt(args),
            DolTool::ExplainStrategy => self.tool_explain_strategy(args),
            DolTool::GenerateExample => self.tool_generate_example(args),

            // AI-powered tools (M3.1-M3.3)
            DolTool::GenerateSchemaFromDescription => {
                self.tool_generate_schema_from_description(args)
            }
            DolTool::ValidateAndSuggest => self.tool_validate_and_suggest(args),
            DolTool::GetSuggestions => self.tool_get_suggestions(args),
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
            let _ = args;
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
            let _ = args;
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

    // === CRDT-Specific Tools ===

    fn tool_validate_schema(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let source = args.get_string("source")?;

        match parse_file(&source) {
            Ok(decl) => {
                // Run standard validation
                let validation_result = validate(&decl);

                // Run CRDT diagnostics
                let diagnostics = SchemaDiagnostics::new();
                let issues = diagnostics.analyze(&decl);

                #[cfg(feature = "serde")]
                {
                    // Manually construct JSON to avoid needing Serialize on ValidationError/Warning
                    let errors_json: Vec<String> = validation_result
                        .errors
                        .iter()
                        .map(|e| format!("{}", e))
                        .collect();

                    let warnings_json: Vec<String> = validation_result
                        .warnings
                        .iter()
                        .map(|w| format!("{}", w))
                        .collect();

                    let response = serde_json::json!({
                        "valid": validation_result.is_valid(),
                        "errors": errors_json,
                        "warnings": warnings_json,
                        "crdt_issues": issues,
                    });
                    Ok(ToolResult::json(
                        serde_json::to_string_pretty(&response)
                            .map_err(|e| format!("Serialization error: {}", e))?,
                    ))
                }
                #[cfg(not(feature = "serde"))]
                {
                    let mut output = format!("Valid: {}\n", validation_result.is_valid());
                    output.push_str(&format!("Errors: {}\n", validation_result.errors.len()));
                    output.push_str(&format!("Warnings: {}\n", validation_result.warnings.len()));
                    output.push_str(&format!("CRDT Issues: {}\n", issues.len()));
                    Ok(ToolResult::text(output))
                }
            }
            Err(e) => Err(format!("Parse error: {}", e)),
        }
    }

    fn tool_recommend_crdt(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let field_name = args.get_string("field_name")?;
        let field_type = args.get_string("field_type")?;
        let usage_pattern_str = args.get_string("usage_pattern")?;
        let consistency_str = args
            .get_optional_string("consistency_requirement")
            .unwrap_or_else(|| "eventual".to_string());

        // Parse usage pattern
        let usage_pattern = match usage_pattern_str.as_str() {
            "write-once" => UsagePattern::WriteOnce,
            "last-write-wins" => UsagePattern::LastWriteWins,
            "collaborative-text" => UsagePattern::CollaborativeText,
            "multi-user-set" => UsagePattern::MultiUserSet,
            "counter" => UsagePattern::Counter,
            "ordered-list" => UsagePattern::OrderedList,
            _ => {
                return Err(format!("Invalid usage pattern: {}", usage_pattern_str));
            }
        };

        // Parse consistency level
        let consistency = match consistency_str.as_str() {
            "eventual" => ConsistencyLevel::Eventual,
            "causal" => ConsistencyLevel::Causal,
            "strong" => ConsistencyLevel::Strong,
            _ => {
                return Err(format!("Invalid consistency level: {}", consistency_str));
            }
        };

        let recommender = CrdtRecommender::new();
        let recommendation =
            recommender.recommend(&field_name, &field_type, usage_pattern, consistency);

        #[cfg(feature = "serde")]
        {
            let json = serde_json::to_string_pretty(&recommendation)
                .map_err(|e| format!("Serialization error: {}", e))?;
            Ok(ToolResult::json(json))
        }
        #[cfg(not(feature = "serde"))]
        {
            let mut output = format!(
                "Recommended Strategy: {}\n",
                recommendation.recommended_strategy
            );
            output.push_str(&format!("Confidence: {}\n", recommendation.confidence));
            output.push_str(&format!("Reasoning: {}\n", recommendation.reasoning));
            output.push_str(&format!("Example: {}\n", recommendation.example));
            Ok(ToolResult::text(output))
        }
    }

    fn tool_explain_strategy(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let strategy = args.get_string("strategy")?;

        let explanation = match strategy.as_str() {
            "immutable" => {
                "Immutable Strategy:\n\
                 - Set once, never modified\n\
                 - Perfect for IDs, creation timestamps\n\
                 - No merge conflicts\n\
                 - Minimal storage overhead\n\
                 - Trade-off: Cannot modify after creation"
            }
            "lww" => {
                "Last-Write-Wins (LWW) Strategy:\n\
                 - Most recent write wins based on timestamp\n\
                 - Simple and efficient\n\
                 - Works for any single-valued type\n\
                 - Trade-off: Concurrent updates are lost\n\
                 - Requires accurate timestamps/clocks"
            }
            "peritext" => {
                "Peritext Strategy:\n\
                 - Collaborative rich text editing\n\
                 - Conflict-free concurrent editing\n\
                 - Preserves formatting and user intent\n\
                 - Based on RGA + formatting marks\n\
                 - Trade-off: Higher storage/merge overhead\n\
                 - Best-in-class for document collaboration"
            }
            "or_set" => {
                "Observed-Remove Set (OR-Set) Strategy:\n\
                 - Add-wins semantics\n\
                 - Each element tagged with unique ID\n\
                 - Concurrent add + remove → element present\n\
                 - Trade-off: Tombstone overhead\n\
                 - Perfect for collaborative tags/members"
            }
            "pn_counter" => {
                "Positive-Negative Counter Strategy:\n\
                 - Separate increment/decrement counters per actor\n\
                 - Value = sum(increments) - sum(decrements)\n\
                 - Commutative and convergent\n\
                 - Trade-off: Per-actor state tracking\n\
                 - Cannot enforce strict bounds without coordination\n\
                 - Ideal for likes, votes, distributed counters"
            }
            "rga" => {
                "Replicated Growable Array (RGA) Strategy:\n\
                 - Ordered sequence with causal insertion order\n\
                 - Each element has unique ID and left reference\n\
                 - Concurrent inserts ordered deterministically\n\
                 - Trade-off: Tombstone overhead for deletes\n\
                 - Perfect for ordered lists, task sequences"
            }
            "mv_register" => {
                "Multi-Value Register Strategy:\n\
                 - Keeps all concurrent values\n\
                 - Application chooses resolution strategy\n\
                 - Useful for detecting conflicts\n\
                 - Trade-off: Requires manual resolution\n\
                 - Can accumulate values without cleanup\n\
                 - Flexible conflict handling"
            }
            _ => {
                return Err(format!("Unknown strategy: {}", strategy));
            }
        };

        Ok(ToolResult::text(explanation.to_string()))
    }

    fn tool_generate_example(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let use_case = args
            .get_optional_string("use_case")
            .unwrap_or_else(|| "chat_message".to_string());

        let example = match use_case.as_str() {
            "chat_message" => {
                r#"gen message.chat {
  @crdt(immutable)
  has id: Uuid

  @crdt(immutable)
  has created_at: Timestamp

  @crdt(lww)
  has author: Identity

  @crdt(peritext, formatting="full", max_length=100000)
  has content: RichText

  @crdt(or_set)
  has reactions: Set<Reaction>

  @crdt(lww)
  has edited_at: Option<Timestamp>
}

exegesis {
  A collaborative chat message with immutable identity,
  real-time text editing (Peritext), and add-wins reactions.
}"#
            }
            "task_board" => {
                r#"gen task.item {
  @crdt(immutable)
  has id: Uuid

  @crdt(lww)
  has title: String

  @crdt(peritext, formatting="markdown")
  has description: RichText

  @crdt(lww)
  has status: TaskStatus

  @crdt(or_set)
  has assignees: Set<Identity>

  @crdt(pn_counter, min_value=0)
  has estimate_hours: Int

  @crdt(lww)
  has due_date: Option<Timestamp>
}

exegesis {
  A collaborative task item with LWW metadata,
  Peritext descriptions, and OR-Set assignees.
}"#
            }
            "user_profile" => {
                r#"gen user.profile {
  @crdt(immutable)
  has id: Uuid

  @crdt(lww)
  has display_name: String

  @crdt(lww)
  has avatar_url: String

  @crdt(peritext, formatting="markdown", max_length=10000)
  has bio: RichText

  @crdt(or_set)
  has interests: Set<String>

  @crdt(pn_counter, min_value=0)
  has follower_count: Int
}

exegesis {
  A user profile with LWW simple fields,
  collaborative bio, and CRDT counters.
}"#
            }
            "counter" => {
                r#"gen post.social {
  @crdt(immutable)
  has id: Uuid

  @crdt(lww)
  has content: String

  @crdt(pn_counter, min_value=0)
  has likes: Int

  @crdt(pn_counter)
  has karma_score: Int

  @crdt(pn_counter, min_value=0, overflow_strategy="saturate")
  has view_count: Int
}

exegesis {
  A social media post with PN-Counter metrics
  for distributed like/view tracking.
}"#
            }
            _ => {
                return Err(format!("Unknown use case: {}", use_case));
            }
        };

        Ok(ToolResult::text(example.to_string()))
    }

    // === AI-Powered Tools (M3.1-M3.3) ===

    fn tool_generate_schema_from_description(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let description = args.get_string("description")?;
        let entity_name = args.get_optional_string("entity_name");
        let constraints = args
            .get_optional_string("constraints")
            .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        let requirement = NlRequirement {
            description,
            entity_name,
            constraints,
        };

        let converter = NlToDolConverter::new();
        match converter.convert(requirement) {
            Ok(schema) => {
                #[cfg(feature = "serde")]
                {
                    let json = serde_json::to_string_pretty(&schema)
                        .map_err(|e| format!("Serialization error: {}", e))?;
                    Ok(ToolResult::json(json))
                }
                #[cfg(not(feature = "serde"))]
                {
                    Ok(ToolResult::text(schema.dol_source))
                }
            }
            Err(e) => Err(format!("Schema generation error: {}", e)),
        }
    }

    fn tool_validate_and_suggest(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let source = args.get_string("source")?;

        let validator = SchemaValidator::new();
        let report = validator.validate_schema(&source, ValidationContext::default());

        #[cfg(feature = "serde")]
        {
            let json = serde_json::to_string_pretty(&report)
                .map_err(|e| format!("Serialization error: {}", e))?;
            Ok(ToolResult::json(json))
        }
        #[cfg(not(feature = "serde"))]
        {
            let mut output = format!("Validation Score: {}/100\n", report.score);
            output.push_str(&format!("Summary: {}\n\n", report.summary));
            output.push_str(&format!("Errors: {}\n", report.error_count));
            output.push_str(&format!("Warnings: {}\n", report.warning_count));
            output.push_str(&format!("Info: {}\n\n", report.info_count));

            if !report.issues.is_empty() {
                output.push_str("Issues:\n");
                for issue in &report.issues {
                    output.push_str(&format!(
                        "  [{:?}] {}: {}\n",
                        issue.severity, issue.location, issue.message
                    ));
                    if let Some(suggestion) = &issue.suggestion {
                        output.push_str(&format!("    Suggestion: {}\n", suggestion));
                    }
                }
            }

            Ok(ToolResult::text(output))
        }
    }

    fn tool_get_suggestions(&self, args: ToolArgs) -> Result<ToolResult, String> {
        let source = args.get_string("source")?;
        let use_case = args.get_optional_string("use_case");
        let expected_scale = args.get_optional_string("expected_scale");
        let performance_priority = args
            .get_optional_string("performance_priority")
            .map(|s| s == "true")
            .unwrap_or(false);
        let security_priority = args
            .get_optional_string("security_priority")
            .map(|s| s == "true")
            .unwrap_or(false);

        let context = SuggestionContext {
            use_case,
            expected_scale,
            performance_priority,
            security_priority,
        };

        let engine = SuggestionEngine::new();
        let suggestions = engine.analyze_and_suggest(&source, context);

        #[cfg(feature = "serde")]
        {
            let json = serde_json::to_string_pretty(&suggestions)
                .map_err(|e| format!("Serialization error: {}", e))?;
            Ok(ToolResult::json(json))
        }
        #[cfg(not(feature = "serde"))]
        {
            let mut output = format!("Health Score: {}/100\n", suggestions.health_score);
            output.push_str(&format!("{}\n\n", suggestions.summary));

            if !suggestions.suggestions.is_empty() {
                output.push_str("Suggestions:\n");
                for (i, suggestion) in suggestions.suggestions.iter().enumerate() {
                    output.push_str(&format!(
                        "\n{}. {} [{:?}]\n",
                        i + 1,
                        suggestion.title,
                        suggestion.priority
                    ));
                    output.push_str(&format!("   {}\n", suggestion.description));
                    output.push_str(&format!("   Rationale: {}\n", suggestion.rationale));
                    output.push_str(&format!("   Example: {}\n", suggestion.code_example));
                }
            }

            Ok(ToolResult::text(output))
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
                // CRDT-specific tools
                ToolDef {
                    name: "validate_schema".to_string(),
                    description: "Validate DOL schema with CRDT annotations and detect anti-patterns".to_string(),
                    parameters: vec![ParamDef {
                        name: "source".to_string(),
                        description: "DOL source code to validate".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "recommend_crdt".to_string(),
                    description: "Recommend CRDT strategy for a field based on usage pattern".to_string(),
                    parameters: vec![
                        ParamDef {
                            name: "field_name".to_string(),
                            description: "Name of the field".to_string(),
                            required: true,
                        },
                        ParamDef {
                            name: "field_type".to_string(),
                            description: "Type of the field (e.g., String, i32, Set<String>)".to_string(),
                            required: true,
                        },
                        ParamDef {
                            name: "usage_pattern".to_string(),
                            description: "Usage pattern: write-once, last-write-wins, collaborative-text, multi-user-set, counter, ordered-list".to_string(),
                            required: true,
                        },
                        ParamDef {
                            name: "consistency_requirement".to_string(),
                            description: "Consistency level: eventual, causal, strong (default: eventual)".to_string(),
                            required: false,
                        },
                    ],
                },
                ToolDef {
                    name: "explain_strategy".to_string(),
                    description: "Explain a CRDT strategy's semantics, trade-offs, and use cases".to_string(),
                    parameters: vec![ParamDef {
                        name: "strategy".to_string(),
                        description: "CRDT strategy: immutable, lww, peritext, or_set, pn_counter, rga, mv_register".to_string(),
                        required: true,
                    }],
                },
                ToolDef {
                    name: "generate_example".to_string(),
                    description: "Generate example DOL schema with CRDT annotations for common use cases".to_string(),
                    parameters: vec![ParamDef {
                        name: "use_case".to_string(),
                        description: "Use case: chat_message, task_board, user_profile, counter (default: chat_message)".to_string(),
                        required: false,
                    }],
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
#[cfg(feature = "serde")]
pub struct ToolArgs {
    args: HashMap<String, serde_json::Value>,
}

#[cfg(not(feature = "serde"))]
pub struct ToolArgs {
    args: HashMap<String, String>,
}

#[cfg(feature = "serde")]
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

#[cfg(not(feature = "serde"))]
impl ToolArgs {
    /// Creates a new ToolArgs from a HashMap.
    pub fn new(args: HashMap<String, String>) -> Self {
        Self { args }
    }

    /// Gets a string argument by name.
    ///
    /// Returns an error if the argument is missing or not a string.
    pub fn get_string(&self, key: &str) -> Result<String, String> {
        self.args
            .get(key)
            .cloned()
            .ok_or_else(|| format!("Missing or invalid argument: {}", key))
    }

    /// Gets an optional string argument by name.
    pub fn get_optional_string(&self, key: &str) -> Option<String> {
        self.args.get(key).cloned()
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
                r#"gene container.exists {
  container has identity
  container has status
}

exegesis {
  A container is the fundamental unit of workload isolation.
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
