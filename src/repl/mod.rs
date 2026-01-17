//! Spirit REPL - Interactive DOL evaluation
//!
//! Provides a read-eval-print loop for DOL code, supporting:
//! - Interactive DOL expression evaluation
//! - Declaration definition and reuse
//! - DOL → Rust → WASM compilation pipeline
//! - Tree-shaked optimized output
//!
//! # Pipeline
//!
//! ```text
//! User Input → Parse → Tree Shake → Codegen (Rust) → WASM → Execute
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::repl::SpiritRepl;
//!
//! let mut repl = SpiritRepl::new();
//!
//! // Define a gene
//! repl.eval("gene Point { point has x: Int64; point has y: Int64 }")?;
//!
//! // Define a function
//! repl.eval("fun add(a: Int64, b: Int64) -> Int64 { a + b }")?;
//!
//! // Call the function
//! let result = repl.eval("add(3, 4)")?;
//! assert_eq!(result, Value::Int(7));
//! ```

mod context;
mod evaluator;
mod session;

pub use context::ReplContext;
pub use evaluator::{EvalResult, ReplEvaluator};
pub use session::{ReplSession, SessionConfig};

use crate::ast::Declaration;
use crate::codegen::compile_to_rust_via_hir;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::transform::TreeShaking;

/// Spirit REPL - Interactive DOL evaluation environment.
///
/// The SpiritRepl maintains state across evaluations, allowing
/// declarations to be defined and reused across multiple inputs.
#[derive(Debug)]
pub struct SpiritRepl {
    /// Accumulated declarations from session
    declarations: Vec<Declaration>,

    /// Tree shaker for dead code elimination
    tree_shaker: TreeShaking,

    /// Session configuration
    config: SessionConfig,

    /// Evaluation context (symbols, types, etc.)
    context: ReplContext,

    /// History of evaluated inputs
    history: Vec<String>,
}

impl Default for SpiritRepl {
    fn default() -> Self {
        Self::new()
    }
}

