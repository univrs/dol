//! Lexical analysis for Metal DOL.
//!
//! This module provides tokenization of DOL source text into a stream of tokens
//! that can be consumed by the parser. The lexer handles keywords, identifiers,
//! operators, version numbers, and string literals.
//!
//! # Example
//!
//! ```rust
//! use metadol::lexer::{Lexer, TokenKind};
//!
//! let mut lexer = Lexer::new("gene container.exists { }");
//!
//! assert_eq!(lexer.next_token().kind, TokenKind::Gene);
//! assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
//! assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
//! assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
//! ```
//!
//! # Token Types
//!
//! The lexer recognizes:
//! - **Keywords**: `gene`, `trait`, `constraint`, `system`, `evolves`, etc.
//! - **Predicates**: `has`, `is`, `derives`, `from`, `requires`, etc.
//! - **Operators**: `@`, `>`, `>=`
//! - **Delimiters**: `{`, `}`
//! - **Identifiers**: Simple and qualified (dot-notation)
//! - **Versions**: Semantic version numbers (X.Y.Z)
//! - **Strings**: Double-quoted string literals

use crate::ast::Span;
use crate::error::LexError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A lexical token produced by the lexer.
///
/// Tokens carry their kind, the original source text (lexeme), and
/// source location information for error reporting.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Token {
    /// The category of this token
    pub kind: TokenKind,

    /// The original source text that produced this token
    pub lexeme: String,

    /// Source location for error reporting
    pub span: Span,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            span,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            span: Span::default(),
        }
    }
}

/// The category of a lexical token.
///
/// TokenKind distinguishes between keywords, operators, literals,
/// and other syntactic elements of the DOL language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TokenKind {
    // === Declaration Keywords ===
    /// The `gene` keyword
    Gene,
    /// The `trait` keyword
    Trait,
    /// The `constraint` keyword
    Constraint,
    /// The `system` keyword
    System,
    /// The `evolves` keyword
    Evolves,
    /// The `exegesis` keyword
    Exegesis,

    // === Predicate Keywords ===
    /// The `has` predicate
    Has,
    /// The `is` predicate
    Is,
    /// The `derives` keyword
    Derives,
    /// The `from` keyword
    From,
    /// The `requires` predicate
    Requires,
    /// The `uses` predicate
    Uses,
    /// The `emits` predicate
    Emits,
    /// The `matches` predicate
    Matches,
    /// The `never` predicate
    Never,

    // === Evolution Keywords ===
    /// The `adds` operator
    Adds,
    /// The `deprecates` operator
    Deprecates,
    /// The `removes` operator
    Removes,
    /// The `because` keyword
    Because,

    // === Test Keywords ===
    /// The `test` keyword
    Test,
    /// The `given` keyword
    Given,
    /// The `when` keyword
    When,
    /// The `then` keyword
    Then,
    /// The `always` keyword
    Always,

    // === Quantifiers ===
    /// The `each` quantifier
    Each,
    /// The `all` quantifier
    All,
    /// The `no` quantifier
    No,

    // === Delimiters ===
    /// Left brace `{`
    LeftBrace,
    /// Right brace `}`
    RightBrace,

    // === Operators ===
    /// At symbol `@`
    At,
    /// Greater-than `>`
    Greater,
    /// Greater-than-or-equal `>=`
    GreaterEqual,
    /// Equals `=`
    Equal,

    // === Literals ===
    /// A dot-notation identifier
    Identifier,
    /// A semantic version number
    Version,
    /// A quoted string literal
    String,

    // === Special ===
    /// End of file
    Eof,
    /// Unrecognized input
    Error,
}

