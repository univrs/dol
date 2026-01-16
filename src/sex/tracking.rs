//! Effect tracking and propagation
//!
//! This module tracks side effects through the DOL AST, allowing the linter
//! to detect purity violations and enforce sex context rules.

use crate::ast::{Declaration, Expr, Gene, Purity, Span, Statement, Stmt, Trait};
use std::collections::{HashMap, HashSet};

/// The kind of side effect being tracked.
///
/// Different kinds of effects may have different severity levels
/// and linting rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectKind {
    /// I/O operations (file, network, console)
    Io,
    /// Foreign Function Interface calls
    Ffi,
    /// Mutable global state access
    MutableGlobal,
    /// Non-deterministic operations (random, time)
    NonDeterministic,
    /// Unsafe operations
    Unsafe,
    /// General side effect (marked with 'sex' keyword)
    General,
}

impl EffectKind {
    /// Returns a human-readable description of this effect kind.
    pub fn description(&self) -> &'static str {
        match self {
            EffectKind::Io => "I/O operation",
            EffectKind::Ffi => "FFI call",
            EffectKind::MutableGlobal => "mutable global state",
            EffectKind::NonDeterministic => "non-deterministic operation",
            EffectKind::Unsafe => "unsafe operation",
            EffectKind::General => "side effect",
        }
    }
}

impl std::fmt::Display for EffectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A tracked side effect occurrence.
///
/// Records where an effect occurs in the source code and what kind of effect it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    /// The kind of effect
    pub kind: EffectKind,
    /// Location in source code
    pub span: Span,
    /// Optional description or context
    pub context: Option<String>,
}

impl Effect {
    /// Create a new effect.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of effect
    /// * `span` - Source location
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::sex::tracking::{Effect, EffectKind};
    /// use metadol::ast::Span;
    ///
    /// let effect = Effect::new(EffectKind::Io, Span::default());
    /// assert_eq!(effect.kind, EffectKind::Io);
    /// ```
    pub fn new(kind: EffectKind, span: Span) -> Self {
        Self {
            kind,
            span,
            context: None,
        }
    }

    /// Create a new effect with context.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of effect
    /// * `span` - Source location
    /// * `context` - Additional context or description
    pub fn with_context(kind: EffectKind, span: Span, context: String) -> Self {
        Self {
            kind,
            span,
            context: Some(context),
        }
    }
}

/// Tracks side effects in DOL code.
///
/// The effect tracker analyzes DOL declarations and expressions to identify
/// all side effects, building a map of which declarations and functions
/// perform which effects.
///
/// # Example
///
/// ```rust
/// use metadol::sex::tracking::EffectTracker;
/// use metadol::ast::{Declaration, Gene, Span, Visibility};
///
/// let mut tracker = EffectTracker::new();
///
/// let gene = Gene {
///     name: "test.gene".to_string(),
///     extends: None,
///     statements: vec![],
///     exegesis: "Test".to_string(),
///     span: Span::default(),
///     visibility: Visibility::default(),
/// };
///
/// tracker.track_declaration(&Declaration::Gene(gene));
/// ```
#[derive(Debug, Default)]
pub struct EffectTracker {
    /// Effects found in each declaration (by name)
    declaration_effects: HashMap<String, Vec<Effect>>,
    /// Set of declarations that perform effects
    effectful_declarations: HashSet<String>,
    /// Purity annotations for declarations
    purity_map: HashMap<String, Purity>,
}

impl EffectTracker {
    /// Create a new effect tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Track effects in a declaration.
    ///
    /// Analyzes the declaration and records any side effects found.
    ///
    /// # Arguments
    ///
    /// * `decl` - The declaration to analyze
    pub fn track_declaration(&mut self, decl: &Declaration) {
        let name = decl.name().to_string();
        let mut effects = Vec::new();

        match decl {
            Declaration::Gene(gene) => {
                self.track_gene(gene, &mut effects);
            }
            Declaration::Trait(trait_decl) => {
                self.track_trait(trait_decl, &mut effects);
            }
            _ => {}
        }

        if !effects.is_empty() {
            self.effectful_declarations.insert(name.clone());
            self.declaration_effects.insert(name, effects);
        }
    }

