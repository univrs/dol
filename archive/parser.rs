//! DOL Parser
//!
//! Recursive descent parser for DOL syntax.
//! Produces AST nodes with full span information for LLVM-style diagnostics.

use crate::ast::*;
use crate::lexer::{Token, SpannedToken, tokenize, LexError};
use std::iter::Peekable;
use std::vec::IntoIter;

/// Parse error with diagnostic information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub expected: Vec<String>,
    pub found: Option<String>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if !self.expected.is_empty() {
            write!(f, " (expected: {})", self.expected.join(", "))?;
        }
        if let Some(ref found) = self.found {
            write!(f, " (found: {})", found)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError {
            message: err.message,
            span: Span::new(err.span.start, err.span.end, 0, 0),
            expected: vec![],
            found: None,
        }
    }
}

/// Parser state
pub struct Parser {
    tokens: Peekable<IntoIter<SpannedToken>>,
    source: String,
    current_line: usize,
}

impl Parser {
    pub fn new(source: &str) -> Result<Self, ParseError> {
        let tokens = tokenize(source)?;
        Ok(Self {
            tokens: tokens.into_iter().peekable(),
            source: source.to_string(),
            current_line: 1,
        })
    }

    /// Parse a complete DOL file
    pub fn parse_file(&mut self) -> Result<DolFile, ParseError> {
        let start = self.current_span();
        let declaration = self.parse_declaration()?;
        let exegesis = self.parse_exegesis_optional()?;
        let end = self.current_span();

        Ok(DolFile {
            path: None,
            declaration,
            exegesis,
            span: start.merge(end),
        })
    }

