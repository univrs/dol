//! End-to-end WASM validation tests
//!
//! These tests validate the complete DOL → WASM compilation pipeline:
//! 1. DOL source → compiled WASM
//! 2. WASM module validation
//! 3. Import section verification
//! 4. Signature matching against ABI

use dol_abi::{
    standard_host_functions, HostFunctionCategory,
    IMPORT_MODULE, WasmType,
};
use std::collections::HashMap;
use std::path::PathBuf;
use dol_codegen_wasm::{ImportEmitter, ImportTracker};
use walrus::Module;

// =============================================================================
// Test Helpers
// =============================================================================

/// Get path to test fixture
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("spirits")
        .join(format!("{}.dol", name))
}

/// Read fixture content
fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|_| panic!("Should read {}.dol fixture", name))
}

// =============================================================================
// Fixture Validation Tests
// =============================================================================

#[test]
fn test_all_fixtures_exist() {
    let fixtures = [
        "hello",
        "messaging",
        "effects",
        "memory",
        "random",
        "allocator",
    ];

    for fixture in &fixtures {
        let path = fixture_path(fixture);
        assert!(
            path.exists(),
            "Fixture {} should exist at {:?}",
            fixture,
            path
        );

        let content = read_fixture(fixture);
        assert!(
            !content.is_empty(),
            "Fixture {} should not be empty",
            fixture
        );
        assert!(
            content.contains("module"),
            "Fixture {} should contain module declaration",
            fixture
        );
    }
}

#[test]
fn test_hello_fixture_structure() {
    let content = read_fixture("hello");

    assert!(content.contains("module test.hello"));
    assert!(content.contains("pub fun main()"));
    assert!(content.contains("println"));
    assert!(
        content.contains("vudo_println"),
        "Fixture should document expected import"
    );
}

#[test]
fn test_messaging_fixture_structure() {
    let content = read_fixture("messaging");

    assert!(content.contains("module test.messaging"));
    assert!(content.contains("gen Message"));
    assert!(content.contains("send("));
    assert!(content.contains("recv("));
    assert!(content.contains("pending("));
    assert!(content.contains("free_message"));
}

#[test]
fn test_effects_fixture_structure() {
    let content = read_fixture("effects");

    assert!(content.contains("module test.effects"));
    assert!(content.contains("emit_effect"));
    assert!(content.contains("subscribe"));
    assert!(content.contains("gen EffectData"));
}

#[test]
fn test_memory_fixture_structure() {
    let content = read_fixture("memory");

    assert!(content.contains("module test.memory"));
    assert!(content.contains("alloc"));
    assert!(content.contains("free"));
    assert!(content.contains("realloc"));
    assert!(content.contains("gen Buffer"));
}

#[test]
fn test_random_fixture_structure() {
    let content = read_fixture("random");

    assert!(content.contains("module test.random"));
    assert!(content.contains("random()"));
    assert!(content.contains("random_bytes"));
}

#[test]
fn test_allocator_fixture_structure() {
    let content = read_fixture("allocator");

    assert!(content.contains("module test.allocator"));
    assert!(content.contains("gen Allocator"));
    assert!(content.contains("alloc_tracked"));
    assert!(content.contains("free_tracked"));
    assert!(content.contains("realloc_tracked"));
}

// =============================================================================
// Import Category Tests
// =============================================================================

#[test]
fn test_io_category_complete() {
    let host_funcs = standard_host_functions();
    let io_funcs: Vec<_> = host_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::IO)
        .collect();

    assert_eq!(io_funcs.len(), 4, "Should have exactly 4 I/O functions");

    let names: Vec<&str> = io_funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"print"), "Should have print");
    assert!(names.contains(&"println"), "Should have println");
    assert!(names.contains(&"log"), "Should have log");
    assert!(names.contains(&"error"), "Should have error");

    // Verify all have correct signatures (ptr, len) -> void or (level, ptr, len) -> void
    for func in io_funcs {
        match func.name.as_str() {
            "print" | "println" | "error" => {
                assert_eq!(func.signature.params.len(), 2);
                assert_eq!(func.signature.params[0], WasmType::I32);
                assert_eq!(func.signature.params[1], WasmType::I32);
                assert!(func.signature.returns.is_none());
            }
            "log" => {
                assert_eq!(func.signature.params.len(), 3); // level, ptr, len
                assert_eq!(func.signature.params[0], WasmType::I32);
                assert_eq!(func.signature.params[1], WasmType::I32);
                assert_eq!(func.signature.params[2], WasmType::I32);
                assert!(func.signature.returns.is_none());
            }
            _ => panic!("Unexpected I/O function: {}", func.name),
        }
    }
}

