//! DOL HIR → LLVM IR Lowering
//!
//! Converts HIR declarations and expressions to LLVM IR using inkwell.

use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::IntPredicate;

use metadol::hir::{
    HirBinaryOp, HirBlockExpr, HirDecl, HirExpr, HirFunctionDecl, HirIfExpr, HirLiteral, HirModule,
    HirNamedType, HirPat, HirStatementKind, HirStmt, HirType, HirTypeDecl, HirTypeDef, HirUnaryOp,
    Symbol, SymbolTable,
};

use crate::abi::AbiGenerator;
use crate::types::TypeMapper;
use crate::{CodegenError, Result};

/// HIR to LLVM IR lowering engine.
pub struct HirLowering<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: Builder<'ctx>,
    type_mapper: TypeMapper<'ctx>,
    symbols: &'a SymbolTable,

    /// Named values in the current scope (variable name → (alloca pointer, stored type))
    named_values: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,

    /// Named struct types already generated
    struct_types: HashMap<String, inkwell::types::StructType<'ctx>>,
}

impl<'a, 'ctx> HirLowering<'a, 'ctx> {
    /// Create a new HIR lowering engine.
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>, symbols: &'a SymbolTable) -> Self {
        // Declare VUDO host functions
        let abi = AbiGenerator::new(context, module);
        abi.declare_all_host_functions();

