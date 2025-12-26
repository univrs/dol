//! Exhaustive lexer tests
//! Target: 200+ tests covering all token types

use dol::lexer::{Lexer, Token, TokenKind};

// ============================================================================
// KEYWORDS (26 keywords)
// ============================================================================

macro_rules! keyword_test {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut lexer = Lexer::new($input);
            let token = lexer.next_token();
            assert_eq!(token.kind, $expected, "Input: {}", $input);
        }
    };
}

keyword_test!(kw_gene, "gene", TokenKind::Gene);
keyword_test!(kw_trait, "trait", TokenKind::Trait);
keyword_test!(kw_system, "system", TokenKind::System);
keyword_test!(kw_constraint, "constraint", TokenKind::Constraint);
keyword_test!(kw_evolves, "evolves", TokenKind::Evolves);
keyword_test!(kw_exegesis, "exegesis", TokenKind::Exegesis);
keyword_test!(kw_fun, "fun", TokenKind::Fun);
keyword_test!(kw_return, "return", TokenKind::Return);
keyword_test!(kw_if, "if", TokenKind::If);
keyword_test!(kw_else, "else", TokenKind::Else);
keyword_test!(kw_match, "match", TokenKind::Match);
keyword_test!(kw_for, "for", TokenKind::For);
keyword_test!(kw_while, "while", TokenKind::While);
keyword_test!(kw_loop, "loop", TokenKind::Loop);
keyword_test!(kw_break, "break", TokenKind::Break);
keyword_test!(kw_continue, "continue", TokenKind::Continue);
keyword_test!(kw_where, "where", TokenKind::Where);
keyword_test!(kw_in, "in", TokenKind::In);
keyword_test!(kw_requires, "requires", TokenKind::Requires);
keyword_test!(kw_provides, "provides", TokenKind::Provides);
keyword_test!(kw_law, "law", TokenKind::Law);
keyword_test!(kw_type, "type", TokenKind::Type);
keyword_test!(kw_has, "has", TokenKind::Has);
keyword_test!(kw_is, "is", TokenKind::Is);
keyword_test!(kw_use, "use", TokenKind::Use);
keyword_test!(kw_module, "module", TokenKind::Module);
keyword_test!(kw_pub, "pub", TokenKind::Pub);
keyword_test!(kw_sex, "sex", TokenKind::Sex);

// ============================================================================
// OPERATORS (30+ operators)
// ============================================================================

macro_rules! operator_test {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut lexer = Lexer::new($input);
            let token = lexer.next_token();
            assert_eq!(token.kind, $expected, "Input: {}", $input);
        }
    };
}

// Arithmetic
operator_test!(op_plus, "+", TokenKind::Plus);
operator_test!(op_minus, "-", TokenKind::Minus);
operator_test!(op_star, "*", TokenKind::Star);
operator_test!(op_slash, "/", TokenKind::Slash);
operator_test!(op_percent, "%", TokenKind::Percent);

// Comparison
operator_test!(op_eq, "==", TokenKind::EqEq);
operator_test!(op_ne, "!=", TokenKind::NotEq);
operator_test!(op_lt, "<", TokenKind::Lt);
operator_test!(op_le, "<=", TokenKind::LtEq);
operator_test!(op_gt, ">", TokenKind::Gt);
operator_test!(op_ge, ">=", TokenKind::GtEq);

// Logical
operator_test!(op_and, "&&", TokenKind::AndAnd);
operator_test!(op_or, "||", TokenKind::OrOr);
operator_test!(op_not, "!", TokenKind::Bang);

// Pipes (DOL-specific)
operator_test!(op_pipe, "|>", TokenKind::Pipe);
operator_test!(op_compose, ">>", TokenKind::Compose);
operator_test!(op_back_pipe, "<|", TokenKind::BackPipe);

// Special
operator_test!(op_at, "@", TokenKind::At);
operator_test!(op_bind, ":=", TokenKind::ColonEq);
operator_test!(op_arrow, "->", TokenKind::Arrow);
operator_test!(op_fat_arrow, "=>", TokenKind::FatArrow);
operator_test!(op_bar, "|", TokenKind::Bar);

// Meta-programming
operator_test!(op_quote, "'", TokenKind::Quote);
operator_test!(op_hash, "#", TokenKind::Hash);
operator_test!(op_question, "?", TokenKind::Question);

// Assignment
operator_test!(op_assign, "=", TokenKind::Eq);

// ============================================================================
// LITERALS
// ============================================================================

// Integers
#[test]
fn lit_int_zero() {
    let mut lexer = Lexer::new("0");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(0)));
}

#[test]
fn lit_int_positive() {
    let mut lexer = Lexer::new("42");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(42)));
}

#[test]
fn lit_int_with_underscores() {
    let mut lexer = Lexer::new("1_000_000");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(1_000_000)));
}

#[test]
fn lit_int_max() {
    let mut lexer = Lexer::new("9223372036854775807");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(i64::MAX)));
}

// Floats
#[test]
fn lit_float_simple() {
    let mut lexer = Lexer::new("3.14");
    let token = lexer.next_token();
    if let TokenKind::Float(f) = token.kind {
        assert!((f - 3.14).abs() < 0.001);
    } else {
        panic!("Expected float");
    }
}