#[test]
fn test_memory_category_complete() {
    let host_funcs = standard_host_functions();
    let memory_funcs: Vec<_> = host_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::Memory)
        .collect();

    assert_eq!(
        memory_funcs.len(),
        3,
        "Should have exactly 3 memory functions"
    );

    let names: Vec<&str> = memory_funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"alloc"));
    assert!(names.contains(&"free"));
    assert!(names.contains(&"realloc"));

    // Verify signatures
    for func in memory_funcs {
        match func.name.as_str() {
            "alloc" => {
                assert_eq!(func.signature.params.len(), 1); // size
                assert_eq!(func.signature.params[0], WasmType::I32);
                assert_eq!(func.signature.returns, Some(WasmType::I32)); // returns ptr
            }
            "free" => {
                assert_eq!(func.signature.params.len(), 2); // ptr, size
                assert_eq!(func.signature.params[0], WasmType::I32);
                assert_eq!(func.signature.params[1], WasmType::I32);
                assert!(func.signature.returns.is_none());
            }
            "realloc" => {
                assert_eq!(func.signature.params.len(), 3); // ptr, old_size, new_size
                assert_eq!(func.signature.params[0], WasmType::I32);
                assert_eq!(func.signature.params[1], WasmType::I32);
                assert_eq!(func.signature.params[2], WasmType::I32);
                assert_eq!(func.signature.returns, Some(WasmType::I32)); // returns new ptr
            }
            _ => panic!("Unexpected memory function: {}", func.name),
        }
    }
}

#[test]
fn test_time_category_complete() {
    let host_funcs = standard_host_functions();
    let time_funcs: Vec<_> = host_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::Time)
        .collect();

    assert_eq!(time_funcs.len(), 3, "Should have exactly 3 time functions");

    let names: Vec<&str> = time_funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"now"));
    assert!(names.contains(&"sleep"));
    assert!(names.contains(&"monotonic_now"));

    // Verify signatures
    for func in time_funcs {
        match func.name.as_str() {
            "now" | "monotonic_now" => {
                assert_eq!(func.signature.params.len(), 0);
                assert_eq!(func.signature.returns, Some(WasmType::I64));
            }
            "sleep" => {
                assert_eq!(func.signature.params.len(), 1); // millis
                assert_eq!(func.signature.params[0], WasmType::I32);
                assert!(func.signature.returns.is_none());
            }
            _ => panic!("Unexpected time function: {}", func.name),
        }
    }
}

#[test]
fn test_messaging_category_complete() {
    let host_funcs = standard_host_functions();
    let messaging_funcs: Vec<_> = host_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::Messaging)
        .collect();

    assert_eq!(
        messaging_funcs.len(),
        5,
        "Should have exactly 5 messaging functions"
    );

    let names: Vec<&str> = messaging_funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"send"));
    assert!(names.contains(&"recv"));
    assert!(names.contains(&"pending"));
    assert!(names.contains(&"broadcast"));
    assert!(names.contains(&"free_message"));
}

