//! HIR type definitions.
//!
//! This module contains the core HIR node types:
//! - [`HirModule`] - Top-level compilation unit
//! - [`HirDecl`] - Declaration forms
//! - [`HirExpr`] - Expression forms
//! - [`HirStmt`] - Statement forms
//! - [`HirType`] - Type forms
//! - [`HirPat`] - Pattern forms

use super::span::HirId;
use super::symbol::Symbol;
use crate::ast::Visibility;

/// Top-level compilation unit.
///
/// A module contains declarations and tracks metadata about the source file.
#[derive(Debug, Clone, PartialEq)]
pub struct HirModule {
    /// Unique identifier for this module
    pub id: HirId,
    /// Module name (from module declaration or filename)
    pub name: Symbol,
    /// Top-level declarations
    pub decls: Vec<HirDecl>,
}

/// Declaration forms (4 total).
///
/// All DOL declarations desugar to one of these forms:
/// - Type declarations (gene, struct, enum)
/// - Trait declarations (trait, constraint)
/// - Function declarations (fun, method)
/// - Module declarations (module, system)
#[derive(Debug, Clone, PartialEq)]
pub enum HirDecl {
    /// Type declaration (gene, struct, enum)
    Type(HirTypeDecl),
    /// Trait declaration (trait, constraint)
    Trait(HirTraitDecl),
    /// Function declaration
    Function(HirFunctionDecl),
    /// Nested module declaration
    Module(HirModuleDecl),
}

/// Type declaration node.
#[derive(Debug, Clone, PartialEq)]
pub struct HirTypeDecl {
    /// Unique identifier
    pub id: HirId,
    /// Type name
    pub name: Symbol,
    /// Visibility modifier
    pub visibility: Visibility,
    /// Type parameters
    pub type_params: Vec<HirTypeParam>,
    /// Type body/definition
    pub body: HirTypeDef,
}

/// Trait declaration node.
#[derive(Debug, Clone, PartialEq)]
pub struct HirTraitDecl {
    /// Unique identifier
    pub id: HirId,
    /// Trait name
    pub name: Symbol,
    /// Visibility modifier
    pub visibility: Visibility,
    /// Type parameters
    pub type_params: Vec<HirTypeParam>,
    /// Super traits (bounds)
    pub bounds: Vec<HirType>,
    /// Trait items (methods, associated types)
    pub items: Vec<HirTraitItem>,
}

/// Function declaration node.
#[derive(Debug, Clone, PartialEq)]
pub struct HirFunctionDecl {
    /// Unique identifier
    pub id: HirId,
    /// Function name
    pub name: Symbol,
    /// Visibility modifier
    pub visibility: Visibility,
    /// Type parameters
    pub type_params: Vec<HirTypeParam>,
    /// Function parameters
    pub params: Vec<HirParam>,
    /// Return type
    pub return_type: HirType,
    /// Function body (None for external functions)
    pub body: Option<HirExpr>,
}

/// Nested module declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct HirModuleDecl {
    /// Unique identifier
    pub id: HirId,
    /// Module name
    pub name: Symbol,
    /// Visibility modifier
    pub visibility: Visibility,
    /// Module contents
    pub decls: Vec<HirDecl>,
}

/// Type parameter declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct HirTypeParam {
    /// Parameter name
    pub name: Symbol,
    /// Bounds on the type parameter
    pub bounds: Vec<HirType>,
}

/// Type definition body.
#[derive(Debug, Clone, PartialEq)]
pub enum HirTypeDef {
    /// Alias to another type
    Alias(HirType),
    /// Struct with named fields
    Struct(Vec<HirField>),
    /// Enum with variants
    Enum(Vec<HirVariant>),
    /// Gene definition (DOL-specific)
    Gene(Vec<HirStatement>),
}

/// Struct field definition.
#[derive(Debug, Clone, PartialEq)]
pub struct HirField {
    /// Field name
    pub name: Symbol,
    /// Field type
    pub ty: HirType,
    /// Visibility modifier
    pub visibility: Visibility,
}

/// Enum variant definition.
#[derive(Debug, Clone, PartialEq)]
pub struct HirVariant {
    /// Variant name
    pub name: Symbol,
    /// Variant payload (if any)
    pub payload: Option<HirType>,
}

