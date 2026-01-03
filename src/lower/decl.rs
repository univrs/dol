//! Declaration lowering
//!
//! Converts AST declarations to HIR declarations.
//! Both `gene` and `type` map to `HirDecl::Type`.
//!
//! # Lowering Rules
//!
//! ## Gene Declarations
//! - `gene Foo { ... }` -> `HirDecl::Type(HirTypeDecl { body: HirTypeDef::Gene(...) })`
//!
//! ## Trait Declarations
//! - `trait Foo { ... }` -> `HirDecl::Trait(HirTraitDecl { ... })`
//!
//! ## Function Declarations
//! - `fun foo(...) { ... }` -> `HirDecl::Function(HirFunctionDecl { ... })`

use super::LoweringContext;
use crate::ast;
use crate::hir::*;

impl LoweringContext {
    /// Lower a declaration (placeholder)
    ///
    /// This will eventually convert AST declarations to HIR declarations.
    pub fn lower_decl(&mut self) -> HirDecl {
        let name = self.intern("placeholder");
        HirDecl::Type(HirTypeDecl {
            id: self.fresh_id(),
            name,
            type_params: vec![],
            body: HirTypeDef::Struct(vec![]),
        })
    }

    /// Lower a gene declaration to HIR
    pub fn lower_gene(&mut self, gene: &ast::Gene) -> HirDecl {
        let name = self.intern(&gene.name);
        let statements: Vec<HirStatement> = gene
            .statements
            .iter()
            .map(|s| self.lower_dol_statement(s))
            .collect();

        HirDecl::Type(HirTypeDecl {
            id: self.fresh_id(),
            name,
            type_params: vec![],
            body: HirTypeDef::Gene(statements),
        })
    }

    /// Lower a trait declaration to HIR
    pub fn lower_trait(&mut self, trait_decl: &ast::Trait) -> HirDecl {
        let name = self.intern(&trait_decl.name);

        // Convert statements to trait items
        let items: Vec<HirTraitItem> = trait_decl
            .statements
            .iter()
            .filter_map(|s| {
                if let ast::Statement::Function(func) = s {
                    Some(HirTraitItem::Method(self.lower_function_decl(func)))
                } else {
                    None
                }
            })
            .collect();

        HirDecl::Trait(HirTraitDecl {
            id: self.fresh_id(),
            name,
            type_params: vec![],
            bounds: vec![],
            items,
        })
    }

