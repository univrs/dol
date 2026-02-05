//! Import tracking for WASM code generation
//!
//! This module tracks which host functions are actually used in the generated code
//! to minimize imports and optimize module size.

use dol_abi::{standard_host_functions, HostFunction};
use std::collections::{HashMap, HashSet};

/// Maps DOL prelude function names to host functions
pub struct ImportTracker {
    /// Map from prelude function name to host function
    function_map: HashMap<String, HostFunction>,
}

impl ImportTracker {
    /// Create a new import tracker
    pub fn new() -> Self {
        let mut function_map = HashMap::new();

        // Populate with standard host functions
        for host_fn in standard_host_functions() {
            // Map both the short name and the full import name
            function_map.insert(host_fn.name.clone(), host_fn.clone());
            function_map.insert(host_fn.import_name(), host_fn);
        }

        Self { function_map }
    }

    /// Get the host function for a prelude function name
    pub fn get_host_function(&self, name: &str) -> Option<&HostFunction> {
        self.function_map.get(name)
    }

    /// Check if a function name maps to a host function
    pub fn is_host_function(&self, name: &str) -> bool {
        self.function_map.contains_key(name)
    }

    /// Get all available host functions
    pub fn all_host_functions(&self) -> Vec<&HostFunction> {
        // Return unique host functions (avoid duplicates from short/full name mapping)
        let mut seen = HashSet::new();
        self.function_map
            .values()
            .filter(|f| seen.insert(&f.name))
            .collect()
    }
}

impl Default for ImportTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks which imports are actually used in the generated code
#[derive(Debug, Default)]
pub struct UsedImports {
    /// Set of used function names
    used: HashSet<String>,
}

impl UsedImports {
    /// Create a new used imports tracker
    pub fn new() -> Self {
        Self {
            used: HashSet::new(),
        }
    }

    /// Track a function call
    pub fn track_call(&mut self, name: impl Into<String>) {
        self.used.insert(name.into());
    }

    /// Mark a function as used
    pub fn mark_used(&mut self, name: impl Into<String>) {
        self.used.insert(name.into());
    }

    /// Check if a function is used
    pub fn is_used(&self, name: &str) -> bool {
        self.used.contains(name)
    }

    /// Get all used function names
    pub fn get_used_imports(&self) -> Vec<String> {
        let mut names: Vec<_> = self.used.iter().cloned().collect();
        names.sort(); // Deterministic ordering
        names
    }

    /// Get the number of used imports
    pub fn len(&self) -> usize {
        self.used.len()
    }

    /// Check if any imports are used
    pub fn is_empty(&self) -> bool {
        self.used.is_empty()
    }

    /// Clear all tracked imports
    pub fn clear(&mut self) {
        self.used.clear();
    }

    /// Filter host functions to only those that are used
    pub fn filter_used<'a>(
        &self,
        host_functions: &'a [HostFunction],
    ) -> Vec<&'a HostFunction> {
        host_functions
            .iter()
            .filter(|f| self.is_used(&f.name) || self.is_used(&f.import_name()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dol_abi::HostFunctionCategory;

    #[test]
    fn test_import_tracker_basic() {
        let tracker = ImportTracker::new();

        // Test standard functions
        assert!(tracker.is_host_function("print"));
        assert!(tracker.is_host_function("vudo_print"));
        assert!(tracker.is_host_function("alloc"));
        assert!(tracker.is_host_function("send"));

        // Test non-existent function
        assert!(!tracker.is_host_function("nonexistent"));
    }

    #[test]
    fn test_import_tracker_get_host_function() {
        let tracker = ImportTracker::new();

        // Test getting by short name
        let print_fn = tracker.get_host_function("print").unwrap();
        assert_eq!(print_fn.name, "print");
        assert_eq!(print_fn.category, HostFunctionCategory::IO);

        // Test getting by import name
        let print_fn2 = tracker.get_host_function("vudo_print").unwrap();
        assert_eq!(print_fn2.name, "print");

        // Test non-existent
        assert!(tracker.get_host_function("nonexistent").is_none());
    }

    #[test]
    fn test_import_tracker_all_functions() {
        let tracker = ImportTracker::new();
        let all_funcs = tracker.all_host_functions();

        // Should have 22 unique standard functions
        assert_eq!(all_funcs.len(), 22);

        // Verify no duplicates
        let mut names: Vec<_> = all_funcs.iter().map(|f| &f.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 22);
    }

    #[test]
    fn test_used_imports_basic() {
        let mut used = UsedImports::new();
        assert!(used.is_empty());
        assert_eq!(used.len(), 0);

        // Track some calls
        used.track_call("print");
        used.track_call("alloc");

        assert!(!used.is_empty());
        assert_eq!(used.len(), 2);
        assert!(used.is_used("print"));
        assert!(used.is_used("alloc"));
        assert!(!used.is_used("send"));
    }

    #[test]
    fn test_used_imports_duplicates() {
        let mut used = UsedImports::new();

        // Track same function multiple times
        used.track_call("print");
        used.track_call("print");
        used.track_call("print");

        // Should only count once
        assert_eq!(used.len(), 1);
    }

    #[test]
    fn test_used_imports_get_used() {
        let mut used = UsedImports::new();

        used.track_call("send");
        used.track_call("print");
        used.track_call("alloc");

        let imports = used.get_used_imports();
        assert_eq!(imports.len(), 3);

        // Should be sorted
        assert_eq!(imports[0], "alloc");
        assert_eq!(imports[1], "print");
        assert_eq!(imports[2], "send");
    }

    #[test]
    fn test_used_imports_filter() {
        let mut used = UsedImports::new();
        used.track_call("print");
        used.track_call("alloc");

        let all_funcs = standard_host_functions();
        let filtered = used.filter_used(&all_funcs);

        // Should only return used functions
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|f| f.name == "print"));
        assert!(filtered.iter().any(|f| f.name == "alloc"));
        assert!(!filtered.iter().any(|f| f.name == "send"));
    }

    #[test]
    fn test_used_imports_clear() {
        let mut used = UsedImports::new();

        used.track_call("print");
        used.track_call("alloc");
        assert_eq!(used.len(), 2);

        used.clear();
        assert!(used.is_empty());
        assert_eq!(used.len(), 0);
    }

    #[test]
    fn test_mark_used() {
        let mut used = UsedImports::new();

        used.mark_used("print");
        used.mark_used("alloc");

        assert!(used.is_used("print"));
        assert!(used.is_used("alloc"));
        assert_eq!(used.len(), 2);
    }
}
