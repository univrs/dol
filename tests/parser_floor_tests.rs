//! Regression tests for the parser/lexer "floor" fixes (audit bugs B1-B6)
//! and the Lean-alignment Phase L0 language steps (L0.1-L0.4).
//!
//! Each section holds a positive oracle (the construct parses to the right
//! shape) and a negative oracle (the forbidden behaviour never happens).

use metadol::ast::{BinaryOp, Declaration, Expr, Literal, TypeExpr};
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

// ============================================
// B5. docs inside a gen body with fields
// ============================================

fn assert_no_skip_markers(decl: &Declaration) {
    if let Declaration::Gene(g) = decl {
        for s in &g.statements {
            let dbg = format!("{:?}", s);
            assert!(
                !dbg.contains("_skip"),
                "placeholder statement leaked: {}",
                dbg
            );
        }
    }
}

#[test]
fn b5_docs_at_end_of_gen_body_with_fields_is_captured() {
    let src = r#"
gen counter.state {
  has value: i64
  has label: string
  docs {
    A counter with a label.
  }
}
"#;
    let decls = metadol::parse_file_all(src).expect("parse");
    assert_eq!(decls.len(), 1);
    assert_eq!(gene_exegesis(&decls[0]), "A counter with a label.");
    assert_no_skip_markers(&decls[0]);
    if let Declaration::Gene(g) = &decls[0] {
        assert_eq!(g.statements.len(), 2, "{:?}", g.statements);
    }
}

#[test]
fn b5_docs_in_middle_of_gen_body_is_captured_and_fields_kept() {
    let src = r#"
gen counter.state {
  has value: i64
  docs {
    Middle docs.
  }
  has label: string
}
"#;
    let decls = metadol::parse_file_all(src).expect("parse");
    assert_eq!(gene_exegesis(&decls[0]), "Middle docs.");
    assert_no_skip_markers(&decls[0]);
    if let Declaration::Gene(g) = &decls[0] {
        assert_eq!(g.statements.len(), 2, "{:?}", g.statements);
    }
}

#[test]
fn b5_docs_inside_rule_body_is_captured() {
    let src = r#"
rule counter.bounded {
  value never negative
  docs {
    Counters never go below zero.
  }
}
"#;
    let decls = metadol::parse_file_all(src).expect("parse");
    match &decls[0] {
        Declaration::Constraint(c) => assert_eq!(c.exegesis, "Counters never go below zero."),
        other => panic!("expected constraint, got {:?}", other),
    }
}

// ============================================
// B3. `use` must not swallow the following declaration
// ============================================

#[test]
fn b3_use_before_evo_keeps_evo() {
    let src = r#"
use container.lifecycle

evo container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
}
"#;
    let decls = metadol::parse_file_all(src).expect("parse");
    assert_eq!(decls.len(), 1, "{:?}", decls);
    assert!(
        matches!(decls[0], Declaration::Evolution(_)),
        "{:?}",
        decls[0]
    );

    let file = metadol::parse_dol_file(src).expect("parse");
    assert_eq!(file.uses.len(), 1);
    assert_eq!(file.declarations.len(), 1);
}

#[test]
fn b3_mid_file_use_before_const_keeps_const() {
    let src = r#"
gen a.thing {
  has id: i64
}

use b.other

const LIMIT: i64 = 10
"#;
    let decls = metadol::parse_file_all(src).expect("parse");
    // Negative oracle: the declaration after a mid-file `use` is never dropped.
    assert_eq!(decls.len(), 2, "{:?}", decls);
    assert_eq!(decls[1].name(), "LIMIT");
}

#[test]
fn b3_mid_file_use_is_recorded_in_dol_file() {
    let src = r#"
use a.first

gen a.thing {
  has id: i64
}

pub use b.other.{ X, Y }

evo a.thing @ 0.0.2 > 0.0.1 {
  adds thing is ready
}
"#;
    let file = metadol::parse_dol_file(src).expect("parse");
    assert_eq!(file.uses.len(), 2, "{:?}", file.uses);
    assert_eq!(file.declarations.len(), 2, "{:?}", file.declarations);
}

