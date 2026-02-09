//! Template Engine for DOL Code Generation
//!
//! This module provides template-based code generation using Handlebars and Tera
//! template engines. It handles:
//! - AST-to-template data model mapping
//! - Template rendering with custom helpers
//! - DOL-specific template functions

use crate::{CodegenContext, CodegenError, Result, Target};
use dol::ast::{Declaration, DolFile, Gen, HasField, Statement, Trait, TypeExpr};
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use heck::{ToKebabCase, ToPascalCase, ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Template data model for a DOL file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateData {
    /// Module name
    pub module_name: String,
    /// File-level declarations
    pub declarations: Vec<DeclarationData>,
    /// Imports/uses
    pub imports: Vec<ImportData>,
    /// Target language
    pub target: String,
    /// Include documentation
    pub include_docs: bool,
}

/// Template data for a single declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclarationData {
    /// Declaration type (gen, trait, rule, system, evo)
    pub kind: String,
    /// Declaration name
    pub name: String,
    /// PascalCase name (for types)
    pub type_name: String,
    /// snake_case name (for variables)
    pub var_name: String,
    /// Documentation/exegesis
    pub docs: String,
    /// Fields (for gens)
    pub fields: Vec<FieldData>,
    /// Methods/functions
    pub methods: Vec<MethodData>,
    /// Uses/dependencies (for traits/systems)
    pub uses: Vec<String>,
    /// Visibility
    pub visibility: String,
}

/// Template data for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldData {
    /// Field name
    pub name: String,
    /// snake_case name
    pub var_name: String,
    /// Field type
    pub type_: String,
    /// Optional default value
    pub default: Option<String>,
    /// Optional CRDT strategy
    pub crdt_strategy: Option<String>,
    /// CRDT options
    pub crdt_options: HashMap<String, String>,
    /// Documentation
    pub docs: String,
    /// Is personal data (GDPR)
    pub is_personal: bool,
}

/// Template data for a method/function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodData {
    /// Method name
    pub name: String,
    /// Parameters
    pub params: Vec<ParamData>,
    /// Return type
    pub return_type: Option<String>,
    /// Is side-effecting (sex)
    pub is_sex: bool,
    /// Documentation
    pub docs: String,
}

/// Template data for a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamData {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_: String,
}

/// Template data for an import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportData {
    /// Import path
    pub path: String,
    /// Items being imported
    pub items: Vec<String>,
}

/// Template engine wrapper
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    target: Target,
}

impl TemplateEngine {
    /// Create a new template engine for the given target
    pub fn new(target: Target) -> Self {
        let mut handlebars = Handlebars::new();

        // Register custom helpers
        register_helpers(&mut handlebars);

        // Register built-in templates
        register_templates(&mut handlebars, target);

        Self {
            handlebars,
            target,
        }
    }

    /// Generate code from a DOL file
    pub fn generate(&self, file: &DolFile, context: &CodegenContext) -> Result<String> {
        let data = self.create_template_data(file, context)?;
        self.render(&data)
    }

    /// Create template data from a DOL file
    fn create_template_data(
        &self,
        file: &DolFile,
        context: &CodegenContext,
    ) -> Result<TemplateData> {
        let module_name = context
            .module_name
            .clone()
            .unwrap_or_else(|| "generated".to_string());

        let declarations = file
            .declarations
            .iter()
            .map(|decl| self.map_declaration(decl, context))
            .collect::<Result<Vec<_>>>()?;

        let imports = file
            .uses
            .iter()
            .map(|use_decl| ImportData {
                path: use_decl.path.join("."),
                items: vec![], // TODO: Extract items from UseItems
            })
            .collect();

        Ok(TemplateData {
            module_name,
            declarations,
            imports,
            target: self.target.language().to_string(),
            include_docs: context.include_docs,
        })
    }

    /// Map a DOL declaration to template data
    fn map_declaration(
        &self,
        decl: &Declaration,
        context: &CodegenContext,
    ) -> Result<DeclarationData> {
        match decl {
            Declaration::Gene(gen) => self.map_gen(gen, context),
            Declaration::Trait(trait_) => self.map_trait(trait_, context),
            _ => Ok(DeclarationData {
                kind: "other".to_string(),
                name: decl.name().to_string(),
                type_name: to_pascal_case(decl.name()),
                var_name: to_snake_case(decl.name()),
                docs: decl.exegesis().to_string(),
                fields: vec![],
                methods: vec![],
                uses: vec![],
                visibility: "public".to_string(),
            }),
        }
    }

