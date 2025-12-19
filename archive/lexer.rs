//! DOL Lexer
//!
//! Tokenizes DOL source text using the logos crate.
//! Designed for LLVM-style diagnostic integration.

use logos::{Logos, Lexer};
use std::fmt;

/// Callback for parsing version numbers
fn parse_version(lex: &mut Lexer<Token>) -> Option<(u32, u32, u32)> {
    let slice = lex.slice();
    // Remove @ prefix if present
    let version_str = slice.trim_start_matches('@').trim();
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() == 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}

/// Callback for parsing quoted strings
fn parse_string(lex: &mut Lexer<Token>) -> String {
    let slice = lex.slice();
    // Remove surrounding quotes
    slice[1..slice.len()-1].to_string()
}

/// DOL tokens
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    // ═══════════════════════════════════════════
    // Keywords - Declaration Types
    // ═══════════════════════════════════════════
    
    #[token("gene")]
    Gene,
    
    #[token("trait")]
    Trait,
    
    #[token("constraint")]
    Constraint,
    
    #[token("system")]
    System,
    
    #[token("evolves")]
    Evolves,
    
    #[token("test")]
    Test,
    
    #[token("exegesis")]
    Exegesis,
    
    // ═══════════════════════════════════════════
    // Keywords - Composition
    // ═══════════════════════════════════════════
    
    #[token("uses")]
    Uses,
    
    #[token("requires")]
    Requires,
    
    // ═══════════════════════════════════════════
    // Keywords - Predicates
    // ═══════════════════════════════════════════
    
    #[token("has")]
    Has,
    
    #[token("is")]
    Is,
    
    #[token("are")]
    Are,
    
    #[token("derives")]
    Derives,
    
    #[token("from")]
    From,
    
    #[token("emits")]
    Emits,
    
    #[token("matches")]
    Matches,
    
    #[token("never")]
    Never,
    
    #[token("via")]
    Via,
    
    #[token("no")]
    No,
    
    // ═══════════════════════════════════════════
    // Keywords - Quantifiers
    // ═══════════════════════════════════════════
    
    #[token("each")]
    Each,
    
    #[token("all")]
    All,
    
    #[token("every")]
    Every,
    
    // ═══════════════════════════════════════════
    // Keywords - Evolution
    // ═══════════════════════════════════════════
    
    #[token("adds")]
    Adds,
    
    #[token("deprecates")]
    Deprecates,
    
    #[token("removes")]
    Removes,
    
    #[token("because")]
    Because,
    
    #[token("migration")]
    Migration,
    
    #[token("maps")]
    Maps,
    
    #[token("to")]
    To,
    
    #[token("then")]
    Then,
    
    // ═══════════════════════════════════════════
    // Keywords - Test
    // ═══════════════════════════════════════════
    
    #[token("given")]
    Given,
    
    #[token("when")]
    When,
    
    #[token("always")]
    Always,
    
    // ═══════════════════════════════════════════
    // Operators and Punctuation
    // ═══════════════════════════════════════════
    
    #[token("{")]
    LBrace,
    
    #[token("}")]
    RBrace,
    
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token(".")]
    Dot,
    
    #[token(">")]
    Gt,
    
    #[token(">=")]
    Gte,
    
    #[token("<")]
    Lt,
    
    #[token("<=")]
    Lte,
    
    #[token("=")]
    Eq,
    
    #[token("@")]
    At,
    
    // ═══════════════════════════════════════════
    // Literals
    // ═══════════════════════════════════════════
    
    /// Version number like 0.0.1
    #[regex(r"[0-9]+\.[0-9]+\.[0-9]+", |lex| parse_version(lex))]
    Version((u32, u32, u32)),
    
    /// Integer
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<u64>().ok())]
    Integer(u64),
    
    /// Quoted string
    #[regex(r#""[^"]*""#, parse_string)]
    String(String),
    
    /// Identifier (lowercase with underscores)
    #[regex(r"[a-z_][a-z0-9_]*")]
    Identifier,
    
    // ═══════════════════════════════════════════
    // Comments
    // ═══════════════════════════════════════════
    
    /// Line comment
    #[regex(r"//[^\n]*", logos::skip)]
    LineComment,
    
    /// Decorative line (═══)
    #[regex(r"═+", logos::skip)]
    DecorativeLine,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Gene => write!(f, "gene"),
            Token::Trait => write!(f, "trait"),
            Token::Constraint => write!(f, "constraint"),
            Token::System => write!(f, "system"),
            Token::Evolves => write!(f, "evolves"),
            Token::Test => write!(f, "test"),
            Token::Exegesis => write!(f, "exegesis"),
            Token::Uses => write!(f, "uses"),
            Token::Requires => write!(f, "requires"),
            Token::Has => write!(f, "has"),
            Token::Is => write!(f, "is"),
            Token::Are => write!(f, "are"),
            Token::Derives => write!(f, "derives"),
            Token::From => write!(f, "from"),
            Token::Emits => write!(f, "emits"),
            Token::Matches => write!(f, "matches"),
            Token::Never => write!(f, "never"),
            Token::Via => write!(f, "via"),
            Token::No => write!(f, "no"),
            Token::Each => write!(f, "each"),
            Token::All => write!(f, "all"),
            Token::Every => write!(f, "every"),
            Token::Adds => write!(f, "adds"),
            Token::Deprecates => write!(f, "deprecates"),
            Token::Removes => write!(f, "removes"),
            Token::Because => write!(f, "because"),
            Token::Migration => write!(f, "migration"),
            Token::Maps => write!(f, "maps"),
            Token::To => write!(f, "to"),
            Token::Then => write!(f, "then"),
            Token::Given => write!(f, "given"),
            Token::When => write!(f, "when"),
            Token::Always => write!(f, "always"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Dot => write!(f, "."),
            Token::Gt => write!(f, ">"),
            Token::Gte => write!(f, ">="),
            Token::Lt => write!(f, "<"),
            Token::Lte => write!(f, "<="),
            Token::Eq => write!(f, "="),
            Token::At => write!(f, "@"),
            Token::Version((major, minor, patch)) => write!(f, "{major}.{minor}.{patch}"),
            Token::Integer(n) => write!(f, "{n}"),
            Token::String(s) => write!(f, "\"{s}\""),
            Token::Identifier => write!(f, "<identifier>"),
            Token::LineComment => write!(f, "<comment>"),
            Token::DecorativeLine => write!(f, "<decorative>"),
        }
    }
}

