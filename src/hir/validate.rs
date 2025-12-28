//! HIR validation module.
//!
//! This module provides semantic validation for HIR nodes:
//! - Scope validation: all identifiers resolve
//! - Type consistency: expressions have consistent types
//! - Declaration validation: no duplicate names
//! - Gene validation: check statement validity
//! - Function validation: check return types
//!
//! # Example
//!
//! ```ignore
//! use dol::hir::{HirModule, validate_module};
//!
//! let module = /* ... */;
//! match validate_module(&module, &symbols, &spans) {
//!     Ok(()) => println!("Module is valid"),
//!     Err(errors) => {
//!         for error in errors {
//!             eprintln!("{}", error);
//!         }
//!     }
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;

use super::span::{HirId, Span, SpanMap};
use super::symbol::{Symbol, SymbolTable};
use super::types::*;
use super::visit::{walk_decl, walk_module, HirVisitor};

/// Severity level for validation diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Error: must be fixed for valid HIR
    Error,
    /// Warning: may indicate a problem
    Warning,
    /// Note: additional information
    Note,
}

/// A validation error or warning.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Source span (if available)
    pub span: Option<Span>,
    /// Severity level
    pub level: DiagnosticLevel,
    /// Related HIR node ID (if available)
    pub node_id: Option<HirId>,
    /// Suggestion for fixing the error (if available)
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Create a new error.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            level: DiagnosticLevel::Error,
            node_id: None,
            suggestion: None,
        }
    }

    /// Create a new warning.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            level: DiagnosticLevel::Warning,
            node_id: None,
            suggestion: None,
        }
    }

    /// Add a span to this error.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Add a node ID to this error.
    pub fn with_node(mut self, id: HirId) -> Self {
        self.node_id = Some(id);
        self
    }

    /// Add a suggestion to this error.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Check if this is an error (not a warning).
    pub fn is_error(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Error)
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Note => "note",
        };

        write!(f, "{}: {}", prefix, self.message)?;

        if let Some(span) = &self.span {
            write!(f, " (at {}..{})", span.start, span.end)?;
        }

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n  suggestion: {}", suggestion)?;
        }

        Ok(())
    }
}

/// Information about a declared symbol.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// The symbol's name
    pub name: Symbol,
    /// The kind of symbol
    pub kind: SymbolKind,
    /// The HIR ID of the defining node
    pub defined_at: HirId,
    /// The type of the symbol (if known)
    pub ty: Option<HirType>,
}

/// Kind of symbol in the symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Type definition (gene, struct, enum)
    Type,
    /// Trait definition
    Trait,
    /// Function definition
    Function,
    /// Module definition
    Module,
    /// Variable binding
    Variable,
    /// Type parameter
    TypeParam,
    /// Field name
    Field,
    /// Variant name
    Variant,
}

/// A scope in the symbol table.
#[derive(Debug, Default)]
struct Scope {
    /// Symbols defined in this scope
    symbols: HashMap<Symbol, SymbolInfo>,
}

impl Scope {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, name: Symbol, info: SymbolInfo) -> Option<SymbolInfo> {
        self.symbols.insert(name, info)
    }

    fn get(&self, name: Symbol) -> Option<&SymbolInfo> {
        self.symbols.get(&name)
    }
}

/// Context for HIR validation.
///
/// Tracks symbols, types, and diagnostics during validation.
pub struct ValidationContext<'a> {
    /// Symbol table for resolving names to strings
    pub symbols: &'a SymbolTable,
    /// Span map for error reporting
    pub spans: &'a SpanMap,
    /// Scope stack for symbol resolution
    scopes: Vec<Scope>,
    /// Collected diagnostics
    diagnostics: Vec<ValidationError>,
    /// Set of symbols that have been referenced (for unused detection)
    referenced_symbols: HashSet<Symbol>,
    /// Current function return type (for return statement validation)
    current_return_type: Option<HirType>,
    /// Whether we're inside a loop (for break validation)
    in_loop: bool,
}

