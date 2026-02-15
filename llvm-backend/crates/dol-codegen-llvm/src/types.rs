//! DOL → LLVM Type Mapping
//!
//! Maps DOL types to their LLVM IR representations.

use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, FloatType, IntType, StructType};

/// DOL type to LLVM type mapper
pub struct TypeMapper<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeMapper<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Map DOL primitive types to LLVM types
    pub fn map_primitive(&self, dol_type: &str) -> Option<BasicTypeEnum<'ctx>> {
        match dol_type {
            // Integers
            "i8" => Some(self.context.i8_type().into()),
            "i16" => Some(self.context.i16_type().into()),
            "i32" => Some(self.context.i32_type().into()),
            "i64" => Some(self.context.i64_type().into()),
            "i128" => Some(self.context.i128_type().into()),

            // Unsigned (same representation, different semantics)
            "u8" => Some(self.context.i8_type().into()),
            "u16" => Some(self.context.i16_type().into()),
            "u32" => Some(self.context.i32_type().into()),
            "u64" => Some(self.context.i64_type().into()),
            "u128" => Some(self.context.i128_type().into()),

            // Floats
            "f32" => Some(self.context.f32_type().into()),
            "f64" => Some(self.context.f64_type().into()),

            // Boolean (i1 in LLVM)
            "bool" => Some(self.context.bool_type().into()),

            // String (pointer + length)
            // Handled specially as struct { i8*, i64 }
            "string" => None, // Complex type, handle separately

            _ => None,
        }
    }

    /// Get the i32 type (commonly used)
    pub fn i32_type(&self) -> IntType<'ctx> {
        self.context.i32_type()
    }

    /// Get the i64 type (commonly used)
    pub fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    /// Get the f64 type (commonly used)
    pub fn f64_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    /// Get the bool type
    pub fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    /// Create a string type (fat pointer: ptr + len)
    pub fn string_type(&self) -> StructType<'ctx> {
        self.context.struct_type(
            &[
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context.i64_type().into(),
            ],
            false,
        )
    }

    /// Create a struct type for a DOL gen
    pub fn create_struct(
        &self,
        name: &str,
        field_types: &[BasicTypeEnum<'ctx>],
    ) -> StructType<'ctx> {
        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(field_types, false);
        struct_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_mapping() {
        let context = Context::create();
        let mapper = TypeMapper::new(&context);

        assert!(mapper.map_primitive("i32").is_some());
        assert!(mapper.map_primitive("f64").is_some());
        assert!(mapper.map_primitive("bool").is_some());
        assert!(mapper.map_primitive("unknown").is_none());
    }
}
