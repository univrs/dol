//! I/O host functions implementation

pub fn print_impl(ptr: *const u8, len: usize) {
    let s = unsafe {
        if ptr.is_null() || len == 0 {
            return;
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
    };
    print!("{}", s);
}

pub fn println_impl(ptr: *const u8, len: usize) {
    let s = unsafe {
        if ptr.is_null() || len == 0 {
            println!();
            return;
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
    };
    println!("{}", s);
}

pub fn log_impl(level: i32, ptr: *const u8, len: usize) {
    let s = unsafe {
        if ptr.is_null() || len == 0 {
            return;
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
    };
    match level {
        0 => tracing::trace!("{}", s),
        1 => tracing::debug!("{}", s),
        2 => tracing::info!("{}", s),
        3 => tracing::warn!("{}", s),
        _ => tracing::error!("{}", s),
    }
}

pub fn error_impl(code: i32, ptr: *const u8, len: usize) {
    let s = unsafe {
        if ptr.is_null() || len == 0 {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
        }
    };
    tracing::error!("Error {}: {}", code, s);
}
