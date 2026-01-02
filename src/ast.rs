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
//!     extends: None,
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

/// Visibility modifier for declarations.
///
/// Controls the accessibility of declarations across module boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Visibility {
    /// Private to the current module (default)
    #[default]
    Private,
    /// Public to all modules
    Public,
    /// Public to the spirit (same logical domain)
    PubSpirit,
    /// Public to the parent module
    PubParent,
}

/// Semantic version number.
///
/// Supports standard semver (major.minor.patch) plus optional suffix
/// for pre-release versions or geological time scales (e.g., "Gya").
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
    /// Optional suffix (e.g., "alpha", "beta", "Gya")
    pub suffix: Option<String>,
}

/// Module declaration: `module name.path @ version`.
///
/// Defines a module with a hierarchical path and optional version.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleDecl {
    /// Module path components (e.g., ["std", "io"])
    pub path: Vec<String>,
    /// Optional version number
    pub version: Option<Version>,
    /// Source location
    pub span: Span,
}

/// Use/import statement.
///
/// Imports declarations from other modules into the current scope.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UseDecl {
    /// Module path to import from
    pub path: Vec<String>,
    /// What items to import
    pub items: UseItems,
    /// Optional alias for the import
    pub alias: Option<String>,
    /// Source location
    pub span: Span,
}

/// What items are imported in a use declaration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UseItems {
    /// Import all items: `use module.*`
    All,
    /// Import specific named items: `use module.{item1, item2}`
    Named(Vec<UseItem>),
    /// Import the module itself: `use module`
    Single,
}

/// A single item in a named import list.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UseItem {
    /// Item name to import
    pub name: String,
    /// Optional alias for this item
    pub alias: Option<String>,
}

/// A complete DOL file with optional module declaration, imports, and declarations.
///
/// This represents the full contents of a .dol file.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DolFile {
    /// Optional module declaration at the start of the file
    pub module: Option<ModuleDecl>,
    /// Use/import declarations
    pub uses: Vec<UseDecl>,
    /// All declarations in the file
    pub declarations: Vec<Declaration>,
}

/// Purity marker for functions and operations.
///
/// Distinguishes pure (side-effect free) from impure (side-effecting) code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Purity {
    /// Pure function (no side effects)
    #[default]
    Pure,
    /// Side-effecting function
    Sex,
}

