//! Comprehensive lexer tests for Metal DOL.
//!
//! These tests verify correct tokenization of all DOL language constructs.

use metadol::lexer::{Lexer, TokenKind};

/// Helper to collect all tokens from input
fn tokenize(input: &str) -> Vec<(TokenKind, String)> {
    Lexer::new(input)
        .map(|t| (t.kind, t.lexeme))
        .collect()
}

/// Helper to get just token kinds
fn token_kinds(input: &str) -> Vec<TokenKind> {
    Lexer::new(input)
        .map(|t| t.kind)
        .collect()
}

// ============================================
// 1. Keyword Tests
// ============================================

#[test]
fn test_declaration_keywords() {
    let tokens = tokenize("gene trait constraint system evolves exegesis");
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens[0].0, TokenKind::Gene);
    assert_eq!(tokens[1].0, TokenKind::Trait);
    assert_eq!(tokens[2].0, TokenKind::Constraint);
    assert_eq!(tokens[3].0, TokenKind::System);
    assert_eq!(tokens[4].0, TokenKind::Evolves);
    assert_eq!(tokens[5].0, TokenKind::Exegesis);
}

#[test]
fn test_predicate_keywords() {
    let tokens = tokenize("has is derives from requires uses emits matches never");
    assert_eq!(tokens.len(), 9);
    assert_eq!(tokens[0].0, TokenKind::Has);
    assert_eq!(tokens[1].0, TokenKind::Is);
    assert_eq!(tokens[2].0, TokenKind::Derives);
    assert_eq!(tokens[3].0, TokenKind::From);
    assert_eq!(tokens[4].0, TokenKind::Requires);
    assert_eq!(tokens[5].0, TokenKind::Uses);
    assert_eq!(tokens[6].0, TokenKind::Emits);
    assert_eq!(tokens[7].0, TokenKind::Matches);
    assert_eq!(tokens[8].0, TokenKind::Never);
}

#[test]
fn test_evolution_keywords() {
    let tokens = tokenize("adds deprecates removes because");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].0, TokenKind::Adds);
    assert_eq!(tokens[1].0, TokenKind::Deprecates);
    assert_eq!(tokens[2].0, TokenKind::Removes);
    assert_eq!(tokens[3].0, TokenKind::Because);
}

#[test]
fn test_test_keywords() {
    let tokens = tokenize("test given when then always");
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].0, TokenKind::Test);
    assert_eq!(tokens[1].0, TokenKind::Given);
    assert_eq!(tokens[2].0, TokenKind::When);
    assert_eq!(tokens[3].0, TokenKind::Then);
    assert_eq!(tokens[4].0, TokenKind::Always);
}

#[test]
fn test_quantifier_keywords() {
    let tokens = tokenize("each all no");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].0, TokenKind::Each);
    assert_eq!(tokens[1].0, TokenKind::All);
    assert_eq!(tokens[2].0, TokenKind::No);
}

// ============================================
// 2. Identifier Tests
// ============================================

#[test]
fn test_simple_identifier() {
    let tokens = tokenize("container");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Identifier);
    assert_eq!(tokens[0].1, "container");
}

#[test]
fn test_qualified_identifier() {
    let tokens = tokenize("container.exists");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Identifier);
    assert_eq!(tokens[0].1, "container.exists");
}

#[test]
fn test_deeply_qualified_identifier() {
    let tokens = tokenize("domain.sub.component.property");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Identifier);
    assert_eq!(tokens[0].1, "domain.sub.component.property");
}

#[test]
fn test_identifier_with_underscore() {
    let tokens = tokenize("container_exists my_property");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].0, TokenKind::Identifier);
    assert_eq!(tokens[0].1, "container_exists");
    assert_eq!(tokens[1].0, TokenKind::Identifier);
    assert_eq!(tokens[1].1, "my_property");
}

#[test]
fn test_identifier_with_digits() {
    let tokens = tokenize("container1 property2v3");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].0, TokenKind::Identifier);
    assert_eq!(tokens[1].0, TokenKind::Identifier);
}

