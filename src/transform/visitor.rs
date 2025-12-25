//! Visitor pattern for AST traversal.
//!
//! This module provides traits for traversing DOL ASTs either immutably
//! (for analysis) or mutably (for transformation).

use crate::ast::{
    BinaryOp, Constraint, Declaration, Evolution, Expr, Gene, Literal, MatchArm, Pattern,
    Statement, Stmt, System, Trait, TypeExpr, UnaryOp,
};

/// Immutable visitor for AST traversal.
///
/// Implement this trait to analyze an AST without modifying it.
pub trait Visitor {
    /// Visit a declaration.
    fn visit_declaration(&mut self, decl: &Declaration) {
        walk_declaration(self, decl);
    }

    /// Visit a gene.
    fn visit_gene(&mut self, gene: &Gene) {
        walk_gene(self, gene);
    }

    /// Visit a trait.
    fn visit_trait(&mut self, tr: &Trait) {
        walk_trait(self, tr);
    }

    /// Visit a constraint.
    fn visit_constraint(&mut self, c: &Constraint) {
        walk_constraint(self, c);
    }

    /// Visit a system.
    fn visit_system(&mut self, sys: &System) {
        walk_system(self, sys);
    }

    /// Visit an evolution.
    fn visit_evolution(&mut self, _evo: &Evolution) {}

    /// Visit a statement.
    fn visit_statement(&mut self, _stmt: &Statement) {}

    /// Visit a DOL 2.0 statement.
    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    /// Visit an expression.
    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    /// Visit a type expression.
    fn visit_type_expr(&mut self, _ty: &TypeExpr) {}

    /// Visit a literal.
    fn visit_literal(&mut self, _lit: &Literal) {}

    /// Visit an identifier.
    fn visit_identifier(&mut self, _name: &str) {}

    /// Visit a binary operator.
    fn visit_binary_op(&mut self, _op: &BinaryOp) {}

    /// Visit a unary operator.
    fn visit_unary_op(&mut self, _op: &UnaryOp) {}

    /// Visit a pattern.
    fn visit_pattern(&mut self, pattern: &Pattern) {
        walk_pattern(self, pattern);
    }

    /// Visit a match arm.
    fn visit_match_arm(&mut self, arm: &MatchArm) {
        self.visit_pattern(&arm.pattern);
        if let Some(ref guard) = arm.guard {
            self.visit_expr(guard);
        }
        self.visit_expr(&arm.body);
    }
}

/// Mutable visitor for AST transformation.
///
/// Implement this trait to transform an AST in place.
pub trait MutVisitor {
    /// Transform a declaration.
    fn visit_declaration(&mut self, decl: &mut Declaration) {
        walk_declaration_mut(self, decl);
    }

    /// Transform a gene.
    fn visit_gene(&mut self, gene: &mut Gene) {
        walk_gene_mut(self, gene);
    }

    /// Transform a trait.
    fn visit_trait(&mut self, tr: &mut Trait) {
        walk_trait_mut(self, tr);
    }

    /// Transform a constraint.
    fn visit_constraint(&mut self, c: &mut Constraint) {
        walk_constraint_mut(self, c);
    }

    /// Transform a system.
    fn visit_system(&mut self, sys: &mut System) {
        walk_system_mut(self, sys);
    }

    /// Transform an evolution.
    fn visit_evolution(&mut self, _evo: &mut Evolution) {}

    /// Transform a statement.
    fn visit_statement(&mut self, _stmt: &mut Statement) {}

    /// Transform a DOL 2.0 statement.
    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        walk_stmt_mut(self, stmt);
    }

    /// Transform an expression.
    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr_mut(self, expr);
    }

    /// Transform a type expression.
    fn visit_type_expr(&mut self, _ty: &mut TypeExpr) {}

    /// Transform a pattern.
    fn visit_pattern(&mut self, pattern: &mut Pattern) {
        walk_pattern_mut(self, pattern);
    }

    /// Transform a match arm.
    fn visit_match_arm(&mut self, arm: &mut MatchArm) {
        self.visit_pattern(&mut arm.pattern);
        if let Some(ref mut guard) = arm.guard {
            self.visit_expr(guard);
        }
        self.visit_expr(&mut arm.body);
    }
}

