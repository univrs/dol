//! Rust code generation from Metal DOL declarations.
//!
//! Generates Rust structs, traits, and types from DOL declarations.
//!
//! # Type Mapping
//!
//! | DOL Type | Rust Type |
//! |----------|-----------|
//! | `Int8` | `i8` |
//! | `Int16` | `i16` |
//! | `Int32` | `i32` |
//! | `Int64` | `i64` |
//! | `UInt8` | `u8` |
//! | `UInt16` | `u16` |
//! | `UInt32` | `u32` |
//! | `UInt64` | `u64` |
//! | `Float32` | `f32` |
//! | `Float64` | `f64` |
//! | `String` | `String` |
//! | `Bool` | `bool` |
//! | `Option<T>` | `Option<T>` |
//! | `Result<T, E>` | `Result<T, E>` |
//! | `List<T>` | `Vec<T>` |
//! | `Map<K, V>` | `std::collections::HashMap<K, V>` |

use crate::ast::{
    Constraint, Declaration, Evolution, Expr, ExternDecl, FunctionDecl, FunctionParam, Gene,
    Literal, Mutability, Statement, Stmt, System, Trait, TypeExpr, VarDecl,
};
use crate::typechecker::Type;

use super::{to_pascal_case, to_snake_case, Codegen, CodegenOptions, TypeMapper, Visibility};

/// Rust code generator.
///
/// Transforms DOL declarations into Rust source code.
#[derive(Debug, Clone, Default)]
pub struct RustCodegen {
    options: CodegenOptions,
}

impl RustCodegen {
    /// Create a new Rust code generator with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Rust code generator with custom options.
    pub fn with_options(options: CodegenOptions) -> Self {
        Self { options }
    }

    /// Generate Rust code from a declaration.
    pub fn generate(decl: &Declaration) -> String {
        Self::new().generate_declaration(decl)
    }

