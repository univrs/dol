//! HIR to MLIR Lowering Pass
//!
//! This module implements the lowering transformation from HIR (High-level
//! Intermediate Representation) to MLIR operations. This is a critical bridge
//! between DOL's semantic analysis and executable code generation.
//!
//! # Overview
//!
//! The lowering pass transforms HIR nodes into MLIR operations:
//!
//! 1. **Modules** → MLIR module operations
//! 2. **Functions** → func.func operations with bodies
//! 3. **Expressions** → Arithmetic/control flow operations producing values
//! 4. **Statements** → Side-effecting operations
//! 5. **Types** → MLIR type system
//!
//! # Architecture
//!
//! ```text
//! HirModule → lower_module → ModuleOp
//!     ↓                           ↓
//! HirDecl → lower_decl → Operations (func.func, etc.)
//!     ↓                           ↓
//! HirExpr → lower_expr → Values (SSA)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::mlir::lowering::HirToMlirLowering;
//! use metadol::mlir::MlirContext;
//! use metadol::hir::HirModule;
//!
//! let ctx = MlirContext::new();
//! let mut lowering = HirToMlirLowering::new(ctx.context());
//!
//! // Lower HIR module to MLIR
//! let mlir_module = lowering.lower_module(&hir_module)?;
//! ```

#[cfg(feature = "mlir")]
use crate::ast::BinaryOp as AstBinaryOp;
#[cfg(feature = "mlir")]
use crate::ast::UnaryOp as AstUnaryOp;
#[cfg(feature = "mlir")]
use crate::hir::symbol::{Symbol, SymbolTable};
#[cfg(feature = "mlir")]
use crate::hir::types::*;
#[cfg(feature = "mlir")]
use crate::mlir::ops::OpBuilder;
#[cfg(feature = "mlir")]
use crate::mlir::MlirError;

#[cfg(feature = "mlir")]
use melior::{
    ir::{
        attribute::{StringAttribute, TypeAttribute},
        operation::OperationBuilder,
        r#type::{FunctionType, IntegerType},
        Block, Location, Module as MlirModule, Operation, Region, Type as MlirType, Value,
        ValueLike,
    },
    Context,
};

#[cfg(feature = "mlir")]
use std::collections::HashMap;

/// HIR to MLIR lowering engine.
///
/// This struct maintains the state needed for lowering HIR nodes to MLIR operations,
/// including the MLIR context, operation builder, and symbol tables for resolving
/// names to values.
#[cfg(feature = "mlir")]
pub struct HirToMlirLowering<'c> {
    /// MLIR context reference
    context: &'c Context,
    /// Operation builder for creating MLIR operations
    builder: OpBuilder<'c>,
    /// Symbol table for resolving symbol IDs to strings
    symbol_table: SymbolTable,
    /// Map from variable symbols to their MLIR values (current scope)
    /// In a full implementation, this would be a stack of scopes
    variables: HashMap<Symbol, Value<'c, 'c>>,
}

#[cfg(feature = "mlir")]
impl<'c> HirToMlirLowering<'c> {
    /// Creates a new HIR to MLIR lowering instance.
    ///
    /// # Arguments
    ///
    /// * `context` - The MLIR context to use for creating operations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = Context::new();
    /// let lowering = HirToMlirLowering::new(&ctx);
    /// ```
    pub fn new(context: &'c Context) -> Self {
        Self {
            context,
            builder: OpBuilder::new(context),
            symbol_table: SymbolTable::new(),
            variables: HashMap::new(),
        }
    }

    /// Sets the symbol table for resolving symbols.
    ///
    /// This should be called with the symbol table used during HIR construction.
    pub fn with_symbol_table(mut self, symbol_table: SymbolTable) -> Self {
        self.symbol_table = symbol_table;
        self
    }

    /// Lowers a complete HIR module to an MLIR module.
    ///
    /// # Arguments
    ///
    /// * `hir_module` - The HIR module to lower
    ///
    /// # Returns
    ///
    /// An MLIR module containing all lowered declarations
    ///
    /// # Errors
    ///
    /// Returns an error if any declaration cannot be lowered
    pub fn lower_module(&mut self, hir_module: &HirModule) -> Result<MlirModule<'c>, MlirError> {
        let location = Location::unknown(self.context);
        let mlir_module = MlirModule::new(location);

