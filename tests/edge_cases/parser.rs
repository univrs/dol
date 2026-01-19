//! Parser Edge Case Tests for DOL
//!
//! Tests parser edge cases to stress-test the recursive descent parser:
//! - Deeply nested expressions (100+ levels)
//! - Very long identifiers
//! - Unicode in strings and identifiers
//! - Escape sequences
//! - Empty modules/spirits
//! - Maximum token length
//! - Ambiguous grammar cases
//! - Reserved words as identifiers in paths
//!
//! These tests help discover bugs in the lexer and parser.

use metadol::parser::Parser;
use metadol::{parse_file, parse_file_all};

// ============================================================================
// DEEPLY NESTED EXPRESSION TESTS
// ============================================================================

mod deeply_nested {
    use super::*;

    #[test]
    fn nested_parentheses_10_levels() {
        let input = "((((((((((42))))))))))";
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        assert!(result.is_ok(), "10 levels of nested parens should parse");
    }

    #[test]
    fn nested_parentheses_50_levels() {
        let depth = 50;
        let open = "(".repeat(depth);
        let close = ")".repeat(depth);
        let input = format!("{}42{}", open, close);

        let mut parser = Parser::new(&input);
        let result = parser.parse_expr(0);

        assert!(
            result.is_ok(),
            "50 levels of nested parens should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn nested_parentheses_100_levels() {
        let depth = 100;
        let open = "(".repeat(depth);
        let close = ")".repeat(depth);
        let input = format!("{}42{}", open, close);

        let mut parser = Parser::new(&input);
        let result = parser.parse_expr(0);

        // This may cause stack overflow or fail - document behavior
        match result {
            Ok(_) => println!("NOTE: 100 levels of nesting parses successfully"),
            Err(e) => println!("NOTE: 100 levels of nesting fails: {:?}", e),
        }
    }

    #[test]
    fn nested_binary_operators_deep() {
        // a + b + c + d + ... (50 levels)
        let depth = 50;
        let vars: Vec<String> = (0..depth).map(|i| format!("x{}", i)).collect();
        let input = vars.join(" + ");

        let mut parser = Parser::new(&input);
        let result = parser.parse_expr(0);

        assert!(
            result.is_ok(),
            "50 chained additions should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn nested_function_calls_deep() {
        // f(g(h(i(j(k(1))))))
        let depth = 20;
        let opens: String = (0..depth).map(|i| format!("f{}(", i)).collect();
        let closes = ")".repeat(depth);
        let input = format!("{}42{}", opens, closes);

        let mut parser = Parser::new(&input);
        let result = parser.parse_expr(0);

        // Document behavior
        match result {
            Ok(_) => println!("NOTE: 20 nested function calls parse successfully"),
            Err(e) => println!("NOTE: 20 nested function calls fail: {:?}", e),
        }
    }

    #[test]
    fn nested_if_expressions() {
        // if a { if b { if c { 1 } else { 2 } } else { 3 } } else { 4 }
        let input = r#"
fun test() -> i64 {
    if true {
        if false {
            if true {
                if false {
                    if true {
                        1
                    } else {
                        2
                    }
                } else {
                    3
                }
            } else {
                4
            }
        } else {
            5
        }
    } else {
        6
    }
}
"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "5 nested if expressions should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn nested_blocks() {
        // Deeply nested block expressions
        let input = r#"
fun test() -> i64 {
    {
        {
            {
                {
                    {
                        42
                    }
                }
            }
        }
    }
}
"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Nested blocks should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn nested_array_literals() {
        // [[[[[[1]]]]]]
        let depth = 10;
        let open = "[".repeat(depth);
        let close = "]".repeat(depth);
        let input = format!("{}1{}", open, close);

        let mut parser = Parser::new(&input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: Nested arrays parse: {}", input),
            Err(e) => println!("NOTE: Nested arrays fail: {:?}", e),
        }
    }
}

// ============================================================================
// UNICODE TESTS
// ============================================================================

mod unicode {
    use super::*;

    #[test]
    fn unicode_string_basic() {
        let input = r#""Hello, \u4e16\u754c""#; // Hello, 世界 (escaped)
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        assert!(
            result.is_ok(),
            "Unicode escape in string should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn unicode_string_emoji() {
        let input = r#""Hello 🌍🚀✨""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        assert!(
            result.is_ok(),
            "Emoji in string should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn unicode_string_various_scripts() {
        // Test various Unicode scripts in strings
        let inputs = vec![
            r#""Привет мир""#,     // Russian
            r#""こんにちは世界""#, // Japanese
            r#""你好世界""#,       // Chinese
            r#""مرحبا بالعالم""#,  // Arabic
            r#""שלום עולם""#,      // Hebrew
            r#""Γειά σου κόσμε""#, // Greek
        ];

        for input in inputs {
            let mut parser = Parser::new(input);
            let result = parser.parse_expr(0);
            assert!(
                result.is_ok(),
                "Unicode script should parse: {} - {:?}",
                input,
                result.err()
            );
        }
    }

    #[test]
    fn unicode_identifier_basic() {
        // Some languages allow Unicode identifiers
        let input = r#"
gen Données {
    données has valeur: i64
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Document whether Unicode identifiers are supported
        match result {
            Ok(_) => println!("NOTE: Unicode identifiers (French) are supported"),
            Err(e) => println!("NOTE: Unicode identifiers not supported: {:?}", e),
        }
    }

    #[test]
    fn unicode_string_null_byte() {
        // Test null byte in string (may cause issues)
        let input = "\"hello\\0world\"";
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: Null byte escape in string is supported"),
            Err(e) => println!("NOTE: Null byte escape fails: {:?}", e),
        }
    }

    #[test]
    fn unicode_bom_at_start() {
        // Test BOM (Byte Order Mark) at file start
        let input = "\u{FEFF}gen Test { test has value }";
        let mut parser = Parser::new(input);
        let result = parser.parse();

        match result {
            Ok(_) => println!("NOTE: BOM at start is handled gracefully"),
            Err(e) => println!("NOTE: BOM at start causes error: {:?}", e),
        }
    }

    #[test]
    fn unicode_zero_width_joiner() {
        // Test zero-width joiner in identifier
        let input = "let a\u{200D}b = 42";
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: Zero-width joiner in identifier accepted"),
            Err(e) => println!("NOTE: Zero-width joiner rejected: {:?}", e),
        }
    }
}

// ============================================================================
// ESCAPE SEQUENCE TESTS
// ============================================================================

mod escape_sequences {
    use super::*;

    #[test]
    fn escape_newline() {
        let input = r#""hello\nworld""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);
        assert!(result.is_ok(), "Newline escape should parse");
    }

    #[test]
    fn escape_tab() {
        let input = r#""hello\tworld""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);
        assert!(result.is_ok(), "Tab escape should parse");
    }

    #[test]
    fn escape_carriage_return() {
        let input = r#""hello\rworld""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);
        assert!(result.is_ok(), "Carriage return escape should parse");
    }

    #[test]
    fn escape_backslash() {
        let input = r#""path\\to\\file""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);
        assert!(result.is_ok(), "Backslash escape should parse");
    }

    #[test]
    fn escape_quote() {
        let input = r#""He said \"Hello\"""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);
        assert!(result.is_ok(), "Quote escape should parse");
    }

    #[test]
    fn escape_single_quote() {
        let input = r#""It\'s fine""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: Single quote escape is supported"),
            Err(_) => println!("NOTE: Single quote escape not needed in double-quoted strings"),
        }
    }

    #[test]
    fn escape_unicode_4_digit() {
        let input = r#""\u0041""#; // Should be 'A'
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: 4-digit Unicode escape is supported"),
            Err(e) => println!("NOTE: 4-digit Unicode escape fails: {:?}", e),
        }
    }

    #[test]
    fn escape_unicode_8_digit() {
        let input = r#""\U0001F600""#; // Should be grinning face emoji
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: 8-digit Unicode escape is supported"),
            Err(e) => println!("NOTE: 8-digit Unicode escape fails: {:?}", e),
        }
    }

