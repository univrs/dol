//! WASM bindings generation for Automerge-backed structs
//!
//! Generates wasm-bindgen wrapper code that exposes Automerge-backed structs
//! to JavaScript/TypeScript with proper serialization and CRDT merge operations.

use crate::{type_mapper, CodegenError, CodegenOptions};
use dol::ast::{Gen, HasField, Statement, TypeExpr};
use quote::{format_ident, quote};
use syn::parse_str;

/// Generate WASM bindings for an Automerge-backed struct
pub fn generate_wasm_bindings(
    gen: &Gen,
    _options: &CodegenOptions,
) -> Result<String, CodegenError> {
    let struct_name = dol::codegen::to_pascal_case(&gen.name);
    let struct_ident = parse_str::<syn::Ident>(&struct_name)
        .map_err(|e| CodegenError::TypeMapping(e.to_string()))?;

    let wasm_name = format_ident!("{}WASM", struct_ident);

    // Extract fields
    let fields = extract_fields(gen);

    // Generate constructor
    let constructor = quote! {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                inner: #struct_ident::default(),
                doc: automerge::Automerge::new(),
            }
        }
    };

    // Generate getters and setters
    let accessors = generate_accessors(&struct_ident, &fields)?;

    // Generate merge method
    let merge_method = quote! {
        #[wasm_bindgen]
        pub fn merge(&mut self, other: &#wasm_name) -> Result<(), wasm_bindgen::JsValue> {
            self.inner
                .merge(&other.inner)
                .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
        }
    };

    // Generate save/load methods for Automerge binary format
    let persistence = quote! {
        #[wasm_bindgen]
        pub fn save(&self) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
            self.doc
                .save()
                .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
        }

        #[wasm_bindgen]
        pub fn load(bytes: &[u8]) -> Result<#wasm_name, wasm_bindgen::JsValue> {
            let doc = automerge::Automerge::load(bytes)
                .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
            let inner = #struct_ident::from_automerge(&doc)
                .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
            Ok(Self { inner, doc })
        }
    };

    // Generate to_json method for debugging
    let json_method = quote! {
        #[wasm_bindgen]
        pub fn to_json(&self) -> Result<String, wasm_bindgen::JsValue> {
            serde_json::to_string(&self.inner)
                .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
        }
    };

    // Combine into WASM wrapper struct
    let output = quote! {
        /// WASM wrapper for #struct_ident with JavaScript bindings
        #[wasm_bindgen]
        pub struct #wasm_name {
            inner: #struct_ident,
            doc: automerge::Automerge,
        }

        #[wasm_bindgen]
        impl #wasm_name {
            #constructor

            #accessors

            #merge_method

            #persistence

            #json_method
        }
    };

    Ok(output.to_string())
}

/// Generate getter and setter methods for all fields
fn generate_accessors(
    struct_ident: &syn::Ident,
    fields: &[&HasField],
) -> Result<proc_macro2::TokenStream, CodegenError> {
    let mut accessors = vec![];

    for field in fields {
        let field_name = dol::codegen::to_snake_case(&field.name);
        let field_ident = parse_str::<syn::Ident>(&field_name)
            .map_err(|e| CodegenError::TypeMapping(e.to_string()))?;

        let getter_name = format_ident!("get_{}", field_name);
        let setter_name = format_ident!("set_{}", field_name);

        // Generate getter based on type
        let getter = generate_getter(&getter_name, &field_ident, &field.type_)?;
        let setter = generate_setter(&setter_name, &field_ident, &field.type_)?;

        accessors.push(getter);
        accessors.push(setter);
    }

    Ok(quote! {
        #(#accessors)*
    })
}