        // Lower each declaration and add to module body
        let body = mlir_module.body();
        for decl in &hir_module.decls {
            let op = self.lower_decl(decl)?;
            body.append_operation(op);
        }

        Ok(mlir_module)
    }

    /// Lowers a HIR declaration to an MLIR operation.
    ///
    /// Dispatches to the appropriate lowering method based on declaration type.
    pub fn lower_decl(&mut self, decl: &HirDecl) -> Result<Operation<'c>, MlirError> {
        match decl {
            HirDecl::Function(func_decl) => self.lower_function(func_decl),
            HirDecl::Type(type_decl) => {
                // For now, type declarations don't generate operations
                // In a full implementation, they would create type definitions
                Err(MlirError::new(format!(
                    "type declaration lowering not yet implemented: {}",
                    self.resolve_symbol(type_decl.name)
                )))
            }
            HirDecl::Trait(trait_decl) => Err(MlirError::new(format!(
                "trait declaration lowering not yet implemented: {}",
                self.resolve_symbol(trait_decl.name)
            ))),
            HirDecl::Module(module_decl) => {
                // Nested modules would create module operations
                Err(MlirError::new(format!(
                    "nested module lowering not yet implemented: {}",
                    self.resolve_symbol(module_decl.name)
                )))
            }
        }
    }

    /// Lowers a HIR function declaration to a func.func operation.
    ///
    /// Creates an MLIR function with the appropriate signature and body.
    pub fn lower_function(
        &mut self,
        func_decl: &HirFunctionDecl,
    ) -> Result<Operation<'c>, MlirError> {
        let location = Location::unknown(self.context);
        let name = self.resolve_symbol(func_decl.name);

        // Lower parameter types
        let param_types: Result<Vec<_>, _> = func_decl
            .params
            .iter()
            .map(|p| self.lower_type(&p.ty))
            .collect();
        let param_types = param_types?;

        // Lower return type
        let return_type = self.lower_type(&func_decl.return_type)?;
        let result_types = if self.is_void_type(&func_decl.return_type) {
            vec![]
        } else {
            vec![return_type]
        };

        // Create function type
        let function_type = FunctionType::new(self.context, &param_types, &result_types);

        // Create function body region
        let region = Region::new();
        let block = Block::new(
            &param_types
                .iter()
                .map(|(ty, _)| (*ty, location))
                .collect::<Vec<_>>(),
        );

        // Map parameters to block arguments
        for (i, param) in func_decl.params.iter().enumerate() {
            if let HirPat::Var(sym) = param.pat {
                let block_arg = block.argument(i).unwrap().into();
                self.variables.insert(sym, block_arg);
            }
        }

        // Lower function body if present
        if let Some(body_expr) = &func_decl.body {
            let body_value = self.lower_expr(body_expr, &block, location)?;

            // Add return statement
            if !result_types.is_empty() {
                let return_op = self.builder.build_return(&[body_value], location)?;
                block.append_operation(return_op);
            } else {
                let return_op = self.builder.build_return(&[], location)?;
                block.append_operation(return_op);
            }
        } else {
            // External function with no body - just add empty return
            let return_op = self.builder.build_return(&[], location)?;
            block.append_operation(return_op);
        }

        region.append_block(block);

        // Build function operation
        self.builder
            .build_func(name, function_type, region, location)
    }

    /// Lowers a HIR expression to an MLIR value.
    ///
    /// Expressions produce SSA values that can be used by other operations.
    pub fn lower_expr(
        &mut self,
        expr: &HirExpr,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        match expr {
            HirExpr::Literal(lit) => self.lower_literal(lit, block, location),
            HirExpr::Var(sym) => self.lower_var(*sym, location),
            HirExpr::Binary(binary_expr) => self.lower_binary(binary_expr, block, location),
            HirExpr::Unary(unary_expr) => self.lower_unary(unary_expr, block, location),
            HirExpr::Call(call_expr) => self.lower_call(call_expr, block, location),
            HirExpr::If(if_expr) => self.lower_if(if_expr, block, location),
            HirExpr::Block(block_expr) => self.lower_block(block_expr, block, location),
            HirExpr::MethodCall(_) => {
                Err(MlirError::new("method call lowering not yet implemented"))
            }
            HirExpr::Field(_) => Err(MlirError::new("field access lowering not yet implemented")),
            HirExpr::Index(_) => Err(MlirError::new("index access lowering not yet implemented")),
            HirExpr::Match(_) => Err(MlirError::new(
                "match expression lowering not yet implemented",
            )),
            HirExpr::Lambda(_) => Err(MlirError::new(
                "lambda expression lowering not yet implemented",
            )),
            HirExpr::Loop(_) | HirExpr::While(_) | HirExpr::For(_) => {
                Err(MlirError::new("loop lowering not yet implemented"))
            }
            HirExpr::Tuple(_) => Err(MlirError::new("tuple lowering not yet implemented")),
            HirExpr::List(_) => Err(MlirError::new("list lowering not yet implemented")),
        }
    }

    /// Lowers a literal value to a constant operation.
    fn lower_literal(
        &mut self,
        lit: &HirLiteral,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        match lit {
            HirLiteral::Bool(b) => {
                let op = self.builder.build_constant_i1(*b, location)?;
                block.append_operation(op);
                let last_op = block
                    .terminator()
                    .or_else(|| {
                        // Get the last operation if no terminator
                        let ops: Vec<_> = block.operations().collect();
                        ops.last().copied()
                    })
                    .ok_or_else(|| MlirError::new("no operation in block"))?;
                Ok(last_op.result(0)?.into())
            }
            HirLiteral::Int(i) => {
                let op = self.builder.build_constant_i64(*i, location)?;
                block.append_operation(op);
                let ops: Vec<_> = block.operations().collect();
                let last_op = ops
                    .last()
                    .ok_or_else(|| MlirError::new("no operation in block"))?;
                Ok(last_op.result(0)?.into())
            }
            HirLiteral::Float(f) => {
                // For now, represent floats as i64 (would need proper float constant builder)
                Err(MlirError::new(format!(
                    "float literal lowering not yet implemented: {}",
                    f
                )))
            }
            HirLiteral::String(s) => Err(MlirError::new(format!(
                "string literal lowering not yet implemented: {}",
                s
            ))),
            HirLiteral::Unit => {
                // Unit is void, return a dummy value
                // In practice, unit values don't produce values in MLIR
                Err(MlirError::new("unit literal cannot be lowered to a value"))
            }
        }
    }

    /// Lowers a variable reference.
    fn lower_var(
        &mut self,
        sym: Symbol,
        _location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        self.variables.get(&sym).copied().ok_or_else(|| {
            MlirError::new(format!("undefined variable: {}", self.resolve_symbol(sym)))
        })
    }

    /// Lowers a binary expression.
    fn lower_binary(
        &mut self,
        binary_expr: &HirBinaryExpr,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        let lhs = self.lower_expr(&binary_expr.left, block, location)?;
        let rhs = self.lower_expr(&binary_expr.right, block, location)?;

        // Convert HIR binary op to AST binary op
        let ast_op = self.hir_binop_to_ast(binary_expr.op)?;

        // Build the operation (assumes integer for now)
        let op = self
            .builder
            .build_binary_arith(ast_op, lhs, rhs, location)?;
        block.append_operation(op);

        let ops: Vec<_> = block.operations().collect();
        let last_op = ops
            .last()
            .ok_or_else(|| MlirError::new("no operation in block"))?;
        Ok(last_op.result(0)?.into())
    }

    /// Lowers a unary expression.
    fn lower_unary(
        &mut self,
        unary_expr: &HirUnaryExpr,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        let operand = self.lower_expr(&unary_expr.operand, block, location)?;

        // Convert HIR unary op to AST unary op
        let ast_op = self.hir_unop_to_ast(unary_expr.op)?;

        let op = self.builder.build_unary(ast_op, operand, location)?;
        block.append_operation(op);

        let ops: Vec<_> = block.operations().collect();
        let last_op = ops
            .last()
            .ok_or_else(|| MlirError::new("no operation in block"))?;
        Ok(last_op.result(0)?.into())
    }

    /// Lowers a function call expression.
    fn lower_call(
        &mut self,
        call_expr: &HirCallExpr,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        // Get the callee name (must be a variable reference)
        let callee_name = match &call_expr.func {
            HirExpr::Var(sym) => self.resolve_symbol(*sym),
            _ => {
                return Err(MlirError::new(
                    "only direct function calls are supported (no closures yet)",
                ))
            }
        };

        // Lower arguments
        let args: Result<Vec<_>, _> = call_expr
            .args
            .iter()
            .map(|arg| self.lower_expr(arg, block, location))
            .collect();
        let args = args?;

        // Determine result types (for now, assume single i64 result)
        // In a full implementation, we'd look up the function signature
        let result_type = IntegerType::new(self.context, 64).into();
        let result_types = vec![result_type];

        let op = self
            .builder
            .build_call(callee_name, &args, &result_types, location)?;
        block.append_operation(op);

        let ops: Vec<_> = block.operations().collect();
        let last_op = ops
            .last()
            .ok_or_else(|| MlirError::new("no operation in block"))?;
        Ok(last_op.result(0)?.into())
    }

    /// Lowers an if expression using scf.if.
    fn lower_if(
        &mut self,
        if_expr: &HirIfExpr,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        // Lower condition
        let cond = self.lower_expr(&if_expr.cond, block, location)?;

        // Create regions for then and else branches
        let then_region = Region::new();
        let then_block = Block::new(&[]);
        let then_value = self.lower_expr(&if_expr.then_branch, &then_block, location)?;

        // Add yield operation for then branch
        let yield_op = OperationBuilder::new("scf.yield", location)
            .add_operands(&[then_value])
            .build()
            .map_err(|e| MlirError::new(format!("failed to create scf.yield: {}", e)))?;
        then_block.append_operation(yield_op);
        then_region.append_block(then_block);

        let else_region = if let Some(else_branch) = &if_expr.else_branch {
            let else_region = Region::new();
            let else_block = Block::new(&[]);
            let else_value = self.lower_expr(else_branch, &else_block, location)?;

            let yield_op = OperationBuilder::new("scf.yield", location)
                .add_operands(&[else_value])
                .build()
                .map_err(|e| MlirError::new(format!("failed to create scf.yield: {}", e)))?;
            else_block.append_operation(yield_op);
            else_region.append_block(else_block);
            Some(else_region)
        } else {
            None
        };

        // Build if operation
        let result_type = then_value.r#type();
        let if_op =
            self.builder
                .build_if(cond, &[result_type], then_region, else_region, location)?;
        block.append_operation(if_op);

        let ops: Vec<_> = block.operations().collect();
        let last_op = ops
            .last()
            .ok_or_else(|| MlirError::new("no operation in block"))?;
        Ok(last_op.result(0)?.into())
    }

    /// Lowers a block expression.
    fn lower_block(
        &mut self,
        block_expr: &HirBlockExpr,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<Value<'c, 'c>, MlirError> {
        // Lower each statement
        for stmt in &block_expr.stmts {
            self.lower_stmt(stmt, block, location)?;
        }

        // Lower final expression if present
        if let Some(expr) = &block_expr.expr {
            self.lower_expr(expr, block, location)
        } else {
            Err(MlirError::new(
                "block without final expression cannot produce a value",
            ))
        }
    }

    /// Lowers a statement (side-effecting operation).
    fn lower_stmt(
        &mut self,
        stmt: &HirStmt,
        block: &Block<'c>,
        location: Location<'c>,
    ) -> Result<(), MlirError> {
        match stmt {
            HirStmt::Val(val_stmt) => {
                let value = self.lower_expr(&val_stmt.init, block, location)?;
                if let HirPat::Var(sym) = val_stmt.pat {
                    self.variables.insert(sym, value);
                }
                Ok(())
            }
            HirStmt::Var(var_stmt) => {
                let value = self.lower_expr(&var_stmt.init, block, location)?;
                if let HirPat::Var(sym) = var_stmt.pat {
                    self.variables.insert(sym, value);
                }
                Ok(())
            }
            HirStmt::Assign(_) => Err(MlirError::new(
                "assignment statement lowering not yet implemented",
            )),
            HirStmt::Expr(expr) => {
                // Expression statement - just lower for side effects
                self.lower_expr(expr, block, location)?;
                Ok(())
            }
            HirStmt::Return(ret_expr) => {
                if let Some(expr) = ret_expr {
                    let value = self.lower_expr(expr, block, location)?;
                    let op = self.builder.build_return(&[value], location)?;
                    block.append_operation(op);
                } else {
                    let op = self.builder.build_return(&[], location)?;
                    block.append_operation(op);
                }
                Ok(())
            }
            HirStmt::Break(_) => Err(MlirError::new(
                "break statement lowering not yet implemented",
            )),
            HirStmt::Continue => Err(MlirError::new(
                "continue statement lowering not yet implemented",
            )),
        }
    }

    /// Lowers a HIR type to an MLIR type.
    fn lower_type(&self, ty: &HirType) -> Result<MlirType<'c>, MlirError> {
        match ty {
            HirType::Named(named) => {
                let name = self.resolve_symbol(named.name);
                match name {
                    "void" => Ok(MlirType::tuple(self.context, &[])),
                    "bool" => Ok(IntegerType::new(self.context, 1).into()),
                    "i8" => Ok(IntegerType::new(self.context, 8).into()),
                    "i16" => Ok(IntegerType::new(self.context, 16).into()),
                    "i32" => Ok(IntegerType::new(self.context, 32).into()),
                    "i64" => Ok(IntegerType::new(self.context, 64).into()),
                    "u8" => Ok(IntegerType::new(self.context, 8).into()),
                    "u16" => Ok(IntegerType::new(self.context, 16).into()),
                    "u32" => Ok(IntegerType::new(self.context, 32).into()),
                    "u64" => Ok(IntegerType::new(self.context, 64).into()),
                    "f32" => Ok(MlirType::float32(self.context)),
                    "f64" => Ok(MlirType::float64(self.context)),
                    _ => Err(MlirError::new(format!("unsupported named type: {}", name))),
                }
            }
            HirType::Tuple(types) => {
                let elem_types: Result<Vec<_>, _> =
                    types.iter().map(|t| self.lower_type(t)).collect();
                Ok(MlirType::tuple(self.context, &elem_types?))
            }
            HirType::Function(func_type) => {
                let param_types: Result<Vec<_>, _> = func_type
                    .params
                    .iter()
                    .map(|p| self.lower_type(p))
                    .collect();
                let ret_type = self.lower_type(&func_type.ret)?;
                let results = if self.is_void_type(&func_type.ret) {
                    vec![]
                } else {
                    vec![ret_type]
                };
                Ok(FunctionType::new(self.context, &param_types?, &results).into())
            }
            HirType::Array(_) => Err(MlirError::new("array type lowering not yet implemented")),
            HirType::Ref(_) => Err(MlirError::new(
                "reference type lowering not yet implemented",
            )),
            HirType::Optional(_) => {
                Err(MlirError::new("optional type lowering not yet implemented"))
            }
            HirType::Var(id) => Err(MlirError::new(format!(
                "cannot lower unresolved type variable ?{}",
                id
            ))),
            HirType::Error => Err(MlirError::new("cannot lower error type")),
        }
    }

    /// Helper: Check if a type is void.
    fn is_void_type(&self, ty: &HirType) -> bool {
        matches!(
            ty,
            HirType::Named(named) if self.resolve_symbol(named.name) == "void"
        )
    }

    /// Helper: Resolve a symbol to its string representation.
    fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.symbol_table.resolve(sym).unwrap_or("<unknown>")
    }

    /// Helper: Convert HIR binary op to AST binary op.
    fn hir_binop_to_ast(&self, op: HirBinaryOp) -> Result<AstBinaryOp, MlirError> {
        Ok(match op {
            HirBinaryOp::Add => AstBinaryOp::Add,
            HirBinaryOp::Sub => AstBinaryOp::Sub,
            HirBinaryOp::Mul => AstBinaryOp::Mul,
            HirBinaryOp::Div => AstBinaryOp::Div,
            HirBinaryOp::Mod => AstBinaryOp::Mod,
            HirBinaryOp::Eq => AstBinaryOp::Eq,
            HirBinaryOp::Ne => AstBinaryOp::Ne,
            HirBinaryOp::Lt => AstBinaryOp::Lt,
            HirBinaryOp::Le => AstBinaryOp::Le,
            HirBinaryOp::Gt => AstBinaryOp::Gt,
            HirBinaryOp::Ge => AstBinaryOp::Ge,
            HirBinaryOp::And => AstBinaryOp::And,
            HirBinaryOp::Or => AstBinaryOp::Or,
        })
    }

    /// Helper: Convert HIR unary op to AST unary op.
    fn hir_unop_to_ast(&self, op: HirUnaryOp) -> Result<AstUnaryOp, MlirError> {
        Ok(match op {
            HirUnaryOp::Neg => AstUnaryOp::Neg,
            HirUnaryOp::Not => AstUnaryOp::Not,
        })
    }
}

