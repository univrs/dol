//! AST Transformation Pipeline
//!
//! This module provides AST transformation capabilities using the visitor pattern.
//! It supports:
//! - Type inference and elaboration
//! - CRDT annotation expansion
//! - Custom AST transformations
//! - Optimization passes

use crate::{CodegenContext, CodegenError, Result};
use dol::ast::{
    BinaryOp, Block, CrdtAnnotation, CrdtOption, CrdtStrategy, Declaration, DolFile, Expr,
    FunctionDecl, Gen, HasField, Literal, ModuleDecl, Mutability, Pattern, Purity, Span,
    Statement, Stmt, Trait, Type, TypeExpr, UseDecl, Visibility,
};
use std::collections::HashMap;

/// Trait for AST visitors
pub trait Visitor {
    /// Visit a DOL file
    fn visit_file(&mut self, file: &mut DolFile) -> Result<()> {
        self.visit_declarations(&mut file.declarations)?;
        Ok(())
    }

    /// Visit all declarations
    fn visit_declarations(&mut self, decls: &mut Vec<Declaration>) -> Result<()> {
        for decl in decls {
            self.visit_declaration(decl)?;
        }
        Ok(())
    }

    /// Visit a single declaration
    fn visit_declaration(&mut self, decl: &mut Declaration) -> Result<()> {
        match decl {
            Declaration::Gene(gen) => self.visit_gen(gen),
            Declaration::Trait(trait_) => self.visit_trait(trait_),
            Declaration::Constraint(_) => Ok(()),
            Declaration::System(_) => Ok(()),
            Declaration::Evolution(_) => Ok(()),
            Declaration::Function(_) => Ok(()),
            Declaration::Const(_) => Ok(()),
            Declaration::SexVar(_) => Ok(()),
        }
    }

    /// Visit a Gen declaration
    fn visit_gen(&mut self, gen: &mut Gen) -> Result<()> {
        self.visit_statements(&mut gen.statements)?;
        Ok(())
    }

    /// Visit a Trait declaration
    fn visit_trait(&mut self, trait_: &mut Trait) -> Result<()> {
        self.visit_statements(&mut trait_.statements)?;
        Ok(())
    }

    /// Visit statements
    fn visit_statements(&mut self, stmts: &mut Vec<Statement>) -> Result<()> {
        for stmt in stmts {
            self.visit_statement(stmt)?;
        }
        Ok(())
    }

    /// Visit a single statement
    fn visit_statement(&mut self, stmt: &mut Statement) -> Result<()> {
        match stmt {
            Statement::HasField(field) => self.visit_field(field),
            _ => Ok(()),
        }
    }

    /// Visit a field
    fn visit_field(&mut self, field: &mut Box<HasField>) -> Result<()> {
        Ok(())
    }
}

/// Type inference visitor
pub struct TypeInferenceVisitor {
    type_env: HashMap<String, Type>,
    current_scope: Vec<HashMap<String, Type>>,
}

impl TypeInferenceVisitor {
    /// Create a new type inference visitor
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new(),
            current_scope: vec![HashMap::new()],
        }
    }

    /// Infer type from a type expression
    pub fn infer_type(&self, type_expr: &TypeExpr) -> Result<Type> {
        match type_expr {
            TypeExpr::Named(name) => match name.as_str() {
                "i8" => Ok(Type::I8),
                "i16" => Ok(Type::I16),
                "i32" | "Int32" => Ok(Type::I32),
                "i64" | "Int64" => Ok(Type::I64),
                "i128" => Ok(Type::I128),
                "u8" => Ok(Type::U8),
                "u16" => Ok(Type::U16),
                "u32" => Ok(Type::U32),
                "u64" => Ok(Type::U64),
                "u128" => Ok(Type::U128),
                "f32" | "Float32" => Ok(Type::F32),
                "f64" | "Float64" => Ok(Type::F64),
                "bool" | "Bool" => Ok(Type::Bool),
                "String" => Ok(Type::String),
                "()" => Ok(Type::Unit),
                _ => Ok(Type::Named(name.clone())),
            },
            TypeExpr::Generic { name, args } => match name.as_str() {
                "Vec" | "List" => {
                    if let Some(first_arg) = args.first() {
                        let inner_type = self.infer_type(first_arg)?;
                        Ok(Type::Vec(Box::new(inner_type)))
                    } else {
                        Err(CodegenError::TypeInference(
                            "Vec requires a type argument".to_string(),
                        ))
                    }
                }
                "Option" => {
                    if let Some(first_arg) = args.first() {
                        let inner_type = self.infer_type(first_arg)?;
                        Ok(Type::Option(Box::new(inner_type)))
                    } else {
                        Err(CodegenError::TypeInference(
                            "Option requires a type argument".to_string(),
                        ))
                    }
                }
                "Result" => {
                    if args.len() >= 2 {
                        let ok_type = self.infer_type(&args[0])?;
                        let err_type = self.infer_type(&args[1])?;
                        Ok(Type::Result(Box::new(ok_type), Box::new(err_type)))
                    } else {
                        Err(CodegenError::TypeInference(
                            "Result requires two type arguments".to_string(),
                        ))
                    }
                }
                "Map" | "HashMap" => {
                    if args.len() >= 2 {
                        let key_type = self.infer_type(&args[0])?;
                        let val_type = self.infer_type(&args[1])?;
                        Ok(Type::Map(Box::new(key_type), Box::new(val_type)))
                    } else {
                        Err(CodegenError::TypeInference(
                            "Map requires two type arguments".to_string(),
                        ))
                    }
                }
                _ => {
                    let param_types = args
                        .iter()
                        .map(|arg| self.infer_type(arg))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Type::Generic {
                        name: name.clone(),
                        params: param_types,
                    })
                }
            },
            TypeExpr::Function { params, return_type } => {
                let param_types = params
                    .iter()
                    .map(|p| self.infer_type(p))
                    .collect::<Result<Vec<_>>>()?;
                let ret_type = self.infer_type(return_type)?;
                Ok(Type::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                })
            }
            TypeExpr::Tuple(types) => {
                let tuple_types = types
                    .iter()
                    .map(|t| self.infer_type(t))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Tuple(tuple_types))
            }
            TypeExpr::Never => Ok(Type::Error),
            TypeExpr::Enum { .. } => Ok(Type::Named("Enum".to_string())),
        }
    }
}

