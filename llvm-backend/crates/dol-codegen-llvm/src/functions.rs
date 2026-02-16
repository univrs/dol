//! DOL Function → LLVM Function Code Generation
//!
//! Generates LLVM IR for DOL function declarations.

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use crate::types::TypeMapper;
use crate::Result;

/// Function code generator
pub struct FunctionCodegen<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: Builder<'ctx>,
    type_mapper: TypeMapper<'ctx>,
}

impl<'a, 'ctx> FunctionCodegen<'a, 'ctx> {
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> Self {
        Self {
            context,
            module,
            builder: context.create_builder(),
            type_mapper: TypeMapper::new(context),
        }
    }

    /// Generate a simple function that adds two i64 values
    /// (Prototype for testing the pipeline)
    pub fn generate_add_function(&self) -> Result<FunctionValue<'ctx>> {
        let i64_type = self.type_mapper.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("add", fn_type, None);

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        let a = function.get_nth_param(0).unwrap().into_int_value();
        let b = function.get_nth_param(1).unwrap().into_int_value();
        let sum = self.builder.build_int_add(a, b, "sum").unwrap();
        self.builder.build_return(Some(&sum)).unwrap();

        Ok(function)
    }

    // TODO: Generate DOL function from HIR
    // pub fn generate_function(&self, fun_decl: &FunDecl) -> Result<FunctionValue<'ctx>> {
    //     ...
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_add() {
        let context = Context::create();
        let module = context.create_module("test");
        let codegen = FunctionCodegen::new(&context, &module);
        let func = codegen.generate_add_function();
        assert!(func.is_ok());
    }
}