// Walk functions for immutable visitor

fn walk_declaration<V: Visitor + ?Sized>(v: &mut V, decl: &Declaration) {
    match decl {
        Declaration::Gene(gene) => v.visit_gene(gene),
        Declaration::Trait(tr) => v.visit_trait(tr),
        Declaration::Constraint(c) => v.visit_constraint(c),
        Declaration::System(sys) => v.visit_system(sys),
        Declaration::Evolution(evo) => v.visit_evolution(evo),
    }
}

fn walk_gene<V: Visitor + ?Sized>(v: &mut V, gene: &Gene) {
    for stmt in &gene.statements {
        v.visit_statement(stmt);
    }
}

fn walk_trait<V: Visitor + ?Sized>(v: &mut V, tr: &Trait) {
    for stmt in &tr.statements {
        v.visit_statement(stmt);
    }
}

fn walk_constraint<V: Visitor + ?Sized>(v: &mut V, c: &Constraint) {
    for stmt in &c.statements {
        v.visit_statement(stmt);
    }
}

fn walk_system<V: Visitor + ?Sized>(v: &mut V, sys: &System) {
    for stmt in &sys.statements {
        v.visit_statement(stmt);
    }
}

fn walk_stmt<V: Visitor + ?Sized>(v: &mut V, stmt: &Stmt) {
    match stmt {
        Stmt::Let { value, .. } => {
            v.visit_expr(value);
        }
        Stmt::Assign { target, value } => {
            v.visit_expr(target);
            v.visit_expr(value);
        }
        Stmt::Expr(expr) => v.visit_expr(expr),
        Stmt::Return(Some(expr)) => v.visit_expr(expr),
        Stmt::For { iterable, body, .. } => {
            v.visit_expr(iterable);
            for s in body {
                v.visit_stmt(s);
            }
        }
        Stmt::While { condition, body } => {
            v.visit_expr(condition);
            for s in body {
                v.visit_stmt(s);
            }
        }
        Stmt::Loop { body } => {
            for s in body {
                v.visit_stmt(s);
            }
        }
        _ => {}
    }
}

fn walk_expr<V: Visitor + ?Sized>(v: &mut V, expr: &Expr) {
    match expr {
        Expr::Literal(lit) => v.visit_literal(lit),
        Expr::Identifier(name) => v.visit_identifier(name),
        Expr::Binary { left, op, right } => {
            v.visit_expr(left);
            v.visit_binary_op(op);
            v.visit_expr(right);
        }
        Expr::Unary { op, operand } => {
            v.visit_unary_op(op);
            v.visit_expr(operand);
        }
        Expr::Call { callee, args } => {
            v.visit_expr(callee);
            for arg in args {
                v.visit_expr(arg);
            }
        }
        Expr::Member { object, .. } => {
            v.visit_expr(object);
        }
        Expr::Lambda { body, .. } => {
            v.visit_expr(body);
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            v.visit_expr(condition);
            v.visit_expr(then_branch);
            if let Some(else_expr) = else_branch {
                v.visit_expr(else_expr);
            }
        }
        Expr::Match { scrutinee, arms } => {
            v.visit_expr(scrutinee);
            for arm in arms {
                v.visit_match_arm(arm);
            }
        }
        Expr::Block {
            statements,
            final_expr,
        } => {
            for stmt in statements {
                v.visit_stmt(stmt);
            }
            if let Some(expr) = final_expr {
                v.visit_expr(expr);
            }
        }
        Expr::Quote(inner) => v.visit_expr(inner),
        Expr::Unquote(inner) => v.visit_expr(inner),
        Expr::QuasiQuote(inner) => v.visit_expr(inner),
        Expr::Eval(inner) => v.visit_expr(inner),
        Expr::Reflect(ty) => v.visit_type_expr(ty),
        Expr::IdiomBracket { func, args } => {
            v.visit_expr(func);
            for arg in args {
                v.visit_expr(arg);
            }
        }
        Expr::Forall(forall_expr) => {
            v.visit_type_expr(&forall_expr.type_);
            v.visit_expr(&forall_expr.body);
        }
        Expr::Exists(exists_expr) => {
            v.visit_type_expr(&exists_expr.type_);
            v.visit_expr(&exists_expr.body);
        }
        Expr::Implies { left, right, .. } => {
            v.visit_expr(left);
            v.visit_expr(right);
        }
        Expr::SexBlock {
            statements,
            final_expr,
        } => {
            for stmt in statements {
                v.visit_stmt(stmt);
            }
            if let Some(expr) = final_expr {
                v.visit_expr(expr);
            }
        }
        Expr::List(elements) => {
            for elem in elements {
                v.visit_expr(elem);
            }
        }
    }
}

