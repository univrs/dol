//! REPL Context - Maintains state across evaluations
//!
//! The context tracks:
//! - Defined symbols and their types
//! - Import graph for tree shaking
//! - Runtime state for expression evaluation

use std::collections::HashMap;

/// REPL context maintaining state across evaluations.
#[derive(Debug, Clone)]
pub struct ReplContext {
    /// Symbol table mapping names to their metadata
    symbols: HashMap<String, SymbolInfo>,

    /// Import graph for dependency tracking
    imports: HashMap<String, Vec<String>>,

    /// Root symbols that should not be tree-shaken
    roots: Vec<String>,
}

/// Information about a defined symbol.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Name of the symbol
    pub name: String,

    /// Kind of declaration (gene, trait, function, etc.)
    pub kind: String,

    /// Dependencies this symbol uses
    pub dependencies: Vec<String>,

    /// Whether this is a public symbol
    pub is_public: bool,

    /// Optional type signature
    pub type_sig: Option<String>,
}

impl Default for ReplContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            imports: HashMap::new(),
            roots: Vec::new(),
        }
    }

    /// Add a declaration to the context.
    pub fn add_declaration(&mut self, name: &str, kind: &str) {
        let info = SymbolInfo {
            name: name.to_string(),
            kind: kind.to_string(),
            dependencies: Vec::new(),
            is_public: true, // REPL declarations are public by default
            type_sig: None,
        };
        self.symbols.insert(name.to_string(), info);

        // Add as root (REPL definitions should be retained)
        if !self.roots.contains(&name.to_string()) {
            self.roots.push(name.to_string());
        }
    }

    /// Add a dependency between symbols.
    pub fn add_dependency(&mut self, from: &str, to: &str) {
        self.imports
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());

        if let Some(info) = self.symbols.get_mut(from) {
            if !info.dependencies.contains(&to.to_string()) {
                info.dependencies.push(to.to_string());
            }
        }
    }

    /// Get information about a symbol.
    pub fn get_symbol(&self, name: &str) -> Option<&SymbolInfo> {
        self.symbols.get(name)
    }

    /// Get all defined symbol names.
    pub fn symbol_names(&self) -> Vec<&str> {
        self.symbols.keys().map(|s| s.as_str()).collect()
    }

    /// Get all root symbols (entry points for tree shaking).
    pub fn roots(&self) -> &[String] {
        &self.roots
    }

    /// Remove a symbol from the context.
    pub fn remove_symbol(&mut self, name: &str) {
        self.symbols.remove(name);
        self.imports.remove(name);
        self.roots.retain(|r| r != name);
    }

    /// Clear all context state.
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.imports.clear();
        self.roots.clear();
    }

    /// Check if a symbol is defined.
    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Get dependencies for a symbol.
    pub fn get_dependencies(&self, name: &str) -> Option<&Vec<String>> {
        self.imports.get(name)
    }

    /// Add a type signature to a symbol.
    pub fn set_type_sig(&mut self, name: &str, sig: &str) {
        if let Some(info) = self.symbols.get_mut(name) {
            info.type_sig = Some(sig.to_string());
        }
    }

    /// Get symbol count.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if context is empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = ReplContext::new();
        assert!(ctx.is_empty());
        assert!(ctx.roots().is_empty());
    }

    #[test]
    fn test_add_declaration() {
        let mut ctx = ReplContext::new();
        ctx.add_declaration("Point", "gene");

        assert!(ctx.has_symbol("Point"));
        assert_eq!(ctx.len(), 1);

        let info = ctx.get_symbol("Point").unwrap();
        assert_eq!(info.name, "Point");
        assert_eq!(info.kind, "gene");
    }

    #[test]
    fn test_add_dependency() {
        let mut ctx = ReplContext::new();
        ctx.add_declaration("Point", "gene");
        ctx.add_declaration("Circle", "gene");
        ctx.add_dependency("Circle", "Point");

        let deps = ctx.get_dependencies("Circle").unwrap();
        assert!(deps.contains(&"Point".to_string()));
    }

    #[test]
    fn test_remove_symbol() {
        let mut ctx = ReplContext::new();
        ctx.add_declaration("Point", "gene");
        assert!(ctx.has_symbol("Point"));

        ctx.remove_symbol("Point");
        assert!(!ctx.has_symbol("Point"));
    }

    #[test]
    fn test_clear() {
        let mut ctx = ReplContext::new();
        ctx.add_declaration("Point", "gene");
        ctx.add_declaration("Circle", "gene");

        ctx.clear();
        assert!(ctx.is_empty());
    }
}
