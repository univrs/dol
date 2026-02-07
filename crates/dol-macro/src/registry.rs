//! Macro registry for DOL.
//!
//! This module provides a registry for storing and looking up macro definitions.

use crate::declarative::DeclarativeMacro;
use crate::error::{MacroError, MacroResult};
use std::collections::HashMap;

/// Registry of macro definitions.
///
/// The registry stores both declarative and procedural macros,
/// allowing them to be looked up by name during expansion.
pub struct MacroRegistry {
    /// Declarative macros
    declarative: HashMap<String, DeclarativeMacro>,
}

impl MacroRegistry {
    /// Creates a new empty macro registry.
    pub fn new() -> Self {
        Self {
            declarative: HashMap::new(),
        }
    }

    /// Registers a declarative macro.
    ///
    /// If a macro with the same name already exists, it is replaced.
    pub fn register_declarative(&mut self, name: impl Into<String>, macro_def: DeclarativeMacro) {
        self.declarative.insert(name.into(), macro_def);
    }

    /// Looks up a declarative macro by name.
    pub fn get_declarative(&self, name: &str) -> Option<&DeclarativeMacro> {
        self.declarative.get(name)
    }

    /// Returns true if a declarative macro with the given name exists.
    pub fn has_declarative(&self, name: &str) -> bool {
        self.declarative.contains_key(name)
    }

    /// Returns an iterator over all declarative macro names.
    pub fn declarative_names(&self) -> impl Iterator<Item = &str> {
        self.declarative.keys().map(|s| s.as_str())
    }

    /// Returns the number of registered macros.
    pub fn len(&self) -> usize {
        self.declarative.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.declarative.is_empty()
    }

    /// Removes a declarative macro from the registry.
    pub fn remove_declarative(&mut self, name: &str) -> Option<DeclarativeMacro> {
        self.declarative.remove(name)
    }

    /// Clears all macros from the registry.
    pub fn clear(&mut self) {
        self.declarative.clear();
    }

    /// Exports all exported macros to a new registry.
    ///
    /// This is useful for creating a registry of macros to export
    /// from a module or library.
    pub fn export(&self) -> MacroRegistry {
        let mut exported = MacroRegistry::new();
        for (name, macro_def) in &self.declarative {
            if macro_def.is_exported() {
                exported.register_declarative(name.clone(), macro_def.clone());
            }
        }
        exported
    }

    /// Merges another registry into this one.
    ///
    /// Macros from the other registry overwrite macros with the same name
    /// in this registry.
    pub fn merge(&mut self, other: MacroRegistry) {
        self.declarative.extend(other.declarative);
    }
}

impl Default for MacroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::MacroRule;
    use crate::pattern::MacroPattern;
    use crate::declarative::MacroTemplate;
    use metadol::ast::{Expr, Literal};

    fn create_test_macro(name: &str) -> DeclarativeMacro {
        let pattern = MacroPattern::Empty;
        let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
        let rule = MacroRule::new(pattern, template);
        DeclarativeMacro::new(name, vec![rule])
    }

    #[test]
    fn test_registry_creation() {
        let registry = MacroRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_lookup() {
        let mut registry = MacroRegistry::new();
        let macro_def = create_test_macro("test");

        registry.register_declarative("test", macro_def);
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.has_declarative("test"));
        assert!(registry.get_declarative("test").is_some());
    }

    #[test]
    fn test_remove_macro() {
        let mut registry = MacroRegistry::new();
        let macro_def = create_test_macro("test");

        registry.register_declarative("test", macro_def);
        assert!(registry.has_declarative("test"));

        let removed = registry.remove_declarative("test");
        assert!(removed.is_some());
        assert!(!registry.has_declarative("test"));
    }

    #[test]
    fn test_clear() {
        let mut registry = MacroRegistry::new();
        registry.register_declarative("test1", create_test_macro("test1"));
        registry.register_declarative("test2", create_test_macro("test2"));

        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_export() {
        let mut registry = MacroRegistry::new();

        let exported_macro = DeclarativeMacro::exported("exported", vec![]);
        let private_macro = DeclarativeMacro::new("private", vec![]);

        registry.register_declarative("exported", exported_macro);
        registry.register_declarative("private", private_macro);

        let exported = registry.export();
        assert_eq!(exported.len(), 1);
        assert!(exported.has_declarative("exported"));
        assert!(!exported.has_declarative("private"));
    }

    #[test]
    fn test_merge() {
        let mut registry1 = MacroRegistry::new();
        let mut registry2 = MacroRegistry::new();

        registry1.register_declarative("test1", create_test_macro("test1"));
        registry2.register_declarative("test2", create_test_macro("test2"));

        registry1.merge(registry2);
        assert_eq!(registry1.len(), 2);
        assert!(registry1.has_declarative("test1"));
        assert!(registry1.has_declarative("test2"));
    }

    #[test]
    fn test_declarative_names() {
        let mut registry = MacroRegistry::new();
        registry.register_declarative("test1", create_test_macro("test1"));
        registry.register_declarative("test2", create_test_macro("test2"));

        let names: Vec<&str> = registry.declarative_names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"test1"));
        assert!(names.contains(&"test2"));
    }
}
