//! REPL Evaluator - Compiles and executes DOL code
//!
//! Provides the evaluation pipeline:
//! 1. Parse DOL input
//! 2. Lower to HIR
//! 3. Tree shake unused code
//! 4. Generate Rust code
//! 5. Compile to WASM
//! 6. Execute via wasmtime
//!
//! For expressions, we wrap them in a temporary function and extract the result.

use std::collections::HashMap;

/// Result of REPL evaluation.
#[derive(Debug, Clone)]
pub enum EvalResult {
    /// Empty input (no-op)
    Empty,

    /// Declaration was defined
    Defined {
        /// Name of the declaration
        name: String,
        /// Kind of declaration (gene, fun, etc.)
        kind: String,
        /// Status message
        message: String,
    },

    /// Expression was evaluated
    Expression {
        /// The input expression
        input: String,
        /// The evaluated value as a string
        value: String,
    },

    /// REPL command help
    Help(String),

    /// Quit signal
    Quit,

    /// Generic message
    Message(String),

    /// Type information
    TypeInfo(String),

    /// Emitted Rust code
    RustCode(String),

    /// WASM compilation info
    WasmInfo {
        /// Size of WASM binary in bytes
        size_bytes: usize,
        /// Number of functions
        functions: usize,
        /// Whether module uses memory
        has_memory: bool,
    },
}

/// Evaluator for DOL expressions and declarations.
///
/// Handles the actual compilation and execution of DOL code,
/// building on top of the existing compiler infrastructure.
#[derive(Debug)]
pub struct ReplEvaluator {
    /// Cached WASM modules by declaration set hash
    wasm_cache: HashMap<u64, Vec<u8>>,

    /// Optimization level
    optimize: bool,

    /// Include debug info
    debug_info: bool,
}

impl Default for ReplEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplEvaluator {
    /// Create a new evaluator with default settings.
    pub fn new() -> Self {
        Self {
            wasm_cache: HashMap::new(),
            optimize: false,
            debug_info: true,
        }
    }

    /// Create an optimizing evaluator.
    pub fn optimized() -> Self {
        Self {
            wasm_cache: HashMap::new(),
            optimize: true,
            debug_info: false,
        }
    }

    /// Enable or disable optimization.
    pub fn with_optimization(mut self, enable: bool) -> Self {
        self.optimize = enable;
        self
    }

    /// Enable or disable debug info.
    pub fn with_debug_info(mut self, enable: bool) -> Self {
        self.debug_info = enable;
        self
    }

    /// Compile DOL source to WASM bytes.
    #[cfg(feature = "wasm-compile")]
    pub fn compile_to_wasm(&mut self, source: &str) -> Result<Vec<u8>, EvalError> {
        use crate::wasm::WasmCompiler;

        let file = crate::parse_dol_file(source).map_err(|e| EvalError::Parse(e.to_string()))?;

        let mut compiler = WasmCompiler::new()
            .with_optimization(self.optimize)
            .with_debug_info(self.debug_info);

        compiler
            .compile_file(&file)
            .map_err(|e| EvalError::Compile(e.message))
    }

    /// Compile DOL source to WASM bytecode.
    ///
    /// This is a stub when the `wasm-compile` feature is not enabled.
    /// Enable the feature to use actual WASM compilation.
    #[cfg(not(feature = "wasm-compile"))]
    pub fn compile_to_wasm(&mut self, _source: &str) -> Result<Vec<u8>, EvalError> {
        Err(EvalError::Feature(
            "wasm-compile feature not enabled".to_string(),
        ))
    }

