//! VUDO Native Runtime Library
//!
//! Linked into every natively compiled DOL program. It provides:
//!
//! - **The host ABI** ([`host`]): the VUDO VM's 15 `host_*` functions with C
//!   linkage (time, random, log, storage, network, credit). A DOL Spirit that
//!   declares them as bodiless `sex fun`s runs in both the VM and natively.
//! - **Runtime intrinsics** (`vudo_*`): native-only support for DOL built-ins
//!   emitted by the compiler (`print`/`println`, `alloc`/`free`/`realloc`,
//!   string concatenation and `to_string`, `assert`/`panic`/`breakpoint`).
//!   They are not part of the host ABI and do not exist in the VM.
//!
//! # Safety
//!
//! All exported FFI functions accept raw pointers from LLVM-generated native code.
//! Callers (the DOL compiler) are responsible for passing valid pointers and lengths.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;

pub mod host;
mod io;
mod memory;

pub use host::*;
pub use io::*;
pub use memory::*;

/// Initialize the VUDO runtime
/// Must be called before any Spirit code runs
#[no_mangle]
pub extern "C" fn vudo_runtime_init() {
    // Nothing to set up yet: host storage is initialized lazily.
    tracing::info!("VUDO native runtime initialized");
}

/// Shutdown the VUDO runtime
/// Should be called after Spirit execution completes
#[no_mangle]
pub extern "C" fn vudo_runtime_shutdown() {
    tracing::info!("VUDO native runtime shutdown");
}

// === I/O Functions ===

#[no_mangle]
pub extern "C" fn vudo_print(ptr: *const u8, len: usize) {
    io::print_impl(ptr, len);
}

#[no_mangle]
pub extern "C" fn vudo_println(ptr: *const u8, len: usize) {
    io::println_impl(ptr, len);
}

// === Memory Functions ===

#[no_mangle]
pub extern "C" fn vudo_alloc(size: usize) -> *mut c_void {
    memory::alloc_impl(size)
}

#[no_mangle]
pub extern "C" fn vudo_free(ptr: *mut c_void) {
    memory::free_impl(ptr);
}

#[no_mangle]
pub extern "C" fn vudo_realloc(ptr: *mut c_void, new_size: usize) -> *mut c_void {
    memory::realloc_impl(ptr, new_size)
}

// === String Functions ===

/// Concatenate two strings, returning pointer and length via out-params.
/// Caller must free the returned pointer with `vudo_free`.
#[no_mangle]
pub extern "C" fn vudo_string_concat(
    ptr1: *const u8,
    len1: usize,
    ptr2: *const u8,
    len2: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) {
    let total = len1 + len2;
    let buf = vudo_alloc(total) as *mut u8;
    if !buf.is_null() {
        unsafe {
            if !ptr1.is_null() && len1 > 0 {
                std::ptr::copy_nonoverlapping(ptr1, buf, len1);
            }
            if !ptr2.is_null() && len2 > 0 {
                std::ptr::copy_nonoverlapping(ptr2, buf.add(len1), len2);
            }
            *out_ptr = buf;
            *out_len = total;
        }
    }
}

/// Convert an i64 to its string representation.
/// Caller must free the returned pointer with `vudo_free`.
#[no_mangle]
pub extern "C" fn vudo_i64_to_string(value: i64, out_ptr: *mut *mut u8, out_len: *mut usize) {
    let s = value.to_string();
    let len = s.len();
    let buf = vudo_alloc(len) as *mut u8;
    if !buf.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), buf, len);
            *out_ptr = buf;
            *out_len = len;
        }
    }
}

// === Debug Functions ===

#[no_mangle]
pub extern "C" fn vudo_breakpoint() {
    // Trigger debugger breakpoint
    #[cfg(debug_assertions)]
    unsafe {
        std::arch::asm!("int3", options(nomem, nostack));
    }
}

#[no_mangle]
pub extern "C" fn vudo_assert(condition: i32, msg_ptr: *const u8, msg_len: usize) {
    if condition == 0 {
        let msg = unsafe {
            if msg_ptr.is_null() || msg_len == 0 {
                "assertion failed"
            } else {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg_ptr, msg_len))
            }
        };
        panic!("VUDO assertion failed: {}", msg);
    }
}

#[no_mangle]
pub extern "C" fn vudo_panic(msg_ptr: *const u8, msg_len: usize) {
    let msg = unsafe {
        if msg_ptr.is_null() || msg_len == 0 {
            "panic"
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg_ptr, msg_len))
        }
    };
    panic!("VUDO panic: {}", msg);
}
