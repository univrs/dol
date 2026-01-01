//! # WASM Compiler
//!
//! Compiles Metal DOL modules to WebAssembly bytecode.
//!
//! This implementation uses direct WASM emission via the `wasm-encoder` crate:
//!
//! ```text
//! DOL AST → WASM bytecode
//! ```
//!
//! For the full MLIR-based pipeline (DOL → MLIR → LLVM → WASM),
//! enable the `wasm-mlir` feature (requires LLVM 18 installed).
//!
//! ## Example
//!
//! ```rust,ignore
//! use metadol::wasm::WasmCompiler;
//! use metadol::parse_file;
//!
//! let source = r#"
//! gene container.exists {
//!   container has identity
//! }
//!
//! exegesis {
//!   A container exists.
//! }
//! "#;
//!
//! let module = parse_file(source)?;
//! let compiler = WasmCompiler::new()
//!     .with_optimization(true);
//!
//! let wasm_bytes = compiler.compile(&module)?;
//! ```

#[cfg(feature = "wasm")]
use crate::ast::Declaration;
#[cfg(feature = "wasm")]
use crate::wasm::alloc::BumpAllocator;
#[cfg(feature = "wasm")]
use crate::wasm::layout::{GeneLayout, GeneLayoutRegistry};
#[cfg(feature = "wasm")]
use crate::wasm::WasmError;
#[cfg(feature = "wasm")]
use std::collections::HashMap;
#[cfg(feature = "wasm")]
use std::path::Path;
#[cfg(feature = "wasm")]
use wasm_encoder;

/// WASM compiler for Metal DOL modules.
///
/// The `WasmCompiler` transforms DOL declarations into WebAssembly bytecode.
/// It provides control over optimization levels and debug information.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::WasmCompiler;
///
/// let compiler = WasmCompiler::new()
///     .with_optimization(true)
///     .with_debug_info(false);
/// ```
#[cfg(feature = "wasm")]
#[derive(Debug, Clone)]
pub struct WasmCompiler {
    /// Enable LLVM optimizations
    optimize: bool,
    /// Include debug information in WASM
    debug_info: bool,
    /// Registry of gene layouts for struct literal compilation
    gene_layouts: GeneLayoutRegistry,
}

/// Table for tracking local variables within a function.
///
/// This tracks both function parameters (which are the first locals in WASM)
/// and declared local variables from let bindings.
#[cfg(feature = "wasm")]
#[derive(Debug, Default)]
struct LocalsTable {
    /// Number of function parameters
    param_count: u32,
    /// Maps variable names to their local index
    name_to_index: HashMap<String, u32>,
    /// Types of declared locals (not including parameters)
    local_types: Vec<wasm_encoder::ValType>,
    /// Maps variable names to their DOL type (gene name) for member access
    dol_types: HashMap<String, String>,
    /// Maps function names to their WASM function indices
    func_indices: HashMap<String, u32>,
}

#[cfg(feature = "wasm")]
impl LocalsTable {
    /// Create a new LocalsTable from function parameters.
    ///
    /// Parameters are assigned indices 0..n-1 and registered in the name map.
    #[allow(dead_code)]
    fn new(params: &[crate::ast::FunctionParam]) -> Self {
        Self::new_with_gene_context(params, None)
    }

    /// Create a new LocalsTable with optional gene context.
    ///
    /// If gene_context is provided, adds an implicit 'self' parameter at index 0,
    /// and tracks field names for implicit field access resolution.
    fn new_with_gene_context(
        params: &[crate::ast::FunctionParam],
        gene_context: Option<&GeneContext>,
    ) -> Self {
        let has_self = gene_context.is_some();
        let self_offset = if has_self { 1u32 } else { 0u32 };

        let mut table = Self {
            param_count: params.len() as u32 + self_offset,
            name_to_index: HashMap::new(),
            local_types: Vec::new(),
            dol_types: HashMap::new(),
            func_indices: HashMap::new(),
        };

        // Add implicit 'self' parameter at index 0 for gene methods
        if has_self {
            table.name_to_index.insert("self".to_string(), 0);
        }

        // Add declared parameters (offset by 1 if we have self)
        for (i, param) in params.iter().enumerate() {
            table
                .name_to_index
                .insert(param.name.clone(), i as u32 + self_offset);
        }

        // Track gene field names for implicit field access
        if let Some(ctx) = gene_context {
            for field_name in &ctx.field_names {
                // Store field names with a special prefix to identify them
                table.dol_types.insert(
                    format!("__gene_field_{}", field_name),
                    ctx.gene_name.clone(),
                );
            }
        }

        table
    }

    /// Check if an identifier is a gene field (for implicit self access).
    fn is_gene_field(&self, name: &str) -> bool {
        self.dol_types
            .contains_key(&format!("__gene_field_{}", name))
    }

    /// Get the gene name for field access.
    fn get_gene_name_for_field(&self, name: &str) -> Option<&str> {
        self.dol_types
            .get(&format!("__gene_field_{}", name))
            .map(|s| s.as_str())
    }

    /// Register a function's WASM index.
    fn register_function(&mut self, name: &str, index: u32) {
        self.func_indices.insert(name.to_string(), index);
    }

    /// Look up a function's WASM index by name.
    fn lookup_function(&self, name: &str) -> Option<u32> {
        self.func_indices.get(name).copied()
    }

    /// Declare a new local variable.
    ///
    /// Returns the index assigned to this local. The index is param_count + local_index.
    fn declare(&mut self, name: &str, val_type: wasm_encoder::ValType) -> u32 {
        let index = self.param_count + self.local_types.len() as u32;
        self.name_to_index.insert(name.to_string(), index);
        self.local_types.push(val_type);
        index
    }

    /// Declare a new local variable with DOL type information.
    ///
    /// Returns the index assigned to this local.
    #[allow(dead_code)]
    fn declare_with_type(
        &mut self,
        name: &str,
        val_type: wasm_encoder::ValType,
        dol_type: &str,
    ) -> u32 {
        let index = self.declare(name, val_type);
        self.dol_types
            .insert(name.to_string(), dol_type.to_string());
        index
    }

    /// Set the DOL type for an existing variable.
    fn set_dol_type(&mut self, name: &str, dol_type: &str) {
        self.dol_types
            .insert(name.to_string(), dol_type.to_string());
    }

    /// Look up a variable by name, returning its local index if found.
    fn lookup(&self, name: &str) -> Option<u32> {
        self.name_to_index.get(name).copied()
    }

    /// Look up the DOL type of a variable by name.
    fn lookup_dol_type(&self, name: &str) -> Option<&str> {
        self.dol_types.get(name).map(|s| s.as_str())
    }

    /// Get the locals declaration for the WASM Function constructor.
    ///
    /// Returns a vector of (count, type) pairs, with consecutive same-type
    /// locals grouped together for efficiency.
    fn get_locals(&self) -> Vec<(u32, wasm_encoder::ValType)> {
        let mut result = Vec::new();
        for ty in &self.local_types {
            if let Some((count, last_ty)) = result.last_mut() {
                if last_ty == ty {
                    *count += 1;
                    continue;
                }
            }
            result.push((1, *ty));
        }
        result
    }
}