        Self {
            context,
            module,
            builder: context.create_builder(),
            type_mapper: TypeMapper::new(context),
            symbols,
            named_values: HashMap::new(),
            struct_types: HashMap::new(),
        }
    }

    /// Resolve a symbol to its string name.
    fn sym(&self, s: Symbol) -> &str {
        self.symbols.resolve(s).unwrap_or("<unknown>")
    }

    // ========================================================================
    // Module & Declaration Lowering
    // ========================================================================

    /// Lower an entire HIR module to LLVM IR.
    pub fn lower_module(&mut self, hir: &HirModule) -> Result<()> {
        for decl in &hir.decls {
            self.lower_decl(decl)?;
        }
        Ok(())
    }

    /// Lower a single declaration.
    fn lower_decl(&mut self, decl: &HirDecl) -> Result<()> {
        match decl {
            HirDecl::Type(type_decl) => self.lower_type_decl(type_decl),
            HirDecl::Function(func_decl) => {
                self.lower_function_decl(func_decl)?;
                Ok(())
            }
            HirDecl::Trait(_) => Ok(()), // Traits are compile-time only
            HirDecl::Module(mod_decl) => {
                for d in &mod_decl.decls {
                    self.lower_decl(d)?;
                }
                Ok(())
            }
        }
    }

    // ========================================================================
    // Type Declarations
    // ========================================================================

    /// Lower a type declaration (struct, enum, gene) to LLVM struct types.
    fn lower_type_decl(&mut self, decl: &HirTypeDecl) -> Result<()> {
        let name = self.sym(decl.name).to_string();

        match &decl.body {
            HirTypeDef::Struct(fields) => {
                let field_types: Vec<BasicTypeEnum<'ctx>> = fields
                    .iter()
                    .map(|f| self.map_hir_type(&f.ty))
                    .collect::<Result<Vec<_>>>()?;

                let struct_type = self.context.opaque_struct_type(&name);
                struct_type.set_body(&field_types, false);
                self.struct_types.insert(name, struct_type);
            }
            HirTypeDef::Gene(statements) => {
                // Gene with `has` statements → struct with inferred fields
                let mut field_types = Vec::new();
                for stmt in statements {
                    if let HirStatementKind::Has { property, .. } = &stmt.kind {
                        let prop_name = self.sym(*property);
                        let ty = self.infer_field_type(prop_name);
                        field_types.push(ty);
                    }
                }
                if !field_types.is_empty() {
                    let struct_type = self.context.opaque_struct_type(&name);
                    struct_type.set_body(&field_types, false);
                    self.struct_types.insert(name, struct_type);
                }
            }
            HirTypeDef::Enum(variants) => {
                // Enum: tag (i32) + largest payload
                let i32_type: BasicTypeEnum = self.context.i32_type().into();
                // For now, generate a simple tagged union with i64 payload
                let payload: BasicTypeEnum = self.context.i64_type().into();
                let struct_type = self.context.opaque_struct_type(&name);
                struct_type.set_body(&[i32_type, payload], false);
                self.struct_types.insert(name.clone(), struct_type);

                // Generate tag constants
                for (i, variant) in variants.iter().enumerate() {
                    let variant_name = self.sym(variant.name);
                    let const_name = format!("{}::{}", name, variant_name);
                    let global = self.module.add_global(
                        self.context.i32_type(),
                        Some(AddressSpace::default()),
                        &const_name,
                    );
                    global.set_initializer(&self.context.i32_type().const_int(i as u64, false));
                    global.set_constant(true);
                }
            }
            HirTypeDef::Alias(_) => {
                // Type aliases don't produce LLVM IR
            }
        }

        Ok(())
    }

    /// Infer LLVM type from a property name (heuristic).
    fn infer_field_type(&self, name: &str) -> BasicTypeEnum<'ctx> {
        match name {
            n if n.contains("name")
                || n.contains("label")
                || n.contains("description")
                || n.contains("title")
                || n.contains("text")
                || n.contains("message") =>
            {
                self.type_mapper.string_type().into()
            }
            n if n.contains("count")
                || n.contains("size")
                || n.contains("length")
                || n.contains("index")
                || n.contains("id")
                || n.contains("port") =>
            {
                self.context.i64_type().into()
            }
            n if n.contains("active")
                || n.contains("enabled")
                || n.contains("alive")
                || n.contains("visible")
                || n.contains("ready") =>
            {
                self.context.bool_type().into()
            }
            n if n.contains("x")
                || n.contains("y")
                || n.contains("z")
                || n.contains("width")
                || n.contains("height")
                || n.contains("ratio") =>
            {
                self.context.f64_type().into()
            }
            _ => self.context.i64_type().into(), // Default to i64
        }
    }

    // ========================================================================
    // Function Declarations
    // ========================================================================

    /// Lower a function declaration to LLVM.
    pub fn lower_function_decl(&mut self, decl: &HirFunctionDecl) -> Result<FunctionValue<'ctx>> {
        let name = self.sym(decl.name).to_string();

        // Map parameter types
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = decl
            .params
            .iter()
            .map(|p| self.map_hir_type(&p.ty).map(|t| t.into()))
            .collect::<Result<Vec<_>>>()?;

        // Map return type.
        // Special case: `main` must return i32 for C ABI compatibility.
        let is_main = name == "main";
        let fn_type = if is_main && param_types.is_empty() {
            self.context.i32_type().fn_type(&param_types, false)
        } else {
            match &decl.return_type {
                HirType::Named(n) if self.sym(n.name) == "()" || self.sym(n.name) == "Unit" => {
                    self.context.void_type().fn_type(&param_types, false)
                }
                HirType::Tuple(elems) if elems.is_empty() => {
                    self.context.void_type().fn_type(&param_types, false)
                }
                ret => {
                    let ret_type = self.map_hir_type(ret)?;
                    ret_type.fn_type(&param_types, false)
                }
            }
        };

        let function = self.module.add_function(&name, fn_type, None);

        // Set parameter names
        for (i, param) in decl.params.iter().enumerate() {
            if let HirPat::Var(sym) = &param.pat {
                let param_name = self.sym(*sym);
                if let Some(p) = function.get_nth_param(i as u32) {
                    p.set_name(param_name);
                }
            }
        }

        // Generate body if present
        if let Some(body) = &decl.body {
            let entry = self.context.append_basic_block(function, "entry");
            self.builder.position_at_end(entry);

            // Create allocas for parameters
            self.named_values.clear();
            for (i, param) in decl.params.iter().enumerate() {
                if let HirPat::Var(sym) = &param.pat {
                    let param_name = self.sym(*sym).to_string();
                    if let Some(p) = function.get_nth_param(i as u32) {
                        let ty = p.get_type();
                        let alloca = self.create_entry_alloca(function, &param_name, ty);
                        self.builder
                            .build_store(alloca, p)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.named_values.insert(param_name, (alloca, ty));
                    }
                }
            }

            // Generate body expression
            let is_void = !is_main
                && (matches!(&decl.return_type,
                    HirType::Named(n) if self.sym(n.name) == "()" || self.sym(n.name) == "Unit"
                ) || matches!(&decl.return_type, HirType::Tuple(elems) if elems.is_empty()));

            match self.lower_expr(body) {
                Ok(Some(val)) if !is_void && !is_main => {
                    if !self.current_block_terminated() {
                        self.builder
                            .build_return(Some(&val))
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }
                _ => {
                    if !self.current_block_terminated() {
                        if is_main {
                            // main returns i32 0 for success
                            let zero = self.context.i32_type().const_int(0, false);
                            self.builder
                                .build_return(Some(&zero))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else if is_void {
                            self.builder
                                .build_return(None)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else {
                            // Non-void function fell through without return:
                            // return a zero-initialized value to avoid UB
                            let ret_ty = self.map_hir_type(&decl.return_type)?;
                            let zero = ret_ty.const_zero();
                            self.builder
                                .build_return(Some(&zero))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }
                }
            }
        }

        Ok(function)
    }

    /// Check if the current basic block already has a terminator instruction.
    fn current_block_terminated(&self) -> bool {
        self.builder
            .get_insert_block()
            .map(|bb| bb.get_terminator().is_some())
            .unwrap_or(true)
    }

    /// Create an alloca in the entry block of a function.
    fn create_entry_alloca(
        &self,
        function: FunctionValue<'ctx>,
        name: &str,
        ty: BasicTypeEnum<'ctx>,
    ) -> PointerValue<'ctx> {
        let entry = function.get_first_basic_block().unwrap();
        let tmp_builder = self.context.create_builder();
        match entry.get_first_instruction() {
            Some(inst) => tmp_builder.position_before(&inst),
            None => tmp_builder.position_at_end(entry),
        }
        tmp_builder.build_alloca(ty, name).unwrap()
    }

    // ========================================================================
    // Type Mapping
    // ========================================================================

    /// Map a HIR type to an LLVM type.
    pub fn map_hir_type(&self, ty: &HirType) -> Result<BasicTypeEnum<'ctx>> {
        match ty {
            HirType::Named(named) => self.map_named_type(named),
            HirType::Tuple(elems) => {
                let elem_types: Vec<BasicTypeEnum<'ctx>> = elems
                    .iter()
                    .map(|e| self.map_hir_type(e))
                    .collect::<Result<Vec<_>>>()?;
                Ok(self.context.struct_type(&elem_types, false).into())
            }
            HirType::Array(arr) => {
                let elem = self.map_hir_type(&arr.elem)?;
                if let Some(size) = arr.size {
                    Ok(elem.array_type(size as u32).into())
                } else {
                    // Dynamic array → fat pointer {ptr, len}
                    let ptr = self.context.ptr_type(AddressSpace::default());
                    let len = self.context.i64_type();
                    Ok(self
                        .context
                        .struct_type(&[ptr.into(), len.into()], false)
                        .into())
                }
            }
            HirType::Function(_) => {
                // Function types are pointers
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            HirType::Ref(_) => Ok(self.context.ptr_type(AddressSpace::default()).into()),
            HirType::Optional(inner) => {
                // Option<T> → { i1 has_value, T value }
                let inner_ty = self.map_hir_type(inner)?;
                Ok(self
                    .context
                    .struct_type(&[self.context.bool_type().into(), inner_ty], false)
                    .into())
            }
            HirType::Var(_) | HirType::Error => {
                // Default to i64 for unresolved types
                Ok(self.context.i64_type().into())
            }
        }
    }

    /// Map a named HIR type to an LLVM type.
    fn map_named_type(&self, named: &HirNamedType) -> Result<BasicTypeEnum<'ctx>> {
        let name = self.sym(named.name);

        // String type → fat string struct { ptr, i64 }
        if name == "string" || name == "String" || name == "str" {
            return Ok(self.dol_string_type().into());
        }

        // Check primitive type mappings
        if let Some(ty) = self.type_mapper.map_primitive(name) {
            return Ok(ty);
        }

        // Check for known struct types
        if let Some(struct_ty) = self.struct_types.get(name) {
            return Ok((*struct_ty).into());
        }

        // Fallback: treat as opaque pointer
        Ok(self.context.ptr_type(AddressSpace::default()).into())
    }

    // ========================================================================
    // Expression Lowering
    // ========================================================================

    /// Lower an expression to an LLVM value.
    ///
    /// Returns `None` for void expressions (statements, void calls, etc.).
    pub fn lower_expr(&mut self, expr: &HirExpr) -> Result<Option<BasicValueEnum<'ctx>>> {
        match expr {
            HirExpr::Literal(lit) => self.lower_literal(lit).map(Some),
            HirExpr::Var(sym) => self.lower_var(*sym).map(Some),
            HirExpr::Binary(bin) => self.lower_binary(bin).map(Some),
            HirExpr::Unary(un) => self.lower_unary(un).map(Some),
            HirExpr::Call(call) => self.lower_call(call),
            HirExpr::MethodCall(mc) => self.lower_method_call(mc),
            HirExpr::Field(_) => {
                // Field access not yet supported in native codegen
                Ok(Some(self.context.i64_type().const_int(0, false).into()))
            }
            HirExpr::Index(_) => {
                // Index access not yet supported in native codegen
                Ok(Some(self.context.i64_type().const_int(0, false).into()))
            }
            HirExpr::Block(block) => self.lower_block(block),
            HirExpr::If(if_expr) => self.lower_if(if_expr),
            HirExpr::Match(_) => {
                // Match not yet supported in native codegen
                Ok(Some(self.context.i64_type().const_int(0, false).into()))
            }
            HirExpr::Lambda(_) => {
                // Lambda not yet supported in native codegen
                Ok(Some(
                    self.context
                        .ptr_type(AddressSpace::default())
                        .const_null()
                        .into(),
                ))
            }
        }
    }

    /// Lower a literal value.
    fn lower_literal(&self, lit: &HirLiteral) -> Result<BasicValueEnum<'ctx>> {
        match lit {
            HirLiteral::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            HirLiteral::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, *n < 0).into()),
            HirLiteral::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            HirLiteral::String(s) => {
                // Create a global string constant as fat string { ptr, len }
                let len = s.len() as u64;
                let global = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                let str_type = self.context.struct_type(
                    &[
                        self.context.ptr_type(AddressSpace::default()).into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let mut result = str_type.get_undef();
                result = self
                    .builder
                    .build_insert_value(result, global.as_pointer_value(), 0, "str_ptr")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                result = self
                    .builder
                    .build_insert_value(
                        result,
                        self.context.i64_type().const_int(len, false),
                        1,
                        "str_len",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                Ok(result.into())
            }
            HirLiteral::Unit => {
                // Unit → zero-sized, represent as i1 false
                Ok(self.context.bool_type().const_int(0, false).into())
            }
        }
    }

    /// Lower a variable reference.
    fn lower_var(&self, sym: Symbol) -> Result<BasicValueEnum<'ctx>> {
        let name = self.sym(sym);
        if let Some((alloca, ty)) = self.named_values.get(name) {
            let val = self
                .builder
                .build_load(*ty, *alloca, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(val)
        } else {
            // Could be a global or function reference
            if let Some(func) = self.module.get_function(name) {
                Ok(func.as_global_value().as_pointer_value().into())
            } else {
                Err(CodegenError::LlvmError(format!(
                    "undefined variable: {}",
                    name
                )))
            }
        }
    }

    /// Get the DOL fat string type: { ptr, i64 }.
    fn dol_string_type(&self) -> inkwell::types::StructType<'ctx> {
        self.context.struct_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.i64_type().into(),
            ],
            false,
        )
    }

    /// Check if a value is a DOL fat string { ptr, i64 }.
    fn is_fat_string(&self, val: &BasicValueEnum<'ctx>) -> bool {
        if let BasicValueEnum::StructValue(sv) = val {
            sv.get_type().count_fields() == 2
        } else {
            false
        }
    }

    /// Coerce a value to a (ptr, len) pair for string operations.
    /// - String literals (ptr) → (ptr, 0) with length looked up from source
    /// - Fat strings { ptr, len } → extracted components
    /// - Integers → call vudo_i64_to_string
    fn coerce_to_string_parts(
        &mut self,
        val: BasicValueEnum<'ctx>,
        source_expr: &HirExpr,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>)> {
        if self.is_fat_string(&val) {
            // Fat string struct: extract ptr and len
            let sv = val.into_struct_value();
            let ptr = self
                .builder
                .build_extract_value(sv, 0, "str_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let len = self
                .builder
                .build_extract_value(sv, 1, "str_len")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok((ptr, len))
        } else if val.is_pointer_value() {
            // Bare string pointer — get length from source expression if possible
            if let HirExpr::Literal(HirLiteral::String(s)) = source_expr {
                let len = self.context.i64_type().const_int(s.len() as u64, false);
                Ok((val, len.into()))
            } else {
                let len = self.context.i64_type().const_int(0, false);
                Ok((val, len.into()))
            }
        } else if val.is_int_value() {
            // Integer: convert to string via vudo_i64_to_string
            let function = self
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();
            let out_ptr_alloca = self.create_entry_alloca(
                function,
                "i2s_ptr",
                self.context.ptr_type(AddressSpace::default()).into(),
            );
            let out_len_alloca =
                self.create_entry_alloca(function, "i2s_len", self.context.i64_type().into());

            let i2s_fn = self
                .module
                .get_function("vudo_i64_to_string")
                .ok_or_else(|| CodegenError::LlvmError("vudo_i64_to_string not declared".into()))?;

            self.builder
                .build_call(
                    i2s_fn,
                    &[val.into(), out_ptr_alloca.into(), out_len_alloca.into()],
                    "i2s",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            let ptr = self
                .builder
                .build_load(
                    self.context.ptr_type(AddressSpace::default()),
                    out_ptr_alloca,
                    "s_ptr",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let len = self
                .builder
                .build_load(self.context.i64_type(), out_len_alloca, "s_len")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            Ok((ptr, len))
        } else {
            Err(CodegenError::LlvmError(
                "cannot coerce value to string".into(),
            ))
        }
    }

    /// Perform string concatenation via vudo_string_concat runtime call.
    fn build_string_concat(
        &mut self,
        lhs: BasicValueEnum<'ctx>,
        lhs_expr: &HirExpr,
        rhs: BasicValueEnum<'ctx>,
        rhs_expr: &HirExpr,
    ) -> Result<BasicValueEnum<'ctx>> {
        let (lhs_ptr, lhs_len) = self.coerce_to_string_parts(lhs, lhs_expr)?;
        let (rhs_ptr, rhs_len) = self.coerce_to_string_parts(rhs, rhs_expr)?;

        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let out_ptr_alloca = self.create_entry_alloca(
            function,
            "cat_ptr",
            self.context.ptr_type(AddressSpace::default()).into(),
        );
        let out_len_alloca =
            self.create_entry_alloca(function, "cat_len", self.context.i64_type().into());

        let concat_fn = self
            .module
            .get_function("vudo_string_concat")
            .ok_or_else(|| CodegenError::LlvmError("vudo_string_concat not declared".into()))?;

        self.builder
            .build_call(
                concat_fn,
                &[
                    lhs_ptr.into(),
                    lhs_len.into(),
                    rhs_ptr.into(),
                    rhs_len.into(),
                    out_ptr_alloca.into(),
                    out_len_alloca.into(),
                ],
                "concat",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Load the result into a fat string struct
        let ptr = self
            .builder
            .build_load(
                self.context.ptr_type(AddressSpace::default()),
                out_ptr_alloca,
                "cat_ptr_val",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let len = self
            .builder
            .build_load(self.context.i64_type(), out_len_alloca, "cat_len_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Build the result as { ptr, len } struct
        let str_type = self.dol_string_type();
        let mut result = str_type.get_undef();
        result = self
            .builder
            .build_insert_value(result, ptr, 0, "str_with_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        result = self
            .builder
            .build_insert_value(result, len, 1, "str_with_len")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();

        Ok(result.into())
    }

    /// Check if an expression involves string types (heuristic).
    /// Lower a binary expression.
    fn lower_binary(&mut self, bin: &metadol::hir::HirBinaryExpr) -> Result<BasicValueEnum<'ctx>> {
        let lhs = self.lower_expr(&bin.left)?.unwrap();
        let rhs = self.lower_expr(&bin.right)?.unwrap();

        // String concatenation: Add with any string-like operand
        if bin.op == HirBinaryOp::Add
            && (self.is_fat_string(&lhs)
                || lhs.is_pointer_value()
                || self.is_fat_string(&rhs)
                || rhs.is_pointer_value())
        {
            return self.build_string_concat(lhs, &bin.left, rhs, &bin.right);
        }

        // Float operations
        if lhs.is_float_value() && rhs.is_float_value() {
            let lhs_f = lhs.into_float_value();
            let rhs_f = rhs.into_float_value();
            let result = match bin.op {
                HirBinaryOp::Add => self.builder.build_float_add(lhs_f, rhs_f, "fadd"),
                HirBinaryOp::Sub => self.builder.build_float_sub(lhs_f, rhs_f, "fsub"),
                HirBinaryOp::Mul => self.builder.build_float_mul(lhs_f, rhs_f, "fmul"),
                HirBinaryOp::Div => self.builder.build_float_div(lhs_f, rhs_f, "fdiv"),
                HirBinaryOp::Mod => self.builder.build_float_rem(lhs_f, rhs_f, "frem"),
                _ => {
                    return Err(CodegenError::LlvmError(format!(
                        "unsupported float binary op: {:?}",
                        bin.op
                    )))
                }
            }
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(result.into());
        }

        // Integer operations
        let lhs_int = lhs.into_int_value();
        let rhs_int = rhs.into_int_value();

        let result = match bin.op {
            HirBinaryOp::Add => self.builder.build_int_add(lhs_int, rhs_int, "add"),
            HirBinaryOp::Sub => self.builder.build_int_sub(lhs_int, rhs_int, "sub"),
            HirBinaryOp::Mul => self.builder.build_int_mul(lhs_int, rhs_int, "mul"),
            HirBinaryOp::Div => self.builder.build_int_signed_div(lhs_int, rhs_int, "div"),
            HirBinaryOp::Mod => self.builder.build_int_signed_rem(lhs_int, rhs_int, "rem"),
            HirBinaryOp::Eq => {
                self.builder
                    .build_int_compare(IntPredicate::EQ, lhs_int, rhs_int, "eq")
            }
            HirBinaryOp::Ne => {
                self.builder
                    .build_int_compare(IntPredicate::NE, lhs_int, rhs_int, "ne")
            }
            HirBinaryOp::Lt => {
                self.builder
                    .build_int_compare(IntPredicate::SLT, lhs_int, rhs_int, "lt")
            }
            HirBinaryOp::Le => {
                self.builder
                    .build_int_compare(IntPredicate::SLE, lhs_int, rhs_int, "le")
            }
            HirBinaryOp::Gt => {
                self.builder
                    .build_int_compare(IntPredicate::SGT, lhs_int, rhs_int, "gt")
            }
            HirBinaryOp::Ge => {
                self.builder
                    .build_int_compare(IntPredicate::SGE, lhs_int, rhs_int, "ge")
            }
            HirBinaryOp::And => self.builder.build_and(lhs_int, rhs_int, "and"),
            HirBinaryOp::Or => self.builder.build_or(lhs_int, rhs_int, "or"),
        }
        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(result.into())
    }

    /// Lower a unary expression.
    fn lower_unary(&mut self, un: &metadol::hir::HirUnaryExpr) -> Result<BasicValueEnum<'ctx>> {
        let operand = self.lower_expr(&un.operand)?.unwrap();
        let int_val = operand.into_int_value();

        let result = match un.op {
            HirUnaryOp::Neg => self.builder.build_int_neg(int_val, "neg"),
            HirUnaryOp::Not => self.builder.build_not(int_val, "not"),
        }
        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(result.into())
    }

    /// Lower a method call expression.
    fn lower_method_call(
        &mut self,
        mc: &metadol::hir::HirMethodCallExpr,
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
        let method_name = self.sym(mc.method).to_string();

        match method_name.as_str() {
            "to_string" => {
                // receiver.to_string() → vudo_i64_to_string(receiver)
                let receiver = self.lower_expr(&mc.receiver)?.unwrap();
                let (ptr, len) = self.coerce_to_string_parts(receiver, &mc.receiver)?;

                // Build a fat string struct
                let str_type = self.dol_string_type();
                let mut result = str_type.get_undef();
                result = self
                    .builder
                    .build_insert_value(result, ptr, 0, "ts_ptr")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                result = self
                    .builder
                    .build_insert_value(result, len, 1, "ts_len")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                Ok(Some(result.into()))
            }
            _ => {
                // Unsupported method — return zero for now
                Ok(Some(self.context.i64_type().const_int(0, false).into()))
            }
        }
    }

    /// Map DOL built-in function names to VUDO host function names.
    fn builtin_to_vudo(name: &str) -> Option<&'static str> {
        match name {
            "print" => Some("vudo_print"),
            "println" => Some("vudo_println"),
            "log" => Some("vudo_log"),
            "error" => Some("vudo_error"),
            "alloc" => Some("vudo_alloc"),
            "free" => Some("vudo_free"),
            "realloc" => Some("vudo_realloc"),
            "now" => Some("vudo_now"),
            "sleep" => Some("vudo_sleep"),
            "random" => Some("vudo_random"),
            "breakpoint" => Some("vudo_breakpoint"),
            "assert" => Some("vudo_assert"),
            "panic" => Some("vudo_panic"),
            _ => None,
        }
    }

    /// Build a call to a string-accepting VUDO host function (ptr, len ABI).
    fn build_string_call(
        &mut self,
        vudo_name: &str,
        string_arg: &HirExpr,
        extra_prefix_args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
        let function = self
            .module
            .get_function(vudo_name)
            .ok_or_else(|| CodegenError::LlvmError(format!("undefined host fn: {}", vudo_name)))?;

        // Lower the string argument and extract (ptr, len) pair
        let val = self.lower_expr(string_arg)?.unwrap();
        let (ptr, len) = self.coerce_to_string_parts(val, string_arg)?;

        let mut args: Vec<BasicMetadataValueEnum<'ctx>> = extra_prefix_args.to_vec();
        args.push(ptr.into());
        args.push(len.into());

        let call_val = self
            .builder
            .build_call(function, &args, vudo_name)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        match call_val.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(val) => Ok(Some(val)),
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }

    /// Lower a function call.
    fn lower_call(
        &mut self,
        call: &metadol::hir::HirCallExpr,
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
        // Resolve function name
        let func_name = match &call.func {
            HirExpr::Var(sym) => self.sym(*sym).to_string(),
            _ => {
                return Err(CodegenError::LlvmError(
                    "indirect calls not supported".into(),
                ))
            }
        };

        // Check for built-in → VUDO host function mapping
        if let Some(vudo_name) = Self::builtin_to_vudo(&func_name) {
            // String-accepting builtins: print, println, panic, assert, error, log
            match func_name.as_str() {
                "print" | "println" if !call.args.is_empty() => {
                    return self.build_string_call(vudo_name, &call.args[0], &[]);
                }
                "panic" if !call.args.is_empty() => {
                    return self.build_string_call(vudo_name, &call.args[0], &[]);
                }
                "error" if call.args.len() >= 2 => {
                    // error(code, msg) → vudo_error(i32, ptr, len)
                    let code = self.lower_expr(&call.args[0])?.unwrap();
                    return self.build_string_call(vudo_name, &call.args[1], &[code.into()]);
                }
                "log" if call.args.len() >= 2 => {
                    // log(level, msg) → vudo_log(i32, ptr, len)
                    let level = self.lower_expr(&call.args[0])?.unwrap();
                    return self.build_string_call(vudo_name, &call.args[1], &[level.into()]);
                }
                "assert" if call.args.len() >= 2 => {
                    // assert(cond, msg) → vudo_assert(i32, ptr, len)
                    let cond = self.lower_expr(&call.args[0])?.unwrap();
                    return self.build_string_call(vudo_name, &call.args[1], &[cond.into()]);
                }
                _ => {
                    // Non-string builtins: fall through to regular call with vudo name
                    let function = self.module.get_function(vudo_name).ok_or_else(|| {
                        CodegenError::LlvmError(format!("undefined host fn: {}", vudo_name))
                    })?;

                    let args: Vec<BasicMetadataValueEnum<'ctx>> = call
                        .args
                        .iter()
                        .map(|a| self.lower_expr(a).map(|v| v.unwrap().into()))
                        .collect::<Result<Vec<_>>>()?;

                    let call_val = self
                        .builder
                        .build_call(function, &args, vudo_name)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    return match call_val.try_as_basic_value() {
                        inkwell::values::ValueKind::Basic(val) => Ok(Some(val)),
                        inkwell::values::ValueKind::Instruction(_) => Ok(None),
                    };
                }
            }
        }

        let function = self
            .module
            .get_function(&func_name)
            .ok_or_else(|| CodegenError::LlvmError(format!("undefined function: {}", func_name)))?;

        // Lower arguments
        let args: Vec<BasicMetadataValueEnum<'ctx>> = call
            .args
            .iter()
            .map(|a| self.lower_expr(a).map(|v| v.unwrap().into()))
            .collect::<Result<Vec<_>>>()?;

        let call_val = self
            .builder
            .build_call(function, &args, &func_name)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Return value or None for void functions
        match call_val.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(val) => Ok(Some(val)),
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }

    /// Lower a block expression.
    fn lower_block(&mut self, block: &HirBlockExpr) -> Result<Option<BasicValueEnum<'ctx>>> {
        for stmt in &block.stmts {
            self.lower_stmt(stmt)?;
        }

        if let Some(expr) = &block.expr {
            self.lower_expr(expr)
        } else {
            Ok(None)
        }
    }

    /// Lower an if expression.
    fn lower_if(&mut self, if_expr: &HirIfExpr) -> Result<Option<BasicValueEnum<'ctx>>> {
        let cond = self.lower_expr(&if_expr.cond)?.unwrap();
        let cond_int = cond.into_int_value();

        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let then_bb = self.context.append_basic_block(function, "then");
        let else_bb = self.context.append_basic_block(function, "else");
        let merge_bb = self.context.append_basic_block(function, "merge");

        self.builder
            .build_conditional_branch(cond_int, then_bb, else_bb)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Then branch
        self.builder.position_at_end(then_bb);
        let then_val = self.lower_expr(&if_expr.then_branch)?;
        let then_terminated = self.current_block_terminated();
        if !then_terminated {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }
        let then_bb = self.builder.get_insert_block().unwrap();

        // Else branch
        self.builder.position_at_end(else_bb);
        let else_val = if let Some(else_branch) = &if_expr.else_branch {
            self.lower_expr(else_branch)?
        } else {
            None
        };
        let else_terminated = self.current_block_terminated();
        if !else_terminated {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }
        let else_bb = self.builder.get_insert_block().unwrap();

        // If both branches terminated (e.g. both returned), merge is unreachable
        if then_terminated && else_terminated {
            // Remove the empty merge block — it's dead code
            unsafe {
                merge_bb.delete().ok();
            }
            return Ok(None);
        }

        // Merge
        self.builder.position_at_end(merge_bb);

        // Create phi node if both branches produce values and neither terminated early
        if !then_terminated && !else_terminated {
            if let (Some(tv), Some(ev)) = (then_val, else_val) {
                let phi = self
                    .builder
                    .build_phi(tv.get_type(), "if_val")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                phi.add_incoming(&[(&tv, then_bb), (&ev, else_bb)]);
                return Ok(Some(phi.as_basic_value()));
            }
        }
        Ok(None)
    }

    // ========================================================================
    // Statement Lowering
    // ========================================================================

    /// Lower a statement.
    fn lower_stmt(&mut self, stmt: &HirStmt) -> Result<()> {
        match stmt {
            HirStmt::Val(_) | HirStmt::Var(_) => {
                let (pat, init) = match stmt {
                    HirStmt::Val(v) => (&v.pat, &v.init),
                    HirStmt::Var(v) => (&v.pat, &v.init),
                    _ => unreachable!(),
                };

                if let HirPat::Var(sym) = pat {
                    let name = self.sym(*sym).to_string();
                    let val = self.lower_expr(init)?.unwrap();

                    let function = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();

                    let ty = val.get_type();
                    let alloca = self.create_entry_alloca(function, &name, ty);
                    self.builder
                        .build_store(alloca, val)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.named_values.insert(name, (alloca, ty));
                }
                Ok(())
            }
            HirStmt::Assign(assign) => {
                if let HirExpr::Var(sym) = &assign.lhs {
                    let name = self.sym(*sym).to_string();
                    let val = self.lower_expr(&assign.rhs)?.unwrap();
                    if let Some((alloca, _ty)) = self.named_values.get(name.as_str()) {
                        self.builder
                            .build_store(*alloca, val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }
                Ok(())
            }
            HirStmt::Expr(expr) => {
                self.lower_expr(expr)?;
                Ok(())
            }
            HirStmt::Return(val) => {
                if let Some(expr) = val {
                    let v = self.lower_expr(expr)?;
                    if let Some(v) = v {
                        self.builder
                            .build_return(Some(&v))
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                } else {
                    self.builder
                        .build_return(None)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
                Ok(())
            }
            HirStmt::Break(_) => {
                // Break requires loop context tracking (future)
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadol::lower::lower_file;

    #[test]
    fn test_lower_simple_function() {
        let source = r#"
fun add(a: i64, b: i64) -> i64 {
    return a + b
}
"#;
        let (hir, ctx) = lower_file(source).expect("parse failed");
        let context = Context::create();
        let module = context.create_module("test");
        let mut lowering = HirLowering::new(&context, &module, &ctx.symbols);
        lowering.lower_module(&hir).expect("lowering failed");

        let ir = module.print_to_string().to_string();
        assert!(ir.contains("define"), "Should contain function definition");
        assert!(ir.contains("add"), "Should contain function name 'add'");
    }

    #[test]
    fn test_lower_gene_to_struct() {
        let source = r#"
gene container.exists {
    container has identity
    container has status
}

exegesis {
    A container is the fundamental unit.
}
"#;
        let (hir, ctx) = lower_file(source).expect("parse failed");
        let context = Context::create();
        let module = context.create_module("test");
        let mut lowering = HirLowering::new(&context, &module, &ctx.symbols);
        lowering.lower_module(&hir).expect("lowering failed");

        // Verify struct was created
        assert!(
            !lowering.struct_types.is_empty(),
            "Should generate struct types"
        );
    }

    #[test]
    fn test_lower_hello_native() {
        let source = r#"
fun main() {
    println("Hello from native DOL!")
}
"#;
        let (hir, ctx) = lower_file(source).expect("parse failed");
        let context = Context::create();
        let module = context.create_module("test");
        let mut lowering = HirLowering::new(&context, &module, &ctx.symbols);
        lowering.lower_module(&hir).expect("lowering failed");

        let ir = module.print_to_string().to_string();
        assert!(ir.contains("@vudo_println"), "Should call vudo_println");
        assert!(ir.contains("i64 22"), "Should pass string length");
        assert!(ir.contains("define i32 @main"), "main should return i32");
        assert!(ir.contains("ret i32 0"), "main should return 0");
    }

    #[test]
    fn test_lower_arithmetic_functions() {
        let source = r#"
fun add(a: i64, b: i64) -> i64 {
    return a + b
}

fun subtract(a: i64, b: i64) -> i64 {
    return a - b
}

fun abs(x: i64) -> i64 {
    if x < 0 {
        return -x
    }
    return x
}

fun main() {
    let sum = add(10, 20)
    let diff = subtract(50, 17)
}
"#;
        let (hir, ctx) = lower_file(source).expect("parse failed");
        let context = Context::create();
        let module = context.create_module("test");
        let mut lowering = HirLowering::new(&context, &module, &ctx.symbols);
        lowering.lower_module(&hir).expect("lowering failed");

        let ir = module.print_to_string().to_string();
        assert!(ir.contains("define i64 @add"), "Should have add function");
        assert!(
            ir.contains("define i64 @subtract"),
            "Should have subtract function"
        );
        assert!(ir.contains("define i64 @abs"), "Should have abs function");
        assert!(
            ir.contains("call i64 @add(i64 10, i64 20)"),
            "main should call add"
        );
        // No dead ret void after early returns
        assert!(
            !ir.contains("ret i64 %neg\n  ret void"),
            "No dead ret void after return"
        );
    }

    #[test]
    fn test_lower_string_concat() {
        let source = r#"
fun main() {
    println("Hello, " + "World!")
}
"#;
        let (hir, ctx) = lower_file(source).expect("parse failed");
        let context = Context::create();
        let module = context.create_module("test");
        let mut lowering = HirLowering::new(&context, &module, &ctx.symbols);
        lowering.lower_module(&hir).expect("lowering failed");

        let ir = module.print_to_string().to_string();
        assert!(
            ir.contains("@vudo_string_concat"),
            "Should call string concat"
        );
        assert!(ir.contains("@vudo_println"), "Should call println");
    }

    #[test]
    fn test_lower_if_with_returns() {
        let source = r#"
fun sign(x: i64) -> string {
    if x > 0 {
        return "positive"
    }
    if x < 0 {
        return "negative"
    }
    return "zero"
}
"#;
        let (hir, ctx) = lower_file(source).expect("parse failed");
        let context = Context::create();
        let module = context.create_module("test");
        let mut lowering = HirLowering::new(&context, &module, &ctx.symbols);
        lowering.lower_module(&hir).expect("lowering failed");

        let ir = module.print_to_string().to_string();
        assert!(
            ir.contains("define { ptr, i64 } @sign"),
            "sign returns fat string"
        );
        assert!(
            ir.contains("ret { ptr, i64 }"),
            "Should return fat string structs"
        );
        // Then-branches should terminate with ret, no dead br after
        assert!(
            !ir.contains("ret { ptr, i64 }\n  br"),
            "No dead br after ret"
        );
    }

    #[test]
    fn test_lower_gen_type() {
        let source = r#"
gen Color {
    has name
    has active
}

docs {
    A color with name and active status.
}
"#;
        let (hir, ctx) = lower_file(source).expect("parse failed");
        let context = Context::create();
        let module = context.create_module("test");
        let mut lowering = HirLowering::new(&context, &module, &ctx.symbols);
        lowering.lower_module(&hir).expect("lowering failed");

        assert!(
            !lowering.struct_types.is_empty(),
            "Should generate struct types"
        );
    }
}
