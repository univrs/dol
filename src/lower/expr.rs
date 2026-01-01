//! Expression lowering
//!
//! Converts AST expressions to HIR expressions with desugaring.
//!
//! # Desugaring Rules
//!
//! ## Pipe Operator
//! - `x |> f |> g` -> `g(f(x))`
//!
//! ## Short-Circuit Operators
//! - `a && b` -> `if a { b } else { false }`
//! - `a || b` -> `if a { true } else { b }`
//!
//! ## Idiom Brackets
//! - `[| f a b |]` -> `f <$> a <*> b`

use super::LoweringContext;
use crate::ast;
use crate::hir::*;

impl LoweringContext {
    /// Lower an expression (placeholder)
    ///
    /// This will eventually convert AST expressions to HIR expressions,
    /// applying all desugaring rules.
    pub fn lower_expr(&mut self) -> HirExpr {
        HirExpr::Literal(HirLiteral::Unit)
    }

    /// Lower an AST expression to HIR expression
    ///
    /// This is the main entry point for expression lowering.
    /// It handles all AST expression types and applies desugaring.
    pub fn lower_ast_expr(&mut self, expr: &ast::Expr) -> HirExpr {
        match expr {
            ast::Expr::Literal(lit) => HirExpr::Literal(self.lower_literal(lit)),

            ast::Expr::Identifier(name) => HirExpr::Var(self.intern(name)),

            ast::Expr::Binary { left, op, right } => self.lower_binary_expr(left, op, right),

            ast::Expr::Unary { op, operand } => self.lower_unary_expr(op, operand),

            ast::Expr::Call { callee, args } => {
                let func = self.lower_ast_expr(callee);
                let lowered_args: Vec<HirExpr> =
                    args.iter().map(|a| self.lower_ast_expr(a)).collect();
                HirExpr::Call(Box::new(HirCallExpr {
                    func,
                    args: lowered_args,
                }))
            }

            ast::Expr::Member { object, field } => {
                let base = self.lower_ast_expr(object);
                HirExpr::Field(Box::new(HirFieldExpr {
                    base,
                    field: self.intern(field),
                }))
            }

            ast::Expr::Lambda {
                params,
                return_type,
                body,
            } => {
                let hir_params: Vec<HirParam> = params
                    .iter()
                    .map(|(name, type_ann)| {
                        let ty = type_ann
                            .as_ref()
                            .map(|t| self.lower_type_expr(t))
                            .unwrap_or(HirType::Error);
                        HirParam {
                            pat: HirPat::Var(self.intern(name)),
                            ty,
                        }
                    })
                    .collect();
                let ret_type = return_type.as_ref().map(|t| self.lower_type_expr(t));
                let hir_body = self.lower_ast_expr(body);

                HirExpr::Lambda(Box::new(HirLambdaExpr {
                    params: hir_params,
                    return_type: ret_type,
                    body: hir_body,
                }))
            }

            ast::Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.lower_ast_expr(condition);
                let then_expr = self.lower_ast_expr(then_branch);
                let else_expr = else_branch.as_ref().map(|e| self.lower_ast_expr(e));

                HirExpr::If(Box::new(HirIfExpr {
                    cond,
                    then_branch: then_expr,
                    else_branch: else_expr,
                }))
            }

            ast::Expr::Match { scrutinee, arms } => {
                let scrut = self.lower_ast_expr(scrutinee);
                let hir_arms: Vec<HirMatchArm> =
                    arms.iter().map(|arm| self.lower_match_arm(arm)).collect();

                HirExpr::Match(Box::new(HirMatchExpr {
                    scrutinee: scrut,
                    arms: hir_arms,
                }))
            }

            ast::Expr::Block {
                statements,
                final_expr,
            } => {
                let stmts: Vec<HirStmt> = statements
                    .iter()
                    .map(|s| self.lower_block_stmt(s))
                    .collect();
                let expr = final_expr.as_ref().map(|e| self.lower_ast_expr(e));

                HirExpr::Block(Box::new(HirBlockExpr { stmts, expr }))
            }

            ast::Expr::List(items) => {
                // Lower list to a call to a 'list' constructor
                let args: Vec<HirExpr> = items.iter().map(|e| self.lower_ast_expr(e)).collect();
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("list")),
                    args,
                }))
            }

            ast::Expr::Tuple(items) => {
                // Lower tuple to a block with the last item as the expression
                // For proper tuple support, we'd need HirExpr::Tuple
                if items.is_empty() {
                    HirExpr::Literal(HirLiteral::Unit)
                } else {
                    // Create a call to 'tuple' constructor
                    let args: Vec<HirExpr> = items.iter().map(|e| self.lower_ast_expr(e)).collect();
                    HirExpr::Call(Box::new(HirCallExpr {
                        func: HirExpr::Var(self.intern("tuple")),
                        args,
                    }))
                }
            }

            ast::Expr::Quote(inner) => {
                // Quote preserves the expression structure
                // For now, just lower it and wrap in a call
                let inner_expr = self.lower_ast_expr(inner);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("quote")),
                    args: vec![inner_expr],
                }))
            }

            ast::Expr::Unquote(inner) => {
                let inner_expr = self.lower_ast_expr(inner);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("unquote")),
                    args: vec![inner_expr],
                }))
            }

            ast::Expr::QuasiQuote(inner) => {
                let inner_expr = self.lower_ast_expr(inner);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("quasiquote")),
                    args: vec![inner_expr],
                }))
            }

            ast::Expr::Eval(inner) => {
                let inner_expr = self.lower_ast_expr(inner);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("eval")),
                    args: vec![inner_expr],
                }))
            }

            ast::Expr::Reflect(type_expr) => {
                // Type reflection becomes a call to 'reflect'
                let type_str = format!("{:?}", type_expr);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("reflect")),
                    args: vec![HirExpr::Literal(HirLiteral::String(type_str))],
                }))
            }

            ast::Expr::IdiomBracket { func, args } => {
                // [| f a b |] -> f <$> a <*> b
                // Desugar to nested applicative apply calls
                self.desugar_idiom_bracket(func, args)
            }

            ast::Expr::Forall(forall) => {
                // Lower forall to a call
                let body = self.lower_ast_expr(&forall.body);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("forall")),
                    args: vec![body],
                }))
            }

            ast::Expr::Exists(exists) => {
                // Lower exists to a call
                let body = self.lower_ast_expr(&exists.body);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("exists")),
                    args: vec![body],
                }))
            }

            ast::Expr::Implies { left, right, .. } => {
                // a => b becomes if a { b } else { true }
                let left_expr = self.lower_ast_expr(left);
                let right_expr = self.lower_ast_expr(right);
                HirExpr::If(Box::new(HirIfExpr {
                    cond: left_expr,
                    then_branch: right_expr,
                    else_branch: Some(HirExpr::Literal(HirLiteral::Bool(true))),
                }))
            }

            ast::Expr::SexBlock {
                statements,
                final_expr,
            } => {
                // Sex blocks are like regular blocks but may have side effects
                let stmts: Vec<HirStmt> = statements
                    .iter()
                    .map(|s| self.lower_block_stmt(s))
                    .collect();
                let expr = final_expr.as_ref().map(|e| self.lower_ast_expr(e));

                HirExpr::Block(Box::new(HirBlockExpr { stmts, expr }))
            }

            ast::Expr::Cast { expr, target_type } => {
                // Lower cast as a call to 'cast' with type info
                let inner = self.lower_ast_expr(expr);
                let type_name = match target_type {
                    ast::TypeExpr::Named(name) => name.clone(),
                    _ => format!("{:?}", target_type),
                };
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("cast")),
                    args: vec![inner, HirExpr::Literal(HirLiteral::String(type_name))],
                }))
            }

            ast::Expr::StructLiteral { type_name, fields } => {
                // Lower struct literal as a call to the type constructor with field arguments
                let field_exprs: Vec<HirExpr> = fields
                    .iter()
                    .map(|(_, expr)| self.lower_ast_expr(expr))
                    .collect();
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern(type_name)),
                    args: field_exprs,
                }))
            }

            ast::Expr::Try(inner) => {
                // Lower try (?) operator as a call to 'try'
                let inner_expr = self.lower_ast_expr(inner);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("try")),
                    args: vec![inner_expr],
                }))
            }
        }
    }

    /// Lower a binary expression, handling desugaring for special operators
    fn lower_binary_expr(
        &mut self,
        left: &ast::Expr,
        op: &ast::BinaryOp,
        right: &ast::Expr,
    ) -> HirExpr {
        match op {
            // Pipe: x |> f -> f(x)
            ast::BinaryOp::Pipe => {
                let arg = self.lower_ast_expr(left);
                let func = self.lower_ast_expr(right);
                HirExpr::Call(Box::new(HirCallExpr {
                    func,
                    args: vec![arg],
                }))
            }

            // Compose: f >> g -> \x. g(f(x))
            ast::BinaryOp::Compose => {
                let f = self.lower_ast_expr(left);
                let g = self.lower_ast_expr(right);
                // Create a lambda that composes the two functions
                let x = self.intern("x");
                let inner_call = HirExpr::Call(Box::new(HirCallExpr {
                    func: f,
                    args: vec![HirExpr::Var(x)],
                }));
                let outer_call = HirExpr::Call(Box::new(HirCallExpr {
                    func: g,
                    args: vec![inner_call],
                }));
                HirExpr::Lambda(Box::new(HirLambdaExpr {
                    params: vec![HirParam {
                        pat: HirPat::Var(x),
                        ty: HirType::Error,
                    }],
                    return_type: None,
                    body: outer_call,
                }))
            }

            // Member access: a.b
            ast::BinaryOp::Member => {
                let base = self.lower_ast_expr(left);
                // Right side should be an identifier
                let field = match right {
                    ast::Expr::Identifier(name) => self.intern(name),
                    _ => self.intern("unknown"),
                };
                HirExpr::Field(Box::new(HirFieldExpr { base, field }))
            }

            // Map functor: f <$> a
            ast::BinaryOp::Map => {
                let func = self.lower_ast_expr(left);
                let arg = self.lower_ast_expr(right);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("fmap")),
                    args: vec![func, arg],
                }))
            }

            // Applicative apply: f <*> a
            ast::BinaryOp::Ap => {
                let func = self.lower_ast_expr(left);
                let arg = self.lower_ast_expr(right);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("ap")),
                    args: vec![func, arg],
                }))
            }

            // Bind: a := b (monadic bind)
            ast::BinaryOp::Bind => {
                let ma = self.lower_ast_expr(left);
                let f = self.lower_ast_expr(right);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("bind")),
                    args: vec![ma, f],
                }))
            }

            // Apply: f @ a
            ast::BinaryOp::Apply => {
                let func = self.lower_ast_expr(left);
                let arg = self.lower_ast_expr(right);
                HirExpr::Call(Box::new(HirCallExpr {
                    func,
                    args: vec![arg],
                }))
            }

            // Range: a..b
            ast::BinaryOp::Range => {
                let start = self.lower_ast_expr(left);
                let end = self.lower_ast_expr(right);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("range")),
                    args: vec![start, end],
                }))
            }

            // Power: a ^ b
            ast::BinaryOp::Pow => {
                let base = self.lower_ast_expr(left);
                let exp = self.lower_ast_expr(right);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("pow")),
                    args: vec![base, exp],
                }))
            }

            // Implies: a => b -> if a { b } else { true }
            ast::BinaryOp::Implies => {
                let left_expr = self.lower_ast_expr(left);
                let right_expr = self.lower_ast_expr(right);
                HirExpr::If(Box::new(HirIfExpr {
                    cond: left_expr,
                    then_branch: right_expr,
                    else_branch: Some(HirExpr::Literal(HirLiteral::Bool(true))),
                }))
            }

            // Standard binary operators
            _ => {
                if let Some(hir_op) = self.lower_binary_op(op) {
                    let left_expr = self.lower_ast_expr(left);
                    let right_expr = self.lower_ast_expr(right);
                    HirExpr::Binary(Box::new(HirBinaryExpr {
                        left: left_expr,
                        op: hir_op,
                        right: right_expr,
                    }))
                } else {
                    // Fallback for unsupported operators
                    self.emit_warning(&format!("Unsupported binary operator: {:?}", op), None);
                    HirExpr::Literal(HirLiteral::Unit)
                }
            }
        }
    }

    /// Lower a unary expression
    fn lower_unary_expr(&mut self, op: &ast::UnaryOp, operand: &ast::Expr) -> HirExpr {
        match op {
            ast::UnaryOp::Neg | ast::UnaryOp::Not => {
                if let Some(hir_op) = self.lower_unary_op(op) {
                    let hir_operand = self.lower_ast_expr(operand);
                    HirExpr::Unary(Box::new(HirUnaryExpr {
                        op: hir_op,
                        operand: hir_operand,
                    }))
                } else {
                    HirExpr::Literal(HirLiteral::Unit)
                }
            }
            ast::UnaryOp::Quote => {
                let inner = self.lower_ast_expr(operand);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("quote")),
                    args: vec![inner],
                }))
            }
            ast::UnaryOp::Reflect => {
                let inner = self.lower_ast_expr(operand);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("reflect")),
                    args: vec![inner],
                }))
            }
            ast::UnaryOp::Deref => {
                let inner = self.lower_ast_expr(operand);
                HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("deref")),
                    args: vec![inner],
                }))
            }
        }
    }

    /// Desugar idiom brackets: [| f a b |] -> f <$> a <*> b
    fn desugar_idiom_bracket(&mut self, func: &ast::Expr, args: &[ast::Expr]) -> HirExpr {
        if args.is_empty() {
            return self.lower_ast_expr(func);
        }

        let f = self.lower_ast_expr(func);
        let mut result = f;

        for (i, arg) in args.iter().enumerate() {
            let a = self.lower_ast_expr(arg);
            if i == 0 {
                // First arg: f <$> a
                result = HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("fmap")),
                    args: vec![result, a],
                }));
            } else {
                // Subsequent args: ... <*> a
                result = HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("ap")),
                    args: vec![result, a],
                }));
            }
        }

        result
    }

    /// Lower a match arm
    fn lower_match_arm(&mut self, arm: &ast::MatchArm) -> HirMatchArm {
        let pat = self.lower_pattern(&arm.pattern);
        let guard = arm.guard.as_ref().map(|g| self.lower_ast_expr(g));
        let body = self.lower_ast_expr(&arm.body);

        HirMatchArm { pat, guard, body }
    }

    /// Lower a pattern
    pub fn lower_pattern(&mut self, pattern: &ast::Pattern) -> HirPat {
        match pattern {
            ast::Pattern::Wildcard => HirPat::Wildcard,
            ast::Pattern::Identifier(name) => HirPat::Var(self.intern(name)),
            ast::Pattern::Literal(lit) => HirPat::Literal(self.lower_literal(lit)),
            ast::Pattern::Constructor { name, fields } => {
                let hir_fields: Vec<HirPat> =
                    fields.iter().map(|p| self.lower_pattern(p)).collect();
                HirPat::Constructor(HirConstructorPat {
                    name: self.intern(name),
                    fields: hir_fields,
                })
            }
            ast::Pattern::Tuple(patterns) => {
                HirPat::Tuple(patterns.iter().map(|p| self.lower_pattern(p)).collect())
            }
            ast::Pattern::Or(patterns) => {
                HirPat::Or(patterns.iter().map(|p| self.lower_pattern(p)).collect())
            }
        }
    }

    /// Lower a block statement (from function bodies)
    pub fn lower_block_stmt(&mut self, stmt: &ast::Stmt) -> HirStmt {
        match stmt {
            ast::Stmt::Let {
                name,
                type_ann,
                value,
            } => {
                let ty = type_ann.as_ref().map(|t| self.lower_type_expr(t));
                let init = self.lower_ast_expr(value);
                HirStmt::Val(HirValStmt {
                    pat: HirPat::Var(self.intern(name)),
                    ty,
                    init,
                })
            }

            ast::Stmt::Assign { target, value } => {
                let lhs = self.lower_ast_expr(target);
                let rhs = self.lower_ast_expr(value);
                HirStmt::Assign(HirAssignStmt { lhs, rhs })
            }

            ast::Stmt::For {
                binding,
                iterable,
                body,
            } => {
                // Desugar for loop to loop + match
                // for x in xs { body } -> loop { match iter.next() { Some(x) => body, None => break } }
                let iter_expr = self.lower_ast_expr(iterable);
                let body_stmts: Vec<HirStmt> =
                    body.iter().map(|s| self.lower_block_stmt(s)).collect();

                // Create the match arms
                let some_pat = HirPat::Constructor(HirConstructorPat {
                    name: self.intern("Some"),
                    fields: vec![HirPat::Var(self.intern(binding))],
                });
                let some_arm = HirMatchArm {
                    pat: some_pat,
                    guard: None,
                    body: HirExpr::Block(Box::new(HirBlockExpr {
                        stmts: body_stmts,
                        expr: None,
                    })),
                };

                let none_arm = HirMatchArm {
                    pat: HirPat::Constructor(HirConstructorPat {
                        name: self.intern("None"),
                        fields: vec![],
                    }),
                    guard: None,
                    body: HirExpr::Block(Box::new(HirBlockExpr {
                        stmts: vec![HirStmt::Break(None)],
                        expr: None,
                    })),
                };

                // Create the loop expression
                // This is simplified - a real implementation would need an iterator variable
                let match_expr = HirExpr::Match(Box::new(HirMatchExpr {
                    scrutinee: HirExpr::Call(Box::new(HirCallExpr {
                        func: HirExpr::Var(self.intern("next")),
                        args: vec![iter_expr],
                    })),
                    arms: vec![some_arm, none_arm],
                }));

                HirStmt::Expr(HirExpr::Block(Box::new(HirBlockExpr {
                    stmts: vec![HirStmt::Expr(match_expr)],
                    expr: None,
                })))
            }

            ast::Stmt::While { condition, body } => {
                // Desugar while to loop + if
                // while cond { body } -> loop { if cond { body } else { break } }
                let cond = self.lower_ast_expr(condition);
                let body_stmts: Vec<HirStmt> =
                    body.iter().map(|s| self.lower_block_stmt(s)).collect();

                let if_expr = HirExpr::If(Box::new(HirIfExpr {
                    cond,
                    then_branch: HirExpr::Block(Box::new(HirBlockExpr {
                        stmts: body_stmts,
                        expr: None,
                    })),
                    else_branch: Some(HirExpr::Block(Box::new(HirBlockExpr {
                        stmts: vec![HirStmt::Break(None)],
                        expr: None,
                    }))),
                }));

                HirStmt::Expr(HirExpr::Block(Box::new(HirBlockExpr {
                    stmts: vec![HirStmt::Expr(if_expr)],
                    expr: None,
                })))
            }

            ast::Stmt::Loop { body } => {
                let body_stmts: Vec<HirStmt> =
                    body.iter().map(|s| self.lower_block_stmt(s)).collect();

                // Loop is represented as a block that loops
                HirStmt::Expr(HirExpr::Block(Box::new(HirBlockExpr {
                    stmts: body_stmts,
                    expr: None,
                })))
            }

            ast::Stmt::Break => HirStmt::Break(None),

            ast::Stmt::Continue => {
                // Continue is lowered as a special marker
                // For now, represent as a call to 'continue'
                HirStmt::Expr(HirExpr::Call(Box::new(HirCallExpr {
                    func: HirExpr::Var(self.intern("continue")),
                    args: vec![],
                })))
            }

            ast::Stmt::Return(expr) => {
                let hir_expr = expr.as_ref().map(|e| self.lower_ast_expr(e));
                HirStmt::Return(hir_expr)
            }

            ast::Stmt::Expr(expr) => HirStmt::Expr(self.lower_ast_expr(expr)),
        }
    }

    /// Lower a literal value
    pub fn lower_literal(&mut self, lit: &crate::ast::Literal) -> HirLiteral {
        match lit {
            crate::ast::Literal::Int(i) => HirLiteral::Int(*i),
            crate::ast::Literal::Float(f) => HirLiteral::Float(*f),
            crate::ast::Literal::String(s) => HirLiteral::String(s.clone()),
            crate::ast::Literal::Bool(b) => HirLiteral::Bool(*b),
            crate::ast::Literal::Char(c) => HirLiteral::String(c.to_string()),
            crate::ast::Literal::Null => HirLiteral::Unit,
        }
    }

    /// Lower a binary operator
    pub fn lower_binary_op(&mut self, op: &crate::ast::BinaryOp) -> Option<HirBinaryOp> {
        match op {
            crate::ast::BinaryOp::Add => Some(HirBinaryOp::Add),
            crate::ast::BinaryOp::Sub => Some(HirBinaryOp::Sub),
            crate::ast::BinaryOp::Mul => Some(HirBinaryOp::Mul),
            crate::ast::BinaryOp::Div => Some(HirBinaryOp::Div),
            crate::ast::BinaryOp::Mod => Some(HirBinaryOp::Mod),
            crate::ast::BinaryOp::Eq => Some(HirBinaryOp::Eq),
            crate::ast::BinaryOp::Ne => Some(HirBinaryOp::Ne),
            crate::ast::BinaryOp::Lt => Some(HirBinaryOp::Lt),
            crate::ast::BinaryOp::Le => Some(HirBinaryOp::Le),
            crate::ast::BinaryOp::Gt => Some(HirBinaryOp::Gt),
            crate::ast::BinaryOp::Ge => Some(HirBinaryOp::Ge),
            crate::ast::BinaryOp::And => Some(HirBinaryOp::And),
            crate::ast::BinaryOp::Or => Some(HirBinaryOp::Or),
            // These operators require desugaring and don't have direct HIR equivalents
            crate::ast::BinaryOp::Pipe
            | crate::ast::BinaryOp::Compose
            | crate::ast::BinaryOp::Apply
            | crate::ast::BinaryOp::Bind
            | crate::ast::BinaryOp::Member
            | crate::ast::BinaryOp::Map
            | crate::ast::BinaryOp::Ap
            | crate::ast::BinaryOp::Implies
            | crate::ast::BinaryOp::Range
            | crate::ast::BinaryOp::Pow => None,
        }
    }

    /// Lower a unary operator
    pub fn lower_unary_op(&mut self, op: &crate::ast::UnaryOp) -> Option<HirUnaryOp> {
        match op {
            crate::ast::UnaryOp::Neg => Some(HirUnaryOp::Neg),
            crate::ast::UnaryOp::Not => Some(HirUnaryOp::Not),
            // These operators require special handling
            crate::ast::UnaryOp::Quote
            | crate::ast::UnaryOp::Reflect
            | crate::ast::UnaryOp::Deref => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;

    #[test]
    fn test_lower_literal_int() {
        let mut ctx = LoweringContext::new();
        let lit = ast::Literal::Int(42);
        let hir_lit = ctx.lower_literal(&lit);
        assert_eq!(hir_lit, HirLiteral::Int(42));
    }

    #[test]
    fn test_lower_literal_bool() {
        let mut ctx = LoweringContext::new();
        let lit = ast::Literal::Bool(true);
        let hir_lit = ctx.lower_literal(&lit);
        assert_eq!(hir_lit, HirLiteral::Bool(true));
    }

    #[test]
    fn test_lower_literal_string() {
        let mut ctx = LoweringContext::new();
        let lit = ast::Literal::String("hello".to_string());
        let hir_lit = ctx.lower_literal(&lit);
        assert_eq!(hir_lit, HirLiteral::String("hello".to_string()));
    }

    #[test]
    fn test_lower_binary_op() {
        let mut ctx = LoweringContext::new();
        assert_eq!(
            ctx.lower_binary_op(&ast::BinaryOp::Add),
            Some(HirBinaryOp::Add)
        );
        assert_eq!(
            ctx.lower_binary_op(&ast::BinaryOp::Eq),
            Some(HirBinaryOp::Eq)
        );
        // Pipe requires desugaring
        assert_eq!(ctx.lower_binary_op(&ast::BinaryOp::Pipe), None);
    }

    #[test]
    fn test_lower_unary_op() {
        let mut ctx = LoweringContext::new();
        assert_eq!(
            ctx.lower_unary_op(&ast::UnaryOp::Neg),
            Some(HirUnaryOp::Neg)
        );
        assert_eq!(
            ctx.lower_unary_op(&ast::UnaryOp::Not),
            Some(HirUnaryOp::Not)
        );
        // Quote requires special handling
        assert_eq!(ctx.lower_unary_op(&ast::UnaryOp::Quote), None);
    }

    #[test]
    fn test_lower_ast_expr_literal() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Literal(ast::Literal::Int(42));
        let hir = ctx.lower_ast_expr(&expr);
        assert_eq!(hir, HirExpr::Literal(HirLiteral::Int(42)));
    }

    #[test]
    fn test_lower_ast_expr_identifier() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Identifier("foo".to_string());
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::Var(sym) => {
                assert_eq!(ctx.resolve(sym), Some("foo"));
            }
            _ => panic!("Expected Var"),
        }
    }

    #[test]
    fn test_lower_ast_expr_binary_add() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Binary {
            left: Box::new(ast::Expr::Literal(ast::Literal::Int(1))),
            op: ast::BinaryOp::Add,
            right: Box::new(ast::Expr::Literal(ast::Literal::Int(2))),
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::Binary(bin) => {
                assert_eq!(bin.op, HirBinaryOp::Add);
                assert_eq!(bin.left, HirExpr::Literal(HirLiteral::Int(1)));
                assert_eq!(bin.right, HirExpr::Literal(HirLiteral::Int(2)));
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_lower_ast_expr_pipe() {
        // x |> f -> f(x)
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Binary {
            left: Box::new(ast::Expr::Identifier("x".to_string())),
            op: ast::BinaryOp::Pipe,
            right: Box::new(ast::Expr::Identifier("f".to_string())),
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::Call(call) => {
                // f should be the function
                match &call.func {
                    HirExpr::Var(sym) => {
                        assert_eq!(ctx.resolve(*sym), Some("f"));
                    }
                    _ => panic!("Expected Var for function"),
                }
                // x should be the argument
                assert_eq!(call.args.len(), 1);
                match &call.args[0] {
                    HirExpr::Var(sym) => {
                        assert_eq!(ctx.resolve(*sym), Some("x"));
                    }
                    _ => panic!("Expected Var for argument"),
                }
            }
            _ => panic!("Expected Call from pipe desugaring"),
        }
    }

    #[test]
    fn test_lower_ast_expr_call() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Call {
            callee: Box::new(ast::Expr::Identifier("print".to_string())),
            args: vec![ast::Expr::Literal(ast::Literal::String(
                "hello".to_string(),
            ))],
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::Call(call) => {
                match &call.func {
                    HirExpr::Var(sym) => {
                        assert_eq!(ctx.resolve(*sym), Some("print"));
                    }
                    _ => panic!("Expected Var for function"),
                }
                assert_eq!(call.args.len(), 1);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_lower_ast_expr_if() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::Literal(ast::Literal::Bool(true))),
            then_branch: Box::new(ast::Expr::Literal(ast::Literal::Int(1))),
            else_branch: Some(Box::new(ast::Expr::Literal(ast::Literal::Int(0)))),
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::If(if_expr) => {
                assert_eq!(if_expr.cond, HirExpr::Literal(HirLiteral::Bool(true)));
                assert_eq!(if_expr.then_branch, HirExpr::Literal(HirLiteral::Int(1)));
                assert_eq!(
                    if_expr.else_branch,
                    Some(HirExpr::Literal(HirLiteral::Int(0)))
                );
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_lower_ast_expr_unary_neg() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Unary {
            op: ast::UnaryOp::Neg,
            operand: Box::new(ast::Expr::Literal(ast::Literal::Int(42))),
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::Unary(unary) => {
                assert_eq!(unary.op, HirUnaryOp::Neg);
                assert_eq!(unary.operand, HirExpr::Literal(HirLiteral::Int(42)));
            }
            _ => panic!("Expected Unary"),
        }
    }

    #[test]
    fn test_lower_pattern_wildcard() {
        let mut ctx = LoweringContext::new();
        let pat = ast::Pattern::Wildcard;
        let hir = ctx.lower_pattern(&pat);
        assert_eq!(hir, HirPat::Wildcard);
    }

    #[test]
    fn test_lower_pattern_identifier() {
        let mut ctx = LoweringContext::new();
        let pat = ast::Pattern::Identifier("x".to_string());
        let hir = ctx.lower_pattern(&pat);
        match hir {
            HirPat::Var(sym) => {
                assert_eq!(ctx.resolve(sym), Some("x"));
            }
            _ => panic!("Expected Var pattern"),
        }
    }

    #[test]
    fn test_lower_pattern_literal() {
        let mut ctx = LoweringContext::new();
        let pat = ast::Pattern::Literal(ast::Literal::Int(42));
        let hir = ctx.lower_pattern(&pat);
        assert_eq!(hir, HirPat::Literal(HirLiteral::Int(42)));
    }

    #[test]
    fn test_lower_block_stmt_let() {
        let mut ctx = LoweringContext::new();
        let stmt = ast::Stmt::Let {
            name: "x".to_string(),
            type_ann: None,
            value: ast::Expr::Literal(ast::Literal::Int(42)),
        };
        let hir = ctx.lower_block_stmt(&stmt);
        match hir {
            HirStmt::Val(val) => {
                assert_eq!(val.init, HirExpr::Literal(HirLiteral::Int(42)));
            }
            _ => panic!("Expected Val statement"),
        }
    }

    #[test]
    fn test_lower_block_stmt_return() {
        let mut ctx = LoweringContext::new();
        let stmt = ast::Stmt::Return(Some(ast::Expr::Literal(ast::Literal::Int(42))));
        let hir = ctx.lower_block_stmt(&stmt);
        match hir {
            HirStmt::Return(Some(expr)) => {
                assert_eq!(expr, HirExpr::Literal(HirLiteral::Int(42)));
            }
            _ => panic!("Expected Return statement"),
        }
    }

    #[test]
    fn test_lower_block_stmt_break() {
        let mut ctx = LoweringContext::new();
        let stmt = ast::Stmt::Break;
        let hir = ctx.lower_block_stmt(&stmt);
        match hir {
            HirStmt::Break(None) => {}
            _ => panic!("Expected Break statement"),
        }
    }

    #[test]
    fn test_lower_ast_expr_member() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Member {
            object: Box::new(ast::Expr::Identifier("obj".to_string())),
            field: "prop".to_string(),
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::Field(field) => {
                match &field.base {
                    HirExpr::Var(sym) => {
                        assert_eq!(ctx.resolve(*sym), Some("obj"));
                    }
                    _ => panic!("Expected Var for base"),
                }
                assert_eq!(ctx.resolve(field.field), Some("prop"));
            }
            _ => panic!("Expected Field"),
        }
    }

    #[test]
    fn test_desugar_idiom_bracket() {
        // [| f a b |] -> fmap(fmap(f, a), b) via ap
        let mut ctx = LoweringContext::new();
        let func = ast::Expr::Identifier("f".to_string());
        let args = vec![
            ast::Expr::Identifier("a".to_string()),
            ast::Expr::Identifier("b".to_string()),
        ];
        let hir = ctx.desugar_idiom_bracket(&func, &args);
        // Should be: ap(fmap(f, a), b)
        match hir {
            HirExpr::Call(call) => {
                // Outer call should be 'ap'
                match &call.func {
                    HirExpr::Var(sym) => {
                        assert_eq!(ctx.resolve(*sym), Some("ap"));
                    }
                    _ => panic!("Expected Var for ap"),
                }
                assert_eq!(call.args.len(), 2);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_lower_ast_expr_implies() {
        // a => b becomes if a { b } else { true }
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Binary {
            left: Box::new(ast::Expr::Identifier("a".to_string())),
            op: ast::BinaryOp::Implies,
            right: Box::new(ast::Expr::Identifier("b".to_string())),
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::If(if_expr) => {
                match &if_expr.cond {
                    HirExpr::Var(sym) => {
                        assert_eq!(ctx.resolve(*sym), Some("a"));
                    }
                    _ => panic!("Expected Var for condition"),
                }
                match &if_expr.then_branch {
                    HirExpr::Var(sym) => {
                        assert_eq!(ctx.resolve(*sym), Some("b"));
                    }
                    _ => panic!("Expected Var for then branch"),
                }
                assert_eq!(
                    if_expr.else_branch,
                    Some(HirExpr::Literal(HirLiteral::Bool(true)))
                );
            }
            _ => panic!("Expected If from implies desugaring"),
        }
    }

    #[test]
    fn test_lower_ast_expr_block() {
        let mut ctx = LoweringContext::new();
        let expr = ast::Expr::Block {
            statements: vec![ast::Stmt::Let {
                name: "x".to_string(),
                type_ann: None,
                value: ast::Expr::Literal(ast::Literal::Int(1)),
            }],
            final_expr: Some(Box::new(ast::Expr::Identifier("x".to_string()))),
        };
        let hir = ctx.lower_ast_expr(&expr);
        match hir {
            HirExpr::Block(block) => {
                assert_eq!(block.stmts.len(), 1);
                assert!(block.expr.is_some());
            }
            _ => panic!("Expected Block"),
        }
    }
}
