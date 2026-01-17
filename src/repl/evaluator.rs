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

    /// Evaluate an expression by wrapping it in a function.
    pub fn eval_expression(&mut self, expr: &str, declarations: &str) -> Result<String, EvalError> {
        // Build source with expression wrapped in __repl_eval__ function
        let source = format!(
            r#"{}

fun __repl_eval__() -> Int64 {{
    {}
}}
"#,
            declarations, expr
        );

        // Compile
        let wasm = self.compile_to_wasm(&source)?;

        // Execute
        let results = self.execute_wasm(&wasm, "__repl_eval__", &[])?;

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
}
