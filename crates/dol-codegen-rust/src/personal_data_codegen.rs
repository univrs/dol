//! Code generation for @personal data fields with encryption.
//!
//! This module extends the DOL code generator to handle fields marked with
//! the @personal annotation, generating encryption/decryption methods for
//! GDPR-compliant personal data handling.

use dol::ast::{Gen, HasField, Statement};
use quote::quote;
use proc_macro2::TokenStream;

/// Check if a field has the @personal annotation.
pub fn has_personal_annotation(field: &HasField) -> bool {
    field.personal
}

/// Generate field declaration for personal data.
///
/// Personal fields are stored as `EncryptedField` instead of their declared type.
pub fn generate_personal_field_declaration(field: &HasField) -> TokenStream {
    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());

    quote! {
        /// Personal data field (encrypted, GDPR-compliant).
        #[autosurgeon(immutable)]
        #[serde(with = "encrypted_field_serde")]
        pub #field_name: vudo_privacy::EncryptedField
    }
}

/// Generate getter method for accessing personal data.
///
/// Generates a method that decrypts the field using the provided crypto manager.
pub fn generate_personal_getter(field: &HasField, gen_name: &str) -> TokenStream {
    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
    let getter_name = syn::Ident::new(
        &format!("get_{}", field.name),
        proc_macro2::Span::call_site(),
    );

    // Determine return type from field type
    let return_type = match &field.type_ {
        dol::ast::TypeExpr::Named(name) => {
            syn::Ident::new(name, proc_macro2::Span::call_site())
        }
        _ => syn::Ident::new("String", proc_macro2::Span::call_site()),
    };

    let doc = format!(
        "Access personal data field `{}` (requires decryption).\n\n\
         Returns the decrypted value if the DEK is available, otherwise returns an error.",
        field.name
    );

    quote! {
        #[doc = #doc]
        pub fn #getter_name(
            &self,
            crypto: &vudo_privacy::PersonalDataCrypto,
        ) -> vudo_privacy::error::Result<#return_type> {
            let dek = crypto.get_dek(&self.id)?;
            let plaintext = crypto.decrypt_field(&dek, &self.#field_name)?;
            let value = String::from_utf8(plaintext)?;
            Ok(value.parse().map_err(|e| {
                vudo_privacy::error::PrivacyError::Other(format!("Parse error: {}", e))
            })?)
        }
    }
}

/// Generate setter method for personal data.
///
/// Generates a method that encrypts the field using the provided crypto manager.
pub fn generate_personal_setter(field: &HasField, gen_name: &str) -> TokenStream {
    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
    let setter_name = syn::Ident::new(
        &format!("set_{}", field.name),
        proc_macro2::Span::call_site(),
    );

    // Determine parameter type from field type
    let param_type = match &field.type_ {
        dol::ast::TypeExpr::Named(name) => {
            syn::Ident::new(name, proc_macro2::Span::call_site())
        }
        _ => syn::Ident::new("String", proc_macro2::Span::call_site()),
    };

    let doc = format!(
        "Set personal data field `{}` (encrypts the value).\n\n\
         Encrypts the value with the user's DEK before storing.",
        field.name
    );

    quote! {
        #[doc = #doc]
        pub fn #setter_name(
            &mut self,
            value: #param_type,
            crypto: &vudo_privacy::PersonalDataCrypto,
        ) -> vudo_privacy::error::Result<()> {
            let dek = crypto.get_dek(&self.id)?;
            let plaintext = value.to_string().into_bytes();
            self.#field_name = crypto.encrypt_field(&dek, &plaintext)?;
            Ok(())
        }
    }
}

/// Generate GDPR deletion method for the gen.
///
/// Generates a method that deletes the user's DEK, making all personal data
/// permanently unrecoverable.
pub fn generate_gdpr_delete_method(gen: &Gen) -> TokenStream {
    let doc = format!(
        "Execute GDPR Article 17 deletion for this {}.\n\n\
         Deletes the user's DEK, making all personal data permanently unrecoverable.\n\
         This operation is irreversible.",
        gen.name
    );

    quote! {
        #[doc = #doc]
        pub async fn gdpr_delete(
            &self,
            engine: &vudo_privacy::GdprComplianceEngine,
        ) -> vudo_privacy::error::Result<vudo_privacy::DeletionReport> {
            let request = vudo_privacy::DeletionRequest::personal_only(
                "app.example".to_string()
            );
            engine.execute_deletion(&self.id, request).await
        }
    }
}

