//! MCP tool implementations for Metal DOL.
//!
//! This module contains helper functions and utilities for MCP tools,
//! particularly for CRDT-related analysis and recommendations.

use crate::ast::{CrdtStrategy, TypeExpr};

/// Formats a CRDT strategy for display.
pub fn format_strategy(strategy: &CrdtStrategy) -> &'static str {
    match strategy {
        CrdtStrategy::Immutable => "immutable",
        CrdtStrategy::Lww => "lww",
        CrdtStrategy::OrSet => "or_set",
        CrdtStrategy::PnCounter => "pn_counter",
        CrdtStrategy::Peritext => "peritext",
        CrdtStrategy::Rga => "rga",
        CrdtStrategy::MvRegister => "mv_register",
    }
}

/// Formats a type expression for display.
pub fn format_type(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Generic { name, args } => {
            let args_str = args.iter().map(format_type).collect::<Vec<_>>().join(", ");
            format!("{}<{}>", name, args_str)
        }
        TypeExpr::Function {
            params,
            return_type,
        } => {
            let params_str = params
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({}) -> {}", params_str, format_type(return_type))
        }
        TypeExpr::Tuple(types) => {
            let types_str = types.iter().map(format_type).collect::<Vec<_>>().join(", ");
            format!("({})", types_str)
        }
        TypeExpr::Never => "!".to_string(),
        TypeExpr::Enum { .. } => "enum { ... }".to_string(),
    }
}

/// Checks if a type is a collection type.
pub fn is_collection_type(type_expr: &TypeExpr) -> bool {
    match type_expr {
        TypeExpr::Generic { name, .. } => {
            matches!(name.as_str(), "Set" | "Vec" | "List" | "Map")
        }
        _ => false,
    }
}

/// Checks if a type is a numeric type.
pub fn is_numeric_type(type_expr: &TypeExpr) -> bool {
    match type_expr {
        TypeExpr::Named(name) => matches!(
            name.as_str(),
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "f32"
                | "f64"
                | "Int"
                | "int"
                | "Float"
                | "float"
        ),
        _ => false,
    }
}

/// Checks if a type is a string type.
pub fn is_string_type(type_expr: &TypeExpr) -> bool {
    match type_expr {
        TypeExpr::Named(name) => matches!(name.as_str(), "String" | "string"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_strategy() {
        assert_eq!(format_strategy(&CrdtStrategy::Immutable), "immutable");
        assert_eq!(format_strategy(&CrdtStrategy::Lww), "lww");
        assert_eq!(format_strategy(&CrdtStrategy::Peritext), "peritext");
    }

    #[test]
    fn test_format_type_simple() {
        let type_expr = TypeExpr::Named("String".to_string());
        assert_eq!(format_type(&type_expr), "String");
    }

    #[test]
    fn test_format_type_generic() {
        let type_expr = TypeExpr::Generic {
            name: "Set".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        assert_eq!(format_type(&type_expr), "Set<String>");
    }

    #[test]
    fn test_is_collection_type() {
        let set_type = TypeExpr::Generic {
            name: "Set".to_string(),
            args: vec![],
        };
        assert!(is_collection_type(&set_type));

        let string_type = TypeExpr::Named("String".to_string());
        assert!(!is_collection_type(&string_type));
    }

    #[test]
    fn test_is_numeric_type() {
        assert!(is_numeric_type(&TypeExpr::Named("i32".to_string())));
        assert!(is_numeric_type(&TypeExpr::Named("f64".to_string())));
        assert!(!is_numeric_type(&TypeExpr::Named("String".to_string())));
    }

    #[test]
    fn test_is_string_type() {
        assert!(is_string_type(&TypeExpr::Named("String".to_string())));
        assert!(is_string_type(&TypeExpr::Named("string".to_string())));
        assert!(!is_string_type(&TypeExpr::Named("i32".to_string())));
    }
}