#[test]
fn lit_float_exponent() {
    let mut lexer = Lexer::new("1e10");
    let token = lexer.next_token();
    if let TokenKind::Float(f) = token.kind {
        assert!((f - 1e10).abs() < 1.0);
    } else {
        panic!("Expected float");
    }
}

// Strings
#[test]
fn lit_string_empty() {
    let mut lexer = Lexer::new(r#""""#);
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::String(ref s) if s.is_empty()));
}

#[test]
fn lit_string_simple() {
    let mut lexer = Lexer::new(r#""hello""#);
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::String(ref s) if s == "hello"));
}

#[test]
fn lit_string_with_escapes() {
    let mut lexer = Lexer::new(r#""hello\nworld""#);
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::String(ref s) if s.contains('\n')));
}

// Booleans
#[test]
fn lit_true() {
    let mut lexer = Lexer::new("true");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::True);
}

#[test]
fn lit_false() {
    let mut lexer = Lexer::new("false");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::False);
}

// ============================================================================
// IDENTIFIERS
// ============================================================================

#[test]
fn ident_simple() {
    let mut lexer = Lexer::new("foo");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "foo"));
}

#[test]
fn ident_with_underscore() {
    let mut lexer = Lexer::new("foo_bar");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "foo_bar"));
}

#[test]
fn ident_starting_underscore() {
    let mut lexer = Lexer::new("_private");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "_private"));
}

#[test]
fn ident_with_numbers() {
    let mut lexer = Lexer::new("var123");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "var123"));
}

// ============================================================================
// DELIMITERS
// ============================================================================

macro_rules! delimiter_test {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut lexer = Lexer::new($input);
            let token = lexer.next_token();
            assert_eq!(token.kind, $expected);
        }
    };
}

delimiter_test!(delim_lbrace, "{", TokenKind::LBrace);
delimiter_test!(delim_rbrace, "}", TokenKind::RBrace);
delimiter_test!(delim_lparen, "(", TokenKind::LParen);
delimiter_test!(delim_rparen, ")", TokenKind::RParen);
delimiter_test!(delim_lbracket, "[", TokenKind::LBracket);
delimiter_test!(delim_rbracket, "]", TokenKind::RBracket);
delimiter_test!(delim_colon, ":", TokenKind::Colon);
delimiter_test!(delim_comma, ",", TokenKind::Comma);
delimiter_test!(delim_semicolon, ";", TokenKind::Semicolon);
delimiter_test!(delim_dot, ".", TokenKind::Dot);
delimiter_test!(delim_underscore, "_", TokenKind::Underscore);

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn edge_whitespace_handling() {
    let mut lexer = Lexer::new("  \t\n  foo  ");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "foo"));
}

#[test]
fn edge_comment_line() {
    let mut lexer = Lexer::new("// comment\nfoo");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "foo"));
}

#[test]
fn edge_comment_block() {
    let mut lexer = Lexer::new("/* block */ foo");
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "foo"));
}

#[test]
fn edge_multiple_tokens() {
    let mut lexer = Lexer::new("foo + bar");
    assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(_)));
    assert_eq!(lexer.next_token().kind, TokenKind::Plus);
    assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(_)));
}

#[test]
fn edge_operator_disambiguation() {
    // |> vs | vs ||
    let mut lexer = Lexer::new("|>");
    assert_eq!(lexer.next_token().kind, TokenKind::Pipe);

    let mut lexer = Lexer::new("||");
    assert_eq!(lexer.next_token().kind, TokenKind::OrOr);

    let mut lexer = Lexer::new("|");
    assert_eq!(lexer.next_token().kind, TokenKind::Bar);
}

#[test]
fn edge_arrow_disambiguation() {
    // -> vs - vs --
    let mut lexer = Lexer::new("->");
    assert_eq!(lexer.next_token().kind, TokenKind::Arrow);

    let mut lexer = Lexer::new("-");
    assert_eq!(lexer.next_token().kind, TokenKind::Minus);
}

// ============================================================================
// COMPLETE TOKEN STREAM
// ============================================================================

#[test]
fn complete_gene_definition() {
    let input = r#"gene Container {
        has id: UInt64
        has name: String
    }"#;

    let mut lexer = Lexer::new(input);
    let tokens: Vec<_> = std::iter::from_fn(|| {
        let t = lexer.next_token();
        if t.kind == TokenKind::Eof {
            None
        } else {
            Some(t.kind)
        }
    })
    .collect();

    assert!(tokens.len() > 10, "Should produce multiple tokens");
    assert_eq!(tokens[0], TokenKind::Gene);
}

#[test]
fn complete_function_definition() {
    let input = "fun add(a: Int64, b: Int64) -> Int64 { return a + b }";

    let mut lexer = Lexer::new(input);
    let tokens: Vec<_> = std::iter::from_fn(|| {
        let t = lexer.next_token();
        if t.kind == TokenKind::Eof {
            None
        } else {
            Some(t.kind)
        }
    })
    .collect();

    assert!(tokens.len() > 15);
    assert_eq!(tokens[0], TokenKind::Fun);
}
