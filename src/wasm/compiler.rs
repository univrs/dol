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

#[cfg(feature = "wasm-compile")]
use crate::ast::Declaration;
#[cfg(feature = "wasm-compile")]
use crate::wasm::alloc::BumpAllocator;
#[cfg(feature = "wasm-compile")]
use crate::wasm::layout::{EnumRegistry, GeneLayout, GeneLayoutRegistry};
#[cfg(feature = "wasm-compile")]
use crate::wasm::WasmError;
#[cfg(feature = "wasm-compile")]
use std::collections::HashMap;
#[cfg(feature = "wasm-compile")]
use std::path::Path;
#[cfg(feature = "wasm-compile")]
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
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone)]
pub struct WasmCompiler {
    /// Enable LLVM optimizations
    optimize: bool,
    /// Include debug information in WASM
    debug_info: bool,
    /// Enable tree shaking to eliminate dead code
    tree_shaking: bool,
    /// Registry of gene layouts for struct literal compilation
    gene_layouts: GeneLayoutRegistry,
    /// Registry of enum definitions for variant compilation
    enum_registry: EnumRegistry,
}

/// Represents a WASM import declaration.
///
/// Used for sex functions without bodies, which are treated as host function imports.
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone)]
struct WasmImport {
    /// Module name for the import (e.g., "loa")
    module: String,
    /// Function name for the import
    name: String,
    /// Parameter types
    params: Vec<wasm_encoder::ValType>,
    /// Return type (None for void)
    result: Option<wasm_encoder::ValType>,
    /// Type index in the type section (set during compilation)
    type_idx: u32,
}

/// Context for tracking loop control flow depths.
///
/// WASM uses relative block depths for `br` (break) instructions. When control
/// flow is nested inside if/match/block expressions, the depths need to be adjusted.
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone, Copy, Default)]
struct LoopContext {
    /// Depth to break target (outer block surrounding loop)
    /// None if not inside a loop
    break_depth: Option<u32>,
    /// Depth to continue target (loop header)
    /// None if not inside a loop
    continue_depth: Option<u32>,
}

#[cfg(feature = "wasm-compile")]
impl LoopContext {
    /// Create a new context for entering a loop.
    /// break_depth=1 (outer block), continue_depth=0 (loop header)
    fn enter_loop() -> Self {
        Self {
            break_depth: Some(1),
            continue_depth: Some(0),
        }
    }

    /// Increment depths for entering a block (if, match, etc.)
    fn enter_block(&self) -> Self {
        Self {
            break_depth: self.break_depth.map(|d| d + 1),
            continue_depth: self.continue_depth.map(|d| d + 1),
        }
    }
}

/// String table for WASM data section.
///
/// Collects and deduplicates string literals, returning (offset, length) pairs.
/// Strings are stored in the WASM data section starting at a configurable base offset.
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
struct StringTable {
    /// Strings stored as (offset, content)
    strings: Vec<(u32, String)>,
    /// Next available offset
    next_offset: u32,
    /// Base offset in memory (typically after heap pointer)
    base_offset: u32,
}

#[cfg(feature = "wasm-compile")]
#[allow(dead_code)]
impl StringTable {
    /// Create a new string table with a base offset.
    ///
    /// The base offset is where strings will start in WASM linear memory.
    /// Typically this is after the heap pointer and any reserved space.
    fn new(base_offset: u32) -> Self {
        Self {
            strings: Vec::new(),
            next_offset: base_offset,
            base_offset,
        }
    }

    /// Add a string to the table.
    ///
    /// Returns (pointer, length) for the string in WASM memory.
    /// Deduplicates: if the string already exists, returns the existing location.
    fn add(&mut self, s: &str) -> (u32, u32) {
        // Check for existing string (deduplication)
        for (offset, existing) in &self.strings {
            if existing == s {
                return (*offset, s.len() as u32);
            }
        }
        // Add new string
        let offset = self.next_offset;
        let len = s.len() as u32;
        self.strings.push((offset, s.to_string()));
        self.next_offset += len;
        (offset, len)
    }

    /// Check if the table has any strings.
    fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Get the number of strings in the table.
    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.strings.len()
    }

    /// Emit the WASM data section for all strings.
    ///
    /// This creates active data segments that initialize memory with the string contents.
    fn emit_data_section(&self, module: &mut wasm_encoder::Module) {
        if self.strings.is_empty() {
            return;
        }

        let mut data_section = wasm_encoder::DataSection::new();

        for (offset, content) in &self.strings {
            // Active data segment: memory 0, offset expression, bytes
            data_section.active(
                0, // memory index
                &wasm_encoder::ConstExpr::i32_const(*offset as i32),
                content.as_bytes().iter().copied(),
            );
        }

        module.section(&data_section);
    }
}

/// Table for tracking local variables within a function.
///
/// This tracks both function parameters (which are the first locals in WASM)
/// and declared local variables from let bindings.
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Default)]
struct LocalsTable {
    /// Number of function parameters
    param_count: u32,
    /// Maps variable names to their local index
    name_to_index: HashMap<String, u32>,
    /// Types of declared locals (not including parameters)
    local_types: Vec<wasm_encoder::ValType>,
    /// Maps variable names to their WASM ValType for type inference
    var_types: HashMap<String, wasm_encoder::ValType>,
    /// Maps variable names to their DOL type (gene name) for member access
    dol_types: HashMap<String, String>,
    /// Maps function names to their WASM function indices
    func_indices: HashMap<String, u32>,
    /// Maps variable names to their WASM type (for parameters that need widening)
    wasm_types: HashMap<String, wasm_encoder::ValType>,
    /// Maps global variable names to their WASM global indices (sex vars)
    global_indices: HashMap<String, u32>,
    /// Maps function names to their return types
    func_return_types: HashMap<String, wasm_encoder::ValType>,
}

#[cfg(feature = "wasm-compile")]
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
            var_types: HashMap::new(),
            dol_types: HashMap::new(),
            func_indices: HashMap::new(),
            wasm_types: HashMap::new(),
            global_indices: HashMap::new(),
            func_return_types: HashMap::new(),
        };

        // Add implicit 'self' parameter at index 0 for gene methods
        if has_self {
            table.name_to_index.insert("self".to_string(), 0);
            table
                .var_types
                .insert("self".to_string(), wasm_encoder::ValType::I32);
        }

        // Add declared parameters (offset by 1 if we have self)
        for (i, param) in params.iter().enumerate() {
            table
                .name_to_index
                .insert(param.name.clone(), i as u32 + self_offset);
            // Track parameter type for type inference
            let val_type = Self::type_expr_to_val_type(&param.type_ann);
            table.var_types.insert(param.name.clone(), val_type);
        }

        // Track gene field names for implicit field access
        if let Some(ctx) = gene_context {
            // Store gene name for 'self' type lookups
            table
                .dol_types
                .insert("self".to_string(), ctx.gene_name.clone());

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

    /// Register a global variable's WASM index (sex var).
    fn register_global(&mut self, name: &str, index: u32) {
        self.global_indices.insert(name.to_string(), index);
    }

    /// Look up a global variable's WASM index by name.
    fn lookup_global(&self, name: &str) -> Option<u32> {
        self.global_indices.get(name).copied()
    }

    /// Declare a new local variable.
    ///
    /// Returns the index assigned to this local. The index is param_count + local_index.
    fn declare(&mut self, name: &str, val_type: wasm_encoder::ValType) -> u32 {
        let index = self.param_count + self.local_types.len() as u32;
        self.name_to_index.insert(name.to_string(), index);
        self.local_types.push(val_type);
        self.var_types.insert(name.to_string(), val_type);
        index
    }

    /// Look up the WASM ValType of a variable by name.
    fn lookup_var_type(&self, name: &str) -> Option<wasm_encoder::ValType> {
        self.var_types.get(name).copied()
    }

    /// Register a function with its return type.
    #[allow(dead_code)]
    fn register_function_with_type(
        &mut self,
        name: &str,
        index: u32,
        return_type: wasm_encoder::ValType,
    ) {
        self.func_indices.insert(name.to_string(), index);
        self.func_return_types.insert(name.to_string(), return_type);
    }

    /// Look up a function's return type by name.
    fn lookup_function_return_type(&self, name: &str) -> Option<wasm_encoder::ValType> {
        self.func_return_types.get(name).copied()
    }

    /// Convert a DOL TypeExpr to a WASM ValType.
    fn type_expr_to_val_type(ty: &crate::ast::TypeExpr) -> wasm_encoder::ValType {
        use crate::ast::TypeExpr;
        match ty {
            TypeExpr::Named(name) => match name.as_str() {
                "i32" | "Int32" => wasm_encoder::ValType::I32,
                "i64" | "Int64" => wasm_encoder::ValType::I64,
                "f32" | "Float32" => wasm_encoder::ValType::F32,
                "f64" | "Float64" => wasm_encoder::ValType::F64,
                "bool" | "Bool" => wasm_encoder::ValType::I32, // booleans as i32
                _ => wasm_encoder::ValType::I32, // default to i32 for references/gene types
            },
            TypeExpr::Generic { .. } => wasm_encoder::ValType::I32, // generics are pointers
            TypeExpr::Function { .. } => wasm_encoder::ValType::I32, // function refs are pointers
            _ => wasm_encoder::ValType::I64,                        // default
        }
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

    /// Set the WASM type for a variable (used for parameter type tracking).
    fn set_wasm_type(&mut self, name: &str, wasm_type: wasm_encoder::ValType) {
        self.wasm_types.insert(name.to_string(), wasm_type);
    }

    /// Check if a variable needs widening from i32 to i64.
    ///
    /// Returns true if the variable is stored as i32 (e.g., enum types)
    /// but needs to participate in i64 operations.
    fn needs_widening(&self, name: &str) -> bool {
        matches!(self.wasm_types.get(name), Some(wasm_encoder::ValType::I32))
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
#[cfg(feature = "wasm-compile")]
struct ExtractedFunction<'a> {
    /// The function declaration
    func: &'a crate::ast::FunctionDecl,
    /// The name to export as (may include gene prefix)
    exported_name: String,
    /// Gene context for methods (provides field information for implicit self)
    gene_context: Option<GeneContext>,
}

/// Context for a gene method, providing field information for implicit self access.
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone)]
struct GeneContext {
    /// The gene name
    gene_name: String,
    /// Field names defined in this gene (for identifier resolution)
    field_names: Vec<String>,
}