    /// Generate Rust code from multiple declarations.
    pub fn generate_all(decls: &[Declaration]) -> String {
        let generator = Self::new();
        decls
            .iter()
            .map(|d| generator.generate_declaration(d))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Generate code for a single declaration.
    fn generate_declaration(&self, decl: &Declaration) -> String {
        match decl {
            Declaration::Gene(gene) => self.generate_gene(gene),
            Declaration::Trait(trait_decl) => self.generate_trait(trait_decl),
            Declaration::Constraint(constraint) => self.generate_constraint(constraint),
            Declaration::System(system) => self.generate_system(system),
            Declaration::Evolution(evolution) => self.generate_evolution(evolution),
        }
    }

    /// Generate a Rust struct from a gene declaration.
    fn generate_gene(&self, gene: &Gene) -> String {
        let struct_name = to_pascal_case(&gene.name);
        let visibility = self.visibility_str();

        // Collect properties from "has" statements
        let fields = self.extract_fields(&gene.statements);

        let mut output = String::new();

        // Doc comment from exegesis (always include by default)
        output.push_str(&self.format_doc_comment(&gene.exegesis));

        // Derive macros
        let derives = self.derive_clause();
        if !derives.is_empty() {
            output.push_str(&format!("#[derive({})]\n", derives));
        }

        // Struct definition
        output.push_str(&format!("{visibility}struct {struct_name} {{\n"));

        for (field_name, field_type, _, _) in &fields {
            let rust_field = to_snake_case(field_name);
            output.push_str(&format!("    {visibility}{rust_field}: {field_type},\n"));
        }

        output.push_str("}\n\n");

        // Generate impl block with constructor and validators
        output.push_str(&self.gen_gene_impl(&struct_name, &fields));

        output
    }

    /// Generate a Rust trait from a trait declaration.
    fn generate_trait(&self, trait_decl: &Trait) -> String {
        let trait_name = to_pascal_case(&trait_decl.name);
        let visibility = self.visibility_str();

        // Collect supertraits from "uses" statements
        let supertraits = self.extract_supertraits(&trait_decl.statements);

        // Collect methods from "is" statements (state transitions become getters)
        let methods = self.extract_methods(&trait_decl.statements);

        let mut output = String::new();

        // Doc comment
        output.push_str(&self.format_doc_comment(&trait_decl.exegesis));

        // Trait definition with supertraits
        let supertrait_clause = if supertraits.is_empty() {
            String::new()
        } else {
            format!(": {}", supertraits.join(" + "))
        };

        output.push_str(&format!(
            "{visibility}trait {trait_name}{supertrait_clause} {{\n"
        ));

        for (method_name, return_type) in &methods {
            let rust_method = to_snake_case(method_name);
            output.push_str(&format!(
                "    /// Get the {} state.\n",
                method_name.replace('_', " ")
            ));
            output.push_str(&format!(
                "    fn {rust_method}(&self) -> {return_type};\n\n"
            ));
        }

        // Remove trailing newline if we have methods
        if !methods.is_empty() {
            output.pop();
        }

        output.push_str("}\n");

        output
    }

    /// Generate Rust assertions/invariants from a constraint declaration.
    fn generate_constraint(&self, constraint: &Constraint) -> String {
        let fn_name = to_snake_case(&constraint.name);
        let visibility = self.visibility_str();

        let mut output = String::new();

        // Doc comment
        output.push_str(&self.format_doc_comment(&constraint.exegesis));

        output.push_str(&format!(
            "/// Validates the {} constraint.\n",
            constraint.name
        ));
        output.push_str(&format!(
            "{visibility}fn validate_{fn_name}<T>(value: &T) -> bool {{\n"
        ));

        // Generate constraint checks from statements
        for stmt in &constraint.statements {
            match stmt {
                Statement::Matches {
                    subject, target, ..
                } => {
                    output.push_str(&format!("    // {subject} matches {target}\n"));
                }
                Statement::Never {
                    subject, action, ..
                } => {
                    output.push_str(&format!("    // {subject} never {action}\n"));
                }
                _ => {}
            }
        }

        output.push_str("    // Implement validation logic based on the constraint rules above\n");
        output.push_str("    true\n");
        output.push_str("}\n");

        output
    }

    /// Generate a Rust module from a system declaration.
    fn generate_system(&self, system: &System) -> String {
        let mod_name = to_snake_case(&system.name);
        let visibility = self.visibility_str();

        let mut output = String::new();

        // Doc comment
        output.push_str(&self.format_doc_comment(&system.exegesis));
        output.push_str(&format!("/// System version: {}\n", system.version));

        // Module with requirements as comments
        output.push_str(&format!("{visibility}mod {mod_name} {{\n"));

        if !system.requirements.is_empty() {
            output.push_str("    //! # Requirements\n    //!\n");
            for req in &system.requirements {
                output.push_str(&format!(
                    "    //! - `{}` {} {}\n",
                    req.name, req.constraint, req.version
                ));
            }
            output.push('\n');
        }

        output.push_str("    // Add system components and implementation here\n");
        output.push_str("}\n");

        output
    }

    /// Generate documentation for an evolution declaration.
    fn generate_evolution(&self, evolution: &Evolution) -> String {
        let mut output = String::new();

        // Evolution is primarily documentation
        output.push_str(&format!(
            "// Evolution: {} @ {} (from {})\n",
            evolution.name, evolution.version, evolution.parent_version
        ));

        if let Some(rationale) = &evolution.rationale {
            output.push_str(&format!("// Rationale: {}\n", rationale));
        }

        output.push_str("//\n");
        output.push_str(&self.format_doc_comment(&evolution.exegesis));

        if !evolution.additions.is_empty() {
            output.push_str("// Additions:\n");
            for stmt in &evolution.additions {
                output.push_str(&format!("//   - {:?}\n", stmt));
            }
        }

        if !evolution.deprecations.is_empty() {
            output.push_str("// Deprecations:\n");
            for stmt in &evolution.deprecations {
                output.push_str(&format!("//   - {:?}\n", stmt));
            }
        }

        if !evolution.removals.is_empty() {
            output.push_str("// Removals:\n");
            for item in &evolution.removals {
                output.push_str(&format!("//   - {}\n", item));
            }
        }

        output
    }

    /// Generate impl block for a gene struct.
    fn gen_gene_impl(
        &self,
        struct_name: &str,
        fields: &[(String, String, Option<Expr>, Option<Expr>)],
    ) -> String {
        let visibility = self.visibility_str();
        let mut output = String::new();

        output.push_str(&format!("impl {struct_name} {{\n"));

        // Generate constructor
        output.push_str(&format!("    {visibility}fn new("));
        let params: Vec<String> = fields
            .iter()
            .map(|(name, ty, _, _)| format!("{}: {}", to_snake_case(name), ty))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(") -> Self {\n");
        output.push_str("        Self {\n");
        for (name, _, _, _) in fields {
            let field = to_snake_case(name);
            output.push_str(&format!("            {field},\n"));
        }
        output.push_str("        }\n");
        output.push_str("    }\n");

        // Generate validators for fields with constraints
        for (name, _, _, constraint) in fields {
            if constraint.is_some() {
                let field = to_snake_case(name);
                output.push_str(&format!(
                    "\n    {visibility}fn validate_{field}(&self) -> bool {{\n"
                ));
                output.push_str("        // TODO: Implement constraint validation\n");
                output.push_str("        true\n");
                output.push_str("    }\n");
            }
        }

        // Generate validate_all if there are any constraints
        let has_constraints = fields.iter().any(|(_, _, _, c)| c.is_some());
        if has_constraints {
            output.push_str(&format!(
                "\n    {visibility}fn validate_all(&self) -> bool {{\n"
            ));
            for (name, _, _, constraint) in fields {
                if constraint.is_some() {
                    let field = to_snake_case(name);
                    output.push_str(&format!(
                        "        if !self.validate_{field}() {{ return false; }}\n"
                    ));
                }
            }
            output.push_str("        true\n");
            output.push_str("    }\n");
        }

        output.push_str("}\n");
        output
    }

    // === Helper Methods ===

    /// Extract fields from statements (for structs).
    /// Returns (field_name, rust_type, optional_default, optional_constraint)
    fn extract_fields(
        &self,
        statements: &[Statement],
    ) -> Vec<(String, String, Option<Expr>, Option<Expr>)> {
        statements
            .iter()
            .filter_map(|stmt| match stmt {
                Statement::Has { property, .. } => {
                    // Legacy: Default to String type for properties without type annotations
                    Some((property.clone(), "String".to_string(), None, None))
                }
                Statement::HasField(field) => {
                    // New: Extract typed field with defaults and constraints
                    let rust_type = Self::map_type_expr(&field.type_);
                    Some((
                        field.name.clone(),
                        rust_type,
                        field.default.clone(),
                        field.constraint.clone(),
                    ))
                }
                _ => None,
            })
            .collect()
    }

    /// Extract supertraits from uses statements.
    fn extract_supertraits(&self, statements: &[Statement]) -> Vec<String> {
        statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Uses { reference, .. } = stmt {
                    Some(to_pascal_case(reference))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract methods from is statements.
    fn extract_methods(&self, statements: &[Statement]) -> Vec<(String, String)> {
        statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Is { state, .. } = stmt {
                    // State becomes a getter method returning bool or state type
                    Some((format!("is_{}", to_snake_case(state)), "bool".to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get visibility string.
    fn visibility_str(&self) -> &'static str {
        match self.options.visibility {
            Visibility::Private => "",
            Visibility::Public => "pub ",
            Visibility::Crate => "pub(crate) ",
        }
    }

    /// Format exegesis as a doc comment.
    fn format_doc_comment(&self, exegesis: &str) -> String {
        let trimmed = exegesis.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        trimmed
            .lines()
            .map(|line| format!("/// {}\n", line.trim()))
            .collect()
    }

    /// Generate derive clause.
    fn derive_clause(&self) -> String {
        let mut derives = vec!["Debug", "Clone"];

        if !self.options.derive_macros.is_empty() {
            derives.extend(self.options.derive_macros.iter().map(|s| s.as_str()));
        }

        derives.join(", ")
    }

    // === SEX (Side Effect eXecution) Code Generation ===

    /// Generate Rust visibility modifier from DOL Visibility.
    pub fn gen_visibility(&self, vis: crate::ast::Visibility) -> &'static str {
        match vis {
            crate::ast::Visibility::Public => "pub ",
            crate::ast::Visibility::PubSpirit => "pub(crate) ",
            crate::ast::Visibility::PubParent => "pub(super) ",
            crate::ast::Visibility::Private => "",
        }
    }

    /// Generate Rust type from DOL TypeExpr.
    pub fn gen_type(&self, ty: &TypeExpr) -> String {
        Self::map_type_expr(ty)
    }

    /// Generate Rust code for a function parameter.
    pub fn gen_param(&self, param: &FunctionParam) -> String {
        format!("{}: {}", param.name, self.gen_type(&param.type_ann))
    }

    /// Generate Rust code for a sex function.
    pub fn gen_sex_function(&self, vis: crate::ast::Visibility, func: &FunctionDecl) -> String {
        let mut output = String::new();

        // Doc comment noting side effects
        output.push_str("    /// Side-effectful function\n");

        // Function signature
        output.push_str("    ");
        output.push_str(self.gen_visibility(vis));
        output.push_str("fn ");
        output.push_str(&func.name);
        output.push('(');

        let params: Vec<String> = func.params.iter().map(|p| self.gen_param(p)).collect();
        output.push_str(&params.join(", "));
        output.push(')');

        if let Some(ref ret) = func.return_type {
            output.push_str(" -> ");
            output.push_str(&self.gen_type(ret));
        }

        output.push_str(" {\n");

        for stmt in &func.body {
            output.push_str(&self.gen_stmt(stmt, 2));
        }

        output.push_str("    }\n");

        output
    }

    /// Generate Rust code for a sex block.
    pub fn gen_sex_block(&self, statements: &[Stmt], final_expr: Option<&Expr>) -> String {
        let mut output = String::new();

        output.push_str("    /* sex block */ {\n");

        for stmt in statements {
            output.push_str(&self.gen_stmt(stmt, 2));
        }
        if let Some(expr) = final_expr {
            output.push_str("        ");
            output.push_str(&self.gen_expr(expr));
            output.push('\n');
        }

        output.push_str("    }\n");

        output
    }

    /// Generate Rust code for a global mutable variable.
    pub fn gen_global_var(&self, var: &VarDecl) -> String {
        let mut output = String::new();

        if var.mutability == Mutability::Mutable {
            // Mutable globals become static mut (unsafe in Rust)
            output.push_str("static mut ");
        } else {
            output.push_str("static ");
        }

        output.push_str(&var.name.to_uppercase());
        output.push_str(": ");

        if let Some(ref type_ann) = var.type_ann {
            output.push_str(&self.gen_type(type_ann));
        } else {
            output.push('_');
        }

        if let Some(ref value) = var.value {
            output.push_str(" = ");
            output.push_str(&self.gen_expr(value));
        }

        output.push_str(";\n");
        output
    }

    /// Generate Rust code for a constant.
    pub fn gen_constant(&self, var: &VarDecl) -> String {
        let mut output = String::new();

        output.push_str("const ");
        output.push_str(&var.name.to_uppercase());
        output.push_str(": ");

        if let Some(ref type_ann) = var.type_ann {
            output.push_str(&self.gen_type(type_ann));
        }

        output.push_str(" = ");
        if let Some(ref value) = var.value {
            output.push_str(&self.gen_expr(value));
        }

        output.push_str(";\n");
        output
    }

    /// Generate Rust code for an extern declaration.
    pub fn gen_extern(&self, decl: &ExternDecl) -> String {
        let mut output = String::new();

        let abi = decl.abi.as_deref().unwrap_or("C");
        output.push_str(&format!("extern \"{}\" {{\n", abi));

        output.push_str("    fn ");
        output.push_str(&decl.name);
        output.push('(');

        let params: Vec<String> = decl.params.iter().map(|p| self.gen_param(p)).collect();
        output.push_str(&params.join(", "));
        output.push(')');

        if let Some(ref ret) = decl.return_type {
            output.push_str(" -> ");
            output.push_str(&self.gen_type(ret));
        }

        output.push_str(";\n");

        output.push_str("}\n");

        output
    }

    /// Generate Rust code for an extern block.
    pub fn gen_extern_block(&self, abi: Option<&str>, functions: &[ExternDecl]) -> String {
        let mut output = String::new();

        let abi_str = abi.unwrap_or("C");
        output.push_str(&format!("extern \"{}\" {{\n", abi_str));

        for func in functions {
            output.push_str("    fn ");
            output.push_str(&func.name);
            output.push('(');

            let params: Vec<String> = func.params.iter().map(|p| self.gen_param(p)).collect();
            output.push_str(&params.join(", "));
            output.push(')');

            if let Some(ref ret) = func.return_type {
                output.push_str(" -> ");
                output.push_str(&self.gen_type(ret));
            }

            output.push_str(";\n");
        }

        output.push_str("}\n");

        output
    }

    /// Generate wrapper for mutable global access.
    pub fn gen_global_access(&self, name: &str) -> String {
        format!("unsafe {{ {} }}", name.to_uppercase())
    }

    /// Generate wrapper for mutable global mutation.
    pub fn gen_global_mutation(&self, name: &str, value: &str) -> String {
        format!("unsafe {{ {} = {}; }}", name.to_uppercase(), value)
    }

    /// Generate Rust code for a statement with indentation.
    fn gen_stmt(&self, stmt: &Stmt, indent_level: usize) -> String {
        let indent = "    ".repeat(indent_level);
        let mut output = String::new();

        match stmt {
            Stmt::Let {
                name,
                type_ann,
                value,
            } => {
                output.push_str(&indent);
                output.push_str("let ");
                output.push_str(name);
                if let Some(ty) = type_ann {
                    output.push_str(": ");
                    output.push_str(&self.gen_type(ty));
                }
                output.push_str(" = ");
                output.push_str(&self.gen_expr(value));
                output.push_str(";\n");
            }
            Stmt::Assign { target, value } => {
                output.push_str(&indent);
                output.push_str(&self.gen_expr(target));
                output.push_str(" = ");
                output.push_str(&self.gen_expr(value));
                output.push_str(";\n");
            }
            Stmt::Return(Some(expr)) => {
                output.push_str(&indent);
                output.push_str("return ");
                output.push_str(&self.gen_expr(expr));
                output.push_str(";\n");
            }
            Stmt::Return(None) => {
                output.push_str(&indent);
                output.push_str("return;\n");
            }
            Stmt::Expr(expr) => {
                output.push_str(&indent);
                output.push_str(&self.gen_expr(expr));
                output.push_str(";\n");
            }
            Stmt::Break => {
                output.push_str(&indent);
                output.push_str("break;\n");
            }
            Stmt::Continue => {
                output.push_str(&indent);
                output.push_str("continue;\n");
            }
            Stmt::For {
                binding,
                iterable,
                body,
            } => {
                output.push_str(&indent);
                output.push_str("for ");
                output.push_str(binding);
                output.push_str(" in ");
                output.push_str(&self.gen_expr(iterable));
                output.push_str(" {\n");
                for s in body {
                    output.push_str(&self.gen_stmt(s, indent_level + 1));
                }
                output.push_str(&indent);
                output.push_str("}\n");
            }
            Stmt::While { condition, body } => {
                output.push_str(&indent);
                output.push_str("while ");
                output.push_str(&self.gen_expr(condition));
                output.push_str(" {\n");
                for s in body {
                    output.push_str(&self.gen_stmt(s, indent_level + 1));
                }
                output.push_str(&indent);
                output.push_str("}\n");
            }
            Stmt::Loop { body } => {
                output.push_str(&indent);
                output.push_str("loop {\n");
                for s in body {
                    output.push_str(&self.gen_stmt(s, indent_level + 1));
                }
                output.push_str(&indent);
                output.push_str("}\n");
            }
        }

        output
    }

    /// Generate Rust code for an expression.
    pub fn gen_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => self.gen_literal(lit),
            Expr::Identifier(name) => name.clone(),
            Expr::Binary { left, op, right } => {
                match op {
                    // Pipe operator: x |> f becomes f(x)
                    crate::ast::BinaryOp::Pipe => {
                        let arg = self.gen_expr(left);
                        let func = self.gen_expr(right);
                        format!("{}({})", func, arg)
                    }
                    // Compose operator: f >> g becomes |__x| g(f(__x))
                    crate::ast::BinaryOp::Compose => {
                        let f = self.gen_expr(left);
                        let g = self.gen_expr(right);
                        format!("|__x| {}({}(__x))", g, f)
                    }
                    // Apply operator: f @ x becomes f(x)
                    crate::ast::BinaryOp::Apply => {
                        let func = self.gen_expr(left);
                        let arg = self.gen_expr(right);
                        format!("{}({})", func, arg)
                    }
                    // Implies operator: a => b becomes (!a || b)
                    crate::ast::BinaryOp::Implies => {
                        let a = self.gen_expr(left);
                        let b = self.gen_expr(right);
                        format!("(!{} || {})", a, b)
                    }
                    // Monadic bind: m := f (depends on monad implementation)
                    crate::ast::BinaryOp::Bind => {
                        let m = self.gen_expr(left);
                        let f = self.gen_expr(right);
                        format!("{}.and_then({})", m, f)
                    }
                    // Functor map: f <$> m becomes m.map(f)
                    crate::ast::BinaryOp::Map => {
                        let f = self.gen_expr(left);
                        let m = self.gen_expr(right);
                        format!("{}.map({})", m, f)
                    }
                    // Applicative apply: mf <*> mx (depends on applicative implementation)
                    crate::ast::BinaryOp::Ap => {
                        let mf = self.gen_expr(left);
                        let mx = self.gen_expr(right);
                        format!("/* applicative apply */ {}.ap({})", mf, mx)
                    }
                    // Exponentiation: x ^ y becomes x.pow(y) for integers
                    crate::ast::BinaryOp::Pow => {
                        let base = self.gen_expr(left);
                        let exp = self.gen_expr(right);
                        format!("{}.pow({} as u32)", base, exp)
                    }
                    // Standard infix operators
                    _ => {
                        let left_str = self.gen_expr(left);
                        let right_str = self.gen_expr(right);
                        let op_str = match op {
                            crate::ast::BinaryOp::Add => "+",
                            crate::ast::BinaryOp::Sub => "-",
                            crate::ast::BinaryOp::Mul => "*",
                            crate::ast::BinaryOp::Div => "/",
                            crate::ast::BinaryOp::Mod => "%",
                            crate::ast::BinaryOp::Eq => "==",
                            crate::ast::BinaryOp::Ne => "!=",
                            crate::ast::BinaryOp::Lt => "<",
                            crate::ast::BinaryOp::Le => "<=",
                            crate::ast::BinaryOp::Gt => ">",
                            crate::ast::BinaryOp::Ge => ">=",
                            crate::ast::BinaryOp::And => "&&",
                            crate::ast::BinaryOp::Or => "||",
                            crate::ast::BinaryOp::Member => ".",
                            _ => "/* unsupported op */",
                        };
                        format!("({} {} {})", left_str, op_str, right_str)
                    }
                }
            }
            Expr::Unary { op, operand } => {
                let operand_str = self.gen_expr(operand);
                let op_str = match op {
                    crate::ast::UnaryOp::Neg => "-",
                    crate::ast::UnaryOp::Not => "!",
                    _ => "/* unsupported op */",
                };
                format!("{}{}", op_str, operand_str)
            }
            Expr::Call { callee, args } => {
                let callee_str = self.gen_expr(callee);
                let args_str: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                format!("{}({})", callee_str, args_str.join(", "))
            }
            Expr::Member { object, field } => {
                format!("{}.{}", self.gen_expr(object), field)
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut output = String::new();
                output.push_str("if ");
                output.push_str(&self.gen_expr(condition));
                output.push_str(" { ");
                output.push_str(&self.gen_expr(then_branch));
                output.push_str(" }");
                if let Some(else_br) = else_branch {
                    output.push_str(" else { ");
                    output.push_str(&self.gen_expr(else_br));
                    output.push_str(" }");
                }
                output
            }
            Expr::Block {
                statements,
                final_expr,
            } => {
                let mut output = String::new();
                output.push_str("{\n");
                for stmt in statements {
                    output.push_str(&self.gen_stmt(stmt, 1));
                }
                if let Some(expr) = final_expr {
                    output.push_str("    ");
                    output.push_str(&self.gen_expr(expr));
                    output.push('\n');
                }
                output.push('}');
                output
            }
            Expr::SexBlock {
                statements,
                final_expr,
            } => {
                let mut output = String::new();
                output.push_str("/* sex */ {\n");
                for stmt in statements {
                    output.push_str(&self.gen_stmt(stmt, 1));
                }
                if let Some(expr) = final_expr {
                    output.push_str("    ");
                    output.push_str(&self.gen_expr(expr));
                    output.push('\n');
                }
                output.push('}');
                output
            }
            Expr::Lambda {
                params,
                return_type,
                body,
            } => {
                let mut output = String::new();
                output.push('|');

                // Generate parameters with optional type annotations
                let params_str: Vec<String> = params
                    .iter()
                    .map(|(name, type_ann)| {
                        if let Some(ty) = type_ann {
                            format!("{}: {}", name, Self::map_type_expr(ty))
                        } else {
                            name.clone()
                        }
                    })
                    .collect();
                output.push_str(&params_str.join(", "));
                output.push('|');

                // Add return type if specified
                if let Some(ret_ty) = return_type {
                    output.push_str(" -> ");
                    output.push_str(&Self::map_type_expr(ret_ty));
                }

                // Generate body
                output.push_str(" { ");
                output.push_str(&self.gen_expr(body));
                output.push_str(" }");

                output
            }
            Expr::Match { scrutinee, arms } => self.gen_match(scrutinee, arms),
            // Meta-programming expressions
            Expr::Quote(inner) => self.gen_quote(inner),
            Expr::Eval(inner) => self.gen_eval(inner),
            Expr::Reflect(ty) => self.gen_reflect(ty),
            Expr::QuasiQuote(inner) => self.gen_quasi_quote(inner),
            Expr::Unquote(inner) => self.gen_unquote(inner),
            Expr::IdiomBracket { func, args } => self.gen_idiom_bracket(func, args),
            // Logical expressions
            Expr::Forall(_) | Expr::Exists(_) | Expr::Implies { .. } => {
                "/* logical expression not yet supported */".to_string()
            }
        }
    }

    /// Generate Rust code for a literal.
    fn gen_literal(&self, lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => format!("{}_i64", n),
            Literal::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}_f64", f)
                } else {
                    format!("{}_f64", f)
                }
            }
            Literal::Bool(b) => b.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Null => "None".to_string(),
        }
    }

    /// Generate Rust code for a pattern in a match expression.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern to generate code for
    ///
    /// # Returns
    ///
    /// Rust code representing the pattern
    ///
    /// # Example Output
    ///
    /// ```ignore
    /// // Pattern::Wildcard => "_"
    /// // Pattern::Identifier("x") => "x"
    /// // Pattern::Literal(Literal::Int(42)) => "42_i64"
    /// // Pattern::Constructor { name: "Some", fields: [Pattern::Identifier("x")] } => "Some(x)"
    /// // Pattern::Tuple([Pattern::Identifier("x"), Pattern::Identifier("y")]) => "(x, y)"
    /// ```
    fn gen_pattern(&self, pattern: &crate::ast::Pattern) -> String {
        match pattern {
            crate::ast::Pattern::Wildcard => "_".to_string(),
            crate::ast::Pattern::Identifier(name) => name.clone(),
            crate::ast::Pattern::Literal(lit) => self.gen_literal(lit),
            crate::ast::Pattern::Constructor { name, fields } => {
                let fields_str: Vec<String> = fields.iter().map(|p| self.gen_pattern(p)).collect();
                if fields.is_empty() {
                    name.clone()
                } else {
                    format!("{}({})", name, fields_str.join(", "))
                }
            }
            crate::ast::Pattern::Tuple(patterns) => {
                let patterns_str: Vec<String> =
                    patterns.iter().map(|p| self.gen_pattern(p)).collect();
                format!("({})", patterns_str.join(", "))
            }
        }
    }

    /// Generate Rust code for a match expression.
    ///
    /// # Arguments
    ///
    /// * `scrutinee` - The expression being matched
    /// * `arms` - The match arms (patterns, guards, and bodies)
    ///
    /// # Returns
    ///
    /// Rust code for the complete match expression
    ///
    /// # Example Output
    ///
    /// ```ignore
    /// match value {
    ///     Some(x) => x,
    ///     None => 0_i64
    /// }
    /// ```
    fn gen_match(&self, scrutinee: &Expr, arms: &[crate::ast::MatchArm]) -> String {
        let scrutinee_code = self.gen_expr(scrutinee);
        let arms_code: Vec<String> = arms
            .iter()
            .map(|arm| {
                let pattern = self.gen_pattern(&arm.pattern);
                let guard = arm
                    .guard
                    .as_ref()
                    .map(|g| format!(" if {}", self.gen_expr(g)))
                    .unwrap_or_default();
                let body = self.gen_expr(&arm.body);
                format!("{}{} => {}", pattern, guard, body)
            })
            .collect();
        format!(
            "match {} {{\n    {}\n}}",
            scrutinee_code,
            arms_code.join(",\n    ")
        )
    }

    // === Meta-Programming Code Generation ===

    /// Generate Rust code for a quote expression.
    ///
    /// Converts an expression to a QuotedExpr literal that can be manipulated at runtime.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to quote
    ///
    /// # Returns
    ///
    /// Rust code that creates a QuotedExpr from the expression
    ///
    /// # Example
    ///
    /// ```ignore
    /// // '(1 + 2) => QuotedExpr::from_expr(&Expr::Binary { ... })
    /// ```
    fn gen_quote(&self, expr: &Expr) -> String {
        // Convert expression to QuotedExpr literal
        format!("QuotedExpr::from_expr(&{})", self.gen_expr(expr))
    }

    /// Generate Rust code for an eval expression.
    ///
    /// Evaluates a quoted expression at runtime.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression containing the quoted code to evaluate
    ///
    /// # Returns
    ///
    /// Rust code that evaluates the expression
    ///
    /// # Example
    ///
    /// ```ignore
    /// // eval(quoted_expr) => eval(quoted_expr)
    /// ```
    fn gen_eval(&self, expr: &Expr) -> String {
        format!("eval({})", self.gen_expr(expr))
    }

    /// Generate Rust code for type reflection.
    ///
    /// Generates code that retrieves type information at runtime.
    ///
    /// # Arguments
    ///
    /// * `ty` - The type to reflect on
    ///
    /// # Returns
    ///
    /// Rust code that gets type information
    ///
    /// # Example
    ///
    /// ```ignore
    /// // ?Int32 => TypeInfo::get::<i32>()
    /// ```
    fn gen_reflect(&self, ty: &TypeExpr) -> String {
        format!("TypeInfo::get::<{}>()", self.gen_type(ty))
    }

    /// Generate Rust code for a quasi-quote expression.
    ///
    /// Similar to quote but allows unquote/splice operations inside.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to quasi-quote
    ///
    /// # Returns
    ///
    /// Rust code for the quasi-quoted expression
    ///
    /// # Example
    ///
    /// ```ignore
    /// // `(1 + ,x) => allows unquoting x inside the quote
    /// ```
    fn gen_quasi_quote(&self, expr: &Expr) -> String {
        // Similar to quote but supports unquotes
        format!("QuotedExpr::from_expr(&{})", self.gen_expr(expr))
    }

    /// Generate Rust code for an unquote/splice expression.
    ///
    /// Splices an evaluated expression into a quasi-quoted context.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to unquote
    ///
    /// # Returns
    ///
    /// Rust code for the unquoted expression
    ///
    /// # Example
    ///
    /// ```ignore
    /// // ,x inside a quasi-quote evaluates x and splices it in
    /// ```
    fn gen_unquote(&self, expr: &Expr) -> String {
        // Unquote evaluates and splices the expression
        self.gen_expr(expr)
    }

    /// Generate Rust code for idiom bracket expressions.
    ///
    /// Idiom brackets provide applicative functor style: [| f a b |] desugars to f <$> a <*> b
    ///
    /// # Arguments
    ///
    /// * `func` - The function to apply
    /// * `args` - The arguments to lift and apply
    ///
    /// # Returns
    ///
    /// Rust code for the applicative style expression
    ///
    /// # Example
    ///
    /// ```ignore
    /// // [| f a b |] => a.map(f).ap(b)
    /// // [| f |] => f (no lifting needed)
    /// ```
    fn gen_idiom_bracket(&self, func: &Expr, args: &[Expr]) -> String {
        if args.is_empty() {
            // No arguments, just return the function
            self.gen_expr(func)
        } else {
            // [| f a b c |] desugars to f <$> a <*> b <*> c
            // In Rust: a.map(f).ap(b).ap(c)
            let mut result = format!("{}.map({})", self.gen_expr(&args[0]), self.gen_expr(func));
            for arg in args.iter().skip(1) {
                result = format!("{}.ap({})", result, self.gen_expr(arg));
            }
            result
        }
    }

    /// Generate TypeInfo registration code for a gene.
    ///
    /// This generates code that registers runtime type information
    /// for reflection and introspection.
    ///
    /// # Arguments
    ///
    /// * `gene` - The gene declaration to generate TypeInfo for
    ///
    /// # Returns
    ///
    /// Rust code that creates and registers a TypeInfo instance
    ///
    /// # Example Output
    ///
    /// ```ignore
    /// TypeInfo::record("User")
    ///     .with_field(FieldInfo::new("id", "Int64"))
    ///     .with_field(FieldInfo::new("name", "String"))
    ///     .with_doc("A user entity")
    /// ```
    pub fn gen_type_info(&self, gene: &Gene) -> String {
        let type_name = to_pascal_case(&gene.name);
        let mut output = String::new();

        // Start TypeInfo::record call
        output.push_str(&format!("    TypeInfo::record(\"{}\")\n", type_name));

        // Add fields from "has" statements
        for stmt in &gene.statements {
            if let Statement::Has { property, .. } = stmt {
                let field_name = to_snake_case(property);
                // Default to String type - in a full implementation we'd infer types
                output.push_str(&format!(
                    "        .with_field(FieldInfo::new(\"{}\", \"String\"))\n",
                    field_name
                ));
            }
        }

        // Add documentation from exegesis
        if !gene.exegesis.trim().is_empty() {
            let escaped_doc = gene.exegesis.replace('\"', "\\\"").replace('\n', " ");
            output.push_str(&format!("        .with_doc(\"{}\")", escaped_doc));
        }

        output
    }

    /// Generate TypeInfo registration for all declarations in a module.
    ///
    /// This generates a function that populates a TypeRegistry with
    /// type information for all genes and types in the module.
    ///
    /// # Arguments
    ///
    /// * `decls` - The declarations to generate registration code for
    ///
    /// # Returns
    ///
    /// Rust code that defines a registration function
    pub fn gen_type_registry(&self, decls: &[Declaration]) -> String {
        let mut output = String::new();

        output.push_str("/// Register all type information in the type registry.\n");
        output.push_str("pub fn register_types(registry: &mut TypeRegistry) {\n");

        for decl in decls {
            if let Declaration::Gene(gene) = decl {
                output.push_str(&format!(
                    "    registry.register(\n{}\n    );\n\n",
                    self.gen_type_info(gene)
                ));
            }
        }

        output.push_str("}\n");
        output
    }
}