fn walk_pattern<V: Visitor + ?Sized>(v: &mut V, pattern: &Pattern) {
    match pattern {
        Pattern::Literal(lit) => v.visit_literal(lit),
        Pattern::Identifier(_) => {}
        Pattern::Wildcard => {}
        Pattern::Constructor { fields, .. } => {
            for p in fields {
                v.visit_pattern(p);
            }
        }
        Pattern::Tuple(patterns) => {
            for p in patterns {
                v.visit_pattern(p);
            }
        }
    }
}

// Walk functions for mutable visitor

fn walk_declaration_mut<V: MutVisitor + ?Sized>(v: &mut V, decl: &mut Declaration) {
    match decl {
        Declaration::Gene(gene) => v.visit_gene(gene),
        Declaration::Trait(tr) => v.visit_trait(tr),
        Declaration::Constraint(c) => v.visit_constraint(c),
        Declaration::System(sys) => v.visit_system(sys),
        Declaration::Evolution(evo) => v.visit_evolution(evo),
    }
}

fn walk_gene_mut<V: MutVisitor + ?Sized>(v: &mut V, gene: &mut Gene) {
    for stmt in &mut gene.statements {
        v.visit_statement(stmt);
    }
}

fn walk_trait_mut<V: MutVisitor + ?Sized>(v: &mut V, tr: &mut Trait) {
    for stmt in &mut tr.statements {
        v.visit_statement(stmt);
    }
}

fn walk_constraint_mut<V: MutVisitor + ?Sized>(v: &mut V, c: &mut Constraint) {
    for stmt in &mut c.statements {
        v.visit_statement(stmt);
    }
}

fn walk_system_mut<V: MutVisitor + ?Sized>(v: &mut V, sys: &mut System) {
    for stmt in &mut sys.statements {
        v.visit_statement(stmt);
    }
}

fn walk_stmt_mut<V: MutVisitor + ?Sized>(v: &mut V, stmt: &mut Stmt) {
    match stmt {
        Stmt::Let { value, .. } => {
            v.visit_expr(value);
        }
        Stmt::Assign { target, value } => {
            v.visit_expr(target);
            v.visit_expr(value);
        }
        Stmt::Expr(expr) => v.visit_expr(expr),
        Stmt::Return(Some(expr)) => v.visit_expr(expr),
        Stmt::For { iterable, body, .. } => {
            v.visit_expr(iterable);
            for s in body {
                v.visit_stmt(s);
            }
        }
        Stmt::While { condition, body } => {
            v.visit_expr(condition);
            for s in body {
                v.visit_stmt(s);
            }
        }
        Stmt::Loop { body } => {
            for s in body {
                v.visit_stmt(s);
            }
        }
        _ => {}
    }
}

