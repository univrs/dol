//! Host ABI conformance tests (Spirit Hosting Plan H0, decision D2).
//!
//! The VUDO VM linker (`univrs/vudo` `vudo_vm/src/linker.rs`) is the canonical
//! host ABI: module `vudo`, functions named `host_*`. A bodiless `sex fun`
//! in DOL must compile to an import that links against that linker exactly.
//!
//! The expected signatures below are written out independently from the
//! compiler's table on purpose: this file is the oracle, copied from the
//! VM linker's `func_wrap` closures.

#![cfg(feature = "wasm")]

use std::sync::{Arc, Mutex};

use metadol::parse_dol_file;
use metadol::wasm::WasmCompiler;
use wasmtime::{Engine, ExternType, Linker, Module, Store, ValType};

/// (name, params, result) exactly as registered by `vudo_vm::linker`.
fn vm_linker_abi() -> Vec<(&'static str, Vec<ValType>, Vec<ValType>)> {
    use ValType::{I32, I64};
    vec![
        ("host_time_now", vec![], vec![I64]),
        ("host_random_bytes", vec![I32, I32], vec![I32]),
        ("host_log", vec![I32, I32, I32], vec![I32]),
        ("host_storage_read", vec![I32, I32, I32, I32], vec![I32]),
        ("host_storage_write", vec![I32, I32, I32, I32], vec![I32]),
        ("host_storage_delete", vec![I32, I32], vec![I32]),
        ("host_network_connect", vec![I32, I32], vec![I64]),
        ("host_network_listen", vec![I32], vec![I64]),
        ("host_network_broadcast", vec![I32, I32], vec![I64]),
        ("host_credit_balance", vec![I32], vec![I64]),
        ("host_credit_transfer", vec![I32, I32, I64], vec![I32]),
        ("host_credit_reserve", vec![I32, I64], vec![I64]),
        ("host_credit_release", vec![I64], vec![I32]),
        ("host_credit_consume", vec![I64], vec![I32]),
        ("host_credit_available", vec![I32], vec![I64]),
    ]
}

fn compile(source: &str) -> Result<Vec<u8>, String> {
    let file = parse_dol_file(source).map_err(|e| format!("parse: {e:?}"))?;
    WasmCompiler::new()
        .compile_file(&file)
        .map_err(|e| e.to_string())
}

/// (module, name, params, results) of every function import.
fn func_imports(engine: &Engine, wasm: &[u8]) -> Vec<(String, String, Vec<String>, Vec<String>)> {
    let module = Module::new(engine, wasm).expect("emitted module must validate");
    module
        .imports()
        .filter_map(|imp| match imp.ty() {
            ExternType::Func(f) => Some((
                imp.module().to_string(),
                imp.name().to_string(),
                f.params().map(|t| t.to_string()).collect(),
                f.results().map(|t| t.to_string()).collect(),
            )),
            _ => None,
        })
        .collect()
}

fn names(tys: &[ValType]) -> Vec<String> {
    tys.iter().map(|t| t.to_string()).collect()
}

const LOG_SOURCE: &str = r#"
sex fun host_log(level: i64, ptr: i64, len: i64) -> i64

sex fun emit(level: i64) -> i64 {
    return host_log(level, 0, 0)
}
"#;

#[test]
fn host_log_imports_from_vudo_with_linker_signature() {
    let wasm = compile(LOG_SOURCE).expect("compile");
    let imports = func_imports(&Engine::default(), &wasm);
    let expected = vm_linker_abi()
        .into_iter()
        .find(|(n, _, _)| *n == "host_log")
        .unwrap();
    assert_eq!(
        imports,
        vec![(
            "vudo".to_string(),
            "host_log".to_string(),
            names(&expected.1),
            names(&expected.2)
        )]
    );
}

#[test]
fn every_vm_host_function_imports_with_linker_signature() {
    let engine = Engine::default();
    for (name, params, results) in vm_linker_abi() {
        let dol_params: Vec<String> = (0..params.len()).map(|i| format!("a{i}: i64")).collect();
        let ret = if results.is_empty() { "" } else { " -> i64" };
        let source = format!("sex fun {name}({}){ret}\n", dol_params.join(", "));
        let wasm = compile(&source).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            func_imports(&engine, &wasm),
            vec![(
                "vudo".to_string(),
                name.to_string(),
                names(&params),
                names(&results)
            )],
            "{name} must match the VM linker"
        );
    }
}

/// Negative oracle: no emitted import ever targets the retired `loa` module.
#[test]
fn no_import_uses_loa_module() {
    let wasm = compile(LOG_SOURCE).expect("compile");
    for (module, name, _, _) in func_imports(&Engine::default(), &wasm) {
        assert_ne!(module, "loa", "import {name} still targets `loa`");
        assert_eq!(module, "vudo");
    }
}

/// Negative oracle: a host function the VM does not provide never compiles,
/// because the resulting module could never link in the VUDO VM.
#[test]
fn unknown_host_function_is_a_compile_error() {
    let err = compile("sex fun vudo_print(ptr: i64, len: i64)\n").unwrap_err();
    assert!(err.contains("vudo_print"), "{err}");
    assert!(
        err.contains("host_log"),
        "error should list the VM ABI: {err}"
    );
}

#[test]
fn host_function_arity_mismatch_is_a_compile_error() {
    let err = compile("sex fun host_log(ptr: i64, len: i64) -> i64\n").unwrap_err();
    assert!(err.contains("host_log"), "{err}");
}

#[test]
fn float_parameter_for_integer_host_param_is_a_compile_error() {
    let err = compile("sex fun host_network_listen(port: f64) -> i64\n").unwrap_err();
    assert!(err.contains("host_network_listen"), "{err}");
}

/// Links against a stub providing exactly the VM's `vudo.host_log`
/// signature, runs the DOL function, and checks the host saw the call.
#[test]
fn dol_spirit_calling_host_log_links_and_runs() {
    let engine = Engine::default();
    let wasm = compile(LOG_SOURCE).expect("compile");
    let module = Module::new(&engine, &wasm).unwrap();

    let calls: Arc<Mutex<Vec<(i32, i32, i32)>>> = Arc::default();
    let seen = calls.clone();
    let mut linker: Linker<()> = Linker::new(&engine);
    linker
        .func_wrap(
            "vudo",
            "host_log",
            move |level: i32, ptr: i32, len: i32| -> i32 {
                seen.lock().unwrap().push((level, ptr, len));
                if level == 4 {
                    -1 // HOST_ERROR, as the VM returns on failure
                } else {
                    0
                }
            },
        )
        .unwrap();

    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("DOL Spirit must link against the VM's host ABI");
    let emit = instance
        .get_typed_func::<i64, i64>(&mut store, "emit")
        .expect("emit exported as (i64) -> i64");

    assert_eq!(emit.call(&mut store, 2).unwrap(), 0);
    // The host's i32 result is sign-extended back to DOL's i64.
    assert_eq!(emit.call(&mut store, 4).unwrap(), -1);
    assert_eq!(*calls.lock().unwrap(), vec![(2, 0, 0), (4, 0, 0)]);
}
