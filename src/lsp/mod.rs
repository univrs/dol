//! Language Server Protocol (LSP) support for DOL.
//!
//! This module provides LSP server implementation for DOL, enabling
//! rich IDE support including:
//! - Intelligent code completion
//! - Real-time diagnostics
//! - Hover information
//! - Go-to-definition
//! - CRDT strategy suggestions
//!
//! # Overview
//!
//! The LSP server integrates with editors like VS Code, Vim, and Emacs
//! to provide real-time assistance when writing DOL schemas.
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::lsp::{DolLspServer, CompletionProvider};
//!
//! let server = DolLspServer::new();
//! let completions = server.provide_completions("gen document.schema { document has ", 35);
//! ```

pub mod completion;

pub use completion::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionProvider,
    CrdtStrategyCompletion, FieldTypeCompletion,
};

/// LSP server for DOL.
pub struct DolLspServer {
    completion_provider: CompletionProvider,
}

impl DolLspServer {
    /// Creates a new DOL LSP server.
    pub fn new() -> Self {
        Self {
            completion_provider: CompletionProvider::new(),
        }
    }

    /// Provides completions at a given position.
    pub fn provide_completions(&self, source: &str, position: usize) -> Vec<CompletionItem> {
        self.completion_provider
            .provide_completions(source, position)
    }

    /// Provides hover information at a given position.
    pub fn provide_hover(&self, _source: &str, _position: usize) -> Option<String> {
        // TODO: Implement hover provider
        None
    }

    /// Provides diagnostics for the source.
    pub fn provide_diagnostics(&self, _source: &str) -> Vec<Diagnostic> {
        // TODO: Implement diagnostics provider
        vec![]
    }
}

impl Default for DolLspServer {
    fn default() -> Self {
        Self::new()
    }
}

/// LSP diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Range in the source
    pub range: (usize, usize),
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Diagnostic message
    pub message: String,
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error
    Error,
    /// Warning
    Warning,
    /// Information
    Information,
    /// Hint
    Hint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_server_creation() {
        let server = DolLspServer::new();
        let completions = server.provide_completions("gen ", 4);
        assert!(!completions.is_empty());
    }
}
