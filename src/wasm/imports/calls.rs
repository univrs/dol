//! Host function call generation for WASM.
//!
//! This module provides the `CallGenerator` which generates WASM instructions
//! for calling the 22 host functions defined in the VUDO runtime ABI.
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::wasm::imports::calls::CallGenerator;
//! use wasm_encoder::{Function, Instruction};
//!
//! let mut func = Function::new(vec![]);
//! let mut call_gen = CallGenerator::new();
//!
//! // Generate a println call
//! let call_site = call_gen.gen_println(0, 13); // ptr=0, len=13
//! func.instruction(&Instruction::I32Const(0));  // ptr
//! func.instruction(&Instruction::I32Const(13)); // len
//! func.instruction(&Instruction::Call(call_site.function_idx));
//! ```

#[cfg(feature = "wasm-compile")]
use wasm_encoder::ValType;

/// A host function call site.
///
/// Contains the information needed to generate a call to a host function,
/// including the function index in the import table and metadata about
/// arguments and results.
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone)]
pub struct HostCallSite {
    /// The WASM function index (index into import table)
    pub function_idx: u32,
    /// The name of the host function
    pub function_name: String,
    /// Parameter types
    pub param_types: Vec<ValType>,
    /// Result type (None for void)
    pub result_type: Option<ValType>,
}

/// Generates WASM instructions for calling host functions.
///
/// The `CallGenerator` maintains the mapping between host function names
/// and their corresponding WASM function indices in the import table.
///
/// # Host Functions (22 total)
///
/// ## I/O (4)
/// - `vudo_print(ptr: i32, len: i32)` - Print without newline
/// - `vudo_println(ptr: i32, len: i32)` - Print with newline
/// - `vudo_log(level: i32, ptr: i32, len: i32)` - Log at level
/// - `vudo_error(ptr: i32, len: i32) -> i32` - Report error
///
/// ## Memory (3)
/// - `vudo_alloc(size: i32) -> i32` - Allocate memory
/// - `vudo_free(ptr: i32, size: i32)` - Free memory
/// - `vudo_realloc(ptr: i32, old_size: i32, new_size: i32) -> i32` - Reallocate
///
/// ## Time (3)
/// - `vudo_now() -> f64` - Wall-clock time in milliseconds
/// - `vudo_sleep(ms: i32)` - Sleep for duration
/// - `vudo_monotonic_now() -> i64` - Monotonic time in milliseconds
///
/// ## Messaging (5)
/// - `vudo_send(target_ptr: i32, target_len: i32, msg_ptr: i32, msg_len: i32) -> i32`
/// - `vudo_recv(timeout_ms: i32, out_ptr: i32, out_len: i32) -> i32`
/// - `vudo_pending() -> i32`
/// - `vudo_broadcast(msg_ptr: i32, msg_len: i32) -> i32`
/// - `vudo_free_message(msg_id: i32)`
///
/// ## Random (2)
/// - `vudo_random() -> f64` - Random float [0, 1)
/// - `vudo_random_bytes(ptr: i32, len: i32)` - Fill buffer with random bytes
///
/// ## Effects (2)
/// - `vudo_emit_effect(effect_ptr: i32, effect_len: i32) -> i32`
/// - `vudo_subscribe(pattern_ptr: i32, pattern_len: i32) -> i32`
///
/// ## Debug (3)
/// - `vudo_breakpoint()` - Trigger debugger breakpoint
/// - `vudo_assert(condition: i32, msg_ptr: i32, msg_len: i32)` - Assert condition
/// - `vudo_panic(msg_ptr: i32, msg_len: i32)` - Panic with message (never returns)
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone)]
pub struct CallGenerator {
    /// Mapping from function name to import index
    function_indices: std::collections::HashMap<String, u32>,
    /// Next available function index
    next_idx: u32,
}

#[cfg(feature = "wasm-compile")]
impl CallGenerator {
    /// Create a new CallGenerator.
    ///
    /// Initializes with no registered functions. Functions will be registered
    /// as they are first used.
    pub fn new() -> Self {
        Self {
            function_indices: std::collections::HashMap::new(),
            next_idx: 0,
        }
    }

    /// Register a host function and return its call site information.
    ///
    /// If the function is already registered, returns the existing call site.
    fn register_function(
        &mut self,
        name: &str,
        params: Vec<ValType>,
        result: Option<ValType>,
    ) -> HostCallSite {
        let idx = *self
            .function_indices
            .entry(name.to_string())
            .or_insert_with(|| {
                let idx = self.next_idx;
                self.next_idx += 1;
                idx
            });

        HostCallSite {
            function_idx: idx,
            function_name: name.to_string(),
            param_types: params,
            result_type: result,
        }
    }

