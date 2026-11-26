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

// ---------------------------------------------------------------------------
// Gene layouts with `string` and `u64` fields (task B2).
//
// Representation: a `string` field is a 4-byte i32 pointer to a
// length-prefixed buffer `[len: u32 LE][utf-8 bytes]`; a `u64` field is an
// 8-byte, 8-aligned i64 slot holding the raw 64 bits (never wrapped to i32).
// ---------------------------------------------------------------------------

const MESSAGE_SOURCE: &str = r#"
gen Message {
    has body: string
    has sent_at: u64
    has id: u64
}

docs { A broadcast message with a timestamp and id. }

sex fun host_network_broadcast(msg: string) -> i64

sex fun send() -> i64 {
    let m = Message { body: "tick from a gene field", sent_at: 1, id: 2 }
    return host_network_broadcast(m.body)
}

fun stamp() -> u64 {
    let m = Message { body: "x", sent_at: 6000000000, id: 7 }
    return m.sent_at
}

fun ident() -> u64 {
    let m = Message { body: "x", sent_at: 1, id: 9223372036854775807 }
    return m.id
}

fun bump(t: u64) -> u64 {
    return t + 1
}
"#;

fn instantiate_with_broadcast_stub(
    wasm: &[u8],
) -> (Store<()>, wasmtime::Instance, Arc<Mutex<Vec<String>>>) {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).expect("emitted module must validate");
    let seen: Arc<Mutex<Vec<String>>> = Arc::default();
    let log = seen.clone();
    let mut linker: Linker<()> = Linker::new(&engine);
    linker
        .func_wrap(
            "vudo",
            "host_network_broadcast",
            move |mut c: Caller<'_, ()>, p: i32, l: i32| -> i64 {
                let m = String::from_utf8(read_guest(&mut c, p, l)).expect("utf-8");
                log.lock().unwrap().push(m);
                42
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("must link against the VM's host ABI");
    (store, instance, seen)
}

#[test]
fn gene_with_string_and_u64_fields_has_expected_layout() {
    use metadol::wasm::layout::WasmFieldType;
    let file = parse_dol_file(MESSAGE_SOURCE).expect("parse");
    let mut compiler = WasmCompiler::new();
    compiler.compile_file(&file).expect("compile");
    let layout = compiler
        .get_gene_layout("Message")
        .expect("Message layout registered");
    let body = layout.get_field("body").unwrap();
    assert_eq!((body.offset, body.wasm_type), (0, WasmFieldType::Ptr));
    let sent_at = layout.get_field("sent_at").unwrap();
    assert_eq!((sent_at.offset, sent_at.wasm_type), (8, WasmFieldType::I64));
    let id = layout.get_field("id").unwrap();
    assert_eq!((id.offset, id.wasm_type), (16, WasmFieldType::I64));
    assert_eq!((layout.total_size, layout.alignment), (24, 8));
}

/// Round trip: a string stored in a gene field reaches the host fn intact.
#[test]
fn string_gene_field_reaches_host_intact() {
    let wasm = compile(MESSAGE_SOURCE).expect("compile");
    let (mut store, instance, seen) = instantiate_with_broadcast_stub(&wasm);
    let ret = instance
        .get_typed_func::<(), i64>(&mut store, "send")
        .unwrap()
        .call(&mut store, ())
        .unwrap();
    assert_eq!(ret, 42);
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["tick from a gene field".to_string()]
    );
}

/// u64 values above u32::MAX survive a gene field store/load untruncated.
#[test]
fn u64_gene_field_is_not_truncated() {
    let wasm = compile(MESSAGE_SOURCE).expect("compile");
    let (mut store, instance, _) = instantiate_with_broadcast_stub(&wasm);
    let stamp = instance
        .get_typed_func::<(), i64>(&mut store, "stamp")
        .expect("u64 return lowers to i64");
    assert_eq!(stamp.call(&mut store, ()).unwrap(), 6_000_000_000);
    let ident = instance
        .get_typed_func::<(), i64>(&mut store, "ident")
        .unwrap();
    assert_eq!(ident.call(&mut store, ()).unwrap(), i64::MAX);
}

