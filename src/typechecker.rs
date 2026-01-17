//! DOL 2.0 Type Checker
//!
//! This module implements type inference and checking for DOL 2.0 expressions.
//! It validates type consistency, infers types for expressions, and ensures
//! type annotations match actual types.
//!
//! # Architecture
//!
//! The type checker uses bidirectional type checking:
//! - **Inference mode**: Synthesizes types from expressions
//! - **Checking mode**: Verifies expressions against expected types
//!
//! # Example
//!
//! ```rust
//! use metadol::typechecker::{TypeChecker, Type};
//! use metadol::ast::{Expr, Literal};
//!
//! let mut checker = TypeChecker::new();
//! let expr = Expr::Literal(Literal::Int(42));
//! let ty = checker.infer(&expr).unwrap();
//! assert_eq!(ty, Type::Int64);
//! ```

#[cfg(test)]
use crate::ast::Span;
use crate::ast::{BinaryOp, Block, Expr, Literal, Pattern, Stmt, TypeExpr, UnaryOp};
use std::collections::HashMap;

/// Semantic types used during type checking.
///
/// Unlike `TypeExpr` which represents syntax, `Type` represents
/// the actual semantic types used for type checking.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Void type (no value)
    Void,

    /// Boolean type
    Bool,

    /// 8-bit signed integer
    Int8,
    /// 16-bit signed integer
    Int16,
    /// 32-bit signed integer
    Int32,
    /// 64-bit signed integer
    Int64,

    /// 8-bit unsigned integer
    UInt8,
    /// 16-bit unsigned integer
    UInt16,
    /// 32-bit unsigned integer
    UInt32,
    /// 64-bit unsigned integer
    UInt64,

    /// 32-bit floating point
    Float32,
    /// 64-bit floating point
    Float64,

    /// String type
    String,

    /// Function type
    Function {
        /// Parameter types
        params: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
    },

    /// Tuple type
    Tuple(Vec<Type>),

    /// Generic/parametric type
    Generic {
        /// Type constructor name
        name: String,
        /// Type arguments
        args: Vec<Type>,
    },

    /// Type variable (for inference)
    Var(usize),

    /// Unknown type (inference placeholder)
    Unknown,

    /// Any type (compatible with everything - for gradual typing)
    Any,

    /// Never type (function never returns)
    Never,

    /// Error type (propagates type errors)
    Error,
}

impl Type {
    /// Returns true if this is a numeric type.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
        )
    }

    /// Returns true if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
        )
    }

    /// Returns true if this is a floating point type.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float32 | Type::Float64)
    }

    /// Returns true if this is a signed type.
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 | Type::Float32 | Type::Float64
        )
    }

    /// Returns the bit width of numeric types.
    pub fn bit_width(&self) -> Option<usize> {
        match self {
            Type::Int8 | Type::UInt8 => Some(8),
            Type::Int16 | Type::UInt16 => Some(16),
            Type::Int32 | Type::UInt32 | Type::Float32 => Some(32),
            Type::Int64 | Type::UInt64 | Type::Float64 => Some(64),
            _ => None,
        }
    }

    /// Creates a type from a TypeExpr.
    pub fn from_type_expr(expr: &TypeExpr) -> Type {
        match expr {
            TypeExpr::Named(name) => match name.as_str() {
                "Void" => Type::Void,
                "Bool" => Type::Bool,
                "Int8" => Type::Int8,
                "Int16" => Type::Int16,
                "Int32" => Type::Int32,
                "Int64" => Type::Int64,
                "UInt8" => Type::UInt8,
                "UInt16" => Type::UInt16,
                "UInt32" => Type::UInt32,
                "UInt64" => Type::UInt64,
                "Float32" => Type::Float32,
                "Float64" => Type::Float64,
                "String" => Type::String,
                _ => Type::Generic {
                    name: name.clone(),
                    args: vec![],
                },
            },
            TypeExpr::Generic { name, args } => Type::Generic {
                name: name.clone(),
                args: args.iter().map(Type::from_type_expr).collect(),
            },
            TypeExpr::Function {
                params,
                return_type,
            } => Type::Function {
                params: params.iter().map(Type::from_type_expr).collect(),
                return_type: Box::new(Type::from_type_expr(return_type)),
            },
            TypeExpr::Tuple(types) => Type::Tuple(types.iter().map(Type::from_type_expr).collect()),
            TypeExpr::Never => Type::Never,
            TypeExpr::Enum { variants } => Type::Generic {
                name: "Enum".to_string(),
                args: variants
                    .iter()
                    .map(|v| Type::Generic {
                        name: v.name.clone(),
                        args: vec![],
                    })
                    .collect(),
            },
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => write!(f, "Void"),
            Type::Bool => write!(f, "Bool"),
            Type::Int8 => write!(f, "Int8"),
            Type::Int16 => write!(f, "Int16"),
            Type::Int32 => write!(f, "Int32"),
            Type::Int64 => write!(f, "Int64"),
            Type::UInt8 => write!(f, "UInt8"),
            Type::UInt16 => write!(f, "UInt16"),
            Type::UInt32 => write!(f, "UInt32"),
            Type::UInt64 => write!(f, "UInt64"),
            Type::Float32 => write!(f, "Float32"),
            Type::Float64 => write!(f, "Float64"),
            Type::String => write!(f, "String"),
            Type::Function {
                params,
                return_type,
            } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Generic { name, args } => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Var(id) => write!(f, "?{}", id),
            Type::Unknown => write!(f, "?"),
            Type::Any => write!(f, "Any"),
            Type::Never => write!(f, "!"),
            Type::Error => write!(f, "Error"),
        }
    }
}

