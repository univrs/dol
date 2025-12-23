//! MLIR code generation for Metal DOL.
//!
//! This module provides MLIR code generation capabilities for DOL declarations,
//! transforming the DOL AST into MLIR IR that can be further optimized and
//! compiled to various target architectures.
//!
//! # Architecture
//!
//! The MLIR codegen operates in several phases:
//! 1. Type lowering - Convert DOL types to MLIR types
//! 2. Declaration lowering - Convert DOL declarations to MLIR modules
//! 3. Expression compilation - Lower expressions to MLIR operations
//! 4. Statement compilation - Lower statements to MLIR operations
//!
//! # Example
//!
//! ```rust,ignore
//! # #[cfg(feature = "mlir")]
//! # {
//! use metadol::mlir::MlirCodegen;
//! use melior::Context;
//!
//! let ctx = Context::new();
//! let codegen = MlirCodegen::new(&ctx)
//!     .with_filename("example.dol");
//!
//! let decl = /* parse DOL file */;
//! let mlir_module = codegen.compile(&decl)?;
//! # }
//! ```

#[cfg(feature = "mlir")]
use melior::{
    dialect::{arith, func, scf, DialectRegistry},
    ir::{
        attribute::{
            FloatAttribute, IntegerAttribute, StringAttribute, TypeAttribute,
        },
        operation::OperationBuilder,
        r#type::{FunctionType, IntegerType, Type as MlirType},
        Block, Identifier, Location, Module as MlirModule, Operation, Region, Value,
    },
    Context as MlirContext,
};

#[cfg(feature = "mlir")]
use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Declaration, Expr, FunctionDecl, Literal, Stmt, TypeExpr, UnaryOp,
};

