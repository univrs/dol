//! Native host ABI and runtime intrinsics.
//!
//! # Host ABI (shared with the VUDO VM)
//!
//! The canonical Spirit host ABI is the VUDO VM's 15 `host_*` functions
//! (`src/wasm/host_abi.rs` at the repository root, mirroring
//! `univrs/vudo` `vudo_vm/src/linker.rs`). A bodiless `sex fun` named after one
//! of them compiles, in the WASM backend, to an import from module `vudo`, and
//! in this backend, to a call of the C-ABI function of the same name exported
//! by `vudo-runtime-native`. One DOL source therefore runs in both.
//!
//! Native signatures keep the VM's integer types. The only difference is that
//! a WASM `i32` guest-memory address becomes a native pointer:
//!
//! | Host function            | VM (WASM) signature              | Native C signature                                   |
//! |--------------------------|----------------------------------|------------------------------------------------------|
//! | `host_time_now`          | `() -> i64`                      | `() -> i64` (Unix ns)                                |
//! | `host_random_bytes`      | `(i32 ptr, i32 len) -> i32`      | `(ptr buf, i32 len) -> i32`                          |
//! | `host_log`               | `(i32 level, i32 ptr, i32 len) -> i32` | `(i32 level, ptr msg, i32 len) -> i32`         |
//! | `host_storage_read`      | `(i32, i32, i32, i32) -> i32`    | `(ptr key, i32 key_len, ptr buf, i32 cap) -> i32`    |
//! | `host_storage_write`     | `(i32, i32, i32, i32) -> i32`    | `(ptr key, i32 key_len, ptr val, i32 val_len) -> i32`|
//! | `host_storage_delete`    | `(i32, i32) -> i32`              | `(ptr key, i32 key_len) -> i32`                      |
//! | `host_network_connect`   | `(i32, i32) -> i64`              | `(ptr addr, i32 addr_len) -> i64`                    |
//! | `host_network_listen`    | `(i32) -> i64`                   | `(i32 port) -> i64`                                  |
//! | `host_network_broadcast` | `(i32, i32) -> i64`              | `(ptr msg, i32 msg_len) -> i64`                      |
//! | `host_credit_balance`    | `(i32) -> i64`                   | `(ptr account) -> i64`                               |
//! | `host_credit_transfer`   | `(i32, i32, i64) -> i32`         | `(ptr from, ptr to, i64 amount) -> i32`              |
//! | `host_credit_reserve`    | `(i32, i64) -> i64`              | `(ptr account, i64 amount) -> i64`                   |
//! | `host_credit_release`    | `(i64) -> i32`                   | `(i64 reservation) -> i32`                           |
//! | `host_credit_consume`    | `(i64) -> i32`                   | `(i64 reservation) -> i32`                           |
//! | `host_credit_available`  | `(i32) -> i64`                   | `(ptr account) -> i64`                               |
//!
//! Lowering of a call to a declared host function (see
//! `HirLowering::lower_function_decl` / `lower_call`):
//!
//! - A DOL `string` argument (native fat string `{ptr, i64}`) is split into
//!   `(ptr, i32 len)`, exactly where the VM ABI has a `(ptr, len)` pair
//!   ([`HostFunction::str_pairs`]). A `string` anywhere else is a compile error.
//! - An integer argument is sign-extended or truncated to the slot width. In a
//!   pointer slot, a pointer value (e.g. from `alloc`) is passed as is and an
//!   integer is converted with `inttoptr`.
//! - The `i32`/`i64` result is converted to the declared DOL return type.
//! - A bodiless function that is not one of these 15, or whose parameters do
//!   not cover the ABI exactly, is a compile error, as in the WASM backend.
//!
//! The 15 host functions are declared up front; a DOL `sex fun` declaration
//! reuses that declaration instead of adding a second one.
//!
//! # Runtime intrinsics (native only, not the host ABI)
//!
//! `vudo_print`, `vudo_println`, `vudo_alloc`, `vudo_free`, `vudo_realloc`,
//! `vudo_string_concat`, `vudo_i64_to_string`, `vudo_breakpoint`,
//! `vudo_assert` and `vudo_panic` implement DOL built-ins (`print`,
//! `println`, `alloc`, ..., string `+`, `.to_string()`). They are emitted by
//! the compiler, never declared by DOL source, and do not exist in the VM: a
//! Spirit that must also run in the VM logs through `host_log`.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