    /// Execute a WASM module and call a specific function.
    #[cfg(feature = "wasm")]
    pub fn execute_wasm(
        &self,
        wasm_bytes: &[u8],
        function: &str,
        args: &[WasmValue],
    ) -> Result<Vec<WasmValue>, EvalError> {
        use wasmtime::{Engine, Instance, Module, Store, Val};

        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes)
            .map_err(|e| EvalError::Runtime(format!("Failed to load WASM: {}", e)))?;

        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| EvalError::Runtime(format!("Failed to instantiate: {}", e)))?;

        let func = instance
            .get_func(&mut store, function)
            .ok_or_else(|| EvalError::Runtime(format!("Function '{}' not found", function)))?;

        // Convert args
        let wasm_args: Vec<Val> = args.iter().map(|a| a.to_wasmtime_val()).collect();

        // Prepare results
        let func_ty = func.ty(&store);
        let mut results: Vec<Val> = func_ty.results().map(|_| Val::I64(0)).collect();

        // Call
        func.call(&mut store, &wasm_args, &mut results)
            .map_err(|e| EvalError::Runtime(format!("Execution error: {}", e)))?;

        // Convert results
        Ok(results.iter().map(WasmValue::from_wasmtime_val).collect())
    }

    /// Execute WASM (stub when wasm feature is not enabled).
    #[cfg(not(feature = "wasm"))]
    pub fn execute_wasm(
        &self,
        _wasm_bytes: &[u8],
        _function: &str,
        _args: &[WasmValue],
    ) -> Result<Vec<WasmValue>, EvalError> {
        Err(EvalError::Feature("wasm feature not enabled".to_string()))
    }

    /// Infer the type of an expression based on its structure.
    ///
    /// Returns the DOL type string (i64, f64, i32, bool) for the expression.
    pub fn infer_expression_type(&self, expr: &str) -> &'static str {
        let expr = expr.trim();

        // Check for boolean literals
        if expr == "true" || expr == "false" {
            return "bool";
        }

        // Check for float literals (contains '.' followed by digits)
        if expr.contains('.') && expr.chars().any(|c| c.is_ascii_digit()) {
            // Could be a float literal like 3.14
            let is_float = expr
                .chars()
                .filter(|c| *c != '.' && *c != '-')
                .all(|c| c.is_ascii_digit() || c == 'e' || c == 'E' || c == '+' || c == '_');
            if is_float {
                return "f64";
            }
        }

        // Check for simple integer literal
        let is_int = expr
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .all(|c| c.is_ascii_digit());
        if is_int && !expr.is_empty() {
            return "i64";
        }

        // Check for operators that produce floats
        if expr.contains("f64") || expr.contains("Float64") {
            return "f64";
        }

        // Check for float operations (simple heuristic: if expression contains a float literal)
        for token in expr.split(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '_') {
            if token.contains('.') {
                let is_float = token
                    .chars()
                    .filter(|c| *c != '.')
                    .all(|c| c.is_ascii_digit() || c == 'e' || c == 'E' || c == '_');
                if is_float && !token.is_empty() {
                    return "f64";
                }
            }
        }

        // Default to i64
        "i64"
    }

    /// Evaluate an expression by wrapping it in a function.
    ///
    /// The expression is wrapped in a `dolReplEval` function, compiled to WASM,
    /// and executed. The result is returned as a formatted string.
    pub fn eval_expression(&mut self, expr: &str, declarations: &str) -> Result<String, EvalError> {
        // Infer the return type of the expression
        let return_type = self.infer_expression_type(expr);

        // Build source with expression wrapped in dolReplEval function
        // Use v0.8.0 syntax (pub fun, i64, f64, etc.)
        // Note: DOL identifiers cannot start with underscore
        let source = format!(
            r#"{}

pub fun dolReplEval() -> {} {{
    {}
}}
"#,
            declarations, return_type, expr
        );

        // Compile
        let wasm = self.compile_to_wasm(&source)?;

        // Execute
        let results = self.execute_wasm(&wasm, "dolReplEval", &[])?;

        // Format result
        if results.is_empty() {
            Ok("()".to_string())
        } else {
            Ok(results
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", "))
        }
    }

    /// Evaluate an expression with explicit return type.
    ///
    /// Use this when you know the expected return type of the expression.
    pub fn eval_expression_typed(
        &mut self,
        expr: &str,
        declarations: &str,
        return_type: &str,
    ) -> Result<String, EvalError> {
        // Build source with expression wrapped in dolReplEval function
        let source = format!(
            r#"{}

pub fun dolReplEval() -> {} {{
    {}
}}
"#,
            declarations, return_type, expr
        );

        // Compile
        let wasm = self.compile_to_wasm(&source)?;

        // Execute
        let results = self.execute_wasm(&wasm, "dolReplEval", &[])?;

        // Format result
        if results.is_empty() {
            Ok("()".to_string())
        } else {
            Ok(results
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", "))
        }
    }

    /// Clear the WASM cache.
    pub fn clear_cache(&mut self) {
        self.wasm_cache.clear();
    }
}