/// Mutability marker for variable declarations.
///
/// Controls whether a variable can be reassigned after initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Mutability {
    /// Immutable (let, const)
    #[default]
    Immutable,
    /// Mutable (var) - only in sex context
    Mutable,
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

    /// A top-level function declaration.
    Function(Box<FunctionDecl>),

    /// A sex var declaration - mutable global state.
    SexVar(VarDecl),

    /// A constant declaration - immutable compile-time value.
    Const(ConstDecl),
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
            Declaration::Function(f) => &f.name,
            Declaration::SexVar(v) => &v.name,
            Declaration::Const(c) => &c.name,
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
            Declaration::Function(f) => &f.exegesis,
            Declaration::SexVar(_) => "sex var",
            Declaration::Const(_) => "const",
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
            Declaration::Function(f) => f.span,
            Declaration::SexVar(v) => v.span,
            Declaration::Const(c) => c.span,
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
            Declaration::Evolution(_)
            | Declaration::Function(_)
            | Declaration::SexVar(_)
            | Declaration::Const(_) => return ids,
        };

        for stmt in statements {
            match stmt {
                Statement::Has {
                    subject, property, ..
                } => {
                    ids.push(subject.clone());
                    ids.push(property.clone());
                }
                Statement::HasField(field) => {
                    ids.push(field.name.clone());
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

    /// Optional parent type this gene extends (v0.3.0)
    pub extends: Option<String>,

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

    /// Typed field declaration: `subject has field: Type [= default] [where constraint]`
    HasField(Box<HasField>),

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

    /// Function declaration inside a gene or trait: `fun name(...) -> Type { ... }`
    Function(Box<FunctionDecl>),
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

// === DOL 2.0 Expression Types ===

/// Binary operator for expressions.
///
/// Represents operators that take two operands and produce a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BinaryOp {
    /// Addition `+`
    Add,
    /// Subtraction `-`
    Sub,
    /// Multiplication `*`
    Mul,
    /// Division `/`
    Div,
    /// Modulo `%`
    Mod,
    /// Exponentiation `^`
    Pow,
    /// Equality `==`
    Eq,
    /// Not equal `!=`
    Ne,
    /// Less than `<`
    Lt,
    /// Less than or equal `<=`
    Le,
    /// Greater than `>`
    Gt,
    /// Greater than or equal `>=`
    Ge,
    /// Logical and `&`
    And,
    /// Logical or `||`
    Or,
    /// Pipe `|>`
    Pipe,
    /// Compose `>>`
    Compose,
    /// Application `@`
    Apply,
    /// Bind `:=`
    Bind,
    /// Member access `.`
    Member,
    /// Functor map `<$>` - maps a function over a functor
    Map,
    /// Applicative apply `<*>` - applies a wrapped function to a wrapped value
    Ap,
    /// Logical implication `=>`
    Implies,
    /// Range `..`
    Range,
}

/// Unary operator for expressions.
///
/// Represents operators that take a single operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UnaryOp {
    /// Negation `-`
    Neg,
    /// Logical not `!`
    Not,
    /// Quote/AST capture `'`
    Quote,
    /// Type reflection `?`
    Reflect,
    /// Dereference `*`
    Deref,
}

/// Type expression for DOL 2.0.
///
/// Represents type annotations and type expressions in function signatures
/// and variable declarations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TypeExpr {
    /// Named type (e.g., `Int32`, `String`)
    Named(String),
    /// Generic type with arguments (e.g., `List<T>`, `Map<K, V>`)
    Generic {
        /// The type name
        name: String,
        /// Type arguments
        args: Vec<TypeExpr>,
    },
    /// Function type (e.g., `(Int32, String) -> Bool`)
    Function {
        /// Parameter types
        params: Vec<TypeExpr>,
        /// Return type
        return_type: Box<TypeExpr>,
    },
    /// Tuple type (e.g., `(Int32, String, Bool)`)
    Tuple(Vec<TypeExpr>),
    /// Never type (e.g., `!`) - indicates a function never returns
    Never,
    /// Inline enum type (e.g., `enum { A, B, C }` or `enum { A { x: Int }, B }`)
    Enum {
        /// Enum variants with optional fields
        variants: Vec<EnumVariant>,
    },
}

/// An enum variant with optional associated fields.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumVariant {
    /// Variant name
    pub name: String,
    /// Optional fields (for struct-like variants like `Variant { x: Int, y: Int }`)
    pub fields: Vec<(String, TypeExpr)>,
    /// Optional tuple types (for tuple variants like `Ok(T)`)
    pub tuple_types: Vec<TypeExpr>,
    /// Optional discriminant value (for variants like `None = 0`)
    pub discriminant: Option<i64>,
}

/// Type parameter: `<T>`, `<T: Trait>`, `<T: Trait + Other>`.
///
/// Represents a type parameter in a generic declaration with optional
/// bounds and default type.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeParam {
    /// Type parameter name (e.g., "T", "K", "V")
    pub name: String,
    /// Type bounds (trait constraints)
    pub bounds: Vec<TypeExpr>,
    /// Optional default type
    pub default: Option<TypeExpr>,
    /// Source location
    pub span: Span,
}

/// Type parameters list: `<T, U, V>`.
///
/// Represents a collection of type parameters for generic declarations.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeParams {
    /// List of type parameters
    pub params: Vec<TypeParam>,
    /// Source location
    pub span: Span,
}

/// Universal quantifier: `forall x: T. expr`.
///
/// Represents universal quantification in first-order logic.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ForallExpr {
    /// Bound variable name
    pub var: String,
    /// Type of the bound variable
    pub type_: TypeExpr,
    /// Body expression (the proposition)
    pub body: Box<Expr>,
    /// Source location
    pub span: Span,
}

/// Existential quantifier: `exists x: T. expr`.
///
/// Represents existential quantification in first-order logic.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExistsExpr {
    /// Bound variable name
    pub var: String,
    /// Type of the bound variable
    pub type_: TypeExpr,
    /// Body expression (the proposition)
    pub body: Box<Expr>,
    /// Source location
    pub span: Span,
}

