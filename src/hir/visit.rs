//! Visitor pattern for HIR traversal.
//!
//! This module provides the [`HirVisitor`] trait for walking HIR trees.

use super::types::*;

/// Visitor trait for HIR traversal.
///
/// Implement this trait to walk the HIR and perform analysis or transformations.
/// The default implementations simply recurse into child nodes.
pub trait HirVisitor: Sized {
    /// Visit a module.
    fn visit_module(&mut self, module: &HirModule) {
        walk_module(self, module);
    }

    /// Visit a declaration.
    fn visit_decl(&mut self, decl: &HirDecl) {
        walk_decl(self, decl);
    }

    /// Visit a type declaration.
    fn visit_type_decl(&mut self, decl: &HirTypeDecl) {
        walk_type_decl(self, decl);
    }

    /// Visit a trait declaration.
    fn visit_trait_decl(&mut self, decl: &HirTraitDecl) {
        walk_trait_decl(self, decl);
    }

    /// Visit a function declaration.
    fn visit_function_decl(&mut self, decl: &HirFunctionDecl) {
        walk_function_decl(self, decl);
    }

    /// Visit a module declaration.
    fn visit_module_decl(&mut self, decl: &HirModuleDecl) {
        walk_module_decl(self, decl);
    }

    /// Visit an expression.
    fn visit_expr(&mut self, expr: &HirExpr) {
        walk_expr(self, expr);
    }

    /// Visit a statement.
    fn visit_stmt(&mut self, stmt: &HirStmt) {
        walk_stmt(self, stmt);
    }

    /// Visit a type.
    fn visit_type(&mut self, ty: &HirType) {
        walk_type(self, ty);
    }

    /// Visit a pattern.
    fn visit_pat(&mut self, pat: &HirPat) {
        walk_pat(self, pat);
    }
}

/// Walk a module's children.
pub fn walk_module<V: HirVisitor>(visitor: &mut V, module: &HirModule) {
    for decl in &module.decls {
        visitor.visit_decl(decl);
    }
}

/// Walk a declaration's children.
pub fn walk_decl<V: HirVisitor>(visitor: &mut V, decl: &HirDecl) {
    match decl {
        HirDecl::Type(ty) => visitor.visit_type_decl(ty),
        HirDecl::Trait(tr) => visitor.visit_trait_decl(tr),
        HirDecl::Function(func) => visitor.visit_function_decl(func),
        HirDecl::Module(module) => visitor.visit_module_decl(module),
    }
}

/// Walk a type declaration's children.
pub fn walk_type_decl<V: HirVisitor>(visitor: &mut V, decl: &HirTypeDecl) {
    for param in &decl.type_params {
        for bound in &param.bounds {
            visitor.visit_type(bound);
        }
    }
    walk_type_def(visitor, &decl.body);
}

/// Walk a type definition's children.
pub fn walk_type_def<V: HirVisitor>(visitor: &mut V, def: &HirTypeDef) {
    match def {
        HirTypeDef::Alias(ty) => visitor.visit_type(ty),
        HirTypeDef::Struct(fields) => {
            for field in fields {
                visitor.visit_type(&field.ty);
            }
        }
        HirTypeDef::Enum(variants) => {
            for variant in variants {
                if let Some(payload) = &variant.payload {
                    visitor.visit_type(payload);
                }
            }
        }
        HirTypeDef::Gene(_statements) => {
            // Gene statements don't contain types to visit
        }
    }
}

/// Walk a trait declaration's children.
pub fn walk_trait_decl<V: HirVisitor>(visitor: &mut V, decl: &HirTraitDecl) {
    for param in &decl.type_params {
        for bound in &param.bounds {
            visitor.visit_type(bound);
        }
    }
    for bound in &decl.bounds {
        visitor.visit_type(bound);
    }
    for item in &decl.items {
        match item {
            HirTraitItem::Method(func) => visitor.visit_function_decl(func),
            HirTraitItem::AssocType(assoc) => {
                for bound in &assoc.bounds {
                    visitor.visit_type(bound);
                }
                if let Some(default) = &assoc.default {
                    visitor.visit_type(default);
                }
            }
        }
    }
}

