//! Canonical VUDO host ABI (`host_*`) for native Spirits.
//!
//! These are the VUDO VM's 15 host functions (`univrs/vudo`
//! `vudo_vm/src/linker.rs`; mirrored by `src/wasm/host_abi.rs` in the DOL
//! repository) with C linkage. Signatures and return values follow the VM;
//! a WASM `i32` guest address becomes a native pointer. `-1` is the VM's
//! error value.
//!
//! Natively there is no capability system, P2P network, or credit ledger:
//! - time, random, and logging are real;
//! - storage is an in-process key/value map (lost at exit);
//! - network and credit functions always return `-1` and never report
//!   success.

use std::collections::HashMap;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Error value returned by host functions (VM `HOST_ERROR`).
pub const HOST_ERROR: i32 = -1;
/// Success value for host functions that return no data (VM `HOST_SUCCESS`).
pub const HOST_SUCCESS: i32 = 0;

/// Largest log message accepted, as in the VM.
const MAX_LOG_MESSAGE_LENGTH: usize = 64 * 1024;
/// Largest random request accepted, as in the VM.
const MAX_RANDOM_BYTES: i32 = 1024 * 1024;

/// Borrow `len` bytes at `ptr`, or `None` if the pair is invalid.
///
/// # Safety
/// A non-null `ptr` must be valid for `len` bytes.
unsafe fn bytes<'a>(ptr: *const u8, len: i32) -> Option<&'a [u8]> {
    if len < 0 || (ptr.is_null() && len > 0) {
        return None;
    }
    if len == 0 {
        return Some(&[]);
    }
    Some(std::slice::from_raw_parts(ptr, len as usize))
}

// === Time ===

/// `host_time_now() -> i64`: Unix time in nanoseconds, or -1 on error.
#[no_mangle]
pub extern "C" fn host_time_now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_nanos()).unwrap_or(-1),
        Err(_) => -1,
    }
}

// === Random ===

/// `host_random_bytes(buf, len) -> i32`: fill `buf` with `len` bytes from the
/// OS CSPRNG. 0 on success, -1 if `len` is not in `1..=1 MiB` or on failure.
#[no_mangle]
pub extern "C" fn host_random_bytes(buf: *mut u8, len: i32) -> i32 {
    if buf.is_null() || len <= 0 || len > MAX_RANDOM_BYTES {
        return HOST_ERROR;
    }
    // SAFETY: caller passes a buffer valid for `len` bytes.
    let out = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
    match getrandom::getrandom(out) {
        Ok(()) => HOST_SUCCESS,
        Err(_) => HOST_ERROR,
    }
}

// === Logging ===

/// `host_log(level, ptr, len) -> i32`: write a UTF-8 message with the VM's
/// prefix. Levels 0 trace, 1 debug, 4 error go to stderr; 2 info, 3 warn to
/// stdout. 0 on success, -1 for an unknown level, invalid UTF-8, or a message
/// over 64 KiB.
#[no_mangle]
pub extern "C" fn host_log(level: i32, ptr: *const u8, len: i32) -> i32 {
    // SAFETY: caller passes a valid (ptr, len) string.
    let Some(raw) = (unsafe { bytes(ptr, len) }) else {
        return HOST_ERROR;
    };
    if raw.len() > MAX_LOG_MESSAGE_LENGTH {
        return HOST_ERROR;
    }
    let Ok(msg) = std::str::from_utf8(raw) else {
        return HOST_ERROR;
    };
    let written = match level {
        0 => writeln!(std::io::stderr(), "[VUDO:TRACE] {msg}"),
        1 => writeln!(std::io::stderr(), "[VUDO:DEBUG] {msg}"),
        2 => writeln!(std::io::stdout(), "[VUDO:INFO] {msg}"),
        3 => writeln!(std::io::stdout(), "[VUDO:WARN] {msg}"),
        4 => writeln!(std::io::stderr(), "[VUDO:ERROR] {msg}"),
        _ => return HOST_ERROR,
    };
    match written {
        Ok(()) => HOST_SUCCESS,
        Err(_) => HOST_ERROR,
    }
}

