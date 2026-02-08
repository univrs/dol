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

    /// Second peeked token for two-token lookahead (if any)
    peeked2: Option<Token>,

    /// Third peeked token for three-token lookahead (if any)
    peeked3: Option<Token>,
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
            peeked2: None,
            peeked3: None,
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

    /// Parses all declarations from the input.
    ///
    /// Skips module declarations and use statements, then parses all
    /// top-level declarations until EOF.
    ///
    /// # Returns
    ///
    /// A vector of all parsed declarations, or a `ParseError` on failure.
    pub fn parse_all(&mut self) -> Result<Vec<Declaration>, ParseError> {
        // Skip module declaration if present
        self.skip_module_and_uses()?;

        let mut declarations = Vec::new();

        while self.current.kind != TokenKind::Eof {
            let decl = self.parse_declaration()?;
            declarations.push(decl);
        }

        Ok(declarations)
    }

    /// Parses a complete DOL file including module and use declarations.
    ///
    /// Returns a `DolFile` containing the module declaration (if any),
    /// use declarations, and all top-level declarations.
    pub fn parse_file(&mut self) -> Result<DolFile, ParseError> {
        // Parse optional module declaration
        let module = if self.current.kind == TokenKind::Module {
            Some(self.parse_module_decl()?)
        } else {
            None
        };

        // Parse use declarations
        let mut uses = Vec::new();
        loop {
            // Handle pub use (re-exports)
            if self.current.kind == TokenKind::Pub && self.peek().kind == TokenKind::Use {
                self.advance(); // consume pub
                let use_decl = self.parse_use_decl(Visibility::Public)?;
                uses.push(use_decl);
            } else if self.current.kind == TokenKind::Use {
                uses.push(self.parse_use_decl(Visibility::Private)?);
            } else {
                break;
            }
        }

        // Parse all declarations
        let mut declarations = Vec::new();
        while self.current.kind != TokenKind::Eof {
            let decl = self.parse_declaration()?;
            declarations.push(decl);
        }

        Ok(DolFile {
            module,
            uses,
            declarations,
        })
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
                && self.current.kind != TokenKind::Gen
                && self.current.kind != TokenKind::Gene
                && self.current.kind != TokenKind::Type
                && self.current.kind != TokenKind::Trait
                && self.current.kind != TokenKind::Constraint
                && self.current.kind != TokenKind::Rule
                && self.current.kind != TokenKind::System
                && self.current.kind != TokenKind::Evolves
                && self.current.kind != TokenKind::Pub
                && self.current.kind != TokenKind::Use
                && self.current.kind != TokenKind::Module
                && self.current.kind != TokenKind::Exegesis
                && self.current.kind != TokenKind::Docs
                && self.current.kind != TokenKind::Macro    // Stop before attributes
                && self.current.kind != TokenKind::Sex      // Stop before sex declarations
                && self.current.kind != TokenKind::Function // Stop before functions
                && self.current.kind != TokenKind::Const
            // Stop before constants
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
    #[allow(dead_code)]
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

        // Regular type expression: consume identifier/type keyword and optional generics
        // Handle built-in type keywords (String, Int8, Int16, etc.)
        if self.current.kind == TokenKind::Identifier
            || self.current.kind == TokenKind::StringType
            || self.current.kind == TokenKind::Int8
            || self.current.kind == TokenKind::Int16
            || self.current.kind == TokenKind::Int32
            || self.current.kind == TokenKind::Int64
            || self.current.kind == TokenKind::UInt8
            || self.current.kind == TokenKind::UInt16
            || self.current.kind == TokenKind::UInt32
            || self.current.kind == TokenKind::UInt64
            || self.current.kind == TokenKind::Float32
            || self.current.kind == TokenKind::Float64
            || self.current.kind == TokenKind::BoolType
        {
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
        // Collect attribute annotations like #[wasm_export] or #[test]
        let mut collected_attributes: Vec<String> = Vec::new();
        while self.current.kind == TokenKind::Macro {
            self.advance(); // consume #
            if self.current.kind == TokenKind::LeftBracket {
                self.advance(); // consume [
                                // Get the attribute name
                let attr_name = if self.current.kind == TokenKind::Identifier {
                    self.current.lexeme.clone()
                } else {
                    String::new()
                };

                // Skip to closing ]
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LeftBracket => depth += 1,
                        TokenKind::RightBracket => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }

                // If it's a #[test] attribute, skip the following function entirely
                if attr_name == "test" {
                    if self.current.kind == TokenKind::Function {
                        self.advance(); // consume 'fun'
                                        // Skip function name and body
                        while self.current.kind != TokenKind::Eof {
                            if self.current.kind == TokenKind::LeftBrace {
                                // Skip function body
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
                                break;
                            }
                            self.advance();
                        }
                    }
                    // Check if we've reached end of file after skipping tests
                    if self.current.kind == TokenKind::Eof {
                        return Ok(Declaration::Gene(Gen {
                            visibility: Visibility::default(),
                            name: "_test_skipped".to_string(),
                            extends: None,
                            statements: vec![],
                            exegesis: "Tests skipped".to_string(),
                            span: self.current.span,
                        }));
                    }
                    // Clear collected attributes since we skipped a test
                    collected_attributes.clear();
                } else if !attr_name.is_empty() {
                    // Store non-test attributes to attach to the next declaration
                    collected_attributes.push(attr_name);
                }
            }
        }

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
            TokenKind::Gene | TokenKind::Gen => self.parse_gene(),
            // type is an alias for gene (v0.3.0)
            TokenKind::Type => self.parse_type_declaration(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Constraint | TokenKind::Rule => self.parse_constraint(),
            TokenKind::System => self.parse_system(),
            TokenKind::Evolves | TokenKind::Evo => self.parse_evolution(),
            TokenKind::Sex => self.parse_sex_top_level_with_attrs(collected_attributes),
            TokenKind::Function => {
                // Top-level pure function
                let mut func = self.parse_function_decl()?;
                func.attributes = collected_attributes;
                Ok(Declaration::Function(Box::new(func)))
            }
            TokenKind::Const => {
                // Top-level constant declaration
                self.parse_const_decl()
            }
            TokenKind::Exegesis | TokenKind::Docs => {
                // Skip file-level exegesis/docs block
                self.advance(); // consume 'exegesis'/'docs'
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
                    Ok(Declaration::Gene(Gen {
                        visibility: Visibility::default(),
                        name: "_module_doc".to_string(),
                        extends: None,
                        statements: vec![],
                        exegesis: "Module-level documentation".to_string(),
                        span: self.current.span,
                    }))
                } else {
                    self.parse_declaration()
                }
            }
            TokenKind::Use => {
                // Skip use statement and parse next declaration
                self.advance(); // consume 'use'
                                // Skip path (identifiers, ::, ., *, { }, etc.)
                while self.current.kind != TokenKind::Eof
                    && self.current.kind != TokenKind::Gen
                    && self.current.kind != TokenKind::Gene
                    && self.current.kind != TokenKind::Type
                    && self.current.kind != TokenKind::Trait
                    && self.current.kind != TokenKind::Constraint
                    && self.current.kind != TokenKind::Rule
                    && self.current.kind != TokenKind::System
                    && self.current.kind != TokenKind::Evolves
                    && self.current.kind != TokenKind::Pub
                    && self.current.kind != TokenKind::Use
                    && self.current.kind != TokenKind::Exegesis
                    && self.current.kind != TokenKind::Docs
                    && self.current.kind != TokenKind::Sex
                    && self.current.kind != TokenKind::Function
                    && self.current.kind != TokenKind::Module
                    && self.current.kind != TokenKind::Macro
                // Stop before attributes
                {
                    self.advance();
                }
                // Parse next declaration
                if self.current.kind == TokenKind::Eof {
                    Ok(Declaration::Gene(Gen {
                        visibility: Visibility::default(),
                        name: "_use_only".to_string(),
                        extends: None,
                        statements: vec![],
                        exegesis: "Use-only file".to_string(),
                        span: self.current.span,
                    }))
                } else {
                    self.parse_declaration()
                }
            }
            TokenKind::Module => {
                // Skip mod/module submodule declaration and parse next declaration
                self.advance(); // consume 'mod' or 'module'
                                // Skip module name
                if self.current.kind == TokenKind::Identifier {
                    self.advance();
                }
                // Skip block content if present
                if self.current.kind == TokenKind::LeftBrace {
                    self.advance();
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
                }
                // Parse next declaration
                if self.current.kind == TokenKind::Eof {
                    Ok(Declaration::Gene(Gen {
                        visibility: Visibility::default(),
                        name: "_module_decl".to_string(),
                        extends: None,
                        statements: vec![],
                        exegesis: "Module-only file".to_string(),
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
    fn parse_module_decl(&mut self) -> Result<ModuleDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Module)?;

        // Parse module path (e.g., "univrs.container.lifecycle")
        // The lexer may produce a single qualified identifier "dol.ast" or separate tokens
        let mut path = Vec::new();
        let ident = self.expect_identifier()?;
        // Split qualified identifiers into path components
        path.extend(ident.split('.').map(|s| s.to_string()));

        while self.current.kind == TokenKind::Dot {
            self.advance();
            let ident = self.expect_identifier()?;
            path.extend(ident.split('.').map(|s| s.to_string()));
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
    ///
    /// Supports import sources:
    /// - Local: `use container`
    /// - Registry: `use @univrs/std`
    /// - Git: `use @git:github.com/org/repo`
    /// - HTTPS: `use @https://example.com/file.dol`
    fn parse_use_decl(&mut self, visibility: Visibility) -> Result<UseDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Use)?;

        // Parse import source prefix
        let (source, mut path) = self.parse_import_source()?;

        // Continue parsing path with :: or . separators
        // Use expect_identifier_or_keyword to allow keywords like "state" in paths
        while self.current.kind == TokenKind::PathSep || self.current.kind == TokenKind::Dot {
            self.advance();
            if self.current.kind == TokenKind::LeftBrace {
                break; // Items list
            }
            if self.current.kind == TokenKind::Star {
                break; // Glob import
            }
            let ident = self.expect_identifier_or_keyword()?;
            path.extend(ident.split('.').map(|s| s.to_string()));
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
            visibility,
            source,
            path,
            items,
            alias,
            span,
        })
    }

    /// Parses the import source prefix and initial path.
    ///
    /// Returns (ImportSource, initial_path_components)
    fn parse_import_source(&mut self) -> Result<(ImportSource, Vec<String>), ParseError> {
        // Check for @ prefix indicating external source
        if self.current.kind == TokenKind::At {
            self.advance(); // consume @

            let prefix = self.expect_identifier()?;

            // Check for special prefixes
            if prefix == "git" && self.current.kind == TokenKind::Colon {
                // @git:github.com/org/repo
                self.advance(); // consume :
                let url = self.parse_url_path()?;
                let reference = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                return Ok((ImportSource::Git { url, reference }, Vec::new()));
            } else if prefix == "https" && self.current.kind == TokenKind::Colon {
                // @https://example.com/file.dol
                self.advance(); // consume :
                                // Expect // after https:
                if self.current.lexeme == "/" {
                    self.advance();
                    self.advance(); // consume //
                }
                let url = format!("https://{}", self.parse_url_path()?);
                return Ok((ImportSource::Https { url, sha256: None }, Vec::new()));
            } else {
                // @org/package format - this is a registry import
                // prefix is the org name
                let org = prefix;
                self.expect(TokenKind::Slash)?;
                let full_ident = self.expect_identifier()?;

                // Split the identifier - first segment is package, rest goes to path
                // e.g., "std.io.println" -> package="std", path=["io", "println"]
                let segments: Vec<&str> = full_ident.split('.').collect();
                let package = segments[0].to_string();
                let path: Vec<String> = segments[1..].iter().map(|s| s.to_string()).collect();

                // Optional version constraint after colon
                let version = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    // Version could be a string like "^1.0" or just identifier
                    if self.current.kind == TokenKind::String {
                        let v = self.current.lexeme.trim_matches('"').to_string();
                        self.advance();
                        Some(v)
                    } else {
                        Some(self.expect_identifier()?)
                    }
                } else {
                    None
                };

                // Return the path segments that come after the package name
                return Ok((
                    ImportSource::Registry {
                        org,
                        package,
                        version,
                    },
                    path,
                ));
            }
        }

        // Local import - parse initial path segment only
        // The main loop in parse_use_decl will handle additional segments
        // Use expect_identifier_or_keyword to allow keywords like "state" as module names
        let mut path = Vec::new();
        let ident = self.expect_identifier_or_keyword()?;
        path.extend(ident.split('.').map(|s| s.to_string()));

        Ok((ImportSource::Local, path))
    }

    /// Parses a URL-like path (for git: and https: sources).
    fn parse_url_path(&mut self) -> Result<String, ParseError> {
        let mut parts = Vec::new();

        // Collect path segments separated by / and .
        loop {
            if self.current.kind == TokenKind::Identifier {
                parts.push(self.current.lexeme.to_string());
                self.advance();
            } else {
                break;
            }

            if self.current.kind == TokenKind::Slash {
                parts.push("/".to_string());
                self.advance();
            } else if self.current.kind == TokenKind::Dot {
                parts.push(".".to_string());
                self.advance();
            } else {
                break;
            }
        }

        if parts.is_empty() {
            return Err(ParseError::UnexpectedToken {
                expected: "URL path".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            });
        }

        Ok(parts.join(""))
    }
    /// Parses a gene declaration.
    fn parse_gene(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;

        // v0.8.0: Accept both "gen" (new) and "gene" (deprecated)
        if self.current.kind == TokenKind::Gen {
            self.advance();
        } else if self.current.kind == TokenKind::Gene {
            eprintln!("warning: 'gene' keyword is deprecated in v0.8.0, use 'gen' instead at line {}, column {}",
                self.current.span.line, self.current.span.column);
            self.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "'gen' or 'gene'".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            });
        }

        let name = self.expect_identifier()?;
        // Skip generic type parameters if present: <T, U: Bound>
        self.skip_type_params()?;

        // Parse optional extends clause (v0.3.0): gene Foo extends Bar { ... }
        let extends = if self.current.kind == TokenKind::Extends {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(TokenKind::LeftBrace)?;

        let statements = self.parse_statements()?;

        // DOL 2.0: exegesis can be inside braces
        let inline_exegesis = self.parse_inline_exegesis()?;

        self.expect(TokenKind::RightBrace)?;

        // DOL 1.0: exegesis can be after braces
        // DOL 2.0: use inline or default to empty
        let exegesis = if let Some(ex) = inline_exegesis {
            ex
        } else if self.current.kind == TokenKind::Exegesis || self.current.kind == TokenKind::Docs {
            self.parse_exegesis()?
        } else {
            String::new() // DOL 2.0 tolerant: empty exegesis if none
        };

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Gene(Gen {
            visibility: Visibility::default(),
            name,
            extends,
            statements,
            exegesis,
            span,
        }))
    }

    /// Parses a type declaration (v0.3.0 - alias for gene).
    ///
    /// Type declarations work exactly like gene declarations but use the `type` keyword.
    /// This provides an alternative syntax that may be more familiar to developers
    /// coming from other languages.
    fn parse_type_declaration(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Type)?;

        let name = self.expect_identifier()?;
        // Skip generic type parameters if present: <T, U: Bound>
        self.skip_type_params()?;

        // Parse optional extends clause: type Foo extends Bar { ... }
        let extends = if self.current.kind == TokenKind::Extends {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(TokenKind::LeftBrace)?;

        let statements = self.parse_statements()?;

        // DOL 2.0: exegesis can be inside braces
        let inline_exegesis = self.parse_inline_exegesis()?;

        self.expect(TokenKind::RightBrace)?;

        // DOL 1.0: exegesis can be after braces
        // DOL 2.0: use inline or default to empty
        let exegesis = if let Some(ex) = inline_exegesis {
            ex
        } else if self.current.kind == TokenKind::Exegesis || self.current.kind == TokenKind::Docs {
            self.parse_exegesis()?
        } else {
            String::new() // DOL 2.0 tolerant: empty exegesis if none
        };

        let span = start_span.merge(&self.previous.span);

        // Type declarations are represented as Gene in the AST
        Ok(Declaration::Gene(Gen {
            visibility: Visibility::default(),
            name,
            extends,
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
            && self.current.kind != TokenKind::Docs
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
        } else if self.current.kind == TokenKind::Exegesis || self.current.kind == TokenKind::Docs {
            self.parse_exegesis()?
        } else {
            String::new()
        };

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Trait(Trait {
            visibility: Visibility::default(),
            name,
            statements,
            exegesis,
            span,
        }))
    }

    /// Parses a constraint declaration.
    fn parse_constraint(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;

        // v0.8.0: Accept both "rule" (new) and "constraint" (deprecated)
        if self.current.kind == TokenKind::Rule {
            self.advance();
        } else if self.current.kind == TokenKind::Constraint {
            eprintln!("warning: 'constraint' keyword is deprecated in v0.8.0, use 'rule' instead at line {}, column {}",
                self.current.span.line, self.current.span.column);
            self.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "'rule' or 'constraint'".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            });
        }

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
        } else if self.current.kind == TokenKind::Exegesis || self.current.kind == TokenKind::Docs {
            self.parse_exegesis()?
        } else {
            String::new()
        };

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Constraint(Rule {
            visibility: Visibility::default(),
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
            && self.current.kind != TokenKind::Docs
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
        } else if self.current.kind == TokenKind::Exegesis || self.current.kind == TokenKind::Docs {
            self.parse_exegesis()?
        } else {
            String::new()
        };

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::System(System {
            visibility: Visibility::default(),
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

        // v0.8.0: Accept both "evo" (new) and "evolves" (deprecated)
        if self.current.kind == TokenKind::Evo {
            self.advance();
        } else if self.current.kind == TokenKind::Evolves {
            eprintln!("warning: 'evolves' keyword is deprecated in v0.8.0, use 'evo' instead at line {}, column {}",
                self.current.span.line, self.current.span.column);
            self.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "'evo' or 'evolves'".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            });
        }

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
            && self.current.kind != TokenKind::Docs
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
        } else if self.current.kind == TokenKind::Exegesis || self.current.kind == TokenKind::Docs {
            self.parse_exegesis()?
        } else {
            String::new()
        };

        let span = start_span.merge(&self.previous.span);

        Ok(Declaration::Evolution(Evo {
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

        // Stop at RightBrace or Eof
        // DOL 2.0/v0.4.0: exegesis blocks can appear throughout gene body, handled in parse_statement
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    /// Parses a single statement.
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start_span = self.current.span;

        // Handle DOL 2.0/v0.4.0 inline exegesis blocks - skip them
        while self.current.kind == TokenKind::Exegesis || self.current.kind == TokenKind::Docs {
            self.advance(); // consume 'exegesis' or 'docs'
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
            // If we're at the end of the block, return a no-op marker
            if self.current.kind == TokenKind::RightBrace || self.current.kind == TokenKind::Eof {
                return Ok(Statement::Is {
                    subject: "_skip".to_string(),
                    state: "_noop".to_string(),
                    span: start_span.merge(&self.previous.span),
                });
            }
        }

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

        // Handle annotations before field declarations: @crdt(strategy), @personal
        // Multiple annotations can appear in any order
        let mut crdt_annotation = None;
        let mut personal = false;

        while self.current.kind == TokenKind::At {
            if self.peek().kind == TokenKind::Identifier {
                match self.peek().lexeme.as_str() {
                    "crdt" => {
                        crdt_annotation = Some(self.parse_crdt_annotation()?);
                    }
                    "personal" => {
                        self.advance(); // consume @
                        self.advance(); // consume 'personal'
                        personal = true;
                    }
                    _ => {
                        // Unknown annotation, stop parsing annotations
                        break;
                    }
                }
            } else {
                break;
            }
        }

        // Handle DOL 2.0 'has' field declarations: has name: Type [= default]
        if self.current.kind == TokenKind::Has {
            self.advance();
            let name = self.expect_identifier_or_keyword()?;
            // Check for typed field: has name: Type
            if self.current.kind == TokenKind::Colon {
                self.advance();
                let type_ = self.parse_type()?;
                // Parse optional default value: = expr
                let default = if self.current.kind == TokenKind::Equal {
                    self.advance();
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                // Parse optional constraint: where expr
                let constraint = if self.current.kind == TokenKind::Where {
                    self.advance();
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                return Ok(Statement::HasField(Box::new(HasField {
                    name,
                    type_,
                    default,
                    constraint,
                    crdt_annotation,
                    personal,
                    span: start_span.merge(&self.previous.span),
                })));
            } else {
                // Skip untyped default value: = expr
                if self.current.kind == TokenKind::Equal {
                    self.advance();
                    self.parse_expr(0)?;
                }
                // If CRDT annotation was present but not used, warn
                if crdt_annotation.is_some() {
                    eprintln!(
                        "warning: CRDT annotation on untyped field '{}' at line {}, column {}",
                        name, start_span.line, start_span.column
                    );
                }
                return Ok(Statement::Has {
                    subject: "self".to_string(),
                    property: name,
                    span: start_span.merge(&self.previous.span),
                });
            }
        }

        // Handle DOL 2.0 inline 'constraint'/'rule' blocks inside declarations
        if self.current.kind == TokenKind::Constraint || self.current.kind == TokenKind::Rule {
            // Emit deprecation warning for old 'constraint' keyword
            if self.current.kind == TokenKind::Constraint {
                eprintln!("warning: 'constraint' keyword is deprecated in v0.8.0, use 'rule' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
            }
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

        // Handle DOL 2.0 function declarations inside genes: [pub] [sex] fun name(...) -> Type { ... }
        // Check for optional visibility modifier
        let mut visibility = Visibility::Private;
        let mut purity = Purity::Pure;

        if self.current.kind == TokenKind::Pub {
            visibility = Visibility::Public;
            self.advance();
        }

        // Check for optional purity modifier (sex = side-effecting)
        if self.current.kind == TokenKind::Sex {
            purity = Purity::Sex;
            self.advance();
        }

        if self.current.kind == TokenKind::Function {
            let mut func = self.parse_function_decl()?;
            func.visibility = visibility;
            func.purity = purity;
            return Ok(Statement::Function(Box::new(func)));
        }

        // If we consumed pub/sex but didn't find 'fun', this is an error
        if visibility != Visibility::Private || purity != Purity::Pure {
            return Err(ParseError::UnexpectedToken {
                expected: "'fun' after visibility/purity modifier".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
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

        // Parse subject - allow keywords as field names (e.g., `type: Int64`)
        let subject = self.expect_identifier_or_keyword()?;

        // Determine statement type based on predicate
        match self.current.kind {
            TokenKind::Has => {
                self.advance();
                let property = self.expect_identifier_or_keyword()?;
                // Check for typed field: has name: Type
                if self.current.kind == TokenKind::Colon {
                    self.advance(); // consume ':'
                    let type_ = self.parse_type()?;
                    // Parse optional default value: = expr
                    let default = if self.current.kind == TokenKind::Equal {
                        self.advance();
                        Some(self.parse_expr(0)?)
                    } else {
                        None
                    };
                    // Parse optional constraint: where expr
                    let constraint = if self.current.kind == TokenKind::Where {
                        self.advance();
                        Some(self.parse_expr(0)?)
                    } else {
                        None
                    };
                    Ok(Statement::HasField(Box::new(HasField {
                        name: property,
                        type_,
                        default,
                        constraint,
                        crdt_annotation,
                        personal,
                        span: start_span.merge(&self.previous.span),
                    })))
                } else {
                    Ok(Statement::Has {
                        subject,
                        property,
                        span: start_span.merge(&self.previous.span),
                    })
                }
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
                                // Parse the type expression
                let type_ = self.parse_type()?;
                // Parse optional default value
                let default = if self.current.kind == TokenKind::Equal {
                    self.advance();
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                Ok(Statement::HasField(Box::new(HasField {
                    name: subject,
                    type_,
                    default,
                    constraint: None,
                    crdt_annotation: None,
                    personal: false,
                    span: start_span.merge(&self.previous.span),
                })))
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
                        let property = self.expect_identifier_or_keyword()?;
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
        let property = self.expect_identifier_or_keyword()?;

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

        let name = self.expect_identifier_or_keyword()?;
        self.expect(TokenKind::Has)?;
        let _property = self.expect_identifier_or_keyword()?; // "property" part becomes part of name

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
            crdt_annotation: None,
            personal: false,
            span: start_span.merge(&self.previous.span),
        })
    }

    /// Parses a CRDT annotation.
    ///
    /// Syntax: @crdt(strategy[, option1=value1, option2=value2, ...])
    ///
    /// # Returns
    ///
    /// The parsed `CrdtAnnotation` on success, or a `ParseError` on failure.
    ///
    /// # Example
    ///
    /// ```dol
    /// @crdt(immutable)
    /// @crdt(lww, tie_break="actor_id")
    /// @crdt(pn_counter, min_value=0, max_value=100)
    /// ```
    pub fn parse_crdt_annotation(&mut self) -> Result<CrdtAnnotation, ParseError> {
        let start_span = self.current.span;

        // Expect @ symbol
        self.expect(TokenKind::At)?;

        // Expect "crdt" identifier
        let crdt_kw = self.expect_identifier()?;
        if crdt_kw != "crdt" {
            return Err(ParseError::UnexpectedToken {
                expected: "crdt".to_string(),
                found: crdt_kw,
                span: self.previous.span,
            });
        }

        // Expect opening parenthesis
        self.expect(TokenKind::LeftParen)?;

        // Parse strategy
        let strategy_str = self.expect_identifier()?;
        let strategy = CrdtStrategy::from_str(&strategy_str).ok_or_else(|| {
            ParseError::InvalidCrdtStrategy {
                strategy: strategy_str.clone(),
                span: self.previous.span,
            }
        })?;

        // Parse optional options
        let mut options = Vec::new();
        while self.current.kind == TokenKind::Comma {
            self.advance(); // consume comma

            let option_start = self.current.span;
            let key = self.expect_identifier()?;

            self.expect(TokenKind::Equal)?;

            // Parse value expression
            let value = self.parse_expr(0)?;

            options.push(CrdtOption {
                key,
                value,
                span: option_start.merge(&self.previous.span),
            });
        }

        // Expect closing parenthesis
        self.expect(TokenKind::RightParen)?;

        Ok(CrdtAnnotation {
            strategy,
            options,
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

    /// Parses a top-level const declaration: const NAME: Type = value
    ///
    /// Returns a Declaration::Const for use in module-level declarations.
    pub fn parse_const_decl(&mut self) -> Result<Declaration, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Const)?;

        let name = self.expect_identifier()?;

        // Parse optional type annotation
        let type_ann = if self.current.kind == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Const requires a value
        self.expect(TokenKind::Equal)?;
        let value = self.parse_expr(0)?;

        // Consume optional semicolon
        self.consume_optional_semicolon();

        Ok(Declaration::Const(ConstDecl {
            visibility: Visibility::default(),
            name,
            type_ann,
            value,
            span: start_span.merge(&self.previous.span),
        }))
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
            // Allow DOL keywords as parameter names (e.g., `gene: GeneDecl`)
            let param_name = self.expect_identifier_or_keyword()?;
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

    /// Parse top-level sex declaration with attributes (sex var, sex fun, sex extern)
    fn parse_sex_top_level_with_attrs(
        &mut self,
        attributes: Vec<String>,
    ) -> Result<Declaration, ParseError> {
        let start = self.current.span;
        // Don't consume 'sex' - let child functions do it

        // Peek at what comes after 'sex'
        let next = self.peek();

        match next.kind {
            TokenKind::Var => {
                let var_decl = self.parse_sex_var()?;
                Ok(Declaration::SexVar(var_decl))
            }
            TokenKind::Function => {
                self.advance(); // consume 'sex'
                let mut func = self.parse_function_decl()?;
                func.purity = crate::ast::Purity::Sex;
                func.attributes = attributes;
                Ok(Declaration::Function(Box::new(func)))
            }
            TokenKind::Extern => {
                let extern_decl = self.parse_sex_extern()?;
                // Keep extern functions as Gene placeholder (FFI stubs need special handling)
                Ok(Declaration::Gene(Gen {
                    visibility: Visibility::default(),
                    name: extern_decl.name.clone(),
                    extends: None,
                    statements: vec![],
                    exegesis: format!("sex extern {}", extern_decl.name),
                    span: extern_decl.span,
                }))
            }
            _ => Err(ParseError::InvalidDeclaration {
                found: format!("sex {}", next.lexeme),
                span: start,
            }),
        }
    }

    /// Parses the exegesis block.
    fn parse_exegesis(&mut self) -> Result<String, ParseError> {
        // v0.8.0: Accept both "docs" (new) and "exegesis" (deprecated)
        if self.current.kind == TokenKind::Docs {
            self.advance(); // consume 'docs'
        } else if self.current.kind == TokenKind::Exegesis {
            eprintln!("warning: 'exegesis' keyword is deprecated in v0.8.0, use 'docs' instead at line {}, column {}",
                self.current.span.line, self.current.span.column);
            self.advance(); // consume 'exegesis'
        } else {
            return Err(ParseError::MissingExegesis {
                span: self.current.span,
            });
        }

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
        if self.current.kind != TokenKind::Exegesis && self.current.kind != TokenKind::Docs {
            return Ok(None);
        }

        self.advance(); // consume 'exegesis' or 'docs'
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
            // Special case: member access (.) should only consume an identifier
            if self.current.kind == TokenKind::Dot {
                self.advance();
                let field = self.expect_identifier()?;

                // Check if this is a struct literal: Type.Variant { ... }
                // Only treat as struct literal if:
                // 1. The field starts with uppercase (type name convention)
                // 2. The content looks like struct fields (identifier: value or empty)
                let is_type_name = field.chars().next().is_some_and(|c| c.is_uppercase());
                // Check if this is a struct literal: Type.Variant { ... }
                // A struct literal has fields in the form `identifier: value`
                // Use two-token lookahead to check for `{ identifier :` pattern
                let is_struct_literal = self.current.kind == TokenKind::LeftBrace
                    && is_type_name
                    && (self.peek().kind == TokenKind::RightBrace
                        || (Self::is_identifier_like(self.peek().kind)
                            && self.peek2().kind == TokenKind::Colon));

                if is_struct_literal {
                    // This is a struct literal like Type.Variant { field: value }
                    // Combine the lhs and field into a path name
                    let path_name = match &lhs {
                        Expr::Identifier(name) => format!("{}.{}", name, field),
                        Expr::Member { object, field: f } => {
                            if let Expr::Identifier(name) = object.as_ref() {
                                format!("{}.{}.{}", name, f, field)
                            } else {
                                field.clone()
                            }
                        }
                        _ => field.clone(),
                    };

                    self.advance(); // consume '{'
                    let mut fields = Vec::new();

                    while self.current.kind != TokenKind::RightBrace
                        && self.current.kind != TokenKind::Eof
                    {
                        // Allow keywords as struct field names (e.g., uses, module, etc.)
                        let field_name = self.expect_identifier_or_keyword()?;
                        self.expect(TokenKind::Colon)?;
                        let value = self.parse_expr(0)?;
                        fields.push((field_name, value));

                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RightBrace)?;

                    // Create struct literal expression
                    lhs = Expr::StructLiteral {
                        type_name: path_name,
                        fields,
                    };
                } else {
                    lhs = Expr::Member {
                        object: Box::new(lhs),
                        field,
                    };
                }
                continue;
            }

            // Check for infix operators (excluding Dot which is handled above)
            if let Some((left_bp, _right_bp)) = infix_binding_power(&self.current.kind) {
                if self.current.kind == TokenKind::Dot {
                    // Already handled above
                    break;
                }
                // Special case: @ followed by annotation keywords should not be treated as an operator
                if self.current.kind == TokenKind::At
                    && self.peek().kind == TokenKind::Identifier
                    && matches!(self.peek().lexeme.as_str(), "crdt" | "personal")
                {
                    // This is an annotation, not an Apply operator
                    break;
                }
                if left_bp < min_bp {
                    break;
                }

                let op = self.current.kind;
                self.advance();

                // Special handling for `as` - it takes a type, not an expression
                if op == TokenKind::As {
                    let target_type = self.parse_type()?;
                    lhs = Expr::Cast {
                        expr: Box::new(lhs),
                        target_type,
                    };
                } else {
                    let rhs = self.parse_expr(_right_bp)?;
                    lhs = self.make_binary_expr(lhs, op, rhs)?;
                }
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
            } else if self.current.kind == TokenKind::Reflect {
                // Postfix `?` - try operator for error propagation
                self.advance();
                lhs = Expr::Try(Box::new(lhs));
            } else if self.current.kind == TokenKind::LeftBrace {
                // Struct literal without path: Identifier { field: value }
                // Only treat as struct literal if:
                // 1. The name starts with uppercase (type name convention)
                // 2. The content looks like struct fields (identifier: value or empty)
                if let Expr::Identifier(name) = &lhs {
                    let is_type_name = name.chars().next().is_some_and(|c| c.is_uppercase());
                    if !is_type_name {
                        break;
                    }

                    // Use two-token lookahead to check for struct literal pattern
                    // - Empty: `Foo {}` - next token is `}`
                    // - With fields: `Foo { x: y }` - next is identifier or keyword, then `:`
                    let is_struct_literal = self.peek().kind == TokenKind::RightBrace
                        || (Self::is_identifier_like(self.peek().kind)
                            && self.peek2().kind == TokenKind::Colon);

                    if !is_struct_literal {
                        // Not a struct literal, likely a block like `if x != None { ... }`
                        break;
                    }

                    let struct_name = name.clone();
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();

                    while self.current.kind != TokenKind::RightBrace
                        && self.current.kind != TokenKind::Eof
                    {
                        // Allow keywords as struct field names (e.g., uses, module, etc.)
                        let field_name = self.expect_identifier_or_keyword()?;
                        self.expect(TokenKind::Colon)?;
                        let value = self.parse_expr(0)?;
                        fields.push((field_name, value));

                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RightBrace)?;

                    // Create struct literal expression
                    lhs = Expr::StructLiteral {
                        type_name: struct_name,
                        fields,
                    };
                } else {
                    break;
                }
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
            // Self-reference: this
            TokenKind::This => {
                self.advance();
                Ok(Expr::This)
            }

            // Literals
            TokenKind::String => {
                let value = self.current.lexeme.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(value)))
            }
            TokenKind::Char => {
                let value = self.current.lexeme.chars().next().unwrap_or('\0');
                self.advance();
                Ok(Expr::Literal(Literal::Char(value)))
            }
            TokenKind::Identifier => {
                let mut name = self.current.lexeme.clone();
                self.advance();

                // Check for special boolean literals
                if name == "true" {
                    return Ok(Expr::Literal(Literal::Bool(true)));
                } else if name == "false" {
                    return Ok(Expr::Literal(Literal::Bool(false)));
                }

                // Check for numeric literals (lexer sends them as identifiers)
                if let Ok(int_val) = name.parse::<i64>() {
                    return Ok(Expr::Literal(Literal::Int(int_val)));
                }
                if let Ok(float_val) = name.parse::<f64>() {
                    return Ok(Expr::Literal(Literal::Float(float_val)));
                }

                // Handle path expressions like Map::new, Type::Variant
                while self.current.kind == TokenKind::PathSep {
                    self.advance(); // consume ::
                    if self.current.kind == TokenKind::Identifier {
                        name.push_str("::");
                        name.push_str(&self.current.lexeme);
                        self.advance();
                    } else {
                        break;
                    }
                }

                Ok(Expr::Identifier(name))
            }

            // Allow DOL keywords to be used as identifiers in expression context
            TokenKind::Gene
            | TokenKind::Trait
            | TokenKind::System
            | TokenKind::Constraint
            | TokenKind::Rule
            | TokenKind::Evolves
            | TokenKind::Exegesis
            | TokenKind::Test
            | TokenKind::Law
            | TokenKind::State
            | TokenKind::Module
            | TokenKind::Use
            // v0.3.0 keywords that may appear as identifiers
            | TokenKind::Type
            | TokenKind::Val
            | TokenKind::Extends
            // Type keywords for referencing type enum variants
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
            | TokenKind::VoidType => {
                let mut name = self.current.lexeme.clone();
                self.advance();

                // Handle path expressions like Type::Variant
                while self.current.kind == TokenKind::PathSep {
                    self.advance(); // consume ::
                    if self.current.kind == TokenKind::Identifier {
                        name.push_str("::");
                        name.push_str(&self.current.lexeme);
                        self.advance();
                    } else {
                        break;
                    }
                }

                Ok(Expr::Identifier(name))
            }

            // Parenthesized expression or tuple
            TokenKind::LeftParen => {
                self.advance();

                // Empty tuple: ()
                if self.current.kind == TokenKind::RightParen {
                    self.advance();
                    return Ok(Expr::Tuple(vec![]));
                }

                let first = self.parse_expr(0)?;

                // Check for tuple (comma separated)
                if self.current.kind == TokenKind::Comma {
                    let mut elements = vec![first];
                    while self.current.kind == TokenKind::Comma {
                        self.advance();
                        if self.current.kind == TokenKind::RightParen {
                            break; // Trailing comma
                        }
                        elements.push(self.parse_expr(0)?);
                    }
                    self.expect(TokenKind::RightParen)?;
                    Ok(Expr::Tuple(elements))
                } else {
                    // Simple parenthesized expression
                    self.expect(TokenKind::RightParen)?;
                    Ok(first)
                }
            }

            // Lambda expression: |params| body
            TokenKind::Bar => self.parse_lambda(),

            // If expression
            TokenKind::If => self.parse_if_expr(),

            // Match expression
            TokenKind::Match => self.parse_match_expr(),

            // Forall quantifier expression (v0.3.0)
            // Syntax 1: forall x: T. expr (first-order logic style)
            // Syntax 2: forall x in iter { body } (iterator style)
            TokenKind::Forall => self.parse_forall_expr(),

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

            // List literal: [] or [expr, expr, ...]
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                while self.current.kind != TokenKind::RightBracket
                    && self.current.kind != TokenKind::Eof
                {
                    elements.push(self.parse_expr(0)?);
                    if self.current.kind == TokenKind::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::RightBracket)?;
                Ok(Expr::List(elements))
            }

            // Control flow as expressions (for use in match arms)
            TokenKind::Break => {
                self.advance();
                Ok(Expr::Block(Block {
                    statements: vec![Stmt::Break],
                    final_expr: None,
                    span: self.previous.span,
                }))
            }

            TokenKind::Continue => {
                self.advance();
                Ok(Expr::Block(Block {
                    statements: vec![Stmt::Continue],
                    final_expr: None,
                    span: self.previous.span,
                }))
            }

            TokenKind::Return => {
                self.advance();
                let start_span = self.previous.span;
                // Check if there's a return value
                let value = if self.current.kind != TokenKind::RightBrace
                    && self.current.kind != TokenKind::Comma
                    && self.current.kind != TokenKind::Eof
                    && !matches!(
                        self.current.kind,
                        TokenKind::RightParen | TokenKind::RightBracket
                    )
                {
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                Ok(Expr::Block(Block {
                    statements: vec![Stmt::Return(value)],
                    final_expr: None,
                    span: start_span,
                }))
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
            TokenKind::DotDot => BinaryOp::Range,
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
            TokenKind::Star => Ok(Expr::Unary {
                op: UnaryOp::Deref,
                operand: Box::new(operand),
            }),
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
            // Parse first pattern
            let first_pattern = self.parse_pattern()?;

            // Check for or-patterns (pattern | pattern | ...)
            let pattern = if self.current.kind == TokenKind::Bar {
                let mut patterns = vec![first_pattern];
                while self.current.kind == TokenKind::Bar {
                    self.advance();
                    patterns.push(self.parse_pattern()?);
                }
                Pattern::Or(patterns)
            } else {
                first_pattern
            };

            let guard = if self.current.kind == TokenKind::If {
                self.advance();
                Some(Box::new(self.parse_expr(0)?))
            } else {
                None
            };

            // Support both `pattern => body` and `pattern { body }` syntax
            let body = if self.current.kind == TokenKind::FatArrow {
                self.advance();
                Box::new(self.parse_expr(0)?)
            } else if self.current.kind == TokenKind::LeftBrace {
                // Parse block expression for brace syntax
                Box::new(self.parse_block_expr()?)
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "'=>' or '{'".to_string(),
                    found: format!("'{}'", self.current.lexeme),
                    span: self.current.span,
                });
            };

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

    /// Parses a forall quantifier expression (v0.3.0).
    ///
    /// Supports two syntaxes:
    /// - `forall x: T. expr` - First-order logic style with type annotation
    /// - `forall x in iter { body }` - Iterator style for collections
    fn parse_forall_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Forall)?;

        let var = self.expect_identifier()?;

        // Check for type-annotated syntax: forall x: T. expr
        if self.current.kind == TokenKind::Colon {
            self.advance();
            let type_ = self.parse_type()?;

            // Expect dot separator for the body
            self.expect(TokenKind::Dot)?;

            let body = self.parse_expr(0)?;
            let span = start_span.merge(&self.previous.span);

            Ok(Expr::Forall(ForallExpr {
                var,
                type_,
                body: Box::new(body),
                span,
            }))
        } else if self.current.kind == TokenKind::In {
            // Iterator syntax: forall x in iter { body }
            self.advance();
            let iter = self.parse_expr(0)?;

            self.expect(TokenKind::LeftBrace)?;
            let mut statements = Vec::new();
            let mut final_expr = None;

            while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof
            {
                if self.is_statement_keyword() {
                    statements.push(self.parse_stmt()?);
                } else {
                    let expr = self.parse_expr(0)?;
                    // Check if this is the final expression or needs to be a statement
                    if self.current.kind == TokenKind::Semicolon {
                        self.advance();
                        statements.push(Stmt::Expr(expr));
                    } else if self.current.kind == TokenKind::RightBrace {
                        final_expr = Some(expr);
                    } else {
                        statements.push(Stmt::Expr(expr));
                    }
                }
            }

            self.expect(TokenKind::RightBrace)?;
            let span = start_span.merge(&self.previous.span);

            // For iterator-style forall, wrap in a Block that represents the comprehension
            // Use a synthetic Inferred type since we don't know the element type statically
            let body = Expr::Block(Block {
                statements,
                final_expr: final_expr.map(Box::new),
                span: Span::default(),
            });

            // Create a ForallExpr using a placeholder type to indicate iterator-style
            // The underscore "_" indicates the type should be inferred from the iterator
            Ok(Expr::Forall(ForallExpr {
                var,
                type_: TypeExpr::Named("_".to_string()),
                body: Box::new(Expr::Binary {
                    left: Box::new(iter),
                    op: BinaryOp::Member,
                    right: Box::new(body),
                }),
                span,
            }))
        } else {
            Err(ParseError::UnexpectedToken {
                expected: ": or in".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            })
        }
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
            TokenKind::Char => {
                let value = self.current.lexeme.chars().next().unwrap_or('\0');
                self.advance();
                Ok(Pattern::Literal(Literal::Char(value)))
            }
            // Allow DOL keywords to be used as pattern identifiers
            TokenKind::Gene
            | TokenKind::Trait
            | TokenKind::System
            | TokenKind::Constraint
            | TokenKind::Rule
            | TokenKind::Evolves
            | TokenKind::Exegesis
            | TokenKind::Test
            | TokenKind::Law
            | TokenKind::State
            | TokenKind::Module
            | TokenKind::Use
            // Type keywords for matching type enum variants
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
            // v0.3.0 keywords that may appear as identifiers in patterns
            | TokenKind::Type
            | TokenKind::Val
            | TokenKind::Extends
            | TokenKind::Forall => {
                let name = self.current.lexeme.clone();
                self.advance();
                Ok(Pattern::Identifier(name))
            }
            TokenKind::Identifier => {
                let mut name = self.current.lexeme.clone();
                self.advance();

                // Handle path patterns like `Statement.Matches`
                while self.current.kind == TokenKind::Dot {
                    self.advance();
                    let part = self.expect_identifier()?;
                    name = format!("{}.{}", name, part);
                }

                // Check for constructor pattern with tuple args: `Some(x)`
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
                }
                // Check for struct destructuring pattern: `Foo { field: binding, field2 }`
                // Must distinguish from match arm body `Foo { stmt; }`.
                // Struct patterns have: `{ ident: binding }`, `{ ident, ident2 }`, etc.
                // Match arm bodies have: `{ expr.method() }`, `{ func() }`, `{ 0 }`, `{ None }`, etc.
                //
                // The heuristic for single-field patterns like `Foo { bar }`:
                // - If followed by another `{` (like `Foo { bar } { body }`), it's a struct pattern
                // - If followed by `:` or `,` in the braces, it's definitely a struct pattern
                // - Otherwise, it's ambiguous and we treat it as NOT a struct pattern
                //   (to avoid misinterpreting `None { x }` as struct pattern when it's a match arm)
                else if self.current.kind == TokenKind::LeftBrace
                    && self.peek().kind == TokenKind::Identifier
                    && !self.peek().lexeme.contains('.')
                    && !self.peek().lexeme.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && (matches!(self.peek2().kind, TokenKind::Colon | TokenKind::Comma)
                        || (self.peek2().kind == TokenKind::RightBrace
                            && self.peek3().kind == TokenKind::LeftBrace))
                {
                    self.advance();
                    let mut fields = Vec::new();
                    while self.current.kind != TokenKind::RightBrace
                        && self.current.kind != TokenKind::Eof
                    {
                        // Parse field name
                        let field_name = self.expect_identifier()?;
                        // Check for field rename pattern: `field: pattern`
                        if self.current.kind == TokenKind::Colon {
                            self.advance();
                            let pattern = self.parse_pattern()?;
                            // Store as a nested pattern with the field name
                            fields.push(Pattern::Constructor {
                                name: field_name,
                                fields: vec![pattern],
                            });
                        } else {
                            // Simple field binding: `field` (shorthand for `field: field`)
                            fields.push(Pattern::Identifier(field_name));
                        }

                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RightBrace)?;
                    // Use Constructor pattern with the struct name and field patterns
                    Ok(Pattern::Constructor { name, fields })
                } else if name == "true" {
                    Ok(Pattern::Literal(Literal::Bool(true)))
                } else if name == "false" {
                    Ok(Pattern::Literal(Literal::Bool(false)))
                } else if let Ok(int_val) = name.parse::<i64>() {
                    // Numeric pattern (lexer sends numbers as identifiers)
                    Ok(Pattern::Literal(Literal::Int(int_val)))
                } else if let Ok(float_val) = name.parse::<f64>() {
                    Ok(Pattern::Literal(Literal::Float(float_val)))
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

        Ok(Expr::SexBlock(Block {
            statements,
            final_expr,
            span: Span::default(),
        }))
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
                // Try to parse as expression, then check if it's an assignment
                let expr = self.parse_expr(0)?;

                // Check for assignment (e.g., a.b = expr)
                if self.current.kind == TokenKind::Equal {
                    self.advance(); // consume '='
                    let value = self.parse_expr(0)?;
                    self.consume_optional_semicolon();
                    statements.push(Stmt::Assign {
                        target: expr,
                        value,
                    });
                } else if self.current.kind == TokenKind::Semicolon {
                    // It's an expression statement
                    self.advance();
                    statements.push(Stmt::Expr(expr));
                } else if self.current.kind == TokenKind::RightBrace
                    || self.current.kind == TokenKind::Eof
                {
                    // It's the final expression (no semicolon needed before })
                    final_expr = Some(Box::new(expr));
                    break;
                } else {
                    // Assume it's a statement without semicolon (DOL style)
                    statements.push(Stmt::Expr(expr));
                }
            }
        }

        Ok(Expr::Block(Block {
            statements,
            final_expr,
            span: Span::default(),
        }))
    }

    /// Checks if a token kind can be used as an identifier (for struct field names, etc.)
    fn is_identifier_like(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Identifier
                | TokenKind::Gene
                | TokenKind::Trait
                | TokenKind::System
                | TokenKind::Constraint
                | TokenKind::Rule
                | TokenKind::Evolves
                | TokenKind::Exegesis
                | TokenKind::Test
                | TokenKind::Law
                | TokenKind::State
                | TokenKind::Module
                | TokenKind::Use
                | TokenKind::Uses
                | TokenKind::Migrate
                | TokenKind::Pub
                | TokenKind::Has
                | TokenKind::Is
                | TokenKind::Requires
                | TokenKind::Var
                | TokenKind::Let
                | TokenKind::Function
                | TokenKind::Return
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::Match
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Loop
                | TokenKind::In
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Where
                | TokenKind::True
                | TokenKind::False
                | TokenKind::From
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
                | TokenKind::Type
                | TokenKind::Val
                | TokenKind::Extends
                | TokenKind::Forall
        )
    }

    /// Checks if the current token is a statement keyword.
    fn is_statement_keyword(&self) -> bool {
        matches!(
            self.current.kind,
            TokenKind::Let
                | TokenKind::Val
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
    /// Consume a semicolon if present (DOL makes semicolons optional)
    fn consume_optional_semicolon(&mut self) {
        if self.current.kind == TokenKind::Semicolon {
            self.advance();
        }
    }

    /// Parses a single statement.
    pub fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.current.kind {
            TokenKind::Let => {
                self.advance();
                // Support `let _ = ...` discard pattern
                let name = if self.current.kind == TokenKind::Underscore {
                    self.advance();
                    "_".to_string()
                } else {
                    // Allow keywords as variable names (e.g., let exegesis = ...)
                    self.expect_identifier_or_keyword()?
                };

                let type_ann = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };

                // Value is optional if type annotation is provided (uninitialized declaration)
                let value = if self.current.kind == TokenKind::Equal {
                    self.advance();
                    self.parse_expr(0)?
                } else if type_ann.is_some() {
                    // Use a placeholder for uninitialized declarations with type annotations
                    // This will be generated as `let name: Type;` in Rust
                    Expr::Identifier("__uninitialized__".to_string())
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "= or type annotation".to_string(),
                        found: format!("'{}'", self.current.lexeme),
                        span: self.current.span,
                    });
                };
                self.consume_optional_semicolon();

                Ok(Stmt::Let {
                    name,
                    type_ann,
                    value,
                })
            }
            // val x: Type = expr (immutable binding, v0.3.0)
            TokenKind::Val => {
                self.advance();
                // Support `val _ = ...` discard pattern
                let name = if self.current.kind == TokenKind::Underscore {
                    self.advance();
                    "_".to_string()
                } else {
                    self.expect_identifier_or_keyword()?
                };

                let type_ann = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };

                // Value is optional if type annotation is provided
                let value = if self.current.kind == TokenKind::Equal {
                    self.advance();
                    self.parse_expr(0)?
                } else if type_ann.is_some() {
                    Expr::Identifier("__uninitialized__".to_string())
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "= or type annotation".to_string(),
                        found: format!("'{}'", self.current.lexeme),
                        span: self.current.span,
                    });
                };
                self.consume_optional_semicolon();

                // val is semantically equivalent to let (immutable)
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
                self.consume_optional_semicolon();

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
                self.consume_optional_semicolon();

                Ok(Stmt::Let {
                    name,
                    type_ann,
                    value,
                })
            }
            TokenKind::Sex => {
                // Parse as expression (sex block)
                let expr = self.parse_expr(0)?;
                self.consume_optional_semicolon();
                Ok(Stmt::Expr(expr))
            }
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Loop => self.parse_loop_stmt(),
            TokenKind::Break => {
                self.advance();
                self.consume_optional_semicolon();
                Ok(Stmt::Break)
            }
            TokenKind::Continue => {
                self.advance();
                self.consume_optional_semicolon();
                Ok(Stmt::Continue)
            }
            TokenKind::Return => {
                self.advance();
                let value = if self.current.kind != TokenKind::Semicolon
                    && self.current.kind != TokenKind::RightBrace
                    && self.current.kind != TokenKind::Eof
                {
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                self.consume_optional_semicolon();
                Ok(Stmt::Return(value))
            }
            _ => {
                // Try to parse as simple assignment (identifier = expr) first
                // This handles DOL's simple assignment syntax without 'let'
                // Use peek() to avoid consuming tokens we can't restore
                if self.current.kind == TokenKind::Identifier
                    && self.peek().kind == TokenKind::Equal
                {
                    let name = self.expect_identifier()?;
                    self.advance(); // consume '='
                    let value = self.parse_expr(0)?;
                    self.consume_optional_semicolon();
                    return Ok(Stmt::Assign {
                        target: Expr::Identifier(name),
                        value,
                    });
                }

                let expr = self.parse_expr(0)?;

                // Check for assignment after expression (e.g., a.b = expr or a[i] = expr)
                if self.current.kind == TokenKind::Equal {
                    self.advance(); // consume '='
                    let value = self.parse_expr(0)?;
                    self.consume_optional_semicolon();
                    return Ok(Stmt::Assign {
                        target: expr,
                        value,
                    });
                }

                self.consume_optional_semicolon();
                Ok(Stmt::Expr(expr))
            }
        }
    }

    /// Parses a for loop statement.
    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::For)?;

        // Allow DOL keywords as loop variable names (e.g., `for law in laws`)
        let binding = self.expect_identifier_or_keyword()?;
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
            // v0.8.0 NEW type keywords (lowercase)
            TokenKind::I8 => {
                self.advance();
                TypeExpr::Named("i8".to_string())
            }
            TokenKind::I16 => {
                self.advance();
                TypeExpr::Named("i16".to_string())
            }
            TokenKind::I32 => {
                self.advance();
                TypeExpr::Named("i32".to_string())
            }
            TokenKind::I64 => {
                self.advance();
                TypeExpr::Named("i64".to_string())
            }
            TokenKind::I128 => {
                self.advance();
                TypeExpr::Named("i128".to_string())
            }
            TokenKind::U8 => {
                self.advance();
                TypeExpr::Named("u8".to_string())
            }
            TokenKind::U16 => {
                self.advance();
                TypeExpr::Named("u16".to_string())
            }
            TokenKind::U32 => {
                self.advance();
                TypeExpr::Named("u32".to_string())
            }
            TokenKind::U64 => {
                self.advance();
                TypeExpr::Named("u64".to_string())
            }
            TokenKind::U128 => {
                self.advance();
                TypeExpr::Named("u128".to_string())
            }
            TokenKind::F32 => {
                self.advance();
                TypeExpr::Named("f32".to_string())
            }
            TokenKind::F64 => {
                self.advance();
                TypeExpr::Named("f64".to_string())
            }
            TokenKind::Bool => {
                self.advance();
                TypeExpr::Named("bool".to_string())
            }
            TokenKind::Str => {
                self.advance();
                TypeExpr::Named("string".to_string())
            }

            // DEPRECATED type keywords (uppercase) - emit warnings
            TokenKind::Int8 => {
                eprintln!("warning: 'Int8' type is deprecated in v0.8.0, use 'i8' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("i8".to_string())
            }
            TokenKind::Int16 => {
                eprintln!("warning: 'Int16' type is deprecated in v0.8.0, use 'i16' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("i16".to_string())
            }
            TokenKind::Int32 => {
                eprintln!("warning: 'Int32' type is deprecated in v0.8.0, use 'i32' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("i32".to_string())
            }
            TokenKind::Int64 => {
                eprintln!("warning: 'Int64' type is deprecated in v0.8.0, use 'i64' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("i64".to_string())
            }
            TokenKind::UInt8 => {
                eprintln!("warning: 'UInt8' type is deprecated in v0.8.0, use 'u8' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("u8".to_string())
            }
            TokenKind::UInt16 => {
                eprintln!("warning: 'UInt16' type is deprecated in v0.8.0, use 'u16' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("u16".to_string())
            }
            TokenKind::UInt32 => {
                eprintln!("warning: 'UInt32' type is deprecated in v0.8.0, use 'u32' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("u32".to_string())
            }
            TokenKind::UInt64 => {
                eprintln!("warning: 'UInt64' type is deprecated in v0.8.0, use 'u64' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("u64".to_string())
            }
            TokenKind::Float32 => {
                eprintln!("warning: 'Float32' type is deprecated in v0.8.0, use 'f32' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("f32".to_string())
            }
            TokenKind::Float64 => {
                eprintln!("warning: 'Float64' type is deprecated in v0.8.0, use 'f64' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("f64".to_string())
            }
            TokenKind::BoolType => {
                eprintln!("warning: 'Bool' type is deprecated in v0.8.0, use 'bool' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("bool".to_string())
            }
            TokenKind::StringType => {
                eprintln!("warning: 'String' type is deprecated in v0.8.0, use 'string' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Named("string".to_string())
            }
            TokenKind::VoidType => {
                eprintln!("warning: 'Void' type is deprecated in v0.8.0, use unit type '()' instead at line {}, column {}",
                    self.current.span.line, self.current.span.column);
                self.advance();
                TypeExpr::Tuple(Vec::new()) // Void maps to unit type
            }
            TokenKind::Bang => {
                self.advance();
                TypeExpr::Never
            }
            TokenKind::Identifier => {
                let name = self.expect_identifier()?;

                // Check for inline enum type: enum { A, B, C } or enum { A { x: Int }, B }
                if name == "enum" && self.current.kind == TokenKind::LeftBrace {
                    self.advance(); // consume '{'
                    let mut variants = Vec::new();
                    while self.current.kind != TokenKind::RightBrace
                        && self.current.kind != TokenKind::Eof
                    {
                        if self.current.kind == TokenKind::RightBrace {
                            break;
                        }
                        // Parse variant name (allow keywords as variant names)
                        let variant_name = self.expect_identifier_or_keyword()?;
                        let mut fields = Vec::new();
                        let mut tuple_types = Vec::new();
                        let mut discriminant = None;

                        // Check for tuple variant: Variant(T, U)
                        if self.current.kind == TokenKind::LeftParen {
                            self.advance(); // consume '('
                            while self.current.kind != TokenKind::RightParen
                                && self.current.kind != TokenKind::Eof
                            {
                                tuple_types.push(self.parse_type()?);
                                if self.current.kind == TokenKind::Comma {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            self.expect(TokenKind::RightParen)?;
                        }
                        // Check for struct fields: Variant { field: Type, ... }
                        else if self.current.kind == TokenKind::LeftBrace {
                            self.advance(); // consume '{'
                            while self.current.kind != TokenKind::RightBrace
                                && self.current.kind != TokenKind::Eof
                            {
                                let field_name = self.expect_identifier_or_keyword()?;
                                self.expect(TokenKind::Colon)?;
                                let field_type = self.parse_type()?;
                                fields.push((field_name, field_type));
                                if self.current.kind == TokenKind::Comma {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            self.expect(TokenKind::RightBrace)?;
                        }

                        // Check for discriminant value: Variant = 0
                        if self.current.kind == TokenKind::Equal {
                            self.advance(); // consume '='
                                            // Numeric values are tokenized as Identifier
                            if let Ok(val) = self.current.lexeme.parse::<i64>() {
                                discriminant = Some(val);
                            }
                            self.advance();
                        }

                        variants.push(EnumVariant {
                            name: variant_name,
                            fields,
                            tuple_types,
                            discriminant,
                        });
                        // Skip comma if present
                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RightBrace)?;
                    TypeExpr::Enum { variants }
                // Check for generic type
                } else if self.current.kind == TokenKind::Lt {
                    // v0.8.0: Emit deprecation warnings for List and Optional
                    if name == "List" {
                        eprintln!("warning: 'List<T>' is deprecated in v0.8.0, use 'Vec<T>' instead at line {}, column {}",
                            self.previous.span.line, self.previous.span.column);
                    } else if name == "Optional" {
                        eprintln!("warning: 'Optional<T>' is deprecated in v0.8.0, use 'Option<T>' instead at line {}, column {}",
                            self.previous.span.line, self.previous.span.column);
                    }

                    self.advance();
                    let mut args = Vec::new();
                    // Also check for Compose (>>) which can occur in nested generics
                    while self.current.kind != TokenKind::Greater
                        && self.current.kind != TokenKind::Compose
                        && self.current.kind != TokenKind::Eof
                    {
                        args.push(self.parse_type()?);
                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    // Use special method that handles >> splitting
                    self.expect_greater_in_type()?;

                    // Normalize deprecated type names in the AST
                    let normalized_name = if name == "List" {
                        "Vec"
                    } else if name == "Optional" {
                        "Option"
                    } else {
                        &name
                    };

                    TypeExpr::Generic {
                        name: normalized_name.to_string(),
                        args,
                    }
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
    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Function)?;

        // Allow DOL keywords as function names (e.g., `fun test()`)
        let name = self.expect_identifier_or_keyword()?;

        self.expect(TokenKind::LeftParen)?;
        let mut params = Vec::new();
        while self.current.kind != TokenKind::RightParen && self.current.kind != TokenKind::Eof {
            // Allow DOL keywords as parameter names (e.g., `gene: GeneDecl`)
            let param_name = self.expect_identifier_or_keyword()?;
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

        // Body is optional for sex fun (import declaration if no body)
        let body = if self.current.kind == TokenKind::LeftBrace {
            self.advance(); // consume '{'
            let mut stmts = Vec::new();
            while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof
            {
                stmts.push(self.parse_stmt()?);
            }
            self.expect(TokenKind::RightBrace)?;
            stmts
        } else {
            // No body - this is an import declaration (only valid for sex fun)
            Vec::new()
        };

        let span = start_span.merge(&self.previous.span);

        Ok(FunctionDecl {
            visibility: Visibility::default(),
            purity: Purity::default(),
            name,
            type_params: None,
            params,
            return_type,
            body,
            exegesis: String::new(),
            span,
            attributes: Vec::new(),
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
            // Allow DOL keywords as parameter names (e.g., `gene: GeneDecl`)
            let param_name = self.expect_identifier_or_keyword()?;
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
                .or_else(|| self.peeked2.take())
                .or_else(|| self.peeked3.take())
                .unwrap_or_else(|| self.lexer.next_token()),
        );
        // Shift peeked tokens down the chain
        if self.peeked.is_none() && self.peeked2.is_some() {
            self.peeked = self.peeked2.take();
        }
        if self.peeked2.is_none() && self.peeked3.is_some() {
            self.peeked2 = self.peeked3.take();
        }
    }

    /// Peeks at the next token without consuming it.
    fn peek(&mut self) -> &Token {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token());
        }
        self.peeked.as_ref().unwrap()
    }

    /// Peeks at the token after the next token (two-token lookahead).
    fn peek2(&mut self) -> &Token {
        // Ensure peeked is populated
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token());
        }
        // Ensure peeked2 is populated
        if self.peeked2.is_none() {
            self.peeked2 = Some(self.lexer.next_token());
        }
        self.peeked2.as_ref().unwrap()
    }

    /// Peeks at the third token ahead (three-token lookahead).
    fn peek3(&mut self) -> &Token {
        // Ensure all prior tokens are populated
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token());
        }
        if self.peeked2.is_none() {
            self.peeked2 = Some(self.lexer.next_token());
        }
        // Ensure peeked3 is populated
        if self.peeked3.is_none() {
            self.peeked3 = Some(self.lexer.next_token());
        }
        self.peeked3.as_ref().unwrap()
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

    /// Expects a `>` token in type context, also accepting `>>` and splitting it.
    /// This handles nested generics like `Option<Box<T>>` where `>>` is tokenized as one token.
    fn expect_greater_in_type(&mut self) -> Result<(), ParseError> {
        if self.current.kind == TokenKind::Greater {
            self.advance();
            Ok(())
        } else if self.current.kind == TokenKind::Compose {
            // >> becomes > after consuming one >
            // Create a new > token with updated span
            self.current = Token {
                kind: TokenKind::Greater,
                lexeme: ">".to_string(),
                span: Span {
                    start: self.current.span.start + 1,
                    end: self.current.span.end,
                    line: self.current.span.line,
                    column: self.current.span.column + 1,
                },
            };
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: ">".to_string(),
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

    /// Expects an identifier or a DOL keyword that can be used as a variable/function name.
    /// This allows keywords like `gene`, `trait`, `test`, etc. to be used as names.
    fn expect_identifier_or_keyword(&mut self) -> Result<String, ParseError> {
        match self.current.kind {
            TokenKind::Identifier
            | TokenKind::Gene
            | TokenKind::Trait
            | TokenKind::System
            | TokenKind::Constraint
            | TokenKind::Rule
            | TokenKind::Evolves
            | TokenKind::Exegesis
            | TokenKind::Test
            | TokenKind::Law
            | TokenKind::State
            | TokenKind::Module
            | TokenKind::Use
            | TokenKind::Uses
            // Additional keywords that can be used as identifiers
            | TokenKind::Migrate
            | TokenKind::Pub
            | TokenKind::Has
            | TokenKind::Is
            | TokenKind::Requires
            | TokenKind::Var
            | TokenKind::Let
            | TokenKind::Function
            | TokenKind::Return
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::Match
            | TokenKind::For
            | TokenKind::While
            | TokenKind::Loop
            | TokenKind::In
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Where
            | TokenKind::True
            | TokenKind::False
            // Evolution/migration keywords that can be field names
            | TokenKind::From
            // Type keywords
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
            // v0.3.0 keywords that can be used as identifiers
            | TokenKind::Type
            | TokenKind::Val
            | TokenKind::Extends
            | TokenKind::Forall
            // v0.9.0 Spirit keywords that can be used as identifiers
            | TokenKind::Spirit
            | TokenKind::Config => {
                let lexeme = self.current.lexeme.clone();
                self.advance();
                Ok(lexeme)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("'{}'", self.current.lexeme),
                span: self.current.span,
            }),
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
