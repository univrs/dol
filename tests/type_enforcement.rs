//! Type enforcement in the validator (`ValidationOptions { typecheck: true }`,
//! i.e. `dol-check --typecheck`).
//!
//! The "should fail" programs were all accepted before the validator wired the
//! type checker in (`validate_statement_types` was an empty placeholder).

use metadol::parse_dol_file;
use metadol::validator::{validate_file_with_options, FileValidationResult, ValidationOptions};

fn typecheck(source: &str) -> FileValidationResult {
    let file = parse_dol_file(source).expect("test program must parse");
    validate_file_with_options(&file, &ValidationOptions { typecheck: true })
}

fn type_errors(source: &str) -> Vec<String> {
    typecheck(source)
        .all_errors()
        .into_iter()
        .map(|e| e.to_string())
        .filter(|m| m.contains("type"))
        .collect()
}

fn assert_type_error(source: &str, needle: &str) {
    let errors = type_errors(source);
    assert!(
        errors.iter().any(|e| e.contains(needle)),
        "expected a type error containing {needle:?}, got {errors:?}"
    );
}

fn assert_well_typed(source: &str) {
    let result = typecheck(source);
    let errors: Vec<String> = result.all_errors().iter().map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "expected no errors, got {errors:?}");
}

// ---------------------------------------------------------------------------
// Programs that must FAIL type checking
// ---------------------------------------------------------------------------

#[test]
fn rejects_string_default_for_u32_field() {
    assert_type_error(
        r#"
gen counter.state {
  has count: u32 = "zero"
  docs { A counter whose default is the wrong type. }
}
"#,
        "count",
    );
}

#[test]
fn rejects_call_with_wrong_argument_type() {
    assert_type_error(
        r#"
fun double(x: i64) -> i64 {
  return x * 2
}

fun caller() -> i64 {
  return double("two")
}
"#,
        "argument",
    );
}

#[test]
fn rejects_undefined_field_type() {
    assert_type_error(
        r#"
gen order.line {
  has price: Moneyy
  docs { Refers to a type that is declared nowhere. }
}
"#,
        "Moneyy",
    );
}

#[test]
fn rejects_return_of_wrong_type() {
    assert_type_error(
        r#"
fun is_ready() -> bool {
  return "yes"
}
"#,
        "return type mismatch: expected Bool",
    );
}

#[test]
fn rejects_let_annotation_mismatch() {
    assert_type_error(
        r#"
fun compute() -> i64 {
  let n: i64 = true
  return n
}
"#,
        "type mismatch",
    );
}

#[test]
fn rejects_non_bool_while_condition() {
    assert_type_error(
        r#"
fun spin(n: i64) -> i64 {
  while n {
    n = n - 1
  }
  return n
}
"#,
        "type mismatch",
    );
}

#[test]
fn rejects_wrong_typed_const() {
    assert_type_error(
        r#"
const LIMIT: i32 = "many"
"#,
        "LIMIT",
    );
}

// ---------------------------------------------------------------------------
// Programs that must PASS type checking
// ---------------------------------------------------------------------------

#[test]
fn accepts_well_typed_gene_defaults() {
    assert_well_typed(
        r#"
gen counter.state {
  has count: u32 = 0
  has label: string = "counter"
  has enabled: bool = true
  has ratio: f64 = 0.5
  docs { A counter with correctly typed defaults. }
}
"#,
    );
}

#[test]
fn accepts_well_typed_function_calls() {
    assert_well_typed(
        r#"
fun double(x: i64) -> i64 {
  return x * 2
}

fun caller() -> i64 {
  let n: i64 = double(21)
  return n
}
"#,
    );
}

#[test]
fn accepts_user_defined_field_types() {
    assert_well_typed(
        r#"
gen money.amount {
  has cents: i64
  docs { An amount of money. }
}

gen order.line {
  has price: money.amount
  has quantity: u32 = 1
  docs { A line of an order. }
}
"#,
    );
}

#[test]
fn accepts_unresolvable_names_gradually() {
    // Identifiers the validator cannot resolve (builtins, host functions,
    // imports) are typed as unknown rather than rejected.
    assert_well_typed(
        r#"
fun greet(name: string) -> i64 {
  println("hello " + name)
  return 0
}
"#,
    );
}

#[test]
fn types_from_use_imports_are_not_reported_as_undefined() {
    assert_well_typed(
        r#"
use shared.types

gen order.line {
  has price: Money
  docs { Money comes from an import the validator cannot resolve. }
}
"#,
    );
}

#[test]
fn typecheck_off_by_default() {
    // Without `typecheck: true` the validator keeps its old behaviour.
    let file = parse_dol_file(
        r#"
gen counter.state {
  has count: u32 = "zero"
  docs { Type checking is opt-in. }
}
"#,
    )
    .unwrap();
    let result = validate_file_with_options(&file, &ValidationOptions::default());
    assert!(result.is_valid());
}

// ---------------------------------------------------------------------------
// Deferred: not enforced yet
// ---------------------------------------------------------------------------

#[test]
#[ignore = "deferred: numeric widths and signedness are not enforced; all numeric types are mutually compatible"]
fn rejects_negative_default_for_unsigned_field() {
    assert_type_error(
        r#"
gen counter.state {
  has count: u32 = -1
  docs { Negative default for an unsigned field. }
}
"#,
        "count",
    );
}

#[test]
#[ignore = "deferred: field access is untyped (`a.b` infers as unknown)"]
fn rejects_field_access_of_wrong_type() {
    assert_type_error(
        r#"
gen point.xy {
  has x: i64
  has label: string
  fun bad() -> i64 {
    return this.label
  }
  docs { Returns a string field from an i64 function. }
}
"#,
        "return type mismatch",
    );
}

#[test]
#[ignore = "deferred: generic/user-defined types are not compared structurally (Vec<i64> vs Vec<string>)"]
fn rejects_generic_argument_mismatch() {
    assert_type_error(
        r#"
fun first(xs: Vec<i64>) -> i64 {
  return 0
}

fun caller() -> i64 {
  let names: Vec<string> = ["a"]
  return first(names)
}
"#,
        "argument",
    );
}

// ---------------------------------------------------------------------------
// dol-check --typecheck
// ---------------------------------------------------------------------------

#[cfg(feature = "cli")]
mod cli {
    use std::process::{Command, Output};

    const BAD: &str = r#"
gen counter.state {
  has count: u32 = "zero"
  docs { A counter whose default is the wrong type. }
}
"#;

    fn dol_check(args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dol-check"))
            .args(args)
            .output()
            .expect("run dol-check")
    }

    #[test]
    fn typecheck_flag_rejects_type_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.dol");
        std::fs::write(&path, BAD).unwrap();
        let path = path.to_str().unwrap();

        let out = dol_check(&["--typecheck", "--ci", path]);
        assert!(!out.status.success(), "type error was accepted");

        // Type checking is opt-in: the same file passes without the flag.
        let out = dol_check(&["--ci", path]);
        assert!(out.status.success());
    }

    #[test]
    fn typecheck_resolves_types_from_sibling_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("money.dol"),
            "gen Money {\n  has cents: i64\n  docs { An amount of money. }\n}\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("order.dol"),
            "gen OrderLine {\n  has price: Money\n  docs { Uses Money from money.dol. }\n}\n",
        )
        .unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let out = dol_check(&["--typecheck", "--ci", dir_path]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );

        // Checked alone, Money is declared nowhere.
        let order = dir.path().join("order.dol");
        let out = dol_check(&["--typecheck", "--ci", order.to_str().unwrap()]);
        assert!(!out.status.success(), "undefined type was accepted");
    }
}