/// Evaluation error types.
#[derive(Debug, Clone)]
pub enum EvalError {
    /// Parse error
    Parse(String),
    /// Compilation error
    Compile(String),
    /// Runtime error
    Runtime(String),
    /// Feature not available
    Feature(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::Parse(msg) => write!(f, "Parse error: {}", msg),
            EvalError::Compile(msg) => write!(f, "Compile error: {}", msg),
            EvalError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            EvalError::Feature(msg) => write!(f, "Feature error: {}", msg),
        }
    }
}

impl std::error::Error for EvalError {}

/// WASM value wrapper for the REPL.
#[derive(Debug, Clone)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl WasmValue {
    /// Convert to wasmtime Val.
    #[cfg(feature = "wasm")]
    fn to_wasmtime_val(&self) -> wasmtime::Val {
        match self {
            WasmValue::I32(v) => wasmtime::Val::I32(*v),
            WasmValue::I64(v) => wasmtime::Val::I64(*v),
            WasmValue::F32(v) => wasmtime::Val::F32(v.to_bits()),
            WasmValue::F64(v) => wasmtime::Val::F64(v.to_bits()),
        }
    }

    /// Convert from wasmtime Val.
    #[cfg(feature = "wasm")]
    fn from_wasmtime_val(val: &wasmtime::Val) -> Self {
        match val {
            wasmtime::Val::I32(v) => WasmValue::I32(*v),
            wasmtime::Val::I64(v) => WasmValue::I64(*v),
            wasmtime::Val::F32(bits) => WasmValue::F32(f32::from_bits(*bits)),
            wasmtime::Val::F64(bits) => WasmValue::F64(f64::from_bits(*bits)),
            _ => WasmValue::I64(0), // Fallback
        }
    }
}