impl SpiritRepl {
    /// Create a new Spirit REPL with default configuration.
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            tree_shaker: TreeShaking::new(),
            config: SessionConfig::default(),
            context: ReplContext::new(),
            history: Vec::new(),
        }
    }

    /// Create a REPL with custom configuration.
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            declarations: Vec::new(),
            tree_shaker: TreeShaking::new(),
            config,
            context: ReplContext::new(),
            history: Vec::new(),
        }
    }

    /// Evaluate a DOL input string.
    ///
    /// The input can be:
    /// - A declaration (gene, trait, constraint, system, function)
    /// - An expression to evaluate
    /// - A REPL command (starting with `:`)
    ///
    /// # Arguments
    ///
    /// * `input` - The DOL source to evaluate
    ///
    /// # Returns
    ///
    /// The result of evaluation, or an error.
    pub fn eval(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        let input = input.trim();

        // Handle REPL commands
        if input.starts_with(':') {
            return self.handle_command(input);
        }

        // Skip empty input
        if input.is_empty() {
            return Ok(EvalResult::Empty);
        }

        // Add to history
        self.history.push(input.to_string());

        // Try to parse as declaration first
        if let Ok(decl) = self.try_parse_declaration(input) {
            return self.process_declaration(decl);
        }

        // Try to parse as expression
        if let Ok(expr_result) = self.try_eval_expression(input) {
            return Ok(expr_result);
        }

        // Failed to parse
        Err(ReplError::Parse(format!(
            "Could not parse input as declaration or expression: {}",
            input
        )))
    }

    /// Handle a REPL command (starts with `:`)
    fn handle_command(&mut self, cmd: &str) -> Result<EvalResult, ReplError> {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).map(|s| s.trim());

        match command {
            ":help" | ":h" | ":?" => Ok(EvalResult::Help(HELP_TEXT.to_string())),

            ":quit" | ":q" | ":exit" => Ok(EvalResult::Quit),

            ":clear" | ":reset" => {
                self.declarations.clear();
                self.context = ReplContext::new();
                Ok(EvalResult::Message("Session cleared".to_string()))
            }

            ":list" | ":ls" => {
                let names: Vec<String> = self
                    .declarations
                    .iter()
                    .map(|d| d.name().to_string())
                    .collect();
                if names.is_empty() {
                    Ok(EvalResult::Message("No declarations defined".to_string()))
                } else {
                    Ok(EvalResult::Message(format!(
                        "Declarations:\n  {}",
                        names.join("\n  ")
                    )))
                }
            }

            ":type" | ":t" => {
                if let Some(name) = args {
                    self.show_type(name)
                } else {
                    Err(ReplError::Command("Usage: :type <name>".to_string()))
                }
            }

            ":emit" | ":rust" => {
                // Emit Rust code for current session
                self.emit_rust()
            }

            ":wasm" => {
                // Compile to WASM and show info
                self.compile_wasm_info()
            }

            ":shake" => {
                // Run tree shaking analysis
                self.analyze_tree_shaking()
            }

            ":history" => {
                let hist = self
                    .history
                    .iter()
                    .enumerate()
                    .map(|(i, h)| format!("{}: {}", i + 1, h))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(EvalResult::Message(hist))
            }

            ":load" => {
                if let Some(path) = args {
                    self.load_file(path)
                } else {
                    Err(ReplError::Command("Usage: :load <file.dol>".to_string()))
                }
            }

            _ => Err(ReplError::Command(format!("Unknown command: {}", command))),
        }
    }

    /// Try to parse input as a declaration.
    fn try_parse_declaration(&self, input: &str) -> Result<Declaration, ParseError> {
        let mut parser = Parser::new(input);
        parser.parse()
    }

    /// Process a parsed declaration.
    fn process_declaration(&mut self, decl: Declaration) -> Result<EvalResult, ReplError> {
        let name = decl.name().to_string();
        let kind = match &decl {
            Declaration::Gene(_) => "gene",
            Declaration::Trait(_) => "trait",
            Declaration::Constraint(_) => "constraint",
            Declaration::System(_) => "system",
            Declaration::Evolution(_) => "evolution",
            Declaration::Function(_) => "function",
            Declaration::Const(_) => "const",
            Declaration::SexVar(_) => "var",
        };

        // Check if we're redefining
        let redefined = self.declarations.iter().any(|d| d.name() == name);

        if redefined {
            // Remove old definition
            self.declarations.retain(|d| d.name() != name);
        }

        // Add new declaration
        self.declarations.push(decl);

        // Update context
        self.context.add_declaration(&name, kind);

        let msg = if redefined {
            format!("Redefined {} {}", kind, name)
        } else {
            format!("Defined {} {}", kind, name)
        };

        Ok(EvalResult::Defined {
            name,
            kind: kind.to_string(),
            message: msg,
        })
    }

    /// Try to evaluate input as an expression.
    fn try_eval_expression(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        // Wrap expression in a temporary function for evaluation
        let wrapper = format!(
            r#"
fun __repl_eval__() -> Int64 {{
    {}
}}
"#,
            input
        );

        // Try to parse the wrapper
        let mut parser = Parser::new(&wrapper);
        let _decl = parser
            .parse()
            .map_err(|e| ReplError::Parse(e.to_string()))?;

        // Compile and execute (placeholder for now)
        // TODO: Integrate with WASM runtime
        Ok(EvalResult::Expression {
            input: input.to_string(),
            value: "TODO: expression evaluation".to_string(),
        })
    }

    /// Show type information for a declaration.
    fn show_type(&self, name: &str) -> Result<EvalResult, ReplError> {
        let decl = self
            .declarations
            .iter()
            .find(|d| d.name() == name)
            .ok_or_else(|| ReplError::NotFound(name.to_string()))?;

        let info = match decl {
            Declaration::Gene(g) => {
                let fields: Vec<String> = g
                    .statements
                    .iter()
                    .filter_map(|s| {
                        if let crate::ast::Statement::HasField(f) = s {
                            Some(format!("  {}: {:?}", f.name, f.type_))
                        } else {
                            None
                        }
                    })
                    .collect();
                format!("gene {} {{\n{}\n}}", name, fields.join("\n"))
            }
            Declaration::Function(f) => {
                let params: Vec<String> = f
                    .params
                    .iter()
                    .map(|p| format!("{}: {:?}", p.name, p.type_ann))
                    .collect();
                format!(
                    "fun {}({}) -> {:?}",
                    f.name,
                    params.join(", "),
                    f.return_type
                )
            }
            _ => format!("{} {}", declaration_kind_name(decl), name),
        };

        Ok(EvalResult::TypeInfo(info))
    }

    /// Emit Rust code for the current session.
    fn emit_rust(&self) -> Result<EvalResult, ReplError> {
        if self.declarations.is_empty() {
            return Ok(EvalResult::Message("No declarations to emit".to_string()));
        }

        // Build a DOL source from declarations (simplified)
        // In practice, we'd need to reconstruct the source or keep it
        let source = self.build_source();

        match compile_to_rust_via_hir(&source) {
            Ok(rust_code) => Ok(EvalResult::RustCode(rust_code)),
            Err(e) => Err(ReplError::Codegen(e.to_string())),
        }
    }

    /// Compile to WASM and show information.
    fn compile_wasm_info(&self) -> Result<EvalResult, ReplError> {
        #[cfg(feature = "wasm-compile")]
        {
            use crate::wasm::WasmCompiler;

            if self.declarations.is_empty() {
                return Ok(EvalResult::Message(
                    "No declarations to compile".to_string(),
                ));
            }

            let source = self.build_source();
            let file =
                crate::parse_dol_file(&source).map_err(|e| ReplError::Parse(e.to_string()))?;

            let mut compiler = WasmCompiler::new();
            let wasm_bytes = compiler
                .compile_file(&file)
                .map_err(|e| ReplError::Wasm(e.message))?;

            Ok(EvalResult::WasmInfo {
                size_bytes: wasm_bytes.len(),
                functions: self.count_functions(),
                has_memory: true,
            })
        }

        #[cfg(not(feature = "wasm-compile"))]
        Err(ReplError::Feature(
            "wasm-compile feature not enabled".to_string(),
        ))
    }

    /// Run tree shaking analysis.
    fn analyze_tree_shaking(&mut self) -> Result<EvalResult, ReplError> {
        if self.declarations.is_empty() {
            return Ok(EvalResult::Message(
                "No declarations to analyze".to_string(),
            ));
        }

        let stats = self.tree_shaker.analyze(&self.declarations);
        Ok(EvalResult::Message(stats.to_string()))
    }

    /// Load declarations from a file.
    fn load_file(&mut self, path: &str) -> Result<EvalResult, ReplError> {
        let source = std::fs::read_to_string(path).map_err(|e| ReplError::Io(e.to_string()))?;

        let file = crate::parse_dol_file(&source).map_err(|e| ReplError::Parse(e.to_string()))?;

        let count = file.declarations.len();
        for decl in file.declarations {
            self.process_declaration(decl)?;
        }

        Ok(EvalResult::Message(format!(
            "Loaded {} declarations from {}",
            count, path
        )))
    }

    /// Build DOL source from accumulated declarations.
    fn build_source(&self) -> String {
        // For now, we need a way to reconstruct source
        // This is a limitation - ideally we'd keep original source
        // For MVP, we'll generate minimal source that can be re-parsed

        let mut source = String::new();

        for decl in &self.declarations {
            match decl {
                Declaration::Function(f) => {
                    // Reconstruct function signature
                    let params: Vec<String> = f
                        .params
                        .iter()
                        .map(|p| format!("{}: {:?}", p.name, p.type_ann))
                        .collect();
                    let ret = match &f.return_type {
                        Some(t) => format!(" -> {:?}", t),
                        None => String::new(),
                    };
                    source.push_str(&format!(
                        "fun {}({}){} {{ 0 }}\n\n",
                        f.name,
                        params.join(", "),
                        ret
                    ));
                }
                Declaration::Gene(g) => {
                    source.push_str(&format!("gene {} {{\n", g.name));
                    for stmt in &g.statements {
                        source.push_str(&format!("  {}\n", stmt.to_dol_string()));
                    }
                    source.push_str("}\n\n");
                    source.push_str(&format!("exegesis {{\n  {}\n}}\n\n", g.exegesis));
                }
                _ => {
                    // For other types, use a placeholder
                    source.push_str(&format!(
                        "// {} {}\n",
                        declaration_kind_name(decl),
                        decl.name()
                    ));
                }
            }
        }

        source
    }

    /// Count functions in declarations.
    fn count_functions(&self) -> usize {
        self.declarations
            .iter()
            .filter(|d| matches!(d, Declaration::Function(_)))
            .count()
    }

    /// Get the current declarations.
    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    /// Get evaluation history.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Get the session configuration.
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