/// MLIR code generation error types.
#[derive(Debug, Clone, PartialEq)]
pub enum CodegenError {
    /// Unsupported declaration type for MLIR compilation
    UnsupportedDeclaration(String),
    /// Unsupported expression type
    UnsupportedExpression(String),
    /// Unsupported statement type
    UnsupportedStatement(String),
    /// Unsupported type
    UnsupportedType(String),
    /// Variable not found in scope
    VariableNotFound(String),
    /// Type mismatch during compilation
    TypeMismatch(String),
    /// MLIR operation builder error
    BuilderError(String),
    /// Module verification failed
    VerificationFailed(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UnsupportedDeclaration(msg) => {
                write!(f, "Unsupported declaration: {}", msg)
            }
            CodegenError::UnsupportedExpression(msg) => {
                write!(f, "Unsupported expression: {}", msg)
            }
            CodegenError::UnsupportedStatement(msg) => {
                write!(f, "Unsupported statement: {}", msg)
            }
            CodegenError::UnsupportedType(msg) => write!(f, "Unsupported type: {}", msg),
            CodegenError::VariableNotFound(name) => write!(f, "Variable not found: {}", name),
            CodegenError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            CodegenError::BuilderError(msg) => write!(f, "Builder error: {}", msg),
            CodegenError::VerificationFailed(msg) => {
                write!(f, "Module verification failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for CodegenError {}

/// Result type for MLIR codegen operations.
pub type CodegenResult<T> = Result<T, CodegenError>;

// ============================================================================
// MLIR-enabled implementation
// ============================================================================

#[cfg(feature = "mlir")]
/// MLIR code generator for DOL declarations.
///
/// This struct manages the state required for generating MLIR IR from DOL AST,
/// including type mappings, variable bindings, and the MLIR context.
pub struct MlirCodegen<'ctx> {
    /// Reference to the MLIR context
    mlir_ctx: &'ctx MlirContext,
    /// Type lowering mapper
    type_lowering: TypeLowering<'ctx>,
    /// Current filename being compiled
    filename: Option<String>,
    /// Variable bindings in current scope
    variables: HashMap<String, (Value<'ctx, 'ctx>, MlirType<'ctx>)>,
}

#[cfg(feature = "mlir")]
impl<'ctx> MlirCodegen<'ctx> {
    /// Create a new MLIR code generator.
    ///
    /// # Arguments
    ///
    /// * `mlir_ctx` - Reference to the MLIR context
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::mlir::MlirCodegen;
    /// use melior::Context;
    ///
    /// let ctx = Context::new();
    /// let codegen = MlirCodegen::new(&ctx);
    /// ```
    pub fn new(mlir_ctx: &'ctx MlirContext) -> Self {
        Self {
            mlir_ctx,
            type_lowering: TypeLowering::new(mlir_ctx),
            filename: None,
            variables: HashMap::new(),
        }
    }

    /// Set the filename for source location tracking.
    ///
    /// # Arguments
    ///
    /// * `filename` - The source filename
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let codegen = MlirCodegen::new(&ctx)
    ///     .with_filename("example.dol");
    /// ```
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Compile a DOL declaration to an MLIR module.
    ///
    /// This is the main entry point for MLIR compilation. It takes a DOL
    /// declaration and produces a complete MLIR module.
    ///
    /// # Arguments
    ///
    /// * `decl` - The DOL declaration to compile
    ///
    /// # Returns
    ///
    /// An MLIR module on success, or a CodegenError on failure.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let module = codegen.compile(&declaration)?;
    /// ```
    pub fn compile(&mut self, decl: &Declaration) -> CodegenResult<MlirModule<'ctx>> {
        // Create MLIR module
        let location = self.create_location();
        let module = MlirModule::new(location);

        // Compile the declaration into the module
        self.compile_declaration(&module, decl)?;

        // Verify the module
        if !module.as_operation().verify() {
            return Err(CodegenError::VerificationFailed(
                "Generated MLIR module failed verification".to_string(),
            ));
        }

        Ok(module)
    }

    /// Compile a DOL declaration into an MLIR module.
    ///
    /// This method handles different declaration types (Gene, Trait, Function, etc.)
    /// and generates appropriate MLIR operations.
    ///
    /// # Arguments
    ///
    /// * `module` - The MLIR module to add operations to
    /// * `decl` - The declaration to compile
    pub fn compile_declaration(
        &mut self,
        module: &MlirModule<'ctx>,
        decl: &Declaration,
    ) -> CodegenResult<()> {
        match decl {
            Declaration::Gene(gene) => {
                // For now, genes don't generate executable code
                // They could generate type definitions or constants in the future
                Ok(())
            }
            Declaration::Trait(trait_decl) => {
                // Traits could generate interface definitions
                // For now, we skip them as they're primarily structural
                Ok(())
            }
            Declaration::Constraint(constraint) => {
                // Constraints could generate runtime checks
                // For now, we skip them
                Ok(())
            }
            Declaration::System(system) => {
                // Systems could generate main functions or initialization code
                // For now, we skip them
                Ok(())
            }
            Declaration::Evolution(_evolution) => {
                // Evolutions are metadata, no code generation needed
                Ok(())
            }
        }
    }

    /// Compile a DOL function declaration to MLIR.
    ///
    /// This method lowers a DOL function to an MLIR `func.func` operation,
    /// including parameter handling, type lowering, and body compilation.
    ///
    /// # Arguments
    ///
    /// * `module` - The MLIR module to add the function to
    /// * `func` - The function declaration to compile
    pub fn compile_function(
        &mut self,
        _module: &MlirModule<'ctx>,
        func: &FunctionDecl,
    ) -> CodegenResult<Operation<'ctx>> {
        let location = self.create_location();

        // Lower parameter types
        let param_types: Vec<MlirType> = func
            .params
            .iter()
            .map(|p| self.type_lowering.lower_type(&p.type_ann))
            .collect::<CodegenResult<Vec<_>>>()?;

        // Lower return type
        let return_type = if let Some(ref ret_ty) = func.return_type {
            vec![self.type_lowering.lower_type(ret_ty)?]
        } else {
            vec![] // void return
        };

        // Create function type
        let func_type = FunctionType::new(self.mlir_ctx, &param_types, &return_type);

        // Create function operation
        let region = Region::new();
        let block = Block::new(&param_types);

        // Set up parameter variables
        for (i, param) in func.params.iter().enumerate() {
            let arg = block.argument(i)?;
            self.variables
                .insert(param.name.clone(), (arg, param_types[i]));
        }

        // Compile function body
        for stmt in &func.body {
            self.compile_stmt(&block, stmt)?;
        }

        region.append_block(block);

        // Build the function operation
        let op = OperationBuilder::new("func.func", location)
            .add_attributes(&[(
                Identifier::new(self.mlir_ctx, "function_type"),
                TypeAttribute::new(func_type.into()).into(),
            )])
            .add_attributes(&[(
                Identifier::new(self.mlir_ctx, "sym_name"),
                StringAttribute::new(self.mlir_ctx, &func.name).into(),
            )])
            .add_regions([region])
            .build()?;

        Ok(op)
    }

    /// Compile a statement to MLIR operations.
    ///
    /// # Arguments
    ///
    /// * `block` - The MLIR block to add operations to
    /// * `stmt` - The statement to compile
    pub fn compile_stmt(
        &mut self,
        block: &Block<'ctx>,
        stmt: &Stmt,
    ) -> CodegenResult<Option<Value<'ctx, 'ctx>>> {
        match stmt {
            Stmt::Let {
                name,
                type_ann,
                value,
            } => {
                let val = self.compile_expr(block, value)?.ok_or_else(|| {
                    CodegenError::UnsupportedExpression("Expression produced no value".to_string())
                })?;

                let ty = if let Some(type_ann) = type_ann {
                    self.type_lowering.lower_type(type_ann)?
                } else {
                    val.r#type()
                };

                self.variables.insert(name.clone(), (val, ty));
                Ok(Some(val))
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    let val = self.compile_expr(block, expr)?;
                    // Create return operation
                    let location = self.create_location();
                    let return_op = OperationBuilder::new("func.return", location)
                        .add_operands(val.iter().collect::<Vec<_>>().as_slice())
                        .build()?;
                    block.append_operation(return_op);
                } else {
                    // Return void
                    let location = self.create_location();
                    let return_op = OperationBuilder::new("func.return", location).build()?;
                    block.append_operation(return_op);
                }
                Ok(None)
            }
            Stmt::Expr(expr) => self.compile_expr(block, expr),
            Stmt::Assign { target, value } => {
                // For now, assignments are not fully supported
                // We would need SSA form with proper phi nodes
                let _target = self.compile_expr(block, target)?;
                let val = self.compile_expr(block, value)?;
                Ok(val)
            }
            Stmt::While { condition, body } => {
                // Compile while loop using scf.while
                let _cond = self.compile_expr(block, condition)?;
                // TODO: Implement scf.while properly
                for stmt in body {
                    self.compile_stmt(block, stmt)?;
                }
                Ok(None)
            }
            Stmt::For {
                binding,
                iterable,
                body,
            } => {
                // Compile for loop
                let _iter = self.compile_expr(block, iterable)?;
                // TODO: Implement scf.for properly
                for stmt in body {
                    self.compile_stmt(block, stmt)?;
                }
                Ok(None)
            }
            Stmt::Loop { body } => {
                // Compile infinite loop
                for stmt in body {
                    self.compile_stmt(block, stmt)?;
                }
                Ok(None)
            }
            Stmt::Break => {
                // TODO: Implement break with proper control flow
                Ok(None)
            }
            Stmt::Continue => {
                // TODO: Implement continue with proper control flow
                Ok(None)
            }
        }
    }

    /// Compile an expression to MLIR operations.
    ///
    /// # Arguments
    ///
    /// * `block` - The MLIR block to add operations to
    /// * `expr` - The expression to compile
    ///
    /// # Returns
    ///
    /// The MLIR value representing the expression result, or None if the
    /// expression doesn't produce a value.
    pub fn compile_expr(
        &mut self,
        block: &Block<'ctx>,
        expr: &Expr,
    ) -> CodegenResult<Option<Value<'ctx, 'ctx>>> {
        match expr {
            Expr::Literal(lit) => self.compile_literal(block, lit),
            Expr::Identifier(name) => {
                let (val, _ty) = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CodegenError::VariableNotFound(name.clone()))?;
                Ok(Some(*val))
            }
            Expr::Binary { left, op, right } => self.compile_binary(block, left, *op, right),
            Expr::Unary { op, operand } => self.compile_unary(block, *op, operand),
            Expr::Call { callee, args } => self.compile_call(block, callee, args),
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => self.compile_if(block, condition, then_branch, else_branch.as_deref()),
            Expr::Block {
                statements,
                final_expr,
            } => {
                for stmt in statements {
                    self.compile_stmt(block, stmt)?;
                }
                if let Some(expr) = final_expr {
                    self.compile_expr(block, expr)
                } else {
                    Ok(None)
                }
            }
            Expr::Lambda {
                params,
                return_type,
                body,
            } => {
                // Lambda compilation would require creating a closure
                // For now, return an error
                Err(CodegenError::UnsupportedExpression(
                    "Lambda expressions not yet supported".to_string(),
                ))
            }
            Expr::Member { object, field } => {
                // Member access would require struct types
                Err(CodegenError::UnsupportedExpression(
                    "Member access not yet supported".to_string(),
                ))
            }
            Expr::Match { scrutinee, arms } => {
                // Pattern matching would require complex control flow
                Err(CodegenError::UnsupportedExpression(
                    "Match expressions not yet supported".to_string(),
                ))
            }
            Expr::Quote(_) | Expr::Unquote(_) | Expr::QuasiQuote(_) => {
                Err(CodegenError::UnsupportedExpression(
                    "Quote expressions not yet supported".to_string(),
                ))
            }
            Expr::Eval(_) => Err(CodegenError::UnsupportedExpression(
                "Eval expressions not yet supported".to_string(),
            )),
            Expr::Reflect(_) => Err(CodegenError::UnsupportedExpression(
                "Reflect expressions not yet supported".to_string(),
            )),
            Expr::IdiomBracket { func, args } => Err(CodegenError::UnsupportedExpression(
                "Idiom bracket expressions not yet supported".to_string(),
            )),
        }
    }

