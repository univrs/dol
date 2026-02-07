//! Hygienic macro expansion for DOL.
//!
//! This module provides hygiene support to prevent name collisions
//! and ensure that macros don't accidentally capture identifiers
//! from their expansion context.
//!
//! # Hygiene
//!
//! Hygienic macro expansion ensures that:
//! - Variables introduced by macros don't shadow variables at the call site
//! - Macros can't accidentally capture variables from the call site
//! - Each macro expansion has its own lexical scope
//!
//! # Example
//!
//! ```text
//! // Without hygiene:
//! macro swap!($a:ident, $b:ident) => {
//!     let temp = $a;   // 'temp' might shadow existing variable!
//!     $a = $b;
//!     $b = temp;
//! }
//!
//! // With hygiene:
//! // 'temp' gets a unique identifier that can't collide
//! ```

use metadol::ast::{Declaration, Expr, Span, Stmt};
use std::collections::HashMap;

/// Syntax context for hygiene tracking.
///
/// Each macro expansion gets a unique syntax context, which is used
/// to track the origin of identifiers and prevent unwanted capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyntaxContext {
    /// Unique expansion ID
    expansion_id: u64,
    /// Depth in macro expansion stack
    depth: usize,
}

impl SyntaxContext {
    /// Creates a new root syntax context.
    pub fn root() -> Self {
        Self {
            expansion_id: 0,
            depth: 0,
        }
    }

    /// Creates a new syntax context for a macro expansion.
    pub fn new_expansion(parent: Self, expansion_id: u64) -> Self {
        Self {
            expansion_id,
            depth: parent.depth + 1,
        }
    }

    /// Returns the expansion ID.
    pub fn expansion_id(&self) -> u64 {
        self.expansion_id
    }

    /// Returns the expansion depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns true if this is the root context.
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }
}

/// Hygiene context for macro expansion.
///
/// Tracks syntax contexts and provides facilities for hygienic
/// identifier generation and resolution.
pub struct HygieneContext {
    /// Next expansion ID
    next_expansion_id: u64,
    /// Next gensym counter
    next_gensym: u64,
    /// Current syntax context stack
    context_stack: Vec<SyntaxContext>,
    /// Mapping from original names to hygienic names
    name_map: HashMap<(String, SyntaxContext), String>,
}

impl HygieneContext {
    /// Creates a new hygiene context.
    pub fn new() -> Self {
        Self {
            next_expansion_id: 1,
            next_gensym: 0,
            context_stack: vec![SyntaxContext::root()],
            name_map: HashMap::new(),
        }
    }

    /// Returns the current syntax context.
    pub fn current_context(&self) -> SyntaxContext {
        *self.context_stack.last().unwrap()
    }

    /// Enters a new macro expansion.
    ///
    /// Returns the new syntax context.
    pub fn enter_expansion(&mut self) -> SyntaxContext {
        let expansion_id = self.next_expansion_id;
        self.next_expansion_id += 1;

        let parent = self.current_context();
        let new_context = SyntaxContext::new_expansion(parent, expansion_id);
        self.context_stack.push(new_context);
        new_context
    }

    /// Exits the current macro expansion.
    pub fn exit_expansion(&mut self) {
        if self.context_stack.len() > 1 {
            self.context_stack.pop();
        }
    }

    /// Generates a fresh, unique identifier.
    ///
    /// The generated identifier is guaranteed not to collide with
    /// any user-written identifier or other gensym'd identifier.
    ///
    /// # Example
    ///
    /// ```text
    /// gensym("temp") => "__temp_0"
    /// gensym("temp") => "__temp_1"
    /// ```
    pub fn gensym(&mut self, base: &str) -> String {
        let id = self.next_gensym;
        self.next_gensym += 1;
        format!("__{}_{}", base, id)
    }

