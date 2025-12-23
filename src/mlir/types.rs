//! Type Lowering from DOL to MLIR
//!
//! This module handles the conversion of DOL's high-level type system
//! to MLIR's type system. This is a critical step in code generation,
//! as it determines how DOL values are represented in memory and
//! how operations on them are performed.
//!
//! # Type Mapping
//!
//! | DOL Type    | MLIR Type                          |
//! |-------------|-------------------------------------|
//! | Void        | None (unit type)                   |
//! | Bool        | i1                                  |
//! | Int8        | i8                                  |
//! | Int16       | i16                                 |
//! | Int32       | i32                                 |
//! | Int64       | i64                                 |
//! | UInt8       | i8 (unsigned semantics)            |
//! | UInt16      | i16 (unsigned semantics)           |
//! | UInt32      | i32 (unsigned semantics)           |
//! | UInt64      | i64 (unsigned semantics)           |
//! | Float32     | f32                                 |
//! | Float64     | f64                                 |
//! | String      | !llvm.ptr (pointer to i8 array)    |
//! | Function    | Function type                       |
//! | Tuple       | Struct type                         |
//! | Generic     | Specialized based on constructor   |
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::mlir::{MlirContext, TypeLowering};
//! use metadol::typechecker::Type;
//!
//! let ctx = MlirContext::new();
//! let lowering = TypeLowering::new(&ctx);
//!
//! let dol_type = Type::Int32;
//! let mlir_type = lowering.lower(&dol_type)?;
//! ```

use crate::mlir::MlirError;
use crate::typechecker::Type;

#[cfg(feature = "mlir")]
use melior::{
    ir::{
        r#type::{FunctionType, IntegerType},
        Type as MlirType,
    },
    Context,
};

/// Type lowering engine for converting DOL types to MLIR types.
///
/// This struct maintains a reference to the MLIR context and provides
/// methods to convert DOL's semantic types into their MLIR equivalents.
#[cfg(feature = "mlir")]
pub struct TypeLowering<'ctx> {
    context: &'ctx Context,
}

#[cfg(feature = "mlir")]
impl<'ctx> TypeLowering<'ctx> {
    /// Creates a new type lowering instance.
    ///
    /// # Arguments
    ///
    /// * `context` - Reference to the MLIR context
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let lowering = TypeLowering::new(ctx.context());
    /// ```
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Lowers a DOL type to its MLIR equivalent.
    ///
    /// This is the main entry point for type conversion. It handles
    /// all DOL types and converts them to appropriate MLIR representations.
    ///
    /// # Arguments
    ///
    /// * `ty` - The DOL type to lower
    ///
    /// # Returns
    ///
    /// The corresponding MLIR type, or an error if the type cannot be lowered
    ///
    /// # Errors
    ///
    /// Returns `MlirError` if:
    /// - The type is not supported (e.g., Type::Unknown, Type::Error)
    /// - The type has invalid parameters
    /// - A nested type cannot be lowered
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mlir_type = lowering.lower(&Type::Int32)?;
    /// ```
    pub fn lower(&self, ty: &Type) -> Result<MlirType<'ctx>, MlirError> {
        match ty {
            // Void type - represented as empty tuple in MLIR
            Type::Void => {
                // In MLIR, void is typically represented as a function with no return
                // For standalone void values, we use an empty tuple (unit type)
                Ok(MlirType::tuple(self.context, &[]))
            }

            // Boolean type - i1
            Type::Bool => Ok(IntegerType::new(self.context, 1).into()),

            // Signed integer types
            Type::Int8 => Ok(IntegerType::new(self.context, 8).into()),
            Type::Int16 => Ok(IntegerType::new(self.context, 16).into()),
            Type::Int32 => Ok(IntegerType::new(self.context, 32).into()),
            Type::Int64 => Ok(IntegerType::new(self.context, 64).into()),

            // Unsigned integer types
            // MLIR doesn't have separate unsigned types, signedness is determined by operations
            Type::UInt8 => Ok(IntegerType::new(self.context, 8).into()),
            Type::UInt16 => Ok(IntegerType::new(self.context, 16).into()),
            Type::UInt32 => Ok(IntegerType::new(self.context, 32).into()),
            Type::UInt64 => Ok(IntegerType::new(self.context, 64).into()),

            // Floating point types
            Type::Float32 => Ok(MlirType::float32(self.context)),
            Type::Float64 => Ok(MlirType::float64(self.context)),

            // String type - represented as pointer to i8 (LLVM style)
            Type::String => {
                // In MLIR, strings are typically represented as !llvm.ptr
                // For now, we use an opaque pointer type
                // In a full implementation, we'd use memref<i8> or similar
                Ok(MlirType::index(self.context)) // Placeholder - would need LLVM dialect
            }

            // Function types
            Type::Function {
                params,
                return_type,
            } => {
                let param_types: Result<Vec<_>, _> =
                    params.iter().map(|p| self.lower(p)).collect();
                let param_types = param_types?;

                let ret_type = self.lower(return_type)?;
                let results = if matches!(**return_type, Type::Void) {
                    vec![]
                } else {
                    vec![ret_type]
                };

                Ok(FunctionType::new(self.context, &param_types, &results).into())
            }

            // Tuple types - represented as MLIR struct/tuple types
            Type::Tuple(types) => {
                let element_types: Result<Vec<_>, _> =
                    types.iter().map(|t| self.lower(t)).collect();
                let element_types = element_types?;

                Ok(MlirType::tuple(self.context, &element_types))
            }

            // Generic types - handle known type constructors
            Type::Generic { name, args } => self.lower_generic(name, args),

            // Type variables - these should be resolved before lowering
            Type::Var(id) => Err(MlirError::new(format!(
                "cannot lower unresolved type variable ?{}",
                id
            ))),

            // Unknown type - error
            Type::Unknown => Err(MlirError::new(
                "cannot lower unknown type (type inference incomplete)",
            )),

            // Any type - error (needs concrete type)
            Type::Any => Err(MlirError::new(
                "cannot lower 'Any' type (requires concrete type)",
            )),

            // Error type - propagate error
            Type::Error => Err(MlirError::new("cannot lower error type")),
        }
    }

    /// Lowers generic/parametric types.
    ///
    /// Handles type constructors like List, Option, Array, etc.
    fn lower_generic(&self, name: &str, args: &[Type]) -> Result<MlirType<'ctx>, MlirError> {
        match name {
            // List<T> - represented as a dynamic array (memref)
            "List" | "Array" => {
                if args.is_empty() {
                    return Err(MlirError::new("List type requires type parameter"));
                }
                let elem_type = self.lower(&args[0])?;
                // In a full implementation, this would be memref<?xT>
                // For now, return the element type as placeholder
                Ok(elem_type)
            }

            // Option<T> - represented as tuple of (i1, T) where i1 is the "is present" flag
            "Option" | "Maybe" => {
                if args.is_empty() {
                    return Err(MlirError::new("Option type requires type parameter"));
                }
                let elem_type = self.lower(&args[0])?;
                let flag_type = IntegerType::new(self.context, 1).into();
                Ok(MlirType::tuple(self.context, &[flag_type, elem_type]))
            }

            // Result<T, E> - represented as tuple of (i1, T, E) where i1 indicates Ok/Err
            "Result" => {
                if args.len() != 2 {
                    return Err(MlirError::new("Result type requires two type parameters"));
                }
                let ok_type = self.lower(&args[0])?;
                let err_type = self.lower(&args[1])?;
                let tag_type = IntegerType::new(self.context, 1).into();
                Ok(MlirType::tuple(
                    self.context,
                    &[tag_type, ok_type, err_type],
                ))
            }

            // Quoted<T> - metaprogramming, represented as opaque type
            "Quoted" => {
                // For quoted expressions, we might want to store the AST
                // For now, use an opaque representation
                Ok(MlirType::index(self.context)) // Placeholder
            }

            // TypeInfo - reflection metadata, opaque type
            "TypeInfo" => Ok(MlirType::index(self.context)), // Placeholder

            // Unknown generic type
            _ => Err(MlirError::new(format!(
                "unsupported generic type constructor: {}",
                name
            ))),
        }
    }
}

