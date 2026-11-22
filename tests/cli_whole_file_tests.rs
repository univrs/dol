//! CLI regression tests: `dol-parse` and `dol-check` must see every
//! declaration in a file, not just the first one (audit bug B6 / L0.5).
#![cfg(feature = "cli")]

use std::io::Write;
use std::process::{Command, Output};

const THREE_DECLS: &str = r#"
use shared.types

gen a.first {
  has id: i64
  docs {
    The first declaration in the file.
  }
}

trait a.second {
  uses a.first
  docs {
    The second declaration in the file.
  }
}

rule a.third {
  id never negative
  docs {
    The third declaration in the file.
  }
}
"#;

const TRAILING_GARBAGE: &str = r#"
gen a.first {
  has id: i64
  docs {
    A valid first declaration.
  }
}

this is not dol }}} at all
"#;

fn write_dol(contents: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".dol").tempfile().unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    f
}

fn run(bin: &str, args: &[&str]) -> Output {
    Command::new(bin).args(args).output().expect("run CLI")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn dol_parse_reports_every_declaration() {
    let f = write_dol(THREE_DECLS);
    let path = f.path().to_str().unwrap();
    let out = run(
        env!("CARGO_BIN_EXE_dol-parse"),
        &["--quiet", "--format", "json", path],
    );
    assert!(out.status.success(), "{}", stdout(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let decls = json["files"][0]["declarations"]
        .as_array()
        .unwrap_or_else(|| panic!("no declarations array: {}", json));
    let names: Vec<_> = decls.iter().map(|d| d["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["a.first", "a.second", "a.third"]);
    let types: Vec<_> = decls.iter().map(|d| d["type"].as_str().unwrap()).collect();
    assert_eq!(types, vec!["gene", "trait", "constraint"]);
}

#[test]
fn dol_parse_compact_lists_every_declaration() {
    let f = write_dol(THREE_DECLS);
    let path = f.path().to_str().unwrap();
    let out = run(
        env!("CARGO_BIN_EXE_dol-parse"),
        &["--format", "compact", path],
    );
    assert!(out.status.success());
    let text = stdout(&out);
    for name in ["a.first", "a.second", "a.third"] {
        assert!(text.contains(name), "missing {} in:\n{}", name, text);
    }
}

#[test]
fn dol_parse_fails_on_trailing_garbage() {
    // Negative oracle: garbage after the first declaration is never ignored.
    let f = write_dol(TRAILING_GARBAGE);
    let path = f.path().to_str().unwrap();
    let out = run(env!("CARGO_BIN_EXE_dol-parse"), &["--ci", path]);
    assert!(!out.status.success(), "trailing garbage was accepted");
}

#[test]
fn dol_check_reports_every_declaration() {
    let f = write_dol(THREE_DECLS);
    let path = f.path().to_str().unwrap();
    let out = run(env!("CARGO_BIN_EXE_dol-check"), &["--format", "json", path]);
    assert!(out.status.success(), "{}", stdout(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let names: Vec<_> = json["declarations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["a.first", "a.second", "a.third"]);
}

#[test]
fn dol_check_fails_on_trailing_garbage() {
    let f = write_dol(TRAILING_GARBAGE);
    let path = f.path().to_str().unwrap();
    let out = run(env!("CARGO_BIN_EXE_dol-check"), &["--ci", path]);
    assert!(!out.status.success(), "trailing garbage was accepted");
}

#[test]
fn dol_check_validates_later_declarations() {
    // A missing exegesis on the *second* declaration must be reported.
    let src = r#"
gen a.first {
  has id: i64
  docs {
    The first declaration has documentation.
  }
}

gen a.second {
  has id: i64
}
"#;
    let f = write_dol(src);
    let path = f.path().to_str().unwrap();
    let out = run(
        env!("CARGO_BIN_EXE_dol-check"),
        &["--require-exegesis", "--ci", path],
    );
    assert!(!out.status.success(), "{}", stdout(&out));
}