    /// Track effects in a gene.
    fn track_gene(&mut self, gene: &Gene, effects: &mut Vec<Effect>) {
        for statement in &gene.statements {
            self.track_statement(statement, effects);
        }
    }

    /// Track effects in a trait.
    fn track_trait(&mut self, trait_decl: &Trait, effects: &mut Vec<Effect>) {
        for statement in &trait_decl.statements {
            self.track_statement(statement, effects);
        }
    }

    /// Track effects in a statement.
    ///
    /// Identifies common patterns that indicate side effects.
    fn track_statement(&mut self, statement: &Statement, effects: &mut Vec<Effect>) {
        if let Statement::Has { property, span, .. } = statement {
            // Check for I/O-related properties
            if self.is_io_property(property) {
                effects.push(Effect::with_context(
                    EffectKind::Io,
                    *span,
                    format!("property '{}'", property),
                ));
            }
            // Check for FFI-related properties
            if self.is_ffi_property(property) {
                effects.push(Effect::with_context(
                    EffectKind::Ffi,
                    *span,
                    format!("property '{}'", property),
                ));
            }
            // Check for mutable global properties
            if self.is_global_property(property) {
                effects.push(Effect::with_context(
                    EffectKind::MutableGlobal,
                    *span,
                    format!("property '{}'", property),
                ));
            }
        }
    }

    /// Check if a property name suggests I/O operations.
    fn is_io_property(&self, property: &str) -> bool {
        let io_keywords = [
            "input", "output", "read", "write", "file", "network", "socket", "stream", "console",
            "stdout", "stderr", "stdin",
        ];
        io_keywords
            .iter()
            .any(|keyword| property.to_lowercase().contains(keyword))
    }

    /// Check if a property name suggests FFI operations.
    fn is_ffi_property(&self, property: &str) -> bool {
        let ffi_keywords = ["ffi", "extern", "native", "syscall", "foreign"];
        ffi_keywords
            .iter()
            .any(|keyword| property.to_lowercase().contains(keyword))
    }

    /// Check if a property name suggests global state.
    fn is_global_property(&self, property: &str) -> bool {
        let global_keywords = ["global", "static", "singleton", "shared"];
        global_keywords
            .iter()
            .any(|keyword| property.to_lowercase().contains(keyword))
    }

