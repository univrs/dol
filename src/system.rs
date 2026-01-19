//! System manifest parsing for DOL v0.9.0.
//!
//! This module handles parsing of `System.dol` manifest files that define
//! multi-spirit systems. A system composes multiple spirits with inter-spirit
//! communication bindings.
//!
//! # System.dol Format
//!
//! ```dol
//! system vudo-container-runtime @ 0.9.0
//!
//! docs "Container runtime system for VUDO OS"
//!
//! # Spirit dependencies
//! spirits {
//!   container: ./spirits/container @ ^0.9
//!   scheduler: ./spirits/scheduler @ ^0.9
//!   network: @univrs/vudo-network @ ^1.0
//! }
//!
//! # System configuration
//! config {
//!   entry: container
//!   runtime: vudo
//!   memory: 256MB
//!   capabilities: ["network", "fs"]
//! }
//!
//! # Inter-spirit bindings
//! bindings {
//!   container.events -> scheduler.queue
//!   scheduler.network -> network.send
//! }
//! ```

use crate::ast::{Span, Version};
use crate::error::ParseError;
use crate::lexer::{Lexer, Token, TokenKind};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A parsed System.dol manifest.
///
/// A system composes multiple spirits into a deployable unit with
/// inter-spirit communication bindings.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemManifest {
    /// System name (e.g., "vudo-container-runtime")
    pub name: String,
    /// System version
    pub version: Version,
    /// System documentation
    pub docs: Option<String>,
    /// Spirit dependencies
    pub spirits: Vec<SpiritDependency>,
    /// System configuration
    pub config: SystemConfig,
    /// Inter-spirit bindings
    pub bindings: Vec<Binding>,
    /// Source span
    pub span: Span,
}

/// A spirit dependency in a system manifest.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SpiritDependency {
    /// Local name for the spirit (used in bindings)
    pub name: String,
    /// Path to the spirit (local path or registry reference)
    pub path: String,
    /// Version constraint
    pub version_constraint: Option<String>,
    /// Source span
    pub span: Span,
}

/// System configuration options.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemConfig {
    /// Entry spirit (main spirit to start)
    pub entry: String,
    /// Target runtime (e.g., "vudo", "wasm")
    pub runtime: String,
    /// Memory limit (e.g., "256MB")
    pub memory: Option<String>,
    /// Required capabilities
    pub capabilities: Vec<String>,
    /// Source span
    pub span: Span,
}

/// An inter-spirit binding.
///
/// Bindings connect output ports of one spirit to input ports of another.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Binding {
    /// Source: spirit.port
    pub source: PortRef,
    /// Target: spirit.port
    pub target: PortRef,
    /// Source span
    pub span: Span,
}

/// A reference to a spirit port.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PortRef {
    /// Spirit name
    pub spirit: String,
    /// Port name
    pub port: String,
}

impl SystemManifest {
    /// Returns the system's qualified name with version.
    pub fn qualified_name(&self) -> String {
        format!("{} @ {}", self.name, self.version)
    }

    /// Returns the entry spirit.
    pub fn entry_spirit(&self) -> Option<&SpiritDependency> {
        self.spirits.iter().find(|s| s.name == self.config.entry)
    }

    /// Returns all spirit names.
    pub fn spirit_names(&self) -> impl Iterator<Item = &str> {
        self.spirits.iter().map(|s| s.name.as_str())
    }
}

impl std::fmt::Display for PortRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.spirit, self.port)
    }
}

/// Parser for System.dol manifest files.
pub struct SystemParser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    previous: Token,
    /// Lookahead token for peeking
    peeked: Option<Token>,
}

