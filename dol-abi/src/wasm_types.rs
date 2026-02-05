//! WASM value types and host function definitions

use serde::{Deserialize, Serialize};

/// WASM value types used in function signatures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmType {
    /// 32-bit integer (i32)
    I32,
    /// 64-bit integer (i64)
    I64,
    /// 32-bit float (f32)
    F32,
    /// 64-bit float (f64)
    F64,
}

impl std::fmt::Display for WasmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmType::I32 => write!(f, "i32"),
            WasmType::I64 => write!(f, "i64"),
            WasmType::F32 => write!(f, "f32"),
            WasmType::F64 => write!(f, "f64"),
        }
    }
}

/// Host function signature definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HostFunctionSignature {
    /// Parameter types
    pub params: Vec<WasmType>,
    /// Return type (None for void)
    pub returns: Option<WasmType>,
}

impl HostFunctionSignature {
    /// Create a new function signature
    pub fn new(params: Vec<WasmType>, returns: Option<WasmType>) -> Self {
        Self { params, returns }
    }

    /// Create a signature with no parameters and no return value
    pub fn void() -> Self {
        Self {
            params: Vec::new(),
            returns: None,
        }
    }

    /// Create a signature with parameters and no return value
    pub fn with_params(params: Vec<WasmType>) -> Self {
        Self {
            params,
            returns: None,
        }
    }

    /// Create a signature with parameters and a return value
    pub fn with_return(params: Vec<WasmType>, returns: WasmType) -> Self {
        Self {
            params,
            returns: Some(returns),
        }
    }
}

impl std::fmt::Display for HostFunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        write!(f, ")")?;
        if let Some(ret) = &self.returns {
            write!(f, " -> {}", ret)?;
        }
        Ok(())
    }
}

/// Host function definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HostFunction {
    /// Function name
    pub name: String,
    /// Function signature
    pub signature: HostFunctionSignature,
    /// Function category
    pub category: HostFunctionCategory,
}

/// Categories of host functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HostFunctionCategory {
    /// I/O functions (print, println, log, error)
    IO,
    /// Memory management (alloc, free, realloc)
    Memory,
    /// Time functions (now, sleep, monotonic_now)
    Time,
    /// Messaging functions (send, recv, pending, broadcast, free_message)
    Messaging,
    /// Random functions (random, random_bytes)
    Random,
    /// Effect functions (emit_effect, subscribe)
    Effects,
    /// Debug functions (breakpoint, assert, panic)
    Debug,
}

impl HostFunction {
    /// Create a new host function
    pub fn new(
        name: impl Into<String>,
        signature: HostFunctionSignature,
        category: HostFunctionCategory,
    ) -> Self {
        Self {
            name: name.into(),
            signature,
            category,
        }
    }

    /// Get the function's fully qualified import name
    pub fn import_name(&self) -> String {
        format!("vudo_{}", self.name)
    }
}

