//! Bump allocator for WASM linear memory.
//!
//! This module provides a simple bump allocator for allocating gene instances
//! in WebAssembly linear memory. The allocator does not support freeing memory.
//!
//! ## Memory Model
//!
//! The allocator uses two WASM globals to track the heap state:
//!
//! - `HEAP_BASE` (global 0): Mutable i32 pointing to the next free address
//! - `HEAP_END` (global 1): Immutable i32 marking the end of available memory
//!
//! ## Allocation Strategy
//!
//! This is a simple bump allocator:
//! 1. Align the current heap pointer to the requested alignment
//! 2. Bump the pointer by the requested size
//! 3. Return the aligned pointer (or 0 if out of memory)
//!
//! Memory is never freed - when the heap is exhausted, allocations fail.
//! This is suitable for short-lived computations or programs with bounded memory use.
//!
//! ## Example
//!
//! ```rust,ignore
//! use metadol::wasm::alloc::BumpAllocator;
//! use wasm_encoder::Module;
//!
//! let mut module = Module::new();
//! let allocator = BumpAllocator::new(0, 1);
//!
//! // Emit memory section (1 page = 64KB)
//! BumpAllocator::emit_memory_section(&mut module, 1);
//!
//! // Emit globals for heap tracking (heap starts at address 1024)
//! BumpAllocator::emit_globals(&mut module, 1024);
//!
//! // Get alloc function instructions for code section
//! let alloc_instrs = BumpAllocator::emit_alloc_function();
//! ```

#[cfg(feature = "wasm-compile")]
use wasm_encoder::{
    BlockType, ConstExpr, Function, GlobalSection, GlobalType, Instruction, MemorySection,
    MemoryType, Module, ValType,
};

/// Default number of memory pages (1 page = 64KB).
#[cfg(feature = "wasm-compile")]
pub const DEFAULT_MEMORY_PAGES: u32 = 1;

/// Maximum number of memory pages (256 pages = 16MB).
#[cfg(feature = "wasm-compile")]
pub const MAX_MEMORY_PAGES: u32 = 256;

/// Size of one WASM memory page in bytes.
#[cfg(feature = "wasm-compile")]
pub const PAGE_SIZE: u32 = 65536;

/// Default initial heap address (after static data area).
#[cfg(feature = "wasm-compile")]
pub const DEFAULT_HEAP_START: u32 = 1024;

/// Bump allocator state stored in WASM globals.
///
/// Memory layout:
/// - Global 0: HEAP_BASE (next free address, mutable)
/// - Global 1: HEAP_END (end of available memory, immutable)
///
/// The allocator is stateless at compile time - all state is stored
/// in WASM globals at runtime.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::alloc::BumpAllocator;
///
/// // Create allocator tracking globals at indices 0 and 1
/// let allocator = BumpAllocator::new(0, 1);
///
/// // Access global indices for code generation
/// assert_eq!(allocator.heap_base_global(), 0);
/// assert_eq!(allocator.heap_end_global(), 1);
/// ```
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone, Copy)]
pub struct BumpAllocator {
    /// Index of HEAP_BASE global (next free address)
    heap_base_global: u32,
    /// Index of HEAP_END global (end of available memory)
    heap_end_global: u32,
}

#[cfg(feature = "wasm-compile")]
impl BumpAllocator {
    /// Create a new bump allocator configuration.
    ///
    /// # Arguments
    ///
    /// * `heap_base_global` - Index of the HEAP_BASE global
    /// * `heap_end_global` - Index of the HEAP_END global
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::alloc::BumpAllocator;
    ///
    /// let allocator = BumpAllocator::new(0, 1);
    /// ```
    pub fn new(heap_base_global: u32, heap_end_global: u32) -> Self {
        Self {
            heap_base_global,
            heap_end_global,
        }
    }

    /// Get the index of the HEAP_BASE global.
    pub fn heap_base_global(&self) -> u32 {
        self.heap_base_global
    }

    /// Get the index of the HEAP_END global.
    pub fn heap_end_global(&self) -> u32 {
        self.heap_end_global
    }

