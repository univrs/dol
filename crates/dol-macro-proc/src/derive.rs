//! Derive macros for DOL.
//!
//! This module provides derive macro implementations for common traits
//! that can be automatically generated for DOL declarations.

use crate::error::{ProcMacroError, ProcMacroResult};
use metadol::ast::{Declaration, Gen, Span};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{DeriveInput, Data, Fields};

/// Trait that can be derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivableTrait {
    /// Debug trait for formatted output
    Debug,
    /// Clone trait for copying
    Clone,
    /// PartialEq trait for equality comparison
    PartialEq,
    /// Eq trait for total equality
    Eq,
    /// Hash trait for hashing
    Hash,
    /// Default trait for default values
    Default,
    /// Gen trait (DOL-specific)
    Gen,
}

impl DerivableTrait {
    /// Parses a trait name into a DerivableTrait.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Debug" => Some(Self::Debug),
            "Clone" => Some(Self::Clone),
            "PartialEq" => Some(Self::PartialEq),
            "Eq" => Some(Self::Eq),
            "Hash" => Some(Self::Hash),
            "Default" => Some(Self::Default),
            "Gen" => Some(Self::Gen),
            _ => None,
        }
    }

    /// Returns the trait name.
    pub fn name(&self) -> &str {
        match self {
            Self::Debug => "Debug",
            Self::Clone => "Clone",
            Self::PartialEq => "PartialEq",
            Self::Eq => "Eq",
            Self::Hash => "Hash",
            Self::Default => "Default",
            Self::Gen => "Gen",
        }
    }
}

/// Derives the Debug trait for a DOL declaration.
///
/// # Example
///
/// ```text
/// #[derive(Debug)]
/// gene container.exists {
///   container has identity
/// }
/// ```
pub fn derive_debug(input: &Gen) -> ProcMacroResult<TokenStream> {
    let name = &input.name;
    let name_ident = format_ident!("{}", name.replace('.', "_"));

    Ok(quote! {
        impl std::fmt::Debug for #name_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#name)
                    .finish()
            }
        }
    })
}

/// Derives the Clone trait for a DOL declaration.
///
/// # Example
///
/// ```text
/// #[derive(Clone)]
/// gene container.exists {
///   container has identity
/// }
/// ```
pub fn derive_clone(input: &Gen) -> ProcMacroResult<TokenStream> {
    let name = &input.name;
    let name_ident = format_ident!("{}", name.replace('.', "_"));

    Ok(quote! {
        impl Clone for #name_ident {
            fn clone(&self) -> Self {
                Self { ..*self }
            }
        }
    })
}

/// Derives the PartialEq trait for a DOL declaration.
///
/// # Example
///
/// ```text
/// #[derive(PartialEq)]
/// gene container.exists {
///   container has identity
/// }
/// ```
pub fn derive_partial_eq(input: &Gen) -> ProcMacroResult<TokenStream> {
    let name = &input.name;
    let name_ident = format_ident!("{}", name.replace('.', "_"));

    Ok(quote! {
        impl PartialEq for #name_ident {
            fn eq(&self, other: &Self) -> bool {
                // Field-by-field comparison would go here
                true
            }
        }
    })
}

/// Derives the Gen trait for a DOL declaration.
///
/// The Gen trait is DOL-specific and provides methods for
/// working with gene declarations.
///
/// # Example
///
/// ```text
/// #[derive(Gen)]
/// gene container.exists {
///   container has identity
/// }
/// ```
pub fn derive_gen_trait(input: &Gen) -> ProcMacroResult<TokenStream> {
    let name = &input.name;
    let name_ident = format_ident!("{}", name.replace('.', "_"));
    let exegesis = &input.exegesis;

    Ok(quote! {
        impl Gen for #name_ident {
            fn name(&self) -> &str {
                #name
            }

            fn exegesis(&self) -> &str {
                #exegesis
            }

            fn validate(&self) -> Result<(), String> {
                // Validation logic would go here
                Ok(())
            }
        }
    })
}