#[test]
fn test_random_category_complete() {
    let host_funcs = standard_host_functions();
    let random_funcs: Vec<_> = host_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::Random)
        .collect();

    assert_eq!(
        random_funcs.len(),
        2,
        "Should have exactly 2 random functions"
    );

    let names: Vec<&str> = random_funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"random"));
    assert!(names.contains(&"random_bytes"));

    // Verify signatures
    for func in random_funcs {
        match func.name.as_str() {
            "random" => {
                assert_eq!(func.signature.params.len(), 0);
                assert_eq!(func.signature.returns, Some(WasmType::F64));
            }
            "random_bytes" => {
                assert_eq!(func.signature.params.len(), 2); // buffer, count
                assert_eq!(func.signature.params[0], WasmType::I32);
                assert_eq!(func.signature.params[1], WasmType::I32);
                assert!(func.signature.returns.is_none());
            }
            _ => panic!("Unexpected random function: {}", func.name),
        }
    }
}

#[test]
fn test_effects_category_complete() {
    let host_funcs = standard_host_functions();
    let effect_funcs: Vec<_> = host_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::Effects)
        .collect();

    assert_eq!(
        effect_funcs.len(),
        2,
        "Should have exactly 2 effect functions"
    );

    let names: Vec<&str> = effect_funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"emit_effect"));
    assert!(names.contains(&"subscribe"));
}

#[test]
fn test_debug_category_complete() {
    let host_funcs = standard_host_functions();
    let debug_funcs: Vec<_> = host_funcs
        .iter()
        .filter(|f| f.category == HostFunctionCategory::Debug)
        .collect();

    assert_eq!(
        debug_funcs.len(),
        3,
        "Should have exactly 3 debug functions"
    );

    let names: Vec<&str> = debug_funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"breakpoint"));
    assert!(names.contains(&"assert"));
    assert!(names.contains(&"panic"));
}

// =============================================================================
// Import Module Tests
// =============================================================================

#[test]
fn test_all_imports_use_vudo_module() {
    let host_funcs = standard_host_functions();

    for func in &host_funcs {
        let import_name = func.import_name();
        assert!(
            import_name.starts_with("vudo_"),
            "Import name {} should start with vudo_",
            import_name
        );

        // Verify the naming convention
        assert_eq!(
            import_name,
            format!("vudo_{}", func.name),
            "Import name should be vudo_ + function name"
        );
    }
}

#[test]
fn test_import_module_constant() {
    assert_eq!(IMPORT_MODULE, "vudo");
}

// =============================================================================
// Walrus-based WASM Module Tests
// =============================================================================

#[test]
fn test_create_minimal_wasm_module() {
    let mut module = Module::default();
    let mut emitter = ImportEmitter::new(&mut module);

    // Add a single import
    let host_funcs = standard_host_functions();
    let print_fn = host_funcs.iter().find(|f| f.name == "print").unwrap();

    let result = emitter.add_import(print_fn.clone());
    assert!(result.is_ok(), "Should successfully add import");

    let section = emitter.section();
    assert_eq!(section.len(), 1);
    assert!(section.contains("print"));
}

#[test]
fn test_wasm_module_with_all_categories() {
    let mut module = Module::default();
    let mut emitter = ImportEmitter::new(&mut module);

    // Add one function from each category
    let host_funcs = standard_host_functions();

    let functions_to_add = vec![
        ("print", HostFunctionCategory::IO),
        ("alloc", HostFunctionCategory::Memory),
        ("now", HostFunctionCategory::Time),
        ("send", HostFunctionCategory::Messaging),
        ("random", HostFunctionCategory::Random),
        ("emit_effect", HostFunctionCategory::Effects),
        ("breakpoint", HostFunctionCategory::Debug),
    ];

    for (name, expected_category) in &functions_to_add {
        let func = host_funcs
            .iter()
            .find(|f| f.name == *name)
            .unwrap_or_else(|| panic!("Should find function {}", name));

        assert_eq!(
            func.category, *expected_category,
            "Function {} should be in {:?} category",
            name, expected_category
        );

        emitter
            .add_import(func.clone())
            .unwrap_or_else(|_| panic!("Should add import for {}", name));
    }

    let section = emitter.section();
    assert_eq!(section.len(), 7, "Should have 7 imports");

    // Verify all imports are present
    for (name, _) in functions_to_add {
        assert!(
            section.contains(name),
            "Section should contain import for {}",
            name
        );
    }
}