impl<'a> SystemParser<'a> {
    /// Creates a new system parser for the given source.
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();
        Self {
            lexer,
            current,
            previous: Token::default(),
            peeked: None,
        }
    }

    /// Parses a System.dol manifest.
    pub fn parse(&mut self) -> Result<SystemManifest, ParseError> {
        let start_span = self.current.span;

        // Parse: system <name> @ <version>
        self.expect(TokenKind::System)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::At)?;
        let version = self.parse_version()?;

        let mut docs = None;
        let mut spirits = Vec::new();
        let mut config = SystemConfig::default();
        let mut bindings = Vec::new();

        // Parse the rest of the manifest
        while self.current.kind != TokenKind::Eof {
            match self.current.kind {
                TokenKind::Docs | TokenKind::Exegesis => {
                    self.advance();
                    docs = Some(self.expect_string()?);
                }
                TokenKind::Identifier if self.current.lexeme == "spirits" => {
                    spirits = self.parse_spirits_block()?;
                }
                TokenKind::Config => {
                    config = self.parse_config()?;
                }
                TokenKind::Identifier if self.current.lexeme == "bindings" => {
                    bindings = self.parse_bindings_block()?;
                }
                _ => {
                    // Skip unknown tokens
                    self.advance();
                }
            }
        }

        // Apply defaults
        if config.runtime.is_empty() {
            config.runtime = "vudo".to_string();
        }

        let span = start_span.merge(&self.previous.span);

        Ok(SystemManifest {
            name,
            version,
            docs,
            spirits,
            config,
            bindings,
            span,
        })
    }

    /// Parses the spirits block.
    fn parse_spirits_block(&mut self) -> Result<Vec<SpiritDependency>, ParseError> {
        self.advance(); // consume "spirits"
        self.expect(TokenKind::LeftBrace)?;

        let mut spirits = Vec::new();

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            let start_span = self.current.span;
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;

            // Parse path (local path or registry reference)
            let path = self.parse_spirit_path()?;

            // Parse optional version constraint
            let version_constraint = if self.current.kind == TokenKind::At {
                self.advance();
                Some(self.parse_version_constraint()?)
            } else {
                None
            };

            let span = start_span.merge(&self.previous.span);

            spirits.push(SpiritDependency {
                name,
                path,
                version_constraint,
                span,
            });

            // Optional comma
            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(spirits)
    }

    /// Parses a spirit path (local or registry).
    ///
    /// Paths can be:
    /// - Local: `./foo/bar`
    /// - Registry: `@scope/package`
    fn parse_spirit_path(&mut self) -> Result<String, ParseError> {
        let mut path = String::new();

        // Handle local paths starting with ./
        if self.current.kind == TokenKind::Dot {
            path.push('.');
            self.advance();
            if self.current.kind == TokenKind::Slash {
                path.push('/');
                self.advance();
            }
        }
        // Handle registry paths starting with @
        else if self.current.kind == TokenKind::At {
            path.push('@');
            self.advance();
        }

        // First identifier (required after prefix)
        if self.current.kind == TokenKind::Identifier || self.current.kind.is_keyword() {
            path.push_str(&self.current.lexeme);
            self.advance();
        }

        // Continue with /identifier pairs only
        // This prevents consuming the next entry's name
        while self.current.kind == TokenKind::Slash {
            path.push('/');
            self.advance();
            if self.current.kind == TokenKind::Identifier || self.current.kind.is_keyword() {
                path.push_str(&self.current.lexeme);
                self.advance();
            }
        }

        Ok(path)
    }

    /// Parses the config block.
    fn parse_config(&mut self) -> Result<SystemConfig, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Config)?;
        self.expect(TokenKind::LeftBrace)?;

        let mut entry = String::new();
        let mut runtime = String::new();
        let mut memory = None;
        let mut capabilities = Vec::new();

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            let key = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;

            match key.as_str() {
                "entry" => {
                    entry = self.expect_identifier()?;
                }
                "runtime" => {
                    runtime = self.expect_identifier()?;
                }
                "memory" => {
                    memory = Some(self.parse_memory_value()?);
                }
                "capabilities" => {
                    capabilities = self.parse_string_array()?;
                }
                _ => {
                    // Skip unknown config keys
                    self.skip_value()?;
                }
            }

            // Optional comma
            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RightBrace)?;

        let span = start_span.merge(&self.previous.span);

        Ok(SystemConfig {
            entry,
            runtime,
            memory,
            capabilities,
            span,
        })
    }

    /// Parses the bindings block.
    fn parse_bindings_block(&mut self) -> Result<Vec<Binding>, ParseError> {
        self.advance(); // consume "bindings"
        self.expect(TokenKind::LeftBrace)?;

        let mut bindings = Vec::new();

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            let start_span = self.current.span;

            // Parse source: spirit.port
            let source = self.parse_port_ref()?;

            // Expect ->
            self.expect(TokenKind::Arrow)?;

            // Parse target: spirit.port
            let target = self.parse_port_ref()?;

            let span = start_span.merge(&self.previous.span);

            bindings.push(Binding {
                source,
                target,
                span,
            });

            // Optional comma
            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(bindings)
    }

    /// Parses a port reference: spirit.port
    ///
    /// The lexer tokenizes "spirit.port" as a single qualified identifier,
    /// so we need to split on the dot.
    fn parse_port_ref(&mut self) -> Result<PortRef, ParseError> {
        let qualified = self.expect_identifier()?;

        // Split qualified identifier on dot
        if let Some(dot_pos) = qualified.find('.') {
            let spirit = qualified[..dot_pos].to_string();
            let port = qualified[dot_pos + 1..].to_string();
            Ok(PortRef { spirit, port })
        } else {
            Err(ParseError::InvalidStatement {
                message: format!(
                    "expected qualified identifier 'spirit.port', found '{}'",
                    qualified
                ),
                span: self.previous.span,
            })
        }
    }

    /// Parses a memory value (e.g., "256MB").
    fn parse_memory_value(&mut self) -> Result<String, ParseError> {
        if self.current.kind == TokenKind::String {
            let s = self.current.lexeme.trim_matches('"').to_string();
            self.advance();
            Ok(s)
        } else {
            // Try to parse as identifier (e.g., 256MB without quotes)
            let value = self.expect_identifier()?;
            Ok(value)
        }
    }

    /// Parses a string array: ["a", "b", "c"]
    fn parse_string_array(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(TokenKind::LeftBracket)?;
        let mut items = Vec::new();

        while self.current.kind != TokenKind::RightBracket && self.current.kind != TokenKind::Eof {
            items.push(self.expect_string()?);
            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RightBracket)?;
        Ok(items)
    }

    /// Parses a version constraint.
    fn parse_version_constraint(&mut self) -> Result<String, ParseError> {
        let mut constraint = String::new();

        // Operators like ^, >=, <=, ~
        while self.current.lexeme == "^"
            || self.current.lexeme == "~"
            || self.current.lexeme == ">"
            || self.current.lexeme == "<"
            || self.current.lexeme == "="
        {
            constraint.push_str(&self.current.lexeme);
            self.advance();
        }

        // Parse version number
        let version = self.parse_version()?;
        constraint.push_str(&version.to_string());

        Ok(constraint)
    }

    /// Parses a version number (expects a Version token).
    fn parse_version(&mut self) -> Result<Version, ParseError> {
        if self.current.kind == TokenKind::Version {
            let version_str = self.current.lexeme.clone();
            self.advance();
            self.parse_version_from_string(&version_str)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "version".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            })
        }
    }

    /// Parses a version from a string.
    fn parse_version_from_string(&self, version_str: &str) -> Result<Version, ParseError> {
        let mut parts = version_str.splitn(2, '-');
        let numbers_part = parts.next().unwrap();
        let suffix = parts.next().map(|s| s.to_string());

        let numbers: Vec<&str> = numbers_part.split('.').collect();
        if numbers.len() != 3 {
            return Err(ParseError::InvalidStatement {
                message: "version must have three parts (X.Y.Z)".to_string(),
                span: self.current.span,
            });
        }

        Ok(Version {
            major: numbers[0].parse().unwrap_or(0),
            minor: numbers[1].parse().unwrap_or(0),
            patch: numbers[2].parse().unwrap_or(0),
            suffix,
        })
    }

    /// Skips a value (for unknown config keys).
    fn skip_value(&mut self) -> Result<(), ParseError> {
        match self.current.kind {
            TokenKind::String | TokenKind::Identifier | TokenKind::Version => {
                self.advance();
            }
            TokenKind::LeftBracket => {
                self.advance();
                while self.current.kind != TokenKind::RightBracket
                    && self.current.kind != TokenKind::Eof
                {
                    self.advance();
                }
                self.advance();
            }
            TokenKind::LeftBrace => {
                self.advance();
                let mut depth = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    if self.current.kind == TokenKind::LeftBrace {
                        depth += 1;
                    } else if self.current.kind == TokenKind::RightBrace {
                        depth -= 1;
                    }
                    self.advance();
                }
            }
            _ => {
                self.advance();
            }
        }
        Ok(())
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        if let Some(peeked) = self.peeked.take() {
            self.current = peeked;
        } else {
            self.current = self.lexer.next_token();
        }
    }

    /// Peeks at the next token without consuming it.
    fn peek(&mut self) -> &Token {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token());
        }
        self.peeked.as_ref().unwrap()
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

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        // Accept keywords that can be used as identifiers
        if self.current.kind == TokenKind::Identifier || self.current.kind.is_keyword() {
            let mut name = self.current.lexeme.clone();
            self.advance();

            // Handle hyphenated names (e.g., container-runtime)
            // Only consume minus if followed by an identifier
            while self.current.kind == TokenKind::Minus {
                let next_kind = self.peek().kind;
                if next_kind == TokenKind::Identifier || next_kind.is_keyword() {
                    self.advance(); // consume the minus
                    name.push('-');
                    name.push_str(&self.current.lexeme);
                    self.advance(); // consume the identifier
                } else {
                    // Minus not followed by identifier, stop here
                    break;
                }
            }

            Ok(name)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            })
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        if self.current.kind == TokenKind::String {
            let s = self.current.lexeme.trim_matches('"').to_string();
            self.advance();
            Ok(s)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "string".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            })
        }
    }
}

