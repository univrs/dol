//! WASM host function imports and call generation.
//!
//! This module provides utilities for generating calls to host functions
//! and managing memory layout for strings and data in WASM linear memory.
//!
//! # Components
//!
//! - [`calls`]: CallGenerator for generating host function call instructions
//! - [`memory`]: StringEncoder and MemoryLayout for managing WASM linear memory
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::wasm::imports::calls::CallGenerator;
//! use metadol::wasm::imports::memory::StringEncoder;
//!
//! let mut encoder = StringEncoder::new();
//! let offset = encoder.encode_string("Hello, World!");
//!
//! let mut call_gen = CallGenerator::new();
//! let call_site = call_gen.gen_println(offset, 13);
//! ```

pub mod calls;
pub mod memory;

pub use calls::{CallGenerator, HostCallSite};
pub use memory::{MemoryLayout, StringEncoder};
