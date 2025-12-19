//! Test file parser for Metal DOL.
//!
//! This module parses `.dol.test` files that define behavioral tests
//! for DOL declarations using given/when/then syntax.
//!
//! # Test File Syntax
//!
//! ```dol
//! test container_creation {
//!   given container exists
//!   when container is created
//!   then container has identity
//!   always
//! }
//!
//! test container_lifecycle {
//!   given container is created
//!   when container is started
//!   then container is running
//! }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::test_parser::parse_test_file;
//!
//! let source = r#"
//! test example {
//!   given thing exists
//!   when thing is activated
//!   then thing has state
//! }
//! "#;
//!
//! let tests = parse_test_file(source).unwrap();
//! assert_eq!(tests.len(), 1);
//! ```

use crate::ast::Span;
use crate::error::ParseError;
use crate::lexer::{Lexer, Token, TokenKind};

/// A test declaration from a .dol.test file.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestDeclaration {
    /// The name of the test
    pub name: String,

    /// Given clauses (preconditions)
    pub given: Vec<String>,

    /// When clauses (actions)
    pub when: Vec<String>,

    /// Then clauses (postconditions)
    pub then: Vec<String>,

    /// Whether this test should always pass (invariant)
    pub always: bool,

    /// Source location
    pub span: Span,
}

/// Parser for .dol.test files.
pub struct TestParser<'a> {
    lexer: Lexer<'a>,
    source: &'a str,
    current: Token,
    previous: Token,
    peeked: Option<Token>,
}

impl<'a> TestParser<'a> {
    /// Creates a new test parser for the given source.
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();

        Self {
            lexer,
            source,
            current,
            previous: Token::default(),
            peeked: None,
        }
    }

    /// Parses all test declarations from the source.
    pub fn parse(&mut self) -> Result<Vec<TestDeclaration>, ParseError> {
        let mut tests = Vec::new();

        while self.current.kind != TokenKind::Eof {
            if self.current.kind == TokenKind::Test {
                tests.push(self.parse_test()?);
            } else {
                // Skip whitespace/comments until we find a test
                self.advance();
            }
        }

        Ok(tests)
    }

    fn parse_test(&mut self) -> Result<TestDeclaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Test)?;

        // Parse test name
        let name = if self.current.kind == TokenKind::Identifier {
            let n = self.current.lexeme.clone();
            self.advance();
            n
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "test name".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            });
        };

        self.expect(TokenKind::LeftBrace)?;

        let mut given = Vec::new();
        let mut when = Vec::new();
        let mut then = Vec::new();
        let mut always = false;

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            match self.current.kind {
                TokenKind::Given => {
                    self.advance();
                    given.push(self.parse_clause_phrase()?);
                }
                TokenKind::When => {
                    self.advance();
                    when.push(self.parse_clause_phrase()?);
                }
                TokenKind::Then => {
                    self.advance();
                    then.push(self.parse_clause_phrase()?);
                }
                TokenKind::Always => {
                    self.advance();
                    always = true;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "given, when, then, always, or '}'".to_string(),
                        found: format!("{:?}", self.current.kind),
                        span: self.current.span,
                    });
                }
            }
        }

        let end_span = self.current.span;
        self.expect(TokenKind::RightBrace)?;

        Ok(TestDeclaration {
            name,
            given,
            when,
            then,
            always,
            span: start_span.merge(&end_span),
        })
    }

    fn parse_clause_phrase(&mut self) -> Result<String, ParseError> {
        let mut phrase = String::new();

        // A clause phrase continues until we hit another clause keyword or closing brace
        while self.current.kind != TokenKind::Given
            && self.current.kind != TokenKind::When
            && self.current.kind != TokenKind::Then
            && self.current.kind != TokenKind::Always
            && self.current.kind != TokenKind::RightBrace
            && self.current.kind != TokenKind::Eof
        {
            if !phrase.is_empty() {
                phrase.push(' ');
            }
            phrase.push_str(&self.current.lexeme);
            self.advance();
        }

        if phrase.is_empty() {
            return Err(ParseError::UnexpectedToken {
                expected: "clause content".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            });
        }

        Ok(phrase)
    }

    fn advance(&mut self) {
        self.previous = std::mem::replace(
            &mut self.current,
            self.peeked
                .take()
                .unwrap_or_else(|| self.lexer.next_token()),
        );
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.current.kind == kind {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            })
        }
    }

    /// Returns the source string.
    #[allow(dead_code)]
    fn lexer_source(&self) -> &'a str {
        self.source
    }
}

