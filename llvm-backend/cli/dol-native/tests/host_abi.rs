//! End-to-end tests for the canonical VUDO host ABI (`host_*`) in native builds.
//!
//! Each test compiles a DOL program to an object file with the LLVM backend,
//! links it against `libvudo_runtime_native.a` with the system `cc`, runs the
//! binary and checks its output. A DOL Spirit that declares the VM's host
//! functions as bodiless `sex fun`s must build and run natively unchanged.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use dol_codegen_llvm::hir_lowering::HirLowering;
use dol_codegen_llvm::LlvmCodegen;
use inkwell::context::Context;

/// Workspace root of the LLVM backend (`llvm-backend/`).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Build the runtime static library once and return the directory holding it.
fn runtime_lib_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let root = workspace_root();
        let status = Command::new(env!("CARGO"))
            .args(["build", "-q", "-p", "vudo-runtime-native"])
            .current_dir(&root)
            .status()
            .expect("run cargo build for vudo-runtime-native");
        assert!(status.success(), "building vudo-runtime-native failed");
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target"));
        let dir = target.join("debug");
        assert!(
            dir.join("libvudo_runtime_native.a").exists(),
            "missing {}",
            dir.join("libvudo_runtime_native.a").display()
        );
        dir
    })
}

/// Compile DOL source to an object file. Returns the codegen error as a string.
fn compile(source: &str, obj: &Path) -> Result<(), String> {
    let (hir, ctx) = metadol::lower::lower_file(source).map_err(|e| e.to_string())?;
    let context = Context::create();
    let codegen = LlvmCodegen::new(&context, "test", "x86_64-unknown-linux-gnu")
        .map_err(|e| e.to_string())?;
    {
        let mut lowering = HirLowering::new(codegen.context(), codegen.module(), &ctx.symbols);
        lowering.lower_module(&hir).map_err(|e| e.to_string())?;
    }
    codegen
        .module()
        .verify()
        .map_err(|e| format!("invalid LLVM module: {e}"))?;
    codegen.emit_object(obj).map_err(|e| e.to_string())
}

/// Output of a native run.
struct Run {
    stdout: String,
    stderr: String,
    code: i32,
}