    /// Compile a literal value to an MLIR constant.
    fn compile_literal(
        &self,
        block: &Block<'ctx>,
        lit: &Literal,
    ) -> CodegenResult<Option<Value<'ctx, 'ctx>>> {
        let location = self.create_location();
        match lit {
            Literal::Int(i) => {
                let ty = IntegerType::new(self.mlir_ctx, 64);
                let attr = IntegerAttribute::new(ty.into(), *i);
                let op = OperationBuilder::new("arith.constant", location)
                    .add_attributes(&[(
                        Identifier::new(self.mlir_ctx, "value"),
                        attr.into(),
                    )])
                    .add_results(&[ty.into()])
                    .build()?;
                block.append_operation(op.clone());
                Ok(Some(op.result(0)?))
            }
            Literal::Float(f) => {
                let ty = MlirType::float64(self.mlir_ctx);
                let attr = FloatAttribute::new(self.mlir_ctx, ty, *f);
                let op = OperationBuilder::new("arith.constant", location)
                    .add_attributes(&[(
                        Identifier::new(self.mlir_ctx, "value"),
                        attr.into(),
                    )])
                    .add_results(&[ty])
                    .build()?;
                block.append_operation(op.clone());
                Ok(Some(op.result(0)?))
            }
            Literal::Bool(b) => {
                let ty = IntegerType::new(self.mlir_ctx, 1);
                let attr = IntegerAttribute::new(ty.into(), if *b { 1 } else { 0 });
                let op = OperationBuilder::new("arith.constant", location)
                    .add_attributes(&[(
                        Identifier::new(self.mlir_ctx, "value"),
                        attr.into(),
                    )])
                    .add_results(&[ty.into()])
                    .build()?;
                block.append_operation(op.clone());
                Ok(Some(op.result(0)?))
            }
            Literal::String(_s) => {
                // String literals would require special handling
                Err(CodegenError::UnsupportedExpression(
                    "String literals not yet supported".to_string(),
                ))
            }
        }
    }