// Stub implementation when MLIR feature is disabled
#[cfg(not(feature = "mlir"))]
pub struct TypeLowering<'ctx> {
    _phantom: std::marker::PhantomData<&'ctx ()>,
}

#[cfg(not(feature = "mlir"))]
impl<'ctx> TypeLowering<'ctx> {
    /// Creates a new type lowering instance (stub).
    pub fn new(_context: &'ctx ()) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Lowers a DOL type (stub - always returns error).
    pub fn lower(&self, ty: &Type) -> Result<(), MlirError> {
        Err(MlirError::new(format!(
            "MLIR feature not enabled, cannot lower type: {}",
            ty
        )))
    }
}

#[cfg(all(test, feature = "mlir"))]
mod tests {
    use super::*;
    use crate::mlir::MlirContext;

    #[test]
    fn test_lower_primitives() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        // Test boolean
        assert!(lowering.lower(&Type::Bool).is_ok());

        // Test integers
        assert!(lowering.lower(&Type::Int8).is_ok());
        assert!(lowering.lower(&Type::Int16).is_ok());
        assert!(lowering.lower(&Type::Int32).is_ok());
        assert!(lowering.lower(&Type::Int64).is_ok());

        // Test unsigned integers
        assert!(lowering.lower(&Type::UInt8).is_ok());
        assert!(lowering.lower(&Type::UInt16).is_ok());
        assert!(lowering.lower(&Type::UInt32).is_ok());
        assert!(lowering.lower(&Type::UInt64).is_ok());

        // Test floats
        assert!(lowering.lower(&Type::Float32).is_ok());
        assert!(lowering.lower(&Type::Float64).is_ok());
    }

    #[test]
    fn test_lower_void() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        assert!(lowering.lower(&Type::Void).is_ok());
    }

    #[test]
    fn test_lower_function() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        let func_type = Type::Function {
            params: vec![Type::Int32, Type::Bool],
            return_type: Box::new(Type::Float64),
        };

        assert!(lowering.lower(&func_type).is_ok());
    }

    #[test]
    fn test_lower_tuple() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        let tuple_type = Type::Tuple(vec![Type::Int32, Type::Bool, Type::Float64]);

        assert!(lowering.lower(&tuple_type).is_ok());
    }

    #[test]
    fn test_lower_generic_option() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        let option_type = Type::Generic {
            name: "Option".to_string(),
            args: vec![Type::Int32],
        };

        assert!(lowering.lower(&option_type).is_ok());
    }

    #[test]
    fn test_lower_generic_list() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        let list_type = Type::Generic {
            name: "List".to_string(),
            args: vec![Type::String],
        };

        assert!(lowering.lower(&list_type).is_ok());
    }

    #[test]
    fn test_lower_error_unknown() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        assert!(lowering.lower(&Type::Unknown).is_err());
        assert!(lowering.lower(&Type::Error).is_err());
        assert!(lowering.lower(&Type::Any).is_err());
    }

    #[test]
    fn test_lower_error_unresolved_var() {
        let ctx = MlirContext::new();
        let lowering = TypeLowering::new(ctx.context());

        assert!(lowering.lower(&Type::Var(42)).is_err());
    }
}
