//! Parser for Metal DOL.
//!
//! This module provides a recursive descent parser that transforms a stream
//! of tokens into an Abstract Syntax Tree (AST).
//!
//! # Example
//!
//! ```rust
//! use metadol::parser::Parser;
//! use metadol::ast::Declaration;
//!
//! let input = r#"
//! gene container.exists {
//!   container has identity
//! }
//!
//! exegesis {
//!   A container is the fundamental unit.
//! }
//! "#;
//!
//! let mut parser = Parser::new(input);
//! let result = parser.parse();
//! assert!(result.is_ok());
//! ```

use crate::ast::*;
use crate::error::ParseError;
use crate::lexer::{Lexer, Token, TokenKind};

/// The parser for Metal DOL source text.
///
/// The parser uses recursive descent to transform tokens into an AST.
/// It provides helpful error messages with source locations.
pub struct Parser<'a> {
    /// The underlying lexer
    lexer: Lexer<'a>,

    /// The source text (for exegesis parsing)
    source: &'a str,

    /// Current token
    current: Token,

    /// Previous token (for span tracking)
    previous: Token,

    /// Peeked token for lookahead (if any)
    peeked: Option<Token>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given source text.
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();
        let previous = Token::new(TokenKind::Eof, "", Span::default());

        Parser {
            lexer,
            source,
            current,
            previous,
            peeked: None,
        }
    }

    /// Parses the source into a declaration.
    ///
    /// # Returns
    ///
    /// The parsed `Declaration` on success, or a `ParseError` on failure.
    pub fn parse(&mut self) -> Result<Declaration, ParseError> {
        let decl = self.parse_declaration()?;
        self.expect(TokenKind::Eof)?;
        Ok(decl)
    }

    /// Parses a declaration.
    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        match self.current.kind {
            TokenKind::Gene => self.parse_gene(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Constraint => self.parse_constraint(),
            TokenKind::System => self.parse_system(),
            TokenKind::Evolves => self.parse_evolution(),
            _ => Err(ParseError::InvalidDeclaration {
                found: self.current.lexeme.clone(),
                span: self.current.span,
            }),
        }
    }

    /// Parses a gene declaration.
    fn parse_gene(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Gene)?;

        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftBrace)?;

        let statements = self.parse_statements()?;

        self.expect(TokenKind::RightBrace)?;

        let exegesis = self.parse_exegesis()?;

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Gene(Gene {
            name,
            statements,
            exegesis,
            span,
        }))
    }

    /// Parses a trait declaration.
    fn parse_trait(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Trait)?;

        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftBrace)?;

        let statements = self.parse_statements()?;

        self.expect(TokenKind::RightBrace)?;

        let exegesis = self.parse_exegesis()?;

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Trait(Trait {
            name,
            statements,
            exegesis,
            span,
        }))
    }

    /// Parses a constraint declaration.
    fn parse_constraint(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Constraint)?;

        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftBrace)?;

        let statements = self.parse_statements()?;

        self.expect(TokenKind::RightBrace)?;

        let exegesis = self.parse_exegesis()?;

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Constraint(Constraint {
            name,
            statements,
            exegesis,
            span,
        }))
    }

    /// Parses a system declaration.
    fn parse_system(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::System)?;

        let name = self.expect_identifier()?;
        self.expect(TokenKind::At)?;
        let version = self.expect_version()?;
        self.expect(TokenKind::LeftBrace)?;

        let mut requirements = Vec::new();
        let mut statements = Vec::new();

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            if self.current.kind == TokenKind::Requires
                && self.peek_is_identifier()
                && self.peek_is_version_constraint()
            {
                requirements.push(self.parse_requirement()?);
            } else {
                statements.push(self.parse_statement()?);
            }
        }

        self.expect(TokenKind::RightBrace)?;

        let exegesis = self.parse_exegesis()?;

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::System(System {
            name,
            version,
            requirements,
            statements,
            exegesis,
            span,
        }))
    }

    /// Parses an evolution declaration.
    fn parse_evolution(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Evolves)?;

        let name = self.expect_identifier()?;
        self.expect(TokenKind::At)?;
        let version = self.expect_version()?;
        self.expect(TokenKind::Greater)?;
        let parent_version = self.expect_version()?;
        self.expect(TokenKind::LeftBrace)?;

        let mut additions = Vec::new();
        let mut deprecations = Vec::new();
        let mut removals = Vec::new();
        let mut rationale = None;

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            match self.current.kind {
                TokenKind::Adds => {
                    self.advance();
                    additions.push(self.parse_statement()?);
                }
                TokenKind::Deprecates => {
                    self.advance();
                    deprecations.push(self.parse_statement()?);
                }
                TokenKind::Removes => {
                    self.advance();
                    let name = self.expect_identifier()?;
                    removals.push(name);
                }
                TokenKind::Because => {
                    self.advance();
                    let text = self.expect_string()?;
                    rationale = Some(text);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "adds, deprecates, removes, or because".to_string(),
                        found: format!("'{}'", self.current.lexeme),
                        span: self.current.span,
                    });
                }
            }
        }

        self.expect(TokenKind::RightBrace)?;

        let exegesis = self.parse_exegesis()?;

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Evolution(Evolution {
            name,
            version,
            parent_version,
            additions,
            deprecations,
            removals,
            rationale,
            exegesis,
            span,
        }))
    }

    /// Parses multiple statements until a closing brace.
    fn parse_statements(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    /// Parses a single statement.
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start_span = self.current.span;

        // Handle 'uses' statements
        if self.current.kind == TokenKind::Uses {
            self.advance();
            let reference = self.expect_identifier()?;
            return Ok(Statement::Uses {
                reference,
                span: start_span.merge(&self.previous.span),
            });
        }

        // Handle quantified statements
        if matches!(self.current.kind, TokenKind::Each | TokenKind::All) {
            let quantifier = match self.current.kind {
                TokenKind::Each => Quantifier::Each,
                TokenKind::All => Quantifier::All,
                _ => unreachable!(),
            };
            self.advance();
            // For quantified statements, parse the complete phrase including predicates
            let phrase = self.parse_quantified_phrase()?;
            return Ok(Statement::Quantified {
                quantifier,
                phrase,
                span: start_span.merge(&self.previous.span),
            });
        }

        // Parse subject
        let subject = self.expect_identifier()?;

        // Determine statement type based on predicate
        match self.current.kind {
            TokenKind::Has => {
                self.advance();
                let property = self.expect_identifier()?;
                Ok(Statement::Has {
                    subject,
                    property,
                    span: start_span.merge(&self.previous.span),
                })
            }
            TokenKind::Is => {
                self.advance();
                let state = self.expect_identifier()?;
                Ok(Statement::Is {
                    subject,
                    state,
                    span: start_span.merge(&self.previous.span),
                })
            }
            TokenKind::Derives => {
                self.advance();
                self.expect(TokenKind::From)?;
                let origin = self.parse_phrase()?;
                Ok(Statement::DerivesFrom {
                    subject,
                    origin,
                    span: start_span.merge(&self.previous.span),
                })
            }
            TokenKind::Requires => {
                self.advance();
                let requirement = self.parse_phrase()?;
                Ok(Statement::Requires {
                    subject,
                    requirement,
                    span: start_span.merge(&self.previous.span),
                })
            }
            TokenKind::Emits => {
                self.advance();
                let event = self.expect_identifier()?;
                Ok(Statement::Emits {
                    action: subject,
                    event,
                    span: start_span.merge(&self.previous.span),
                })
            }
            TokenKind::Matches => {
                self.advance();
                let target = self.parse_phrase()?;
                Ok(Statement::Matches {
                    subject,
                    target,
                    span: start_span.merge(&self.previous.span),
                })
            }
            TokenKind::Never => {
                self.advance();
                let action = self.expect_identifier()?;
                Ok(Statement::Never {
                    subject,
                    action,
                    span: start_span.merge(&self.previous.span),
                })
            }
            // Handle phrases that continue with more identifiers
            TokenKind::Identifier => {
                // This might be part of a longer phrase
                let mut phrase = subject;
                while self.current.kind == TokenKind::Identifier {
                    phrase.push(' ');
                    phrase.push_str(&self.current.lexeme);
                    self.advance();

                    // Check if we've hit a predicate
                    if self.current.kind.is_predicate() {
                        break;
                    }
                }

                // Now check what predicate follows
                match self.current.kind {
                    TokenKind::Emits => {
                        self.advance();
                        let event = self.expect_identifier()?;
                        Ok(Statement::Emits {
                            action: phrase,
                            event,
                            span: start_span.merge(&self.previous.span),
                        })
                    }
                    TokenKind::Never => {
                        self.advance();
                        let action = self.expect_identifier()?;
                        Ok(Statement::Never {
                            subject: phrase,
                            action,
                            span: start_span.merge(&self.previous.span),
                        })
                    }
                    TokenKind::Matches => {
                        self.advance();
                        let target = self.parse_phrase()?;
                        Ok(Statement::Matches {
                            subject: phrase,
                            target,
                            span: start_span.merge(&self.previous.span),
                        })
                    }
                    TokenKind::Is => {
                        self.advance();
                        let state = self.expect_identifier()?;
                        Ok(Statement::Is {
                            subject: phrase,
                            state,
                            span: start_span.merge(&self.previous.span),
                        })
                    }
                    TokenKind::Has => {
                        self.advance();
                        let property = self.expect_identifier()?;
                        Ok(Statement::Has {
                            subject: phrase,
                            property,
                            span: start_span.merge(&self.previous.span),
                        })
                    }
                    TokenKind::Requires => {
                        self.advance();
                        let requirement = self.parse_phrase()?;
                        Ok(Statement::Requires {
                            subject: phrase,
                            requirement,
                            span: start_span.merge(&self.previous.span),
                        })
                    }
                    _ => Err(ParseError::InvalidStatement {
                        message: format!("expected predicate after '{}'", phrase),
                        span: self.current.span,
                    }),
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "predicate (has, is, derives, requires, etc.)".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            }),
        }
    }

    /// Parses a phrase (one or more identifiers).
    ///
    /// Uses lookahead to avoid consuming identifiers that start new statements.
    /// If the token after an identifier is a predicate, that identifier starts
    /// a new statement and should not be included in this phrase.
    ///
    /// Note: The `no` keyword is allowed in phrases since it's not used as a
    /// quantifier (only `each` and `all` are used).
    fn parse_phrase(&mut self) -> Result<String, ParseError> {
        let mut phrase = String::new();

        // First token must be identifier or 'no' (which can appear in phrases)
        if self.current.kind != TokenKind::Identifier && self.current.kind != TokenKind::No {
            return Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            });
        }

        phrase.push_str(&self.current.lexeme);
        self.advance();

        // Continue while we see identifiers or 'no', but use lookahead to stop
        // at statement boundaries
        while self.current.kind == TokenKind::Identifier || self.current.kind == TokenKind::No {
            // Peek at what comes after this token
            let next_kind = self.peek().kind;

            // If the next token is a predicate, this identifier starts
            // a new statement - don't include it in this phrase
            if next_kind.is_predicate() {
                break;
            }

            phrase.push(' ');
            phrase.push_str(&self.current.lexeme);
            self.advance();
        }

        Ok(phrase)
    }

    /// Parses a quantified phrase (for 'each'/'all' statements).
    ///
    /// This continues parsing until end of statement, including predicates like 'emits'.
    /// For example: "each transition emits event" captures "transition emits event".
    fn parse_quantified_phrase(&mut self) -> Result<String, ParseError> {
        let mut phrase = String::new();

        // First token (identifier) is required
        if self.current.kind != TokenKind::Identifier {
            return Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            });
        }

        phrase.push_str(&self.current.lexeme);
        self.advance();

        // Continue until we hit a statement boundary (RightBrace, EOF, or start of new statement)
        loop {
            match self.current.kind {
                // End of statement boundaries
                TokenKind::RightBrace | TokenKind::Eof => break,

                // New statement starters (not predicates)
                TokenKind::Uses | TokenKind::Each | TokenKind::All => break,

                // Identifiers continue the phrase
                TokenKind::Identifier => {
                    phrase.push(' ');
                    phrase.push_str(&self.current.lexeme);
                    self.advance();
                }

                // Predicates that can appear in quantified phrases
                TokenKind::Has
                | TokenKind::Is
                | TokenKind::Emits
                | TokenKind::Matches
                | TokenKind::Never
                | TokenKind::Requires
                | TokenKind::Derives
                | TokenKind::From => {
                    phrase.push(' ');
                    phrase.push_str(&self.current.lexeme);
                    self.advance();
                }

                // Any other token ends the phrase
                _ => break,
            }
        }

        Ok(phrase)
    }

    /// Parses a version requirement.
    fn parse_requirement(&mut self) -> Result<Requirement, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Requires)?;

        let name = self.expect_identifier()?;

        let constraint = match self.current.kind {
            TokenKind::GreaterEqual => {
                self.advance();
                ">=".to_string()
            }
            TokenKind::Greater => {
                self.advance();
                ">".to_string()
            }
            TokenKind::Equal => {
                self.advance();
                "=".to_string()
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "version constraint (>=, >, =)".to_string(),
                    found: format!("'{}'", self.current.lexeme),
                    span: self.current.span,
                });
            }
        };

        let version = self.expect_version()?;

        Ok(Requirement {
            name,
            constraint,
            version,
            span: start_span.merge(&self.previous.span),
        })
    }

    /// Parses the exegesis block.
    fn parse_exegesis(&mut self) -> Result<String, ParseError> {
        if self.current.kind != TokenKind::Exegesis {
            return Err(ParseError::MissingExegesis {
                span: self.current.span,
            });
        }

        self.advance(); // consume 'exegesis'
        self.expect(TokenKind::LeftBrace)?;

        // Collect all text until closing brace
        // We need to handle nested braces
        let mut content = String::new();
        let mut brace_depth = 1;

        // Get position after opening brace
        let start_pos = self.current.span.start;

        // Re-lex from the source to get raw text
        let source_after_brace = &self.lexer_source()[start_pos..];

        for ch in source_after_brace.chars() {
            if ch == '{' {
                brace_depth += 1;
                content.push(ch);
            } else if ch == '}' {
                brace_depth -= 1;
                if brace_depth == 0 {
                    break;
                }
                content.push(ch);
            } else {
                content.push(ch);
            }
        }

        // Skip past the exegesis content in the lexer
        // We need to advance until we find the matching closing brace
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            self.advance();
        }

        if self.current.kind == TokenKind::RightBrace {
            self.advance();
        }

        Ok(content.trim().to_string())
    }

    // === Helper Methods ===

    /// Returns the source text (for exegesis parsing).
    fn lexer_source(&self) -> &'a str {
        self.source
    }

    /// Advances to the next token.
    fn advance(&mut self) {
        self.previous = std::mem::replace(
            &mut self.current,
            self.peeked
                .take()
                .unwrap_or_else(|| self.lexer.next_token()),
        );
    }

    /// Peeks at the next token without consuming it.
    fn peek(&mut self) -> &Token {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token());
        }
        self.peeked.as_ref().unwrap()
    }

    /// Expects the current token to be of a specific kind.
    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.current.kind == kind {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: kind.to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            })
        }
    }

    /// Expects an identifier and returns it.
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if self.current.kind == TokenKind::Identifier {
            let lexeme = self.current.lexeme.clone();
            self.advance();
            Ok(lexeme)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            })
        }
    }

    /// Expects a version and returns it.
    fn expect_version(&mut self) -> Result<String, ParseError> {
        if self.current.kind == TokenKind::Version {
            let lexeme = self.current.lexeme.clone();
            self.advance();
            Ok(lexeme)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "version number".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            })
        }
    }

    /// Expects a string and returns it.
    fn expect_string(&mut self) -> Result<String, ParseError> {
        if self.current.kind == TokenKind::String {
            let lexeme = self.current.lexeme.clone();
            self.advance();
            Ok(lexeme)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "string".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            })
        }
    }

    /// Checks if the next token is an identifier.
    fn peek_is_identifier(&self) -> bool {
        // Simple lookahead - would need proper implementation
        true
    }

    /// Checks if a version constraint follows.
    fn peek_is_version_constraint(&self) -> bool {
        // Simple lookahead - would need proper implementation
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gene() {
        let input = r#"
gene container.exists {
  container has identity
  container has state
}

exegesis {
  A container is fundamental.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        if let Declaration::Gene(gene) = result.unwrap() {
            assert_eq!(gene.name, "container.exists");
            assert_eq!(gene.statements.len(), 2);
        } else {
            panic!("Expected Gene");
        }
    }

    #[test]
    fn test_parse_trait() {
        let input = r#"
trait container.lifecycle {
  uses container.exists
  container is created
}

exegesis {
  Lifecycle management.
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_exegesis() {
        let input = r#"
gene container.exists {
  container has identity
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_err());

        if let Err(ParseError::MissingExegesis { .. }) = result {
            // Expected
        } else {
            panic!("Expected MissingExegesis error");
        }
    }
}