#[test]
fn test_identifier_not_keyword() {
    // "genes" is not "gene" keyword
    let tokens = tokenize("genes traits constraints");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].0, TokenKind::Identifier);
    assert_eq!(tokens[1].0, TokenKind::Identifier);
    assert_eq!(tokens[2].0, TokenKind::Identifier);
}

// ============================================
// 3. Version Tests
// ============================================

#[test]
fn test_simple_version() {
    let tokens = tokenize("0.0.1");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Version);
    assert_eq!(tokens[0].1, "0.0.1");
}

#[test]
fn test_multidigit_version() {
    let tokens = tokenize("10.20.30");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Version);
    assert_eq!(tokens[0].1, "10.20.30");
}

#[test]
fn test_version_in_context() {
    let tokens = tokenize("@ 1.2.3");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].0, TokenKind::At);
    assert_eq!(tokens[1].0, TokenKind::Version);
}

#[test]
fn test_version_comparison() {
    let tokens = tokenize(">= 0.0.1");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].0, TokenKind::GreaterEqual);
    assert_eq!(tokens[1].0, TokenKind::Version);
}

// ============================================
// 4. Operator Tests
// ============================================

#[test]
fn test_at_operator() {
    let tokens = tokenize("@");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::At);
}

#[test]
fn test_greater_operator() {
    let tokens = tokenize(">");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Greater);
}

#[test]
fn test_greater_equal_operator() {
    let tokens = tokenize(">=");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::GreaterEqual);
}

#[test]
fn test_equal_operator() {
    let tokens = tokenize("=");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Equal);
}

#[test]
fn test_braces() {
    let tokens = tokenize("{ }");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].0, TokenKind::LeftBrace);
    assert_eq!(tokens[1].0, TokenKind::RightBrace);
}

// ============================================
// 5. String Tests
// ============================================

#[test]
fn test_simple_string() {
    let tokens = tokenize(r#""hello world""#);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::String);
    assert_eq!(tokens[0].1, "hello world");
}

#[test]
fn test_string_with_escape_quote() {
    let tokens = tokenize(r#""say \"hello\"""#);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::String);
    assert!(tokens[0].1.contains("\"hello\""));
}

#[test]
fn test_string_with_escape_newline() {
    let tokens = tokenize(r#""line1\nline2""#);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::String);
    assert!(tokens[0].1.contains('\n'));
}

#[test]
fn test_empty_string() {
    let tokens = tokenize(r#""""#);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::String);
    assert_eq!(tokens[0].1, "");
}

// ============================================
// 6. Whitespace and Comment Tests
// ============================================

#[test]
fn test_whitespace_handling() {
    let tokens = tokenize("  gene   trait  ");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].0, TokenKind::Gene);
    assert_eq!(tokens[1].0, TokenKind::Trait);
}

#[test]
fn test_newline_handling() {
    let tokens = tokenize("gene\ntrait\nconstraint");
    assert_eq!(tokens.len(), 3);
}

#[test]
fn test_single_line_comment() {
    let tokens = tokenize("gene // this is a comment\ntrait");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].0, TokenKind::Gene);
    assert_eq!(tokens[1].0, TokenKind::Trait);
}

#[test]
fn test_comment_at_end() {
    let tokens = tokenize("gene // comment at end");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].0, TokenKind::Gene);
}

// ============================================
// 7. Span Tracking Tests
// ============================================

#[test]
fn test_span_start_position() {
    let mut lexer = Lexer::new("gene");
    let token = lexer.next_token();
    assert_eq!(token.span.start, 0);
    assert_eq!(token.span.line, 1);
    assert_eq!(token.span.column, 1);
}

#[test]
fn test_span_after_whitespace() {
    let mut lexer = Lexer::new("  gene");
    let token = lexer.next_token();
    assert_eq!(token.span.column, 3);
}

#[test]
fn test_span_on_second_line() {
    let mut lexer = Lexer::new("\ngene");
    let token = lexer.next_token();
    assert_eq!(token.span.line, 2);
    assert_eq!(token.span.column, 1);
}