    /// Compile a binary operation.
    fn compile_binary(
        &mut self,
        block: &Block<'ctx>,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> CodegenResult<Option<Value<'ctx, 'ctx>>> {
        let lhs = self
            .compile_expr(block, left)?
            .ok_or_else(|| CodegenError::UnsupportedExpression("Left operand has no value".to_string()))?;
        let rhs = self
            .compile_expr(block, right)?
            .ok_or_else(|| CodegenError::UnsupportedExpression("Right operand has no value".to_string()))?;

        let location = self.create_location();
        let result_type = lhs.r#type();

        let op_name = match op {
            BinaryOp::Add => "arith.addi",
            BinaryOp::Sub => "arith.subi",
            BinaryOp::Mul => "arith.muli",
            BinaryOp::Div => "arith.divsi",
            BinaryOp::Mod => "arith.remsi",
            BinaryOp::Eq => "arith.cmpi",
            BinaryOp::Ne => "arith.cmpi",
            BinaryOp::Lt => "arith.cmpi",
            BinaryOp::Le => "arith.cmpi",
            BinaryOp::Gt => "arith.cmpi",
            BinaryOp::Ge => "arith.cmpi",
            BinaryOp::And => "arith.andi",
            BinaryOp::Or => "arith.ori",
            _ => {
                return Err(CodegenError::UnsupportedExpression(format!(
                    "Binary operator {:?} not yet supported",
                    op
                )))
            }
        };

        let operation = OperationBuilder::new(op_name, location)
            .add_operands(&[lhs, rhs])
            .add_results(&[result_type])
            .build()?;

        block.append_operation(operation.clone());
        Ok(Some(operation.result(0)?))
    }

