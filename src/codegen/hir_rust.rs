//! HIR -> Rust Code Generator
//!
//! Much simpler than AST-based codegen because HIR is canonical.
//! All desugaring has already happened, so we just need to translate
//! the canonical HIR forms to Rust.

use crate::hir::*;

/// HIR-based Rust code generator
pub struct HirRustCodegen {
    output: String,
    indent: usize,
    symbols: SymbolTable,
}

impl HirRustCodegen {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            symbols: SymbolTable::new(),
        }
    }

    /// Create a code generator with an existing symbol table
    pub fn with_symbols(symbols: SymbolTable) -> Self {
        Self {
            output: String::new(),
            indent: 0,
            symbols,
        }
    }

    /// Generate Rust code from a HIR module
    pub fn generate(&mut self, module: &HirModule) -> String {
        self.output.clear();

        // Module header
        self.emit_line("// Generated from DOL HIR");
        self.emit_line("");

        // Generate declarations
        for decl in &module.decls {
            self.gen_decl(decl);
            self.emit_line("");
        }

        std::mem::take(&mut self.output)
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        self.emit_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn sym(&self, s: Symbol) -> &str {
        self.symbols.resolve(s).unwrap_or("_unknown")
    }

    /// Intern a symbol (for use in tests and construction)
    pub fn intern(&mut self, s: &str) -> Symbol {
        self.symbols.intern(s)
    }

    /// Escape Rust keywords by prepending r#
    fn escape_keyword(&self, name: &str) -> String {
        match name {
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern"
            | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod"
            | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct"
            | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while"
            | "async" | "await" | "dyn" | "abstract" | "become" | "box" | "do" | "final"
            | "macro" | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield"
            | "try" => {
                format!("r#{}", name)
            }
            _ => name.to_string(),
        }
    }

    /// Infer Rust type from property name (basic heuristics)
    fn infer_type_from_name(&self, name: &str) -> String {
        let lower = name.to_lowercase();
        if lower.contains("count") || lower.contains("size") || lower.contains("length") {
            "usize".to_string()
        } else if lower == "x"
            || lower == "y"
            || lower == "z"
            || lower == "width"
            || lower == "height"
        {
            "i32".to_string()
        } else if lower == "alive" || lower == "active" || lower == "enabled" {
            "bool".to_string()
        } else if lower == "neighbors" {
            "u8".to_string()
        } else if lower == "name" || lower == "label" || lower == "description" {
            "String".to_string()
        } else if lower.ends_with("state") {
            // If property name ends with "state", try to infer the type
            name.replace("state", "State")
        } else if lower == "pos" || lower == "position" {
            "Position".to_string()
        } else {
            // Default to String for unknown types
            "String".to_string()
        }
    }

    /// Generate expression as a return value (no trailing semicolon)
    fn gen_expr_as_return(&self, expr: &HirExpr) -> String {
        match expr {
            HirExpr::Block(block) => {
                let mut parts = Vec::new();
                let stmt_count = block.stmts.len();

                for (i, stmt) in block.stmts.iter().enumerate() {
                    let is_last = i == stmt_count - 1;
                    // If this is the last statement and there's no final expr,
                    // and it's an Expr statement, generate it without semicolon
                    if is_last && block.expr.is_none() {
                        if let HirStmt::Expr(e) = stmt {
                            parts.push(self.gen_expr(e)); // No semicolon for return value
                            continue;
                        }
                    }
                    parts.push(self.gen_stmt(stmt));
                }

                if let Some(expr) = &block.expr {
                    parts.push(self.gen_expr(expr)); // No semicolon for final expr
                }
                format!("{{ {} }}", parts.join(" "))
            }
            _ => self.gen_expr(expr),
        }
    }

    /// Generate a declaration
    fn gen_decl(&mut self, decl: &HirDecl) {
        match decl {
            HirDecl::Type(type_decl) => self.gen_type_decl(type_decl),
            HirDecl::Trait(trait_decl) => self.gen_trait_decl(trait_decl),
            HirDecl::Function(func_decl) => self.gen_function_decl(func_decl),
            HirDecl::Module(module_decl) => self.gen_module_decl(module_decl),
        }
    }

    /// Generate a type declaration (gene, struct, enum)
    fn gen_type_decl(&mut self, decl: &HirTypeDecl) {
        let name = self.sym(decl.name).to_string();

        match &decl.body {
            HirTypeDef::Struct(fields) => {
                self.emit_line(&format!("pub struct {} {{", name));
                self.indent();
                for field in fields {
                    let ty = self.gen_type(&field.ty);
                    let field_name = self.escape_keyword(self.sym(field.name));
                    self.emit_line(&format!("pub {}: {},", field_name, ty));
                }
                self.dedent();
                self.emit_line("}");
            }
            HirTypeDef::Enum(variants) => {
                self.emit_line(&format!("pub enum {} {{", name));
                self.indent();
                for variant in variants {
                    if let Some(payload) = &variant.payload {
                        let ty = self.gen_type(payload);
                        self.emit_line(&format!("{}({}),", self.sym(variant.name), ty));
                    } else {
                        self.emit_line(&format!("{},", self.sym(variant.name)));
                    }
                }
                self.dedent();
                self.emit_line("}");
            }
            HirTypeDef::Alias(target) => {
                let ty = self.gen_type(target);
                self.emit_line(&format!("pub type {} = {};", name, ty));
            }
            HirTypeDef::Gene(statements) => {
                // Gene declarations become structs with properties as fields
                self.emit_line(&format!("/// Gene: {}", name));
                self.emit_line(&format!("pub struct {} {{", name));
                self.indent();
                for stmt in statements {
                    match &stmt.kind {
                        HirStatementKind::Has { subject, property } => {
                            // subject has property -> field: property
                            let prop_name = self.sym(*property).to_string();
                            let subj_name = self.sym(*subject).to_string();
                            self.emit_line(&format!("/// {} has {}", subj_name, prop_name));
                            // Escape keywords and infer basic types from property names
                            let field_name = self.escape_keyword(&prop_name);
                            let field_type = self.infer_type_from_name(&prop_name);
                            self.emit_line(&format!("pub {}: {},", field_name, field_type));
                        }
                        HirStatementKind::Is { subject, type_name } => {
                            self.emit_line(&format!(
                                "// {} is {}",
                                self.sym(*subject),
                                self.sym(*type_name)
                            ));
                        }
                        HirStatementKind::DerivesFrom { subject, parent } => {
                            self.emit_line(&format!(
                                "// {} derives from {}",
                                self.sym(*subject),
                                self.sym(*parent)
                            ));
                        }
                        HirStatementKind::Requires {
                            subject,
                            dependency,
                        } => {
                            self.emit_line(&format!(
                                "// {} requires {}",
                                self.sym(*subject),
                                self.sym(*dependency)
                            ));
                        }
                        HirStatementKind::Uses { subject, resource } => {
                            self.emit_line(&format!(
                                "// {} uses {}",
                                self.sym(*subject),
                                self.sym(*resource)
                            ));
                        }
                    }
                }
                self.dedent();
                self.emit_line("}");
            }
        }
    }

    /// Generate a trait declaration
    fn gen_trait_decl(&mut self, decl: &HirTraitDecl) {
        let name = self.sym(decl.name).to_string();

        self.emit_line(&format!("pub trait {} {{", name));
        self.indent();

        for item in &decl.items {
            match item {
                HirTraitItem::Method(method) => {
                    self.emit_indent();
                    self.emit(&format!("fn {}(", self.sym(method.name)));
                    for (i, param) in method.params.iter().enumerate() {
                        if i > 0 {
                            self.emit(", ");
                        }
                        if let HirPat::Var(sym) = &param.pat {
                            self.emit(&format!("{}: {}", self.sym(*sym), self.gen_type(&param.ty)));
                        } else {
                            self.emit(&format!("_: {}", self.gen_type(&param.ty)));
                        }
                    }
                    self.emit(&format!(") -> {};\n", self.gen_type(&method.return_type)));
                }
                HirTraitItem::AssocType(assoc) => {
                    self.emit_line(&format!("type {};", self.sym(assoc.name)));
                }
            }
        }

        self.dedent();
        self.emit_line("}");
    }

    /// Generate a function declaration
    fn gen_function_decl(&mut self, decl: &HirFunctionDecl) {
        let name = self.sym(decl.name).to_string();

        // Function signature
        self.emit_indent();
        self.emit(&format!("pub fn {}(", name));

        for (i, param) in decl.params.iter().enumerate() {
            if i > 0 {
                self.emit(", ");
            }
            if let HirPat::Var(sym) = &param.pat {
                self.emit(&format!(
                    "{}: {}",
                    self.escape_keyword(self.sym(*sym)),
                    self.gen_type(&param.ty)
                ));
            } else {
                self.emit(&format!("_: {}", self.gen_type(&param.ty)));
            }
        }

        self.emit(&format!(") -> {}", self.gen_type(&decl.return_type)));

        if let Some(body_expr) = &decl.body {
            self.emit(" {\n");
            self.indent();
            let body_code = self.gen_expr_as_return(body_expr);
            self.emit_line(&body_code);
            self.dedent();
            self.emit_line("}");
        } else {
            self.emit(";\n");
        }
    }

    /// Generate a module declaration
    fn gen_module_decl(&mut self, decl: &HirModuleDecl) {
        let name = self.sym(decl.name).to_string();

        self.emit_line(&format!("pub mod {} {{", name));
        self.indent();

        for item in &decl.decls {
            self.gen_decl(item);
        }

        self.dedent();
        self.emit_line("}");
    }

    /// Generate an expression
    fn gen_expr(&self, expr: &HirExpr) -> String {
        match expr {
            HirExpr::Literal(lit) => self.gen_lit(lit),
            HirExpr::Var(sym) => self.sym(*sym).to_string(),
            HirExpr::Binary(bin) => {
                let l = self.gen_expr(&bin.left);
                let r = self.gen_expr(&bin.right);
                let op_str = self.gen_binop(bin.op);
                format!("({} {} {})", l, op_str, r)
            }
            HirExpr::Unary(un) => {
                let o = self.gen_expr(&un.operand);
                let op_str = self.gen_unop(un.op);
                format!("({}{})", op_str, o)
            }
            HirExpr::Call(call) => {
                let f = self.gen_expr(&call.func);
                let args_str: Vec<_> = call.args.iter().map(|a| self.gen_expr(a)).collect();
                format!("{}({})", f, args_str.join(", "))
            }
            HirExpr::MethodCall(mc) => {
                let recv = self.gen_expr(&mc.receiver);
                let args_str: Vec<_> = mc.args.iter().map(|a| self.gen_expr(a)).collect();
                format!("{}.{}({})", recv, self.sym(mc.method), args_str.join(", "))
            }
            HirExpr::Field(field) => {
                // Check if the base is a simple variable (type name) or a complex expression
                let is_type_access = matches!(&field.base, HirExpr::Var(_));
                let base = self.gen_expr(&field.base);
                let field_name = self.escape_keyword(self.sym(field.field));

                // If accessing from a type name (e.g., CellState.Alive), use ::
                // Heuristic: base is capitalized (type name) or is a simple identifier
                if is_type_access && base.chars().next().is_some_and(|c| c.is_uppercase()) {
                    format!("{}::{}", base, field_name)
                } else {
                    format!("{}.{}", base, field_name)
                }
            }
            HirExpr::Index(idx) => {
                format!(
                    "{}[{}]",
                    self.gen_expr(&idx.base),
                    self.gen_expr(&idx.index)
                )
            }
            HirExpr::Block(block) => {
                let mut parts = Vec::new();
                for stmt in &block.stmts {
                    parts.push(self.gen_stmt(stmt));
                }
                if let Some(expr) = &block.expr {
                    parts.push(self.gen_expr(expr));
                }
                format!("{{ {} }}", parts.join(" "))
            }
            HirExpr::If(if_expr) => {
                let c = self.gen_expr(&if_expr.cond);
                let t = self.gen_expr(&if_expr.then_branch);
                if let Some(e) = &if_expr.else_branch {
                    format!("if {} {{ {} }} else {{ {} }}", c, t, self.gen_expr(e))
                } else {
                    format!("if {} {{ {} }}", c, t)
                }
            }
            HirExpr::Match(match_expr) => {
                let scrutinee = self.gen_expr(&match_expr.scrutinee);
                let mut arms = Vec::new();
                for arm in &match_expr.arms {
                    let pat = self.gen_pat(&arm.pat);
                    let body = self.gen_expr(&arm.body);
                    if let Some(guard) = &arm.guard {
                        arms.push(format!("{} if {} => {}", pat, self.gen_expr(guard), body));
                    } else {
                        arms.push(format!("{} => {}", pat, body));
                    }
                }
                format!("match {} {{ {} }}", scrutinee, arms.join(", "))
            }
            HirExpr::Lambda(lambda) => {
                let params: Vec<_> = lambda
                    .params
                    .iter()
                    .map(|p| {
                        if let HirPat::Var(sym) = &p.pat {
                            format!("{}: {}", self.sym(*sym), self.gen_type(&p.ty))
                        } else {
                            format!("_: {}", self.gen_type(&p.ty))
                        }
                    })
                    .collect();
                let body = self.gen_expr(&lambda.body);
                format!("|{}| {}", params.join(", "), body)
            }
        }
    }

    /// Generate a statement (for blocks)
    fn gen_stmt(&self, stmt: &HirStmt) -> String {
        match stmt {
            HirStmt::Val(val) => {
                // val x = e -> let x = e;
                if let HirPat::Var(sym) = &val.pat {
                    format!("let {} = {};", self.sym(*sym), self.gen_expr(&val.init))
                } else {
                    format!("let _ = {};", self.gen_expr(&val.init))
                }
            }
            HirStmt::Var(var) => {
                // var x = e -> let mut x = e;
                if let HirPat::Var(sym) = &var.pat {
                    format!("let mut {} = {};", self.sym(*sym), self.gen_expr(&var.init))
                } else {
                    format!("let mut _ = {};", self.gen_expr(&var.init))
                }
            }
            HirStmt::Assign(assign) => {
                format!(
                    "{} = {};",
                    self.gen_expr(&assign.lhs),
                    self.gen_expr(&assign.rhs)
                )
            }
            HirStmt::Expr(expr) => {
                format!("{};", self.gen_expr(expr))
            }
            HirStmt::Return(value) => {
                if let Some(v) = value {
                    format!("return {};", self.gen_expr(v))
                } else {
                    "return;".to_string()
                }
            }
            HirStmt::Break(value) => {
                if let Some(v) = value {
                    format!("break {};", self.gen_expr(v))
                } else {
                    "break;".to_string()
                }
            }
        }
    }

    /// Generate a pattern
    fn gen_pat(&self, pat: &HirPat) -> String {
        match pat {
            HirPat::Wildcard => "_".to_string(),
            HirPat::Var(sym) => self.escape_keyword(self.sym(*sym)),
            HirPat::Literal(lit) => self.gen_lit(lit),
            HirPat::Constructor(ctor) => {
                let name = self.sym(ctor.name);
                // Convert dot notation to :: for enum variants
                let qualified_name = if name.contains('.') {
                    name.replace('.', "::")
                } else {
                    // Check if this looks like an enum variant (capitalized)
                    // If so, assume it needs qualification
                    name.to_string()
                };

                if ctor.fields.is_empty() {
                    qualified_name
                } else {
                    let fields: Vec<_> = ctor.fields.iter().map(|f| self.gen_pat(f)).collect();
                    format!("{}({})", qualified_name, fields.join(", "))
                }
            }
            HirPat::Tuple(pats) => {
                let parts: Vec<_> = pats.iter().map(|p| self.gen_pat(p)).collect();
                format!("({})", parts.join(", "))
            }
            HirPat::Or(pats) => {
                let parts: Vec<_> = pats.iter().map(|p| self.gen_pat(p)).collect();
                parts.join(" | ")
            }
        }
    }

    /// Generate a type
    pub fn gen_type(&self, ty: &HirType) -> String {
        match ty {
            HirType::Named(named) => {
                let base = self.sym(named.name);
                if named.args.is_empty() {
                    // Map common DOL types to Rust types
                    match base {
                        "Int8" => "i8".to_string(),
                        "Int16" => "i16".to_string(),
                        "Int32" => "i32".to_string(),
                        "Int64" => "i64".to_string(),
                        "UInt8" => "u8".to_string(),
                        "UInt16" => "u16".to_string(),
                        "UInt32" => "u32".to_string(),
                        "UInt64" => "u64".to_string(),
                        "Float32" => "f32".to_string(),
                        "Float64" => "f64".to_string(),
                        "String" => "String".to_string(),
                        "Bool" => "bool".to_string(),
                        "Unit" | "()" => "()".to_string(),
                        _ => base.to_string(),
                    }
                } else {
                    let args_str: Vec<_> = named.args.iter().map(|a| self.gen_type(a)).collect();
                    // Map common generic types
                    match base {
                        "List" => format!("Vec<{}>", args_str.join(", ")),
                        "Map" => format!("std::collections::HashMap<{}>", args_str.join(", ")),
                        "Option" => format!("Option<{}>", args_str.join(", ")),
                        "Result" => format!("Result<{}>", args_str.join(", ")),
                        _ => format!("{}<{}>", base, args_str.join(", ")),
                    }
                }
            }
            HirType::Tuple(elems) => {
                let inner: Vec<_> = elems.iter().map(|e| self.gen_type(e)).collect();
                format!("({})", inner.join(", "))
            }
            HirType::Array(arr) => {
                let elem = self.gen_type(&arr.elem);
                if let Some(size) = arr.size {
                    format!("[{}; {}]", elem, size)
                } else {
                    format!("Vec<{}>", elem)
                }
            }
            HirType::Function(func) => {
                let params: Vec<_> = func.params.iter().map(|p| self.gen_type(p)).collect();
                let ret = self.gen_type(&func.ret);
                format!("fn({}) -> {}", params.join(", "), ret)
            }
            HirType::Ref(r) => {
                let inner = self.gen_type(&r.ty);
                if r.mutable {
                    format!("&mut {}", inner)
                } else {
                    format!("&{}", inner)
                }
            }
            HirType::Optional(inner) => {
                format!("Option<{}>", self.gen_type(inner))
            }
            HirType::Var(id) => {
                // Type variable - use a placeholder
                format!("T{}", id)
            }
            HirType::Error => "_".to_string(),
        }
    }

    fn gen_lit(&self, lit: &HirLiteral) -> String {
        match lit {
            HirLiteral::Bool(b) => b.to_string(),
            HirLiteral::Int(i) => i.to_string(),
            HirLiteral::Float(f) => format!("{:?}", f),
            HirLiteral::String(s) => format!("{:?}", s),
            HirLiteral::Unit => "()".to_string(),
        }
    }

    fn gen_binop(&self, op: HirBinaryOp) -> &'static str {
        match op {
            HirBinaryOp::Add => "+",
            HirBinaryOp::Sub => "-",
            HirBinaryOp::Mul => "*",
            HirBinaryOp::Div => "/",
            HirBinaryOp::Mod => "%",
            HirBinaryOp::Eq => "==",
            HirBinaryOp::Ne => "!=",
            HirBinaryOp::Lt => "<",
            HirBinaryOp::Le => "<=",
            HirBinaryOp::Gt => ">",
            HirBinaryOp::Ge => ">=",
            HirBinaryOp::And => "&&",
            HirBinaryOp::Or => "||",
        }
    }

    fn gen_unop(&self, op: HirUnaryOp) -> &'static str {
        match op {
            HirUnaryOp::Neg => "-",
            HirUnaryOp::Not => "!",
        }
    }
}