/// Walk a function declaration's children.
pub fn walk_function_decl<V: HirVisitor>(visitor: &mut V, decl: &HirFunctionDecl) {
    for param in &decl.type_params {
        for bound in &param.bounds {
            visitor.visit_type(bound);
        }
    }
    for param in &decl.params {
        visitor.visit_pat(&param.pat);
        visitor.visit_type(&param.ty);
    }
    visitor.visit_type(&decl.return_type);
    if let Some(body) = &decl.body {
        visitor.visit_expr(body);
    }
}

/// Walk a module declaration's children.
pub fn walk_module_decl<V: HirVisitor>(visitor: &mut V, decl: &HirModuleDecl) {
    for d in &decl.decls {
        visitor.visit_decl(d);
    }
}

/// Walk an expression's children.
pub fn walk_expr<V: HirVisitor>(visitor: &mut V, expr: &HirExpr) {
    match expr {
        HirExpr::Literal(_) | HirExpr::Var(_) => {}
        HirExpr::Binary(bin) => {
            visitor.visit_expr(&bin.left);
            visitor.visit_expr(&bin.right);
        }
        HirExpr::Unary(un) => {
            visitor.visit_expr(&un.operand);
        }
        HirExpr::Call(call) => {
            visitor.visit_expr(&call.func);
            for arg in &call.args {
                visitor.visit_expr(arg);
            }
        }
        HirExpr::MethodCall(call) => {
            visitor.visit_expr(&call.receiver);
            for arg in &call.args {
                visitor.visit_expr(arg);
            }
        }
        HirExpr::Field(field) => {
            visitor.visit_expr(&field.base);
        }
        HirExpr::Index(idx) => {
            visitor.visit_expr(&idx.base);
            visitor.visit_expr(&idx.index);
        }
        HirExpr::Block(block) => {
            for stmt in &block.stmts {
                visitor.visit_stmt(stmt);
            }
            if let Some(expr) = &block.expr {
                visitor.visit_expr(expr);
            }
        }
        HirExpr::If(if_expr) => {
            visitor.visit_expr(&if_expr.cond);
            visitor.visit_expr(&if_expr.then_branch);
            if let Some(else_branch) = &if_expr.else_branch {
                visitor.visit_expr(else_branch);
            }
        }
        HirExpr::Match(match_expr) => {
            visitor.visit_expr(&match_expr.scrutinee);
            for arm in &match_expr.arms {
                visitor.visit_pat(&arm.pat);
                if let Some(guard) = &arm.guard {
                    visitor.visit_expr(guard);
                }
                visitor.visit_expr(&arm.body);
            }
        }
        HirExpr::Lambda(lambda) => {
            for param in &lambda.params {
                visitor.visit_pat(&param.pat);
                visitor.visit_type(&param.ty);
            }
            if let Some(ret) = &lambda.return_type {
                visitor.visit_type(ret);
            }
            visitor.visit_expr(&lambda.body);
        }
        HirExpr::Loop(loop_expr) => {
            visitor.visit_expr(&loop_expr.body);
        }
        HirExpr::While(while_expr) => {
            visitor.visit_expr(&while_expr.cond);
            visitor.visit_expr(&while_expr.body);
        }
        HirExpr::For(for_expr) => {
            visitor.visit_pat(&for_expr.pat);
            visitor.visit_expr(&for_expr.iter);
            visitor.visit_expr(&for_expr.body);
        }
        HirExpr::Tuple(items) | HirExpr::List(items) => {
            for item in items {
                visitor.visit_expr(item);
            }
        }
    }
}

