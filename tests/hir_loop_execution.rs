//! Execution oracle for HIR loop lowering.
//!
//! Regression for the "loops run once" bug: AST→HIR lowering used to turn
//! `loop`, `while` and `for` into plain blocks, so the body executed at most
//! once (and the emitted `break` was outside any loop).
//!
//! Each test lowers a DOL function through the HIR, generates Rust with
//! `HirRustCodegen`, compiles it with `rustc` next to a tiny harness, runs the
//! binary and checks how many times the loop body actually executed. The body
//! calls a host function `tick()` that bumps a counter, so the DOL source needs
//! no mutable bindings.

use metadol::codegen::HirRustCodegen;
use metadol::lower::lower_file;
use std::process::Command;

/// Harness linked with the generated code. `keep_going()` returns true for
/// the first `LIMIT` calls; `seq()` yields `LIMIT` items; `tick()` counts.
const HARNESS: &str = r#"
use std::sync::atomic::{AtomicI64, Ordering};
static TICKS: AtomicI64 = AtomicI64::new(0);
static CHECKS: AtomicI64 = AtomicI64::new(0);
const LIMIT: i64 = 5;
fn tick() { TICKS.fetch_add(1, Ordering::SeqCst); }
fn ticks() -> i64 { TICKS.load(Ordering::SeqCst) }
fn keep_going() -> bool { CHECKS.fetch_add(1, Ordering::SeqCst) < LIMIT }
fn seq() -> std::ops::Range<i64> { 0..LIMIT }
fn main() {
    run();
    println!("ticks={}", ticks());
}
"#;

/// Lower `source` via HIR, generate Rust, compile it with rustc and run it.
/// Returns stdout of the program, or panics with the generated code on error.
fn run_via_hir(source: &str) -> String {
    let (hir, ctx) = lower_file(source).expect("DOL source should parse");
    assert!(
        !ctx.has_errors(),
        "lowering errors: {:?}",
        ctx.diagnostics()
    );
    let mut codegen = HirRustCodegen::with_symbols(ctx.symbols);
    let generated = codegen.generate(&hir);

    let program = format!("#![allow(warnings)]\n{}\n{}", generated, HARNESS);
    let dir = tempfile::tempdir().expect("tempdir");
    let src_path = dir.path().join("prog.rs");
    let bin_path = dir.path().join("prog");
    std::fs::write(&src_path, &program).expect("write program");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let out = Command::new(rustc)
        .arg("--edition=2021")
        .arg("-o")
        .arg(&bin_path)
        .arg(&src_path)
        .output()
        .expect("failed to spawn rustc");
    assert!(
        out.status.success(),
        "generated Rust failed to compile:\n{}\n--- program ---\n{}",
        String::from_utf8_lossy(&out.stderr),
        program
    );

    let run = Command::new(&bin_path).output().expect("run program");
    assert!(run.status.success(), "program failed: {:?}", run);
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

#[test]
fn while_loop_body_runs_n_times() {
    let src = r#"
fun run() {
    while keep_going() {
        tick()
    }
}
"#;
    assert_eq!(run_via_hir(src), "ticks=5");
}

#[test]
fn infinite_loop_with_break_runs_n_times() {
    let src = r#"
fun run() {
    loop {
        if keep_going() {
            tick()
        } else {
            break
        }
    }
}
"#;
    assert_eq!(run_via_hir(src), "ticks=5");
}

#[test]
fn for_loop_body_runs_once_per_item() {
    let src = r#"
fun run() {
    for i in seq() {
        tick()
    }
}
"#;
    assert_eq!(run_via_hir(src), "ticks=5");
}

#[test]
fn continue_skips_rest_of_body() {
    // Every iteration hits `continue` before the second tick, so only one
    // tick per iteration may be recorded.
    let src = r#"
fun run() {
    while keep_going() {
        tick()
        continue
        tick()
    }
}
"#;
    assert_eq!(run_via_hir(src), "ticks=5");
}
