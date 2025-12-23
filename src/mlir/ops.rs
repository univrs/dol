//! MLIR operation builders for Metal DOL.
//!
//! This module provides the `OpBuilder` struct for constructing MLIR operations
//! from DOL AST nodes. It supports arithmetic, control flow, and function operations
//! using the melior crate's dialect system.
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::mlir::OpBuilder;
//! use melior::Context;
//!
//! let context = Context::new();
//! let builder = OpBuilder::new(&context);
//!
//! // Build an addition operation
//! let location = Location::unknown(&context);
//! let add_op = builder.build_binary_arith(BinaryOp::Add, lhs, rhs, location)?;
//! ```

#[cfg(feature = "mlir")]
use super::MlirError;

#[cfg(feature = "mlir")]
use crate::ast::{BinaryOp, UnaryOp};

#[cfg(feature = "mlir")]
use melior::{
    dialect::{arith, func, scf},
    ir::{
        attribute::{IntegerAttribute, StringAttribute, TypeAttribute},
        operation::OperationBuilder,
        r#type::{FunctionType, IntegerType},
        Block, Location, Operation, Region, Type, Value, ValueLike,
    },
    Context,
};

/// Builder for MLIR operations.
///
/// This struct provides methods to construct MLIR operations from DOL AST nodes.
/// It holds a reference to the MLIR context needed for operation creation.
#[cfg(feature = "mlir")]
pub struct OpBuilder<'c> {
    /// Reference to the MLIR context
    context: &'c Context,
}

#[cfg(feature = "mlir")]
impl<'c> OpBuilder<'c> {
    /// Creates a new operation builder with the given context.
    ///
    /// # Arguments
    ///
    /// * `context` - The MLIR context to use for creating operations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let context = Context::new();
    /// let builder = OpBuilder::new(&context);
    /// ```
    pub fn new(context: &'c Context) -> Self {
        Self { context }
    }