    /// Track effects in an expression.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to analyze
    /// * `effects` - Vector to accumulate effects into
    pub fn track_expr(&mut self, expr: &Expr, effects: &mut Vec<Effect>) {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.track_expr(left, effects);
                self.track_expr(right, effects);
            }
            Expr::Unary { operand, .. } => {
                self.track_expr(operand, effects);
            }
            Expr::Call { callee, args, .. } => {
                self.track_expr(callee, effects);
                for arg in args {
                    self.track_expr(arg, effects);
                }
            }
            Expr::Lambda { body, .. } => {
                self.track_expr(body, effects);
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.track_expr(condition, effects);
                self.track_expr(then_branch, effects);
                if let Some(else_expr) = else_branch {
                    self.track_expr(else_expr, effects);
                }
            }
            Expr::Block {
                statements,
                final_expr,
                ..
            } => {
                for stmt in statements {
                    self.track_stmt(stmt, effects);
                }
                if let Some(expr) = final_expr {
                    self.track_expr(expr, effects);
                }
            }
            _ => {}
        }
    }

    /// Track effects in a statement.
    fn track_stmt(&mut self, stmt: &Stmt, effects: &mut Vec<Effect>) {
        match stmt {
            Stmt::Let { value, .. } => {
                self.track_expr(value, effects);
            }
            Stmt::Assign { target, value } => {
                self.track_expr(target, effects);
                self.track_expr(value, effects);
            }
            Stmt::For { iterable, body, .. } => {
                self.track_expr(iterable, effects);
                for s in body {
                    self.track_stmt(s, effects);
                }
            }
            Stmt::While { condition, body } => {
                self.track_expr(condition, effects);
                for s in body {
                    self.track_stmt(s, effects);
                }
            }
            Stmt::Loop { body } => {
                for s in body {
                    self.track_stmt(s, effects);
                }
            }
            Stmt::Return(Some(expr)) => {
                self.track_expr(expr, effects);
            }
            Stmt::Expr(expr) => {
                self.track_expr(expr, effects);
            }
            _ => {}
        }
    }

    /// Check if a declaration has effects.
    ///
    /// # Arguments
    ///
    /// * `name` - The declaration name
    ///
    /// # Returns
    ///
    /// `true` if the declaration has recorded effects
    pub fn has_effects(&self, name: &str) -> bool {
        self.effectful_declarations.contains(name)
    }

    /// Get the effects for a declaration.
    ///
    /// # Arguments
    ///
    /// * `name` - The declaration name
    ///
    /// # Returns
    ///
    /// A slice of effects, or an empty slice if none were found
    pub fn get_effects(&self, name: &str) -> &[Effect] {
        self.declaration_effects
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Set the purity annotation for a declaration.
    ///
    /// # Arguments
    ///
    /// * `name` - The declaration name
    /// * `purity` - The purity level
    pub fn set_purity(&mut self, name: String, purity: Purity) {
        self.purity_map.insert(name, purity);
    }

    /// Get the purity annotation for a declaration.
    ///
    /// # Arguments
    ///
    /// * `name` - The declaration name
    ///
    /// # Returns
    ///
    /// The purity level, or `None` if not annotated
    pub fn get_purity(&self, name: &str) -> Option<Purity> {
        self.purity_map.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Visibility;

    #[test]
    fn test_effect_kind_display() {
        assert_eq!(EffectKind::Io.to_string(), "I/O operation");
        assert_eq!(EffectKind::Ffi.to_string(), "FFI call");
    }

    #[test]
    fn test_effect_creation() {
        let effect = Effect::new(EffectKind::Io, Span::default());
        assert_eq!(effect.kind, EffectKind::Io);
        assert!(effect.context.is_none());

        let effect_with_ctx =
            Effect::with_context(EffectKind::Ffi, Span::default(), "test".to_string());
        assert_eq!(effect_with_ctx.context, Some("test".to_string()));
    }

    #[test]
    fn test_effect_tracker_io_detection() {
        let tracker = EffectTracker::new();
        assert!(tracker.is_io_property("file_read"));
        assert!(tracker.is_io_property("network_socket"));
        assert!(!tracker.is_io_property("counter"));
    }

    #[test]
    fn test_effect_tracker_ffi_detection() {
        let tracker = EffectTracker::new();
        assert!(tracker.is_ffi_property("ffi_call"));
        assert!(tracker.is_ffi_property("extern_func"));
        assert!(!tracker.is_ffi_property("normal_func"));
    }

    #[test]
    fn test_effect_tracker_global_detection() {
        let tracker = EffectTracker::new();
        assert!(tracker.is_global_property("global_state"));
        assert!(tracker.is_global_property("static_var"));
        assert!(!tracker.is_global_property("local_var"));
    }

    #[test]
    fn test_effect_tracker_gene_tracking() {
        let mut tracker = EffectTracker::new();

        let gene = Gene {
            visibility: Visibility::default(),
            name: "io.gene".to_string(),
            extends: None,
            statements: vec![Statement::Has {
                subject: "io".to_string(),
                property: "file_read".to_string(),
                span: Span::default(),
            }],
            exegesis: "Test".to_string(),
            span: Span::default(),
        };

        tracker.track_declaration(&Declaration::Gene(gene));
        assert!(tracker.has_effects("io.gene"));

        let effects = tracker.get_effects("io.gene");
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].kind, EffectKind::Io);
    }
}