/// Pool for collecting string literals during compilation.
///
/// String literals are stored in the WASM data section. This pool
/// collects unique strings and assigns them memory offsets.
///
/// Memory layout for strings:
/// ```text
/// [4 bytes: length][UTF-8 bytes]
/// ```
///
/// The string value returned by the compiler is a single i32 pointer
/// to the start of this structure (the length prefix).
#[cfg(feature = "wasm-compile")]
#[derive(Debug, Clone, Default)]
struct StringPool {
    /// Maps string content to (offset, length) in data section
    strings: HashMap<String, (u32, u32)>,
    /// Raw bytes to be placed in data section
    data: Vec<u8>,
    /// Current offset in data section
    current_offset: u32,
}

#[cfg(feature = "wasm-compile")]
impl StringPool {
    /// Create a new empty string pool.
    fn new() -> Self {
        Self::default()
    }

    /// Add a string to the pool, returning its memory offset.
    ///
    /// If the string already exists, returns the existing offset.
    /// The returned offset points to the length-prefixed string.
    fn add(&mut self, s: &str) -> u32 {
        if let Some(&(offset, _len)) = self.strings.get(s) {
            return offset;
        }

        let offset = self.current_offset;
        let len = s.len() as u32;

        // Write length prefix (4 bytes, little-endian)
        self.data.extend_from_slice(&len.to_le_bytes());
        // Write string bytes
        self.data.extend_from_slice(s.as_bytes());

        // Track total size: 4 bytes for length + string bytes
        let total_size = 4 + len;
        self.current_offset += total_size;

        // Align to 4 bytes for next entry
        let padding = (4 - (self.current_offset % 4)) % 4;
        self.data
            .extend(std::iter::repeat(0).take(padding as usize));
        self.current_offset += padding;

        self.strings.insert(s.to_string(), (offset, len));
        offset
    }

    /// Get the total size of the data section.
    fn data_size(&self) -> u32 {
        self.current_offset
    }

