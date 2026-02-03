//! Spirit manifest parsing for DOL v0.9.0.
//!
//! This module handles parsing of `Spirit.dol` manifest files that define
//! DOL packages (spirits). A spirit is a compilation unit that produces
//! a WASM binary.
//!
//! # Spirit.dol Format
//!
//! ```dol
//! spirit container @ 0.9.0
//!
//! docs "Container management spirit for VUDO"
//!
//! # Dependencies
//! use @univrs/std @ ^1.0
//! use @univrs/wasm-runtime @ ^0.5
//!
//! # Spirit configuration
//! config {
//!   entry: "lib.dol"
//!   target: wasm32
//!   features: ["async", "gc"]
//! }
//!
//! # Exported modules
//! pub mod lib
//! pub mod types
//! mod internal  # private
//! ```

use crate::ast::{ImportSource, Span, Version, Visibility};
use crate::error::ParseError;
use crate::lexer::{Lexer, Token, TokenKind};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A parsed Spirit.dol manifest.
///
/// A spirit is a DOL package that compiles to a single WASM binary.
/// It declares dependencies, configuration, and exported modules.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SpiritManifest {
    /// Spirit name (e.g., "container")
    pub name: String,
    /// Spirit version
    pub version: Version,
    /// Spirit documentation
    pub docs: Option<String>,
    /// Dependencies on other packages
    pub dependencies: Vec<Dependency>,
    /// Requirements with version constraints
    pub requirements: Vec<Requirement>,
    /// Spirit configuration
    pub config: SpiritConfig,
    /// Target-specific build settings
    pub targets: Option<TargetsConfig>,
    /// Build configuration (derived from targets)
    pub build_config: Option<BuildConfig>,
    /// Module declarations
    pub modules: Vec<ModuleExport>,
    /// Source span
    pub span: Span,
}

/// A dependency declaration in a spirit manifest.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Dependency {
    /// Import source (registry, git, https)
    pub source: ImportSource,
    /// Path within the package
    pub path: Vec<String>,
    /// Version constraint (e.g., "^1.0", ">=0.5.0")
    pub version_constraint: Option<String>,
    /// Source span
    pub span: Span,
}

/// Spirit configuration options.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SpiritConfig {
    /// Entry point file (default: "lib.dol")
    pub entry: String,
    /// Compilation target (default: "wasm32")
    pub target: String,
    /// Enabled features
    pub features: Vec<String>,
    /// Source span
    pub span: Span,
}

/// Build configuration for compilation targets.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BuildConfig {
    /// Rust edition (e.g., "2024", "2021")
    pub rust_edition: String,
    /// WASM target triple (e.g., "wasm32-unknown-unknown")
    pub wasm_target: String,
    /// Enable WASM optimizations
    pub optimize: bool,
    /// Enabled features for build
    pub features: Vec<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            rust_edition: "2021".to_string(),
            wasm_target: "wasm32-unknown-unknown".to_string(),
            optimize: false,
            features: Vec::new(),
        }
    }
}

/// Target-specific build settings.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TargetsConfig {
    /// Rust-specific settings
    pub rust_edition: Option<String>,
    /// WASM-specific settings
    pub wasm_target: Option<String>,
    /// WASM optimization flag
    pub wasm_optimize: bool,
    /// Source span
    pub span: Span,
}

/// A requirement declaration (dependency with version constraint).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Requirement {
    /// Package identifier (e.g., "@univrs/std")
    pub package: String,
    /// Version constraint (e.g., "^0.8", ">=1.0.0")
    pub version_constraint: String,
    /// Source span
    pub span: Span,
}

/// A module export declaration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleExport {
    /// Module visibility
    pub visibility: Visibility,
    /// Module name
    pub name: String,
    /// Source span
    pub span: Span,
}

impl SpiritManifest {
    /// Returns the spirit's qualified name with version.
    pub fn qualified_name(&self) -> String {
        format!("{} @ {}", self.name, self.version)
    }

    /// Returns the entry point file path.
    pub fn entry_file(&self) -> &str {
        if self.config.entry.is_empty() {
            "lib.dol"
        } else {
            &self.config.entry
        }
    }

    /// Returns the public modules.
    pub fn public_modules(&self) -> impl Iterator<Item = &ModuleExport> {
        self.modules
            .iter()
            .filter(|m| m.visibility == Visibility::Public)
    }

