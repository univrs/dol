//! Code generation for procedural macros.
//!
//! This module provides code generation utilities for transforming
//! DOL AST into Rust code, WIT interfaces, and other target formats.

use crate::error::{ProcMacroError, ProcMacroResult};
use metadol::ast::{Declaration, Expr, Gen, Literal, Statement, TypeExpr};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

/// Code generator for procedural macros.
pub struct CodeGenerator {
    /// Target language/format
    target: CodegenTarget,
}

/// Target format for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    /// Rust code
    Rust,
    /// WebAssembly Interface Types (WIT)
    Wit,
    /// TypeScript definitions
    TypeScript,
    /// Documentation
    Documentation,
}

impl CodeGenerator {
    /// Creates a new code generator for the specified target.
    pub fn new(target: CodegenTarget) -> Self {
        Self { target }
    }

    /// Generates code for a declaration.
    pub fn generate_decl(&self, decl: &Declaration) -> ProcMacroResult<String> {
        match self.target {
            CodegenTarget::Rust => self.generate_rust_decl(decl),
            CodegenTarget::Wit => self.generate_wit_decl(decl),
            CodegenTarget::TypeScript => self.generate_ts_decl(decl),
            CodegenTarget::Documentation => self.generate_doc_decl(decl),
        }
    }

    /// Generates Rust code for a declaration.
    fn generate_rust_decl(&self, decl: &Declaration) -> ProcMacroResult<String> {
        match decl {
            Declaration::Gene(gen) => self.generate_rust_gen(gen),
            _ => Err(ProcMacroError::unsupported("declaration type")),
        }
    }

    /// Generates Rust code for a gene declaration.
    fn generate_rust_gen(&self, gen: &Gen) -> ProcMacroResult<String> {
        let name = gen.name.replace('.', "_");
        let name_ident = format_ident!("{}", name);
        let exegesis = &gen.exegesis;

        let mut fields = Vec::new();
        for stmt in &gen.statements {
            if let Statement::Has { property, .. } = stmt {
                fields.push(format_ident!("{}", property));
            }
        }

        let code = if fields.is_empty() {
            quote! {
                #[doc = #exegesis]
                pub struct #name_ident;
            }
        } else {
            quote! {
                #[doc = #exegesis]
                pub struct #name_ident {
                    #(pub #fields: (),)*
                }
            }
        };

        Ok(code.to_string())
    }

    /// Generates WIT code for a declaration.
    fn generate_wit_decl(&self, decl: &Declaration) -> ProcMacroResult<String> {
        match decl {
            Declaration::Gene(gen) => self.generate_wit_gen(gen),
            _ => Err(ProcMacroError::unsupported("declaration type")),
        }
    }

    /// Generates WIT code for a gene declaration.
    fn generate_wit_gen(&self, gen: &Gen) -> ProcMacroResult<String> {
        let name = gen.name.replace('.', "-");
        let mut wit = format!("/// {}\nrecord {} {{\n", gen.exegesis, name);

        for stmt in &gen.statements {
            if let Statement::Has { property, .. } = stmt {
                wit.push_str(&format!("  {}: unit,\n", property));
            }
        }

        wit.push_str("}\n");
        Ok(wit)
    }

    /// Generates TypeScript code for a declaration.
    fn generate_ts_decl(&self, decl: &Declaration) -> ProcMacroResult<String> {
        match decl {
            Declaration::Gene(gen) => self.generate_ts_gen(gen),
            _ => Err(ProcMacroError::unsupported("declaration type")),
        }
    }

    /// Generates TypeScript code for a gene declaration.
    fn generate_ts_gen(&self, gen: &Gen) -> ProcMacroResult<String> {
        let name = gen.name.replace('.', "_");
        let mut ts = format!("/**\n * {}\n */\n", gen.exegesis);
        ts.push_str(&format!("export interface {} {{\n", name));

        for stmt in &gen.statements {
            if let Statement::Has { property, .. } = stmt {
                ts.push_str(&format!("  {}: any;\n", property));
            }
        }

        ts.push_str("}\n");
        Ok(ts)
    }

    /// Generates documentation for a declaration.
    fn generate_doc_decl(&self, decl: &Declaration) -> ProcMacroResult<String> {
        match decl {
            Declaration::Gene(gen) => {
                let mut doc = format!("# {}\n\n", gen.name);
                doc.push_str(&format!("{}\n\n", gen.exegesis));
                doc.push_str("## Properties\n\n");

                for stmt in &gen.statements {
                    if let Statement::Has { subject, property, .. } = stmt {
                        doc.push_str(&format!("- **{}**: {}\n", property, subject));
                    }
                }

                Ok(doc)
            }
            _ => Err(ProcMacroError::unsupported("declaration type")),
        }
    }

    /// Generates code for an expression.
    pub fn generate_expr(&self, expr: &Expr) -> ProcMacroResult<String> {
        match self.target {
            CodegenTarget::Rust => self.generate_rust_expr(expr),
            _ => Err(ProcMacroError::unsupported("expression codegen for target")),
        }
    }

    /// Generates Rust code for an expression.
    fn generate_rust_expr(&self, expr: &Expr) -> ProcMacroResult<String> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(n) => Ok(n.to_string()),
                Literal::Float(f) => Ok(f.to_string()),
                Literal::String(s) => Ok(format!("\"{}\"", s)),
                Literal::Bool(b) => Ok(b.to_string()),
                _ => Ok("()".to_string()),
            },
            Expr::Identifier(name) => Ok(name.clone()),
            Expr::Binary { op, left, right } => {
                let left_code = self.generate_rust_expr(left)?;
                let right_code = self.generate_rust_expr(right)?;
                let op_str = format!("{:?}", op).to_lowercase();
                Ok(format!("({} {} {})", left_code, op_str, right_code))
            }
            _ => Err(ProcMacroError::unsupported("expression type")),
        }
    }
}