#[test]
fn b3_use_path_may_name_keyword_modules() {
    // `evo`/`gen`/`docs` are valid module names in a use path, like their
    // deprecated spellings `evolves`/`gene`/`exegesis`.
    let src = r#"
module biology @ 1.0.0

pub use types.*
pub use evo.*
use gen.core.{ A }

gen a.thing {
  has id: i64
}
"#;
    let file = metadol::parse_dol_file(src).expect("parse");
    assert_eq!(file.uses.len(), 3);
    assert_eq!(metadol::parse_file_all(src).expect("parse").len(), 1);
}

#[test]
fn b3_junk_after_use_path_is_an_error() {
    // Negative oracle: tokens after a `use` path are not silently skipped.
    let src = "use a.b c d e\n\ngen a.thing {\n  has id: i64\n}\n";
    assert!(metadol::parse_file_all(src).is_err());
    assert!(metadol::parse_dol_file(src).is_err());
}

// ============================================
// B4. `mod x { }` blocks
// ============================================

fn assert_no_placeholders(decls: &[Declaration]) {
    for d in decls {
        assert!(
            !d.name().starts_with('_'),
            "placeholder declaration leaked: {}",
            d.name()
        );
    }
}

#[test]
fn b4_mod_block_as_first_item() {
    let src = r#"
mod helpers {
  gen inner.thing {
    has id: i64
  }
}

gen a.thing {
  has id: i64
}
"#;
    let decls = metadol::parse_file_all(src).expect("parse_file_all");
    assert_eq!(decls.len(), 1, "{:?}", decls);
    assert_eq!(decls[0].name(), "a.thing");

    let file = metadol::parse_dol_file(src).expect("parse_dol_file");
    assert!(file.module.is_none(), "a mod block is not a file header");
    assert_eq!(file.declarations.len(), 1);

    let first = metadol::parse_file(src).expect("parse_file");
    assert_eq!(first.name(), "a.thing");
}

#[test]
fn b4_pub_mod_block_with_uses_and_trailing_mod() {
    let src = r#"
module biology @ 1.0.0

pub mod prelude {
    pub use dol.{ lex, parse }
}

gen a.thing {
  has id: i64
}

mod empty { }
"#;
    let decls = metadol::parse_file_all(src).expect("parse_file_all");
    // Negative oracle: skipped mod blocks never surface as fake declarations.
    assert_no_placeholders(&decls);
    assert_eq!(decls.len(), 1, "{:?}", decls);

    let file = metadol::parse_dol_file(src).expect("parse_dol_file");
    assert!(file.module.is_some());
    assert_no_placeholders(&file.declarations);
    assert_eq!(file.declarations.len(), 1);
}

#[test]
fn b4_unterminated_mod_block_is_an_error() {
    let src = "mod helpers {\n  gen a.thing {\n    has id: i64\n  }\n";
    assert!(metadol::parse_file_all(src).is_err());
    assert!(metadol::parse_dol_file(src).is_err());
}

// ============================================
// B6. Whole-file parsing
// ============================================

#[test]
fn b6_module_docs_only_file_has_no_fake_declarations() {
    let src = r#"
module biology @ 1.0.0

pub use types.*

docs {
  Module-level documentation.
}
"#;
    // Negative oracle: a file-level docs block never becomes a declaration.
    assert_eq!(metadol::parse_file_all(src).expect("parse").len(), 0);
    let file = metadol::parse_dol_file(src).expect("parse");
    assert_eq!(file.declarations.len(), 0, "{:?}", file.declarations);
    assert_eq!(file.uses.len(), 1);
}

