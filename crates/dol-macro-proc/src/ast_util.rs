//! AST manipulation utilities for procedural macros.
//!
//! This module provides utilities for traversing and transforming
//! DOL AST nodes in procedural macros.

use crate::error::{ProcMacroError, ProcMacroResult};
use metadol::ast::{Block, Declaration, Expr, Span, Stmt, TypeExpr};

/// Trait for transforming AST nodes.
pub trait AstTransform {
    /// Transforms an expression.
    fn transform_expr(&mut self, expr: &Expr) -> ProcMacroResult<Expr> {
        Ok(expr.clone())
    }

    /// Transforms a statement.
    fn transform_stmt(&mut self, stmt: &Stmt) -> ProcMacroResult<Stmt> {
        Ok(stmt.clone())
    }

    /// Transforms a declaration.
    fn transform_decl(&mut self, decl: &Declaration) -> ProcMacroResult<Declaration> {
        Ok(decl.clone())
    }
}

/// AST manipulator for procedural macros.
///
/// Provides utilities for traversing and modifying AST nodes.
pub struct AstManipulator;

impl AstManipulator {
    /// Creates a new AST manipulator.
    pub fn new() -> Self {
        Self
    }

    /// Recursively walks an expression tree, applying a transformation.
    pub fn walk_expr<F>(&self, expr: &Expr, f: &mut F) -> ProcMacroResult<Expr>
    where
        F: FnMut(&Expr) -> ProcMacroResult<Expr>,
    {
        match expr {
            Expr::Binary { op, left, right } => {
                let left = Box::new(self.walk_expr(left, f)?);
                let right = Box::new(self.walk_expr(right, f)?);
                f(&Expr::Binary {
                    op: op.clone(),
                    left,
                    right,
                })
            }

            Expr::Unary { op, operand } => {
                let operand = Box::new(self.walk_expr(operand, f)?);
                f(&Expr::Unary {
                    op: op.clone(),
                    operand,
                })
            }

            Expr::Call { callee, args } => {
                let callee = Box::new(self.walk_expr(callee, f)?);
                let args: ProcMacroResult<Vec<Expr>> =
                    args.iter().map(|a| self.walk_expr(a, f)).collect();
                f(&Expr::Call {
                    callee,
                    args: args?,
                })
            }

            Expr::Member { object, field } => {
                let object = Box::new(self.walk_expr(object, f)?);
                f(&Expr::Member {
                    object,
                    field: field.clone(),
                })
            }

            Expr::List(elements) => {
                let elements: ProcMacroResult<Vec<Expr>> =
                    elements.iter().map(|e| self.walk_expr(e, f)).collect();
                f(&Expr::List(elements?))
            }

            Expr::Tuple(elements) => {
                let elements: ProcMacroResult<Vec<Expr>> =
                    elements.iter().map(|e| self.walk_expr(e, f)).collect();
                f(&Expr::Tuple(elements?))
            }

            _ => f(expr),
        }
    }

    /// Walks a statement tree, applying a transformation.
    pub fn walk_stmt<F>(&self, stmt: &Stmt, f: &mut F) -> ProcMacroResult<Stmt>
    where
        F: FnMut(&Stmt) -> ProcMacroResult<Stmt>,
    {
        match stmt {
            Stmt::Let {
                name,
                type_ann,
                value,
            } => {
                let value = self.walk_expr(value, &mut |e: &Expr| Ok(e.clone()))?;
                f(&Stmt::Let {
                    name: name.clone(),
                    type_ann: type_ann.clone(),
                    value,
                })
            }

            Stmt::Expr(expr) => {
                let expr = self.walk_expr(expr, &mut |e: &Expr| Ok(e.clone()))?;
                f(&Stmt::Expr(expr))
            }

            Stmt::Return(value) => {
                let value = if let Some(v) = value {
                    Some(self.walk_expr(v, &mut |e: &Expr| Ok(e.clone()))?)
                } else {
                    None
                };
                f(&Stmt::Return(value))
            }

            _ => f(stmt),
        }
    }

    /// Finds all identifiers in an expression.
    pub fn find_identifiers(&self, expr: &Expr) -> Vec<String> {
        let mut idents = Vec::new();
        self.collect_identifiers(expr, &mut idents);
        idents
    }