/// Parses a System.dol manifest from source.
pub fn parse_system_manifest(source: &str) -> Result<SystemManifest, ParseError> {
    let mut parser = SystemParser::new(source);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_system() {
        let source = r#"
system test @ 0.1.0
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        assert_eq!(manifest.name, "test");
        assert_eq!(manifest.version.major, 0);
        assert_eq!(manifest.version.minor, 1);
        assert_eq!(manifest.version.patch, 0);
    }

    #[test]
    fn test_parse_system_with_docs() {
        let source = r#"
system container-runtime @ 0.9.0

docs "Container runtime system for VUDO OS"
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        assert_eq!(manifest.name, "container-runtime");
        assert_eq!(
            manifest.docs,
            Some("Container runtime system for VUDO OS".to_string())
        );
    }

    #[test]
    fn test_parse_system_with_spirits() {
        let source = r#"
system myapp @ 1.0.0

spirits {
    core: ./spirits/core
    utils: ./spirits/utils
}
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        assert_eq!(manifest.spirits.len(), 2);
        assert_eq!(manifest.spirits[0].name, "core");
        assert_eq!(manifest.spirits[0].path, "./spirits/core");
        assert_eq!(manifest.spirits[1].name, "utils");
    }

    #[test]
    fn test_parse_system_with_config() {
        let source = r#"
system configured @ 2.0.0

config {
    entry: main
    runtime: vudo
    capabilities: ["network", "fs"]
}
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        assert_eq!(manifest.config.entry, "main");
        assert_eq!(manifest.config.runtime, "vudo");
        assert_eq!(manifest.config.capabilities, vec!["network", "fs"]);
    }

    #[test]
    fn test_parse_bindings_only() {
        // Simpler test without spirits to isolate bindings parsing
        let source = r#"
system connected @ 1.0.0

bindings {
    producer.output -> consumer.input
}
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        assert_eq!(manifest.bindings.len(), 1);
        assert_eq!(manifest.bindings[0].source.spirit, "producer");
        assert_eq!(manifest.bindings[0].source.port, "output");
        assert_eq!(manifest.bindings[0].target.spirit, "consumer");
        assert_eq!(manifest.bindings[0].target.port, "input");
    }

    #[test]
    fn test_parse_system_with_bindings() {
        let source = r#"
system connected @ 1.0.0

spirits {
    producer: ./producer
    consumer: ./consumer
}

bindings {
    producer.output -> consumer.input
}
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        assert_eq!(manifest.bindings.len(), 1);
        assert_eq!(manifest.bindings[0].source.spirit, "producer");
        assert_eq!(manifest.bindings[0].source.port, "output");
        assert_eq!(manifest.bindings[0].target.spirit, "consumer");
        assert_eq!(manifest.bindings[0].target.port, "input");
    }

    #[test]
    fn test_qualified_name() {
        let source = r#"system myapp @ 1.2.3"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        assert_eq!(manifest.qualified_name(), "myapp @ 1.2.3");
    }

    #[test]
    fn test_spirit_names() {
        let source = r#"
system test @ 1.0.0

spirits {
    a: ./a
    b: ./b
    c: ./c
}
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        let names: Vec<_> = manifest.spirit_names().collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_entry_spirit() {
        let source = r#"
system test @ 1.0.0

spirits {
    main: ./main
    helper: ./helper
}

config {
    entry: main
}
"#;
        let manifest = parse_system_manifest(source).expect("should parse");
        let entry = manifest.entry_spirit().expect("should have entry");
        assert_eq!(entry.name, "main");
    }
}