impl<'a> ValidationContext<'a> {
    /// Create a new validation context.
    pub fn new(symbols: &'a SymbolTable, spans: &'a SpanMap) -> Self {
        Self {
            symbols,
            spans,
            scopes: vec![Scope::new()], // Global scope
            diagnostics: Vec::new(),
            referenced_symbols: HashSet::new(),
            current_return_type: None,
            in_loop: false,
        }
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a symbol in the current scope.
    ///
    /// Returns an error if the symbol is already defined in this scope.
    pub fn define(&mut self, name: Symbol, info: SymbolInfo) -> Result<(), ValidationError> {
        let scope = self.scopes.last_mut().expect("no active scope");

        if let Some(existing) = scope.get(name) {
            let name_str = self.symbols.resolve(name).unwrap_or("<unknown>");
            let err = ValidationError::error(format!("duplicate definition of `{}`", name_str))
                .with_node(info.defined_at)
                .with_suggestion(format!(
                    "`{}` was previously defined at node {:?}",
                    name_str, existing.defined_at
                ));
            return Err(err);
        }

        scope.insert(name, info);
        Ok(())
    }

    /// Look up a symbol in all scopes.
    pub fn lookup(&self, name: Symbol) -> Option<&SymbolInfo> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Mark a symbol as referenced.
    pub fn reference(&mut self, name: Symbol) {
        self.referenced_symbols.insert(name);
    }

    /// Add an error diagnostic.
    pub fn error(&mut self, error: ValidationError) {
        self.diagnostics.push(error);
    }

    /// Add a warning diagnostic.
    pub fn warning(&mut self, warning: ValidationError) {
        self.diagnostics.push(warning);
    }

    /// Get the span for an HIR node.
    pub fn span_of(&self, id: HirId) -> Span {
        self.spans.get_or_dummy(id)
    }

    /// Resolve a symbol name to a string.
    pub fn resolve_name(&self, name: Symbol) -> &str {
        self.symbols.resolve(name).unwrap_or("<unknown>")
    }

    /// Get all collected diagnostics.
    pub fn diagnostics(&self) -> &[ValidationError] {
        &self.diagnostics
    }

    /// Take ownership of all diagnostics.
    pub fn into_diagnostics(self) -> Vec<ValidationError> {
        self.diagnostics
    }

    /// Check if there are any errors (not just warnings).
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }
}

/// Validate an HIR module.
///
/// Returns `Ok(())` if the module is valid, or a vector of errors otherwise.
///
/// # Example
///
/// ```ignore
/// use dol::hir::{HirModule, validate_module};
///
/// let module = /* ... */;
/// if let Err(errors) = validate_module(&module, &symbols, &spans) {
///     for error in errors {
///         eprintln!("{}", error);
///     }
/// }
/// ```
pub fn validate_module(
    module: &HirModule,
    symbols: &SymbolTable,
    spans: &SpanMap,
) -> Result<(), Vec<ValidationError>> {
    let mut ctx = ValidationContext::new(symbols, spans);

    // Pass 1: Collect all top-level declarations
    let mut collector = DeclarationCollector { ctx: &mut ctx };
    collector.collect_declarations(module);

    // Pass 2: Validate all declarations
    let mut validator = DeclarationValidator { ctx: &mut ctx };
    validator.visit_module(module);

    // Collect errors
    if ctx.has_errors() {
        let errors = ctx
            .into_diagnostics()
            .into_iter()
            .filter(|d| d.is_error())
            .collect();
        Err(errors)
    } else {
        Ok(())
    }
}

/// First pass: collect all top-level declarations.
struct DeclarationCollector<'a, 'b> {
    ctx: &'a mut ValidationContext<'b>,
}

impl<'a, 'b> DeclarationCollector<'a, 'b> {
    fn collect_declarations(&mut self, module: &HirModule) {
        for decl in &module.decls {
            self.collect_decl(decl);
        }
    }

