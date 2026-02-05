//! Memory layout and string encoding utilities for WASM.
//!
//! This module provides utilities for managing WASM linear memory,
//! including encoding string literals into the data section and
//! tracking memory layout for static allocations.
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::wasm::imports::memory::{StringEncoder, MemoryLayout};
//!
//! let mut encoder = StringEncoder::new();
//! let offset1 = encoder.encode_string("Hello");
//! let offset2 = encoder.encode_string("World");
//!
//! // Get the complete data section
//! let data = encoder.finalize();
//!
//! // Create a memory layout
//! let mut layout = MemoryLayout::new(0x10000); // 64KB initial
//! let heap_offset = layout.allocate_static(1024);
//! ```

#[cfg(feature = "wasm-compile")]
use std::collections::HashMap;

/// Encodes string literals into WASM data section.
///
/// The `StringEncoder` collects all string literals used in the module
/// and assigns them offsets in the WASM linear memory. Strings are
/// deduplicated to avoid storing the same string multiple times.
///
/// # Memory Layout
///
/// Strings are stored contiguously starting at offset 0 in the data section.
/// Each string is stored as raw UTF-8 bytes without null termination or
/// length prefix. The caller is responsible for tracking string lengths.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::imports::memory::StringEncoder;
///
/// let mut encoder = StringEncoder::new();
///
/// // Encode some strings
/// let hello_offset = encoder.encode_string("Hello, World!");
/// let bye_offset = encoder.encode_string("Goodbye!");
///
/// // Duplicate strings return the same offset
/// let hello_again = encoder.encode_string("Hello, World!");
/// assert_eq!(hello_offset, hello_again);
///
/// // Get the final data section bytes
/// let data = encoder.finalize();
/// ```
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone)]
pub struct StringEncoder {
    /// Maps string content to offset in data section
    string_offsets: HashMap<String, u32>,
    /// Accumulated data section bytes
    data: Vec<u8>,
    /// Current write offset
    next_offset: u32,
}

#[cfg(feature = "wasm-compile")]
impl StringEncoder {
    /// Create a new StringEncoder with an empty data section.
    pub fn new() -> Self {
        Self {
            string_offsets: HashMap::new(),
            data: Vec::new(),
            next_offset: 0,
        }
    }

    /// Encode a string literal into the data section.
    ///
    /// If the string has already been encoded, returns the existing offset.
    /// Otherwise, appends the string to the data section and returns the new offset.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to encode
    ///
    /// # Returns
    ///
    /// The offset in WASM linear memory where the string is stored.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut encoder = StringEncoder::new();
    /// let offset = encoder.encode_string("Hello");
    /// assert_eq!(offset, 0); // First string starts at offset 0
    /// ```
    pub fn encode_string(&mut self, s: &str) -> u32 {
        // Check if we've already encoded this string
        if let Some(&offset) = self.string_offsets.get(s) {
            return offset;
        }

        // Encode the string at the current offset
        let offset = self.next_offset;
        let bytes = s.as_bytes();
        self.data.extend_from_slice(bytes);
        self.next_offset += bytes.len() as u32;

        // Remember this string's offset for deduplication
        self.string_offsets.insert(s.to_string(), offset);

        offset
    }

    /// Get the offset of a previously encoded string.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to look up
    ///
    /// # Returns
    ///
    /// The offset if the string has been encoded, or `None` if not found.
    pub fn get_offset(&self, s: &str) -> Option<u32> {
        self.string_offsets.get(s).copied()
    }

    /// Get the total size of the data section in bytes.
    ///
    /// # Returns
    ///
    /// The number of bytes that have been written to the data section.
    pub fn size(&self) -> u32 {
        self.next_offset
    }

    /// Get the number of unique strings that have been encoded.
    pub fn string_count(&self) -> usize {
        self.string_offsets.len()
    }

    /// Finalize the encoder and return the complete data section bytes.
    ///
    /// After calling this method, the encoder can still be used to encode
    /// more strings, but they will be appended to the existing data.
    ///
    /// # Returns
    ///
    /// A vector containing all encoded string bytes.
    pub fn finalize(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Clear all encoded strings and reset to empty state.
    pub fn clear(&mut self) {
        self.string_offsets.clear();
        self.data.clear();
        self.next_offset = 0;
    }
}

#[cfg(feature = "wasm-compile")]
impl Default for StringEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages WASM linear memory layout.
///
/// The `MemoryLayout` tracks the allocation of memory regions in WASM
/// linear memory, including the data section, heap, and stack.
///
/// # Memory Regions
///
/// WASM linear memory is organized as follows:
///
/// ```text
/// ┌─────────────────────────────────────┐
/// │ Data Section (strings, constants)   │  0x0000 - data_end
/// ├─────────────────────────────────────┤
/// │ Static Allocations                  │  data_end - heap_start
/// ├─────────────────────────────────────┤
/// │ Heap (dynamic allocations)          │  heap_start - heap_end
/// ├─────────────────────────────────────┤
/// │ Stack (grows down)                  │  heap_end - memory_size
/// └─────────────────────────────────────┘
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::imports::memory::MemoryLayout;
///
/// // Create layout with 64KB initial memory
/// let mut layout = MemoryLayout::new(0x10000);
///
/// // Reserve space for data section
/// layout.set_data_size(1024);
///
/// // Allocate static buffer
/// let buffer_offset = layout.allocate_static(512);
///
/// // Get heap start offset
/// let heap_start = layout.heap_start();
/// ```
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone)]
pub struct MemoryLayout {
    /// Size of the data section (strings, constants)
    data_size: u32,
    /// Current offset for static allocations
    static_offset: u32,
    /// Start of the heap region
    heap_start: u32,
    /// Total size of linear memory
    memory_size: u32,
    /// Minimum number of WASM pages (64KB each)
    min_pages: u32,
    /// Maximum number of WASM pages (optional)
    max_pages: Option<u32>,
}