    /// Get the raw data bytes for the data section.
    fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Check if the pool is empty.
    fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

#[cfg(feature = "wasm-compile")]
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
            tree_shaking: false,
            gene_layouts: GeneLayoutRegistry::new(),
            enum_registry: EnumRegistry::new(),
        }
    }

    /// Register an enum type for compilation.
    ///
    /// When enums are registered, the compiler can resolve enum variant
    /// access expressions (e.g., `AccountType.Node`) to their i32 discriminant.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the enum type
    /// * `variant_names` - The ordered list of variant names
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let mut compiler = WasmCompiler::new();
    ///
    /// // Register AccountType enum
    /// compiler.register_enum("AccountType", vec![
    ///     "Node".to_string(),
    ///     "RevivalPool".to_string(),
    ///     "Treasury".to_string(),
    /// ]);
    ///
    /// // Now AccountType.Node resolves to 0, AccountType.Treasury to 2
    /// ```
    pub fn register_enum(&mut self, name: &str, variant_names: Vec<String>) {
        self.enum_registry.register_enum(name, variant_names);
    }

    /// Get the variant index for an enum variant.
    ///
    /// Returns `None` if the enum or variant doesn't exist.
    pub fn get_enum_variant_index(&self, enum_name: &str, variant_name: &str) -> Option<i32> {
        self.enum_registry
            .get_variant_index(enum_name, variant_name)
    }

    /// Check if an enum with the given name is registered.
    pub fn has_enum(&self, name: &str) -> bool {
        self.enum_registry.contains(name)
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

    /// Get the list of gene names with layouts.
    pub fn get_gene_layout_names(&self) -> Vec<String> {
        self.gene_layouts.names()
    }

    /// Get a gene layout by name.
    pub fn get_gene_layout(&self, name: &str) -> Option<&GeneLayout> {
        self.gene_layouts.get(name)
    }

    /// Build a constructor function for a gene.
    ///
    /// The constructor takes field values as parameters and returns a pointer (i32)
    /// to the newly allocated and initialized gene instance.
    ///
    /// # Arguments
    ///
    /// * `layout` - The gene layout to build a constructor for
    /// * `alloc_func_idx` - The function index of the alloc function
    ///
    /// # Returns
    ///
    /// A tuple of (param_types, result_types, function_body)
    #[cfg(feature = "wasm-compile")]
    fn build_gene_constructor(
        layout: &GeneLayout,
        alloc_func_idx: u32,
    ) -> (
        Vec<wasm_encoder::ValType>,
        Vec<wasm_encoder::ValType>,
        wasm_encoder::Function,
    ) {
        use crate::wasm::layout::WasmFieldType;
        use wasm_encoder::{Function, Instruction, ValType};

        // Build parameter types from fields
        let params: Vec<ValType> = layout
            .fields
            .iter()
            .map(|f| f.wasm_type.to_val_type())
            .collect();

        // Result is always i32 (pointer)
        let results = vec![ValType::I32];

        // Build the function body
        // Locals: one i32 for the struct pointer
        let locals = vec![(1, ValType::I32)];
        let mut func = Function::new(locals);

        // Local 0..n-1 are parameters, local n is __ptr
        let ptr_local = params.len() as u32;

        // Allocate memory: push size, alignment, call alloc
        func.instruction(&Instruction::I32Const(layout.total_size as i32));
        func.instruction(&Instruction::I32Const(layout.alignment as i32));
        func.instruction(&Instruction::Call(alloc_func_idx));
        func.instruction(&Instruction::LocalTee(ptr_local));

        // Initialize each field
        for (param_idx, field) in layout.fields.iter().enumerate() {
            // Get pointer
            func.instruction(&Instruction::LocalGet(ptr_local));
            // Get parameter value
            func.instruction(&Instruction::LocalGet(param_idx as u32));
            // Store at field offset
            let mem_arg = wasm_encoder::MemArg {
                offset: field.offset as u64,
                align: field.wasm_type.alignment_log2(),
                memory_index: 0,
            };
            match field.wasm_type {
                WasmFieldType::I32 | WasmFieldType::Ptr => {
                    func.instruction(&Instruction::I32Store(mem_arg));
                }
                WasmFieldType::I64 => {
                    func.instruction(&Instruction::I64Store(mem_arg));
                }
                WasmFieldType::F32 => {
                    func.instruction(&Instruction::F32Store(mem_arg));
                }
                WasmFieldType::F64 => {
                    func.instruction(&Instruction::F64Store(mem_arg));
                }
            }
        }

        // Return pointer - already on stack from LocalTee above
        // The LocalTee instruction both stores to local AND leaves the value on the stack
        func.instruction(&Instruction::End);

        (params, results, func)
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

    /// Enable or disable tree shaking.
    ///
    /// When enabled, the compiler will eliminate unreachable declarations
    /// before generating WASM bytecode. This reduces output size by removing
    /// unused code.
    ///
    /// # Arguments
    ///
    /// * `tree_shaking` - `true` to enable tree shaking, `false` to disable
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use metadol::wasm::WasmCompiler;
    ///
    /// let compiler = WasmCompiler::new().with_tree_shaking(true);
    /// ```
    pub fn with_tree_shaking(mut self, tree_shaking: bool) -> Self {
        self.tree_shaking = tree_shaking;
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

        // Collect string literals from the module
        let mut string_pool = StringPool::new();
        self.collect_strings_from_declaration(module, &mut string_pool);

        // Check if we need memory allocation (when gene layouts are registered or strings are used)
        let needs_memory = !self.gene_layouts.is_empty() || !string_pool.is_empty();

        // Calculate heap start after string data (aligned to 8 bytes)
        let heap_start = if !string_pool.is_empty() {
            crate::wasm::alloc::align_up(string_pool.data_size(), 8).max(1024)
        } else {
            1024
        };

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
            BumpAllocator::emit_globals(&mut wasm_module, heap_start);
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

            // Track WASM types for parameters (for type-aware operations like i32 widening)
            for param in &extracted.func.params {
                if let Ok(wasm_type) = self.dol_type_to_wasm(&param.type_ann) {
                    locals_table.set_wasm_type(&param.name, wasm_type);
                }
            }

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
            self.emit_function_body(&mut function, extracted.func, &locals_table, &string_pool)?;
            code.function(&function);
        }
        wasm_module.section(&code);

        // Emit data section for string literals (if any)
        if !string_pool.is_empty() {
            use wasm_encoder::DataSection;
            let mut data = DataSection::new();
            // Active data segment at memory 0, offset 0
            data.active(
                0,
                &wasm_encoder::ConstExpr::i32_const(0),
                string_pool.get_data().iter().copied(),
            );
            wasm_module.section(&data);
        }

        Ok(wasm_module.finish())
    }

    /// Compile a complete DOL file containing multiple declarations.
    ///
    /// This method handles files with multiple gene declarations, properly
    /// ordering them so that parent genes are registered before children
    /// (required for inheritance).
    ///
    /// # Arguments
    ///
    /// * `file` - The parsed DOL file containing declarations
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the WASM bytecode on success, or a `WasmError`
    /// if compilation fails.
    pub fn compile_file(&mut self, file: &crate::ast::DolFile) -> Result<Vec<u8>, WasmError> {
        use wasm_encoder::{
            CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
            ImportSection, Module, TypeSection, ValType,
        };

        // Apply tree shaking if enabled
        let declarations = if self.tree_shaking {
            use crate::transform::TreeShaking;
            let mut shaker = TreeShaking::new();
            shaker.shake(file.declarations.clone())
        } else {
            file.declarations.clone()
        };

        // Register gene layouts in dependency order (parents before children)
        self.register_gene_layouts_ordered(&declarations)?;

        // Extract imports (sex fun without body)
        let mut imports = self.extract_imports(&declarations)?;

        // Extract all local functions (with body) from all declarations
        let mut functions: Vec<ExtractedFunction> = Vec::new();
        for decl in &declarations {
            let extracted = self.extract_functions(decl)?;
            // Filter out import functions (sex fun without body)
            for f in extracted {
                if !Self::is_import(f.func) {
                    functions.push(f);
                }
            }
        }

        // Allow modules with only imports (no local functions)
        let has_local_functions = !functions.is_empty();

        // Collect string literals from all declarations
        let mut string_pool = StringPool::new();
        for decl in &declarations {
            self.collect_strings_from_declaration(decl, &mut string_pool);
        }

        // Check if we need memory allocation (when gene layouts are registered or strings are used)
        let needs_memory = !self.gene_layouts.is_empty() || !string_pool.is_empty();

        // Calculate heap start after string data (aligned to 8 bytes)
        let _heap_start = if !string_pool.is_empty() {
            crate::wasm::alloc::align_up(string_pool.data_size(), 8).max(1024)
        } else {
            1024
        };

        // Build WASM module
        let mut wasm_module = Module::new();

        // ============= TYPE SECTION =============
        // Order: alloc type (if needed), import types, local function types
        let mut types = TypeSection::new();
        let mut next_type_idx = 0u32;

        // If we need memory, add the alloc function type first
        if needs_memory {
            let (alloc_params, alloc_results) = BumpAllocator::alloc_type_signature();
            types.function(alloc_params, alloc_results);
            next_type_idx += 1;
        }

        // Add import function types
        for import in &mut imports {
            let results = if let Some(res) = import.result {
                vec![res]
            } else {
                vec![]
            };
            types.function(import.params.clone(), results);
            import.type_idx = next_type_idx;
            next_type_idx += 1;
        }

        // Add local function types
        let local_func_type_offset = next_type_idx;
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
            next_type_idx += 1;
        }

        // Add constructor types for each gene layout
        // Collect gene constructors for later use
        let gene_names: Vec<String> = self.gene_layouts.names();
        let constructor_type_offset = next_type_idx;
        let mut constructor_infos: Vec<(String, Vec<ValType>, Vec<ValType>)> = Vec::new();

        for gene_name in &gene_names {
            if let Some(layout) = self.gene_layouts.get(gene_name) {
                // Build parameter types from fields
                let params: Vec<ValType> = layout
                    .fields
                    .iter()
                    .map(|f| f.wasm_type.to_val_type())
                    .collect();
                let results = vec![ValType::I32]; // Return pointer
                types.function(params.clone(), results.clone());
                constructor_infos.push((gene_name.clone(), params, results));
                next_type_idx += 1;
            }
        }

        wasm_module.section(&types);

        // ============= IMPORT SECTION =============
        // Import functions come from the "loa" module (host Loa functions)
        if !imports.is_empty() {
            let mut import_section = ImportSection::new();
            for import in &imports {
                import_section.import(
                    &import.module,
                    &import.name,
                    EntityType::Function(import.type_idx),
                );
            }
            wasm_module.section(&import_section);
        }

        // ============= FUNCTION SECTION =============
        // Function indices: imports get 0..n-1, then alloc (if needed), then local functions, then constructors
        // Note: imports already occupy indices 0..import_count-1
        let import_count = imports.len() as u32;
        let alloc_func_idx = if needs_memory {
            Some(import_count) // alloc comes right after imports
        } else {
            None
        };
        let local_func_idx_offset = import_count + if needs_memory { 1 } else { 0 };
        let constructor_func_idx_offset = local_func_idx_offset + functions.len() as u32;

        let mut funcs = FunctionSection::new();
        // Add alloc function type reference
        if needs_memory {
            funcs.function(0); // alloc uses type 0
        }
        // Add local function type references
        for (i, _func) in functions.iter().enumerate() {
            funcs.function(local_func_type_offset + i as u32);
        }
        // Add constructor function type references
        for (i, _) in constructor_infos.iter().enumerate() {
            funcs.function(constructor_type_offset + i as u32);
        }
        wasm_module.section(&funcs);

        // ============= MEMORY SECTION =============
        if needs_memory {
            BumpAllocator::emit_memory_section(&mut wasm_module, 1);
            // Note: globals are emitted below in the GLOBAL SECTION, not here
            // This avoids duplicate global sections
        }

        // ============= GLOBAL SECTION =============
        // Extract sex var declarations and emit globals
        let user_globals = self.extract_sex_vars(&declarations)?;
        let has_user_globals = !user_globals.is_empty();
        let mut user_global_map: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        if needs_memory || has_user_globals {
            use wasm_encoder::{ConstExpr, GlobalSection, GlobalType};

            let mut globals = GlobalSection::new();

            // Allocator globals (index 0, 1) - only if we need memory
            if needs_memory {
                // HEAP_BASE: mutable i32, starts after static data
                globals.global(
                    GlobalType {
                        val_type: ValType::I32,
                        mutable: true,
                    },
                    &ConstExpr::i32_const(1024),
                );
                // HEAP_END: immutable i32, end of first memory page
                globals.global(
                    GlobalType {
                        val_type: ValType::I32,
                        mutable: false,
                    },
                    &ConstExpr::i32_const(65536),
                );
            }

            // User globals (sex var) start at index 2 if memory is needed, otherwise 0
            let user_global_start = if needs_memory { 2u32 } else { 0u32 };
            for (i, (name, var)) in user_globals.iter().enumerate() {
                let global_idx = user_global_start + i as u32;
                user_global_map.insert(name.clone(), global_idx);

                // Determine the WASM type and initial value from the var declaration
                let (val_type, init_expr) = self.get_global_type_and_init(var)?;
                globals.global(
                    GlobalType {
                        val_type,
                        mutable: true, // sex var is always mutable
                    },
                    &init_expr,
                );
            }

            wasm_module.section(&globals);
        }

        // ============= EXPORT SECTION =============
        let mut exports = ExportSection::new();
        // Export local functions (not imports)
        for (idx, extracted) in functions.iter().enumerate() {
            exports.export(
                &extracted.exported_name,
                ExportKind::Func,
                local_func_idx_offset + idx as u32,
            );
        }
        // Export constructor functions as new_GeneName
        for (i, (gene_name, _, _)) in constructor_infos.iter().enumerate() {
            let export_name = format!("new_{}", gene_name);
            exports.export(
                &export_name,
                ExportKind::Func,
                constructor_func_idx_offset + i as u32,
            );
        }
        if needs_memory {
            exports.export("memory", ExportKind::Memory, 0);
        }
        wasm_module.section(&exports);

        // ============= CODE SECTION =============
        let mut code = CodeSection::new();

        // Add alloc function if memory is needed
        if needs_memory {
            let alloc_func = BumpAllocator::build_alloc_function();
            code.function(&alloc_func);
        }

        // Build function name map for call resolution
        // Include both imports and local functions
        let mut func_name_map: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        // Register imports
        for (i, import) in imports.iter().enumerate() {
            func_name_map.insert(import.name.clone(), i as u32);
        }

        // Register local functions
        for (i, f) in functions.iter().enumerate() {
            let wasm_idx = local_func_idx_offset + i as u32;
            func_name_map.insert(f.exported_name.clone(), wasm_idx);
            func_name_map.insert(f.func.name.clone(), wasm_idx);
        }

        // Add local function code
        for (idx, extracted) in functions.iter().enumerate() {
            let mut locals_table = LocalsTable::new_with_gene_context(
                &extracted.func.params,
                extracted.gene_context.as_ref(),
            );

            // Track WASM types for parameters (for type-aware operations like i32 widening)
            for param in &extracted.func.params {
                if let Ok(wasm_type) = self.dol_type_to_wasm(&param.type_ann) {
                    locals_table.set_wasm_type(&param.name, wasm_type);
                }
            }

            // Register all function indices for call resolution
            for (name, wasm_idx) in &func_name_map {
                locals_table.register_function(name, *wasm_idx);
            }

            // Register all global variables (sex var) for global.get/set
            for (name, global_idx) in &user_global_map {
                locals_table.register_global(name, *global_idx);
            }

            // Collect locals from function body
            self.collect_locals(&extracted.func.body, &mut locals_table)?;

            // If we have gene layouts, add a temp local for struct pointer
            if !self.gene_layouts.is_empty() {
                locals_table.declare("__struct_ptr", ValType::I32);
            }

            // Match expressions need a temp local
            locals_table.declare("__match_temp", ValType::I64);

            // Build the locals vector
            let locals = locals_table.get_locals();

            let mut function = Function::new(locals);
            let _ = idx;
            self.emit_function_body(&mut function, extracted.func, &locals_table, &string_pool)?;
            code.function(&function);
        }

        // Add constructor function code
        if let Some(alloc_idx) = alloc_func_idx {
            for (i, (gene_name, _, _)) in constructor_infos.iter().enumerate() {
                if let Some(layout) = self.gene_layouts.get(gene_name) {
                    let (_, _, constructor_func) = Self::build_gene_constructor(layout, alloc_idx);
                    code.function(&constructor_func);

                    // Register constructor in function name map for call resolution
                    let constructor_name = format!("new_{}", gene_name);
                    func_name_map.insert(constructor_name, constructor_func_idx_offset + i as u32);
                }
            }
        }

        // Only emit code section if we have local functions or constructors
        let has_constructors = !constructor_infos.is_empty();
        if has_local_functions || needs_memory || has_constructors {
            wasm_module.section(&code);
        }

        // ============= DATA SECTION =============
        // Emit data section for string literals (if any)
        if !string_pool.is_empty() {
            use wasm_encoder::DataSection;
            let mut data = DataSection::new();
            // Active data segment at memory 0, offset 0
            data.active(
                0,
                &wasm_encoder::ConstExpr::i32_const(0),
                string_pool.get_data().iter().copied(),
            );
            wasm_module.section(&data);
        }

        Ok(wasm_module.finish())
    }

    /// Register gene layouts in dependency order (parents before children).
    ///
    /// This performs a topological sort of genes based on their extends relationships,
    /// ensuring that parent genes are registered before their children.
    fn register_gene_layouts_ordered(
        &mut self,
        declarations: &[crate::ast::Declaration],
    ) -> Result<(), WasmError> {
        use crate::ast::Declaration;
        use crate::wasm::layout::compute_gene_layout;

        // Collect all genes with their dependencies
        let mut genes: Vec<&crate::ast::Gen> = Vec::new();
        for decl in declarations {
            if let Declaration::Gene(gene) = decl {
                genes.push(gene);
            }
        }

        // Simple topological sort: keep processing until all genes are registered
        // This handles the case where parent genes come after children in the source
        let mut remaining: Vec<&crate::ast::Gen> = genes;
        let mut max_iterations = remaining.len() + 1;

        while !remaining.is_empty() && max_iterations > 0 {
            max_iterations -= 1;
            let mut still_remaining = Vec::new();

            for gene in remaining {
                // Skip if already registered
                if self.gene_layouts.contains(&gene.name) {
                    continue;
                }

                // Check if this gene has any fields
                let has_fields = gene
                    .statements
                    .iter()
                    .any(|stmt| matches!(stmt, crate::ast::Statement::HasField(_)));

                if !has_fields {
                    // No fields, skip layout registration
                    continue;
                }

                // Check if parent is registered (if any)
                if let Some(parent_name) = &gene.extends {
                    if !self.gene_layouts.contains(parent_name) {
                        // Parent not yet registered, try again later
                        still_remaining.push(gene);
                        continue;
                    }
                }

                // Compute and register layout
                match compute_gene_layout(gene, &self.gene_layouts) {
                    Ok(layout) => {
                        self.gene_layouts.register(layout);
                    }
                    Err(e) => {
                        return Err(WasmError::new(format!(
                            "Failed to compute layout for gene '{}': {}",
                            gene.name, e.message
                        )));
                    }
                }
            }

            remaining = still_remaining;
        }

        // Check if there are any remaining genes (circular dependency)
        if !remaining.is_empty() {
            let names: Vec<&str> = remaining.iter().map(|g| g.name.as_str()).collect();
            return Err(WasmError::new(format!(
                "Circular or unresolved gene dependencies: {:?}",
                names
            )));
        }

        Ok(())
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
                // Start with parent fields if this gene extends another
                let mut field_names: Vec<String> = Vec::new();

                if let Some(parent_name) = &gene.extends {
                    // Get parent layout to get inherited field names
                    if let Some(parent_layout) = self.gene_layouts.get(parent_name) {
                        for parent_field in &parent_layout.fields {
                            field_names.push(parent_field.name.clone());
                        }
                    }
                }

                // Add this gene's own fields
                for stmt in &gene.statements {
                    if let Statement::HasField(field) = stmt {
                        field_names.push(field.name.clone());
                    }
                }

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
            Declaration::Const(_) => {
                // Constants don't contain functions, return empty
                Ok(vec![])
            }
            _ => {
                // For now, we only support direct function declarations and genes
                // Future: extract functions from Trait/System bodies
                Ok(vec![])
            }
        }
    }

    /// Extract imports from DOL declarations.
    ///
    /// A sex fun without body is treated as a WASM import.
    /// All imports use the "loa" module name (the Loa host function registry).
    fn extract_imports(&self, declarations: &[Declaration]) -> Result<Vec<WasmImport>, WasmError> {
        use crate::ast::{Declaration, Purity};

        let mut imports = Vec::new();

        for decl in declarations {
            if let Declaration::Function(func) = decl {
                // A sex fun without body is an import
                if func.purity == Purity::Sex && func.body.is_empty() {
                    let params: Vec<wasm_encoder::ValType> = func
                        .params
                        .iter()
                        .map(|p| self.dol_type_to_wasm(&p.type_ann))
                        .collect::<Result<Vec<_>, _>>()?;

                    let result = if let Some(ref ret_type) = func.return_type {
                        Some(self.dol_type_to_wasm(ret_type)?)
                    } else {
                        None
                    };

                    imports.push(WasmImport {
                        module: "loa".to_string(),
                        name: func.name.clone(),
                        params,
                        result,
                        type_idx: 0, // Will be set later
                    });
                }
            }
        }

        Ok(imports)
    }

    /// Check if a function is an import (sex fun without body).
    fn is_import(func: &crate::ast::FunctionDecl) -> bool {
        use crate::ast::Purity;
        func.purity == Purity::Sex && func.body.is_empty()
    }

    /// Extract sex var declarations from a list of declarations.
    ///
    /// Returns a vector of (name, VarDecl) pairs for all SexVar declarations.
    fn extract_sex_vars<'a>(
        &self,
        declarations: &'a [Declaration],
    ) -> Result<Vec<(String, &'a crate::ast::VarDecl)>, WasmError> {
        let mut sex_vars = Vec::new();

        for decl in declarations {
            if let Declaration::SexVar(var) = decl {
                sex_vars.push((var.name.clone(), var));
            }
        }

        Ok(sex_vars)
    }

    /// Extract const declarations from DOL modules.
    ///
    /// Returns a vector of (name, ConstDecl) pairs for all Const declarations.
    #[allow(dead_code)]
    fn extract_consts<'a>(
        &self,
        declarations: &'a [Declaration],
    ) -> Result<Vec<(String, &'a crate::ast::ConstDecl)>, WasmError> {
        let mut consts = Vec::new();

        for decl in declarations {
            if let Declaration::Const(const_decl) = decl {
                consts.push((const_decl.name.clone(), const_decl));
            }
        }

        Ok(consts)
    }

    /// Get the WASM type and initialization expression for a const declaration.
    #[allow(dead_code)]
    fn get_const_type_and_init(
        &self,
        const_decl: &crate::ast::ConstDecl,
    ) -> Result<(wasm_encoder::ValType, wasm_encoder::ConstExpr), WasmError> {
        use wasm_encoder::{ConstExpr, ValType};

        // Determine the type from the type annotation or infer from value
        let val_type = if let Some(ref type_ann) = const_decl.type_ann {
            self.dol_type_to_wasm(type_ann)?
        } else {
            // Infer from value
            match &const_decl.value {
                crate::ast::Expr::Literal(lit) => match lit {
                    crate::ast::Literal::Int(_) => ValType::I64,
                    crate::ast::Literal::Float(_) => ValType::F64,
                    crate::ast::Literal::Bool(_) => ValType::I32,
                    _ => return Err(WasmError::new("Cannot infer type for const")),
                },
                _ => {
                    return Err(WasmError::new(
                        "Cannot infer type for complex const expression",
                    ))
                }
            }
        };

        // Determine the initial value
        let init_expr = match &const_decl.value {
            crate::ast::Expr::Literal(lit) => match lit {
                crate::ast::Literal::Int(i) => match val_type {
                    ValType::I32 => ConstExpr::i32_const(*i as i32),
                    ValType::I64 => ConstExpr::i64_const(*i),
                    ValType::F32 => ConstExpr::f32_const(*i as f32),
                    ValType::F64 => ConstExpr::f64_const(*i as f64),
                    _ => return Err(WasmError::new("Unsupported const type")),
                },
                crate::ast::Literal::Float(f) => match val_type {
                    ValType::F32 => ConstExpr::f32_const(*f as f32),
                    ValType::F64 => ConstExpr::f64_const(*f),
                    ValType::I32 => ConstExpr::i32_const(*f as i32),
                    ValType::I64 => ConstExpr::i64_const(*f as i64),
                    _ => return Err(WasmError::new("Unsupported const type")),
                },
                crate::ast::Literal::Bool(b) => ConstExpr::i32_const(if *b { 1 } else { 0 }),
                _ => return Err(WasmError::new("Unsupported literal for const")),
            },
            _ => {
                // For complex expressions, use a default value
                // In a full implementation, we'd evaluate the const expression at compile time
                match val_type {
                    ValType::I32 => ConstExpr::i32_const(0),
                    ValType::I64 => ConstExpr::i64_const(0),
                    ValType::F32 => ConstExpr::f32_const(0.0),
                    ValType::F64 => ConstExpr::f64_const(0.0),
                    _ => return Err(WasmError::new("Unsupported const type")),
                }
            }
        };

        Ok((val_type, init_expr))
    }

    /// Get the WASM type and initialization expression for a sex var.
    fn get_global_type_and_init(
        &self,
        var: &crate::ast::VarDecl,
    ) -> Result<(wasm_encoder::ValType, wasm_encoder::ConstExpr), WasmError> {
        use wasm_encoder::{ConstExpr, ValType};

        // Determine the type from the type annotation or default to i64
        let val_type = if let Some(ref type_ann) = var.type_ann {
            self.dol_type_to_wasm(type_ann)?
        } else {
            ValType::I64 // Default to i64
        };

        // Determine the initial value
        let init_expr = if let Some(ref value) = var.value {
            match value {
                crate::ast::Expr::Literal(lit) => match lit {
                    crate::ast::Literal::Int(i) => match val_type {
                        ValType::I32 => ConstExpr::i32_const(*i as i32),
                        ValType::I64 => ConstExpr::i64_const(*i),
                        ValType::F32 => ConstExpr::f32_const(*i as f32),
                        ValType::F64 => ConstExpr::f64_const(*i as f64),
                        _ => return Err(WasmError::new("Unsupported global type")),
                    },
                    crate::ast::Literal::Float(f) => match val_type {
                        ValType::F32 => ConstExpr::f32_const(*f as f32),
                        ValType::F64 => ConstExpr::f64_const(*f),
                        ValType::I32 => ConstExpr::i32_const(*f as i32),
                        ValType::I64 => ConstExpr::i64_const(*f as i64),
                        _ => return Err(WasmError::new("Unsupported global type")),
                    },
                    crate::ast::Literal::Bool(b) => ConstExpr::i32_const(if *b { 1 } else { 0 }),
                    _ => return Err(WasmError::new("Unsupported literal for global")),
                },
                _ => {
                    return Err(WasmError::new(
                        "Global variable initializer must be a constant literal",
                    ))
                }
            }
        } else {
            // Default initialization to zero
            match val_type {
                ValType::I32 => ConstExpr::i32_const(0),
                ValType::I64 => ConstExpr::i64_const(0),
                ValType::F32 => ConstExpr::f32_const(0.0),
                ValType::F64 => ConstExpr::f64_const(0.0),
                _ => return Err(WasmError::new("Unsupported global type")),
            }
        };

        Ok((val_type, init_expr))
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
            TypeExpr::Named(name) => {
                // First check primitive types (lowercase comparison)
                match name.to_lowercase().as_str() {
                    "i32" | "i64" | "int" | "int32" | "int64" => Ok(ValType::I64),
                    "f32" | "f64" | "float" | "float32" | "float64" => Ok(ValType::F64),
                    "bool" | "boolean" => Ok(ValType::I32),
                    // String is represented as a single i32 pointer to length-prefixed data
                    "string" | "str" => Ok(ValType::I32),
                    _ => {
                        // Check if it's a registered enum type
                        if self.enum_registry.contains(name) {
                            // Enum types are represented as i32 discriminants
                            Ok(ValType::I32)
                        } else {
                            Err(WasmError::new(format!(
                                "Unsupported type for WASM compilation: {}",
                                name
                            )))
                        }
                    }
                }
            }
            TypeExpr::Generic { name, args: _ } => {
                // Handle collection types as i32 pointers
                match name.as_str() {
                    "List" | "Vec" | "Option" | "Map" | "HashMap" | "Result" => Ok(ValType::I32),
                    // Reference types
                    _ if name.starts_with('&') => Ok(ValType::I32),
                    _ => Err(WasmError::new(format!(
                        "Unsupported generic type for WASM compilation: {}",
                        name
                    ))),
                }
            }
            TypeExpr::Function { .. } => Err(WasmError::new(
                "Function types not yet supported in WASM compilation",
            )),
            TypeExpr::Tuple(_) => Err(WasmError::new(
                "Tuple types not yet supported in WASM compilation",
            )),
            TypeExpr::Enum { .. } => {
                // Inline enum types are represented as i32 discriminants
                Ok(ValType::I32)
            }
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
            Expr::Block(crate::ast::Block {
                statements,
                final_expr,
                ..
            }) => {
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

    /// Collect all string literals from a declaration into a StringPool.
    fn collect_strings_from_declaration(&self, decl: &Declaration, pool: &mut StringPool) {
        use crate::ast::{Declaration, Statement};

        match decl {
            Declaration::Function(func) => {
                for stmt in &func.body {
                    self.collect_strings_from_stmt(stmt, pool);
                }
            }
            Declaration::Gene(gene) => {
                for stmt in &gene.statements {
                    if let Statement::Function(func) = stmt {
                        for body_stmt in &func.body {
                            self.collect_strings_from_stmt(body_stmt, pool);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Collect string literals from a statement.
    fn collect_strings_from_stmt(&self, stmt: &crate::ast::Stmt, pool: &mut StringPool) {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Expr(expr) | Stmt::Return(Some(expr)) => {
                self.collect_strings_from_expr(expr, pool);
            }
            Stmt::Let { value, .. } => {
                self.collect_strings_from_expr(value, pool);
            }
            Stmt::Assign { value, .. } => {
                self.collect_strings_from_expr(value, pool);
            }
            Stmt::While { condition, body } => {
                self.collect_strings_from_expr(condition, pool);
                for s in body {
                    self.collect_strings_from_stmt(s, pool);
                }
            }
            Stmt::For { iterable, body, .. } => {
                self.collect_strings_from_expr(iterable, pool);
                for s in body {
                    self.collect_strings_from_stmt(s, pool);
                }
            }
            Stmt::Loop { body } => {
                for s in body {
                    self.collect_strings_from_stmt(s, pool);
                }
            }
            _ => {}
        }
    }

    /// Collect string literals from an expression.
    fn collect_strings_from_expr(&self, expr: &crate::ast::Expr, pool: &mut StringPool) {
        use crate::ast::{Expr, Literal};

        match expr {
            Expr::Literal(Literal::String(s)) => {
                pool.add(s);
            }
            Expr::Binary { left, right, .. } => {
                self.collect_strings_from_expr(left, pool);
                self.collect_strings_from_expr(right, pool);
            }
            Expr::Unary { operand, .. } => {
                self.collect_strings_from_expr(operand, pool);
            }
            Expr::Call { callee, args } => {
                self.collect_strings_from_expr(callee, pool);
                for arg in args {
                    self.collect_strings_from_expr(arg, pool);
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_strings_from_expr(condition, pool);
                self.collect_strings_from_expr(then_branch, pool);
                if let Some(e) = else_branch {
                    self.collect_strings_from_expr(e, pool);
                }
            }
            Expr::Match { scrutinee, arms } => {
                self.collect_strings_from_expr(scrutinee, pool);
                for arm in arms {
                    self.collect_strings_from_expr(&arm.body, pool);
                }
            }
            Expr::Block(crate::ast::Block {
                statements,
                final_expr,
                ..
            }) => {
                for stmt in statements {
                    self.collect_strings_from_stmt(stmt, pool);
                }
                if let Some(e) = final_expr {
                    self.collect_strings_from_expr(e, pool);
                }
            }
            Expr::StructLiteral { fields, .. } => {
                for (_name, value) in fields {
                    self.collect_strings_from_expr(value, pool);
                }
            }
            Expr::List(items) | Expr::Tuple(items) => {
                for item in items {
                    self.collect_strings_from_expr(item, pool);
                }
            }
            Expr::Member { object, .. } => {
                self.collect_strings_from_expr(object, pool);
            }
            Expr::Lambda { body, .. } => {
                self.collect_strings_from_expr(body, pool);
            }
            _ => {}
        }
    }

    /// Emit the body of a function as WASM instructions.
    fn emit_function_body(
        &self,
        function: &mut wasm_encoder::Function,
        func_decl: &crate::ast::FunctionDecl,
        locals: &LocalsTable,
        string_pool: &StringPool,
    ) -> Result<(), WasmError> {
        use crate::ast::Stmt;
        use wasm_encoder::Instruction;

        let has_return_type = func_decl.return_type.is_some();
        let stmt_count = func_decl.body.len();

        // Emit each statement in the function body
        // Start with empty loop context (no break/continue targets)
        let loop_ctx = LoopContext::default();

        for (i, stmt) in func_decl.body.iter().enumerate() {
            let is_last = i == stmt_count - 1;

            // Special handling for last expression statement in functions with return types
            if is_last && has_return_type {
                if let Stmt::Expr(expr) = stmt {
                    // Emit the expression without dropping - its value becomes the return
                    self.emit_expression(function, expr, locals, loop_ctx, string_pool)?;
                    // No Drop - the value on stack is the return value
                } else if let Stmt::Return(Some(expr)) = stmt {
                    // Explicit return - emit expression and return
                    self.emit_expression(function, expr, locals, loop_ctx, string_pool)?;
                    function.instruction(&Instruction::Return);
                } else if let Stmt::Return(None) = stmt {
                    // Explicit void return
                    function.instruction(&Instruction::Return);
                } else {
                    // Other statement types - emit normally
                    self.emit_statement(function, stmt, locals, loop_ctx, string_pool)?;
                }
            } else {
                // Not the last statement - emit normally
                self.emit_statement(function, stmt, locals, loop_ctx, string_pool)?;
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
        loop_ctx: LoopContext,
        string_pool: &StringPool,
    ) -> Result<(), WasmError> {
        use crate::ast::{Expr, Stmt};
        use wasm_encoder::Instruction;

        match stmt {
            Stmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.emit_expression(function, expr, locals, loop_ctx, string_pool)?;
                }
                function.instruction(&Instruction::Return);
            }
            Stmt::Expr(expr) => {
                self.emit_expression(function, expr, locals, loop_ctx, string_pool)?;
                // Drop the result if it's an expression statement that produces a value
                // Note: Some expressions like if-without-else produce no value
                if self.expression_produces_value(expr) {
                    function.instruction(&Instruction::Drop);
                }
            }
            Stmt::Let { name, value, .. } => {
                // Emit the value expression
                self.emit_expression(function, value, locals, loop_ctx, string_pool)?;

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
                        self.emit_expression(function, value, locals, loop_ctx, string_pool)?;

                        // Check if it's a global variable (sex var) first
                        if let Some(global_idx) = locals.lookup_global(name) {
                            // Store to global variable
                            function.instruction(&Instruction::GlobalSet(global_idx));
                        } else {
                            // Look up the local index
                            let local_idx = locals.lookup(name).ok_or_else(|| {
                                WasmError::new(format!(
                                    "Cannot assign to unknown variable: {}",
                                    name
                                ))
                            })?;

                            // Store the value in the local
                            function.instruction(&Instruction::LocalSet(local_idx));
                        }
                    }
                    Expr::Member { object, field } => {
                        // Field assignment: object.field = value
                        // WASM store expects [address, value] on stack

                        // Infer gene type from object expression
                        let gene_type = match object.as_ref() {
                            Expr::Identifier(var_name) => {
                                // Look up DOL type from locals table (works for 'self' and other vars)
                                locals.lookup_dol_type(var_name).map(|s| s.to_string())
                            }
                            _ => None,
                        };

                        // Emit object expression (pushes pointer onto stack)
                        self.emit_expression(function, object, locals, loop_ctx, string_pool)?;

                        // Emit value expression (pushes value onto stack)
                        self.emit_expression(function, value, locals, loop_ctx, string_pool)?;

                        // Look up gene layout and emit store instruction
                        if let Some(type_name) = gene_type {
                            if let Some(layout) = self.gene_layouts.get(&type_name) {
                                if let Some(field_layout) = layout.get_field(field) {
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
                                        field, type_name
                                    )));
                                }
                            } else {
                                return Err(WasmError::new(format!(
                                    "Gene type '{}' not registered for field assignment",
                                    type_name
                                )));
                            }
                        } else {
                            return Err(WasmError::new(format!(
                                "Cannot determine gene type for field assignment to '{}'",
                                field
                            )));
                        }
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

                // Create loop context for body statements
                let body_ctx = LoopContext::enter_loop();

                // Evaluate condition (in parent context, before loop constructs)
                self.emit_expression(function, condition, locals, loop_ctx, string_pool)?;

                // Branch out of outer block if condition is false (i32.eqz inverts boolean)
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1)); // Break to outer block

                // Loop body with loop context
                for stmt in body {
                    self.emit_statement(function, stmt, locals, body_ctx, string_pool)?;
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

                    // Initialize loop variable with start value (in parent context)
                    self.emit_expression(function, left, locals, loop_ctx, string_pool)?;
                    function.instruction(&Instruction::LocalSet(loop_var));

                    // Store end value (in parent context)
                    self.emit_expression(function, right, locals, loop_ctx, string_pool)?;
                    function.instruction(&Instruction::LocalSet(end_var));

                    // Outer block for break
                    function.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));

                    // Loop
                    function.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

                    // Create loop context for body statements
                    let body_ctx = LoopContext::enter_loop();

                    // Check condition: loop_var < end_var
                    function.instruction(&Instruction::LocalGet(loop_var));
                    function.instruction(&Instruction::LocalGet(end_var));
                    function.instruction(&Instruction::I64LtS);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::BrIf(1)); // Break if not less

                    // Body with loop context
                    for stmt in body {
                        self.emit_statement(function, stmt, locals, body_ctx, string_pool)?;
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

                // Create loop context for body statements
                let body_ctx = LoopContext::enter_loop();

                // Loop body with loop context
                for stmt in body {
                    self.emit_statement(function, stmt, locals, body_ctx, string_pool)?;
                }

                // Continue - infinite loop back to start
                function.instruction(&Instruction::Br(0));

                function.instruction(&Instruction::End); // End loop
                function.instruction(&Instruction::End); // End block
            }
            Stmt::Break => {
                // Break to outer block using tracked depth
                let depth = loop_ctx
                    .break_depth
                    .ok_or_else(|| WasmError::new("break statement outside of loop"))?;
                function.instruction(&Instruction::Br(depth));
            }
            Stmt::Continue => {
                // Continue to loop start using tracked depth
                let depth = loop_ctx
                    .continue_depth
                    .ok_or_else(|| WasmError::new("continue statement outside of loop"))?;
                function.instruction(&Instruction::Br(depth));
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
        loop_ctx: LoopContext,
        string_pool: &StringPool,
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
                Literal::String(s) => {
                    // String literals are stored in the data section.
                    // We look up the offset from the pre-collected string pool
                    // and emit an i32.const with the pointer to the length-prefixed string.
                    let offset =
                        string_pool
                            .strings
                            .get(s)
                            .map(|(off, _)| *off)
                            .ok_or_else(|| {
                                WasmError::new(format!(
                                    "Internal error: string literal '{}' not found in string pool",
                                    s
                                ))
                            })?;
                    function.instruction(&Instruction::I32Const(offset as i32));
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

                    // First check if this is an enum variant access (e.g., AccountType.Node)
                    if let Some(variant_index) = self
                        .enum_registry
                        .get_variant_index(object_name, field_name)
                    {
                        // Emit the enum discriminant as i32 (enum types are always i32)
                        function.instruction(&Instruction::I32Const(variant_index));
                        return Ok(());
                    }

                    // Look up the object variable (fall back to struct field access)
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
                } else if let Some(global_idx) = locals.lookup_global(name) {
                    // Global variable (sex var) - emit global.get
                    function.instruction(&Instruction::GlobalGet(global_idx));
                } else {
                    // Simple identifier - look up in locals table
                    let local_idx = locals
                        .lookup(name)
                        .ok_or_else(|| WasmError::new(format!("Unknown identifier: {}", name)))?;
                    function.instruction(&Instruction::LocalGet(local_idx));

                    // If this is an i32 parameter (e.g., enum type), widen to i64
                    // for compatibility with i64-based operations
                    if locals.needs_widening(name) {
                        function.instruction(&Instruction::I64ExtendI32S);
                    }
                }
            }
            Expr::Binary { left, op, right } => {
                // Infer the operand type for correct instruction selection
                let operand_type = self.infer_expression_type(left, locals);
                // Emit left operand
                self.emit_expression(function, left, locals, loop_ctx, string_pool)?;
                // Emit right operand
                self.emit_expression(function, right, locals, loop_ctx, string_pool)?;
                // Emit operation
                self.emit_binary_op(function, *op, operand_type)?;
            }
            Expr::Call { callee, args } => {
                match callee.as_ref() {
                    // Direct function call: func(args)
                    Expr::Identifier(func_name) => {
                        // Emit arguments
                        for arg in args {
                            self.emit_expression(function, arg, locals, loop_ctx, string_pool)?;
                        }
                        // Look up function index
                        let func_idx = locals.lookup_function(func_name).ok_or_else(|| {
                            WasmError::new(format!("Unknown function: {}", func_name))
                        })?;
                        function.instruction(&Instruction::Call(func_idx));
                    }
                    // Method call: object.method(args)
                    Expr::Member { object, field } => {
                        // Emit the object (receiver) first
                        self.emit_expression(function, object, locals, loop_ctx, string_pool)?;

                        // Handle collection/iterator methods as passthrough for now
                        // These methods return the object pointer unchanged (lazy evaluation)
                        match field.as_str() {
                            // Iterator creation methods - return the collection pointer
                            "iter" | "into_iter" | "iter_mut" => {
                                // Collection is already on stack, no-op
                            }
                            // Transformation methods - return an iterator/collection pointer
                            "map" | "filter" | "filter_map" | "flat_map" | "take" | "skip"
                            | "enumerate" | "zip" | "chain" | "rev" => {
                                // Emit any closure/function arguments
                                for arg in args {
                                    self.emit_expression(
                                        function,
                                        arg,
                                        locals,
                                        loop_ctx,
                                        string_pool,
                                    )?;
                                    // Drop the closure arg for now (lazy evaluation)
                                    function.instruction(&Instruction::Drop);
                                }
                                // Keep the collection pointer on stack
                            }
                            // Terminal methods - consume iterator and produce result
                            "collect" | "sum" | "product" | "count" | "last" | "first" => {
                                // For now, just return the pointer
                                // TODO: Implement actual collection materialization
                            }
                            // Option methods
                            "unwrap" | "unwrap_or" | "unwrap_or_else" | "expect" => {
                                // For unwrap, just return the inner value pointer
                                // TODO: Implement proper unwrapping with panic handling
                            }
                            "is_some" | "is_none" => {
                                // Check if pointer is null
                                function.instruction(&Instruction::I32Const(0));
                                if field == "is_none" {
                                    function.instruction(&Instruction::I32Eq);
                                } else {
                                    function.instruction(&Instruction::I32Ne);
                                }
                            }
                            // List/Vec methods
                            "len" | "length" | "size" => {
                                // Load length from first word of collection struct
                                function.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }));
                                // Widen to i64 for consistency
                                function.instruction(&Instruction::I64ExtendI32U);
                            }
                            "is_empty" => {
                                // Check if length is 0
                                function.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }));
                                function.instruction(&Instruction::I32Const(0));
                                function.instruction(&Instruction::I32Eq);
                            }
                            "push" | "pop" | "insert" | "remove" | "clear" => {
                                // Mutation methods - emit args and drop for now
                                for arg in args {
                                    self.emit_expression(
                                        function,
                                        arg,
                                        locals,
                                        loop_ctx,
                                        string_pool,
                                    )?;
                                    function.instruction(&Instruction::Drop);
                                }
                            }
                            "get" => {
                                // Get element at index - emit index arg
                                if let Some(idx_arg) = args.first() {
                                    self.emit_expression(
                                        function,
                                        idx_arg,
                                        locals,
                                        loop_ctx,
                                        string_pool,
                                    )?;
                                    // TODO: Implement proper array indexing
                                    function.instruction(&Instruction::Drop);
                                }
                            }
                            // Map methods
                            "keys" | "values" | "entries" | "contains_key" => {
                                // For now, return the map pointer
                                for arg in args {
                                    self.emit_expression(
                                        function,
                                        arg,
                                        locals,
                                        loop_ctx,
                                        string_pool,
                                    )?;
                                    function.instruction(&Instruction::Drop);
                                }
                            }
                            // Clone/Copy
                            "clone" | "copy" => {
                                // For now, just return the same pointer
                            }
                            // Default: Try as gene method call
                            _ => {
                                // Emit arguments (self is already on stack)
                                for arg in args {
                                    self.emit_expression(
                                        function,
                                        arg,
                                        locals,
                                        loop_ctx,
                                        string_pool,
                                    )?;
                                }
                                // Look up method as function
                                let func_idx = locals.lookup_function(field).ok_or_else(|| {
                                    WasmError::new(format!("Unknown method: {}", field))
                                })?;
                                function.instruction(&Instruction::Call(func_idx));
                            }
                        }
                    }
                    _ => {
                        return Err(WasmError::new(
                            "Only direct function calls and method calls are supported in WASM compilation",
                        ));
                    }
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Emit condition (should produce i32 boolean value)
                self.emit_expression(function, condition, locals, loop_ctx, string_pool)?;

                // Determine result type based on whether we have an else branch
                // AND whether the branches actually produce values
                let block_type =
                    if else_branch.is_some() && self.expression_produces_value(then_branch) {
                        // Both branches exist and produce a value - infer the type
                        let result_type = self.infer_expression_type(then_branch, locals);
                        wasm_encoder::BlockType::Result(result_type)
                    } else {
                        // No else branch or branches don't produce values (e.g., assignments)
                        wasm_encoder::BlockType::Empty
                    };

                function.instruction(&Instruction::If(block_type));

                // If block adds a level of nesting, so increment depths for break/continue
                let if_ctx = loop_ctx.enter_block();

                // Emit then branch with updated context
                self.emit_expression(function, then_branch, locals, if_ctx, string_pool)?;

                // Emit else branch if present
                if let Some(else_expr) = else_branch {
                    function.instruction(&Instruction::Else);
                    self.emit_expression(function, else_expr, locals, if_ctx, string_pool)?;
                }

                function.instruction(&Instruction::End);
            }
            Expr::Block(crate::ast::Block {
                statements,
                final_expr,
                ..
            }) => {
                // Block expressions might contain statements with break/continue
                // Note: A pure block expression doesn't add WASM block structure,
                // so we don't increment the loop context here
                for stmt in statements {
                    self.emit_statement(function, stmt, locals, loop_ctx, string_pool)?;
                }

                // Emit final expression if present (this becomes the block's value)
                if let Some(expr) = final_expr {
                    self.emit_expression(function, expr, locals, loop_ctx, string_pool)?;
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
                self.emit_expression(function, scrutinee, locals, loop_ctx, string_pool)?;

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
                self.emit_match_arms(
                    function,
                    arms,
                    0,
                    temp_local,
                    wildcard_idx,
                    locals,
                    loop_ctx,
                    string_pool,
                )?;
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
                self.emit_expression(function, object, locals, loop_ctx, string_pool)?;

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
                        self.emit_expression(function, operand, locals, loop_ctx, string_pool)?;
                        function.instruction(&Instruction::I64Sub);
                    }
                    UnaryOp::Not => {
                        // For boolean not: eqz (value == 0)
                        self.emit_expression(function, operand, locals, loop_ctx, string_pool)?;
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
            Expr::This => {
                return Err(WasmError::new(
                    "'this' self-reference not yet supported in WASM compilation",
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
                            self.emit_expression(
                                function,
                                field_value,
                                locals,
                                loop_ctx,
                                string_pool,
                            )?;
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

                    // Pointer is already on stack from LocalTee above - no need for additional LocalGet
                    // The LocalTee instruction both stores to local AND leaves the value on the stack
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
    #[allow(clippy::too_many_arguments)]
    fn emit_match_arms(
        &self,
        function: &mut wasm_encoder::Function,
        arms: &[crate::ast::MatchArm],
        current_idx: usize,
        temp_local: u32,
        wildcard_idx: Option<usize>,
        locals: &LocalsTable,
        loop_ctx: LoopContext,
        string_pool: &StringPool,
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
                    loop_ctx,
                    string_pool,
                );
            } else {
                // This is the last arm and it's wildcard - emit its body directly
                self.emit_expression(function, &arm.body, locals, loop_ctx, string_pool)?;
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

                // Match arms' if blocks add nesting level
                let if_ctx = loop_ctx.enter_block();

                // Emit the body for this arm
                self.emit_expression(function, &arm.body, locals, if_ctx, string_pool)?;

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
                        if_ctx,
                        string_pool,
                    )?;
                } else if let Some(wild_idx) = wildcard_idx {
                    // Emit wildcard body
                    self.emit_expression(
                        function,
                        &arms[wild_idx].body,
                        locals,
                        if_ctx,
                        string_pool,
                    )?;
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
                self.emit_expression(function, &arm.body, locals, loop_ctx, string_pool)?;
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

    /// Infer the type of an expression based on the locals table.
    ///
    /// Note: For identifiers that will be widened (e.g., i32 enum params),
    /// this returns the widened type (i64) to match the actual stack type
    /// after emit_expression.
    #[allow(clippy::only_used_in_recursion)]
    fn infer_expression_type(
        &self,
        expr: &crate::ast::Expr,
        locals: &LocalsTable,
    ) -> wasm_encoder::ValType {
        use crate::ast::Expr;
        use wasm_encoder::ValType;

        match expr {
            Expr::Literal(lit) => match lit {
                crate::ast::Literal::Int(_) => ValType::I64,
                crate::ast::Literal::Float(_) => ValType::F64,
                crate::ast::Literal::Bool(_) => ValType::I32,
                crate::ast::Literal::String(_) => ValType::I32, // pointer
                _ => ValType::I64,                              // default
            },
            Expr::Identifier(name) => {
                // Check if this is a dotted identifier (e.g., "p.x") which represents
                // a member access. The lexer produces these as single tokens.
                if let Some(dot_pos) = name.find('.') {
                    let object_name = &name[..dot_pos];
                    let field_name = &name[dot_pos + 1..];

                    // Check if object is an enum type (e.g., AccountType.Node)
                    // Enum variant access always returns i32
                    if self
                        .enum_registry
                        .get_variant_index(object_name, field_name)
                        .is_some()
                    {
                        return ValType::I32;
                    }

                    // Get the gene type for this variable and look up field type
                    if let Some(gene_name) = locals.lookup_dol_type(object_name) {
                        if let Some(layout) = self.gene_layouts.get(gene_name) {
                            if let Some(field_layout) = layout.get_field(field_name) {
                                return field_layout.wasm_type.to_val_type();
                            }
                        }
                    }
                    ValType::I64 // default for member access we can't resolve
                } else if locals.needs_widening(name) {
                    // Check if this identifier will be widened during emit_expression
                    // If so, return i64 to match the actual stack type after widening
                    ValType::I64
                } else {
                    locals.lookup_var_type(name).unwrap_or(ValType::I64)
                }
            }
            Expr::Binary { left, op, .. } => {
                // For comparison ops, result is always i32 (boolean)
                use crate::ast::BinaryOp;
                match op {
                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => ValType::I32,
                    _ => self.infer_expression_type(left, locals),
                }
            }
            Expr::Call { callee, .. } => {
                if let Expr::Identifier(func_name) = callee.as_ref() {
                    locals
                        .lookup_function_return_type(func_name)
                        .unwrap_or(ValType::I64)
                } else {
                    ValType::I64
                }
            }
            Expr::Member { object, field } => {
                // For member access like p.x, infer type from gene field layout
                if let Expr::Identifier(obj_name) = object.as_ref() {
                    // Get the gene type for this variable
                    if let Some(gene_name) = locals.lookup_dol_type(obj_name) {
                        // Look up field type from gene layout
                        if let Some(layout) = self.gene_layouts.get(gene_name) {
                            if let Some(field_layout) = layout.get_field(field) {
                                return field_layout.wasm_type.to_val_type();
                            }
                        }
                    }
                }
                ValType::I64 // default for member access we can't resolve
            }
            _ => ValType::I64, // default
        }
    }

    /// Emit a binary operation as a WASM instruction.
    fn emit_binary_op(
        &self,
        function: &mut wasm_encoder::Function,
        op: crate::ast::BinaryOp,
        val_type: wasm_encoder::ValType,
    ) -> Result<(), WasmError> {
        use crate::ast::BinaryOp;
        use wasm_encoder::Instruction;
        use wasm_encoder::ValType;

        match (op, val_type) {
            // Arithmetic operations
            (BinaryOp::Add, ValType::I64) => {
                function.instruction(&Instruction::I64Add);
            }
            (BinaryOp::Add, ValType::I32) => {
                function.instruction(&Instruction::I32Add);
            }
            (BinaryOp::Add, ValType::F64) => {
                function.instruction(&Instruction::F64Add);
            }
            (BinaryOp::Add, ValType::F32) => {
                function.instruction(&Instruction::F32Add);
            }

            (BinaryOp::Sub, ValType::I64) => {
                function.instruction(&Instruction::I64Sub);
            }
            (BinaryOp::Sub, ValType::I32) => {
                function.instruction(&Instruction::I32Sub);
            }
            (BinaryOp::Sub, ValType::F64) => {
                function.instruction(&Instruction::F64Sub);
            }
            (BinaryOp::Sub, ValType::F32) => {
                function.instruction(&Instruction::F32Sub);
            }

            (BinaryOp::Mul, ValType::I64) => {
                function.instruction(&Instruction::I64Mul);
            }
            (BinaryOp::Mul, ValType::I32) => {
                function.instruction(&Instruction::I32Mul);
            }
            (BinaryOp::Mul, ValType::F64) => {
                function.instruction(&Instruction::F64Mul);
            }
            (BinaryOp::Mul, ValType::F32) => {
                function.instruction(&Instruction::F32Mul);
            }

            (BinaryOp::Div, ValType::I64) => {
                function.instruction(&Instruction::I64DivS);
            }
            (BinaryOp::Div, ValType::I32) => {
                function.instruction(&Instruction::I32DivS);
            }
            (BinaryOp::Div, ValType::F64) => {
                function.instruction(&Instruction::F64Div);
            }
            (BinaryOp::Div, ValType::F32) => {
                function.instruction(&Instruction::F32Div);
            }

            (BinaryOp::Mod, ValType::I64) => {
                function.instruction(&Instruction::I64RemS);
            }
            (BinaryOp::Mod, ValType::I32) => {
                function.instruction(&Instruction::I32RemS);
            }
            (BinaryOp::Mod, ValType::F64 | ValType::F32) => {
                return Err(WasmError::new(
                    "Modulo not supported for floating point types",
                ))
            }

            // Comparison operations - result type is always i32
            (BinaryOp::Eq, ValType::I64) => {
                function.instruction(&Instruction::I64Eq);
            }
            (BinaryOp::Eq, ValType::I32) => {
                function.instruction(&Instruction::I32Eq);
            }
            (BinaryOp::Eq, ValType::F64) => {
                function.instruction(&Instruction::F64Eq);
            }
            (BinaryOp::Eq, ValType::F32) => {
                function.instruction(&Instruction::F32Eq);
            }

            (BinaryOp::Ne, ValType::I64) => {
                function.instruction(&Instruction::I64Ne);
            }
            (BinaryOp::Ne, ValType::I32) => {
                function.instruction(&Instruction::I32Ne);
            }
            (BinaryOp::Ne, ValType::F64) => {
                function.instruction(&Instruction::F64Ne);
            }
            (BinaryOp::Ne, ValType::F32) => {
                function.instruction(&Instruction::F32Ne);
            }

            (BinaryOp::Lt, ValType::I64) => {
                function.instruction(&Instruction::I64LtS);
            }
            (BinaryOp::Lt, ValType::I32) => {
                function.instruction(&Instruction::I32LtS);
            }
            (BinaryOp::Lt, ValType::F64) => {
                function.instruction(&Instruction::F64Lt);
            }
            (BinaryOp::Lt, ValType::F32) => {
                function.instruction(&Instruction::F32Lt);
            }

            (BinaryOp::Le, ValType::I64) => {
                function.instruction(&Instruction::I64LeS);
            }
            (BinaryOp::Le, ValType::I32) => {
                function.instruction(&Instruction::I32LeS);
            }
            (BinaryOp::Le, ValType::F64) => {
                function.instruction(&Instruction::F64Le);
            }
            (BinaryOp::Le, ValType::F32) => {
                function.instruction(&Instruction::F32Le);
            }

            (BinaryOp::Gt, ValType::I64) => {
                function.instruction(&Instruction::I64GtS);
            }
            (BinaryOp::Gt, ValType::I32) => {
                function.instruction(&Instruction::I32GtS);
            }
            (BinaryOp::Gt, ValType::F64) => {
                function.instruction(&Instruction::F64Gt);
            }
            (BinaryOp::Gt, ValType::F32) => {
                function.instruction(&Instruction::F32Gt);
            }

            (BinaryOp::Ge, ValType::I64) => {
                function.instruction(&Instruction::I64GeS);
            }
            (BinaryOp::Ge, ValType::I32) => {
                function.instruction(&Instruction::I32GeS);
            }
            (BinaryOp::Ge, ValType::F64) => {
                function.instruction(&Instruction::F64Ge);
            }
            (BinaryOp::Ge, ValType::F32) => {
                function.instruction(&Instruction::F32Ge);
            }

            // Bitwise operations (integer only)
            (BinaryOp::And, ValType::I64) => {
                function.instruction(&Instruction::I64And);
            }
            (BinaryOp::And, ValType::I32) => {
                function.instruction(&Instruction::I32And);
            }
            (BinaryOp::And, ValType::F64 | ValType::F32) => {
                return Err(WasmError::new(
                    "Bitwise AND not supported for floating point types",
                ))
            }

            (BinaryOp::Or, ValType::I64) => {
                function.instruction(&Instruction::I64Or);
            }
            (BinaryOp::Or, ValType::I32) => {
                function.instruction(&Instruction::I32Or);
            }
            (BinaryOp::Or, ValType::F64 | ValType::F32) => {
                return Err(WasmError::new(
                    "Bitwise OR not supported for floating point types",
                ))
            }

            (BinaryOp::Pow, _) => {
                return Err(WasmError::new(
                    "Exponentiation not supported in basic WASM (requires math functions)",
                ))
            }
            (
                BinaryOp::Pipe
                | BinaryOp::Compose
                | BinaryOp::Apply
                | BinaryOp::Bind
                | BinaryOp::Member
                | BinaryOp::Map
                | BinaryOp::Ap
                | BinaryOp::Implies
                | BinaryOp::Range,
                _,
            ) => {
                return Err(WasmError::new(format!(
                    "Operator {:?} not supported in WASM compilation",
                    op
                )))
            }
            // Default for unsupported combinations
            _ => {
                return Err(WasmError::new(format!(
                    "Unsupported type {:?} for operator {:?}",
                    val_type, op
                )))
            }
        }

        Ok(())
    }

    /// Check if an expression produces a value on the WASM stack.
    ///
    /// Some expressions like `if` without `else` produce no value.
    /// Function calls are conservatively assumed to not produce values
    /// since we don't have return type info at this point.
    #[allow(clippy::only_used_in_recursion)]
    fn expression_produces_value(&self, expr: &crate::ast::Expr) -> bool {
        use crate::ast::Expr;

        match expr {
            // If without else produces no value
            Expr::If {
                else_branch: None, ..
            } => false,

            // If-else: check if the then branch produces a value
            // Assignments and blocks without final expressions don't produce values
            Expr::If {
                then_branch,
                else_branch: Some(_),
                ..
            } => self.expression_produces_value(then_branch),

            // Block without final expression produces no value
            Expr::Block(crate::ast::Block {
                final_expr: None, ..
            }) => false,

            // Block with statements that are assignments produces no value
            Expr::Block(crate::ast::Block {
                statements,
                final_expr: Some(final_expr),
                ..
            }) => {
                // Check if the final_expr is an assignment (produces no value)
                // or if it's a value-producing expression
                if statements.is_empty() {
                    self.expression_produces_value(final_expr)
                } else {
                    // Block with statements and final expression - check the final
                    self.expression_produces_value(final_expr)
                }
            }

            // Function calls - conservatively assume no value produced
            // This is safe for void functions, and for non-void functions
            // the value will remain on the stack (not dropped), which is
            // fine since we're about to return anyway or move to next statement.
            Expr::Call { .. } => false,

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

    /// Check if any declarations contain string literals.
    ///
    /// This is used to determine if memory needs to be allocated for the data section.
    #[allow(dead_code)]
    fn has_string_literals(declarations: &[crate::ast::Declaration]) -> bool {
        use crate::ast::{Declaration, Expr, Literal, Statement, Stmt};

        fn check_expr(expr: &Expr) -> bool {
            match expr {
                Expr::Literal(Literal::String(_)) => true,
                Expr::Binary { left, right, .. } => check_expr(left) || check_expr(right),
                Expr::Unary { operand, .. } => check_expr(operand),
                Expr::Call { callee, args, .. } => {
                    check_expr(callee) || args.iter().any(check_expr)
                }
                Expr::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    check_expr(condition)
                        || check_expr(then_branch)
                        || else_branch.as_ref().is_some_and(|e| check_expr(e))
                }
                Expr::Block(crate::ast::Block {
                    statements,
                    final_expr,
                    ..
                }) => {
                    statements.iter().any(check_stmt)
                        || final_expr.as_ref().is_some_and(|e| check_expr(e))
                }
                Expr::Member { object, .. } => check_expr(object),
                Expr::StructLiteral { fields, .. } => fields.iter().any(|(_, e)| check_expr(e)),
                Expr::Match { scrutinee, arms } => {
                    check_expr(scrutinee)
                        || arms.iter().any(|arm| {
                            check_expr(&arm.body)
                                || arm.guard.as_ref().is_some_and(|g| check_expr(g))
                        })
                }
                Expr::Lambda { body, .. } => check_expr(body),
                Expr::List(exprs) | Expr::Tuple(exprs) => exprs.iter().any(check_expr),
                Expr::IdiomBracket { func, args } => {
                    check_expr(func) || args.iter().any(check_expr)
                }
                Expr::Quote(e) | Expr::Unquote(e) | Expr::QuasiQuote(e) | Expr::Eval(e) => {
                    check_expr(e)
                }
                Expr::Implies { left, right, .. } => check_expr(left) || check_expr(right),
                Expr::SexBlock(crate::ast::Block { statements, .. }) => {
                    statements.iter().any(check_stmt)
                }
                _ => false,
            }
        }

        fn check_stmt(stmt: &Stmt) -> bool {
            match stmt {
                Stmt::Expr(e) | Stmt::Return(Some(e)) => check_expr(e),
                Stmt::Let { value, .. } => check_expr(value),
                Stmt::Assign { target, value } => check_expr(target) || check_expr(value),
                Stmt::For { iterable, body, .. } => {
                    check_expr(iterable) || body.iter().any(check_stmt)
                }
                Stmt::While { condition, body } => {
                    check_expr(condition) || body.iter().any(check_stmt)
                }
                Stmt::Loop { body } => body.iter().any(check_stmt),
                _ => false,
            }
        }

        fn check_function(func: &crate::ast::FunctionDecl) -> bool {
            func.body.iter().any(check_stmt)
        }

        for decl in declarations {
            match decl {
                Declaration::Gene(gene) => {
                    // Check methods in gene statements
                    for stmt in &gene.statements {
                        if let Statement::Function(func) = stmt {
                            if check_function(func) {
                                return true;
                            }
                        }
                    }
                }
                Declaration::Function(func) => {
                    if check_function(func) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(feature = "wasm-compile")]
impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-compile")]
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
            attributes: Vec::new(),
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
            attributes: Vec::new(),
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
        use crate::ast::{Gen, Visibility};

        // Try to compile a Gene (not supported)
        let gene = Gen {
            name: "test.gene".to_string(),
            visibility: Visibility::Private,
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

    #[test]
    fn test_gene_constructor_generation() {
        use crate::ast::{DolFile, Gen, HasField, Statement, Visibility};

        // Create a gene with two i64 fields
        let gene = Gen {
            name: "Point".to_string(),
            visibility: Visibility::Public,
            extends: None,
            statements: vec![
                Statement::HasField(Box::new(HasField {
                    name: "x".to_string(),
                    type_: TypeExpr::Named("Int64".to_string()),
                    default: None,
                    constraint: None,
                    span: Span::default(),
                })),
                Statement::HasField(Box::new(HasField {
                    name: "y".to_string(),
                    type_: TypeExpr::Named("Int64".to_string()),
                    default: None,
                    constraint: None,
                    span: Span::default(),
                })),
            ],
            exegesis: "A 2D point".to_string(),
            span: Span::default(),
        };

        // Create a simple function that uses the gene
        let func = FunctionDecl {
            visibility: Visibility::Public,
            purity: Purity::Pure,
            name: "get_origin".to_string(),
            type_params: None,
            params: vec![],
            return_type: Some(TypeExpr::Named("i32".to_string())), // Returns pointer
            body: vec![Stmt::Return(Some(Expr::StructLiteral {
                type_name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Expr::Literal(Literal::Int(0))),
                    ("y".to_string(), Expr::Literal(Literal::Int(0))),
                ],
            }))],
            exegesis: "Creates origin point".to_string(),
            span: Span::default(),
            attributes: Vec::new(),
        };

        let file = DolFile {
            module: None,
            uses: vec![],
            declarations: vec![
                Declaration::Gene(gene),
                Declaration::Function(Box::new(func)),
            ],
        };

        let mut compiler = WasmCompiler::new();
        let wasm_bytes = compiler.compile_file(&file).expect("Compilation failed");

        // Verify WASM magic number
        assert!(wasm_bytes.len() >= 8, "WASM output too short");
        assert_eq!(
            &wasm_bytes[0..4],
            &[0x00, 0x61, 0x73, 0x6D],
            "Invalid WASM magic number"
        );

        // Verify we have the layout registered
        assert!(compiler.has_gene_layouts());
        let names = compiler.get_gene_layout_names();
        assert!(names.contains(&"Point".to_string()));

        // Verify the layout is correct
        let layout = compiler
            .get_gene_layout("Point")
            .expect("Point layout not found");
        assert_eq!(layout.total_size, 16); // Two i64 fields = 16 bytes
        assert_eq!(layout.alignment, 8); // Aligned to largest field
        assert_eq!(layout.fields.len(), 2);
        assert_eq!(layout.fields[0].name, "x");
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[1].name, "y");
        assert_eq!(layout.fields[1].offset, 8);
    }
}
