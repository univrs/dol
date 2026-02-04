//! Host function imports for VUDO runtime.
//!
//! This module declares the external host functions that are imported from the WASM host
//! environment. These functions provide the interface between WASM guest code and the
//! host runtime, enabling I/O, memory management, inter-agent messaging, and more.
//!
//! All functions are declared as `extern "C"` and linked from the `vudo` import module.
//!
//! # Safety
//!
//! All functions in this module are `unsafe` because they:
//! - Accept raw pointers without lifetime guarantees
//! - Perform operations that can fail in ways not represented by Rust's type system
//! - Interface with external code that may have different invariants
//!
//! Callers must ensure:
//! - Pointers are valid and properly aligned
//! - Length parameters accurately reflect buffer sizes
//! - String data is valid UTF-8 where expected
//! - Memory is not freed while still in use by the host

#[link(wasm_import_module = "vudo")]
extern "C" {
    // ================================
    // I/O Functions (4)
    // ================================

    /// Prints a message to the host's standard output without a trailing newline.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the UTF-8 encoded message buffer
    /// - `len`: Length of the message in bytes
    ///
    /// # Safety
    ///
    /// - `ptr` must point to valid memory containing `len` bytes
    /// - The buffer must contain valid UTF-8 data
    /// - `ptr` must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_print;
    /// let msg = "Hello, World!";
    /// unsafe {
    ///     vudo_print(msg.as_ptr(), msg.len());
    /// }
    /// ```
    pub fn vudo_print(ptr: *const u8, len: usize);

    /// Prints a message to the host's standard output with a trailing newline.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the UTF-8 encoded message buffer
    /// - `len`: Length of the message in bytes
    ///
    /// # Safety
    ///
    /// - `ptr` must point to valid memory containing `len` bytes
    /// - The buffer must contain valid UTF-8 data
    /// - `ptr` must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_println;
    /// let msg = "Hello, World!";
    /// unsafe {
    ///     vudo_println(msg.as_ptr(), msg.len());
    /// }
    /// ```
    pub fn vudo_println(ptr: *const u8, len: usize);

    /// Logs a message at the specified log level.
    ///
    /// # Parameters
    ///
    /// - `level`: Log level (0=TRACE, 1=DEBUG, 2=INFO, 3=WARN, 4=ERROR)
    /// - `ptr`: Pointer to the UTF-8 encoded log message buffer
    /// - `len`: Length of the message in bytes
    ///
    /// # Safety
    ///
    /// - `ptr` must point to valid memory containing `len` bytes
    /// - The buffer must contain valid UTF-8 data
    /// - `ptr` must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_log;
    /// const INFO: u32 = 2;
    /// let msg = "System initialized";
    /// unsafe {
    ///     vudo_log(INFO, msg.as_ptr(), msg.len());
    /// }
    /// ```
    pub fn vudo_log(level: u32, ptr: *const u8, len: usize);

    /// Reports an error to the host and returns an error code.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the UTF-8 encoded error message buffer
    /// - `len`: Length of the error message in bytes
    ///
    /// # Returns
    ///
    /// Error code assigned by the host (typically non-zero).
    ///
    /// # Safety
    ///
    /// - `ptr` must point to valid memory containing `len` bytes
    /// - The buffer must contain valid UTF-8 data
    /// - `ptr` must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_error;
    /// let error = "Failed to initialize module";
    /// let error_code = unsafe {
    ///     vudo_error(error.as_ptr(), error.len())
    /// };
    /// ```
    pub fn vudo_error(ptr: *const u8, len: usize) -> u32;

    // ================================
    // Memory Functions (3)
    // ================================

    /// Allocates memory from the host allocator.
    ///
    /// # Parameters
    ///
    /// - `size`: Number of bytes to allocate
    ///
    /// # Returns
    ///
    /// Pointer to the allocated memory, or null if allocation fails.
    ///
    /// # Safety
    ///
    /// - The returned pointer must be freed with `vudo_free` when no longer needed
    /// - Do not mix with Rust's allocator (Box, Vec, etc.)
    /// - Memory is not initialized and may contain arbitrary data
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::{vudo_alloc, vudo_free};
    /// unsafe {
    ///     let ptr = vudo_alloc(1024);
    ///     if !ptr.is_null() {
    ///         // Use memory...
    ///         vudo_free(ptr, 1024);
    ///     }
    /// }
    /// ```
    pub fn vudo_alloc(size: usize) -> *mut u8;

    /// Frees memory previously allocated by `vudo_alloc`.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to memory previously allocated by `vudo_alloc`
    /// - `size`: Size of the allocation in bytes (must match original allocation size)
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated by `vudo_alloc` with the same `size`
    /// - Must not be called more than once on the same pointer
    /// - `ptr` must not be used after this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::{vudo_alloc, vudo_free};
    /// unsafe {
    ///     let ptr = vudo_alloc(1024);
    ///     // Use memory...
    ///     vudo_free(ptr, 1024);
    /// }
    /// ```
    pub fn vudo_free(ptr: *mut u8, size: usize);

    /// Reallocates memory to a new size.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to memory previously allocated by `vudo_alloc`
    /// - `old_size`: Current size of the allocation in bytes
    /// - `new_size`: Desired new size in bytes
    ///
    /// # Returns
    ///
    /// Pointer to the reallocated memory, or null if reallocation fails.
    /// If reallocation succeeds, the old pointer is invalidated.
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated by `vudo_alloc` with size `old_size`
    /// - If reallocation succeeds, the old pointer must not be used
    /// - If reallocation fails, the old pointer remains valid
    /// - Data is preserved up to the minimum of old and new sizes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::{vudo_alloc, vudo_realloc, vudo_free};
    /// unsafe {
    ///     let mut ptr = vudo_alloc(1024);
    ///     // Need more space...
    ///     let new_ptr = vudo_realloc(ptr, 1024, 2048);
    ///     if !new_ptr.is_null() {
    ///         ptr = new_ptr;
    ///     }
    ///     vudo_free(ptr, 2048);
    /// }
    /// ```
    pub fn vudo_realloc(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8;

    // ================================
    // Time Functions (3)
    // ================================

    /// Returns the current wall-clock time in milliseconds since Unix epoch.
    ///
    /// # Returns
    ///
    /// Timestamp as a 64-bit floating point value representing milliseconds.
    /// This may not be monotonic and can be affected by system clock changes.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_now;
    /// let start = unsafe { vudo_now() };
    /// // Do work...
    /// let elapsed = unsafe { vudo_now() } - start;
    /// ```
    pub fn vudo_now() -> f64;

    /// Suspends execution for the specified duration.
    ///
    /// # Parameters
    ///
    /// - `ms`: Duration to sleep in milliseconds
    ///
    /// # Note
    ///
    /// This yields control back to the host, which may schedule other agents.
    /// The actual sleep duration may be longer than requested.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_sleep;
    /// unsafe {
    ///     vudo_sleep(1000); // Sleep for 1 second
    /// }
    /// ```
    pub fn vudo_sleep(ms: u32);

    /// Returns a monotonic timestamp in milliseconds.
    ///
    /// # Returns
    ///
    /// Monotonically increasing timestamp in milliseconds.
    /// Guaranteed not to go backwards, unaffected by system clock changes.
    /// Suitable for measuring elapsed time and timeouts.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_monotonic_now;
    /// let start = unsafe { vudo_monotonic_now() };
    /// // Do work...
    /// let elapsed = unsafe { vudo_monotonic_now() } - start;
    /// ```
    pub fn vudo_monotonic_now() -> u64;

    // ================================
    // Messaging Functions (5)
    // ================================

    /// Sends a message to a target agent.
    ///
    /// # Parameters
    ///
    /// - `target_ptr`: Pointer to the target agent identifier (UTF-8)
    /// - `target_len`: Length of the target identifier in bytes
    /// - `msg_ptr`: Pointer to the message payload
    /// - `msg_len`: Length of the message payload in bytes
    ///
    /// # Returns
    ///
    /// 0 on success, non-zero error code on failure.
    ///
    /// # Safety
    ///
    /// - Both `target_ptr` and `msg_ptr` must point to valid memory
    /// - Lengths must accurately reflect buffer sizes
    /// - Buffers must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_send;
    /// let target = "agent-42";
    /// let message = b"Hello, agent!";
    /// let result = unsafe {
    ///     vudo_send(
    ///         target.as_ptr(),
    ///         target.len(),
    ///         message.as_ptr(),
    ///         message.len()
    ///     )
    /// };
    /// ```
    pub fn vudo_send(
        target_ptr: *const u8,
        target_len: usize,
        msg_ptr: *const u8,
        msg_len: usize,
    ) -> u32;

    /// Receives a message from the agent's message queue.
    ///
    /// # Parameters
    ///
    /// - `timeout_ms`: Maximum time to wait in milliseconds (0 for non-blocking)
    /// - `out_ptr`: Pointer to buffer where message will be written
    /// - `out_len`: Size of the output buffer in bytes
    ///
    /// # Returns
    ///
    /// - Positive value: Number of bytes written to the buffer
    /// - 0: No message available (timeout expired)
    /// - Negative value: Error code
    ///
    /// # Safety
    ///
    /// - `out_ptr` must point to a valid buffer of at least `out_len` bytes
    /// - If a message is larger than `out_len`, it will be truncated
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_recv;
    /// let mut buffer = [0u8; 1024];
    /// let bytes_read = unsafe {
    ///     vudo_recv(1000, buffer.as_mut_ptr(), buffer.len())
    /// };
    /// if bytes_read > 0 {
    ///     let msg = &buffer[..bytes_read as usize];
    ///     // Process message...
    /// }
    /// ```
    pub fn vudo_recv(timeout_ms: u32, out_ptr: *mut u8, out_len: usize) -> i32;

    /// Returns the number of pending messages in the agent's queue.
    ///
    /// # Returns
    ///
    /// Number of messages waiting to be received.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_pending;
    /// let count = unsafe { vudo_pending() };
    /// if count > 0 {
    ///     // Process messages...
    /// }
    /// ```
    pub fn vudo_pending() -> u32;

    /// Broadcasts a message to all agents in the system.
    ///
    /// # Parameters
    ///
    /// - `msg_ptr`: Pointer to the message payload
    /// - `msg_len`: Length of the message payload in bytes
    ///
    /// # Returns
    ///
    /// Number of agents the message was sent to, or 0 on error.
    ///
    /// # Safety
    ///
    /// - `msg_ptr` must point to valid memory containing `msg_len` bytes
    /// - Buffer must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_broadcast;
    /// let announcement = b"System shutting down";
    /// let recipient_count = unsafe {
    ///     vudo_broadcast(announcement.as_ptr(), announcement.len())
    /// };
    /// ```
    pub fn vudo_broadcast(msg_ptr: *const u8, msg_len: usize) -> u32;

    /// Frees resources associated with a received message.
    ///
    /// # Parameters
    ///
    /// - `msg_id`: Message identifier returned by a previous receive operation
    ///
    /// # Safety
    ///
    /// - `msg_id` must be a valid message ID from a previous receive
    /// - Must not be called more than once with the same ID
    ///
    /// # Note
    ///
    /// In the current implementation, this may be a no-op, but should be
    /// called for forward compatibility with message lifecycle management.
    pub fn vudo_free_message(msg_id: u32);

    // ================================
    // Random Functions (2)
    // ================================

    /// Generates a random floating-point number in the range [0.0, 1.0).
    ///
    /// # Returns
    ///
    /// Cryptographically secure random number.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_random;
    /// let random_value = unsafe { vudo_random() };
    /// let coin_flip = random_value < 0.5;
    /// ```
    pub fn vudo_random() -> f64;

    /// Fills a buffer with cryptographically secure random bytes.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the buffer to fill
    /// - `len`: Number of random bytes to generate
    ///
    /// # Safety
    ///
    /// - `ptr` must point to a valid buffer of at least `len` bytes
    /// - Buffer must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_random_bytes;
    /// let mut nonce = [0u8; 32];
    /// unsafe {
    ///     vudo_random_bytes(nonce.as_mut_ptr(), nonce.len());
    /// }
    /// ```
    pub fn vudo_random_bytes(ptr: *mut u8, len: usize);

    // ================================
    // Effects Functions (2)
    // ================================

    /// Emits a side effect event that can be observed by the host.
    ///
    /// # Parameters
    ///
    /// - `effect_ptr`: Pointer to the effect payload (typically JSON)
    /// - `effect_len`: Length of the effect payload in bytes
    ///
    /// # Returns
    ///
    /// Effect ID assigned by the host, or 0 on error.
    ///
    /// # Safety
    ///
    /// - `effect_ptr` must point to valid memory containing `effect_len` bytes
    /// - Buffer must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_emit_effect;
    /// let effect = r#"{"type": "state_change", "value": 42}"#;
    /// let effect_id = unsafe {
    ///     vudo_emit_effect(effect.as_ptr(), effect.len())
    /// };
    /// ```
    pub fn vudo_emit_effect(effect_ptr: *const u8, effect_len: usize) -> u32;

    /// Subscribes to effect events matching a pattern.
    ///
    /// # Parameters
    ///
    /// - `pattern_ptr`: Pointer to the pattern string (glob or regex)
    /// - `pattern_len`: Length of the pattern string in bytes
    ///
    /// # Returns
    ///
    /// Subscription ID, or 0 on error.
    ///
    /// # Safety
    ///
    /// - `pattern_ptr` must point to valid UTF-8 memory containing `pattern_len` bytes
    /// - Buffer must remain valid for the duration of this call
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_subscribe;
    /// let pattern = "state_change.*";
    /// let sub_id = unsafe {
    ///     vudo_subscribe(pattern.as_ptr(), pattern.len())
    /// };
    /// ```
    pub fn vudo_subscribe(pattern_ptr: *const u8, pattern_len: usize) -> u32;

    // ================================
    // Debug Functions (3)
    // ================================

    /// Triggers a breakpoint in the host debugger.
    ///
    /// If no debugger is attached, this may be a no-op or cause the
    /// program to pause, depending on the host implementation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_breakpoint;
    /// # let suspicious_value = 42;
    /// if suspicious_value > 100 {
    ///     unsafe { vudo_breakpoint(); }
    /// }
    /// ```
    pub fn vudo_breakpoint();

    /// Asserts that a condition is true, panicking with a message if false.
    ///
    /// # Parameters
    ///
    /// - `condition`: 1 if the assertion should pass, 0 if it should fail
    /// - `msg_ptr`: Pointer to the assertion message (UTF-8)
    /// - `msg_len`: Length of the message in bytes
    ///
    /// # Safety
    ///
    /// - `msg_ptr` must point to valid UTF-8 memory containing `msg_len` bytes
    /// - Buffer must remain valid for the duration of this call
    /// - If assertion fails, this function does not return
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_assert;
    /// # let value = 42;
    /// let message = "Value must be less than 100";
    /// unsafe {
    ///     vudo_assert(
    ///         (value < 100) as u32,
    ///         message.as_ptr(),
    ///         message.len()
    ///     );
    /// }
    /// ```
    pub fn vudo_assert(condition: u32, msg_ptr: *const u8, msg_len: usize);

    /// Immediately terminates execution with a panic message.
    ///
    /// # Parameters
    ///
    /// - `msg_ptr`: Pointer to the panic message (UTF-8)
    /// - `msg_len`: Length of the message in bytes
    ///
    /// # Safety
    ///
    /// - `msg_ptr` must point to valid UTF-8 memory containing `msg_len` bytes
    /// - This function never returns
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use univrs_dol::host::vudo_panic;
    /// let error = "Unrecoverable error occurred";
    /// unsafe {
    ///     vudo_panic(error.as_ptr(), error.len())
    /// }
    /// ```
    pub fn vudo_panic(msg_ptr: *const u8, msg_len: usize) -> !;
}