/// Standard host functions defined by the DOL ABI
pub fn standard_host_functions() -> Vec<HostFunction> {
    use HostFunctionCategory::*;
    use WasmType::*;

    vec![
        // I/O Functions
        HostFunction::new(
            "print",
            HostFunctionSignature::with_params(vec![I32, I32]),
            IO,
        ),
        HostFunction::new(
            "println",
            HostFunctionSignature::with_params(vec![I32, I32]),
            IO,
        ),
        HostFunction::new(
            "log",
            HostFunctionSignature::with_params(vec![I32, I32, I32]),
            IO,
        ),
        HostFunction::new(
            "error",
            HostFunctionSignature::with_params(vec![I32, I32]),
            IO,
        ),
        // Memory Functions
        HostFunction::new(
            "alloc",
            HostFunctionSignature::with_return(vec![I32], I32),
            Memory,
        ),
        HostFunction::new(
            "free",
            HostFunctionSignature::with_params(vec![I32, I32]),
            Memory,
        ),
        HostFunction::new(
            "realloc",
            HostFunctionSignature::with_return(vec![I32, I32, I32], I32),
            Memory,
        ),
        // Time Functions
        HostFunction::new("now", HostFunctionSignature::with_return(vec![], I64), Time),
        HostFunction::new(
            "sleep",
            HostFunctionSignature::with_params(vec![I32]),
            Time,
        ),
        HostFunction::new(
            "monotonic_now",
            HostFunctionSignature::with_return(vec![], I64),
            Time,
        ),
        // Messaging Functions
        HostFunction::new(
            "send",
            HostFunctionSignature::with_return(vec![I32, I32, I32, I32], I32),
            Messaging,
        ),
        HostFunction::new(
            "recv",
            HostFunctionSignature::with_return(vec![], I32),
            Messaging,
        ),
        HostFunction::new(
            "pending",
            HostFunctionSignature::with_return(vec![], I32),
            Messaging,
        ),
        HostFunction::new(
            "broadcast",
            HostFunctionSignature::with_return(vec![I32, I32], I32),
            Messaging,
        ),
        HostFunction::new(
            "free_message",
            HostFunctionSignature::with_params(vec![I32]),
            Messaging,
        ),
        // Random Functions
        HostFunction::new(
            "random",
            HostFunctionSignature::with_return(vec![], F64),
            Random,
        ),
        HostFunction::new(
            "random_bytes",
            HostFunctionSignature::with_params(vec![I32, I32]),
            Random,
        ),
        // Effect Functions
        HostFunction::new(
            "emit_effect",
            HostFunctionSignature::with_return(vec![I32, I32, I32], I32),
            Effects,
        ),
        HostFunction::new(
            "subscribe",
            HostFunctionSignature::with_return(vec![I32, I32], I32),
            Effects,
        ),
        // Debug Functions
        HostFunction::new("breakpoint", HostFunctionSignature::void(), Debug),
        HostFunction::new(
            "assert",
            HostFunctionSignature::with_params(vec![I32, I32, I32]),
            Debug,
        ),
        HostFunction::new(
            "panic",
            HostFunctionSignature::with_params(vec![I32, I32]),
            Debug,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_type_display() {
        assert_eq!(WasmType::I32.to_string(), "i32");
        assert_eq!(WasmType::I64.to_string(), "i64");
        assert_eq!(WasmType::F32.to_string(), "f32");
        assert_eq!(WasmType::F64.to_string(), "f64");
    }

    #[test]
    fn test_signature_display() {
        let sig = HostFunctionSignature::void();
        assert_eq!(sig.to_string(), "()");

        let sig = HostFunctionSignature::with_params(vec![WasmType::I32, WasmType::I32]);
        assert_eq!(sig.to_string(), "(i32, i32)");

        let sig = HostFunctionSignature::with_return(vec![WasmType::I32], WasmType::I64);
        assert_eq!(sig.to_string(), "(i32) -> i64");
    }

    #[test]
    fn test_standard_host_functions() {
        let funcs = standard_host_functions();
        assert_eq!(funcs.len(), 22);

        // Verify specific functions
        let print_fn = funcs.iter().find(|f| f.name == "print").unwrap();
        assert_eq!(print_fn.category, HostFunctionCategory::IO);
        assert_eq!(print_fn.signature.params.len(), 2);
        assert_eq!(print_fn.signature.returns, None);

        let alloc_fn = funcs.iter().find(|f| f.name == "alloc").unwrap();
        assert_eq!(alloc_fn.category, HostFunctionCategory::Memory);
        assert_eq!(alloc_fn.signature.returns, Some(WasmType::I32));
    }

    #[test]
    fn test_import_name() {
        let func = HostFunction::new(
            "print",
            HostFunctionSignature::void(),
            HostFunctionCategory::IO,
        );
        assert_eq!(func.import_name(), "vudo_print");
    }
}
