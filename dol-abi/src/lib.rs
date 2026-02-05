//! DOL ABI (Application Binary Interface) module
//!
//! This module provides the core ABI types and interfaces for DOL WASM-based applications.
//! It defines the contract between the DOL runtime and compiled DOL programs.

pub const ABI_VERSION: &str = "0.1.0";
pub const IMPORT_MODULE: &str = "vudo";

pub mod host;
pub mod message;
pub mod types;
pub mod error;
pub mod wasm_types;

pub use error::{Error, Result};
pub use types::*;
pub use wasm_types::{
    HostFunction, HostFunctionCategory, HostFunctionSignature, WasmType,
    standard_host_functions,
};
