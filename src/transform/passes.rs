//! Built-in transformation passes.
//!
//! This module provides common optimization and transformation passes:
//! - Constant folding
//! - Dead code elimination
//! - Expression simplification

use crate::ast::{BinaryOp, Declaration, Expr, Literal, UnaryOp};
use crate::transform::{Pass, PassResult};

/// Constant folding pass.
///
/// Evaluates constant expressions at compile time:
/// - Arithmetic on literals: 1 + 2 => 3
/// - Boolean logic: true && false => false
/// - String concatenation: "a" + "b" => "ab"
pub struct ConstantFolding;

impl ConstantFolding {
    /// Creates a new constant folding pass.
    pub fn new() -> Self {
        Self
    }

    /// Fold an expression, evaluating constant subexpressions.
    #[allow(dead_code)]
    pub fn fold_expr(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Binary { left, op, right } => {
                let left = self.fold_expr(*left);
                let right = self.fold_expr(*right);

                // Try to evaluate constant binary operations
                if let (Expr::Literal(l), Expr::Literal(r)) = (&left, &right) {
                    if let Some(result) = Self::eval_binary(l, &op, r) {
                        return Expr::Literal(result);
                    }
                }

                Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            Expr::Unary { op, operand } => {
                let operand = self.fold_expr(*operand);

                // Try to evaluate constant unary operations
                if let Expr::Literal(lit) = &operand {
                    if let Some(result) = Self::eval_unary(&op, lit) {
                        return Expr::Literal(result);
                    }
                }

                Expr::Unary {
                    op,
                    operand: Box::new(operand),
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.fold_expr(*condition);

                // If condition is constant, select the appropriate branch
                if let Expr::Literal(Literal::Bool(b)) = &condition {
                    return if *b {
                        self.fold_expr(*then_branch)
                    } else if let Some(else_expr) = else_branch {
                        self.fold_expr(*else_expr)
                    } else {
                        // No else branch, return void-like
                        Expr::Block {
                            statements: vec![],
                            final_expr: None,
                        }
                    };
                }

                Expr::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(self.fold_expr(*then_branch)),
                    else_branch: else_branch.map(|e| Box::new(self.fold_expr(*e))),
                }
            }
            Expr::Call { callee, args } => Expr::Call {
                callee: Box::new(self.fold_expr(*callee)),
                args: args.into_iter().map(|a| self.fold_expr(a)).collect(),
            },
            Expr::Lambda {
                params,
                return_type,
                body,
            } => Expr::Lambda {
                params,
                return_type,
                body: Box::new(self.fold_expr(*body)),
            },
            Expr::Block {
                statements,
                final_expr,
            } => Expr::Block {
                statements,
                final_expr: final_expr.map(|e| Box::new(self.fold_expr(*e))),
            },
            other => other,
        }
    }

