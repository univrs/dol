//! Tests for DOL v0.8.0 type syntax changes.

use metadol::parser::Parser;

#[test]
fn test_new_lowercase_type_syntax() {
    let input = r#"
gen TestTypes {
    docs { Test new v0.8.0 lowercase type syntax }

    fun test_types(
        a: i8,
        b: i16,
        c: i32,
        d: i64,
        e: i128,
        f: u8,
        g: u16,
        h: u32,
        i: u64,
        j: u128,
        k: f32,
        l: f64,
        m: bool,
        n: string
    ) -> i32 {
        return 42;
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse new type syntax: {:?}",
        result
    );
}

#[test]
fn test_deprecated_type_syntax_with_warnings() {
    let input = r#"
gene TestDeprecatedTypes {
    exegesis { Test deprecated type syntax still works }

    fun old_types(
        a: Int8,
        b: Int16,
        c: Int32,
        d: Int64,
        e: UInt8,
        f: UInt16,
        g: UInt32,
        h: UInt64,
        i: Float32,
        j: Float64,
        k: Bool,
        l: String
    ) -> Int32 {
        return 42;
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    // Should parse successfully but emit deprecation warnings
    assert!(
        result.is_ok(),
        "Failed to parse deprecated type syntax: {:?}",
        result
    );
}

#[test]
fn test_unit_type() {
    let input = r#"
gen UnitType {
    docs { Test unit type () }

    fun no_return() -> () {
        let x = 42;
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse unit type: {:?}", result);
}

#[test]
fn test_vec_type() {
    let input = r#"
gen VecTest {
    docs { Test Vec<T> generic type }

    fun process(items: Vec<i32>) -> Vec<string> {
        return Vec::new();
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse Vec<T>: {:?}", result);
}

#[test]
fn test_option_type() {
    let input = r#"
gen OptionTest {
    docs { Test Option<T> generic type }

    fun maybe(value: Option<i32>) -> Option<string> {
        return Option::None;
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse Option<T>: {:?}", result);
}

#[test]
fn test_deprecated_list_type() {
    let input = r#"
gene ListTest {
    exegesis { Test deprecated List<T> generic type }

    fun process(items: List<Int32>) -> List<String> {
        return List::new();
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    // Should parse but emit deprecation warning for List
    assert!(result.is_ok(), "Failed to parse List<T>: {:?}", result);
}

#[test]
fn test_deprecated_optional_type() {
    let input = r#"
gene OptionalTest {
    exegesis { Test deprecated Optional<T> generic type }

    fun maybe(value: Optional<Int32>) -> Optional<String> {
        return Optional::None;
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    // Should parse but emit deprecation warning for Optional
    assert!(result.is_ok(), "Failed to parse Optional<T>: {:?}", result);
}

#[test]
fn test_mixed_old_and_new_syntax() {
    let input = r#"
gen MixedSyntax {
    docs { Test mixing old and new syntax }

    fun mixed(
        new_type: i32,
        old_type: Int32,
        new_vec: Vec<i64>,
        old_list: List<Int64>
    ) -> bool {
        return true;
    }
}
"#;

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse mixed syntax: {:?}", result);
}