/// Check if a gen has any personal fields.
pub fn has_personal_fields(gen: &Gen) -> bool {
    gen.statements.iter().any(|stmt| match stmt {
        Statement::HasField(field) => field.personal,
        _ => false,
    })
}

/// Generate complete personal data implementation for a gen.
///
/// This includes:
/// - Field declarations (as EncryptedField)
/// - Getter methods (decrypt and return)
/// - Setter methods (encrypt and store)
/// - GDPR deletion method
pub fn generate_personal_data_impl(gen: &Gen) -> TokenStream {
    if !has_personal_fields(gen) {
        return quote! {};
    }

    let struct_name = syn::Ident::new(&gen.name.replace('.', "_"), proc_macro2::Span::call_site());

    // Generate getters and setters for personal fields
    let personal_methods: Vec<TokenStream> = gen
        .statements
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::HasField(field) if field.personal => {
                let getter = generate_personal_getter(field, &gen.name);
                let setter = generate_personal_setter(field, &gen.name);
                Some(quote! {
                    #getter
                    #setter
                })
            }
            _ => None,
        })
        .collect();

    let gdpr_delete = generate_gdpr_delete_method(gen);

    quote! {
        impl #struct_name {
            #(#personal_methods)*

            #gdpr_delete
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dol::ast::{CrdtAnnotation, CrdtStrategy, Gen, HasField, Span, Statement, TypeExpr, Visibility};

    fn make_personal_field(name: &str, type_: &str) -> HasField {
        HasField {
            name: name.to_string(),
            type_: TypeExpr::Named(type_.to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::Lww,
                options: vec![],
                span: Span::default(),
            }),
            personal: true,
            span: Span::default(),
        }
    }

    fn make_public_field(name: &str, type_: &str) -> HasField {
        HasField {
            name: name.to_string(),
            type_: TypeExpr::Named(type_.to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::Lww,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        }
    }

    #[test]
    fn test_has_personal_annotation() {
        let personal = make_personal_field("email", "String");
        let public = make_public_field("username", "String");

        assert!(has_personal_annotation(&personal));
        assert!(!has_personal_annotation(&public));
    }

    #[test]
    fn test_has_personal_fields() {
        let gen = Gen {
            visibility: Visibility::default(),
            name: "UserProfile".to_string(),
            extends: None,
            statements: vec![
                Statement::HasField(Box::new(make_personal_field("email", "String"))),
                Statement::HasField(Box::new(make_public_field("username", "String"))),
            ],
            exegesis: "Test".to_string(),
            span: Span::default(),
        };

        assert!(has_personal_fields(&gen));
    }

    #[test]
    fn test_generate_personal_field_declaration() {
        let field = make_personal_field("email", "String");
        let tokens = generate_personal_field_declaration(&field);
        let code = tokens.to_string();

        assert!(code.contains("email"));
        assert!(code.contains("EncryptedField"));
        assert!(code.contains("autosurgeon"));
    }

    #[test]
    fn test_generate_personal_getter() {
        let field = make_personal_field("email", "String");
        let tokens = generate_personal_getter(&field, "UserProfile");
        let code = tokens.to_string();

        assert!(code.contains("get_email"));
        assert!(code.contains("decrypt_field"));
        assert!(code.contains("get_dek"));
    }

    #[test]
    fn test_generate_personal_setter() {
        let field = make_personal_field("email", "String");
        let tokens = generate_personal_setter(&field, "UserProfile");
        let code = tokens.to_string();

        assert!(code.contains("set_email"));
        assert!(code.contains("encrypt_field"));
        assert!(code.contains("get_dek"));
    }

    #[test]
    fn test_generate_gdpr_delete_method() {
        let gen = Gen {
            visibility: Visibility::default(),
            name: "UserProfile".to_string(),
            extends: None,
            statements: vec![],
            exegesis: "Test".to_string(),
            span: Span::default(),
        };

        let tokens = generate_gdpr_delete_method(&gen);
        let code = tokens.to_string();

        assert!(code.contains("gdpr_delete"));
        assert!(code.contains("GdprComplianceEngine"));
        assert!(code.contains("execute_deletion"));
    }
}
