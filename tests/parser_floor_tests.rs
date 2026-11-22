//! Regression tests for the parser/lexer "floor" fixes (audit bugs B1-B6)
//! and the Lean-alignment Phase L0 language steps (L0.1-L0.4).
//!
//! Each section holds a positive oracle (the construct parses to the right
//! shape) and a negative oracle (the forbidden behaviour never happens).

use metadol::ast::{Declaration, Expr, Literal};
use metadol::lexer::{Lexer, TokenKind};
use metadol::parser::Parser;

// ============================================
// B1. Numeric literals
// ============================================

/// Numeric literals with their expected parsed value.
fn numeric_cases() -> Vec<(&'static str, Literal)> {
    vec![
        ("42", Literal::Int(42)),
        ("1_000", Literal::Int(1000)),
        ("1_000_000", Literal::Int(1_000_000)),
        ("0x10", Literal::Int(16)),
        ("0xFF_FF", Literal::Int(0xFFFF)),
        ("0b1010", Literal::Int(10)),
        ("0o17", Literal::Int(15)),
        ("3.14", Literal::Float(3.14)),
        ("1e9", Literal::Float(1e9)),
        ("1E9", Literal::Float(1e9)),
        ("3.0e8", Literal::Float(3.0e8)),
        ("6.022e23", Literal::Float(6.022e23)),
        ("1.5e-3", Literal::Float(1.5e-3)),
        ("2e+10", Literal::Float(2e10)),
        ("1_000.5", Literal::Float(1000.5)),
    ]
}

#[test]
fn b1_numeric_literals_lex_as_single_token() {
    // Negative oracle: no numeric literal is ever split into number + identifier.
    for (src, _) in numeric_cases() {
        let tokens: Vec<_> = Lexer::new(src).collect();
        assert_eq!(
            tokens.len(),
            1,
            "`{}` lexed into {} tokens: {:?}",
            src,
            tokens.len(),
            tokens
                .iter()
                .map(|t| (t.kind, t.lexeme.clone()))
                .collect::<Vec<_>>()
        );
        assert_eq!(tokens[0].lexeme, src);
    }
}

#[test]
fn b1_numeric_literals_parse_to_value() {
    for (src, expected) in numeric_cases() {
        let mut parser = Parser::new(src);
        let expr = parser
            .parse_expr(0)
            .unwrap_or_else(|e| panic!("`{}` failed: {}", src, e));
        assert_eq!(expr, Expr::Literal(expected), "value of `{}`", src);
    }
}

#[test]
fn b1_numeric_literal_in_binary_expression() {
    let mut parser = Parser::new("x * 1e9 + 0x10");
    let expr = parser.parse_expr(0).unwrap();
    let dbg = format!("{:?}", expr);
    assert!(dbg.contains("Float(1000000000.0)"), "{}", dbg);
    assert!(dbg.contains("Int(16)"), "{}", dbg);
}

#[test]
fn b1_identifier_and_version_lexing_unchanged() {
    // `e` after a number is only an exponent when digits follow.
    let kinds: Vec<_> = Lexer::new("1 else").map(|t| t.kind).collect();
    assert_eq!(kinds, vec![TokenKind::Identifier, TokenKind::Else]);
    // Versions still lex as versions.
    let toks: Vec<_> = Lexer::new("1.2.3").collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind, TokenKind::Version);
    // Range syntax `0..10` must not be eaten by float lexing.
    let lexemes: Vec<_> = Lexer::new("0..10").map(|t| t.lexeme).collect();
    assert_eq!(lexemes, vec!["0", "..", "10"]);
}

// ============================================
// B2. Balanced braces inside docs / exegesis
// ============================================

fn gene_exegesis(decl: &Declaration) -> String {
    match decl {
        Declaration::Gene(g) => g.exegesis.clone(),
        other => panic!("expected gene, got {:?}", other),
    }
}

#[test]
fn b2_braces_inside_trailing_docs_block() {
    let src = r#"
gen a.thing {
  thing has id
}

docs {
  Example: `gen X { has y: i64 }` and a map {k: v}.
}

gen b.thing {
  thing has id
}
"#;
    let decls = metadol::parse_file_all(src).expect("parse");
    // Negative oracle: the docs block must not end early and leak `}` text
    // into the token stream, and the following declaration must survive.
    assert_eq!(decls.len(), 2, "{:?}", decls);
    let ex = gene_exegesis(&decls[0]);
    assert!(ex.contains("gen X { has y: i64 }"), "exegesis: {:?}", ex);
    assert!(ex.ends_with("a map {k: v}."), "exegesis: {:?}", ex);
    assert_eq!(decls[1].name(), "b.thing");
}

#[test]
fn b2_braces_inside_inline_docs_block() {
    let src = r#"
trait a.thing {
  thing is ready
  docs {
    Nested { braces { two deep } } and "quotes" and it's fine.
  }
}

gen b.thing {
  has id: i64
}
"#;
    let decls = metadol::parse_file_all(src).expect("parse");
    assert_eq!(decls.len(), 2, "{:?}", decls);
    let ex = match &decls[0] {
        Declaration::Trait(t) => t.exegesis.clone(),
        other => panic!("expected trait, got {:?}", other),
    };
    assert_eq!(
        ex,
        "Nested { braces { two deep } } and \"quotes\" and it's fine."
    );
}

#[test]
fn b2_unterminated_docs_block_is_an_error() {
    let src = "gen a.thing {\n  has id: i64\n}\ndocs {\n  never { closed\n";
    assert!(metadol::parse_file_all(src).is_err());
}