/// Effect context for tracking purity during type checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EffectContext {
    /// Pure context - no side effects allowed (default)
    #[default]
    Pure,
    /// Sex context - side effects permitted
    Sex,
}

/// Type checking error.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    /// Error message
    pub message: String,
    /// Expected type (if applicable)
    pub expected: Option<Type>,
    /// Actual type (if applicable)
    pub actual: Option<Type>,
}

impl TypeError {
    /// Creates a new type error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            expected: None,
            actual: None,
        }
    }

    /// Creates a type mismatch error.
    pub fn mismatch(expected: Type, actual: Type) -> Self {
        Self {
            message: format!("type mismatch: expected {}, found {}", expected, actual),
            expected: Some(expected),
            actual: Some(actual),
        }
    }

    /// Creates an undefined variable error.
    pub fn undefined(name: &str) -> Self {
        Self {
            message: format!("undefined variable: {}", name),
            expected: None,
            actual: None,
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeError {}

/// Type environment for tracking variable bindings.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Variable bindings in current scope
    bindings: HashMap<String, Type>,
    /// Parent scope (for nested scopes)
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a child environment with this as parent.
    pub fn child(&self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Binds a variable to a type.
    pub fn bind(&mut self, name: impl Into<String>, ty: Type) {
        self.bindings.insert(name.into(), ty);
    }

    /// Looks up a variable's type.
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.bindings
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }
}

/// The type checker.
#[derive(Debug)]
pub struct TypeChecker {
    /// Current type environment
    env: TypeEnv,
    /// Counter for generating fresh type variables
    var_counter: usize,
    /// Collected type errors
    errors: Vec<TypeError>,
    /// Current effect context
    effect_context: EffectContext,
    /// Effect context stack for nested contexts
    effect_stack: Vec<EffectContext>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    /// Creates a new type checker.
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            var_counter: 0,
            errors: Vec::new(),
            effect_context: EffectContext::Pure,
            effect_stack: Vec::new(),
        }
    }

    /// Returns collected errors.
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// Returns true if type checking passed without errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Clears collected errors.
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Generates a fresh type variable.
    fn fresh_var(&mut self) -> Type {
        let id = self.var_counter;
        self.var_counter += 1;
        Type::Var(id)
    }

    /// Adds a type error.
    fn error(&mut self, err: TypeError) {
        self.errors.push(err);
    }

    /// Enter a sex context (e.g., for sex blocks or sex functions)
    pub fn enter_sex_context(&mut self) {
        self.effect_stack.push(self.effect_context);
        self.effect_context = EffectContext::Sex;
    }

    /// Exit the current effect context
    pub fn exit_sex_context(&mut self) {
        self.effect_context = self.effect_stack.pop().unwrap_or(EffectContext::Pure);
    }

    /// Check if currently in a sex context
    pub fn in_sex_context(&self) -> bool {
        self.effect_context == EffectContext::Sex
    }

    /// Get the current effect context
    pub fn current_effect_context(&self) -> EffectContext {
        self.effect_context
    }

    /// Infers the type of an expression.
    pub fn infer(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            // Literals
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Ok(Type::Int64),
                Literal::Float(_) => Ok(Type::Float64),
                Literal::Bool(_) => Ok(Type::Bool),
                Literal::String(_) => Ok(Type::String),
                Literal::Char(_) => Ok(Type::String), // Char treated as String
                Literal::Null => Ok(Type::Unknown),   // Null is polymorphic
            },

            // Identifiers
            Expr::Identifier(name) => self
                .env
                .lookup(name)
                .cloned()
                .ok_or_else(|| TypeError::undefined(name)),

            // Unary expressions
            Expr::Unary { op, operand } => self.infer_unary(op, operand),

            // Binary expressions
            Expr::Binary { op, left, right } => self.infer_binary(op, left, right),

            // Function calls
            Expr::Call { callee, args } => self.infer_call(callee, args),

            // Lambdas
            Expr::Lambda {
                params,
                body,
                return_type,
            } => self.infer_lambda(params, body, return_type.as_ref()),

            // If expressions
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => self.infer_if(condition, then_branch, else_branch.as_deref()),

            // Match expressions
            Expr::Match { scrutinee, arms } => self.infer_match(scrutinee, arms),

            // Block expressions
            Expr::Block(Block {
                statements,
                final_expr,
                ..
            }) => self.infer_block(statements, final_expr.as_deref()),

            // Member/Field access
            Expr::Member { object, field } => {
                let _obj_type = self.infer(object)?;
                // For now, field access returns Unknown (structural typing TBD)
                let _ = field;
                Ok(Type::Unknown)
            }

            // Quote/Eval/Reflect - return special types
            Expr::Quote(inner) => {
                let inner_type = self.infer(inner)?;
                Ok(Type::Generic {
                    name: "Quoted".to_string(),
                    args: vec![inner_type],
                })
            }
            Expr::Eval(inner) => {
                let inner_type = self.infer(inner)?;
                match inner_type {
                    Type::Generic { name, args } if name == "Quoted" && !args.is_empty() => {
                        // Eval of Quoted<T> returns T
                        Ok(args.into_iter().next().unwrap())
                    }
                    Type::Generic { name, .. } if name == "Quoted" => {
                        // Quoted without type param - return Unknown
                        Ok(Type::Unknown)
                    }
                    Type::Unknown => Ok(Type::Unknown),
                    _ => {
                        self.error(TypeError::new(format!(
                            "cannot eval non-quoted type: {}",
                            inner_type
                        )));
                        Ok(Type::Error)
                    }
                }
            }
            Expr::Reflect(_type_expr) => {
                // Type reflection returns type metadata
                Ok(Type::Generic {
                    name: "TypeInfo".to_string(),
                    args: vec![],
                })
            }
            Expr::IdiomBracket { func, args } => {
                // Idiom brackets [| f a b |] desugar to f <$> a <*> b
                // For typing, we check that func is a function and args are applicative contexts
                let func_type = self.infer(func)?;

                // Infer types of all arguments
                for arg in args {
                    self.infer(arg)?;
                }

                // The result type depends on the function's return type wrapped in the applicative
                // For now, return the function's return type (simplified)
                match func_type {
                    Type::Function {
                        ref return_type, ..
                    } => Ok((**return_type).clone()),
                    _ => Ok(Type::Unknown),
                }
            }
            Expr::Unquote(inner) => {
                // Unquote inside a quote evaluates the inner expression
                self.infer(inner)
            }
            Expr::QuasiQuote(inner) => {
                // QuasiQuote is like Quote but allows splicing via Unquote
                let inner_type = self.infer(inner)?;
                Ok(Type::Generic {
                    name: "Quoted".to_string(),
                    args: vec![inner_type],
                })
            }
            // Logic expressions
            Expr::Forall(forall_expr) => {
                // Forall expressions have type Bool (they are propositions)
                self.infer(&forall_expr.body)?;
                Ok(Type::Bool)
            }
            Expr::Exists(exists_expr) => {
                // Exists expressions have type Bool (they are propositions)
                self.infer(&exists_expr.body)?;
                Ok(Type::Bool)
            }
            Expr::Implies { left, right, .. } => {
                // Implication requires both sides to be Bool
                let left_type = self.infer(left)?;
                let right_type = self.infer(right)?;
                if left_type != Type::Bool || right_type != Type::Bool {
                    self.error(TypeError::new(format!(
                        "implication requires Bool, found {} and {}",
                        left_type, right_type
                    )));
                }
                Ok(Type::Bool)
            }
            // Sex block - enter sex context, infer type, exit context
            Expr::SexBlock(Block {
                statements,
                final_expr,
                ..
            }) => {
                self.enter_sex_context();
                let result = self.infer_block(statements, final_expr.as_deref());
                self.exit_sex_context();
                result
            }
            // List literal
            Expr::List(elements) => {
                if elements.is_empty() {
                    // Empty list: List<Unknown>
                    Ok(Type::Generic {
                        name: "List".to_string(),
                        args: vec![Type::Unknown],
                    })
                } else {
                    // Infer element type from first element
                    let elem_type = self.infer(&elements[0])?;
                    // Check that all elements have the same type
                    for elem in elements.iter().skip(1) {
                        let t = self.infer(elem)?;
                        if t != elem_type {
                            self.error(TypeError::new(format!(
                                "list elements have inconsistent types: {} vs {}",
                                elem_type, t
                            )));
                        }
                    }
                    Ok(Type::Generic {
                        name: "List".to_string(),
                        args: vec![elem_type],
                    })
                }
            }
            // Tuple literal
            Expr::Tuple(elements) => {
                let mut elem_types = Vec::new();
                for elem in elements {
                    elem_types.push(self.infer(elem)?);
                }
                Ok(Type::Tuple(elem_types))
            }

            // Type cast - the result is the target type
            Expr::Cast { expr, target_type } => {
                // Type-check the expression being cast
                let _expr_type = self.infer(expr)?;
                // The result type is the target type
                Ok(Type::from_type_expr(target_type))
            }

            // Struct literal - the result is the struct type
            Expr::StructLiteral { type_name, fields } => {
                // Type-check all field expressions
                for (_, field_expr) in fields {
                    self.infer(field_expr)?;
                }
                // The result type is the struct type (represented as a generic with no args)
                Ok(Type::Generic {
                    name: type_name.clone(),
                    args: vec![],
                })
            }

            // Try expression - propagates errors, returns inner type on success
            Expr::Try(inner) => {
                // Type-check the inner expression
                let inner_type = self.infer(inner)?;
                // For now, just return the inner type (proper Result handling would be more complex)
                Ok(inner_type)
            }

            // This - reference to current instance
            Expr::This => {
                // Look up 'this' type in the environment
                self.env.lookup("this").cloned().ok_or_else(|| {
                    TypeError::new("'this' used outside of method context".to_string())
                })
            }
        }
    }

    /// Infers type for unary expressions.
    fn infer_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Type, TypeError> {
        let operand_type = self.infer(operand)?;

        match op {
            UnaryOp::Neg => {
                if !operand_type.is_numeric() {
                    self.error(TypeError::new(format!(
                        "cannot negate non-numeric type {}",
                        operand_type
                    )));
                    Ok(Type::Error)
                } else {
                    Ok(operand_type)
                }
            }
            UnaryOp::Not => {
                if operand_type != Type::Bool {
                    self.error(TypeError::new(format!(
                        "logical not requires Bool, found {}",
                        operand_type
                    )));
                    Ok(Type::Error)
                } else {
                    Ok(Type::Bool)
                }
            }
            UnaryOp::Quote => {
                // The operand was already inferred, pass its type to Quoted
                Ok(Type::Generic {
                    name: "Quoted".to_string(),
                    args: vec![operand_type],
                })
            }
            UnaryOp::Reflect => Ok(Type::Generic {
                name: "TypeInfo".to_string(),
                args: vec![],
            }),
            UnaryOp::Deref => {
                // Dereference - for now just return the operand type
                // Full implementation would unwrap pointer/reference types
                Ok(operand_type)
            }
        }
    }

    /// Infers type for binary expressions.
    fn infer_binary(
        &mut self,
        op: &BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<Type, TypeError> {
        let left_type = self.infer(left)?;
        let right_type = self.infer(right)?;

        match op {
            // Arithmetic operators
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if !left_type.is_numeric() || !right_type.is_numeric() {
                    self.error(TypeError::new(format!(
                        "arithmetic requires numeric types, found {} and {}",
                        left_type, right_type
                    )));
                    return Ok(Type::Error);
                }
                // Promote to larger type
                Ok(self.promote_numeric(&left_type, &right_type))
            }

            // Comparison operators
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality works on any matching types
                if !self.types_compatible(&left_type, &right_type) {
                    self.error(TypeError::new(format!(
                        "cannot compare {} with {}",
                        left_type, right_type
                    )));
                }
                Ok(Type::Bool)
            }

            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if !left_type.is_numeric() || !right_type.is_numeric() {
                    self.error(TypeError::new(format!(
                        "comparison requires numeric types, found {} and {}",
                        left_type, right_type
                    )));
                }
                Ok(Type::Bool)
            }

            // Logical operators
            BinaryOp::And | BinaryOp::Or => {
                if left_type != Type::Bool || right_type != Type::Bool {
                    self.error(TypeError::new(format!(
                        "logical operators require Bool, found {} and {}",
                        left_type, right_type
                    )));
                }
                Ok(Type::Bool)
            }

            // Pipe operator: a |> f means f(a)
            BinaryOp::Pipe => {
                // Right must be a function that accepts left
                match right_type {
                    Type::Function {
                        params,
                        return_type,
                    } => {
                        if params.is_empty() {
                            self.error(TypeError::new(
                                "pipe target function must accept at least one argument",
                            ));
                            Ok(Type::Error)
                        } else if !self.types_compatible(&left_type, &params[0]) {
                            self.error(TypeError::mismatch(params[0].clone(), left_type));
                            Ok(Type::Error)
                        } else {
                            Ok(*return_type)
                        }
                    }
                    Type::Unknown | Type::Any => Ok(Type::Unknown),
                    _ => {
                        // For untyped identifiers, assume it works
                        Ok(Type::Unknown)
                    }
                }
            }

            // Compose operator: f >> g means g(f(x))
            BinaryOp::Compose => {
                // Both must be functions
                match (&left_type, &right_type) {
                    (
                        Type::Function {
                            return_type: left_ret,
                            params: left_params,
                        },
                        Type::Function {
                            params: right_params,
                            return_type: right_ret,
                        },
                    ) => {
                        // f's return type must match g's first param
                        if !right_params.is_empty()
                            && !self.types_compatible(left_ret, &right_params[0])
                        {
                            self.error(TypeError::new(format!(
                                "cannot compose: {} does not match {}",
                                left_ret, right_params[0]
                            )));
                        }
                        Ok(Type::Function {
                            params: left_params.clone(),
                            return_type: right_ret.clone(),
                        })
                    }
                    (Type::Unknown, _) | (_, Type::Unknown) => Ok(Type::Unknown),
                    _ => {
                        // Assume identifier composition works
                        Ok(Type::Unknown)
                    }
                }
            }

            // Exponentiation
            BinaryOp::Pow => {
                if !left_type.is_numeric() || !right_type.is_numeric() {
                    self.error(TypeError::new(format!(
                        "exponentiation requires numeric types, found {} and {}",
                        left_type, right_type
                    )));
                    return Ok(Type::Error);
                }
                Ok(self.promote_numeric(&left_type, &right_type))
            }

            // Function application
            BinaryOp::Apply => {
                // Similar to pipe but different syntax
                match right_type {
                    Type::Function {
                        params,
                        return_type,
                    } => {
                        if params.is_empty() {
                            self.error(TypeError::new(
                                "apply target function must accept at least one argument",
                            ));
                            Ok(Type::Error)
                        } else if !self.types_compatible(&left_type, &params[0]) {
                            self.error(TypeError::mismatch(params[0].clone(), left_type));
                            Ok(Type::Error)
                        } else {
                            Ok(*return_type)
                        }
                    }
                    Type::Unknown | Type::Any => Ok(Type::Unknown),
                    _ => Ok(Type::Unknown),
                }
            }

            // Bind operator (assignment-like)
            BinaryOp::Bind => {
                // Bind returns the right-hand side type
                Ok(right_type)
            }

            // Member access operator
            BinaryOp::Member => {
                // Member access returns unknown (structural typing TBD)
                Ok(Type::Unknown)
            }

            // Functor map operator <$>
            BinaryOp::Map => {
                // Map applies a function to a value inside a functor
                // Type: (a -> b) -> f a -> f b
                // For now, return Unknown (full functor support TBD)
                Ok(Type::Unknown)
            }

            // Applicative apply operator <*>
            BinaryOp::Ap => {
                // Ap applies a wrapped function to a wrapped value
                // Type: f (a -> b) -> f a -> f b
                // For now, return Unknown (full applicative support TBD)
                Ok(Type::Unknown)
            }

            // Logical implication
            BinaryOp::Implies => {
                if left_type != Type::Bool || right_type != Type::Bool {
                    self.error(TypeError::new(format!(
                        "implication requires Bool, found {} and {}",
                        left_type, right_type
                    )));
                }
                Ok(Type::Bool)
            }

            // Range operator
            BinaryOp::Range => {
                if !left_type.is_numeric() || !right_type.is_numeric() {
                    self.error(TypeError::new(format!(
                        "range requires numeric types, found {} and {}",
                        left_type, right_type
                    )));
                }
                // Return a Range type (for now, use a tuple representation)
                Ok(Type::Tuple(vec![left_type, right_type]))
            }
        }
    }

    /// Infers type for function calls.
    fn infer_call(&mut self, function: &Expr, args: &[Expr]) -> Result<Type, TypeError> {
        let func_type = self.infer(function)?;

        match func_type {
            Type::Function {
                params,
                return_type,
            } => {
                // Check argument count
                if args.len() != params.len() {
                    self.error(TypeError::new(format!(
                        "expected {} arguments, found {}",
                        params.len(),
                        args.len()
                    )));
                    return Ok(*return_type);
                }

                // Check argument types
                for (i, (arg, param)) in args.iter().zip(params.iter()).enumerate() {
                    let arg_type = self.infer(arg)?;
                    if !self.types_compatible(&arg_type, param) {
                        self.error(TypeError::new(format!(
                            "argument {} has type {}, expected {}",
                            i, arg_type, param
                        )));
                    }
                }

                Ok(*return_type)
            }
            Type::Unknown | Type::Any => {
                // Infer all arguments for side effects, return unknown
                for arg in args {
                    let _ = self.infer(arg)?;
                }
                Ok(Type::Unknown)
            }
            _ => {
                self.error(TypeError::new(format!(
                    "cannot call non-function type {}",
                    func_type
                )));
                Ok(Type::Error)
            }
        }
    }

    /// Infers type for lambda expressions.
    fn infer_lambda(
        &mut self,
        params: &[(String, Option<TypeExpr>)],
        body: &Expr,
        return_type: Option<&TypeExpr>,
    ) -> Result<Type, TypeError> {
        // Create child environment with parameters
        let old_env = std::mem::take(&mut self.env);
        self.env = old_env.child();

        let param_types: Vec<Type> = params
            .iter()
            .map(|(name, ty_expr)| {
                let ty = ty_expr
                    .as_ref()
                    .map(Type::from_type_expr)
                    .unwrap_or_else(|| self.fresh_var());
                self.env.bind(name.clone(), ty.clone());
                ty
            })
            .collect();

        // Infer body type
        let body_type = self.infer(body)?;

        // Restore environment
        self.env = old_env;

        // Check return type annotation if present
        let ret_type = if let Some(ret_expr) = return_type {
            let expected = Type::from_type_expr(ret_expr);
            if !self.types_compatible(&body_type, &expected) {
                self.error(TypeError::mismatch(expected.clone(), body_type));
            }
            expected
        } else {
            body_type
        };

        Ok(Type::Function {
            params: param_types,
            return_type: Box::new(ret_type),
        })
    }

    /// Infers type for if expressions.
    fn infer_if(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<Type, TypeError> {
        // Condition must be bool
        let cond_type = self.infer(condition)?;
        if cond_type != Type::Bool && cond_type != Type::Unknown {
            self.error(TypeError::mismatch(Type::Bool, cond_type));
        }

        // Infer branch types
        let then_type = self.infer(then_branch)?;

        if let Some(else_expr) = else_branch {
            let else_type = self.infer(else_expr)?;
            // Both branches should have compatible types
            if !self.types_compatible(&then_type, &else_type) {
                self.error(TypeError::new(format!(
                    "if branches have incompatible types: {} and {}",
                    then_type, else_type
                )));
            }
            Ok(then_type)
        } else {
            // No else branch, result is Void
            Ok(Type::Void)
        }
    }

    /// Infers type for match expressions.
    fn infer_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[crate::ast::MatchArm],
    ) -> Result<Type, TypeError> {
        let scrutinee_type = self.infer(scrutinee)?;
        let _ = scrutinee_type; // Used for pattern checking (TODO)

        if arms.is_empty() {
            return Ok(Type::Void);
        }

        // Infer first arm's type
        let first_type = self.infer_match_arm(&arms[0])?;

        // Check all arms have compatible types
        for arm in arms.iter().skip(1) {
            let arm_type = self.infer_match_arm(arm)?;
            if !self.types_compatible(&first_type, &arm_type) {
                self.error(TypeError::new(format!(
                    "match arms have incompatible types: {} and {}",
                    first_type, arm_type
                )));
            }
        }

        Ok(first_type)
    }

    /// Infers type for a single match arm.
    fn infer_match_arm(&mut self, arm: &crate::ast::MatchArm) -> Result<Type, TypeError> {
        // Create child env for pattern bindings
        let old_env = std::mem::take(&mut self.env);
        self.env = old_env.child();

        // Bind pattern variables
        self.bind_pattern(&arm.pattern);

        // Check guard if present
        if let Some(guard) = &arm.guard {
            let guard_type = self.infer(guard)?;
            if guard_type != Type::Bool && guard_type != Type::Unknown {
                self.error(TypeError::mismatch(Type::Bool, guard_type));
            }
        }

        // Infer body type
        let body_type = self.infer(&arm.body)?;

        // Restore environment
        self.env = old_env;

        Ok(body_type)
    }

    /// Binds pattern variables to types.
    fn bind_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(name) => {
                self.env.bind(name.clone(), Type::Unknown);
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.bind_pattern(p);
                }
            }
            Pattern::Constructor { fields, .. } => {
                for p in fields {
                    self.bind_pattern(p);
                }
            }
            Pattern::Or(patterns) => {
                // For or-patterns, bind variables from first pattern
                // (all patterns should bind the same variables)
                if let Some(p) = patterns.first() {
                    self.bind_pattern(p);
                }
            }
            Pattern::Literal(_) | Pattern::Wildcard => {}
        }
    }

    /// Infers type for block expressions.
    fn infer_block(
        &mut self,
        statements: &[Stmt],
        final_expr: Option<&Expr>,
    ) -> Result<Type, TypeError> {
        let old_env = std::mem::take(&mut self.env);
        self.env = old_env.child();

        // Type check statements
        for stmt in statements {
            self.check_stmt(stmt)?;
        }

        // Infer final expression type
        let result_type = if let Some(expr) = final_expr {
            self.infer(expr)?
        } else {
            Type::Void
        };

        self.env = old_env;
        Ok(result_type)
    }

    /// Type checks a statement.
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let ty = self.infer(value)?;
                self.env.bind(name.clone(), ty);
            }
            Stmt::Expr(expr) => {
                let _ = self.infer(expr)?;
            }
            Stmt::For {
                binding,
                iterable,
                body,
                ..
            } => {
                // Infer iterator type
                let iter_type = self.infer(iterable)?;

                // Create child env with binding
                let old_env = std::mem::take(&mut self.env);
                self.env = old_env.child();

                // Bind element type (extract from iterator)
                let elem_type = match iter_type {
                    Type::Generic { name, args } if name == "Array" && !args.is_empty() => {
                        args[0].clone()
                    }
                    _ => Type::Unknown,
                };
                self.env.bind(binding.clone(), elem_type);

                // Check body
                for s in body {
                    self.check_stmt(s)?;
                }

                self.env = old_env;
            }
            Stmt::While { condition, body } => {
                let cond_type = self.infer(condition)?;
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.error(TypeError::mismatch(Type::Bool, cond_type));
                }
                for s in body {
                    self.check_stmt(s)?;
                }
            }
            Stmt::Loop { body } => {
                for s in body {
                    self.check_stmt(s)?;
                }
            }
            Stmt::Break | Stmt::Continue => {}
            Stmt::Return(Some(e)) => {
                let _ = self.infer(e)?;
            }
            Stmt::Return(None) => {}
            _ => {} // Other statements (DOL-specific)
        }
        Ok(())
    }

    /// Promotes two numeric types to their common supertype.
    fn promote_numeric(&self, left: &Type, right: &Type) -> Type {
        // Float > Int, larger width wins, signed > unsigned at same width
        let left_width = left.bit_width().unwrap_or(64);
        let right_width = right.bit_width().unwrap_or(64);

        if left.is_float() || right.is_float() {
            if left_width >= 64 || right_width >= 64 {
                Type::Float64
            } else {
                Type::Float32
            }
        } else {
            let max_width = left_width.max(right_width);
            let signed = left.is_signed() || right.is_signed();
            match (max_width, signed) {
                (8, true) => Type::Int8,
                (8, false) => Type::UInt8,
                (16, true) => Type::Int16,
                (16, false) => Type::UInt16,
                (32, true) => Type::Int32,
                (32, false) => Type::UInt32,
                (_, true) => Type::Int64,
                (_, false) => Type::UInt64,
            }
        }
    }

    /// Checks if two types are compatible.
    #[allow(clippy::only_used_in_recursion)]
    fn types_compatible(&self, ty1: &Type, ty2: &Type) -> bool {
        match (ty1, ty2) {
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Error, _) | (_, Type::Error) => true,
            (Type::Var(a), Type::Var(b)) => a == b,
            (a, b) if a == b => true,
            // Numeric coercion
            (a, b) if a.is_numeric() && b.is_numeric() => true,
            // Generic types
            (Type::Generic { name: n1, args: a1 }, Type::Generic { name: n2, args: a2 }) => {
                n1 == n2
                    && a1.len() == a2.len()
                    && a1.iter().zip(a2).all(|(x, y)| self.types_compatible(x, y))
            }
            // Function types
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2).all(|(x, y)| self.types_compatible(x, y))
                    && self.types_compatible(r1, r2)
            }
            // Tuples
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                t1.len() == t2.len() && t1.iter().zip(t2).all(|(x, y)| self.types_compatible(x, y))
            }
            _ => false,
        }
    }

    /// Checks an expression against an expected type.
    pub fn check(&mut self, expr: &Expr, expected: &Type) -> Result<(), TypeError> {
        let actual = self.infer(expr)?;
        if !self.types_compatible(&actual, expected) {
            self.error(TypeError::mismatch(expected.clone(), actual));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int_lit(n: i64) -> Expr {
        Expr::Literal(Literal::Int(n))
    }

    fn float_lit(n: f64) -> Expr {
        Expr::Literal(Literal::Float(n))
    }

    fn bool_lit(b: bool) -> Expr {
        Expr::Literal(Literal::Bool(b))
    }

    fn string_lit(s: &str) -> Expr {
        Expr::Literal(Literal::String(s.to_string()))
    }

    #[test]
    fn test_infer_literals() {
        let mut checker = TypeChecker::new();

        assert_eq!(checker.infer(&int_lit(42)).unwrap(), Type::Int64);
        assert_eq!(checker.infer(&float_lit(1.5)).unwrap(), Type::Float64);
        assert_eq!(checker.infer(&bool_lit(true)).unwrap(), Type::Bool);
        assert_eq!(checker.infer(&string_lit("hello")).unwrap(), Type::String);
    }

    #[test]
    fn test_infer_arithmetic() {
        let mut checker = TypeChecker::new();

        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(int_lit(1)),
            right: Box::new(int_lit(2)),
        };
        assert!(checker.infer(&expr).unwrap().is_integer());
    }

    #[test]
    fn test_infer_comparison() {
        let mut checker = TypeChecker::new();

        let expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(int_lit(1)),
            right: Box::new(int_lit(2)),
        };
        assert_eq!(checker.infer(&expr).unwrap(), Type::Bool);
    }

    #[test]
    fn test_infer_lambda() {
        let mut checker = TypeChecker::new();

        let lambda = Expr::Lambda {
            params: vec![("x".to_string(), Some(TypeExpr::Named("Int32".to_string())))],
            body: Box::new(Expr::Identifier("x".to_string())),
            return_type: None,
        };

        let ty = checker.infer(&lambda).unwrap();
        match ty {
            Type::Function { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], Type::Int32);
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_infer_if() {
        let mut checker = TypeChecker::new();

        let if_expr = Expr::If {
            condition: Box::new(bool_lit(true)),
            then_branch: Box::new(int_lit(1)),
            else_branch: Some(Box::new(int_lit(2))),
        };

        let ty = checker.infer(&if_expr).unwrap();
        assert!(ty.is_integer());
    }

    #[test]
    fn test_type_mismatch_error() {
        let mut checker = TypeChecker::new();

        let if_expr = Expr::If {
            condition: Box::new(int_lit(42)), // Should be Bool!
            then_branch: Box::new(int_lit(1)),
            else_branch: None,
        };

        let _ = checker.infer(&if_expr);
        assert!(!checker.is_ok());
    }

    #[test]
    fn test_variable_binding() {
        let mut checker = TypeChecker::new();
        checker.env.bind("x", Type::Int32);

        let expr = Expr::Identifier("x".to_string());
        assert_eq!(checker.infer(&expr).unwrap(), Type::Int32);
    }

    #[test]
    fn test_undefined_variable() {
        let mut checker = TypeChecker::new();

        let expr = Expr::Identifier("undefined".to_string());
        assert!(checker.infer(&expr).is_err());
    }

    #[test]
    fn test_type_from_type_expr() {
        let type_expr = TypeExpr::Function {
            params: vec![TypeExpr::Named("Int32".to_string())],
            return_type: Box::new(TypeExpr::Named("Bool".to_string())),
        };

        let ty = Type::from_type_expr(&type_expr);
        match ty {
            Type::Function {
                params,
                return_type,
            } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], Type::Int32);
                assert_eq!(*return_type, Type::Bool);
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_quote_preserves_inner_type() {
        let mut checker = TypeChecker::new();

        // Quote of Int64 should return Quoted<Int64>
        let expr = Expr::Quote(Box::new(int_lit(42)));
        let ty = checker.infer(&expr).unwrap();

        match ty {
            Type::Generic { name, args } => {
                assert_eq!(name, "Quoted");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Type::Int64);
            }
            _ => panic!("Expected Quoted<Int64>, found {}", ty),
        }
    }

    #[test]
    fn test_quote_bool() {
        let mut checker = TypeChecker::new();

        // Quote of Bool should return Quoted<Bool>
        let expr = Expr::Quote(Box::new(bool_lit(true)));
        let ty = checker.infer(&expr).unwrap();

        match ty {
            Type::Generic { name, args } => {
                assert_eq!(name, "Quoted");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Type::Bool);
            }
            _ => panic!("Expected Quoted<Bool>, found {}", ty),
        }
    }

    #[test]
    fn test_eval_unwraps_quoted_type() {
        let mut checker = TypeChecker::new();

        // Eval of Quoted<Int64> should return Int64
        let quoted_expr = Expr::Quote(Box::new(int_lit(42)));
        let eval_expr = Expr::Eval(Box::new(quoted_expr));
        let ty = checker.infer(&eval_expr).unwrap();

        assert_eq!(ty, Type::Int64);
    }

    #[test]
    fn test_eval_of_non_quoted_type_errors() {
        let mut checker = TypeChecker::new();

        // Eval of Int64 (non-quoted) should produce error
        let eval_expr = Expr::Eval(Box::new(int_lit(42)));
        let ty = checker.infer(&eval_expr).unwrap();

        assert_eq!(ty, Type::Error);
        assert!(!checker.is_ok());
        assert!(!checker.errors().is_empty());
        assert!(checker.errors()[0]
            .message
            .contains("cannot eval non-quoted type"));
    }

    #[test]
    fn test_unary_quote_preserves_type() {
        let mut checker = TypeChecker::new();

        // Unary quote operator should also preserve type
        let expr = Expr::Unary {
            op: UnaryOp::Quote,
            operand: Box::new(string_lit("hello")),
        };
        let ty = checker.infer(&expr).unwrap();

        match ty {
            Type::Generic { name, args } => {
                assert_eq!(name, "Quoted");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Type::String);
            }
            _ => panic!("Expected Quoted<String>, found {}", ty),
        }
    }

    #[test]
    fn test_quasiquote_preserves_type() {
        let mut checker = TypeChecker::new();

        // QuasiQuote should work like Quote
        let expr = Expr::QuasiQuote(Box::new(float_lit(1.5)));
        let ty = checker.infer(&expr).unwrap();

        match ty {
            Type::Generic { name, args } => {
                assert_eq!(name, "Quoted");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Type::Float64);
            }
            _ => panic!("Expected Quoted<Float64>, found {}", ty),
        }
    }

    #[test]
    fn test_unquote_evaluates_inner_expr() {
        let mut checker = TypeChecker::new();

        // Unquote should return the type of the inner expression
        let expr = Expr::Unquote(Box::new(int_lit(42)));
        let ty = checker.infer(&expr).unwrap();

        assert_eq!(ty, Type::Int64);
    }

    #[test]
    fn test_nested_quote_eval() {
        let mut checker = TypeChecker::new();

        // Quote(Eval(Quote(42))) should be Quoted<Int64>
        let inner_quote = Expr::Quote(Box::new(int_lit(42)));
        let eval_expr = Expr::Eval(Box::new(inner_quote));
        let outer_quote = Expr::Quote(Box::new(eval_expr));
        let ty = checker.infer(&outer_quote).unwrap();

        match ty {
            Type::Generic { name, args } => {
                assert_eq!(name, "Quoted");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Type::Int64);
            }
            _ => panic!("Expected Quoted<Int64>, found {}", ty),
        }
    }

    #[test]
    fn test_effect_context_default() {
        let checker = TypeChecker::new();
        assert_eq!(checker.current_effect_context(), EffectContext::Pure);
        assert!(!checker.in_sex_context());
    }

    #[test]
    fn test_effect_context_enter_exit() {
        let mut checker = TypeChecker::new();

        // Initially pure
        assert!(!checker.in_sex_context());

        // Enter sex context
        checker.enter_sex_context();
        assert!(checker.in_sex_context());
        assert_eq!(checker.current_effect_context(), EffectContext::Sex);

        // Exit sex context
        checker.exit_sex_context();
        assert!(!checker.in_sex_context());
        assert_eq!(checker.current_effect_context(), EffectContext::Pure);
    }

    #[test]
    fn test_nested_effect_contexts() {
        let mut checker = TypeChecker::new();

        // Pure -> Sex -> Sex -> Pure (stack)
        checker.enter_sex_context();
        assert!(checker.in_sex_context());

        checker.enter_sex_context();
        assert!(checker.in_sex_context());

        checker.exit_sex_context();
        assert!(checker.in_sex_context());

        checker.exit_sex_context();
        assert!(!checker.in_sex_context());
    }

    #[test]
    fn test_sex_block_inference() {
        let mut checker = TypeChecker::new();

        // Create a sex block with an integer literal
        let sex_block = Expr::SexBlock(Block {
            statements: vec![],
            final_expr: Some(Box::new(int_lit(42))),
            span: Span::default(),
        });

        let ty = checker.infer(&sex_block).unwrap();
        assert_eq!(ty, Type::Int64);

        // Should be back in pure context after inference
        assert!(!checker.in_sex_context());
    }
}