#[test]
fn test_span_length() {
    let mut lexer = Lexer::new("container.exists");
    let token = lexer.next_token();
    assert_eq!(token.span.end - token.span.start, 16);
}

// ============================================
// 8. Complete Expression Tests
// ============================================

#[test]
fn test_gene_declaration_tokens() {
    let kinds = token_kinds("gene container.exists { }");
    assert_eq!(kinds, vec![
        TokenKind::Gene,
        TokenKind::Identifier,
        TokenKind::LeftBrace,
        TokenKind::RightBrace,
    ]);
}

#[test]
fn test_has_statement_tokens() {
    let kinds = token_kinds("container has identity");
    assert_eq!(kinds, vec![
        TokenKind::Identifier,
        TokenKind::Has,
        TokenKind::Identifier,
    ]);
}

#[test]
fn test_system_version_tokens() {
    let kinds = token_kinds("system univrs.orchestrator @ 0.1.0");
    assert_eq!(kinds, vec![
        TokenKind::System,
        TokenKind::Identifier,
        TokenKind::At,
        TokenKind::Version,
    ]);
}

#[test]
fn test_requires_version_tokens() {
    let kinds = token_kinds("requires container.lifecycle >= 0.0.2");
    assert_eq!(kinds, vec![
        TokenKind::Requires,
        TokenKind::Identifier,
        TokenKind::GreaterEqual,
        TokenKind::Version,
    ]);
}

#[test]
fn test_evolution_tokens() {
    let kinds = token_kinds("evolves container.lifecycle @ 0.0.2 > 0.0.1");
    assert_eq!(kinds, vec![
        TokenKind::Evolves,
        TokenKind::Identifier,
        TokenKind::At,
        TokenKind::Version,
        TokenKind::Greater,
        TokenKind::Version,
    ]);
}

#[test]
fn test_because_with_string() {
    let kinds = token_kinds(r#"because "migration support""#);
    assert_eq!(kinds, vec![
        TokenKind::Because,
        TokenKind::String,
    ]);
}

// ============================================
// 9. Error Cases
// ============================================

#[test]
fn test_error_on_unexpected_char() {
    let mut lexer = Lexer::new("$invalid");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Error);
    assert!(!lexer.errors().is_empty());
}

#[test]
fn test_errors_collected() {
    let mut lexer = Lexer::new("$ % ^");
    while lexer.next_token().kind != TokenKind::Eof {}
    assert!(lexer.errors().len() >= 3);
}

// ============================================
// 10. Iterator Tests
// ============================================

#[test]
fn test_lexer_iterator() {
    let tokens: Vec<_> = Lexer::new("gene trait").collect();
    assert_eq!(tokens.len(), 2);
}

#[test]
fn test_lexer_eof() {
    let mut lexer = Lexer::new("");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Eof);
}

#[test]
fn test_multiple_eof_calls() {
    let mut lexer = Lexer::new("gene");
    lexer.next_token(); // gene
    let eof1 = lexer.next_token();
    let eof2 = lexer.next_token();
    assert_eq!(eof1.kind, TokenKind::Eof);
    assert_eq!(eof2.kind, TokenKind::Eof);
}

// ============================================
// 11. TokenKind Methods Tests
// ============================================

#[test]
fn test_is_keyword() {
    assert!(TokenKind::Gene.is_keyword());
    assert!(TokenKind::Has.is_keyword());
    assert!(!TokenKind::Identifier.is_keyword());
    assert!(!TokenKind::LeftBrace.is_keyword());
}

#[test]
fn test_is_predicate() {
    assert!(TokenKind::Has.is_predicate());
    assert!(TokenKind::Is.is_predicate());
    assert!(TokenKind::Matches.is_predicate());
    assert!(!TokenKind::Gene.is_predicate());
    assert!(!TokenKind::Identifier.is_predicate());
}

#[test]
fn test_token_kind_display() {
    assert_eq!(format!("{}", TokenKind::Gene), "gene");
    assert_eq!(format!("{}", TokenKind::Identifier), "identifier");
    assert_eq!(format!("{}", TokenKind::LeftBrace), "{");
}