/// Walk a statement's children.
pub fn walk_stmt<V: HirVisitor>(visitor: &mut V, stmt: &HirStmt) {
    match stmt {
        HirStmt::Val(val) => {
            visitor.visit_pat(&val.pat);
            if let Some(ty) = &val.ty {
                visitor.visit_type(ty);
            }
            visitor.visit_expr(&val.init);
        }
        HirStmt::Var(var) => {
            visitor.visit_pat(&var.pat);
            if let Some(ty) = &var.ty {
                visitor.visit_type(ty);
            }
            visitor.visit_expr(&var.init);
        }
        HirStmt::Assign(assign) => {
            visitor.visit_expr(&assign.lhs);
            visitor.visit_expr(&assign.rhs);
        }
        HirStmt::Expr(expr) => {
            visitor.visit_expr(expr);
        }
        HirStmt::Return(ret) => {
            if let Some(expr) = ret {
                visitor.visit_expr(expr);
            }
        }
        HirStmt::Break(brk) => {
            if let Some(expr) = brk {
                visitor.visit_expr(expr);
            }
        }
        HirStmt::Continue => {}
    }
}

/// Walk a type's children.
pub fn walk_type<V: HirVisitor>(visitor: &mut V, ty: &HirType) {
    match ty {
        HirType::Named(named) => {
            for arg in &named.args {
                visitor.visit_type(arg);
            }
        }
        HirType::Tuple(types) => {
            for t in types {
                visitor.visit_type(t);
            }
        }
        HirType::Array(arr) => {
            visitor.visit_type(&arr.elem);
        }
        HirType::Function(func) => {
            for p in &func.params {
                visitor.visit_type(p);
            }
            visitor.visit_type(&func.ret);
        }
        HirType::Ref(r) => {
            visitor.visit_type(&r.ty);
        }
        HirType::Optional(inner) => {
            visitor.visit_type(inner);
        }
        HirType::Var(_) | HirType::Error => {}
    }
}

/// Walk a pattern's children.
pub fn walk_pat<V: HirVisitor>(visitor: &mut V, pat: &HirPat) {
    match pat {
        HirPat::Wildcard | HirPat::Var(_) | HirPat::Literal(_) => {}
        HirPat::Constructor(ctor) => {
            for field in &ctor.fields {
                visitor.visit_pat(field);
            }
        }
        HirPat::Tuple(pats) => {
            for p in pats {
                visitor.visit_pat(p);
            }
        }
        HirPat::Or(pats) => {
            for p in pats {
                visitor.visit_pat(p);
            }
        }
    }
}

/// Mutable visitor trait for HIR transformations.
///
/// Like [`HirVisitor`] but takes mutable references for in-place modifications.
pub trait HirMutVisitor: Sized {
    /// Visit a module mutably.
    fn visit_module_mut(&mut self, module: &mut HirModule) {
        walk_module_mut(self, module);
    }

    /// Visit a declaration mutably.
    fn visit_decl_mut(&mut self, decl: &mut HirDecl) {
        walk_decl_mut(self, decl);
    }

    /// Visit an expression mutably.
    fn visit_expr_mut(&mut self, expr: &mut HirExpr) {
        walk_expr_mut(self, expr);
    }

    /// Visit a statement mutably.
    fn visit_stmt_mut(&mut self, stmt: &mut HirStmt) {
        walk_stmt_mut(self, stmt);
    }

    /// Visit a type mutably.
    fn visit_type_mut(&mut self, ty: &mut HirType) {
        walk_type_mut(self, ty);
    }

    /// Visit a pattern mutably.
    fn visit_pat_mut(&mut self, pat: &mut HirPat) {
        walk_pat_mut(self, pat);
    }
}

/// Walk a module mutably.
pub fn walk_module_mut<V: HirMutVisitor>(visitor: &mut V, module: &mut HirModule) {
    for decl in &mut module.decls {
        visitor.visit_decl_mut(decl);
    }
}

