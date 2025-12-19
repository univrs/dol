//! Abstract Syntax Tree definitions for Metal DOL.
//!
//! This module defines the complete AST representation for parsed DOL files,
//! including genes, traits, constraints, systems, and evolution declarations.
//!
//! # Structure
//!
//! Every DOL file contains one primary declaration followed by an exegesis section:
//!
//! ```text
//! <declaration-type> <name> {
//!   <statements>
//! }
//!
//! exegesis {
//!   <plain english description>
//! }
//! ```
//!
//! # Example
//!
//! ```rust
//! use metadol::ast::{Declaration, Gene, Statement, Span};
//!
//! let gene = Gene {
//!     name: "container.exists".to_string(),
//!     statements: vec![
//!         Statement::Has {
//!             subject: "container".to_string(),
//!             property: "identity".to_string(),
//!             span: Span::default(),
//!         },
//!     ],
//!     exegesis: "A container is the fundamental unit.".to_string(),
//!     span: Span::default(),
//! };
//!
//! let decl = Declaration::Gene(gene);
//! ```

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Source location information for error reporting and tooling.
///
/// Spans track the byte offsets and line/column positions of AST nodes
/// in the original source, enabling precise error messages and IDE integration.
///
/// # Example
///
/// ```rust
/// use metadol::ast::Span;
///
/// let span = Span::new(0, 10, 1, 1);
/// assert_eq!(span.start, 0);
/// assert_eq!(span.end, 10);
/// assert_eq!(span.line, 1);
/// assert_eq!(span.column, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Span {
    /// Starting byte offset (inclusive)
    pub start: usize,
    /// Ending byte offset (exclusive)
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl Span {
    /// Creates a new span with the given positions.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting byte offset (inclusive)
    /// * `end` - Ending byte offset (exclusive)
    /// * `line` - Line number (1-indexed)
    /// * `column` - Column number (1-indexed)
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Merges two spans, creating a span that covers both.
    ///
    /// The resulting span starts at the earlier position and ends at the later.
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: if self.line <= other.line {
                self.column
            } else {
                other.column
            },
        }
    }

    /// Returns the length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if the span is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The top-level declaration types in Metal DOL.
///
/// Every DOL file contains exactly one primary declaration followed by
/// an exegesis section. This enum represents all possible declaration types.
///
/// # Variants
///
/// - [`Gene`]: Atomic units declaring fundamental truths
/// - [`Trait`]: Composable behaviors built from genes
/// - [`Constraint`]: Invariants that must always hold
/// - [`System`]: Top-level composition of subsystems
/// - [`Evolution`]: Lineage records of ontology changes
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Declaration {
    /// A gene declaration - the atomic unit of DOL.
    Gene(Gene),

    /// A trait declaration - composable behaviors.
    Trait(Trait),

    /// A constraint declaration - system invariants.
    Constraint(Constraint),

    /// A system declaration - top-level composition.
    System(System),

    /// An evolution declaration - version lineage.
    Evolution(Evolution),
}

impl Declaration {
    /// Returns the name of the declaration.
    pub fn name(&self) -> &str {
        match self {
            Declaration::Gene(g) => &g.name,
            Declaration::Trait(t) => &t.name,
            Declaration::Constraint(c) => &c.name,
            Declaration::System(s) => &s.name,
            Declaration::Evolution(e) => &e.name,
        }
    }

    /// Returns the exegesis text.
    pub fn exegesis(&self) -> &str {
        match self {
            Declaration::Gene(g) => &g.exegesis,
            Declaration::Trait(t) => &t.exegesis,
            Declaration::Constraint(c) => &c.exegesis,
            Declaration::System(s) => &s.exegesis,
            Declaration::Evolution(e) => &e.exegesis,
        }
    }

    /// Returns the span of the declaration.
    pub fn span(&self) -> Span {
        match self {
            Declaration::Gene(g) => g.span,
            Declaration::Trait(t) => t.span,
            Declaration::Constraint(c) => c.span,
            Declaration::System(s) => s.span,
            Declaration::Evolution(e) => e.span,
        }
    }

    /// Collects all identifiers referenced in this declaration.
    pub fn collect_identifiers(&self) -> Vec<String> {
        let mut ids = vec![self.name().to_string()];
        
        let statements = match self {
            Declaration::Gene(g) => &g.statements,
            Declaration::Trait(t) => &t.statements,
            Declaration::Constraint(c) => &c.statements,
            Declaration::System(s) => &s.statements,
            Declaration::Evolution(_) => return ids,
        };
        
        for stmt in statements {
            match stmt {
                Statement::Has { subject, property, .. } => {
                    ids.push(subject.clone());
                    ids.push(property.clone());
                }
                Statement::Is { subject, state, .. } => {
                    ids.push(subject.clone());
                    ids.push(state.clone());
                }
                Statement::Uses { reference, .. } => {
                    ids.push(reference.clone());
                }
                _ => {}
            }
        }
        
        ids
    }