    /// Compile a unary operation.
    fn compile_unary(
        &mut self,
        block: &Block<'ctx>,
        op: UnaryOp,
        operand: &Expr,
    ) -> CodegenResult<Option<Value<'ctx, 'ctx>>> {
        let val = self
            .compile_expr(block, operand)?
            .ok_or_else(|| CodegenError::UnsupportedExpression("Operand has no value".to_string()))?;

        let location = self.create_location();
        let result_type = val.r#type();

        match op {
            UnaryOp::Neg => {
                // Negate: 0 - x
                let zero_ty = IntegerType::new(self.mlir_ctx, 64);
                let zero_attr = IntegerAttribute::new(zero_ty.into(), 0);
                let zero_op = OperationBuilder::new("arith.constant", location)
                    .add_attributes(&[(
                        Identifier::new(self.mlir_ctx, "value"),
                        zero_attr.into(),
                    )])
                    .add_results(&[zero_ty.into()])
                    .build()?;
                block.append_operation(zero_op.clone());
                let zero_val = zero_op.result(0)?;

                let neg_op = OperationBuilder::new("arith.subi", location)
                    .add_operands(&[zero_val, val])
                    .add_results(&[result_type])
                    .build()?;
                block.append_operation(neg_op.clone());
                Ok(Some(neg_op.result(0)?))
            }
            UnaryOp::Not => {
                // Logical not: xor with all 1s
                let op = OperationBuilder::new("arith.xori", location)
                    .add_operands(&[val])
                    .add_results(&[result_type])
                    .build()?;
                block.append_operation(op.clone());
                Ok(Some(op.result(0)?))
            }
            _ => Err(CodegenError::UnsupportedExpression(format!(
                "Unary operator {:?} not yet supported",
                op
            ))),
        }
    }

    /// Compile a function call.
    fn compile_call(
        &mut self,
        block: &Block<'ctx>,
        callee: &Expr,
        args: &[Expr],
    ) -> CodegenResult<Option<Value<'ctx, 'ctx>>> {
        // For now, only direct function calls are supported
        if let Expr::Identifier(func_name) = callee {
            let arg_vals: Vec<Value> = args
                .iter()
                .map(|arg| {
                    self.compile_expr(block, arg)?
                        .ok_or_else(|| CodegenError::UnsupportedExpression("Argument has no value".to_string()))
                })
                .collect::<CodegenResult<Vec<_>>>()?;

            let location = self.create_location();
            // Note: We would need to look up the function signature
            // For now, assume void return
            let op = OperationBuilder::new("func.call", location)
                .add_attributes(&[(
                    Identifier::new(self.mlir_ctx, "callee"),
                    StringAttribute::new(self.mlir_ctx, func_name).into(),
                )])
                .add_operands(&arg_vals)
                .build()?;

            block.append_operation(op.clone());
            // If the function returns a value, extract it
            // For now, assume void return
            Ok(None)
        } else {
            Err(CodegenError::UnsupportedExpression(
                "Indirect calls not yet supported".to_string(),
            ))
        }
    }

    /// Compile an if expression.
    fn compile_if(
        &mut self,
        block: &Block<'ctx>,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> CodegenResult<Option<Value<'ctx, 'ctx>>> {
        let cond = self
            .compile_expr(block, condition)?
            .ok_or_else(|| CodegenError::UnsupportedExpression("Condition has no value".to_string()))?;

        // For now, compile as basic blocks without using scf.if
        // A full implementation would use scf.if for proper control flow
        let then_val = self.compile_expr(block, then_branch)?;
        if let Some(else_expr) = else_branch {
            let else_val = self.compile_expr(block, else_expr)?;
            // TODO: Use scf.if to properly select between branches
            Ok(then_val)
        } else {
            Ok(then_val)
        }
    }

    /// Create a location for MLIR operations.
    fn create_location(&self) -> Location<'ctx> {
        if let Some(ref filename) = self.filename {
            Location::new(self.mlir_ctx, filename, 0, 0)
        } else {
            Location::unknown(self.mlir_ctx)
        }
    }
}

#[cfg(feature = "mlir")]
/// Type lowering from DOL types to MLIR types.
struct TypeLowering<'ctx> {
    mlir_ctx: &'ctx MlirContext,
}

