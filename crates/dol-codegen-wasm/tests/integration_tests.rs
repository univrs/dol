//! Integration tests for dol-codegen-wasm

use dol_codegen_wasm::{ImportEmitter, ImportTracker, UsedImports};
use dol_abi::{standard_host_functions, HostFunctionCategory, IMPORT_MODULE};
use walrus::Module;

#[test]
fn test_full_import_pipeline() {
    // Create a WASM module
    let mut module = Module::default();
    let mut emitter = ImportEmitter::new(&mut module);

    // Track usage
    let mut used = UsedImports::new();
    used.track_call("print");
    used.track_call("println");
    used.track_call("alloc");

    // Filter to used functions
    let all_funcs = standard_host_functions();
    let used_funcs: Vec<_> = used.filter_used(&all_funcs)
        .into_iter()
        .cloned()
        .collect();
    assert_eq!(used_funcs.len(), 3);

    // Emit imports
    let section = emitter.emit_all(&used_funcs).unwrap();
    assert_eq!(section.len(), 3);

    // Verify we can get function IDs
    assert!(section.get_func_id("print").is_some());
    assert!(section.get_func_id("println").is_some());
    assert!(section.get_func_id("alloc").is_some());
    assert!(section.get_func_id("send").is_none()); // Not imported
}

#[test]
fn test_import_by_category() {
    let mut module = Module::default();
    let mut emitter = ImportEmitter::new(&mut module);

    // Import only I/O functions
    let all_funcs = standard_host_functions();
    let io_funcs: Vec<_> = all_funcs
        .into_iter()
        .filter(|f| f.category == HostFunctionCategory::IO)
        .collect();

    let section = emitter.emit_all(&io_funcs).unwrap();
    assert_eq!(section.len(), 4); // print, println, log, error

    assert!(section.contains("print"));
    assert!(section.contains("println"));
    assert!(section.contains("log"));
    assert!(section.contains("error"));
}

#[test]
fn test_import_tracker_lookup() {
    let tracker = ImportTracker::new();

    // Verify all 22 standard functions are available
    assert!(tracker.is_host_function("print"));
    assert!(tracker.is_host_function("println"));
    assert!(tracker.is_host_function("alloc"));
    assert!(tracker.is_host_function("free"));
    assert!(tracker.is_host_function("now"));
    assert!(tracker.is_host_function("send"));
    assert!(tracker.is_host_function("recv"));
    assert!(tracker.is_host_function("random"));
    assert!(tracker.is_host_function("emit_effect"));
    assert!(tracker.is_host_function("breakpoint"));

    // Test getting by short name
    let print_fn = tracker.get_host_function("print").unwrap();
    assert_eq!(print_fn.name, "print");
    assert_eq!(print_fn.import_name(), "vudo_print");
    assert_eq!(print_fn.category, HostFunctionCategory::IO);

    // Test getting by full import name
    let alloc_fn = tracker.get_host_function("vudo_alloc").unwrap();
    assert_eq!(alloc_fn.name, "alloc");
}

#[test]
fn test_used_imports_deterministic_ordering() {
    let mut used = UsedImports::new();

    // Add in random order
    used.track_call("send");
    used.track_call("alloc");
    used.track_call("print");
    used.track_call("free");
    used.track_call("println");

    // Get imports - should be sorted
    let imports = used.get_used_imports();
    assert_eq!(imports, vec!["alloc", "free", "print", "println", "send"]);
}

#[test]
fn test_minimal_module() {
    // Create a minimal WASM module with just one import
    let mut module = Module::default();
    let mut emitter = ImportEmitter::new(&mut module);

    let all_funcs = standard_host_functions();
    let print_fn = all_funcs.iter().find(|f| f.name == "print").unwrap();

    let func_id = emitter.add_import(print_fn.clone()).unwrap();

    // Verify the function was added to the module
    let section = emitter.section();
    assert_eq!(section.len(), 1);

    let info = section.get("print").unwrap();
    assert_eq!(info.func_id, func_id);
    assert_eq!(info.function.name, "print");
}

#[test]
fn test_import_all_standard_functions() {
    let mut module = Module::default();
    let mut emitter = ImportEmitter::new(&mut module);

    // Import all 22 standard functions
    let all_funcs = standard_host_functions();
    let section = emitter.emit_all(&all_funcs).unwrap();

    assert_eq!(section.len(), 22);

    // Verify categories are represented
    let io_count = all_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::IO)
        .count();
    assert_eq!(io_count, 4);

    let memory_count = all_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::Memory)
        .count();
    assert_eq!(memory_count, 3);
}

#[test]
fn test_import_module_name() {
    let all_funcs = standard_host_functions();
    let print_fn = all_funcs.iter().find(|f| f.name == "print").unwrap();

    // All imports should use the "vudo" module namespace
    assert_eq!(print_fn.import_name(), "vudo_print");
    assert_eq!(IMPORT_MODULE, "vudo");
}

#[test]
fn test_usage_tracking_with_duplicates() {
    let mut used = UsedImports::new();

    // Track the same function multiple times
    used.track_call("print");
    used.track_call("print");
    used.track_call("print");
    used.track_call("alloc");
    used.track_call("alloc");

    // Should only count each once
    assert_eq!(used.len(), 2);

    let imports = used.get_used_imports();
    assert_eq!(imports, vec!["alloc", "print"]);
}

#[test]
fn test_filter_preserves_function_metadata() {
    let mut used = UsedImports::new();
    used.track_call("print");

    let all_funcs = standard_host_functions();
    let filtered = used.filter_used(&all_funcs);

    assert_eq!(filtered.len(), 1);
    let print_fn = filtered[0];
    assert_eq!(print_fn.name, "print");
    assert_eq!(print_fn.category, HostFunctionCategory::IO);
    assert_eq!(print_fn.signature.params.len(), 2); // (ptr: i32, len: i32)
    assert!(print_fn.signature.returns.is_none()); // void
}