    /// Collects all dependencies (uses references) in this declaration.
    pub fn collect_dependencies(&self) -> Vec<String> {
        let statements = match self {
            Declaration::Trait(t) => &t.statements,
            Declaration::System(s) => &s.statements,
            _ => return vec![],
        };
        
        statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Uses { reference, .. } = stmt {
                    Some(reference.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// A gene declaration representing atomic ontological truths.
///
/// Genes are the fundamental building blocks of DOL. They declare
/// properties that cannot be decomposed further.
///
/// # DOL Syntax
///
/// ```dol
/// gene container.exists {
///   container has identity
///   container has state
///   container has boundaries
/// }
///
/// exegesis {
///   A container is the fundamental unit of workload isolation.
/// }
/// ```
///
/// # Naming Convention
///
/// Genes use dot notation: `domain.property`
/// Examples: `container.exists`, `identity.cryptographic`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Gene {
    /// The fully qualified name using dot notation
    pub name: String,

    /// The declarative statements within the gene body
    pub statements: Vec<Statement>,

    /// The mandatory exegesis explaining intent and context
    pub exegesis: String,

    /// Source location for error reporting
    pub span: Span,
}

/// A trait declaration for composable behaviors.
///
/// Traits build on genes using `uses` statements and declare
/// behaviors that components exhibit.
///
/// # DOL Syntax
///
/// ```dol
/// trait container.lifecycle {
///   uses container.exists
///   uses identity.cryptographic
///
///   container is created
///   container is started
///   container is stopped
///   
///   each transition emits event
/// }
///
/// exegesis {
///   The container lifecycle defines the state machine.
/// }
/// ```
///
/// # Naming Convention
///
/// Traits use dot notation: `domain.behavior`
/// Examples: `container.lifecycle`, `node.discovery`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trait {
    /// The fully qualified name using dot notation
    pub name: String,

    /// The statements including uses and behavior declarations
    pub statements: Vec<Statement>,

    /// The mandatory exegesis
    pub exegesis: String,

    /// Source location
    pub span: Span,
}

/// A constraint declaration for system invariants.
///
/// Constraints define rules that must always hold true in the system.
///
/// # DOL Syntax
///
/// ```dol
/// constraint container.integrity {
///   container state matches declared state
///   container identity never changes
///   container boundaries are enforced
/// }
///
/// exegesis {
///   Container integrity ensures runtime matches declared ontology.
/// }
/// ```
///
/// # Naming Convention
///
/// Constraints use dot notation: `domain.invariant`
/// Examples: `container.integrity`, `cluster.consistency`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Constraint {
    /// The fully qualified name
    pub name: String,

    /// The constraint statements (matches, never, etc.)
    pub statements: Vec<Statement>,

    /// The mandatory exegesis
    pub exegesis: String,

    /// Source location
    pub span: Span,
}

/// A system declaration for top-level composition.
///
/// Systems combine multiple traits and constraints with version requirements.
///
/// # DOL Syntax
///
/// ```dol
/// system univrs.orchestrator @ 0.1.0 {
///   requires container.lifecycle >= 0.0.2
///   requires node.discovery >= 0.0.1
///
///   nodes discover peers via gossip
///   all operations are authenticated
/// }
///
/// exegesis {
///   The Univrs orchestrator is the primary system composition.
/// }
/// ```
///
/// # Naming Convention
///
/// Systems use dot notation: `product.component`
/// Examples: `univrs.orchestrator`, `univrs.scheduler`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct System {
    /// The fully qualified name
    pub name: String,

    /// The system version (semver)
    pub version: String,

    /// Version requirements for dependencies
    pub requirements: Vec<Requirement>,

    /// System-level statements
    pub statements: Vec<Statement>,

    /// The mandatory exegesis
    pub exegesis: String,

    /// Source location
    pub span: Span,
}

/// A version requirement for system dependencies.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Requirement {
    /// The referenced declaration name
    pub name: String,

    /// The version constraint operator (>=, >, =)
    pub constraint: String,

    /// The required version
    pub version: String,

    /// Source location
    pub span: Span,
}

