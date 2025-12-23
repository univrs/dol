//! MLIR Context Management
//!
//! This module provides a wrapper around the MLIR context, which is the
//! central hub for all MLIR operations. The context manages types, operations,
//! and attributes, ensuring they are properly allocated and deallocated.
//!
//! # Design
//!
//! The `MlirContext` wrapper provides:
//! - Safe initialization and cleanup
//! - Module creation
//! - Location tracking for error reporting
//! - Dialect registration
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::mlir::MlirContext;
//!
//! let ctx = MlirContext::new();
//! let module = ctx.create_module("my_module");
//! ```

use crate::ast::Span;

// When MLIR feature is enabled, use the full melior implementation
#[cfg(feature = "mlir")]
use melior::{
    dialect::DialectRegistry,
    ir::{Location, Module},
    utility::register_all_dialects,
    Context,
};

/// MLIR context wrapper.
///
/// The context is the top-level container for all MLIR entities.
/// It owns and manages the lifetime of types, operations, and attributes.
///
/// # Example
///
/// ```rust,ignore
/// let ctx = MlirContext::new();
/// let module = ctx.create_module("example");
/// ```
#[cfg(feature = "mlir")]
pub struct MlirContext {
    context: Context,
}

#[cfg(feature = "mlir")]
impl MlirContext {
    /// Creates a new MLIR context with all dialects registered.
    ///
    /// This initializes the MLIR context and registers standard dialects
    /// including arith, func, scf, memref, and others needed for DOL compilation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = MlirContext::new();
    /// ```
    pub fn new() -> Self {
        let registry = DialectRegistry::new();
        register_all_dialects(&registry);

        let context = Context::new();
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();

        Self { context }
    }

    /// Returns a reference to the underlying melior Context.
    ///
    /// This provides access to the raw MLIR context for advanced operations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = MlirContext::new();
    /// let raw_ctx = ctx.context();
    /// ```
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Creates a new MLIR module with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the module (used for debugging)
    ///
    /// # Returns
    ///
    /// A new MLIR module instance
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = MlirContext::new();
    /// let module = ctx.create_module("my_module");
    /// ```
    pub fn create_module(&self, name: &str) -> Module {
        Module::new(self.unknown_location(name))
    }

    /// Creates an MLIR location from a DOL source span.
    ///
    /// Locations are used for error reporting and debugging, mapping
    /// MLIR operations back to their source code positions.
    ///
    /// # Arguments
    ///
    /// * `span` - The source code span
    ///
    /// # Returns
    ///
    /// An MLIR location representing this span
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let span = Span { start: 10, end: 20 };
    /// let loc = ctx.location_from_span(&span);
    /// ```
    pub fn location_from_span(&self, span: &Span) -> Location {
        // For now, create a simple file location
        // In a full implementation, we'd track filename and line/column info
        Location::unknown(&self.context)
    }

    /// Creates an unknown location with an optional identifier.
    ///
    /// Used when precise source location tracking is not available.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Optional identifier for debugging
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loc = ctx.unknown_location("temp_op");
    /// ```
    pub fn unknown_location(&self, _identifier: &str) -> Location {
        Location::unknown(&self.context)
    }
}

#[cfg(feature = "mlir")]
impl Default for MlirContext {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation when MLIR feature is disabled
// This allows the code to compile without the feature, but operations will fail at runtime
#[cfg(not(feature = "mlir"))]
pub struct MlirContext {
    _private: (),
}

#[cfg(not(feature = "mlir"))]
impl MlirContext {
    /// Creates a new MLIR context.
    ///
    /// Note: This is a stub implementation when the `mlir` feature is disabled.
    /// Any attempt to use this context will result in compilation errors.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[cfg(not(feature = "mlir"))]
impl Default for MlirContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "mlir"))]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let _ctx = MlirContext::new();
        // If we got here, context creation succeeded
    }

    #[test]
    fn test_module_creation() {
        let ctx = MlirContext::new();
        let _module = ctx.create_module("test_module");
        // If we got here, module creation succeeded
    }

    #[test]
    fn test_location_from_span() {
        let ctx = MlirContext::new();
        let span = Span { start: 10, end: 20 };
        let _loc = ctx.location_from_span(&span);
        // If we got here, location creation succeeded
    }

    #[test]
    fn test_default_context() {
        let _ctx = MlirContext::default();
        // If we got here, default context creation succeeded
    }
}