    fn collect_identifiers(&self, expr: &Expr, idents: &mut Vec<String>) {
        match expr {
            Expr::Identifier(name) => idents.push(name.clone()),
            Expr::Binary { left, right, .. } => {
                self.collect_identifiers(left, idents);
                self.collect_identifiers(right, idents);
            }
            Expr::Unary { operand, .. } => {
                self.collect_identifiers(operand, idents);
            }
            Expr::Call { callee, args } => {
                self.collect_identifiers(callee, idents);
                for arg in args {
                    self.collect_identifiers(arg, idents);
                }
            }
            Expr::List(elements) | Expr::Tuple(elements) => {
                for elem in elements {
                    self.collect_identifiers(elem, idents);
                }
            }
            _ => {}
        }
    }

    /// Replaces all occurrences of an identifier in an expression.
    pub fn replace_identifier(&self, expr: &Expr, old: &str, new: &str) -> Expr {
        match expr {
            Expr::Identifier(name) if name == old => Expr::Identifier(new.to_string()),

            Expr::Binary { op, left, right } => Expr::Binary {
                op: op.clone(),
                left: Box::new(self.replace_identifier(left, old, new)),
                right: Box::new(self.replace_identifier(right, old, new)),
            },

            Expr::Unary { op, operand } => Expr::Unary {
                op: op.clone(),
                operand: Box::new(self.replace_identifier(operand, old, new)),
            },

            Expr::Call { callee, args } => Expr::Call {
                callee: Box::new(self.replace_identifier(callee, old, new)),
                args: args
                    .iter()
                    .map(|a| self.replace_identifier(a, old, new))
                    .collect(),
            },

            _ => expr.clone(),
        }
    }

    /// Counts the number of nodes in an expression tree.
    pub fn count_nodes(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Binary { left, right, .. } => {
                1 + self.count_nodes(left) + self.count_nodes(right)
            }
            Expr::Unary { operand, .. } => 1 + self.count_nodes(operand),
            Expr::Call { callee, args } => {
                1 + self.count_nodes(callee) + args.iter().map(|a| self.count_nodes(a)).sum::<usize>()
            }
            Expr::List(elements) | Expr::Tuple(elements) => {
                1 + elements.iter().map(|e| self.count_nodes(e)).sum::<usize>()
            }
            _ => 1,
        }
    }

    /// Checks if an expression contains a specific identifier.
    pub fn contains_identifier(&self, expr: &Expr, name: &str) -> bool {
        self.find_identifiers(expr).iter().any(|id| id == name)
    }
}

impl Default for AstManipulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadol::ast::{BinaryOp, Literal};

    #[test]
    fn test_find_identifiers() {
        let manipulator = AstManipulator::new();

        // x + y
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Identifier("y".to_string())),
        };

        let idents = manipulator.find_identifiers(&expr);
        assert_eq!(idents.len(), 2);
        assert!(idents.contains(&"x".to_string()));
        assert!(idents.contains(&"y".to_string()));
    }

    #[test]
    fn test_replace_identifier() {
        let manipulator = AstManipulator::new();

        let expr = Expr::Identifier("x".to_string());
        let replaced = manipulator.replace_identifier(&expr, "x", "y");

        if let Expr::Identifier(name) = replaced {
            assert_eq!(name, "y");
        } else {
            panic!("Expected identifier");
        }
    }

    #[test]
    fn test_replace_identifier_in_binary() {
        let manipulator = AstManipulator::new();

        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        let replaced = manipulator.replace_identifier(&expr, "x", "y");

        if let Expr::Binary { left, .. } = replaced {
            if let Expr::Identifier(name) = *left {
                assert_eq!(name, "y");
            } else {
                panic!("Expected identifier");
            }
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_count_nodes() {
        let manipulator = AstManipulator::new();

        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        let count = manipulator.count_nodes(&expr);
        assert_eq!(count, 3); // Binary + Identifier + Literal
    }

    #[test]
    fn test_contains_identifier() {
        let manipulator = AstManipulator::new();

        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        assert!(manipulator.contains_identifier(&expr, "x"));
        assert!(!manipulator.contains_identifier(&expr, "y"));
    }

    #[test]
    fn test_walk_expr() {
        let manipulator = AstManipulator::new();

        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        // Walk and count identifier nodes
        let mut count = 0;
        let _ = manipulator.walk_expr(&expr, &mut |e| {
            if matches!(e, Expr::Identifier(_)) {
                count += 1;
            }
            Ok(e.clone())
        });

        assert_eq!(count, 1);
    }
}