    /// Lower a function declaration to HIR
    pub fn lower_function_decl(&mut self, func: &ast::FunctionDecl) -> HirFunctionDecl {
        let name = self.intern(&func.name);

        // Lower type parameters
        let type_params: Vec<HirTypeParam> = func
            .type_params
            .as_ref()
            .map(|tp| {
                tp.params
                    .iter()
                    .map(|p| HirTypeParam {
                        name: self.intern(&p.name),
                        bounds: p.bounds.iter().map(|b| self.lower_type_expr(b)).collect(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Lower parameters
        let params: Vec<HirParam> = func
            .params
            .iter()
            .map(|p| HirParam {
                pat: HirPat::Var(self.intern(&p.name)),
                ty: self.lower_type_expr(&p.type_ann),
            })
            .collect();

        // Lower return type
        let return_type = func
            .return_type
            .as_ref()
            .map(|t| self.lower_type_expr(t))
            .unwrap_or(HirType::Tuple(vec![])); // Unit type

        // Lower function body
        let body = if func.body.is_empty() {
            None
        } else {
            // Convert statements to HIR statements
            let stmts: Vec<HirStmt> = func.body.iter().map(|s| self.lower_block_stmt(s)).collect();

            // The last statement might be the return value if it's an expression
            // For now, wrap all statements in a block
            Some(HirExpr::Block(Box::new(HirBlockExpr { stmts, expr: None })))
        };

        HirFunctionDecl {
            id: self.fresh_id(),
            name,
            type_params,
            params,
            return_type,
            body,
        }
    }

    /// Lower a type expression to HIR
    pub fn lower_type_expr(&mut self, type_expr: &ast::TypeExpr) -> HirType {
        match type_expr {
            ast::TypeExpr::Named(name) => HirType::Named(HirNamedType {
                name: self.intern(name),
                args: vec![],
            }),
            ast::TypeExpr::Generic { name, args } => HirType::Named(HirNamedType {
                name: self.intern(name),
                args: args.iter().map(|a| self.lower_type_expr(a)).collect(),
            }),
            ast::TypeExpr::Function {
                params,
                return_type,
            } => HirType::Function(Box::new(HirFunctionType {
                params: params.iter().map(|p| self.lower_type_expr(p)).collect(),
                ret: self.lower_type_expr(return_type),
            })),
            ast::TypeExpr::Tuple(types) => {
                HirType::Tuple(types.iter().map(|t| self.lower_type_expr(t)).collect())
            }
            ast::TypeExpr::Never => HirType::Error, // Map Never to Error for now
            ast::TypeExpr::Enum { .. } => HirType::Error, // Inline enums not fully supported yet
        }
    }

    /// Lower a full AST declaration
    pub fn lower_declaration(&mut self, decl: &ast::Declaration) -> HirDecl {
        match decl {
            ast::Declaration::Gene(gene) => self.lower_gene(gene),
            ast::Declaration::Trait(trait_decl) => self.lower_trait(trait_decl),
            ast::Declaration::Function(func) => HirDecl::Function(self.lower_function_decl(func)),
            ast::Declaration::Constraint(constraint) => {
                // Lower constraint as a trait with constraint semantics
                let name = self.intern(&constraint.name);
                HirDecl::Trait(HirTraitDecl {
                    id: self.fresh_id(),
                    name,
                    type_params: vec![],
                    bounds: vec![],
                    items: vec![],
                })
            }
            ast::Declaration::System(system) => {
                // Lower system as a module
                let name = self.intern(&system.name);
                HirDecl::Module(HirModuleDecl {
                    id: self.fresh_id(),
                    name,
                    decls: vec![],
                })
            }
            ast::Declaration::Evolution(evolution) => {
                // Evolution is metadata, lower as empty module
                let name = self.intern(&evolution.name);
                HirDecl::Module(HirModuleDecl {
                    id: self.fresh_id(),
                    name,
                    decls: vec![],
                })
            }
            ast::Declaration::Const(var) => {
                // Const declarations are lowered as module-level constants
                let name = self.intern(&var.name);
                HirDecl::Module(HirModuleDecl {
                    id: self.fresh_id(),
                    name,
                    decls: vec![],
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_gene() {
        let mut ctx = LoweringContext::new();
        let gene = ast::Gene {
            name: "container.exists".to_string(),
            extends: None,
            statements: vec![ast::Statement::Has {
                subject: "container".to_string(),
                property: "identity".to_string(),
                span: ast::Span::default(),
            }],
            exegesis: "Test gene".to_string(),
            span: ast::Span::default(),
        };

        let hir_decl = ctx.lower_gene(&gene);
        match hir_decl {
            HirDecl::Type(type_decl) => {
                assert_eq!(ctx.resolve(type_decl.name), Some("container.exists"));
                match type_decl.body {
                    HirTypeDef::Gene(stmts) => {
                        assert_eq!(stmts.len(), 1);
                    }
                    _ => panic!("Expected Gene body"),
                }
            }
            _ => panic!("Expected Type declaration"),
        }
    }

    #[test]
    fn test_lower_type_expr_named() {
        let mut ctx = LoweringContext::new();
        let type_expr = ast::TypeExpr::Named("Int32".to_string());
        let hir_type = ctx.lower_type_expr(&type_expr);

        match hir_type {
            HirType::Named(named) => {
                assert_eq!(ctx.resolve(named.name), Some("Int32"));
            }
            _ => panic!("Expected Named type"),
        }
    }

    #[test]
    fn test_lower_type_expr_generic() {
        let mut ctx = LoweringContext::new();
        let type_expr = ast::TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![ast::TypeExpr::Named("Int32".to_string())],
        };
        let hir_type = ctx.lower_type_expr(&type_expr);

        match hir_type {
            HirType::Named(named) => {
                assert_eq!(ctx.resolve(named.name), Some("List"));
                assert_eq!(named.args.len(), 1);
            }
            _ => panic!("Expected Named type with args"),
        }
    }

    #[test]
    fn test_lower_type_expr_function() {
        let mut ctx = LoweringContext::new();
        let type_expr = ast::TypeExpr::Function {
            params: vec![ast::TypeExpr::Named("Int32".to_string())],
            return_type: Box::new(ast::TypeExpr::Named("Bool".to_string())),
        };
        let hir_type = ctx.lower_type_expr(&type_expr);

        match hir_type {
            HirType::Function(func_type) => {
                assert_eq!(func_type.params.len(), 1);
            }
            _ => panic!("Expected Function type"),
        }
    }

    #[test]
    fn test_lower_type_expr_tuple() {
        let mut ctx = LoweringContext::new();
        let type_expr = ast::TypeExpr::Tuple(vec![
            ast::TypeExpr::Named("Int32".to_string()),
            ast::TypeExpr::Named("String".to_string()),
        ]);
        let hir_type = ctx.lower_type_expr(&type_expr);

        match hir_type {
            HirType::Tuple(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected Tuple type"),
        }
    }
}