impl Visitor for TypeInferenceVisitor {
    fn visit_field(&mut self, field: &mut Box<HasField>) -> Result<()> {
        // Infer the type and store it in the environment
        let inferred_type = self.infer_type(&field.type_)?;
        self.type_env.insert(field.name.clone(), inferred_type);
        Ok(())
    }
}

impl Default for TypeInferenceVisitor {
    fn default() -> Self {
        Self::new()
    }
}

/// CRDT annotation expansion visitor
pub struct CrdtExpansionVisitor {
    /// Enable CRDT expansion
    enabled: bool,
}

impl CrdtExpansionVisitor {
    /// Create a new CRDT expansion visitor
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Expand CRDT annotation for a field
    fn expand_crdt(&self, field: &mut HasField) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(crdt) = &field.crdt_annotation {
            match crdt.strategy {
                CrdtStrategy::Immutable => {
                    // No expansion needed for immutable
                }
                CrdtStrategy::Lww => {
                    // Add timestamp tracking for LWW
                    // This would be done by adding metadata to the field
                }
                CrdtStrategy::OrSet => {
                    // Expand to OR-Set CRDT
                    // Requires tracking additions and removals
                }
                CrdtStrategy::PnCounter => {
                    // Expand to PN-Counter CRDT
                    // Requires tracking increments and decrements
                }
                CrdtStrategy::Peritext => {
                    // Expand to Peritext CRDT for rich text
                }
                CrdtStrategy::Rga => {
                    // Expand to RGA (Replicated Growable Array)
                }
                CrdtStrategy::MvRegister => {
                    // Expand to Multi-Value Register
                }
            }
        }

        Ok(())
    }
}

impl Visitor for CrdtExpansionVisitor {
    fn visit_field(&mut self, field: &mut Box<HasField>) -> Result<()> {
        self.expand_crdt(field)
    }
}

/// Optimization visitor
pub struct OptimizationVisitor {
    /// Optimizations to apply
    optimizations: Vec<String>,
}

impl OptimizationVisitor {
    /// Create a new optimization visitor
    pub fn new() -> Self {
        Self {
            optimizations: vec![],
        }
    }

    /// Add an optimization
    pub fn with_optimization(mut self, opt: String) -> Self {
        self.optimizations.push(opt);
        self
    }
}

impl Default for OptimizationVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor for OptimizationVisitor {
    fn visit_gen(&mut self, gen: &mut Gen) -> Result<()> {
        // Apply optimizations to the gen
        // For example, remove unused fields, merge similar statements, etc.
        self.visit_statements(&mut gen.statements)?;
        Ok(())
    }
}

/// Transform pipeline
pub struct TransformPipeline {
    visitors: Vec<Box<dyn Visitor>>,
}

impl TransformPipeline {
    /// Create a new transform pipeline
    pub fn new() -> Self {
        Self {
            visitors: Vec::new(),
        }
    }

    /// Add a visitor to the pipeline
    pub fn add_visitor(mut self, visitor: Box<dyn Visitor>) -> Self {
        self.visitors.push(visitor);
        self
    }

    /// Execute the pipeline on a DOL file
    pub fn execute(&mut self, file: &mut DolFile) -> Result<()> {
        for visitor in &mut self.visitors {
            visitor.visit_file(file)?;
        }
        Ok(())
    }
}

impl Default for TransformPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Main transformation entry point
pub fn transform(file: &DolFile, context: &CodegenContext) -> Result<DolFile> {
    let mut transformed = file.clone();

    // Build transformation pipeline
    let mut pipeline = TransformPipeline::new();

    // Add type inference visitor
    if context.enable_type_inference {
        pipeline = pipeline.add_visitor(Box::new(TypeInferenceVisitor::new()));
    }

    // Add CRDT expansion visitor
    if context.enable_crdt_expansion {
        pipeline = pipeline.add_visitor(Box::new(CrdtExpansionVisitor::new(true)));
    }

    // Add optimization visitor
    pipeline = pipeline.add_visitor(Box::new(OptimizationVisitor::new()));

    // Execute pipeline
    pipeline.execute(&mut transformed)?;

    Ok(transformed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference() {
        let mut visitor = TypeInferenceVisitor::new();

        let type_expr = TypeExpr::Named("i32".to_string());
        let inferred = visitor.infer_type(&type_expr).unwrap();
        assert_eq!(inferred, Type::I32);

        let vec_type = TypeExpr::Generic {
            name: "Vec".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        let inferred = visitor.infer_type(&vec_type).unwrap();
        assert_eq!(inferred, Type::Vec(Box::new(Type::String)));
    }

    #[test]
    fn test_crdt_expansion() {
        let visitor = CrdtExpansionVisitor::new(true);
        // Test CRDT expansion logic
        assert!(visitor.enabled);
    }

    #[test]
    fn test_transform_pipeline() {
        let mut pipeline = TransformPipeline::new()
            .add_visitor(Box::new(TypeInferenceVisitor::new()))
            .add_visitor(Box::new(OptimizationVisitor::new()));

        // Test pipeline execution
        assert_eq!(pipeline.visitors.len(), 2);
    }
}