/// An evolution declaration tracking version changes.
///
/// Evolutions record how declarations change over time, maintaining
/// an accumulative history.
///
/// # DOL Syntax
///
/// ```dol
/// evolves container.lifecycle @ 0.0.2 > 0.0.1 {
///   adds container is paused
///   adds container is resumed
///   
///   because "workload migration requires state preservation"
/// }
///
/// exegesis {
///   Version 0.0.2 extends the lifecycle for migration support.
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Evolution {
    /// The declaration being evolved
    pub name: String,

    /// The new version
    pub version: String,

    /// The parent version (lineage)
    pub parent_version: String,

    /// Statements being added
    pub additions: Vec<Statement>,

    /// Statements being deprecated
    pub deprecations: Vec<Statement>,

    /// Items being removed
    pub removals: Vec<String>,

    /// Rationale for the evolution (from `because`)
    pub rationale: Option<String>,

    /// The mandatory exegesis
    pub exegesis: String,

    /// Source location
    pub span: Span,
}

/// A statement within a DOL declaration.
///
/// Statements use simple predicates to declare relationships and properties.
/// The predicate determines the semantic meaning of the statement.
///
/// # Predicates
///
/// | Statement | Syntax | Example |
/// |-----------|--------|---------|
/// | Has | `subject has property` | `container has identity` |
/// | Is | `subject is state` | `container is created` |
/// | DerivesFrom | `subject derives from origin` | `identity derives from ed25519 keypair` |
/// | Requires | `subject requires requirement` | `identity requires no authority` |
/// | Uses | `uses reference` | `uses container.exists` |
/// | Emits | `action emits event` | `transition emits event` |
/// | Matches | `subject matches target` | `state matches declared state` |
/// | Never | `subject never action` | `identity never changes` |
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Statement {
    /// Property possession: `subject has property`
    Has {
        /// The entity that has the property
        subject: String,
        /// The property being possessed
        property: String,
        /// Source location
        span: Span,
    },

    /// State or behavior: `subject is state`
    Is {
        /// The entity in the state
        subject: String,
        /// The state or behavior
        state: String,
        /// Source location
        span: Span,
    },

    /// Origin relationship: `subject derives from origin`
    DerivesFrom {
        /// The entity that derives
        subject: String,
        /// The origin or source
        origin: String,
        /// Source location
        span: Span,
    },

    /// Dependency: `subject requires requirement`
    Requires {
        /// The entity with the requirement
        subject: String,
        /// What is required
        requirement: String,
        /// Source location
        span: Span,
    },

    /// Composition: `uses reference`
    Uses {
        /// The referenced declaration
        reference: String,
        /// Source location
        span: Span,
    },

    /// Event production: `action emits event`
    Emits {
        /// The action that produces the event
        action: String,
        /// The event being emitted
        event: String,
        /// Source location
        span: Span,
    },

    /// Equivalence constraint: `subject matches target`
    Matches {
        /// The actual value
        subject: String,
        /// The expected value
        target: String,
        /// Source location
        span: Span,
    },

    /// Negative constraint: `subject never action`
    Never {
        /// The entity being constrained
        subject: String,
        /// The forbidden action
        action: String,
        /// Source location
        span: Span,
    },

    /// Quantified statement: `each|all subject predicate`
    Quantified {
        /// The quantifier (each, all)
        quantifier: Quantifier,
        /// The rest of the statement
        phrase: String,
        /// Source location
        span: Span,
    },
}

/// Quantifier for statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Quantifier {
    /// "each" - applies to every individual
    Each,
    /// "all" - applies universally
    All,
}

impl std::fmt::Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Quantifier::Each => write!(f, "each"),
            Quantifier::All => write!(f, "all"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 10, 1, 1);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 5, 1, 1);
        let span2 = Span::new(10, 20, 2, 5);
        let merged = span1.merge(&span2);
        
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_declaration_name() {
        let gene = Gene {
            name: "container.exists".to_string(),
            statements: vec![],
            exegesis: "Test".to_string(),
            span: Span::default(),
        };
        let decl = Declaration::Gene(gene);
        
        assert_eq!(decl.name(), "container.exists");
    }

    #[test]
    fn test_collect_dependencies() {
        let trait_decl = Trait {
            name: "test.trait".to_string(),
            statements: vec![
                Statement::Uses {
                    reference: "dep.one".to_string(),
                    span: Span::default(),
                },
                Statement::Uses {
                    reference: "dep.two".to_string(),
                    span: Span::default(),
                },
                Statement::Is {
                    subject: "test".to_string(),
                    state: "active".to_string(),
                    span: Span::default(),
                },
            ],
            exegesis: "Test".to_string(),
            span: Span::default(),
        };
        
        let decl = Declaration::Trait(trait_decl);
        let deps = decl.collect_dependencies();
        
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"dep.one".to_string()));
        assert!(deps.contains(&"dep.two".to_string()));
    }
}