    /// Get all registered host functions in registration order.
    ///
    /// Returns a vector of (name, call_site) pairs sorted by function index.
    pub fn get_registered_functions(&self) -> Vec<(String, u32)> {
        let mut functions: Vec<_> = self.function_indices.iter().collect();
        functions.sort_by_key(|(_, idx)| *idx);
        functions
            .into_iter()
            .map(|(name, idx)| (name.clone(), *idx))
            .collect()
    }

    // ================================
    // I/O Functions (4)
    // ================================

    /// Generate a call to `vudo_print(ptr: i32, len: i32)`.
    ///
    /// Prints a message to stdout without a trailing newline.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Offset in WASM linear memory where the string is stored
    /// * `len` - Length of the string in bytes
    pub fn gen_print(&mut self) -> HostCallSite {
        self.register_function("vudo_print", vec![ValType::I32, ValType::I32], None)
    }

    /// Generate a call to `vudo_println(ptr: i32, len: i32)`.
    ///
    /// Prints a message to stdout with a trailing newline.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Offset in WASM linear memory where the string is stored
    /// * `len` - Length of the string in bytes
    pub fn gen_println(&mut self) -> HostCallSite {
        self.register_function("vudo_println", vec![ValType::I32, ValType::I32], None)
    }

    /// Generate a call to `vudo_log(level: i32, ptr: i32, len: i32)`.
    ///
    /// Logs a message at the specified log level.
    ///
    /// # Arguments
    ///
    /// * `level` - Log level (0=TRACE, 1=DEBUG, 2=INFO, 3=WARN, 4=ERROR)
    /// * `ptr` - Offset in WASM linear memory where the string is stored
    /// * `len` - Length of the string in bytes
    pub fn gen_log(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_log",
            vec![ValType::I32, ValType::I32, ValType::I32],
            None,
        )
    }

    /// Generate a call to `vudo_error(ptr: i32, len: i32) -> i32`.
    ///
    /// Reports an error to the host and returns an error code.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Offset in WASM linear memory where the error message is stored
    /// * `len` - Length of the error message in bytes
    ///
    /// # Returns
    ///
    /// Error code assigned by the host (typically non-zero).
    pub fn gen_error(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_error",
            vec![ValType::I32, ValType::I32],
            Some(ValType::I32),
        )
    }

    // ================================
    // Memory Functions (3)
    // ================================

    /// Generate a call to `vudo_alloc(size: i32) -> i32`.
    ///
    /// Allocates memory from the host allocator.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate
    ///
    /// # Returns
    ///
    /// Pointer to allocated memory, or null (0) if allocation fails.
    pub fn gen_alloc(&mut self) -> HostCallSite {
        self.register_function("vudo_alloc", vec![ValType::I32], Some(ValType::I32))
    }

    /// Generate a call to `vudo_free(ptr: i32, size: i32)`.
    ///
    /// Frees memory previously allocated by `vudo_alloc`.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Pointer to memory to free
    /// * `size` - Size of the allocation in bytes (must match original allocation)
    pub fn gen_free(&mut self) -> HostCallSite {
        self.register_function("vudo_free", vec![ValType::I32, ValType::I32], None)
    }

    /// Generate a call to `vudo_realloc(ptr: i32, old_size: i32, new_size: i32) -> i32`.
    ///
    /// Reallocates memory to a new size.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Pointer to existing allocation
    /// * `old_size` - Current size of the allocation in bytes
    /// * `new_size` - Desired new size in bytes
    ///
    /// # Returns
    ///
    /// Pointer to reallocated memory, or null (0) if reallocation fails.
    pub fn gen_realloc(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_realloc",
            vec![ValType::I32, ValType::I32, ValType::I32],
            Some(ValType::I32),
        )
    }

    // ================================
    // Time Functions (3)
    // ================================

    /// Generate a call to `vudo_now() -> f64`.
    ///
    /// Returns the current wall-clock time in milliseconds since Unix epoch.
    ///
    /// # Returns
    ///
    /// Timestamp as a 64-bit float representing milliseconds.
    pub fn gen_now(&mut self) -> HostCallSite {
        self.register_function("vudo_now", vec![], Some(ValType::F64))
    }

    /// Generate a call to `vudo_sleep(ms: i32)`.
    ///
    /// Suspends execution for the specified duration.
    ///
    /// # Arguments
    ///
    /// * `ms` - Duration to sleep in milliseconds
    pub fn gen_sleep(&mut self) -> HostCallSite {
        self.register_function("vudo_sleep", vec![ValType::I32], None)
    }

    /// Generate a call to `vudo_monotonic_now() -> i64`.
    ///
    /// Returns a monotonic timestamp in milliseconds.
    ///
    /// # Returns
    ///
    /// Monotonically increasing timestamp in milliseconds.
    pub fn gen_monotonic_now(&mut self) -> HostCallSite {
        self.register_function("vudo_monotonic_now", vec![], Some(ValType::I64))
    }

    // ================================
    // Messaging Functions (5)
    // ================================

    /// Generate a call to `vudo_send(target_ptr: i32, target_len: i32, msg_ptr: i32, msg_len: i32) -> i32`.
    ///
    /// Sends a message to a target agent.
    ///
    /// # Arguments
    ///
    /// * `target_ptr` - Pointer to target agent identifier
    /// * `target_len` - Length of target identifier
    /// * `msg_ptr` - Pointer to message payload
    /// * `msg_len` - Length of message payload
    ///
    /// # Returns
    ///
    /// 0 on success, non-zero error code on failure.
    pub fn gen_send(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_send",
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            Some(ValType::I32),
        )
    }

    /// Generate a call to `vudo_recv(timeout_ms: i32, out_ptr: i32, out_len: i32) -> i32`.
    ///
    /// Receives a message from the agent's message queue.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Maximum time to wait in milliseconds (0 for non-blocking)
    /// * `out_ptr` - Pointer to output buffer
    /// * `out_len` - Size of output buffer
    ///
    /// # Returns
    ///
    /// Number of bytes written (positive), 0 if no message, negative on error.
    pub fn gen_recv(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_recv",
            vec![ValType::I32, ValType::I32, ValType::I32],
            Some(ValType::I32),
        )
    }

    /// Generate a call to `vudo_pending() -> i32`.
    ///
    /// Returns the number of pending messages in the agent's queue.
    ///
    /// # Returns
    ///
    /// Number of messages waiting to be received.
    pub fn gen_pending(&mut self) -> HostCallSite {
        self.register_function("vudo_pending", vec![], Some(ValType::I32))
    }

    /// Generate a call to `vudo_broadcast(msg_ptr: i32, msg_len: i32) -> i32`.
    ///
    /// Broadcasts a message to all agents in the system.
    ///
    /// # Arguments
    ///
    /// * `msg_ptr` - Pointer to message payload
    /// * `msg_len` - Length of message payload
    ///
    /// # Returns
    ///
    /// Number of agents the message was sent to, or 0 on error.
    pub fn gen_broadcast(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_broadcast",
            vec![ValType::I32, ValType::I32],
            Some(ValType::I32),
        )
    }

    /// Generate a call to `vudo_free_message(msg_id: i32)`.
    ///
    /// Frees resources associated with a received message.
    ///
    /// # Arguments
    ///
    /// * `msg_id` - Message identifier from a previous receive operation
    pub fn gen_free_message(&mut self) -> HostCallSite {
        self.register_function("vudo_free_message", vec![ValType::I32], None)
    }

    // ================================
    // Random Functions (2)
    // ================================

    /// Generate a call to `vudo_random() -> f64`.
    ///
    /// Generates a random floating-point number in the range [0.0, 1.0).
    ///
    /// # Returns
    ///
    /// Cryptographically secure random number.
    pub fn gen_random(&mut self) -> HostCallSite {
        self.register_function("vudo_random", vec![], Some(ValType::F64))
    }

    /// Generate a call to `vudo_random_bytes(ptr: i32, len: i32)`.
    ///
    /// Fills a buffer with cryptographically secure random bytes.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Pointer to buffer to fill
    /// * `len` - Number of random bytes to generate
    pub fn gen_random_bytes(&mut self) -> HostCallSite {
        self.register_function("vudo_random_bytes", vec![ValType::I32, ValType::I32], None)
    }

    // ================================
    // Effects Functions (2)
    // ================================

    /// Generate a call to `vudo_emit_effect(effect_ptr: i32, effect_len: i32) -> i32`.
    ///
    /// Emits a side effect event that can be observed by the host.
    ///
    /// # Arguments
    ///
    /// * `effect_ptr` - Pointer to effect payload (typically JSON)
    /// * `effect_len` - Length of effect payload
    ///
    /// # Returns
    ///
    /// Effect ID assigned by the host, or 0 on error.
    pub fn gen_emit_effect(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_emit_effect",
            vec![ValType::I32, ValType::I32],
            Some(ValType::I32),
        )
    }

    /// Generate a call to `vudo_subscribe(pattern_ptr: i32, pattern_len: i32) -> i32`.
    ///
    /// Subscribes to effect events matching a pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern_ptr` - Pointer to pattern string (glob or regex)
    /// * `pattern_len` - Length of pattern string
    ///
    /// # Returns
    ///
    /// Subscription ID, or 0 on error.
    pub fn gen_subscribe(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_subscribe",
            vec![ValType::I32, ValType::I32],
            Some(ValType::I32),
        )
    }

    // ================================
    // Debug Functions (3)
    // ================================

    /// Generate a call to `vudo_breakpoint()`.
    ///
    /// Triggers a breakpoint in the host debugger.
    pub fn gen_breakpoint(&mut self) -> HostCallSite {
        self.register_function("vudo_breakpoint", vec![], None)
    }

    /// Generate a call to `vudo_assert(condition: i32, msg_ptr: i32, msg_len: i32)`.
    ///
    /// Asserts that a condition is true, panicking with a message if false.
    ///
    /// # Arguments
    ///
    /// * `condition` - 1 if assertion should pass, 0 if it should fail
    /// * `msg_ptr` - Pointer to assertion message
    /// * `msg_len` - Length of message
    pub fn gen_assert(&mut self) -> HostCallSite {
        self.register_function(
            "vudo_assert",
            vec![ValType::I32, ValType::I32, ValType::I32],
            None,
        )
    }

    /// Generate a call to `vudo_panic(msg_ptr: i32, msg_len: i32)`.
    ///
    /// Immediately terminates execution with a panic message.
    ///
    /// Note: This function never returns in the WASM context.
    ///
    /// # Arguments
    ///
    /// * `msg_ptr` - Pointer to panic message
    /// * `msg_len` - Length of message
    pub fn gen_panic(&mut self) -> HostCallSite {
        self.register_function("vudo_panic", vec![ValType::I32, ValType::I32], None)
    }
}

