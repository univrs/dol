//! Macro expansion engine for Metal DOL 2.0.
//!
//! This module provides the macro expander that processes macro invocations
//! during parsing and transforms them into their expanded forms.
//!
//! # Expansion Process
//!
//! 1. **Detection**: Identify macro invocations in the token stream
//! 2. **Resolution**: Look up the macro in the registry
//! 3. **Input Preparation**: Collect and prepare macro arguments
//! 4. **Expansion**: Call the macro's `expand` method
//! 5. **Integration**: Insert expanded code into the AST
//!
//! # Example
//!
//! ```rust,ignore
//! use metadol::macros::{MacroExpander, MacroContext};
//!
//! let mut expander = MacroExpander::new();
//! expander.register_builtins();
//!
//! let ctx = MacroContext::new();
//! let expanded = expander.expand_expr(&expr, &ctx)?;
//! ```

use super::{
    AttributeArg, BuiltinMacros, Macro, MacroAttribute, MacroContext, MacroError, MacroInput,
    MacroInvocation, MacroOutput,
};
use crate::ast::{Block, Declaration, Expr, Literal, Span, Stmt};
use std::collections::HashMap;
use std::sync::Arc;

/// The macro expander processes and expands macro invocations.
///
/// The expander maintains a registry of available macros and handles
/// the expansion of both attribute macros and expression macros.
pub struct MacroExpander {
    /// Registered macros by name
    macros: HashMap<String, Arc<dyn Macro>>,

    /// Whether to enable recursive macro expansion
    recursive: bool,

    /// Maximum expansion depth (to prevent infinite recursion)
    max_depth: usize,
}

