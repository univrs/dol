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
    /// The `gene` keyword (DEPRECATED in v0.8.0, use `gen`)
    Gene,
    /// The `gen` keyword (v0.8.0+)
    Gen,
    /// The `trait` keyword
    Trait,
    /// The `constraint` keyword (DEPRECATED in v0.8.0, use `rule`)
    Constraint,
    /// The `rule` keyword (v0.8.0+)
    Rule,
    /// The `system` keyword
    System,
    /// The `evolves` keyword (DEPRECATED in v0.8.0, use `evo`)
    Evolves,
    /// The `evo` keyword (v0.8.0+)
    Evo,
    /// The `exegesis` keyword (DEPRECATED in v0.8.0, use `docs`)
    Exegesis,
    /// The `docs` keyword (v0.8.0+)
    Docs,

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

    // === Composition Operators (DOL 2.0) ===
    /// Forward pipe `|>`
    Pipe,
    /// Function composition `>>`
    Compose,
    /// Monadic bind `:=`
    Bind,
    /// Backward pipe `<|`
    BackPipe,

    // === Meta-Programming Operators (DOL 2.0) ===
    /// Quote/AST capture `'`
    Quote,
    /// Eval/logical not `!`
    Bang,
    /// Macro invocation `#`
    Macro,
    /// Type reflection `?`
    Reflect,
    /// Idiom bracket open `[|`
    IdiomOpen,
    /// Idiom bracket close `|]`
    IdiomClose,

    // === Control Flow Keywords (DOL 2.0) ===
    /// The `let` keyword
    Let,
    /// The `if` keyword
    If,
    /// The `else` keyword
    Else,
    /// The `match` keyword
    Match,
    /// The `for` keyword
    For,
    /// The `while` keyword
    While,
    /// The `loop` keyword
    Loop,
    /// The `break` keyword
    Break,
    /// The `continue` keyword
    Continue,
    /// The `return` keyword
    Return,
    /// The `in` keyword
    In,
    /// The `where` keyword
    Where,

    // === Lambda and Type Syntax (DOL 2.0) ===
    /// Return type/lambda arrow `->`
    Arrow,
    /// Match arm/closure `=>`
    FatArrow,
    /// Lambda parameter delimiter/bitwise or `|`
    Bar,
    /// Wildcard pattern `_`
    Underscore,

    // === Type Keywords (DOL 2.0) ===
    // Legacy type names (DEPRECATED in v0.8.0)
    /// 8-bit signed integer type (DEPRECATED: use `i8`)
    Int8,
    /// 16-bit signed integer type (DEPRECATED: use `i16`)
    Int16,
    /// 32-bit signed integer type (DEPRECATED: use `i32`)
    Int32,
    /// 64-bit signed integer type (DEPRECATED: use `i64`)
    Int64,
    /// 8-bit unsigned integer type (DEPRECATED: use `u8`)
    UInt8,
    /// 16-bit unsigned integer type (DEPRECATED: use `u16`)
    UInt16,
    /// 32-bit unsigned integer type (DEPRECATED: use `u32`)
    UInt32,
    /// 64-bit unsigned integer type (DEPRECATED: use `u64`)
    UInt64,
    /// 32-bit floating point type (DEPRECATED: use `f32`)
    Float32,
    /// 64-bit floating point type (DEPRECATED: use `f64`)
    Float64,
    /// Boolean type (DEPRECATED: use `bool`)
    BoolType,
    /// String type (DEPRECATED: use `string`)
    StringType,
    /// Void type (DEPRECATED: maps to unit type)
    VoidType,

    // New shortened type names (v0.8.0+)
    /// 8-bit signed integer type
    I8,
    /// 16-bit signed integer type
    I16,
    /// 32-bit signed integer type
    I32,
    /// 64-bit signed integer type
    I64,
    /// 128-bit signed integer type
    I128,
    /// 8-bit unsigned integer type
    U8,
    /// 16-bit unsigned integer type
    U16,
    /// 32-bit unsigned integer type
    U32,
    /// 64-bit unsigned integer type
    U64,
    /// 128-bit unsigned integer type
    U128,
    /// 32-bit floating point type
    F32,
    /// 64-bit floating point type
    F64,
    /// Boolean type
    Bool,
    /// String type
    Str,

    // === Function Keyword (DOL 2.0) ===
    /// The `fun` keyword
    Function,

    // === Visibility Keywords (DOL 2.0) ===
    /// The `pub` keyword
    Pub,
    /// The `module` keyword
    Module,
    /// The `use` keyword (import)
    Use,
    /// The `spirit` keyword
    Spirit,
    /// The `config` keyword
    Config,

    // === SEX Keywords (DOL 2.0) ===
    /// The `sex` keyword (side effect marker)
    Sex,
    /// The `var` keyword (mutable variable)
    Var,
    /// The `val` keyword - immutable binding (v0.3.0)
    Val,
    /// The `const` keyword
    Const,
    /// The `extern` keyword
    Extern,

    // === Logic Keywords (DOL 2.0) ===
    /// The `implies` keyword
    Implies,
    /// The `forall` keyword
    Forall,
    /// The `exists` keyword (existential quantifier)
    Exists,

    // === Other Keywords (DOL 2.0) ===
    /// The `impl` keyword (trait implementation)
    Impl,
    /// The `as` keyword
    As,
    /// The `state` keyword (system state)
    State,
    /// The `law` keyword (trait laws)
    Law,
    /// The `mut` keyword (mutable parameter)
    Mut,
    /// The `not` keyword (logical negation)
    Not,
    /// The `migrate` keyword
    Migrate,
    /// The `extends` keyword - inheritance (v0.3.0)
    Extends,
    /// The `type` keyword - type declaration (v0.3.0)
    Type,
    /// The `this` keyword - instance self-reference (v0.7.1)
    This,

    // === Boolean and Null Literals (DOL 2.0) ===
    /// The `true` literal
    True,
    /// The `false` literal
    False,
    /// The `null` literal
    Null,

    // === Operators ===
    /// At symbol `@`
    At,
    /// Greater-than `>`
    Greater,
    /// Greater-than-or-equal `>=`
    GreaterEqual,
    /// Equals `=`
    Equal,
    /// Plus `+`
    Plus,
    /// Minus `-`
    Minus,
    /// Star/multiply `*`
    Star,
    /// Slash/divide `/`
    Slash,
    /// Percent/modulo `%`
    Percent,
    /// Caret/power `^`
    Caret,
    /// Ampersand/bitwise and `&`
    And,
    /// Logical or `||`
    Or,
    /// Equality `==`
    Eq,
    /// Not equal `!=`
    Ne,
    /// Less than `<`
    Lt,
    /// Less than or equal `<=`
    Le,
    /// Member access `.`
    Dot,
    /// Range operator `..`
    DotDot,
    /// Path separator `::`
    PathSep,
    /// Plus-equals `+=`
    PlusEquals,
    /// Minus-equals `-=`
    MinusEquals,
    /// Star-equals `*=`
    StarEquals,
    /// Slash-equals `/=`
    SlashEquals,
    /// Spread operator `...`
    Spread,

    // === Delimiters ===
    /// Left parenthesis `(`
    LeftParen,
    /// Right parenthesis `)`
    RightParen,
    /// Left bracket `[`
    LeftBracket,
    /// Right bracket `]`
    RightBracket,
    /// Comma `,`
    Comma,
    /// Colon `:`
    Colon,
    /// Semicolon `;`
    Semicolon,

    // === Literals ===
    /// A dot-notation identifier
    Identifier,
    /// A semantic version number
    Version,
    /// A quoted string literal
    String,
    /// A character literal (single-quoted)
    Char,

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
                | TokenKind::Gen
                | TokenKind::Trait
                | TokenKind::Constraint
                | TokenKind::Rule
                | TokenKind::System
                | TokenKind::Evolves
                | TokenKind::Evo
                | TokenKind::Exegesis
                | TokenKind::Docs
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
                // DOL 2.0 Control Flow Keywords
                | TokenKind::Let
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::Match
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Loop
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Return
                | TokenKind::In
                | TokenKind::Where
                // DOL 2.0 Type Keywords (Legacy)
                | TokenKind::Int8
                | TokenKind::Int16
                | TokenKind::Int32
                | TokenKind::Int64
                | TokenKind::UInt8
                | TokenKind::UInt16
                | TokenKind::UInt32
                | TokenKind::UInt64
                | TokenKind::Float32
                | TokenKind::Float64
                | TokenKind::BoolType
                | TokenKind::StringType
                | TokenKind::VoidType
                // DOL v0.8.0 Type Keywords (New)
                | TokenKind::I8
                | TokenKind::I16
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::I128
                | TokenKind::U8
                | TokenKind::U16
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::U128
                | TokenKind::F32
                | TokenKind::F64
                | TokenKind::Bool
                | TokenKind::Str
                // DOL 2.0 Function Keyword
                | TokenKind::Function
                // DOL 2.0 Visibility Keywords
                | TokenKind::Pub
                | TokenKind::Module
                | TokenKind::Use
                | TokenKind::Spirit
                // DOL 2.0 SEX Keywords
                | TokenKind::Sex
                | TokenKind::Var
                | TokenKind::Val
                | TokenKind::Const
                | TokenKind::Extern
                // DOL 2.0 Logic Keywords
                | TokenKind::Implies
                | TokenKind::Forall
                | TokenKind::Exists
                // DOL 2.0 Other Keywords
                | TokenKind::Impl
                | TokenKind::As
                | TokenKind::State
                | TokenKind::Law
                | TokenKind::Mut
                | TokenKind::Not
                | TokenKind::Migrate
                | TokenKind::Extends
                | TokenKind::Type
                | TokenKind::This
                // DOL 2.0 Boolean and Null Literals
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
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
            TokenKind::Gen => write!(f, "gen"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Constraint => write!(f, "constraint"),
            TokenKind::Rule => write!(f, "rule"),
            TokenKind::System => write!(f, "system"),
            TokenKind::Evolves => write!(f, "evolves"),
            TokenKind::Evo => write!(f, "evo"),
            TokenKind::Exegesis => write!(f, "exegesis"),
            TokenKind::Docs => write!(f, "docs"),
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
            // DOL 2.0 Composition Operators
            TokenKind::Pipe => write!(f, "|>"),
            TokenKind::Compose => write!(f, ">>"),
            TokenKind::Bind => write!(f, ":="),
            TokenKind::BackPipe => write!(f, "<|"),
            // DOL 2.0 Meta-Programming Operators
            TokenKind::Quote => write!(f, "'"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Macro => write!(f, "#"),
            TokenKind::Reflect => write!(f, "?"),
            TokenKind::IdiomOpen => write!(f, "[|"),
            TokenKind::IdiomClose => write!(f, "|]"),
            // DOL 2.0 Control Flow Keywords
            TokenKind::Let => write!(f, "let"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::For => write!(f, "for"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Where => write!(f, "where"),
            // DOL 2.0 Lambda and Type Syntax
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Bar => write!(f, "|"),
            TokenKind::Underscore => write!(f, "_"),
            // DOL 2.0 Type Keywords (Legacy)
            TokenKind::Int8 => write!(f, "Int8"),
            TokenKind::Int16 => write!(f, "Int16"),
            TokenKind::Int32 => write!(f, "Int32"),
            TokenKind::Int64 => write!(f, "Int64"),
            TokenKind::UInt8 => write!(f, "UInt8"),
            TokenKind::UInt16 => write!(f, "UInt16"),
            TokenKind::UInt32 => write!(f, "UInt32"),
            TokenKind::UInt64 => write!(f, "UInt64"),
            TokenKind::Float32 => write!(f, "Float32"),
            TokenKind::Float64 => write!(f, "Float64"),
            TokenKind::BoolType => write!(f, "Bool"),
            TokenKind::StringType => write!(f, "String"),
            TokenKind::VoidType => write!(f, "Void"),
            // DOL v0.8.0 Type Keywords (New)
            TokenKind::I8 => write!(f, "i8"),
            TokenKind::I16 => write!(f, "i16"),
            TokenKind::I32 => write!(f, "i32"),
            TokenKind::I64 => write!(f, "i64"),
            TokenKind::I128 => write!(f, "i128"),
            TokenKind::U8 => write!(f, "u8"),
            TokenKind::U16 => write!(f, "u16"),
            TokenKind::U32 => write!(f, "u32"),
            TokenKind::U64 => write!(f, "u64"),
            TokenKind::U128 => write!(f, "u128"),
            TokenKind::F32 => write!(f, "f32"),
            TokenKind::F64 => write!(f, "f64"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Str => write!(f, "string"),
            // DOL 2.0 Function Keyword
            TokenKind::Function => write!(f, "fun"),
            // DOL 2.0 Visibility Keywords
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Spirit => write!(f, "spirit"),
            TokenKind::Config => write!(f, "config"),
            // DOL 2.0 SEX Keywords
            TokenKind::Sex => write!(f, "sex"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Val => write!(f, "val"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Extern => write!(f, "extern"),
            // DOL 2.0 Logic Keywords
            TokenKind::Implies => write!(f, "implies"),
            TokenKind::Forall => write!(f, "forall"),
            TokenKind::Exists => write!(f, "exists"),
            // DOL 2.0 Other Keywords
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::As => write!(f, "as"),
            TokenKind::State => write!(f, "state"),
            TokenKind::Law => write!(f, "law"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Migrate => write!(f, "migrate"),
            TokenKind::Extends => write!(f, "extends"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::This => write!(f, "this"),
            // DOL 2.0 Boolean and Null Literals
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Null => write!(f, "null"),
            // Operators
            TokenKind::At => write!(f, "@"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Equal => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::And => write!(f, "&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Eq => write!(f, "=="),
            TokenKind::Ne => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Le => write!(f, "<="),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::PathSep => write!(f, "::"),
            TokenKind::PlusEquals => write!(f, "+="),
            TokenKind::MinusEquals => write!(f, "-="),
            TokenKind::StarEquals => write!(f, "*="),
            TokenKind::SlashEquals => write!(f, "/="),
            TokenKind::Spread => write!(f, "..."),
            // Delimiters
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            // Literals
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::Version => write!(f, "version"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Char => write!(f, "char"),
            // Special
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

        // Check for char literals before operators (since ' could be Quote or char literal)
        if let Some(token) = self.try_char() {
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

            // Skip comments (// style or -- style)
            if self.remaining.starts_with("//") || self.remaining.starts_with("--") {
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

    /// Tries to lex a character literal.
    /// Character literals are single-quoted like 'a', '\n', '\\'
    fn try_char(&mut self) -> Option<Token> {
        if !self.remaining.starts_with('\'') {
            return None;
        }

        // Look ahead to see if this is a char literal (pattern: 'x' or '\x')
        let chars: Vec<char> = self.remaining.chars().take(5).collect();
        if chars.len() < 2 {
            return None; // Not enough chars for a char literal
        }

        // Check for escaped char: '\x'
        if chars.len() >= 4 && chars[1] == '\\' && chars[3] == '\'' {
            let start_pos = self.position;
            let start_line = self.line;
            let start_col = self.column;

            self.advance(1); // Skip opening quote
            self.advance(1); // Skip backslash

            let escaped_char = chars[2];
            let actual_char = match escaped_char {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                '0' => '\0',
                _ => escaped_char,
            };

            self.advance(escaped_char.len_utf8()); // Skip the escaped char
            self.advance(1); // Skip closing quote

            return Some(Token::new(
                TokenKind::Char,
                actual_char.to_string(),
                Span::new(start_pos, self.position, start_line, start_col),
            ));
        }

        // Check for simple char: 'x'
        if chars.len() >= 3 && chars[2] == '\'' && chars[1] != '\\' {
            let start_pos = self.position;
            let start_line = self.line;
            let start_col = self.column;

            self.advance(1); // Skip opening quote
            let ch = chars[1];
            self.advance(ch.len_utf8()); // Skip the char
            self.advance(1); // Skip closing quote

            return Some(Token::new(
                TokenKind::Char,
                ch.to_string(),
                Span::new(start_pos, self.position, start_line, start_col),
            ));
        }

        // Not a char literal, might be a quote operator
        None
    }

    /// Tries to lex an operator.
    fn try_operator(&mut self) -> Option<Token> {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        // Check multi-character operators first (longest match)
        // Check 3-character operators first
        let (kind, len) = if self.remaining.starts_with("...") {
            (TokenKind::Spread, 3)
        // Check 2-character operators
        } else if self.remaining.starts_with("|>") {
            (TokenKind::Pipe, 2)
        } else if self.remaining.starts_with(">>") {
            (TokenKind::Compose, 2)
        } else if self.remaining.starts_with("::") {
            (TokenKind::PathSep, 2)
        } else if self.remaining.starts_with(":=") {
            (TokenKind::Bind, 2)
        } else if self.remaining.starts_with("+=") {
            (TokenKind::PlusEquals, 2)
        } else if self.remaining.starts_with("-=") {
            (TokenKind::MinusEquals, 2)
        } else if self.remaining.starts_with("*=") {
            (TokenKind::StarEquals, 2)
        } else if self.remaining.starts_with("/=") {
            (TokenKind::SlashEquals, 2)
        } else if self.remaining.starts_with("[|") {
            (TokenKind::IdiomOpen, 2)
        } else if self.remaining.starts_with("|]") {
            (TokenKind::IdiomClose, 2)
        } else if self.remaining.starts_with("->") {
            (TokenKind::Arrow, 2)
        } else if self.remaining.starts_with("=>") {
            (TokenKind::FatArrow, 2)
        } else if self.remaining.starts_with("==") {
            (TokenKind::Eq, 2)
        } else if self.remaining.starts_with("!=") {
            (TokenKind::Ne, 2)
        } else if self.remaining.starts_with("<=") {
            (TokenKind::Le, 2)
        } else if self.remaining.starts_with(">=") {
            (TokenKind::GreaterEqual, 2)
        } else if self.remaining.starts_with("&&") {
            (TokenKind::And, 2)
        } else if self.remaining.starts_with("||") {
            (TokenKind::Or, 2)
        } else if self.remaining.starts_with("<|") {
            (TokenKind::BackPipe, 2)
        } else if self.remaining.starts_with("..") {
            (TokenKind::DotDot, 2)
        // Single-character operators
        } else if self.remaining.starts_with('>') {
            (TokenKind::Greater, 1)
        } else if self.remaining.starts_with('<') {
            (TokenKind::Lt, 1)
        } else if self.remaining.starts_with('@') {
            (TokenKind::At, 1)
        } else if self.remaining.starts_with('=') {
            (TokenKind::Equal, 1)
        } else if self.remaining.starts_with('+') {
            (TokenKind::Plus, 1)
        } else if self.remaining.starts_with('-') {
            (TokenKind::Minus, 1)
        } else if self.remaining.starts_with('*') {
            (TokenKind::Star, 1)
        } else if self.remaining.starts_with('/') {
            // Check if this is a comment, not division
            if self.remaining.starts_with("//") {
                return None;
            }
            (TokenKind::Slash, 1)
        } else if self.remaining.starts_with('%') {
            (TokenKind::Percent, 1)
        } else if self.remaining.starts_with('^') {
            (TokenKind::Caret, 1)
        } else if self.remaining.starts_with('&') {
            (TokenKind::And, 1)
        } else if self.remaining.starts_with('|') {
            (TokenKind::Bar, 1)
        } else if self.remaining.starts_with('\'') {
            (TokenKind::Quote, 1)
        } else if self.remaining.starts_with('!') {
            (TokenKind::Bang, 1)
        } else if self.remaining.starts_with('#') {
            (TokenKind::Macro, 1)
        } else if self.remaining.starts_with('?') {
            (TokenKind::Reflect, 1)
        } else if self.remaining.starts_with('(') {
            (TokenKind::LeftParen, 1)
        } else if self.remaining.starts_with(')') {
            (TokenKind::RightParen, 1)
        } else if self.remaining.starts_with('[') {
            (TokenKind::LeftBracket, 1)
        } else if self.remaining.starts_with(']') {
            (TokenKind::RightBracket, 1)
        } else if self.remaining.starts_with('{') {
            (TokenKind::LeftBrace, 1)
        } else if self.remaining.starts_with('}') {
            (TokenKind::RightBrace, 1)
        } else if self.remaining.starts_with(',') {
            (TokenKind::Comma, 1)
        } else if self.remaining.starts_with(':') {
            (TokenKind::Colon, 1)
        } else if self.remaining.starts_with(';') {
            (TokenKind::Semicolon, 1)
        } else if self.remaining.starts_with('.') {
            (TokenKind::Dot, 1)
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

        // Check for underscore wildcard pattern
        if first == '_' {
            // Peek at next character to see if this is a standalone underscore
            let next = self.remaining.chars().nth(1);
            if next.is_none() || !next.unwrap().is_alphanumeric() {
                let start_pos = self.position;
                let start_line = self.line;
                let start_col = self.column;
                self.advance(1);
                return Some(Token::new(
                    TokenKind::Underscore,
                    "_",
                    Span::new(start_pos, self.position, start_line, start_col),
                ));
            }
            // Otherwise, it's part of an identifier, continue below
        }

        // Must start with letter or underscore
        if !first.is_alphabetic() && first != '_' {
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
            // DOL 1.x keywords (with v0.8.0 deprecations)
            "gene" => Some(TokenKind::Gene),
            "gen" => Some(TokenKind::Gen),
            "trait" => Some(TokenKind::Trait),
            "constraint" => Some(TokenKind::Constraint),
            "rule" => Some(TokenKind::Rule),
            "system" => Some(TokenKind::System),
            "evolves" => Some(TokenKind::Evolves),
            "evo" => Some(TokenKind::Evo),
            "exegesis" => Some(TokenKind::Exegesis),
            "docs" => Some(TokenKind::Docs),
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
            // DOL 2.0 control flow keywords
            "let" => Some(TokenKind::Let),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "match" => Some(TokenKind::Match),
            "for" => Some(TokenKind::For),
            "while" => Some(TokenKind::While),
            "loop" => Some(TokenKind::Loop),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "return" => Some(TokenKind::Return),
            "in" => Some(TokenKind::In),
            "where" => Some(TokenKind::Where),
            // DOL 2.0 type keywords (DEPRECATED in v0.8.0)
            "Int8" => Some(TokenKind::Int8),
            "Int16" => Some(TokenKind::Int16),
            "Int32" => Some(TokenKind::Int32),
            "Int64" => Some(TokenKind::Int64),
            "UInt8" => Some(TokenKind::UInt8),
            "UInt16" => Some(TokenKind::UInt16),
            "UInt32" => Some(TokenKind::UInt32),
            "UInt64" => Some(TokenKind::UInt64),
            "Float32" => Some(TokenKind::Float32),
            "Float64" => Some(TokenKind::Float64),
            "Bool" => Some(TokenKind::BoolType),
            "String" => Some(TokenKind::StringType),
            "Void" => Some(TokenKind::VoidType),
            // DOL v0.8.0 type keywords (NEW)
            "i8" => Some(TokenKind::I8),
            "i16" => Some(TokenKind::I16),
            "i32" => Some(TokenKind::I32),
            "i64" => Some(TokenKind::I64),
            "i128" => Some(TokenKind::I128),
            "u8" => Some(TokenKind::U8),
            "u16" => Some(TokenKind::U16),
            "u32" => Some(TokenKind::U32),
            "u64" => Some(TokenKind::U64),
            "u128" => Some(TokenKind::U128),
            "f32" => Some(TokenKind::F32),
            "f64" => Some(TokenKind::F64),
            "bool" => Some(TokenKind::Bool),
            "string" => Some(TokenKind::Str),
            // DOL 2.0 function keyword
            "fun" => Some(TokenKind::Function),
            // DOL 2.0 visibility keywords
            "pub" => Some(TokenKind::Pub),
            "module" => Some(TokenKind::Module),
            "mod" => Some(TokenKind::Module), // Short form of module
            "use" => Some(TokenKind::Use),
            "spirit" => Some(TokenKind::Spirit),
            "config" => Some(TokenKind::Config),
            // DOL 2.0 SEX keywords
            "sex" => Some(TokenKind::Sex),
            "var" => Some(TokenKind::Var),
            "val" => Some(TokenKind::Val),
            "const" => Some(TokenKind::Const),
            "extern" => Some(TokenKind::Extern),
            // DOL 2.0 logic keywords
            "implies" => Some(TokenKind::Implies),
            "forall" => Some(TokenKind::Forall),
            "exists" => Some(TokenKind::Exists),
            // DOL 2.0 other keywords
            "impl" => Some(TokenKind::Impl),
            "as" => Some(TokenKind::As),
            "state" => Some(TokenKind::State),
            "law" => Some(TokenKind::Law),
            "mut" => Some(TokenKind::Mut),
            "not" => Some(TokenKind::Not),
            "migrate" => Some(TokenKind::Migrate),
            "extends" => Some(TokenKind::Extends),
            "type" => Some(TokenKind::Type),
            "this" => Some(TokenKind::This),
            // DOL 2.0 boolean and null literals
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "null" => Some(TokenKind::Null),
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

    // DOL 2.0 Tests

    #[test]
    fn test_dol2_composition_operators() {
        let mut lexer = Lexer::new("|> >> := <|");
        assert_eq!(lexer.next_token().kind, TokenKind::Pipe);
        assert_eq!(lexer.next_token().kind, TokenKind::Compose);
        assert_eq!(lexer.next_token().kind, TokenKind::Bind);
        assert_eq!(lexer.next_token().kind, TokenKind::BackPipe);
    }

    #[test]
    fn test_dol2_meta_operators() {
        let mut lexer = Lexer::new("' ! # ? [| |]");
        assert_eq!(lexer.next_token().kind, TokenKind::Quote);
        assert_eq!(lexer.next_token().kind, TokenKind::Bang);
        assert_eq!(lexer.next_token().kind, TokenKind::Macro);
        assert_eq!(lexer.next_token().kind, TokenKind::Reflect);
        assert_eq!(lexer.next_token().kind, TokenKind::IdiomOpen);
        assert_eq!(lexer.next_token().kind, TokenKind::IdiomClose);
    }

    #[test]
    fn test_dol2_control_flow_keywords() {
        let mut lexer = Lexer::new("if else match for while loop break continue return in where");
        assert_eq!(lexer.next_token().kind, TokenKind::If);
        assert_eq!(lexer.next_token().kind, TokenKind::Else);
        assert_eq!(lexer.next_token().kind, TokenKind::Match);
        assert_eq!(lexer.next_token().kind, TokenKind::For);
        assert_eq!(lexer.next_token().kind, TokenKind::While);
        assert_eq!(lexer.next_token().kind, TokenKind::Loop);
        assert_eq!(lexer.next_token().kind, TokenKind::Break);
        assert_eq!(lexer.next_token().kind, TokenKind::Continue);
        assert_eq!(lexer.next_token().kind, TokenKind::Return);
        assert_eq!(lexer.next_token().kind, TokenKind::In);
        assert_eq!(lexer.next_token().kind, TokenKind::Where);
    }

    #[test]
    fn test_dol2_lambda_and_type_syntax() {
        let mut lexer = Lexer::new("-> => | _");
        assert_eq!(lexer.next_token().kind, TokenKind::Arrow);
        assert_eq!(lexer.next_token().kind, TokenKind::FatArrow);
        assert_eq!(lexer.next_token().kind, TokenKind::Bar);
        assert_eq!(lexer.next_token().kind, TokenKind::Underscore);
    }

    #[test]
    fn test_dol2_type_keywords() {
        let mut lexer = Lexer::new(
            "Int8 Int16 Int32 Int64 UInt8 UInt16 UInt32 UInt64 Float32 Float64 Bool String Void",
        );
        assert_eq!(lexer.next_token().kind, TokenKind::Int8);
        assert_eq!(lexer.next_token().kind, TokenKind::Int16);
        assert_eq!(lexer.next_token().kind, TokenKind::Int32);
        assert_eq!(lexer.next_token().kind, TokenKind::Int64);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt8);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt16);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt32);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt64);
        assert_eq!(lexer.next_token().kind, TokenKind::Float32);
        assert_eq!(lexer.next_token().kind, TokenKind::Float64);
        assert_eq!(lexer.next_token().kind, TokenKind::BoolType);
        assert_eq!(lexer.next_token().kind, TokenKind::StringType);
        assert_eq!(lexer.next_token().kind, TokenKind::VoidType);
    }

    #[test]
    fn test_dol2_function_keyword() {
        let mut lexer = Lexer::new("fun");
        assert_eq!(lexer.next_token().kind, TokenKind::Function);
    }

    #[test]
    fn test_dol2_arithmetic_operators() {
        let mut lexer = Lexer::new("+ - * / % ^");
        assert_eq!(lexer.next_token().kind, TokenKind::Plus);
        assert_eq!(lexer.next_token().kind, TokenKind::Minus);
        assert_eq!(lexer.next_token().kind, TokenKind::Star);
        assert_eq!(lexer.next_token().kind, TokenKind::Slash);
        assert_eq!(lexer.next_token().kind, TokenKind::Percent);
        assert_eq!(lexer.next_token().kind, TokenKind::Caret);
    }

    #[test]
    fn test_dol2_comparison_operators() {
        let mut lexer = Lexer::new("== != < <= > >=");
        assert_eq!(lexer.next_token().kind, TokenKind::Eq);
        assert_eq!(lexer.next_token().kind, TokenKind::Ne);
        assert_eq!(lexer.next_token().kind, TokenKind::Lt);
        assert_eq!(lexer.next_token().kind, TokenKind::Le);
        assert_eq!(lexer.next_token().kind, TokenKind::Greater);
        assert_eq!(lexer.next_token().kind, TokenKind::GreaterEqual);
    }

    #[test]
    fn test_dol2_logical_operators() {
        let mut lexer = Lexer::new("&& ||");
        assert_eq!(lexer.next_token().kind, TokenKind::And);
        assert_eq!(lexer.next_token().kind, TokenKind::Or);
    }

    #[test]
    fn test_dol2_delimiters() {
        let mut lexer = Lexer::new("( ) [ ] { } , : ; .");
        assert_eq!(lexer.next_token().kind, TokenKind::LeftParen);
        assert_eq!(lexer.next_token().kind, TokenKind::RightParen);
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBracket);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBracket);
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::Comma);
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        assert_eq!(lexer.next_token().kind, TokenKind::Dot);
    }

    #[test]
    fn test_dol2_underscore_wildcard() {
        // Standalone underscore
        let mut lexer = Lexer::new("_ _,");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Underscore);
        assert_eq!(token.lexeme, "_");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Underscore);

        // Underscore in identifier
        let mut lexer = Lexer::new("_foo foo_bar");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "_foo");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "foo_bar");
    }

    #[test]
    fn test_dol2_member_access_vs_qualified_identifier() {
        // Qualified identifier (no spaces)
        let mut lexer = Lexer::new("container.exists");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "container.exists");

        // Member access (with spaces)
        let mut lexer = Lexer::new("obj . field");
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Dot);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
    }

    #[test]
    fn test_dol2_operator_disambiguation() {
        // Test that multi-char operators are matched correctly
        let mut lexer = Lexer::new("| ||");
        assert_eq!(lexer.next_token().kind, TokenKind::Bar);
        assert_eq!(lexer.next_token().kind, TokenKind::Or);

        let mut lexer = Lexer::new("> >>");
        assert_eq!(lexer.next_token().kind, TokenKind::Greater);
        assert_eq!(lexer.next_token().kind, TokenKind::Compose);

        let mut lexer = Lexer::new(": :=");
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Bind);
    }

    #[test]
    fn test_v030_keywords() {
        let mut lexer = Lexer::new("val var extends forall type");
        assert_eq!(lexer.next_token().kind, TokenKind::Val);
        assert_eq!(lexer.next_token().kind, TokenKind::Var);
        assert_eq!(lexer.next_token().kind, TokenKind::Extends);
        assert_eq!(lexer.next_token().kind, TokenKind::Forall);
        assert_eq!(lexer.next_token().kind, TokenKind::Type);
    }

    // DOL v0.8.0 Tests

    #[test]
    fn test_v080_new_keywords() {
        let mut lexer = Lexer::new("gen rule evo docs");
        assert_eq!(lexer.next_token().kind, TokenKind::Gen);
        assert_eq!(lexer.next_token().kind, TokenKind::Rule);
        assert_eq!(lexer.next_token().kind, TokenKind::Evo);
        assert_eq!(lexer.next_token().kind, TokenKind::Docs);
    }

    #[test]
    fn test_v080_deprecated_keywords() {
        // Legacy keywords should still work
        let mut lexer = Lexer::new("gene constraint evolves exegesis");
        assert_eq!(lexer.next_token().kind, TokenKind::Gene);
        assert_eq!(lexer.next_token().kind, TokenKind::Constraint);
        assert_eq!(lexer.next_token().kind, TokenKind::Evolves);
        assert_eq!(lexer.next_token().kind, TokenKind::Exegesis);
    }

    #[test]
    fn test_v080_new_type_keywords_signed() {
        let mut lexer = Lexer::new("i8 i16 i32 i64 i128");
        assert_eq!(lexer.next_token().kind, TokenKind::I8);
        assert_eq!(lexer.next_token().kind, TokenKind::I16);
        assert_eq!(lexer.next_token().kind, TokenKind::I32);
        assert_eq!(lexer.next_token().kind, TokenKind::I64);
        assert_eq!(lexer.next_token().kind, TokenKind::I128);
    }

    #[test]
    fn test_v080_new_type_keywords_unsigned() {
        let mut lexer = Lexer::new("u8 u16 u32 u64 u128");
        assert_eq!(lexer.next_token().kind, TokenKind::U8);
        assert_eq!(lexer.next_token().kind, TokenKind::U16);
        assert_eq!(lexer.next_token().kind, TokenKind::U32);
        assert_eq!(lexer.next_token().kind, TokenKind::U64);
        assert_eq!(lexer.next_token().kind, TokenKind::U128);
    }

    #[test]
    fn test_v080_new_type_keywords_float() {
        let mut lexer = Lexer::new("f32 f64");
        assert_eq!(lexer.next_token().kind, TokenKind::F32);
        assert_eq!(lexer.next_token().kind, TokenKind::F64);
    }

    #[test]
    fn test_v080_new_type_keywords_bool_string() {
        let mut lexer = Lexer::new("bool string");
        assert_eq!(lexer.next_token().kind, TokenKind::Bool);
        assert_eq!(lexer.next_token().kind, TokenKind::Str);
    }

    #[test]
    fn test_v080_deprecated_type_keywords() {
        // Legacy type keywords should still work
        let mut lexer = Lexer::new("Int8 Int16 Int32 Int64 UInt8 UInt16 UInt32 UInt64");
        assert_eq!(lexer.next_token().kind, TokenKind::Int8);
        assert_eq!(lexer.next_token().kind, TokenKind::Int16);
        assert_eq!(lexer.next_token().kind, TokenKind::Int32);
        assert_eq!(lexer.next_token().kind, TokenKind::Int64);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt8);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt16);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt32);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt64);
    }

    #[test]
    fn test_v080_deprecated_type_keywords_float() {
        let mut lexer = Lexer::new("Float32 Float64");
        assert_eq!(lexer.next_token().kind, TokenKind::Float32);
        assert_eq!(lexer.next_token().kind, TokenKind::Float64);
    }

    #[test]
    fn test_v080_deprecated_bool_string_void() {
        let mut lexer = Lexer::new("Bool String Void");
        assert_eq!(lexer.next_token().kind, TokenKind::BoolType);
        assert_eq!(lexer.next_token().kind, TokenKind::StringType);
        assert_eq!(lexer.next_token().kind, TokenKind::VoidType);
    }

    #[test]
    fn test_v080_mixed_old_new_keywords() {
        // Test that both old and new keywords can coexist
        let mut lexer = Lexer::new("gene gen trait rule");
        assert_eq!(lexer.next_token().kind, TokenKind::Gene);
        assert_eq!(lexer.next_token().kind, TokenKind::Gen);
        assert_eq!(lexer.next_token().kind, TokenKind::Trait);
        assert_eq!(lexer.next_token().kind, TokenKind::Rule);
    }

    #[test]
    fn test_v080_mixed_old_new_types() {
        // Test that both old and new type keywords can coexist
        let mut lexer = Lexer::new("Int32 i32 u64 UInt64");
        assert_eq!(lexer.next_token().kind, TokenKind::Int32);
        assert_eq!(lexer.next_token().kind, TokenKind::I32);
        assert_eq!(lexer.next_token().kind, TokenKind::U64);
        assert_eq!(lexer.next_token().kind, TokenKind::UInt64);
    }

    #[test]
    fn test_v080_gen_declaration() {
        // Test a complete gen declaration with new syntax
        let mut lexer = Lexer::new("gen container.exists { }");
        assert_eq!(lexer.next_token().kind, TokenKind::Gen);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_v080_rule_declaration() {
        // Test a rule declaration with new syntax
        let mut lexer = Lexer::new("rule no.null.refs { }");
        assert_eq!(lexer.next_token().kind, TokenKind::Rule);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_v080_evo_keyword() {
        // Test the evo keyword
        let mut lexer = Lexer::new("evo 0.2.0 { }");
        assert_eq!(lexer.next_token().kind, TokenKind::Evo);
        assert_eq!(lexer.next_token().kind, TokenKind::Version);
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_v080_docs_keyword() {
        // Test the docs keyword
        let mut lexer = Lexer::new("docs \"Documentation string\"");
        assert_eq!(lexer.next_token().kind, TokenKind::Docs);
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.lexeme, "Documentation string");
    }

    #[test]
    fn test_v080_i128_type() {
        // Test the new i128 type
        let mut lexer = Lexer::new("i128");
        assert_eq!(lexer.next_token().kind, TokenKind::I128);
    }

    #[test]
    fn test_v080_u128_type() {
        // Test the new u128 type
        let mut lexer = Lexer::new("u128");
        assert_eq!(lexer.next_token().kind, TokenKind::U128);
    }

    #[test]
    fn test_v080_all_new_keywords_lexemes() {
        // Test that lexemes are captured correctly
        let mut lexer = Lexer::new("gen rule evo docs");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Gen);
        assert_eq!(token.lexeme, "gen");

        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Rule);
        assert_eq!(token.lexeme, "rule");

        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Evo);
        assert_eq!(token.lexeme, "evo");

        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Docs);
        assert_eq!(token.lexeme, "docs");
    }

    #[test]
    fn test_v080_all_new_type_keywords_lexemes() {
        // Test that type lexemes are captured correctly
        let mut lexer = Lexer::new("i8 u8 f32 bool string");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::I8);
        assert_eq!(token.lexeme, "i8");

        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::U8);
        assert_eq!(token.lexeme, "u8");

        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::F32);
        assert_eq!(token.lexeme, "f32");

        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Bool);
        assert_eq!(token.lexeme, "bool");

        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Str);
        assert_eq!(token.lexeme, "string");
    }

    #[test]
    fn test_v080_keyword_case_sensitivity() {
        // Test that keywords are case-sensitive
        let mut lexer = Lexer::new("Gen GEN gEn RULE Rule");

        // Gen is not a keyword, should be identifier
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "Gen");

        // GEN is not a keyword
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);

        // gEn is not a keyword
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);

        // RULE is not a keyword
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);

        // Rule is not a keyword
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Identifier);
    }

    #[test]
    fn test_v080_type_case_sensitivity() {
        // Test that type keywords are case-sensitive
        let mut lexer = Lexer::new("I8 U8 F32 BOOL STRING");

        // All should be identifiers, not type keywords
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
    }
}
