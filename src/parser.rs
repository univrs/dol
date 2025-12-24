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
use crate::macros::{AttributeArg, MacroAttribute, MacroInvocation};
use crate::pratt::{infix_binding_power, prefix_binding_power};

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
        // Skip module declaration if present
        self.skip_module_and_uses()?;

        let decl = self.parse_declaration()?;

        // Allow trailing content (other declarations in the file)
        // For now, we just return the first declaration
        Ok(decl)
    }

    /// Skips module declaration and use statements at the start of a file.
    fn skip_module_and_uses(&mut self) -> Result<(), ParseError> {
        // Skip module declaration
        if self.current.kind == TokenKind::Module {
            self.advance(); // module
                            // Skip path
            while self.current.kind == TokenKind::Identifier || self.current.kind == TokenKind::Dot
            {
                self.advance();
            }
            // Skip version
            if self.current.kind == TokenKind::At {
                self.advance();
                if self.current.kind == TokenKind::Version {
                    self.advance();
                }
            }
        }

        // Skip use declarations (including pub use)
        loop {
            // Skip optional pub modifier
            if self.current.kind == TokenKind::Pub {
                if self.peek().kind == TokenKind::Use {
                    self.advance(); // pub
                } else {
                    // pub followed by something else (like pub gene) - stop skipping
                    break;
                }
            }

            if self.current.kind != TokenKind::Use {
                break;
            }

            self.advance(); // use
                            // Skip path (identifiers, ::, ., *, etc.)
            while self.current.kind != TokenKind::Eof
                && self.current.kind != TokenKind::Gene
                && self.current.kind != TokenKind::Trait
                && self.current.kind != TokenKind::Constraint
                && self.current.kind != TokenKind::System
                && self.current.kind != TokenKind::Evolves
                && self.current.kind != TokenKind::Pub
                && self.current.kind != TokenKind::Use
                && self.current.kind != TokenKind::Module
                && self.current.kind != TokenKind::Exegesis
            {
                self.advance();
            }
        }

        Ok(())
    }

    /// Skips generic type parameters: <T, U: Bound, V = Default>
    fn skip_type_params(&mut self) -> Result<(), ParseError> {
        if self.current.kind != TokenKind::Lt {
            return Ok(());
        }

        self.advance(); // consume <

        let mut depth = 1;
        while depth > 0 && self.current.kind != TokenKind::Eof {
            match self.current.kind {
                TokenKind::Lt => depth += 1,
                TokenKind::Greater => depth -= 1,
                _ => {}
            }
            self.advance();
        }

        Ok(())
    }

    /// Skips a type expression (handles simple types and complex ones like `enum { ... }`).
    fn skip_type_expr(&mut self) -> Result<(), ParseError> {
        // Handle enum keyword with brace block
        if self.current.kind == TokenKind::Identifier && self.current.lexeme == "enum" {
            self.advance(); // consume 'enum'
            if self.current.kind == TokenKind::LeftBrace {
                self.advance(); // consume '{'
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftBrace => depth += 1,
                        TokenKind::RightBrace => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            return Ok(());
        }

        // Regular type expression: consume identifier and optional generics
        if self.current.kind == TokenKind::Identifier {
            self.advance();
        } else if self.current.kind == TokenKind::LeftBracket {
            // Array type: [Type] or [Type; size]
            self.advance();
            let mut depth = 1;
            while depth > 0 && self.current.kind != TokenKind::Eof {
                match self.current.kind {
                    TokenKind::LeftBracket => depth += 1,
                    TokenKind::RightBracket => depth -= 1,
                    _ => {}
                }
                self.advance();
            }
            return Ok(());
        }

        // Skip generic parameters: <T, U>
        self.skip_type_params()?;

        Ok(())
    }

    /// Parses a declaration.
    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        // Skip visibility modifier
        if self.current.kind == TokenKind::Pub {
            self.advance();
            // Skip optional (spirit) or (parent)
            if self.current.kind == TokenKind::LeftParen {
                self.advance(); // (
                self.advance(); // spirit/parent
                if self.current.kind == TokenKind::RightParen {
                    self.advance(); // )
                }
            }
        }

        match self.current.kind {
            TokenKind::Gene => self.parse_gene(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Constraint => self.parse_constraint(),
            TokenKind::System => self.parse_system(),
            TokenKind::Evolves => self.parse_evolution(),
            TokenKind::Exegesis => {
                // Skip file-level exegesis block
                self.advance(); // consume 'exegesis'
                self.expect(TokenKind::LeftBrace)?;
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    if self.current.kind == TokenKind::LeftBrace {
                        depth += 1;
                    }
                    if self.current.kind == TokenKind::RightBrace {
                        depth -= 1;
                    }
                    self.advance();
                }
                // Try to parse next declaration, or return placeholder if EOF
                if self.current.kind == TokenKind::Eof {
                    Ok(Declaration::Gene(Gene {
                        name: "_module_doc".to_string(),
                        statements: vec![],
                        exegesis: "Module-level documentation".to_string(),
                        span: self.current.span,
                    }))
                } else {
                    self.parse_declaration()
                }
            }
            _ => Err(ParseError::InvalidDeclaration {
                found: self.current.lexeme.clone(),
                span: self.current.span,
            }),
        }
    }

    /// Parses an optional visibility modifier.
    /// Returns Visibility::Private if no modifier is present.
    #[allow(dead_code)]
    fn parse_visibility(&mut self) -> Result<Visibility, ParseError> {
        match self.current.kind {
            TokenKind::Pub => {
                self.advance();
                // Check for pub(spirit) or pub(parent)
                if self.current.kind == TokenKind::LeftParen {
                    self.advance();
                    if self.current.kind == TokenKind::Spirit {
                        self.advance();
                        self.expect(TokenKind::RightParen)?;
                        Ok(Visibility::PubSpirit)
                    } else if self.current.lexeme == "parent" {
                        self.advance();
                        self.expect(TokenKind::RightParen)?;
                        Ok(Visibility::PubParent)
                    } else {
                        Err(ParseError::UnexpectedToken {
                            expected: "spirit or parent".to_string(),
                            found: format!("'{}'", self.current.lexeme),
                            span: self.current.span,
                        })
                    }
                } else {
                    Ok(Visibility::Public)
                }
            }
            _ => Ok(Visibility::Private),
        }
    }

    /// Parses a module declaration: module path.to.module @ version
    #[allow(dead_code)]
    fn parse_module_decl(&mut self) -> Result<ModuleDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Module)?;

        // Parse module path (e.g., "univrs.container.lifecycle")
        let mut path = Vec::new();
        path.push(self.expect_identifier()?);

        while self.current.kind == TokenKind::Dot {
            self.advance();
            path.push(self.expect_identifier()?);
        }

        // Parse optional version
        let version = if self.current.kind == TokenKind::At {
            self.advance();
            Some(self.parse_version()?)
        } else {
            None
        };

        let span = start_span.merge(&self.previous.span);

        Ok(ModuleDecl {
            path,
            version,
            span,
        })
    }

    /// Parses a version: 1.2.3 or 1.2.3-alpha
    fn parse_version(&mut self) -> Result<Version, ParseError> {
        let version_str = self.expect_version()?;
        // Parse the version string into components
        let mut parts = version_str.splitn(2, '-');
        let numbers_part = parts.next().unwrap();
        let suffix = parts.next().map(|s| s.to_string());

        let numbers: Vec<&str> = numbers_part.split('.').collect();
        if numbers.len() != 3 {
            return Err(ParseError::InvalidStatement {
                message: "version must have three parts".to_string(),
                span: self.previous.span,
            });
        }

        Ok(Version {
            major: numbers[0].parse().unwrap_or(0),
            minor: numbers[1].parse().unwrap_or(0),
            patch: numbers[2].parse().unwrap_or(0),
            suffix,
        })
    }

    /// Parses a use declaration: use path::to::module::{items}
    #[allow(dead_code)]
    fn parse_use_decl(&mut self) -> Result<UseDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Use)?;

        // Parse path with :: separators
        let mut path = Vec::new();
        path.push(self.expect_identifier()?);

        while self.current.kind == TokenKind::PathSep {
            self.advance();
            if self.current.kind == TokenKind::LeftBrace {
                break; // Items list
            }
            if self.current.kind == TokenKind::Star {
                break; // Glob import
            }
            path.push(self.expect_identifier()?);
        }

        // Parse items
        let items = if self.current.kind == TokenKind::Star {
            self.advance();
            UseItems::All
        } else if self.current.kind == TokenKind::LeftBrace {
            self.advance();
            let mut items = Vec::new();
            while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof
            {
                let name = self.expect_identifier()?;
                let alias = if self.current.kind == TokenKind::As {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                items.push(UseItem { name, alias });
                if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RightBrace)?;
            UseItems::Named(items)
        } else {
            UseItems::Single
        };

        // Parse optional alias
        let alias = if self.current.kind == TokenKind::As {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        let span = start_span.merge(&self.previous.span);

        Ok(UseDecl {
            path,
            items,
            alias,
            span,
        })
    }
    /// Parses a gene declaration.
    fn parse_gene(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Gene)?;

        let name = self.expect_identifier()?;
        // Skip generic type parameters if present: <T, U: Bound>
        self.skip_type_params()?;
        self.expect(TokenKind::LeftBrace)?;

        let statements = self.parse_statements()?;

        // DOL 2.0: exegesis can be inside braces
        let inline_exegesis = self.parse_inline_exegesis()?;

        self.expect(TokenKind::RightBrace)?;

        // DOL 1.0: exegesis can be after braces
        // DOL 2.0: use inline or default to empty
        let exegesis = if let Some(ex) = inline_exegesis {
            ex
        } else if self.current.kind == TokenKind::Exegesis {
            self.parse_exegesis()?
        } else {
            String::new() // DOL 2.0 tolerant: empty exegesis if none
        };

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
        // Skip generic type parameters if present
        self.skip_type_params()?;
        self.expect(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();
        let mut _laws: Vec<LawDecl> = Vec::new();

        while self.current.kind != TokenKind::RightBrace
            && self.current.kind != TokenKind::Eof
            && self.current.kind != TokenKind::Exegesis
        {
            // Check for law declarations
            if self.current.kind == TokenKind::Law {
                let law = self.parse_law_decl()?;
                _laws.push(law);
            } else {
                statements.push(self.parse_statement()?);
            }
        }

        // DOL 2.0: exegesis can be inside braces
        let inline_exegesis = self.parse_inline_exegesis()?;

        self.expect(TokenKind::RightBrace)?;

        // DOL 1.0: exegesis can be after braces
        let exegesis = if let Some(ex) = inline_exegesis {
            ex
        } else if self.current.kind == TokenKind::Exegesis {
            self.parse_exegesis()?
        } else {
            String::new()
        };

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
        // Skip generic type parameters if present
        self.skip_type_params()?;
        self.expect(TokenKind::LeftBrace)?;

        let statements = self.parse_statements()?;

        // DOL 2.0: exegesis can be inside braces
        let inline_exegesis = self.parse_inline_exegesis()?;

        self.expect(TokenKind::RightBrace)?;

        // DOL 1.0: exegesis can be after braces
        let exegesis = if let Some(ex) = inline_exegesis {
            ex
        } else if self.current.kind == TokenKind::Exegesis {
            self.parse_exegesis()?
        } else {
            String::new()
        };

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
        // Skip generic type parameters if present
        self.skip_type_params()?;

        // DOL 2.0: version is optional
        let version = if self.current.kind == TokenKind::At {
            self.advance();
            self.expect_version()?
        } else {
            "0.0.0".to_string()
        };

        self.expect(TokenKind::LeftBrace)?;

        let mut requirements = Vec::new();
        let mut statements = Vec::new();
        let mut _states: Vec<StateDecl> = Vec::new(); // States for future use

        while self.current.kind != TokenKind::RightBrace
            && self.current.kind != TokenKind::Eof
            && self.current.kind != TokenKind::Exegesis
        {
            if self.current.kind == TokenKind::Requires
                && self.peek_is_identifier()
                && self.peek_is_version_constraint()
            {
                requirements.push(self.parse_requirement()?);
            } else if self.current.kind == TokenKind::State {
                // Parse state declaration
                let state = self.parse_state_decl()?;
                _states.push(state); // Store in local vector for future use
            } else {
                statements.push(self.parse_statement()?);
            }
        }

        // DOL 2.0: exegesis can be inside braces
        let inline_exegesis = self.parse_inline_exegesis()?;

        self.expect(TokenKind::RightBrace)?;

        // DOL 1.0: exegesis can be after braces
        let exegesis = if let Some(ex) = inline_exegesis {
            ex
        } else if self.current.kind == TokenKind::Exegesis {
            self.parse_exegesis()?
        } else {
            String::new()
        };

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
        let mut _migrate: Option<Vec<Stmt>> = None;

        while self.current.kind != TokenKind::RightBrace
            && self.current.kind != TokenKind::Eof
            && self.current.kind != TokenKind::Exegesis
        {
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
                TokenKind::Migrate => {
                    _migrate = Some(self.parse_migrate_block()?);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "adds, deprecates, removes, migrate, or because".to_string(),
                        found: format!("'{}'", self.current.lexeme),
                        span: self.current.span,
                    });
                }
            }
        }

        // DOL 2.0: exegesis can be inside braces
        let inline_exegesis = self.parse_inline_exegesis()?;

        self.expect(TokenKind::RightBrace)?;

        // DOL 1.0: exegesis can be after braces
        let exegesis = if let Some(ex) = inline_exegesis {
            ex
        } else if self.current.kind == TokenKind::Exegesis {
            self.parse_exegesis()?
        } else {
            String::new()
        };

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

        // Stop at RightBrace, Eof, or Exegesis (DOL 2.0 puts exegesis inside declaration braces)
        while self.current.kind != TokenKind::RightBrace
            && self.current.kind != TokenKind::Eof
            && self.current.kind != TokenKind::Exegesis
        {
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

        // Handle DOL 2.0 'has' field declarations: has name: Type [= default]
        if self.current.kind == TokenKind::Has {
            self.advance();
            let name = self.expect_identifier()?;
            // Skip type annotation: Type
            if self.current.kind == TokenKind::Colon {
                self.advance();
                self.parse_type()?;
            }
            // Skip default value: = expr
            if self.current.kind == TokenKind::Equal {
                self.advance();
                self.parse_expr(0)?;
            }
            return Ok(Statement::Has {
                subject: "self".to_string(),
                property: name,
                span: start_span.merge(&self.previous.span),
            });
        }

        // Handle DOL 2.0 inline 'constraint' blocks inside declarations
        if self.current.kind == TokenKind::Constraint {
            self.advance();
            let name = self.expect_identifier()?;
            // Skip constraint body: { ... }
            if self.current.kind == TokenKind::LeftBrace {
                self.advance();
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftBrace => depth += 1,
                        TokenKind::RightBrace => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            return Ok(Statement::Requires {
                subject: "self".to_string(),
                requirement: name,
                span: start_span.merge(&self.previous.span),
            });
        }

        // Handle DOL 2.0 'fun' function declarations inside genes
        if self.current.kind == TokenKind::Function {
            self.advance();
            let name = self.expect_identifier()?;
            // Skip function params and body
            if self.current.kind == TokenKind::LeftParen {
                self.advance();
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftParen => depth += 1,
                        TokenKind::RightParen => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            // Skip return type
            if self.current.kind == TokenKind::Arrow {
                self.advance();
                self.parse_type()?;
            }
            // Skip function body
            if self.current.kind == TokenKind::LeftBrace {
                self.advance();
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftBrace => depth += 1,
                        TokenKind::RightBrace => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            return Ok(Statement::Has {
                subject: "self".to_string(),
                property: name,
                span: start_span.merge(&self.previous.span),
            });
        }

        // Handle DOL 2.0 'law' declarations
        if self.current.kind == TokenKind::Law {
            self.advance();
            let name = self.expect_identifier()?;
            // Skip law params
            if self.current.kind == TokenKind::LeftParen {
                self.advance();
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftParen => depth += 1,
                        TokenKind::RightParen => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            // Skip law body
            if self.current.kind == TokenKind::LeftBrace {
                self.advance();
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftBrace => depth += 1,
                        TokenKind::RightBrace => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            return Ok(Statement::Requires {
                subject: "self".to_string(),
                requirement: name,
                span: start_span.merge(&self.previous.span),
            });
        }

        // Handle visibility modifiers (pub, pub(spirit), etc.)
        if self.current.kind == TokenKind::Pub {
            self.advance();
            // Skip pub(...) if present
            if self.current.kind == TokenKind::LeftParen {
                self.advance();
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftParen => depth += 1,
                        TokenKind::RightParen => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }
            // Continue to parse the actual statement
            return self.parse_statement();
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
            // DOL 2.0: name: Type field syntax (without 'has' keyword)
            TokenKind::Colon => {
                self.advance(); // consume ':'
                                // Skip type expression (handles complex types like enum { ... })
                self.skip_type_expr()?;
                // Skip default value if present
                if self.current.kind == TokenKind::Equal {
                    self.advance();
                    self.parse_expr(0)?;
                }
                Ok(Statement::Has {
                    subject: "self".to_string(),
                    property: subject,
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

    /// Parses a has statement with optional default value and constraint.
    /// Syntax: subject has property [: Type] [= default] [where constraint]
    /// Returns a Statement::Has with extended information
    pub fn parse_has_statement(
        &mut self,
        subject: String,
        start_span: Span,
    ) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Has)?;
        let property = self.expect_identifier()?;

        // Check for HasField with type, default, and constraint
        // This is for DOL 2.0 extended has syntax
        // For now, just return the simple Has statement
        // Extended parsing can be added when needed

        Ok(Statement::Has {
            subject,
            property,
            span: start_span.merge(&self.previous.span),
        })
    }

    /// Parses a has field declaration in a gene body.
    /// Syntax: subject has property: Type [= default] [where constraint]
    pub fn parse_has_field(&mut self) -> Result<HasField, ParseError> {
        let start_span = self.current.span;

        let name = self.expect_identifier()?;
        self.expect(TokenKind::Has)?;
        let _property = self.expect_identifier()?; // "property" part becomes part of name

        // Parse optional type
        let type_ = if self.current.kind == TokenKind::Colon {
            self.advance();
            self.parse_type()?
        } else {
            TypeExpr::Named("Any".to_string())
        };

        // Parse optional default
        let default = if self.current.kind == TokenKind::Equal {
            self.advance();
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        // Parse optional constraint
        let constraint = if self.current.kind == TokenKind::Where {
            self.advance();
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        Ok(HasField {
            name,
            type_,
            default,
            constraint,
            span: start_span.merge(&self.previous.span),
        })
    }

    /// Parses a state declaration in a system.
    /// Syntax: state name: Type [= default]
    pub fn parse_state_decl(&mut self) -> Result<StateDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::State)?;

        let name = self.expect_identifier()?;

        // Type is required for state
        self.expect(TokenKind::Colon)?;
        let type_ = self.parse_type()?;

        // Parse optional default
        let default = if self.current.kind == TokenKind::Equal {
            self.advance();
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        Ok(StateDecl {
            name,
            type_,
            default,
            span: start_span.merge(&self.previous.span),
        })
    }

    /// Parses a variable declaration: var/const name: Type [= value]
    /// Used for sex var and const declarations.
    pub fn parse_var_decl(&mut self, mutability: Mutability) -> Result<VarDecl, ParseError> {
        let start_span = self.current.span;

        let name = self.expect_identifier()?;

        // Parse optional type annotation
        let type_ann = if self.current.kind == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse optional value
        let value = if self.current.kind == TokenKind::Equal {
            self.advance();
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        Ok(VarDecl {
            mutability,
            name,
            type_ann,
            value,
            span: start_span.merge(&self.previous.span),
        })
    }

    /// Parses a sex var declaration: sex var name: Type [= value]
    pub fn parse_sex_var(&mut self) -> Result<VarDecl, ParseError> {
        self.expect(TokenKind::Sex)?;
        self.expect(TokenKind::Var)?;
        self.parse_var_decl(Mutability::Mutable)
    }

    /// Parses a const declaration: const name: Type = value
    pub fn parse_const(&mut self) -> Result<VarDecl, ParseError> {
        self.expect(TokenKind::Const)?;
        self.parse_var_decl(Mutability::Immutable)
    }

    /// Parses an extern function declaration: sex extern [abi] fun name(...) -> Type
    pub fn parse_sex_extern(&mut self) -> Result<ExternDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Sex)?;
        self.expect(TokenKind::Extern)?;

        // Parse optional ABI
        let abi = if self.current.kind == TokenKind::String {
            Some(self.expect_string()?)
        } else {
            None
        };

        self.expect(TokenKind::Function)?;

        let name = self.expect_identifier()?;

        // Parse parameters
        self.expect(TokenKind::LeftParen)?;
        let mut params = Vec::new();
        while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof {
            let param_name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let type_ann = self.parse_type()?;
            params.push(FunctionParam {
                name: param_name,
                type_ann,
            });

            if self.current.kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightParen)?;

        // Parse optional return type
        let return_type = if self.current.kind == TokenKind::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(ExternDecl {
            abi,
            name,
            params,
            return_type,
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

    /// Parses an optional inline exegesis block (DOL 2.0 style).
    /// Returns None if no exegesis is present.
    fn parse_inline_exegesis(&mut self) -> Result<Option<String>, ParseError> {
        if self.current.kind != TokenKind::Exegesis {
            return Ok(None);
        }

        self.advance(); // consume 'exegesis'
        self.expect(TokenKind::LeftBrace)?;

        // Collect all text until closing brace
        let mut content = String::new();
        let mut brace_depth = 1;

        let start_pos = self.current.span.start;
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
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            self.advance();
        }

        if self.current.kind == TokenKind::RightBrace {
            self.advance();
        }

        Ok(Some(content.trim().to_string()))
    }

    // === DOL 2.0 Expression Parsing ===

    /// Parses an expression using Pratt parsing for operator precedence.
    ///
    /// # Arguments
    ///
    /// * `min_bp` - Minimum binding power for this expression context
    ///
    /// # Returns
    ///
    /// The parsed expression on success, or a ParseError on failure.
    pub fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        // Parse prefix or atom
        let mut lhs = self.parse_prefix_or_atom()?;

        // Parse infix operators with binding power
        loop {
            // Check for infix operators
            if let Some((left_bp, right_bp)) = infix_binding_power(&self.current.kind) {
                if left_bp < min_bp {
                    break;
                }

                let op = self.current.kind;
                self.advance();

                let rhs = self.parse_expr(right_bp)?;
                lhs = self.make_binary_expr(lhs, op, rhs)?;
            } else if self.current.kind == TokenKind::LeftParen {
                // Function call
                self.advance();
                let mut args = Vec::new();
                while self.current.kind != TokenKind::RightParen
                    && self.current.kind != TokenKind::Eof
                {
                    args.push(self.parse_expr(0)?);
                    if self.current.kind == TokenKind::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::RightParen)?;
                lhs = Expr::Call {
                    callee: Box::new(lhs),
                    args,
                };
            } else if self.current.kind == TokenKind::LeftBracket {
                // Array indexing (parsed as function call for now)
                self.advance();
                let index = self.parse_expr(0)?;
                self.expect(TokenKind::RightBracket)?;
                lhs = Expr::Call {
                    callee: Box::new(lhs),
                    args: vec![index],
                };
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    /// Parses prefix operators and atomic expressions.
    fn parse_prefix_or_atom(&mut self) -> Result<Expr, ParseError> {
        // Special case for Bang: check if it's eval (!{...}) or logical not (!expr)
        if self.current.kind == TokenKind::Bang {
            self.advance();
            if self.current.kind == TokenKind::LeftBrace {
                // Eval: !{ expr }
                self.advance();
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RightBrace)?;
                return Ok(Expr::Eval(Box::new(expr)));
            } else {
                // Logical not: !expr
                let bp = prefix_binding_power(&TokenKind::Bang).unwrap();
                let operand = self.parse_expr(bp)?;
                return Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                });
            }
        }

        // Check for QuasiQuote (double quote: ''expr)
        if self.current.kind == TokenKind::Quote {
            self.advance();
            // Check if the next token is also a quote
            if self.current.kind == TokenKind::Quote {
                self.advance();
                // This is a quasi-quote
                let bp = prefix_binding_power(&TokenKind::Quote).unwrap();
                let operand = self.parse_expr(bp)?;
                return Ok(Expr::QuasiQuote(Box::new(operand)));
            } else {
                // Single quote - regular quote
                let bp = prefix_binding_power(&TokenKind::Quote).unwrap();
                let operand = self.parse_expr(bp)?;
                return Ok(Expr::Quote(Box::new(operand)));
            }
        }

        // Check for other prefix operators
        if let Some(bp) = prefix_binding_power(&self.current.kind) {
            let op = self.current.kind;
            self.advance();

            let operand = self.parse_expr(bp)?;
            return self.make_unary_expr(op, operand);
        }

        // Parse atoms
        match self.current.kind {
            // Literals
            TokenKind::String => {
                let value = self.current.lexeme.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(value)))
            }
            TokenKind::Identifier => {
                let name = self.current.lexeme.clone();
                self.advance();

                // Check for special boolean literals
                if name == "true" {
                    return Ok(Expr::Literal(Literal::Bool(true)));
                } else if name == "false" {
                    return Ok(Expr::Literal(Literal::Bool(false)));
                }

                Ok(Expr::Identifier(name))
            }

            // Parenthesized expression
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RightParen)?;
                Ok(expr)
            }

            // Lambda expression: |params| body
            TokenKind::Bar => self.parse_lambda(),

            // If expression
            TokenKind::If => self.parse_if_expr(),

            // Match expression
            TokenKind::Match => self.parse_match_expr(),

            // Block expression
            TokenKind::LeftBrace => self.parse_block_expr(),

            // Sex block expression
            TokenKind::Sex => self.parse_sex_block(),

            // Eval or logical not: handled by prefix operators
            // Type reflection: ?TypeName
            TokenKind::Reflect => {
                self.advance();
                let type_expr = self.parse_type()?;
                Ok(Expr::Reflect(Box::new(type_expr)))
            }

            // Macro invocation: #macro_name(args)
            TokenKind::Macro => self.parse_macro_invocation_expr(),

            // Idiom brackets: [| f a b |]
            TokenKind::IdiomOpen => self.parse_idiom_bracket(),

            // Boolean literals
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }

            // Null literal
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }

            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            }),
        }
    }

    /// Creates a binary expression from operator token.
    fn make_binary_expr(
        &self,
        left: Expr,
        op_token: TokenKind,
        right: Expr,
    ) -> Result<Expr, ParseError> {
        let op = match op_token {
            TokenKind::Plus => BinaryOp::Add,
            TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Star => BinaryOp::Mul,
            TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Mod,
            TokenKind::Caret => BinaryOp::Pow,
            TokenKind::Eq => BinaryOp::Eq,
            TokenKind::Ne => BinaryOp::Ne,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Greater => BinaryOp::Gt,
            TokenKind::GreaterEqual => BinaryOp::Ge,
            TokenKind::And => BinaryOp::And,
            TokenKind::Or => BinaryOp::Or,
            TokenKind::Pipe => BinaryOp::Pipe,
            TokenKind::Compose => BinaryOp::Compose,
            TokenKind::At => BinaryOp::Apply,
            TokenKind::Bind => BinaryOp::Bind,
            TokenKind::Dot => BinaryOp::Member,
            _ => {
                return Err(ParseError::InvalidStatement {
                    message: format!("invalid binary operator: {:?}", op_token),
                    span: self.current.span,
                })
            }
        };

        Ok(Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    /// Creates a unary expression from operator token.
    fn make_unary_expr(&self, op_token: TokenKind, operand: Expr) -> Result<Expr, ParseError> {
        match op_token {
            TokenKind::Minus => Ok(Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            }),
            TokenKind::Bang => Ok(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            }),
            TokenKind::Quote => Ok(Expr::Quote(Box::new(operand))),
            TokenKind::Reflect => Ok(Expr::Unary {
                op: UnaryOp::Reflect,
                operand: Box::new(operand),
            }),
            TokenKind::Comma => {
                // Comma as unquote operator (,expr)
                Ok(Expr::Unquote(Box::new(operand)))
            }
            _ => Err(ParseError::InvalidStatement {
                message: format!("invalid unary operator: {:?}", op_token),
                span: self.current.span,
            }),
        }
    }

    /// Parses a lambda expression: |params| body
    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Bar)?;

        let mut params = Vec::new();
        while self.current.kind != TokenKind::Bar && self.current.kind != TokenKind::Eof {
            let name = self.expect_identifier()?;
            let type_ann = if self.current.kind == TokenKind::Colon {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push((name, type_ann));

            if self.current.kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::Bar)?;

        let return_type = if self.current.kind == TokenKind::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = Box::new(self.parse_expr(0)?);

        Ok(Expr::Lambda {
            params,
            return_type,
            body,
        })
    }

    /// Parses an if expression: if condition { then } else { else }
    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::If)?;

        let condition = Box::new(self.parse_expr(0)?);

        self.expect(TokenKind::LeftBrace)?;
        let then_branch = Box::new(self.parse_block_expr_inner()?);
        self.expect(TokenKind::RightBrace)?;

        let else_branch = if self.current.kind == TokenKind::Else {
            self.advance();
            if self.current.kind == TokenKind::If {
                // else if
                Some(Box::new(self.parse_if_expr()?))
            } else {
                self.expect(TokenKind::LeftBrace)?;
                let else_expr = Box::new(self.parse_block_expr_inner()?);
                self.expect(TokenKind::RightBrace)?;
                Some(else_expr)
            }
        } else {
            None
        };

        Ok(Expr::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    /// Parses a match expression.
    fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Match)?;

        let scrutinee = Box::new(self.parse_expr(0)?);

        self.expect(TokenKind::LeftBrace)?;

        let mut arms = Vec::new();
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            let pattern = self.parse_pattern()?;

            let guard = if self.current.kind == TokenKind::If {
                self.advance();
                Some(Box::new(self.parse_expr(0)?))
            } else {
                None
            };

            self.expect(TokenKind::FatArrow)?;

            let body = Box::new(self.parse_expr(0)?);

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Expr::Match { scrutinee, arms })
    }

    /// Parses a pattern for match expressions.
    pub fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.current.kind {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::String => {
                let value = self.current.lexeme.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(value)))
            }
            TokenKind::Identifier => {
                let name = self.current.lexeme.clone();
                self.advance();

                // Check for constructor pattern
                if self.current.kind == TokenKind::LeftParen {
                    self.advance();
                    let mut fields = Vec::new();
                    while self.current.kind != TokenKind::RightParen
                        && self.current.kind != TokenKind::Eof
                    {
                        fields.push(self.parse_pattern()?);
                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RightParen)?;
                    Ok(Pattern::Constructor { name, fields })
                } else if name == "true" {
                    Ok(Pattern::Literal(Literal::Bool(true)))
                } else if name == "false" {
                    Ok(Pattern::Literal(Literal::Bool(false)))
                } else {
                    Ok(Pattern::Identifier(name))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let mut patterns = Vec::new();
                while self.current.kind != TokenKind::RightParen
                    && self.current.kind != TokenKind::Eof
                {
                    patterns.push(self.parse_pattern()?);
                    if self.current.kind == TokenKind::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::RightParen)?;
                Ok(Pattern::Tuple(patterns))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "pattern".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            }),
        }
    }

    /// Parses a block expression: { statements; final_expr }
    fn parse_block_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        let expr = self.parse_block_expr_inner()?;
        self.expect(TokenKind::RightBrace)?;
        Ok(expr)
    }

    /// Parses a sex block expression: sex { statements }
    fn parse_sex_block(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Sex)?;
        self.expect(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();
        let mut final_expr = None;

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            // Check if this is a statement or final expression
            if self.is_statement_keyword() {
                statements.push(self.parse_stmt()?);
            } else {
                // Try to parse as expression
                let expr = self.parse_expr(0)?;

                // If followed by semicolon, it's a statement
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                    statements.push(Stmt::Expr(expr));
                } else {
                    // It's the final expression
                    final_expr = Some(Box::new(expr));
                    break;
                }
            }
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Expr::SexBlock {
            statements,
            final_expr,
        })
    }

    /// Parses the interior of a block expression (without braces).
    fn parse_block_expr_inner(&mut self) -> Result<Expr, ParseError> {
        let mut statements = Vec::new();
        let mut final_expr = None;

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            // Check if this is a statement or final expression
            if self.is_statement_keyword() {
                statements.push(self.parse_stmt()?);
            } else {
                // Try to parse as expression
                let expr = self.parse_expr(0)?;

                // If followed by semicolon, it's a statement
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                    statements.push(Stmt::Expr(expr));
                } else {
                    // It's the final expression
                    final_expr = Some(Box::new(expr));
                    break;
                }
            }
        }

        Ok(Expr::Block {
            statements,
            final_expr,
        })
    }

    /// Checks if the current token is a statement keyword.
    fn is_statement_keyword(&self) -> bool {
        matches!(
            self.current.kind,
            TokenKind::Let
                | TokenKind::Var
                | TokenKind::Const
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Loop
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Return
                | TokenKind::Sex
        )
    }

    /// Parses a statement.
    pub fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.current.kind {
            TokenKind::Let => {
                self.advance();
                let name = self.expect_identifier()?;

                let type_ann = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };

                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr(0)?;
                self.expect(TokenKind::Semicolon)?;

                Ok(Stmt::Let {
                    name,
                    type_ann,
                    value,
                })
            }
            TokenKind::Var => {
                self.advance();
                let name = self.expect_identifier()?;

                let type_ann = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };

                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr(0)?;
                self.expect(TokenKind::Semicolon)?;

                Ok(Stmt::Let {
                    name,
                    type_ann,
                    value,
                })
            }
            TokenKind::Const => {
                self.advance();
                let name = self.expect_identifier()?;

                let type_ann = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };

                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr(0)?;
                self.expect(TokenKind::Semicolon)?;

                Ok(Stmt::Let {
                    name,
                    type_ann,
                    value,
                })
            }
            TokenKind::Sex => {
                // Parse as expression (sex block)
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Loop => self.parse_loop_stmt(),
            TokenKind::Break => {
                self.advance();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Break)
            }
            TokenKind::Continue => {
                self.advance();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Continue)
            }
            TokenKind::Return => {
                self.advance();
                let value = if self.current.kind != TokenKind::Semicolon {
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Return(value))
            }
            _ => {
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    /// Parses a for loop statement.
    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::For)?;

        let binding = self.expect_identifier()?;
        self.expect(TokenKind::In)?;
        let iterable = self.parse_expr(0)?;

        self.expect(TokenKind::LeftBrace)?;
        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            body.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::For {
            binding,
            iterable,
            body,
        })
    }

    /// Parses a while loop statement.
    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::While)?;

        let condition = self.parse_expr(0)?;

        self.expect(TokenKind::LeftBrace)?;
        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            body.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::While { condition, body })
    }

    /// Parses a loop statement.
    fn parse_loop_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Loop)?;

        self.expect(TokenKind::LeftBrace)?;
        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            body.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::Loop { body })
    }

    /// Parses a type expression.
    pub fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        // Handle built-in type keywords
        let base_type = match self.current.kind {
            TokenKind::Int8 => {
                self.advance();
                TypeExpr::Named("Int8".to_string())
            }
            TokenKind::Int16 => {
                self.advance();
                TypeExpr::Named("Int16".to_string())
            }
            TokenKind::Int32 => {
                self.advance();
                TypeExpr::Named("Int32".to_string())
            }
            TokenKind::Int64 => {
                self.advance();
                TypeExpr::Named("Int64".to_string())
            }
            TokenKind::UInt8 => {
                self.advance();
                TypeExpr::Named("UInt8".to_string())
            }
            TokenKind::UInt16 => {
                self.advance();
                TypeExpr::Named("UInt16".to_string())
            }
            TokenKind::UInt32 => {
                self.advance();
                TypeExpr::Named("UInt32".to_string())
            }
            TokenKind::UInt64 => {
                self.advance();
                TypeExpr::Named("UInt64".to_string())
            }
            TokenKind::Float32 => {
                self.advance();
                TypeExpr::Named("Float32".to_string())
            }
            TokenKind::Float64 => {
                self.advance();
                TypeExpr::Named("Float64".to_string())
            }
            TokenKind::BoolType => {
                self.advance();
                TypeExpr::Named("Bool".to_string())
            }
            TokenKind::StringType => {
                self.advance();
                TypeExpr::Named("String".to_string())
            }
            TokenKind::VoidType => {
                self.advance();
                TypeExpr::Named("Void".to_string())
            }
            TokenKind::Identifier => {
                let name = self.expect_identifier()?;

                // Check for generic type
                if self.current.kind == TokenKind::Lt {
                    self.advance();
                    let mut args = Vec::new();
                    while self.current.kind != TokenKind::Greater
                        && self.current.kind != TokenKind::Eof
                    {
                        args.push(self.parse_type()?);
                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::Greater)?;
                    TypeExpr::Generic { name, args }
                } else {
                    TypeExpr::Named(name)
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let mut types = Vec::new();
                while self.current.kind != TokenKind::RightParen
                    && self.current.kind != TokenKind::Eof
                {
                    types.push(self.parse_type()?);
                    if self.current.kind == TokenKind::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::RightParen)?;

                // Check if it's a function type
                if self.current.kind == TokenKind::Arrow {
                    self.advance();
                    let return_type = Box::new(self.parse_type()?);
                    TypeExpr::Function {
                        params: types,
                        return_type,
                    }
                } else {
                    TypeExpr::Tuple(types)
                }
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "type".to_string(),
                    found: format!("'{}'", self.current.lexeme),
                    span: self.current.span,
                })
            }
        };

        Ok(base_type)
    }

    /// Parses a fun declaration (for DOL 2.0 gene/trait bodies).
    #[allow(dead_code)]
    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Function)?;

        let name = self.expect_identifier()?;

        self.expect(TokenKind::LeftParen)?;
        let mut params = Vec::new();
        while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof {
            let param_name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let type_ann = self.parse_type()?;
            params.push(FunctionParam {
                name: param_name,
                type_ann,
            });

            if self.current.kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightParen)?;

        let return_type = if self.current.kind == TokenKind::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::LeftBrace)?;
        let mut body = Vec::new();
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            body.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RightBrace)?;

        let span = start_span.merge(&self.previous.span);

        Ok(FunctionDecl {
            visibility: Visibility::default(),
            purity: Purity::default(),
            name,
            type_params: None,
            params,
            return_type,
            body,
            span,
        })
    }

    /// Parses a law declaration in a trait.
    ///
    /// Syntax: `law name(params) { body } [exegesis { ... }]`
    ///
    /// Laws are declarative constraints/properties that must hold for a trait.
    /// Unlike `fun` which contains implementation code, `law` bodies are
    /// logical expressions (predicates).
    pub fn parse_law_decl(&mut self) -> Result<LawDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Law)?;

        let name = self.expect_identifier()?;

        // Parse parameters
        self.expect(TokenKind::LeftParen)?;
        let mut params = Vec::new();
        while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof {
            let param_name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let type_ann = self.parse_type()?;
            params.push(FunctionParam {
                name: param_name,
                type_ann,
            });

            if self.current.kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightParen)?;

        // Parse body expression (a predicate/constraint)
        self.expect(TokenKind::LeftBrace)?;
        let body = self.parse_expr(0)?;
        self.expect(TokenKind::RightBrace)?;

        // Parse optional exegesis
        let exegesis = if self.current.kind == TokenKind::Exegesis {
            Some(self.parse_exegesis()?)
        } else {
            None
        };

        Ok(LawDecl {
            name,
            params,
            body,
            exegesis,
            span: start_span.merge(&self.previous.span),
        })
    }

    /// Parses a migrate block for evolution.
    ///
    /// Syntax: `migrate { statements }`
    ///
    /// Migrate blocks contain imperative migration code that transforms
    /// data or state from the old version to the new version.
    pub fn parse_migrate_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(TokenKind::Migrate)?;
        self.expect(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            statements.push(self.parse_stmt()?);
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(statements)
    }

    // === Macro Parsing ===

    /// Parses a macro invocation expression: #macro_name(args)
    fn parse_macro_invocation_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Macro)?; // consume #

        // Get macro name
        let name = self.expect_identifier()?;

        // Parse optional arguments
        let args = if self.current.kind == TokenKind::LeftParen {
            self.advance();
            let mut args = Vec::new();
            while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof
            {
                args.push(self.parse_expr(0)?);
                if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RightParen)?;
            args
        } else {
            Vec::new()
        };

        let _span = start_span.merge(&self.previous.span);

        // Return as a special MacroCall expression
        // We encode this as a Call with a special identifier prefix
        Ok(Expr::Call {
            callee: Box::new(Expr::Identifier(format!("#{}", name))),
            args,
        })
    }

    /// Parses idiom brackets: [| f a b |]
    /// Desugars to f <$> a <*> b for applicative functor style.
    fn parse_idiom_bracket(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::IdiomOpen)?; // consume [|

        // Parse the function (first expression)
        let func = self.parse_expr(0)?;

        // Parse arguments until we hit |]
        let mut args = Vec::new();
        while self.current.kind != TokenKind::IdiomClose && self.current.kind != TokenKind::Eof {
            args.push(self.parse_expr(0)?);
        }

        self.expect(TokenKind::IdiomClose)?; // consume |]

        Ok(Expr::IdiomBracket {
            func: Box::new(func),
            args,
        })
    }

    /// Parses a macro invocation and returns the MacroInvocation AST node.
    pub fn parse_macro_invocation(&mut self) -> Result<MacroInvocation, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Macro)?; // consume #

        // Get macro name
        let name = self.expect_identifier()?;

        // Parse optional arguments
        let args = if self.current.kind == TokenKind::LeftParen {
            self.advance();
            let mut args = Vec::new();
            while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof
            {
                args.push(self.parse_expr(0)?);
                if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RightParen)?;
            args
        } else {
            Vec::new()
        };

        let span = start_span.merge(&self.previous.span);

        Ok(MacroInvocation::new(name, args, span))
    }

    /// Parses an attribute macro: #[macro_name(args)]
    pub fn parse_macro_attribute(&mut self) -> Result<MacroAttribute, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Macro)?; // consume #
        self.expect(TokenKind::LeftBracket)?; // consume [

        // Get macro name
        let name = self.expect_identifier()?;

        // Parse optional arguments
        let args = if self.current.kind == TokenKind::LeftParen {
            self.advance();
            let mut args = Vec::new();
            while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof
            {
                args.push(self.parse_attribute_arg()?);
                if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RightParen)?;
            args
        } else {
            Vec::new()
        };

        self.expect(TokenKind::RightBracket)?; // consume ]

        let span = start_span.merge(&self.previous.span);

        Ok(MacroAttribute::new(name, args, span))
    }

    /// Parses an attribute argument.
    fn parse_attribute_arg(&mut self) -> Result<AttributeArg, ParseError> {
        let name = self.expect_identifier()?;

        // Check for key = value or nested attribute
        if self.current.kind == TokenKind::Equal {
            self.advance();
            let value = self.parse_expr(0)?;
            Ok(AttributeArg::KeyValue { key: name, value })
        } else if self.current.kind == TokenKind::LeftParen {
            // Nested attribute
            self.advance();
            let mut args = Vec::new();
            while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof
            {
                args.push(self.parse_attribute_arg()?);
                if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RightParen)?;
            Ok(AttributeArg::Nested { name, args })
        } else {
            // Simple identifier
            Ok(AttributeArg::Ident(name))
        }
    }

    /// Checks if we're at the start of an attribute macro.
    pub fn is_at_attribute(&self) -> bool {
        // Check for #[ pattern
        if self.current.kind != TokenKind::Macro {
            return false;
        }
        // Would need lookahead to check for [
        true // Simplified check
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
  container has status
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
        // DOL 2.0 tolerant: missing exegesis defaults to empty string
        let input = r#"
gene container.exists {
  container has identity
}
"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());

        // Verify that exegesis is empty when not provided
        let decl = result.unwrap();
        if let Declaration::Gene(gene) = decl {
            assert!(gene.exegesis.is_empty());
        } else {
            panic!("Expected Gene declaration");
        }
    }
}