#[cfg(feature = "wasm-compile")]
impl MemoryLayout {
    /// WASM page size in bytes (64KB)
    pub const PAGE_SIZE: u32 = 65536;

    /// Create a new memory layout with the specified initial memory size.
    ///
    /// # Arguments
    ///
    /// * `initial_size` - Initial size of linear memory in bytes (must be multiple of 64KB)
    ///
    /// # Panics
    ///
    /// Panics if `initial_size` is not a multiple of 64KB (WASM page size).
    pub fn new(initial_size: u32) -> Self {
        assert!(
            initial_size % Self::PAGE_SIZE == 0,
            "Initial memory size must be a multiple of 64KB (WASM page size)"
        );

        let min_pages = initial_size / Self::PAGE_SIZE;

        Self {
            data_size: 0,
            static_offset: 0,
            heap_start: 0,
            memory_size: initial_size,
            min_pages,
            max_pages: None,
        }
    }

    /// Create a new memory layout with specified min and max pages.
    ///
    /// # Arguments
    ///
    /// * `min_pages` - Minimum number of 64KB pages
    /// * `max_pages` - Maximum number of 64KB pages (None for unbounded)
    pub fn with_pages(min_pages: u32, max_pages: Option<u32>) -> Self {
        Self {
            data_size: 0,
            static_offset: 0,
            heap_start: 0,
            memory_size: min_pages * Self::PAGE_SIZE,
            min_pages,
            max_pages,
        }
    }

    /// Set the size of the data section.
    ///
    /// This should be called after encoding all strings and constants.
    /// Static allocations will start after the data section.
    ///
    /// # Arguments
    ///
    /// * `size` - Size of the data section in bytes
    pub fn set_data_size(&mut self, size: u32) {
        self.data_size = size;
        self.static_offset = size;
        self.heap_start = size;
    }

    /// Allocate a static region of memory.
    ///
    /// Static allocations are placed between the data section and the heap.
    /// They are used for global variables and other compile-time allocations.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate
    ///
    /// # Returns
    ///
    /// The offset in linear memory where the allocation starts.
    pub fn allocate_static(&mut self, size: u32) -> u32 {
        let offset = self.static_offset;
        self.static_offset += size;
        self.heap_start = self.static_offset;
        offset
    }

    /// Get the offset where the heap starts.
    ///
    /// The heap is the region of memory used for dynamic allocations
    /// at runtime (via `vudo_alloc`).
    pub fn heap_start(&self) -> u32 {
        self.heap_start
    }

    /// Get the size of the data section.
    pub fn data_size(&self) -> u32 {
        self.data_size
    }

    /// Get the total size of static allocations (data + static).
    pub fn static_size(&self) -> u32 {
        self.static_offset
    }

    /// Get the total memory size in bytes.
    pub fn memory_size(&self) -> u32 {
        self.memory_size
    }

    /// Get the minimum number of WASM pages.
    pub fn min_pages(&self) -> u32 {
        self.min_pages
    }

    /// Get the maximum number of WASM pages.
    pub fn max_pages(&self) -> Option<u32> {
        self.max_pages
    }

    /// Set the maximum number of WASM pages.
    ///
    /// This limits how much the memory can grow at runtime.
    pub fn set_max_pages(&mut self, max_pages: u32) {
        self.max_pages = Some(max_pages);
    }

    /// Calculate the number of pages needed for the current layout.
    ///
    /// This rounds up the static size to the nearest WASM page boundary.
    pub fn required_pages(&self) -> u32 {
        self.static_offset.div_ceil(Self::PAGE_SIZE)
    }

    /// Align an offset to the specified boundary.
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to align
    /// * `alignment` - The alignment boundary (must be power of 2)
    ///
    /// # Returns
    ///
    /// The smallest offset >= `offset` that is aligned to `alignment`.
    pub fn align(offset: u32, alignment: u32) -> u32 {
        assert!(alignment.is_power_of_two(), "Alignment must be power of 2");
        (offset + alignment - 1) & !(alignment - 1)
    }

    /// Allocate a static region with specified alignment.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate
    /// * `alignment` - Required alignment (must be power of 2)
    ///
    /// # Returns
    ///
    /// The aligned offset in linear memory where the allocation starts.
    pub fn allocate_static_aligned(&mut self, size: u32, alignment: u32) -> u32 {
        let aligned_offset = Self::align(self.static_offset, alignment);
        self.static_offset = aligned_offset + size;
        self.heap_start = self.static_offset;
        aligned_offset
    }
}