/// Trait item (method or associated type).
#[derive(Debug, Clone, PartialEq)]
pub enum HirTraitItem {
    /// Method signature/definition
    Method(HirFunctionDecl),
    /// Associated type
    AssocType(HirAssocType),
}

/// Associated type in a trait.
#[derive(Debug, Clone, PartialEq)]
pub struct HirAssocType {
    /// Type name
    pub name: Symbol,
    /// Bounds on the type
    pub bounds: Vec<HirType>,
    /// Default value (if any)
    pub default: Option<HirType>,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    /// Parameter pattern
    pub pat: HirPat,
    /// Parameter type
    pub ty: HirType,
}

/// DOL statement (gene body statements).
#[derive(Debug, Clone, PartialEq)]
pub struct HirStatement {
    /// Unique identifier
    pub id: HirId,
    /// Statement kind
    pub kind: HirStatementKind,
}

/// Statement kinds for gene bodies.
#[derive(Debug, Clone, PartialEq)]
pub enum HirStatementKind {
    /// subject has property
    Has {
        /// The subject of the statement
        subject: Symbol,
        /// The property being declared
        property: Symbol,
    },
    /// subject is type
    Is {
        /// The subject of the statement
        subject: Symbol,
        /// The type name
        type_name: Symbol,
    },
    /// subject derives_from parent
    DerivesFrom {
        /// The subject of the statement
        subject: Symbol,
        /// The parent being derived from
        parent: Symbol,
    },
    /// subject requires dependency
    Requires {
        /// The subject of the statement
        subject: Symbol,
        /// The required dependency
        dependency: Symbol,
    },
    /// subject uses resource
    Uses {
        /// The subject of the statement
        subject: Symbol,
        /// The resource being used
        resource: Symbol,
    },
}

/// Expression forms (12 total).
#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    /// Literal value
    Literal(HirLiteral),
    /// Variable reference
    Var(Symbol),
    /// Binary operation
    Binary(Box<HirBinaryExpr>),
    /// Unary operation
    Unary(Box<HirUnaryExpr>),
    /// Function call
    Call(Box<HirCallExpr>),
    /// Method call
    MethodCall(Box<HirMethodCallExpr>),
    /// Field access
    Field(Box<HirFieldExpr>),
    /// Index access
    Index(Box<HirIndexExpr>),
    /// Block expression
    Block(Box<HirBlockExpr>),
    /// If expression
    If(Box<HirIfExpr>),
    /// Match expression
    Match(Box<HirMatchExpr>),
    /// Lambda/closure
    Lambda(Box<HirLambdaExpr>),
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum HirLiteral {
    /// Boolean literal
    Bool(bool),
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// Unit literal
    Unit,
}

/// Binary expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirBinaryExpr {
    /// Left operand
    pub left: HirExpr,
    /// Operator
    pub op: HirBinaryOp,
    /// Right operand
    pub right: HirExpr,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBinaryOp {
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Modulo
    Mod,
    /// Equality
    Eq,
    /// Not equal
    Ne,
    /// Less than
    Lt,
    /// Less than or equal
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal
    Ge,
    /// Logical and
    And,
    /// Logical or
    Or,
}

/// Unary expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirUnaryExpr {
    /// Operator
    pub op: HirUnaryOp,
    /// Operand
    pub operand: HirExpr,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirUnaryOp {
    /// Negation
    Neg,
    /// Logical not
    Not,
}

/// Function call expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirCallExpr {
    /// Function being called
    pub func: HirExpr,
    /// Arguments
    pub args: Vec<HirExpr>,
}

/// Method call expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirMethodCallExpr {
    /// Receiver object
    pub receiver: HirExpr,
    /// Method name
    pub method: Symbol,
    /// Arguments
    pub args: Vec<HirExpr>,
}

/// Field access expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirFieldExpr {
    /// Base expression
    pub base: HirExpr,
    /// Field name
    pub field: Symbol,
}

/// Index access expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirIndexExpr {
    /// Base expression
    pub base: HirExpr,
    /// Index expression
    pub index: HirExpr,
}

/// Block expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirBlockExpr {
    /// Statements in the block
    pub stmts: Vec<HirStmt>,
    /// Final expression (if any)
    pub expr: Option<HirExpr>,
}