    fn collect_decl(&mut self, decl: &HirDecl) {
        match decl {
            HirDecl::Type(ty_decl) => {
                let info = SymbolInfo {
                    name: ty_decl.name,
                    kind: SymbolKind::Type,
                    defined_at: ty_decl.id,
                    ty: None,
                };
                if let Err(err) = self.ctx.define(ty_decl.name, info) {
                    self.ctx.error(err.with_span(self.ctx.span_of(ty_decl.id)));
                }
            }
            HirDecl::Trait(trait_decl) => {
                let info = SymbolInfo {
                    name: trait_decl.name,
                    kind: SymbolKind::Trait,
                    defined_at: trait_decl.id,
                    ty: None,
                };
                if let Err(err) = self.ctx.define(trait_decl.name, info) {
                    self.ctx
                        .error(err.with_span(self.ctx.span_of(trait_decl.id)));
                }
            }
            HirDecl::Function(func_decl) => {
                let info = SymbolInfo {
                    name: func_decl.name,
                    kind: SymbolKind::Function,
                    defined_at: func_decl.id,
                    ty: Some(HirType::Function(Box::new(HirFunctionType {
                        params: func_decl.params.iter().map(|p| p.ty.clone()).collect(),
                        ret: func_decl.return_type.clone(),
                    }))),
                };
                if let Err(err) = self.ctx.define(func_decl.name, info) {
                    self.ctx
                        .error(err.with_span(self.ctx.span_of(func_decl.id)));
                }
            }
            HirDecl::Module(mod_decl) => {
                let info = SymbolInfo {
                    name: mod_decl.name,
                    kind: SymbolKind::Module,
                    defined_at: mod_decl.id,
                    ty: None,
                };
                if let Err(err) = self.ctx.define(mod_decl.name, info) {
                    self.ctx.error(err.with_span(self.ctx.span_of(mod_decl.id)));
                }
                // Recursively collect nested module declarations
                self.ctx.push_scope();
                for nested in &mod_decl.decls {
                    self.collect_decl(nested);
                }
                self.ctx.pop_scope();
            }
        }
    }
}

/// Second pass: validate all declarations.
struct DeclarationValidator<'a, 'b> {
    ctx: &'a mut ValidationContext<'b>,
}

impl<'a, 'b> HirVisitor for DeclarationValidator<'a, 'b> {
    fn visit_module(&mut self, module: &HirModule) {
        walk_module(self, module);
    }

    fn visit_decl(&mut self, decl: &HirDecl) {
        walk_decl(self, decl);
    }

    fn visit_type_decl(&mut self, decl: &HirTypeDecl) {
        // Validate type parameters
        self.ctx.push_scope();
        for param in &decl.type_params {
            let info = SymbolInfo {
                name: param.name,
                kind: SymbolKind::TypeParam,
                defined_at: decl.id,
                ty: None,
            };
            if let Err(err) = self.ctx.define(param.name, info) {
                self.ctx.error(err);
            }

            // Validate bounds
            for bound in &param.bounds {
                self.validate_type(bound, decl.id);
            }
        }

        // Validate body
        self.validate_type_def(&decl.body, decl.id);
        self.ctx.pop_scope();
    }

    fn visit_trait_decl(&mut self, decl: &HirTraitDecl) {
        self.ctx.push_scope();

        // Validate type parameters
        for param in &decl.type_params {
            let info = SymbolInfo {
                name: param.name,
                kind: SymbolKind::TypeParam,
                defined_at: decl.id,
                ty: None,
            };
            if let Err(err) = self.ctx.define(param.name, info) {
                self.ctx.error(err);
            }

            for bound in &param.bounds {
                self.validate_type(bound, decl.id);
            }
        }

        // Validate super traits
        for bound in &decl.bounds {
            self.validate_type(bound, decl.id);
        }

        // Validate trait items
        for item in &decl.items {
            match item {
                HirTraitItem::Method(func) => {
                    self.visit_function_decl(func);
                }
                HirTraitItem::AssocType(assoc) => {
                    for bound in &assoc.bounds {
                        self.validate_type(bound, decl.id);
                    }
                    if let Some(default) = &assoc.default {
                        self.validate_type(default, decl.id);
                    }
                }
            }
        }

        self.ctx.pop_scope();
    }

    fn visit_function_decl(&mut self, decl: &HirFunctionDecl) {
        self.ctx.push_scope();

        // Validate type parameters
        for param in &decl.type_params {
            let info = SymbolInfo {
                name: param.name,
                kind: SymbolKind::TypeParam,
                defined_at: decl.id,
                ty: None,
            };
            if let Err(err) = self.ctx.define(param.name, info) {
                self.ctx.error(err);
            }

            for bound in &param.bounds {
                self.validate_type(bound, decl.id);
            }
        }

        // Validate parameters
        for param in &decl.params {
            self.validate_pattern(&param.pat, Some(&param.ty), decl.id);
            self.validate_type(&param.ty, decl.id);
        }

        // Validate return type
        self.validate_type(&decl.return_type, decl.id);

        // Set current return type for body validation
        let old_return_type = self.ctx.current_return_type.take();
        self.ctx.current_return_type = Some(decl.return_type.clone());

        // Validate body
        if let Some(body) = &decl.body {
            self.visit_expr(body);
        }

        self.ctx.current_return_type = old_return_type;
        self.ctx.pop_scope();
    }

