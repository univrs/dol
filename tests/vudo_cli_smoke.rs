//! Smoke tests for the `vudo` binary (requires the `vudo` feature).
//!
//! Guards against the binary silently failing to compile (e.g. a new
//! `EvalResult` variant not handled in the REPL loop).
#![cfg(feature = "vudo")]

use std::process::{Command, Stdio};

fn vudo() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vudo"))
}

#[test]
fn vudo_help_lists_subcommands() {
    let out = vudo().arg("--help").output().expect("run vudo --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for sub in ["run", "compile", "check", "repl"] {
        assert!(stdout.contains(sub), "missing subcommand {sub}: {stdout}");
    }
}

#[test]
fn vudo_repl_exits_cleanly_on_eof() {
    let out = vudo()
        .arg("repl")
        .stdin(Stdio::null())
        .output()
        .expect("run vudo repl");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