// === Storage ===

fn storage() -> &'static Mutex<HashMap<Vec<u8>, Vec<u8>>> {
    static STORE: OnceLock<Mutex<HashMap<Vec<u8>, Vec<u8>>>> = OnceLock::new();
    STORE.get_or_init(Mutex::default)
}

/// `host_storage_read(key, key_len, buf, cap) -> i32`: copy the value for
/// `key` into `buf`. Returns the bytes written, 0 if the key is missing, or -1
/// on error (including a value larger than `cap`).
#[no_mangle]
pub extern "C" fn host_storage_read(
    key_ptr: *const u8,
    key_len: i32,
    val_ptr: *mut u8,
    val_cap: i32,
) -> i32 {
    // SAFETY: caller passes a valid (ptr, len) key.
    let Some(key) = (unsafe { bytes(key_ptr, key_len) }) else {
        return HOST_ERROR;
    };
    if val_cap < 0 {
        return HOST_ERROR;
    }
    let Ok(store) = storage().lock() else {
        return HOST_ERROR;
    };
    let Some(value) = store.get(key) else {
        return 0;
    };
    if value.len() > val_cap as usize || (val_ptr.is_null() && !value.is_empty()) {
        return HOST_ERROR;
    }
    if !value.is_empty() {
        // SAFETY: caller passes a buffer valid for `val_cap` bytes.
        unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), val_ptr, value.len()) };
    }
    value.len() as i32
}

/// `host_storage_write(key, key_len, val, val_len) -> i32`: store `val` under
/// `key`. 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn host_storage_write(
    key_ptr: *const u8,
    key_len: i32,
    val_ptr: *const u8,
    val_len: i32,
) -> i32 {
    // SAFETY: caller passes valid (ptr, len) pairs.
    let (Some(key), Some(value)) = (unsafe { bytes(key_ptr, key_len) }, unsafe {
        bytes(val_ptr, val_len)
    }) else {
        return HOST_ERROR;
    };
    let Ok(mut store) = storage().lock() else {
        return HOST_ERROR;
    };
    store.insert(key.to_vec(), value.to_vec());
    HOST_SUCCESS
}

/// `host_storage_delete(key, key_len) -> i32`: 1 if deleted, 0 if not found,
/// -1 on error.
#[no_mangle]
pub extern "C" fn host_storage_delete(key_ptr: *const u8, key_len: i32) -> i32 {
    // SAFETY: caller passes a valid (ptr, len) key.
    let Some(key) = (unsafe { bytes(key_ptr, key_len) }) else {
        return HOST_ERROR;
    };
    let Ok(mut store) = storage().lock() else {
        return HOST_ERROR;
    };
    i32::from(store.remove(key).is_some())
}

// === Network (unavailable natively: always -1) ===

/// `host_network_connect(addr, addr_len) -> i64`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_network_connect(_addr_ptr: *const u8, _addr_len: i32) -> i64 {
    -1
}

/// `host_network_listen(port) -> i64`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_network_listen(_port: i32) -> i64 {
    -1
}

/// `host_network_broadcast(msg, msg_len) -> i64`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_network_broadcast(_msg_ptr: *const u8, _msg_len: i32) -> i64 {
    -1
}

// === Credit (no ledger natively: always -1) ===

/// `host_credit_balance(account) -> i64`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_credit_balance(_account_ptr: *const u8) -> i64 {
    -1
}

/// `host_credit_transfer(from, to, amount) -> i32`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_credit_transfer(
    _from_ptr: *const u8,
    _to_ptr: *const u8,
    _amount: i64,
) -> i32 {
    HOST_ERROR
}

/// `host_credit_reserve(account, amount) -> i64`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_credit_reserve(_account_ptr: *const u8, _amount: i64) -> i64 {
    -1
}

