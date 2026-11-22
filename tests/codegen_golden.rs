//! Golden file tests for code generation
//! Compare generated output against expected files

use metadol::codegen::RustCodegen;
use metadol::parser::Parser;

fn normalize(s: &str) -> String {
    s.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

macro_rules! golden_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let input = include_str!(concat!("codegen/golden/input/", stringify!($name), ".dol"));
            let expected = include_str!(concat!(
                "codegen/golden/expected/",
                stringify!($name),
                ".rs"
            ));

            let ast = Parser::new(input).parse_file().expect("Parse failed");
            // Use static method form since generate takes &Declaration
            let actual = ast
                .declarations
                .iter()
                .map(|decl| RustCodegen::generate(decl))
                .collect::<Vec<_>>()
                .join("\n\n");

            assert_eq!(
                normalize(&actual),
                normalize(expected),
                "\n=== ACTUAL ===\n{}\n=== EXPECTED ===\n{}",
                actual,
                expected
            );
        }
    };
}

/// Golden test for a non-Rust target: `$codegen::generate_all` output is
/// compared against `expected/<name>.<ext>`.
macro_rules! target_golden_test {
    ($test:ident, $name:literal, $codegen:ty, $ext:literal) => {
        #[test]
        fn $test() {
            let input = include_str!(concat!("codegen/golden/input/", $name, ".dol"));
            let expected = include_str!(concat!("codegen/golden/expected/", $name, ".", $ext));

            let ast = Parser::new(input).parse_file().expect("Parse failed");
            let actual = <$codegen>::generate_all(&ast.declarations);

            assert_eq!(
                normalize(&actual),
                normalize(expected),
                "\n=== ACTUAL ===\n{}\n=== EXPECTED ===\n{}",
                actual,
                expected
            );
        }
    };
}

target_golden_test!(
    typescript_typed_fields,
    "typed_fields",
    metadol::codegen::TypeScriptCodegen,
    "ts"
);
target_golden_test!(
    jsonschema_typed_fields,
    "typed_fields",
    metadol::codegen::JsonSchemaCodegen,
    "schema.json"
);

golden_test!(simple_gene);
golden_test!(gene_with_fields);
golden_test!(function);
// golden_test!(pipe_operators);