// Stub implementation when MLIR feature is disabled
#[cfg(not(feature = "mlir"))]
pub struct HirToMlirLowering<'c> {
    _phantom: std::marker::PhantomData<&'c ()>,
}

#[cfg(not(feature = "mlir"))]
impl<'c> HirToMlirLowering<'c> {
    /// Creates a new HIR to MLIR lowering instance (stub).
    pub fn new(_context: &'c ()) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(all(test, feature = "mlir"))]
mod tests {
    use super::*;
    use crate::hir::symbol::Symbol;

    #[test]
    fn test_lowering_creation() {
        let context = Context::new();
        let _lowering = HirToMlirLowering::new(&context);
    }

    #[test]
    fn test_lower_simple_module() {
        let context = Context::new();
        let mut lowering = HirToMlirLowering::new(&context);

        let mut sym_table = SymbolTable::new();
        let mod_name = sym_table.intern("test");

        lowering = lowering.with_symbol_table(sym_table);

        let hir_module = HirModule {
            id: HirId::new(),
            name: mod_name,
            decls: vec![],
        };

        let result = lowering.lower_module(&hir_module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower_type_primitives() {
        let context = Context::new();
        let lowering = HirToMlirLowering::new(&context);

        let mut sym_table = SymbolTable::new();
        let i64_name = sym_table.intern("i64");
        let bool_name = sym_table.intern("bool");

        let i64_type = HirType::Named(HirNamedType {
            name: i64_name,
            args: vec![],
        });

        let bool_type = HirType::Named(HirNamedType {
            name: bool_name,
            args: vec![],
        });

        // Note: These will fail without symbol table set
        // Just testing that the method exists
        let _ = lowering.lower_type(&i64_type);
        let _ = lowering.lower_type(&bool_type);
    }

    #[test]
    fn test_binop_conversion() {
        let context = Context::new();
        let lowering = HirToMlirLowering::new(&context);

        assert!(matches!(
            lowering.hir_binop_to_ast(HirBinaryOp::Add),
            Ok(AstBinaryOp::Add)
        ));
        assert!(matches!(
            lowering.hir_binop_to_ast(HirBinaryOp::Sub),
            Ok(AstBinaryOp::Sub)
        ));
        assert!(matches!(
            lowering.hir_binop_to_ast(HirBinaryOp::Eq),
            Ok(AstBinaryOp::Eq)
        ));
    }

    #[test]
    fn test_unop_conversion() {
        let context = Context::new();
        let lowering = HirToMlirLowering::new(&context);

        assert!(matches!(
            lowering.hir_unop_to_ast(HirUnaryOp::Neg),
            Ok(AstUnaryOp::Neg)
        ));
        assert!(matches!(
            lowering.hir_unop_to_ast(HirUnaryOp::Not),
            Ok(AstUnaryOp::Not)
        ));
    }
}