    /// Makes an identifier hygienic by associating it with the current syntax context.
    ///
    /// If the identifier has already been made hygienic in this context,
    /// returns the existing hygienic name. Otherwise, generates a new one.
    pub fn make_hygienic(&mut self, name: &str) -> String {
        let ctx = self.current_context();
        let key = (name.to_string(), ctx);

        if let Some(hygienic_name) = self.name_map.get(&key) {
            hygienic_name.clone()
        } else {
            let hygienic_name = format!("{}_{}", name, ctx.expansion_id());
            self.name_map.insert(key, hygienic_name.clone());
            hygienic_name
        }
    }

    /// Applies hygiene to an expression.
    ///
    /// Traverses the expression tree and makes all identifiers hygienic
    /// according to their syntax context.
    pub fn apply_hygiene_to_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Ident(name) => {
                let hygienic_name = self.make_hygienic(name);
                Expr::Ident(hygienic_name)
            }

            Expr::Binary { op, left, right } => Expr::Binary {
                op: op.clone(),
                left: Box::new(self.apply_hygiene_to_expr(left)),
                right: Box::new(self.apply_hygiene_to_expr(right)),
            },

            Expr::Unary { op, operand } => Expr::Unary {
                op: op.clone(),
                operand: Box::new(self.apply_hygiene_to_expr(operand)),
            },

            Expr::Call { func, args } => Expr::Call {
                func: Box::new(self.apply_hygiene_to_expr(func)),
                args: args.iter().map(|a| self.apply_hygiene_to_expr(a)).collect(),
            },

            Expr::Index { base, index } => Expr::Index {
                base: Box::new(self.apply_hygiene_to_expr(base)),
                index: Box::new(self.apply_hygiene_to_expr(index)),
            },

            Expr::Field { base, field } => Expr::Field {
                base: Box::new(self.apply_hygiene_to_expr(base)),
                field: field.clone(),
            },

            Expr::Cast { expr, ty } => Expr::Cast {
                expr: Box::new(self.apply_hygiene_to_expr(expr)),
                ty: ty.clone(),
            },

            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => Expr::If {
                condition: Box::new(self.apply_hygiene_to_expr(condition)),
                then_branch: Box::new(self.apply_hygiene_to_block(then_branch)),
                else_branch: else_branch
                    .as_ref()
                    .map(|b| Box::new(self.apply_hygiene_to_block(b))),
            },

            Expr::Match { scrutinee, arms } => Expr::Match {
                scrutinee: Box::new(self.apply_hygiene_to_expr(scrutinee)),
                arms: arms.clone(), // TODO: Apply hygiene to match arms
            },

            Expr::Lambda { params, body } => Expr::Lambda {
                params: params.clone(), // TODO: Apply hygiene to params
                body: Box::new(self.apply_hygiene_to_expr(body)),
            },

            Expr::Array(elements) => Expr::Array(
                elements
                    .iter()
                    .map(|e| self.apply_hygiene_to_expr(e))
                    .collect(),
            ),

            Expr::Tuple(elements) => Expr::Tuple(
                elements
                    .iter()
                    .map(|e| self.apply_hygiene_to_expr(e))
                    .collect(),
            ),

