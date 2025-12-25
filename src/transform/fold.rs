//! Fold pattern for expression transformation.
//!
//! The Fold trait provides a functional approach to AST transformation,
//! where each node is transformed and a new AST is produced.

use crate::ast::{
    BinaryOp, Declaration, Expr, Gene, Literal, MatchArm, Pattern, Statement, Stmt, TypeExpr,
    UnaryOp,
};

/// Trait for transforming expressions by folding over the AST.
///
/// Unlike MutVisitor which modifies in place, Fold creates new nodes.
pub trait Fold {
    /// Fold a literal.
    fn fold_literal(&mut self, lit: Literal) -> Literal {
        lit
    }

    /// Fold an identifier.
    fn fold_identifier(&mut self, name: String) -> Expr {
        Expr::Identifier(name)
    }

    /// Fold a binary expression.
    fn fold_binary(&mut self, left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self.fold_expr(left)),
            op,
            right: Box::new(self.fold_expr(right)),
        }
    }

    /// Fold a unary expression.
    fn fold_unary(&mut self, op: UnaryOp, operand: Expr) -> Expr {
        Expr::Unary {
            op,
            operand: Box::new(self.fold_expr(operand)),
        }
    }

    /// Fold a function call.
    fn fold_call(&mut self, callee: Expr, args: Vec<Expr>) -> Expr {
        Expr::Call {
            callee: Box::new(self.fold_expr(callee)),
            args: args.into_iter().map(|a| self.fold_expr(a)).collect(),
        }
    }

    /// Fold a member access.
    fn fold_member(&mut self, object: Expr, field: String) -> Expr {
        Expr::Member {
            object: Box::new(self.fold_expr(object)),
            field,
        }
    }

    /// Fold a lambda expression.
    fn fold_lambda(
        &mut self,
        params: Vec<(String, Option<TypeExpr>)>,
        return_type: Option<TypeExpr>,
        body: Expr,
    ) -> Expr {
        Expr::Lambda {
            params,
            return_type,
            body: Box::new(self.fold_expr(body)),
        }
    }

    /// Fold an if expression.
    fn fold_if(&mut self, condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
        Expr::If {
            condition: Box::new(self.fold_expr(condition)),
            then_branch: Box::new(self.fold_expr(then_branch)),
            else_branch: else_branch.map(|e| Box::new(self.fold_expr(e))),
        }
    }

    /// Fold a match expression.
    fn fold_match(&mut self, scrutinee: Expr, arms: Vec<MatchArm>) -> Expr {
        Expr::Match {
            scrutinee: Box::new(self.fold_expr(scrutinee)),
            arms: arms.into_iter().map(|a| self.fold_match_arm(a)).collect(),
        }
    }

    /// Fold a match arm.
    fn fold_match_arm(&mut self, arm: MatchArm) -> MatchArm {
        MatchArm {
            pattern: self.fold_pattern(arm.pattern),
            guard: arm.guard.map(|g| Box::new(self.fold_expr(*g))),
            body: Box::new(self.fold_expr(*arm.body)),
        }
    }

    /// Fold a pattern.
    fn fold_pattern(&mut self, pattern: Pattern) -> Pattern {
        match pattern {
            Pattern::Constructor { name, fields } => Pattern::Constructor {
                name,
                fields: fields.into_iter().map(|p| self.fold_pattern(p)).collect(),
            },
            Pattern::Tuple(patterns) => {
                Pattern::Tuple(patterns.into_iter().map(|p| self.fold_pattern(p)).collect())
            }
            p => p,
        }
    }

    /// Fold a block expression.
    fn fold_block(&mut self, statements: Vec<Stmt>, final_expr: Option<Expr>) -> Expr {
        Expr::Block {
            statements: statements.into_iter().map(|s| self.fold_stmt(s)).collect(),
            final_expr: final_expr.map(|e| Box::new(self.fold_expr(e))),
        }
    }

    /// Fold a quote expression.
    fn fold_quote(&mut self, inner: Expr) -> Expr {
        Expr::Quote(Box::new(self.fold_expr(inner)))
    }

    /// Fold an unquote expression.
    fn fold_unquote(&mut self, inner: Expr) -> Expr {
        Expr::Unquote(Box::new(self.fold_expr(inner)))
    }

    /// Fold a quasi-quote expression.
    fn fold_quasi_quote(&mut self, inner: Expr) -> Expr {
        Expr::QuasiQuote(Box::new(self.fold_expr(inner)))
    }

    /// Fold an eval expression.
    fn fold_eval(&mut self, inner: Expr) -> Expr {
        Expr::Eval(Box::new(self.fold_expr(inner)))
    }

    /// Fold a reflect expression.
    fn fold_reflect(&mut self, ty: TypeExpr) -> Expr {
        Expr::Reflect(Box::new(ty))
    }

    /// Fold an idiom bracket expression.
    fn fold_idiom_bracket(&mut self, func: Expr, args: Vec<Expr>) -> Expr {
        Expr::IdiomBracket {
            func: Box::new(self.fold_expr(func)),
            args: args.into_iter().map(|a| self.fold_expr(a)).collect(),
        }
    }

    /// Fold any expression by dispatching to specific methods.
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::Literal(lit) => Expr::Literal(self.fold_literal(lit)),
            Expr::Identifier(name) => self.fold_identifier(name),
            Expr::Binary { left, op, right } => self.fold_binary(*left, op, *right),
            Expr::Unary { op, operand } => self.fold_unary(op, *operand),
            Expr::Call { callee, args } => self.fold_call(*callee, args),
            Expr::Member { object, field } => self.fold_member(*object, field),
            Expr::Lambda {
                params,
                return_type,
                body,
            } => self.fold_lambda(params, return_type, *body),
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => self.fold_if(*condition, *then_branch, else_branch.map(|e| *e)),
            Expr::Match { scrutinee, arms } => self.fold_match(*scrutinee, arms),
            Expr::Block {
                statements,
                final_expr,
            } => self.fold_block(statements, final_expr.map(|e| *e)),
            Expr::Quote(inner) => self.fold_quote(*inner),
            Expr::Unquote(inner) => self.fold_unquote(*inner),
            Expr::QuasiQuote(inner) => self.fold_quasi_quote(*inner),
            Expr::Eval(inner) => self.fold_eval(*inner),
            Expr::Reflect(ty) => self.fold_reflect(*ty),
            Expr::IdiomBracket { func, args } => self.fold_idiom_bracket(*func, args),
            Expr::Forall(forall_expr) => {
                use crate::ast::ForallExpr;
                Expr::Forall(ForallExpr {
                    var: forall_expr.var.clone(),
                    type_: forall_expr.type_.clone(),
                    body: Box::new(self.fold_expr(*forall_expr.body)),
                    span: forall_expr.span,
                })
            }
            Expr::Exists(exists_expr) => {
                use crate::ast::ExistsExpr;
                Expr::Exists(ExistsExpr {
                    var: exists_expr.var.clone(),
                    type_: exists_expr.type_.clone(),
                    body: Box::new(self.fold_expr(*exists_expr.body)),
                    span: exists_expr.span,
                })
            }
            Expr::Implies { left, right, span } => Expr::Implies {
                left: Box::new(self.fold_expr(*left)),
                right: Box::new(self.fold_expr(*right)),
                span,
            },
            Expr::SexBlock {
                statements,
                final_expr,
            } => self.fold_sex_block(statements, final_expr.map(|e| *e)),
            Expr::List(elements) => {
                Expr::List(elements.into_iter().map(|e| self.fold_expr(e)).collect())
            }
        }
    }

    /// Fold a sex block expression.
    fn fold_sex_block(&mut self, statements: Vec<Stmt>, final_expr: Option<Expr>) -> Expr {
        Expr::SexBlock {
            statements: statements
                .into_iter()
                .map(|stmt| self.fold_stmt(stmt))
                .collect(),
            final_expr: final_expr.map(|e| Box::new(self.fold_expr(e))),
        }
    }

    /// Fold a DOL 2.0 statement.
    fn fold_stmt(&mut self, stmt: Stmt) -> Stmt {
        match stmt {
            Stmt::Let {
                name,
                type_ann,
                value,
            } => Stmt::Let {
                name,
                type_ann,
                value: self.fold_expr(value),
            },
            Stmt::Assign { target, value } => Stmt::Assign {
                target: self.fold_expr(target),
                value: self.fold_expr(value),
            },
            Stmt::Expr(expr) => Stmt::Expr(self.fold_expr(expr)),
            Stmt::Return(expr) => Stmt::Return(expr.map(|e| self.fold_expr(e))),
            Stmt::For {
                binding,
                iterable,
                body,
            } => Stmt::For {
                binding,
                iterable: self.fold_expr(iterable),
                body: body.into_iter().map(|s| self.fold_stmt(s)).collect(),
            },
            Stmt::While { condition, body } => Stmt::While {
                condition: self.fold_expr(condition),
                body: body.into_iter().map(|s| self.fold_stmt(s)).collect(),
            },
            Stmt::Loop { body } => Stmt::Loop {
                body: body.into_iter().map(|s| self.fold_stmt(s)).collect(),
            },
            s => s, // Break, Continue pass through unchanged
        }
    }

    /// Fold a DOL 1.0 statement (Has, Is, etc.).
    fn fold_statement(&mut self, stmt: Statement) -> Statement {
        // DOL 1.0 statements don't contain expressions to fold
        stmt
    }

    /// Fold a gene.
    fn fold_gene(&mut self, gene: Gene) -> Gene {
        Gene {
            name: gene.name,
            statements: gene
                .statements
                .into_iter()
                .map(|s| self.fold_statement(s))
                .collect(),
            exegesis: gene.exegesis,
            span: gene.span,
        }
    }

    /// Fold a declaration.
    fn fold_declaration(&mut self, decl: Declaration) -> Declaration {
        match decl {
            Declaration::Gene(gene) => Declaration::Gene(self.fold_gene(gene)),
            d => d,
        }
    }
}