/// `host_credit_release(reservation) -> i32`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_credit_release(_reservation_id: i64) -> i32 {
    HOST_ERROR
}

/// `host_credit_consume(reservation) -> i32`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_credit_consume(_reservation_id: i64) -> i32 {
    HOST_ERROR
}

/// `host_credit_available(account) -> i64`: always -1 natively.
#[no_mangle]
pub extern "C" fn host_credit_available(_account_ptr: *const u8) -> i64 {
    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> (*const u8, i32) {
        (v.as_ptr(), v.len() as i32)
    }

    #[test]
    fn time_is_unix_nanoseconds() {
        let t = host_time_now();
        // After 2023-11-14 in ns, before 2262 (i64 overflow).
        assert!(t > 1_700_000_000_000_000_000, "{t}");
    }

    #[test]
    fn random_fills_buffer_and_rejects_bad_lengths() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        assert_eq!(host_random_bytes(a.as_mut_ptr(), 32), 0);
        assert_eq!(host_random_bytes(b.as_mut_ptr(), 32), 0);
        assert_ne!(a, b);
        assert_eq!(host_random_bytes(a.as_mut_ptr(), 0), -1);
        assert_eq!(host_random_bytes(a.as_mut_ptr(), -4), -1);
        assert_eq!(host_random_bytes(std::ptr::null_mut(), 4), -1);
    }

    #[test]
    fn log_levels_and_errors() {
        let (p, l) = s("native log test");
        for level in 0..=4 {
            assert_eq!(host_log(level, p, l), 0);
        }
        assert_eq!(host_log(5, p, l), -1);
        assert_eq!(host_log(-1, p, l), -1);
        let bad = [0xffu8, 0xfe];
        assert_eq!(host_log(2, bad.as_ptr(), 2), -1);
        assert_eq!(host_log(2, std::ptr::null(), 3), -1);
    }

    #[test]
    fn storage_follows_vm_semantics() {
        let (k, kl) = s("unit-test-key");
        let (v, vl) = s("hello");
        let mut buf = [0u8; 16];
        assert_eq!(host_storage_read(k, kl, buf.as_mut_ptr(), 16), 0);
        assert_eq!(host_storage_write(k, kl, v, vl), 0);
        assert_eq!(host_storage_read(k, kl, buf.as_mut_ptr(), 16), 5);
        assert_eq!(&buf[..5], b"hello");
        assert_eq!(host_storage_read(k, kl, buf.as_mut_ptr(), 4), -1);
        assert_eq!(host_storage_delete(k, kl), 1);
        assert_eq!(host_storage_delete(k, kl), 0);
        assert_eq!(host_storage_read(k, kl, buf.as_mut_ptr(), 16), 0);
        assert_eq!(host_storage_write(k, -1, v, vl), -1);
    }

    /// Negative oracle: no network or ledger exists natively, so no call may
    /// ever report success, whatever the arguments.
    #[test]
    fn network_and_credit_never_succeed() {
        let (a, al) = s("127.0.0.1:9000");
        let account = [7u8; 32];
        let other = [9u8; 32];
        for port in [0, 80, 9000, 65535] {
            assert_eq!(host_network_listen(port), -1);
        }
        assert_eq!(host_network_connect(a, al), -1);
        assert_eq!(host_network_broadcast(a, al), -1);
        assert_eq!(host_credit_balance(account.as_ptr()), -1);
        assert_eq!(host_credit_available(account.as_ptr()), -1);
        for amount in [0, 1, 1_000_000] {
            assert_eq!(
                host_credit_transfer(account.as_ptr(), other.as_ptr(), amount),
                -1
            );
            assert_eq!(host_credit_reserve(account.as_ptr(), amount), -1);
        }
        for id in [0, 1, 42] {
            assert_eq!(host_credit_release(id), -1);
            assert_eq!(host_credit_consume(id), -1);
        }
    }
}