fn walk_expr_mut<V: MutVisitor + ?Sized>(v: &mut V, expr: &mut Expr) {
    match expr {
        Expr::Binary { left, right, .. } => {
            v.visit_expr(left);
            v.visit_expr(right);
        }
        Expr::Unary { operand, .. } => {
            v.visit_expr(operand);
        }
        Expr::Call { callee, args } => {
            v.visit_expr(callee);
            for arg in args {
                v.visit_expr(arg);
            }
        }
        Expr::Member { object, .. } => {
            v.visit_expr(object);
        }
        Expr::Lambda { body, .. } => {
            v.visit_expr(body);
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            v.visit_expr(condition);
            v.visit_expr(then_branch);
            if let Some(else_expr) = else_branch {
                v.visit_expr(else_expr);
            }
        }
        Expr::Match { scrutinee, arms } => {
            v.visit_expr(scrutinee);
            for arm in arms {
                v.visit_match_arm(arm);
            }
        }
        Expr::Block {
            statements,
            final_expr,
        } => {
            for stmt in statements {
                v.visit_stmt(stmt);
            }
            if let Some(e) = final_expr {
                v.visit_expr(e);
            }
        }
        Expr::Quote(inner) => v.visit_expr(inner),
        Expr::Unquote(inner) => v.visit_expr(inner),
        Expr::QuasiQuote(inner) => v.visit_expr(inner),
        Expr::Eval(inner) => v.visit_expr(inner),
        Expr::Reflect(ty) => v.visit_type_expr(ty),
        Expr::IdiomBracket { func, args } => {
            v.visit_expr(func);
            for arg in args {
                v.visit_expr(arg);
            }
        }
        Expr::Forall(forall_expr) => {
            v.visit_type_expr(&mut forall_expr.type_);
            v.visit_expr(&mut forall_expr.body);
        }
        Expr::Exists(exists_expr) => {
            v.visit_type_expr(&mut exists_expr.type_);
            v.visit_expr(&mut exists_expr.body);
        }
        Expr::Implies { left, right, .. } => {
            v.visit_expr(left);
            v.visit_expr(right);
        }
        Expr::SexBlock {
            statements,
            final_expr,
        } => {
            for stmt in statements {
                v.visit_stmt(stmt);
            }
            if let Some(e) = final_expr {
                v.visit_expr(e);
            }
        }
        Expr::Literal(_) | Expr::Identifier(_) => {}
        Expr::List(elements) => {
            for elem in elements {
                v.visit_expr(elem);
            }
        }
    }
}

fn walk_pattern_mut<V: MutVisitor + ?Sized>(v: &mut V, pattern: &mut Pattern) {
    match pattern {
        Pattern::Constructor { fields, .. } => {
            for p in fields {
                v.visit_pattern(p);
            }
        }
        Pattern::Tuple(patterns) => {
            for p in patterns {
                v.visit_pattern(p);
            }
        }
        Pattern::Literal(_) | Pattern::Identifier(_) | Pattern::Wildcard => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ExprCounter {
        count: usize,
    }

    impl Visitor for ExprCounter {
        fn visit_expr(&mut self, expr: &Expr) {
            self.count += 1;
            walk_expr(self, expr);
        }
    }

    #[test]
    fn test_visitor_counts_expressions() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Int(2))),
                op: BinaryOp::Mul,
                right: Box::new(Expr::Literal(Literal::Int(3))),
            }),
        };

        let mut counter = ExprCounter { count: 0 };
        counter.visit_expr(&expr);

        // 1 + (2 * 3) = 5 expressions total
        assert_eq!(counter.count, 5);
    }

    struct LiteralDoubler;

    impl MutVisitor for LiteralDoubler {
        fn visit_expr(&mut self, expr: &mut Expr) {
            if let Expr::Literal(Literal::Int(n)) = expr {
                *n *= 2;
            }
            walk_expr_mut(self, expr);
        }
    }

    #[test]
    fn test_mut_visitor_transforms() {
        let mut expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(5))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(10))),
        };

        let mut doubler = LiteralDoubler;
        doubler.visit_expr(&mut expr);

        match expr {
            Expr::Binary { left, right, .. } => {
                assert_eq!(*left, Expr::Literal(Literal::Int(10)));
                assert_eq!(*right, Expr::Literal(Literal::Int(20)));
            }
            _ => panic!("Expected binary expression"),
        }
    }
}