    fn visit_module_decl(&mut self, decl: &HirModuleDecl) {
        self.ctx.push_scope();
        for d in &decl.decls {
            self.visit_decl(d);
        }
        self.ctx.pop_scope();
    }

    fn visit_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Var(name) => {
                // Check that the variable is defined
                if self.ctx.lookup(*name).is_none() {
                    let name_str = self.ctx.resolve_name(*name);
                    self.ctx.error(
                        ValidationError::error(format!("undefined variable `{}`", name_str))
                            .with_suggestion("make sure the variable is defined before use"),
                    );
                } else {
                    self.ctx.reference(*name);
                }
            }
            HirExpr::Binary(bin) => {
                self.visit_expr(&bin.left);
                self.visit_expr(&bin.right);
            }
            HirExpr::Unary(un) => {
                self.visit_expr(&un.operand);
            }
            HirExpr::Call(call) => {
                self.visit_expr(&call.func);
                for arg in &call.args {
                    self.visit_expr(arg);
                }
            }
            HirExpr::MethodCall(call) => {
                self.visit_expr(&call.receiver);
                for arg in &call.args {
                    self.visit_expr(arg);
                }
            }
            HirExpr::Field(field) => {
                self.visit_expr(&field.base);
            }
            HirExpr::Index(idx) => {
                self.visit_expr(&idx.base);
                self.visit_expr(&idx.index);
            }
            HirExpr::Block(block) => {
                self.ctx.push_scope();
                for stmt in &block.stmts {
                    self.visit_stmt(stmt);
                }
                if let Some(expr) = &block.expr {
                    self.visit_expr(expr);
                }
                self.ctx.pop_scope();
            }
            HirExpr::If(if_expr) => {
                self.visit_expr(&if_expr.cond);
                self.visit_expr(&if_expr.then_branch);
                if let Some(else_branch) = &if_expr.else_branch {
                    self.visit_expr(else_branch);
                }
            }
            HirExpr::Match(match_expr) => {
                self.visit_expr(&match_expr.scrutinee);
                for arm in &match_expr.arms {
                    self.ctx.push_scope();
                    self.validate_pattern(&arm.pat, None, HirId::new());
                    if let Some(guard) = &arm.guard {
                        self.visit_expr(guard);
                    }
                    self.visit_expr(&arm.body);
                    self.ctx.pop_scope();
                }
            }
            HirExpr::Lambda(lambda) => {
                self.ctx.push_scope();
                for param in &lambda.params {
                    self.validate_pattern(&param.pat, Some(&param.ty), HirId::new());
                    self.validate_type(&param.ty, HirId::new());
                }
                if let Some(ret) = &lambda.return_type {
                    self.validate_type(ret, HirId::new());
                }

                // Set up context for lambda body
                let old_return_type = self.ctx.current_return_type.take();
                self.ctx.current_return_type = lambda.return_type.clone();

                self.visit_expr(&lambda.body);

                self.ctx.current_return_type = old_return_type;
                self.ctx.pop_scope();
            }
            HirExpr::Literal(_) => {
                // Literals are always valid
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Val(val) => {
                self.visit_expr(&val.init);
                if let Some(ty) = &val.ty {
                    self.validate_type(ty, HirId::new());
                }
                self.validate_pattern(&val.pat, val.ty.as_ref(), HirId::new());
            }
            HirStmt::Var(var) => {
                self.visit_expr(&var.init);
                if let Some(ty) = &var.ty {
                    self.validate_type(ty, HirId::new());
                }
                self.validate_pattern(&var.pat, var.ty.as_ref(), HirId::new());
            }
            HirStmt::Assign(assign) => {
                // Validate that LHS is a place expression
                if !is_place_expr(&assign.lhs) {
                    self.ctx.error(
                        ValidationError::error("invalid assignment target")
                            .with_suggestion("left-hand side must be a variable, field, or index"),
                    );
                }
                self.visit_expr(&assign.lhs);
                self.visit_expr(&assign.rhs);
            }
            HirStmt::Expr(expr) => {
                self.visit_expr(expr);
            }
            HirStmt::Return(ret) => {
                if let Some(expr) = ret {
                    self.visit_expr(expr);
                }
                // Note: Full return type checking requires type inference
            }
            HirStmt::Break(brk) => {
                if !self.ctx.in_loop {
                    self.ctx.error(
                        ValidationError::error("`break` outside of loop")
                            .with_suggestion("break can only be used inside a loop"),
                    );
                }
                if let Some(expr) = brk {
                    self.visit_expr(expr);
                }
            }
        }
    }
}

