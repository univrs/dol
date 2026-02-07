//! Macro expansion engine for DOL.
//!
//! This module provides the macro expander that processes macro invocations
//! and expands them using registered macro definitions.

use crate::declarative::DeclarativeMacro;
use crate::error::{MacroError, MacroResult};
use crate::hygiene::HygieneContext;
use crate::registry::MacroRegistry;
use metadol::ast::{Declaration, Expr, Span, Stmt};

/// Maximum recursion depth for macro expansion.
const MAX_EXPANSION_DEPTH: usize = 128;

/// Macro invocation to be expanded.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroInvocation {
    /// Macro name
    pub name: String,
    /// Arguments to the macro
    pub args: Vec<Expr>,
    /// Source location
    pub span: Span,
}

impl MacroInvocation {
    /// Creates a new macro invocation.
    pub fn new(name: impl Into<String>, args: Vec<Expr>, span: Span) -> Self {
        Self {
            name: name.into(),
            args,
            span,
        }
    }

    /// Creates a macro invocation with no arguments.
    pub fn simple(name: impl Into<String>, span: Span) -> Self {
        Self::new(name, Vec::new(), span)
    }
}

/// Macro expansion engine.
///
/// The expander maintains a registry of macros and handles the expansion
/// process, including recursion tracking and hygiene management.
pub struct MacroExpander {
    /// Macro registry
    registry: MacroRegistry,
    /// Hygiene context
    hygiene: HygieneContext,
    /// Current expansion depth
    depth: usize,
    /// Maximum expansion depth
    max_depth: usize,
    /// Whether to enable recursive expansion
    recursive: bool,
}

impl MacroExpander {
    /// Creates a new macro expander with an empty registry.
    pub fn new() -> Self {
        Self {
            registry: MacroRegistry::new(),
            hygiene: HygieneContext::new(),
            depth: 0,
            max_depth: MAX_EXPANSION_DEPTH,
            recursive: true,
        }
    }

    /// Creates a new macro expander with the given registry.
    pub fn with_registry(registry: MacroRegistry) -> Self {
        Self {
            registry,
            hygiene: HygieneContext::new(),
            depth: 0,
            max_depth: MAX_EXPANSION_DEPTH,
            recursive: true,
        }
    }

    /// Returns a reference to the macro registry.
    pub fn registry(&self) -> &MacroRegistry {
        &self.registry
    }

    /// Returns a mutable reference to the macro registry.
    pub fn registry_mut(&mut self) -> &mut MacroRegistry {
        &mut self.registry
    }

    /// Sets the maximum expansion depth.
    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    /// Enables or disables recursive expansion.
    pub fn set_recursive(&mut self, recursive: bool) {
        self.recursive = recursive;
    }

    /// Expands a macro invocation.
    ///
    /// # Arguments
    ///
    /// * `invocation` - The macro invocation to expand
    ///
    /// # Returns
    ///
    /// The expanded expressions on success, or an error if expansion fails.
    pub fn expand(&mut self, invocation: &MacroInvocation) -> MacroResult<Vec<Expr>> {
        if self.depth >= self.max_depth {
            return Err(MacroError::recursion_limit(self.max_depth));
        }

        self.depth += 1;
        let result = self.expand_impl(invocation);
        self.depth -= 1;

        result
    }

    fn expand_impl(&mut self, invocation: &MacroInvocation) -> MacroResult<Vec<Expr>> {
        // Look up the macro in the registry
        let macro_def = self
            .registry
            .get_declarative(&invocation.name)
            .ok_or_else(|| MacroError::undefined(&invocation.name))?;

        // Clone the macro to avoid borrow checker issues
        let macro_def = macro_def.clone();

        // Expand using the declarative macro
        let expanded = macro_def.expand(&invocation.args, &mut self.hygiene)?;

        // Recursively expand nested macros if enabled
        if self.recursive {
            self.expand_recursive(expanded)
        } else {
            Ok(expanded)
        }
    }

    /// Recursively expands nested macro invocations.
    fn expand_recursive(&mut self, exprs: Vec<Expr>) -> MacroResult<Vec<Expr>> {
        let mut result = Vec::new();

        for expr in exprs {
            let expanded = self.expand_expr_recursive(&expr)?;
            result.push(expanded);
        }

        Ok(result)
    }