/// Expression node for DOL 2.0.
///
/// Represents computational expressions that can be evaluated to produce values.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Variable or identifier reference
    Identifier(String),
    /// List literal: [expr, expr, ...]
    List(Vec<Expr>),
    /// Tuple literal: (expr, expr, ...)
    Tuple(Vec<Expr>),
    /// Binary operation
    Binary {
        /// Left operand
        left: Box<Expr>,
        /// Operator
        op: BinaryOp,
        /// Right operand
        right: Box<Expr>,
    },
    /// Unary operation
    Unary {
        /// Operator
        op: UnaryOp,
        /// Operand
        operand: Box<Expr>,
    },
    /// Function call
    Call {
        /// Function being called
        callee: Box<Expr>,
        /// Arguments
        args: Vec<Expr>,
    },
    /// Struct/Gene literal expression
    ///
    /// Represents a gene instance construction like `Point { x: 1.0, y: 2.0 }`
    ///
    /// # Example
    ///
    /// ```text
    /// Point { x: 1.0, y: 2.0 }
    /// geometry.Rectangle { width: 10, height: 20 }
    /// ```
    StructLiteral {
        /// The type name (e.g., "Point" or "geometry.Rectangle")
        type_name: String,
        /// Field name-value pairs in declaration order
        fields: Vec<(String, Expr)>,
    },
    /// Member access (field or method)
    Member {
        /// Object being accessed
        object: Box<Expr>,
        /// Field name
        field: String,
    },
    /// Lambda expression
    Lambda {
        /// Parameters with optional type annotations
        params: Vec<(String, Option<TypeExpr>)>,
        /// Optional return type
        return_type: Option<TypeExpr>,
        /// Lambda body
        body: Box<Expr>,
    },
    /// If expression
    If {
        /// Condition
        condition: Box<Expr>,
        /// Then branch
        then_branch: Box<Expr>,
        /// Optional else branch
        else_branch: Option<Box<Expr>>,
    },
    /// Pattern matching
    Match {
        /// Value being matched
        scrutinee: Box<Expr>,
        /// Match arms
        arms: Vec<MatchArm>,
    },
    /// Block expression
    Block {
        /// Statements in the block
        statements: Vec<Stmt>,
        /// Optional final expression (return value)
        final_expr: Option<Box<Expr>>,
    },
    /// Quote expression (AST capture)
    Quote(Box<Expr>),
    /// Unquote/splice - insert evaluated expr into quote
    /// Used inside quotes: '(1 + ,x) where x is evaluated
    Unquote(Box<Expr>),
    /// Quasi-quote - quote with splicing support
    QuasiQuote(Box<Expr>),
    /// Eval expression
    Eval(Box<Expr>),
    /// Type reflection
    Reflect(Box<TypeExpr>),
    /// Idiom brackets for applicative functor style
    /// [| f a b |] desugars to f <$> a <*> b
    IdiomBracket {
        /// The function to apply
        func: Box<Expr>,
        /// Arguments to lift and apply
        args: Vec<Expr>,
    },
    /// Universal quantification: `forall x: T. expr`
    Forall(ForallExpr),
    /// Existential quantification: `exists x: T. expr`
    Exists(ExistsExpr),
    /// Logical implication (can also be represented as Binary with Implies op)
    Implies {
        /// Left operand (antecedent)
        left: Box<Expr>,
        /// Right operand (consequent)
        right: Box<Expr>,
        /// Source location
        span: Span,
    },
    /// Sex block expression for side-effecting code
    /// Syntax: `sex { statements }`
    SexBlock {
        /// Statements in the sex block
        statements: Vec<Stmt>,
        /// Optional final expression (return value)
        final_expr: Option<Box<Expr>>,
    },
    /// Type cast expression
    /// Syntax: `expr as Type`
    Cast {
        /// Expression being cast
        expr: Box<Expr>,
        /// Target type
        target_type: TypeExpr,
    },
    /// Try expression (error propagation)
    /// Syntax: `expr?`
    Try(Box<Expr>),
}

