//! Automerge backend code generation
//!
//! Generates Rust structs with Automerge CRDT backing based on `@crdt(...)` annotations.

use crate::{type_mapper, CodegenError, CodegenOptions};
use dol::ast::{CrdtStrategy, Gen, HasField, Statement};
use quote::quote;
use syn::parse_str;

/// Generate an Automerge-backed struct from a Gen declaration
pub fn generate_automerge_struct(
    gen: &Gen,
    options: &CodegenOptions,
) -> Result<String, CodegenError> {
    let struct_name = dol::codegen::to_pascal_case(&gen.name);
    let struct_ident = parse_str::<syn::Ident>(&struct_name)
        .map_err(|e| CodegenError::TypeMapping(e.to_string()))?;

    // Extract fields
    let fields = extract_fields(gen);

    // Generate field declarations with autosurgeon attributes
    let field_tokens: Vec<_> = fields
        .iter()
        .map(|field| generate_field_with_crdt(field))
        .collect::<Result<Vec<_>, _>>()?;

    // Generate derives
    let mut derives = vec![
        quote! { Debug },
        quote! { Clone },
        quote! { autosurgeon::Reconcile },
        quote! { autosurgeon::Hydrate },
    ];

    if options.derive_serde {
        derives.push(quote! { serde::Serialize });
        derives.push(quote! { serde::Deserialize });
    }

    // Generate struct
    let struct_def = quote! {
        #[derive(#(#derives),*)]
        pub struct #struct_ident {
            #(#field_tokens),*
        }
    };

    // Generate impl block with merge functionality
    let merge_impl = generate_merge_impl(&struct_ident, gen)?;

    // Combine into final output
    let output = quote! {
        #struct_def

        #merge_impl
    };

    Ok(output.to_string())
}

/// Generate a field declaration with CRDT attributes
fn generate_field_with_crdt(field: &HasField) -> Result<proc_macro2::TokenStream, CodegenError> {
    let field_name = dol::codegen::to_snake_case(&field.name);
    let field_ident =
        parse_str::<syn::Ident>(&field_name).map_err(|e| CodegenError::TypeMapping(e.to_string()))?;

    let field_type_str = type_mapper::map_type_expr(&field.type_);
    let field_type =
        parse_str::<syn::Type>(&field_type_str).map_err(|e| CodegenError::TypeMapping(e.to_string()))?;

    // Generate autosurgeon attribute based on CRDT strategy
    let crdt_attr = if let Some(crdt) = &field.crdt_annotation {
        generate_crdt_attribute(&crdt.strategy)?
    } else {
        quote! {}
    };

    Ok(quote! {
        #crdt_attr
        pub #field_ident: #field_type
    })
}

/// Generate autosurgeon attribute for a CRDT strategy
fn generate_crdt_attribute(
    strategy: &CrdtStrategy,
) -> Result<proc_macro2::TokenStream, CodegenError> {
    match strategy {
        CrdtStrategy::Immutable => Ok(quote! {
            #[autosurgeon(immutable)]
        }),
        CrdtStrategy::Lww => {
            // Last-write-wins is the default behavior
            Ok(quote! {})
        }
        CrdtStrategy::OrSet => Ok(quote! {
            #[autosurgeon(set)]
        }),
        CrdtStrategy::PnCounter => Ok(quote! {
            #[autosurgeon(counter)]
        }),
        CrdtStrategy::Peritext => Ok(quote! {
            #[autosurgeon(text)]
        }),
        CrdtStrategy::Rga => Ok(quote! {
            #[autosurgeon(list)]
        }),
        CrdtStrategy::MvRegister => {
            // Multi-value register requires custom handling
            // For now, we'll use default behavior
            Ok(quote! {})
        }
    }
}

