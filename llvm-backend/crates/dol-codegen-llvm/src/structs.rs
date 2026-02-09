//! DOL Gen → LLVM Struct Code Generation
//!
//! Generates LLVM IR for DOL gen (struct) declarations.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;

use crate::types::TypeMapper;
use crate::Result;

/// Struct code generator for DOL gen declarations
pub struct StructCodegen<'a, 'ctx> {
    context: &'ctx Context,
    #[allow(dead_code)]
    module: &'a Module<'ctx>,
    type_mapper: TypeMapper<'ctx>,
}

impl<'a, 'ctx> StructCodegen<'a, 'ctx> {
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> Self {
        Self {
            context,
            module,
            type_mapper: TypeMapper::new(context),
        }
    }

    /// Generate a simple Point struct (x: i64, y: i64)
    /// (Prototype for testing the pipeline)
    pub fn generate_point_struct(&self) -> Result<StructType<'ctx>> {
        let i64_type = self.type_mapper.i64_type();
        let struct_type = self.type_mapper.create_struct(
            "Point",
            &[i64_type.into(), i64_type.into()],
        );
        Ok(struct_type)
    }

    /// Generate a Spirit identity struct
    pub fn generate_spirit_identity(&self) -> Result<StructType<'ctx>> {
        let string_type = self.type_mapper.string_type();
        let i64_type = self.type_mapper.i64_type();
        
        // SpiritIdentity { name, did, public_key, created_at, description, avatar_ref }
        let struct_type = self.context.opaque_struct_type("SpiritIdentity");
        struct_type.set_body(
            &[
                string_type.into(), // name
                string_type.into(), // did
                string_type.into(), // public_key
                i64_type.into(),    // created_at
                string_type.into(), // description
                string_type.into(), // avatar_ref
            ],
            false,
        );
        
        Ok(struct_type)
    }

    // TODO: Generate DOL gen from HIR
    // pub fn generate_gen(&self, gen_decl: &GenDecl) -> Result<StructType<'ctx>> {
    //     ...
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_point() {
        let context = Context::create();
        let module = context.create_module("test");
        let codegen = StructCodegen::new(&context, &module);
        let struct_type = codegen.generate_point_struct();
        assert!(struct_type.is_ok());
    }

    #[test]
    fn test_generate_spirit_identity() {
        let context = Context::create();
        let module = context.create_module("test");
        let codegen = StructCodegen::new(&context, &module);
        let struct_type = codegen.generate_spirit_identity();
        assert!(struct_type.is_ok());
    }
}