impl<'a, 'b> DeclarationValidator<'a, 'b> {
    /// Validate a type reference.
    fn validate_type(&mut self, ty: &HirType, _context_id: HirId) {
        match ty {
            HirType::Named(named) => {
                // Check that the type name is defined
                if let Some(info) = self.ctx.lookup(named.name) {
                    if !matches!(info.kind, SymbolKind::Type | SymbolKind::TypeParam) {
                        let name_str = self.ctx.resolve_name(named.name);
                        self.ctx.error(ValidationError::error(format!(
                            "`{}` is not a type",
                            name_str
                        )));
                    }
                    self.ctx.reference(named.name);
                } else {
                    let name_str = self.ctx.resolve_name(named.name);
                    // Don't error on built-in types
                    if !is_builtin_type(name_str) {
                        self.ctx.error(
                            ValidationError::error(format!("undefined type `{}`", name_str))
                                .with_suggestion("make sure the type is defined or imported"),
                        );
                    }
                }

                // Validate type arguments
                for arg in &named.args {
                    self.validate_type(arg, _context_id);
                }
            }
            HirType::Tuple(types) => {
                for t in types {
                    self.validate_type(t, _context_id);
                }
            }
            HirType::Array(arr) => {
                self.validate_type(&arr.elem, _context_id);
            }
            HirType::Function(func) => {
                for p in &func.params {
                    self.validate_type(p, _context_id);
                }
                self.validate_type(&func.ret, _context_id);
            }
            HirType::Ref(r) => {
                self.validate_type(&r.ty, _context_id);
            }
            HirType::Optional(inner) => {
                self.validate_type(inner, _context_id);
            }
            HirType::Var(_) | HirType::Error => {
                // Type variables and error types don't need validation
            }
        }
    }

    /// Validate a type definition body.
    fn validate_type_def(&mut self, def: &HirTypeDef, context_id: HirId) {
        match def {
            HirTypeDef::Alias(ty) => {
                self.validate_type(ty, context_id);
            }
            HirTypeDef::Struct(fields) => {
                let mut seen_fields = HashSet::new();
                for field in fields {
                    if !seen_fields.insert(field.name) {
                        let name_str = self.ctx.resolve_name(field.name);
                        self.ctx.error(ValidationError::error(format!(
                            "duplicate field `{}`",
                            name_str
                        )));
                    }
                    self.validate_type(&field.ty, context_id);
                }
            }
            HirTypeDef::Enum(variants) => {
                let mut seen_variants = HashSet::new();
                for variant in variants {
                    if !seen_variants.insert(variant.name) {
                        let name_str = self.ctx.resolve_name(variant.name);
                        self.ctx.error(ValidationError::error(format!(
                            "duplicate variant `{}`",
                            name_str
                        )));
                    }
                    if let Some(payload) = &variant.payload {
                        self.validate_type(payload, context_id);
                    }
                }
            }
            HirTypeDef::Gene(statements) => {
                self.validate_gene_statements(statements, context_id);
            }
        }
    }

    /// Validate gene statements.
    fn validate_gene_statements(&mut self, statements: &[HirStatement], _context_id: HirId) {
        for stmt in statements {
            self.validate_gene_statement(stmt);
        }
    }