/// Generate merge implementation for the struct
fn generate_merge_impl(
    struct_ident: &syn::Ident,
    gen: &Gen,
) -> Result<proc_macro2::TokenStream, CodegenError> {
    // For now, we'll generate a basic merge method that delegates to autosurgeon
    // In the future, this should enforce DOL constraints from Rule declarations

    let _constraint_checks = generate_constraint_checks(gen)?;

    Ok(quote! {
        impl #struct_ident {
            /// Merge changes from another instance using CRDT semantics
            ///
            /// This method performs automatic conflict resolution based on the
            /// CRDT strategies specified in the DOL `@crdt(...)` annotations.
            pub fn merge(&mut self, other: &Self) -> Result<(), autosurgeon::ReconcileError> {
                // Use autosurgeon to perform the merge
                autosurgeon::reconcile(self, other)
            }

            /// Create a new instance from an Automerge document
            pub fn from_automerge(doc: &automerge::Automerge) -> Result<Self, autosurgeon::HydrateError> {
                autosurgeon::hydrate(doc)
            }

            /// Convert this instance to an Automerge document
            pub fn to_automerge(&self) -> Result<automerge::Automerge, autosurgeon::ReconcileError> {
                let mut doc = automerge::Automerge::new();
                autosurgeon::reconcile(&mut doc, self)?;
                Ok(doc)
            }
        }
    })
}

/// Generate constraint checks from Rule declarations
///
/// This is a placeholder for future implementation. It should extract
/// constraint logic from DOL Rule declarations and generate validation code.
fn generate_constraint_checks(
    _gen: &Gen,
) -> Result<Vec<proc_macro2::TokenStream>, CodegenError> {
    // TODO: Extract constraint checks from associated Rule declarations
    // For now, return empty vec
    Ok(vec![])
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
    use dol::ast::{CrdtAnnotation, Span, TypeExpr, Visibility};

    #[test]
    fn test_generate_crdt_attribute() {
        let immutable_attr = generate_crdt_attribute(&CrdtStrategy::Immutable).unwrap();
        let expected = quote! {
            #[autosurgeon(immutable)]
        };
        assert_eq!(immutable_attr.to_string(), expected.to_string());

        let text_attr = generate_crdt_attribute(&CrdtStrategy::Peritext).unwrap();
        let expected = quote! {
            #[autosurgeon(text)]
        };
        assert_eq!(text_attr.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_field_with_crdt() {
        let field = HasField {
            name: "content".to_string(),
            type_: TypeExpr::Named("String".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::Peritext,
                options: vec![],
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let result = generate_field_with_crdt(&field).unwrap();
        let result_str = result.to_string();

        assert!(result_str.contains("content"));
        assert!(result_str.contains("String"));
        assert!(result_str.contains("#[autosurgeon(text)]"));
    }

    #[test]
    fn test_generate_automerge_struct() {
        let gen = Gen {
            visibility: Visibility::default(),
            name: "chat.message".to_string(),
            extends: None,
            statements: vec![
                Statement::HasField(Box::new(HasField {
                    name: "id".to_string(),
                    type_: TypeExpr::Named("String".to_string()),
                    default: None,
                    constraint: None,
                    crdt_annotation: Some(CrdtAnnotation {
                        strategy: CrdtStrategy::Immutable,
                        options: vec![],
                        span: Span::default(),
                    }),
                    span: Span::default(),
                })),
                Statement::HasField(Box::new(HasField {
                    name: "content".to_string(),
                    type_: TypeExpr::Named("String".to_string()),
                    default: None,
                    constraint: None,
                    crdt_annotation: Some(CrdtAnnotation {
                        strategy: CrdtStrategy::Peritext,
                        options: vec![],
                        span: Span::default(),
                    }),
                    span: Span::default(),
                })),
            ],
            exegesis: "A chat message".to_string(),
            span: Span::default(),
        };

        let options = CodegenOptions::default();
        let result = generate_automerge_struct(&gen, &options).unwrap();

        assert!(result.contains("ChatMessage"));
        assert!(result.contains("Reconcile"));
        assert!(result.contains("Hydrate"));
        assert!(result.contains("merge"));
    }
}
