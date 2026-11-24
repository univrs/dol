//! Integration tests for the `dol-migrate` binary (requires the `cli` feature).
#![cfg(feature = "cli")]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const V07_SOURCE: &str = r#"// This gene comment mentions gene, constraint and Int32.
gene container.exists {
    container has identity: String
    container has label: "a gene label with Int32"
}

exegesis {
    A gene describes a container. Each constraint and every String,
    List<Int32> or Bool written here is prose { nested } and must stay.
}
"#;

fn migrate(args: &[&str], path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dol-migrate"))
        .args(args)
        .arg(path)
        .output()
        .expect("run dol-migrate")
}

fn write_source(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("container.dol");
    fs::write(&path, V07_SOURCE).unwrap();
    path
}

#[test]
fn diff_alone_does_not_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_source(dir.path());
    let out = migrate(&["0.7-to-0.8", "--diff"], &path);
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("+ gen container.exists"), "{stdout}");
    assert_eq!(fs::read_to_string(&path).unwrap(), V07_SOURCE);
}

#[test]
fn diff_with_write_shows_and_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_source(dir.path());
    let out = migrate(&["0.7-to-0.8", "--diff", "--write"], &path);
    assert!(out.status.success(), "{:?}", out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("+ gen container.exists"));
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("gen container.exists"));
}

#[test]
fn dry_run_does_not_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_source(dir.path());
    let out = migrate(&["0.7-to-0.8", "--dry-run"], &path);
    assert!(out.status.success(), "{:?}", out);
    assert_eq!(fs::read_to_string(&path).unwrap(), V07_SOURCE);
}

#[test]
fn default_writes_and_never_alters_docs_comments_or_strings() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_source(dir.path());
    let out = migrate(&["0.7-to-0.8"], &path);
    assert!(out.status.success(), "{:?}", out);
    let migrated = fs::read_to_string(&path).unwrap();

    // Code positions are migrated.
    assert!(migrated.contains("gen container.exists {"), "{migrated}");
    assert!(migrated.contains("identity: string"), "{migrated}");
    assert!(migrated.contains("docs {"), "{migrated}");

    // Negative oracle: prose is never altered.
    assert!(migrated.contains("// This gene comment mentions gene, constraint and Int32."));
    assert!(migrated.contains("\"a gene label with Int32\""));
    assert!(migrated.contains(
        "    A gene describes a container. Each constraint and every String,\n    \
         List<Int32> or Bool written here is prose { nested } and must stay.\n"
    ));
}