#[cfg(feature = "mlir")]
impl<'ctx> TypeLowering<'ctx> {
    fn new(mlir_ctx: &'ctx MlirContext) -> Self {
        Self { mlir_ctx }
    }

    /// Lower a DOL type expression to an MLIR type.
    fn lower_type(&self, ty: &TypeExpr) -> CodegenResult<MlirType<'ctx>> {
        match ty {
            TypeExpr::Named(name) => match name.as_str() {
                "Int32" | "i32" => Ok(IntegerType::new(self.mlir_ctx, 32).into()),
                "Int64" | "i64" => Ok(IntegerType::new(self.mlir_ctx, 64).into()),
                "Float32" | "f32" => Ok(MlirType::float32(self.mlir_ctx)),
                "Float64" | "f64" => Ok(MlirType::float64(self.mlir_ctx)),
                "Bool" | "bool" => Ok(IntegerType::new(self.mlir_ctx, 1).into()),
                _ => Err(CodegenError::UnsupportedType(format!(
                    "Unknown type: {}",
                    name
                ))),
            },
            TypeExpr::Generic { name, args } => Err(CodegenError::UnsupportedType(
                "Generic types not yet supported".to_string(),
            )),
            TypeExpr::Function { params, return_type } => {
                let param_types: Vec<MlirType> = params
                    .iter()
                    .map(|p| self.lower_type(p))
                    .collect::<CodegenResult<Vec<_>>>()?;
                let ret_type = self.lower_type(return_type)?;
                Ok(FunctionType::new(self.mlir_ctx, &param_types, &[ret_type]).into())
            }
            TypeExpr::Tuple(types) => Err(CodegenError::UnsupportedType(
                "Tuple types not yet supported".to_string(),
            )),
        }
    }
}

// ============================================================================
// Stub implementation when MLIR feature is disabled
// ============================================================================

#[cfg(not(feature = "mlir"))]
/// Stub MLIR code generator when the MLIR feature is disabled.
///
/// This provides a no-op implementation that allows code to compile
/// without the MLIR feature, but will panic if actually used.
pub struct MlirCodegen;

#[cfg(not(feature = "mlir"))]
impl MlirCodegen {
    /// Create a new MLIR code generator (stub).
    ///
    /// # Panics
    ///
    /// This will panic when called, as MLIR support is not enabled.
    pub fn new<T>(_ctx: T) -> Self {
        panic!("MLIR support not enabled. Enable the 'mlir' feature to use MLIR codegen.");
    }

    /// Set filename (stub).
    pub fn with_filename(self, _filename: impl Into<String>) -> Self {
        self
    }

    /// Compile a declaration (stub).
    ///
    /// # Panics
    ///
    /// This will panic when called, as MLIR support is not enabled.
    pub fn compile<T>(&mut self, _decl: &Declaration) -> CodegenResult<T> {
        panic!("MLIR support not enabled. Enable the 'mlir' feature to use MLIR codegen.");
    }
}

#[cfg(test)]
#[cfg(feature = "mlir")]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_mlir_codegen_creation() {
        let ctx = MlirContext::new();
        let codegen = MlirCodegen::new(&ctx);
        assert!(codegen.filename.is_none());
    }

    #[test]
    fn test_mlir_codegen_with_filename() {
        let ctx = MlirContext::new();
        let codegen = MlirCodegen::new(&ctx).with_filename("test.dol");
        assert_eq!(codegen.filename, Some("test.dol".to_string()));
    }

    #[test]
    fn test_type_lowering_primitives() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(&ctx);

        let int32 = lowering.lower_type(&TypeExpr::Named("Int32".to_string()));
        assert!(int32.is_ok());

        let float64 = lowering.lower_type(&TypeExpr::Named("Float64".to_string()));
        assert!(float64.is_ok());

        let bool_type = lowering.lower_type(&TypeExpr::Named("Bool".to_string()));
        assert!(bool_type.is_ok());
    }

    #[test]
    fn test_compile_gene_declaration() {
        let ctx = MlirContext::new();
        ctx.append_dialect_registry(&{
            let registry = DialectRegistry::new();
            registry.register::<func::Dialect>();
            registry.register::<arith::Dialect>();
            registry
        });
        ctx.load_all_available_dialects();

        let mut codegen = MlirCodegen::new(&ctx);
        let gene = Gene {
            name: "test.gene".to_string(),
            statements: vec![],
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        };

        let decl = Declaration::Gene(gene);
        let result = codegen.compile(&decl);
        assert!(result.is_ok());
    }
}