/// REPL error types.
#[derive(Debug, Clone)]
pub enum ReplError {
    /// Parse error
    Parse(String),
    /// Command error
    Command(String),
    /// Declaration not found
    NotFound(String),
    /// Code generation error
    Codegen(String),
    /// WASM compilation error
    Wasm(String),
    /// I/O error
    Io(String),
    /// Feature not available
    Feature(String),
}

impl std::fmt::Display for ReplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ReplError::Command(msg) => write!(f, "Command error: {}", msg),
            ReplError::NotFound(name) => write!(f, "Not found: {}", name),
            ReplError::Codegen(msg) => write!(f, "Codegen error: {}", msg),
            ReplError::Wasm(msg) => write!(f, "WASM error: {}", msg),
            ReplError::Io(msg) => write!(f, "I/O error: {}", msg),
            ReplError::Feature(msg) => write!(f, "Feature error: {}", msg),
        }
    }
}

impl std::error::Error for ReplError {}

/// Help text for the REPL.
const HELP_TEXT: &str = r#"
Spirit REPL - Interactive DOL Environment

Commands:
  :help, :h, :?     Show this help
  :quit, :q, :exit  Exit the REPL
  :clear, :reset    Clear all declarations
  :list, :ls        List defined declarations
  :type <name>      Show type info for a declaration
  :emit, :rust      Emit Rust code for session
  :wasm             Compile to WASM and show info
  :shake            Run tree shaking analysis
  :history          Show input history
  :load <file>      Load declarations from file