/// A function extracted from a DOL module with its export name.
#[cfg(feature = "wasm")]
struct ExtractedFunction<'a> {
    /// The function declaration
    func: &'a crate::ast::FunctionDecl,
    /// The name to export as (may include gene prefix)
    exported_name: String,
    /// Gene context for methods (provides field information for implicit self)
    gene_context: Option<GeneContext>,
}

/// Context for a gene method, providing field information for implicit self access.
#[cfg(feature = "wasm")]
#[derive(Debug, Clone)]
struct GeneContext {
    /// The gene name
    gene_name: String,
    /// Field names defined in this gene (for identifier resolution)
    field_names: Vec<String>,
}

#[cfg(feature = "wasm")]
impl WasmCompiler {
    /// Create a new WASM compiler with default settings.
    ///
    /// Default settings:
    /// - Optimization: disabled
    /// - Debug info: enabled
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let compiler = WasmCompiler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            optimize: false,
            debug_info: true,
            gene_layouts: GeneLayoutRegistry::new(),
        }
    }

    /// Register a gene layout for struct literal compilation.
    ///
    /// When gene layouts are registered, the compiler will add memory
    /// and allocation support to the generated WASM module, enabling
    /// struct literal expressions to be compiled.
    ///
    /// # Arguments
    ///
    /// * `layout` - The gene layout to register
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    /// use metadol::wasm::layout::{GeneLayout, FieldLayout, WasmFieldType};
    ///
    /// let mut compiler = WasmCompiler::new();
    ///
    /// let point_layout = GeneLayout {
    ///     name: "Point".to_string(),
    ///     fields: vec![
    ///         FieldLayout::primitive("x", 0, WasmFieldType::F64),
    ///         FieldLayout::primitive("y", 8, WasmFieldType::F64),
    ///     ],
    ///     total_size: 16,
    ///     alignment: 8,
    /// };
    ///
    /// compiler.register_gene_layout(point_layout);
    /// ```
    pub fn register_gene_layout(&mut self, layout: GeneLayout) {
        self.gene_layouts.register(layout);
    }

    /// Check if any gene layouts have been registered.
    pub fn has_gene_layouts(&self) -> bool {
        !self.gene_layouts.is_empty()
    }

    /// Auto-register gene layouts from a module declaration.
    ///
    /// This scans the module for gene declarations with fields and
    /// automatically computes and registers their layouts.
    fn auto_register_gene_layouts(&mut self, module: &Declaration) {
        use crate::wasm::layout::compute_gene_layout;

        if let Declaration::Gene(gene) = module {
            // Check if this gene has any fields
            let has_fields = gene
                .statements
                .iter()
                .any(|stmt| matches!(stmt, crate::ast::Statement::HasField(_)));

            if has_fields {
                // Don't re-register if already registered
                if !self.gene_layouts.contains(&gene.name) {
                    // Compute the layout using the registry (for nested types)
                    if let Ok(layout) = compute_gene_layout(gene, &self.gene_layouts) {
                        self.gene_layouts.register(layout);
                    }
                }
            }
        }
    }

    /// Enable or disable optimizations.
    ///
    /// When enabled, LLVM will run optimization passes on the IR before
    /// generating WASM bytecode. This produces smaller and faster WASM
    /// modules but increases compilation time.
    ///
    /// # Arguments
    ///
    /// * `optimize` - `true` to enable optimizations, `false` to disable
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let compiler = WasmCompiler::new().with_optimization(true);
    /// ```
    pub fn with_optimization(mut self, optimize: bool) -> Self {
        self.optimize = optimize;
        self
    }

    /// Enable or disable debug information.
    ///
    /// When enabled, the WASM module will include source location information
    /// for debugging. This increases module size but improves debuggability.
    ///
    /// # Arguments
    ///
    /// * `debug_info` - `true` to include debug info, `false` to omit
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let compiler = WasmCompiler::new().with_debug_info(false);
    /// ```
    pub fn with_debug_info(mut self, debug_info: bool) -> Self {
        self.debug_info = debug_info;
        self
    }

    /// Compile a DOL module to WASM bytecode.
    ///
    /// Takes a DOL declaration AST and transforms it to WebAssembly bytecode.
    /// This implementation uses direct WASM emission via wasm-encoder.
    ///
    /// # Arguments
    ///
    /// * `module` - The DOL module to compile
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the WASM bytecode on success, or a `WasmError`
    /// if compilation fails.
    ///
    /// # Supported Constructs
    ///
    /// - Function declarations with basic types (i32, i64, f32, f64)
    /// - Integer and float literals
    /// - Binary operations (add, sub, mul, div, etc.)
    /// - Function calls
    /// - Return statements
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    /// use metadol::parse_file;
    ///
    /// let source = r#"
    /// fun add(a: i64, b: i64) -> i64 {
    ///   return a + b
    /// }
    ///
    /// exegesis {
    ///   Adds two integers.
    /// }
    /// "#;
    ///
    /// let module = parse_file(source)?;
    /// let compiler = WasmCompiler::new();
    /// let wasm_bytes = compiler.compile(&module)?;
    /// ```
    pub fn compile(&mut self, module: &Declaration) -> Result<Vec<u8>, WasmError> {
        use wasm_encoder::{
            CodeSection, ExportKind, ExportSection, Function, FunctionSection, Module, TypeSection,
            ValType,
        };

        // Auto-register gene layouts for genes with fields
        self.auto_register_gene_layouts(module);

        // Extract function declarations from the module
        let functions = self.extract_functions(module)?;

        if functions.is_empty() {
            return Err(WasmError::new(
                "No functions found in module - only function declarations are currently supported for WASM compilation",
            ));
        }

        // Check if we need memory allocation (when gene layouts are registered)
        let needs_memory = !self.gene_layouts.is_empty();

        // Build WASM module
        let mut wasm_module = Module::new();

        // Type section: function signatures
        let mut types = TypeSection::new();
        let mut type_indices = Vec::new();

        // If we need memory, add the alloc function type first
        if needs_memory {
            let (alloc_params, alloc_results) = BumpAllocator::alloc_type_signature();
            types.function(alloc_params, alloc_results);
            type_indices.push(0); // alloc function uses type index 0
        }

        // Add user function types (offset by 1 if we have alloc)
        let type_offset = if needs_memory { 1u32 } else { 0u32 };
        for extracted in &functions {
            let mut params: Vec<ValType> = Vec::new();

            // For gene methods, add implicit 'self' parameter (i32 pointer)
            if extracted.gene_context.is_some() {
                params.push(ValType::I32);
            }

            // Add declared parameters
            for p in &extracted.func.params {
                params.push(self.dol_type_to_wasm(&p.type_ann)?);
            }

            let results = if let Some(ref ret_type) = extracted.func.return_type {
                vec![self.dol_type_to_wasm(ret_type)?]
            } else {
                vec![]
            };

            types.function(params, results);
            let user_func_idx = type_indices.len() - if needs_memory { 1 } else { 0 };
            type_indices.push(type_offset + user_func_idx as u32);
        }

        wasm_module.section(&types);

        // Function section: function indices
        let mut funcs = FunctionSection::new();
        if needs_memory {
            funcs.function(0); // alloc function uses type 0
        }
        for (i, _func) in functions.iter().enumerate() {
            funcs.function(type_offset + i as u32);
        }
        wasm_module.section(&funcs);

        // Memory section (if needed)
        if needs_memory {
            BumpAllocator::emit_memory_section(&mut wasm_module, 1);
            BumpAllocator::emit_globals(&mut wasm_module, 1024);
        }

        // Export section: export all user functions (not alloc, which is internal)
        let mut exports = ExportSection::new();
        let func_idx_offset = if needs_memory { 1u32 } else { 0u32 };
        for (idx, extracted) in functions.iter().enumerate() {
            exports.export(
                &extracted.exported_name,
                ExportKind::Func,
                func_idx_offset + idx as u32,
            );
        }
        // Also export memory if we have it
        if needs_memory {
            exports.export("memory", ExportKind::Memory, 0);
        }
        wasm_module.section(&exports);

        // Code section: function bodies
        let mut code = CodeSection::new();

        // Add alloc function code if needed
        if needs_memory {
            let alloc_func = BumpAllocator::build_alloc_function();
            code.function(&alloc_func);
        }

        // Add user function code
        let alloc_func_idx = if needs_memory { Some(0u32) } else { None };
        for (idx, extracted) in functions.iter().enumerate() {
            // Create a LocalsTable for this function (with gene context for methods)
            let mut locals_table = LocalsTable::new_with_gene_context(
                &extracted.func.params,
                extracted.gene_context.as_ref(),
            );

            // Register all function indices for call resolution
            for (func_idx, other_extracted) in functions.iter().enumerate() {
                let wasm_idx = func_idx_offset + func_idx as u32;
                // Register both the exported name and the raw function name
                locals_table.register_function(&other_extracted.exported_name, wasm_idx);
                locals_table.register_function(&other_extracted.func.name, wasm_idx);
            }

            // Pre-scan statements to collect all let bindings
            self.collect_locals(&extracted.func.body, &mut locals_table)?;

            // If we have gene layouts, add a temp local for struct pointer
            if needs_memory {
                locals_table.declare("__struct_ptr", ValType::I32);
            }

            // Add __match_temp for match expressions (if needed)
            // This is declared unconditionally for simplicity; could be optimized
            // to only declare when match expressions are present
            locals_table.declare("__match_temp", ValType::I64);

            // Build the locals vector for the Function constructor
            let locals = locals_table.get_locals();
            let mut function = Function::new(locals);

            let _ = alloc_func_idx; // Will be used when struct allocation is implemented
            let _ = idx; // Function index available for debugging
            self.emit_function_body(&mut function, extracted.func, &locals_table)?;
            code.function(&function);
        }
        wasm_module.section(&code);

        Ok(wasm_module.finish())
    }

    /// Extract function declarations from a DOL module.
    ///
    /// Supports top-level function declarations and gene methods.
    fn extract_functions<'a>(
        &self,
        module: &'a Declaration,
    ) -> Result<Vec<ExtractedFunction<'a>>, WasmError> {
        use crate::ast::{Declaration, Statement};

        match module {
            Declaration::Function(func) => Ok(vec![ExtractedFunction {
                func: func.as_ref(),
                exported_name: func.name.clone(),
                gene_context: None,
            }]),
            Declaration::Gene(gene) => {
                // Collect field names from gene for implicit self access
                let field_names: Vec<String> = gene
                    .statements
                    .iter()
                    .filter_map(|stmt| {
                        if let Statement::HasField(field) = stmt {
                            Some(field.name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                // Only set gene_context if the gene has fields (requires implicit self)
                let gene_context = if field_names.is_empty() {
                    None
                } else {
                    Some(GeneContext {
                        gene_name: gene.name.clone(),
                        field_names,
                    })
                };

                // Extract methods from gene
                let mut funcs = Vec::new();
                for stmt in &gene.statements {
                    if let Statement::Function(func) = stmt {
                        funcs.push(ExtractedFunction {
                            func: func.as_ref(),
                            exported_name: format!("{}.{}", gene.name, func.name),
                            gene_context: gene_context.clone(),
                        });
                    }
                }
                Ok(funcs)
            }
            _ => {
                // For now, we only support direct function declarations and genes
                // Future: extract functions from Trait/System bodies
                Ok(vec![])
            }
        }
    }

    /// Convert a DOL type expression to a WASM value type.
    ///
    /// Maps DOL types to their WASM equivalents:
    /// - i32, i64 → i64
    /// - f32, f64 → f64
    /// - bool → i32
    fn dol_type_to_wasm(
        &self,
        type_expr: &crate::ast::TypeExpr,
    ) -> Result<wasm_encoder::ValType, WasmError> {
        use crate::ast::TypeExpr;
        use wasm_encoder::ValType;

        match type_expr {
            TypeExpr::Named(name) => match name.to_lowercase().as_str() {
                "i32" | "i64" | "int" | "int32" | "int64" => Ok(ValType::I64),
                "f32" | "f64" | "float" | "float32" | "float64" => Ok(ValType::F64),
                "bool" | "boolean" => Ok(ValType::I32),
                other => Err(WasmError::new(format!(
                    "Unsupported type for WASM compilation: {}",
                    other
                ))),
            },
            TypeExpr::Generic { .. } => Err(WasmError::new(
                "Generic types not yet supported in WASM compilation",
            )),
            TypeExpr::Function { .. } => Err(WasmError::new(
                "Function types not yet supported in WASM compilation",
            )),
            TypeExpr::Tuple(_) => Err(WasmError::new(
                "Tuple types not yet supported in WASM compilation",
            )),
            TypeExpr::Enum { .. } => Err(WasmError::new(
                "Enum types not yet supported in WASM compilation",
            )),
            TypeExpr::Never => Err(WasmError::new(
                "Never type not supported in WASM compilation",
            )),
        }
    }

    /// Pre-scan statements to collect all let bindings and declare them in the LocalsTable.
    ///
    /// This must be done before emitting any instructions so that all locals
    /// are declared upfront in the WASM function.
    fn collect_locals(
        &self,
        stmts: &[crate::ast::Stmt],
        locals: &mut LocalsTable,
    ) -> Result<(), WasmError> {
        use crate::ast::Stmt;
        use wasm_encoder::ValType;

        for stmt in stmts {
            match stmt {
                Stmt::Let {
                    name,
                    type_ann,
                    value,
                } => {
                    // Infer WASM type from annotation or default to i64
                    let val_type = if let Some(ty) = type_ann {
                        self.dol_type_to_wasm(ty)?
                    } else {
                        // For struct literals, use I32 (pointer)
                        if matches!(value, crate::ast::Expr::StructLiteral { .. }) {
                            ValType::I32
                        } else {
                            // Default to i64 for untyped locals
                            ValType::I64
                        }
                    };
                    locals.declare(name, val_type);

                    // Track DOL type for struct literals to enable member access
                    if let crate::ast::Expr::StructLiteral { type_name, .. } = value {
                        locals.set_dol_type(name, type_name);
                    }
                }
                // For loops: declare the loop variable and a temp for the end value
                Stmt::For { binding, body, .. } => {
                    // Declare the loop variable (i64 for now)
                    locals.declare(binding, ValType::I64);
                    // Declare a temp local for the end value of the range
                    // We use a unique name based on the binding to avoid conflicts
                    let end_var_name = format!("__for_end_{}", binding);
                    locals.declare(&end_var_name, ValType::I64);
                    // Recursively collect from body
                    self.collect_locals(body, locals)?;
                }
                // While and Loop: just recursively collect from body
                Stmt::While { body, .. } | Stmt::Loop { body, .. } => {
                    self.collect_locals(body, locals)?;
                }
                // Expression statements and return statements may contain match expressions
                Stmt::Expr(expr) | Stmt::Return(Some(expr)) => {
                    self.collect_locals_from_expr(expr, locals)?;
                }
                // Other statements don't declare locals
                _ => {}
            }
        }

        Ok(())
    }

    /// Scan an expression tree for match expressions and declare necessary locals.
    #[allow(clippy::only_used_in_recursion)]
    fn collect_locals_from_expr(
        &self,
        expr: &crate::ast::Expr,
        locals: &mut LocalsTable,
    ) -> Result<(), WasmError> {
        use crate::ast::Expr;
        use wasm_encoder::ValType;

        match expr {
            Expr::Match { arms, scrutinee } => {
                // Declare the match temp local if not already declared
                if locals.lookup("__match_temp").is_none() {
                    locals.declare("__match_temp", ValType::I64);
                }
                // Recurse into scrutinee and arm bodies
                self.collect_locals_from_expr(scrutinee, locals)?;
                for arm in arms {
                    self.collect_locals_from_expr(&arm.body, locals)?;
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_locals_from_expr(condition, locals)?;
                self.collect_locals_from_expr(then_branch, locals)?;
                if let Some(else_expr) = else_branch {
                    self.collect_locals_from_expr(else_expr, locals)?;
                }
            }
            Expr::Block {
                statements,
                final_expr,
            } => {
                for stmt in statements {
                    match stmt {
                        crate::ast::Stmt::Expr(e) | crate::ast::Stmt::Return(Some(e)) => {
                            self.collect_locals_from_expr(e, locals)?;
                        }
                        _ => {}
                    }
                }
                if let Some(expr) = final_expr {
                    self.collect_locals_from_expr(expr, locals)?;
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_locals_from_expr(left, locals)?;
                self.collect_locals_from_expr(right, locals)?;
            }
            Expr::Unary { operand, .. } => {
                self.collect_locals_from_expr(operand, locals)?;
            }
            Expr::Call { callee, args } => {
                self.collect_locals_from_expr(callee, locals)?;
                for arg in args {
                    self.collect_locals_from_expr(arg, locals)?;
                }
            }
            // Other expressions don't contain nested expressions that matter
            _ => {}
        }

        Ok(())
    }

    /// Emit the body of a function as WASM instructions.
    fn emit_function_body(
        &self,
        function: &mut wasm_encoder::Function,
        func_decl: &crate::ast::FunctionDecl,
        locals: &LocalsTable,
    ) -> Result<(), WasmError> {
        use crate::ast::Stmt;
        use wasm_encoder::Instruction;

        let has_return_type = func_decl.return_type.is_some();
        let stmt_count = func_decl.body.len();

        // Emit each statement in the function body
        for (i, stmt) in func_decl.body.iter().enumerate() {
            let is_last = i == stmt_count - 1;

            // Special handling for last expression statement in functions with return types
            if is_last && has_return_type {
                if let Stmt::Expr(expr) = stmt {
                    // Emit the expression without dropping - its value becomes the return
                    self.emit_expression(function, expr, locals)?;
                    // No Drop - the value on stack is the return value
                } else if let Stmt::Return(Some(expr)) = stmt {
                    // Explicit return - emit expression and return
                    self.emit_expression(function, expr, locals)?;
                    function.instruction(&Instruction::Return);
                } else if let Stmt::Return(None) = stmt {
                    // Explicit void return
                    function.instruction(&Instruction::Return);
                } else {
                    // Other statement types - emit normally
                    self.emit_statement(function, stmt, locals)?;
                }
            } else {
                // Not the last statement - emit normally
                self.emit_statement(function, stmt, locals)?;
            }
        }

        // Add end instruction (required for all WASM functions)
        function.instruction(&Instruction::End);

        Ok(())
    }

    /// Emit a statement as WASM instructions.
    fn emit_statement(
        &self,
        function: &mut wasm_encoder::Function,
        stmt: &crate::ast::Stmt,
        locals: &LocalsTable,
    ) -> Result<(), WasmError> {
        use crate::ast::{Expr, Stmt};
        use wasm_encoder::Instruction;

        match stmt {
            Stmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.emit_expression(function, expr, locals)?;
                }
                function.instruction(&Instruction::Return);
            }
            Stmt::Expr(expr) => {
                self.emit_expression(function, expr, locals)?;
                // Drop the result if it's an expression statement that produces a value
                // Note: Some expressions like if-without-else produce no value
                if self.expression_produces_value(expr) {
                    function.instruction(&Instruction::Drop);
                }
            }
            Stmt::Let { name, value, .. } => {
                // Emit the value expression
                self.emit_expression(function, value, locals)?;

                // Look up the local index (should exist from collect_locals pass)
                let local_idx = locals.lookup(name).ok_or_else(|| {
                    WasmError::new(format!(
                        "Internal error: local '{}' not found in locals table",
                        name
                    ))
                })?;

                // Store the value in the local
                function.instruction(&Instruction::LocalSet(local_idx));
            }
            Stmt::Assign { target, value } => {
                // Handle assignment to different target types
                match target {
                    Expr::Identifier(name) => {
                        // Emit the value expression
                        self.emit_expression(function, value, locals)?;

                        // Look up the local index
                        let local_idx = locals.lookup(name).ok_or_else(|| {
                            WasmError::new(format!("Cannot assign to unknown variable: {}", name))
                        })?;

                        // Store the value in the local
                        function.instruction(&Instruction::LocalSet(local_idx));
                    }
                    Expr::Member { .. } => {
                        return Err(WasmError::new(
                            "Member assignment not yet supported in WASM compilation (Phase 3)",
                        ))
                    }
                    _ => {
                        return Err(WasmError::new(format!(
                            "Unsupported assignment target: {:?}",
                            target
                        )))
                    }
                }
            }
            Stmt::While { condition, body } => {
                // Outer block for break target (depth 1 from inside loop)
                function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));

                // Inner loop for continue target (depth 0 from inside loop)
                function.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

                // Evaluate condition
                self.emit_expression(function, condition, locals)?;

                // Branch out of outer block if condition is false (i32.eqz inverts boolean)
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1)); // Break to outer block

                // Loop body
                for stmt in body {
                    self.emit_statement(function, stmt, locals)?;
                }

                // Continue - branch back to loop start (depth 0)
                function.instruction(&Instruction::Br(0));

                function.instruction(&Instruction::End); // End loop
                function.instruction(&Instruction::End); // End block
            }
            Stmt::For {
                binding,
                iterable,
                body,
            } => {
                // For now, only support range iteration: for i in start..end
                if let Expr::Binary {
                    left,
                    op: crate::ast::BinaryOp::Range,
                    right,
                } = iterable
                {
                    // Look up the loop variable (declared in collect_locals)
                    let loop_var = locals.lookup(binding).ok_or_else(|| {
                        WasmError::new(format!(
                            "Internal error: loop variable '{}' not found",
                            binding
                        ))
                    })?;

                    // Look up the end variable
                    let end_var_name = format!("__for_end_{}", binding);
                    let end_var = locals.lookup(&end_var_name).ok_or_else(|| {
                        WasmError::new(format!(
                            "Internal error: end variable '{}' not found",
                            end_var_name
                        ))
                    })?;

                    // Initialize loop variable with start value
                    self.emit_expression(function, left, locals)?;
                    function.instruction(&Instruction::LocalSet(loop_var));

                    // Store end value
                    self.emit_expression(function, right, locals)?;
                    function.instruction(&Instruction::LocalSet(end_var));

                    // Outer block for break
                    function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));

                    // Loop
                    function.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

                    // Check condition: loop_var < end_var
                    function.instruction(&Instruction::LocalGet(loop_var));
                    function.instruction(&Instruction::LocalGet(end_var));
                    function.instruction(&Instruction::I64LtS);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::BrIf(1)); // Break if not less

                    // Body
                    for stmt in body {
                        self.emit_statement(function, stmt, locals)?;
                    }

                    // Increment loop variable
                    function.instruction(&Instruction::LocalGet(loop_var));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(loop_var));

                    // Continue
                    function.instruction(&Instruction::Br(0));

                    function.instruction(&Instruction::End); // End loop
                    function.instruction(&Instruction::End); // End block
                } else {
                    return Err(WasmError::new(
                        "For loops currently only support range iteration (start..end)",
                    ));
                }
            }
            Stmt::Loop { body } => {
                // Outer block for break target
                function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));

                // Inner loop for continue target
                function.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

                // Loop body
                for stmt in body {
                    self.emit_statement(function, stmt, locals)?;
                }

                // Continue - infinite loop back to start
                function.instruction(&Instruction::Br(0));

                function.instruction(&Instruction::End); // End loop
                function.instruction(&Instruction::End); // End block
            }
            Stmt::Break => {
                // Break to outer block (depth 1 from inside the loop)
                // Note: This assumes we're directly inside a loop. For nested
                // control flow, we would need block depth tracking.
                function.instruction(&Instruction::Br(1));
            }
            Stmt::Continue => {
                // Continue to loop start (depth 0 from inside the loop)
                // Note: This assumes we're directly inside a loop. For nested
                // control flow, we would need block depth tracking.
                function.instruction(&Instruction::Br(0));
            }
        }

        Ok(())
    }

    /// Emit an expression as WASM instructions.
    fn emit_expression(
        &self,
        function: &mut wasm_encoder::Function,
        expr: &crate::ast::Expr,
        locals: &LocalsTable,
    ) -> Result<(), WasmError> {
        use crate::ast::{Expr, Literal};
        use wasm_encoder::Instruction;

        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(i) => {
                    function.instruction(&Instruction::I64Const(*i));
                }
                Literal::Float(f) => {
                    function.instruction(&Instruction::F64Const(*f));
                }
                Literal::Bool(b) => {
                    function.instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                }
                Literal::String(_) => {
                    return Err(WasmError::new(
                        "String literals not yet supported in WASM compilation",
                    ))
                }
                Literal::Char(_) => {
                    return Err(WasmError::new(
                        "Char literals not yet supported in WASM compilation",
                    ))
                }
                Literal::Null => {
                    return Err(WasmError::new(
                        "Null literals not yet supported in WASM compilation",
                    ))
                }
            },
            Expr::Identifier(name) => {
                // Check if this is a dotted identifier (e.g., "p.x") which the lexer
                // produces as a single token when there's no whitespace around the dot.
                // This is effectively a member access that needs special handling.
                if let Some(dot_pos) = name.find('.') {
                    let object_name = &name[..dot_pos];
                    let field_name = &name[dot_pos + 1..];

                    // Look up the object variable
                    let local_idx = locals.lookup(object_name).ok_or_else(|| {
                        WasmError::new(format!("Unknown identifier: {}", object_name))
                    })?;

                    // Get the DOL type of the object
                    let gene_type = locals.lookup_dol_type(object_name).map(|s| s.to_string());

                    // Emit the object (pointer) onto the stack
                    function.instruction(&Instruction::LocalGet(local_idx));

                    // Look up the gene layout and field
                    if let Some(type_name) = gene_type {
                        if let Some(layout) = self.gene_layouts.get(&type_name) {
                            if let Some(field_layout) = layout.get_field(field_name) {
                                // Emit load instruction based on field type
                                use crate::wasm::layout::WasmFieldType;
                                match field_layout.wasm_type {
                                    WasmFieldType::I64 => {
                                        function.instruction(&Instruction::I64Load(
                                            wasm_encoder::MemArg {
                                                offset: field_layout.offset as u64,
                                                align: 3, // log2(8) = 3
                                                memory_index: 0,
                                            },
                                        ));
                                    }
                                    WasmFieldType::F64 => {
                                        function.instruction(&Instruction::F64Load(
                                            wasm_encoder::MemArg {
                                                offset: field_layout.offset as u64,
                                                align: 3, // log2(8) = 3
                                                memory_index: 0,
                                            },
                                        ));
                                    }
                                    WasmFieldType::I32 | WasmFieldType::Ptr => {
                                        function.instruction(&Instruction::I32Load(
                                            wasm_encoder::MemArg {
                                                offset: field_layout.offset as u64,
                                                align: 2, // log2(4) = 2
                                                memory_index: 0,
                                            },
                                        ));
                                    }
                                    WasmFieldType::F32 => {
                                        function.instruction(&Instruction::F32Load(
                                            wasm_encoder::MemArg {
                                                offset: field_layout.offset as u64,
                                                align: 2, // log2(4) = 2
                                                memory_index: 0,
                                            },
                                        ));
                                    }
                                }
                            } else {
                                return Err(WasmError::new(format!(
                                    "Unknown field '{}' in gene '{}'",
                                    field_name, type_name
                                )));
                            }
                        } else {
                            return Err(WasmError::new(format!(
                                "Gene type '{}' not registered. Call compiler.register_gene_layout() first.",
                                type_name
                            )));
                        }
                    } else {
                        return Err(WasmError::new(format!(
                            "Member access to field '{}' requires type inference. \
                             Ensure the object '{}' is assigned from a struct literal.",
                            field_name, object_name
                        )));
                    }
                } else if locals.is_gene_field(name) {
                    // This is an implicit self field access (e.g., 'value' in a gene method)
                    // Emit: local.get $self; i64.load <offset>
                    let gene_name = locals.get_gene_name_for_field(name).ok_or_else(|| {
                        WasmError::new(format!(
                            "Internal error: gene field '{}' has no associated gene name",
                            name
                        ))
                    })?;

                    // Get the gene layout
                    if let Some(layout) = self.gene_layouts.get(gene_name) {
                        if let Some(field_layout) = layout.get_field(name) {
                            // Emit: local.get $self (index 0 for gene methods)
                            function.instruction(&Instruction::LocalGet(0));

                            // Emit load instruction based on field type
                            use crate::wasm::layout::WasmFieldType;
                            match field_layout.wasm_type {
                                WasmFieldType::I64 => {
                                    function.instruction(&Instruction::I64Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 3, // log2(8) = 3
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::F64 => {
                                    function.instruction(&Instruction::F64Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 3, // log2(8) = 3
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::I32 | WasmFieldType::Ptr => {
                                    function.instruction(&Instruction::I32Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 2, // log2(4) = 2
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::F32 => {
                                    function.instruction(&Instruction::F32Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 2, // log2(4) = 2
                                            memory_index: 0,
                                        },
                                    ));
                                }
                            }
                        } else {
                            return Err(WasmError::new(format!(
                                "Unknown field '{}' in gene '{}'",
                                name, gene_name
                            )));
                        }
                    } else {
                        return Err(WasmError::new(format!(
                            "Gene '{}' not registered. Gene layouts are auto-registered for genes with fields.",
                            gene_name
                        )));
                    }
                } else {
                    // Simple identifier - look up in locals table
                    let local_idx = locals
                        .lookup(name)
                        .ok_or_else(|| WasmError::new(format!("Unknown identifier: {}", name)))?;
                    function.instruction(&Instruction::LocalGet(local_idx));
                }
            }
            Expr::Binary { left, op, right } => {
                // Emit left operand
                self.emit_expression(function, left, locals)?;
                // Emit right operand
                self.emit_expression(function, right, locals)?;
                // Emit operation
                self.emit_binary_op(function, *op)?;
            }
            Expr::Call { callee, args } => {
                // For now, only support direct function calls (not expressions)
                if let Expr::Identifier(func_name) = callee.as_ref() {
                    // Emit arguments
                    for arg in args {
                        self.emit_expression(function, arg, locals)?;
                    }
                    // Look up function index
                    let func_idx = locals.lookup_function(func_name).ok_or_else(|| {
                        WasmError::new(format!("Unknown function: {}", func_name))
                    })?;
                    function.instruction(&Instruction::Call(func_idx));
                } else {
                    return Err(WasmError::new(
                        "Only direct function calls are supported in WASM compilation",
                    ));
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Emit condition (should produce i32 boolean value)
                self.emit_expression(function, condition, locals)?;

                // Determine result type based on whether we have an else branch
                let block_type = if else_branch.is_some() {
                    // Both branches produce a value - assume i64 for now
                    wasm_encoder::BlockType::Result(wasm_encoder::ValType::I64)
                } else {
                    // No else branch means no value produced
                    wasm_encoder::BlockType::Empty
                };

                function.instruction(&Instruction::If(block_type));

                // Emit then branch
                self.emit_expression(function, then_branch, locals)?;

                // Emit else branch if present
                if let Some(else_expr) = else_branch {
                    function.instruction(&Instruction::Else);
                    self.emit_expression(function, else_expr, locals)?;
                }

                function.instruction(&Instruction::End);
            }
            Expr::Block {
                statements,
                final_expr,
            } => {
                // Emit all statements in the block
                for stmt in statements {
                    self.emit_statement(function, stmt, locals)?;
                }

                // Emit final expression if present (this becomes the block's value)
                if let Some(expr) = final_expr {
                    self.emit_expression(function, expr, locals)?;
                }
            }
            Expr::Match { scrutinee, arms } => {
                use crate::ast::Pattern;

                if arms.is_empty() {
                    return Err(WasmError::new(
                        "Match expression must have at least one arm",
                    ));
                }

                // Find if there's a wildcard arm (for the else case)
                let wildcard_idx = arms
                    .iter()
                    .position(|arm| matches!(&arm.pattern, Pattern::Wildcard));

                // Emit the scrutinee value
                self.emit_expression(function, scrutinee, locals)?;

                // Store in a temporary local for multiple pattern checks
                let temp_local = locals.lookup("__match_temp").ok_or_else(|| {
                    WasmError::new(
                        "Match expressions require __match_temp local to be declared. \
                         Ensure collect_locals handles match expressions.",
                    )
                })?;
                function.instruction(&Instruction::LocalSet(temp_local));

                // Generate nested if-else for each arm
                // Start from the first arm (excluding wildcard if it exists)
                self.emit_match_arms(function, arms, 0, temp_local, wildcard_idx, locals)?;
            }
            Expr::Lambda { .. } => {
                return Err(WasmError::new(
                    "Lambda expressions not yet supported in WASM compilation",
                ))
            }
            Expr::Member { object, field } => {
                // Try to infer the gene type from the object expression
                let gene_type = match object.as_ref() {
                    Expr::Identifier(var_name) => {
                        // Look up DOL type from locals table
                        locals.lookup_dol_type(var_name).map(|s| s.to_string())
                    }
                    _ => None, // For complex expressions, type inference not yet supported
                };

                // Emit object expression (should produce a pointer)
                self.emit_expression(function, object, locals)?;

                // Look up the gene layout and field
                if let Some(type_name) = gene_type {
                    if let Some(layout) = self.gene_layouts.get(&type_name) {
                        if let Some(field_layout) = layout.get_field(field) {
                            // Emit load instruction based on field type
                            use crate::wasm::layout::WasmFieldType;
                            match field_layout.wasm_type {
                                WasmFieldType::I64 => {
                                    function.instruction(&Instruction::I64Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 3, // log2(8) = 3
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::F64 => {
                                    function.instruction(&Instruction::F64Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 3, // log2(8) = 3
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::I32 | WasmFieldType::Ptr => {
                                    function.instruction(&Instruction::I32Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 2, // log2(4) = 2
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::F32 => {
                                    function.instruction(&Instruction::F32Load(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 2, // log2(4) = 2
                                            memory_index: 0,
                                        },
                                    ));
                                }
                            }
                        } else {
                            return Err(WasmError::new(format!(
                                "Unknown field '{}' in gene '{}'",
                                field, type_name
                            )));
                        }
                    } else {
                        return Err(WasmError::new(format!(
                            "Gene type '{}' not registered. Call compiler.register_gene_layout() first.",
                            type_name
                        )));
                    }
                } else {
                    return Err(WasmError::new(format!(
                        "Member access to field '{}' requires type inference. \
                         Currently only simple variable access (e.g., 'p.x' where 'p' is assigned a struct literal) is supported.",
                        field
                    )));
                }
            }
            Expr::Unary { op, operand } => {
                use crate::ast::UnaryOp;

                match op {
                    UnaryOp::Neg => {
                        // For negation: 0 - value (i64)
                        function.instruction(&Instruction::I64Const(0));
                        self.emit_expression(function, operand, locals)?;
                        function.instruction(&Instruction::I64Sub);
                    }
                    UnaryOp::Not => {
                        // For boolean not: eqz (value == 0)
                        self.emit_expression(function, operand, locals)?;
                        function.instruction(&Instruction::I64Eqz);
                    }
                    _ => {
                        return Err(WasmError::new(format!(
                            "Unsupported unary operator: {:?}",
                            op
                        )));
                    }
                }
            }
            Expr::List(_) | Expr::Tuple(_) => {
                return Err(WasmError::new(
                    "List/tuple literals not yet supported in WASM compilation",
                ))
            }
            Expr::Forall { .. } | Expr::Exists { .. } => {
                return Err(WasmError::new(
                    "Quantifier expressions not yet supported in WASM compilation",
                ))
            }
            Expr::Quote(_) | Expr::Unquote(_) | Expr::Reflect(_) => {
                return Err(WasmError::new(
                    "Metaprogramming expressions not yet supported in WASM compilation",
                ))
            }
            Expr::SexBlock { .. } => {
                return Err(WasmError::new(
                    "Sex blocks not yet supported in WASM compilation",
                ))
            }
            Expr::Cast { .. } => {
                return Err(WasmError::new(
                    "Type casts not yet supported in WASM compilation",
                ))
            }
            Expr::Try(_) => {
                return Err(WasmError::new(
                    "Try expressions not yet supported in WASM compilation",
                ))
            }
            Expr::QuasiQuote(_) | Expr::Eval(_) => {
                return Err(WasmError::new(
                    "Quasi-quote/eval expressions not yet supported in WASM compilation",
                ))
            }
            Expr::IdiomBracket { .. } => {
                return Err(WasmError::new(
                    "Idiom bracket expressions not yet supported in WASM compilation",
                ))
            }
            Expr::Implies { .. } => {
                return Err(WasmError::new(
                    "Implies expressions not yet supported in WASM compilation",
                ))
            }
            Expr::StructLiteral { type_name, fields } => {
                // Check if we have a layout for this gene
                if let Some(layout) = self.gene_layouts.get(type_name) {
                    // Allocate space - call alloc function
                    // Alloc signature: (size: i32, align: i32) -> i32 (pointer)
                    function.instruction(&Instruction::I32Const(layout.total_size as i32));
                    function.instruction(&Instruction::I32Const(layout.alignment as i32));
                    function.instruction(&Instruction::Call(0)); // alloc is at index 0 when memory is enabled

                    // Store the pointer in the temp local (__struct_ptr should be declared)
                    let ptr_local = locals.lookup("__struct_ptr").ok_or_else(|| {
                        WasmError::new(
                            "Internal error: __struct_ptr local not found. \
                             Ensure gene layouts are registered before compilation.",
                        )
                    })?;
                    function.instruction(&Instruction::LocalTee(ptr_local));

                    // Initialize each field
                    for (field_name, field_value) in fields {
                        if let Some(field_layout) = layout.get_field(field_name) {
                            // Get pointer
                            function.instruction(&Instruction::LocalGet(ptr_local));
                            // Emit value
                            self.emit_expression(function, field_value, locals)?;
                            // Store at offset
                            use crate::wasm::layout::WasmFieldType;
                            match field_layout.wasm_type {
                                WasmFieldType::I64 => {
                                    function.instruction(&Instruction::I64Store(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 3, // log2(8) = 3
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::F64 => {
                                    function.instruction(&Instruction::F64Store(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 3, // log2(8) = 3
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::I32 | WasmFieldType::Ptr => {
                                    function.instruction(&Instruction::I32Store(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 2, // log2(4) = 2
                                            memory_index: 0,
                                        },
                                    ));
                                }
                                WasmFieldType::F32 => {
                                    function.instruction(&Instruction::F32Store(
                                        wasm_encoder::MemArg {
                                            offset: field_layout.offset as u64,
                                            align: 2, // log2(4) = 2
                                            memory_index: 0,
                                        },
                                    ));
                                }
                            }
                        } else {
                            return Err(WasmError::new(format!(
                                "Unknown field '{}' in gene '{}'",
                                field_name, type_name
                            )));
                        }
                    }

                    // Leave pointer on stack as result
                    function.instruction(&Instruction::LocalGet(ptr_local));
                } else {
                    return Err(WasmError::new(format!(
                        "Unknown gene type for struct literal: '{}'. \
                         Register the gene layout with compiler.register_gene_layout() first.",
                        type_name
                    )));
                }
            }
        }

        Ok(())
    }

    /// Emit match arms as nested if-else expressions.
    ///
    /// This generates a chain of if-else blocks for pattern matching:
    /// - Literal patterns: compare scrutinee with literal
    /// - Wildcard pattern: unconditional else case
    /// - Other patterns: currently unsupported
    fn emit_match_arms(
        &self,
        function: &mut wasm_encoder::Function,
        arms: &[crate::ast::MatchArm],
        current_idx: usize,
        temp_local: u32,
        wildcard_idx: Option<usize>,
        locals: &LocalsTable,
    ) -> Result<(), WasmError> {
        use crate::ast::{Literal, Pattern};
        use wasm_encoder::Instruction;

        if current_idx >= arms.len() {
            // No more arms, should have a wildcard
            return Err(WasmError::new(
                "Match expression is not exhaustive (no wildcard arm)",
            ));
        }

        let arm = &arms[current_idx];

        // Skip wildcard pattern if we're iterating - it's handled as the else case
        if matches!(&arm.pattern, Pattern::Wildcard) {
            if current_idx + 1 < arms.len() {
                // Wildcard should be last, but continue with other arms
                return self.emit_match_arms(
                    function,
                    arms,
                    current_idx + 1,
                    temp_local,
                    wildcard_idx,
                    locals,
                );
            } else {
                // This is the last arm and it's wildcard - emit its body directly
                self.emit_expression(function, &arm.body, locals)?;
                return Ok(());
            }
        }

        // Pattern matching for current arm
        match &arm.pattern {
            Pattern::Literal(lit) => {
                // Get the scrutinee value
                function.instruction(&Instruction::LocalGet(temp_local));

                // Emit comparison with literal
                match lit {
                    Literal::Int(n) => {
                        function.instruction(&Instruction::I64Const(*n));
                        function.instruction(&Instruction::I64Eq);
                    }
                    Literal::Bool(b) => {
                        function.instruction(&Instruction::I64Const(if *b { 1 } else { 0 }));
                        function.instruction(&Instruction::I64Eq);
                    }
                    _ => {
                        return Err(WasmError::new(format!(
                            "Unsupported literal pattern type: {:?}",
                            lit
                        )))
                    }
                }

                // If matches, emit body; else try next arm
                function.instruction(&Instruction::If(wasm_encoder::BlockType::Result(
                    wasm_encoder::ValType::I64,
                )));

                // Emit the body for this arm
                self.emit_expression(function, &arm.body, locals)?;

                function.instruction(&Instruction::Else);

                // Try remaining arms or wildcard
                let next_non_wildcard = (current_idx + 1..arms.len())
                    .find(|&i| !matches!(&arms[i].pattern, Pattern::Wildcard));

                if let Some(next_idx) = next_non_wildcard {
                    self.emit_match_arms(
                        function,
                        arms,
                        next_idx,
                        temp_local,
                        wildcard_idx,
                        locals,
                    )?;
                } else if let Some(wild_idx) = wildcard_idx {
                    // Emit wildcard body
                    self.emit_expression(function, &arms[wild_idx].body, locals)?;
                } else {
                    // No wildcard - this is non-exhaustive
                    // Emit unreachable or a default value
                    function.instruction(&Instruction::Unreachable);
                }

                function.instruction(&Instruction::End);
            }
            Pattern::Identifier(_) => {
                // Identifier pattern binds the value - for now, treat like wildcard
                // In a full implementation, we'd bind to a local variable
                self.emit_expression(function, &arm.body, locals)?;
            }
            _ => {
                return Err(WasmError::new(format!(
                    "Unsupported pattern type in match expression: {:?}",
                    arm.pattern
                )))
            }
        }

        Ok(())
    }

    /// Emit a binary operation as a WASM instruction.
    fn emit_binary_op(
        &self,
        function: &mut wasm_encoder::Function,
        op: crate::ast::BinaryOp,
    ) -> Result<(), WasmError> {
        use crate::ast::BinaryOp;
        use wasm_encoder::Instruction;

        // For simplicity, assume i64 operations
        // A full implementation would track types through the expression tree
        match op {
            BinaryOp::Add => {
                function.instruction(&Instruction::I64Add);
            }
            BinaryOp::Sub => {
                function.instruction(&Instruction::I64Sub);
            }
            BinaryOp::Mul => {
                function.instruction(&Instruction::I64Mul);
            }
            BinaryOp::Div => {
                function.instruction(&Instruction::I64DivS);
            }
            BinaryOp::Mod => {
                function.instruction(&Instruction::I64RemS);
            }
            BinaryOp::Eq => {
                function.instruction(&Instruction::I64Eq);
            }
            BinaryOp::Ne => {
                function.instruction(&Instruction::I64Ne);
            }
            BinaryOp::Lt => {
                function.instruction(&Instruction::I64LtS);
            }
            BinaryOp::Le => {
                function.instruction(&Instruction::I64LeS);
            }
            BinaryOp::Gt => {
                function.instruction(&Instruction::I64GtS);
            }
            BinaryOp::Ge => {
                function.instruction(&Instruction::I64GeS);
            }
            BinaryOp::And => {
                function.instruction(&Instruction::I64And);
            }
            BinaryOp::Or => {
                function.instruction(&Instruction::I64Or);
            }
            BinaryOp::Pow => {
                return Err(WasmError::new(
                    "Exponentiation not supported in basic WASM (requires math functions)",
                ))
            }
            BinaryOp::Pipe
            | BinaryOp::Compose
            | BinaryOp::Apply
            | BinaryOp::Bind
            | BinaryOp::Member
            | BinaryOp::Map
            | BinaryOp::Ap
            | BinaryOp::Implies
            | BinaryOp::Range => {
                return Err(WasmError::new(format!(
                    "Operator {:?} not supported in WASM compilation",
                    op
                )))
            }
        }

        Ok(())
    }

    /// Check if an expression produces a value on the WASM stack.
    ///
    /// Some expressions like `if` without `else` produce no value.
    fn expression_produces_value(&self, expr: &crate::ast::Expr) -> bool {
        use crate::ast::Expr;

        match expr {
            // If without else produces no value
            Expr::If {
                else_branch: None, ..
            } => false,

            // Block without final expression produces no value
            Expr::Block {
                final_expr: None, ..
            } => false,

            // All other expressions produce a value
            _ => true,
        }
    }

    /// Compile a DOL module to WASM and write to a file.
    ///
    /// Convenience method that calls [`compile`](WasmCompiler::compile) and
    /// writes the resulting bytecode to a file.
    ///
    /// # Arguments
    ///
    /// * `module` - The DOL module to compile
    /// * `output_path` - Path to write the WASM file
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `WasmError` if compilation or writing fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    /// use metadol::parse_file;
    ///
    /// let module = parse_file(source)?;
    /// let compiler = WasmCompiler::new();
    /// compiler.compile_to_file(&module, "output.wasm")?;
    /// ```
    pub fn compile_to_file(
        &mut self,
        module: &Declaration,
        output_path: impl AsRef<Path>,
    ) -> Result<(), WasmError> {
        let wasm_bytes = self.compile(module)?;
        std::fs::write(output_path, wasm_bytes)?;
        Ok(())
    }
}

#[cfg(feature = "wasm")]
impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(feature = "wasm")]
mod tests {
    use super::*;
    use crate::ast::{
        BinaryOp, Expr, FunctionDecl, FunctionParam, Literal, Purity, Span, Stmt, TypeExpr,
        Visibility,
    };

    #[test]
    fn test_compiler_new() {
        let compiler = WasmCompiler::new();
        assert!(!compiler.optimize);
        assert!(compiler.debug_info);
    }

    #[test]
    fn test_compiler_with_optimization() {
        let compiler = WasmCompiler::new().with_optimization(true);
        assert!(compiler.optimize);
    }

    #[test]
    fn test_compiler_with_debug_info() {
        let compiler = WasmCompiler::new().with_debug_info(false);
        assert!(!compiler.debug_info);
    }

    #[test]
    fn test_compiler_chaining() {
        let compiler = WasmCompiler::new()
            .with_optimization(true)
            .with_debug_info(false);
        assert!(compiler.optimize);
        assert!(!compiler.debug_info);
    }

    #[test]
    fn test_compiler_default() {
        let compiler = WasmCompiler::default();
        assert!(!compiler.optimize);
        assert!(compiler.debug_info);
    }

    #[test]
    fn test_compile_simple_function() {
        // Create a simple function: fun add(a: i64, b: i64) -> i64 { return a + b }
        let func = FunctionDecl {
            visibility: Visibility::Public,
            purity: Purity::Pure,
            name: "add".to_string(),
            type_params: None,
            params: vec![
                FunctionParam {
                    name: "a".to_string(),
                    type_ann: TypeExpr::Named("i64".to_string()),
                },
                FunctionParam {
                    name: "b".to_string(),
                    type_ann: TypeExpr::Named("i64".to_string()),
                },
            ],
            return_type: Some(TypeExpr::Named("i64".to_string())),
            body: vec![Stmt::Return(Some(Expr::Binary {
                left: Box::new(Expr::Identifier("a".to_string())),
                op: BinaryOp::Add,
                right: Box::new(Expr::Identifier("b".to_string())),
            }))],
            exegesis: "Adds two numbers".to_string(),
            span: Span::default(),
        };

        let decl = Declaration::Function(Box::new(func));
        let mut compiler = WasmCompiler::new();
        let wasm_bytes = compiler.compile(&decl).expect("Compilation failed");

        // Verify WASM magic number (0x00 0x61 0x73 0x6D) and version (0x01 0x00 0x00 0x00)
        assert!(wasm_bytes.len() >= 8, "WASM output too short");
        assert_eq!(
            &wasm_bytes[0..4],
            &[0x00, 0x61, 0x73, 0x6D],
            "Invalid WASM magic number"
        );
        assert_eq!(
            &wasm_bytes[4..8],
            &[0x01, 0x00, 0x00, 0x00],
            "Invalid WASM version"
        );
    }

    #[test]
    fn test_compile_function_with_literals() {
        // Create a function that returns a constant: fun get_answer() -> i64 { return 42 }
        let func = FunctionDecl {
            visibility: Visibility::Public,
            purity: Purity::Pure,
            name: "get_answer".to_string(),
            type_params: None,
            params: vec![],
            return_type: Some(TypeExpr::Named("i64".to_string())),
            body: vec![Stmt::Return(Some(Expr::Literal(Literal::Int(42))))],
            exegesis: "Returns the answer to everything".to_string(),
            span: Span::default(),
        };

        let decl = Declaration::Function(Box::new(func));
        let mut compiler = WasmCompiler::new();
        let wasm_bytes = compiler.compile(&decl).expect("Compilation failed");

        // Verify valid WASM output
        assert!(wasm_bytes.len() >= 8);
        assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(&wasm_bytes[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_compile_non_function_declaration_fails() {
        use crate::ast::Gene;

        // Try to compile a Gene (not supported)
        let gene = Gene {
            name: "test.gene".to_string(),
            extends: None,
            statements: vec![],
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        };

        let decl = Declaration::Gene(gene);
        let mut compiler = WasmCompiler::new();
        let result = compiler.compile(&decl);

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("No functions found"));
    }
}
