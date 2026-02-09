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
    HirBinaryOp, HirBlockExpr, HirDecl, HirExpr, HirFunctionDecl, HirIfExpr,
    HirLiteral, HirModule, HirNamedType, HirPat, HirStmt,
    HirStatementKind, HirType, HirTypeDef, HirTypeDecl, HirUnaryOp, Symbol, SymbolTable,
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

    /// Named values in the current scope (variable name → alloca pointer)
    named_values: HashMap<String, PointerValue<'ctx>>,

    /// Named struct types already generated
    struct_types: HashMap<String, inkwell::types::StructType<'ctx>>,
}

impl<'a, 'ctx> HirLowering<'a, 'ctx> {
    /// Create a new HIR lowering engine.
    pub fn new(
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        symbols: &'a SymbolTable,
    ) -> Self {
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
                    global.set_initializer(
                        &self.context.i32_type().const_int(i as u64, false),
                    );
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
            n if n.contains("name") || n.contains("label") || n.contains("description")
                || n.contains("title") || n.contains("text") || n.contains("message") =>
            {
                self.type_mapper.string_type().into()
            }
            n if n.contains("count") || n.contains("size") || n.contains("length")
                || n.contains("index") || n.contains("id") || n.contains("port") =>
            {
                self.context.i64_type().into()
            }
            n if n.contains("active") || n.contains("enabled") || n.contains("alive")
                || n.contains("visible") || n.contains("ready") =>
            {
                self.context.bool_type().into()
            }
            n if n.contains("x") || n.contains("y") || n.contains("z")
                || n.contains("width") || n.contains("height") || n.contains("ratio") =>
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
    pub fn lower_function_decl(
        &mut self,
        decl: &HirFunctionDecl,
    ) -> Result<FunctionValue<'ctx>> {
        let name = self.sym(decl.name).to_string();

        // Map parameter types
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = decl
            .params
            .iter()
            .map(|p| self.map_hir_type(&p.ty).map(|t| t.into()))
            .collect::<Result<Vec<_>>>()?;

        // Map return type
        let fn_type = match &decl.return_type {
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
                        let alloca = self.create_entry_alloca(
                            function,
                            &param_name,
                            p.get_type(),
                        );
                        self.builder.build_store(alloca, p)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.named_values.insert(param_name, alloca);
                    }
                }
            }

            // Generate body expression
            let is_void = matches!(&decl.return_type,
                HirType::Named(n) if self.sym(n.name) == "()" || self.sym(n.name) == "Unit"
            ) || matches!(&decl.return_type, HirType::Tuple(elems) if elems.is_empty());

            match self.lower_expr(body) {
                Ok(Some(val)) if !is_void => {
                    self.builder.build_return(Some(&val))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
                _ => {
                    self.builder.build_return(None)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }
        }

        Ok(function)
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
                    Ok(self.context.struct_type(&[ptr.into(), len.into()], false).into())
                }
            }
            HirType::Function(_) => {
                // Function types are pointers
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            HirType::Ref(_) => {
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
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
            HirExpr::MethodCall(_) => {
                // Method calls not yet supported in native codegen
                Ok(Some(self.context.i64_type().const_int(0, false).into()))
            }
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
                Ok(Some(self.context.ptr_type(AddressSpace::default())
                    .const_null().into()))
            }
        }
    }

    /// Lower a literal value.
    fn lower_literal(&self, lit: &HirLiteral) -> Result<BasicValueEnum<'ctx>> {
        match lit {
            HirLiteral::Bool(b) => Ok(self
                .context
                .bool_type()
                .const_int(*b as u64, false)
                .into()),
            HirLiteral::Int(n) => Ok(self
                .context
                .i64_type()
                .const_int(*n as u64, *n < 0)
                .into()),
            HirLiteral::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            HirLiteral::String(s) => {
                // Create a global string constant
                let global = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(global.as_pointer_value().into())
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
        if let Some(alloca) = self.named_values.get(name) {
            let val = self.builder.build_load(
                self.context.i64_type(), // TODO: track actual types
                *alloca,
                name,
            ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(val)
        } else {
            // Could be a global or function reference
            if let Some(func) = self.module.get_function(name) {
                Ok(func.as_global_value().as_pointer_value().into())
            } else {
                Err(CodegenError::LlvmError(format!("undefined variable: {}", name)))
            }
        }
    }

    /// Lower a binary expression.
    fn lower_binary(
        &mut self,
        bin: &metadol::hir::HirBinaryExpr,
    ) -> Result<BasicValueEnum<'ctx>> {
        let lhs = self.lower_expr(&bin.left)?.unwrap();
        let rhs = self.lower_expr(&bin.right)?.unwrap();

        // Integer operations (default for now)
        let lhs_int = lhs.into_int_value();
        let rhs_int = rhs.into_int_value();

        let result = match bin.op {
            HirBinaryOp::Add => self.builder.build_int_add(lhs_int, rhs_int, "add"),
            HirBinaryOp::Sub => self.builder.build_int_sub(lhs_int, rhs_int, "sub"),
            HirBinaryOp::Mul => self.builder.build_int_mul(lhs_int, rhs_int, "mul"),
            HirBinaryOp::Div => self.builder.build_int_signed_div(lhs_int, rhs_int, "div"),
            HirBinaryOp::Mod => self.builder.build_int_signed_rem(lhs_int, rhs_int, "rem"),
            HirBinaryOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, lhs_int, rhs_int, "eq"),
            HirBinaryOp::Ne => self.builder.build_int_compare(IntPredicate::NE, lhs_int, rhs_int, "ne"),
            HirBinaryOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, lhs_int, rhs_int, "lt"),
            HirBinaryOp::Le => self.builder.build_int_compare(IntPredicate::SLE, lhs_int, rhs_int, "le"),
            HirBinaryOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, lhs_int, rhs_int, "gt"),
            HirBinaryOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, lhs_int, rhs_int, "ge"),
            HirBinaryOp::And => self.builder.build_and(lhs_int, rhs_int, "and"),
            HirBinaryOp::Or => self.builder.build_or(lhs_int, rhs_int, "or"),
        }
        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(result.into())
    }

    /// Lower a unary expression.
    fn lower_unary(
        &mut self,
        un: &metadol::hir::HirUnaryExpr,
    ) -> Result<BasicValueEnum<'ctx>> {
        let operand = self.lower_expr(&un.operand)?.unwrap();
        let int_val = operand.into_int_value();

        let result = match un.op {
            HirUnaryOp::Neg => self.builder.build_int_neg(int_val, "neg"),
            HirUnaryOp::Not => self.builder.build_not(int_val, "not"),
        }
        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(result.into())
    }

    /// Lower a function call.
    fn lower_call(
        &mut self,
        call: &metadol::hir::HirCallExpr,
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
        // Resolve function name
        let func_name = match &call.func {
            HirExpr::Var(sym) => self.sym(*sym).to_string(),
            _ => return Err(CodegenError::LlvmError("indirect calls not supported".into())),
        };

        let function = self
            .module
            .get_function(&func_name)
            .ok_or_else(|| CodegenError::LlvmError(format!("undefined function: {}", func_name)))?;

        // Lower arguments
        let args: Vec<BasicMetadataValueEnum<'ctx>> = call
            .args
            .iter()
            .map(|a| {
                self.lower_expr(a)
                    .map(|v| v.unwrap().into())
            })
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
    fn lower_block(
        &mut self,
        block: &HirBlockExpr,
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
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
    fn lower_if(
        &mut self,
        if_expr: &HirIfExpr,
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
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
        self.builder
            .build_unconditional_branch(merge_bb)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let then_bb = self.builder.get_insert_block().unwrap();

        // Else branch
        self.builder.position_at_end(else_bb);
        let else_val = if let Some(else_branch) = &if_expr.else_branch {
            self.lower_expr(else_branch)?
        } else {
            None
        };
        self.builder
            .build_unconditional_branch(merge_bb)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let else_bb = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge_bb);

        // Create phi node if both branches produce values
        if let (Some(tv), Some(ev)) = (then_val, else_val) {
            let phi = self
                .builder
                .build_phi(tv.get_type(), "if_val")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            phi.add_incoming(&[(&tv, then_bb), (&ev, else_bb)]);
            Ok(Some(phi.as_basic_value()))
        } else {
            Ok(None)
        }
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

                    let alloca = self.create_entry_alloca(function, &name, val.get_type());
                    self.builder.build_store(alloca, val)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.named_values.insert(name, alloca);
                }
                Ok(())
            }
            HirStmt::Assign(assign) => {
                if let HirExpr::Var(sym) = &assign.lhs {
                    let name = self.sym(*sym).to_string();
                    let val = self.lower_expr(&assign.rhs)?.unwrap();
                    if let Some(alloca) = self.named_values.get(name.as_str()) {
                        self.builder.build_store(*alloca, val)
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
                        self.builder.build_return(Some(&v))
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        self.builder.build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                } else {
                    self.builder.build_return(None)
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
        assert!(!lowering.struct_types.is_empty(), "Should generate struct types");
    }
}
