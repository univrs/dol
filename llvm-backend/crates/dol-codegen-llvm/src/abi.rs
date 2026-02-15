//! VUDO ABI - Host Function Declarations
//!
//! Declares the 22 VUDO host functions as LLVM external function declarations.
//! These functions are implemented by vudo-runtime-native and linked at build time.

use inkwell::context::Context;
use inkwell::module::Module;

use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

/// ABI generator for VUDO host functions
pub struct AbiGenerator<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
}

impl<'a, 'ctx> AbiGenerator<'a, 'ctx> {
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> Self {
        Self { context, module }
    }

    /// Declare all VUDO host functions
    pub fn declare_all_host_functions(&self) {
        // I/O
        self.declare_vudo_print();
        self.declare_vudo_println();
        self.declare_vudo_log();
        self.declare_vudo_error();

        // Memory
        self.declare_vudo_alloc();
        self.declare_vudo_free();
        self.declare_vudo_realloc();

        // Time
        self.declare_vudo_now();
        self.declare_vudo_sleep();
        self.declare_vudo_monotonic_now();

        // Messaging
        self.declare_vudo_send();
        self.declare_vudo_recv();
        self.declare_vudo_pending();
        self.declare_vudo_broadcast();
        self.declare_vudo_free_message();

        // Random
        self.declare_vudo_random();
        self.declare_vudo_random_bytes();

        // Effects
        self.declare_vudo_emit_effect();
        self.declare_vudo_subscribe();

        // String
        self.declare_vudo_string_concat();
        self.declare_vudo_i64_to_string();

        // Debug
        self.declare_vudo_breakpoint();
        self.declare_vudo_assert();
        self.declare_vudo_panic();
    }

    // === I/O Functions ===

    fn declare_vudo_print(&self) -> FunctionValue<'ctx> {
        // void vudo_print(const char* ptr, size_t len)
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_print", fn_type, None)
    }

    fn declare_vudo_println(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_println", fn_type, None)
    }

    fn declare_vudo_log(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type =
            void_type.fn_type(&[i32_type.into(), ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_log", fn_type, None)
    }

    fn declare_vudo_error(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type =
            void_type.fn_type(&[i32_type.into(), ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_error", fn_type, None)
    }

    // === Memory Functions ===

    fn declare_vudo_alloc(&self) -> FunctionValue<'ctx> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = ptr_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("vudo_alloc", fn_type, None)
    }

    fn declare_vudo_free(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let fn_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("vudo_free", fn_type, None)
    }

    fn declare_vudo_realloc(&self) -> FunctionValue<'ctx> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_realloc", fn_type, None)
    }

    // === Time Functions ===

    fn declare_vudo_now(&self) -> FunctionValue<'ctx> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        self.module.add_function("vudo_now", fn_type, None)
    }

    fn declare_vudo_sleep(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let i64_type = self.context.i64_type();
        let fn_type = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("vudo_sleep", fn_type, None)
    }

    fn declare_vudo_monotonic_now(&self) -> FunctionValue<'ctx> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        self.module
            .add_function("vudo_monotonic_now", fn_type, None)
    }

    // === Messaging Functions ===

    fn declare_vudo_send(&self) -> FunctionValue<'ctx> {
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = i32_type.fn_type(&[i32_type.into(), ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_send", fn_type, None)
    }

    fn declare_vudo_recv(&self) -> FunctionValue<'ctx> {
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let fn_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_recv", fn_type, None)
    }

    fn declare_vudo_pending(&self) -> FunctionValue<'ctx> {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        self.module.add_function("vudo_pending", fn_type, None)
    }

    fn declare_vudo_broadcast(&self) -> FunctionValue<'ctx> {
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = i32_type.fn_type(&[i32_type.into(), ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_broadcast", fn_type, None)
    }

    fn declare_vudo_free_message(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let fn_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("vudo_free_message", fn_type, None)
    }

    // === Random Functions ===

    fn declare_vudo_random(&self) -> FunctionValue<'ctx> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        self.module.add_function("vudo_random", fn_type, None)
    }

    fn declare_vudo_random_bytes(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_random_bytes", fn_type, None)
    }

    // === Effects Functions ===

    fn declare_vudo_emit_effect(&self) -> FunctionValue<'ctx> {
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = i32_type.fn_type(&[i32_type.into(), ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_emit_effect", fn_type, None)
    }

    fn declare_vudo_subscribe(&self) -> FunctionValue<'ctx> {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into()], false);
        self.module.add_function("vudo_subscribe", fn_type, None)
    }

    // === String Functions ===

    fn declare_vudo_string_concat(&self) -> FunctionValue<'ctx> {
        // void vudo_string_concat(ptr, len, ptr, len, *out_ptr, *out_len)
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = void_type.fn_type(
            &[
                ptr_type.into(),
                i64_type.into(), // str1
                ptr_type.into(),
                i64_type.into(), // str2
                ptr_type.into(),
                ptr_type.into(), // out_ptr, out_len
            ],
            false,
        );
        self.module
            .add_function("vudo_string_concat", fn_type, None)
    }

    fn declare_vudo_i64_to_string(&self) -> FunctionValue<'ctx> {
        // void vudo_i64_to_string(i64 value, *out_ptr, *out_len)
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type =
            void_type.fn_type(&[i64_type.into(), ptr_type.into(), ptr_type.into()], false);
        self.module
            .add_function("vudo_i64_to_string", fn_type, None)
    }

    // === Debug Functions ===

    fn declare_vudo_breakpoint(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        self.module.add_function("vudo_breakpoint", fn_type, None)
    }

    fn declare_vudo_assert(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type =
            void_type.fn_type(&[i32_type.into(), ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_assert", fn_type, None)
    }

    fn declare_vudo_panic(&self) -> FunctionValue<'ctx> {
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let fn_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vudo_panic", fn_type, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declare_all() {
        let context = Context::create();
        let module = context.create_module("test");
        let abi = AbiGenerator::new(&context, &module);
        abi.declare_all_host_functions();

        // Verify some functions exist
        assert!(module.get_function("vudo_print").is_some());
        assert!(module.get_function("vudo_alloc").is_some());
        assert!(module.get_function("vudo_send").is_some());
        assert!(module.get_function("vudo_panic").is_some());
    }
}