            // Literals and other leaf nodes don't need hygiene
            _ => expr.clone(),
        }
    }

    /// Applies hygiene to a block.
    pub fn apply_hygiene_to_block(&mut self, block: &metadol::ast::Block) -> metadol::ast::Block {
        metadol::ast::Block {
            stmts: block
                .stmts
                .iter()
                .map(|s| self.apply_hygiene_to_stmt(s))
                .collect(),
            span: block.span,
        }
    }

    /// Applies hygiene to a statement.
    pub fn apply_hygiene_to_stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Let {
                name,
                ty,
                value,
                span,
            } => {
                let hygienic_name = self.make_hygienic(name);
                Stmt::Let {
                    name: hygienic_name,
                    ty: ty.clone(),
                    value: value.as_ref().map(|v| self.apply_hygiene_to_expr(v)),
                    span: *span,
                }
            }

            Stmt::Expr(expr) => Stmt::Expr(self.apply_hygiene_to_expr(expr)),

            Stmt::Return { value, span } => Stmt::Return {
                value: value.as_ref().map(|v| self.apply_hygiene_to_expr(v)),
                span: *span,
            },

            Stmt::Assign {
                target,
                value,
                span,
            } => Stmt::Assign {
                target: self.apply_hygiene_to_expr(target),
                value: self.apply_hygiene_to_expr(value),
                span: *span,
            },

            Stmt::While {
                condition,
                body,
                span,
            } => Stmt::While {
                condition: self.apply_hygiene_to_expr(condition),
                body: self.apply_hygiene_to_block(body),
                span: *span,
            },

            Stmt::For {
                var,
                iter,
                body,
                span,
            } => {
                let hygienic_var = self.make_hygienic(var);
                Stmt::For {
                    var: hygienic_var,
                    iter: self.apply_hygiene_to_expr(iter),
                    body: self.apply_hygiene_to_block(body),
                    span: *span,
                }
            }

            _ => stmt.clone(),
        }
    }

    /// Applies hygiene to a declaration.
    pub fn apply_hygiene_to_decl(&mut self, decl: &Declaration) -> Declaration {
        // Most declarations don't need hygiene at the top level
        // (their names are intentionally exported)
        // But we can apply hygiene to their bodies
        decl.clone()
    }

    /// Resets the hygiene context.
    pub fn reset(&mut self) {
        self.next_expansion_id = 1;
        self.next_gensym = 0;
        self.context_stack = vec![SyntaxContext::root()];
        self.name_map.clear();
    }
}

impl Default for HygieneContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadol::ast::Literal;

    #[test]
    fn test_syntax_context() {
        let root = SyntaxContext::root();
        assert!(root.is_root());
        assert_eq!(root.depth(), 0);

        let child = SyntaxContext::new_expansion(root, 1);
        assert!(!child.is_root());
        assert_eq!(child.depth(), 1);
        assert_eq!(child.expansion_id(), 1);
    }

    #[test]
    fn test_hygiene_context() {
        let mut ctx = HygieneContext::new();
        assert!(ctx.current_context().is_root());

        let expansion_ctx = ctx.enter_expansion();
        assert_eq!(expansion_ctx.depth(), 1);

        ctx.exit_expansion();
        assert!(ctx.current_context().is_root());
    }

    #[test]
    fn test_gensym() {
        let mut ctx = HygieneContext::new();

        let name1 = ctx.gensym("temp");
        let name2 = ctx.gensym("temp");

        assert_ne!(name1, name2);
        assert!(name1.starts_with("__temp_"));
        assert!(name2.starts_with("__temp_"));
    }

    #[test]
    fn test_make_hygienic() {
        let mut ctx = HygieneContext::new();

        ctx.enter_expansion();
        let name1 = ctx.make_hygienic("x");

        ctx.enter_expansion();
        let name2 = ctx.make_hygienic("x");

        // Same name in different contexts should get different hygienic names
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_apply_hygiene_to_expr() {
        let mut ctx = HygieneContext::new();
        ctx.enter_expansion();

        let expr = Expr::Ident("x".to_string());
        let hygienic = ctx.apply_hygiene_to_expr(&expr);

        if let Expr::Ident(name) = hygienic {
            assert_ne!(name, "x");
            assert!(name.contains("x"));
        } else {
            panic!("Expected identifier");
        }
    }

    #[test]
    fn test_apply_hygiene_to_binary() {
        let mut ctx = HygieneContext::new();
        ctx.enter_expansion();

        let expr = Expr::Binary {
            op: metadol::ast::BinaryOp::Add,
            left: Box::new(Expr::Ident("x".to_string())),
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        let hygienic = ctx.apply_hygiene_to_expr(&expr);

        if let Expr::Binary { left, .. } = hygienic {
            if let Expr::Ident(name) = *left {
                assert_ne!(name, "x");
            } else {
                panic!("Expected identifier");
            }
        } else {
            panic!("Expected binary expression");
        }
    }
}