    /// Parse the primary declaration
    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        let token = self.peek_token()?;
        match token {
            Token::Gene => self.parse_gene().map(Declaration::Gene),
            Token::Trait => self.parse_trait().map(Declaration::Trait),
            Token::Constraint => self.parse_constraint().map(Declaration::Constraint),
            Token::System => self.parse_system().map(Declaration::System),
            Token::Evolves => self.parse_evolves().map(Declaration::Evolves),
            Token::Test => self.parse_test().map(Declaration::Test),
            _ => Err(self.error(
                "Expected declaration (gene, trait, constraint, system, evolves, test)",
                vec!["gene", "trait", "constraint", "system", "evolves", "test"],
            )),
        }
    }

    /// Parse a gene declaration
    fn parse_gene(&mut self) -> Result<Gene, ParseError> {
        let start = self.current_span();
        self.expect(Token::Gene)?;
        let name = self.parse_qualified_name()?;
        self.expect(Token::LBrace)?;
        let statements = self.parse_statements()?;
        self.expect(Token::RBrace)?;

        Ok(Gene {
            name,
            statements,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a trait declaration
    fn parse_trait(&mut self) -> Result<Trait, ParseError> {
        let start = self.current_span();
        self.expect(Token::Trait)?;
        let name = self.parse_qualified_name()?;
        self.expect(Token::LBrace)?;

        // Parse uses clauses first
        let mut uses = Vec::new();
        while self.check(Token::Uses) {
            self.advance();
            uses.push(self.parse_qualified_name()?);
        }

        let statements = self.parse_statements()?;
        self.expect(Token::RBrace)?;

        Ok(Trait {
            name,
            uses,
            statements,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a constraint declaration
    fn parse_constraint(&mut self) -> Result<Constraint, ParseError> {
        let start = self.current_span();
        self.expect(Token::Constraint)?;
        let name = self.parse_qualified_name()?;
        self.expect(Token::LBrace)?;
        let statements = self.parse_statements()?;
        self.expect(Token::RBrace)?;

        Ok(Constraint {
            name,
            statements,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a system declaration
    fn parse_system(&mut self) -> Result<System, ParseError> {
        let start = self.current_span();
        self.expect(Token::System)?;
        let name = self.parse_qualified_name()?;
        self.expect(Token::At)?;
        let version = self.parse_version()?;
        self.expect(Token::LBrace)?;

        // Parse requires clauses
        let mut requires = Vec::new();
        while self.check(Token::Requires) {
            requires.push(self.parse_requirement()?);
        }

        let statements = self.parse_statements()?;
        self.expect(Token::RBrace)?;

        Ok(System {
            name,
            version,
            requires,
            statements,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse an evolves block
    fn parse_evolves(&mut self) -> Result<Evolves, ParseError> {
        let start = self.current_span();
        self.expect(Token::Evolves)?;
        let target = self.parse_qualified_name()?;
        self.expect(Token::At)?;
        let version = self.parse_version()?;
        self.expect(Token::Gt)?;
        let from = self.parse_version()?;
        self.expect(Token::LBrace)?;

        // Parse changes
        let mut changes = Vec::new();
        let mut because = None;
        let mut migration = None;

        while !self.check(Token::RBrace) {
            if self.check(Token::Adds) || self.check(Token::Deprecates) || self.check(Token::Removes) {
                changes.push(self.parse_change()?);
            } else if self.check(Token::Because) {
                self.advance();
                if let Token::String(s) = self.advance()?.token {
                    because = Some(s);
                }
            } else if self.check(Token::Migration) {
                migration = Some(self.parse_migration()?);
            } else {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Evolves {
            target,
            version,
            from,
            changes,
            because,
            migration,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a test declaration
    fn parse_test(&mut self) -> Result<Test, ParseError> {
        let start = self.current_span();
        self.expect(Token::Test)?;
        let name = self.parse_qualified_name()?;
        self.expect(Token::LBrace)?;

        let mut given = Vec::new();
        let mut when = Vec::new();
        let mut then = Vec::new();
        let mut always = false;

        while !self.check(Token::RBrace) {
            if self.check(Token::Given) {
                self.advance();
                given.push(self.parse_condition()?);
            } else if self.check(Token::When) {
                self.advance();
                when.push(self.parse_action()?);
            } else if self.check(Token::Then) {
                self.advance();
                then.push(self.parse_assertion()?);
            } else if self.check(Token::Always) {
                self.advance();
                always = true;
            } else {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Test {
            name,
            given,
            when,
            then,
            always,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse optional exegesis section
    fn parse_exegesis_optional(&mut self) -> Result<Option<Exegesis>, ParseError> {
        if !self.check(Token::Exegesis) {
            return Ok(None);
        }

        let start = self.current_span();
        self.expect(Token::Exegesis)?;
        self.expect(Token::LBrace)?;

        // Collect all content until closing brace
        let mut content = String::new();
        let mut depth = 1;
        
        // For exegesis, we need to capture raw text
        // This is a simplified approach - in production we'd use a separate lexer mode
        while let Some(token) = self.tokens.next() {
            match &token.token {
                Token::LBrace => depth += 1,
                Token::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            if depth > 0 {
                content.push_str(&token.slice);
                content.push(' ');
            }
        }

        Ok(Some(Exegesis::new(
            content.trim().to_string(),
            start.merge(self.current_span()),
        )))
    }

    /// Parse a qualified name (domain.property)
    fn parse_qualified_name(&mut self) -> Result<QualifiedName, ParseError> {
        let start = self.current_span();
        let mut segments = Vec::new();

        // First segment
        let first = self.expect_identifier()?;
        segments.push(first);

        // Additional segments
        while self.check(Token::Dot) {
            self.advance();
            let segment = self.expect_identifier()?;
            segments.push(segment);
        }

        Ok(QualifiedName::new(segments, start.merge(self.current_span())))
    }

    /// Parse a version number
    fn parse_version(&mut self) -> Result<Version, ParseError> {
        let span = self.current_span();
        let token = self.advance()?;
        
        if let Token::Version((major, minor, patch)) = token.token {
            Ok(Version::new(major, minor, patch, span))
        } else {
            Err(self.error("Expected version number", vec!["x.y.z"]))
        }
    }

    /// Parse a requirement clause
    fn parse_requirement(&mut self) -> Result<Requirement, ParseError> {
        let start = self.current_span();
        self.expect(Token::Requires)?;
        let name = self.parse_qualified_name()?;

        let operator = if self.check(Token::Gte) {
            self.advance();
            VersionOp::Gte
        } else if self.check(Token::Gt) {
            self.advance();
            VersionOp::Gt
        } else if self.check(Token::Lte) {
            self.advance();
            VersionOp::Lte
        } else if self.check(Token::Lt) {
            self.advance();
            VersionOp::Lt
        } else if self.check(Token::Eq) {
            self.advance();
            VersionOp::Eq
        } else {
            return Err(self.error("Expected version operator", vec![">=", ">", "<=", "<", "="]));
        };

        let version = self.parse_version()?;

        Ok(Requirement {
            name,
            operator,
            version,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse statements within a block
    fn parse_statements(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(Token::RBrace) && !self.is_at_end() {
            // Skip uses clauses (handled at trait level)
            if self.check(Token::Uses) {
                break;
            }

            if let Some(stmt) = self.try_parse_statement()? {
                statements.push(stmt);
            } else {
                break;
            }
        }

        Ok(statements)
    }

    /// Try to parse a statement
    fn try_parse_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let start = self.current_span();

        // Parse subject
        let subject = if self.check(Token::Each) {
            self.advance();
            let id = self.expect_identifier()?;
            Subject::Each(id)
        } else if self.check(Token::All) {
            self.advance();
            let id = self.expect_identifier()?;
            Subject::All(id)
        } else if self.check(Token::No) {
            self.advance();
            let id = self.expect_identifier()?;
            Subject::No(id)
        } else if let Some(token) = self.tokens.peek() {
            if token.token == Token::Identifier {
                let id = self.expect_identifier()?;
                Subject::Identifier(id)
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None);
        };

        // Parse predicate
        let predicate = self.parse_predicate()?;

        // Parse object (rest of the line)
        let object = self.parse_object()?;

        Ok(Some(Statement {
            subject,
            predicate,
            object,
            span: start.merge(self.current_span()),
        }))
    }

    /// Parse a predicate
    fn parse_predicate(&mut self) -> Result<Predicate, ParseError> {
        let token = self.peek_token()?;
        let predicate = match token {
            Token::Has => { self.advance(); Predicate::Has }
            Token::Is => { self.advance(); Predicate::Is }
            Token::Are => { self.advance(); Predicate::Are }
            Token::Derives => {
                self.advance();
                self.expect(Token::From)?;
                Predicate::DerivesFrom
            }
            Token::Requires => { self.advance(); Predicate::Requires }
            Token::Emits => { self.advance(); Predicate::Emits }
            Token::Matches => { self.advance(); Predicate::Matches }
            Token::Never => { self.advance(); Predicate::Never }
            Token::Via => { self.advance(); Predicate::Via }
            _ => {
                return Err(self.error(
                    "Expected predicate",
                    vec!["has", "is", "are", "derives from", "requires", "emits", "matches", "never"],
                ));
            }
        };
        Ok(predicate)
    }

    /// Parse an object (phrase or identifier)
    fn parse_object(&mut self) -> Result<Option<Object>, ParseError> {
        let mut words = Vec::new();

        // Collect words until end of statement
        while let Some(token) = self.tokens.peek() {
            match &token.token {
                Token::Identifier => {
                    let word = token.slice.clone();
                    self.advance();
                    words.push(word);
                }
                Token::No => {
                    self.advance();
                    let inner = self.parse_object()?;
                    if let Some(obj) = inner {
                        return Ok(Some(Object::Negated(Box::new(obj))));
                    }
                }
                Token::Dot => {
                    // Part of qualified name
                    if words.len() == 1 {
                        self.advance();
                        let rest = self.expect_identifier()?;
                        return Ok(Some(Object::QualifiedName(QualifiedName::new(
                            vec![words.remove(0), rest],
                            Span::default(),
                        ))));
                    }
                    break;
                }
                _ => break,
            }
        }

        if words.is_empty() {
            Ok(None)
        } else if words.len() == 1 {
            Ok(Some(Object::Identifier(words.remove(0))))
        } else {
            Ok(Some(Object::Phrase(words)))
        }
    }

    /// Parse a change in an evolves block
    fn parse_change(&mut self) -> Result<Change, ParseError> {
        let start = self.current_span();
        
        let kind = if self.check(Token::Adds) {
            self.advance();
            ChangeKind::Adds
        } else if self.check(Token::Deprecates) {
            self.advance();
            ChangeKind::Deprecates
        } else if self.check(Token::Removes) {
            self.advance();
            ChangeKind::Removes
        } else {
            return Err(self.error("Expected change kind", vec!["adds", "deprecates", "removes"]));
        };

        let statement = self.try_parse_statement()?.ok_or_else(|| {
            self.error("Expected statement after change kind", vec![])
        })?;

        Ok(Change {
            kind,
            statement,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a migration block
    fn parse_migration(&mut self) -> Result<Migration, ParseError> {
        let start = self.current_span();
        self.expect(Token::Migration)?;
        self.expect(Token::LBrace)?;

        let mut mappings = Vec::new();

        while !self.check(Token::RBrace) {
            let mapping_start = self.current_span();
            let from = self.expect_identifier()?;
            self.expect(Token::Maps)?;
            self.expect(Token::To)?;

            let mut to = Vec::new();
            to.push(self.expect_identifier()?);

            while self.check(Token::Then) {
                self.advance();
                to.push(self.expect_identifier()?);
            }

            mappings.push(MigrationMapping {
                from,
                to,
                span: mapping_start.merge(self.current_span()),
            });
        }

        self.expect(Token::RBrace)?;

        Ok(Migration {
            mappings,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a test condition
    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        let start = self.current_span();
        let negated = self.check(Token::No);
        if negated {
            self.advance();
        }

        let subject = self.expect_identifier()?;
        let state = if self.check(Token::Identifier) {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        Ok(Condition {
            negated,
            subject,
            state,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a test action
    fn parse_action(&mut self) -> Result<Action, ParseError> {
        let start = self.current_span();
        let subject = self.expect_identifier()?;
        let verb = self.expect_identifier()?;
        let object = if self.check(Token::Identifier) {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        Ok(Action {
            subject,
            verb,
            object,
            span: start.merge(self.current_span()),
        })
    }

    /// Parse a test assertion
    fn parse_assertion(&mut self) -> Result<Assertion, ParseError> {
        let start = self.current_span();
        let subject = self.expect_identifier()?;
        let predicate = self.expect_identifier()?;
        let object = if self.check(Token::Identifier) {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        Ok(Assertion {
            subject,
            predicate,
            object,
            span: start.merge(self.current_span()),
        })
    }

    // ═══════════════════════════════════════════
    // Helper methods
    // ═══════════════════════════════════════════

    fn peek_token(&mut self) -> Result<Token, ParseError> {
        self.tokens
            .peek()
            .map(|t| t.token.clone())
            .ok_or_else(|| self.error("Unexpected end of input", vec![]))
    }

    fn check(&mut self, expected: Token) -> bool {
        self.tokens
            .peek()
            .map(|t| std::mem::discriminant(&t.token) == std::mem::discriminant(&expected))
            .unwrap_or(false)
    }

    fn advance(&mut self) -> Result<SpannedToken, ParseError> {
        self.tokens
            .next()
            .ok_or_else(|| self.error("Unexpected end of input", vec![]))
    }

    fn expect(&mut self, expected: Token) -> Result<SpannedToken, ParseError> {
        let token = self.advance()?;
        if std::mem::discriminant(&token.token) == std::mem::discriminant(&expected) {
            Ok(token)
        } else {
            Err(ParseError {
                message: format!("Expected {}", expected),
                span: Span::new(token.span.start, token.span.end, 0, 0),
                expected: vec![expected.to_string()],
                found: Some(token.slice),
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.advance()?;
        if token.token == Token::Identifier {
            Ok(token.slice)
        } else {
            Err(ParseError {
                message: "Expected identifier".to_string(),
                span: Span::new(token.span.start, token.span.end, 0, 0),
                expected: vec!["identifier".to_string()],
                found: Some(token.slice),
            })
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.tokens.peek().is_none()
    }

    fn current_span(&self) -> Span {
        Span::default()
    }

    fn error(&self, message: &str, expected: Vec<&str>) -> ParseError {
        ParseError {
            message: message.to_string(),
            span: Span::default(),
            expected: expected.into_iter().map(|s| s.to_string()).collect(),
            found: None,
        }
    }
}

/// Parse a DOL source string
pub fn parse(source: &str) -> Result<DolFile, ParseError> {
    let mut parser = Parser::new(source)?;
    parser.parse_file()
}

/// Parse multiple DOL files into a repository
pub fn parse_repository(files: Vec<(&str, &str)>) -> Result<DolRepository, Vec<ParseError>> {
    let mut repo = DolRepository::new();
    let mut errors = Vec::new();

    for (path, source) in files {
        match parse(source) {
            Ok(mut file) => {
                file.path = Some(path.to_string());
                repo.add_file(file);
            }
            Err(e) => errors.push(e),
        }
    }

    if errors.is_empty() {
        Ok(repo)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_gene() {
        let source = r#"
            gene container.exists {
                container has identity
                container has state
                container has boundaries
            }
        "#;

        let file = parse(source).unwrap();
        if let Declaration::Gene(gene) = file.declaration {
            assert_eq!(gene.name.to_string(), "container.exists");
            assert_eq!(gene.statements.len(), 3);
        } else {
            panic!("Expected gene declaration");
        }
    }

    #[test]
    fn parse_trait_with_uses() {
        let source = r#"
            trait container.lifecycle {
                uses container.exists
                uses identity.cryptographic

                container is created
                container is started
                container is stopped
            }
        "#;

        let file = parse(source).unwrap();
        if let Declaration::Trait(t) = file.declaration {
            assert_eq!(t.uses.len(), 2);
            assert_eq!(t.statements.len(), 3);
        } else {
            panic!("Expected trait declaration");
        }
    }

    #[test]
    fn parse_evolves_with_lineage() {
        let source = r#"
            evolves container.lifecycle @ 0.0.2 > 0.0.1 {
                adds container is paused
                adds container is resumed
                
                because "migration requires state preservation"
            }
        "#;

        let file = parse(source).unwrap();
        if let Declaration::Evolves(e) = file.declaration {
            assert_eq!(e.version.to_string(), "0.0.2");
            assert_eq!(e.from.to_string(), "0.0.1");
            assert_eq!(e.changes.len(), 2);
            assert!(e.because.is_some());
        } else {
            panic!("Expected evolves declaration");
        }
    }

    #[test]
    fn parse_with_exegesis() {
        let source = r#"
            gene identity.cryptographic {
                identity derives from ed25519 keypair
                identity is self sovereign
                identity requires no authority
            }

            exegesis {
                Cryptographic identity is the foundation of trust.
                Every entity possesses an Ed25519 keypair.
            }
        "#;

        let file = parse(source).unwrap();
        assert!(file.exegesis.is_some());
    }
}