/// Parses a .dol.test file source string into test declarations.
pub fn parse_test_file(source: &str) -> Result<Vec<TestDeclaration>, ParseError> {
    let mut parser = TestParser::new(source);
    parser.parse()
}

/// Generates Rust test code from test declarations.
pub fn generate_rust_tests(tests: &[TestDeclaration], module_name: &str) -> String {
    let mut output = String::new();

    output.push_str("//! Generated tests from .dol.test file\n");
    output.push_str("//! DO NOT EDIT - regenerate with dol-test\n\n");
    output.push_str("#[cfg(test)]\n");
    output.push_str(&format!("mod {} {{\n", module_name));
    output.push_str("    use super::*;\n\n");

    for test in tests {
        output.push_str(&generate_single_test(test));
        output.push('\n');
    }

    output.push_str("}\n");
    output
}

fn generate_single_test(test: &TestDeclaration) -> String {
    let mut output = String::new();

    // Generate doc comment
    output.push_str(&format!("    /// Test: {}\n", test.name));
    if !test.given.is_empty() {
        output.push_str("    ///\n");
        output.push_str("    /// Given:\n");
        for g in &test.given {
            output.push_str(&format!("    /// - {}\n", g));
        }
    }
    if !test.when.is_empty() {
        output.push_str("    /// When:\n");
        for w in &test.when {
            output.push_str(&format!("    /// - {}\n", w));
        }
    }
    if !test.then.is_empty() {
        output.push_str("    /// Then:\n");
        for t in &test.then {
            output.push_str(&format!("    /// - {}\n", t));
        }
    }
    if test.always {
        output.push_str("    /// (invariant - always holds)\n");
    }

    output.push_str("    #[test]\n");
    output.push_str(&format!("    fn test_{}() {{\n", sanitize_name(&test.name)));
    output.push_str("        // TODO: Implement test assertions\n");

    // Generate assertion stubs for each then clause
    for then_clause in &test.then {
        output.push_str(&format!("        // assert: {}\n", then_clause));
    }

    if test.always {
        output.push_str("        // This is an invariant - should always hold\n");
    }

    output.push_str("        todo!(\"Implement test\");\n");
    output.push_str("    }\n");

    output
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_test() {
        let source = r#"
        test example {
            given thing exists
            when thing is activated
            then thing has state
        }
        "#;

        let tests = parse_test_file(source).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "example");
        assert_eq!(tests[0].given.len(), 1);
        assert_eq!(tests[0].when.len(), 1);
        assert_eq!(tests[0].then.len(), 1);
        assert!(!tests[0].always);
    }

    #[test]
    fn test_parse_test_with_always() {
        let source = r#"
        test invariant {
            given system is running
            then state is consistent
            always
        }
        "#;

        let tests = parse_test_file(source).unwrap();
        assert_eq!(tests.len(), 1);
        assert!(tests[0].always);
    }

    #[test]
    fn test_parse_multiple_tests() {
        let source = r#"
        test first {
            given a
            then b
        }
        test second {
            given c
            then d
        }
        "#;

        let tests = parse_test_file(source).unwrap();
        assert_eq!(tests.len(), 2);
        assert_eq!(tests[0].name, "first");
        assert_eq!(tests[1].name, "second");
    }

    #[test]
    fn test_generate_rust_test() {
        let test = TestDeclaration {
            name: "container_lifecycle".to_string(),
            given: vec!["container exists".to_string()],
            when: vec!["container is started".to_string()],
            then: vec!["container is running".to_string()],
            always: false,
            span: Span::default(),
        };

        let output = generate_single_test(&test);
        assert!(output.contains("fn test_container_lifecycle()"));
        assert!(output.contains("container is running"));
    }
}