/// Walk a declaration mutably.
pub fn walk_decl_mut<V: HirMutVisitor>(visitor: &mut V, decl: &mut HirDecl) {
    match decl {
        HirDecl::Type(ty) => {
            for param in &mut ty.type_params {
                for bound in &mut param.bounds {
                    visitor.visit_type_mut(bound);
                }
            }
        }
        HirDecl::Trait(tr) => {
            for param in &mut tr.type_params {
                for bound in &mut param.bounds {
                    visitor.visit_type_mut(bound);
                }
            }
            for bound in &mut tr.bounds {
                visitor.visit_type_mut(bound);
            }
        }
        HirDecl::Function(func) => {
            for param in &mut func.params {
                visitor.visit_pat_mut(&mut param.pat);
                visitor.visit_type_mut(&mut param.ty);
            }
            visitor.visit_type_mut(&mut func.return_type);
            if let Some(body) = &mut func.body {
                visitor.visit_expr_mut(body);
            }
        }
        HirDecl::Module(module) => {
            for d in &mut module.decls {
                visitor.visit_decl_mut(d);
            }
        }
    }
}

/// Walk an expression mutably.
pub fn walk_expr_mut<V: HirMutVisitor>(visitor: &mut V, expr: &mut HirExpr) {
    match expr {
        HirExpr::Literal(_) | HirExpr::Var(_) => {}
        HirExpr::Binary(bin) => {
            visitor.visit_expr_mut(&mut bin.left);
            visitor.visit_expr_mut(&mut bin.right);
        }
        HirExpr::Unary(un) => {
            visitor.visit_expr_mut(&mut un.operand);
        }
        HirExpr::Call(call) => {
            visitor.visit_expr_mut(&mut call.func);
            for arg in &mut call.args {
                visitor.visit_expr_mut(arg);
            }
        }
        HirExpr::MethodCall(call) => {
            visitor.visit_expr_mut(&mut call.receiver);
            for arg in &mut call.args {
                visitor.visit_expr_mut(arg);
            }
        }
        HirExpr::Field(field) => {
            visitor.visit_expr_mut(&mut field.base);
        }
        HirExpr::Index(idx) => {
            visitor.visit_expr_mut(&mut idx.base);
            visitor.visit_expr_mut(&mut idx.index);
        }
        HirExpr::Block(block) => {
            for stmt in &mut block.stmts {
                visitor.visit_stmt_mut(stmt);
            }
            if let Some(expr) = &mut block.expr {
                visitor.visit_expr_mut(expr);
            }
        }
        HirExpr::If(if_expr) => {
            visitor.visit_expr_mut(&mut if_expr.cond);
            visitor.visit_expr_mut(&mut if_expr.then_branch);
            if let Some(else_branch) = &mut if_expr.else_branch {
                visitor.visit_expr_mut(else_branch);
            }
        }
        HirExpr::Match(match_expr) => {
            visitor.visit_expr_mut(&mut match_expr.scrutinee);
            for arm in &mut match_expr.arms {
                visitor.visit_pat_mut(&mut arm.pat);
                if let Some(guard) = &mut arm.guard {
                    visitor.visit_expr_mut(guard);
                }
                visitor.visit_expr_mut(&mut arm.body);
            }
        }
        HirExpr::Lambda(lambda) => {
            for param in &mut lambda.params {
                visitor.visit_pat_mut(&mut param.pat);
                visitor.visit_type_mut(&mut param.ty);
            }
            if let Some(ret) = &mut lambda.return_type {
                visitor.visit_type_mut(ret);
            }
            visitor.visit_expr_mut(&mut lambda.body);
        }
        HirExpr::Loop(loop_expr) => {
            visitor.visit_expr_mut(&mut loop_expr.body);
        }
        HirExpr::While(while_expr) => {
            visitor.visit_expr_mut(&mut while_expr.cond);
            visitor.visit_expr_mut(&mut while_expr.body);
        }
        HirExpr::For(for_expr) => {
            visitor.visit_pat_mut(&mut for_expr.pat);
            visitor.visit_expr_mut(&mut for_expr.iter);
            visitor.visit_expr_mut(&mut for_expr.body);
        }
        HirExpr::Tuple(items) | HirExpr::List(items) => {
            for item in items {
                visitor.visit_expr_mut(item);
            }
        }
    }
}