    /// Builds a binary arithmetic or comparison operation.
    ///
    /// This method handles integer arithmetic (add, sub, mul, div, mod) and
    /// comparison operations (eq, ne, lt, le, gt, ge) using the arith dialect.
    ///
    /// # Arguments
    ///
    /// * `op` - The binary operator
    /// * `lhs` - Left-hand side value
    /// * `rhs` - Right-hand side value
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created operation on success, or an error if the operation is unsupported.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let add_op = builder.build_binary_arith(
    ///     BinaryOp::Add,
    ///     lhs_value,
    ///     rhs_value,
    ///     location
    /// )?;
    /// ```
    pub fn build_binary_arith(
        &self,
        op: BinaryOp,
        lhs: Value<'c, '_>,
        rhs: Value<'c, '_>,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        match op {
            BinaryOp::Add => Ok(arith::addi(lhs, rhs, location).into()),
            BinaryOp::Sub => Ok(arith::subi(lhs, rhs, location).into()),
            BinaryOp::Mul => Ok(arith::muli(lhs, rhs, location).into()),
            BinaryOp::Div => Ok(arith::divsi(lhs, rhs, location).into()),
            BinaryOp::Mod => Ok(arith::remsi(lhs, rhs, location).into()),
            BinaryOp::Eq => {
                Ok(arith::cmpi(self.context, arith::CmpiPredicate::Eq, lhs, rhs, location).into())
            }
            BinaryOp::Ne => {
                Ok(arith::cmpi(self.context, arith::CmpiPredicate::Ne, lhs, rhs, location).into())
            }
            BinaryOp::Lt => {
                Ok(arith::cmpi(self.context, arith::CmpiPredicate::Slt, lhs, rhs, location).into())
            }
            BinaryOp::Le => {
                Ok(arith::cmpi(self.context, arith::CmpiPredicate::Sle, lhs, rhs, location).into())
            }
            BinaryOp::Gt => {
                Ok(arith::cmpi(self.context, arith::CmpiPredicate::Sgt, lhs, rhs, location).into())
            }
            BinaryOp::Ge => {
                Ok(arith::cmpi(self.context, arith::CmpiPredicate::Sge, lhs, rhs, location).into())
            }
            BinaryOp::And => Ok(arith::andi(lhs, rhs, location).into()),
            BinaryOp::Or => Ok(arith::ori(lhs, rhs, location).into()),
            BinaryOp::Pow => Err(MlirError::new(
                "exponentiation (^) requires math dialect or custom implementation",
            )),
            BinaryOp::Pipe | BinaryOp::Compose | BinaryOp::Apply | BinaryOp::Bind => {
                Err(MlirError::new(format!(
                    "functional operator {:?} not supported in MLIR lowering",
                    op
                )))
            }
            BinaryOp::Member => Err(MlirError::new(
                "member access requires struct/object lowering",
            )),
            BinaryOp::Map | BinaryOp::Ap => Err(MlirError::new(format!(
                "functor operator {:?} requires custom dialect",
                op
            ))),
        }
    }

    /// Builds a binary floating-point operation.
    ///
    /// This method handles floating-point arithmetic operations using the arith dialect.
    ///
    /// # Arguments
    ///
    /// * `op` - The binary operator
    /// * `lhs` - Left-hand side value
    /// * `rhs` - Right-hand side value
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created operation on success, or an error if the operation is unsupported.
    pub fn build_binary_float(
        &self,
        op: BinaryOp,
        lhs: Value<'c, '_>,
        rhs: Value<'c, '_>,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        match op {
            BinaryOp::Add => Ok(arith::addf(lhs, rhs, location).into()),
            BinaryOp::Sub => Ok(arith::subf(lhs, rhs, location).into()),
            BinaryOp::Mul => Ok(arith::mulf(lhs, rhs, location).into()),
            BinaryOp::Div => Ok(arith::divf(lhs, rhs, location).into()),
            BinaryOp::Mod => Ok(arith::remf(lhs, rhs, location).into()),
            BinaryOp::Eq => {
                Ok(arith::cmpf(self.context, arith::CmpfPredicate::Oeq, lhs, rhs, location).into())
            }
            BinaryOp::Ne => {
                Ok(arith::cmpf(self.context, arith::CmpfPredicate::One, lhs, rhs, location).into())
            }
            BinaryOp::Lt => {
                Ok(arith::cmpf(self.context, arith::CmpfPredicate::Olt, lhs, rhs, location).into())
            }
            BinaryOp::Le => {
                Ok(arith::cmpf(self.context, arith::CmpfPredicate::Ole, lhs, rhs, location).into())
            }
            BinaryOp::Gt => {
                Ok(arith::cmpf(self.context, arith::CmpfPredicate::Ogt, lhs, rhs, location).into())
            }
            BinaryOp::Ge => {
                Ok(arith::cmpf(self.context, arith::CmpfPredicate::Oge, lhs, rhs, location).into())
            }
            _ => Err(MlirError::new(format!(
                "operator {:?} not supported for floating-point",
                op
            ))),
        }
    }

    /// Builds a unary operation.
    ///
    /// This method handles unary operations like negation and logical not.
    ///
    /// # Arguments
    ///
    /// * `op` - The unary operator
    /// * `operand` - The operand value
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created operation on success, or an error if the operation is unsupported.
    pub fn build_unary(
        &self,
        op: UnaryOp,
        operand: Value<'c, '_>,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        match op {
            UnaryOp::Neg => {
                // Negate by subtracting from zero
                let zero = self.build_constant_i64(0, location)?;
                let zero_value = zero.result(0)?.into();
                Ok(arith::subi(zero_value, operand, location).into())
            }
            UnaryOp::Not => {
                // Logical not by XOR with true (i1 = 1)
                let one = self.build_constant_i1(true, location)?;
                let one_value = one.result(0)?.into();
                Ok(arith::xori(operand, one_value, location).into())
            }
            UnaryOp::Quote | UnaryOp::Reflect => Err(MlirError::new(format!(
                "metaprogramming operator {:?} not supported in MLIR lowering",
                op
            ))),
        }
    }

    /// Builds a 64-bit integer constant.
    ///
    /// # Arguments
    ///
    /// * `value` - The constant value
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created constant operation.
    pub fn build_constant_i64(
        &self,
        value: i64,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        let r#type = IntegerType::new(self.context, 64).into();
        Ok(arith::constant(
            self.context,
            IntegerAttribute::new(value.into(), r#type).into(),
            location,
        )
        .into())
    }

    /// Builds a boolean (i1) constant.
    ///
    /// # Arguments
    ///
    /// * `value` - The boolean value
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created constant operation.
    pub fn build_constant_i1(
        &self,
        value: bool,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        let r#type = IntegerType::new(self.context, 1).into();
        let int_value = if value { 1i64 } else { 0i64 };
        Ok(arith::constant(
            self.context,
            IntegerAttribute::new(int_value.into(), r#type).into(),
            location,
        )
        .into())
    }

    /// Builds an if operation using scf.if.
    ///
    /// # Arguments
    ///
    /// * `condition` - The condition value (must be i1)
    /// * `result_types` - The types of values returned by the if
    /// * `then_region` - The region to execute when condition is true
    /// * `else_region` - Optional region to execute when condition is false
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created if operation.
    pub fn build_if(
        &self,
        condition: Value<'c, '_>,
        result_types: &[Type<'c>],
        then_region: Region<'c>,
        else_region: Option<Region<'c>>,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        let regions = if let Some(else_r) = else_region {
            vec![then_region, else_r]
        } else {
            vec![then_region]
        };

        let mut op_builder = OperationBuilder::new("scf.if", location)
            .add_operands(&[condition])
            .add_results(result_types);

        for region in regions {
            op_builder = op_builder.add_regions([region]);
        }

        op_builder
            .build()
            .map_err(|e| MlirError::new(format!("failed to create scf.if operation: {}", e)))
    }

    /// Builds a for loop using scf.for.
    ///
    /// # Arguments
    ///
    /// * `lower_bound` - The loop's lower bound
    /// * `upper_bound` - The loop's upper bound
    /// * `step` - The loop step
    /// * `init_args` - Initial values for loop-carried variables
    /// * `body_region` - The loop body region
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created for loop operation.
    pub fn build_for(
        &self,
        lower_bound: Value<'c, '_>,
        upper_bound: Value<'c, '_>,
        step: Value<'c, '_>,
        init_args: &[Value<'c, '_>],
        body_region: Region<'c>,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        let mut operands = vec![lower_bound, upper_bound, step];
        operands.extend_from_slice(init_args);

        let result_types: Vec<Type> = init_args.iter().map(|v| v.r#type()).collect();

        OperationBuilder::new("scf.for", location)
            .add_operands(&operands)
            .add_results(&result_types)
            .add_regions([body_region])
            .build()
            .map_err(|e| MlirError::new(format!("failed to create scf.for operation: {}", e)))
    }

    /// Builds a while loop using scf.while.
    ///
    /// # Arguments
    ///
    /// * `init_args` - Initial values for loop-carried variables
    /// * `before_region` - The condition region
    /// * `after_region` - The body region
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created while loop operation.
    pub fn build_while(
        &self,
        init_args: &[Value<'c, '_>],
        before_region: Region<'c>,
        after_region: Region<'c>,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        let result_types: Vec<Type> = init_args.iter().map(|v| v.r#type()).collect();

        OperationBuilder::new("scf.while", location)
            .add_operands(init_args)
            .add_results(&result_types)
            .add_regions([before_region, after_region])
            .build()
            .map_err(|e| MlirError::new(format!("failed to create scf.while operation: {}", e)))
    }

    /// Builds a function declaration using func.func.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name
    /// * `function_type` - The function signature (params -> results)
    /// * `body_region` - The function body
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created function operation.
    pub fn build_func(
        &self,
        name: &str,
        function_type: FunctionType<'c>,
        body_region: Region<'c>,
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        OperationBuilder::new("func.func", location)
            .add_attributes(&[
                (
                    StringAttribute::new(self.context, "sym_name").into(),
                    StringAttribute::new(self.context, name).into(),
                ),
                (
                    StringAttribute::new(self.context, "function_type").into(),
                    TypeAttribute::new(function_type.into()).into(),
                ),
            ])
            .add_regions([body_region])
            .build()
            .map_err(|e| MlirError::new(format!("failed to create func.func operation: {}", e)))
    }

    /// Builds a function call using func.call.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name of the function to call
    /// * `arguments` - The call arguments
    /// * `result_types` - The types of returned values
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created call operation.
    pub fn build_call(
        &self,
        callee: &str,
        arguments: &[Value<'c, '_>],
        result_types: &[Type<'c>],
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        OperationBuilder::new("func.call", location)
            .add_attributes(&[(
                StringAttribute::new(self.context, "callee").into(),
                StringAttribute::new(self.context, callee).into(),
            )])
            .add_operands(arguments)
            .add_results(result_types)
            .build()
            .map_err(|e| MlirError::new(format!("failed to create func.call operation: {}", e)))
    }

    /// Builds a return statement using func.return.
    ///
    /// # Arguments
    ///
    /// * `operands` - The values to return
    /// * `location` - Source location for debugging
    ///
    /// # Returns
    ///
    /// The created return operation.
    pub fn build_return(
        &self,
        operands: &[Value<'c, '_>],
        location: Location<'c>,
    ) -> Result<Operation<'c>, MlirError> {
        OperationBuilder::new("func.return", location)
            .add_operands(operands)
            .build()
            .map_err(|e| MlirError::new(format!("failed to create func.return operation: {}", e)))
    }
}

#[cfg(all(test, feature = "mlir"))]
mod tests {
    use super::*;
    use melior::ir::Location;

    #[test]
    fn test_opbuilder_creation() {
        let context = Context::new();
        let _builder = OpBuilder::new(&context);
        // If we get here without panic, the builder was created successfully
    }

    #[test]
    fn test_build_constant_i64() {
        let context = Context::new();
        let builder = OpBuilder::new(&context);
        let location = Location::unknown(&context);

        let const_op = builder.build_constant_i64(42, location);
        assert!(const_op.is_ok());
    }

    #[test]
    fn test_build_constant_i1() {
        let context = Context::new();
        let builder = OpBuilder::new(&context);
        let location = Location::unknown(&context);

        let const_op = builder.build_constant_i1(true, location);
        assert!(const_op.is_ok());
    }

    #[test]
    fn test_unsupported_operations() {
        let context = Context::new();
        let builder = OpBuilder::new(&context);
        let location = Location::unknown(&context);

        // Create dummy values for testing
        let const_op = builder.build_constant_i64(1, location).unwrap();
        let dummy_value = const_op.result(0).unwrap().into();

        // Test unsupported binary operations
        let result = builder.build_binary_arith(BinaryOp::Pow, dummy_value, dummy_value, location);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("exponentiation"));

        let result = builder.build_binary_arith(BinaryOp::Pipe, dummy_value, dummy_value, location);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("functional operator"));

        // Test unsupported unary operations
        let result = builder.build_unary(UnaryOp::Quote, dummy_value, location);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("metaprogramming operator"));
    }
}
