//! VUDO Native Runtime Library
//!
//! Implements the 22 VUDO host functions for native (non-WASM) Spirits.
//! This library is linked against compiled Spirit binaries to provide
//! I/O, memory, time, messaging, and effect capabilities.
//!
//! # ABI Compatibility
//!
//! All functions use the C calling convention and match the WASM ABI exactly.
//! This ensures Spirit code is portable between WASM and native targets.
//!
//! # Safety
//!
//! All exported FFI functions accept raw pointers from LLVM-generated native code.
//! Callers (the DOL compiler) are responsible for passing valid pointers and lengths.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;

mod effects;
mod io;
mod memory;
mod messaging;
mod time;

// Re-export all host functions
pub use effects::*;
pub use io::*;
pub use memory::*;
pub use messaging::*;
pub use time::*;

/// Initialize the VUDO runtime
/// Must be called before any Spirit code runs
#[no_mangle]
pub extern "C" fn vudo_runtime_init() {
    // Initialize logging, allocator, message queues, etc.
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

#[no_mangle]
pub extern "C" fn vudo_log(level: i32, ptr: *const u8, len: usize) {
    io::log_impl(level, ptr, len);
}

#[no_mangle]
pub extern "C" fn vudo_error(code: i32, ptr: *const u8, len: usize) {
    io::error_impl(code, ptr, len);
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

// === Time Functions ===

#[no_mangle]
pub extern "C" fn vudo_now() -> i64 {
    time::now_impl()
}

#[no_mangle]
pub extern "C" fn vudo_sleep(millis: i64) {
    time::sleep_impl(millis);
}

#[no_mangle]
pub extern "C" fn vudo_monotonic_now() -> i64 {
    time::monotonic_now_impl()
}

// === Messaging Functions ===

#[no_mangle]
pub extern "C" fn vudo_send(recipient: i32, msg_ptr: *const u8, msg_len: usize) -> i32 {
    messaging::send_impl(recipient, msg_ptr, msg_len)
}

#[no_mangle]
pub extern "C" fn vudo_recv(buf: *mut u8, max_len: usize) -> usize {
    messaging::recv_impl(buf, max_len)
}

#[no_mangle]
pub extern "C" fn vudo_pending() -> i32 {
    messaging::pending_impl()
}

#[no_mangle]
pub extern "C" fn vudo_broadcast(channel: i32, msg_ptr: *const u8, msg_len: usize) -> i32 {
    messaging::broadcast_impl(channel, msg_ptr, msg_len)
}

#[no_mangle]
pub extern "C" fn vudo_free_message(msg: *mut c_void) {
    messaging::free_message_impl(msg);
}

// === Random Functions ===

#[no_mangle]
pub extern "C" fn vudo_random() -> u64 {
    // Use system random
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}

#[no_mangle]
pub extern "C" fn vudo_random_bytes(buf: *mut u8, len: usize) {
    if buf.is_null() || len == 0 {
        return;
    }
    // Fill with random bytes
    unsafe {
        for i in 0..len {
            *buf.add(i) = (vudo_random() & 0xFF) as u8;
        }
    }
}

// === Effects Functions ===

#[no_mangle]
pub extern "C" fn vudo_emit_effect(
    effect_type: i32,
    payload_ptr: *const u8,
    payload_len: usize,
) -> i32 {
    effects::emit_effect_impl(effect_type, payload_ptr, payload_len)
}

#[no_mangle]
pub extern "C" fn vudo_subscribe(effect_type: i32) -> i32 {
    effects::subscribe_impl(effect_type)
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