/// Walk a statement mutably.
pub fn walk_stmt_mut<V: HirMutVisitor>(visitor: &mut V, stmt: &mut HirStmt) {
    match stmt {
        HirStmt::Val(val) => {
            visitor.visit_pat_mut(&mut val.pat);
            if let Some(ty) = &mut val.ty {
                visitor.visit_type_mut(ty);
            }
            visitor.visit_expr_mut(&mut val.init);
        }
        HirStmt::Var(var) => {
            visitor.visit_pat_mut(&mut var.pat);
            if let Some(ty) = &mut var.ty {
                visitor.visit_type_mut(ty);
            }
            visitor.visit_expr_mut(&mut var.init);
        }
        HirStmt::Assign(assign) => {
            visitor.visit_expr_mut(&mut assign.lhs);
            visitor.visit_expr_mut(&mut assign.rhs);
        }
        HirStmt::Expr(expr) => {
            visitor.visit_expr_mut(expr);
        }
        HirStmt::Return(ret) => {
            if let Some(expr) = ret {
                visitor.visit_expr_mut(expr);
            }
        }
        HirStmt::Break(brk) => {
            if let Some(expr) = brk {
                visitor.visit_expr_mut(expr);
            }
        }
        HirStmt::Continue => {}
    }
}

/// Walk a type mutably.
pub fn walk_type_mut<V: HirMutVisitor>(visitor: &mut V, ty: &mut HirType) {
    match ty {
        HirType::Named(named) => {
            for arg in &mut named.args {
                visitor.visit_type_mut(arg);
            }
        }
        HirType::Tuple(types) => {
            for t in types {
                visitor.visit_type_mut(t);
            }
        }
        HirType::Array(arr) => {
            visitor.visit_type_mut(&mut arr.elem);
        }
        HirType::Function(func) => {
            for p in &mut func.params {
                visitor.visit_type_mut(p);
            }
            visitor.visit_type_mut(&mut func.ret);
        }
        HirType::Ref(r) => {
            visitor.visit_type_mut(&mut r.ty);
        }
        HirType::Optional(inner) => {
            visitor.visit_type_mut(inner);
        }
        HirType::Var(_) | HirType::Error => {}
    }
}

/// Walk a pattern mutably.
pub fn walk_pat_mut<V: HirMutVisitor>(visitor: &mut V, pat: &mut HirPat) {
    match pat {
        HirPat::Wildcard | HirPat::Var(_) | HirPat::Literal(_) => {}
        HirPat::Constructor(ctor) => {
            for field in &mut ctor.fields {
                visitor.visit_pat_mut(field);
            }
        }
        HirPat::Tuple(pats) => {
            for p in pats {
                visitor.visit_pat_mut(p);
            }
        }
        HirPat::Or(pats) => {
            for p in pats {
                visitor.visit_pat_mut(p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counter visitor for testing.
    struct CountingVisitor {
        decl_count: usize,
    }

    impl HirVisitor for CountingVisitor {
        fn visit_decl(&mut self, decl: &HirDecl) {
            self.decl_count += 1;
            walk_decl(self, decl);
        }
    }

    #[test]
    fn test_visitor_basic() {
        use super::super::symbol::SymbolTable;

        let mut symbols = SymbolTable::new();
        let name = symbols.intern("test");
        let module = HirModule::new(name);

        let mut visitor = CountingVisitor { decl_count: 0 };
        visitor.visit_module(&module);

        // Empty module should have 0 declarations
        assert_eq!(visitor.decl_count, 0);
    }
}