    /// Recursively expands a single expression.
    fn expand_expr_recursive(&mut self, expr: &Expr) -> MacroResult<Expr> {
        match expr {
            // Check if this is a macro invocation
            Expr::Call { func, args } => {
                if let Expr::Ident(name) = &**func {
                    // Check if this is a macro call
                    if name.ends_with('!') || self.registry.has_declarative(name) {
                        let macro_name = name.trim_end_matches('!');
                        let invocation =
                            MacroInvocation::new(macro_name, args.clone(), Span::default());
                        let expanded = self.expand(&invocation)?;

                        // Return the first expanded expression
                        if let Some(first) = expanded.into_iter().next() {
                            return Ok(first);
                        }
                    }
                }

                // Not a macro, recursively expand arguments
                let expanded_args: MacroResult<Vec<Expr>> = args
                    .iter()
                    .map(|a| self.expand_expr_recursive(a))
                    .collect();

                Ok(Expr::Call {
                    func: Box::new(self.expand_expr_recursive(func)?),
                    args: expanded_args?,
                })
            }

            Expr::Binary { op, left, right } => Ok(Expr::Binary {
                op: op.clone(),
                left: Box::new(self.expand_expr_recursive(left)?),
                right: Box::new(self.expand_expr_recursive(right)?),
            }),

            Expr::Unary { op, operand } => Ok(Expr::Unary {
                op: op.clone(),
                operand: Box::new(self.expand_expr_recursive(operand)?),
            }),

            Expr::Index { base, index } => Ok(Expr::Index {
                base: Box::new(self.expand_expr_recursive(base)?),
                index: Box::new(self.expand_expr_recursive(index)?),
            }),

            Expr::Field { base, field } => Ok(Expr::Field {
                base: Box::new(self.expand_expr_recursive(base)?),
                field: field.clone(),
            }),

            Expr::Cast { expr, ty } => Ok(Expr::Cast {
                expr: Box::new(self.expand_expr_recursive(expr)?),
                ty: ty.clone(),
            }),

            Expr::Array(elements) => {
                let expanded: MacroResult<Vec<Expr>> = elements
                    .iter()
                    .map(|e| self.expand_expr_recursive(e))
                    .collect();
                Ok(Expr::Array(expanded?))
            }

            Expr::Tuple(elements) => {
                let expanded: MacroResult<Vec<Expr>> = elements
                    .iter()
                    .map(|e| self.expand_expr_recursive(e))
                    .collect();
                Ok(Expr::Tuple(expanded?))
            }

            // Leaf nodes don't need expansion
            _ => Ok(expr.clone()),
        }
    }

    /// Expands macros in a statement.
    pub fn expand_stmt(&mut self, stmt: &Stmt) -> MacroResult<Stmt> {
        match stmt {
            Stmt::Let {
                name,
                ty,
                value,
                span,
            } => {
                let expanded_value = if let Some(v) = value {
                    Some(self.expand_expr_recursive(v)?)
                } else {
                    None
                };

                Ok(Stmt::Let {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: expanded_value,
                    span: *span,
                })
            }

            Stmt::Expr(expr) => {
                let expanded = self.expand_expr_recursive(expr)?;
                Ok(Stmt::Expr(expanded))
            }

            Stmt::Return { value, span } => {
                let expanded_value = if let Some(v) = value {
                    Some(self.expand_expr_recursive(v)?)
                } else {
                    None
                };

                Ok(Stmt::Return {
                    value: expanded_value,
                    span: *span,
                })
            }

            Stmt::Assign {
                target,
                value,
                span,
            } => Ok(Stmt::Assign {
                target: self.expand_expr_recursive(target)?,
                value: self.expand_expr_recursive(value)?,
                span: *span,
            }),

            _ => Ok(stmt.clone()),
        }
    }

    /// Expands macros in a declaration.
    pub fn expand_decl(&mut self, decl: &Declaration) -> MacroResult<Declaration> {
        // For now, just return the declaration unchanged
        // More sophisticated expansion would recurse into the declaration's body
        Ok(decl.clone())
    }

    /// Resets the expander state.
    pub fn reset(&mut self) {
        self.depth = 0;
        self.hygiene.reset();
    }

    /// Returns the current expansion depth.
    pub fn depth(&self) -> usize {
        self.depth
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::MacroRule;
    use crate::declarative::MacroTemplate;
    use crate::pattern::MacroPattern;
    use metadol::ast::Literal;

    #[test]
    fn test_macro_invocation() {
        let invoc = MacroInvocation::simple("test", Span::default());
        assert_eq!(invoc.name, "test");
        assert!(invoc.args.is_empty());
    }

    #[test]
    fn test_expander_creation() {
        let expander = MacroExpander::new();
        assert_eq!(expander.depth(), 0);
        assert!(expander.registry().is_empty());
    }

    #[test]
    fn test_simple_expansion() {
        let mut expander = MacroExpander::new();

        // Register a simple macro: test!() => 42
        let pattern = MacroPattern::Empty;
        let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
        let rule = MacroRule::new(pattern, template);
        let macro_def = DeclarativeMacro::new("test", vec![rule]);

        expander.registry_mut().register_declarative("test", macro_def);

        // Expand the macro
        let invocation = MacroInvocation::simple("test", Span::default());
        let result = expander.expand(&invocation);

        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);

        if let Expr::Literal(Literal::Int(n)) = exprs[0] {
            assert_eq!(n, 42);
        } else {
            panic!("Expected integer literal");
        }
    }

    #[test]
    fn test_undefined_macro() {
        let mut expander = MacroExpander::new();
        let invocation = MacroInvocation::simple("undefined", Span::default());
        let result = expander.expand(&invocation);

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err.kind, crate::error::MacroErrorKind::UndefinedMacro));
        }
    }

    #[test]
    fn test_recursion_limit() {
        let mut expander = MacroExpander::new();
        expander.set_max_depth(2);

        // This would create infinite recursion, but we should hit the limit
        expander.depth = 3;
        let invocation = MacroInvocation::simple("test", Span::default());
        let result = expander.expand(&invocation);

        assert!(result.is_err());
    }

    #[test]
    fn test_expand_stmt() {
        let mut expander = MacroExpander::new();

        let stmt = Stmt::Expr(Expr::Literal(Literal::Int(42)));
        let result = expander.expand_stmt(&stmt);

        assert!(result.is_ok());
    }
}