    /// Returns the build configuration, creating it from targets if needed.
    pub fn get_build_config(&self) -> BuildConfig {
        if let Some(ref bc) = self.build_config {
            return bc.clone();
        }

        // Derive from targets if available
        if let Some(ref targets) = self.targets {
            BuildConfig {
                rust_edition: targets
                    .rust_edition
                    .clone()
                    .unwrap_or_else(|| "2021".to_string()),
                wasm_target: targets
                    .wasm_target
                    .clone()
                    .unwrap_or_else(|| "wasm32-unknown-unknown".to_string()),
                optimize: targets.wasm_optimize,
                features: self.config.features.clone(),
            }
        } else {
            BuildConfig::default()
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref suffix) = self.suffix {
            write!(f, "-{}", suffix)?;
        }
        Ok(())
    }
}

/// Parser for Spirit.dol manifest files.
pub struct ManifestParser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    previous: Token,
}

impl<'a> ManifestParser<'a> {
    /// Creates a new manifest parser for the given source.
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();
        Self {
            lexer,
            current,
            previous: Token::default(),
        }
    }

    /// Parses a Spirit.dol manifest.
    pub fn parse(&mut self) -> Result<SpiritManifest, ParseError> {
        let start_span = self.current.span;

        // Parse: spirit <name> @ <version>
        self.expect(TokenKind::Spirit)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::At)?;
        let version = self.parse_version()?;

        let mut docs = None;
        let mut dependencies = Vec::new();
        let mut requirements = Vec::new();
        let mut config = SpiritConfig::default();
        let mut targets = None;
        let mut modules = Vec::new();

        // Parse the rest of the manifest
        while self.current.kind != TokenKind::Eof {
            match self.current.kind {
                TokenKind::Docs | TokenKind::Exegesis => {
                    self.advance();
                    docs = Some(self.expect_string()?);
                }
                TokenKind::Use => {
                    dependencies.push(self.parse_dependency()?);
                }
                TokenKind::Config => {
                    config = self.parse_config()?;
                }
                TokenKind::Identifier if self.current.lexeme == "targets" => {
                    targets = Some(self.parse_targets()?);
                }
                TokenKind::Requires => {
                    requirements.extend(self.parse_requires()?);
                }
                TokenKind::Pub => {
                    // pub mod <name>
                    self.advance();
                    self.expect(TokenKind::Module)?;
                    let mod_name = self.expect_identifier()?;
                    modules.push(ModuleExport {
                        visibility: Visibility::Public,
                        name: mod_name,
                        span: self.previous.span,
                    });
                }
                TokenKind::Module => {
                    // mod <name> (private)
                    self.advance();
                    let mod_name = self.expect_identifier()?;
                    modules.push(ModuleExport {
                        visibility: Visibility::Private,
                        name: mod_name,
                        span: self.previous.span,
                    });
                }
                _ => {
                    // Skip unknown tokens or comments
                    self.advance();
                }
            }
        }

        // Apply defaults
        if config.entry.is_empty() {
            config.entry = "lib.dol".to_string();
        }
        if config.target.is_empty() {
            config.target = "wasm32".to_string();
        }

        // Derive build_config from targets if present
        let build_config = targets.as_ref().map(|t| BuildConfig {
            rust_edition: t.rust_edition.clone().unwrap_or_else(|| "2021".to_string()),
            wasm_target: t
                .wasm_target
                .clone()
                .unwrap_or_else(|| "wasm32-unknown-unknown".to_string()),
            optimize: t.wasm_optimize,
            features: config.features.clone(),
        });

        let span = start_span.merge(&self.previous.span);

        Ok(SpiritManifest {
            name,
            version,
            docs,
            dependencies,
            requirements,
            config,
            targets,
            build_config,
            modules,
            span,
        })
    }

    /// Parses a dependency declaration.
    fn parse_dependency(&mut self) -> Result<Dependency, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Use)?;

        // Parse import source
        let (source, path) = self.parse_import_source()?;

        // Parse optional version constraint: @ <version>
        let version_constraint = if self.current.kind == TokenKind::At {
            self.advance();
            Some(self.parse_version_constraint()?)
        } else {
            None
        };

        let span = start_span.merge(&self.previous.span);

        Ok(Dependency {
            source,
            path,
            version_constraint,
            span,
        })
    }

    /// Parses an import source (registry, git, https, local).
    fn parse_import_source(&mut self) -> Result<(ImportSource, Vec<String>), ParseError> {
        if self.current.kind == TokenKind::At {
            self.advance();
            let prefix = self.expect_identifier()?;

            if prefix == "git" && self.current.kind == TokenKind::Colon {
                // @git:url
                self.advance();
                let url = self.parse_url()?;
                let reference = if self.current.kind == TokenKind::Colon {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                return Ok((ImportSource::Git { url, reference }, Vec::new()));
            } else if prefix == "https" && self.current.kind == TokenKind::Colon {
                // @https://url
                self.advance();
                if self.current.lexeme == "/" {
                    self.advance();
                    self.advance();
                }
                let url = format!("https://{}", self.parse_url()?);
                return Ok((ImportSource::Https { url, sha256: None }, Vec::new()));
            } else {
                // @org/package
                let org = prefix;
                self.expect(TokenKind::Slash)?;
                let package = self.expect_identifier()?;

                // Split package name if it contains dots
                let segments: Vec<&str> = package.split('.').collect();
                let package_name = segments[0].to_string();
                let path: Vec<String> = segments[1..].iter().map(|s| s.to_string()).collect();

                return Ok((
                    ImportSource::Registry {
                        org,
                        package: package_name,
                        version: None,
                    },
                    path,
                ));
            }
        }

        // Local import
        let ident = self.expect_identifier()?;
        let path: Vec<String> = ident.split('.').map(|s| s.to_string()).collect();
        Ok((ImportSource::Local, path))
    }

    /// Parses a version constraint (e.g., "^1.0", ">=0.5.0").
    fn parse_version_constraint(&mut self) -> Result<String, ParseError> {
        let mut constraint = String::new();

        // Operators like ^, >=, <=, ~
        if self.current.lexeme == "^"
            || self.current.lexeme == "~"
            || self.current.lexeme == ">"
            || self.current.lexeme == "<"
            || self.current.lexeme == "="
        {
            constraint.push_str(&self.current.lexeme);
            self.advance();

            // Handle >=, <=
            if self.current.lexeme == "=" {
                constraint.push_str(&self.current.lexeme);
                self.advance();
            }
        }

        // Parse version number
        let version = self.parse_version()?;
        constraint.push_str(&version.to_string());

        Ok(constraint)
    }

    /// Parses the targets block: targets { rust: {...}, wasm: {...} }
    fn parse_targets(&mut self) -> Result<TargetsConfig, ParseError> {
        let start_span = self.current.span;
        self.expect_identifier()?; // consume "targets"
        self.expect(TokenKind::LeftBrace)?;

        let mut rust_edition = None;
        let mut wasm_target = None;
        let mut wasm_optimize = false;

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            let key = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;

            match key.as_str() {
                "rust" => {
                    // Parse rust: { edition: "2024" }
                    self.expect(TokenKind::LeftBrace)?;
                    while self.current.kind != TokenKind::RightBrace
                        && self.current.kind != TokenKind::Eof
                    {
                        let rust_key = self.expect_identifier()?;
                        self.expect(TokenKind::Colon)?;

                        match rust_key.as_str() {
                            "edition" => {
                                rust_edition = Some(self.expect_string()?);
                            }
                            _ => {
                                self.skip_value()?;
                            }
                        }

                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RightBrace)?;
                }
                "wasm" => {
                    // Parse wasm: { optimize: true, target: "wasm32-unknown-unknown" }
                    self.expect(TokenKind::LeftBrace)?;
                    while self.current.kind != TokenKind::RightBrace
                        && self.current.kind != TokenKind::Eof
                    {
                        let wasm_key = self.expect_identifier()?;
                        self.expect(TokenKind::Colon)?;

                        match wasm_key.as_str() {
                            "optimize" => {
                                let val = self.expect_identifier()?;
                                wasm_optimize = val == "true";
                            }
                            "target" => {
                                wasm_target = Some(self.expect_string()?);
                            }
                            _ => {
                                self.skip_value()?;
                            }
                        }

                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RightBrace)?;
                }
                _ => {
                    self.skip_value()?;
                }
            }

            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RightBrace)?;

        let span = start_span.merge(&self.previous.span);

        Ok(TargetsConfig {
            rust_edition,
            wasm_target,
            wasm_optimize,
            span,
        })
    }

    /// Parses the requires block: requires { @org/package: "^1.0", ... }
    fn parse_requires(&mut self) -> Result<Vec<Requirement>, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Requires)?; // consume "requires"
        self.expect(TokenKind::LeftBrace)?;

        let mut requirements = Vec::new();

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            // Parse package identifier (e.g., @univrs/std or @univrs/wasm-runtime)
            let package = if self.current.kind == TokenKind::At {
                self.advance();
                let org = self.expect_identifier()?;
                self.expect(TokenKind::Slash)?;

                // Handle hyphenated package names (e.g., wasm-runtime)
                let mut pkg_parts = vec![self.expect_identifier()?];
                while self.current.kind == TokenKind::Minus {
                    self.advance();
                    pkg_parts.push("-".to_string());
                    pkg_parts.push(self.expect_identifier()?);
                }
                let pkg = pkg_parts.join("");

                format!("@{}/{}", org, pkg)
            } else {
                // Handle non-scoped package names
                let mut pkg_parts = vec![self.expect_identifier()?];
                while self.current.kind == TokenKind::Minus {
                    self.advance();
                    pkg_parts.push("-".to_string());
                    pkg_parts.push(self.expect_identifier()?);
                }
                pkg_parts.join("")
            };

            self.expect(TokenKind::Colon)?;
            let version_constraint = self.expect_string()?;

            let span = start_span.merge(&self.previous.span);

            requirements.push(Requirement {
                package,
                version_constraint,
                span,
            });

            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(requirements)
    }

    /// Parses the config block.
    fn parse_config(&mut self) -> Result<SpiritConfig, ParseError> {
        let start_span = self.current.span;
        self.expect(TokenKind::Config)?;
        self.expect(TokenKind::LeftBrace)?;

        let mut entry = String::new();
        let mut target = String::new();
        let mut features = Vec::new();

        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            let key = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;

            match key.as_str() {
                "entry" => {
                    entry = self.expect_string()?;
                }
                "target" => {
                    target = self.expect_identifier()?;
                }
                "features" => {
                    features = self.parse_string_array()?;
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

        Ok(SpiritConfig {
            entry,
            target,
            features,
            span,
        })
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

    /// Parses a URL path.
    fn parse_url(&mut self) -> Result<String, ParseError> {
        let mut parts = Vec::new();

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

        Ok(parts.join(""))
    }

    /// Parses a version number (expects a Version token).
    fn parse_version(&mut self) -> Result<Version, ParseError> {
        self.expect_version_token()
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
        self.current = self.lexer.next_token();
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
        // Accept keywords that can be used as identifiers in Spirit names
        if self.current.kind == TokenKind::Identifier || self.current.kind.is_keyword() {
            let lexeme = self.current.lexeme.clone();
            self.advance();
            Ok(lexeme)
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

    /// Expects a version token and parses it.
    fn expect_version_token(&mut self) -> Result<Version, ParseError> {
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

    /// Parses a version from a string like "1.2.3" or "1.2.3-alpha".
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
}

/// Parses a Spirit.dol manifest from source.
pub fn parse_spirit_manifest(source: &str) -> Result<SpiritManifest, ParseError> {
    let mut parser = ManifestParser::new(source);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let source = r#"
spirit test @ 0.1.0
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert_eq!(manifest.name, "test");
        assert_eq!(manifest.version.major, 0);
        assert_eq!(manifest.version.minor, 1);
        assert_eq!(manifest.version.patch, 0);
    }

    #[test]
    fn test_parse_manifest_with_docs() {
        let source = r#"
spirit container @ 0.9.0

docs "Container management spirit for VUDO"
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert_eq!(manifest.name, "container");
        assert_eq!(
            manifest.docs,
            Some("Container management spirit for VUDO".to_string())
        );
    }

    #[test]
    fn test_parse_manifest_with_modules() {
        let source = r#"
spirit myspirit @ 1.0.0

pub mod lib
pub mod types
mod internal
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert_eq!(manifest.modules.len(), 3);
        assert_eq!(manifest.modules[0].name, "lib");
        assert_eq!(manifest.modules[0].visibility, Visibility::Public);
        assert_eq!(manifest.modules[1].name, "types");
        assert_eq!(manifest.modules[2].name, "internal");
        assert_eq!(manifest.modules[2].visibility, Visibility::Private);
    }

    #[test]
    fn test_parse_manifest_with_config() {
        let source = r#"
spirit configured @ 2.0.0

config {
    entry: "main.dol"
    target: wasm32
    features: ["async", "gc"]
}
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert_eq!(manifest.config.entry, "main.dol");
        assert_eq!(manifest.config.target, "wasm32");
        assert_eq!(manifest.config.features, vec!["async", "gc"]);
    }

    #[test]
    fn test_entry_file_default() {
        let source = r#"spirit test @ 0.1.0"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert_eq!(manifest.entry_file(), "lib.dol");
    }

    #[test]
    fn test_qualified_name() {
        let source = r#"spirit mypackage @ 1.2.3"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert_eq!(manifest.qualified_name(), "mypackage @ 1.2.3");
    }

    #[test]
    fn test_public_modules() {
        let source = r#"
spirit test @ 0.1.0
pub mod a
mod b
pub mod c
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        let public: Vec<_> = manifest.public_modules().collect();
        assert_eq!(public.len(), 2);
        assert_eq!(public[0].name, "a");
        assert_eq!(public[1].name, "c");
    }

    #[test]
    fn test_parse_targets_block() {
        let source = r#"
spirit buildtest @ 1.0.0

targets {
    rust: { edition: "2024" }
    wasm: { optimize: true, target: "wasm32-unknown-unknown" }
}
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert!(manifest.targets.is_some());
        let targets = manifest.targets.unwrap();
        assert_eq!(targets.rust_edition, Some("2024".to_string()));
        assert_eq!(
            targets.wasm_target,
            Some("wasm32-unknown-unknown".to_string())
        );
        assert!(targets.wasm_optimize);
    }

    #[test]
    fn test_parse_requires_block() {
        let source = r#"
spirit deptest @ 1.0.0

requires {
    @univrs/std: "^0.8"
    @univrs/wasm-runtime: ">=0.5.0"
}
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        assert_eq!(manifest.requirements.len(), 2);
        assert_eq!(manifest.requirements[0].package, "@univrs/std");
        assert_eq!(manifest.requirements[0].version_constraint, "^0.8");
        assert_eq!(manifest.requirements[1].package, "@univrs/wasm-runtime");
        assert_eq!(manifest.requirements[1].version_constraint, ">=0.5.0");
    }

    #[test]
    fn test_get_build_config_from_targets() {
        let source = r#"
spirit buildtest @ 1.0.0

targets {
    rust: { edition: "2024" }
    wasm: { optimize: true, target: "wasm32-unknown-unknown" }
}

config {
    features: ["async", "gc"]
}
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        let build_config = manifest.get_build_config();
        assert_eq!(build_config.rust_edition, "2024");
        assert_eq!(build_config.wasm_target, "wasm32-unknown-unknown");
        assert!(build_config.optimize);
        assert_eq!(build_config.features, vec!["async", "gc"]);
    }

    #[test]
    fn test_build_config_defaults() {
        let source = r#"spirit simple @ 1.0.0"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");
        let build_config = manifest.get_build_config();
        assert_eq!(build_config.rust_edition, "2021");
        assert_eq!(build_config.wasm_target, "wasm32-unknown-unknown");
        assert!(!build_config.optimize);
        assert!(build_config.features.is_empty());
    }

    #[test]
    fn test_full_manifest_with_all_features() {
        let source = r#"
spirit complete @ 2.0.0

docs "A complete spirit manifest example"

requires {
    @univrs/std: "^0.8"
    @univrs/async: "^1.0"
}

targets {
    rust: { edition: "2024" }
    wasm: { optimize: true, target: "wasm32-unknown-unknown" }
}

config {
    entry: "main.dol"
    target: wasm32
    features: ["async", "gc", "simd"]
}

pub mod lib
pub mod api
mod internal
"#;
        let manifest = parse_spirit_manifest(source).expect("should parse");

        // Check basic info
        assert_eq!(manifest.name, "complete");
        assert_eq!(manifest.version.major, 2);
        assert_eq!(
            manifest.docs,
            Some("A complete spirit manifest example".to_string())
        );

        // Check requirements
        assert_eq!(manifest.requirements.len(), 2);
        assert_eq!(manifest.requirements[0].package, "@univrs/std");
        assert_eq!(manifest.requirements[1].package, "@univrs/async");

        // Check targets
        assert!(manifest.targets.is_some());
        let targets = manifest.targets.as_ref().unwrap();
        assert_eq!(targets.rust_edition, Some("2024".to_string()));
        assert!(targets.wasm_optimize);

        // Check config
        assert_eq!(manifest.config.entry, "main.dol");
        assert_eq!(manifest.config.features.len(), 3);

        // Check build config
        let build_config = manifest.get_build_config();
        assert_eq!(build_config.rust_edition, "2024");
        assert_eq!(build_config.wasm_target, "wasm32-unknown-unknown");
        assert!(build_config.optimize);

        // Check modules
        assert_eq!(manifest.modules.len(), 3);
        let public: Vec<_> = manifest.public_modules().collect();
        assert_eq!(public.len(), 2);
    }
}
