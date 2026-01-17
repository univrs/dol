//! Idiom bracket desugaring pass.
//!
//! This pass transforms idiom brackets `[| f a b |]` into applicative functor
//! operations using `<$>` (Map) and `<*>` (Ap).
//!
//! # Desugaring Rules
//!
//! - `[| f |]` → `f` (no args, just return the function)
//! - `[| f a |]` → `f <$> a` (single arg, use Map)
//! - `[| f a b |]` → `(f <$> a) <*> b` (two args)
//! - `[| f a b c |]` → `((f <$> a) <*> b) <*> c` (three args, left associative)
//!
//! Generally: fmap the function over the first argument, then apply (`<*>`)
//! each subsequent argument left-to-right.
//!
//! # Example
//!
//! ```text
//! [| add x y |]
//! ```
//!
//! Desugars to:
//!
//! ```text
//! (add <$> x) <*> y
//! ```

use crate::ast::{BinaryOp, Block, Declaration, Expr, Span};
use crate::transform::{Pass, PassResult};

/// Desugaring pass for idiom brackets.
///
/// Transforms idiom bracket expressions into explicit applicative functor
/// operations using the Map (`<$>`) and Ap (`<*>`) operators.
pub struct IdiomDesugar;

impl IdiomDesugar {
    /// Creates a new idiom bracket desugaring pass.
    pub fn new() -> Self {
        Self
    }