    /// Map a Gen declaration to template data
    fn map_gen(&self, gen: &Gen, context: &CodegenContext) -> Result<DeclarationData> {
        let fields = gen
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::HasField(field) = stmt {
                    Some(self.map_field(field.as_ref(), context))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(DeclarationData {
            kind: "gen".to_string(),
            name: gen.name.clone(),
            type_name: to_pascal_case(&gen.name),
            var_name: to_snake_case(&gen.name),
            docs: gen.exegesis.clone(),
            fields,
            methods: vec![],
            uses: vec![],
            visibility: format!("{:?}", gen.visibility).to_lowercase(),
        })
    }

    /// Map a Trait declaration to template data
    fn map_trait(&self, trait_: &Trait, context: &CodegenContext) -> Result<DeclarationData> {
        let uses = trait_
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Uses { reference, .. } = stmt {
                    Some(reference.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(DeclarationData {
            kind: "trait".to_string(),
            name: trait_.name.clone(),
            type_name: to_pascal_case(&trait_.name),
            var_name: to_snake_case(&trait_.name),
            docs: trait_.exegesis.clone(),
            fields: vec![],
            methods: vec![],
            uses,
            visibility: format!("{:?}", trait_.visibility).to_lowercase(),
        })
    }

    /// Map a field to template data
    fn map_field(&self, field: &HasField, _context: &CodegenContext) -> Result<FieldData> {
        let crdt_strategy = field
            .crdt_annotation
            .as_ref()
            .map(|ann| ann.strategy.as_str().to_string());

        let crdt_options = field
            .crdt_annotation
            .as_ref()
            .map(|ann| {
                ann.options
                    .iter()
                    .map(|opt| (opt.key.clone(), format!("{:?}", opt.value)))
                    .collect()
            })
            .unwrap_or_default();

        Ok(FieldData {
            name: field.name.clone(),
            var_name: to_snake_case(&field.name),
            type_: map_type_expr(&field.type_, self.target),
            default: field.default.as_ref().map(|_| "None".to_string()),
            crdt_strategy,
            crdt_options,
            docs: "".to_string(),
            is_personal: field.personal,
        })
    }

    /// Render template data to code
    fn render(&self, data: &TemplateData) -> Result<String> {
        let template_name = match self.target {
            Target::Rust => "rust_module",
            Target::TypeScript => "typescript_module",
            Target::Wit => "wit_module",
            Target::Python => "python_module",
            Target::JsonSchema => "json_schema",
        };

        self.handlebars
            .render(template_name, data)
            .map_err(|e| CodegenError::Handlebars(e.to_string()))
    }
}

/// Register custom Handlebars helpers
fn register_helpers(hb: &mut Handlebars) {
    // Case conversion helpers
    handlebars_helper!(pascal_case: |s: String| to_pascal_case(&s));
    handlebars_helper!(snake_case: |s: String| to_snake_case(&s));
    handlebars_helper!(kebab_case: |s: String| to_kebab_case(&s));
    handlebars_helper!(upper_case: |s: String| s.to_uppercase());
    handlebars_helper!(lower_case: |s: String| s.to_lowercase());

    hb.register_helper("pascal_case", Box::new(pascal_case));
    hb.register_helper("snake_case", Box::new(snake_case));
    hb.register_helper("kebab_case", Box::new(kebab_case));
    hb.register_helper("upper_case", Box::new(upper_case));
    hb.register_helper("lower_case", Box::new(lower_case));

    // DOL-specific helper
    hb.register_helper("type_annotation", Box::new(type_annotation_helper));
    hb.register_helper("default_value", Box::new(default_value_helper));
}

/// Custom helper for type annotations
fn type_annotation_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let type_str = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or_else(|| handlebars::RenderError::new("type parameter required"))?;

    let target = h
        .param(1)
        .and_then(|v| v.value().as_str())
        .unwrap_or("rust");

    let result = match target {
        "rust" => format!(": {}", type_str),
        "typescript" => format!(": {}", type_str),
        "python" => format!(": {}", type_str),
        _ => String::new(),
    };

    out.write(&result)?;
    Ok(())
}

/// Custom helper for default values
fn default_value_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(default) = h.param(0).and_then(|v| v.value().as_str()) {
        out.write(&format!(" = {}", default))?;
    }
    Ok(())
}

/// Register built-in templates
fn register_templates(hb: &mut Handlebars, target: Target) {
    match target {
        Target::Rust => {
            let template = include_str!("../templates/rust_module.hbs");
            hb.register_template_string("rust_module", template)
                .ok();
        }
        Target::TypeScript => {
            let template = include_str!("../templates/typescript_module.hbs");
            hb.register_template_string("typescript_module", template)
                .ok();
        }
        Target::Wit => {
            let template = include_str!("../templates/wit_module.hbs");
            hb.register_template_string("wit_module", template)
                .ok();
        }
        Target::Python => {
            let template = include_str!("../templates/python_module.hbs");
            hb.register_template_string("python_module", template)
                .ok();
        }
        Target::JsonSchema => {
            let template = include_str!("../templates/json_schema.hbs");
            hb.register_template_string("json_schema", template)
                .ok();
        }
    }
}

/// Map a DOL type expression to a target language type
pub fn map_type_expr(type_expr: &TypeExpr, target: Target) -> String {
    match target {
        Target::Rust => map_type_expr_rust(type_expr),
        Target::TypeScript => map_type_expr_typescript(type_expr),
        Target::Wit => map_type_expr_wit(type_expr),
        Target::Python => map_type_expr_python(type_expr),
        Target::JsonSchema => map_type_expr_json_schema(type_expr),
    }
}

/// Map DOL type to Rust type
fn map_type_expr_rust(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            "String" => "String".to_string(),
            "Int32" | "i32" => "i32".to_string(),
            "Int64" | "i64" => "i64".to_string(),
            "Float32" | "f32" => "f32".to_string(),
            "Float64" | "f64" => "f64".to_string(),
            "Bool" | "bool" => "bool".to_string(),
            other => to_pascal_case(other),
        },
        TypeExpr::Generic { name, args } => {
            let arg_strs: Vec<_> = args.iter().map(map_type_expr_rust).collect();
            match name.as_str() {
                "Vec" | "List" => format!("Vec<{}>", arg_strs.join(", ")),
                "Option" => format!("Option<{}>", arg_strs.join(", ")),
                "Result" => format!("Result<{}>", arg_strs.join(", ")),
                "Map" | "HashMap" => format!("HashMap<{}>", arg_strs.join(", ")),
                "Set" | "HashSet" => format!("HashSet<{}>", arg_strs.join(", ")),
                other => format!("{}<{}>", to_pascal_case(other), arg_strs.join(", ")),
            }
        }
        TypeExpr::Function { params, return_type } => {
            let param_strs: Vec<_> = params.iter().map(map_type_expr_rust).collect();
            let ret_str = map_type_expr_rust(return_type);
            format!("fn({}) -> {}", param_strs.join(", "), ret_str)
        }
        TypeExpr::Tuple(types) => {
            let type_strs: Vec<_> = types.iter().map(map_type_expr_rust).collect();
            format!("({})", type_strs.join(", "))
        }
        TypeExpr::Never => "!".to_string(),
        TypeExpr::Enum { .. } => "Enum".to_string(),
    }
}

