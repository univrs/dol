//! WASM validation tests

use gen_registry::wasm::{WasmModule, WasmValidator};

#[test]
fn test_wasm_module_from_bytes() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, // Magic: \0asm
        0x01, 0x00, 0x00, 0x00, // Version: 1
    ];

    let module = WasmModule::from_bytes(bytes.clone());
    assert_eq!(module.size(), 8);
    assert_eq!(module.bytes(), &bytes);
}

#[test]
fn test_wasm_hash() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ];

    let module = WasmModule::from_bytes(bytes);
    let hash = module.hash();

    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64); // SHA-256 hex string
}

#[test]
fn test_wasm_hash_deterministic() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ];

    let module1 = WasmModule::from_bytes(bytes.clone());
    let module2 = WasmModule::from_bytes(bytes);

    assert_eq!(module1.hash(), module2.hash());
}

#[test]
fn test_wasm_validator_new() {
    let validator = WasmValidator::new();
    // Should create successfully
    drop(validator);
}

#[test]
fn test_validate_valid_wasm() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ];

    let module = WasmModule::from_bytes(bytes);
    let validator = WasmValidator::new();

    assert!(validator.validate(&module).is_ok());
}

#[test]
fn test_validate_invalid_magic() {
    let bytes = vec![
        0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00,
    ];

    let module = WasmModule::from_bytes(bytes);
    let validator = WasmValidator::new();

    assert!(validator.validate(&module).is_err());
}

#[test]
fn test_validate_invalid_version() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, 0xFF, 0x00, 0x00, 0x00,
    ];

    let module = WasmModule::from_bytes(bytes);
    let validator = WasmValidator::new();

    assert!(validator.validate(&module).is_err());
}

#[test]
fn test_validate_too_short() {
    let bytes = vec![0x00, 0x61, 0x73];

    let module = WasmModule::from_bytes(bytes);
    let validator = WasmValidator::new();

    assert!(validator.validate(&module).is_err());
}

#[test]
fn test_verify_hash_success() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ];

    let module = WasmModule::from_bytes(bytes);
    let validator = WasmValidator::new();
    let hash = module.hash();

    assert!(validator.verify_hash(&module, &hash).is_ok());
}

#[test]
fn test_verify_hash_mismatch() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ];

    let module = WasmModule::from_bytes(bytes);
    let validator = WasmValidator::new();

    assert!(validator.verify_hash(&module, "invalid_hash").is_err());
}

#[test]
fn test_extract_capabilities() {
    let bytes = vec![
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ];

    let module = WasmModule::from_bytes(bytes);
    let capabilities = module.extract_capabilities().unwrap();

    // Empty module should have no capabilities
    assert_eq!(capabilities.len(), 0);
}

#[test]
fn test_different_bytes_different_hash() {
    let bytes1 = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    let bytes2 = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x01];

    let module1 = WasmModule::from_bytes(bytes1);
    let module2 = WasmModule::from_bytes(bytes2);

    assert_ne!(module1.hash(), module2.hash());
}

#[test]
fn test_wasm_size() {
    let bytes = vec![1, 2, 3, 4, 5];
    let module = WasmModule::from_bytes(bytes);
    assert_eq!(module.size(), 5);
}