impl Default for HirRustCodegen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_type_primitives() {
        let mut gen = HirRustCodegen::new();

        // Test named types
        let int32 = gen.intern("Int32");
        let int32_type = HirType::Named(HirNamedType {
            name: int32,
            args: vec![],
        });
        assert_eq!(gen.gen_type(&int32_type), "i32");

        let uint64 = gen.intern("UInt64");
        let uint64_type = HirType::Named(HirNamedType {
            name: uint64,
            args: vec![],
        });
        assert_eq!(gen.gen_type(&uint64_type), "u64");

        let float64 = gen.intern("Float64");
        let float64_type = HirType::Named(HirNamedType {
            name: float64,
            args: vec![],
        });
        assert_eq!(gen.gen_type(&float64_type), "f64");

        let string = gen.intern("String");
        let string_type = HirType::Named(HirNamedType {
            name: string,
            args: vec![],
        });
        assert_eq!(gen.gen_type(&string_type), "String");

        let bool_sym = gen.intern("Bool");
        let bool_type = HirType::Named(HirNamedType {
            name: bool_sym,
            args: vec![],
        });
        assert_eq!(gen.gen_type(&bool_type), "bool");
    }

    #[test]
    fn test_gen_type_compound() {
        let mut gen = HirRustCodegen::new();

        // Test array type
        let int32 = gen.intern("Int32");
        let array_type = HirType::Array(Box::new(HirArrayType {
            elem: HirType::Named(HirNamedType {
                name: int32,
                args: vec![],
            }),
            size: None,
        }));
        assert_eq!(gen.gen_type(&array_type), "Vec<i32>");

        // Test fixed array
        let fixed_array = HirType::Array(Box::new(HirArrayType {
            elem: HirType::Named(HirNamedType {
                name: int32,
                args: vec![],
            }),
            size: Some(10),
        }));
        assert_eq!(gen.gen_type(&fixed_array), "[i32; 10]");

        // Test tuple type
        let bool_sym = gen.intern("Bool");
        let string = gen.intern("String");
        let tuple_type = HirType::Tuple(vec![
            HirType::Named(HirNamedType {
                name: bool_sym,
                args: vec![],
            }),
            HirType::Named(HirNamedType {
                name: string,
                args: vec![],
            }),
        ]);
        assert_eq!(gen.gen_type(&tuple_type), "(bool, String)");
    }

    #[test]
    fn test_gen_lit() {
        let gen = HirRustCodegen::new();

        assert_eq!(gen.gen_lit(&HirLiteral::Bool(true)), "true");
        assert_eq!(gen.gen_lit(&HirLiteral::Bool(false)), "false");
        assert_eq!(gen.gen_lit(&HirLiteral::Int(42)), "42");
        assert_eq!(gen.gen_lit(&HirLiteral::Unit), "()");
    }

    #[test]
    fn test_gen_stmt_val_var() {
        let mut gen = HirRustCodegen::new();
        let x = gen.intern("x");

        let val_stmt = HirStmt::Val(HirValStmt {
            pat: HirPat::Var(x),
            ty: None,
            init: HirExpr::Literal(HirLiteral::Int(42)),
        });

        let var_stmt = HirStmt::Var(HirVarStmt {
            pat: HirPat::Var(x),
            ty: None,
            init: HirExpr::Literal(HirLiteral::Int(0)),
        });

        assert_eq!(gen.gen_stmt(&val_stmt), "let x = 42;");
        assert_eq!(gen.gen_stmt(&var_stmt), "let mut x = 0;");
    }

    #[test]
    fn test_gen_binary_expr() {
        let mut gen = HirRustCodegen::new();
        let a = gen.intern("a");
        let b = gen.intern("b");

        let bin_expr = HirExpr::Binary(Box::new(HirBinaryExpr {
            left: HirExpr::Var(a),
            op: HirBinaryOp::Add,
            right: HirExpr::Var(b),
        }));

        assert_eq!(gen.gen_expr(&bin_expr), "(a + b)");
    }

    #[test]
    fn test_gen_call_expr() {
        let mut gen = HirRustCodegen::new();
        let foo = gen.intern("foo");
        let x = gen.intern("x");

        let call_expr = HirExpr::Call(Box::new(HirCallExpr {
            func: HirExpr::Var(foo),
            args: vec![HirExpr::Var(x), HirExpr::Literal(HirLiteral::Int(42))],
        }));

        assert_eq!(gen.gen_expr(&call_expr), "foo(x, 42)");
    }

    #[test]
    fn test_gen_if_expr() {
        let mut gen = HirRustCodegen::new();
        let cond = gen.intern("cond");

        let if_expr = HirExpr::If(Box::new(HirIfExpr {
            cond: HirExpr::Var(cond),
            then_branch: HirExpr::Literal(HirLiteral::Int(1)),
            else_branch: Some(HirExpr::Literal(HirLiteral::Int(0))),
        }));

        assert_eq!(gen.gen_expr(&if_expr), "if cond { 1 } else { 0 }");
    }

    #[test]
    fn test_gen_optional_type() {
        let mut gen = HirRustCodegen::new();
        let int32 = gen.intern("Int32");

        let opt_type = HirType::Optional(Box::new(HirType::Named(HirNamedType {
            name: int32,
            args: vec![],
        })));

        assert_eq!(gen.gen_type(&opt_type), "Option<i32>");
    }

    #[test]
    fn test_gen_ref_type() {
        let mut gen = HirRustCodegen::new();
        let string = gen.intern("String");

        let ref_type = HirType::Ref(Box::new(HirRefType {
            mutable: false,
            ty: HirType::Named(HirNamedType {
                name: string,
                args: vec![],
            }),
        }));
        assert_eq!(gen.gen_type(&ref_type), "&String");

        let mut_ref_type = HirType::Ref(Box::new(HirRefType {
            mutable: true,
            ty: HirType::Named(HirNamedType {
                name: string,
                args: vec![],
            }),
        }));
        assert_eq!(gen.gen_type(&mut_ref_type), "&mut String");
    }

    #[test]
    fn test_gen_function_type() {
        let mut gen = HirRustCodegen::new();
        let int32 = gen.intern("Int32");
        let bool_sym = gen.intern("Bool");

        let func_type = HirType::Function(Box::new(HirFunctionType {
            params: vec![
                HirType::Named(HirNamedType {
                    name: int32,
                    args: vec![],
                }),
                HirType::Named(HirNamedType {
                    name: int32,
                    args: vec![],
                }),
            ],
            ret: HirType::Named(HirNamedType {
                name: bool_sym,
                args: vec![],
            }),
        }));

        assert_eq!(gen.gen_type(&func_type), "fn(i32, i32) -> bool");
    }
}