#[cfg(feature = "wasm-compile")]
impl Default for MemoryLayout {
    /// Create a default memory layout with 1MB (16 pages) initial memory.
    fn default() -> Self {
        Self::new(16 * Self::PAGE_SIZE)
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-compile")]
mod tests {
    use super::*;

    #[test]
    fn test_string_encoder_basic() {
        let mut encoder = StringEncoder::new();

        let offset1 = encoder.encode_string("Hello");
        assert_eq!(offset1, 0);

        let offset2 = encoder.encode_string("World");
        assert_eq!(offset2, 5);

        assert_eq!(encoder.size(), 10);
        assert_eq!(encoder.string_count(), 2);
    }

    #[test]
    fn test_string_encoder_deduplication() {
        let mut encoder = StringEncoder::new();

        let offset1 = encoder.encode_string("Hello");
        let offset2 = encoder.encode_string("Hello");

        assert_eq!(offset1, offset2);
        assert_eq!(encoder.string_count(), 1);
    }

    #[test]
    fn test_string_encoder_finalize() {
        let mut encoder = StringEncoder::new();

        encoder.encode_string("Hello");
        encoder.encode_string("World");

        let data = encoder.finalize();
        assert_eq!(data, b"HelloWorld");
    }

    #[test]
    fn test_string_encoder_get_offset() {
        let mut encoder = StringEncoder::new();

        encoder.encode_string("Hello");
        encoder.encode_string("World");

        assert_eq!(encoder.get_offset("Hello"), Some(0));
        assert_eq!(encoder.get_offset("World"), Some(5));
        assert_eq!(encoder.get_offset("NotFound"), None);
    }

    #[test]
    fn test_string_encoder_clear() {
        let mut encoder = StringEncoder::new();

        encoder.encode_string("Hello");
        encoder.clear();

        assert_eq!(encoder.size(), 0);
        assert_eq!(encoder.string_count(), 0);
        assert_eq!(encoder.get_offset("Hello"), None);
    }

    #[test]
    fn test_memory_layout_basic() {
        let layout = MemoryLayout::new(65536); // 1 page

        assert_eq!(layout.min_pages(), 1);
        assert_eq!(layout.memory_size(), 65536);
        assert_eq!(layout.heap_start(), 0);
    }

    #[test]
    fn test_memory_layout_data_section() {
        let mut layout = MemoryLayout::new(65536);

        layout.set_data_size(1024);
        assert_eq!(layout.data_size(), 1024);
        assert_eq!(layout.heap_start(), 1024);
    }

    #[test]
    fn test_memory_layout_static_allocation() {
        let mut layout = MemoryLayout::new(65536);

        layout.set_data_size(100);

        let offset1 = layout.allocate_static(50);
        assert_eq!(offset1, 100);

        let offset2 = layout.allocate_static(30);
        assert_eq!(offset2, 150);

        assert_eq!(layout.heap_start(), 180);
        assert_eq!(layout.static_size(), 180);
    }

    #[test]
    fn test_memory_layout_alignment() {
        assert_eq!(MemoryLayout::align(0, 4), 0);
        assert_eq!(MemoryLayout::align(1, 4), 4);
        assert_eq!(MemoryLayout::align(4, 4), 4);
        assert_eq!(MemoryLayout::align(5, 8), 8);
        assert_eq!(MemoryLayout::align(17, 16), 32);
    }

    #[test]
    fn test_memory_layout_aligned_allocation() {
        let mut layout = MemoryLayout::new(65536);

        layout.set_data_size(100);

        let offset1 = layout.allocate_static_aligned(10, 16);
        assert_eq!(offset1, 112); // 100 rounded up to 16-byte boundary

        let offset2 = layout.allocate_static_aligned(10, 8);
        assert_eq!(offset2, 128); // 122 rounded up to 8-byte boundary
    }

    #[test]
    fn test_memory_layout_required_pages() {
        let mut layout = MemoryLayout::new(65536);

        layout.set_data_size(1000);
        assert_eq!(layout.required_pages(), 1);

        layout.allocate_static(70000);
        assert_eq!(layout.required_pages(), 2);
    }

    #[test]
    fn test_memory_layout_with_pages() {
        let layout = MemoryLayout::with_pages(2, Some(16));

        assert_eq!(layout.min_pages(), 2);
        assert_eq!(layout.max_pages(), Some(16));
        assert_eq!(layout.memory_size(), 131072); // 2 * 64KB
    }

    #[test]
    #[should_panic(expected = "Initial memory size must be a multiple of 64KB")]
    fn test_memory_layout_invalid_size() {
        MemoryLayout::new(1000); // Not a multiple of 64KB
    }

    #[test]
    #[should_panic(expected = "Alignment must be power of 2")]
    fn test_memory_layout_invalid_alignment() {
        MemoryLayout::align(0, 3); // 3 is not a power of 2
    }
}