impl Codegen for RustCodegen {
    fn generate(decl: &Declaration) -> String {
        RustCodegen::generate(decl)
    }
}

impl TypeMapper for RustCodegen {
    fn map_type(ty: &Type) -> String {
        match ty {
            Type::Void => "()".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int8 => "i8".to_string(),
            Type::Int16 => "i16".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::UInt8 => "u8".to_string(),
            Type::UInt16 => "u16".to_string(),
            Type::UInt32 => "u32".to_string(),
            Type::UInt64 => "u64".to_string(),
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),
            Type::String => "String".to_string(),
            Type::Function {
                params,
                return_type,
            } => {
                let param_types: Vec<_> = params.iter().map(Self::map_type).collect();
                let ret = Self::map_type(return_type);
                format!("fn({}) -> {}", param_types.join(", "), ret)
            }
            Type::Tuple(types) => {
                let mapped: Vec<_> = types.iter().map(Self::map_type).collect();
                format!("({})", mapped.join(", "))
            }
            Type::Generic { name, args } => {
                let mapped_args: Vec<_> = args.iter().map(Self::map_type).collect();
                // Map DOL generic types to Rust equivalents
                let rust_name = match name.as_str() {
                    "List" => "Vec",
                    "Map" => "std::collections::HashMap",
                    "Option" => "Option",
                    "Result" => "Result",
                    _ => name.as_str(),
                };
                if args.is_empty() {
                    to_pascal_case(name)
                } else {
                    format!("{}<{}>", rust_name, mapped_args.join(", "))
                }
            }
            Type::Var(id) => format!("T{}", id),
            Type::Any => "Box<dyn std::any::Any>".to_string(),
            Type::Unknown => "/* unknown */".to_string(),
            Type::Error => "/* error */".to_string(),
        }
    }

    fn map_type_expr(ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(name) => match name.as_str() {
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
                "Void" => "()".to_string(),
                _ => to_pascal_case(name),
            },
            TypeExpr::Generic { name, args } => {
                let mapped_args: Vec<_> = args.iter().map(Self::map_type_expr).collect();
                let rust_name = match name.as_str() {
                    "List" => "Vec",
                    "Map" => "std::collections::HashMap",
                    "Option" => "Option",
                    "Result" => "Result",
                    _ => name,
                };
                format!("{}<{}>", rust_name, mapped_args.join(", "))
            }
            TypeExpr::Function {
                params,
                return_type,
            } => {
                let param_types: Vec<_> = params.iter().map(Self::map_type_expr).collect();
                let ret = Self::map_type_expr(return_type);
                format!("fn({}) -> {}", param_types.join(", "), ret)
            }
            TypeExpr::Tuple(types) => {
                let mapped: Vec<_> = types.iter().map(Self::map_type_expr).collect();
                format!("({})", mapped.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    #[test]
    fn test_generate_gene_struct() {
        let gene = Gene {
            name: "container.exists".to_string(),
            statements: vec![
                Statement::Has {
                    subject: "container".to_string(),
                    property: "id".to_string(),
                    span: Span::default(),
                },
                Statement::Has {
                    subject: "container".to_string(),
                    property: "image".to_string(),
                    span: Span::default(),
                },
            ],
            exegesis: "A container is the fundamental unit.".to_string(),
            span: Span::default(),
        };

        let code = RustCodegen::generate(&Declaration::Gene(gene));

        assert!(code.contains("pub struct ContainerExists"));
        assert!(code.contains("pub id: String"));
        assert!(code.contains("pub image: String"));
        assert!(code.contains("/// A container is the fundamental unit."));
    }

    #[test]
    fn test_generate_trait() {
        let trait_decl = Trait {
            name: "container.lifecycle".to_string(),
            statements: vec![
                Statement::Uses {
                    reference: "container.exists".to_string(),
                    span: Span::default(),
                },
                Statement::Is {
                    subject: "container".to_string(),
                    state: "created".to_string(),
                    span: Span::default(),
                },
                Statement::Is {
                    subject: "container".to_string(),
                    state: "started".to_string(),
                    span: Span::default(),
                },
            ],
            exegesis: "Container lifecycle management.".to_string(),
            span: Span::default(),
        };

        let code = RustCodegen::generate(&Declaration::Trait(trait_decl));

        assert!(code.contains("pub trait ContainerLifecycle: ContainerExists"));
        assert!(code.contains("fn is_created(&self) -> bool"));
        assert!(code.contains("fn is_started(&self) -> bool"));
    }

    #[test]
    fn test_map_type() {
        assert_eq!(RustCodegen::map_type(&Type::Int32), "i32");
        assert_eq!(RustCodegen::map_type(&Type::String), "String");
        assert_eq!(RustCodegen::map_type(&Type::Bool), "bool");
        assert_eq!(
            RustCodegen::map_type(&Type::Generic {
                name: "Option".to_string(),
                args: vec![Type::String]
            }),
            "Option<String>"
        );
        assert_eq!(
            RustCodegen::map_type(&Type::Generic {
                name: "List".to_string(),
                args: vec![Type::Int32]
            }),
            "Vec<i32>"
        );
        assert_eq!(
            RustCodegen::map_type(&Type::Generic {
                name: "Map".to_string(),
                args: vec![Type::String, Type::Int64]
            }),
            "std::collections::HashMap<String, i64>"
        );
    }

    #[test]
    fn test_map_type_expr() {
        assert_eq!(
            RustCodegen::map_type_expr(&TypeExpr::Named("Int32".to_string())),
            "i32"
        );
        assert_eq!(
            RustCodegen::map_type_expr(&TypeExpr::Generic {
                name: "List".to_string(),
                args: vec![TypeExpr::Named("String".to_string())]
            }),
            "Vec<String>"
        );
        assert_eq!(
            RustCodegen::map_type_expr(&TypeExpr::Generic {
                name: "Map".to_string(),
                args: vec![
                    TypeExpr::Named("String".to_string()),
                    TypeExpr::Named("Int32".to_string())
                ]
            }),
            "std::collections::HashMap<String, i32>"
        );
    }

    #[test]
    fn test_generate_constraint() {
        let constraint = Constraint {
            name: "container.integrity".to_string(),
            statements: vec![
                Statement::Matches {
                    subject: "state".to_string(),
                    target: "declared_state".to_string(),
                    span: Span::default(),
                },
                Statement::Never {
                    subject: "identity".to_string(),
                    action: "changes".to_string(),
                    span: Span::default(),
                },
            ],
            exegesis: "Container integrity constraints.".to_string(),
            span: Span::default(),
        };

        let code = RustCodegen::generate(&Declaration::Constraint(constraint));

        assert!(code.contains("fn validate_container_integrity"));
        assert!(code.contains("// state matches declared_state"));
        assert!(code.contains("// identity never changes"));
    }

    #[test]
    fn test_generate_system() {
        let system = System {
            name: "univrs.orchestrator".to_string(),
            version: "0.1.0".to_string(),
            requirements: vec![crate::ast::Requirement {
                name: "container.lifecycle".to_string(),
                constraint: ">=".to_string(),
                version: "0.0.2".to_string(),
                span: Span::default(),
            }],
            statements: vec![],
            exegesis: "The Univrs orchestrator.".to_string(),
            span: Span::default(),
        };

        let code = RustCodegen::generate(&Declaration::System(system));

        assert!(code.contains("pub mod univrs_orchestrator"));
        assert!(code.contains("/// System version: 0.1.0"));
        assert!(code.contains("`container.lifecycle` >= 0.0.2"));
    }

    // === SEX Code Generation Tests ===

    #[test]
    fn test_gen_type_primitives() {
        let gen = RustCodegen::new();

        assert_eq!(gen.gen_type(&TypeExpr::Named("Int32".to_string())), "i32");
        assert_eq!(gen.gen_type(&TypeExpr::Named("Int64".to_string())), "i64");
        assert_eq!(gen.gen_type(&TypeExpr::Named("Bool".to_string())), "bool");
        assert_eq!(
            gen.gen_type(&TypeExpr::Named("String".to_string())),
            "String"
        );
        assert_eq!(gen.gen_type(&TypeExpr::Named("Void".to_string())), "()");
    }

    #[test]
    fn test_gen_type_generic() {
        let gen = RustCodegen::new();

        let list_type = TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![TypeExpr::Named("Int32".to_string())],
        };
        assert_eq!(gen.gen_type(&list_type), "Vec<i32>");
    }

    #[test]
    fn test_gen_constant() {
        let gen = RustCodegen::new();

        let var = VarDecl {
            mutability: Mutability::Immutable,
            name: "MAX_SIZE".to_string(),
            type_ann: Some(TypeExpr::Named("Int64".to_string())),
            value: Some(Expr::Literal(Literal::Int(100))),
            span: Span::default(),
        };

        let output = gen.gen_constant(&var);
        assert!(output.contains("const MAX_SIZE: i64 = 100_i64;"));
    }

    #[test]
    fn test_gen_global_var() {
        let gen = RustCodegen::new();

        let var = VarDecl {
            mutability: Mutability::Mutable,
            name: "counter".to_string(),
            type_ann: Some(TypeExpr::Named("Int64".to_string())),
            value: Some(Expr::Literal(Literal::Int(0))),
            span: Span::default(),
        };

        let output = gen.gen_global_var(&var);
        assert!(output.contains("static mut COUNTER: i64 = 0_i64;"));
    }

    #[test]
    fn test_gen_extern() {
        let gen = RustCodegen::new();

        let decl = ExternDecl {
            abi: Some("C".to_string()),
            name: "malloc".to_string(),
            params: vec![FunctionParam {
                name: "size".to_string(),
                type_ann: TypeExpr::Named("UInt64".to_string()),
            }],
            return_type: Some(TypeExpr::Generic {
                name: "Ptr".to_string(),
                args: vec![TypeExpr::Named("Void".to_string())],
            }),
            span: Span::default(),
        };

        let output = gen.gen_extern(&decl);
        assert!(output.contains("extern \"C\" {"));
        assert!(output.contains("fn malloc(size: u64) -> Ptr<()>;"));
    }

    #[test]
    fn test_gen_global_access() {
        let gen = RustCodegen::new();
        assert_eq!(gen.gen_global_access("counter"), "unsafe { COUNTER }");
    }

    #[test]
    fn test_gen_global_mutation() {
        let gen = RustCodegen::new();
        assert_eq!(
            gen.gen_global_mutation("counter", "42"),
            "unsafe { COUNTER = 42; }"
        );
    }

    #[test]
    fn test_gen_literal() {
        let gen = RustCodegen::new();
        assert_eq!(gen.gen_literal(&Literal::Int(42)), "42_i64");
        assert_eq!(gen.gen_literal(&Literal::Float(2.5)), "2.5_f64");
        assert_eq!(gen.gen_literal(&Literal::Float(3.0)), "3.0_f64");
        assert_eq!(gen.gen_literal(&Literal::Bool(true)), "true");
        assert_eq!(
            gen.gen_literal(&Literal::String("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(gen.gen_literal(&Literal::Null), "None");
    }

    #[test]
    fn test_gen_visibility() {
        let gen = RustCodegen::new();
        assert_eq!(gen.gen_visibility(crate::ast::Visibility::Public), "pub ");
        assert_eq!(
            gen.gen_visibility(crate::ast::Visibility::PubSpirit),
            "pub(crate) "
        );
        assert_eq!(
            gen.gen_visibility(crate::ast::Visibility::PubParent),
            "pub(super) "
        );
        assert_eq!(gen.gen_visibility(crate::ast::Visibility::Private), "");
    }

    #[test]
    fn test_gen_sex_function() {
        let gen = RustCodegen::new();

        let func = FunctionDecl {
            visibility: crate::ast::Visibility::Public,
            purity: crate::ast::Purity::Sex,
            name: "mutate".to_string(),
            type_params: None,
            params: vec![FunctionParam {
                name: "x".to_string(),
                type_ann: TypeExpr::Named("Int32".to_string()),
            }],
            return_type: Some(TypeExpr::Named("Int32".to_string())),
            body: vec![Stmt::Return(Some(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Add,
                right: Box::new(Expr::Literal(Literal::Int(1))),
            }))],
            span: Span::default(),
        };

        let output = gen.gen_sex_function(crate::ast::Visibility::Public, &func);
        assert!(output.contains("/// Side-effectful function"));
        assert!(output.contains("pub fn mutate(x: i32) -> i32"));
        assert!(output.contains("return (x + 1_i64);"));
    }

    #[test]
    fn test_gen_expr_binary() {
        let gen = RustCodegen::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(2))),
            op: crate::ast::BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(3))),
        };
        assert_eq!(gen.gen_expr(&expr), "(2_i64 * 3_i64)");
    }

    #[test]
    fn test_gen_expr_call() {
        let gen = RustCodegen::new();
        let expr = Expr::Call {
            callee: Box::new(Expr::Identifier("foo".to_string())),
            args: vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
            ],
        };
        assert_eq!(gen.gen_expr(&expr), "foo(1_i64, 2_i64)");
    }

    // === Lambda Expression Tests ===

    #[test]
    fn test_gen_lambda_simple() {
        let gen = RustCodegen::new();
        let expr = Expr::Lambda {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Add,
                right: Box::new(Expr::Literal(Literal::Int(1))),
            }),
        };
        assert_eq!(gen.gen_expr(&expr), "|x| { (x + 1_i64) }");
    }

    #[test]
    fn test_gen_lambda_with_type() {
        let gen = RustCodegen::new();
        let expr = Expr::Lambda {
            params: vec![("x".to_string(), Some(TypeExpr::Named("Int32".to_string())))],
            return_type: None,
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Mul,
                right: Box::new(Expr::Literal(Literal::Int(2))),
            }),
        };
        assert_eq!(gen.gen_expr(&expr), "|x: i32| { (x * 2_i64) }");
    }

    #[test]
    fn test_gen_lambda_multiple_params() {
        let gen = RustCodegen::new();
        let expr = Expr::Lambda {
            params: vec![
                ("x".to_string(), Some(TypeExpr::Named("Int32".to_string()))),
                ("y".to_string(), Some(TypeExpr::Named("Int32".to_string()))),
            ],
            return_type: None,
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Add,
                right: Box::new(Expr::Identifier("y".to_string())),
            }),
        };
        assert_eq!(gen.gen_expr(&expr), "|x: i32, y: i32| { (x + y) }");
    }

    #[test]
    fn test_gen_lambda_with_return_type() {
        let gen = RustCodegen::new();
        let expr = Expr::Lambda {
            params: vec![("x".to_string(), Some(TypeExpr::Named("Int32".to_string())))],
            return_type: Some(TypeExpr::Named("Int32".to_string())),
            body: Box::new(Expr::Identifier("x".to_string())),
        };
        assert_eq!(gen.gen_expr(&expr), "|x: i32| -> i32 { x }");
    }

    #[test]
    fn test_gen_lambda_with_block_body() {
        let gen = RustCodegen::new();
        let expr = Expr::Lambda {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Block {
                statements: vec![Stmt::Let {
                    name: "doubled".to_string(),
                    type_ann: None,
                    value: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        op: crate::ast::BinaryOp::Mul,
                        right: Box::new(Expr::Literal(Literal::Int(2))),
                    },
                }],
                final_expr: Some(Box::new(Expr::Identifier("doubled".to_string()))),
            }),
        };
        let result = gen.gen_expr(&expr);
        assert!(result.starts_with("|x| {"));
        assert!(result.contains("let doubled = (x * 2_i64);"));
        assert!(result.contains("doubled"));
    }

    #[test]
    fn test_gen_lambda_no_params() {
        let gen = RustCodegen::new();
        let expr = Expr::Lambda {
            params: vec![],
            return_type: None,
            body: Box::new(Expr::Literal(Literal::Int(42))),
        };
        assert_eq!(gen.gen_expr(&expr), "|| { 42_i64 }");
    }

    #[test]
    fn test_gen_lambda_nested() {
        let gen = RustCodegen::new();
        // Create a lambda that returns another lambda: |x| |y| x + y
        let inner_lambda = Expr::Lambda {
            params: vec![("y".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Add,
                right: Box::new(Expr::Identifier("y".to_string())),
            }),
        };
        let outer_lambda = Expr::Lambda {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(inner_lambda),
        };
        assert_eq!(gen.gen_expr(&outer_lambda), "|x| { |y| { (x + y) } }");
    }

    // === Method Call Tests ===

    #[test]
    fn test_gen_method_call_simple() {
        let gen = RustCodegen::new();
        // obj.method(arg)
        let expr = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Identifier("obj".to_string())),
                field: "method".to_string(),
            }),
            args: vec![Expr::Literal(Literal::Int(42))],
        };
        assert_eq!(gen.gen_expr(&expr), "obj.method(42_i64)");
    }

    #[test]
    fn test_gen_method_call_no_args() {
        let gen = RustCodegen::new();
        // obj.get()
        let expr = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Identifier("obj".to_string())),
                field: "get".to_string(),
            }),
            args: vec![],
        };
        assert_eq!(gen.gen_expr(&expr), "obj.get()");
    }

    #[test]
    fn test_gen_method_call_multiple_args() {
        let gen = RustCodegen::new();
        // obj.calculate(1, 2, 3)
        let expr = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Identifier("obj".to_string())),
                field: "calculate".to_string(),
            }),
            args: vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
                Expr::Literal(Literal::Int(3)),
            ],
        };
        assert_eq!(gen.gen_expr(&expr), "obj.calculate(1_i64, 2_i64, 3_i64)");
    }

    #[test]
    fn test_gen_method_call_chained() {
        let gen = RustCodegen::new();
        // obj.method1().method2(42)
        let inner_call = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Identifier("obj".to_string())),
                field: "method1".to_string(),
            }),
            args: vec![],
        };
        let outer_call = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(inner_call),
                field: "method2".to_string(),
            }),
            args: vec![Expr::Literal(Literal::Int(42))],
        };
        assert_eq!(gen.gen_expr(&outer_call), "obj.method1().method2(42_i64)");
    }

    #[test]
    fn test_gen_method_call_on_member() {
        let gen = RustCodegen::new();
        // obj.field.method(arg)
        let expr = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Member {
                    object: Box::new(Expr::Identifier("obj".to_string())),
                    field: "field".to_string(),
                }),
                field: "method".to_string(),
            }),
            args: vec![Expr::Literal(Literal::String("test".to_string()))],
        };
        assert_eq!(gen.gen_expr(&expr), "obj.field.method(\"test\")");
    }

    #[test]
    fn test_gen_lambda_as_argument() {
        let gen = RustCodegen::new();
        // map(|x| x + 1)
        let lambda = Expr::Lambda {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Add,
                right: Box::new(Expr::Literal(Literal::Int(1))),
            }),
        };
        let call = Expr::Call {
            callee: Box::new(Expr::Identifier("map".to_string())),
            args: vec![lambda],
        };
        assert_eq!(gen.gen_expr(&call), "map(|x| { (x + 1_i64) })");
    }

    #[test]
    fn test_gen_lambda_in_method_call() {
        let gen = RustCodegen::new();
        // collection.map(|x| x * 2)
        let lambda = Expr::Lambda {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Mul,
                right: Box::new(Expr::Literal(Literal::Int(2))),
            }),
        };
        let method_call = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Identifier("collection".to_string())),
                field: "map".to_string(),
            }),
            args: vec![lambda],
        };
        assert_eq!(
            gen.gen_expr(&method_call),
            "collection.map(|x| { (x * 2_i64) })"
        );
    }

    // === Meta-Programming Tests ===

    #[test]
    fn test_gen_quote() {
        let gen = RustCodegen::new();
        // '(1 + 2)
        let expr = Expr::Quote(Box::new(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: crate::ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        }));
        assert_eq!(
            gen.gen_expr(&expr),
            "QuotedExpr::from_expr(&(1_i64 + 2_i64))"
        );
    }

    #[test]
    fn test_gen_quote_identifier() {
        let gen = RustCodegen::new();
        // 'x
        let expr = Expr::Quote(Box::new(Expr::Identifier("x".to_string())));
        assert_eq!(gen.gen_expr(&expr), "QuotedExpr::from_expr(&x)");
    }

    #[test]
    fn test_gen_eval() {
        let gen = RustCodegen::new();
        // eval(quoted_expr)
        let expr = Expr::Eval(Box::new(Expr::Identifier("quoted_expr".to_string())));
        assert_eq!(gen.gen_expr(&expr), "eval(quoted_expr)");
    }

    #[test]
    fn test_gen_eval_with_quote() {
        let gen = RustCodegen::new();
        // eval('(1 + 2))
        let expr = Expr::Eval(Box::new(Expr::Quote(Box::new(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: crate::ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        }))));
        assert_eq!(
            gen.gen_expr(&expr),
            "eval(QuotedExpr::from_expr(&(1_i64 + 2_i64)))"
        );
    }

    #[test]
    fn test_gen_reflect() {
        let gen = RustCodegen::new();
        // ?Int32
        let expr = Expr::Reflect(Box::new(TypeExpr::Named("Int32".to_string())));
        assert_eq!(gen.gen_expr(&expr), "TypeInfo::get::<i32>()");
    }

    #[test]
    fn test_gen_reflect_generic() {
        let gen = RustCodegen::new();
        // ?List<Int32>
        let expr = Expr::Reflect(Box::new(TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![TypeExpr::Named("Int32".to_string())],
        }));
        assert_eq!(gen.gen_expr(&expr), "TypeInfo::get::<Vec<i32>>()");
    }

    #[test]
    fn test_gen_quasi_quote() {
        let gen = RustCodegen::new();
        // `(1 + 2)
        let expr = Expr::QuasiQuote(Box::new(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: crate::ast::BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        }));
        assert_eq!(
            gen.gen_expr(&expr),
            "QuotedExpr::from_expr(&(1_i64 + 2_i64))"
        );
    }

    #[test]
    fn test_gen_unquote() {
        let gen = RustCodegen::new();
        // ,x (inside a quasi-quote)
        let expr = Expr::Unquote(Box::new(Expr::Identifier("x".to_string())));
        assert_eq!(gen.gen_expr(&expr), "x");
    }

    #[test]
    fn test_gen_idiom_bracket_no_args() {
        let gen = RustCodegen::new();
        // [| f |] => f
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![],
        };
        assert_eq!(gen.gen_expr(&expr), "f");
    }

    #[test]
    fn test_gen_idiom_bracket_one_arg() {
        let gen = RustCodegen::new();
        // [| f a |] => a.map(f)
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![Expr::Identifier("a".to_string())],
        };
        assert_eq!(gen.gen_expr(&expr), "a.map(f)");
    }

    #[test]
    fn test_gen_idiom_bracket_two_args() {
        let gen = RustCodegen::new();
        // [| f a b |] => a.map(f).ap(b)
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![
                Expr::Identifier("a".to_string()),
                Expr::Identifier("b".to_string()),
            ],
        };
        assert_eq!(gen.gen_expr(&expr), "a.map(f).ap(b)");
    }

    #[test]
    fn test_gen_idiom_bracket_three_args() {
        let gen = RustCodegen::new();
        // [| f a b c |] => a.map(f).ap(b).ap(c)
        let expr = Expr::IdiomBracket {
            func: Box::new(Expr::Identifier("f".to_string())),
            args: vec![
                Expr::Identifier("a".to_string()),
                Expr::Identifier("b".to_string()),
                Expr::Identifier("c".to_string()),
            ],
        };
        assert_eq!(gen.gen_expr(&expr), "a.map(f).ap(b).ap(c)");
    }

    #[test]
    fn test_gen_idiom_bracket_with_lambda() {
        let gen = RustCodegen::new();
        // [| |x| x + 1  opt_val |] => opt_val.map(|x| { (x + 1_i64) })
        let lambda = Expr::Lambda {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: crate::ast::BinaryOp::Add,
                right: Box::new(Expr::Literal(Literal::Int(1))),
            }),
        };
        let expr = Expr::IdiomBracket {
            func: Box::new(lambda),
            args: vec![Expr::Identifier("opt_val".to_string())],
        };
        assert_eq!(gen.gen_expr(&expr), "opt_val.map(|x| { (x + 1_i64) })");
    }

    #[test]
    fn test_gen_nested_quote() {
        let gen = RustCodegen::new();
        // ''x
        let expr = Expr::Quote(Box::new(Expr::Quote(Box::new(Expr::Identifier(
            "x".to_string(),
        )))));
        assert_eq!(
            gen.gen_expr(&expr),
            "QuotedExpr::from_expr(&QuotedExpr::from_expr(&x))"
        );
    }

    #[test]
    fn test_gen_quote_function_call() {
        let gen = RustCodegen::new();
        // '(f(1, 2))
        let expr = Expr::Quote(Box::new(Expr::Call {
            callee: Box::new(Expr::Identifier("f".to_string())),
            args: vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
            ],
        }));
        assert_eq!(
            gen.gen_expr(&expr),
            "QuotedExpr::from_expr(&f(1_i64, 2_i64))"
        );
    }

    #[test]
    fn test_gen_reflect_function_type() {
        let gen = RustCodegen::new();
        // ?(Int32 -> Bool)
        let expr = Expr::Reflect(Box::new(TypeExpr::Function {
            params: vec![TypeExpr::Named("Int32".to_string())],
            return_type: Box::new(TypeExpr::Named("Bool".to_string())),
        }));
        assert_eq!(gen.gen_expr(&expr), "TypeInfo::get::<fn(i32) -> bool>()");
    }

    // === Pattern Matching Codegen Tests ===

    #[test]
    fn test_gen_pattern_wildcard() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Wildcard;
        assert_eq!(gen.gen_pattern(&pattern), "_");
    }

    #[test]
    fn test_gen_pattern_identifier() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Identifier("x".to_string());
        assert_eq!(gen.gen_pattern(&pattern), "x");
    }

    #[test]
    fn test_gen_pattern_literal_int() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Literal(Literal::Int(42));
        assert_eq!(gen.gen_pattern(&pattern), "42_i64");
    }

    #[test]
    fn test_gen_pattern_literal_string() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Literal(Literal::String("hello".to_string()));
        assert_eq!(gen.gen_pattern(&pattern), "\"hello\"");
    }

    #[test]
    fn test_gen_pattern_literal_bool() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Literal(Literal::Bool(true));
        assert_eq!(gen.gen_pattern(&pattern), "true");
    }

    #[test]
    fn test_gen_pattern_constructor_no_fields() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Constructor {
            name: "None".to_string(),
            fields: vec![],
        };
        assert_eq!(gen.gen_pattern(&pattern), "None");
    }

    #[test]
    fn test_gen_pattern_constructor_with_fields() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Constructor {
            name: "Some".to_string(),
            fields: vec![crate::ast::Pattern::Identifier("x".to_string())],
        };
        assert_eq!(gen.gen_pattern(&pattern), "Some(x)");
    }

    #[test]
    fn test_gen_pattern_constructor_multiple_fields() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Constructor {
            name: "Point".to_string(),
            fields: vec![
                crate::ast::Pattern::Identifier("x".to_string()),
                crate::ast::Pattern::Identifier("y".to_string()),
            ],
        };
        assert_eq!(gen.gen_pattern(&pattern), "Point(x, y)");
    }

    #[test]
    fn test_gen_pattern_tuple() {
        let gen = RustCodegen::new();
        let pattern = crate::ast::Pattern::Tuple(vec![
            crate::ast::Pattern::Identifier("x".to_string()),
            crate::ast::Pattern::Identifier("y".to_string()),
        ]);
        assert_eq!(gen.gen_pattern(&pattern), "(x, y)");
    }

    #[test]
    fn test_gen_pattern_nested_constructor() {
        let gen = RustCodegen::new();
        // Some(Point(x, y))
        let pattern = crate::ast::Pattern::Constructor {
            name: "Some".to_string(),
            fields: vec![crate::ast::Pattern::Constructor {
                name: "Point".to_string(),
                fields: vec![
                    crate::ast::Pattern::Identifier("x".to_string()),
                    crate::ast::Pattern::Identifier("y".to_string()),
                ],
            }],
        };
        assert_eq!(gen.gen_pattern(&pattern), "Some(Point(x, y))");
    }

    #[test]
    fn test_gen_match_simple() {
        let gen = RustCodegen::new();
        // match value { Some(x) => x, None => 0_i64 }
        let scrutinee = Expr::Identifier("value".to_string());
        let arms = vec![
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Constructor {
                    name: "Some".to_string(),
                    fields: vec![crate::ast::Pattern::Identifier("x".to_string())],
                },
                guard: None,
                body: Box::new(Expr::Identifier("x".to_string())),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Constructor {
                    name: "None".to_string(),
                    fields: vec![],
                },
                guard: None,
                body: Box::new(Expr::Literal(Literal::Int(0))),
            },
        ];
        let result = gen.gen_match(&scrutinee, &arms);
        assert!(result.contains("match value {"));
        assert!(result.contains("Some(x) => x"));
        assert!(result.contains("None => 0_i64"));
    }

    #[test]
    fn test_gen_match_with_guard() {
        let gen = RustCodegen::new();
        // match value { x if x > 0 => x, _ => 0 }
        let scrutinee = Expr::Identifier("value".to_string());
        let arms = vec![
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Identifier("x".to_string()),
                guard: Some(Box::new(Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    op: crate::ast::BinaryOp::Gt,
                    right: Box::new(Expr::Literal(Literal::Int(0))),
                })),
                body: Box::new(Expr::Identifier("x".to_string())),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::Literal(Literal::Int(0))),
            },
        ];
        let result = gen.gen_match(&scrutinee, &arms);
        assert!(result.contains("match value {"));
        assert!(result.contains("x if (x > 0_i64) => x"));
        assert!(result.contains("_ => 0_i64"));
    }

    #[test]
    fn test_gen_match_tuple_patterns() {
        let gen = RustCodegen::new();
        // match point { (0, 0) => "origin", (x, 0) => "x-axis", (0, y) => "y-axis", _ => "plane" }
        let scrutinee = Expr::Identifier("point".to_string());
        let arms = vec![
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Tuple(vec![
                    crate::ast::Pattern::Literal(Literal::Int(0)),
                    crate::ast::Pattern::Literal(Literal::Int(0)),
                ]),
                guard: None,
                body: Box::new(Expr::Literal(Literal::String("origin".to_string()))),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Tuple(vec![
                    crate::ast::Pattern::Identifier("x".to_string()),
                    crate::ast::Pattern::Literal(Literal::Int(0)),
                ]),
                guard: None,
                body: Box::new(Expr::Literal(Literal::String("x-axis".to_string()))),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Tuple(vec![
                    crate::ast::Pattern::Literal(Literal::Int(0)),
                    crate::ast::Pattern::Identifier("y".to_string()),
                ]),
                guard: None,
                body: Box::new(Expr::Literal(Literal::String("y-axis".to_string()))),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::Literal(Literal::String("plane".to_string()))),
            },
        ];
        let result = gen.gen_match(&scrutinee, &arms);
        assert!(result.contains("match point {"));
        assert!(result.contains("(0_i64, 0_i64) => \"origin\""));
        assert!(result.contains("(x, 0_i64) => \"x-axis\""));
        assert!(result.contains("(0_i64, y) => \"y-axis\""));
        assert!(result.contains("_ => \"plane\""));
    }

    #[test]
    fn test_gen_expr_match() {
        let gen = RustCodegen::new();
        // match option { Some(x) => x, None => 0 }
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Identifier("option".to_string())),
            arms: vec![
                crate::ast::MatchArm {
                    pattern: crate::ast::Pattern::Constructor {
                        name: "Some".to_string(),
                        fields: vec![crate::ast::Pattern::Identifier("x".to_string())],
                    },
                    guard: None,
                    body: Box::new(Expr::Identifier("x".to_string())),
                },
                crate::ast::MatchArm {
                    pattern: crate::ast::Pattern::Constructor {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                    guard: None,
                    body: Box::new(Expr::Literal(Literal::Int(0))),
                },
            ],
        };
        let result = gen.gen_expr(&expr);
        assert!(result.contains("match option {"));
        assert!(result.contains("Some(x) => x"));
        assert!(result.contains("None => 0_i64"));
    }

    #[test]
    fn test_gen_match_nested_patterns() {
        let gen = RustCodegen::new();
        // match result { Ok(Some(x)) => x, Ok(None) => 0, Err(_) => -1 }
        let scrutinee = Expr::Identifier("result".to_string());
        let arms = vec![
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Constructor {
                    name: "Ok".to_string(),
                    fields: vec![crate::ast::Pattern::Constructor {
                        name: "Some".to_string(),
                        fields: vec![crate::ast::Pattern::Identifier("x".to_string())],
                    }],
                },
                guard: None,
                body: Box::new(Expr::Identifier("x".to_string())),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Constructor {
                    name: "Ok".to_string(),
                    fields: vec![crate::ast::Pattern::Constructor {
                        name: "None".to_string(),
                        fields: vec![],
                    }],
                },
                guard: None,
                body: Box::new(Expr::Literal(Literal::Int(0))),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Constructor {
                    name: "Err".to_string(),
                    fields: vec![crate::ast::Pattern::Wildcard],
                },
                guard: None,
                body: Box::new(Expr::Literal(Literal::Int(-1))),
            },
        ];
        let result = gen.gen_match(&scrutinee, &arms);
        assert!(result.contains("match result {"));
        assert!(result.contains("Ok(Some(x)) => x"));
        assert!(result.contains("Ok(None) => 0_i64"));
        assert!(result.contains("Err(_) => -1_i64"));
    }

    #[test]
    fn test_gen_match_with_complex_body() {
        let gen = RustCodegen::new();
        // match value { Some(x) => x + 1, None => 0 }
        let scrutinee = Expr::Identifier("value".to_string());
        let arms = vec![
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Constructor {
                    name: "Some".to_string(),
                    fields: vec![crate::ast::Pattern::Identifier("x".to_string())],
                },
                guard: None,
                body: Box::new(Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    op: crate::ast::BinaryOp::Add,
                    right: Box::new(Expr::Literal(Literal::Int(1))),
                }),
            },
            crate::ast::MatchArm {
                pattern: crate::ast::Pattern::Constructor {
                    name: "None".to_string(),
                    fields: vec![],
                },
                guard: None,
                body: Box::new(Expr::Literal(Literal::Int(0))),
            },
        ];
        let result = gen.gen_match(&scrutinee, &arms);
        assert!(result.contains("match value {"));
        assert!(result.contains("Some(x) => (x + 1_i64)"));
        assert!(result.contains("None => 0_i64"));
    }
}