/// If expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirIfExpr {
    /// Condition
    pub cond: HirExpr,
    /// Then branch
    pub then_branch: HirExpr,
    /// Else branch (if any)
    pub else_branch: Option<HirExpr>,
}

/// Match expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirMatchExpr {
    /// Scrutinee
    pub scrutinee: HirExpr,
    /// Match arms
    pub arms: Vec<HirMatchArm>,
}

/// Match arm.
#[derive(Debug, Clone, PartialEq)]
pub struct HirMatchArm {
    /// Pattern to match
    pub pat: HirPat,
    /// Guard expression (if any)
    pub guard: Option<HirExpr>,
    /// Body expression
    pub body: HirExpr,
}

/// Lambda expression.
#[derive(Debug, Clone, PartialEq)]
pub struct HirLambdaExpr {
    /// Parameters
    pub params: Vec<HirParam>,
    /// Return type (if annotated)
    pub return_type: Option<HirType>,
    /// Body
    pub body: HirExpr,
}

/// Statement forms (6 total).
#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    /// Immutable binding
    Val(HirValStmt),
    /// Mutable binding
    Var(HirVarStmt),
    /// Assignment
    Assign(HirAssignStmt),
    /// Expression statement
    Expr(HirExpr),
    /// Return statement
    Return(Option<HirExpr>),
    /// Break statement
    Break(Option<HirExpr>),
}

/// Immutable binding statement.
#[derive(Debug, Clone, PartialEq)]
pub struct HirValStmt {
    /// Pattern to bind
    pub pat: HirPat,
    /// Type annotation (if any)
    pub ty: Option<HirType>,
    /// Initializer expression
    pub init: HirExpr,
}

/// Mutable binding statement.
#[derive(Debug, Clone, PartialEq)]
pub struct HirVarStmt {
    /// Pattern to bind
    pub pat: HirPat,
    /// Type annotation (if any)
    pub ty: Option<HirType>,
    /// Initializer expression
    pub init: HirExpr,
}

/// Assignment statement.
#[derive(Debug, Clone, PartialEq)]
pub struct HirAssignStmt {
    /// Left-hand side (place expression)
    pub lhs: HirExpr,
    /// Right-hand side (value)
    pub rhs: HirExpr,
}

/// Type forms (8 total).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HirType {
    /// Named type (possibly with type arguments)
    Named(HirNamedType),
    /// Tuple type
    Tuple(Vec<HirType>),
    /// Array type
    Array(Box<HirArrayType>),
    /// Function type
    Function(Box<HirFunctionType>),
    /// Reference type
    Ref(Box<HirRefType>),
    /// Optional type
    Optional(Box<HirType>),
    /// Type variable (for inference)
    Var(u32),
    /// Error type (for error recovery)
    #[default]
    Error,
}

/// Named type with optional type arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct HirNamedType {
    /// Type name
    pub name: Symbol,
    /// Type arguments
    pub args: Vec<HirType>,
}

/// Array type.
#[derive(Debug, Clone, PartialEq)]
pub struct HirArrayType {
    /// Element type
    pub elem: HirType,
    /// Size (if fixed)
    pub size: Option<usize>,
}

/// Function type.
#[derive(Debug, Clone, PartialEq)]
pub struct HirFunctionType {
    /// Parameter types
    pub params: Vec<HirType>,
    /// Return type
    pub ret: HirType,
}

/// Reference type.
#[derive(Debug, Clone, PartialEq)]
pub struct HirRefType {
    /// Mutability
    pub mutable: bool,
    /// Referenced type
    pub ty: HirType,
}

/// Pattern forms (6 total).
#[derive(Debug, Clone, PartialEq)]
pub enum HirPat {
    /// Wildcard pattern
    Wildcard,
    /// Variable binding
    Var(Symbol),
    /// Literal pattern
    Literal(HirLiteral),
    /// Constructor pattern
    Constructor(HirConstructorPat),
    /// Tuple pattern
    Tuple(Vec<HirPat>),
    /// Or pattern
    Or(Vec<HirPat>),
}

/// Constructor pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct HirConstructorPat {
    /// Constructor name
    pub name: Symbol,
    /// Sub-patterns
    pub fields: Vec<HirPat>,
}

impl HirModule {
    /// Create a new empty module.
    pub fn new(name: Symbol) -> Self {
        Self {
            id: HirId::new(),
            name,
            decls: Vec::new(),
        }
    }
}