    /// Validate a single gene statement.
    fn validate_gene_statement(&mut self, stmt: &HirStatement) {
        match &stmt.kind {
            HirStatementKind::Has { subject, property } => {
                // Validate that subject and property are valid identifiers
                let subj_str = self.ctx.resolve_name(*subject).to_string();
                let prop_str = self.ctx.resolve_name(*property).to_string();

                if subj_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty subject in 'has' statement")
                            .with_node(stmt.id),
                    );
                }
                if prop_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty property in 'has' statement")
                            .with_node(stmt.id),
                    );
                }
            }
            HirStatementKind::Is { subject, type_name } => {
                let subj_str = self.ctx.resolve_name(*subject).to_string();
                let type_str = self.ctx.resolve_name(*type_name).to_string();

                if subj_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty subject in 'is' statement")
                            .with_node(stmt.id),
                    );
                }
                if type_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty type name in 'is' statement")
                            .with_node(stmt.id),
                    );
                }
            }
            HirStatementKind::DerivesFrom { subject, parent } => {
                let subj_str = self.ctx.resolve_name(*subject).to_string();
                let parent_str = self.ctx.resolve_name(*parent).to_string();

                if subj_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty subject in 'derives_from' statement")
                            .with_node(stmt.id),
                    );
                }
                if parent_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty parent in 'derives_from' statement")
                            .with_node(stmt.id),
                    );
                }

                // Warning: self-derivation
                if subject == parent {
                    self.ctx.warning(
                        ValidationError::warning(format!(
                            "type `{}` derives from itself",
                            subj_str
                        ))
                        .with_node(stmt.id)
                        .with_suggestion("a type cannot derive from itself"),
                    );
                }
            }
            HirStatementKind::Requires {
                subject,
                dependency,
            } => {
                let subj_str = self.ctx.resolve_name(*subject).to_string();
                let dep_str = self.ctx.resolve_name(*dependency).to_string();

                if subj_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty subject in 'requires' statement")
                            .with_node(stmt.id),
                    );
                }
                if dep_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty dependency in 'requires' statement")
                            .with_node(stmt.id),
                    );
                }

                // Warning: self-requirement
                if subject == dependency {
                    self.ctx.warning(
                        ValidationError::warning(format!("`{}` requires itself", subj_str))
                            .with_node(stmt.id)
                            .with_suggestion("a gene cannot require itself"),
                    );
                }
            }
            HirStatementKind::Uses { subject, resource } => {
                let subj_str = self.ctx.resolve_name(*subject).to_string();
                let res_str = self.ctx.resolve_name(*resource).to_string();

                if subj_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty subject in 'uses' statement")
                            .with_node(stmt.id),
                    );
                }
                if res_str.is_empty() {
                    self.ctx.error(
                        ValidationError::error("empty resource in 'uses' statement")
                            .with_node(stmt.id),
                    );
                }
            }
        }
    }

    /// Validate a pattern and bind variables.
    fn validate_pattern(&mut self, pat: &HirPat, ty: Option<&HirType>, context_id: HirId) {
        match pat {
            HirPat::Wildcard => {
                // Wildcard is always valid
            }
            HirPat::Var(name) => {
                // Bind the variable in the current scope
                let info = SymbolInfo {
                    name: *name,
                    kind: SymbolKind::Variable,
                    defined_at: context_id,
                    ty: ty.cloned(),
                };
                if let Err(err) = self.ctx.define(*name, info) {
                    self.ctx.error(err);
                }
            }
            HirPat::Literal(_) => {
                // Literals are always valid
            }
            HirPat::Constructor(ctor) => {
                // Check that the constructor is defined
                if self.ctx.lookup(ctor.name).is_none() {
                    let name_str = self.ctx.resolve_name(ctor.name);
                    self.ctx.error(
                        ValidationError::error(format!("undefined constructor `{}`", name_str))
                            .with_suggestion("make sure the type variant is defined"),
                    );
                } else {
                    self.ctx.reference(ctor.name);
                }

                // Validate sub-patterns
                for field in &ctor.fields {
                    self.validate_pattern(field, None, context_id);
                }
            }
            HirPat::Tuple(pats) => {
                for p in pats {
                    self.validate_pattern(p, None, context_id);
                }
            }
            HirPat::Or(pats) => {
                // All alternatives must bind the same variables
                if pats.is_empty() {
                    self.ctx.error(ValidationError::error("empty or-pattern"));
                    return;
                }

                // Collect variables from first pattern
                let mut first_vars = HashSet::new();
                collect_pattern_vars(&pats[0], &mut first_vars);

                // Check that all alternatives have the same variables
                for (i, p) in pats.iter().enumerate().skip(1) {
                    let mut vars = HashSet::new();
                    collect_pattern_vars(p, &mut vars);

                    if vars != first_vars {
                        self.ctx.error(
                            ValidationError::error(format!(
                                "or-pattern alternative {} binds different variables",
                                i + 1
                            ))
                            .with_suggestion("all alternatives must bind the same variables"),
                        );
                    }
                }

                // Validate all patterns
                for p in pats {
                    // Use a temporary scope to collect variables
                    self.ctx.push_scope();
                    self.validate_pattern(p, ty, context_id);
                    self.ctx.pop_scope();
                }

                // Bind variables from the first pattern in the outer scope
                self.validate_pattern(&pats[0], ty, context_id);
            }
        }
    }
}