/// Generates Rust code from a DOL declaration.
///
/// # Example
///
/// ```rust,ignore
/// let code = generate_rust_code(&decl)?;
/// ```
pub fn generate_rust_code(decl: &Declaration) -> ProcMacroResult<String> {
    let generator = CodeGenerator::new(CodegenTarget::Rust);
    generator.generate_decl(decl)
}

/// Generates WIT interface from a DOL declaration.
///
/// # Example
///
/// ```rust,ignore
/// let wit = generate_wit_code(&decl)?;
/// ```
pub fn generate_wit_code(decl: &Declaration) -> ProcMacroResult<String> {
    let generator = CodeGenerator::new(CodegenTarget::Wit);
    generator.generate_decl(decl)
}

/// Generates TypeScript definitions from a DOL declaration.
pub fn generate_typescript_code(decl: &Declaration) -> ProcMacroResult<String> {
    let generator = CodeGenerator::new(CodegenTarget::TypeScript);
    generator.generate_decl(decl)
}

/// Generates documentation from a DOL declaration.
pub fn generate_documentation(decl: &Declaration) -> ProcMacroResult<String> {
    let generator = CodeGenerator::new(CodegenTarget::Documentation);
    generator.generate_decl(decl)
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadol::ast::{Span, Visibility};

    fn create_test_gen() -> Gen {
        Gen {
            visibility: Visibility::default(),
            name: "test.gene".to_string(),
            extends: None,
            statements: vec![Statement::Has {
                subject: "test".to_string(),
                property: "field".to_string(),
                span: Span::default(),
            }],
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        }
    }

    #[test]
    fn test_generate_rust_code() {
        let gen = create_test_gen();
        let decl = Declaration::Gene(gen);

        let result = generate_rust_code(&decl);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("struct"));
        assert!(code.contains("test_gene"));
    }

    #[test]
    fn test_generate_wit_code() {
        let gen = create_test_gen();
        let decl = Declaration::Gene(gen);

        let result = generate_wit_code(&decl);
        assert!(result.is_ok());

        let wit = result.unwrap();
        assert!(wit.contains("record"));
        assert!(wit.contains("test-gene"));
    }

    #[test]
    fn test_generate_typescript_code() {
        let gen = create_test_gen();
        let decl = Declaration::Gene(gen);

        let result = generate_typescript_code(&decl);
        assert!(result.is_ok());

        let ts = result.unwrap();
        assert!(ts.contains("interface"));
        assert!(ts.contains("test_gene"));
    }

    #[test]
    fn test_generate_documentation() {
        let gen = create_test_gen();
        let decl = Declaration::Gene(gen);

        let result = generate_documentation(&decl);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert!(doc.contains("# test.gene"));
        assert!(doc.contains("Test gene"));
    }

    #[test]
    fn test_generate_expr_literal() {
        let generator = CodeGenerator::new(CodegenTarget::Rust);
        let expr = Expr::Literal(Literal::Int(42));

        let result = generator.generate_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_generate_expr_ident() {
        let generator = CodeGenerator::new(CodegenTarget::Rust);
        let expr = Expr::Identifier("x".to_string());

        let result = generator.generate_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "x");
    }

    #[test]
    fn test_codegen_target() {
        assert_eq!(CodegenTarget::Rust, CodegenTarget::Rust);
        assert_ne!(CodegenTarget::Rust, CodegenTarget::Wit);
    }
}