/// Native type of one host ABI parameter or result slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// 32-bit integer.
    I32,
    /// 64-bit integer.
    I64,
    /// Native pointer (a WASM `i32` guest address in the VM).
    Ptr,
}

/// One canonical host function, with its native C signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostFunction {
    /// Symbol name (always `host_*`).
    pub name: &'static str,
    /// Native parameter slots.
    pub params: &'static [Slot],
    /// Result slot (always an integer).
    pub result: Slot,
    /// Indices `i` where params `(i, i + 1)` are an input byte string
    /// `(ptr, i32 len)`. A DOL `string` argument lowers to such a pair.
    pub str_pairs: &'static [usize],
}

use Slot::{Ptr, I32, I64};

/// The VUDO VM's 15 host functions, in linker registration order.
pub const HOST_FUNCTIONS: &[HostFunction] = &[
    HostFunction {
        name: "host_time_now",
        params: &[],
        result: I64,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_random_bytes",
        params: &[Ptr, I32],
        result: I32,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_log",
        params: &[I32, Ptr, I32],
        result: I32,
        str_pairs: &[1],
    },
    HostFunction {
        name: "host_storage_read",
        params: &[Ptr, I32, Ptr, I32],
        result: I32,
        str_pairs: &[0],
    },
    HostFunction {
        name: "host_storage_write",
        params: &[Ptr, I32, Ptr, I32],
        result: I32,
        str_pairs: &[0, 2],
    },
    HostFunction {
        name: "host_storage_delete",
        params: &[Ptr, I32],
        result: I32,
        str_pairs: &[0],
    },
    HostFunction {
        name: "host_network_connect",
        params: &[Ptr, I32],
        result: I64,
        str_pairs: &[0],
    },
    HostFunction {
        name: "host_network_listen",
        params: &[I32],
        result: I64,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_network_broadcast",
        params: &[Ptr, I32],
        result: I64,
        str_pairs: &[0],
    },
    HostFunction {
        name: "host_credit_balance",
        params: &[Ptr],
        result: I64,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_credit_transfer",
        params: &[Ptr, Ptr, I64],
        result: I32,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_credit_reserve",
        params: &[Ptr, I64],
        result: I64,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_credit_release",
        params: &[I64],
        result: I32,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_credit_consume",
        params: &[I64],
        result: I32,
        str_pairs: &[],
    },
    HostFunction {
        name: "host_credit_available",
        params: &[Ptr],
        result: I64,
        str_pairs: &[],
    },
];

/// Look up a host function by name.
pub fn lookup(name: &str) -> Option<&'static HostFunction> {
    HOST_FUNCTIONS.iter().find(|f| f.name == name)
}