/// Check if an expression is a valid place expression (can be assigned to).
fn is_place_expr(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Var(_) => true,
        HirExpr::Field(field) => is_place_expr(&field.base),
        HirExpr::Index(idx) => is_place_expr(&idx.base),
        _ => false,
    }
}

/// Collect all variable names from a pattern.
fn collect_pattern_vars(pat: &HirPat, vars: &mut HashSet<Symbol>) {
    match pat {
        HirPat::Wildcard | HirPat::Literal(_) => {}
        HirPat::Var(name) => {
            vars.insert(*name);
        }
        HirPat::Constructor(ctor) => {
            for field in &ctor.fields {
                collect_pattern_vars(field, vars);
            }
        }
        HirPat::Tuple(pats) => {
            for p in pats {
                collect_pattern_vars(p, vars);
            }
        }
        HirPat::Or(pats) => {
            // Use variables from first alternative
            if let Some(first) = pats.first() {
                collect_pattern_vars(first, vars);
            }
        }
    }
}

/// Check if a type name is a built-in type.
fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "Bool"
            | "Int"
            | "Float"
            | "String"
            | "Unit"
            | "Never"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "str"
            | "Self"
            | "self"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_module(symbols: &mut SymbolTable) -> HirModule {
        let name = symbols.intern("test_module");
        HirModule::new(name)
    }

    #[test]
    fn test_valid_empty_module() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();
        let module = create_test_module(&mut symbols);

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_function_declaration() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let func_name = symbols.intern("my_func");
        let int_type = symbols.intern("Int");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Function(HirFunctionDecl {
            id: HirId::new(),
            name: func_name,
            type_params: vec![],
            params: vec![],
            return_type: HirType::Named(HirNamedType {
                name: int_type,
                args: vec![],
            }),
            body: Some(HirExpr::Literal(HirLiteral::Int(42))),
        }));

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_function_names() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let func_name = symbols.intern("duplicate_func");
        let int_type = symbols.intern("Int");

        let mut module = HirModule::new(module_name);

        // Add two functions with the same name
        for _ in 0..2 {
            module.decls.push(HirDecl::Function(HirFunctionDecl {
                id: HirId::new(),
                name: func_name,
                type_params: vec![],
                params: vec![],
                return_type: HirType::Named(HirNamedType {
                    name: int_type,
                    args: vec![],
                }),
                body: None,
            }));
        }

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("duplicate")));
    }

    #[test]
    fn test_undefined_variable() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let func_name = symbols.intern("my_func");
        let undefined_var = symbols.intern("undefined_var");
        let int_type = symbols.intern("Int");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Function(HirFunctionDecl {
            id: HirId::new(),
            name: func_name,
            type_params: vec![],
            params: vec![],
            return_type: HirType::Named(HirNamedType {
                name: int_type,
                args: vec![],
            }),
            body: Some(HirExpr::Var(undefined_var)),
        }));

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("undefined variable")));
    }

    #[test]
    fn test_valid_type_declaration() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let type_name = symbols.intern("MyStruct");
        let field_name = symbols.intern("field");
        let int_type = symbols.intern("Int");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Type(HirTypeDecl {
            id: HirId::new(),
            name: type_name,
            type_params: vec![],
            body: HirTypeDef::Struct(vec![HirField {
                name: field_name,
                ty: HirType::Named(HirNamedType {
                    name: int_type,
                    args: vec![],
                }),
            }]),
        }));

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_struct_fields() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let type_name = symbols.intern("MyStruct");
        let field_name = symbols.intern("duplicate_field");
        let int_type = symbols.intern("Int");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Type(HirTypeDecl {
            id: HirId::new(),
            name: type_name,
            type_params: vec![],
            body: HirTypeDef::Struct(vec![
                HirField {
                    name: field_name,
                    ty: HirType::Named(HirNamedType {
                        name: int_type,
                        args: vec![],
                    }),
                },
                HirField {
                    name: field_name,
                    ty: HirType::Named(HirNamedType {
                        name: int_type,
                        args: vec![],
                    }),
                },
            ]),
        }));

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("duplicate field")));
    }

    #[test]
    fn test_gene_validation() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let gene_name = symbols.intern("MyGene");
        let subject = symbols.intern("container");
        let property = symbols.intern("storage");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Type(HirTypeDecl {
            id: HirId::new(),
            name: gene_name,
            type_params: vec![],
            body: HirTypeDef::Gene(vec![HirStatement {
                id: HirId::new(),
                kind: HirStatementKind::Has { subject, property },
            }]),
        }));

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_ok());
    }

    #[test]
    fn test_self_derivation_warning() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let gene_name = symbols.intern("MyGene");
        let subject = symbols.intern("container");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Type(HirTypeDecl {
            id: HirId::new(),
            name: gene_name,
            type_params: vec![],
            body: HirTypeDef::Gene(vec![HirStatement {
                id: HirId::new(),
                kind: HirStatementKind::DerivesFrom {
                    subject,
                    parent: subject, // Self-derivation
                },
            }]),
        }));

        // This should not error (just warn), so validation passes
        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_type_reference() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let func_name = symbols.intern("my_func");
        let undefined_type = symbols.intern("UndefinedType");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Function(HirFunctionDecl {
            id: HirId::new(),
            name: func_name,
            type_params: vec![],
            params: vec![],
            return_type: HirType::Named(HirNamedType {
                name: undefined_type,
                args: vec![],
            }),
            body: None,
        }));

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("undefined type")));
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::error("test error")
            .with_span(Span::new(10, 20))
            .with_suggestion("try this instead");

        let display = format!("{}", err);
        assert!(display.contains("error"));
        assert!(display.contains("test error"));
        assert!(display.contains("10..20"));
        assert!(display.contains("try this instead"));
    }

    #[test]
    fn test_validation_context_scope() {
        let symbols = SymbolTable::new();
        let spans = SpanMap::new();
        let mut ctx = ValidationContext::new(&symbols, &spans);

        // Initially one scope (global)
        assert!(ctx.lookup(Symbol::new(999)).is_none());

        // Push and pop scopes
        ctx.push_scope();
        ctx.push_scope();
        ctx.pop_scope();
        ctx.pop_scope();

        // Should still work
        assert!(ctx.lookup(Symbol::new(999)).is_none());
    }

    #[test]
    fn test_builtin_types() {
        assert!(is_builtin_type("Int"));
        assert!(is_builtin_type("Bool"));
        assert!(is_builtin_type("String"));
        assert!(is_builtin_type("i32"));
        assert!(is_builtin_type("f64"));
        assert!(!is_builtin_type("CustomType"));
        assert!(!is_builtin_type("MyStruct"));
    }

    #[test]
    fn test_invalid_assignment_target() {
        let mut symbols = SymbolTable::new();
        let spans = SpanMap::new();

        let module_name = symbols.intern("test_module");
        let func_name = symbols.intern("my_func");
        let unit_type = symbols.intern("Unit");

        let mut module = HirModule::new(module_name);
        module.decls.push(HirDecl::Function(HirFunctionDecl {
            id: HirId::new(),
            name: func_name,
            type_params: vec![],
            params: vec![],
            return_type: HirType::Named(HirNamedType {
                name: unit_type,
                args: vec![],
            }),
            body: Some(HirExpr::Block(Box::new(HirBlockExpr {
                stmts: vec![HirStmt::Assign(HirAssignStmt {
                    // Invalid: assigning to a literal
                    lhs: HirExpr::Literal(HirLiteral::Int(1)),
                    rhs: HirExpr::Literal(HirLiteral::Int(2)),
                })],
                expr: None,
            }))),
        }));

        let result = validate_module(&module, &symbols, &spans);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("invalid assignment")));
    }
}