    /// Desugar an expression, transforming idiom brackets recursively.
    ///
    /// This method traverses the entire expression tree and transforms
    /// any `IdiomBracket` nodes into `Binary` nodes using `Map` and `Ap`.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to desugar
    ///
    /// # Returns
    ///
    /// The desugared expression with all idiom brackets expanded.
    #[allow(dead_code)]
    #[allow(clippy::only_used_in_recursion)]
    pub fn desugar_expr(&self, expr: Expr) -> Expr {
        match expr {
            Expr::IdiomBracket { func, args } => {
                // First, recursively desugar the function and arguments
                let func = self.desugar_expr(*func);
                let args: Vec<Expr> = args.into_iter().map(|a| self.desugar_expr(a)).collect();

                // Apply desugaring rules based on number of arguments
                match args.len() {
                    // [| f |] → f
                    0 => func,

                    // [| f a |] → f <$> a
                    1 => Expr::Binary {
                        left: Box::new(func),
                        op: BinaryOp::Map,
                        right: Box::new(args.into_iter().next().unwrap()),
                    },

                    // [| f a b ... |] → ((f <$> a) <*> b) <*> ...
                    _ => {
                        let mut iter = args.into_iter();
                        let first_arg = iter.next().unwrap();

                        // Start with: f <$> first_arg
                        let mut result = Expr::Binary {
                            left: Box::new(func),
                            op: BinaryOp::Map,
                            right: Box::new(first_arg),
                        };

                        // Apply remaining arguments with <*>
                        for arg in iter {
                            result = Expr::Binary {
                                left: Box::new(result),
                                op: BinaryOp::Ap,
                                right: Box::new(arg),
                            };
                        }

                        result
                    }
                }
            }

            // Recursively desugar nested expressions
            Expr::Binary { left, op, right } => Expr::Binary {
                left: Box::new(self.desugar_expr(*left)),
                op,
                right: Box::new(self.desugar_expr(*right)),
            },

            Expr::Unary { op, operand } => Expr::Unary {
                op,
                operand: Box::new(self.desugar_expr(*operand)),
            },

            Expr::Call { callee, args } => Expr::Call {
                callee: Box::new(self.desugar_expr(*callee)),
                args: args.into_iter().map(|a| self.desugar_expr(a)).collect(),
            },

            Expr::Member { object, field } => Expr::Member {
                object: Box::new(self.desugar_expr(*object)),
                field,
            },

            Expr::Lambda {
                params,
                return_type,
                body,
            } => Expr::Lambda {
                params,
                return_type,
                body: Box::new(self.desugar_expr(*body)),
            },

            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => Expr::If {
                condition: Box::new(self.desugar_expr(*condition)),
                then_branch: Box::new(self.desugar_expr(*then_branch)),
                else_branch: else_branch.map(|e| Box::new(self.desugar_expr(*e))),
            },

            Expr::Match { scrutinee, arms } => {
                use crate::ast::MatchArm;

                Expr::Match {
                    scrutinee: Box::new(self.desugar_expr(*scrutinee)),
                    arms: arms
                        .into_iter()
                        .map(|arm| MatchArm {
                            pattern: arm.pattern,
                            guard: arm.guard.map(|g| Box::new(self.desugar_expr(*g))),
                            body: Box::new(self.desugar_expr(*arm.body)),
                        })
                        .collect(),
                }
            }

            Expr::Block(Block {
                statements,
                final_expr,
                ..
            }) => {
                use crate::ast::Stmt;

                // Desugar expressions within statements
                let statements = statements
                    .into_iter()
                    .map(|stmt| match stmt {
                        Stmt::Let {
                            name,
                            type_ann,
                            value,
                        } => Stmt::Let {
                            name,
                            type_ann,
                            value: self.desugar_expr(value),
                        },
                        Stmt::Assign { target, value } => Stmt::Assign {
                            target: self.desugar_expr(target),
                            value: self.desugar_expr(value),
                        },
                        Stmt::For {
                            binding,
                            iterable,
                            body,
                        } => Stmt::For {
                            binding,
                            iterable: self.desugar_expr(iterable),
                            body,
                        },
                        Stmt::While { condition, body } => Stmt::While {
                            condition: self.desugar_expr(condition),
                            body,
                        },
                        Stmt::Return(expr) => Stmt::Return(expr.map(|e| self.desugar_expr(e))),
                        Stmt::Expr(e) => Stmt::Expr(self.desugar_expr(e)),
                        other => other,
                    })
                    .collect();

                Expr::Block(Block {
                    statements,
                    final_expr: final_expr.map(|e| Box::new(self.desugar_expr(*e))),
                    span: Span::default(),
                })
            }

            Expr::Quote(inner) => Expr::Quote(Box::new(self.desugar_expr(*inner))),
            Expr::Unquote(inner) => Expr::Unquote(Box::new(self.desugar_expr(*inner))),
            Expr::QuasiQuote(inner) => Expr::QuasiQuote(Box::new(self.desugar_expr(*inner))),
            Expr::Eval(inner) => Expr::Eval(Box::new(self.desugar_expr(*inner))),

            // Logic expressions - recursively desugar body
            Expr::Forall(forall_expr) => {
                use crate::ast::ForallExpr;
                Expr::Forall(ForallExpr {
                    var: forall_expr.var,
                    type_: forall_expr.type_,
                    body: Box::new(self.desugar_expr(*forall_expr.body)),
                    span: forall_expr.span,
                })
            }
            Expr::Exists(exists_expr) => {
                use crate::ast::ExistsExpr;
                Expr::Exists(ExistsExpr {
                    var: exists_expr.var,
                    type_: exists_expr.type_,
                    body: Box::new(self.desugar_expr(*exists_expr.body)),
                    span: exists_expr.span,
                })
            }
            Expr::Implies { left, right, span } => Expr::Implies {
                left: Box::new(self.desugar_expr(*left)),
                right: Box::new(self.desugar_expr(*right)),
                span,
            },

            // Sex block - recursively desugar statements and final expression
            Expr::SexBlock(Block {
                statements,
                final_expr,
                ..
            }) => {
                use crate::ast::Stmt;

                // Desugar expressions within statements
                let statements = statements
                    .into_iter()
                    .map(|stmt| match stmt {
                        Stmt::Let {
                            name,
                            type_ann,
                            value,
                        } => Stmt::Let {
                            name,
                            type_ann,
                            value: self.desugar_expr(value),
                        },
                        Stmt::Assign { target, value } => Stmt::Assign {
                            target: self.desugar_expr(target),
                            value: self.desugar_expr(value),
                        },
                        Stmt::For {
                            binding,
                            iterable,
                            body,
                        } => Stmt::For {
                            binding,
                            iterable: self.desugar_expr(iterable),
                            body,
                        },
                        Stmt::While { condition, body } => Stmt::While {
                            condition: self.desugar_expr(condition),
                            body,
                        },
                        Stmt::Return(expr) => Stmt::Return(expr.map(|e| self.desugar_expr(e))),
                        Stmt::Expr(e) => Stmt::Expr(self.desugar_expr(e)),
                        other => other,
                    })
                    .collect();

                Expr::SexBlock(Block {
                    statements,
                    final_expr: final_expr.map(|e| Box::new(self.desugar_expr(*e))),
                    span: Span::default(),
                })
            }

            // Leaf nodes - no transformation needed
            Expr::Literal(_) | Expr::Identifier(_) | Expr::Reflect(_) => expr,

            // List - transform elements
            Expr::List(elements) => {
                Expr::List(elements.into_iter().map(|e| self.desugar_expr(e)).collect())
            }

            // Tuple - transform elements
            Expr::Tuple(elements) => {
                Expr::Tuple(elements.into_iter().map(|e| self.desugar_expr(e)).collect())
            }

            // Cast - transform inner expression
            Expr::Cast { expr, target_type } => Expr::Cast {
                expr: Box::new(self.desugar_expr(*expr)),
                target_type,
            },

            // Struct literal - transform field expressions
            Expr::StructLiteral { type_name, fields } => Expr::StructLiteral {
                type_name,
                fields: fields
                    .into_iter()
                    .map(|(name, expr)| (name, self.desugar_expr(expr)))
                    .collect(),
            },

            // Try - transform inner expression
            Expr::Try(inner) => Expr::Try(Box::new(self.desugar_expr(*inner))),

            // This - pass through unchanged
            Expr::This => Expr::This,
        }
    }
}