/// Comma-separated list of all host function names (for diagnostics).
pub fn names() -> String {
    HOST_FUNCTIONS
        .iter()
        .map(|f| f.name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Declares host functions and runtime intrinsics in an LLVM module.
pub struct AbiGenerator<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
}

impl<'a, 'ctx> AbiGenerator<'a, 'ctx> {
    /// Create a generator for `module`.
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> Self {
        Self { context, module }
    }

    /// Declare the 15 host functions and all runtime intrinsics.
    pub fn declare_all_host_functions(&self) {
        for host in HOST_FUNCTIONS {
            self.declare_host_function(host);
        }
        self.declare_intrinsics();
    }

    /// Declare one host function with its native C signature.
    pub fn declare_host_function(&self, host: &HostFunction) -> FunctionValue<'ctx> {
        let params: Vec<BasicMetadataTypeEnum<'ctx>> =
            host.params.iter().map(|s| self.slot_type(*s)).collect();
        let fn_type = match host.result {
            I32 => self.context.i32_type().fn_type(&params, false),
            I64 => self.context.i64_type().fn_type(&params, false),
            Ptr => self.ptr().fn_type(&params, false),
        };
        self.module.add_function(host.name, fn_type, None)
    }

    fn slot_type(&self, slot: Slot) -> BasicMetadataTypeEnum<'ctx> {
        match slot {
            I32 => self.context.i32_type().into(),
            I64 => self.context.i64_type().into(),
            Ptr => self.ptr().into(),
        }
    }

    fn ptr(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(AddressSpace::default())
    }

    /// Declare the native-only runtime intrinsics behind DOL built-ins.
    fn declare_intrinsics(&self) {
        let void = self.context.void_type();
        let i32_t = self.context.i32_type();
        let i64_t = self.context.i64_type();
        let ptr = self.ptr();
        let decl = |name: &str, ty| {
            self.module.add_function(name, ty, None);
        };

        // void vudo_print(ptr, len) / vudo_println(ptr, len)
        decl(
            "vudo_print",
            void.fn_type(&[ptr.into(), i64_t.into()], false),
        );
        decl(
            "vudo_println",
            void.fn_type(&[ptr.into(), i64_t.into()], false),
        );

        // Memory
        decl("vudo_alloc", ptr.fn_type(&[i64_t.into()], false));
        decl("vudo_free", void.fn_type(&[ptr.into()], false));
        decl(
            "vudo_realloc",
            ptr.fn_type(&[ptr.into(), i64_t.into()], false),
        );

        // void vudo_string_concat(ptr, len, ptr, len, *out_ptr, *out_len)
        decl(
            "vudo_string_concat",
            void.fn_type(
                &[
                    ptr.into(),
                    i64_t.into(),
                    ptr.into(),
                    i64_t.into(),
                    ptr.into(),
                    ptr.into(),
                ],
                false,
            ),
        );
        // void vudo_i64_to_string(i64 value, *out_ptr, *out_len)
        decl(
            "vudo_i64_to_string",
            void.fn_type(&[i64_t.into(), ptr.into(), ptr.into()], false),
        );

        // Debug
        decl("vudo_breakpoint", void.fn_type(&[], false));
        decl(
            "vudo_assert",
            void.fn_type(&[i32_t.into(), ptr.into(), i64_t.into()], false),
        );
        decl(
            "vudo_panic",
            void.fn_type(&[ptr.into(), i64_t.into()], false),
        );
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

        for host in HOST_FUNCTIONS {
            let f = module.get_function(host.name).expect(host.name);
            assert_eq!(
                f.count_params() as usize,
                host.params.len(),
                "{}",
                host.name
            );
        }
        assert!(module.get_function("vudo_println").is_some());
        assert!(module.get_function("vudo_string_concat").is_some());
        assert!(module.get_function("vudo_panic").is_some());
    }

    /// Negative oracle: the pre-0.8.2 `vudo_*` host ABI is never declared.
    #[test]
    fn test_old_vudo_host_abi_not_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        AbiGenerator::new(&context, &module).declare_all_host_functions();
        for old in [
            "vudo_log",
            "vudo_error",
            "vudo_now",
            "vudo_sleep",
            "vudo_monotonic_now",
            "vudo_send",
            "vudo_recv",
            "vudo_pending",
            "vudo_broadcast",
            "vudo_free_message",
            "vudo_random",
            "vudo_random_bytes",
            "vudo_emit_effect",
            "vudo_subscribe",
        ] {
            assert!(module.get_function(old).is_none(), "{old} still declared");
        }
    }

    #[test]
    fn test_table_matches_vm() {
        assert_eq!(HOST_FUNCTIONS.len(), 15);
        for f in HOST_FUNCTIONS {
            assert!(f.name.starts_with("host_"));
            for &i in f.str_pairs {
                assert_eq!(f.params.get(i), Some(&Ptr), "{} ptr {}", f.name, i);
                assert_eq!(f.params.get(i + 1), Some(&I32), "{} len {}", f.name, i + 1);
            }
            assert_ne!(f.result, Ptr, "{}", f.name);
        }
    }
}