/// Literal value.
///
/// Represents constant values in expressions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Floating-point literal
    Float(f64),
    /// String literal
    String(String),
    /// Character literal
    Char(char),
    /// Boolean literal
    Bool(bool),
    /// Null literal
    Null,
}

/// Runtime representation of quoted code.
///
/// This enum represents AST nodes that have been quoted and can be
/// manipulated at runtime. It mirrors the structure of `Expr` but is
/// designed for code-as-data scenarios.
///
/// # Example
///
/// ```rust
/// use metadol::ast::{QuotedExpr, Literal};
///
/// let quoted = QuotedExpr::Literal(Literal::Int(42));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QuotedExpr {
    /// Literal value
    Literal(Literal),
    /// Identifier
    Ident(String),
    /// Binary operation
    Binary {
        /// Binary operator
        op: BinaryOp,
        /// Left operand
        left: Box<QuotedExpr>,
        /// Right operand
        right: Box<QuotedExpr>,
    },
    /// Unary operation
    Unary {
        /// Unary operator
        op: UnaryOp,
        /// Operand
        operand: Box<QuotedExpr>,
    },
    /// Function call
    Call {
        /// Function being called
        callee: Box<QuotedExpr>,
        /// Arguments
        args: Vec<QuotedExpr>,
    },
    /// List of expressions
    List(Vec<QuotedExpr>),
    /// Lambda expression
    Lambda {
        /// Parameter names
        params: Vec<String>,
        /// Lambda body
        body: Box<QuotedExpr>,
    },
    /// If expression
    If {
        /// Condition
        condition: Box<QuotedExpr>,
        /// Then branch
        then_branch: Box<QuotedExpr>,
        /// Optional else branch
        else_branch: Option<Box<QuotedExpr>>,
    },
    /// Nested quote
    Quote(Box<QuotedExpr>),
    /// Unquote (splice)
    Unquote(Box<QuotedExpr>),
}

impl QuotedExpr {
    /// Convert an Expr to a QuotedExpr.
    ///
    /// This captures the structure of an expression as data,
    /// allowing it to be manipulated at runtime.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to quote
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::ast::{Expr, Literal, QuotedExpr};
    ///
    /// let expr = Expr::Literal(Literal::Int(42));
    /// let quoted = QuotedExpr::from_expr(&expr);
    /// ```
    pub fn from_expr(expr: &Expr) -> Self {
        match expr {
            Expr::Literal(lit) => QuotedExpr::Literal(lit.clone()),
            Expr::Identifier(name) => QuotedExpr::Ident(name.clone()),
            Expr::Binary { left, op, right } => QuotedExpr::Binary {
                op: *op,
                left: Box::new(QuotedExpr::from_expr(left)),
                right: Box::new(QuotedExpr::from_expr(right)),
            },
            Expr::Unary { op, operand } => QuotedExpr::Unary {
                op: *op,
                operand: Box::new(QuotedExpr::from_expr(operand)),
            },
            Expr::Call { callee, args } => QuotedExpr::Call {
                callee: Box::new(QuotedExpr::from_expr(callee)),
                args: args.iter().map(QuotedExpr::from_expr).collect(),
            },
            Expr::Lambda {
                params,
                body,
                return_type: _,
            } => QuotedExpr::Lambda {
                params: params.iter().map(|(name, _)| name.clone()).collect(),
                body: Box::new(QuotedExpr::from_expr(body)),
            },
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => QuotedExpr::If {
                condition: Box::new(QuotedExpr::from_expr(condition)),
                then_branch: Box::new(QuotedExpr::from_expr(then_branch)),
                else_branch: else_branch
                    .as_ref()
                    .map(|e| Box::new(QuotedExpr::from_expr(e))),
            },
            Expr::Quote(inner) => QuotedExpr::Quote(Box::new(QuotedExpr::from_expr(inner))),
            Expr::Unquote(inner) => QuotedExpr::Unquote(Box::new(QuotedExpr::from_expr(inner))),
            Expr::QuasiQuote(inner) => {
                // QuasiQuote is treated similar to Quote but allows unquotes
                QuotedExpr::Quote(Box::new(QuotedExpr::from_expr(inner)))
            }
            Expr::Forall(_) | Expr::Exists(_) | Expr::Implies { .. } | Expr::SexBlock { .. } => {
                // For logical expressions and sex blocks, convert to identifier (simplified)
                QuotedExpr::Ident(format!("{:?}", expr))
            }
            // For other expression types, convert to identifier (simplified)
            _ => QuotedExpr::Ident(format!("{:?}", expr)),
        }
    }