/// Generate a getter method for a field
fn generate_getter(
    getter_name: &syn::Ident,
    field_ident: &syn::Ident,
    type_expr: &TypeExpr,
) -> Result<proc_macro2::TokenStream, CodegenError> {
    // For simple types, we can return directly or clone
    // For complex types, we may need to serialize to JsValue
    let is_copy = is_copy_type(type_expr);

    if is_copy {
        Ok(quote! {
            #[wasm_bindgen]
            pub fn #getter_name(&self) -> #field_ident {
                self.inner.#field_ident
            }
        })
    } else {
        // For non-Copy types, return as JsValue
        Ok(quote! {
            #[wasm_bindgen]
            pub fn #getter_name(&self) -> wasm_bindgen::JsValue {
                serde_wasm_bindgen::to_value(&self.inner.#field_ident)
                    .unwrap_or(wasm_bindgen::JsValue::NULL)
            }
        })
    }
}

/// Generate a setter method for a field
fn generate_setter(
    setter_name: &syn::Ident,
    field_ident: &syn::Ident,
    type_expr: &TypeExpr,
) -> Result<proc_macro2::TokenStream, CodegenError> {
    let rust_type = type_mapper::map_type_expr(type_expr);
    let is_copy = is_copy_type(type_expr);

    if is_copy {
        let type_token = parse_str::<syn::Type>(&rust_type)
            .map_err(|e| CodegenError::TypeMapping(e.to_string()))?;

        Ok(quote! {
            #[wasm_bindgen]
            pub fn #setter_name(&mut self, value: #type_token) {
                self.inner.#field_ident = value;
            }
        })
    } else {
        Ok(quote! {
            #[wasm_bindgen]
            pub fn #setter_name(&mut self, value: wasm_bindgen::JsValue) -> Result<(), wasm_bindgen::JsValue> {
                let parsed: #rust_type = serde_wasm_bindgen::from_value(value)
                    .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
                self.inner.#field_ident = parsed;
                Ok(())
            }
        })
    }
}

/// Check if a type is Copy (can be passed by value across WASM boundary)
fn is_copy_type(type_expr: &TypeExpr) -> bool {
    match type_expr {
        TypeExpr::Named(name) => matches!(
            name.as_str(),
            "bool" | "Bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
                | "f32" | "f64" | "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt"
                | "UInt8" | "UInt16" | "UInt32" | "UInt64" | "Float" | "Float32" | "Float64"
        ),
        _ => false,
    }
}

/// Extract HasField statements from a Gen
fn extract_fields(gen: &Gen) -> Vec<&HasField> {
    gen.statements
        .iter()
        .filter_map(|stmt| {
            if let Statement::HasField(field) = stmt {
                Some(field.as_ref())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dol::ast::{CrdtAnnotation, CrdtStrategy, Span, Visibility};

    #[test]
    fn test_is_copy_type() {
        assert!(is_copy_type(&TypeExpr::Named("bool".to_string())));
        assert!(is_copy_type(&TypeExpr::Named("i32".to_string())));
        assert!(is_copy_type(&TypeExpr::Named("f64".to_string())));
        assert!(!is_copy_type(&TypeExpr::Named("String".to_string())));
    }

    #[test]
    fn test_generate_wasm_bindings() {
        let gen = Gen {
            visibility: Visibility::default(),
            name: "counter".to_string(),
            extends: None,
            statements: vec![
                Statement::HasField(Box::new(HasField {
                    name: "value".to_string(),
                    type_: TypeExpr::Named("i64".to_string()),
                    default: None,
                    constraint: None,
                    crdt_annotation: Some(CrdtAnnotation {
                        strategy: CrdtStrategy::PnCounter,
                        options: vec![],
                        span: Span::default(),
                    }),
                    span: Span::default(),
                })),
            ],
            exegesis: "A counter".to_string(),
            span: Span::default(),
        };

        let options = CodegenOptions::default();
        let result = generate_wasm_bindings(&gen, &options).unwrap();

        assert!(result.contains("CounterWASM"));
        assert!(result.contains("wasm_bindgen"));
        assert!(result.contains("merge"));
        assert!(result.contains("save"));
        assert!(result.contains("load"));
    }
}