    /// Emit WASM globals for allocator state.
    ///
    /// Adds two globals to the module:
    /// - Global 0: `HEAP_BASE` (mutable i32) - initialized to `initial_heap`
    /// - Global 1: `HEAP_END` (immutable i32) - set to end of first memory page (64KB)
    ///
    /// # Arguments
    ///
    /// * `module` - The WASM module to add globals to
    /// * `initial_heap` - Starting address for the heap (after static data)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::alloc::BumpAllocator;
    /// use wasm_encoder::Module;
    ///
    /// let mut module = Module::new();
    /// BumpAllocator::emit_globals(&mut module, 1024);
    /// ```
    pub fn emit_globals(module: &mut Module, initial_heap: u32) {
        let mut globals = GlobalSection::new();

        // HEAP_BASE: mutable i32, starts after static data
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
            },
            &ConstExpr::i32_const(initial_heap as i32),
        );

        // HEAP_END: immutable i32, end of first memory page (64KB)
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: false,
            },
            &ConstExpr::i32_const(PAGE_SIZE as i32),
        );

        module.section(&globals);
    }

    /// Emit WASM globals with custom heap end.
    ///
    /// Like `emit_globals`, but allows specifying both the initial heap
    /// address and the heap end address.
    ///
    /// # Arguments
    ///
    /// * `module` - The WASM module to add globals to
    /// * `initial_heap` - Starting address for the heap
    /// * `heap_end` - End address for the heap
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::alloc::BumpAllocator;
    /// use wasm_encoder::Module;
    ///
    /// let mut module = Module::new();
    /// // 4 pages = 256KB of heap space
    /// BumpAllocator::emit_globals_with_end(&mut module, 1024, 4 * 65536);
    /// ```
    pub fn emit_globals_with_end(module: &mut Module, initial_heap: u32, heap_end: u32) {
        let mut globals = GlobalSection::new();

        // HEAP_BASE: mutable i32, starts after static data
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
            },
            &ConstExpr::i32_const(initial_heap as i32),
        );

        // HEAP_END: immutable i32, end of available memory
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: false,
            },
            &ConstExpr::i32_const(heap_end as i32),
        );

        module.section(&globals);
    }

    /// Emit WASM memory section.
    ///
    /// Adds a memory section with the specified initial size and a maximum
    /// of 256 pages (16MB).
    ///
    /// # Arguments
    ///
    /// * `module` - The WASM module to add memory to
    /// * `initial_pages` - Initial number of 64KB pages
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::alloc::BumpAllocator;
    /// use wasm_encoder::Module;
    ///
    /// let mut module = Module::new();
    /// BumpAllocator::emit_memory_section(&mut module, 1); // 64KB initial
    /// ```
    pub fn emit_memory_section(module: &mut Module, initial_pages: u32) {
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            minimum: initial_pages as u64,
            maximum: Some(MAX_MEMORY_PAGES as u64),
            memory64: false,
            shared: false,
        });
        module.section(&memories);
    }

    /// Generate the alloc function instructions.
    ///
    /// Generates WASM instructions for the bump allocator function:
    ///
    /// ```wat
    /// (func $alloc (param $size i32) (param $align i32) (result i32)
    ///   (local $aligned_ptr i32)
    ///   (local $new_heap_base i32)
    ///   ;; ... allocation logic ...
    /// )
    /// ```
    ///
    /// Function signature: `alloc(size: i32, align: i32) -> i32`
    ///
    /// # Returns
    ///
    /// A vector of WASM instructions implementing the allocator.
    /// Returns a pointer to allocated memory, or 0 (null) on failure.
    ///
    /// # Algorithm
    ///
    /// 1. Load current heap pointer (global 0)
    /// 2. Align to requested alignment: `ptr = (ptr + align - 1) & ~(align - 1)`
    /// 3. Calculate new heap pointer: `new_ptr = aligned_ptr + size`
    /// 4. Check if `new_ptr > heap_end` - if so, return 0
    /// 5. Update heap_base global with new_ptr
    /// 6. Return aligned_ptr
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::alloc::BumpAllocator;
    ///
    /// let instructions = BumpAllocator::emit_alloc_function();
    /// // Use instructions to build a Function in the code section
    /// ```
    pub fn emit_alloc_function() -> Vec<Instruction<'static>> {
        // Parameters:
        //   local 0: size (i32)
        //   local 1: align (i32)
        // Locals (declared in function):
        //   local 2: aligned_ptr (i32)
        //   local 3: new_heap_base (i32)

        vec![
            // Load current heap pointer
            Instruction::GlobalGet(0), // heap_base
            // Align: ptr = (ptr + align - 1) & ~(align - 1)
            // Step 1: ptr + align - 1
            Instruction::LocalGet(1), // align
            Instruction::I32Add,
            Instruction::I32Const(1),
            Instruction::I32Sub,
            // Step 2: ~(align - 1) = (align - 1) ^ -1
            Instruction::LocalGet(1), // align
            Instruction::I32Const(1),
            Instruction::I32Sub,
            Instruction::I32Const(-1),
            Instruction::I32Xor,
            // Step 3: AND to get aligned pointer
            Instruction::I32And,
            // Save aligned pointer to local 2
            Instruction::LocalTee(2), // aligned_ptr
            // Calculate new heap pointer: aligned_ptr + size
            Instruction::LocalGet(0), // size
            Instruction::I32Add,
            // Save new heap base to local 3
            Instruction::LocalTee(3), // new_heap_base
            // Check if we exceeded heap end
            Instruction::GlobalGet(1), // heap_end
            Instruction::I32GtU,
            // If exceeded, return 0 (null)
            Instruction::If(BlockType::Empty),
            Instruction::I32Const(0),
            Instruction::Return,
            Instruction::End,
            // Update heap base global
            Instruction::LocalGet(3),  // new_heap_base
            Instruction::GlobalSet(0), // heap_base = new_heap_base
            // Return allocated pointer
            Instruction::LocalGet(2), // aligned_ptr
            Instruction::End,
        ]
    }

    /// Build a complete WASM Function for the allocator.
    ///
    /// Creates a `wasm_encoder::Function` with the proper local declarations
    /// and alloc instructions. This is a convenience method that combines
    /// local setup with `emit_alloc_function()`.
    ///
    /// # Returns
    ///
    /// A complete Function ready to be added to a CodeSection.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::alloc::BumpAllocator;
    /// use wasm_encoder::CodeSection;
    ///
    /// let mut codes = CodeSection::new();
    /// let alloc_func = BumpAllocator::build_alloc_function();
    /// codes.function(&alloc_func);
    /// ```
    pub fn build_alloc_function() -> Function {
        // Declare 2 local i32 variables: aligned_ptr and new_heap_base
        let locals = vec![(2, ValType::I32)];
        let mut function = Function::new(locals);

        // Add all instructions
        for instr in Self::emit_alloc_function() {
            function.instruction(&instr);
        }

        function
    }

    /// Get the function type signature for the alloc function.
    ///
    /// Returns a tuple of (params, results) suitable for adding to a TypeSection.
    ///
    /// Signature: `(i32, i32) -> i32`
    /// - param 0: size in bytes
    /// - param 1: alignment requirement
    /// - result: pointer to allocated memory (or 0 on failure)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::alloc::BumpAllocator;
    /// use wasm_encoder::TypeSection;
    ///
    /// let mut types = TypeSection::new();
    /// let (params, results) = BumpAllocator::alloc_type_signature();
    /// types.function(params, results);
    /// ```
    pub fn alloc_type_signature() -> (Vec<ValType>, Vec<ValType>) {
        (vec![ValType::I32, ValType::I32], vec![ValType::I32])
    }
}