    /// Convert a QuotedExpr back to an Expr.
    ///
    /// This allows quoted code to be evaluated or further processed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::ast::{QuotedExpr, Literal};
    ///
    /// let quoted = QuotedExpr::Literal(Literal::Int(42));
    /// let expr = quoted.to_expr();
    /// ```
    pub fn to_expr(&self) -> Expr {
        match self {
            QuotedExpr::Literal(lit) => Expr::Literal(lit.clone()),
            QuotedExpr::Ident(name) => Expr::Identifier(name.clone()),
            QuotedExpr::Binary { op, left, right } => Expr::Binary {
                left: Box::new(left.to_expr()),
                op: *op,
                right: Box::new(right.to_expr()),
            },
            QuotedExpr::Unary { op, operand } => Expr::Unary {
                op: *op,
                operand: Box::new(operand.to_expr()),
            },
            QuotedExpr::Call { callee, args } => Expr::Call {
                callee: Box::new(callee.to_expr()),
                args: args.iter().map(|a| a.to_expr()).collect(),
            },
            QuotedExpr::List(exprs) => {
                // Convert list to a block with expressions
                Expr::Block {
                    statements: vec![],
                    final_expr: Some(Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("list".to_string())),
                        args: exprs.iter().map(|e| e.to_expr()).collect(),
                    })),
                }
            }
            QuotedExpr::Lambda { params, body } => Expr::Lambda {
                params: params.iter().map(|p| (p.clone(), None)).collect(),
                return_type: None,
                body: Box::new(body.to_expr()),
            },
            QuotedExpr::If {
                condition,
                then_branch,
                else_branch,
            } => Expr::If {
                condition: Box::new(condition.to_expr()),
                then_branch: Box::new(then_branch.to_expr()),
                else_branch: else_branch.as_ref().map(|e| Box::new(e.to_expr())),
            },
            QuotedExpr::Quote(inner) => Expr::Quote(Box::new(inner.to_expr())),
            QuotedExpr::Unquote(inner) => Expr::Unquote(Box::new(inner.to_expr())),
        }
    }
}

/// Match arm for pattern matching.
///
/// Represents a single arm in a match expression.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MatchArm {
    /// Pattern to match
    pub pattern: Pattern,
    /// Optional guard condition
    pub guard: Option<Box<Expr>>,
    /// Expression to evaluate if pattern matches
    pub body: Box<Expr>,
}

/// Pattern for pattern matching.
///
/// Represents patterns used in match expressions and destructuring.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Pattern {
    /// Wildcard pattern `_` (matches anything)
    Wildcard,
    /// Identifier pattern (binds to variable)
    Identifier(String),
    /// Literal pattern (matches exact value)
    Literal(Literal),
    /// Constructor pattern with fields
    Constructor {
        /// Constructor name
        name: String,
        /// Field patterns
        fields: Vec<Pattern>,
    },
    /// Tuple pattern
    Tuple(Vec<Pattern>),
    /// Or pattern (alternatives)
    Or(Vec<Pattern>),
}

/// Statement node for function bodies.
///
/// Represents imperative statements that perform actions or control flow.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Stmt {
    /// Let binding (variable declaration)
    Let {
        /// Variable name
        name: String,
        /// Optional type annotation
        type_ann: Option<TypeExpr>,
        /// Initial value
        value: Expr,
    },
    /// Assignment to existing variable
    Assign {
        /// Target of assignment
        target: Expr,
        /// Value being assigned
        value: Expr,
    },
    /// For loop
    For {
        /// Loop variable
        binding: String,
        /// Iterable expression
        iterable: Expr,
        /// Loop body
        body: Vec<Stmt>,
    },
    /// While loop
    While {
        /// Loop condition
        condition: Expr,
        /// Loop body
        body: Vec<Stmt>,
    },
    /// Infinite loop
    Loop {
        /// Loop body
        body: Vec<Stmt>,
    },
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Return statement
    Return(Option<Expr>),
    /// Expression statement
    Expr(Expr),
}