impl MacroExpander {
    /// Creates a new macro expander with no macros registered.
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            recursive: true,
            max_depth: 64,
        }
    }

    /// Creates a new macro expander with built-in macros registered.
    pub fn with_builtins() -> Self {
        let mut expander = Self::new();
        expander.register_builtins();
        expander
    }

    /// Registers all built-in macros.
    pub fn register_builtins(&mut self) {
        let builtins = BuiltinMacros::new();
        for name in builtins.names() {
            if let Some(m) = builtins.get(name) {
                self.macros.insert(name.to_string(), m);
            }
        }
    }

    /// Registers a custom macro.
    pub fn register(&mut self, macro_impl: Arc<dyn Macro>) {
        self.macros
            .insert(macro_impl.name().to_string(), macro_impl);
    }

    /// Looks up a macro by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Macro>> {
        self.macros.get(name).cloned()
    }

    /// Returns true if a macro with the given name is registered.
    pub fn has_macro(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Sets the maximum recursion depth.
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
    /// * `ctx` - The expansion context
    ///
    /// # Returns
    ///
    /// The expanded output, or an error if expansion fails.
    pub fn expand(
        &self,
        invocation: &MacroInvocation,
        ctx: &MacroContext,
    ) -> Result<MacroOutput, MacroError> {
        self.expand_with_depth(invocation, ctx, 0)
    }

    /// Expands a macro invocation with depth tracking.
    fn expand_with_depth(
        &self,
        invocation: &MacroInvocation,
        ctx: &MacroContext,
        depth: usize,
    ) -> Result<MacroOutput, MacroError> {
        if depth >= self.max_depth {
            return Err(MacroError::new(format!(
                "maximum macro expansion depth ({}) exceeded for #{}",
                self.max_depth, invocation.name
            )));
        }

        let macro_impl = self.macros.get(&invocation.name).ok_or_else(|| {
            MacroError::with_span(
                format!("undefined macro: #{}", invocation.name),
                invocation.span,
            )
        })?;

        // Prepare input from arguments
        let input = self.prepare_input(&invocation.args)?;

        // Validate input
        macro_impl.validate(&input)?;

        // Expand the macro
        let mut output = macro_impl.expand(input, ctx)?;

        // Recursively expand nested macros if enabled
        if self.recursive {
            output = self.expand_output_recursively(output, ctx, depth + 1)?;
        }

        Ok(output)
    }

    /// Expands an attribute macro on a declaration.
    ///
    /// # Arguments
    ///
    /// * `attribute` - The attribute macro to expand
    /// * `declaration` - The declaration being annotated
    /// * `ctx` - The expansion context
    ///
    /// # Returns
    ///
    /// The modified declaration, or an error if expansion fails.
    pub fn expand_attribute(
        &self,
        attribute: &MacroAttribute,
        declaration: Declaration,
        ctx: &MacroContext,
    ) -> Result<Declaration, MacroError> {
        let macro_impl = self.macros.get(&attribute.name).ok_or_else(|| {
            MacroError::with_span(
                format!("undefined attribute macro: #{}", attribute.name),
                attribute.span,
            )
        })?;

        if !macro_impl.is_attribute_macro() {
            return Err(MacroError::with_span(
                format!(
                    "macro #{} cannot be used as an attribute macro",
                    attribute.name
                ),
                attribute.span,
            ));
        }

        // Prepare input from attribute arguments
        let mut ident_list = Vec::new();
        for arg in &attribute.args {
            if let AttributeArg::Ident(name) = arg {
                ident_list.push(name.clone());
            }
        }

        let input = if ident_list.is_empty() {
            MacroInput::Declaration(Box::new(declaration.clone()))
        } else {
            MacroInput::IdentList(ident_list)
        };

        // Validate and expand
        macro_impl.validate(&input)?;
        let output = macro_impl.expand(input, ctx)?;

        // Extract the declaration from output
        match output {
            MacroOutput::Declaration(decl) => Ok(*decl),
            MacroOutput::None => Ok(declaration), // No transformation
            _ => Err(MacroError::new(
                "attribute macro must produce a declaration",
            )),
        }
    }

    /// Expands a macro expression inline.
    ///
    /// # Arguments
    ///
    /// * `name` - The macro name (without #)
    /// * `args` - The macro arguments
    /// * `span` - The source location
    /// * `ctx` - The expansion context
    ///
    /// # Returns
    ///
    /// The expanded expression, or an error if expansion fails.
    pub fn expand_expr(
        &self,
        name: &str,
        args: Vec<Expr>,
        span: Span,
        ctx: &MacroContext,
    ) -> Result<Expr, MacroError> {
        let invocation = MacroInvocation::new(name, args, span);
        let output = self.expand(&invocation, ctx)?;

        match output {
            MacroOutput::Expr(expr) => Ok(*expr),
            MacroOutput::None => Ok(Expr::Literal(Literal::Bool(true))), // Empty expansion
            MacroOutput::ExprList(exprs) if exprs.len() == 1 => {
                Ok(exprs.into_iter().next().unwrap())
            }
            _ => Err(MacroError::new(
                "expression macro must produce a single expression",
            )),
        }
    }

    /// Prepares macro input from expression arguments.
    fn prepare_input(&self, args: &[Expr]) -> Result<MacroInput, MacroError> {
        if args.is_empty() {
            return Ok(MacroInput::Empty);
        }

        if args.len() == 1 {
            // Single argument - could be ident or expr
            match &args[0] {
                Expr::Identifier(name) => Ok(MacroInput::Ident(name.clone())),
                expr => Ok(MacroInput::Expr(Box::new(expr.clone()))),
            }
        } else {
            // Multiple arguments - could be ident list or expr list
            let all_idents = args.iter().all(|a| matches!(a, Expr::Identifier(_)));

            if all_idents {
                let idents: Vec<String> = args
                    .iter()
                    .filter_map(|a| {
                        if let Expr::Identifier(name) = a {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(MacroInput::IdentList(idents))
            } else {
                Ok(MacroInput::ExprList(args.to_vec()))
            }
        }
    }

    /// Recursively expands macros in the output.
    fn expand_output_recursively(
        &self,
        output: MacroOutput,
        ctx: &MacroContext,
        depth: usize,
    ) -> Result<MacroOutput, MacroError> {
        match output {
            MacroOutput::Expr(expr) => {
                let expanded = self.expand_expr_recursively(*expr, ctx, depth)?;
                Ok(MacroOutput::Expr(Box::new(expanded)))
            }
            MacroOutput::ExprList(exprs) => {
                let expanded: Result<Vec<Expr>, MacroError> = exprs
                    .into_iter()
                    .map(|e| self.expand_expr_recursively(e, ctx, depth))
                    .collect();
                Ok(MacroOutput::ExprList(expanded?))
            }
            MacroOutput::Stmt(stmt) => {
                let expanded = self.expand_stmt_recursively(*stmt, ctx, depth)?;
                Ok(MacroOutput::Stmt(Box::new(expanded)))
            }
            MacroOutput::StmtList(stmts) => {
                let expanded: Result<Vec<Stmt>, MacroError> = stmts
                    .into_iter()
                    .map(|s| self.expand_stmt_recursively(s, ctx, depth))
                    .collect();
                Ok(MacroOutput::StmtList(expanded?))
            }
            // Pass through other output types
            other => Ok(other),
        }
    }

    /// Recursively expands macros in an expression.
    fn expand_expr_recursively(
        &self,
        expr: Expr,
        ctx: &MacroContext,
        depth: usize,
    ) -> Result<Expr, MacroError> {
        match expr {
            // Handle macro calls (represented as function calls to macro names)
            Expr::Call { callee, args } => {
                // Check if this is a macro call
                if let Expr::Identifier(name) = callee.as_ref() {
                    if self.has_macro(name) {
                        let invocation = MacroInvocation::new(name, args.clone(), Span::default());
                        let output = self.expand_with_depth(&invocation, ctx, depth)?;
                        return match output {
                            MacroOutput::Expr(e) => Ok(*e),
                            MacroOutput::None => Ok(Expr::Literal(Literal::Bool(true))),
                            _ => Err(MacroError::new("expected expression from macro")),
                        };
                    }
                }

                // Not a macro - recursively expand arguments
                let expanded_callee = self.expand_expr_recursively(*callee, ctx, depth)?;
                let expanded_args: Result<Vec<Expr>, MacroError> = args
                    .into_iter()
                    .map(|a| self.expand_expr_recursively(a, ctx, depth))
                    .collect();

                Ok(Expr::Call {
                    callee: Box::new(expanded_callee),
                    args: expanded_args?,
                })
            }

            Expr::Binary { left, op, right } => {
                let expanded_left = self.expand_expr_recursively(*left, ctx, depth)?;
                let expanded_right = self.expand_expr_recursively(*right, ctx, depth)?;
                Ok(Expr::Binary {
                    left: Box::new(expanded_left),
                    op,
                    right: Box::new(expanded_right),
                })
            }

            Expr::Unary { op, operand } => {
                let expanded = self.expand_expr_recursively(*operand, ctx, depth)?;
                Ok(Expr::Unary {
                    op,
                    operand: Box::new(expanded),
                })
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let expanded_cond = self.expand_expr_recursively(*condition, ctx, depth)?;
                let expanded_then = self.expand_expr_recursively(*then_branch, ctx, depth)?;
                let expanded_else = else_branch
                    .map(|e| self.expand_expr_recursively(*e, ctx, depth))
                    .transpose()?
                    .map(Box::new);

                Ok(Expr::If {
                    condition: Box::new(expanded_cond),
                    then_branch: Box::new(expanded_then),
                    else_branch: expanded_else,
                })
            }

            Expr::Lambda {
                params,
                return_type,
                body,
            } => {
                let expanded_body = self.expand_expr_recursively(*body, ctx, depth)?;
                Ok(Expr::Lambda {
                    params,
                    return_type,
                    body: Box::new(expanded_body),
                })
            }

            Expr::Block(Block {
                statements,
                final_expr,
                ..
            }) => {
                let expanded_stmts: Result<Vec<Stmt>, MacroError> = statements
                    .into_iter()
                    .map(|s| self.expand_stmt_recursively(s, ctx, depth))
                    .collect();
                let expanded_final = final_expr
                    .map(|e| self.expand_expr_recursively(*e, ctx, depth))
                    .transpose()?
                    .map(Box::new);

                Ok(Expr::Block(Block {
                    statements: expanded_stmts?,
                    final_expr: expanded_final,
                    span: Span::default(),
                }))
            }

            Expr::Quote(inner) => {
                // Don't expand inside quotes (by design)
                Ok(Expr::Quote(inner))
            }

            Expr::QuasiQuote(inner) => {
                // QuasiQuotes allow unquote expansion
                let expanded = self.expand_quasi_quote(*inner, ctx, depth)?;
                Ok(Expr::QuasiQuote(Box::new(expanded)))
            }

            Expr::Eval(inner) => {
                let expanded = self.expand_expr_recursively(*inner, ctx, depth)?;
                Ok(Expr::Eval(Box::new(expanded)))
            }

            Expr::Match { scrutinee, arms } => {
                let expanded_scrutinee = self.expand_expr_recursively(*scrutinee, ctx, depth)?;
                let expanded_arms: Result<Vec<_>, MacroError> = arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.body = Box::new(self.expand_expr_recursively(*arm.body, ctx, depth)?);
                        if let Some(guard) = arm.guard {
                            arm.guard =
                                Some(Box::new(self.expand_expr_recursively(*guard, ctx, depth)?));
                        }
                        Ok(arm)
                    })
                    .collect();

                Ok(Expr::Match {
                    scrutinee: Box::new(expanded_scrutinee),
                    arms: expanded_arms?,
                })
            }

            Expr::Member { object, field } => {
                let expanded = self.expand_expr_recursively(*object, ctx, depth)?;
                Ok(Expr::Member {
                    object: Box::new(expanded),
                    field,
                })
            }

            // Pass through leaf expressions
            other => Ok(other),
        }
    }

    /// Recursively expands macros in a statement.
    fn expand_stmt_recursively(
        &self,
        stmt: Stmt,
        ctx: &MacroContext,
        depth: usize,
    ) -> Result<Stmt, MacroError> {
        match stmt {
            Stmt::Let {
                name,
                type_ann,
                value,
            } => {
                let expanded = self.expand_expr_recursively(value, ctx, depth)?;
                Ok(Stmt::Let {
                    name,
                    type_ann,
                    value: expanded,
                })
            }

            Stmt::Assign { target, value } => {
                let expanded_target = self.expand_expr_recursively(target, ctx, depth)?;
                let expanded_value = self.expand_expr_recursively(value, ctx, depth)?;
                Ok(Stmt::Assign {
                    target: expanded_target,
                    value: expanded_value,
                })
            }

            Stmt::Expr(expr) => {
                let expanded = self.expand_expr_recursively(expr, ctx, depth)?;
                Ok(Stmt::Expr(expanded))
            }

            Stmt::Return(opt_expr) => {
                let expanded = opt_expr
                    .map(|e| self.expand_expr_recursively(e, ctx, depth))
                    .transpose()?;
                Ok(Stmt::Return(expanded))
            }

            Stmt::For {
                binding,
                iterable,
                body,
            } => {
                let expanded_iter = self.expand_expr_recursively(iterable, ctx, depth)?;
                let expanded_body: Result<Vec<Stmt>, MacroError> = body
                    .into_iter()
                    .map(|s| self.expand_stmt_recursively(s, ctx, depth))
                    .collect();
                Ok(Stmt::For {
                    binding,
                    iterable: expanded_iter,
                    body: expanded_body?,
                })
            }

            Stmt::While { condition, body } => {
                let expanded_cond = self.expand_expr_recursively(condition, ctx, depth)?;
                let expanded_body: Result<Vec<Stmt>, MacroError> = body
                    .into_iter()
                    .map(|s| self.expand_stmt_recursively(s, ctx, depth))
                    .collect();
                Ok(Stmt::While {
                    condition: expanded_cond,
                    body: expanded_body?,
                })
            }

            Stmt::Loop { body } => {
                let expanded_body: Result<Vec<Stmt>, MacroError> = body
                    .into_iter()
                    .map(|s| self.expand_stmt_recursively(s, ctx, depth))
                    .collect();
                Ok(Stmt::Loop {
                    body: expanded_body?,
                })
            }

            // Pass through control flow
            other => Ok(other),
        }
    }

    /// Expands unquotes inside a quasi-quote.
    fn expand_quasi_quote(
        &self,
        expr: Expr,
        ctx: &MacroContext,
        depth: usize,
    ) -> Result<Expr, MacroError> {
        match expr {
            Expr::Unquote(inner) => {
                // Expand the unquoted expression
                self.expand_expr_recursively(*inner, ctx, depth)
            }

            Expr::Binary { left, op, right } => {
                let expanded_left = self.expand_quasi_quote(*left, ctx, depth)?;
                let expanded_right = self.expand_quasi_quote(*right, ctx, depth)?;
                Ok(Expr::Binary {
                    left: Box::new(expanded_left),
                    op,
                    right: Box::new(expanded_right),
                })
            }

            Expr::Call { callee, args } => {
                let expanded_callee = self.expand_quasi_quote(*callee, ctx, depth)?;
                let expanded_args: Result<Vec<Expr>, MacroError> = args
                    .into_iter()
                    .map(|a| self.expand_quasi_quote(a, ctx, depth))
                    .collect();
                Ok(Expr::Call {
                    callee: Box::new(expanded_callee),
                    args: expanded_args?,
                })
            }

            // Recursively handle other expression types
            other => Ok(other),
        }
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::with_builtins()
    }
}

/// Result of expanding all macros in a compilation unit.
pub struct ExpansionResult {
    /// Expanded declarations
    pub declarations: Vec<Declaration>,
    /// Any warnings generated during expansion
    pub warnings: Vec<String>,
    /// Expansion statistics
    pub stats: ExpansionStats,
}

/// Statistics about macro expansion.
#[derive(Debug, Default)]
pub struct ExpansionStats {
    /// Number of macros expanded
    pub macros_expanded: usize,
    /// Maximum expansion depth reached
    pub max_depth_reached: usize,
    /// Total time spent expanding (in microseconds)
    pub expansion_time_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_expander_creation() {
        let expander = MacroExpander::new();
        assert!(!expander.has_macro("stringify"));

        let expander = MacroExpander::with_builtins();
        assert!(expander.has_macro("stringify"));
        assert!(expander.has_macro("concat"));
        assert!(expander.has_macro("env"));
    }

    #[test]
    fn test_expand_stringify() {
        let expander = MacroExpander::with_builtins();
        let ctx = MacroContext::new();

        let result = expander
            .expand_expr(
                "stringify",
                vec![Expr::Identifier("foo.bar".to_string())],
                Span::default(),
                &ctx,
            )
            .unwrap();

        match result {
            Expr::Literal(Literal::String(s)) => {
                assert_eq!(s, "foo.bar");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_expand_concat() {
        let expander = MacroExpander::with_builtins();
        let ctx = MacroContext::new();

        let result = expander
            .expand_expr(
                "concat",
                vec![
                    Expr::Literal(Literal::String("hello".to_string())),
                    Expr::Literal(Literal::String("world".to_string())),
                ],
                Span::default(),
                &ctx,
            )
            .unwrap();

        match result {
            Expr::Literal(Literal::String(s)) => {
                assert_eq!(s, "helloworld");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_expand_undefined_macro() {
        let expander = MacroExpander::with_builtins();
        let ctx = MacroContext::new();

        let result = expander.expand_expr("nonexistent", vec![], Span::default(), &ctx);

        assert!(result.is_err());
    }

    #[test]
    fn test_expand_with_context() {
        let expander = MacroExpander::with_builtins();
        let ctx = MacroContext::with_location(Some("test.dol".to_string()), 42, 10);

        let result = expander
            .expand_expr("line", vec![], Span::default(), &ctx)
            .unwrap();

        match result {
            Expr::Literal(Literal::Int(n)) => {
                assert_eq!(n, 42);
            }
            _ => panic!("Expected int literal"),
        }
    }

    #[test]
    fn test_macro_invocation() {
        let invoc = MacroInvocation::new(
            "test",
            vec![Expr::Literal(Literal::Int(1))],
            Span::default(),
        );
        assert_eq!(invoc.name, "test");
        assert_eq!(invoc.args.len(), 1);
    }

    #[test]
    fn test_max_depth_limit() {
        let mut expander = MacroExpander::new();
        expander.set_max_depth(2);

        // This would normally cause infinite recursion
        // but depth limit should catch it
    }

    #[test]
    fn test_prepare_input_empty() {
        let expander = MacroExpander::new();
        let input = expander.prepare_input(&[]).unwrap();
        assert!(matches!(input, MacroInput::Empty));
    }

    #[test]
    fn test_prepare_input_single_ident() {
        let expander = MacroExpander::new();
        let args = vec![Expr::Identifier("test".to_string())];
        let input = expander.prepare_input(&args).unwrap();
        assert!(matches!(input, MacroInput::Ident(ref s) if s == "test"));
    }

    #[test]
    fn test_prepare_input_expr_list() {
        let expander = MacroExpander::new();
        let args = vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
        ];
        let input = expander.prepare_input(&args).unwrap();
        assert!(matches!(input, MacroInput::ExprList(_)));
    }
}