impl std::fmt::Display for WasmValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmValue::I32(v) => write!(f, "{}", v),
            WasmValue::I64(v) => write!(f, "{}", v),
            WasmValue::F32(v) => write!(f, "{}", v),
            WasmValue::F64(v) => write!(f, "{}", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_new() {
        let eval = ReplEvaluator::new();
        assert!(!eval.optimize);
        assert!(eval.debug_info);
    }

    #[test]
    fn test_evaluator_optimized() {
        let eval = ReplEvaluator::optimized();
        assert!(eval.optimize);
        assert!(!eval.debug_info);
    }

    #[test]
    fn test_wasm_value_display() {
        assert_eq!(format!("{}", WasmValue::I32(42)), "42");
        assert_eq!(format!("{}", WasmValue::I64(100)), "100");
        assert_eq!(format!("{}", WasmValue::F32(3.14)), "3.14");
        assert_eq!(format!("{}", WasmValue::F64(2.718)), "2.718");
    }

    #[test]
    fn test_infer_type_integer_literals() {
        let eval = ReplEvaluator::new();
        assert_eq!(eval.infer_expression_type("42"), "i64");
        assert_eq!(eval.infer_expression_type("0"), "i64");
        assert_eq!(eval.infer_expression_type("-1"), "i64");
        assert_eq!(eval.infer_expression_type("1_000_000"), "i64");
    }

    #[test]
    fn test_infer_type_float_literals() {
        let eval = ReplEvaluator::new();
        assert_eq!(eval.infer_expression_type("3.14"), "f64");
        assert_eq!(eval.infer_expression_type("0.0"), "f64");
        assert_eq!(eval.infer_expression_type("-1.5"), "f64");
        assert_eq!(eval.infer_expression_type("1.0e10"), "f64");
        assert_eq!(eval.infer_expression_type("2.5E-3"), "f64");
    }

    #[test]
    fn test_infer_type_boolean_literals() {
        let eval = ReplEvaluator::new();
        assert_eq!(eval.infer_expression_type("true"), "bool");
        assert_eq!(eval.infer_expression_type("false"), "bool");
    }

    #[test]
    fn test_infer_type_with_whitespace() {
        let eval = ReplEvaluator::new();
        assert_eq!(eval.infer_expression_type("  42  "), "i64");
        assert_eq!(eval.infer_expression_type("\t3.14\n"), "f64");
        assert_eq!(eval.infer_expression_type("  true  "), "bool");
    }

    #[test]
    fn test_infer_type_expressions_with_floats() {
        let eval = ReplEvaluator::new();
        // Expressions containing float literals
        assert_eq!(eval.infer_expression_type("1.0 + 2.0"), "f64");
        assert_eq!(eval.infer_expression_type("x + 3.14"), "f64");
        assert_eq!(eval.infer_expression_type("a * 0.5"), "f64");
    }

    #[test]
    fn test_infer_type_expressions_with_type_hints() {
        let eval = ReplEvaluator::new();
        // Type hints in expression
        assert_eq!(eval.infer_expression_type("x as f64"), "f64");
        assert_eq!(eval.infer_expression_type("Float64::from(x)"), "f64");
    }

    #[test]
    fn test_infer_type_complex_expressions_default_to_i64() {
        let eval = ReplEvaluator::new();
        // Complex expressions without clear type hints default to i64
        assert_eq!(eval.infer_expression_type("a + b"), "i64");
        assert_eq!(eval.infer_expression_type("foo(x, y)"), "i64");
        assert_eq!(eval.infer_expression_type("if x { 1 } else { 2 }"), "i64");
    }

    #[test]
    fn test_builder_methods() {
        let eval = ReplEvaluator::new()
            .with_optimization(true)
            .with_debug_info(false);
        assert!(eval.optimize);
        assert!(!eval.debug_info);

        let eval2 = ReplEvaluator::new()
            .with_optimization(false)
            .with_debug_info(true);
        assert!(!eval2.optimize);
        assert!(eval2.debug_info);
    }

    #[test]
    fn test_clear_cache() {
        let mut eval = ReplEvaluator::new();
        // Just ensure it doesn't panic
        eval.clear_cache();
        assert!(eval.wasm_cache.is_empty());
    }

    #[test]
    fn test_eval_error_display() {
        assert_eq!(
            format!("{}", EvalError::Parse("unexpected token".to_string())),
            "Parse error: unexpected token"
        );
        assert_eq!(
            format!("{}", EvalError::Compile("type mismatch".to_string())),
            "Compile error: type mismatch"
        );
        assert_eq!(
            format!("{}", EvalError::Runtime("division by zero".to_string())),
            "Runtime error: division by zero"
        );
        assert_eq!(
            format!("{}", EvalError::Feature("wasm not enabled".to_string())),
            "Feature error: wasm not enabled"
        );
    }

    #[test]
    fn test_eval_result_variants() {
        // Test that all EvalResult variants can be created
        let empty = EvalResult::Empty;
        assert!(matches!(empty, EvalResult::Empty));

        let defined = EvalResult::Defined {
            name: "foo".to_string(),
            kind: "gene".to_string(),
            message: "defined".to_string(),
        };
        assert!(matches!(defined, EvalResult::Defined { .. }));

        let expr = EvalResult::Expression {
            input: "1 + 2".to_string(),
            value: "3".to_string(),
        };
        assert!(matches!(expr, EvalResult::Expression { .. }));

        let help = EvalResult::Help("help text".to_string());
        assert!(matches!(help, EvalResult::Help(_)));

        let quit = EvalResult::Quit;
        assert!(matches!(quit, EvalResult::Quit));

        let msg = EvalResult::Message("hello".to_string());
        assert!(matches!(msg, EvalResult::Message(_)));

        let type_info = EvalResult::TypeInfo("i64".to_string());
        assert!(matches!(type_info, EvalResult::TypeInfo(_)));

        let rust_code = EvalResult::RustCode("fn main() {}".to_string());
        assert!(matches!(rust_code, EvalResult::RustCode(_)));

        let wasm_info = EvalResult::WasmInfo {
            size_bytes: 100,
            functions: 5,
            has_memory: true,
        };
        assert!(matches!(wasm_info, EvalResult::WasmInfo { .. }));
    }
}