#[cfg(feature = "wasm-compile")]
impl Default for CallGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-compile")]
mod tests {
    use super::*;

    #[test]
    fn test_call_generator_registration() {
        let mut gen = CallGenerator::new();

        // Register some functions
        let print_call = gen.gen_print();
        assert_eq!(print_call.function_name, "vudo_print");
        assert_eq!(print_call.function_idx, 0);
        assert_eq!(print_call.param_types.len(), 2);
        assert!(print_call.result_type.is_none());

        let alloc_call = gen.gen_alloc();
        assert_eq!(alloc_call.function_name, "vudo_alloc");
        assert_eq!(alloc_call.function_idx, 1);
        assert_eq!(alloc_call.param_types.len(), 1);
        assert_eq!(alloc_call.result_type, Some(ValType::I32));

        // Re-registering should return the same index
        let print_call2 = gen.gen_print();
        assert_eq!(print_call2.function_idx, 0);
    }

    #[test]
    fn test_all_host_functions() {
        let mut gen = CallGenerator::new();

        // I/O functions
        assert_eq!(gen.gen_print().param_types.len(), 2);
        assert_eq!(gen.gen_println().param_types.len(), 2);
        assert_eq!(gen.gen_log().param_types.len(), 3);
        assert_eq!(gen.gen_error().result_type, Some(ValType::I32));

        // Memory functions
        assert_eq!(gen.gen_alloc().result_type, Some(ValType::I32));
        assert_eq!(gen.gen_free().param_types.len(), 2);
        assert_eq!(gen.gen_realloc().param_types.len(), 3);

        // Time functions
        assert_eq!(gen.gen_now().result_type, Some(ValType::F64));
        assert_eq!(gen.gen_sleep().param_types.len(), 1);
        assert_eq!(gen.gen_monotonic_now().result_type, Some(ValType::I64));

        // Messaging functions
        assert_eq!(gen.gen_send().param_types.len(), 4);
        assert_eq!(gen.gen_recv().param_types.len(), 3);
        assert_eq!(gen.gen_pending().param_types.len(), 0);
        assert_eq!(gen.gen_broadcast().param_types.len(), 2);
        assert_eq!(gen.gen_free_message().param_types.len(), 1);

        // Random functions
        assert_eq!(gen.gen_random().result_type, Some(ValType::F64));
        assert_eq!(gen.gen_random_bytes().param_types.len(), 2);

        // Effects functions
        assert_eq!(gen.gen_emit_effect().result_type, Some(ValType::I32));
        assert_eq!(gen.gen_subscribe().result_type, Some(ValType::I32));

        // Debug functions
        assert_eq!(gen.gen_breakpoint().param_types.len(), 0);
        assert_eq!(gen.gen_assert().param_types.len(), 3);
        assert_eq!(gen.gen_panic().param_types.len(), 2);

        // Should have registered 22 functions
        assert_eq!(gen.get_registered_functions().len(), 22);
    }

    #[test]
    fn test_get_registered_functions() {
        let mut gen = CallGenerator::new();

        gen.gen_print();
        gen.gen_alloc();
        gen.gen_now();

        let functions = gen.get_registered_functions();
        assert_eq!(functions.len(), 3);
        assert_eq!(functions[0].0, "vudo_print");
        assert_eq!(functions[1].0, "vudo_alloc");
        assert_eq!(functions[2].0, "vudo_now");
    }
}