/// Map DOL type to TypeScript type
fn map_type_expr_typescript(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            "String" => "string".to_string(),
            "Int32" | "Int64" | "i32" | "i64" | "Float32" | "Float64" | "f32" | "f64" => {
                "number".to_string()
            }
            "Bool" | "bool" => "boolean".to_string(),
            other => to_pascal_case(other),
        },
        TypeExpr::Generic { name, args } => {
            let arg_strs: Vec<_> = args.iter().map(map_type_expr_typescript).collect();
            match name.as_str() {
                "Vec" | "List" => format!("{}[]", arg_strs[0]),
                "Option" => format!("{} | null", arg_strs[0]),
                "Result" => format!("Result<{}>", arg_strs.join(", ")),
                "Map" | "HashMap" => format!("Map<{}>", arg_strs.join(", ")),
                "Set" | "HashSet" => format!("Set<{}>", arg_strs[0]),
                other => format!("{}<{}>", to_pascal_case(other), arg_strs.join(", ")),
            }
        }
        TypeExpr::Function { params, return_type } => {
            let param_strs: Vec<_> = params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("p{}: {}", i, map_type_expr_typescript(t)))
                .collect();
            let ret_str = map_type_expr_typescript(return_type);
            format!("({}) => {}", param_strs.join(", "), ret_str)
        }
        TypeExpr::Tuple(types) => {
            let type_strs: Vec<_> = types.iter().map(map_type_expr_typescript).collect();
            format!("[{}]", type_strs.join(", "))
        }
        TypeExpr::Never => "never".to_string(),
        TypeExpr::Enum { .. } => "Enum".to_string(),
    }
}

