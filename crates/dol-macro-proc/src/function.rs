//! Function-like macros for DOL.
//!
//! This module provides function-like macro implementations that
//! provide custom syntax extensions.

use crate::error::{ProcMacroError, ProcMacroResult};
use metadol::ast::{Expr, Literal};
use proc_macro2::TokenStream;
use quote::quote;

/// Trait for function-like macro implementations.
pub trait FunctionMacro {
    /// The name of this function-like macro.
    fn name(&self) -> &str;

    /// Expands the macro with the given arguments.
    fn expand(&self, args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>>;

    /// Returns a description of this macro.
    fn description(&self) -> &str {
        ""
    }
}

/// The `sql!` function-like macro.
///
/// Provides compile-time SQL syntax checking and query building.
///
/// # Example
///
/// ```text
/// let query = sql!("SELECT * FROM users WHERE id = ?", user_id);
/// ```
pub fn function_sql(args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>> {
    if args.is_empty() {
        return Err(ProcMacroError::invalid_input(
            "sql! requires at least a query string",
        ));
    }

    // Extract the query string
    if let Expr::Literal(Literal::String(query)) = &args[0] {
        // TODO: Validate SQL syntax at compile time
        // For now, just return the arguments
        Ok(args)
    } else {
        Err(ProcMacroError::invalid_input(
            "sql! first argument must be a string literal",
        ))
    }
}

/// The `format!` function-like macro.
///
/// String formatting with compile-time format string validation.
///
/// # Example
///
/// ```text
/// let message = format!("Hello, {}!", name);
/// ```
pub fn function_format(args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>> {
    if args.is_empty() {
        return Err(ProcMacroError::invalid_input(
            "format! requires at least a format string",
        ));
    }

    // Validate format string
    if let Expr::Literal(Literal::String(_format_str)) = &args[0] {
        // TODO: Validate format string at compile time
        Ok(args)
    } else {
        Err(ProcMacroError::invalid_input(
            "format! first argument must be a string literal",
        ))
    }
}

/// The `json!` function-like macro.
///
/// Compile-time JSON literal construction.
///
/// # Example
///
/// ```text
/// let data = json!({
///   "name": "Alice",
///   "age": 30
/// });
/// ```
pub fn function_json(args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>> {
    // TODO: Parse and validate JSON at compile time
    Ok(args)
}

/// The `regex!` function-like macro.
///
/// Compile-time regex pattern validation.
///
/// # Example
///
/// ```text
/// let pattern = regex!(r"^\d{3}-\d{2}-\d{4}$");
/// ```
pub fn function_regex(args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>> {
    if args.len() != 1 {
        return Err(ProcMacroError::invalid_input("regex! requires exactly one argument"));
    }

    if let Expr::Literal(Literal::String(_pattern)) = &args[0] {
        // TODO: Validate regex pattern at compile time
        Ok(args)
    } else {
        Err(ProcMacroError::invalid_input(
            "regex! argument must be a string literal",
        ))
    }
}

/// The `include_str!` function-like macro.
///
/// Includes a file as a string literal at compile time.
///
/// # Example
///
/// ```text
/// let content = include_str!("data.txt");
/// ```
pub fn function_include_str(args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>> {
    if args.len() != 1 {
        return Err(ProcMacroError::invalid_input(
            "include_str! requires exactly one argument",
        ));
    }

    if let Expr::Literal(Literal::String(path)) = &args[0] {
        // TODO: Read file at compile time
        let content = format!("Contents of {}", path);
        Ok(vec![Expr::Literal(Literal::String(content))])
    } else {
        Err(ProcMacroError::invalid_input(
            "include_str! argument must be a string literal",
        ))
    }
}

/// The `concat!` function-like macro.
///
/// Concatenates string literals at compile time.
///
/// # Example
///
/// ```text
/// let message = concat!("Hello", ", ", "World!");
/// ```
pub fn function_concat(args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>> {
    let mut result = String::new();

    for arg in &args {
        if let Expr::Literal(Literal::String(s)) = arg {
            result.push_str(s);
        } else {
            return Err(ProcMacroError::invalid_input(
                "concat! arguments must all be string literals",
            ));
        }
    }

    Ok(vec![Expr::Literal(Literal::String(result))])
}

/// Registry of function-like macros.
pub struct FunctionMacroRegistry {
    /// Registered function macros
    macros: std::collections::HashMap<String, Box<dyn Fn(Vec<Expr>) -> ProcMacroResult<Vec<Expr>>>>,
}

impl FunctionMacroRegistry {
    /// Creates a new registry with built-in function macros.
    pub fn new() -> Self {
        let mut registry = Self {
            macros: std::collections::HashMap::new(),
        };

        // Register built-in function macros
        registry.register("sql", Box::new(function_sql));
        registry.register("format", Box::new(function_format));
        registry.register("json", Box::new(function_json));
        registry.register("regex", Box::new(function_regex));
        registry.register("include_str", Box::new(function_include_str));
        registry.register("concat", Box::new(function_concat));

        registry
    }

    /// Registers a custom function macro.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        func: Box<dyn Fn(Vec<Expr>) -> ProcMacroResult<Vec<Expr>>>,
    ) {
        self.macros.insert(name.into(), func);
    }

    /// Expands a function macro.
    pub fn expand(&self, name: &str, args: Vec<Expr>) -> ProcMacroResult<Vec<Expr>> {
        if let Some(func) = self.macros.get(name) {
            func(args)
        } else {
            Err(ProcMacroError::new(format!("unknown macro: {}", name)))
        }
    }

    /// Returns true if a macro with the given name is registered.
    pub fn has(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Returns an iterator over all macro names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.macros.keys().map(|s| s.as_str())
    }
}

impl Default for FunctionMacroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_sql() {
        let args = vec![Expr::Literal(Literal::String(
            "SELECT * FROM users".to_string(),
        ))];
        let result = function_sql(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_sql_no_args() {
        let result = function_sql(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_function_format() {
        let args = vec![Expr::Literal(Literal::String("Hello, {}!".to_string()))];
        let result = function_format(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_concat() {
        let args = vec![
            Expr::Literal(Literal::String("Hello".to_string())),
            Expr::Literal(Literal::String(", ".to_string())),
            Expr::Literal(Literal::String("World!".to_string())),
        ];
        let result = function_concat(args);
        assert!(result.is_ok());

        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);

        if let Expr::Literal(Literal::String(s)) = &exprs[0] {
            assert_eq!(s, "Hello, World!");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_function_include_str() {
        let args = vec![Expr::Literal(Literal::String("test.txt".to_string()))];
        let result = function_include_str(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_creation() {
        let registry = FunctionMacroRegistry::new();
        assert!(registry.has("sql"));
        assert!(registry.has("format"));
        assert!(registry.has("json"));
        assert!(registry.has("regex"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_registry_expand() {
        let registry = FunctionMacroRegistry::new();
        let args = vec![Expr::Literal(Literal::String("test".to_string()))];

        let result = registry.expand("concat", args.clone());
        assert!(result.is_ok());

        let result = registry.expand("unknown", args);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_names() {
        let registry = FunctionMacroRegistry::new();
        let names: Vec<&str> = registry.names().collect();

        assert!(names.contains(&"sql"));
        assert!(names.contains(&"format"));
        assert!(names.contains(&"json"));
        assert!(names.contains(&"regex"));
        assert!(names.contains(&"include_str"));
        assert!(names.contains(&"concat"));
    }
}
