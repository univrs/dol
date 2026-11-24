//! The `docs/examples/*.dol` programs must target the canonical VUDO host ABI.
//!
//! Every example is parsed, compiled to WASM, and its imports are checked
//! against the 15 `host_*` functions the VUDO VM linker provides in module
//! `vudo` (see `src/wasm/host_abi.rs`). The list is written out here on
//! purpose so this test is an independent oracle. Each example's `main` is
//! also run against VM-like stubs so its comments stay truthful.

#![cfg(feature = "wasm")]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use metadol::parse_dol_file;
use metadol::wasm::WasmCompiler;
use wasmtime::{Caller, Engine, Linker, Module, Store, Val};

const VM_HOST_FUNCTIONS: [&str; 15] = [
    "host_time_now",
    "host_random_bytes",
    "host_log",
    "host_storage_read",
    "host_storage_write",
    "host_storage_delete",
    "host_network_connect",
    "host_network_listen",
    "host_network_broadcast",
    "host_credit_balance",
    "host_credit_transfer",
    "host_credit_reserve",
    "host_credit_release",
    "host_credit_consume",
    "host_credit_available",
];

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/examples")
}

fn example_files() -> Vec<PathBuf> {
    let dir = examples_dir();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "dol"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no .dol files in {}", dir.display());
    files
}

fn compile(engine: &Engine, path: &Path) -> Module {
    let source = std::fs::read_to_string(path).unwrap();
    let file = parse_dol_file(&source)
        .unwrap_or_else(|e| panic!("{}: parse error: {e:?}", path.display()));
    let wasm = WasmCompiler::new()
        .compile_file(&file)
        .unwrap_or_else(|e| panic!("{}: compile error: {e}", path.display()));
    Module::new(engine, &wasm).unwrap_or_else(|e| panic!("{}: invalid wasm: {e:?}", path.display()))
}

/// (module, name) of every import of the compiled example.
fn compile_imports(path: &Path) -> Vec<(String, String)> {
    compile(&Engine::default(), path)
        .imports()
        .map(|i| (i.module().to_string(), i.name().to_string()))
        .collect()
}

#[test]
fn docs_examples_import_only_canonical_host_abi() {
    for path in example_files() {
        let imports = compile_imports(&path);
        assert!(
            !imports.is_empty(),
            "{}: example should use at least one host function",
            path.display()
        );
        for (module, name) in imports {
            assert_eq!(module, "vudo", "{}: import {name}", path.display());
            assert!(
                VM_HOST_FUNCTIONS.contains(&name.as_str()),
                "{}: `{name}` is not a VUDO VM host function",
                path.display()
            );
        }
    }
}

/// Negative oracle: no example uses the retired `vudo_*` names or `loa` module.
#[test]
fn docs_examples_never_import_retired_abi() {
    for path in example_files() {
        for (module, name) in compile_imports(&path) {
            assert_ne!(module, "loa", "{}: import {name}", path.display());
            assert!(
                !name.starts_with("vudo_"),
                "{}: retired import `{name}`",
                path.display()
            );
        }
    }
}

fn read_str(caller: &mut Caller<'_, ()>, ptr: i32, len: i32) -> String {
    let mem = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .expect("guest must export memory");
    let bytes = &mem.data(&caller)[ptr as usize..(ptr + len) as usize];
    String::from_utf8(bytes.to_vec()).expect("utf-8")
}

/// Runs `main` of an example against stubs that behave like the VM's
/// success paths (host_log rejects levels above 4 with -1, as the VM does;
/// broadcast reaches 3 peers). Returns (main's result, host call trace).
fn run_main(name: &str) -> (i64, Vec<String>) {
    let engine = Engine::default();
    let module = compile(&engine, &examples_dir().join(name));
    let trace: Arc<Mutex<Vec<String>>> = Arc::default();
    let mut linker: Linker<()> = Linker::new(&engine);
    let t = trace.clone();
    linker
        .func_wrap(
            "vudo",
            "host_log",
            move |mut c: Caller<'_, ()>, level: i32, p: i32, l: i32| -> i32 {
                let msg = read_str(&mut c, p, l);
                t.lock().unwrap().push(format!("log {level} {msg}"));
                if (0..=4).contains(&level) {
                    0
                } else {
                    -1
                }
            },
        )
        .unwrap();
    let t = trace.clone();
    linker
        .func_wrap(
            "vudo",
            "host_storage_write",
            move |mut c: Caller<'_, ()>, kp: i32, kl: i32, vp: i32, vl: i32| -> i32 {
                let (k, v) = (read_str(&mut c, kp, kl), read_str(&mut c, vp, vl));
                t.lock().unwrap().push(format!("write {k}={v}"));
                0
            },
        )
        .unwrap();
    let t = trace.clone();
    linker
        .func_wrap(
            "vudo",
            "host_network_broadcast",
            move |mut c: Caller<'_, ()>, p: i32, l: i32| -> i64 {
                let msg = read_str(&mut c, p, l);
                t.lock().unwrap().push(format!("broadcast {msg}"));
                3
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let main = instance.get_func(&mut store, "main").expect("main export");
    let mut out = [Val::I64(0)];
    main.call(&mut store, &[], &mut out).unwrap();
    let ret = match out[0] {
        Val::I32(v) => v as i64,
        Val::I64(v) => v,
        ref v => panic!("unexpected result {v:?}"),
    };
    let calls = trace.lock().unwrap().clone();
    (ret, calls)
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn hello_world_logs_at_info_and_debug() {
    assert_eq!(
        run_main("hello-world.dol"),
        (
            0,
            strings(&["log 2 Hello, World!", "log 1 hello-world example finished"])
        )
    );
}

#[test]
fn error_handling_reports_rejected_log_level() {
    assert_eq!(
        run_main("error-handling.dol"),
        (
            -1,
            strings(&[
                "write greeting=hello",
                "log 2 storage write ok",
                "log 7 this level does not exist",
                "log 4 host_log rejected level 7",
            ])
        )
    );
}

#[test]
fn messaging_broadcasts_and_returns_peer_count() {
    assert_eq!(
        run_main("messaging.dol"),
        (
            3,
            strings(&["broadcast hello from a DOL Spirit", "log 2 broadcast sent"])
        )
    );
}

#[test]
fn effects_persists_broadcasts_and_logs() {
    assert_eq!(
        run_main("effects.dol"),
        (
            0,
            strings(&[
                "write state=running",
                "broadcast running",
                "log 2 state changed",
            ])
        )
    );
}