/// Map DOL type to WIT type
fn map_type_expr_wit(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            "String" => "string".to_string(),
            "Int32" | "i32" => "s32".to_string(),
            "Int64" | "i64" => "s64".to_string(),
            "Float32" | "f32" => "f32".to_string(),
            "Float64" | "f64" => "f64".to_string(),
            "Bool" | "bool" => "bool".to_string(),
            other => to_kebab_case(other),
        },
        TypeExpr::Generic { name, args } => {
            let arg_strs: Vec<_> = args.iter().map(map_type_expr_wit).collect();
            match name.as_str() {
                "Vec" | "List" => format!("list<{}>", arg_strs[0]),
                "Option" => format!("option<{}>", arg_strs[0]),
                "Result" => format!("result<{}>", arg_strs.join(", ")),
                other => format!("{}<{}>", to_kebab_case(other), arg_strs.join(", ")),
            }
        }
        TypeExpr::Tuple(types) => {
            let type_strs: Vec<_> = types.iter().map(map_type_expr_wit).collect();
            format!("tuple<{}>", type_strs.join(", "))
        }
        _ => "unknown".to_string(),
    }
}

/// Map DOL type to Python type
fn map_type_expr_python(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            "String" => "str".to_string(),
            "Int32" | "Int64" | "i32" | "i64" => "int".to_string(),
            "Float32" | "Float64" | "f32" | "f64" => "float".to_string(),
            "Bool" | "bool" => "bool".to_string(),
            other => to_pascal_case(other),
        },
        TypeExpr::Generic { name, args } => {
            let arg_strs: Vec<_> = args.iter().map(map_type_expr_python).collect();
            match name.as_str() {
                "Vec" | "List" => format!("List[{}]", arg_strs[0]),
                "Option" => format!("Optional[{}]", arg_strs[0]),
                "Result" => format!("Result[{}]", arg_strs.join(", ")),
                "Map" | "HashMap" => format!("Dict[{}]", arg_strs.join(", ")),
                "Set" | "HashSet" => format!("Set[{}]", arg_strs[0]),
                other => format!("{}[{}]", to_pascal_case(other), arg_strs.join(", ")),
            }
        }
        TypeExpr::Tuple(types) => {
            let type_strs: Vec<_> = types.iter().map(map_type_expr_python).collect();
            format!("Tuple[{}]", type_strs.join(", "))
        }
        _ => "Any".to_string(),
    }
}

/// Map DOL type to JSON Schema type
fn map_type_expr_json_schema(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            "String" => "string".to_string(),
            "Int32" | "Int64" | "i32" | "i64" => "integer".to_string(),
            "Float32" | "Float64" | "f32" | "f64" => "number".to_string(),
            "Bool" | "bool" => "boolean".to_string(),
            other => other.to_string(),
        },
        TypeExpr::Generic { name, args } => match name.as_str() {
            "Vec" | "List" => "array".to_string(),
            _ => "object".to_string(),
        },
        _ => "object".to_string(),
    }
}

// Case conversion utilities
fn to_pascal_case(s: &str) -> String {
    s.to_pascal_case()
}

fn to_snake_case(s: &str) -> String {
    s.to_snake_case()
}

fn to_kebab_case(s: &str) -> String {
    s.to_kebab_case()
}

/// Main entry point for template-based code generation
pub fn generate(file: &DolFile, context: &CodegenContext) -> Result<String> {
    let engine = TemplateEngine::new(context.target);
    engine.generate(file, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_conversions() {
        assert_eq!(to_pascal_case("hello.world"), "HelloWorld");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
    }

    #[test]
    fn test_type_mapping_rust() {
        let type_expr = TypeExpr::Named("String".to_string());
        assert_eq!(map_type_expr(&type_expr, Target::Rust), "String");

        let vec_type = TypeExpr::Generic {
            name: "Vec".to_string(),
            args: vec![TypeExpr::Named("i32".to_string())],
        };
        assert_eq!(map_type_expr(&vec_type, Target::Rust), "Vec<i32>");
    }

    #[test]
    fn test_type_mapping_typescript() {
        let type_expr = TypeExpr::Named("String".to_string());
        assert_eq!(map_type_expr(&type_expr, Target::TypeScript), "string");

        let vec_type = TypeExpr::Generic {
            name: "Vec".to_string(),
            args: vec![TypeExpr::Named("i32".to_string())],
        };
        assert_eq!(map_type_expr(&vec_type, Target::TypeScript), "number[]");
    }
}