#[test]
fn b6_whole_file_rejects_trailing_garbage() {
    let src = "gen a.thing {\n  has id: i64\n}\n\nthis is not dol }}}\n";
    assert!(metadol::parse_file_all(src).is_err());
    assert!(metadol::parse_dol_file(src).is_err());
}

// ============================================
// L0.1 `implies` as a binary operator
// ============================================

fn expr(src: &str) -> Expr {
    Parser::new(src)
        .parse_expr(0)
        .unwrap_or_else(|e| panic!("`{}` failed: {}", src, e))
}

fn id(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn bin(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

#[test]
fn l0_1_implies_parses_as_binary_operator() {
    assert_eq!(
        expr("a implies b"),
        bin(id("a"), BinaryOp::Implies, id("b"))
    );
}

#[test]
fn l0_1_implies_binds_looser_than_or() {
    // Negative oracle: `||` never captures an `implies` operand.
    assert_eq!(
        expr("a || b implies c"),
        bin(
            bin(id("a"), BinaryOp::Or, id("b")),
            BinaryOp::Implies,
            id("c")
        )
    );
    assert_eq!(
        expr("a implies b || c"),
        bin(
            id("a"),
            BinaryOp::Implies,
            bin(id("b"), BinaryOp::Or, id("c"))
        )
    );
}

#[test]
fn l0_1_implies_is_right_associative() {
    assert_eq!(
        expr("a implies b implies c"),
        bin(
            id("a"),
            BinaryOp::Implies,
            bin(id("b"), BinaryOp::Implies, id("c"))
        )
    );
}

#[test]
fn l0_1_implies_inside_forall_body() {
    match expr("forall x: Int. p(x) implies q(x)") {
        Expr::Forall(f) => assert!(
            matches!(
                *f.body,
                Expr::Binary {
                    op: BinaryOp::Implies,
                    ..
                }
            ),
            "{:?}",
            f.body
        ),
        other => panic!("expected forall, got {:?}", other),
    }
}

// ============================================
// L0.2 `exists x: T. p` in expression position
// ============================================

#[test]
fn l0_2_exists_parses_in_expression_position() {
    match expr("exists x: Int. x > 0") {
        Expr::Exists(e) => {
            assert_eq!(e.var, "x");
            assert_eq!(e.type_, TypeExpr::Named("Int".to_string()));
            assert_eq!(
                *e.body,
                bin(id("x"), BinaryOp::Gt, Expr::Literal(Literal::Int(0)))
            );
        }
        other => panic!("expected exists, got {:?}", other),
    }
}

#[test]
fn l0_2_exists_nests_with_forall_and_implies() {
    match expr("forall x: Int. exists y: Int. y > x implies p(y)") {
        Expr::Forall(f) => match *f.body {
            Expr::Exists(e) => assert!(
                matches!(
                    *e.body,
                    Expr::Binary {
                        op: BinaryOp::Implies,
                        ..
                    }
                ),
                "{:?}",
                e.body
            ),
            other => panic!("expected exists body, got {:?}", other),
        },
        other => panic!("expected forall, got {:?}", other),
    }
}

#[test]
fn l0_2_exists_inside_function_body() {
    let src = r#"
fun has_positive(xs: List<Int>) -> Bool {
  return exists x: Int. x > 0
}
"#;
    assert_eq!(metadol::parse_file_all(src).expect("parse").len(), 1);
}

#[test]
fn l0_2_exists_requires_typed_binder() {
    // Negative oracle: a malformed quantifier is an error, not an identifier.
    assert!(Parser::new("exists x. p(x)").parse_expr(0).is_err());
    assert!(Parser::new("exists x: Int p(x)").parse_expr(0).is_err());
}

#[cfg(feature = "serde")]
#[test]
fn l0_2_exists_round_trips_through_serde() {
    let e = expr("exists x: Int. x > 0 implies q(x)");
    let json = serde_json::to_string(&e).unwrap();
    let back: Expr = serde_json::from_str(&json).unwrap();
    assert_eq!(back, e);
}
