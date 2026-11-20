//! Canonical VUDO host ABI (Spirit Hosting Plan, decision D2).
//!
//! The VUDO VM linker (`univrs/vudo`, `vudo_vm/src/linker.rs`) is the single
//! source of truth for host functions available to a Spirit: every function
//! lives in WASM import module [`HOST_MODULE`] (`"vudo"`) and is named
//! `host_*`. A bodiless `sex fun` in DOL compiles to an import from this
//! table, with exactly these WASM types, so that the module links in the VM.
//!
//! Keep this table in lock-step with the VM linker. The conformance tests in
//! `tests/wasm_host_abi.rs` pin it independently.

use wasm_encoder::ValType;

/// WASM import module providing all host functions.
pub const HOST_MODULE: &str = "vudo";

/// One host function as registered by the VUDO VM linker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostFunction {
    /// Import name (always `host_*`).
    pub name: &'static str,
    /// WASM parameter types.
    pub params: &'static [ValType],
    /// WASM result type (every VM host function returns exactly one value).
    pub result: ValType,
    /// Indices `i` where params `(i, i + 1)` are an input byte string
    /// `(ptr, len)`. A DOL `string` argument lowers to such a pair.
    pub str_pairs: &'static [usize],
    /// Indices of every param that is a pointer into guest linear memory
    /// (string pairs, output buffers, 32-byte account ids).
    pub ptr_params: &'static [usize],
}

impl HostFunction {
    /// Whether the host reads or writes guest memory through any parameter,
    /// so the importing module must define and export `memory`.
    pub fn takes_pointer(&self) -> bool {
        !self.ptr_params.is_empty()
    }
}

use ValType::{I32, I64};

/// All host functions provided by the VUDO VM, in linker registration order.
pub const HOST_FUNCTIONS: &[HostFunction] = &[
    // Time
    HostFunction {
        name: "host_time_now",
        params: &[],
        result: I64,
        str_pairs: &[],
        ptr_params: &[],
    },
    // Random
    HostFunction {
        name: "host_random_bytes",
        params: &[I32, I32],
        result: I32,
        str_pairs: &[],
        ptr_params: &[0],
    },
    // Logging: (level, ptr, len)
    HostFunction {
        name: "host_log",
        params: &[I32, I32, I32],
        result: I32,
        str_pairs: &[1],
        ptr_params: &[1],
    },
    // Storage: (key_ptr, key_len, val_ptr, val_cap|val_len)
    HostFunction {
        name: "host_storage_read",
        params: &[I32, I32, I32, I32],
        result: I32,
        str_pairs: &[0],
        ptr_params: &[0, 2],
    },
    HostFunction {
        name: "host_storage_write",
        params: &[I32, I32, I32, I32],
        result: I32,
        str_pairs: &[0, 2],
        ptr_params: &[0, 2],
    },
    HostFunction {
        name: "host_storage_delete",
        params: &[I32, I32],
        result: I32,
        str_pairs: &[0],
        ptr_params: &[0],
    },
    // Network
    HostFunction {
        name: "host_network_connect",
        params: &[I32, I32],
        result: I64,
        str_pairs: &[0],
        ptr_params: &[0],
    },
    HostFunction {
        name: "host_network_listen",
        params: &[I32],
        result: I64,
        str_pairs: &[],
        ptr_params: &[],
    },
    HostFunction {
        name: "host_network_broadcast",
        params: &[I32, I32],
        result: I64,
        str_pairs: &[0],
        ptr_params: &[0],
    },
    // Credit
    HostFunction {
        name: "host_credit_balance",
        params: &[I32],
        result: I64,
        str_pairs: &[],
        ptr_params: &[0],
    },
    HostFunction {
        name: "host_credit_transfer",
        params: &[I32, I32, I64],
        result: I32,
        str_pairs: &[],
        ptr_params: &[0, 1],
    },
    HostFunction {
        name: "host_credit_reserve",
        params: &[I32, I64],
        result: I64,
        str_pairs: &[],
        ptr_params: &[0],
    },
    HostFunction {
        name: "host_credit_release",
        params: &[I64],
        result: I32,
        str_pairs: &[],
        ptr_params: &[],
    },
    HostFunction {
        name: "host_credit_consume",
        params: &[I64],
        result: I32,
        str_pairs: &[],
        ptr_params: &[],
    },
    HostFunction {
        name: "host_credit_available",
        params: &[I32],
        result: I64,
        str_pairs: &[],
        ptr_params: &[0],
    },
];

/// Look up a host function by its import name.
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

/// Whether a DOL-declared WASM type can be converted to/from a host type.
///
/// Integer widths convert (wrap / sign-extend at the call site); floats only
/// match floats. The VM ABI has no float parameters or results.
pub(crate) fn is_compatible(dol: ValType, host: ValType) -> bool {
    matches!(
        (dol, host),
        (I32 | I64, I32 | I64) | (ValType::F64, ValType::F64) | (ValType::F32, ValType::F32)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_pairs_are_i32_pointer_and_length() {
        for f in HOST_FUNCTIONS {
            for &i in f.str_pairs {
                assert_eq!(f.params.get(i), Some(&I32), "{} ptr {}", f.name, i);
                assert_eq!(f.params.get(i + 1), Some(&I32), "{} len {}", f.name, i + 1);
                assert!(
                    f.ptr_params.contains(&i),
                    "{} pair ptr {} not a pointer",
                    f.name,
                    i
                );
            }
            for &i in f.ptr_params {
                assert_eq!(f.params.get(i), Some(&I32), "{} ptr {}", f.name, i);
            }
        }
    }
}
