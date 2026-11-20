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

// ---------------------------------------------------------------------------
// String lowering: DOL `string` arguments become the ABI's (ptr, len) pairs.
// ---------------------------------------------------------------------------

use wasmtime::Caller;

/// Read `len` bytes at `ptr` from the guest's exported memory.
fn read_guest(caller: &mut Caller<'_, ()>, ptr: i32, len: i32) -> Vec<u8> {
    let mem = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .expect("guest must export memory");
    let data = mem.data(&caller);
    data[ptr as usize..(ptr + len) as usize].to_vec()
}

fn exports_memory(engine: &Engine, wasm: &[u8]) -> bool {
    Module::new(engine, wasm)
        .unwrap()
        .exports()
        .any(|e| e.name() == "memory" && matches!(e.ty(), ExternType::Memory(_)))
}

const HELLO_SOURCE: &str = r#"
sex fun host_log(level: i64, msg: string) -> i64

sex fun hello() -> i64 {
    return host_log(2, "hello from DOL")
}

sex fun say(msg: string) -> i64 {
    return host_log(1, msg)
}

sex fun via_param() -> i64 {
    return say("via a string parameter")
}
"#;

/// One observed `host_log` call: (level, text, ptr, len).
type LogCall = (i32, String, i32, i32);

/// Instantiate `wasm` with a `vudo.host_log` stub that records (level, text).
fn run_with_log_stub(wasm: &[u8], entry: &str) -> (i64, Vec<LogCall>) {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let seen: Arc<Mutex<Vec<LogCall>>> = Arc::default();
    let log = seen.clone();
    let mut linker: Linker<()> = Linker::new(&engine);
    linker
        .func_wrap(
            "vudo",
            "host_log",
            move |mut caller: Caller<'_, ()>, level: i32, ptr: i32, len: i32| -> i32 {
                let bytes = read_guest(&mut caller, ptr, len);
                let text = String::from_utf8(bytes).expect("utf-8");
                log.lock().unwrap().push((level, text, ptr, len));
                0
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let f = instance
        .get_typed_func::<(), i64>(&mut store, entry)
        .unwrap();
    let ret = f.call(&mut store, ()).unwrap();
    let calls = seen.lock().unwrap().clone();
    (ret, calls)
}

#[test]
fn string_arg_keeps_linker_signature_and_exports_memory() {
    let engine = Engine::default();
    let wasm = compile(HELLO_SOURCE).expect("compile");
    assert_eq!(
        func_imports(&engine, &wasm),
        vec![(
            "vudo".to_string(),
            "host_log".to_string(),
            names(&[ValType::I32, ValType::I32, ValType::I32]),
            names(&[ValType::I32])
        )]
    );
    assert!(exports_memory(&engine, &wasm), "module must export memory");
}

#[test]
fn string_literal_lowers_to_data_segment_ptr_and_len() {
    let wasm = compile(HELLO_SOURCE).expect("compile");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm).unwrap();
    type Seen = (i32, i32, Vec<u8>); // (ptr, len, length-prefix bytes)
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::default();
    let log = seen.clone();
    let mut linker: Linker<()> = Linker::new(&engine);
    linker
        .func_wrap(
            "vudo",
            "host_log",
            move |mut caller: Caller<'_, ()>, _level: i32, ptr: i32, len: i32| -> i32 {
                // The 4 bytes before ptr are the pool's little-endian length prefix.
                let prefix = read_guest(&mut caller, ptr - 4, 4);
                log.lock().unwrap().push((ptr, len, prefix));
                0
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    instance
        .get_typed_func::<(), i64>(&mut store, "hello")
        .unwrap()
        .call(&mut store, ())
        .unwrap();
    let (ptr, len, prefix) = seen.lock().unwrap()[0].clone();
    assert_eq!(len, "hello from DOL".len() as i32);
    assert!(ptr >= 4, "ptr points past the length prefix");
    assert_eq!(prefix, (len as u32).to_le_bytes().to_vec());
}

#[test]
fn host_log_stub_sees_the_dol_string() {
    let wasm = compile(HELLO_SOURCE).expect("compile");
    let (ret, calls) = run_with_log_stub(&wasm, "hello");
    assert_eq!(ret, 0);
    assert_eq!(calls.len(), 1);
    assert_eq!((calls[0].0, calls[0].1.as_str()), (2, "hello from DOL"));
}

#[test]
fn string_parameter_lowers_to_ptr_and_len() {
    let wasm = compile(HELLO_SOURCE).expect("compile");
    let (_, calls) = run_with_log_stub(&wasm, "via_param");
    assert_eq!(calls.len(), 1);
    assert_eq!(
        (calls[0].0, calls[0].1.as_str()),
        (1, "via a string parameter")
    );
}

#[test]
fn storage_key_value_and_broadcast_strings_lower() {
    let source = r#"
sex fun host_storage_write(key: string, value: string) -> i64
sex fun host_network_broadcast(msg: string) -> i64

sex fun run() -> i64 {
    host_storage_write("counter", "41")
    return host_network_broadcast("tick")
}
"#;
    let wasm = compile(source).expect("compile");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm).unwrap();
    let seen: Arc<Mutex<Vec<String>>> = Arc::default();
    let (w, b) = (seen.clone(), seen.clone());
    let mut linker: Linker<()> = Linker::new(&engine);
    linker
        .func_wrap(
            "vudo",
            "host_storage_write",
            move |mut c: Caller<'_, ()>, kp: i32, kl: i32, vp: i32, vl: i32| -> i32 {
                let k = String::from_utf8(read_guest(&mut c, kp, kl)).unwrap();
                let v = String::from_utf8(read_guest(&mut c, vp, vl)).unwrap();
                w.lock().unwrap().push(format!("write {k}={v}"));
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "vudo",
            "host_network_broadcast",
            move |mut c: Caller<'_, ()>, p: i32, l: i32| -> i64 {
                let m = String::from_utf8(read_guest(&mut c, p, l)).unwrap();
                b.lock().unwrap().push(format!("broadcast {m}"));
                7
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let ret = instance
        .get_typed_func::<(), i64>(&mut store, "run")
        .unwrap()
        .call(&mut store, ())
        .unwrap();
    assert_eq!(ret, 7);
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["write counter=41".to_string(), "broadcast tick".to_string()]
    );
}

#[test]
fn pointer_taking_import_exports_memory_without_strings() {
    let engine = Engine::default();
    let wasm = compile("sex fun host_credit_balance(account: i64) -> i64\n").expect("compile");
    assert!(exports_memory(&engine, &wasm));
}

#[test]
fn pointer_free_import_needs_no_memory() {
    let engine = Engine::default();
    let wasm = compile("sex fun host_time_now() -> i64\n").expect("compile");
    assert!(!exports_memory(&engine, &wasm));
}

/// Negative oracle: a string where the ABI expects a plain integer never compiles.
#[test]
fn string_declared_for_integer_host_param_is_a_compile_error() {
    let err = compile("sex fun host_network_listen(port: string) -> i64\n").unwrap_err();
    assert!(err.contains("host_network_listen"), "{err}");
}

/// Negative oracle: a string literal passed to an integer-declared host param.
#[test]
fn string_literal_for_integer_host_param_is_a_compile_error() {
    let source = r#"
sex fun host_log(level: i64, ptr: i64, len: i64) -> i64

sex fun bad() -> i64 {
    return host_log("oops", 0, 0)
}
"#;
    let err = compile(source).unwrap_err();
    assert!(err.contains("host_log"), "{err}");
}