/// Function parameter with type annotation.
///
/// Represents a parameter in a `fun` declaration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FunctionParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_ann: TypeExpr,
}

/// Variable declaration for DOL 2.0 (`var`, `const`, `let` keywords).
///
/// Represents a variable binding with optional type annotation and value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VarDecl {
    /// Mutability (immutable for let/const, mutable for var)
    pub mutability: Mutability,
    /// Variable name
    pub name: String,
    /// Optional type annotation
    pub type_ann: Option<TypeExpr>,
    /// Optional initial value
    pub value: Option<Expr>,
    /// Source location
    pub span: Span,
}

/// A top-level constant declaration: `const NAME: Type = value`.
///
/// Constants are immutable values that are evaluated at compile time
/// and can be referenced throughout the module.
///
/// # DOL Syntax
///
/// ```dol
/// const PI: Float64 = 3.14159
/// const MAX_HOPS: Int32 = 100
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConstDecl {
    /// Constant name
    pub name: String,
    /// Optional type annotation
    pub type_ann: Option<TypeExpr>,
    /// Constant value (required for const declarations)
    pub value: Expr,
    /// Source location
    pub span: Span,
}

/// External function declaration for FFI.
///
/// Represents a `sex extern fun` declaration for foreign function interface.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExternDecl {
    /// Optional ABI specification (e.g., "C", "Rust")
    pub abi: Option<String>,
    /// Function name
    pub name: String,
    /// Function parameters
    pub params: Vec<FunctionParam>,
    /// Optional return type
    pub return_type: Option<TypeExpr>,
    /// Source location
    pub span: Span,
}

/// Function declaration for DOL 2.0 (`fun` keyword).
///
/// Represents a function definition inside a gene or trait body.
/// In DOL syntax, these are declared using the `fun` keyword.
///
/// # SEX System
///
/// Functions can be marked with `sex` to indicate they have side effects:
/// - `fun name()` - pure function (default)
/// - `sex fun name()` - impure function with side effects
///
/// # Visibility
///
/// Functions can have visibility modifiers:
/// - private (default)
/// - `pub` - public to all modules
/// - `pub(spirit)` - public to the same spirit
/// - `pub(parent)` - public to the parent module
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FunctionDecl {
    /// Visibility of the function
    pub visibility: Visibility,
    /// Purity of the function (pure or sex)
    pub purity: Purity,
    /// Function name
    pub name: String,
    /// Optional type parameters for generic functions
    pub type_params: Option<TypeParams>,
    /// Function parameters
    pub params: Vec<FunctionParam>,
    /// Optional return type
    pub return_type: Option<TypeExpr>,
    /// Function body (statements)
    pub body: Vec<Stmt>,
    /// Documentation for the function
    pub exegesis: String,
    /// Source location
    pub span: Span,
}

/// State declaration in a system.
///
/// Represents a stateful variable in a system declaration with optional default value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StateDecl {
    /// State variable name
    pub name: String,
    /// Type of the state variable
    pub type_: TypeExpr,
    /// Optional default value
    pub default: Option<Expr>,
    /// Source location
    pub span: Span,
}

/// Law declaration in a trait.
///
/// Represents a logical law or property that must hold for a trait.
/// Laws are expressed as logical propositions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LawDecl {
    /// Law name
    pub name: String,
    /// Parameters for the law (universally quantified variables)
    pub params: Vec<FunctionParam>,
    /// The law body (logical proposition)
    pub body: Expr,
    /// Optional exegesis explaining the law
    pub exegesis: Option<String>,
    /// Source location
    pub span: Span,
}

/// Field declaration in a gene with optional default value.
///
/// Represents a field in a gene declaration, potentially with default value
/// and constraint.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HasField {
    /// Field name
    pub name: String,
    /// Field type
    pub type_: TypeExpr,
    /// Optional default value
    pub default: Option<Expr>,
    /// Optional constraint on the field
    pub constraint: Option<Expr>,
    /// Source location
    pub span: Span,
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
            extends: None,
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