impl Default for IdiomDesugar {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for IdiomDesugar {
    fn name(&self) -> &str {
        "idiom_desugar"
    }

    fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        // DOL 1.0 declarations (Gene, Trait, Constraint, System) don't contain
        // DOL 2.0 expressions directly - they use Statement predicates.
        // When DOL 2.0 function bodies are added to declarations, this pass
        // will need to traverse and desugar those expressions.
        //
        // For now, we return the declaration unchanged.
        Ok(decl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};

    #[test]
    fn test_idiom_bracket_no_args() {
        let pass = IdiomDesugar::new();

        // [| f |] → f
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![],
        };

        let result = pass.desugar_expr(expr);
        assert_eq!(result, Expr::Identifier("f".to_string()));
    }

    #[test]
    fn test_idiom_bracket_one_arg() {
        let pass = IdiomDesugar::new();

        // [| f a |] → f <$> a
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![Expr::Identifier("a".to_string())],
        };

        let result = pass.desugar_expr(expr);

        let expected = Expr::Binary {
            left: Box::new(Expr::Identifier("f".to_string())),
            op: BinaryOp::Map,
            right: Box::new(Expr::Identifier("a".to_string())),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_idiom_bracket_two_args() {
        let pass = IdiomDesugar::new();

        // [| f a b |] → (f <$> a) <*> b
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![
                Expr::Identifier("a".to_string()),
                Expr::Identifier("b".to_string()),
            ],
        };

        let result = pass.desugar_expr(expr);

        let expected = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("f".to_string())),
                op: BinaryOp::Map,
                right: Box::new(Expr::Identifier("a".to_string())),
            }),
            op: BinaryOp::Ap,
            right: Box::new(Expr::Identifier("b".to_string())),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_idiom_bracket_three_args() {
        let pass = IdiomDesugar::new();

        // [| f a b c |] → ((f <$> a) <*> b) <*> c
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![
                Expr::Identifier("a".to_string()),
                Expr::Identifier("b".to_string()),
                Expr::Identifier("c".to_string()),
            ],
        };

        let result = pass.desugar_expr(expr);

        let expected = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Identifier("f".to_string())),
                    op: BinaryOp::Map,
                    right: Box::new(Expr::Identifier("a".to_string())),
                }),
                op: BinaryOp::Ap,
                right: Box::new(Expr::Identifier("b".to_string())),
            }),
            op: BinaryOp::Ap,
            right: Box::new(Expr::Identifier("c".to_string())),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_idiom_bracket_nested() {
        let pass = IdiomDesugar::new();

        // [| f [| g x |] |] → f <$> (g <$> x)
        let inner = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("g".to_string())),
            args: vec![Expr::Identifier("x".to_string())],
        };

        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![inner],
        };

        let result = pass.desugar_expr(expr);

        let expected = Expr::Binary {
            left: Box::new(Expr::Identifier("f".to_string())),
            op: BinaryOp::Map,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("g".to_string())),
                op: BinaryOp::Map,
                right: Box::new(Expr::Identifier("x".to_string())),
            }),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_idiom_bracket_in_binary() {
        let pass = IdiomDesugar::new();

        // 1 + [| f a |] → 1 + (f <$> a)
        let idiom = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![Expr::Identifier("a".to_string())],
        };

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: BinaryOp::Add,
            right: Box::new(idiom),
        };

        let result = pass.desugar_expr(expr);

        let expected = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("f".to_string())),
                op: BinaryOp::Map,
                right: Box::new(Expr::Identifier("a".to_string())),
            }),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_transformation_for_literals() {
        let pass = IdiomDesugar::new();

        let expr = Expr::Literal(Literal::Int(42));
        let result = pass.desugar_expr(expr.clone());
        assert_eq!(result, expr);
    }

    #[test]
    fn test_no_transformation_for_identifiers() {
        let pass = IdiomDesugar::new();

        let expr = Expr::Identifier("x".to_string());
        let result = pass.desugar_expr(expr.clone());
        assert_eq!(result, expr);
    }
}