/// Compile, link and run a DOL program.
fn build_and_run(name: &str, source: &str) -> Run {
    let dir = std::env::temp_dir().join(format!("dol-native-host-abi-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let obj = dir.join(format!("{name}.o"));
    let exe = dir.join(name);
    compile(source, &obj).unwrap_or_else(|e| panic!("compile {name}: {e}"));

    let out = Command::new("cc")
        .arg(&obj)
        .arg(format!("-L{}", runtime_lib_dir().display()))
        .args([
            "-Wl,-Bstatic",
            "-lvudo_runtime_native",
            "-Wl,-Bdynamic",
            "-lpthread",
            "-ldl",
            "-lm",
            "-o",
        ])
        .arg(&exe)
        .output()
        .expect("run cc");
    assert!(
        out.status.success(),
        "link {name} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = Command::new(&exe).output().expect("run native binary");
    Run {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

#[test]
fn host_log_compiles_links_and_prints() {
    let run = build_and_run(
        "host_log",
        r#"
sex fun host_log(level: i32, msg: string) -> i32

fun main() {
    host_log(2, "hello from native host_log")
    host_log(4, "an error line")
    val rc = host_log(3, "warned")
    println("host_log returned " + rc)
}
"#,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(
        run.stdout
            .contains("[VUDO:INFO] hello from native host_log"),
        "stdout: {}",
        run.stdout
    );
    assert!(run.stdout.contains("[VUDO:WARN] warned"), "{}", run.stdout);
    assert!(
        run.stderr.contains("[VUDO:ERROR] an error line"),
        "{}",
        run.stderr
    );
    assert!(run.stdout.contains("host_log returned 0"), "{}", run.stdout);
}

#[test]
fn host_log_rejects_unknown_level_like_the_vm() {
    let run = build_and_run(
        "host_log_level",
        r#"
sex fun host_log(level: i32, msg: string) -> i32

fun main() {
    println("rc=" + host_log(9, "nope"))
}
"#,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("rc=-1"), "{}", run.stdout);
    assert!(!run.stdout.contains("nope") && !run.stderr.contains("nope"));
}

#[test]
fn time_and_random_are_real() {
    let run = build_and_run(
        "time_random",
        r#"
sex fun host_time_now() -> i64
sex fun host_random_bytes(buf: i64, len: i32) -> i32

fun main() {
    if host_time_now() > 1700000000000000000 {
        println("time ok")
    }
    val buf = alloc(32)
    println("random=" + host_random_bytes(buf, 32))
    println("random0=" + host_random_bytes(buf, 0))
}
"#,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("time ok"), "{}", run.stdout);
    assert!(run.stdout.contains("random=0"), "{}", run.stdout);
    assert!(run.stdout.contains("random0=-1"), "{}", run.stdout);
}

#[test]
fn storage_round_trip_has_vm_semantics() {
    let run = build_and_run(
        "storage",
        r#"
sex fun host_storage_write(key: string, value: string) -> i32
sex fun host_storage_read(key: string, buf: i64, cap: i32) -> i32
sex fun host_storage_delete(key: string) -> i32

fun main() {
    val buf = alloc(64)
    println("missing=" + host_storage_read("counter", buf, 64))
    println("write=" + host_storage_write("counter", "41"))
    println("read=" + host_storage_read("counter", buf, 64))
    println("small=" + host_storage_read("counter", buf, 1))
    println("delete=" + host_storage_delete("counter"))
    println("again=" + host_storage_delete("counter"))
    println("gone=" + host_storage_read("counter", buf, 64))
}
"#,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let expected = "missing=0\nwrite=0\nread=2\nsmall=-1\ndelete=1\nagain=0\ngone=0\n";
    assert_eq!(run.stdout, expected);
}

/// Negative oracle: there is no network or ledger natively, so these host
/// functions must never report success.
#[test]
fn network_and_credit_never_report_success() {
    let run = build_and_run(
        "net_credit",
        r#"
sex fun host_network_connect(addr: string) -> i64
sex fun host_network_listen(port: i32) -> i64
sex fun host_network_broadcast(msg: string) -> i64
sex fun host_credit_balance(account: i64) -> i64
sex fun host_credit_transfer(src: i64, dst: i64, amount: i64) -> i32
sex fun host_credit_reserve(account: i64, amount: i64) -> i64
sex fun host_credit_release(reservation: i64) -> i32
sex fun host_credit_consume(reservation: i64) -> i32
sex fun host_credit_available(account: i64) -> i64

fun main() {
    val acct = alloc(32)
    val other = alloc(32)
    println("" + host_network_connect("127.0.0.1:9000"))
    println("" + host_network_listen(9000))
    println("" + host_network_broadcast("tick"))
    println("" + host_credit_balance(acct))
    println("" + host_credit_transfer(acct, other, 5))
    println("" + host_credit_reserve(acct, 5))
    println("" + host_credit_release(1))
    println("" + host_credit_consume(1))
    println("" + host_credit_available(acct))
}
"#,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(lines.len(), 9, "{}", run.stdout);
    for line in lines {
        assert_eq!(line, "-1", "a native network/credit call reported success");
    }
}

/// Negative oracle: a bodiless function that is not a VM host function can
/// never link in the VM, so the native backend rejects it too.
#[test]
fn unknown_host_function_is_a_compile_error() {
    let obj = std::env::temp_dir().join("dol-native-unknown-host.o");
    let err = compile("sex fun vudo_send(to: i32, msg: string) -> i32\n", &obj).unwrap_err();
    assert!(err.contains("vudo_send"), "{err}");
}

/// Negative oracle: a string where the ABI expects a plain integer.
#[test]
fn string_for_integer_host_param_is_a_compile_error() {
    let obj = std::env::temp_dir().join("dol-native-bad-host.o");
    let err = compile("sex fun host_network_listen(port: string) -> i64\n", &obj).unwrap_err();
    assert!(err.contains("host_network_listen"), "{err}");
}

/// Negative oracle: declaration arity must cover exactly the ABI parameters.
#[test]
fn host_arity_mismatch_is_a_compile_error() {
    let obj = std::env::temp_dir().join("dol-native-arity.o");
    let err = compile("sex fun host_log(msg: string) -> i32\n", &obj).unwrap_err();
    assert!(err.contains("host_log"), "{err}");
}

/// `examples/native/vudo_host.dol` (which also compiles for the VM) builds,
/// links and runs natively.
#[test]
fn vudo_host_example_runs_natively() {
    let path = workspace_root().join("../examples/native/vudo_host.dol");
    let source = std::fs::read_to_string(&path).expect("read vudo_host.dol");
    let run = build_and_run("vudo_host", &source);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(
        run.stdout,
        "[VUDO:INFO] Spirit started\n\
         [VUDO:INFO] clock: time is after the epoch\n\
         [VUDO:INFO] storage: wrote greeting\n\
         [VUDO:INFO] storage: deleted greeting\n\
         [VUDO:INFO] network: broadcast unavailable (-1, as expected natively)\n"
    );
}