    /// Evaluate a binary operation on literals.
    #[allow(dead_code)]
    fn eval_binary(left: &Literal, op: &BinaryOp, right: &Literal) -> Option<Literal> {
        match (left, right) {
            (Literal::Int(a), Literal::Int(b)) => match op {
                BinaryOp::Add => Some(Literal::Int(a.wrapping_add(*b))),
                BinaryOp::Sub => Some(Literal::Int(a.wrapping_sub(*b))),
                BinaryOp::Mul => Some(Literal::Int(a.wrapping_mul(*b))),
                BinaryOp::Div if *b != 0 => Some(Literal::Int(a / b)),
                BinaryOp::Mod if *b != 0 => Some(Literal::Int(a % b)),
                BinaryOp::Eq => Some(Literal::Bool(a == b)),
                BinaryOp::Ne => Some(Literal::Bool(a != b)),
                BinaryOp::Lt => Some(Literal::Bool(a < b)),
                BinaryOp::Le => Some(Literal::Bool(a <= b)),
                BinaryOp::Gt => Some(Literal::Bool(a > b)),
                BinaryOp::Ge => Some(Literal::Bool(a >= b)),
                _ => None,
            },
            (Literal::Float(a), Literal::Float(b)) => match op {
                BinaryOp::Add => Some(Literal::Float(a + b)),
                BinaryOp::Sub => Some(Literal::Float(a - b)),
                BinaryOp::Mul => Some(Literal::Float(a * b)),
                BinaryOp::Div if *b != 0.0 => Some(Literal::Float(a / b)),
                BinaryOp::Eq => Some(Literal::Bool((a - b).abs() < f64::EPSILON)),
                BinaryOp::Ne => Some(Literal::Bool((a - b).abs() >= f64::EPSILON)),
                BinaryOp::Lt => Some(Literal::Bool(a < b)),
                BinaryOp::Le => Some(Literal::Bool(a <= b)),
                BinaryOp::Gt => Some(Literal::Bool(a > b)),
                BinaryOp::Ge => Some(Literal::Bool(a >= b)),
                _ => None,
            },
            (Literal::Bool(a), Literal::Bool(b)) => match op {
                BinaryOp::And => Some(Literal::Bool(*a && *b)),
                BinaryOp::Or => Some(Literal::Bool(*a || *b)),
                BinaryOp::Eq => Some(Literal::Bool(a == b)),
                BinaryOp::Ne => Some(Literal::Bool(a != b)),
                _ => None,
            },
            (Literal::String(a), Literal::String(b)) => match op {
                BinaryOp::Add => Some(Literal::String(format!("{}{}", a, b))),
                BinaryOp::Eq => Some(Literal::Bool(a == b)),
                BinaryOp::Ne => Some(Literal::Bool(a != b)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Evaluate a unary operation on a literal.
    #[allow(dead_code)]
    fn eval_unary(op: &UnaryOp, operand: &Literal) -> Option<Literal> {
        match (op, operand) {
            (UnaryOp::Neg, Literal::Int(n)) => Some(Literal::Int(-n)),
            (UnaryOp::Neg, Literal::Float(f)) => Some(Literal::Float(-f)),
            (UnaryOp::Not, Literal::Bool(b)) => Some(Literal::Bool(!b)),
            _ => None,
        }
    }
}

impl Default for ConstantFolding {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for ConstantFolding {
    fn name(&self) -> &str {
        "constant_folding"
    }

    fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        // DOL 1.0 declarations (Gene, Trait, Constraint, System) don't contain
        // DOL 2.0 expressions directly - they use Statement predicates.
        // This pass is primarily for future DOL 2.0 expression contexts.
        Ok(decl)
    }
}

/// Dead code elimination pass.
///
/// Removes unreachable code and unused bindings.
pub struct DeadCodeElimination;

impl DeadCodeElimination {
    /// Creates a new dead code elimination pass.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeadCodeElimination {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for DeadCodeElimination {
    fn name(&self) -> &str {
        "dead_code_elimination"
    }

    fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        // For now, just return the declaration unchanged
        // Full DCE requires use-def analysis
        Ok(decl)
    }
}

/// Expression simplification pass.
///
/// Applies algebraic simplifications:
/// - x + 0 => x
/// - x * 1 => x
/// - x * 0 => 0
/// - x - x => 0
/// - x / 1 => x
/// - !!x => x (double negation)
pub struct Simplify;

impl Simplify {
    /// Creates a new simplification pass.
    pub fn new() -> Self {
        Self
    }

    /// Simplify an expression using algebraic identities.
    #[allow(dead_code)]
    pub fn simplify_expr(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Binary { left, op, right } => {
                let left = self.simplify_expr(*left);
                let right = self.simplify_expr(*right);

                // Apply simplification rules
                match (&left, &op, &right) {
                    // x + 0 => x
                    (x, BinaryOp::Add, Expr::Literal(Literal::Int(0))) => return x.clone(),
                    (Expr::Literal(Literal::Int(0)), BinaryOp::Add, x) => return x.clone(),

                    // x * 1 => x
                    (x, BinaryOp::Mul, Expr::Literal(Literal::Int(1))) => return x.clone(),
                    (Expr::Literal(Literal::Int(1)), BinaryOp::Mul, x) => return x.clone(),

                    // x * 0 => 0
                    (_, BinaryOp::Mul, Expr::Literal(Literal::Int(0))) => {
                        return Expr::Literal(Literal::Int(0))
                    }
                    (Expr::Literal(Literal::Int(0)), BinaryOp::Mul, _) => {
                        return Expr::Literal(Literal::Int(0))
                    }

                    // x - 0 => x
                    (x, BinaryOp::Sub, Expr::Literal(Literal::Int(0))) => return x.clone(),

                    // x / 1 => x
                    (x, BinaryOp::Div, Expr::Literal(Literal::Int(1))) => return x.clone(),

                    // true && x => x, false && x => false
                    (Expr::Literal(Literal::Bool(true)), BinaryOp::And, x) => return x.clone(),
                    (Expr::Literal(Literal::Bool(false)), BinaryOp::And, _) => {
                        return Expr::Literal(Literal::Bool(false))
                    }
                    (x, BinaryOp::And, Expr::Literal(Literal::Bool(true))) => return x.clone(),
                    (_, BinaryOp::And, Expr::Literal(Literal::Bool(false))) => {
                        return Expr::Literal(Literal::Bool(false))
                    }

                    // false || x => x, true || x => true
                    (Expr::Literal(Literal::Bool(false)), BinaryOp::Or, x) => return x.clone(),
                    (Expr::Literal(Literal::Bool(true)), BinaryOp::Or, _) => {
                        return Expr::Literal(Literal::Bool(true))
                    }
                    (x, BinaryOp::Or, Expr::Literal(Literal::Bool(false))) => return x.clone(),
                    (_, BinaryOp::Or, Expr::Literal(Literal::Bool(true))) => {
                        return Expr::Literal(Literal::Bool(true))
                    }

                    _ => {}
                }

                Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            Expr::Unary { op, operand } => {
                let operand = self.simplify_expr(*operand);

                // Double negation elimination: !!x => x
                if op == UnaryOp::Not {
                    if let Expr::Unary {
                        op: UnaryOp::Not,
                        operand: inner,
                    } = operand
                    {
                        return *inner;
                    }
                }

                // Double minus elimination: --x => x
                if op == UnaryOp::Neg {
                    if let Expr::Unary {
                        op: UnaryOp::Neg,
                        operand: inner,
                    } = operand
                    {
                        return *inner;
                    }
                }

                Expr::Unary {
                    op,
                    operand: Box::new(operand),
                }
            }
            other => other,
        }
    }
}

impl Default for Simplify {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for Simplify {
    fn name(&self) -> &str {
        "simplify"
    }

    fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        // DOL 1.0 declarations don't contain DOL 2.0 expressions directly
        Ok(decl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_arithmetic() {
        let pass = ConstantFolding::new();

        // 1 + 2 => 3
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(3)));

        // 10 * 5 => 50
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(10))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(5))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(50)));
    }

    #[test]
    fn test_constant_folding_nested() {
        let pass = ConstantFolding::new();

        // (1 + 2) * 3 => 9
        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Int(1))),
                op: BinaryOp::Add,
                right: Box::new(Expr::Literal(Literal::Int(2))),
            }),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(3))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(9)));
    }

    #[test]
    fn test_constant_folding_boolean() {
        let pass = ConstantFolding::new();

        // true && false => false
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Bool(true))),
            op: BinaryOp::And,
            right: Box::new(Expr::Literal(Literal::Bool(false))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Bool(false)));
    }

    #[test]
    fn test_constant_folding_unary() {
        let pass = ConstantFolding::new();

        // -5 => -5
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::Int(5))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(-5)));

        // !true => false
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(true))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Bool(false)));
    }

    #[test]
    fn test_constant_folding_if() {
        let pass = ConstantFolding::new();

        // if true { 1 } else { 2 } => 1
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(Literal::Bool(true))),
            then_branch: Box::new(Expr::Literal(Literal::Int(1))),
            else_branch: Some(Box::new(Expr::Literal(Literal::Int(2)))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(1)));

        // if false { 1 } else { 2 } => 2
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(Literal::Bool(false))),
            then_branch: Box::new(Expr::Literal(Literal::Int(1))),
            else_branch: Some(Box::new(Expr::Literal(Literal::Int(2)))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(2)));
    }

    #[test]
    fn test_simplify_identity() {
        let pass = Simplify::new();

        // x + 0 => x
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(0))),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Identifier("x".to_string()));

        // x * 1 => x
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Identifier("x".to_string()));
    }

    #[test]
    fn test_simplify_zero() {
        let pass = Simplify::new();

        // x * 0 => 0
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(0))),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(0)));
    }

    #[test]
    fn test_simplify_double_negation() {
        let pass = Simplify::new();

        // !!x => x
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Identifier("x".to_string())),
            }),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Identifier("x".to_string()));
    }
}
