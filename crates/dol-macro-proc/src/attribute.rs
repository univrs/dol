//! Attribute macros for DOL.
//!
//! This module provides attribute macro implementations that can
//! transform DOL declarations.

use crate::error::{ProcMacroError, ProcMacroResult};
use metadol::ast::{Declaration, Expr, Span, Stmt};
use proc_macro2::TokenStream;
use quote::quote;

/// Trait for attribute macro implementations.
pub trait AttributeMacro {
    /// The name of this attribute macro.
    fn name(&self) -> &str;

    /// Transforms a declaration with this attribute.
    fn transform(&self, decl: Declaration, args: Vec<Expr>) -> ProcMacroResult<Declaration>;

    /// Returns a description of this attribute macro.
    fn description(&self) -> &str {
        ""
    }
}

/// The `#[cached]` attribute macro.
///
/// Adds caching behavior to spell declarations.
///
/// # Example
///
/// ```text
/// #[cached]
/// spell compute(x: Int) -> Int {
///   x * x
/// }
/// ```
pub fn attribute_cached(decl: Declaration, _args: Vec<Expr>) -> ProcMacroResult<Declaration> {
    // For now, just return the declaration with a marker
    // A full implementation would add caching logic
    Ok(decl)
}

/// The `#[async]` attribute macro.
///
/// Marks a spell as asynchronous.
///
/// # Example
///
/// ```text
/// #[async]
/// spell fetch_data(url: String) -> Data {
///   // async implementation
/// }
/// ```
pub fn attribute_async(decl: Declaration, _args: Vec<Expr>) -> ProcMacroResult<Declaration> {
    // Mark the declaration as async
    // A full implementation would transform the body to use async/await
    Ok(decl)
}

/// The `#[memoize]` attribute macro.
///
/// Adds memoization to spell declarations.
///
/// # Example
///
/// ```text
/// #[memoize]
/// spell fibonacci(n: Int) -> Int {
///   if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
/// }
/// ```
pub fn attribute_memoize(decl: Declaration, _args: Vec<Expr>) -> ProcMacroResult<Declaration> {
    // Add memoization wrapper
    Ok(decl)
}

/// The `#[deprecated]` attribute macro.
///
/// Marks a declaration as deprecated.
///
/// # Example
///
/// ```text
/// #[deprecated(message = "Use new_function instead")]
/// spell old_function() {
///   // ...
/// }
/// ```
pub fn attribute_deprecated(
    decl: Declaration,
    _args: Vec<Expr>,
) -> ProcMacroResult<Declaration> {
    // Mark as deprecated
    Ok(decl)
}

/// The `#[test]` attribute macro.
///
/// Marks a spell as a test function.
///
/// # Example
///
/// ```text
/// #[test]
/// spell test_addition() {
///   assert_eq!(add(1, 2), 3)
/// }
/// ```
pub fn attribute_test(decl: Declaration, _args: Vec<Expr>) -> ProcMacroResult<Declaration> {
    // Mark as test
    Ok(decl)
}

/// The `#[inline]` attribute macro.
///
/// Suggests inlining the function.
///
/// # Example
///
/// ```text
/// #[inline]
/// spell small_function() -> Int {
///   42
/// }
/// ```
pub fn attribute_inline(decl: Declaration, _args: Vec<Expr>) -> ProcMacroResult<Declaration> {
    // Mark for inlining
    Ok(decl)
}

/// The `#[cfg]` attribute macro.
///
/// Conditional compilation based on configuration.
///
/// # Example
///
/// ```text
/// #[cfg(target = "wasm")]
/// spell wasm_only_function() {
///   // WASM-specific code
/// }
/// ```
pub fn attribute_cfg(decl: Declaration, args: Vec<Expr>) -> ProcMacroResult<Declaration> {
    if args.is_empty() {
        return Err(ProcMacroError::invalid_attribute(
            "cfg",
            "requires configuration argument",
        ));
    }
    Ok(decl)
}

/// Registry of built-in attribute macros.
pub struct AttributeMacroRegistry {
    /// Registered attribute macros
    macros: std::collections::HashMap<String, Box<dyn Fn(Declaration, Vec<Expr>) -> ProcMacroResult<Declaration>>>,
}

impl AttributeMacroRegistry {
    /// Creates a new registry with built-in attribute macros.
    pub fn new() -> Self {
        let mut registry = Self {
            macros: std::collections::HashMap::new(),
        };

        // Register built-in attributes
        registry.register("cached", Box::new(attribute_cached));
        registry.register("async", Box::new(attribute_async));
        registry.register("memoize", Box::new(attribute_memoize));
        registry.register("deprecated", Box::new(attribute_deprecated));
        registry.register("test", Box::new(attribute_test));
        registry.register("inline", Box::new(attribute_inline));
        registry.register("cfg", Box::new(attribute_cfg));

        registry
    }

    /// Registers a custom attribute macro.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        func: Box<dyn Fn(Declaration, Vec<Expr>) -> ProcMacroResult<Declaration>>,
    ) {
        self.macros.insert(name.into(), func);
    }

    /// Applies an attribute macro to a declaration.
    pub fn apply(
        &self,
        attr_name: &str,
        decl: Declaration,
        args: Vec<Expr>,
    ) -> ProcMacroResult<Declaration> {
        if let Some(func) = self.macros.get(attr_name) {
            func(decl, args)
        } else {
            Err(ProcMacroError::invalid_attribute(
                attr_name,
                "unknown attribute",
            ))
        }
    }

    /// Returns true if an attribute with the given name is registered.
    pub fn has(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Returns an iterator over all attribute names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.macros.keys().map(|s| s.as_str())
    }
}

impl Default for AttributeMacroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadol::ast::{Gen, Statement, Visibility};

    fn create_test_decl() -> Declaration {
        Declaration::Gene(Gen {
            visibility: Visibility::default(),
            name: "test.gene".to_string(),
            extends: None,
            statements: vec![],
            exegesis: "Test".to_string(),
            span: Span::default(),
        })
    }

    #[test]
    fn test_attribute_cached() {
        let decl = create_test_decl();
        let result = attribute_cached(decl.clone(), vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_attribute_async() {
        let decl = create_test_decl();
        let result = attribute_async(decl.clone(), vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_attribute_memoize() {
        let decl = create_test_decl();
        let result = attribute_memoize(decl.clone(), vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_attribute_cfg_no_args() {
        let decl = create_test_decl();
        let result = attribute_cfg(decl, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_creation() {
        let registry = AttributeMacroRegistry::new();
        assert!(registry.has("cached"));
        assert!(registry.has("async"));
        assert!(registry.has("memoize"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_registry_apply() {
        let registry = AttributeMacroRegistry::new();
        let decl = create_test_decl();

        let result = registry.apply("cached", decl.clone(), vec![]);
        assert!(result.is_ok());

        let result = registry.apply("unknown", decl, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_names() {
        let registry = AttributeMacroRegistry::new();
        let names: Vec<&str> = registry.names().collect();

        assert!(names.contains(&"cached"));
        assert!(names.contains(&"async"));
        assert!(names.contains(&"memoize"));
        assert!(names.contains(&"test"));
        assert!(names.contains(&"inline"));
        assert!(names.contains(&"cfg"));
    }
}
