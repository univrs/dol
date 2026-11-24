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