Input Types:
  - Declarations: gene, trait, constraint, system, fun
  - Expressions: arithmetic, function calls (evaluation in progress)

Examples:
  gene Point { point has x: Int64; point has y: Int64 }
  fun add(a: Int64, b: Int64) -> Int64 { a + b }
  :type Point
  :emit
"#;

/// Helper function to get the kind name of a declaration.
fn declaration_kind_name(decl: &Declaration) -> &'static str {
    match decl {
        Declaration::Gene(_) => "gene",
        Declaration::Trait(_) => "trait",
        Declaration::Constraint(_) => "rule",
        Declaration::System(_) => "system",
        Declaration::Evolution(_) => "evo",
        Declaration::Function(_) => "fun",
        Declaration::Const(_) => "const",
        Declaration::SexVar(_) => "sex var",
    }
}

// Extension trait for Statement to generate DOL string
trait StatementExt {
    fn to_dol_string(&self) -> String;
}

impl StatementExt for crate::ast::Statement {
    fn to_dol_string(&self) -> String {
        match self {
            crate::ast::Statement::Has {
                subject, property, ..
            } => {
                format!("{} has {}", subject, property)
            }
            crate::ast::Statement::HasField(f) => {
                // HasField has: name, type_, default, constraint, span
                format!("has {}: {:?}", f.name, f.type_)
            }
            crate::ast::Statement::Is { subject, state, .. } => {
                format!("{} is {}", subject, state)
            }
            crate::ast::Statement::DerivesFrom {
                subject, origin, ..
            } => {
                format!("{} derives from {}", subject, origin)
            }
            crate::ast::Statement::Requires {
                subject,
                requirement,
                ..
            } => {
                format!("{} requires {}", subject, requirement)
            }
            crate::ast::Statement::Uses { reference, .. } => {
                format!("uses {}", reference)
            }
            crate::ast::Statement::Emits { action, event, .. } => {
                format!("{} emits {}", action, event)
            }
            crate::ast::Statement::Matches {
                subject, target, ..
            } => {
                format!("{} matches {}", subject, target)
            }
            crate::ast::Statement::Never {
                subject, action, ..
            } => {
                format!("{} never {}", subject, action)
            }
            crate::ast::Statement::Quantified {
                quantifier, phrase, ..
            } => {
                format!("{} {}", quantifier, phrase)
            }
            crate::ast::Statement::Function(_) => "// function".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_new() {
        let repl = SpiritRepl::new();
        assert!(repl.declarations.is_empty());
        assert!(repl.history.is_empty());
    }

    #[test]
    fn test_repl_help_command() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":help");
        assert!(matches!(result, Ok(EvalResult::Help(_))));
    }

    #[test]
    fn test_repl_quit_command() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":quit");
        assert!(matches!(result, Ok(EvalResult::Quit)));
    }

    #[test]
    fn test_repl_empty_input() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("");
        assert!(matches!(result, Ok(EvalResult::Empty)));
    }

    #[test]
    fn test_repl_list_empty() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":list");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("No declarations"));
            }
            _ => panic!("Expected message about no declarations"),
        }
    }
}