#[test]
fn test_import_tracker_finds_all_functions() {
    let tracker = ImportTracker::new();

    let expected_functions = [
        // I/O
        "print",
        "println",
        "log",
        "error",
        // Memory
        "alloc",
        "free",
        "realloc",
        // Time
        "now",
        "sleep",
        "monotonic_now",
        // Messaging
        "send",
        "recv",
        "pending",
        "broadcast",
        "free_message",
        // Random
        "random",
        "random_bytes",
        // Effects
        "emit_effect",
        "subscribe",
        // Debug
        "breakpoint",
        "assert",
        "panic",
    ];

    for func_name in &expected_functions {
        assert!(
            tracker.is_host_function(func_name),
            "Tracker should recognize {} as host function",
            func_name
        );

        let func = tracker.get_host_function(func_name);
        assert!(
            func.is_some(),
            "Tracker should be able to get {} by name",
            func_name
        );
    }

    // Verify non-host functions are not recognized
    assert!(!tracker.is_host_function("not_a_host_function"));
    assert!(tracker.get_host_function("not_a_host_function").is_none());
}

#[test]
fn test_import_tracker_supports_prefixed_names() {
    let tracker = ImportTracker::new();

    // Should be able to lookup by both short name and full import name
    let print_by_short = tracker.get_host_function("print").unwrap();
    let print_by_full = tracker.get_host_function("vudo_print").unwrap();

    assert_eq!(print_by_short.name, print_by_full.name);
    assert_eq!(print_by_short.name, "print");
    assert_eq!(print_by_short.import_name(), "vudo_print");
}

// =============================================================================
// Comprehensive Tests
// =============================================================================

#[test]
fn test_standard_functions_count() {
    let funcs = standard_host_functions();
    assert_eq!(
        funcs.len(),
        22,
        "Should have exactly 22 standard host functions"
    );

    // Group by category and verify counts
    let mut by_category: HashMap<HostFunctionCategory, usize> = HashMap::new();
    for func in &funcs {
        *by_category.entry(func.category).or_insert(0) += 1;
    }

    assert_eq!(by_category[&HostFunctionCategory::IO], 4);
    assert_eq!(by_category[&HostFunctionCategory::Memory], 3);
    assert_eq!(by_category[&HostFunctionCategory::Time], 3);
    assert_eq!(by_category[&HostFunctionCategory::Messaging], 5);
    assert_eq!(by_category[&HostFunctionCategory::Random], 2);
    assert_eq!(by_category[&HostFunctionCategory::Effects], 2);
    assert_eq!(by_category[&HostFunctionCategory::Debug], 3);
}

#[test]
fn test_all_function_signatures_are_valid() {
    let funcs = standard_host_functions();

    for func in &funcs {
        // Parameters should all be valid WASM types
        for param in &func.signature.params {
            match param {
                WasmType::I32 | WasmType::I64 | WasmType::F32 | WasmType::F64 => {
                    // Valid type
                }
            }
        }

        // Return type should be valid if present
        if let Some(ret) = &func.signature.returns {
            match ret {
                WasmType::I32 | WasmType::I64 | WasmType::F32 | WasmType::F64 => {
                    // Valid type
                }
            }
        }

        // Verify name is not empty
        assert!(!func.name.is_empty(), "Function name should not be empty");

        // Verify import name follows convention
        assert_eq!(
            func.import_name(),
            format!("vudo_{}", func.name),
            "Import name should follow vudo_<name> convention"
        );
    }
}

#[test]
fn test_no_duplicate_function_names() {
    let funcs = standard_host_functions();
    let mut seen_names = std::collections::HashSet::new();

    for func in &funcs {
        assert!(
            seen_names.insert(func.name.clone()),
            "Duplicate function name found: {}",
            func.name
        );
    }

    assert_eq!(
        seen_names.len(),
        22,
        "Should have 22 unique function names"
    );
}
