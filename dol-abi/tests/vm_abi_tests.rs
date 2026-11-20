//! The DOL ABI must match the VUDO VM linker (Spirit Hosting Plan, D2):
//! module `vudo`, `host_*` names, exact WASM signatures.

use dol_abi::{vm_host_functions, HostFunctionCategory, WasmType, IMPORT_MODULE};

#[test]
fn import_module_is_vudo() {
    assert_eq!(IMPORT_MODULE, "vudo");
}

#[test]
fn vm_host_functions_match_vm_linker() {
    use WasmType::{I32, I64};
    let expected: Vec<(&str, Vec<WasmType>, WasmType)> = vec![
        ("host_time_now", vec![], I64),
        ("host_random_bytes", vec![I32, I32], I32),
        ("host_log", vec![I32, I32, I32], I32),
        ("host_storage_read", vec![I32, I32, I32, I32], I32),
        ("host_storage_write", vec![I32, I32, I32, I32], I32),
        ("host_storage_delete", vec![I32, I32], I32),
        ("host_network_connect", vec![I32, I32], I64),
        ("host_network_listen", vec![I32], I64),
        ("host_network_broadcast", vec![I32, I32], I64),
        ("host_credit_balance", vec![I32], I64),
        ("host_credit_transfer", vec![I32, I32, I64], I32),
        ("host_credit_reserve", vec![I32, I64], I64),
        ("host_credit_release", vec![I64], I32),
        ("host_credit_consume", vec![I64], I32),
        ("host_credit_available", vec![I32], I64),
    ];
    let actual: Vec<(String, Vec<WasmType>, Option<WasmType>)> = vm_host_functions()
        .into_iter()
        .map(|f| (f.import_name(), f.signature.params, f.signature.returns))
        .collect();
    let expected: Vec<(String, Vec<WasmType>, Option<WasmType>)> = expected
        .into_iter()
        .map(|(n, p, r)| (n.to_string(), p, Some(r)))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn vm_host_function_categories() {
    let funcs = vm_host_functions();
    let cat = |n: &str| funcs.iter().find(|f| f.name == n).unwrap().category;
    assert_eq!(cat("host_log"), HostFunctionCategory::IO);
    assert_eq!(cat("host_time_now"), HostFunctionCategory::Time);
    assert_eq!(cat("host_random_bytes"), HostFunctionCategory::Random);
    assert_eq!(cat("host_storage_read"), HostFunctionCategory::Storage);
    assert_eq!(cat("host_network_broadcast"), HostFunctionCategory::Network);
    assert_eq!(cat("host_credit_transfer"), HostFunctionCategory::Credit);
}

/// Negative oracle: no VM host function import name uses the legacy `vudo_` prefix.
#[test]
fn vm_import_names_are_host_prefixed() {
    for f in vm_host_functions() {
        assert!(f.import_name().starts_with("host_"), "{}", f.import_name());
    }
}