#[cfg(feature = "wasm-compile")]
impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new(0, 1)
    }
}

/// Align a value up to the given alignment.
///
/// # Arguments
///
/// * `offset` - The value to align
/// * `alignment` - The alignment requirement (must be a power of 2)
///
/// # Returns
///
/// The smallest value >= `offset` that is a multiple of `alignment`.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::alloc::align_up;
///
/// assert_eq!(align_up(0, 4), 0);
/// assert_eq!(align_up(1, 4), 4);
/// assert_eq!(align_up(4, 4), 4);
/// assert_eq!(align_up(5, 8), 8);
/// ```
#[cfg(feature = "wasm-compile")]
pub fn align_up(offset: u32, alignment: u32) -> u32 {
    (offset + alignment - 1) & !(alignment - 1)
}

#[cfg(test)]
#[cfg(feature = "wasm-compile")]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(3, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(7, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
    }

    #[test]
    fn test_bump_allocator_new() {
        let allocator = BumpAllocator::new(0, 1);
        assert_eq!(allocator.heap_base_global(), 0);
        assert_eq!(allocator.heap_end_global(), 1);
    }

    #[test]
    fn test_bump_allocator_default() {
        let allocator = BumpAllocator::default();
        assert_eq!(allocator.heap_base_global(), 0);
        assert_eq!(allocator.heap_end_global(), 1);
    }

    #[test]
    fn test_alloc_type_signature() {
        let (params, results) = BumpAllocator::alloc_type_signature();
        assert_eq!(params, vec![ValType::I32, ValType::I32]);
        assert_eq!(results, vec![ValType::I32]);
    }

    #[test]
    fn test_emit_alloc_function_not_empty() {
        let instructions = BumpAllocator::emit_alloc_function();
        assert!(!instructions.is_empty());
        // Should end with End instruction
        assert!(matches!(instructions.last(), Some(Instruction::End)));
    }

    #[test]
    fn test_build_alloc_function() {
        // Just verify it doesn't panic
        let _function = BumpAllocator::build_alloc_function();
    }

    #[test]
    fn test_emit_memory_section() {
        let mut module = Module::new();
        BumpAllocator::emit_memory_section(&mut module, 1);
        // Verify module can be encoded without error
        let _bytes = module.finish();
    }

    #[test]
    fn test_emit_globals() {
        let mut module = Module::new();
        BumpAllocator::emit_globals(&mut module, 1024);
        // Verify module can be encoded without error
        let _bytes = module.finish();
    }

    #[test]
    fn test_emit_globals_with_end() {
        let mut module = Module::new();
        BumpAllocator::emit_globals_with_end(&mut module, 1024, 4 * PAGE_SIZE);
        // Verify module can be encoded without error
        let _bytes = module.finish();
    }

    #[test]
    fn test_constants() {
        assert_eq!(PAGE_SIZE, 65536);
        assert_eq!(DEFAULT_MEMORY_PAGES, 1);
        assert_eq!(MAX_MEMORY_PAGES, 256);
        assert_eq!(DEFAULT_HEAP_START, 1024);
    }
}