/// u64 parameters and results are i64 at the WASM boundary, full width.
#[test]
fn u64_param_and_result_lower_to_i64() {
    let wasm = compile(MESSAGE_SOURCE).expect("compile");
    let (mut store, instance, _) = instantiate_with_broadcast_stub(&wasm);
    let bump = instance
        .get_typed_func::<i64, i64>(&mut store, "bump")
        .expect("bump exported as (i64) -> i64");
    // u64 bit pattern 0xFFFF_FFFF_FFFF_FFFE + 1 == 0xFFFF_FFFF_FFFF_FFFF.
    let max_u64_minus_1 = u64::MAX - 1;
    let got = bump.call(&mut store, max_u64_minus_1 as i64).unwrap();
    assert_eq!(got as u64, u64::MAX);
    assert_eq!(bump.call(&mut store, 1 << 40).unwrap(), (1 << 40) + 1);
}

/// u64 host results/params keep all 64 bits through the ABI.
#[test]
fn u64_host_time_and_credit_round_trip() {
    let source = r#"
sex fun host_time_now() -> u64
sex fun host_credit_consume(reservation: u64) -> i64

sex fun tick() -> u64 {
    return host_time_now()
}

sex fun spend(r: u64) -> i64 {
    return host_credit_consume(r)
}
"#;
    let wasm = compile(source).expect("compile");
    let engine = Engine::default();
    let imports = func_imports(&engine, &wasm);
    assert!(imports.contains(&(
        "vudo".to_string(),
        "host_time_now".to_string(),
        vec![],
        names(&[ValType::I64])
    )));
    assert!(imports.contains(&(
        "vudo".to_string(),
        "host_credit_consume".to_string(),
        names(&[ValType::I64]),
        names(&[ValType::I32])
    )));

    let module = Module::new(&engine, &wasm).unwrap();
    let seen: Arc<Mutex<Vec<i64>>> = Arc::default();
    let log = seen.clone();
    let mut linker: Linker<()> = Linker::new(&engine);
    let now: u64 = 1_764_000_000_123_456_789; // ns timestamp > u32::MAX
    linker
        .func_wrap("vudo", "host_time_now", move || -> i64 { now as i64 })
        .unwrap();
    linker
        .func_wrap("vudo", "host_credit_consume", move |r: i64| -> i32 {
            log.lock().unwrap().push(r);
            0
        })
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let tick = instance
        .get_typed_func::<(), i64>(&mut store, "tick")
        .unwrap();
    assert_eq!(tick.call(&mut store, ()).unwrap() as u64, now);
    let spend = instance
        .get_typed_func::<i64, i64>(&mut store, "spend")
        .unwrap();
    let big = (u64::MAX - 5) as i64;
    assert_eq!(spend.call(&mut store, big).unwrap(), 0);
    assert_eq!(*seen.lock().unwrap(), vec![big]);
}

/// A gene method passing its own (implicit `self`) `string` field to the
/// host: the host-built `[len][bytes]` buffer arrives intact.
#[test]
fn gene_method_passes_own_string_field_to_host() {
    let source = r#"
gen Note {
    has text: string
    has seq: u64

    sex fun publish() -> i64 {
        return host_network_broadcast(text)
    }
}

docs { A note that broadcasts its own text. }

sex fun host_network_broadcast(msg: string) -> i64
"#;
    let wasm = compile(source).expect("compile");
    let (mut store, instance, seen) = instantiate_with_broadcast_stub(&wasm);

    // Host writes a length-prefixed string into guest memory.
    let text = "héllo, mycelium";
    let at: usize = 60_000;
    let mem = instance.get_memory(&mut store, "memory").unwrap();
    let mut buf = (text.len() as u32).to_le_bytes().to_vec();
    buf.extend_from_slice(text.as_bytes());
    mem.write(&mut store, at, &buf).unwrap();

    let new_note = instance
        .get_typed_func::<(i32, i64), i32>(&mut store, "new_Note")
        .expect("constructor takes (string ptr: i32, seq: i64)");
    let note = new_note.call(&mut store, (at as i32, 1 << 50)).unwrap();
    let publish = instance
        .get_typed_func::<i32, i64>(&mut store, "Note.publish")
        .unwrap();
    assert_eq!(publish.call(&mut store, note).unwrap(), 42);
    assert_eq!(*seen.lock().unwrap(), vec![text.to_string()]);
}