    #[test]
    fn escape_hex_byte() {
        let input = r#""\x41""#; // Should be 'A'
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: Hex byte escape is supported"),
            Err(e) => println!("NOTE: Hex byte escape fails: {:?}", e),
        }
    }

    #[test]
    fn escape_invalid() {
        let input = r#""\z""#; // Invalid escape
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: Invalid escape \\z is accepted (passed through?)"),
            Err(e) => println!("NOTE: Invalid escape \\z correctly rejected: {:?}", e),
        }
    }

    #[test]
    fn escape_at_end_of_string() {
        let input = r#""test\"#; // Trailing backslash
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        // Should fail - unterminated escape
        assert!(result.is_err(), "Trailing backslash should fail to parse");
    }
}

// ============================================================================
// EMPTY MODULE TESTS
// ============================================================================

mod empty_constructs {
    use super::*;

    #[test]
    fn empty_gene() {
        let input = r#"
gen Empty {
}
"#;
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Empty gene should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn empty_trait() {
        let input = r#"
trait Empty {
}
"#;
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Empty trait should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn empty_rule() {
        let input = r#"
rule Empty {
}
"#;
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Empty rule should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn empty_function_body() {
        let input = r#"
fun empty() -> () {
}
"#;
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Empty function body should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn empty_docs_block() {
        let input = r#"
docs {
}

gen Test {
    test has value
}
"#;
        let result = parse_file(input);
        match result {
            Ok(_) => println!("NOTE: Empty docs block is accepted"),
            Err(e) => println!("NOTE: Empty docs block fails: {:?}", e),
        }
    }

    #[test]
    fn empty_file() {
        let input = "";
        let result = parse_file_all(input);

        match result {
            Ok(decls) => {
                assert!(
                    decls.is_empty(),
                    "Empty file should produce no declarations"
                );
            }
            Err(e) => println!("NOTE: Empty file produces error: {:?}", e),
        }
    }

    #[test]
    fn whitespace_only_file() {
        let input = "   \n\t\n   ";
        let result = parse_file_all(input);

        match result {
            Ok(decls) => {
                println!(
                    "NOTE: Whitespace-only file produces {} declarations",
                    decls.len()
                );
            }
            Err(e) => println!("NOTE: Whitespace-only file error: {:?}", e),
        }
    }

    #[test]
    fn comments_only_file() {
        let input = r#"
// This is a comment
// Another comment

/* Block comment */

// More comments
"#;
        let result = parse_file_all(input);

        match result {
            Ok(decls) => {
                println!(
                    "NOTE: Comments-only file produces {} declarations",
                    decls.len()
                );
            }
            Err(e) => println!("NOTE: Comments-only file error: {:?}", e),
        }
    }
}

// ============================================================================
// LONG IDENTIFIER TESTS
// ============================================================================

mod long_identifiers {
    use super::*;

    #[test]
    fn identifier_100_chars() {
        let name = "a".repeat(100);
        let input = format!("gen {} {{ {} has value }}\n", name, name.to_lowercase());
        let result = parse_file(&input);

        assert!(
            result.is_ok(),
            "100-char identifier should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn identifier_1000_chars() {
        let name = "a".repeat(1000);
        let input = format!("gen {} {{ {} has value }}\n", name, name.to_lowercase());
        let result = parse_file(&input);

        match result {
            Ok(_) => println!("NOTE: 1000-char identifier parses successfully"),
            Err(e) => println!("NOTE: 1000-char identifier fails: {:?}", e),
        }
    }

    #[test]
    fn identifier_10000_chars() {
        let name = "a".repeat(10000);
        let input = format!("gen {} {{ {} has value }}\n", name, name.to_lowercase());
        let result = parse_file(&input);

        match result {
            Ok(_) => println!("NOTE: 10000-char identifier parses successfully"),
            Err(e) => println!("NOTE: 10000-char identifier fails: {:?}", e),
        }
    }

    #[test]
    fn deeply_qualified_identifier() {
        // a.b.c.d.e.f.g.h.i.j (10 levels)
        let segments: Vec<&str> = (0..10).map(|_| "segment").collect();
        let name = segments.join(".");
        let input = format!("gen {} {{ item has value }}\n", name);

        let result = parse_file(&input);
        assert!(
            result.is_ok(),
            "10-segment qualified name should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn very_long_string_literal() {
        let content = "x".repeat(10000);
        let input = format!("\"{}\"", content);

        let mut parser = Parser::new(&input);
        let result = parser.parse_expr(0);

        match result {
            Ok(_) => println!("NOTE: 10000-char string literal parses successfully"),
            Err(e) => println!("NOTE: 10000-char string literal fails: {:?}", e),
        }
    }
}

// ============================================================================
// AMBIGUOUS GRAMMAR TESTS
// ============================================================================

mod ambiguous_grammar {
    use super::*;

    #[test]
    fn dangling_else() {
        // Classic dangling else ambiguity
        let input = r#"
fun test() -> i64 {
    if a {
        if b {
            1
        }
    } else {
        2
    }
}
"#;
        let result = parse_file(input);

        assert!(
            result.is_ok(),
            "Dangling else should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn function_call_vs_grouping() {
        // f(x) could be call or identifier followed by grouping
        let input = "f(x)";
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        assert!(result.is_ok(), "f(x) should parse as function call");
    }

    #[test]
    fn array_index_vs_array_literal() {
        // a[0] vs [0] - index vs literal
        let inputs = vec!["a[0]", "[0]", "a[0][1]", "[1, 2, 3]"];

        for input in inputs {
            let mut parser = Parser::new(input);
            let result = parser.parse_expr(0);
            assert!(result.is_ok(), "{} should parse: {:?}", input, result.err());
        }
    }

    #[test]
    fn minus_as_binary_vs_unary() {
        // a-b vs a - b vs a- b vs a -b
        let inputs = vec!["a-b", "a - b", "a- b", "a -b", "-a", "--a", "a--b"];

        for input in inputs {
            let mut parser = Parser::new(input);
            let result = parser.parse_expr(0);
            println!("'{}' parse result: {:?}", input, result.is_ok());
        }
    }

    #[test]
    fn generics_vs_comparison() {
        // Vec<T> vs a < b
        let inputs = vec!["a < b", "a > b", "a<b>()", "Vec<T>"];

        for input in inputs {
            let mut parser = Parser::new(input);
            let result = parser.parse_expr(0);
            println!("'{}' parse result: {:?}", input, result.is_ok());
        }
    }

    #[test]
    fn pipe_operator_precedence() {
        // x |> f |> g - should be left associative
        let input = "x |> f |> g |> h";
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        assert!(
            result.is_ok(),
            "Pipe chain should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn compose_vs_shift() {
        // >> could be compose or right shift
        let inputs = vec!["f >> g", "a >> 2", "f >> g >> h"];

        for input in inputs {
            let mut parser = Parser::new(input);
            let result = parser.parse_expr(0);
            println!("'{}' parse result: {:?}", input, result.is_ok());
        }
    }
}

// ============================================================================
// RESERVED WORD TESTS
// ============================================================================

mod reserved_words {
    use super::*;

    #[test]
    fn reserved_word_in_path() {
        // gen.fun.trait - using reserved words in qualified paths
        let input = r#"
gen module.fun {
    item has trait
}
"#;
        let result = parse_file(input);

        match result {
            Ok(_) => println!("NOTE: Reserved words in qualified paths are allowed"),
            Err(e) => println!(
                "NOTE: Reserved words in qualified paths are forbidden: {:?}",
                e
            ),
        }
    }

    #[test]
    fn reserved_word_as_field() {
        // Using reserved words as field names
        let input = r#"
gen Test {
    test has fun: i64
    test has gen: string
}
"#;
        let result = parse_file(input);

        match result {
            Ok(_) => println!("NOTE: Reserved words as field names are allowed"),
            Err(e) => println!("NOTE: Reserved words as field names are forbidden: {:?}", e),
        }
    }

    #[test]
    fn underscore_identifiers() {
        // _name, __name, _
        let inputs = vec!["_", "_x", "__x", "_x_y", "___"];

        for input in inputs {
            let expr_input = format!("let {} = 42", input);
            let mut parser = Parser::new(&expr_input);
            let result = parser.parse_expr(0);
            println!("'{}' as identifier: {:?}", input, result.is_ok());
        }
    }

    #[test]
    fn number_like_identifiers() {
        // Identifiers that look like numbers
        let inputs = vec!["x1", "x123", "_1", "x1y2z3"];

        for input in inputs {
            let expr_input = format!("let {} = 42", input);
            let mut parser = Parser::new(&expr_input);
            let result = parser.parse_expr(0);
            println!("'{}' as identifier: {:?}", input, result.is_ok());
        }
    }

    #[test]
    fn soft_keywords() {
        // Words that might be contextually reserved
        let words = vec![
            "async", "await", "yield", "type", "where", "impl", "self", "super", "crate",
        ];

        for word in words {
            let input = format!("gen {} {{ {} has value }}\n", word.to_uppercase(), word);
            let result = parse_file(&input);
            println!("'{}' as identifier: {:?}", word, result.is_ok());
        }
    }
}

// ============================================================================
// WHITESPACE HANDLING TESTS
// ============================================================================

mod whitespace {
    use super::*;

    #[test]
    fn minimal_whitespace() {
        let input = "gen Test{test has value}";
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Minimal whitespace should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn excessive_whitespace() {
        let input = r#"


        gen     Test    {


            test     has     value


        }


"#;
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Excessive whitespace should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn tabs_vs_spaces() {
        let input = "gen\tTest\t{\n\ttest\thas\tvalue\n}";
        let result = parse_file(input);
        assert!(result.is_ok(), "Tabs should parse: {:?}", result.err());
    }

    #[test]
    fn crlf_line_endings() {
        let input = "gen Test {\r\n    test has value\r\n}\r\n";
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "CRLF line endings should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn mixed_line_endings() {
        let input = "gen Test {\n    test has value\r\n}\r";
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Mixed line endings should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn trailing_whitespace() {
        let input = "gen Test {    \n    test has value    \n}    ";
        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Trailing whitespace should parse: {:?}",
            result.err()
        );
    }
}

// ============================================================================
// COMMENT EDGE CASES
// ============================================================================

mod comments {
    use super::*;

    #[test]
    fn nested_block_comments() {
        let input = r#"
/* outer /* nested */ still outer */
gen Test { test has value }
"#;
        let result = parse_file(input);

        match result {
            Ok(_) => println!("NOTE: Nested block comments are supported"),
            Err(e) => println!("NOTE: Nested block comments fail: {:?}", e),
        }
    }

    #[test]
    fn comment_in_string() {
        let input = r#""hello // not a comment""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        assert!(result.is_ok(), "// inside string should not be comment");
    }

    #[test]
    fn block_comment_in_string() {
        let input = r#""hello /* not a comment */ world""#;
        let mut parser = Parser::new(input);
        let result = parser.parse_expr(0);

        assert!(result.is_ok(), "/* */ inside string should not be comment");
    }

    #[test]
    fn unclosed_block_comment() {
        let input = r#"
/* This comment never ends
gen Test { test has value }
"#;
        let result = parse_file(input);

        assert!(result.is_err(), "Unclosed block comment should fail");
    }

    #[test]
    fn comment_at_eof() {
        let input = "gen Test { test has value } // comment at end";
        let result = parse_file(input);

        assert!(
            result.is_ok(),
            "Comment at EOF should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn doc_comment_style() {
        let input = r#"
/// Documentation comment
gen Test {
    test has value
}
"#;
        let result = parse_file(input);

        match result {
            Ok(_) => println!("NOTE: Triple-slash doc comments are supported"),
            Err(e) => println!("NOTE: Triple-slash doc comments fail: {:?}", e),
        }
    }
}