impl TokenKind {
    /// Returns true if this is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Gene
                | TokenKind::Trait
                | TokenKind::Constraint
                | TokenKind::System
                | TokenKind::Evolves
                | TokenKind::Exegesis
                | TokenKind::Has
                | TokenKind::Is
                | TokenKind::Derives
                | TokenKind::From
                | TokenKind::Requires
                | TokenKind::Uses
                | TokenKind::Emits
                | TokenKind::Matches
                | TokenKind::Never
                | TokenKind::Adds
                | TokenKind::Deprecates
                | TokenKind::Removes
                | TokenKind::Because
                | TokenKind::Test
                | TokenKind::Given
                | TokenKind::When
                | TokenKind::Then
                | TokenKind::Always
                | TokenKind::Each
                | TokenKind::All
                | TokenKind::No
        )
    }

    /// Returns true if this is a predicate keyword.
    pub fn is_predicate(&self) -> bool {
        matches!(
            self,
            TokenKind::Has
                | TokenKind::Is
                | TokenKind::Derives
                | TokenKind::Requires
                | TokenKind::Uses
                | TokenKind::Emits
                | TokenKind::Matches
                | TokenKind::Never
        )
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Gene => write!(f, "gene"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Constraint => write!(f, "constraint"),
            TokenKind::System => write!(f, "system"),
            TokenKind::Evolves => write!(f, "evolves"),
            TokenKind::Exegesis => write!(f, "exegesis"),
            TokenKind::Has => write!(f, "has"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Derives => write!(f, "derives"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Requires => write!(f, "requires"),
            TokenKind::Uses => write!(f, "uses"),
            TokenKind::Emits => write!(f, "emits"),
            TokenKind::Matches => write!(f, "matches"),
            TokenKind::Never => write!(f, "never"),
            TokenKind::Adds => write!(f, "adds"),
            TokenKind::Deprecates => write!(f, "deprecates"),
            TokenKind::Removes => write!(f, "removes"),
            TokenKind::Because => write!(f, "because"),
            TokenKind::Test => write!(f, "test"),
            TokenKind::Given => write!(f, "given"),
            TokenKind::When => write!(f, "when"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Always => write!(f, "always"),
            TokenKind::Each => write!(f, "each"),
            TokenKind::All => write!(f, "all"),
            TokenKind::No => write!(f, "no"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Equal => write!(f, "="),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::Version => write!(f, "version"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Eof => write!(f, "end of file"),
            TokenKind::Error => write!(f, "error"),
        }
    }
}

/// The lexer for Metal DOL source text.
///
/// The lexer maintains internal state as it scans through source text,
/// producing tokens on demand. It handles whitespace and comments
/// automatically, and provides source location tracking.
///
/// # Example
///
/// ```rust
/// use metadol::lexer::Lexer;
///
/// let input = r#"
/// gene container.exists {
///   container has identity
/// }
/// "#;
///
/// let lexer = Lexer::new(input);
/// let tokens: Vec<_> = lexer.collect();
///
/// assert!(tokens.len() > 0);
/// ```
pub struct Lexer<'a> {
    /// The source text being tokenized
    source: &'a str,

    /// Remaining source to process
    remaining: &'a str,

    /// Current byte position in source
    position: usize,

    /// Current line number (1-indexed)
    line: usize,

    /// Current column number (1-indexed)
    column: usize,

    /// Accumulated errors
    errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source text.
    ///
    /// # Arguments
    ///
    /// * `source` - The DOL source text to tokenize
    ///
    /// # Returns
    ///
    /// A new `Lexer` instance positioned at the start of the source
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            remaining: source,
            position: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }

    /// Returns any errors accumulated during lexing.
    pub fn errors(&self) -> &[LexError] {
        &self.errors
    }

    /// Produces the next token from the source.
    ///
    /// Advances the lexer position and returns the next token.
    /// Returns `TokenKind::Eof` when the source is exhausted.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        if self.remaining.is_empty() {
            return Token::new(
                TokenKind::Eof,
                "",
                Span::new(self.position, self.position, self.line, self.column),
            );
        }

        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        // Try to match various token types
        if let Some(token) = self.try_string() {
            return token;
        }

        if let Some(token) = self.try_operator() {
            return token;
        }

        if let Some(token) = self.try_keyword_or_identifier() {
            return token;
        }

        // Unknown character - produce error token
        let ch = self.remaining.chars().next().unwrap();
        self.advance(ch.len_utf8());

        let error = LexError::UnexpectedChar {
            ch,
            span: Span::new(start_pos, self.position, start_line, start_col),
        };
        self.errors.push(error);

        Token::new(
            TokenKind::Error,
            ch.to_string(),
            Span::new(start_pos, self.position, start_line, start_col),
        )
    }

    /// Skips whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            let before = self.remaining.len();
            self.skip_whitespace();

            // Skip comments
            if self.remaining.starts_with("//") {
                self.skip_line_comment();
            }

            // If we didn't skip anything, we're done
            if self.remaining.len() == before {
                break;
            }
        }
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.remaining.chars().next() {
            if ch.is_whitespace() {
                self.advance(ch.len_utf8());
            } else {
                break;
            }
        }
    }

    /// Skips a single-line comment.
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.remaining.chars().next() {
            self.advance(ch.len_utf8());
            if ch == '\n' {
                break;
            }
        }
    }

    /// Tries to lex a string literal.
    fn try_string(&mut self) -> Option<Token> {
        if !self.remaining.starts_with('"') {
            return None;
        }

        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        self.advance(1); // Skip opening quote

        let mut content = String::new();
        let mut escaped = false;

        while let Some(ch) = self.remaining.chars().next() {
            if escaped {
                match ch {
                    'n' => content.push('\n'),
                    't' => content.push('\t'),
                    'r' => content.push('\r'),
                    '"' => content.push('"'),
                    '\\' => content.push('\\'),
                    _ => {
                        let error = LexError::InvalidEscape {
                            ch,
                            span: Span::new(
                                self.position - 1,
                                self.position + 1,
                                self.line,
                                self.column - 1,
                            ),
                        };
                        self.errors.push(error);
                        content.push(ch);
                    }
                }
                escaped = false;
                self.advance(ch.len_utf8());
            } else if ch == '\\' {
                escaped = true;
                self.advance(ch.len_utf8());
            } else if ch == '"' {
                self.advance(1); // Skip closing quote
                return Some(Token::new(
                    TokenKind::String,
                    content,
                    Span::new(start_pos, self.position, start_line, start_col),
                ));
            } else if ch == '\n' {
                // Unterminated string
                let error = LexError::UnterminatedString {
                    span: Span::new(start_pos, self.position, start_line, start_col),
                };
                self.errors.push(error);
                return Some(Token::new(
                    TokenKind::Error,
                    content,
                    Span::new(start_pos, self.position, start_line, start_col),
                ));
            } else {
                content.push(ch);
                self.advance(ch.len_utf8());
            }
        }

        // EOF while in string
        let error = LexError::UnterminatedString {
            span: Span::new(start_pos, self.position, start_line, start_col),
        };
        self.errors.push(error);
        Some(Token::new(
            TokenKind::Error,
            content,
            Span::new(start_pos, self.position, start_line, start_col),
        ))
    }

    /// Tries to lex an operator.
    fn try_operator(&mut self) -> Option<Token> {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        let (kind, len) = if self.remaining.starts_with(">=") {
            (TokenKind::GreaterEqual, 2)
        } else if self.remaining.starts_with('>') {
            (TokenKind::Greater, 1)
        } else if self.remaining.starts_with('@') {
            (TokenKind::At, 1)
        } else if self.remaining.starts_with('=') {
            (TokenKind::Equal, 1)
        } else if self.remaining.starts_with('{') {
            (TokenKind::LeftBrace, 1)
        } else if self.remaining.starts_with('}') {
            (TokenKind::RightBrace, 1)
        } else {
            return None;
        };

        let lexeme: String = self.remaining.chars().take(len).collect();
        self.advance(len);

        Some(Token::new(
            kind,
            lexeme,
            Span::new(start_pos, self.position, start_line, start_col),
        ))
    }

    /// Tries to lex a keyword, identifier, or version.
    fn try_keyword_or_identifier(&mut self) -> Option<Token> {
        let first = self.remaining.chars().next()?;

        // Check for version number
        if first.is_ascii_digit() {
            return self.try_version();
        }

        // Must start with letter
        if !first.is_alphabetic() {
            return None;
        }

        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        // Collect identifier (letters, digits, underscores, dots)
        let mut lexeme = String::new();
        while let Some(ch) = self.remaining.chars().next() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                lexeme.push(ch);
                self.advance(ch.len_utf8());
            } else {
                break;
            }
        }

        // Strip trailing dot if present
        if lexeme.ends_with('.') {
            lexeme.pop();
            self.position -= 1;
            self.column -= 1;
            self.remaining = &self.source[self.position..];
        }

        let kind = self.keyword_kind(&lexeme).unwrap_or(TokenKind::Identifier);

        Some(Token::new(
            kind,
            lexeme,
            Span::new(start_pos, self.position, start_line, start_col),
        ))
    }

    /// Tries to lex a version number.
    fn try_version(&mut self) -> Option<Token> {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        let mut lexeme = String::new();
        let mut dots = 0;

        while let Some(ch) = self.remaining.chars().next() {
            if ch.is_ascii_digit() {
                lexeme.push(ch);
                self.advance(ch.len_utf8());
            } else if ch == '.' && dots < 2 {
                // Check if next char is a digit (version) or not (identifier)
                let next = self.remaining.chars().nth(1);
                if next.is_some_and(|c| c.is_ascii_digit()) {
                    lexeme.push(ch);
                    self.advance(1);
                    dots += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if dots == 2 {
            Some(Token::new(
                TokenKind::Version,
                lexeme,
                Span::new(start_pos, self.position, start_line, start_col),
            ))
        } else {
            // Not a valid version, treat as identifier or error
            Some(Token::new(
                TokenKind::Identifier,
                lexeme,
                Span::new(start_pos, self.position, start_line, start_col),
            ))
        }
    }

    /// Returns the keyword kind for a lexeme, if it's a keyword.
    fn keyword_kind(&self, lexeme: &str) -> Option<TokenKind> {
        match lexeme {
            "gene" => Some(TokenKind::Gene),
            "trait" => Some(TokenKind::Trait),
            "constraint" => Some(TokenKind::Constraint),
            "system" => Some(TokenKind::System),
            "evolves" => Some(TokenKind::Evolves),
            "exegesis" => Some(TokenKind::Exegesis),
            "has" => Some(TokenKind::Has),
            "is" => Some(TokenKind::Is),
            "derives" => Some(TokenKind::Derives),
            "from" => Some(TokenKind::From),
            "requires" => Some(TokenKind::Requires),
            "uses" => Some(TokenKind::Uses),
            "emits" => Some(TokenKind::Emits),
            "matches" => Some(TokenKind::Matches),
            "never" => Some(TokenKind::Never),
            "adds" => Some(TokenKind::Adds),
            "deprecates" => Some(TokenKind::Deprecates),
            "removes" => Some(TokenKind::Removes),
            "because" => Some(TokenKind::Because),
            "test" => Some(TokenKind::Test),
            "given" => Some(TokenKind::Given),
            "when" => Some(TokenKind::When),
            "then" => Some(TokenKind::Then),
            "always" => Some(TokenKind::Always),
            "each" => Some(TokenKind::Each),
            "all" => Some(TokenKind::All),
            "no" => Some(TokenKind::No),
            _ => None,
        }
    }

    /// Advances the lexer by the given number of bytes.
    fn advance(&mut self, bytes: usize) {
        let consumed = &self.remaining[..bytes];
        for ch in consumed.chars() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += bytes;
        self.remaining = &self.source[self.position..];
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("gene trait constraint");
        assert_eq!(lexer.next_token().kind, TokenKind::Gene);
        assert_eq!(lexer.next_token().kind, TokenKind::Trait);
        assert_eq!(lexer.next_token().kind, TokenKind::Constraint);
    }

    #[test]
    fn test_qualified_identifier() {
        let mut lexer = Lexer::new("container.exists");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "container.exists");
    }

    #[test]
    fn test_version() {
        let mut lexer = Lexer::new("0.0.1");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Version);
        assert_eq!(token.lexeme, "0.0.1");
    }

    #[test]
    fn test_string() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.lexeme, "hello world");
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("@ > >=");
        assert_eq!(lexer.next_token().kind, TokenKind::At);
        assert_eq!(lexer.next_token().kind, TokenKind::Greater);
        assert_eq!(lexer.next_token().kind, TokenKind::GreaterEqual);
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("gene // comment\ncontainer");
        assert_eq!(lexer.next_token().kind, TokenKind::Gene);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
    }
}
