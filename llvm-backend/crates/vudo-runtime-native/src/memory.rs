//! Memory host functions implementation

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ffi::c_void;

pub fn alloc_impl(size: usize) -> *mut c_void {
    if size == 0 {
        return std::ptr::null_mut();
    }
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, 8);
        alloc(layout) as *mut c_void
    }
}

pub fn free_impl(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    // Note: We don't know the size, so this is a simplified implementation
    // Real implementation would track allocations
    unsafe {
        let layout = Layout::from_size_align_unchecked(1, 8);
        dealloc(ptr as *mut u8, layout);
    }
}

pub fn realloc_impl(ptr: *mut c_void, new_size: usize) -> *mut c_void {
    if ptr.is_null() {
        return alloc_impl(new_size);
    }
    if new_size == 0 {
        free_impl(ptr);
        return std::ptr::null_mut();
    }
    unsafe {
        let layout = Layout::from_size_align_unchecked(1, 8);
        realloc(ptr as *mut u8, layout, new_size) as *mut c_void
    }
}