/// Derives multiple traits for a declaration.
///
/// # Arguments
///
/// * `traits` - List of trait names to derive
/// * `decl` - The declaration to derive traits for
pub fn derive_traits(traits: &[String], decl: &Declaration) -> ProcMacroResult<Vec<TokenStream>> {
    let mut implementations = Vec::new();

    // Extract the gen if this is a gene declaration
    let gen = match decl {
        Declaration::Gene(g) => g,
        _ => {
            return Err(ProcMacroError::invalid_input(
                "derive only works on gene declarations",
            ))
        }
    };

    for trait_name in traits {
        let derivable = DerivableTrait::from_str(trait_name).ok_or_else(|| {
            ProcMacroError::unsupported(&format!("trait '{}'", trait_name))
        })?;

        let impl_tokens = match derivable {
            DerivableTrait::Debug => derive_debug(gen)?,
            DerivableTrait::Clone => derive_clone(gen)?,
            DerivableTrait::PartialEq => derive_partial_eq(gen)?,
            DerivableTrait::Gen => derive_gen_trait(gen)?,
            _ => {
                return Err(ProcMacroError::unsupported(&format!(
                    "trait '{}'",
                    trait_name
                )))
            }
        };

        implementations.push(impl_tokens);
    }

    Ok(implementations)
}

/// Derives traits from a Rust-style derive input.
///
/// This is used when integrating with Rust's procedural macro system.
pub fn derive_from_rust_input(input: DeriveInput) -> ProcMacroResult<TokenStream> {
    let name = &input.ident;

    match input.data {
        Data::Struct(data) => {
            // Generate basic implementations for structs
            match data.fields {
                Fields::Named(_) => Ok(quote! {
                    impl std::fmt::Debug for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            f.debug_struct(stringify!(#name))
                                .finish()
                        }
                    }

                    impl Clone for #name {
                        fn clone(&self) -> Self {
                            Self { ..*self }
                        }
                    }
                }),
                _ => Err(ProcMacroError::unsupported("tuple structs")),
            }
        }
        Data::Enum(_) => Err(ProcMacroError::unsupported("enums")),
        Data::Union(_) => Err(ProcMacroError::unsupported("unions")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadol::ast::{Statement, Visibility};

    fn create_test_gen() -> Gen {
        Gen {
            visibility: Visibility::default(),
            name: "test.gene".to_string(),
            extends: None,
            statements: vec![],
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        }
    }

    #[test]
    fn test_derivable_trait_parsing() {
        assert_eq!(
            DerivableTrait::from_str("Debug"),
            Some(DerivableTrait::Debug)
        );
        assert_eq!(
            DerivableTrait::from_str("Clone"),
            Some(DerivableTrait::Clone)
        );
        assert_eq!(DerivableTrait::from_str("Invalid"), None);
    }

    #[test]
    fn test_trait_names() {
        assert_eq!(DerivableTrait::Debug.name(), "Debug");
        assert_eq!(DerivableTrait::Clone.name(), "Clone");
        assert_eq!(DerivableTrait::Gen.name(), "Gen");
    }

    #[test]
    fn test_derive_debug() {
        let gen = create_test_gen();
        let result = derive_debug(&gen);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("impl"));
        assert!(code.contains("Debug"));
    }

    #[test]
    fn test_derive_clone() {
        let gen = create_test_gen();
        let result = derive_clone(&gen);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("impl"));
        assert!(code.contains("Clone"));
    }

    #[test]
    fn test_derive_gen_trait() {
        let gen = create_test_gen();
        let result = derive_gen_trait(&gen);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("impl"));
        assert!(code.contains("Gen"));
        assert!(code.contains("name"));
        assert!(code.contains("exegesis"));
    }

    #[test]
    fn test_derive_multiple_traits() {
        let gen = create_test_gen();
        let decl = Declaration::Gene(gen);
        let traits = vec!["Debug".to_string(), "Clone".to_string()];

        let result = derive_traits(&traits, &decl);
        assert!(result.is_ok());

        let impls = result.unwrap();
        assert_eq!(impls.len(), 2);
    }

    #[test]
    fn test_derive_unsupported_trait() {
        let gen = create_test_gen();
        let decl = Declaration::Gene(gen);
        let traits = vec!["Unsupported".to_string()];

        let result = derive_traits(&traits, &decl);
        assert!(result.is_err());
    }
}
