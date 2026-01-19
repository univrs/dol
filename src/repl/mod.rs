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

    /// Original source text for each declaration (for WASM compilation)
    source_texts: Vec<String>,

    /// Tree shaker for dead code elimination
    tree_shaker: TreeShaking,

    /// Session configuration
    config: SessionConfig,

    /// Evaluation context (symbols, types, etc.)
    context: ReplContext,

    /// History of evaluated inputs
    history: Vec<String>,

    /// Evaluator for expression execution
    evaluator: ReplEvaluator,
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
            source_texts: Vec::new(),
            tree_shaker: TreeShaking::new(),
            config: SessionConfig::default(),
            context: ReplContext::new(),
            history: Vec::new(),
            evaluator: ReplEvaluator::new(),
        }
    }

    /// Create a REPL with custom configuration.
    pub fn with_config(config: SessionConfig) -> Self {
        let evaluator = if config.optimize {
            ReplEvaluator::optimized()
        } else {
            ReplEvaluator::new()
        };

        Self {
            declarations: Vec::new(),
            source_texts: Vec::new(),
            tree_shaker: TreeShaking::new(),
            config,
            context: ReplContext::new(),
            history: Vec::new(),
            evaluator,
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
            return self.process_declaration(decl, input);
        }

        // Try to evaluate as expression
        // Return the expression result or error directly
        self.try_eval_expression(input)
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
                self.source_texts.clear();
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
    fn process_declaration(
        &mut self,
        decl: Declaration,
        source: &str,
    ) -> Result<EvalResult, ReplError> {
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
        let existing_idx = self.declarations.iter().position(|d| d.name() == name);
        let redefined = existing_idx.is_some();

        if let Some(idx) = existing_idx {
            // Remove old definition and its source text
            self.declarations.remove(idx);
            if idx < self.source_texts.len() {
                self.source_texts.remove(idx);
            }
        }

        // Add new declaration and its source
        self.declarations.push(decl);
        self.source_texts.push(source.to_string());

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
    ///
    /// Wraps the expression in a function, compiles to WASM, and executes it.
    /// Returns the evaluated result.
    #[cfg(feature = "wasm")]
    fn try_eval_expression(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        use crate::repl::evaluator::EvalError;

        // Get declarations source for context
        let declarations_source = self.build_source();

        // Use the evaluator to compile and execute
        match self.evaluator.eval_expression(input, &declarations_source) {
            Ok(value) => Ok(EvalResult::Expression {
                input: input.to_string(),
                value,
            }),
            Err(e) => {
                // Convert EvalError to ReplError
                let repl_err = match e {
                    EvalError::Parse(msg) => ReplError::Parse(msg),
                    EvalError::Compile(msg) => ReplError::Wasm(msg),
                    EvalError::Runtime(msg) => ReplError::Wasm(format!("Runtime: {}", msg)),
                    EvalError::Feature(msg) => ReplError::Feature(msg),
                };
                Err(repl_err)
            }
        }
    }

    /// Try to evaluate input as an expression (stub when wasm feature not enabled).
    #[cfg(not(feature = "wasm"))]
    fn try_eval_expression(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        // Infer the type for display purposes
        let return_type = self.evaluator.infer_expression_type(input);

        // Wrap expression in a temporary function for parsing validation
        let wrapper = format!(
            r#"
pub fun dolReplEval() -> {} {{
    {}
}}
"#,
            return_type, input
        );

        // Try to parse the wrapper to validate syntax
        let mut parser = Parser::new(&wrapper);
        let _decl = parser
            .parse()
            .map_err(|e| ReplError::Parse(e.to_string()))?;

        // Return message that evaluation requires wasm feature
        Err(ReplError::Feature(
            "Expression evaluation requires the 'wasm' feature. Use `cargo build --features wasm` to enable.".to_string()
        ))
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
        // For file loading, we use the whole source for all declarations
        // since we can't easily extract individual declaration sources
        for decl in file.declarations {
            self.process_declaration(decl, &source)?;
        }

        Ok(EvalResult::Message(format!(
            "Loaded {} declarations from {}",
            count, path
        )))
    }

    /// Build DOL source from accumulated declarations.
    fn build_source(&self) -> String {
        // Use the stored original source texts for accurate compilation
        self.source_texts.join("\n\n")
    }

    /// Count functions in declarations.
    #[allow(dead_code)]
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
#[allow(dead_code)]
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

    // ==================== Basic REPL Operations ====================

    #[test]
    fn test_repl_new() {
        let repl = SpiritRepl::new();
        assert!(repl.declarations.is_empty());
        assert!(repl.history.is_empty());
    }

    #[test]
    fn test_repl_default() {
        let repl = SpiritRepl::default();
        assert!(repl.declarations.is_empty());
    }

    #[test]
    fn test_repl_with_config() {
        let config = SessionConfig::with_name("test-session");
        let repl = SpiritRepl::with_config(config);
        assert_eq!(repl.config().name, "test-session");
    }

    // ==================== REPL Commands ====================

    #[test]
    fn test_repl_help_command() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":help");
        assert!(matches!(result, Ok(EvalResult::Help(_))));
    }

    #[test]
    fn test_repl_help_short_commands() {
        let mut repl = SpiritRepl::new();
        assert!(matches!(repl.eval(":h"), Ok(EvalResult::Help(_))));
        assert!(matches!(repl.eval(":?"), Ok(EvalResult::Help(_))));
    }

    #[test]
    fn test_repl_quit_command() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":quit");
        assert!(matches!(result, Ok(EvalResult::Quit)));
    }

    #[test]
    fn test_repl_quit_aliases() {
        let mut repl = SpiritRepl::new();
        assert!(matches!(repl.eval(":q"), Ok(EvalResult::Quit)));
        assert!(matches!(repl.eval(":exit"), Ok(EvalResult::Quit)));
    }

    #[test]
    fn test_repl_empty_input() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("");
        assert!(matches!(result, Ok(EvalResult::Empty)));
    }

    #[test]
    fn test_repl_whitespace_input() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("   \t  \n  ");
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

    #[test]
    fn test_repl_list_alias() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":ls");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("No declarations"));
            }
            _ => panic!("Expected message"),
        }
    }

    #[test]
    fn test_repl_unknown_command() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":foobar");
        assert!(matches!(result, Err(ReplError::Command(_))));
    }

    // ==================== Gene Declarations ====================

    #[test]
    fn test_repl_gene_declaration_legacy() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("gene Point { has x: Int64\n has y: Int64 }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "Point");
                assert_eq!(kind, "gene");
            }
            Err(e) => panic!("Failed to define gene: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    #[test]
    fn test_repl_gen_declaration_v080() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("gen Point { has x: i64\n has y: i64 }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "Point");
                assert_eq!(kind, "gene");
            }
            Err(e) => panic!("Failed to define gen: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    #[test]
    fn test_repl_gene_with_float_fields() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("gen Vector2D { has dx: f64\n has dy: f64 }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "Vector2D");
                assert_eq!(kind, "gene");
            }
            Err(e) => panic!("Failed to define gene with floats: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    // ==================== Function Declarations ====================

    #[test]
    fn test_repl_function_declaration() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("pub fun add(a: i64, b: i64) -> i64 { a + b }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "add");
                assert_eq!(kind, "function");
            }
            Err(e) => panic!("Failed to define function: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    #[test]
    fn test_repl_function_without_pub() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("fun multiply(x: i64, y: i64) -> i64 { x * y }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "multiply");
                assert_eq!(kind, "function");
            }
            Err(e) => panic!("Failed to define function: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    #[test]
    fn test_repl_function_with_gene_constructor() {
        let mut repl = SpiritRepl::new();

        // First define a gene
        let _ = repl.eval("gen Point { has x: i64\n has y: i64 }");

        // Then define a function that uses the gene
        let result =
            repl.eval("pub fun create_point() -> i64 { let p = Point { x: 10, y: 20 }\n p.x }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "create_point");
                assert_eq!(kind, "function");
            }
            Err(e) => panic!("Failed to define function with gene: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    // ==================== Trait Declarations ====================

    #[test]
    fn test_repl_trait_declaration() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval("trait Addable { has value: i64 }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "Addable");
                assert_eq!(kind, "trait");
            }
            Err(e) => panic!("Failed to define trait: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    // ==================== Declaration Management ====================

    #[test]
    fn test_repl_list_declarations() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();
        repl.eval("gen Circle { has radius: i64 }").unwrap();

        let result = repl.eval(":list");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("Point"));
                assert!(msg.contains("Circle"));
            }
            _ => panic!("Expected message with declarations"),
        }
    }

    #[test]
    fn test_repl_redefinition() {
        let mut repl = SpiritRepl::new();

        // Define Point
        repl.eval("gen Point { has x: i64 }").unwrap();
        assert_eq!(repl.declarations().len(), 1);

        // Redefine Point with different fields
        let result = repl.eval("gen Point { has x: i64\n has y: i64 }");
        match result {
            Ok(EvalResult::Defined { message, .. }) => {
                assert!(message.contains("Redefined"));
            }
            Err(e) => panic!("Failed to redefine: {:?}", e),
            _ => panic!("Expected Defined result"),
        }

        // Still only one declaration
        assert_eq!(repl.declarations().len(), 1);
    }

    #[test]
    fn test_repl_clear() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();
        repl.eval("gen Circle { has r: i64 }").unwrap();
        assert_eq!(repl.declarations().len(), 2);

        let result = repl.eval(":clear");
        assert!(matches!(result, Ok(EvalResult::Message(_))));
        assert!(repl.declarations().is_empty());
    }

    #[test]
    fn test_repl_reset_alias() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();

        let result = repl.eval(":reset");
        assert!(matches!(result, Ok(EvalResult::Message(_))));
        assert!(repl.declarations().is_empty());
    }

    // ==================== Type Information ====================

    #[test]
    fn test_repl_type_gene() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64\n has y: i64 }").unwrap();

        let result = repl.eval(":type Point");
        match result {
            Ok(EvalResult::TypeInfo(info)) => {
                assert!(info.contains("Point"));
                assert!(info.contains("x"));
                assert!(info.contains("y"));
            }
            Err(e) => panic!("Failed to get type info: {:?}", e),
            _ => panic!("Expected TypeInfo result"),
        }
    }

    #[test]
    fn test_repl_type_function() {
        let mut repl = SpiritRepl::new();
        repl.eval("fun add(a: i64, b: i64) -> i64 { a + b }")
            .unwrap();

        let result = repl.eval(":type add");
        match result {
            Ok(EvalResult::TypeInfo(info)) => {
                assert!(info.contains("add"));
                assert!(info.contains("a"));
                assert!(info.contains("b"));
            }
            Err(e) => panic!("Failed to get type info: {:?}", e),
            _ => panic!("Expected TypeInfo result"),
        }
    }

    #[test]
    fn test_repl_type_alias() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();

        let result = repl.eval(":t Point");
        assert!(matches!(result, Ok(EvalResult::TypeInfo(_))));
    }

    #[test]
    fn test_repl_type_not_found() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":type NonExistent");
        assert!(matches!(result, Err(ReplError::NotFound(_))));
    }

    #[test]
    fn test_repl_type_no_arg() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":type");
        assert!(matches!(result, Err(ReplError::Command(_))));
    }

    // ==================== History ====================

    #[test]
    fn test_repl_history() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();
        repl.eval("gen Circle { has r: i64 }").unwrap();

        assert_eq!(repl.history().len(), 2);
        assert!(repl.history()[0].contains("Point"));
        assert!(repl.history()[1].contains("Circle"));
    }

    #[test]
    fn test_repl_history_command() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();

        let result = repl.eval(":history");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("Point"));
            }
            _ => panic!("Expected history message"),
        }
    }

    // ==================== Expression Evaluation ====================

    #[test]
    #[cfg(feature = "wasm")]
    fn test_repl_expression_now_supported() {
        let mut repl = SpiritRepl::new();
        // Simple expressions like "1 + 2" are now supported with wasm feature
        let result = repl.eval("1 + 2");
        match result {
            Ok(EvalResult::Expression { value, .. }) => {
                assert_eq!(value, "3");
            }
            other => panic!("Expected Expression result with value 3, got {:?}", other),
        }
    }

    #[test]
    #[cfg(not(feature = "wasm"))]
    fn test_repl_expression_requires_wasm() {
        let mut repl = SpiritRepl::new();
        // Without wasm feature, expression evaluation returns Feature error
        let result = repl.eval("1 + 2");
        assert!(matches!(result, Err(ReplError::Feature(_))));
    }

    #[test]
    fn test_repl_function_as_expression() {
        // For now, define a function and call it via declarations
        let mut repl = SpiritRepl::new();

        // Define a function
        let result = repl.eval("pub fun calculate() -> i64 { 1 + 2 }");
        match result {
            Ok(EvalResult::Defined { name, kind, .. }) => {
                assert_eq!(name, "calculate");
                assert_eq!(kind, "function");
            }
            Err(e) => panic!("Failed to define function: {:?}", e),
            _ => panic!("Expected Defined result"),
        }
    }

    // ==================== Tree Shaking ====================

    #[test]
    fn test_repl_shake_empty() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":shake");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("No declarations"));
            }
            _ => panic!("Expected message about no declarations"),
        }
    }

    #[test]
    fn test_repl_shake_with_declarations() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();
        repl.eval("pub fun test() -> i64 { let p = Point { x: 10 }\n p.x }")
            .unwrap();

        let result = repl.eval(":shake");
        assert!(matches!(result, Ok(EvalResult::Message(_))));
    }

    // ==================== Emit Rust ====================

    #[test]
    fn test_repl_emit_empty() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":emit");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("No declarations"));
            }
            _ => panic!("Expected message about no declarations"),
        }
    }

    #[test]
    fn test_repl_emit_alias() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":rust");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("No declarations"));
            }
            _ => panic!("Expected message"),
        }
    }

    // ==================== WASM Compilation ====================

    #[cfg(feature = "wasm-compile")]
    #[test]
    fn test_repl_wasm_empty() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":wasm");
        match result {
            Ok(EvalResult::Message(msg)) => {
                assert!(msg.contains("No declarations"));
            }
            _ => panic!("Expected message about no declarations"),
        }
    }

    #[cfg(feature = "wasm-compile")]
    #[test]
    fn test_repl_wasm_gene_constructor() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64\n has y: i64 }").unwrap();
        repl.eval("pub fun test() -> i64 { let p = Point { x: 10, y: 20 }\n p.x }")
            .unwrap();

        let result = repl.eval(":wasm");
        match result {
            Ok(EvalResult::WasmInfo {
                size_bytes,
                functions,
                has_memory,
            }) => {
                assert!(size_bytes > 0);
                assert!(functions >= 1);
                assert!(has_memory);
            }
            Err(e) => panic!("WASM compilation failed: {:?}", e),
            _ => panic!("Expected WasmInfo result"),
        }
    }

    #[cfg(feature = "wasm-compile")]
    #[test]
    fn test_repl_wasm_float_operations() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Vector2D { has dx: f64\n has dy: f64 }")
            .unwrap();
        repl.eval(
            "pub fun magnitude() -> f64 { let v = Vector2D { dx: 3.0, dy: 4.0 }\n v.dx + v.dy }",
        )
        .unwrap();

        let result = repl.eval(":wasm");
        match result {
            Ok(EvalResult::WasmInfo { size_bytes, .. }) => {
                assert!(size_bytes > 0);
            }
            Err(e) => panic!("WASM compilation with floats failed: {:?}", e),
            _ => panic!("Expected WasmInfo result"),
        }
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_repl_expression_evaluation_integer() {
        let mut repl = SpiritRepl::new();
        // Define a simple function that the expression can use
        repl.eval("pub fun add(a: i64, b: i64) -> i64 { a + b }")
            .unwrap();

        // Evaluate an expression that calls the function
        let result = repl.eval("add(2, 3)");
        match result {
            Ok(EvalResult::Expression { input, value }) => {
                assert_eq!(input, "add(2, 3)");
                assert_eq!(value, "5");
            }
            Err(e) => panic!("Expression evaluation failed: {:?}", e),
            other => panic!("Expected Expression result, got {:?}", other),
        }
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_repl_expression_evaluation_literal() {
        let mut repl = SpiritRepl::new();

        // Evaluate a simple literal
        let result = repl.eval("42");
        match result {
            Ok(EvalResult::Expression { input, value }) => {
                assert_eq!(input, "42");
                assert_eq!(value, "42");
            }
            Err(e) => panic!("Literal evaluation failed: {:?}", e),
            other => panic!("Expected Expression result, got {:?}", other),
        }
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_repl_expression_evaluation_float() {
        let mut repl = SpiritRepl::new();

        // Evaluate a float expression
        let result = repl.eval("3.14");
        match result {
            Ok(EvalResult::Expression { input, value }) => {
                assert_eq!(input, "3.14");
                assert!(value.starts_with("3.14"));
            }
            Err(e) => panic!("Float evaluation failed: {:?}", e),
            other => panic!("Expected Expression result, got {:?}", other),
        }
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_repl_expression_evaluation_arithmetic() {
        let mut repl = SpiritRepl::new();

        // Evaluate arithmetic
        let result = repl.eval("10 + 20 * 2");
        match result {
            Ok(EvalResult::Expression { value, .. }) => {
                assert_eq!(value, "50");
            }
            Err(e) => panic!("Arithmetic evaluation failed: {:?}", e),
            other => panic!("Expected Expression result, got {:?}", other),
        }
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_repl_expression_with_gene_field_access() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64\n has y: i64 }").unwrap();

        // Define a function that creates and accesses gene field
        // (inline constructor field access requires type inference)
        repl.eval("pub fun getX() -> i64 { let p = Point { x: 100, y: 200 }\n p.x }")
            .unwrap();

        // Evaluate the function call
        let result = repl.eval("getX()");
        match result {
            Ok(EvalResult::Expression { value, .. }) => {
                assert_eq!(value, "100");
            }
            Err(e) => panic!("Gene field access evaluation failed: {:?}", e),
            other => panic!("Expected Expression result, got {:?}", other),
        }
    }

    #[cfg(not(feature = "wasm-compile"))]
    #[test]
    fn test_repl_wasm_feature_disabled() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();

        let result = repl.eval(":wasm");
        assert!(matches!(result, Err(ReplError::Feature(_))));
    }

    // ==================== Load File ====================

    #[test]
    fn test_repl_load_no_arg() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":load");
        assert!(matches!(result, Err(ReplError::Command(_))));
    }

    #[test]
    fn test_repl_load_nonexistent_file() {
        let mut repl = SpiritRepl::new();
        let result = repl.eval(":load /nonexistent/file.dol");
        assert!(matches!(result, Err(ReplError::Io(_))));
    }

    // ==================== Error Types ====================

    #[test]
    fn test_repl_error_display() {
        let err = ReplError::Parse("test error".to_string());
        assert_eq!(format!("{}", err), "Parse error: test error");

        let err = ReplError::NotFound("Point".to_string());
        assert_eq!(format!("{}", err), "Not found: Point");

        let err = ReplError::Command("bad command".to_string());
        assert_eq!(format!("{}", err), "Command error: bad command");

        let err = ReplError::Codegen("gen error".to_string());
        assert_eq!(format!("{}", err), "Codegen error: gen error");

        let err = ReplError::Wasm("wasm error".to_string());
        assert_eq!(format!("{}", err), "WASM error: wasm error");

        let err = ReplError::Io("io error".to_string());
        assert_eq!(format!("{}", err), "I/O error: io error");

        let err = ReplError::Feature("missing".to_string());
        assert_eq!(format!("{}", err), "Feature error: missing");
    }

    // ==================== Public API ====================

    #[test]
    fn test_repl_declarations_accessor() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();
        repl.eval("gen Circle { has r: i64 }").unwrap();

        let decls = repl.declarations();
        assert_eq!(decls.len(), 2);
    }

    #[test]
    fn test_repl_history_accessor() {
        let mut repl = SpiritRepl::new();
        repl.eval("gen Point { has x: i64 }").unwrap();

        let history = repl.history();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_repl_config_accessor() {
        let config = SessionConfig::with_name("my-session");
        let repl = SpiritRepl::with_config(config);

        assert_eq!(repl.config().name, "my-session");
    }
}