/// Identity fold that returns the AST unchanged.
pub struct IdentityFold;

impl Fold for IdentityFold {}

#[cfg(test)]
mod tests {
    use super::*;

    struct IncrementLiterals;

    impl Fold for IncrementLiterals {
        fn fold_literal(&mut self, lit: Literal) -> Literal {
            match lit {
                Literal::Int(n) => Literal::Int(n + 1),
                other => other,
            }
        }
    }

    #[test]
    fn test_fold_increments_literals() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        };

        let mut fold = IncrementLiterals;
        let result = fold.fold_expr(expr);

        match result {
            Expr::Binary { left, right, .. } => {
                assert_eq!(*left, Expr::Literal(Literal::Int(2)));
                assert_eq!(*right, Expr::Literal(Literal::Int(3)));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    struct DoubleNegation;

    impl Fold for DoubleNegation {
        fn fold_unary(&mut self, op: UnaryOp, operand: Expr) -> Expr {
            let folded = self.fold_expr(operand);
            // Remove double negation: --x => x
            if op == UnaryOp::Neg {
                if let Expr::Unary {
                    op: UnaryOp::Neg,
                    operand: inner,
                } = folded
                {
                    return *inner;
                }
            }
            Expr::Unary {
                op,
                operand: Box::new(folded),
            }
        }
    }

    #[test]
    fn test_fold_removes_double_negation() {
        // --5 should become 5
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(Expr::Literal(Literal::Int(5))),
            }),
        };

        let mut fold = DoubleNegation;
        let result = fold.fold_expr(expr);

        assert_eq!(result, Expr::Literal(Literal::Int(5)));
    }
}
