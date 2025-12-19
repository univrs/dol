//! DOL Abstract Syntax Tree
//!
//! Represents the parsed structure of DOL files. The AST is designed for:
//! - Direct serialization to JSON for tooling
//! - Traversal for code generation
//! - Validation and constraint checking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete DOL file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DolFile {
    /// Source file path
    pub path: Option<String>,
    /// The primary declaration in this file
    pub declaration: Declaration,
    /// Plain English exegesis
    pub exegesis: Option<Exegesis>,
    /// Span information for error reporting
    pub span: Span,
}

/// Source location tracking
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: if self.line <= other.line { self.column } else { other.column },
        }
    }
}

/// Top-level declaration types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Declaration {
    Gene(Gene),
    Trait(Trait),
    Constraint(Constraint),
    System(System),
    Evolves(Evolves),
    Test(Test),
}

/// Gene: atomic unit of truth
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Gene {
    pub name: QualifiedName,
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// Trait: composable behaviors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trait {
    pub name: QualifiedName,
    pub uses: Vec<QualifiedName>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// Constraint: invariants that must hold
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraint {
    pub name: QualifiedName,
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// System: top-level composition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct System {
    pub name: QualifiedName,
    pub version: Version,
    pub requires: Vec<Requirement>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// Evolution block: lineage record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evolves {
    pub target: QualifiedName,
    pub version: Version,
    pub from: Version,
    pub changes: Vec<Change>,
    pub because: Option<String>,
    pub migration: Option<Migration>,
    pub span: Span,
}

/// Test declaration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Test {
    pub name: QualifiedName,
    pub given: Vec<Condition>,
    pub when: Vec<Action>,
    pub then: Vec<Assertion>,
    pub always: bool,
    pub span: Span,
}

/// Qualified name: domain.property
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub segments: Vec<String>,
    pub span: Span,
}

impl QualifiedName {
    pub fn new(segments: Vec<String>, span: Span) -> Self {
        Self { segments, span }
    }

    pub fn to_string(&self) -> String {
        self.segments.join(".")
    }

    pub fn domain(&self) -> Option<&str> {
        self.segments.first().map(|s| s.as_str())
    }

    pub fn property(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("."))
    }
}

/// Semantic version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub span: Span,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, span: Span) -> Self {
        Self { major, minor, patch, span }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version requirement with operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requirement {
    pub name: QualifiedName,
    pub operator: VersionOp,
    pub version: Version,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VersionOp {
    Eq,      // =
    Gte,     // >=
    Gt,      // >
    Lte,     // <=
    Lt,      // <
}

/// A statement within a declaration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statement {
    pub subject: Subject,
    pub predicate: Predicate,
    pub object: Option<Object>,
    pub span: Span,
}

/// Subject of a statement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Subject {
    Identifier(String),
    Each(String),
    All(String),
    No(String),
}

/// Predicate (verb) of a statement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    Has,
    Is,
    DerivesFrom,
    Requires,
    Uses,
    Emits,
    Matches,
    Never,
    Via,
    Are,
}

/// Object of a statement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Object {
    Identifier(String),
    QualifiedName(QualifiedName),
    Phrase(Vec<String>),
    Negated(Box<Object>),
}

/// Evolution change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change {
    pub kind: ChangeKind,
    pub statement: Statement,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Adds,
    Deprecates,
    Removes,
}

/// Migration block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Migration {
    pub mappings: Vec<MigrationMapping>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationMapping {
    pub from: String,
    pub to: Vec<String>,
    pub span: Span,
}

/// Test condition (given clause)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    pub negated: bool,
    pub subject: String,
    pub state: Option<String>,
    pub span: Span,
}

/// Test action (when clause)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub subject: String,
    pub verb: String,
    pub object: Option<String>,
    pub span: Span,
}

/// Test assertion (then clause)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assertion {
    pub subject: String,
    pub predicate: String,
    pub object: Option<String>,
    pub span: Span,
}

/// Plain English exegesis section
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exegesis {
    pub content: String,
    pub paragraphs: Vec<String>,
    pub span: Span,
}

impl Exegesis {
    pub fn new(content: String, span: Span) -> Self {
        let paragraphs = content
            .split("\n\n")
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
        Self { content, paragraphs, span }
    }
}

/// Complete DOL repository
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DolRepository {
    pub genes: HashMap<String, Gene>,
    pub traits: HashMap<String, Trait>,
    pub constraints: HashMap<String, Constraint>,
    pub systems: HashMap<String, System>,
    pub evolutions: Vec<Evolves>,
    pub tests: HashMap<String, Test>,
}

impl DolRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, file: DolFile) {
        match file.declaration {
            Declaration::Gene(g) => {
                self.genes.insert(g.name.to_string(), g);
            }
            Declaration::Trait(t) => {
                self.traits.insert(t.name.to_string(), t);
            }
            Declaration::Constraint(c) => {
                self.constraints.insert(c.name.to_string(), c);
            }
            Declaration::System(s) => {
                self.systems.insert(s.name.to_string(), s);
            }
            Declaration::Evolves(e) => {
                self.evolutions.push(e);
            }
            Declaration::Test(t) => {
                self.tests.insert(t.name.to_string(), t);
            }
        }
    }

    /// Get all versions of a trait/gene in evolution order
    pub fn evolution_chain(&self, name: &str) -> Vec<&Evolves> {
        let mut chain: Vec<_> = self.evolutions
            .iter()
            .filter(|e| e.target.to_string() == name)
            .collect();
        chain.sort_by(|a, b| a.version.cmp(&b.version));
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_name_display() {
        let name = QualifiedName::new(
            vec!["container".to_string(), "exists".to_string()],
            Span::default(),
        );
        assert_eq!(name.to_string(), "container.exists");
        assert_eq!(name.domain(), Some("container"));
        assert_eq!(name.property(), Some("exists"));
    }

    #[test]
    fn version_ordering() {
        let v1 = Version::new(0, 0, 1, Span::default());
        let v2 = Version::new(0, 0, 2, Span::default());
        let v3 = Version::new(0, 1, 0, Span::default());
        
        assert!(v1 < v2);
        assert!(v2 < v3);
    }
}