/// A token with span information
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: std::ops::Range<usize>,
    pub slice: String,
}

/// Tokenize DOL source
pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    
    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => {
                tokens.push(SpannedToken {
                    token,
                    span: lexer.span(),
                    slice: lexer.slice().to_string(),
                });
            }
            Err(_) => {
                return Err(LexError {
                    span: lexer.span(),
                    message: format!("Unexpected character: {:?}", lexer.slice()),
                });
            }
        }
    }
    
    Ok(tokens)
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub span: std::ops::Range<usize>,
    pub message: String,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexical error at {:?}: {}", self.span, self.message)
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_gene_declaration() {
        let source = r#"
            gene container.exists {
                container has identity
            }
        "#;
        
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| t.token == Token::Gene));
        assert!(tokens.iter().any(|t| t.token == Token::Has));
    }

    #[test]
    fn lex_version() {
        let source = "@ 0.0.1";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| matches!(t.token, Token::Version((0, 0, 1)))));
    }

    #[test]
    fn lex_evolves_lineage() {
        let source = "evolves container.lifecycle @ 0.0.2 > 0.0.1";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| t.token == Token::Evolves));
        assert!(tokens.iter().any(|t| t.token == Token::Gt));
    }

    #[test]
    fn lex_quoted_string() {
        let source = r#"because "migration requires state""#;
        let tokens = tokenize(source).unwrap();
        let string_token = tokens.iter().find(|t| matches!(&t.token, Token::String(_)));
        assert!(string_token.is_some());
    }

    #[test]
    fn skip_comments() {
        let source = r#"
            // This is a comment
            gene container.exists {
                // Another comment
                container has identity
            }
        "#;
        
        let tokens = tokenize(source).unwrap();
        // Comments should be skipped
        assert!(!tokens.iter().any(|t| t.token == Token::LineComment));
    }
}
